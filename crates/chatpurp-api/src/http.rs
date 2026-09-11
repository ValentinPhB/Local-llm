use crate::{
    json,
    outgoing::{Chat, MODEL},
    sessions::{COOKIE, SessionManager, cookie_token},
};
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::State,
    http::{Request, Response, StatusCode, header},
};
use chatpurp_core::application::{Application, Failure};
use hyper_util::{
    rt::{TokioIo, TokioTimer},
    service::TowerToHyperService,
};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::Semaphore, time::timeout};

pub struct StateData {
    pub host: String,
    pub app: Arc<Application>,
    pub sessions: Arc<SessionManager>,
    pub chat: Arc<dyn Chat>,
    pub assets: std::collections::BTreeMap<String, crate::assets::Asset>,
}
pub fn router(state: Arc<StateData>) -> Router {
    Router::new().fallback(dispatch).with_state(state)
}
pub async fn serve(listener: TcpListener, state: Arc<StateData>) -> std::io::Result<()> {
    let router = router(state);
    let gate = Arc::new(Semaphore::new(16));
    loop {
        let (stream, _) = listener.accept().await?;
        let Ok(permit) = gate.clone().try_acquire_owned() else {
            continue;
        };
        let service = router.clone();
        tokio::spawn(async move {
            let _permit = permit;
            connection(stream, service).await;
        });
    }
}
pub(crate) async fn connection<T>(stream: T, router: Router)
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let mut builder = hyper::server::conn::http1::Builder::new();
    // Une requête par connexion : cadrage mixte et corps rejeté ne peuvent
    // jamais être suivis d'une seconde exécution sur ce même socket.
    builder
        .keep_alive(false)
        .timer(TokioTimer::new())
        .header_read_timeout(Duration::from_secs(5))
        .max_buf_size(16 * 1024);
    let _ = builder
        .serve_connection(TokioIo::new(stream), TowerToHyperService::new(router))
        .await;
}
pub fn response(status: u16, value: Value) -> Response<Body> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-store")
        .header("x-content-type-options", "nosniff")
        .body(Body::from(
            serde_json::to_vec(&value).expect("JSON Value serializable"),
        ))
        .unwrap()
}
fn error(status: u16, message: &str) -> Response<Body> {
    response(status, json!({"error":message}))
}

#[cfg(test)]
mod tests;
fn failure(f: Failure) -> Response<Body> {
    let (status, message) = match f {
        Failure::Session => (401, "Session de démonstration requise."),
        Failure::Resource => (400, "Ressource de démonstration invalide."),
        Failure::Forbidden => (403, "Accès au document refusé."),
        Failure::Audit => (503, "Journal de sécurité indisponible."),
        Failure::Document => (500, "Document indisponible."),
        Failure::Query => (400, "Requête de recherche invalide."),
        Failure::Message => (400, "Message invalide."),
    };
    error(status, message)
}
fn one_header<'a>(headers: &'a axum::http::HeaderMap, name: &str) -> Result<Option<&'a str>, ()> {
    let mut values = headers.get_all(name).iter();
    let first = values.next();
    if values.next().is_some() {
        return Err(());
    }
    first.map(|h| h.to_str().map_err(|_| ())).transpose()
}
fn json_content_type(value: &str) -> bool {
    let parts: Vec<_> = value.split(';').map(str::trim).collect();
    parts[0].eq_ignore_ascii_case("application/json")
        && (parts.len() == 1
            || (parts.len() == 2 && parts[1].eq_ignore_ascii_case("charset=utf-8")))
}
async fn dispatch(State(state): State<Arc<StateData>>, request: Request<Body>) -> Response<Body> {
    let head = request.method() == "HEAD";
    let mut result = handle(state, request).await;
    if head {
        *result.body_mut() = Body::empty();
    }
    result
}
async fn handle(state: Arc<StateData>, request: Request<Body>) -> Response<Body> {
    let (parts, body) = request.into_parts();
    if one_header(&parts.headers, "host")
        .ok()
        .flatten()
        .map(str::trim)
        != Some(state.host.as_str())
    {
        return error(403, "Requête HTTP refusée.");
    }
    if parts.uri.scheme().is_some() || parts.uri.authority().is_some() {
        return error(400, "Requête HTTP invalide.");
    }
    let path = parts.uri.path();
    let method = parts.method.as_str();
    if let Some(asset) = state.assets.get(path) {
        if method != "GET" && method != "HEAD" {
            let mut r = error(405, "Méthode non autorisée.");
            r.headers_mut()
                .insert(header::ALLOW, "GET, HEAD".parse().unwrap());
            return r;
        }
        if parts.uri.query().is_some() {
            return error(404, "Route introuvable.");
        }
        if !matches!(
            timeout(Duration::from_secs(5), to_bytes(body, 0)).await,
            Ok(Ok(_))
        ) {
            return error(400, "Requête HTTP invalide.");
        }
        return Response::builder()
            .status(200)
            .header(header::CONTENT_TYPE, asset.mime)
            .header(header::CONTENT_LENGTH, asset.bytes.len())
            .header(header::CACHE_CONTROL, "no-store")
            .header("x-content-type-options", "nosniff")
            .header("content-security-policy", crate::assets::CSP)
            .body(if method == "HEAD" {
                Body::empty()
            } else {
                Body::from(asset.bytes.clone())
            })
            .unwrap();
    }
    if path.starts_with("/api/") {
        let origin = format!("http://{}", state.host);
        match one_header(&parts.headers, "origin") {
            Ok(Some(value)) if value == origin => (),
            Ok(None) if method != "POST" => (),
            _ => return error(403, "Requête HTTP refusée."),
        }
    }
    let expected = match path {
        "/healthz" | "/api/session" | "/api/access-check" => "GET",
        "/api/demo-session" | "/api/logout" | "/api/retrieve" | "/api/chat" | "/api/rag-chat" => {
            "POST"
        }
        p if p.starts_with("/api/documents/") => "GET",
        _ => return error(404, "Route introuvable."),
    };
    if method != expected {
        let mut r = error(405, "Méthode non autorisée.");
        r.headers_mut()
            .insert(header::ALLOW, expected.parse().unwrap());
        return r;
    }
    if parts.headers.contains_key(header::CONTENT_ENCODING) {
        return error(415, "Format de requête non pris en charge.");
    }
    let limit = match path {
        "/api/demo-session" => 256,
        "/api/retrieve" => 1024,
        "/api/chat" | "/api/rag-chat" => 16000,
        _ => 0,
    };
    if limit > 0
        && !one_header(&parts.headers, "content-type")
            .ok()
            .flatten()
            .is_some_and(json_content_type)
    {
        return error(415, "Format de requête non pris en charge.");
    }
    let invalid = || {
        error(
            400,
            if path == "/api/demo-session" {
                "Identité de démonstration invalide."
            } else {
                "Requête HTTP invalide."
            },
        )
    };
    let bytes = match timeout(Duration::from_secs(5), to_bytes(body, limit)).await {
        Ok(Ok(b)) => b,
        _ => return invalid(),
    };
    let input = if limit > 0 {
        match json::parse(&bytes) {
            Ok(v) if v.is_object() => v,
            _ => return invalid(),
        }
    } else {
        Value::Null
    };
    if path == "/healthz" {
        return response(200, json!({"status":"ok","model":MODEL}));
    }
    if path == "/api/logout" {
        let mut r = response(200, json!({"authenticated":false}));
        r.headers_mut().insert(
            header::SET_COOKIE,
            format!("{COOKIE}=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0")
                .parse()
                .unwrap(),
        );
        return r;
    }
    if path == "/api/demo-session" {
        let Some(id) = input
            .get("identity_id")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
        else {
            return invalid();
        };
        let Some(identity) = state.sessions.directory.identity(id) else {
            return error(401, "Identité de démonstration refusée.");
        };
        let token = match state.sessions.issue(id) {
            Ok(t) => t,
            Err(_) => return error(401, "Identité de démonstration refusée."),
        };
        let mut r = response(
            200,
            json!({"authenticated":true,"identity":{"id":identity.id,"display_name":identity.display_name}}),
        );
        r.headers_mut().insert(
            header::SET_COOKIE,
            format!("{COOKIE}={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age=900")
                .parse()
                .unwrap(),
        );
        return r;
    }
    let cookie_headers = parts
        .headers
        .get_all(header::COOKIE)
        .iter()
        .map(|h| h.to_str())
        .collect::<Result<Vec<_>, _>>();
    let token = cookie_headers
        .ok()
        .and_then(|v| cookie_token(v.into_iter()).ok().flatten());
    let app = state.app.clone();
    let route = path.to_owned();
    // Aucun travail disque bloquant sur le thread de transport.
    if path == "/api/chat" || path == "/api/rag-chat" {
        let message = input
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let rag = path == "/api/rag-chat";
        let prepared = match tokio::task::spawn_blocking(move || {
            app.prepare_chat(token.as_deref(), &message, rag)
        })
        .await
        {
            Ok(Ok(p)) => p,
            Ok(Err(e)) => return failure(e),
            Err(_) => return failure(Failure::Document),
        };
        let sources = prepared.sources.clone();
        return match state.chat.complete(prepared).await {
            Ok(content) => response(
                200,
                if rag {
                    json!({"content":content,"sources":sources})
                } else {
                    json!({"content":content})
                },
            ),
            Err(_) => error(502, "Le modèle local est indisponible."),
        };
    }
    let query = parts.uri.query().unwrap_or("").to_owned();
    let result=tokio::task::spawn_blocking(move || -> Result<Value,Failure> {
        let t=token.as_deref();
        if route=="/api/session" { let verified=app.session(t)?;let id=state.sessions.directory.identity(verified.id()).ok_or(Failure::Session)?;return Ok(json!({"authenticated":true,"identity":{"id":id.id,"display_name":id.display_name}})); }
        if let Some(id)=route.strip_prefix("/api/documents/") { let d=app.read(t,id)?;return Ok(json!({"resource_id":d.resource_id,"classification":d.classification,"content":d.content})); }
        if route=="/api/access-check" { let values:Vec<_>=query.split('&').filter_map(|p|p.strip_prefix("resource_id=")).collect();let id=if values.len()==1 {values[0]}else{""};app.access(t,id)?;return Ok(json!({"resource_id":id,"allowed":true})); }
        let query=input.get("query").and_then(Value::as_str).unwrap_or(""); let results=app.retrieve(t,query)?;
        Ok(json!({"results":results.into_iter().map(|p|json!({"resource_id":p.resource_id,"classification":p.classification,"excerpt":p.excerpt})).collect::<Vec<_>>()}))
    }).await;
    match result {
        Ok(Ok(v)) => response(200, v),
        Ok(Err(f)) => failure(f),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "Opération indisponible.",
        ),
    }
}

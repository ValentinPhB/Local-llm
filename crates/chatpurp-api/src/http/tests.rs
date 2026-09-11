use super::*;
use crate::{outgoing::ChatFuture, sessions::SystemClock, storage::FileReader};
use chatpurp_core::{
    application::{Audit, AuditDecision, PreparedChat},
    policy::{Directory, Policy},
};
use std::{
    path::Path,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Default)]
struct Spy {
    audit: Mutex<Vec<AuditDecision>>,
    chats: Mutex<Vec<PreparedChat>>,
    fail: AtomicBool,
}
impl Audit for Spy {
    fn record(&self, d: AuditDecision) -> Result<(), Failure> {
        self.audit.lock().unwrap().push(d);
        if self.fail.load(Ordering::SeqCst) {
            Err(Failure::Audit)
        } else {
            Ok(())
        }
    }
}
impl Chat for Spy {
    fn complete(&self, p: PreparedChat) -> ChatFuture<'_> {
        self.chats.lock().unwrap().push(p);
        Box::pin(async { Ok("Réponse fictive <script>window.bad=true</script>".into()) })
    }
}
fn setup() -> (Arc<StateData>, Arc<Spy>) {
    let directory: Directory =
        serde_json::from_str(include_str!("../../../../config/demo-idp/directory.json")).unwrap();
    let policy: Policy = serde_json::from_str(include_str!(
        "../../../../config/access-control/demo-policy.json"
    ))
    .unwrap();
    policy.validate(&directory).unwrap();
    let sessions =
        Arc::new(SessionManager::new(Arc::new(directory), Arc::new(SystemClock)).unwrap());
    let spy = Arc::new(Spy::default());
    let app = Application {
        policy: Arc::new(policy),
        sessions: sessions.clone(),
        reader: Arc::new(
            FileReader::new(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap(),
        ),
        audit: spy.clone(),
    };
    (
        Arc::new(StateData {
            host: "127.0.0.1:3211".into(),
            app: Arc::new(app),
            sessions,
            chat: spy.clone(),
            assets: Default::default(),
        }),
        spy,
    )
}
async fn wire(state: Arc<StateData>, raw: &str) -> String {
    let (mut client, server) = tokio::io::duplex(131072);
    let task = tokio::spawn(connection(server, router(state)));
    client.write_all(raw.as_bytes()).await.unwrap();
    let mut response = Vec::new();
    timeout(Duration::from_secs(8), client.read_to_end(&mut response))
        .await
        .unwrap()
        .unwrap();
    task.await.unwrap();
    String::from_utf8(response).unwrap()
}
fn request(method: &str, path: &str, cookie: &str, body: &str) -> String {
    format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:3211\r\nOrigin: http://127.0.0.1:3211\r\nContent-Type: application/json\r\n{}Content-Length: {}\r\n\r\n{body}",
        if cookie.is_empty() {
            String::new()
        } else {
            format!("Cookie: {cookie}\r\n")
        },
        body.len()
    )
}
fn cookie(state: &StateData, id: &str) -> String {
    format!("{COOKIE}={}", state.sessions.issue(id).unwrap())
}
fn status(response: &str) -> u16 {
    response.split_whitespace().nth(1).unwrap().parse().unwrap()
}
#[tokio::test]
async fn complete_four_identity_document_matrix_over_real_http_parser() {
    let (state, _) = setup();
    for id in ["alice", "bob", "charlie", "oscar"] {
        let token = cookie(&state, id);
        for r in &state.app.policy.resources {
            let allowed = match id {
                "alice" => r.classification != "IT",
                "bob" => r.classification != "RH",
                "charlie" => r.classification == "PUBLIC",
                _ => r.id == "public-welcome",
            };
            let res = wire(
                state.clone(),
                &request("GET", &format!("/api/documents/{}", r.id), &token, ""),
            )
            .await;
            assert_eq!(
                status(&res),
                if allowed { 200 } else { 403 },
                "{id}/{}: {res}",
                r.id
            );
            if !allowed {
                assert!(!res.contains("classification"));
                assert!(!res.contains("content\""));
            }
            assert!(!res.to_lowercase().contains("access-control-allow"));
        }
    }
}
#[tokio::test]
async fn transport_rejects_before_audit_session_or_cookie_issue() {
    let (state, spy) = setup();
    let good = request(
        "POST",
        "/api/demo-session",
        "",
        r#"{"identity_id":"oscar"}"#,
    );
    let cases = [
        (
            good.replace("Host: 127.0.0.1:3211", "Host: localhost:3211"),
            403,
        ),
        (good.replace("Origin: http://127.0.0.1:3211\r\n", ""), 403),
        (
            good.replace("Origin: http://127.0.0.1:3211", "Origin: null"),
            403,
        ),
        (
            good.replace("Content-Type: application/json", "Content-Type: text/plain"),
            415,
        ),
        (
            good.replace(
                "Content-Type: application/json",
                "Content-Type: application/json\r\nContent-Encoding: gzip",
            ),
            415,
        ),
        (
            good.replace(
                "Host: 127.0.0.1:3211",
                "Host: 127.0.0.1:3211\r\nHost: 127.0.0.1:3211",
            ),
            403,
        ),
        (
            good.replace(
                "Origin: http://127.0.0.1:3211",
                "Origin: http://127.0.0.1:3211\r\nOrigin: http://127.0.0.1:3211",
            ),
            403,
        ),
    ];
    for (raw, expected) in cases {
        let res = wire(state.clone(), &raw).await;
        assert_eq!(status(&res), expected, "{res}");
        assert!(!res.contains("set-cookie"));
    }
    assert!(spy.audit.lock().unwrap().is_empty());
}
#[tokio::test]
async fn raw_paths_preserve_session_before_invalid_resource() {
    let (state, spy) = setup();
    for id in [
        "",
        "../it-security",
        "%70ublic-welcome",
        "public/welcome",
        "public.welcome",
        "public\\welcome",
    ] {
        let path = format!("/api/documents/{id}");
        assert_eq!(
            status(&wire(state.clone(), &request("GET", &path, "", "")).await),
            401
        );
        assert_eq!(
            status(
                &wire(
                    state.clone(),
                    &request("GET", &path, &cookie(&state, "oscar"), "")
                )
                .await
            ),
            400
        );
    }
    assert!(
        spy.audit
            .lock()
            .unwrap()
            .iter()
            .filter(|d| d.identity_id.is_none())
            .all(|d| d.resource_id.is_none())
    );
}
#[tokio::test]
async fn duplicate_json_cookie_and_body_limits() {
    let (state, _) = setup();
    for body in [
        r#"{"identity_id":"oscar","identity_id":"alice"}"#,
        r#"{"identity_id":"oscar","extra":{"roles":1,"roles":2}}"#,
        r#"[]"#,
    ] {
        assert_eq!(
            status(
                &wire(
                    state.clone(),
                    &request("POST", "/api/demo-session", "", body)
                )
                .await
            ),
            400
        );
    }
    let base = r#"{"identity_id":"oscar"}"#;
    for size in [256, 257] {
        let b = format!("{base}{}", " ".repeat(size - base.len()));
        let r = wire(state.clone(), &request("POST", "/api/demo-session", "", &b)).await;
        assert_eq!(status(&r), if size == 256 { 200 } else { 400 });
    }
    let c = cookie(&state, "oscar");
    let r = wire(
        state.clone(),
        &request("GET", "/api/session", &format!("{c}; {c}"), ""),
    )
    .await;
    assert_eq!(status(&r), 401);
    assert_eq!(
        status(&wire(state.clone(), &request("GET", "/api/session", &c, "x")).await),
        400
    );
    let r = wire(state.clone(), &request("HEAD", "/api/session", &c, "")).await;
    assert_eq!(status(&r), 405);
    assert!(r.to_lowercase().contains("allow: get"));
    assert!(r.ends_with("\r\n\r\n"));
}
#[tokio::test]
async fn session_creation_ignores_client_roles_and_preserves_unknown_identity_contract() {
    let (state, _) = setup();
    for id in ["Oscar", " oscar ", "unknown", "é"] {
        let body = serde_json::to_string(&json!({"identity_id":id})).unwrap();
        let r = wire(
            state.clone(),
            &request("POST", "/api/demo-session", "", &body),
        )
        .await;
        assert_eq!(status(&r), 401);
        assert!(r.contains("Identité de démonstration refusée."));
        assert!(!r.contains("set-cookie:"));
    }
    let r = wire(
        state.clone(),
        &request(
            "POST",
            "/api/demo-session",
            "",
            r#"{"identity_id":"oscar","roles":["rh_reader"],"groups":["HR"]}"#,
        ),
    )
    .await;
    assert_eq!(status(&r), 200);
    let cookie = r
        .lines()
        .find_map(|l| l.strip_prefix("set-cookie: "))
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    assert_eq!(
        status(
            &wire(
                state,
                &request("GET", "/api/documents/rh-onboarding", cookie, "")
            )
            .await
        ),
        403
    );
}
#[tokio::test]
async fn framing_normalization_closes_without_second_application_call() {
    let (state, spy) = setup();
    let token = cookie(&state, "oscar");
    let first = request("GET", "/api/documents/public-welcome", &token, "");
    let second = request("GET", "/api/documents/public-welcome", &token, "");
    let raw = first.replace(
        "Content-Length: 0",
        "Content-Length: 0\r\nContent-Length: 0",
    ) + &second;
    assert_eq!(status(&wire(state.clone(), &raw).await), 200);
    assert_eq!(spy.audit.lock().unwrap().len(), 1);
    spy.audit.lock().unwrap().clear();
    let raw = first.replace(
        "Content-Length: 0",
        "Content-Length: 0\r\nContent-Length: 1",
    ) + &second;
    let response = wire(state.clone(), &raw).await;
    assert!(response.is_empty() || status(&response) == 400);
    assert!(spy.audit.lock().unwrap().is_empty());
    for mixed in [
        "Content-Length: 123\r\nTransfer-Encoding: chunked",
        "Transfer-Encoding: chunked\r\nContent-Length: 123",
    ] {
        for id in ["public-welcome", "rh-onboarding"] {
            let raw = request("GET", &format!("/api/documents/{id}"), &token, "")
                .replace("Content-Length: 0", mixed)
                + "0\r\n\r\n"
                + &second;
            let r = wire(state.clone(), &raw).await;
            assert_eq!(
                status(&r),
                if id == "public-welcome" { 200 } else { 403 },
                "{r}"
            );
            assert_eq!(spy.audit.lock().unwrap().len(), 1);
            spy.audit.lock().unwrap().clear();
        }
        for size in [256, 257] {
            let base = r#"{"identity_id":"oscar"}"#;
            let body = format!("{base}{}", " ".repeat(size - base.len()));
            let raw = request("POST", "/api/demo-session", "", "")
                .replace("Content-Length: 0", mixed)
                + &format!("{size:x}\r\n{body}\r\n0\r\n\r\n")
                + &second;
            let r = wire(state.clone(), &raw).await;
            assert_eq!(status(&r), if size == 256 { 200 } else { 400 }, "{r}");
            assert!(spy.audit.lock().unwrap().is_empty());
            assert_eq!(r.matches("HTTP/1.1 ").count(), 1);
        }
    }
}
#[tokio::test(start_paused = true)]
async fn slow_headers_and_body_have_total_deadline() {
    let (state, spy) = setup();
    for raw in [
        "GET /api/session HTTP/1.1\r\nHost: 127.0.0.1:3211",
        "POST /api/demo-session HTTP/1.1\r\nHost: 127.0.0.1:3211\r\nOrigin: http://127.0.0.1:3211\r\nContent-Type: application/json\r\nContent-Length: 20\r\n\r\n{",
    ] {
        let start = tokio::time::Instant::now();
        let _ = wire(state.clone(), raw).await;
        assert!(start.elapsed() >= Duration::from_secs(5));
        assert!(start.elapsed() < Duration::from_secs(6));
    }
    assert!(spy.audit.lock().unwrap().is_empty());
}
#[tokio::test]
async fn oversized_headers_never_reach_domain() {
    let (state, spy) = setup();
    let raw = request(
        "GET",
        "/api/documents/public-welcome",
        &cookie(&state, "oscar"),
        "",
    )
    .replace(
        "Content-Length: 0",
        &format!("X-Fill: {}\r\nContent-Length: 0", "x".repeat(17000)),
    );
    let r = wire(state, &raw).await;
    assert!(r.is_empty() || status(&r) >= 400);
    assert!(spy.audit.lock().unwrap().is_empty());
}
#[tokio::test]
async fn chat_and_rag_never_call_model_before_successful_audit() {
    let (state, spy) = setup();
    let body = r#"{"message":"Bienvenue sécurité","model":"remote","context":"RH SECRET","sources":["rh-onboarding"]}"#;
    for route in ["/api/chat", "/api/rag-chat"] {
        assert_eq!(
            status(&wire(state.clone(), &request("POST", route, "", body)).await),
            401
        );
        spy.fail.store(true, Ordering::SeqCst);
        assert_eq!(
            status(
                &wire(
                    state.clone(),
                    &request("POST", route, &cookie(&state, "oscar"), body)
                )
                .await
            ),
            503
        );
        spy.fail.store(false, Ordering::SeqCst);
    }
    assert!(spy.chats.lock().unwrap().is_empty());
    let r = wire(
        state.clone(),
        &request("POST", "/api/rag-chat", &cookie(&state, "oscar"), body),
    )
    .await;
    assert_eq!(status(&r), 200);
    let chats = spy.chats.lock().unwrap();
    assert_eq!(chats.len(), 1);
    assert!(chats[0].sources.iter().all(|s| s == "public-welcome"));
    assert!(!chats[0].context.as_ref().unwrap().contains("RH SECRET"));
}

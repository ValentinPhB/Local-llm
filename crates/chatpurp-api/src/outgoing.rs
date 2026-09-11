//! HTTP sortant explicite : socket loopback fixe, ni proxy, ni redirection.
use axum::{
    body::{Body, to_bytes},
    http::Request,
};
use chatpurp_core::application::PreparedChat;
use hyper_util::rt::TokioIo;
use serde_json::{Value, json};
use std::{
    future::Future,
    net::{Ipv4Addr, SocketAddr},
    pin::Pin,
    time::Duration,
};
use tokio::{net::TcpStream, sync::Semaphore, time::timeout};

pub const MODEL: &str = "qwen3:4b";
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UpstreamError;
pub type ChatFuture<'a> = Pin<Box<dyn Future<Output = Result<String, UpstreamError>> + Send + 'a>>;
pub trait Chat: Send + Sync {
    fn complete(&self, prepared: PreparedChat) -> ChatFuture<'_>;
}
pub struct Ollama {
    gate: Semaphore,
}
impl Default for Ollama {
    fn default() -> Self {
        Self {
            gate: Semaphore::new(1),
        }
    }
}
impl Chat for Ollama {
    fn complete(&self, prepared: PreparedChat) -> ChatFuture<'_> {
        Box::pin(async move {
            // Pas de file illimitée de prompts en mémoire ; un seul appel simultané.
            let _permit = self.gate.try_acquire().map_err(|_| UpstreamError)?;
            let mut messages = Vec::new();
            if let Some(context) = prepared.context {
                messages.push(json!({"role":"system","content":context}));
            }
            messages.push(json!({"role":"user","content":prepared.message}));
            let result = request_json(
                11434,
                "POST",
                "/api/chat",
                Some(json!({"model":MODEL,"messages":messages,"stream":false,"think":false})),
                Duration::from_secs(120),
                1_048_576,
            )
            .await?;
            chat_content(&result)
        })
    }
}
pub fn chat_content(value: &Value) -> Result<String, UpstreamError> {
    let raw = value
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(Value::as_str)
        .ok_or(UpstreamError)?;
    // ASCII case-fold préserve les indices UTF-8, même après un préfixe Unicode.
    let folded = raw.to_ascii_lowercase();
    let clean = if let Some(i) = folded.find("</think>") {
        &raw[i + 8..]
    } else {
        raw
    };
    if clean.trim().is_empty() {
        return Err(UpstreamError);
    }
    Ok(clean.trim().into())
}
pub async fn request_json(
    port: u16,
    method: &str,
    path: &str,
    value: Option<Value>,
    deadline: Duration,
    max: usize,
) -> Result<Value, UpstreamError> {
    timeout(deadline, async {
        let socket = TcpStream::connect(SocketAddr::from((Ipv4Addr::LOCALHOST, port)))
            .await
            .map_err(|_| UpstreamError)?;
        exchange(socket, port, method, path, value, max).await
    })
    .await
    .map_err(|_| UpstreamError)?
}
async fn exchange<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static>(
    socket: T,
    port: u16,
    method: &str,
    path: &str,
    value: Option<Value>,
    max: usize,
) -> Result<Value, UpstreamError> {
    let (mut sender, connection) = hyper::client::conn::http1::handshake(TokioIo::new(socket))
        .await
        .map_err(|_| UpstreamError)?;
    let driver = tokio::spawn(async move {
        let _ = connection.await;
    });
    // Annulation de la requête (délai inclus) ferme aussi son pilote/socket.
    struct Abort(tokio::task::JoinHandle<()>);
    impl Drop for Abort {
        fn drop(&mut self) {
            self.0.abort();
        }
    }
    let _driver = Abort(driver);
    let body = match value {
        Some(v) => Body::from(serde_json::to_vec(&v).map_err(|_| UpstreamError)?),
        None => Body::empty(),
    };
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("host", format!("127.0.0.1:{port}"))
        .header("content-type", "application/json")
        .header("connection", "close")
        .body(body)
        .map_err(|_| UpstreamError)?;
    let response = sender
        .send_request(request)
        .await
        .map_err(|_| UpstreamError)?;
    if response.status() == 404
        && method == "DELETE"
        && path == "/collections/lab_semantic_documents"
    {
        // Seule l'absence de la collection fixe lors de sa suppression est admise.
        return Ok(json!({"status":"ok","already_absent":true}));
    }
    if !response.status().is_success() || response.headers().contains_key("content-encoding") {
        return Err(UpstreamError);
    }
    let bytes = to_bytes(Body::new(response.into_body()), max)
        .await
        .map_err(|_| UpstreamError)?;
    crate::json::parse(&bytes).map_err(|_| UpstreamError)
}
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    async fn fixture(response: String, max: usize) -> Result<Value, UpstreamError> {
        let (client, mut server) = tokio::io::duplex(131072);
        let peer = tokio::spawn(async move {
            let mut input = [0; 2048];
            let n = server.read(&mut input).await.unwrap();
            assert!(
                std::str::from_utf8(&input[..n])
                    .unwrap()
                    .starts_with("POST /api/chat HTTP/1.1")
            );
            let _ = server.write_all(response.as_bytes()).await;
        });
        let result = timeout(
            Duration::from_secs(1),
            exchange(
                client,
                11434,
                "POST",
                "/api/chat",
                Some(json!({"model":MODEL})),
                max,
            ),
        )
        .await
        .unwrap();
        peer.await.unwrap();
        result
    }
    #[tokio::test]
    async fn real_http_client_bounds_rejects_redirects_compression_and_invalid_json() {
        let r = "{\"ok\":true}";
        assert_eq!(
            fixture(
                format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{r}", r.len()),
                100
            )
            .await
            .unwrap(),
            json!({"ok":true})
        );
        for raw in [
            "HTTP/1.1 302 Found\r\nLocation: https://example.invalid/secret\r\nContent-Length: 0\r\n\r\n",
            "HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Length: 2\r\n\r\n{}",
            "HTTP/1.1 200 OK\r\nContent-Length: 6\r\n\r\nnot js",
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n6\r\n123456\r\n0\r\n\r\n",
        ] {
            assert!(fixture(raw.into(), 4).await.is_err());
        }
    }
    #[tokio::test(start_paused = true)]
    async fn timeout_cancels_upstream_driver_and_closes_stream() {
        let (client, mut server) = tokio::io::duplex(4096);
        let operation = timeout(
            Duration::from_secs(5),
            exchange(client, 11434, "POST", "/api/chat", None, 100),
        );
        assert!(operation.await.is_err());
        let mut rest = Vec::new();
        assert!(
            timeout(Duration::from_secs(1), server.read_to_end(&mut rest))
                .await
                .is_ok()
        );
    }
    #[test]
    fn thinking_prefix_is_removed_with_unicode_and_mixed_case() {
        for s in [
            "raisonnement é</THINK> réponse",
            "<think>secret</think> réponse",
            "réponse",
        ] {
            assert_eq!(
                chat_content(&json!({"message":{"content":s}})).unwrap(),
                "réponse"
            );
        }
        for v in [
            json!({}),
            json!({"message":{"content":1}}),
            json!({"message":{"content":"secret</think> "}}),
        ] {
            assert!(chat_content(&v).is_err());
        }
    }
}

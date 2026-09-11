//! Adaptateurs préparés, non raccordés aux routes utilisateur.
use crate::outgoing::{UpstreamError, request_json};
use chatpurp_core::{
    application::Passage,
    indexing::{Chunk, IndexedPassage, validate_batch, validate_vector},
    policy::Resource,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, future::Future, pin::Pin, time::Duration};
pub const COLLECTION: &str = "lab_semantic_documents";
pub type JsonFuture<'a> = Pin<Box<dyn Future<Output = Result<Value, UpstreamError>> + Send + 'a>>;
pub trait Transport: Send + Sync {
    fn send<'a>(
        &'a self,
        port: u16,
        method: &'a str,
        path: String,
        body: Option<Value>,
        deadline: Duration,
        max: usize,
    ) -> JsonFuture<'a>;
}
pub struct LocalTransport;
impl Transport for LocalTransport {
    fn send<'a>(
        &'a self,
        port: u16,
        method: &'a str,
        path: String,
        body: Option<Value>,
        deadline: Duration,
        max: usize,
    ) -> JsonFuture<'a> {
        Box::pin(async move { request_json(port, method, &path, body, deadline, max).await })
    }
}
pub struct Semantic<T: Transport> {
    pub transport: T,
}
fn vector(value: &Value) -> Result<Vec<f64>, UpstreamError> {
    let v = value
        .as_array()
        .ok_or(UpstreamError)?
        .iter()
        .map(|n| n.as_f64().ok_or(UpstreamError))
        .collect::<Result<Vec<_>, _>>()?;
    validate_vector(&v).map_err(|_| UpstreamError)?;
    Ok(v)
}
impl<T: Transport> Semantic<T> {
    pub async fn embed(&self, text: &str) -> Result<Vec<f64>, UpstreamError> {
        let text = text.trim();
        if text.is_empty() || text.chars().count() > 8000 {
            return Err(UpstreamError);
        }
        let result = self
            .transport
            .send(
                11434,
                "POST",
                "/api/embed".into(),
                Some(json!({"model":"embeddinggemma","input":text})),
                Duration::from_secs(30),
                262144,
            )
            .await?;
        let embeddings = result["embeddings"]
            .as_array()
            .filter(|e| e.len() == 1)
            .ok_or(UpstreamError)?;
        vector(&embeddings[0])
    }
    /// allowed provient obligatoirement du Policy::authorized après session/audit.
    /// Le payload retourné est revérifié contre ces ressources, classification comprise.
    pub async fn search(
        &self,
        query: &[f64],
        allowed: &[&Resource],
    ) -> Result<Vec<Passage>, UpstreamError> {
        if allowed.is_empty() {
            return Ok(vec![]);
        }
        validate_vector(query).map_err(|_| UpstreamError)?;
        let ids: BTreeSet<_> = allowed.iter().map(|r| r.id.as_str()).collect();
        if ids.iter().any(|id| !chatpurp_core::policy::valid_id(id)) {
            return Err(UpstreamError);
        }
        let result=self.transport.send(6333,"POST",format!("/collections/{COLLECTION}/points/query"),Some(json!({"query":query,"filter":{"must":[{"key":"resource_id","match":{"any":ids}}]},"limit":3,"with_payload":true,"with_vector":false})),Duration::from_secs(5),262144).await?;
        let points = result["result"]["points"]
            .as_array()
            .filter(|p| p.len() <= 3)
            .ok_or(UpstreamError)?;
        points
            .iter()
            .map(|p| {
                let payload = &p["payload"];
                let id = payload["resource_id"].as_str().ok_or(UpstreamError)?;
                let r = allowed.iter().find(|r| r.id == id).ok_or(UpstreamError)?;
                if payload["classification"].as_str() != Some(&r.classification)
                    || !p["score"].as_f64().is_some_and(f64::is_finite)
                {
                    return Err(UpstreamError);
                }
                Ok(Passage {
                    resource_id: id.into(),
                    classification: r.classification.clone(),
                    excerpt: payload["text"]
                        .as_str()
                        .ok_or(UpstreamError)?
                        .chars()
                        .take(500)
                        .collect(),
                })
            })
            .collect()
    }
    pub async fn embed_batch(
        &self,
        chunks: Vec<Chunk>,
    ) -> Result<Vec<IndexedPassage>, UpstreamError> {
        if chunks.is_empty() || chunks.len() > 512 {
            return Err(UpstreamError);
        }
        let mut batch = Vec::new();
        for chunk in chunks {
            let vector = self.embed(&chunk.text).await?;
            batch.push(IndexedPassage { chunk, vector });
        }
        validate_batch(&batch).map_err(|_| UpstreamError)?;
        Ok(batch)
    }
    /// Opération destructive administrative, jamais appelée depuis HTTP.
    /// La validation complète précède la première requête d'écriture.
    pub async fn replace_lab_index(&self, batch: &[IndexedPassage]) -> Result<(), UpstreamError> {
        let dimensions = validate_batch(batch).map_err(|_| UpstreamError)?;
        let points:Vec<_>=batch.iter().map(|p|json!({"id":point_id(&p.chunk),"vector":p.vector,"payload":{"resource_id":p.chunk.resource_id,"classification":p.chunk.classification,"chunk_id":p.chunk.chunk_id,"text":p.chunk.text}})).collect();
        for (method, suffix, body) in [
            ("DELETE", "", None),
            (
                "PUT",
                "",
                Some(json!({"vectors":{"size":dimensions,"distance":"Cosine"}})),
            ),
            ("PUT", "/points?wait=true", Some(json!({"points":points}))),
        ] {
            let result = self
                .transport
                .send(
                    6333,
                    method,
                    format!("/collections/{COLLECTION}{suffix}"),
                    body,
                    Duration::from_secs(5),
                    262144,
                )
                .await?;
            if result["status"] != "ok" {
                return Err(UpstreamError);
            }
        }
        Ok(())
    }
}
/// UUID v5 compatible avec l'indexeur initial ; SHA-1 sert d'identifiant, pas de signature.
fn point_id(chunk: &Chunk) -> String {
    let namespace = [
        0xc7, 0x7c, 0x3f, 0xc0, 0xba, 0xa8, 0x4b, 0x0d, 0x9d, 0x7d, 0x9f, 0x30, 0xe4, 0xfc, 0xad,
        0x2d,
    ];
    let mut input = namespace.to_vec();
    input.extend_from_slice(
        format!("{}:{}:{}", chunk.resource_id, chunk.chunk_id, chunk.text).as_bytes(),
    );
    let digest = aws_lc_rs::digest::digest(&aws_lc_rs::digest::SHA1_FOR_LEGACY_USE_ONLY, &input);
    let mut id = digest.as_ref()[..16].to_vec();
    id[6] = (id[6] & 0x0f) | 0x50;
    id[8] = (id[8] & 0x3f) | 0x80;
    let hex: String = id.iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    type Call = (u16, String, String, Option<Value>);
    struct Fake {
        calls: Mutex<Vec<Call>>,
        responses: Mutex<Vec<Value>>,
    }
    impl Transport for Fake {
        fn send<'a>(
            &'a self,
            port: u16,
            method: &'a str,
            path: String,
            body: Option<Value>,
            _: Duration,
            _: usize,
        ) -> JsonFuture<'a> {
            self.calls
                .lock()
                .unwrap()
                .push((port, method.into(), path, body));
            let result = self.responses.lock().unwrap().pop().ok_or(UpstreamError);
            Box::pin(async move { result })
        }
    }
    fn fake(values: Vec<Value>) -> Semantic<Fake> {
        Semantic {
            transport: Fake {
                calls: Mutex::new(vec![]),
                responses: Mutex::new(values.into_iter().rev().collect()),
            },
        }
    }
    fn resource() -> Resource {
        Resource {
            id: "public-welcome".into(),
            classification: "PUBLIC".into(),
            path: String::new(),
            allowed_roles: vec![],
        }
    }
    #[tokio::test]
    async fn embeddings_validate_input_dimensions_and_response() {
        let s = fake(vec![json!({"embeddings":[[1.,2.]]})]);
        assert_eq!(s.embed("Bonjour").await.unwrap(), [1., 2.]);
        let calls = s.transport.calls.lock().unwrap().clone();
        assert_eq!(calls[0].0, 11434);
        assert_eq!(calls[0].3.as_ref().unwrap()["model"], "embeddinggemma");
        drop(calls);
        for value in [
            json!({"embeddings":[[]]}),
            json!({"embeddings":[[true]]}),
            json!({"embeddings":[[1],[2]]}),
        ] {
            assert!(fake(vec![value]).embed("texte").await.is_err());
        }
        let s = fake(vec![]);
        assert!(s.embed(" ").await.is_err());
        assert!(s.transport.calls.lock().unwrap().is_empty());
    }
    #[tokio::test]
    async fn mandatory_acl_filter_and_malicious_payload_rejection() {
        let point = json!({"score":0.9,"payload":{"resource_id":"public-welcome","classification":"PUBLIC","text":"fictif"}});
        let r = resource();
        let s = fake(vec![json!({"result":{"points":[point.clone()]}})]);
        assert_eq!(s.search(&[1., 2.], &[&r]).await.unwrap().len(), 1);
        let calls = s.transport.calls.lock().unwrap().clone();
        assert_eq!(
            calls[0].3.as_ref().unwrap()["filter"]["must"][0]["match"]["any"],
            json!(["public-welcome"])
        );
        drop(calls);
        for (field, value) in [("resource_id", "rh-onboarding"), ("classification", "RH")] {
            let mut p = point.clone();
            p["payload"][field] = json!(value);
            assert!(
                fake(vec![json!({"result":{"points":[p]}})])
                    .search(&[1.], &[&r])
                    .await
                    .is_err()
            );
        }
        let s = fake(vec![]);
        assert!(s.search(&[], &[]).await.unwrap().is_empty());
        assert!(s.transport.calls.lock().unwrap().is_empty());
    }
    #[tokio::test]
    async fn invalid_batch_never_touches_qdrant_and_valid_batch_has_fixed_target() {
        let s = fake(vec![json!({"status":"ok"}); 3]);
        assert!(s.replace_lab_index(&[]).await.is_err());
        assert!(s.transport.calls.lock().unwrap().is_empty());
        let p = IndexedPassage {
            chunk: Chunk {
                resource_id: "public-welcome".into(),
                classification: "PUBLIC".into(),
                chunk_id: 0,
                text: "Fictif".into(),
            },
            vector: vec![1., 2.],
        };
        s.replace_lab_index(&[p]).await.unwrap();
        let calls = s.transport.calls.lock().unwrap();
        assert_eq!(
            calls.iter().map(|c| c.1.as_str()).collect::<Vec<_>>(),
            ["DELETE", "PUT", "PUT"]
        );
        assert!(
            calls
                .iter()
                .all(|c| c.0 == 6333 && c.2.starts_with("/collections/lab_semantic_documents"))
        );
    }
}

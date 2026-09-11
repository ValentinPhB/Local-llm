//! Vérification automatisée réservée à un Qdrant éphémère, jamais au port du lab.
use chatpurp_api::{
    outgoing::request_json,
    semantic::{JsonFuture, Semantic, Transport},
};
use chatpurp_core::{
    indexing::{Chunk, IndexedPassage},
    policy::Resource,
};
use serde_json::Value;
use std::time::Duration;
struct Fixture(u16);
impl Transport for Fixture {
    fn send<'a>(
        &'a self,
        port: u16,
        method: &'a str,
        path: String,
        body: Option<Value>,
        deadline: Duration,
        max: usize,
    ) -> JsonFuture<'a> {
        assert_eq!(port, 6333);
        Box::pin(async move { request_json(self.0, method, &path, body, deadline, max).await })
    }
}
#[tokio::main(flavor = "current_thread")]
async fn main() {
    if run().await.is_err() {
        eprintln!("Échec du contrat Qdrant éphémère.");
        std::process::exit(1);
    }
}
async fn run() -> Result<(), ()> {
    let port: u16 = std::env::args().nth(1).ok_or(())?.parse().map_err(|_| ())?;
    if port < 1024 || [3210, 3211, 6333, 11434].contains(&port) {
        return Err(());
    }
    let semantic = Semantic {
        transport: Fixture(port),
    };
    let batch: Vec<_> = [
        ("public-welcome", "PUBLIC", vec![1., 0.]),
        ("rh-onboarding", "RH", vec![0.9, 0.1]),
        ("it-workstation", "IT", vec![0.8, 0.2]),
    ]
    .into_iter()
    .map(|(id, c, vector)| IndexedPassage {
        chunk: Chunk {
            resource_id: id.into(),
            classification: c.into(),
            chunk_id: 0,
            text: format!("Passage de test fictif {id}"),
        },
        vector,
    })
    .collect();
    semantic.replace_lab_index(&batch).await.map_err(|_| ())?;
    let resource = Resource {
        id: "public-welcome".into(),
        classification: "PUBLIC".into(),
        path: String::new(),
        allowed_roles: vec![],
    };
    let matches = semantic
        .search(&[1., 0.], &[&resource])
        .await
        .map_err(|_| ())?;
    if matches.len() != 1 || matches[0].resource_id != "public-welcome" {
        return Err(());
    }
    // Un remplacement répété doit être valide et ne pas dupliquer les points.
    semantic.replace_lab_index(&batch).await.map_err(|_| ())?;
    let result = semantic.search(&[1., 0.], &[]).await.map_err(|_| ())?;
    if !result.is_empty() {
        return Err(());
    }
    println!("Qdrant éphémère : création, remplacement et filtre ACL validés.");
    Ok(())
}

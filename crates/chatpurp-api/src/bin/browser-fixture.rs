//! Exécutable de test uniquement : port éphémère, aucun appel Ollama/Qdrant.
use chatpurp_api::{
    assets::load_assets,
    http::{StateData, serve},
    outgoing::{Chat, ChatFuture},
    sessions::{SessionManager, SystemClock},
    storage::{FileAudit, FileReader},
};
use chatpurp_core::{
    application::{Application, PreparedChat},
    policy::{Directory, Policy},
};
use std::{io::Read, path::PathBuf, sync::Arc};
struct FakeChat;
impl Chat for FakeChat {
    fn complete(&self, _: PreparedChat) -> ChatFuture<'_> {
        Box::pin(async { Ok("Réponse fictive <script>window.bad=true</script>".into()) })
    }
}
#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Fixture refusée : {e}");
        std::process::exit(1);
    }
}
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let assets = PathBuf::from(std::env::args_os().nth(1).ok_or("assets requis")?);
    if !assets
        .canonicalize()?
        .starts_with(root.join(".local").canonicalize()?)
    {
        return Err("artefacts hors .local".into());
    }
    let policy: Policy = serde_json::from_str(include_str!(
        "../../../../config/access-control/demo-policy.json"
    ))?;
    let directory: Directory =
        serde_json::from_str(include_str!("../../../../config/demo-idp/directory.json"))?;
    policy.validate(&directory).map_err(|_| "politique")?;
    let sessions = Arc::new(
        SessionManager::new(Arc::new(directory), Arc::new(SystemClock)).map_err(|_| "session")?,
    );
    let mut random = [0u8; 16];
    aws_lc_rs::rand::fill(&mut random).map_err(|_| "random")?;
    let suffix: String = random.iter().map(|b| format!("{b:02x}")).collect();
    let temp = std::env::temp_dir().join(format!("chatpurp-browser-{suffix}"));
    std::fs::create_dir(&temp)?;
    struct Cleanup(PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(temp.clone());
    let app = Application {
        policy: Arc::new(policy),
        sessions: sessions.clone(),
        reader: Arc::new(FileReader::new(&root).map_err(|_| "reader")?),
        audit: Arc::new(FileAudit::new(&temp, Arc::new(SystemClock)).map_err(|_| "audit")?),
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let state = StateData {
        host: format!("127.0.0.1:{port}"),
        app: Arc::new(app),
        sessions,
        chat: Arc::new(FakeChat),
        assets: load_assets(&assets)?,
    };
    let paths: Vec<_> = state.assets.keys().cloned().collect();
    println!(
        "{}",
        serde_json::json!({"origin":format!("http://127.0.0.1:{port}"),"paths":paths})
    );
    let (done, recv) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = std::io::stdin().read_to_end(&mut bytes);
        let _ = done.send(());
    });
    tokio::select! { result=serve(listener,Arc::new(state))=>{result?;},_=recv=>{},_=tokio::time::sleep(std::time::Duration::from_secs(90))=>return Err("deadline fixture".into()) }
    Ok(())
}

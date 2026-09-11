use crate::{
    assets::load_assets,
    http::StateData,
    outgoing::Ollama,
    sessions::{SessionManager, SystemClock},
    storage::{FileAudit, FileReader},
};
use chatpurp_core::{
    application::Application,
    policy::{Directory, Policy},
};
use std::{fs::File, io::Read, path::Path, sync::Arc};

fn configuration<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, &'static str> {
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|_| "Configuration indisponible.")?
        .take(65537)
        .read_to_end(&mut bytes)
        .map_err(|_| "Configuration illisible.")?;
    if bytes.len() > 65536 {
        return Err("Configuration trop volumineuse.");
    }
    let value = crate::json::parse(&bytes).map_err(|_| "Configuration JSON invalide.")?;
    if value
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        != Some("1.0")
    {
        return Err("Version de configuration invalide.");
    }
    serde_json::from_value(value).map_err(|_| "Configuration invalide.")
}
pub fn load_policy(root: &Path) -> Result<(Policy, Directory), &'static str> {
    let directory: Directory = configuration(&root.join("config/demo-idp/directory.json"))?;
    let policy: Policy = configuration(&root.join("config/access-control/demo-policy.json"))?;
    policy
        .validate(&directory)
        .map_err(|_| "Politique incohérente.")?;
    Ok((policy, directory))
}
pub fn assemble(root: &Path, assets: &Path, port: u16) -> Result<StateData, &'static str> {
    if port == 0 || port == 3210 || port == 11434 || port == 6333 {
        return Err("Port réservé ou invalide.");
    }
    let directory: Directory = configuration(&root.join("config/demo-idp/directory.json"))?;
    let policy: Policy = configuration(&root.join("config/access-control/demo-policy.json"))?;
    policy
        .validate(&directory)
        .map_err(|_| "Politique incohérente.")?;
    let sessions = Arc::new(
        SessionManager::new(Arc::new(directory), Arc::new(SystemClock))
            .map_err(|_| "Sessions indisponibles.")?,
    );
    let app = Application {
        policy: Arc::new(policy),
        sessions: sessions.clone(),
        reader: Arc::new(FileReader::new(root).map_err(|_| "Documents indisponibles.")?),
        audit: Arc::new(
            FileAudit::new(root, Arc::new(SystemClock)).map_err(|_| "Audit indisponible.")?,
        ),
    };
    Ok(StateData {
        host: format!("127.0.0.1:{port}"),
        app: Arc::new(app),
        sessions,
        chat: Arc::new(Ollama::default()),
        assets: load_assets(assets).map_err(|_| "Interface compilée indisponible.")?,
    })
}

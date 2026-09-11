use chatpurp_contracts::{ChatResponse, DocumentResponse, SearchResponse, SessionResponse};
use dioxus::prelude::*;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

fn main() {
    dioxus::launch(App);
}

/// Types partagés pour l'affichage, jamais une autorité d'accès côté client.
async fn api<T: DeserializeOwned>(
    method: &str,
    path: &str,
    input: Option<Value>,
) -> Result<T, String> {
    let origin = web_sys::window()
        .ok_or("Navigateur indisponible.")?
        .location()
        .origin()
        .map_err(|_| "Origine indisponible.")?;
    let mut request = reqwest::Client::new().request(
        method.parse().map_err(|_| "Méthode invalide.")?,
        format!("{origin}{path}"),
    );
    if let Some(value) = input {
        request = request.json(&value);
    }
    let response = request
        .send()
        .await
        .map_err(|_| "API locale indisponible.")?;
    let status = response.status();
    let value = response
        .json::<Value>()
        .await
        .map_err(|_| "Réponse locale invalide.")?;
    if !status.is_success() {
        return Err(value
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("Opération refusée.")
            .into());
    }
    serde_json::from_value(value).map_err(|_| "Réponse locale invalide.".into())
}

#[component]
fn App() -> Element {
    let mut identity = use_signal(String::new);
    let mut selected = use_signal(|| "oscar".to_owned());
    let mut resource = use_signal(|| "public-welcome".to_owned());
    let mut document = use_signal(String::new);
    let mut query = use_signal(String::new);
    let mut results = use_signal(String::new);
    let mut message = use_signal(String::new);
    let mut answer = use_signal(String::new);
    let mut feedback = use_signal(String::new);
    let mut busy = use_signal(|| true);

    // Attendre cette réponse évite une course avec le choix d'une nouvelle identité.
    use_effect(move || {
        spawn(async move {
            if let Ok(session) = api::<SessionResponse>("GET", "/api/session", None).await {
                identity.set(session.identity.display_name);
            }
            busy.set(false);
        });
    });

    let login = move |_: MouseEvent| {
        busy.set(true);
        feedback.set(String::new());
        identity.set(String::new());
        document.set(String::new());
        results.set(String::new());
        answer.set(String::new());
        query.set(String::new());
        message.set(String::new());
        spawn(async move {
            match api::<SessionResponse>(
                "POST",
                "/api/demo-session",
                Some(json!({"identity_id":selected()})),
            )
            .await
            {
                Ok(session) => identity.set(session.identity.display_name),
                Err(error) => feedback.set(error),
            }
            busy.set(false);
        });
    };
    let logout = move |_: MouseEvent| {
        busy.set(true);
        spawn(async move {
            match api::<Value>("POST", "/api/logout", None).await {
                Ok(_) => {
                    identity.set(String::new());
                    document.set(String::new());
                    results.set(String::new());
                    answer.set(String::new());
                    query.set(String::new());
                    message.set(String::new());
                    feedback.set("Session fermée.".into());
                }
                Err(error) => feedback.set(error),
            }
            busy.set(false);
        });
    };
    let read = move |_: MouseEvent| {
        busy.set(true);
        document.set(String::new());
        feedback.set(String::new());
        spawn(async move {
            let id = resource();
            // Validation ergonomique seulement : l'API revérifie toujours.
            if id.is_empty()
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            {
                feedback.set("Identifiant de document invalide.".into());
            } else {
                match api::<DocumentResponse>("GET", &format!("/api/documents/{id}"), None).await {
                    Ok(result) => document.set(result.content),
                    Err(error) => feedback.set(error),
                }
            }
            busy.set(false);
        });
    };
    let retrieve = move |_: MouseEvent| {
        busy.set(true);
        results.set(String::new());
        feedback.set(String::new());
        spawn(async move {
            match api::<SearchResponse>("POST", "/api/retrieve", Some(json!({"query":query()})))
                .await
            {
                Ok(result) => {
                    let text = result
                        .results
                        .iter()
                        .map(|p| format!("{} ({})\n{}", p.resource_id, p.classification, p.excerpt))
                        .collect::<Vec<_>>()
                        .join("\n\n");
                    results.set(if text.is_empty() {
                        "Aucun extrait autorisé trouvé.".into()
                    } else {
                        text
                    });
                }
                Err(error) => feedback.set(error),
            }
            busy.set(false);
        });
    };
    let mut send_chat = move |route: &'static str| {
        busy.set(true);
        answer.set("Réponse locale en cours…".into());
        feedback.set(String::new());
        spawn(async move {
            match api::<ChatResponse>("POST", route, Some(json!({"message":message()}))).await {
                Ok(result) => {
                    let mut text = result.content;
                    if let Some(sources) = result.sources {
                        text.push_str("\n\nSources : ");
                        text.push_str(&sources.join(", "));
                    }
                    answer.set(text);
                }
                Err(error) => {
                    answer.set(String::new());
                    feedback.set(error);
                }
            }
            busy.set(false);
        });
    };

    rsx! {
        h1 { "ChatPurp" }
        p { class: "muted", "Lab local · interface et API Rust · Ollama qwen3:4b" }
        p { "Les identités sont fictives et librement sélectionnables : ceci ne prouve pas l’identité d’une personne réelle." }
        p { id: "feedback", role: "status", "{feedback}" }
        section {
            h2 { "1. Session de démonstration" }
            p { id: "identity", if identity().is_empty() { "Aucune session" } else { "{identity}" } }
            label { r#for: "identity-select", "Identité fictive" }
            select { id: "identity-select", value: "{selected}", disabled: busy(),
                onchange: move |e| selected.set(e.value()),
                option { value: "alice", "Alice — PUBLIC et RH" }
                option { value: "bob", "Bob — PUBLIC et IT" }
                option { value: "charlie", "Charlie — PUBLIC" }
                option { value: "oscar", "Oscar — public-welcome seulement" }
            }
            button { id: "login", disabled: busy(), onclick: login, "Ouvrir la session" }
            button { id: "logout", disabled: busy(), onclick: logout, "Fermer la session" }
        }
        section {
            h2 { "2. Lecture contrôlée" }
            label { r#for: "resource", "Identifiant du document (pas un chemin)" }
            input { id: "resource", value: "{resource}", maxlength: 80, disabled: busy(),
                oninput: move |e| resource.set(e.value()) }
            button { id: "read", disabled: busy(), onclick: read, "Lire le document" }
            pre { id: "document", "{document}" }
        }
        section {
            h2 { "3. Recherche documentaire" }
            p { class: "muted", "Recherche lexicale : seulement les documents autorisés, trois extraits maximum. Le mode sémantique n’est pas activé." }
            label { r#for: "query", "Mots recherchés" }
            input { id: "query", value: "{query}", maxlength: 500, disabled: busy(),
                oninput: move |e| query.set(e.value()) }
            button { id: "retrieve", disabled: busy(), onclick: retrieve, "Rechercher" }
            pre { id: "results", "{results}" }
        }
        section {
            h2 { "4. Conversation locale" }
            label { r#for: "message", "Message" }
            textarea { id: "message", value: "{message}", maxlength: 8000, disabled: busy(),
                oninput: move |e| message.set(e.value()) }
            for (id, label, route) in [
                ("chat", "Chat simple", "/api/chat"),
                ("rag", "Chat avec documents autorisés", "/api/rag-chat"),
            ] {
                button { id, disabled: busy(), onclick: move |_| send_chat(route), "{label}" }
            }
            pre { id: "answer", "{answer}" }
        }
        p { class: "muted", "Aucun historique enregistré. Aucun outil ni MCP. L’API décide des droits ; le modèle ne les décide jamais." }
    }
}

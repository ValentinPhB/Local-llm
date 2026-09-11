//! Commande administrative distincte du serveur : aucune route d'indexation.
use chatpurp_api::{
    bootstrap::load_policy,
    semantic::{LocalTransport, Semantic},
    storage::FileReader,
};
use chatpurp_core::indexing::prepare;
#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
async fn run() -> Result<(), &'static str> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 || !["--plan", "--replace-lab-index"].contains(&args[0].as_str()) {
        return Err(
            "Usage : chatpurp-index --plan|--replace-lab-index <racine> <resource_id>... ; replace efface/reconstruit uniquement lab_semantic_documents.",
        );
    }
    let root = std::path::Path::new(&args[1]);
    let (policy, _) = load_policy(root)?;
    let reader = FileReader::new(root).map_err(|_| "Lecteur indisponible.")?;
    let chunks =
        prepare(&policy, &reader, &args[2..]).map_err(|_| "Préparation documentaire refusée.")?;
    println!(
        "{}",
        serde_json::json!({"mode":args[0],"documents":args.len()-2,"passages":chunks.len(),"max_chunk_chars":700,"collection":"lab_semantic_documents"})
    );
    if args[0] == "--plan" {
        return Ok(());
    }
    // Seul ce mode explicitement choisi atteint un fournisseur et une écriture.
    let semantic = Semantic {
        transport: LocalTransport,
    };
    let batch = semantic
        .embed_batch(chunks)
        .await
        .map_err(|_| "Embeddings refusés ou indisponibles. Aucune écriture d'index.")?;
    semantic.replace_lab_index(&batch).await.map_err(|_|"Reconstruction échouée : l'index peut être absent ou partiel. Ne pas activer la recherche avant vérification.")?;
    println!("Index de démonstration reconstruit. Le raccordement aux routes reste désactivé.");
    Ok(())
}

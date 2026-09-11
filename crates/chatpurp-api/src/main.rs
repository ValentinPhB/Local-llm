#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(message) = run().await {
        eprintln!("{message}");
        std::process::exit(1);
    }
}
async fn run() -> Result<(), &'static str> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err(
            "Usage : chatpurp-api <racine du projet> <artefacts frontend compilés>. Écoute fixe 127.0.0.1:3211.",
        );
    }
    let state = chatpurp_api::bootstrap::assemble(
        std::path::Path::new(&args[0]),
        std::path::Path::new(&args[1]),
        3211,
    )?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3211")
        .await
        .map_err(|_| "Port local 3211 indisponible.")?;
    println!("ChatPurp Rust : http://127.0.0.1:3211 — sessions fictives uniquement.");
    chatpurp_api::http::serve(listener, std::sync::Arc::new(state))
        .await
        .map_err(|_| "Arrêt du serveur local.")
}

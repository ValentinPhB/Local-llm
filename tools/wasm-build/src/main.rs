use chatpurp_wasm_build::transform;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        eprintln!(
            "Usage: chatpurp-wasm-transform <application.wasm de confiance> <nouvelle-sortie>"
        );
        return ExitCode::from(64);
    }
    match transform(Path::new(&args[0]), Path::new(&args[1])) {
        Ok(files) => {
            println!(
                "Transformation vérifiée : {} fichiers ; aucun navigateur exécuté.",
                files.len()
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!(
                "Transformation refusée ou échouée : {error}. Ne pas servir une sortie partielle."
            );
            ExitCode::FAILURE
        }
    }
}

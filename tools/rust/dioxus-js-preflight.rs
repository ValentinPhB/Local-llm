//! Pré-vol local pour lazy-js-bundle 0.7.10, sans exécuter son build.rs.
//! DefaultHasher reproduit un cache amont, pas une preuve d'authenticité.
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::Hasher;
use std::io;
use std::path::Path;

fn source_hash(source: &str) -> u64 {
    let mut hash = DefaultHasher::new();
    for line in source.lines() {
        hash.write(line.as_bytes());
    }
    hash.finish()
}

fn verify(root: &Path, outputs: &[&str]) -> io::Result<()> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(root.join("src/ts"))? {
        let path = entry?.path();
        if matches!(path.extension().and_then(|s| s.to_str()), Some("ts" | "js")) {
            paths.push(path);
        }
    }
    paths.sort();
    if paths.is_empty() {
        return Err(io::Error::other("aucune source JS/TS à vérifier"));
    }
    let hashes = paths
        .iter()
        .map(|path| fs::read_to_string(path).map(|s| source_hash(&s)))
        .collect::<io::Result<Vec<_>>>()?;
    let expected = fs::read_to_string(root.join("src/js/hash.txt"))?;
    if expected.trim() != format!("{hashes:?}") {
        return Err(io::Error::other(
            "empreintes différentes : build Bun refusé",
        ));
    }
    for output in outputs {
        let path = root.join("src/js").join(output);
        if !fs::metadata(&path)?.is_file() || fs::read(&path)?.is_empty() {
            return Err(io::Error::other("JS préconstruit absent ou vide"));
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err(io::Error::other(
            "attendu : dossiers web, document, interpreter-js",
        ));
    }
    let outputs: &[&[&str]] = &[
        &["eval.js"],
        &["head.js"],
        &[
            "set_attribute.js",
            "native.js",
            "core.js",
            "hydrate.js",
            "patch_console.js",
            "initialize_streaming.js",
            "common.js",
        ],
    ];
    for (root, files) in args.iter().zip(outputs) {
        verify(Path::new(root), files)?;
    }
    println!("Pré-vol : empreintes conformes et JS présents pour les trois crates.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Dossier fourni par mktemp dans le lanceur, sans accès au cache amont.
    fn fixture(name: &str) -> std::path::PathBuf {
        let root = std::path::PathBuf::from(std::env::var_os("DIOXUS_PREFLIGHT_TEST_DIR").unwrap())
            .join(name);
        fs::create_dir_all(root.join("src/ts")).unwrap();
        fs::create_dir_all(root.join("src/js")).unwrap();
        fs::write(root.join("src/ts/a.ts"), "const a = 1;\n").unwrap();
        fs::write(root.join("src/js/a.js"), "const a=1;").unwrap();
        fs::write(
            root.join("src/js/hash.txt"),
            format!("[{}]", source_hash("const a = 1;\n")),
        )
        .unwrap();
        root
    }

    #[test]
    fn matching_sources_and_js_are_accepted() {
        let root = fixture("matching");
        assert!(verify(&root, &["a.js"]).is_ok());
    }

    #[test]
    fn changed_source_is_rejected() {
        let root = fixture("changed");
        fs::write(root.join("src/ts/a.ts"), "changed").unwrap();
        assert!(verify(&root, &["a.js"]).is_err());
    }

    #[test]
    fn missing_output_is_rejected() {
        let root = fixture("missing");
        assert!(verify(&root, &["missing.js"]).is_err());
    }

    #[test]
    fn missing_hash_is_rejected() {
        let root = fixture("hash");
        fs::remove_file(root.join("src/js/hash.txt")).unwrap();
        assert!(verify(&root, &["a.js"]).is_err());
    }

    #[test]
    fn line_endings_follow_upstream_normalization() {
        assert_eq!(source_hash("a\r\nb\r\n"), source_hash("a\nb\n"));
    }
}

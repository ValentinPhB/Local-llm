//! Thin adapter around the official engine. Trusted local build artifacts only.
//! This is not a sandbox for hostile WASM or concurrent same-user processes.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use wasm_bindgen_cli_support::Bindgen;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub const MAX_INPUT_BYTES: u64 = 64 * 1024 * 1024;
const STEM: &str = "chatpurp_web";
const WASM_HEADER: &[u8] = b"\0asm\x01\0\0\0";

pub fn check_environment() -> Result<()> {
    if std::env::var_os("RAYON_NUM_THREADS").as_deref() != Some("1".as_ref()) {
        return Err("RAYON_NUM_THREADS doit valoir 1".into());
    }
    if std::env::vars_os().any(|(key, _)| key.to_string_lossy().starts_with("WASM_BINDGEN_")) {
        return Err("retirer les variables WASM_BINDGEN_* héritées".into());
    }
    Ok(())
}

// Only portable, nonempty relative paths; no ambiguous or special components.
fn safe_relative(path: &str) -> bool {
    !path.is_empty()
        && path.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        })
}

fn add_file(files: &mut BTreeMap<PathBuf, Vec<u8>>, path: String, bytes: Vec<u8>) -> Result<()> {
    if !safe_relative(&path) || files.insert(PathBuf::from(&path), bytes).is_some() {
        return Err(format!("chemin de sortie interdit ou dupliqué : {path}").into());
    }
    Ok(())
}

fn validate_destination(destination: &Path) -> Result<()> {
    if !matches!(
        destination.components().next_back(),
        Some(Component::Normal(_))
    ) {
        return Err("destination : nom final normal requis".into());
    }
    match fs::symlink_metadata(destination) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Ok(_) => return Err("destination déjà existante : aucun écrasement autorisé".into()),
        Err(error) => return Err(error.into()),
    }
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if !fs::metadata(parent)?.is_dir() {
        return Err("le parent de destination doit être un répertoire existant".into());
    }
    Ok(())
}

/// Returns the checked relative files. On error an output directory may be partial:
/// never serve it, and never reuse it. No existing directory is removed or reused.
pub fn transform(input: &Path, destination: &Path) -> Result<Vec<PathBuf>> {
    check_environment()?;
    validate_destination(destination)?;
    let metadata = fs::symlink_metadata(input)?;
    if !metadata.file_type().is_file() {
        return Err("entrée : fichier régulier sans lien symbolique requis".into());
    }
    if metadata.len() > MAX_INPUT_BYTES {
        return Err("entrée supérieure à 64 Mio".into());
    }
    let mut bytes = Vec::new();
    File::open(input)?
        .take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_INPUT_BYTES || !bytes.starts_with(WASM_HEADER) {
        return Err("entrée : en-tête WASM v1 invalide ou taille excessive".into());
    }

    let mut engine = Bindgen::new();
    // input_bytes avoids a second read by the engine, but does not sandbox any
    // package.json path embedded in the trusted build's metadata.
    engine.input_bytes(STEM, bytes);
    engine.web(true)?;
    engine.typescript(false).omit_default_module_path(false);
    let mut output = engine.generate_output()?;
    if !output.npm_dependencies().is_empty() {
        return Err("dépendances npm générées non autorisées pour cette application".into());
    }

    // Validate all engine-provided snippet paths before any output is emitted.
    let mut snippets = BTreeMap::new();
    for (identifier, list) in output.snippets() {
        if !safe_relative(identifier) {
            return Err("identifiant de snippet interdit".into());
        }
        for (index, js) in list.iter().enumerate() {
            add_file(
                &mut snippets,
                format!("snippets/{identifier}/inline{index}.js"),
                js.as_bytes().to_vec(),
            )?;
        }
    }
    for (path, js) in output.local_modules() {
        if !safe_relative(path) {
            return Err("chemin de module local interdit".into());
        }
        add_file(
            &mut snippets,
            format!("snippets/{path}"),
            js.as_bytes().to_vec(),
        )?;
    }

    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    // Atomic create: an existing destination (even empty) never gets reused.
    builder.create(destination)?;
    output.emit(destination)?;

    let js_path = PathBuf::from(format!("{STEM}.js"));
    let wasm_path = PathBuf::from(format!("{STEM}_bg.wasm"));
    let js = fs::read_to_string(destination.join(&js_path))?;
    if js.is_empty() || !js.contains("export") || !js.contains("chatpurp_web_bg.wasm") {
        return Err("chargeur web généré incomplet".into());
    }
    let mut header = [0; 8];
    File::open(destination.join(&wasm_path))?.read_exact(&mut header)?;
    if header != WASM_HEADER {
        return Err("WASM généré invalide".into());
    }
    for (path, expected) in &snippets {
        if fs::read(destination.join(path))? != *expected {
            return Err("snippet généré absent ou différent du moteur".into());
        }
    }
    let mut expected = vec![js_path, wasm_path];
    expected.extend(snippets.into_keys());
    expected.sort();
    let mut actual = Vec::new();
    collect_files(destination, destination, &mut actual)?;
    actual.sort();
    if actual != expected {
        return Err("inventaire des fichiers générés inattendu".into());
    }
    Ok(expected)
}

fn collect_files(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            collect_files(root, &entry.path(), files)?;
        } else if kind.is_file() {
            files.push(entry.path().strip_prefix(root)?.to_path_buf());
        } else {
            return Err("sortie spéciale ou lien symbolique interdit".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snippet_paths_are_relative_portable_and_unambiguous() {
        for path in ["dioxus-web-123/inline0.js", "module.js"] {
            assert!(safe_relative(path), "{path}");
        }
        for path in [
            "", "/a", "../a", "a/../b", "a//b", "a/", "a/./b", "a\\b", "C:/a", "a\0b",
        ] {
            assert!(!safe_relative(path), "{path:?}");
        }
    }

    #[test]
    fn duplicate_snippet_paths_fail() {
        let mut files = BTreeMap::new();
        add_file(&mut files, "snippets/a.js".into(), vec![]).unwrap();
        assert!(add_file(&mut files, "snippets/a.js".into(), vec![]).is_err());
        assert!(add_file(&mut files, "../escape.js".into(), vec![]).is_err());
    }
}

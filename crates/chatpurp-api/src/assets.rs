//! Artefacts frontend chargés une fois ; aucune exposition du dépôt.
use axum::body::Bytes;
use std::{collections::BTreeMap, fs, io::Read, path::Path};

const MAX_BYTES: usize = 32 * 1024 * 1024;
const MAX_FILES: usize = 64;
const HTML: &str = include_str!("../../chatpurp-web/assets/index.html");
const BOOTSTRAP: &str = include_str!("../../chatpurp-web/assets/bootstrap.js");
pub const CSP: &str = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; connect-src 'self'; style-src 'self'; worker-src 'none'; frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";

#[derive(Clone)]
pub struct Asset {
    pub bytes: Bytes,
    pub mime: &'static str,
}

pub fn allowed_asset(path: &str) -> bool {
    if path == "chatpurp_web.js" || path == "chatpurp_web_bg.wasm" {
        return true;
    }
    path.starts_with("snippets/")
        && path.ends_with(".js")
        && path.split('/').all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        })
}

fn collect(
    root: &Path,
    dir: &Path,
    depth: usize,
    files: &mut BTreeMap<String, Asset>,
    total: &mut usize,
) -> Result<(), String> {
    if depth > 8 {
        return Err("too many directory levels".into());
    }
    for item in fs::read_dir(dir).map_err(|_| "cannot list assets")? {
        let item = item.map_err(|_| "cannot inspect asset")?;
        let kind = item.file_type().map_err(|_| "cannot inspect asset type")?;
        if kind.is_dir() {
            collect(root, &item.path(), depth + 1, files, total)?;
        } else if kind.is_file() {
            if files.len() >= MAX_FILES {
                return Err("too many assets".into());
            }
            let path = item.path();
            let name = path
                .strip_prefix(root)
                .map_err(|_| "invalid root")?
                .to_str()
                .ok_or("invalid name")?;
            if !allowed_asset(name) {
                return Err("unexpected asset".into());
            }
            let mut bytes = Vec::new();
            fs::File::open(&path)
                .map_err(|_| "cannot open asset")?
                .take((MAX_BYTES - *total + 1) as u64)
                .read_to_end(&mut bytes)
                .map_err(|_| "cannot read asset")?;
            *total += bytes.len();
            if *total > MAX_BYTES {
                return Err("asset size limit".into());
            }
            let mime = if name.ends_with(".wasm") {
                if !bytes.starts_with(b"\0asm\x01\0\0\0") {
                    return Err("invalid WASM header".into());
                }
                "application/wasm"
            } else {
                "text/javascript; charset=utf-8"
            };
            files.insert(
                format!("/{name}"),
                Asset {
                    bytes: bytes.into(),
                    mime,
                },
            );
        } else {
            return Err("symlink or special asset refused".into());
        }
    }
    Ok(())
}

pub fn load_assets(root: &Path) -> Result<BTreeMap<String, Asset>, String> {
    if !fs::symlink_metadata(root)
        .map_err(|_| "missing asset directory")?
        .is_dir()
    {
        return Err("asset directory required".into());
    }
    let mut files = BTreeMap::new();
    collect(root, root, 0, &mut files, &mut 0)?;
    if !files.contains_key("/chatpurp_web.js") || !files.contains_key("/chatpurp_web_bg.wasm") {
        return Err("missing generated JS/WASM pair".into());
    }
    files.insert(
        "/".into(),
        Asset {
            bytes: Bytes::from_static(HTML.as_bytes()),
            mime: "text/html; charset=utf-8",
        },
    );
    files.insert(
        "/bootstrap.js".into(),
        Asset {
            bytes: Bytes::from_static(BOOTSTRAP.as_bytes()),
            mime: "text/javascript; charset=utf-8",
        },
    );
    files.insert(
        "/style.css".into(),
        Asset {
            bytes: Bytes::from_static(include_bytes!("../../chatpurp-web/assets/style.css")),
            mime: "text/css; charset=utf-8",
        },
    );
    Ok(files)
}

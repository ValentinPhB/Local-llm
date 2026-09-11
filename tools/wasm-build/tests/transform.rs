//! Runs the actual executable; no mocked engine and no browser or network.
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

static NEXT: AtomicUsize = AtomicUsize::new(0);
const HEADER: &[u8] = b"\0asm\x01\0\0\0";

struct Case(PathBuf);

impl Case {
    fn new() -> Self {
        let parent =
            std::env::var_os("CHATPURP_TRANSFORM_TEST_DIR").expect("utiliser sh tools/check.sh");
        let path = PathBuf::from(parent).join(format!(
            "case-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    fn input(&self, bytes: &[u8]) -> PathBuf {
        let path = self.path("input.wasm");
        fs::write(&path, bytes).unwrap();
        path
    }
}

impl Drop for Case {
    fn drop(&mut self) {
        // Only the fresh test-owned case directory, never a caller-selected root.
        fs::remove_dir_all(&self.0).expect("nettoyage de la fixture privée");
    }
}

fn command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_chatpurp-wasm-transform"));
    command.env_clear().env("RAYON_NUM_THREADS", "1");
    command
}

fn run(mut command: Command) -> Output {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(45);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            child.kill().unwrap();
            let output = child.wait_with_output().unwrap();
            panic!("transformateur bloqué après 45 s : {output:?}");
        }
        // Process supervision, not a fixed delay used as a success condition.
        std::thread::sleep(Duration::from_millis(10));
    }
    child.wait_with_output().unwrap()
}

fn convert(input: &Path, output: &Path) -> Output {
    let mut cmd = command();
    cmd.arg(input).arg(output);
    run(cmd)
}

fn refused(output: Output, message: &str) {
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty(), "pas de faux succès : {output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(message),
        "{output:?}"
    );
}

#[test]
fn arguments_are_required_and_extra_arguments_rejected() {
    for args in [vec![], vec!["input"], vec!["a", "b", "c"]] {
        let mut cmd = command();
        cmd.args(args);
        let output = run(cmd);
        assert_eq!(output.status.code(), Some(64));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
    }
}

#[test]
fn inherited_engine_options_and_parallelism_are_rejected() {
    let case = Case::new();
    let input = case.input(HEADER);
    for (key, value) in [
        ("WASM_BINDGEN_ANYREF", ""),
        ("WASM_BINDGEN_MULTI_VALUE", "1"),
        ("RAYON_NUM_THREADS", "4"),
    ] {
        let mut cmd = command();
        cmd.arg(&input).arg(case.path("output")).env(key, value);
        refused(
            run(cmd),
            if key == "RAYON_NUM_THREADS" {
                "RAYON_NUM_THREADS"
            } else {
                "WASM_BINDGEN_"
            },
        );
        assert!(!case.path("output").exists());
    }
    let mut cmd = command();
    cmd.arg(input)
        .arg(case.path("output"))
        .env_remove("RAYON_NUM_THREADS");
    refused(run(cmd), "RAYON_NUM_THREADS");
}

#[test]
fn missing_input_does_not_create_output() {
    let case = Case::new();
    refused(
        convert(&case.path("absent.wasm"), &case.path("output")),
        "échouée",
    );
    assert!(!case.path("output").exists());
}

#[test]
fn directory_input_is_rejected() {
    let case = Case::new();
    refused(convert(&case.0, &case.path("output")), "fichier régulier");
    assert!(!case.path("output").exists());
}

#[test]
fn input_larger_than_limit_is_rejected_without_reading_it() {
    let case = Case::new();
    let input = case.input(HEADER);
    File::options()
        .write(true)
        .open(&input)
        .unwrap()
        .set_len(chatpurp_wasm_build::MAX_INPUT_BYTES + 1)
        .unwrap();
    refused(convert(&input, &case.path("output")), "64 Mio");
    assert!(!case.path("output").exists());
}

#[test]
fn invalid_header_and_corrupt_wasm_fail_before_output() {
    let case = Case::new();
    for bytes in [b"not wasm".to_vec(), [HEADER, &[1, 255]].concat()] {
        refused(
            convert(&case.input(&bytes), &case.path("output")),
            "échouée",
        );
        assert!(!case.path("output").exists());
    }
}

#[test]
fn incompatible_schema_is_rejected_by_official_engine() {
    // Handcrafted test fixture only, not a production WASM encoder/parser.
    let name = b"__wasm_bindgen_unstable";
    let schema = br#"{"schema_version":"chatpurp-incompatible","version":"0.0.0"}"#;
    let mut section = vec![name.len() as u8];
    section.extend_from_slice(name);
    section.extend_from_slice(&(schema.len() as u32).to_le_bytes());
    section.extend_from_slice(schema);
    assert!(section.len() < 128);
    let bytes = [HEADER, &[0, section.len() as u8], &section].concat();
    let case = Case::new();
    refused(
        convert(&case.input(&bytes), &case.path("output")),
        "different bindgen format",
    );
    assert!(!case.path("output").exists());
}

#[test]
fn existing_directory_is_not_overwritten() {
    let case = Case::new();
    let output = case.path("output");
    fs::create_dir(&output).unwrap();
    fs::write(output.join("sentinel"), b"keep").unwrap();
    refused(convert(&case.input(HEADER), &output), "déjà existante");
    assert_eq!(fs::read(output.join("sentinel")).unwrap(), b"keep");
    assert_eq!(fs::read_dir(output).unwrap().count(), 1);
}

#[test]
fn existing_file_and_missing_parent_are_rejected() {
    let case = Case::new();
    let input = case.input(HEADER);
    let output = case.path("output");
    fs::write(&output, b"keep").unwrap();
    refused(convert(&input, &output), "déjà existante");
    assert_eq!(fs::read(&output).unwrap(), b"keep");
    refused(convert(&input, &case.path("absent/output")), "échouée");
    assert!(!case.path("absent").exists());
    refused(convert(&input, &case.path("output/child")), "échouée");
}

#[cfg(unix)]
#[test]
fn input_symlinks_and_output_symlinks_even_dangling_are_rejected() {
    use std::os::unix::fs::symlink;
    let case = Case::new();
    let input = case.input(HEADER);
    symlink(&input, case.path("input-link")).unwrap();
    refused(
        convert(&case.path("input-link"), &case.path("output")),
        "sans lien",
    );
    symlink(case.path("absent"), case.path("output")).unwrap();
    refused(convert(&input, &case.path("output")), "déjà existante");
    assert!(
        fs::symlink_metadata(case.path("output"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert!(!case.path("absent").exists());
}

fn tree(root: &Path, path: &Path, files: &mut std::collections::BTreeMap<PathBuf, Vec<u8>>) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            tree(root, &entry.path(), files);
        } else {
            assert!(entry.file_type().unwrap().is_file());
            files.insert(
                entry.path().strip_prefix(root).unwrap().into(),
                fs::read(entry.path()).unwrap(),
            );
        }
    }
}

#[test]
fn real_dioxus_application_generates_complete_repeatable_web_artifacts() {
    let input = PathBuf::from(
        std::env::var_os("CHATPURP_APP_WASM")
            .expect("application compilée requise, jamais ignorée"),
    );
    assert!(
        input.is_file(),
        "lancer tools/check.sh pour préparer le build"
    );
    let original = fs::read(&input).unwrap();
    let case = Case::new();
    let mut runs = Vec::new();
    for name in ["first", "second"] {
        let destination = case.path(name);
        let result = convert(&input, &destination);
        assert!(result.status.success(), "{result:?}");
        assert!(String::from_utf8_lossy(&result.stdout).contains("Transformation vérifiée"));
        assert!(result.stderr.is_empty(), "{result:?}");
        let mut files = std::collections::BTreeMap::new();
        tree(&destination, &destination, &mut files);
        let js = String::from_utf8(files[Path::new("chatpurp_web.js")].clone()).unwrap();
        assert!(js.contains("export { initSync, __wbg_init as default };"));
        assert!(js.contains("new URL('chatpurp_web_bg.wasm', import.meta.url)"));
        assert!(files[Path::new("chatpurp_web_bg.wasm")].starts_with(HEADER));
        assert_ne!(files[Path::new("chatpurp_web_bg.wasm")], original);
        assert!(
            files.keys().any(|p| p.starts_with("snippets")),
            "snippets Dioxus attendus"
        );
        // The pinned engine also emits local modules not imported by this
        // particular entrypoint. Check import -> file, not the converse.
        let imports: Vec<_> = js
            .lines()
            .filter(|line| line.starts_with("import "))
            .collect();
        assert!(!imports.is_empty());
        for import in imports {
            let path = import
                .split('\'')
                .nth(1)
                .expect("forme d'import de la version verrouillée");
            let relative = path
                .strip_prefix("./snippets/")
                .expect("import local attendu");
            assert!(
                files.contains_key(Path::new(&format!("snippets/{relative}"))),
                "import absent : {path}"
            );
        }
        assert!(
            !files
                .keys()
                .any(|p| p.extension().is_some_and(|e| e == "ts" || e == "json"))
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&destination).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        runs.push(files);
    }
    assert!(
        runs[0] == runs[1],
        "deux transformations fraîches doivent concorder"
    );
    assert_eq!(
        fs::read(input).unwrap(),
        original,
        "ne pas modifier le WASM source"
    );
}

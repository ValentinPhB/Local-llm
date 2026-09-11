//! Descripteurs de répertoires conservés : jamais de chemin reçu du navigateur.
use crate::sessions::Clock;
use chatpurp_core::{
    application::{Audit, AuditDecision, Document, Failure, Reader},
    policy::{Resource, valid_id},
};
use rustix::{
    fd::OwnedFd,
    fs::{self, FileType, Mode, OFlags},
};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::Path,
    sync::{Arc, Mutex},
};

const DIRECTORY: OFlags = OFlags::RDONLY
    .union(OFlags::DIRECTORY)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::CLOEXEC);
const DOCUMENT_LIMIT: u64 = 32_768;
pub struct FileReader {
    root: OwnedFd,
}
impl FileReader {
    pub fn new(project: &Path) -> Result<Self, Failure> {
        let root = fs::open(project.join("demo-documents"), DIRECTORY, Mode::empty())
            .map_err(|_| Failure::Document)?;
        Ok(Self { root })
    }
}
fn metadata(content: &str) -> Option<BTreeMap<&str, &str>> {
    let mut lines = content.lines();
    if lines.next()? != "---" {
        return None;
    }
    let mut result = BTreeMap::new();
    for line in lines {
        if line == "---" {
            return Some(result);
        }
        let (key, value) = line.split_once(':')?;
        if key.is_empty() || value.trim().is_empty() || result.insert(key, value.trim()).is_some() {
            return None;
        }
    }
    None
}
impl Reader for FileReader {
    fn read(&self, resource: &Resource) -> Result<Document, Failure> {
        let read = || -> Option<String> {
            let relative = resource.path.strip_prefix("demo-documents/")?;
            let parts: Vec<_> = relative.split('/').collect();
            if !relative.ends_with(".md")
                || parts.iter().any(|p| {
                    p.is_empty()
                        || *p == "."
                        || *p == ".."
                        || !p
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
                })
            {
                return None;
            }
            let mut parent = fs::openat(&self.root, ".", DIRECTORY, Mode::empty()).ok()?;
            for part in &parts[..parts.len() - 1] {
                parent = fs::openat(&parent, *part, DIRECTORY, Mode::empty()).ok()?;
            }
            let fd = fs::openat(
                &parent,
                *parts.last()?,
                OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .ok()?;
            let stat = fs::fstat(&fd).ok()?;
            if FileType::from_raw_mode(stat.st_mode) != FileType::RegularFile
                || stat.st_size < 0
                || stat.st_size as u64 > DOCUMENT_LIMIT
            {
                return None;
            }
            let mut bytes = Vec::new();
            File::from(fd)
                .take(DOCUMENT_LIMIT + 1)
                .read_to_end(&mut bytes)
                .ok()?;
            if bytes.len() as u64 > DOCUMENT_LIMIT {
                return None;
            }
            let content = String::from_utf8(bytes).ok()?;
            let m = metadata(&content)?;
            if m.get("id") != Some(&resource.id.as_str())
                || m.get("classification") != Some(&resource.classification.as_str())
            {
                return None;
            }
            Some(content)
        };
        Ok(Document {
            resource_id: resource.id.clone(),
            classification: resource.classification.clone(),
            content: read().ok_or(Failure::Document)?,
        })
    }
}

const CURRENT: &str = "access-decisions.jsonl";
const BACKUP: &str = "access-decisions.1.jsonl";
pub struct FileAudit {
    dir: OwnedFd,
    lock: Mutex<()>,
    clock: Arc<dyn Clock>,
    limit: u64,
}
impl FileAudit {
    /// Appelé avec un répertoire de données local explicite, jamais depuis HTTP.
    pub fn new(project: &Path, clock: Arc<dyn Clock>) -> Result<Self, Failure> {
        Self::with_limit(project, clock, 1_048_576)
    }
    fn with_limit(project: &Path, clock: Arc<dyn Clock>, limit: u64) -> Result<Self, Failure> {
        let make = || -> Result<OwnedFd, rustix::io::Errno> {
            let mut dir = fs::open(project, DIRECTORY, Mode::empty())?;
            for component in [".local", "rust-api", "audit"] {
                match fs::mkdirat(&dir, component, Mode::from_raw_mode(0o700)) {
                    Ok(()) | Err(rustix::io::Errno::EXIST) => (),
                    Err(e) => return Err(e),
                }
                dir = fs::openat(&dir, component, DIRECTORY, Mode::empty())?;
            }
            fs::fchmod(&dir, Mode::from_raw_mode(0o700))?;
            Ok(dir)
        };
        Ok(Self {
            dir: make().map_err(|_| Failure::Audit)?,
            lock: Mutex::new(()),
            clock,
            limit,
        })
    }
    fn open_file(&self, name: &str, create: bool) -> Result<File, Failure> {
        let flags = OFlags::WRONLY
            | OFlags::APPEND
            | OFlags::NOFOLLOW
            | OFlags::NONBLOCK
            | OFlags::CLOEXEC
            | if create {
                OFlags::CREATE
            } else {
                OFlags::empty()
            };
        let fd = fs::openat(&self.dir, name, flags, Mode::from_raw_mode(0o600))
            .map_err(|_| Failure::Audit)?;
        let stat = fs::fstat(&fd).map_err(|_| Failure::Audit)?;
        if FileType::from_raw_mode(stat.st_mode) != FileType::RegularFile || stat.st_nlink != 1 {
            return Err(Failure::Audit);
        }
        fs::fchmod(&fd, Mode::from_raw_mode(0o600)).map_err(|_| Failure::Audit)?;
        Ok(File::from(fd))
    }
}
impl Audit for FileAudit {
    fn record(&self, decision: AuditDecision) -> Result<(), Failure> {
        if decision
            .identity_id
            .as_deref()
            .is_some_and(|id| !valid_id(id))
            || decision
                .resource_id
                .as_deref()
                .is_some_and(|id| !valid_id(id))
        {
            return Err(Failure::Audit);
        }
        let unix = i64::try_from(self.clock.now().map_err(|_| Failure::Audit)?)
            .map_err(|_| Failure::Audit)?;
        let dt = time::OffsetDateTime::from_unix_timestamp(unix).map_err(|_| Failure::Audit)?;
        let stamp = format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            dt.year(),
            u8::from(dt.month()),
            dt.day(),
            dt.hour(),
            dt.minute(),
            dt.second()
        );
        let mut line = serde_json::to_vec(&serde_json::json!({"timestamp":stamp,"event":"access_decision","route":decision.route.as_str(),"outcome":if decision.allowed {"allowed"} else {"denied"},"identity_id":decision.identity_id,"resource_id":decision.resource_id})).map_err(|_| Failure::Audit)?;
        line.push(b'\n');
        if line.len() as u64 > self.limit {
            return Err(Failure::Audit);
        }
        let _guard = self.lock.lock().map_err(|_| Failure::Audit)?;
        let mut file = self.open_file(CURRENT, true)?;
        if file.metadata().map_err(|_| Failure::Audit)?.len() + line.len() as u64 > self.limit {
            match fs::statat(&self.dir, BACKUP, fs::AtFlags::SYMLINK_NOFOLLOW) {
                Ok(_) => {
                    self.open_file(BACKUP, false)?;
                }
                Err(rustix::io::Errno::NOENT) => (),
                Err(_) => return Err(Failure::Audit),
            }
            fs::renameat(&self.dir, CURRENT, &self.dir, BACKUP).map_err(|_| Failure::Audit)?;
            file = self.open_file(CURRENT, true)?;
        }
        file.write_all(&line)
            .and_then(|_| file.flush())
            .map_err(|_| Failure::Audit)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::sessions::SystemClock;
    use std::{
        os::unix::fs::{PermissionsExt, symlink},
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };
    pub struct Temp(pub PathBuf);
    impl Temp {
        pub fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "chatpurp-rust-test-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::SeqCst)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn resource() -> Resource {
        Resource {
            id: "public-welcome".into(),
            classification: "PUBLIC".into(),
            path: "demo-documents/public/welcome.md".into(),
            allowed_roles: vec![],
        }
    }
    const CONTENT: &str = "---\nid: public-welcome\nclassification: PUBLIC\n---\nBienvenue fictive";
    #[test]
    fn all_actual_fictive_documents_validate() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let policy: chatpurp_core::policy::Policy = serde_json::from_str(include_str!(
            "../../../config/access-control/demo-policy.json"
        ))
        .unwrap();
        let reader = FileReader::new(&root).unwrap();
        for r in &policy.resources {
            assert!(reader.read(r).is_ok(), "{}", r.id);
        }
    }
    #[test]
    fn reader_refuses_symlinks_traversal_sizes_and_metadata() {
        let t = Temp::new();
        let dir = t.0.join("demo-documents/public");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("welcome.md");
        std::fs::write(&path, CONTENT).unwrap();
        let reader = FileReader::new(&t.0).unwrap();
        let mut r = resource();
        assert!(reader.read(&r).is_ok());
        for content in [
            "missing".into(),
            CONTENT.replace("PUBLIC", "RH"),
            CONTENT.replace("classification:", "id: duplicate\nclassification:"),
            "x".repeat(32769),
        ] {
            std::fs::write(&path, content).unwrap();
            assert!(reader.read(&r).is_err());
        }
        std::fs::write(&path, [255, 254]).unwrap();
        assert!(reader.read(&r).is_err());
        std::fs::remove_file(&path).unwrap();
        symlink(t.0.join("outside"), &path).unwrap();
        std::fs::write(t.0.join("outside"), CONTENT).unwrap();
        assert!(reader.read(&r).is_err());
        r.path = "demo-documents/../outside.md".into();
        assert!(reader.read(&r).is_err());
        std::fs::rename(&dir, t.0.join("moved")).unwrap();
        symlink(t.0.join("moved"), &dir).unwrap();
        assert!(reader.read(&resource()).is_err());
    }
    #[test]
    fn directories_and_fifos_never_block_the_reader() {
        let t = Temp::new();
        let dir = t.0.join("demo-documents/public");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("welcome.md");
        std::fs::create_dir(&path).unwrap();
        let reader = FileReader::new(&t.0).unwrap();
        assert!(reader.read(&resource()).is_err());
        std::fs::remove_dir(&path).unwrap();
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
        assert!(reader.read(&resource()).is_err());
    }
    fn decision() -> AuditDecision {
        AuditDecision {
            route: chatpurp_core::application::Route::Document,
            allowed: true,
            identity_id: Some("oscar".into()),
            resource_id: Some("public-welcome".into()),
        }
    }
    #[test]
    fn audit_is_private_bounded_and_minimal() {
        let t = Temp::new();
        let audit = FileAudit::with_limit(&t.0, Arc::new(SystemClock), 450).unwrap();
        for _ in 0..12 {
            audit.record(decision()).unwrap();
        }
        let dir = t.0.join(".local/rust-api/audit");
        assert_eq!(
            std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 2);
        for name in [CURRENT, BACKUP] {
            let p = dir.join(name);
            let m = std::fs::metadata(&p).unwrap();
            assert!(m.len() <= 450);
            assert_eq!(m.permissions().mode() & 0o777, 0o600);
            for line in std::fs::read_to_string(p).unwrap().lines() {
                let v: serde_json::Value = serde_json::from_str(line).unwrap();
                assert_eq!(v.as_object().unwrap().len(), 6);
                assert_eq!(v["identity_id"], "oscar");
            }
        }
    }
    #[test]
    fn audit_refuses_link_destinations_and_link_directories() {
        let t = Temp::new();
        let audit = FileAudit::with_limit(&t.0, Arc::new(SystemClock), 250).unwrap();
        let dir = t.0.join(".local/rust-api/audit");
        let outside = t.0.join("outside");
        std::fs::write(&outside, "untouched").unwrap();
        symlink(&outside, dir.join(CURRENT)).unwrap();
        assert_eq!(audit.record(decision()), Err(Failure::Audit));
        assert_eq!(std::fs::read_to_string(&outside).unwrap(), "untouched");
        std::fs::remove_file(dir.join(CURRENT)).unwrap();
        audit.record(decision()).unwrap();
        symlink(&outside, dir.join(BACKUP)).unwrap();
        assert_eq!(audit.record(decision()), Err(Failure::Audit));
        let other = Temp::new();
        symlink(&t.0, other.0.join(".local")).unwrap();
        assert!(FileAudit::new(&other.0, Arc::new(SystemClock)).is_err());
    }
}

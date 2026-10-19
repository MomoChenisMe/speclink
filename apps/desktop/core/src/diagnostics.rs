//! Bounded, schema-filtered diagnostics for device operations.
use serde::Serialize;
use speclink_host::network::NetworkDiagnostic;
use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{sync_channel, SyncSender},
        Arc, Mutex,
    },
};

#[derive(Clone)]
pub struct Diagnostics {
    inner: Arc<Inner>,
}
struct Inner {
    tx: SyncSender<Message>,
    state: Arc<Mutex<Writer>>,
    enabled: Arc<AtomicBool>,
    failed: Arc<AtomicBool>,
    dropped: Arc<AtomicU64>,
}
struct Writer {
    root: PathBuf,
    max_bytes: u64,
}
enum Message {
    Event(NetworkDiagnostic),
    Flush(SyncSender<()>),
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportReceipt {
    pub path: String,
    pub bytes: u64,
    pub skipped_records: usize,
}
const NAMES: [&str; 4] = [
    "events.jsonl",
    "events.1.jsonl",
    "events.2.jsonl",
    "events.3.jsonl",
];
impl Diagnostics {
    pub fn new(root: PathBuf, enabled: bool) -> Self {
        Self::with_limit(root, enabled, 2 * 1024 * 1024)
    }
    fn with_limit(root: PathBuf, enabled: bool, max_bytes: u64) -> Self {
        let (tx, rx) = sync_channel(256);
        let state = Arc::new(Mutex::new(Writer { root, max_bytes }));
        let enabled = Arc::new(AtomicBool::new(enabled));
        let failed = Arc::new(AtomicBool::new(false));
        let dropped = Arc::new(AtomicU64::new(0));
        let (writer, on, error, lost) = (
            state.clone(),
            enabled.clone(),
            failed.clone(),
            dropped.clone(),
        );
        let launched = std::thread::Builder::new()
            .name("device-diagnostics".into())
            .spawn(move || {
                while let Ok(message) = rx.recv() {
                    match message {
                        Message::Flush(done) => {
                            let _ = done.send(());
                        }
                        Message::Event(mut event) => {
                            let Ok(mut writer) = writer.lock() else {
                                error.store(true, Ordering::Relaxed);
                                continue;
                            };
                            if !on.load(Ordering::Acquire) {
                                continue;
                            }
                            event.dropped_count = lost.swap(0, Ordering::Relaxed);
                            if writer.append(&event).is_err() {
                                error.store(true, Ordering::Relaxed);
                            }
                        }
                    }
                }
            });
        if launched.is_err() {
            failed.store(true, Ordering::Relaxed);
        }
        Self {
            inner: Arc::new(Inner {
                tx,
                state,
                enabled,
                failed,
                dropped,
            }),
        }
    }
    pub fn set_enabled(&self, enabled: bool) {
        let _writer = self.inner.state.lock().unwrap();
        self.inner.enabled.store(enabled, Ordering::Release);
    }
    pub fn status(&self) -> &'static str {
        if !self.inner.enabled.load(Ordering::Acquire) {
            "off"
        } else if self.inner.failed.load(Ordering::Relaxed) {
            "writeFailed"
        } else {
            "recording"
        }
    }
    pub fn record(&self, event: NetworkDiagnostic) -> Result<(), String> {
        if !self.inner.enabled.load(Ordering::Acquire) {
            return Ok(());
        }
        let event = clean_event(event)?;
        if serde_json::to_vec(&event)
            .map_err(|_| "diagnostics_invalid")?
            .len()
            > 16 * 1024
        {
            return Err("diagnostics_invalid".into());
        }
        match self.inner.tx.try_send(Message::Event(event)) {
            Ok(()) => {}
            Err(std::sync::mpsc::TrySendError::Full(_)) => {
                self.inner.dropped.fetch_add(1, Ordering::Relaxed);
            }
            Err(_) => {
                self.inner.failed.store(true, Ordering::Relaxed);
            }
        }
        Ok(())
    }
    pub fn sink(&self) -> speclink_host::network::DiagnosticSink {
        let log = self.clone();
        Arc::new(move |event| {
            let _ = log.record(event);
        })
    }
    pub fn flush(&self) {
        let (tx, rx) = sync_channel(0);
        if self.inner.tx.send(Message::Flush(tx)).is_ok() {
            let _ = rx.recv();
        }
    }
    pub fn export(&self, path: &Path, app_version: &str) -> Result<ExportReceipt, String> {
        self.flush();
        let writer = self
            .inner
            .state
            .lock()
            .map_err(|_| "diagnostics_write_failed")?;
        let mut entries = Vec::new();
        let mut skipped = 0;
        for name in NAMES {
            let file = writer.root.join(name);
            match std::fs::symlink_metadata(&file) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Ok(meta) if meta.is_file() && meta.len() <= 2 * 1024 * 1024 => {}
                _ => return Err("diagnostics_invalid".into()),
            }
            let bytes = std::fs::read(file).map_err(|_| "diagnostics_write_failed")?;
            let mut cleaned = Vec::new();
            for line in bytes.split(|b| *b == b'\n').filter(|l| !l.is_empty()) {
                let record = serde_json::from_slice::<NetworkDiagnostic>(line)
                    .ok()
                    .and_then(|event| clean_event(event).ok());
                if let Some(event) = record {
                    serde_json::to_writer(&mut cleaned, &event)
                        .map_err(|_| "diagnostics_invalid")?;
                    cleaned.push(b'\n');
                } else {
                    skipped += 1;
                }
            }
            entries.push((name, cleaned));
        }
        if entries.is_empty() {
            return Err("diagnostics_empty".into());
        }
        let manifest = serde_json::json!({"formatVersion":1,"appVersion":app_version,"os":std::env::consts::OS,
            "arch":std::env::consts::ARCH,"exportedAt":chrono::Utc::now().to_rfc3339(),
            "debugEnabled":self.inner.enabled.load(Ordering::Acquire),"skippedRecords":skipped,
            "files":entries.iter().map(|(name,bytes)|serde_json::json!({"name":name,"sha256":speclink_core::security::data::sha256(bytes)})).collect::<Vec<_>>()});
        let publish = || -> std::io::Result<ExportReceipt> {
            let parent = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
            {
                let mut zip = zip::ZipWriter::new(temporary.as_file_mut());
                let options = zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated)
                    .unix_permissions(0o600);
                zip.start_file("manifest.json", options)?;
                zip.write_all(&serde_json::to_vec_pretty(&manifest)?)?;
                for (name, bytes) in &entries {
                    zip.start_file(*name, options)?;
                    zip.write_all(bytes)?;
                }
                zip.finish()?;
            }
            let bytes = temporary.as_file().metadata()?.len();
            if bytes > 9 * 1024 * 1024 {
                return Err(std::io::Error::other("size_limit"));
            }
            temporary.as_file().sync_all()?;
            temporary.persist_noclobber(path).map_err(|e| e.error)?;
            Ok(ExportReceipt {
                path: path.to_string_lossy().into_owned(),
                bytes,
                skipped_records: skipped,
            })
        };
        publish().map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                "file_exists"
            } else {
                "diagnostics_write_failed"
            }
            .into()
        })
    }
}
impl Writer {
    fn append(&mut self, event: &NetworkDiagnostic) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.root)?;
        if std::fs::symlink_metadata(&self.root)?
            .file_type()
            .is_symlink()
        {
            return Err(std::io::Error::other("diagnostics_invalid"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.root, std::fs::Permissions::from_mode(0o700))?;
        }
        let mut bytes = serde_json::to_vec(event)?;
        bytes.push(b'\n');
        let path = self.root.join(NAMES[0]);
        match std::fs::symlink_metadata(&path) {
            Ok(meta) if !meta.is_file() => {
                return Err(std::io::Error::other("diagnostics_invalid"))
            }
            Ok(meta) if meta.len() + bytes.len() as u64 > self.max_bytes => {
                for i in (1..4).rev() {
                    let source = self.root.join(NAMES[i - 1]);
                    let target = self.root.join(NAMES[i]);
                    if source.exists() {
                        if target.exists() {
                            std::fs::remove_file(&target)?;
                        }
                        std::fs::rename(source, target)?;
                    }
                }
            }
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
        let mut options = std::fs::OpenOptions::new();
        options.create(true).append(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options.open(path)?.write_all(&bytes)
    }
}
fn clean_event(mut e: NetworkDiagnostic) -> Result<NetworkDiagnostic, String> {
    if e.format_version != 1
        || ulid::Ulid::from_string(&e.operation_id).is_err()
        || chrono::DateTime::parse_from_rfc3339(&e.timestamp).is_err()
    {
        return Err("diagnostics_invalid".into());
    }
    let origin = |raw: Option<String>| {
        raw.and_then(|v| url::Url::parse(&v).ok())
            .filter(|u| matches!(u.scheme(), "http" | "https") && u.host_str().is_some())
            .map(|u| u.origin().ascii_serialization())
    };
    e.destination = origin(e.destination);
    e.proxy_endpoint = origin(e.proxy_endpoint);
    e.http_status = e.http_status.filter(|s| (100..=599).contains(s));
    e.error_code = e.error_code.map(|code| safe_code(&code));
    e.error_chain.truncate(16);
    for detail in &mut e.error_chain {
        if ![
            "dns",
            "connect",
            "timeout",
            "tls",
            "proxy-auth",
            "http",
            "parse",
            "cancelled",
            "unknown",
            "frontend",
        ]
        .contains(&detail.kind.as_str())
        {
            detail.kind = "unknown".into();
        }
        detail.code = if detail.kind == "frontend" && safe_script_location(&detail.code) {
            detail.code.clone()
        } else {
            safe_code(&detail.code)
        };
    }
    Ok(e)
}
fn safe_script_location(code: &str) -> bool {
    let Some(raw) = code.strip_prefix("app:assets/") else {
        return false;
    };
    let parts: Vec<_> = raw.split(':').collect();
    parts.len() == 3
        && parts[0].len() <= 103
        && parts[0].strip_suffix(".js").is_some_and(|name| {
            !name.is_empty()
                && name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        })
        && parts[1..]
            .iter()
            .all(|n| !n.is_empty() && n.len() <= 7 && n.bytes().all(|b| b.is_ascii_digit()))
}
fn safe_code(code: &str) -> String {
    if [
        "download_failed",
        "untrusted_source",
        "proxy_invalid",
        "proxy_unsupported",
        "system-no-static-route",
        "proxy-auth",
        "timeout",
        "dns",
        "connect",
        "tls",
        "invalid-certificate",
        "unknown-issuer",
        "http",
        "parse",
        "cancelled",
        "unknown",
        "updater-response-unavailable",
        "Error",
        "TypeError",
        "SyntaxError",
        "RangeError",
        "ReferenceError",
        "render-error",
        "size_limit",
        "budget_exceeded",
    ]
    .contains(&code)
        || code
            .strip_prefix("os-")
            .is_some_and(|v| !v.is_empty() && v.len() < 12 && v.bytes().all(|b| b.is_ascii_digit()))
    {
        code.into()
    } else {
        "unknown".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_host::network::{DiagnosticAction, DiagnosticArea};
    fn event() -> NetworkDiagnostic {
        NetworkDiagnostic::new(
            DiagnosticArea::Frontend,
            DiagnosticAction::RenderError,
            "01ARZ3NDEKTSV4RRFFQ69G5FAV".into(),
        )
    }
    #[test]
    fn diagnostics_off_then_on_then_off_and_export() {
        let root = tempfile::tempdir().unwrap();
        let log = Diagnostics::new(root.path().to_path_buf(), false);
        log.record(event()).unwrap();
        log.flush();
        assert!(!root.path().join("events.jsonl").exists());
        log.set_enabled(true);
        log.record(event()).unwrap();
        log.flush();
        assert_eq!(log.status(), "recording");
        let before = std::fs::read(root.path().join("events.jsonl")).unwrap();
        log.set_enabled(false);
        log.record(event()).unwrap();
        log.flush();
        assert_eq!(
            std::fs::read(root.path().join("events.jsonl")).unwrap(),
            before
        );
        let output = root.path().join("diagnostics.zip");
        let receipt = log.export(&output, "test").unwrap();
        assert!(receipt.bytes > 0);
        assert_eq!(receipt.skipped_records, 0);
        assert!(log.export(&output, "test").is_err());
    }
    #[test]
    fn rotation_queue_limit_and_write_failure_are_observable() {
        let root = tempfile::tempdir().unwrap();
        let log = Diagnostics::with_limit(root.path().to_path_buf(), true, 1024);
        for _ in 0..20 {
            log.record(event()).unwrap();
            log.flush();
        }
        let paths = std::fs::read_dir(root.path())
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect::<Vec<_>>();
        assert_eq!(paths.len(), 4);
        assert!(paths
            .iter()
            .all(|p| std::fs::metadata(p).unwrap().len() <= 1024));
        let guard = log.inner.state.lock().unwrap();
        for _ in 0..300 {
            log.record(event()).unwrap();
        }
        assert!(log.inner.dropped.load(Ordering::Relaxed) > 0);
        drop(guard);
        log.flush();
        let file = root.path().join("not-directory");
        std::fs::write(&file, "unchanged").unwrap();
        let failed = Diagnostics::new(file, true);
        failed.record(event()).unwrap();
        failed.flush();
        assert_eq!(failed.status(), "writeFailed");
    }
    #[test]
    fn diagnostics_redacts_external_values_and_corrupt_records() {
        let root = tempfile::tempdir().unwrap();
        let log = Diagnostics::new(root.path().to_path_buf(), true);
        let mut e = event();
        e.destination =
            Some("https://user:SECRET@github.com/private/SECRET?token=SECRET#SECRET".into());
        e.error_code = Some("Bearer SECRET".into());
        e.error_chain.push(speclink_host::network::ErrorDetail {
            kind: "unknown".into(),
            code: "/Users/SECRET/project".into(),
        });
        log.record(e).unwrap();
        log.flush();
        let raw = std::fs::read_to_string(root.path().join("events.jsonl")).unwrap();
        assert!(!raw.contains("SECRET"));
        use std::io::Write;
        std::fs::OpenOptions::new()
            .append(true)
            .open(root.path().join("events.jsonl"))
            .unwrap()
            .write_all(b"corrupt SECRET\n")
            .unwrap();
        let path = root.path().join("export.zip");
        let receipt = log.export(&path, "test").unwrap();
        assert_eq!(receipt.skipped_records, 1);
        let mut archive = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
        use std::io::Read;
        let mut raw = String::new();
        archive
            .by_name("events.jsonl")
            .unwrap()
            .read_to_string(&mut raw)
            .unwrap();
        assert!(!raw.contains("SECRET"));
    }
}

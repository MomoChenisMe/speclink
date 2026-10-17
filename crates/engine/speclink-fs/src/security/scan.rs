//! Authorized, bounded project input. No subprocess or network access.
use anyhow::{ensure, Context, Result};
use speclink_core::security::{
    input::{
        CoverageEntry, InputKind, Language, ProjectInput, ScanInputs, ScanLimits, SourceDocument,
    },
    ReviewStatus,
};
use std::{
    fs::{File, OpenOptions},
    io::Read,
    path::{Component, Path, PathBuf},
};

pub struct AuthorizedProject {
    root: PathBuf,
    handle: File,
}
type Stop<'a> = &'a dyn Fn() -> Option<&'static str>;
impl AuthorizedProject {
    /// The caller selects this root explicitly. Only the root itself is canonicalized;
    /// all descendant opens reject links and are anchored to this directory handle.
    pub fn open(root: &Path) -> Result<Self> {
        let root = root
            .canonicalize()
            .context("unauthorized_root: cannot open selected project")?;
        ensure!(root.is_absolute(), "unauthorized_root");
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC);
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            // Keep the authorized directory from being renamed while descendants are opened.
            options.share_mode(
                windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ
                    | windows_sys::Win32::Storage::FileSystem::FILE_SHARE_WRITE,
            );
            options.custom_flags(
                windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT
                    | windows_sys::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS,
            );
        }
        let handle = options.open(&root)?;
        ensure!(
            handle.metadata()?.is_dir(),
            "unauthorized_root: expected directory"
        );
        #[cfg(windows)]
        verify_windows_handle(&handle, &root)?;
        Ok(Self { root, handle })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn verified_path(&self, relative: &str) -> Result<PathBuf> {
        let path = relative_path(relative)?;
        let file = self.open_relative(&path)?;
        ensure!(file.metadata()?.is_file(), "source_unavailable");
        Ok(self.root.join(path))
    }
    pub fn read_text(&self, relative: &str, limit: usize, stop: Stop<'_>) -> Result<String> {
        let path = relative_path(relative)?;
        let file = self.open_relative(&path)?;
        let mut consumed = 0;
        read_text_file(file, limit, stop, &mut consumed).map_err(|error| {
            anyhow::Error::new(speclink_core::security::input::InputReadFailure {
                code: stop().unwrap_or_else(|| error_code(&error)),
                bytes_read: consumed,
            })
        })
    }
    pub fn read_optional_text(
        &self,
        relative: &str,
        limit: usize,
        stop: Stop<'_>,
    ) -> Result<Option<String>> {
        match self.read_text(relative, limit, stop) {
            Ok(text) => Ok(Some(text)),
            Err(error)
                if error
                    .downcast_ref::<std::io::Error>()
                    .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound) =>
            {
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }
    fn open_relative(&self, path: &Path) -> Result<File> {
        #[cfg(unix)]
        {
            use std::{
                ffi::CString,
                os::{
                    fd::{AsRawFd, FromRawFd},
                    unix::ffi::OsStrExt,
                },
            };
            let mut current = self.handle.try_clone()?;
            let components = path.components().collect::<Vec<_>>();
            for (index, component) in components.iter().enumerate() {
                let Component::Normal(name) = component else {
                    anyhow::bail!("unauthorized_path");
                };
                let name = CString::new(name.as_bytes())
                    .map_err(|_| anyhow::anyhow!("unauthorized_path"))?;
                let flags = libc::O_RDONLY
                    | libc::O_NOFOLLOW
                    | libc::O_CLOEXEC
                    | libc::O_NONBLOCK
                    | if index + 1 < components.len() {
                        libc::O_DIRECTORY
                    } else {
                        0
                    };
                let descriptor = unsafe { libc::openat(current.as_raw_fd(), name.as_ptr(), flags) };
                if descriptor < 0 {
                    let error = std::io::Error::last_os_error();
                    if error.raw_os_error() == Some(libc::ELOOP) {
                        anyhow::bail!("skipped_link");
                    }
                    return Err(error).context("io_error: secure project open");
                }
                // SAFETY: successful openat returns an exclusively owned descriptor.
                current = unsafe { File::from_raw_fd(descriptor) };
                if index + 1 < components.len() {
                    ensure!(current.metadata()?.is_dir(), "unauthorized_path");
                }
            }
            Ok(current)
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            let mut current = self.root.clone();
            let mut opened = None;
            for component in path.components() {
                let Component::Normal(name) = component else {
                    anyhow::bail!("unauthorized_path");
                };
                current.push(name);
                let file = OpenOptions::new()
                    .read(true)
                    .custom_flags(
                        windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT
                            | windows_sys::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS,
                    )
                    .open(&current)?;
                verify_windows_handle(&file, &current)?;
                opened = Some(file);
            }
            opened.ok_or_else(|| anyhow::anyhow!("unauthorized_path"))
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = path;
            anyhow::bail!("unsupported_platform: secure project reader unavailable")
        }
    }
    pub fn scan(
        &self,
        exclude_dirs: &[String],
        limits: &ScanLimits,
        stop: Stop<'_>,
    ) -> Result<ScanInputs> {
        self.scan_from(vec![PathBuf::new()], None, exclude_dirs, limits, stop)
    }
    fn scan_from(
        &self,
        mut pending: Vec<PathBuf>,
        kind: Option<InputKind>,
        exclude_dirs: &[String],
        limits: &ScanLimits,
        stop: Stop<'_>,
    ) -> Result<ScanInputs> {
        for excluded in exclude_dirs {
            relative_path(excluded)?;
        }
        let started = std::time::Instant::now();
        let check = || {
            stop().or_else(|| {
                (started.elapsed().as_secs() >= limits.seconds).then_some("budget_exceeded")
            })
        };
        let mut result = ScanInputs {
            visited_files: 0,
            documents: Vec::new(),
            coverage: Vec::new(),
            remaining_targets: Vec::new(),
            total_bytes: 0,
            complete: true,
            stop_reason: None,
        };
        let mut files = 0usize;
        let mut entries = 0usize;
        while let Some(path) = pending.pop() {
            let label = display_path(&path);
            if let Some(reason) = check() {
                stop_scan(&mut result, reason, label, &pending);
                break;
            }
            if path.to_str().is_none() || path.to_string_lossy().chars().any(char::is_control) {
                skip(&mut result, label, "invalid_path");
                continue;
            }
            let depth = path.components().count();
            if depth > limits.depth {
                skip(&mut result, label, "depth_limit");
                continue;
            }
            if !path.as_os_str().is_empty()
                && speclink_core::security::input::is_excluded(&path, exclude_dirs)
            {
                skip(&mut result, label, "excluded");
                continue;
            }
            let opened = if path.as_os_str().is_empty() {
                self.handle.try_clone().map_err(anyhow::Error::from)
            } else {
                self.open_relative(&path)
            };
            let file = match opened {
                Ok(file) => file,
                Err(error) => {
                    skip(&mut result, label, error_code(&error));
                    continue;
                }
            };
            let metadata = match file.metadata() {
                Ok(m) => m,
                Err(_) => {
                    skip(&mut result, label, "io_error");
                    continue;
                }
            };
            if metadata.is_dir() {
                match directory_names(&file, &self.root.join(&path), &check) {
                    Ok(mut names) => {
                        entries += names.len();
                        if entries > 20_000 {
                            stop_scan(&mut result, "entry_limit", label, &pending);
                            break;
                        }
                        names.sort();
                        for name in names.into_iter().rev() {
                            pending.push(path.join(name));
                        }
                    }
                    Err(error) => {
                        if let Some(reason) = check() {
                            stop_scan(&mut result, reason, label, &pending);
                            break;
                        }
                        skip(&mut result, label, error_code(&error));
                    }
                }
                continue;
            }
            if !metadata.is_file() {
                skip(&mut result, label, "special_file");
                continue;
            }
            let language = Language::from_path(&label);
            if kind.is_some_and(|kind| match kind {
                InputKind::Code => language == Language::Markdown,
                InputKind::Api => !matches!(language, Language::Json | Language::Yaml),
                InputKind::Spec => language != Language::Markdown,
            }) {
                skip(&mut result, label, "review_not_selected");
                continue;
            }
            if files >= limits.files {
                stop_scan(&mut result, "file_limit", label, &pending);
                break;
            }
            files += 1;
            result.visited_files = files;
            if metadata.len() > limits.file_bytes as u64 {
                skip(&mut result, label, "file_too_large");
                continue;
            }
            if metadata.len()
                > limits
                    .total_bytes
                    .saturating_sub(result.total_bytes as usize) as u64
            {
                stop_scan(&mut result, "byte_limit", label, &pending);
                break;
            }
            let allowance = limits.file_bytes.min(
                limits
                    .total_bytes
                    .saturating_sub(result.total_bytes as usize),
            );
            let mut consumed = 0;
            let read = read_text_file(file, allowance, &check, &mut consumed);
            result.total_bytes += consumed as u64;
            if result.total_bytes > limits.total_bytes as u64 {
                stop_scan(&mut result, "byte_limit", label, &pending);
                break;
            }
            match read {
                Ok(text) => {
                    let prefix = &text[..text
                        .char_indices()
                        .nth(2048)
                        .map(|(i, _)| i)
                        .unwrap_or(text.len())];
                    if prefix.contains("@generated")
                        || prefix.contains("generatedBy: \"Speclink\"")
                        || prefix.contains("DO NOT EDIT")
                        || prefix.contains("AUTO-GENERATED")
                    {
                        skip(&mut result, label, "generated");
                        continue;
                    }
                    if Language::from_path(&label) == Language::Unsupported {
                        skip(&mut result, label, "unsupported_language");
                        continue;
                    }
                    // Reading is not a successful security check. Rules determine PASS later.
                    result.coverage.push(CoverageEntry {
                        path: label.clone(),
                        status: ReviewStatus::NotVerified,
                        reason: Some("awaiting_rules".into()),
                        bytes: text.len() as u64,
                    });
                    result.documents.push(SourceDocument::new(label, text));
                }
                Err(error) => {
                    if let Some(reason) = check() {
                        stop_scan(&mut result, reason, label, &pending);
                        break;
                    }
                    skip(&mut result, label, error_code(&error));
                }
            }
        }
        Ok(result)
    }
}

impl ProjectInput for AuthorizedProject {
    fn root(&self) -> &Path {
        &self.root
    }
    fn read_optional_text(
        &self,
        path: &str,
        limit: usize,
        stop: Stop<'_>,
    ) -> Result<Option<String>> {
        AuthorizedProject::read_optional_text(self, path, limit, stop)
    }
    fn directory_exists(&self, path: &str) -> Result<bool> {
        match self.open_relative(&relative_path(path)?) {
            Ok(file) => Ok(file.metadata()?.is_dir()),
            Err(error)
                if error
                    .downcast_ref::<std::io::Error>()
                    .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound) =>
            {
                Ok(false)
            }
            Err(error) => Err(error),
        }
    }
    fn scan_selected(
        &self,
        paths: &[String],
        kind: InputKind,
        exclude: &[String],
        limits: &ScanLimits,
        stop: Stop<'_>,
    ) -> Result<ScanInputs> {
        let roots = if paths.is_empty() {
            vec![PathBuf::new()]
        } else {
            paths
                .iter()
                .map(|p| relative_path(p))
                .collect::<Result<Vec<_>>>()?
        };
        self.scan_from(roots, Some(kind), exclude, limits, stop)
    }
}

fn relative_path(raw: &str) -> Result<PathBuf> {
    ensure!(
        !raw.is_empty() && !raw.contains(['\\', ':']) && !raw.chars().any(char::is_control),
        "unauthorized_path: expected project-relative path"
    );
    let path = Path::new(raw);
    ensure!(
        path.components().all(|c| matches!(c, Component::Normal(_)))
            && raw
                .split('/')
                .all(|c| !c.is_empty() && c != "." && c != ".."),
        "unauthorized_path"
    );
    Ok(path.to_path_buf())
}
fn display_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .chars()
        .filter(|c| !c.is_control())
        .collect()
}
fn skip(result: &mut ScanInputs, path: String, reason: &str) {
    if !matches!(reason, "excluded" | "generated" | "review_not_selected") {
        result.complete = false;
    }
    result.coverage.push(CoverageEntry {
        path,
        status: if matches!(reason, "excluded" | "generated" | "review_not_selected") {
            ReviewStatus::NotApplicable
        } else {
            ReviewStatus::NotVerified
        },
        reason: Some(reason.into()),
        bytes: 0,
    });
}
fn stop_scan(result: &mut ScanInputs, reason: &str, current: String, pending: &[PathBuf]) {
    result.complete = false;
    result.stop_reason = Some(reason.into());
    result.remaining_targets.push(current);
    result
        .remaining_targets
        .extend(pending.iter().map(|p| display_path(p)));
}
fn error_code(error: &anyhow::Error) -> &'static str {
    let message = error.to_string();
    for code in [
        "skipped_link",
        "special_file",
        "binary",
        "unsupported_encoding",
        "file_too_large",
        "entry_limit",
        "unsupported_platform",
    ] {
        if message.starts_with(code) {
            return code;
        }
    }
    "io_error"
}
fn read_text_file(
    mut file: File,
    limit: usize,
    stop: Stop<'_>,
    consumed: &mut usize,
) -> Result<String> {
    ensure!(file.metadata()?.is_file(), "special_file");
    ensure!(file.metadata()?.len() <= limit as u64, "file_too_large");
    let mut bytes = Vec::new();
    let mut chunk = [0; 64 * 1024];
    loop {
        if let Some(reason) = stop() {
            anyhow::bail!(reason);
        }
        let capacity = chunk
            .len()
            .min(limit.saturating_sub(*consumed).saturating_add(1));
        let count = file.read(&mut chunk[..capacity]).context("io_error")?;
        *consumed += count;
        if let Some(reason) = stop() {
            anyhow::bail!(reason);
        }
        if count == 0 {
            break;
        }
        ensure!(count <= limit.saturating_sub(bytes.len()), "file_too_large");
        ensure!(!chunk[..count].contains(&0), "binary");
        bytes.extend_from_slice(&chunk[..count]);
    }
    String::from_utf8(bytes).map_err(|_| anyhow::anyhow!("unsupported_encoding"))
}

#[cfg(unix)]
fn directory_names(file: &File, _path: &Path, stop: Stop<'_>) -> Result<Vec<std::ffi::OsString>> {
    use std::{
        ffi::{CStr, OsString},
        os::{fd::AsRawFd, unix::ffi::OsStringExt},
    };
    // Open a new description: dup would share the directory cursor, breaking a
    // second scan. fdopendir takes ownership of this new descriptor.
    let descriptor = unsafe {
        libc::openat(
            file.as_raw_fd(),
            c".".as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC,
        )
    };
    ensure!(descriptor >= 0, "io_error: open directory stream");
    let directory = unsafe { libc::fdopendir(descriptor) };
    if directory.is_null() {
        unsafe { libc::close(descriptor) };
        anyhow::bail!("io_error: directory stream");
    }
    struct Stream(*mut libc::DIR);
    impl Drop for Stream {
        fn drop(&mut self) {
            unsafe { libc::closedir(self.0) };
        }
    }
    let _stream = Stream(directory);
    let mut names = Vec::new();
    loop {
        if let Some(reason) = stop() {
            anyhow::bail!(reason);
        }
        #[cfg(target_os = "macos")]
        let errno = unsafe { libc::__error() };
        #[cfg(target_os = "linux")]
        let errno = unsafe { libc::__errno_location() };
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            anyhow::bail!("unsupported_platform: secure enumeration unavailable");
        }
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        unsafe {
            *errno = 0;
            let entry = libc::readdir(directory);
            if entry.is_null() {
                ensure!(*errno == 0, "io_error: read directory stream");
                break;
            }
            let name = CStr::from_ptr((*entry).d_name.as_ptr()).to_bytes();
            if name == b"." || name == b".." {
                continue;
            }
            ensure!(names.len() < 20_001, "entry_limit");
            names.push(OsString::from_vec(name.to_vec()));
        }
    }
    Ok(names)
}
#[cfg(windows)]
fn verify_windows_handle(file: &File, expected: &Path) -> Result<()> {
    use std::os::windows::{fs::MetadataExt, io::AsRawHandle};
    use windows_sys::Win32::Storage::FileSystem::{
        GetFinalPathNameByHandleW, FILE_NAME_NORMALIZED, VOLUME_NAME_DOS,
    };
    ensure!(
        file.metadata()?.file_attributes() & 0x400 == 0,
        "skipped_link"
    );
    let mut buffer = vec![0u16; 32_768];
    let count = unsafe {
        GetFinalPathNameByHandleW(
            file.as_raw_handle(),
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            FILE_NAME_NORMALIZED | VOLUME_NAME_DOS,
        )
    };
    ensure!(
        count > 0 && (count as usize) < buffer.len(),
        "unauthorized_path: cannot verify opened file"
    );
    let actual = String::from_utf16(&buffer[..count as usize])
        .map_err(|_| anyhow::anyhow!("unauthorized_path"))?;
    let normalize = |path: &str| {
        path.strip_prefix("\\\\?\\UNC\\")
            .map(|rest| format!("\\\\{rest}"))
            .unwrap_or_else(|| path.strip_prefix("\\\\?\\").unwrap_or(path).to_owned())
            .to_lowercase()
    };
    ensure!(
        normalize(&actual) == normalize(&expected.to_string_lossy()),
        "unauthorized_path: opened file identity changed"
    );
    Ok(())
}
#[cfg(windows)]
fn directory_names(file: &File, path: &Path, stop: Stop<'_>) -> Result<Vec<std::ffi::OsString>> {
    verify_windows_handle(file, path)?;
    let mut names = Vec::new();
    for entry in std::fs::read_dir(path)? {
        if let Some(reason) = stop() {
            anyhow::bail!(reason);
        }
        ensure!(names.len() < 20_001, "entry_limit");
        names.push(entry?.file_name());
    }
    verify_windows_handle(file, path)?;
    Ok(names)
}
#[cfg(not(any(unix, windows)))]
fn directory_names(_file: &File, _path: &Path, _stop: Stop<'_>) -> Result<Vec<std::ffi::OsString>> {
    anyhow::bail!("unsupported_platform")
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_core::security::input::ScanLimits;
    use std::{
        fs,
        path::PathBuf,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
    };
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!("speclink-scan-{}", ulid::Ulid::new()));
            fs::create_dir_all(&root).unwrap();
            Self(root)
        }
        fn write(&self, path: &str, bytes: impl AsRef<[u8]>) {
            let path = self.0.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn security_scan_exclusions_generated_binary_and_resource_boundaries() {
        let fixture = Fixture::new();
        for path in [
            "node_modules/pkg/index.js",
            "target/lib.rs",
            "fixtures/example.ts",
            ".speclink/context/design.md",
        ] {
            fixture.write(path, b"outside-scan-secret");
        }
        fixture.write("src/main.rs", b"fn main() {}\n");
        fixture.write(
            "src/generated.ts",
            b"// @generated\nconst value = 'never-read-as-source';",
        );
        fixture.write("asset.bin", [0, 1, 2, 3]);
        fixture.write("large.ts", vec![b'x'; 1_048_577]);
        fixture.write("unknown.py", b"print('unsupported')");
        fixture.write("deep/a/b/c/code.ts", b"const v=1;");
        let root = AuthorizedProject::open(&fixture.0).unwrap();
        let mut limits = ScanLimits::default();
        limits.depth = 3;
        let result = root.scan(&["fixtures".into()], &limits, &|| None).unwrap();
        assert_eq!(
            result
                .documents
                .iter()
                .map(|d| d.path.as_str())
                .collect::<Vec<_>>(),
            ["src/main.rs"]
        );
        for reason in [
            "excluded",
            "generated",
            "binary",
            "file_too_large",
            "unsupported_language",
            "depth_limit",
        ] {
            assert!(
                result
                    .coverage
                    .iter()
                    .any(|e| e.reason.as_deref() == Some(reason)),
                "{reason}"
            );
        }
        assert!(!result.complete);
        assert!(root
            .scan(&["../outside".into()], &limits, &|| None)
            .is_err());
        assert!(root.read_text("../outside", 1024, &|| None).is_err());
        let result = root
            .scan(
                &[],
                &ScanLimits {
                    files: 1,
                    ..Default::default()
                },
                &|| None,
            )
            .unwrap();
        assert_eq!(result.stop_reason.as_deref(), Some("file_limit"));
        assert!(!result.remaining_targets.is_empty());
        let result = root
            .scan(
                &[],
                &ScanLimits {
                    total_bytes: 1,
                    ..Default::default()
                },
                &|| None,
            )
            .unwrap();
        assert_eq!(result.stop_reason.as_deref(), Some("byte_limit"));
        let result = root.scan(&[], &limits, &|| Some("cancelled")).unwrap();
        assert_eq!(result.stop_reason.as_deref(), Some("cancelled"));
        assert!(result.documents.is_empty());
        assert_eq!(ScanLimits::default().files, 10_000);
        assert_eq!(ScanLimits::default().total_bytes, 104_857_600);
        assert_eq!(ScanLimits::default().depth, 32);
        assert_eq!(ScanLimits::default().seconds, 60);
    }
    #[cfg(unix)]
    #[test]
    fn security_scan_links_special_files_and_exchange_cannot_read_outside_root() {
        use std::{
            ffi::CString,
            os::unix::{ffi::OsStrExt, fs::symlink},
        };
        let fixture = Fixture::new();
        let outside = Fixture::new();
        outside.write(
            "secret.ts",
            b"const secret='root-external-secret-sentinel';",
        );
        fixture.write("real/safe.ts", b"const safe=1;");
        symlink(&outside.0, fixture.0.join("escape")).unwrap();
        symlink("real", fixture.0.join("internal")).unwrap();
        let fifo = CString::new(fixture.0.join("pipe.ts").as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
        let root = AuthorizedProject::open(&fixture.0).unwrap();
        assert!(root.read_text("escape/secret.ts", 1024, &|| None).is_err());
        let result = root.scan(&[], &ScanLimits::default(), &|| None).unwrap();
        assert!(result
            .coverage
            .iter()
            .any(|e| e.reason.as_deref() == Some("special_file")));
        assert!(result
            .coverage
            .iter()
            .filter(|e| e.path == "escape" || e.path == "internal")
            .all(|e| e.reason.as_deref() == Some("skipped_link")));
        assert!(!result.documents.iter().any(|d| d.text.contains("sentinel")));
        let done = Arc::new(AtomicBool::new(false));
        let signal = done.clone();
        let path = fixture.0.join("swapped");
        let external = outside.0.clone();
        let writer = std::thread::spawn(move || {
            while !signal.load(Ordering::Relaxed) {
                let _ = symlink(&external, &path);
                let _ = fs::remove_file(&path);
                let _ = fs::create_dir(&path);
                let _ = fs::write(path.join("secret.ts"), "const safe=1;");
                let _ = fs::remove_file(path.join("secret.ts"));
                let _ = fs::remove_dir(&path);
            }
        });
        for _ in 0..200 {
            if let Ok(text) = root.read_text("swapped/secret.ts", 1024, &|| None) {
                assert!(!text.contains("sentinel"));
            }
        }
        done.store(true, Ordering::Relaxed);
        writer.join().unwrap();
    }
    #[test]
    fn security_scan_exact_file_limit_and_actual_binary_byte_budget() {
        let fixture = Fixture::new();
        for index in 0..10_001 {
            fixture.write(&format!("{index:05}.ts"), b"const x=1;");
        }
        let root = AuthorizedProject::open(&fixture.0).unwrap();
        let result = root.scan(&[], &ScanLimits::default(), &|| None).unwrap();
        assert_eq!(result.documents.len(), 10_000);
        assert_eq!(result.remaining_targets, ["10000.ts"]);
        assert_eq!(result.stop_reason.as_deref(), Some("file_limit"));
        let binary = Fixture::new();
        binary.write("a.bin", [0u8; 8]);
        binary.write("b.bin", [0u8; 8]);
        let result = AuthorizedProject::open(&binary.0)
            .unwrap()
            .scan(
                &[],
                &ScanLimits {
                    total_bytes: 8,
                    ..Default::default()
                },
                &|| None,
            )
            .unwrap();
        assert_eq!(result.total_bytes, 8);
        assert_eq!(result.stop_reason.as_deref(), Some("byte_limit"));
        assert_eq!(result.coverage[0].reason.as_deref(), Some("binary"));
    }
    #[test]
    fn security_scan_failed_text_read_retains_consumed_bytes_without_echoing_data() {
        let fixture = Fixture::new();
        fixture.write(
            "proposal.md",
            [b'p', b'r', b'i', b'v', b'a', b't', b'e', 0xff],
        );
        let root = AuthorizedProject::open(&fixture.0).unwrap();
        let error = root.read_text("proposal.md", 1024, &|| None).unwrap_err();
        let failure = error
            .downcast_ref::<speclink_core::security::input::InputReadFailure>()
            .unwrap();
        assert_eq!(failure.bytes_read, 8);
        assert_eq!(failure.code, "unsupported_encoding");
        assert!(!error.to_string().contains("private"));
    }
    #[cfg(windows)]
    #[test]
    fn security_scan_windows_reparse_is_never_followed() {
        let fixture = Fixture::new();
        let outside = Fixture::new();
        outside.write("secret.ts", b"external");
        // Windows runner requires Developer Mode/symlink privilege for this test.
        std::os::windows::fs::symlink_dir(&outside.0, fixture.0.join("escape"))
            .expect("Windows symlink permission required");
        let root = AuthorizedProject::open(&fixture.0).unwrap();
        assert!(root.read_text("escape/secret.ts", 1024, &|| None).is_err());
    }
}

/// Export only masked bytes to an explicitly selected directory. Never overwrite.
pub(super) fn export_new_file(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    ensure!(path.is_absolute(), "invalid_path");
    let name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("invalid_path"))?;
    let parent = AuthorizedProject::open(
        path.parent()
            .ok_or_else(|| anyhow::anyhow!("invalid_path"))?,
    )?;
    #[cfg(unix)]
    {
        use std::{
            ffi::CString,
            os::unix::{
                ffi::OsStrExt,
                io::{AsRawFd, FromRawFd},
            },
        };
        let temp = CString::new(format!(".security-export-{}.tmp", ulid::Ulid::new()))?;
        let destination = CString::new(name.as_bytes())?;
        let fd = parent.handle.as_raw_fd();
        let raw = unsafe {
            libc::openat(
                fd,
                temp.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600,
            )
        };
        ensure!(raw >= 0, "export_failed");
        let mut file = unsafe { File::from_raw_fd(raw) };
        let result = (|| -> Result<()> {
            file.write_all(bytes)?;
            file.sync_all()?;
            if unsafe { libc::linkat(fd, temp.as_ptr(), fd, destination.as_ptr(), 0) } != 0 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    anyhow::bail!("file_exists")
                };
                return Err(error.into());
            }
            Ok(())
        })();
        unsafe { libc::unlinkat(fd, temp.as_ptr(), 0) };
        result?;
        // Publication already succeeded; a directory sync failure cannot undo it.
        let _ = parent.handle.sync_all();
        Ok(())
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        let destination = parent.root.join(name);
        let mut options = OpenOptions::new();
        options
            .write(true)
            .create_new(true)
            .share_mode(0)
            .custom_flags(windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT);
        let mut file = options.open(&destination).map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                anyhow::anyhow!("file_exists")
            } else {
                e.into()
            }
        })?;
        verify_windows_handle(&parent.handle, &parent.root)?;
        verify_windows_handle(&file, &destination)?;
        let result = file.write_all(bytes).and_then(|_| file.sync_all());
        drop(file);
        if result.is_err() {
            let _ = std::fs::remove_file(destination);
        }
        result?;
        Ok(())
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (name, parent, bytes);
        anyhow::bail!("unsupported_platform")
    }
}

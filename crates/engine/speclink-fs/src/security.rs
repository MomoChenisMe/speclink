//! Device-owned security data. Generations are immutable; only current.json changes.
pub mod knowledge;
pub mod prepared;
pub mod reports;
pub mod scan;
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use speclink_core::security::{
    data::{
        sha256, InstallReceipt, InstalledStandard, StandardInstall, StandardMetadata, StandardPin,
        StandardRepository, ValidatedStandard,
    },
    standards::{
        valid_data_name, StandardId, MAX_DATA_FILE_BYTES, MAX_PACKAGE_BYTES, MAX_PACKAGE_ENTRIES,
        MAX_XML_BYTES, PARSER_VERSION,
    },
};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct FsSecurityData {
    application_data: PathBuf,
    root: PathBuf,
    #[cfg(test)]
    fault: Option<CommitFault>,
}

#[cfg(test)]
#[derive(Clone, Copy, PartialEq)]
enum CommitFault {
    BeforePointer,
    AfterPointerCleanup,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Current {
    format_version: u8,
    generation: String,
    metadata_sha256: String,
}

impl FsSecurityData {
    pub fn new(application_data: &Path) -> Result<Self> {
        ensure!(
            application_data.is_absolute(),
            "data_directory_unavailable: absolute application data directory required"
        );
        Ok(Self {
            application_data: application_data.to_path_buf(),
            root: application_data.join("security/standards"),
            #[cfg(test)]
            fault: None,
        })
    }
    fn version_path(&self, id: StandardId, version: &str) -> Result<PathBuf> {
        ensure!(id.supports_version(version), "unsupported_version");
        Ok(self.root.join(id.as_str()).join(version))
    }
    fn read_generation(
        &self,
        id: StandardId,
        version: &str,
        export: bool,
    ) -> Result<
        Option<(
            InstalledStandard,
            std::collections::BTreeMap<String, Vec<u8>>,
        )>,
    > {
        let base = self.version_path(id, version)?;
        if !owned_directory(&base, &self.application_data, false)? {
            return Ok(None);
        }
        let pointer = base.join("current.json");
        if !pointer.try_exists()? {
            return Ok(None);
        }
        for _ in 0..3 {
            let pointer_bytes = read_limited(&pointer, 4096)?;
            let current: Current = serde_json::from_slice(&pointer_bytes)
                .context("invalid_package: current pointer")?;
            ensure!(
                current.format_version == 1 && valid_generation(&current.generation),
                "invalid_package: current generation"
            );
            let result: Result<
                Option<(
                    InstalledStandard,
                    std::collections::BTreeMap<String, Vec<u8>>,
                )>,
            > = (|| {
                let generation = base.join("generations").join(&current.generation);
                check_directory(&base.join("generations"))?;
                check_directory(&generation)?;
                // Old generations are retained for at least 24h. A reader pins its selected
                // generation while checking every file, so cleanup cannot remove it mid-read.
                let pin = open_regular(&generation.join(".pin"), false)?;
                match pin.try_lock_shared() {
                    Ok(()) => {}
                    Err(fs::TryLockError::WouldBlock) => {
                        anyhow::bail!("busy: selected generation is being cleaned")
                    }
                    Err(fs::TryLockError::Error(error)) => {
                        return Err(error).context("storage_error: generation pin")
                    }
                }
                let _pin = FileLock(pin);
                let metadata_bytes = read_limited(&generation.join("metadata.json"), 128 * 1024)?;
                ensure!(
                    sha256(&metadata_bytes) == current.metadata_sha256,
                    "integrity_error: metadata fingerprint"
                );
                let mut metadata: StandardMetadata =
                    serde_json::from_slice(&metadata_bytes).context("invalid_package: metadata")?;
                ensure!(
                    metadata.standard_id == id
                        && metadata.version == version
                        && metadata.generation == current.generation
                        && metadata.parser_version == PARSER_VERSION
                        && metadata.installed_status == "installed",
                    "invalid_package: metadata identity"
                );
                ensure!(
                    metadata.files.len() <= MAX_PACKAGE_ENTRIES
                        && metadata.files.contains_key("LICENSE"),
                    "invalid_package: metadata entries"
                );
                metadata.knowledge_status = speclink_core::security::knowledge::registry()?
                    .status_digests(&metadata.files)?
                    .into();
                let mut total = 0usize;
                let mut files = std::collections::BTreeMap::new();
                for (name, expected) in &metadata.files {
                    ensure!(
                        valid_data_name(name),
                        "invalid_package: unsafe metadata path"
                    );
                    if name.starts_with("raw/") {
                        check_directory(&generation.join("raw"))?;
                    }
                    let bytes = read_limited(
                        &generation.join(name),
                        if name.ends_with(".xml") {
                            MAX_XML_BYTES
                        } else {
                            MAX_DATA_FILE_BYTES
                        },
                    )?;
                    total += bytes.len();
                    ensure!(total <= MAX_PACKAGE_BYTES, "size_limit: installed package");
                    ensure!(
                        sha256(&bytes) == *expected,
                        "integrity_error: installed data fingerprint"
                    );
                    if export {
                        files.insert(name.clone(), bytes);
                    }
                }
                let parsed = read_limited(&generation.join("parsed.json"), MAX_PACKAGE_BYTES)?;
                ensure!(
                    sha256(&parsed) == metadata.parsed_sha256,
                    "integrity_error: parsed fingerprint"
                );
                let snapshot = serde_json::from_slice::<
                    speclink_core::security::standards::StandardSnapshot,
                >(&parsed)
                .context("invalid_package: parsed data")?;
                ensure!(
                    snapshot.standard_id == id && snapshot.version == version,
                    "invalid_package: parsed identity"
                );
                Ok(Some((InstalledStandard { metadata, snapshot }, files)))
            })();
            if result.is_ok() || read_limited(&pointer, 4096)? == pointer_bytes {
                return result;
            }
            // A stale generation may disappear between selecting current and pinning it.
            // Retry the new pointer, with a bound so concurrent changes cannot spin forever.
        }
        anyhow::bail!("busy: current generation keeps changing")
    }
}

impl StandardRepository for FsSecurityData {
    fn remember_policy(
        &self,
        key: &str,
        policy: &speclink_core::security::policy::SecurityPolicy,
    ) -> Result<()> {
        ensure!(!key.is_empty() && key.len() <= 4096, "invalid_project_key");
        let references = self.application_data.join("security/references");
        owned_directory(&references, &self.application_data, true)?;
        let _lock = references_lock(&references)?;
        let target = references.join(format!("{}.json", sha256(key.as_bytes())));
        let temporary = references.join(format!("{}.tmp", ulid::Ulid::new()));
        write_new(&temporary, &serde_json::to_vec(policy)?, &|| false)?;
        let result = atomic_replace(&temporary, &target);
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result?;
        sync_directory(&references)
    }

    fn pin(&self, id: StandardId, version: &str) -> Result<Box<dyn StandardPin>> {
        let base = self.version_path(id, version)?;
        ensure!(
            owned_directory(&base, &self.application_data, false)?,
            "missing_data"
        );
        let lock = open_lock(&base.join(".lock"))?;
        lock.try_lock_shared()
            .map_err(|_| anyhow::anyhow!("busy: standard version is being modified"))?;
        let lock = FileLock(lock);
        ensure!(base.join("current.json").try_exists()?, "missing_data");
        Ok(Box::new(lock))
    }

    fn remove(&self, id: StandardId, version: &str) -> Result<()> {
        let base = self.version_path(id, version)?;
        let references = self.application_data.join("security/references");
        owned_directory(&references, &self.application_data, true)?;
        let _references_lock = references_lock(&references)?;
        ensure!(
            !prepared::references_version(&self.application_data, id, version)?,
            "in_use: selected by a locally prepared snapshot"
        );
        let mut count = 0;
        for entry in fs::read_dir(&references)? {
            let path = entry?.path();
            if path.extension().is_some_and(|v| v == "json") {
                count += 1;
                ensure!(count <= 10_000, "size_limit: known policy references");
                let policy: speclink_core::security::policy::SecurityPolicy =
                    serde_json::from_slice(&read_limited(&path, 64 * 1024)?)
                        .context("invalid_reference: cannot safely check known policy")?;
                ensure!(
                    !policy.enabled
                        || !policy
                            .standards
                            .selected()
                            .iter()
                            .any(|(standard, selected, _)| *standard == id.as_str()
                                && *selected == version),
                    "in_use: selected by a locally known policy or prepared snapshot"
                );
            }
        }
        if !owned_directory(&base, &self.application_data, false)? {
            return Ok(());
        }
        let lock = open_lock(&base.join(".lock"))?;
        lock.try_lock()
            .map_err(|_| anyhow::anyhow!("busy: standard version is in use"))?;
        let _lock = FileLock(lock);
        let generations = base.join("generations");
        let mut pins = Vec::new();
        let mut paths = Vec::new();
        if generations.try_exists()? {
            check_directory(&generations)?;
            for entry in fs::read_dir(&generations)? {
                let entry = entry?;
                ensure!(
                    valid_generation(&entry.file_name().to_string_lossy()),
                    "invalid_package: unexpected generation"
                );
                check_directory(&entry.path())?;
                let pin = open_regular(&entry.path().join(".pin"), true)?;
                pin.try_lock()
                    .map_err(|_| anyhow::anyhow!("in_use: generation is being read"))?;
                pins.push(FileLock(pin));
                paths.push(entry.path());
                ensure!(paths.len() <= 10_000, "size_limit: generation count");
            }
        }
        let current = base.join("current.json");
        if current.try_exists()? {
            // Remove visibility first. A failed cleanup is safely retryable and
            // never leaves an incomplete generation advertised as installed.
            let _ = read_limited(&current, 4096)?;
            fs::remove_file(&current)?;
            sync_directory(&base)?;
        }
        for path in paths {
            fs::remove_dir_all(path)?;
        }
        drop(pins);
        sync_directory(&base)
    }
    fn import_package(&self, path: &Path, cancelled: &dyn Fn() -> bool) -> Result<InstallReceipt> {
        ensure!(!cancelled(), "cancelled");
        let bytes = read_limited(path, speclink_core::security::standards::MAX_DOWNLOAD_BYTES)?;
        let mut archive = safe_zip(&bytes, cancelled)?;
        let manifest = package_manifest(&mut archive, cancelled)?;
        let source = manifest.source()?;
        let transaction = self.begin_install(source.standard_id, &source.version)?;
        let standard = decode_package(archive, manifest, cancelled)?;
        transaction.commit_cancellable(standard, cancelled)
    }
    fn versions(&self) -> Result<Vec<(StandardId, String)>> {
        if !owned_directory(&self.root, &self.application_data, false)? {
            return Ok(Vec::new());
        }
        let mut versions = Vec::new();
        for id in [
            StandardId::OwaspTop10,
            StandardId::OwaspApiTop10,
            StandardId::OwaspAsvs,
            StandardId::Cwe,
        ] {
            let directory = self.root.join(id.as_str());
            if !directory.try_exists()? {
                continue;
            }
            check_directory(&directory)?;
            for entry in fs::read_dir(directory)? {
                let entry = entry?;
                let Some(version) = entry.file_name().to_str().map(str::to_owned) else {
                    continue;
                };
                if id.supports_version(&version) {
                    check_directory(&entry.path())?;
                    if entry.path().join("current.json").try_exists()? {
                        versions.push((id, version));
                    }
                    ensure!(
                        versions.len() <= 10_000,
                        "size_limit: installed version count"
                    );
                }
            }
        }
        versions.sort();
        Ok(versions)
    }

    fn read(&self, id: StandardId, version: &str) -> Result<Option<InstalledStandard>> {
        Ok(self
            .read_generation(id, version, false)?
            .map(|(installed, _)| installed))
    }
    fn read_export_source(
        &self,
        id: StandardId,
        version: &str,
    ) -> Result<Option<speclink_core::security::data::ExportSource>> {
        Ok(self
            .read_generation(id, version, true)?
            .map(
                |(installed, files)| speclink_core::security::data::ExportSource {
                    metadata: installed.metadata,
                    files,
                },
            ))
    }

    fn begin_install(&self, id: StandardId, version: &str) -> Result<Box<dyn StandardInstall>> {
        let base = self.version_path(id, version)?;
        owned_directory(&base, &self.application_data, true)?;
        let lock = open_lock(&base.join(".lock"))?;
        match lock.try_lock() {
            Ok(()) => {}
            Err(fs::TryLockError::WouldBlock) => {
                anyhow::bail!("busy: this standard version is in use")
            }
            Err(fs::TryLockError::Error(error)) => {
                return Err(error).context("storage_error: version lock")
            }
        }
        let lock = FileLock(lock);
        cleanup_stale(&base, "")?;
        Ok(Box::new(InstallTransaction {
            base,
            id,
            version: version.into(),
            _lock: lock,
            #[cfg(test)]
            fault: self.fault,
        }))
    }
}

pub(crate) struct FileLock(pub(crate) File);

impl Drop for FileLock {
    fn drop(&mut self) {
        // Closing alone can leave a Unix lock held by a forked child's inherited handle.
        let _ = self.0.unlock();
    }
}

impl StandardPin for FileLock {}

fn references_lock(references: &Path) -> Result<FileLock> {
    let lock = open_lock(&references.join(".lock"))?;
    lock.try_lock()
        .map_err(|_| anyhow::anyhow!("busy: known policy references are being updated"))?;
    Ok(FileLock(lock))
}

struct InstallTransaction {
    base: PathBuf,
    id: StandardId,
    version: String,
    _lock: FileLock,
    #[cfg(test)]
    fault: Option<CommitFault>,
}

impl StandardInstall for InstallTransaction {
    fn commit_cancellable(
        self: Box<Self>,
        standard: ValidatedStandard,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<InstallReceipt> {
        ensure!(
            standard.source().standard_id == self.id && standard.source().version == self.version,
            "version_mismatch: install transaction"
        );
        ensure!(!cancelled(), "cancelled");
        let generation = ulid::Ulid::new().to_string();
        let staging_parent = self.base.join("staging");
        let generations = self.base.join("generations");
        create_directories(&staging_parent)?;
        create_directories(&generations)?;
        let staging = staging_parent.join(&generation);
        fs::create_dir(&staging)?;
        let cleanup = StagingCleanup(staging.clone());
        fs::create_dir(staging.join("raw"))?;
        for (name, bytes) in standard.files() {
            write_new(&staging.join(name), bytes, cancelled)?;
        }
        let metadata = standard.metadata(generation.clone(), chrono::Utc::now().to_rfc3339())?;
        let metadata_bytes = serde_json::to_vec(&metadata)?;
        write_new(&staging.join("metadata.json"), &metadata_bytes, cancelled)?;
        write_new(
            &staging.join("parsed.json"),
            &serde_json::to_vec(standard.snapshot())?,
            cancelled,
        )?;
        write_new(&staging.join(".pin"), &[], cancelled)?;
        sync_directory(&staging.join("raw"))?;
        sync_directory(&staging)?;
        fs::rename(&staging, generations.join(&generation))?;
        sync_directory(&generations)?;
        drop(cleanup);
        let pointer = self.base.join(format!("current-{generation}.tmp"));
        let pointer_cleanup = StagingCleanup(pointer.clone());
        write_new(
            &pointer,
            &serde_json::to_vec(&Current {
                format_version: 1,
                generation: generation.clone(),
                metadata_sha256: sha256(&metadata_bytes),
            })?,
            cancelled,
        )?;
        #[cfg(test)]
        if self.fault == Some(CommitFault::BeforePointer) {
            anyhow::bail!("injected before pointer commit");
        }
        ensure!(!cancelled(), "cancelled");
        atomic_replace(&pointer, &self.base.join("current.json"))?;
        drop(pointer_cleanup);
        // The commit has happened. Subsequent failures are warnings, never a false failure.
        let mut warnings = Vec::new();
        if sync_directory(&self.base).is_err() {
            warnings.push("durability_warning: pointer committed but directory sync failed".into());
        }
        #[cfg(test)]
        if self.fault == Some(CommitFault::AfterPointerCleanup) {
            warnings
                .push("cleanup_warning: data installed; temporary cleanup can be retried".into());
        }
        if cleanup_stale(&self.base, &generation).is_err() {
            warnings
                .push("cleanup_warning: data installed; stale data cleanup can be retried".into());
        }
        Ok(InstallReceipt {
            knowledge_status: metadata.knowledge_status,
            standard_id: self.id,
            version: self.version.clone(),
            installed_status: "installed".into(),
            sha256: metadata.sha256,
            source_revision: metadata.source_revision,
            integrity_kind: metadata.integrity_kind,
            generation,
            warnings,
        })
    }
}

struct StagingCleanup(PathBuf);
impl Drop for StagingCleanup {
    fn drop(&mut self) {
        if self.0.is_dir() {
            let _ = fs::remove_dir_all(&self.0);
        } else {
            let _ = fs::remove_file(&self.0);
        }
    }
}

fn valid_generation(value: &str) -> bool {
    ulid::Ulid::from_string(value).is_ok_and(|id| id.to_string() == value)
}

pub(crate) fn check_directory(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    ensure!(
        metadata.is_dir() && !metadata.file_type().is_symlink(),
        "untrusted_link: data directory"
    );
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        ensure!(
            metadata.file_attributes() & 0x400 == 0,
            "untrusted_link: reparse data directory"
        );
    }
    Ok(())
}

pub(crate) fn owned_directory(path: &Path, root: &Path, create: bool) -> Result<bool> {
    if create {
        create_directories(root)?;
    } else if !root.try_exists()? {
        return Ok(false);
    }
    check_directory(root)?;
    let mut current = root.to_path_buf();
    for part in path.strip_prefix(root)?.components() {
        ensure!(
            matches!(part, std::path::Component::Normal(_)),
            "invalid_path: data directory"
        );
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                if !create {
                    return Ok(false);
                }
                match fs::create_dir(&current) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(e) => return Err(e.into()),
                }
            }
            Err(e) => return Err(e.into()),
        }
        check_directory(&current)?;
    }
    Ok(true)
}

fn create_directories(path: &Path) -> Result<()> {
    if path.try_exists()? {
        return check_directory(path);
    }
    if let Some(parent) = path.parent() {
        create_directories(parent)?;
    }
    match fs::create_dir(path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e.into()),
    }
    check_directory(path)
}

fn file_options(write: bool) -> OpenOptions {
    let mut options = OpenOptions::new();
    options.read(true).write(write);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
            .mode(0o600);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT);
    }
    options
}

fn regular(file: File) -> Result<File> {
    let metadata = file.metadata()?;
    ensure!(
        metadata.is_file(),
        "invalid_file: expected regular data file"
    );
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        ensure!(
            metadata.file_attributes() & 0x400 == 0,
            "untrusted_link: data file"
        );
    }
    Ok(file)
}
fn open_regular(path: &Path, write: bool) -> Result<File> {
    regular(file_options(write).open(path)?)
}
pub(crate) fn open_lock(path: &Path) -> Result<File> {
    regular(file_options(true).create(true).truncate(false).open(path)?)
}

pub(crate) fn read_limited(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let mut file = open_regular(path, false)?;
    ensure!(
        file.metadata()?.len() <= limit as u64,
        "size_limit: data file"
    );
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= limit, "size_limit: data file");
    Ok(bytes)
}

pub(crate) fn write_new(path: &Path, bytes: &[u8], cancelled: &dyn Fn() -> bool) -> Result<()> {
    let mut file = regular(file_options(true).create_new(true).open(path)?)?;
    for chunk in bytes.chunks(64 * 1024) {
        ensure!(!cancelled(), "cancelled");
        file.write_all(chunk)?;
    }
    file.sync_all()?;
    Ok(())
}

pub(crate) fn sync_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        File::open(path)?.sync_all()?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

pub(crate) fn atomic_replace(source: &Path, destination: &Path) -> Result<()> {
    // Rust's rename replaces an existing file on Windows as well as Unix.
    // ReplaceFileW can briefly remove the destination name during replacement,
    // making a concurrent inventory/read incorrectly report missing data.
    fs::rename(source, destination).context("storage_error: atomic pointer replace")
}

type DataZip<'a> = zip::ZipArchive<std::io::Cursor<&'a [u8]>>;

pub fn read_offline_package(
    bytes: &[u8],
    cancelled: &dyn Fn() -> bool,
) -> Result<ValidatedStandard> {
    let mut archive = safe_zip(bytes, cancelled)?;
    let manifest = package_manifest(&mut archive, cancelled)?;
    decode_package(archive, manifest, cancelled)
}

fn package_manifest(
    archive: &mut DataZip<'_>,
    cancelled: &dyn Fn() -> bool,
) -> Result<speclink_core::security::data::PackageManifest> {
    let bytes = zip_entry(archive, "manifest.json", 64 * 1024, cancelled)?;
    serde_json::from_slice(&bytes).context("invalid_package: manifest JSON")
}

fn decode_package(
    mut archive: DataZip<'_>,
    manifest: speclink_core::security::data::PackageManifest,
    cancelled: &dyn Fn() -> bool,
) -> Result<ValidatedStandard> {
    let source = manifest.source()?;
    let expected: std::collections::BTreeSet<_> = manifest
        .entries
        .iter()
        .map(|e| e.name.as_str())
        .chain(Some("manifest.json"))
        .collect();
    for i in 0..archive.len() {
        ensure!(!cancelled(), "cancelled");
        let file = archive.by_index(i).context("invalid_package: ZIP entry")?;
        if file.name() == "raw/" && file.is_dir() {
            continue;
        }
        ensure!(
            expected.contains(file.name()),
            "invalid_package: unlisted entry"
        );
    }
    let mut files = std::collections::BTreeMap::new();
    let mut total = 0usize;
    for entry in &manifest.entries {
        let bytes = zip_entry(
            &mut archive,
            &entry.name,
            if entry.name.ends_with(".xml") {
                MAX_XML_BYTES
            } else {
                MAX_DATA_FILE_BYTES
            },
            cancelled,
        )?;
        total += bytes.len();
        ensure!(total <= MAX_PACKAGE_BYTES, "size_limit: expanded package");
        ensure!(
            sha256(&bytes).eq_ignore_ascii_case(&entry.sha256),
            "integrity_error: package fingerprint"
        );
        files.insert(entry.name.clone(), bytes);
    }
    expand_cwe(&source, &mut files, cancelled)?;
    ValidatedStandard::new(
        source,
        files,
        speclink_core::security::data::IntegrityKind::Imported,
        cancelled,
    )
}

/// Shared by explicitly requested official CWE downloads and offline imports.
pub fn expand_cwe(
    source: &speclink_core::security::standards::StandardSource,
    files: &mut std::collections::BTreeMap<String, Vec<u8>>,
    cancelled: &dyn Fn() -> bool,
) -> Result<()> {
    if source.standard_id != StandardId::Cwe {
        return Ok(());
    }
    let xml = format!("cwec_v{}.xml", source.version);
    let key = format!("raw/{xml}");
    if let Some(bytes) = files.get(&format!("raw/{xml}.zip")) {
        let mut archive = checked_zip(bytes, Some(&xml), cancelled)?;
        ensure!(
            archive.len() == 1 && archive.by_index(0)?.name() == xml,
            "invalid_package: CWE archive must contain only the expected XML"
        );
        let decoded = zip_entry(&mut archive, &xml, MAX_XML_BYTES, cancelled)?;
        if let Some(existing) = files.get(&key) {
            ensure!(
                existing == &decoded,
                "integrity_error: CWE XML differs from its archive"
            );
        }
        files.insert(key, decoded);
    }
    Ok(())
}

pub fn decode_sources(
    source: speclink_core::security::standards::StandardSource,
    mut files: std::collections::BTreeMap<String, Vec<u8>>,
    integrity: speclink_core::security::data::IntegrityKind,
    cancelled: &dyn Fn() -> bool,
) -> Result<ValidatedStandard> {
    expand_cwe(&source, &mut files, cancelled)?;
    ValidatedStandard::new(source, files, integrity, cancelled)
}

fn zip_entry(
    archive: &mut DataZip<'_>,
    name: &str,
    limit: usize,
    cancelled: &dyn Fn() -> bool,
) -> Result<Vec<u8>> {
    let mut file = archive
        .by_name(name)
        .context("invalid_package: missing ZIP entry")?;
    ensure!(file.size() <= limit as u64, "size_limit: ZIP entry");
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 64 * 1024];
    loop {
        ensure!(!cancelled(), "cancelled");
        let count = file
            .read(&mut chunk)
            .context("invalid_package: ZIP decompression or CRC")?;
        if count == 0 {
            break;
        }
        ensure!(bytes.len() + count <= limit, "size_limit: actual ZIP entry");
        bytes.extend_from_slice(&chunk[..count]);
    }
    Ok(bytes)
}

fn safe_zip<'a>(bytes: &'a [u8], cancelled: &dyn Fn() -> bool) -> Result<DataZip<'a>> {
    checked_zip(bytes, None, cancelled)
}

fn checked_zip<'a>(
    bytes: &'a [u8],
    cwe_xml: Option<&str>,
    cancelled: &dyn Fn() -> bool,
) -> Result<DataZip<'a>> {
    use speclink_core::security::standards::MAX_DOWNLOAD_BYTES;
    ensure!(!cancelled(), "cancelled");
    ensure!(
        bytes.len() <= MAX_DOWNLOAD_BYTES,
        "size_limit: compressed package"
    );
    let entries = zip_preflight(bytes)?;
    let config = zip::read::Config {
        archive_offset: zip::read::ArchiveOffset::Known(0),
    };
    let mut archive = zip::ZipArchive::with_config(config, std::io::Cursor::new(bytes))
        .context("invalid_package: ZIP directory")?;
    ensure!(
        archive.len() == entries,
        "invalid_package: duplicate ZIP directory records"
    );
    ensure!(
        !archive
            .has_overlapping_files()
            .context("invalid_package: ZIP layout")?,
        "invalid_package: overlapping ZIP entries"
    );
    let mut names = std::collections::BTreeSet::new();
    let mut total = 0u64;
    for i in 0..archive.len() {
        ensure!(!cancelled(), "cancelled");
        let file = archive.by_index(i).context("invalid_package: ZIP entry")?;
        crate::security_packages::check_local_header(
            &mut std::io::Cursor::new(bytes),
            file.header_start(),
        )?;
        ensure!(
            file.extra_data()
                .is_none_or(|data| !crate::security_packages::has_zip64(data)),
            "unsupported_format: ZIP64 entry"
        );
        let name = std::str::from_utf8(file.name_raw())
            .context("invalid_package: ZIP file name must be UTF-8")?;
        ensure!(
            names.insert(name.to_ascii_lowercase()),
            "invalid_package: case-colliding ZIP entries"
        );
        ensure!(
            !file.encrypted() && !file.is_symlink(),
            "invalid_package: encrypted or linked ZIP entry"
        );
        ensure!(
            matches!(
                file.compression(),
                zip::CompressionMethod::Stored | zip::CompressionMethod::Deflated
            ),
            "unsupported_format: ZIP codec"
        );
        if file.is_dir() {
            ensure!(
                name == "raw/" && file.size() == 0,
                "invalid_package: unexpected ZIP directory"
            );
            continue;
        }
        ensure!(
            file.unix_mode()
                .is_none_or(|mode| (mode & 0o111 == 0 || cwe_xml == Some(name))
                    && matches!(mode & 0o170000, 0 | 0o100000)),
            "invalid_package: executable or special ZIP entry"
        );
        // Also allow the inner official CWE XML name, without extracting paths.
        ensure!(
            name == "manifest.json"
                || valid_data_name(name)
                || (name.starts_with("cwec_v")
                    && name.ends_with(".xml")
                    && !name.contains(['/', '\\', ':'])),
            "invalid_package: unsafe ZIP path"
        );
        let limit = if name.ends_with(".xml") {
            MAX_XML_BYTES
        } else if name == "manifest.json" {
            64 * 1024
        } else {
            MAX_DATA_FILE_BYTES
        };
        ensure!(
            file.size() <= limit as u64,
            "size_limit: expanded ZIP entry"
        );
        total = total
            .checked_add(file.size())
            .ok_or_else(|| anyhow::anyhow!("size_limit: expanded ZIP"))?;
        ensure!(
            total <= MAX_PACKAGE_BYTES as u64,
            "size_limit: expanded ZIP package"
        );
    }
    Ok(archive)
}

// Validate classic EOCD/counts before ZipArchive allocates its directory index.
// Packages are small and contain <=128 entries; ZIP64 and multi-disk are unnecessary.
fn zip_preflight(bytes: &[u8]) -> Result<usize> {
    ensure!(
        bytes.len() >= 22 && bytes.starts_with(b"PK\x03\x04"),
        "invalid_package: ZIP header"
    );
    let mut found = None;
    for offset in (bytes.len().saturating_sub(65_557)..=bytes.len() - 22).rev() {
        if bytes.get(offset..offset + 4) != Some(b"PK\x05\x06".as_slice()) {
            continue;
        }
        let u16_at =
            |n| u16::from_le_bytes(bytes[offset + n..offset + n + 2].try_into().unwrap()) as usize;
        let u32_at =
            |n| u32::from_le_bytes(bytes[offset + n..offset + n + 4].try_into().unwrap()) as usize;
        if offset + 22 + u16_at(20) != bytes.len() {
            continue;
        }
        ensure!(
            u16_at(4) == 0 && u16_at(6) == 0 && u16_at(8) == u16_at(10),
            "invalid_package: multi-disk ZIP"
        );
        let count = u16_at(10);
        ensure!(
            count > 0 && count <= MAX_PACKAGE_ENTRIES,
            "size_limit: ZIP entry count"
        );
        let start = u32_at(16);
        let size = u32_at(12);
        ensure!(
            start <= offset && size <= offset - start,
            "invalid_package: ZIP directory bounds"
        );
        ensure!(
            offset < 20 || bytes.get(offset - 20..offset - 16) != Some(b"PK\x06\x07".as_slice()),
            "unsupported_format: ZIP64 package"
        );
        found = Some(count);
    }
    found.ok_or_else(|| anyhow::anyhow!("invalid_package: missing ZIP end record"))
}

fn cleanup_stale(base: &Path, current: &str) -> Result<()> {
    for directory in ["staging", "generations"] {
        if directory == "generations" && current.is_empty() {
            continue;
        }
        let entries = match fs::read_dir(base.join(directory)) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        };
        for entry in entries {
            let entry = entry?;
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if name == current || !valid_generation(&name) {
                continue;
            }
            let metadata = fs::symlink_metadata(entry.path())?;
            if !metadata.is_dir() || metadata.file_type().is_symlink() {
                continue;
            }
            if metadata.modified()?.elapsed().unwrap_or_default().as_secs() <= 24 * 60 * 60 {
                continue;
            }
            // A live reader pins a generation. Staging is protected by the version lock.
            let pin = if directory == "generations" {
                let pin = open_regular(&entry.path().join(".pin"), true)?;
                if pin.try_lock().is_err() {
                    continue;
                }
                Some(FileLock(pin))
            } else {
                None
            };
            fs::remove_dir_all(entry.path())?;
            drop(pin);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_core::security::data::{IntegrityKind, StandardRepository, ValidatedStandard};
    use speclink_core::security::standards::{standard_source, StandardId};
    use std::collections::BTreeMap;

    #[test]
    #[cfg(unix)]
    fn security_references_lock_release_survives_inherited_handle() {
        let dir = TestDirectory::new();
        let references = dir.0.join("security/references");
        fs::create_dir_all(&references).unwrap();
        let lock = references_lock(&references).unwrap();
        let inherited = lock.0.try_clone().unwrap();
        assert!(references_lock(&references).is_err());
        drop(lock);
        let next = references_lock(&references).unwrap();
        drop(next);
        drop(inherited);
    }

    #[test]
    #[cfg(unix)]
    fn security_install_lock_release_survives_inherited_handle() {
        let dir = TestDirectory::new();
        let repository = FsSecurityData::new(&dir.0).unwrap();
        let base = repository
            .version_path(StandardId::OwaspAsvs, "5.0.0")
            .unwrap();
        fs::create_dir_all(&base).unwrap();
        let lock = open_lock(&base.join(".lock")).unwrap();
        lock.try_lock().unwrap();
        let inherited = lock.try_clone().unwrap();
        let transaction = InstallTransaction {
            base,
            id: StandardId::OwaspAsvs,
            version: "5.0.0".into(),
            _lock: FileLock(lock),
            fault: None,
        };
        assert!(repository
            .begin_install(StandardId::OwaspAsvs, "5.0.0")
            .is_err());
        drop(transaction);
        let next = repository
            .begin_install(StandardId::OwaspAsvs, "5.0.0")
            .unwrap();
        drop(next);
        drop(inherited);
    }

    #[test]
    #[cfg(unix)]
    fn security_generation_pin_release_survives_inherited_handle() {
        let dir = TestDirectory::new();
        let file = open_lock(&dir.0.join(".pin")).unwrap();
        file.try_lock_shared().unwrap();
        let inherited = file.try_clone().unwrap();
        let pin = FileLock(file);
        let writer = open_lock(&dir.0.join(".pin")).unwrap();
        assert!(writer.try_lock().is_err());
        drop(pin);
        writer.try_lock().unwrap();
        writer.unlock().unwrap();
        drop(inherited);
    }

    #[test]
    #[ignore = "requires explicit official fixture download; no network in this test"]
    fn security_data_official_fixtures_install_and_read_offline() {
        let root = std::env::var_os("SPECLINK_SECURITY_OFFICIAL_FIXTURES")
            .expect("set the official fixture directory");
        let dir = TestDirectory::new();
        let repository = FsSecurityData::new(&dir.0).unwrap();
        let sources = speclink_core::security::standards::source_catalog().unwrap();
        for source in &sources {
            let input = Path::new(&root).join(format!(
                "{}_{}",
                source.standard_id.as_str(),
                source.version
            ));
            let mut files = source
                .files
                .iter()
                .map(|f| {
                    (
                        f.name.clone(),
                        fs::read(input.join(f.name.strip_prefix("raw/").unwrap_or(&f.name)))
                            .unwrap(),
                    )
                })
                .collect();
            expand_cwe(source, &mut files, &|| false).unwrap();
            let kind = if source.files.iter().all(|f| f.sha256.is_some()) {
                IntegrityKind::PinnedDigest
            } else {
                IntegrityKind::SourceTls
            };
            let standard = ValidatedStandard::new(source.clone(), files, kind, &|| false).unwrap();
            let receipt = repository
                .begin_install(source.standard_id, &source.version)
                .unwrap()
                .commit(standard)
                .unwrap();
            let loaded = repository
                .read(source.standard_id, &source.version)
                .unwrap()
                .unwrap();
            assert_eq!(loaded.metadata.sha256, receipt.sha256);
            assert_eq!(loaded.metadata.license_name, source.license_name);
            assert!(!loaded.snapshot.items.is_empty());
        }
        assert_eq!(repository.versions().unwrap().len(), sources.len());
    }

    #[test]
    fn security_remove_holds_pins_and_references_without_modifying_policies() {
        let dir = TestDirectory::new();
        let repository = FsSecurityData::new(&dir.0).unwrap();
        let (manifest, files) = offline_fixture();
        let package = dir.0.join("fixture.zip");
        fs::write(&package, fixture_zip(&manifest, &files, None)).unwrap();
        repository.import_package(&package, &|| false).unwrap();
        let pin = repository.pin(StandardId::Cwe, "4.20").unwrap();
        assert!(repository
            .remove(StandardId::Cwe, "4.20")
            .unwrap_err()
            .to_string()
            .contains("busy"));
        assert!(repository.read(StandardId::Cwe, "4.20").unwrap().is_some());
        drop(pin);
        let policy = serde_json::from_value(serde_json::json!({"enabled":true,"standards":{"cwe":{"enabled":true,"version":"4.20"}}})).unwrap();
        repository.remember_policy("project-key", &policy).unwrap();
        let reference = dir
            .0
            .join("security/references")
            .join(format!("{}.json", sha256(b"project-key")));
        let original = fs::read(&reference).unwrap();
        assert!(repository
            .remove(StandardId::Cwe, "4.20")
            .unwrap_err()
            .to_string()
            .contains("in_use"));
        assert_eq!(fs::read(&reference).unwrap(), original);
        repository
            .remember_policy("project-key", &Default::default())
            .unwrap();
        repository.remove(StandardId::Cwe, "4.20").unwrap();
        assert!(repository.read(StandardId::Cwe, "4.20").unwrap().is_none());
        assert!(repository.versions().unwrap().is_empty());
    }

    fn offline_fixture() -> (serde_json::Value, Vec<(String, Vec<u8>)>) {
        let source = standard_source(StandardId::Cwe, "4.20").unwrap();
        let files = vec![
            ("raw/cwec_v4.20.xml".into(), br#"<Weakness_Catalog xmlns="http://cwe.mitre.org/cwe-7" Version="4.20"><Weaknesses><Weakness ID="79" Name="Synthetic"><Description>Synthetic fixture</Description></Weakness></Weaknesses></Weakness_Catalog>"#.to_vec()),
            ("LICENSE".into(), b"Synthetic fixture: MITRE copyright and CWE Terms of Use are retained by real packages.".to_vec()),
        ];
        let manifest = serde_json::json!({
            "formatVersion":1,"standardId":source.standard_id,"version":source.version,"sourceRevision":source.source_revision,
            "sourceUrl":source.source_url,"sourceFormat":source.source_format,"licenseName":source.license_name,
            "licenseUrl":source.license_url,"attribution":source.attribution,
            "entries":files.iter().map(|(name, bytes)| serde_json::json!({"name":name,"sha256":sha256(bytes)})).collect::<Vec<_>>()
        });
        (manifest, files)
    }

    fn fixture_zip(
        manifest: &serde_json::Value,
        files: &[(String, Vec<u8>)],
        special: Option<(&str, bool)>,
    ) -> Vec<u8> {
        use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};
        let mut zip = ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o644);
        zip.start_file("manifest.json", options).unwrap();
        zip.write_all(&serde_json::to_vec(manifest).unwrap())
            .unwrap();
        for (name, bytes) in files {
            zip.start_file(name, options).unwrap();
            zip.write_all(bytes).unwrap();
        }
        if let Some((name, symlink)) = special {
            if symlink {
                zip.add_symlink(name, "/must-not-read", options).unwrap();
            } else {
                zip.start_file(name, options.unix_permissions(0o755))
                    .unwrap();
                zip.write_all(b"do not execute").unwrap();
            }
        }
        zip.finish().unwrap().into_inner()
    }

    #[test]
    fn export_source_retains_same_generation_raw_and_license() {
        let (manifest, files) = offline_fixture();
        let dir = TestDirectory::new();
        let repo = FsSecurityData::new(&dir.0).unwrap();
        let path = dir.0.join("source.zip");
        fs::write(&path, fixture_zip(&manifest, &files, None)).unwrap();
        let installed = repo.import_package(&path, &|| false).unwrap();
        let exported = repo
            .read_export_source(StandardId::Cwe, "4.20")
            .unwrap()
            .unwrap();
        assert_eq!(exported.metadata.generation, installed.generation);
        assert_eq!(exported.files, files.into_iter().collect());
        let json = serde_json::to_value(speclink_core::security::data::ExportReceipt {
            path: "a.zip".into(),
            format_version: 1,
            bytes: 42,
            packages: vec![],
            skipped: vec![],
        })
        .unwrap();
        assert_eq!(json["formatVersion"], 1);
        assert!(json["bytes"].is_u64());
    }

    #[test]
    fn security_import_valid_manifest_is_offline_and_never_promotes_trust() {
        let (manifest, files) = offline_fixture();
        let bytes = fixture_zip(&manifest, &files, None);
        let parsed = read_offline_package(&bytes, &|| false).unwrap();
        assert_eq!(parsed.snapshot().version, "4.20");
        assert_eq!(parsed.snapshot().items[0].id, "CWE-79");
        let dir = TestDirectory::new();
        let path = dir.0.join("package.zip");
        fs::write(&path, bytes).unwrap();
        let repository = FsSecurityData::new(&dir.0).unwrap();
        let receipt = repository.import_package(&path, &|| false).unwrap();
        assert_eq!(receipt.integrity_kind, IntegrityKind::Imported);
        assert_eq!(
            repository
                .read(StandardId::Cwe, "4.20")
                .unwrap()
                .unwrap()
                .snapshot,
            *parsed.snapshot()
        );
    }

    #[test]
    fn security_import_rejects_invalid_manifest_fingerprint_version_license_and_keeps_old() {
        let (manifest, files) = offline_fixture();
        let dir = TestDirectory::new();
        let repository = FsSecurityData::new(&dir.0).unwrap();
        let path = dir.0.join("package.zip");
        fs::write(&path, fixture_zip(&manifest, &files, None)).unwrap();
        let first = repository.import_package(&path, &|| false).unwrap();
        for field in [
            "formatVersion",
            "version",
            "sourceRevision",
            "licenseName",
            "licenseUrl",
            "sourceUrl",
            "entries",
        ] {
            let mut bad = manifest.clone();
            bad[field] = if field == "formatVersion" {
                serde_json::json!(2)
            } else {
                serde_json::json!(null)
            };
            fs::write(&path, fixture_zip(&bad, &files, None)).unwrap();
            assert!(
                repository.import_package(&path, &|| false).is_err(),
                "field {field}"
            );
            assert_eq!(
                repository
                    .read(StandardId::Cwe, "4.20")
                    .unwrap()
                    .unwrap()
                    .metadata
                    .generation,
                first.generation
            );
        }
        let mut bad = manifest.clone();
        bad["entries"][0]["sha256"] = serde_json::json!("0".repeat(64));
        assert!(read_offline_package(&fixture_zip(&bad, &files, None), &|| false).is_err());
        let mut wrong = files.clone();
        wrong[0].1 = String::from_utf8(wrong[0].1.clone())
            .unwrap()
            .replace("Version=\"4.20\"", "Version=\"4.19\"")
            .into_bytes();
        let mut matching_hash = manifest.clone();
        matching_hash["entries"][0]["sha256"] = serde_json::json!(sha256(&wrong[0].1));
        assert!(
            read_offline_package(&fixture_zip(&matching_hash, &wrong, None), &|| false)
                .err()
                .unwrap()
                .to_string()
                .contains("version_mismatch")
        );
    }

    #[test]
    fn security_import_rejects_paths_links_executable_unlisted_and_case_collisions() {
        let (manifest, files) = offline_fixture();
        for path in [
            "../outside.xml",
            "/absolute.xml",
            "C:/secret.xml",
            "raw/../../outside.xml",
            "raw\\secret.xml",
            "raw/./secret.xml",
            "raw//secret.xml",
            "raw/CON.xml",
            "raw/secret.xml.",
        ] {
            let mut bad_files = files.clone();
            bad_files[0].0 = path.into();
            let mut bad = manifest.clone();
            bad["entries"][0]["name"] = serde_json::json!(path);
            assert!(
                read_offline_package(&fixture_zip(&bad, &bad_files, None), &|| false).is_err(),
                "{path}"
            );
        }
        for special in [("raw/link.xml", true), ("raw/tool.json", false)] {
            assert!(
                read_offline_package(&fixture_zip(&manifest, &files, Some(special)), &|| false)
                    .is_err()
            );
        }
        let mut extra = files.clone();
        extra.push(("raw/extra.json".into(), b"{}".to_vec()));
        assert!(read_offline_package(&fixture_zip(&manifest, &extra, None), &|| false).is_err());
        extra.push(("license".into(), b"ambiguous license".to_vec()));
        assert!(read_offline_package(&fixture_zip(&manifest, &extra, None), &|| false).is_err());
    }

    #[test]
    fn security_import_rejects_broken_zip_duplicate_directory_records_bombs_and_cancel() {
        let (manifest, mut files) = offline_fixture();
        assert!(read_offline_package(b"not ZIP", &|| false).is_err());
        let valid = fixture_zip(&manifest, &files, None);
        assert!(read_offline_package(&valid[..valid.len() - 10], &|| false).is_err());
        assert!(read_offline_package(&valid, &|| true)
            .err()
            .unwrap()
            .to_string()
            .contains("cancelled"));
        let mut duplicate = fixture_zip(
            &manifest,
            &[
                ("raw/a.xml".into(), b"a".to_vec()),
                ("raw/b.xml".into(), b"b".to_vec()),
            ],
            None,
        );
        let needle = b"raw/b.xml";
        for index in 0..duplicate.len() - needle.len() {
            if &duplicate[index..index + needle.len()] == needle {
                duplicate[index + 4] = b'a';
            }
        }
        assert!(read_offline_package(&duplicate, &|| false).is_err());
        files[1].1 = vec![b'x'; MAX_DATA_FILE_BYTES + 1];
        let bomb = fixture_zip(&manifest, &files, None);
        assert!(bomb.len() < 64 * 1024);
        assert!(read_offline_package(&bomb, &|| false)
            .err()
            .unwrap()
            .to_string()
            .contains("size_limit"));
    }

    #[test]
    fn security_import_rejects_dtd_and_header_entry_count_before_index_allocation() {
        let (mut manifest, mut files) = offline_fixture();
        let mut xml = b"<!DOCTYPE x [<!ENTITY external SYSTEM 'file:///must-not-read'>]>".to_vec();
        xml.extend_from_slice(&files[0].1);
        files[0].1 = xml;
        manifest["entries"][0]["sha256"] = serde_json::json!(sha256(&files[0].1));
        assert!(
            read_offline_package(&fixture_zip(&manifest, &files, None), &|| false)
                .err()
                .unwrap()
                .to_string()
                .contains("DTD")
        );
        let (manifest, files) = offline_fixture();
        let mut bad = fixture_zip(&manifest, &files, None);
        let end = bad.len() - 22;
        bad[end + 8..end + 10].copy_from_slice(&129u16.to_le_bytes());
        bad[end + 10..end + 12].copy_from_slice(&129u16.to_le_bytes());
        assert!(read_offline_package(&bad, &|| false)
            .err()
            .unwrap()
            .to_string()
            .contains("size_limit"));
        // The imported source is explicitly untrusted; caller-supplied digests
        // check package consistency and never confer an official signature.
    }

    #[test]
    fn security_import_cwe_inner_permissions_are_not_outer_package_permissions() {
        let (manifest, mut files) = offline_fixture();
        let source = standard_source(StandardId::Cwe, "4.20").unwrap();
        let options = zip::write::SimpleFileOptions::default()
            .unix_permissions(0o750)
            .compression_method(zip::CompressionMethod::Deflated);
        let make_inner = |name: &str, link: bool| {
            let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
            if link {
                zip.add_symlink(name, "/must-not-read", options).unwrap();
            } else {
                zip.start_file(name, options).unwrap();
                zip.write_all(&files[0].1).unwrap();
            }
            zip.finish().unwrap().into_inner()
        };
        for (name, link) in [
            ("cwec_v4.20.xml", false),
            ("other.xml", false),
            ("cwec_v4.20.xml", true),
        ] {
            let bytes = make_inner(name, link);
            let mut raw = BTreeMap::from([("raw/cwec_v4.20.xml.zip".into(), bytes)]);
            let result = expand_cwe(&source, &mut raw, &|| false);
            assert_eq!(result.is_ok(), name == "cwec_v4.20.xml" && !link);
        }
        // Exactly the same executable permission on an outer XML entry is refused.
        let xml = files.remove(0);
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        zip.start_file("manifest.json", options.unix_permissions(0o644))
            .unwrap();
        zip.write_all(&serde_json::to_vec(&manifest).unwrap())
            .unwrap();
        zip.start_file(xml.0, options).unwrap();
        zip.write_all(&xml.1).unwrap();
        let bytes = zip.finish().unwrap().into_inner();
        assert!(read_offline_package(&bytes, &|| false)
            .err()
            .unwrap()
            .to_string()
            .contains("executable"));
    }

    struct TestDirectory(std::path::PathBuf);
    impl TestDirectory {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("speclink-security-data-{}", ulid::Ulid::new()));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn standard(label: &str) -> ValidatedStandard {
        let mut source = standard_source(StandardId::OwaspAsvs, "5.0.0").unwrap();
        for file in &mut source.files {
            file.sha256 = None;
        }
        let name = source
            .files
            .iter()
            .find(|f| f.name.ends_with(".json"))
            .unwrap()
            .name
            .clone();
        let files = BTreeMap::from([
            (name, serde_json::to_vec(&serde_json::json!({"Version":"5.0.0","Requirements":[{"Shortcode":"V1.1.1","Description":label,"L":"1"}]})).unwrap()),
            ("LICENSE".into(), b"Synthetic licensed fixture; not official data".to_vec()),
        ]);
        ValidatedStandard::new(source, files, IntegrityKind::Imported, &|| false).unwrap()
    }

    #[test]
    fn security_data_atomic_generation_metadata_and_failed_replacement() {
        let dir = TestDirectory::new();
        let repository = FsSecurityData::new(&dir.0).unwrap();
        assert!(FsSecurityData::new(std::path::Path::new("relative")).is_err());
        assert!(repository
            .read(StandardId::OwaspAsvs, "5.0.0")
            .unwrap()
            .is_none());
        let first = repository
            .begin_install(StandardId::OwaspAsvs, "5.0.0")
            .unwrap()
            .commit(standard("first"))
            .unwrap();
        let loaded = repository
            .read(StandardId::OwaspAsvs, "5.0.0")
            .unwrap()
            .unwrap();
        assert_eq!(loaded.metadata.generation, first.generation);
        assert_eq!(loaded.metadata.integrity_kind, IntegrityKind::Imported);
        assert_eq!(loaded.metadata.license_name, "CC-BY-SA-4.0");
        assert_eq!(loaded.snapshot.items[0].description, "first");
        assert_eq!(loaded.metadata.sha256.len(), 64);
        assert!(!loaded.metadata.downloaded_at.is_empty());
        let mut broken = repository.clone();
        broken.fault = Some(CommitFault::BeforePointer);
        assert!(broken
            .begin_install(StandardId::OwaspAsvs, "5.0.0")
            .unwrap()
            .commit(standard("second"))
            .is_err());
        assert_eq!(
            repository
                .read(StandardId::OwaspAsvs, "5.0.0")
                .unwrap()
                .unwrap(),
            loaded
        );
        broken.fault = Some(CommitFault::AfterPointerCleanup);
        let installed = broken
            .begin_install(StandardId::OwaspAsvs, "5.0.0")
            .unwrap()
            .commit(standard("third"))
            .unwrap();
        assert!(!installed.warnings.is_empty());
        assert_eq!(
            repository
                .read(StandardId::OwaspAsvs, "5.0.0")
                .unwrap()
                .unwrap()
                .snapshot
                .items[0]
                .description,
            "third"
        );
        assert_eq!(
            repository.versions().unwrap(),
            [(StandardId::OwaspAsvs, "5.0.0".into())]
        );
    }

    #[test]
    fn security_data_cross_process_lock_and_crash_release() {
        let dir = TestDirectory::new();
        let repository = FsSecurityData::new(&dir.0).unwrap();
        let held = repository
            .begin_install(StandardId::OwaspAsvs, "5.0.0")
            .unwrap();
        let run = |mode| {
            std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "security::tests::security_data_lock_child",
                    "--nocapture",
                ])
                .env("SPECLINK_SECURITY_LOCK_PROBE", &dir.0)
                .env("SPECLINK_SECURITY_LOCK_MODE", mode)
                .output()
                .unwrap()
        };
        assert!(run("busy").status.success());
        drop(held);
        assert!(run("crash").status.success());
        assert!(repository
            .begin_install(StandardId::OwaspAsvs, "5.0.0")
            .is_ok());
    }

    #[test]
    fn security_data_lock_child() {
        let Some(root) = std::env::var_os("SPECLINK_SECURITY_LOCK_PROBE") else {
            return;
        };
        let repository = FsSecurityData::new(std::path::Path::new(&root)).unwrap();
        if std::env::var("SPECLINK_SECURITY_LOCK_MODE").unwrap() == "busy" {
            assert!(repository
                .begin_install(StandardId::OwaspAsvs, "5.0.0")
                .err()
                .unwrap()
                .to_string()
                .contains("busy"));
        } else {
            let _held = repository
                .begin_install(StandardId::OwaspAsvs, "5.0.0")
                .unwrap();
            std::process::exit(0); // Model a crash: the OS, not Drop, must release the lock.
        }
    }

    #[test]
    fn security_data_pointer_replace_preserves_open_reader_and_failed_update() {
        let dir = TestDirectory::new();
        let current = dir.0.join("current.json");
        let next = dir.0.join("next.json");
        write_new(&current, b"old", &|| false).unwrap();
        write_new(&next, b"new", &|| false).unwrap();
        let mut reader = open_regular(&current, false).unwrap();
        atomic_replace(&next, &current).unwrap();
        assert_eq!(read_limited(&current, 4096).unwrap(), b"new");
        let mut old = Vec::new();
        reader.read_to_end(&mut old).unwrap();
        assert_eq!(old, b"old");
        assert!(!next.exists());
        assert!(atomic_replace(&next, &current).is_err());
        assert_eq!(read_limited(&current, 4096).unwrap(), b"new");
    }

    #[test]
    fn security_data_read_during_replacement_never_mixes_generations() {
        let dir = TestDirectory::new();
        let repository = FsSecurityData::new(&dir.0).unwrap();
        repository
            .begin_install(StandardId::OwaspAsvs, "5.0.0")
            .unwrap()
            .commit(standard("first"))
            .unwrap();
        std::thread::scope(|scope| {
            scope.spawn(|| {
                for _ in 0..20 {
                    repository
                        .begin_install(StandardId::OwaspAsvs, "5.0.0")
                        .unwrap()
                        .commit(standard("second"))
                        .unwrap();
                }
            });
            for _ in 0..100 {
                let value = repository
                    .read(StandardId::OwaspAsvs, "5.0.0")
                    .unwrap()
                    .expect("an installed version must stay visible during replacement");
                assert!(matches!(
                    value.snapshot.items[0].description.as_str(),
                    "first" | "second"
                ));
                assert_eq!(value.metadata.standard_id, value.snapshot.standard_id);
                assert_eq!(value.metadata.version, value.snapshot.version);
            }
        });
    }

    #[test]
    fn security_data_cancellation_before_commit_keeps_old_current() {
        let dir = TestDirectory::new();
        let repository = FsSecurityData::new(&dir.0).unwrap();
        let first = repository
            .begin_install(StandardId::OwaspAsvs, "5.0.0")
            .unwrap()
            .commit(standard("first"))
            .unwrap();
        let calls = std::cell::Cell::new(0);
        let error = repository
            .begin_install(StandardId::OwaspAsvs, "5.0.0")
            .unwrap()
            .commit_cancellable(standard("second"), &|| {
                calls.set(calls.get() + 1);
                calls.get() >= 4
            })
            .unwrap_err();
        assert!(error.to_string().contains("cancelled"));
        let loaded = repository
            .read(StandardId::OwaspAsvs, "5.0.0")
            .unwrap()
            .unwrap();
        assert_eq!(loaded.metadata.generation, first.generation);
        assert_eq!(loaded.snapshot.items[0].description, "first");
    }

    #[cfg(unix)]
    #[test]
    fn security_data_rejects_linked_storage_ancestors() {
        let dir = TestDirectory::new();
        let outside = TestDirectory::new();
        std::fs::create_dir_all(outside.0.join("standards/owasp_asvs/5.0.0")).unwrap();
        std::os::unix::fs::symlink(&outside.0, dir.0.join("security")).unwrap();
        let repository = FsSecurityData::new(&dir.0).unwrap();
        assert!(repository
            .begin_install(StandardId::OwaspAsvs, "5.0.0")
            .is_err());
        assert!(repository.versions().is_err());
        assert!(!outside.0.join("standards/owasp_asvs/5.0.0/.lock").exists());
    }

    #[test]
    #[cfg(unix)]
    fn security_data_cleanup_respects_live_generation_pins_and_integrity() {
        let dir = TestDirectory::new();
        let repository = FsSecurityData::new(&dir.0).unwrap();
        let first = repository
            .begin_install(StandardId::OwaspAsvs, "5.0.0")
            .unwrap()
            .commit(standard("first"))
            .unwrap();
        let base = repository
            .version_path(StandardId::OwaspAsvs, "5.0.0")
            .unwrap();
        let generation = base.join("generations").join(&first.generation);
        let pin = open_regular(&generation.join(".pin"), false).unwrap();
        pin.lock_shared().unwrap();
        // A concurrent subprocess can inherit this handle until exec closes it.
        let inherited_pin = pin.try_clone().unwrap();
        let yesterday = std::time::SystemTime::now() - std::time::Duration::from_secs(90_000);
        std::fs::File::open(&generation)
            .unwrap()
            .set_modified(yesterday)
            .unwrap();
        let stale = base.join("staging").join(ulid::Ulid::new().to_string());
        std::fs::create_dir(&stale).unwrap();
        std::fs::File::open(&stale)
            .unwrap()
            .set_modified(yesterday)
            .unwrap();
        let second = repository
            .begin_install(StandardId::OwaspAsvs, "5.0.0")
            .unwrap()
            .commit(standard("second"))
            .unwrap();
        assert!(
            generation.exists(),
            "live reader generation stays available"
        );
        assert!(
            !stale.exists(),
            "crashed old staging is cleaned under the version lock"
        );
        // Explicit unlock also releases the lock held through inherited handles.
        pin.unlock().unwrap();
        drop(pin);
        cleanup_stale(&base, &second.generation).unwrap();
        assert!(!generation.exists());
        drop(inherited_pin);
        std::fs::write(
            base.join("generations")
                .join(second.generation)
                .join("parsed.json"),
            "{}",
        )
        .unwrap();
        assert!(repository
            .read(StandardId::OwaspAsvs, "5.0.0")
            .unwrap_err()
            .to_string()
            .contains("integrity_error"));
    }
}

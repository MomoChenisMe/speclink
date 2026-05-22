//! `LocalArtifactStore` — filesystem-backed artifact I/O 與 sha256 並發控制。
//!
//! Etag 採 `sha256(file bytes)`，artifact 不進 state.db。寫入採 tempfile-in-same-dir + rename。

#![allow(clippy::doc_markdown)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use speclink_provider::{
    ArtifactKind, ArtifactStore, Etag, ExpectedEtag, ProviderError, Versioned, validate_kebab_id,
};

use crate::paths::{change_dir, specs_dir};
use crate::state_db::StateDb;

/// LocalProvider 的 `ArtifactStore` 實作。
pub struct LocalArtifactStore {
    working_dir: PathBuf,
    state_root: PathBuf,
}

impl LocalArtifactStore {
    /// 建立 store handle；不接觸磁碟。
    #[must_use]
    pub fn new(working_dir: PathBuf, state_root: PathBuf) -> Self {
        Self {
            working_dir,
            state_root,
        }
    }

    /// Working tree root 路徑。
    #[must_use]
    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }

    /// State root 路徑。
    #[must_use]
    pub fn state_root(&self) -> &Path {
        &self.state_root
    }

    fn open_db(&self) -> Result<StateDb, ProviderError> {
        fs::create_dir_all(&self.state_root)
            .map_err(|e| ProviderError::Internal(format!("create state root: {e}")))?;
        let path = self.state_root.join("state.db");
        let db = StateDb::open(&path)
            .map_err(|e| ProviderError::Internal(format!("open state.db: {e}")))?;
        db.migrate(2)
            .map_err(|e| ProviderError::Internal(format!("migrate state.db: {e}")))?;
        Ok(db)
    }

    fn require_change_exists(&self, name: &str) -> Result<(), ProviderError> {
        let db = self.open_db()?;
        if db
            .get_change_by_name(name)
            .map_err(|e| ProviderError::Internal(format!("query change row: {e}")))?
            .is_none()
        {
            return Err(ProviderError::ChangeNotFound {
                name: name.to_string(),
            });
        }
        Ok(())
    }
}

/// 計算 artifact 在檔案系統上的絕對路徑。
///
/// 規則：
/// - `proposal` → `<change_dir>/proposal.md`
/// - `design`   → `<change_dir>/design.md`
/// - `tasks`    → `<change_dir>/tasks.md`
/// - `spec`     → `<change_dir>/specs/<capability>/spec.md`（`capability` 必填且需通過 grammar）
///
/// 非 `spec` 的 `capability` 引數在路徑上會被忽略，但仍會驗證 grammar（不合法時回
/// `ArtifactKindInvalid`）。
///
/// # Errors
/// - `ArtifactCapabilityRequired`：`kind=spec` 但 `capability` 為 `None`
/// - `ArtifactKindInvalid`：`capability` 不符合 `validate_kebab_id`
pub fn resolve_path(
    working_dir: &Path,
    change_name: &str,
    kind: ArtifactKind,
    capability: Option<&str>,
) -> Result<PathBuf, ProviderError> {
    // 若帶 capability，先驗 grammar（不論 kind）
    if let Some(cap) = capability {
        validate_kebab_id(cap).map_err(|_| ProviderError::ArtifactKindInvalid {
            kind: format!("invalid capability id `{cap}`"),
        })?;
    }
    match kind {
        ArtifactKind::Proposal => Ok(change_dir(working_dir, change_name).join("proposal.md")),
        ArtifactKind::Design => Ok(change_dir(working_dir, change_name).join("design.md")),
        ArtifactKind::Tasks => Ok(change_dir(working_dir, change_name).join("tasks.md")),
        ArtifactKind::Spec => {
            let cap = capability.ok_or(ProviderError::ArtifactCapabilityRequired)?;
            Ok(specs_dir(working_dir, change_name)
                .join(cap)
                .join("spec.md"))
        }
    }
}

/// `kind=spec` 以外的 kind 若傳了 `--capability`，視為應該忽略。回傳是否該發 warning。
#[must_use]
pub fn capability_ignored(kind: ArtifactKind, capability: Option<&str>) -> bool {
    capability.is_some() && !kind.requires_capability()
}

/// 對 bytes 計算 sha256 Etag。
#[must_use]
pub fn compute_etag(bytes: &[u8]) -> Etag {
    Etag::from_bytes(bytes)
}

/// Atomic write: tempfile in same parent dir + `persist` (rename)。
///
/// 若 parent dir 不存在會先建立。寫入失敗時 tempfile 自動清理（RAII）。
///
/// # Errors
/// 任何 IO 失敗回 [`ProviderError::Internal`]。
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), ProviderError> {
    let parent = path.parent().ok_or_else(|| {
        ProviderError::Internal(format!("artifact path has no parent: {}", path.display()))
    })?;
    fs::create_dir_all(parent).map_err(|e| {
        ProviderError::Internal(format!("create artifact parent {}: {e}", parent.display()))
    })?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent)
        .map_err(|e| ProviderError::Internal(format!("create tempfile: {e}")))?;
    tmp.write_all(bytes)
        .map_err(|e| ProviderError::Internal(format!("write tempfile: {e}")))?;
    tmp.as_file()
        .sync_all()
        .map_err(|e| ProviderError::Internal(format!("fsync tempfile: {e}")))?;
    tmp.persist(path).map_err(|e| {
        ProviderError::Internal(format!("rename tempfile to {}: {e}", path.display()))
    })?;
    Ok(())
}

#[async_trait]
impl ArtifactStore for LocalArtifactStore {
    async fn read_artifact(
        &self,
        change: &str,
        kind: ArtifactKind,
        capability: Option<&str>,
    ) -> Result<Versioned<Vec<u8>>, ProviderError> {
        self.require_change_exists(change)?;
        let path = resolve_path(&self.working_dir, change, kind, capability)?;
        match fs::read(&path) {
            Ok(bytes) => {
                let etag = compute_etag(&bytes);
                Ok(Versioned { value: bytes, etag })
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Err(ProviderError::ArtifactNotFound {
                    path: path.display().to_string(),
                })
            }
            Err(e) => Err(ProviderError::Internal(format!(
                "read artifact {}: {e}",
                path.display()
            ))),
        }
    }

    async fn write_artifact(
        &self,
        change: &str,
        kind: ArtifactKind,
        capability: Option<&str>,
        bytes: &[u8],
        expected: ExpectedEtag,
    ) -> Result<Versioned<()>, ProviderError> {
        self.require_change_exists(change)?;
        let path = resolve_path(&self.working_dir, change, kind, capability)?;

        // 並發矩陣：根據檔案是否存在 + expected 是否為 Some 分四種情形
        match (fs::read(&path), &expected) {
            // 1. 檔案存在
            (Ok(current), ExpectedEtag::None) => {
                // 覆寫缺 etag → 一律 conflict
                Err(ProviderError::ArtifactVersionConflict {
                    expected: None,
                    actual: compute_etag(&current),
                })
            }
            (Ok(current), ExpectedEtag::Some(expected_etag)) => {
                let actual = compute_etag(&current);
                if actual != *expected_etag {
                    return Err(ProviderError::ArtifactVersionConflict {
                        expected: Some(expected_etag.clone()),
                        actual,
                    });
                }
                atomic_write(&path, bytes)?;
                Ok(Versioned {
                    value: (),
                    etag: compute_etag(bytes),
                })
            }
            // 2. 檔案不存在
            (Err(e), expected) if e.kind() == std::io::ErrorKind::NotFound => match expected {
                ExpectedEtag::None => {
                    atomic_write(&path, bytes)?;
                    Ok(Versioned {
                        value: (),
                        etag: compute_etag(bytes),
                    })
                }
                ExpectedEtag::Some(_) => Err(ProviderError::ArtifactNotFound {
                    path: path.display().to_string(),
                }),
            },
            // 3. 其他 IO 錯誤
            (Err(e), _) => Err(ProviderError::Internal(format!(
                "stat artifact {}: {e}",
                path.display()
            ))),
        }
    }

    async fn list_spec_capabilities(&self, change: &str) -> Result<Vec<String>, ProviderError> {
        self.require_change_exists(change)?;
        let dir = specs_dir(&self.working_dir, change);
        let entries = match fs::read_dir(&dir) {
            Ok(it) => it,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => {
                return Err(ProviderError::Internal(format!(
                    "read_dir {}: {e}",
                    dir.display()
                )));
            }
        };
        let mut out = Vec::new();
        for entry in entries {
            let entry =
                entry.map_err(|e| ProviderError::Internal(format!("read_dir entry: {e}")))?;
            if !entry
                .file_type()
                .map_err(|e| ProviderError::Internal(format!("entry file_type: {e}")))?
                .is_dir()
            {
                continue;
            }
            let name = match entry.file_name().into_string() {
                Ok(s) => s,
                Err(_) => continue, // 跳過非 UTF-8 檔名
            };
            if entry.path().join("spec.md").is_file() {
                out.push(name);
            }
        }
        out.sort();
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change_store::LocalChangeStore;
    use speclink_provider::ChangeStore;
    use tempfile::TempDir;

    fn fixture(tmp: &TempDir) -> (LocalChangeStore, LocalArtifactStore) {
        let working = tmp.path().to_path_buf();
        let state = working.join(".git").join("speclink");
        std::fs::create_dir_all(&state).expect("state dir");
        (
            LocalChangeStore::new(working.clone(), state.clone()),
            LocalArtifactStore::new(working, state),
        )
    }

    async fn seed_change(cs: &LocalChangeStore, name: &str) {
        cs.create_change(name, "spec-driven").await.expect("seed");
    }

    // --- §4.1 path resolution -------------------------------------------------

    #[test]
    fn resolve_path_for_each_kind() {
        let work = std::path::Path::new("/tmp/work");
        let p = resolve_path(work, "foo", ArtifactKind::Proposal, None).expect("proposal");
        assert!(p.ends_with(".speclink/changes/foo/proposal.md"));
        let d = resolve_path(work, "foo", ArtifactKind::Design, None).expect("design");
        assert!(d.ends_with(".speclink/changes/foo/design.md"));
        let t = resolve_path(work, "foo", ArtifactKind::Tasks, None).expect("tasks");
        assert!(t.ends_with(".speclink/changes/foo/tasks.md"));
        let s = resolve_path(work, "foo", ArtifactKind::Spec, Some("user-auth")).expect("spec");
        assert!(s.ends_with(".speclink/changes/foo/specs/user-auth/spec.md"));
    }

    #[test]
    fn resolve_path_spec_without_capability_errors() {
        let work = std::path::Path::new("/tmp/work");
        let err = resolve_path(work, "foo", ArtifactKind::Spec, None)
            .expect_err("spec without capability");
        assert!(matches!(err, ProviderError::ArtifactCapabilityRequired));
    }

    #[test]
    fn resolve_path_invalid_capability_errors() {
        let work = std::path::Path::new("/tmp/work");
        let err = resolve_path(work, "foo", ArtifactKind::Spec, Some("User_Auth"))
            .expect_err("invalid capability");
        assert!(matches!(err, ProviderError::ArtifactKindInvalid { .. }));
    }

    #[test]
    fn capability_ignored_helper() {
        assert!(capability_ignored(ArtifactKind::Proposal, Some("x")));
        assert!(!capability_ignored(ArtifactKind::Proposal, None));
        assert!(!capability_ignored(ArtifactKind::Spec, Some("x")));
    }

    // --- §4.2 etag round-trip -------------------------------------------------

    #[tokio::test]
    async fn write_then_read_returns_same_bytes_and_etag() {
        let tmp = TempDir::new().expect("tempdir");
        let (cs, ast) = fixture(&tmp);
        seed_change(&cs, "foo").await;
        let body = b"## Why\n\nfirst write\n";
        let written = ast
            .write_artifact(
                "foo",
                ArtifactKind::Proposal,
                None,
                body,
                ExpectedEtag::None,
            )
            .await
            .expect("write");
        let read = ast
            .read_artifact("foo", ArtifactKind::Proposal, None)
            .await
            .expect("read");
        assert_eq!(read.value, body);
        assert_eq!(read.etag, written.etag);
        assert_eq!(read.etag, Etag::from_bytes(body));
    }

    // --- §4.3 concurrency matrix ---------------------------------------------

    #[tokio::test]
    async fn write_new_without_etag_ok() {
        let tmp = TempDir::new().expect("tempdir");
        let (cs, ast) = fixture(&tmp);
        seed_change(&cs, "foo").await;
        let res = ast
            .write_artifact(
                "foo",
                ArtifactKind::Proposal,
                None,
                b"x",
                ExpectedEtag::None,
            )
            .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn write_new_with_etag_not_found() {
        let tmp = TempDir::new().expect("tempdir");
        let (cs, ast) = fixture(&tmp);
        seed_change(&cs, "foo").await;
        let phantom = Etag::from_bytes(b"phantom");
        let err = ast
            .write_artifact(
                "foo",
                ArtifactKind::Proposal,
                None,
                b"x",
                ExpectedEtag::Some(phantom),
            )
            .await
            .expect_err("non-null etag on new file");
        assert!(matches!(err, ProviderError::ArtifactNotFound { .. }));
        // File must not exist
        let p = tmp.path().join(".speclink/changes/foo/proposal.md");
        assert!(!p.exists());
    }

    #[tokio::test]
    async fn write_existing_without_etag_conflict() {
        let tmp = TempDir::new().expect("tempdir");
        let (cs, ast) = fixture(&tmp);
        seed_change(&cs, "foo").await;
        ast.write_artifact(
            "foo",
            ArtifactKind::Proposal,
            None,
            b"B0",
            ExpectedEtag::None,
        )
        .await
        .expect("first write");
        let err = ast
            .write_artifact(
                "foo",
                ArtifactKind::Proposal,
                None,
                b"B1",
                ExpectedEtag::None,
            )
            .await
            .expect_err("overwrite without etag");
        assert!(matches!(err, ProviderError::ArtifactVersionConflict { .. }));
        let read = ast
            .read_artifact("foo", ArtifactKind::Proposal, None)
            .await
            .expect("read");
        assert_eq!(read.value, b"B0", "file must not be modified");
    }

    #[tokio::test]
    async fn write_existing_matching_etag_ok() {
        let tmp = TempDir::new().expect("tempdir");
        let (cs, ast) = fixture(&tmp);
        seed_change(&cs, "foo").await;
        let v0 = ast
            .write_artifact(
                "foo",
                ArtifactKind::Proposal,
                None,
                b"B0",
                ExpectedEtag::None,
            )
            .await
            .expect("first write");
        ast.write_artifact(
            "foo",
            ArtifactKind::Proposal,
            None,
            b"B1",
            ExpectedEtag::Some(v0.etag),
        )
        .await
        .expect("overwrite with matching etag");
        let read = ast
            .read_artifact("foo", ArtifactKind::Proposal, None)
            .await
            .expect("read");
        assert_eq!(read.value, b"B1");
    }

    #[tokio::test]
    async fn write_existing_mismatching_etag_conflict() {
        let tmp = TempDir::new().expect("tempdir");
        let (cs, ast) = fixture(&tmp);
        seed_change(&cs, "foo").await;
        ast.write_artifact(
            "foo",
            ArtifactKind::Proposal,
            None,
            b"B0",
            ExpectedEtag::None,
        )
        .await
        .expect("first write");
        let wrong = Etag::from_bytes(b"some-other-content");
        let err = ast
            .write_artifact(
                "foo",
                ArtifactKind::Proposal,
                None,
                b"B1",
                ExpectedEtag::Some(wrong),
            )
            .await
            .expect_err("mismatching etag");
        assert!(matches!(err, ProviderError::ArtifactVersionConflict { .. }));
        let read = ast
            .read_artifact("foo", ArtifactKind::Proposal, None)
            .await
            .expect("read");
        assert_eq!(read.value, b"B0", "file must not be modified on conflict");
    }

    // --- §4.4 atomic write + parent dir ---------------------------------------

    #[tokio::test]
    async fn write_spec_creates_capability_parent_dir() {
        let tmp = TempDir::new().expect("tempdir");
        let (cs, ast) = fixture(&tmp);
        seed_change(&cs, "foo").await;
        let cap_dir = tmp.path().join(".speclink/changes/foo/specs/user-auth");
        assert!(!cap_dir.exists());
        ast.write_artifact(
            "foo",
            ArtifactKind::Spec,
            Some("user-auth"),
            b"# spec\n",
            ExpectedEtag::None,
        )
        .await
        .expect("write spec");
        assert!(cap_dir.join("spec.md").exists());
    }

    #[test]
    fn atomic_write_creates_no_tempfile_residue_on_success() {
        let tmp = TempDir::new().expect("tempdir");
        let dir = tmp.path().join(".speclink/changes/foo");
        std::fs::create_dir_all(&dir).expect("dir");
        let target = dir.join("proposal.md");
        atomic_write(&target, b"hello").expect("write");
        assert_eq!(std::fs::read(&target).expect("read"), b"hello");
        // No tempfile siblings should remain
        let siblings: Vec<_> = std::fs::read_dir(&dir)
            .expect("readdir")
            .filter_map(Result::ok)
            .filter(|e| e.path() != target)
            .collect();
        assert!(
            siblings.is_empty(),
            "no temp residue should remain, got: {siblings:?}"
        );
    }

    // --- §4.5 list_spec_capabilities ------------------------------------------

    #[tokio::test]
    async fn list_spec_capabilities_empty_when_no_specs_dir() {
        let tmp = TempDir::new().expect("tempdir");
        let (cs, ast) = fixture(&tmp);
        seed_change(&cs, "foo").await;
        assert!(
            ast.list_spec_capabilities("foo")
                .await
                .expect("list")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn list_spec_capabilities_sorted_and_filtered() {
        let tmp = TempDir::new().expect("tempdir");
        let (cs, ast) = fixture(&tmp);
        seed_change(&cs, "foo").await;
        // Write 2 spec capabilities + 1 incomplete subdir without spec.md.
        ast.write_artifact(
            "foo",
            ArtifactKind::Spec,
            Some("user-auth"),
            b"# spec\n",
            ExpectedEtag::None,
        )
        .await
        .expect("user-auth");
        ast.write_artifact(
            "foo",
            ArtifactKind::Spec,
            Some("rate-limiting"),
            b"# spec\n",
            ExpectedEtag::None,
        )
        .await
        .expect("rate-limiting");
        // Incomplete subdir (no spec.md): create dir but no file.
        std::fs::create_dir_all(tmp.path().join(".speclink/changes/foo/specs/incomplete"))
            .expect("incomplete dir");
        let caps = ast.list_spec_capabilities("foo").await.expect("list");
        assert_eq!(caps, vec!["rate-limiting", "user-auth"]);
    }

    #[tokio::test]
    async fn list_spec_capabilities_unknown_change_returns_not_found() {
        let tmp = TempDir::new().expect("tempdir");
        let (_cs, ast) = fixture(&tmp);
        let err = ast
            .list_spec_capabilities("orphan")
            .await
            .expect_err("missing change");
        assert!(matches!(err, ProviderError::ChangeNotFound { .. }));
    }

    // --- existence check precedes filesystem ---------------------------------

    #[tokio::test]
    async fn read_artifact_unknown_change_returns_not_found_before_fs() {
        let tmp = TempDir::new().expect("tempdir");
        let (_cs, ast) = fixture(&tmp);
        // Create a stray dir to simulate "filesystem present but no change row"
        std::fs::create_dir_all(tmp.path().join(".speclink/changes/orphan")).expect("orphan");
        std::fs::write(
            tmp.path().join(".speclink/changes/orphan/proposal.md"),
            b"orphan",
        )
        .expect("write orphan");
        let err = ast
            .read_artifact("orphan", ArtifactKind::Proposal, None)
            .await
            .expect_err("orphan change");
        assert!(matches!(err, ProviderError::ChangeNotFound { .. }));
    }

    #[tokio::test]
    async fn read_artifact_missing_file_returns_not_found() {
        let tmp = TempDir::new().expect("tempdir");
        let (cs, ast) = fixture(&tmp);
        seed_change(&cs, "foo").await;
        let err = ast
            .read_artifact("foo", ArtifactKind::Proposal, None)
            .await
            .expect_err("file missing");
        assert!(matches!(err, ProviderError::ArtifactNotFound { .. }));
    }
}

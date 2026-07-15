//! Agent Context Projection materializer (platform architecture §7).
//!
//! Remote canon, local read-only snapshot: the materializer writes one
//! consistent [`ContextSnapshot`] into `<workspace>/.speclink/context/` —
//! manifest.json (snapshot id, policy revision, per-file digests), INDEX.md
//! and the openspec mirror — so agents Read/Search/Grep files instead of
//! per-document API round trips. The projection is disposable (delete and
//! rebuild any time), always gitignored, and never a second writable canon:
//! direct edits are detected by digest verification and fail closed. Local
//! fs mode never materializes anything.
//!
//! The snapshot source is injected as a [`SnapshotProvider`] — tests use a
//! local-tree double; the real HTTP Context API arrives with the Phase 2
//! server.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use speclink_core::workspace::{StoreMode, Workspace};
use speclink_protocol::context::{ContextDocument, ContextSnapshot, ContextSnapshotRequest};
use std::path::{Component, Path, PathBuf};

/// Where a context snapshot comes from. Input and output are the protocol's
/// Context DTOs (the wire shapes fixed by the protocol-typed-client knife) —
/// a provider narrows nothing; flow selection is the materializer's job.
pub trait SnapshotProvider {
    fn snapshot(&self, request: &ContextSnapshotRequest) -> Result<ContextSnapshot>;
}

/// The digest form snapshot documents and the manifest agree on — re-exported
/// so providers compute the same digests the materializer verifies.
pub use speclink_store::content_digest;

/// The manifest.json shape (camelCase, mirroring the protocol
/// [`ContextSnapshot`] fields): snapshot id, policy revision when present,
/// and one entry per projected file — INDEX.md included, so every readable
/// file in the projection is digest-covered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionManifest {
    pub snapshot_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_revision: Option<u64>,
    pub files: Vec<ManifestFile>,
}

/// One projection file inside [`ProjectionManifest`]; `path` is relative to
/// the projection root, slash-separated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestFile {
    pub path: String,
    pub digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<u64>,
}

/// A successful materialization: the manifest that was written, plus
/// warnings the caller (CLI) surfaces on stderr.
#[derive(Debug)]
pub struct Materialized {
    pub manifest: ProjectionManifest,
    pub warnings: Vec<String>,
}

/// The projection root: `<workspace>/.speclink/context/`.
pub fn projection_dir(ws: &Workspace) -> PathBuf {
    ws.work_dir().join("context")
}

/// The staging directory a snapshot is fully produced in before the atomic
/// switch; a leftover from a failed switch is kept for retry and cleared by
/// the next materialize.
fn staging_dir(ws: &Workspace) -> PathBuf {
    ws.work_dir().join("context.staging")
}

/// Best-effort read-only attribute (blueprint §7.2): integrity is judged by
/// digest verification, never by whether this attribute stuck.
fn set_readonly_best_effort(path: &Path) {
    if let Ok(md) = std::fs::metadata(path) {
        let mut perm = md.permissions();
        perm.set_readonly(true);
        let _ = std::fs::set_permissions(path, perm);
    }
}

/// `remove_dir_all` that first clears the best-effort read-only attribute —
/// Windows refuses to delete read-only files.
fn remove_tree(dir: &Path) -> std::io::Result<()> {
    for p in speclink_core::util::walk_files(dir) {
        if let Ok(md) = std::fs::metadata(&p) {
            let mut perm = md.permissions();
            if perm.readonly() {
                #[allow(clippy::permissions_set_readonly_false)]
                perm.set_readonly(false);
                let _ = std::fs::set_permissions(&p, perm);
            }
        }
    }
    std::fs::remove_dir_all(dir)
}

/// A document path is written under the projection root verbatim — reject
/// anything that could escape it (absolute, `..`, empty). The provider is an
/// external input boundary; a hostile snapshot must not write outside the
/// projection.
fn safe_rel_path(path: &str) -> Result<PathBuf> {
    let p = Path::new(path);
    let ok = !path.is_empty()
        && p.components().all(|c| matches!(c, Component::Normal(_)));
    if !ok {
        bail!("snapshot document path escapes the projection: {path}");
    }
    Ok(p.to_path_buf())
}

/// The projection's own files are reserved at the document-ingest boundary:
/// a snapshot document by these names would be silently overwritten
/// (manifest.json, INDEX.md) or would mark the projection stale at birth
/// (STALE).
fn reject_reserved_document(path: &str) -> Result<()> {
    if matches!(path, "manifest.json" | "INDEX.md" | STALE_MARKER) {
        bail!("snapshot document path collides with a reserved projection file: {path}");
    }
    Ok(())
}

/// What a snapshot document is, judged structurally from its path — the
/// first component is the spec root, whatever the server names it.
enum DocKind<'a> {
    Config,
    Language,
    CanonicalSpec { capability: &'a str },
    ChangeDoc { change: &'a str, file: &'a str },
    DeltaSpec { change: &'a str, capability: &'a str },
    Discussion,
    Schema,
    Other,
}

fn classify(path: &str) -> DocKind<'_> {
    let Some((_root, rest)) = path.split_once('/') else {
        return DocKind::Other;
    };
    match rest {
        "config.yaml" => return DocKind::Config,
        "LANGUAGE.md" => return DocKind::Language,
        _ => {}
    }
    let comps: Vec<&str> = rest.split('/').collect();
    match comps.as_slice() {
        ["specs", cap, ..] => DocKind::CanonicalSpec { capability: cap },
        ["changes", change, "specs", cap, ..] => DocKind::DeltaSpec { change, capability: cap },
        ["changes", change, file] => DocKind::ChangeDoc { change, file },
        ["discussions", ..] => DocKind::Discussion,
        ["schemas", ..] => DocKind::Schema,
        _ => DocKind::Other,
    }
}

/// The flow-scoped default sets (blueprint §7.3) — the materializer's single
/// implementation; providers never narrow. No flow means the full snapshot;
/// an unknown flow fails closed rather than guessing a set.
fn select<'s>(
    documents: &[&'s ContextDocument],
    request: &ContextSnapshotRequest,
) -> Result<Vec<&'s ContextDocument>> {
    let Some(flow) = request.flow.as_deref() else {
        return Ok(documents.to_vec());
    };
    if !matches!(flow, "discuss" | "propose" | "apply" | "verify" | "archive") {
        bail!("unknown context flow: {flow}");
    }
    let change = request.change.as_deref();
    let in_change = |c: &str| change.map_or(true, |n| n == c);
    // "對應 base specs"／"canonical base"：delta 觸及的 capabilities。
    let delta_caps: std::collections::BTreeSet<&str> = documents
        .iter()
        .filter_map(|d| match classify(&d.path) {
            DocKind::DeltaSpec { change: c, capability } if in_change(c) => Some(capability),
            _ => None,
        })
        .collect();
    let keep = |d: &ContextDocument| match (flow, classify(&d.path)) {
        ("discuss", DocKind::Config | DocKind::Language) => true,
        ("propose", DocKind::Discussion | DocKind::CanonicalSpec { .. } | DocKind::Schema) => true,
        ("apply" | "verify", DocKind::ChangeDoc { change: c, file }) => {
            in_change(c) && matches!(file, "proposal.md" | "design.md" | "tasks.md")
        }
        ("apply" | "verify" | "archive", DocKind::DeltaSpec { change: c, .. }) => in_change(c),
        ("apply" | "verify" | "archive", DocKind::CanonicalSpec { capability }) => {
            delta_caps.contains(capability)
        }
        // verify = apply 集合加驗證規則（workflow config 的 rules 條目）。
        ("verify", DocKind::Config) => true,
        ("archive", DocKind::ChangeDoc { change: c, file }) => in_change(c) && file == "tasks.md",
        _ => false,
    };
    Ok(documents.iter().copied().filter(|d| keep(d)).collect())
}

/// The INDEX.md content: snapshot identity, the read-only rule, and the full
/// snapshot document index — flow-narrowed documents stay listed (marked
/// remote-only) so agents can still discover what exists.
fn index_content(
    snapshot_id: &str,
    selected: &[&ContextDocument],
    all: &[&ContextDocument],
) -> String {
    let projected: std::collections::BTreeSet<&str> =
        selected.iter().map(|d| d.path.as_str()).collect();
    let mut out = String::new();
    out.push_str("# Context Projection\n\n");
    out.push_str(&format!("Snapshot: {snapshot_id}\n\n"));
    out.push_str(
        "Read-only projection of the remote canon — do not edit these files; \
any spec change goes through speclink verbs.\n\n## Documents\n\n",
    );
    for d in all {
        if projected.contains(d.path.as_str()) {
            out.push_str(&format!("- {}\n", d.path));
        } else {
            out.push_str(&format!("- {} (remote only — not in this projection)\n", d.path));
        }
    }
    out
}

/// Fault-injection points for the staging/switch contract (store
/// `FaultPoint` precedent): private — only the in-file tests crash a
/// materialization here; production always passes `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fault {
    /// Fail after the first staged document write.
    MidStaging,
    /// Fail the switch step after staging is complete (the Windows
    /// open-handle rename restriction's error path).
    BeforeSwitch,
}

/// Materialize one snapshot into the projection directory.
pub fn materialize(
    ws: &Workspace,
    provider: &dyn SnapshotProvider,
    request: &ContextSnapshotRequest,
) -> Result<Materialized> {
    materialize_with_fault(ws, provider, request, None)
}

fn materialize_with_fault(
    ws: &Workspace,
    provider: &dyn SnapshotProvider,
    request: &ContextSnapshotRequest,
    fault: Option<Fault>,
) -> Result<Materialized> {
    let snapshot = provider.snapshot(request)?;
    let mut documents: Vec<&ContextDocument> = snapshot.documents.iter().collect();
    documents.sort_by(|a, b| a.path.cmp(&b.path));

    // Ingest-boundary validation before any write: a document whose digest
    // does not match its content is a corrupt snapshot — refuse it rather
    // than stage a projection that can never verify.
    for d in &documents {
        if speclink_store::content_digest(&d.content) != d.digest {
            bail!("snapshot document digest mismatch: {}", d.path);
        }
    }

    // Flow narrowing, then path safety for everything that will be written —
    // a path that escapes the projection is refused outright.
    let selected = select(&documents, request)?;
    let mut rels = Vec::with_capacity(selected.len());
    for d in &selected {
        reject_reserved_document(&d.path)?;
        rels.push(safe_rel_path(&d.path)?);
    }

    let index = index_content(&snapshot.snapshot_id, &selected, &documents);
    let mut files: Vec<ManifestFile> = vec![ManifestFile {
        path: "INDEX.md".to_string(),
        digest: speclink_store::content_digest(&index),
        revision: None,
    }];
    files.extend(selected.iter().map(|d| ManifestFile {
        path: d.path.clone(),
        digest: d.digest.clone(),
        revision: d.revision,
    }));
    let manifest = ProjectionManifest {
        snapshot_id: snapshot.snapshot_id.clone(),
        policy_revision: snapshot.policy_revision,
        files,
    };

    // The projection must be gitignore-covered before anything is written —
    // an unignored projection would get committed and become a second canon.
    // Amending is warned about, never silent (init/update's existing block).
    let mut warnings = Vec::new();
    if speclink_core::init::ensure_gitignore(&ws.root.join(".gitignore"))
        .with_context(|| "context projection gitignore check failed")?
    {
        warnings.push(
            ".gitignore did not cover the .speclink/ work directory — amended it so the context projection stays out of the repo".to_string(),
        );
    }

    // Staging: the complete snapshot (documents, INDEX, manifest) is
    // produced beside the projection; the current projection stays untouched
    // until the switch. A stale staging from an earlier failure is cleared —
    // retry rebuilds from scratch.
    stage(ws, &selected, &rels, &index, &manifest, fault)
        .with_context(|| "context projection staging failed")?;

    // Switch: rename-based swap — never a file-by-file overwrite of a
    // projection an agent may be reading.
    switch(ws, fault).with_context(|| "context projection switch failed")?;

    Ok(Materialized { manifest, warnings })
}

/// Produce the complete snapshot in the staging directory.
fn stage(
    ws: &Workspace,
    documents: &[&ContextDocument],
    rels: &[PathBuf],
    index: &str,
    manifest: &ProjectionManifest,
    fault: Option<Fault>,
) -> Result<()> {
    let staging = staging_dir(ws);
    if staging.exists() {
        remove_tree(&staging)
            .with_context(|| format!("clear stale staging at {}", staging.display()))?;
    }
    for (i, d) in documents.iter().enumerate() {
        let path = staging.join(&rels[i]);
        speclink_core::util::write_file(&path, &d.content)
            .with_context(|| format!("write document {}", d.path))?;
        set_readonly_best_effort(&path);
        if i == 0 && fault == Some(Fault::MidStaging) {
            bail!("fault injected after the first document write");
        }
    }
    speclink_core::util::write_file(&staging.join("INDEX.md"), index)?;
    set_readonly_best_effort(&staging.join("INDEX.md"));
    let manifest_json = format!("{}\n", serde_json::to_string_pretty(manifest)?);
    speclink_core::util::write_file(&staging.join("manifest.json"), &manifest_json)?;
    set_readonly_best_effort(&staging.join("manifest.json"));
    Ok(())
}

/// Atomically switch the staged snapshot in: the old projection is renamed
/// aside, staging renamed into place, then the old tree removed. Any rename
/// failure (the Windows open-handle restriction) rolls the old projection
/// back and keeps staging complete for retry.
fn switch(ws: &Workspace, fault: Option<Fault>) -> Result<()> {
    if fault == Some(Fault::BeforeSwitch) {
        bail!("fault injected before the rename swap");
    }
    let staging = staging_dir(ws);
    let dir = projection_dir(ws);
    let old = ws.work_dir().join("context.old");
    // A leftover from an earlier crash is disposable garbage.
    if old.exists() {
        remove_tree(&old).with_context(|| format!("clear leftover {}", old.display()))?;
    }
    let had_projection = dir.exists();
    if had_projection {
        std::fs::rename(&dir, &old)
            .with_context(|| format!("rename current projection aside to {}", old.display()))?;
    }
    if let Err(e) = std::fs::rename(&staging, &dir) {
        // Roll the old projection back so no half-state is left; staging
        // stays complete for retry either way.
        if had_projection {
            let _ = std::fs::rename(&old, &dir);
        }
        return Err(e).with_context(|| format!("rename staging into {}", dir.display()));
    }
    if had_projection {
        remove_tree(&old).with_context(|| format!("remove old projection at {}", old.display()))?;
    }
    Ok(())
}

/// The stale marker's fixed file name at the projection root. Events and
/// explicit operations only ever mark the projection stale — documents are
/// never swapped under a reading agent.
pub const STALE_MARKER: &str = "STALE";

/// Verify the projection against its manifest, fail closed: a missing or
/// unparsable manifest, a missing listed file, or any digest mismatch is
/// rejected as "modified or incomplete — refresh"; a direct edit is never
/// interpreted as a remote write (verification reads only).
pub fn verify_projection(ws: &Workspace) -> Result<()> {
    let dir = projection_dir(ws);
    let reject = |detail: String| {
        anyhow::anyhow!(
            "the context projection has been modified or is incomplete ({detail}) — refresh it before continuing"
        )
    };
    let Some(text) = speclink_core::util::read_opt(&dir.join("manifest.json")) else {
        return Err(reject("manifest.json is missing".to_string()));
    };
    let manifest: ProjectionManifest = serde_json::from_str(&text)
        .map_err(|e| reject(format!("manifest.json cannot be parsed: {e}")))?;
    for f in &manifest.files {
        let Ok(rel) = safe_rel_path(&f.path) else {
            return Err(reject(format!("unsafe manifest path: {}", f.path)));
        };
        let Some(content) = speclink_core::util::read_opt(&dir.join(rel)) else {
            return Err(reject(format!("missing file: {}", f.path)));
        };
        if speclink_store::content_digest(&content) != f.digest {
            return Err(reject(format!("digest mismatch: {}", f.path)));
        }
    }
    Ok(())
}

/// Mark the current projection stale: write the marker file only — no
/// document content changes.
pub fn mark_stale(ws: &Workspace) -> Result<()> {
    let dir = projection_dir(ws);
    if !dir.is_dir() {
        bail!("no context projection to mark stale at {}", dir.display());
    }
    speclink_core::util::write_file(
        &dir.join(STALE_MARKER),
        "This context projection is stale — refresh it before relying on its contents.\n",
    )?;
    Ok(())
}

/// Reader-side check: a marked projection should prompt a refresh.
pub fn is_stale(ws: &Workspace) -> bool {
    projection_dir(ws).join(STALE_MARKER).is_file()
}

/// The snapshot id the current projection's manifest records, if any — the
/// value a refresh sends as `If-None-Match` so an unchanged scope can skip the
/// rewrite. Absent when there is no projection or its manifest is unreadable.
pub fn current_snapshot_id(ws: &Workspace) -> Option<String> {
    let text = speclink_core::util::read_opt(&projection_dir(ws).join("manifest.json"))?;
    let manifest: ProjectionManifest = serde_json::from_str(&text).ok()?;
    Some(manifest.snapshot_id)
}

/// Rebuild the projection from a fresh snapshot (full rebuild — the
/// blueprint's disposable semantics); the whole-directory switch means the
/// stale marker does not survive.
pub fn refresh(
    ws: &Workspace,
    provider: &dyn SnapshotProvider,
    request: &ContextSnapshotRequest,
) -> Result<Materialized> {
    materialize(ws, provider, request)
}

/// The verb-flow entry point: local fs mode never materializes (`Ok(None)`,
/// nothing written); remote mode materializes and returns the outcome.
pub fn materialize_for_mode(
    ws: &Workspace,
    mode: &StoreMode,
    provider: &dyn SnapshotProvider,
    request: &ContextSnapshotRequest,
) -> Result<Option<Materialized>> {
    match mode {
        StoreMode::Fs => Ok(None),
        StoreMode::Remote(_) => materialize(ws, provider, request).map(Some),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_core::workspace::RemoteConnection;
    use speclink_protocol::context::ContextDocument;
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    /// Throwaway workspace with an openspec tree (two canonical specs, one
    /// change with delta specs, one unrelated change) and a .gitignore that
    /// already covers `.speclink/`.
    struct TempProject {
        root: PathBuf,
    }

    impl TempProject {
        fn new(tag: &str) -> TempProject {
            let root = std::env::temp_dir()
                .join(format!("speclink-host-projection-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&root);
            let write = |rel: &str, content: &str| {
                speclink_core::util::write_file(&root.join(rel), content).unwrap();
            };
            write(".gitignore", ".speclink/\n");
            write("openspec/config.yaml", "schema: spec-driven\n");
            write("openspec/LANGUAGE.md", "Respond in English.\n");
            write(
                "openspec/specs/payment/spec.md",
                "### Requirement: Pay\nPay SHALL work.\n",
            );
            write(
                "openspec/specs/auth/spec.md",
                "### Requirement: Auth\nAuth SHALL work.\n",
            );
            write("openspec/changes/add-payment/proposal.md", "## Why\n\nPay.\n");
            write("openspec/changes/add-payment/design.md", "## Context\n\nPay design.\n");
            write("openspec/changes/add-payment/tasks.md", "- [ ] 1.1 wire pay\n");
            write(
                "openspec/changes/add-payment/specs/payment/spec.md",
                "## MODIFIED Requirements\n\n### Requirement: Pay\n",
            );
            write("openspec/changes/other-change/proposal.md", "## Why\n\nOther.\n");
            write(
                "openspec/discussions/demo-topic.md",
                "---\ntopic: Demo topic\n---\n\n# Discussion: Demo topic\n",
            );
            write("openspec/schemas/custom.yaml", "name: custom\n");
            TempProject { root }
        }

        fn ws(&self) -> Workspace {
            Workspace { root: self.root.clone(), spec_dir_name: "openspec".to_string() }
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            // 投影文件盡力唯讀 — 清理需先解除（Windows 拒刪唯讀檔）。
            let _ = remove_tree(&self.root);
        }
    }

    /// Local-tree snapshot double: one consistent snapshot of the openspec
    /// tree, per-document digests computed with the store's content digest.
    /// Never narrows — flow selection is the materializer's single
    /// implementation, not the provider's.
    struct TreeProvider {
        root: PathBuf,
        policy_revision: Option<u64>,
        snapshot_id: std::cell::RefCell<String>,
    }

    impl TreeProvider {
        fn new(root: &Path) -> TreeProvider {
            TreeProvider {
                root: root.to_path_buf(),
                policy_revision: Some(7),
                snapshot_id: std::cell::RefCell::new("snap-0001".to_string()),
            }
        }
    }

    impl SnapshotProvider for TreeProvider {
        fn snapshot(&self, _request: &ContextSnapshotRequest) -> Result<ContextSnapshot> {
            let mut documents = Vec::new();
            for p in speclink_core::util::walk_files(&self.root.join("openspec")) {
                let rel = p.strip_prefix(&self.root).unwrap();
                let content = std::fs::read_to_string(&p)?;
                let digest = speclink_store::content_digest(&content);
                documents.push(ContextDocument {
                    path: speclink_core::util::to_slash(rel),
                    content,
                    revision: Some(1),
                    digest,
                });
            }
            let combined: Vec<&str> = documents.iter().map(|d| d.digest.as_str()).collect();
            Ok(ContextSnapshot {
                snapshot_id: self.snapshot_id.borrow().clone(),
                policy_revision: self.policy_revision,
                digest: speclink_store::content_digest(&combined.join("\n")),
                documents,
            })
        }
    }

    fn full_request() -> ContextSnapshotRequest {
        ContextSnapshotRequest { change: None, flow: None }
    }

    /// Every file under `dir` as (slash relative path → bytes).
    fn dir_snapshot(dir: &Path) -> BTreeMap<String, Vec<u8>> {
        speclink_core::util::walk_files(dir)
            .into_iter()
            .map(|p| {
                let rel = speclink_core::util::to_slash(p.strip_prefix(dir).unwrap());
                (rel, std::fs::read(&p).unwrap())
            })
            .collect()
    }

    // --- 投影佈局與 manifest：manifest.json（camelCase、逐文件 digest）、INDEX.md、openspec 鏡像 ---

    #[test]
    fn materialize_writes_manifest_index_and_openspec_mirror() {
        let p = TempProject::new("layout");
        let provider = TreeProvider::new(&p.root);
        let out = materialize(&p.ws(), &provider, &full_request()).unwrap();
        let dir = projection_dir(&p.ws());

        // openspec 鏡像文件逐位元等於來源。
        for rel in [
            "openspec/config.yaml",
            "openspec/LANGUAGE.md",
            "openspec/specs/payment/spec.md",
            "openspec/specs/auth/spec.md",
            "openspec/changes/add-payment/proposal.md",
            "openspec/changes/add-payment/design.md",
            "openspec/changes/add-payment/tasks.md",
            "openspec/changes/add-payment/specs/payment/spec.md",
        ] {
            assert!(dir.join(rel).is_file(), "{rel} is projected");
            assert_eq!(
                std::fs::read(dir.join(rel)).unwrap(),
                std::fs::read(p.root.join(rel)).unwrap(),
                "{rel} mirrors the source bytes"
            );
        }

        // manifest.json：camelCase 欄位、snapshotId、policyRevision、逐文件 digest 與 revision。
        let manifest_text = std::fs::read_to_string(dir.join("manifest.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&manifest_text).unwrap();
        assert_eq!(json["snapshotId"], "snap-0001", "camelCase snapshotId: {json}");
        assert_eq!(json["policyRevision"], 7, "camelCase policyRevision");
        let files = json["files"].as_array().expect("files array");
        let entry = files
            .iter()
            .find(|f| f["path"] == "openspec/config.yaml")
            .expect("config.yaml has a manifest entry");
        let content = std::fs::read_to_string(dir.join("openspec/config.yaml")).unwrap();
        assert_eq!(entry["digest"], speclink_store::content_digest(&content));
        assert_eq!(entry["revision"], 1, "per-file revision carried");

        // manifest 覆蓋投影內全部可讀文件（INDEX.md 也在內）——manifest.json 自身除外。
        let listed: Vec<&str> = files.iter().map(|f| f["path"].as_str().unwrap()).collect();
        for (rel, _) in dir_snapshot(&dir) {
            if rel == "manifest.json" {
                continue;
            }
            assert!(listed.contains(&rel.as_str()), "{rel} is digest-covered by the manifest");
        }

        // INDEX.md 列出投影文件；回傳值與 manifest.json 一致。
        let index = std::fs::read_to_string(dir.join("INDEX.md")).unwrap();
        assert!(
            index.contains("openspec/changes/add-payment/proposal.md"),
            "INDEX.md names the projected documents: {index}"
        );
        assert_eq!(out.manifest.snapshot_id, "snap-0001");
        assert_eq!(
            serde_json::to_value(&out.manifest).unwrap(),
            json,
            "the returned manifest is what was written"
        );
    }

    // --- 投影可隨時整目錄刪除重建：重建結果逐位元等價 ---

    #[test]
    fn delete_then_rebuild_yields_an_equivalent_projection() {
        let p = TempProject::new("rebuild");
        let provider = TreeProvider::new(&p.root);
        let dir = projection_dir(&p.ws());

        materialize(&p.ws(), &provider, &full_request()).unwrap();
        let before = dir_snapshot(&dir);
        assert!(!before.is_empty(), "projection has files");

        remove_tree(&dir).unwrap();
        materialize(&p.ws(), &provider, &full_request()).unwrap();
        assert_eq!(before, dir_snapshot(&dir), "rebuild is byte-equivalent");
    }

    // --- staging 產生後原子切換：故障不留半套、切換失敗保留 staging ---

    #[test]
    fn successful_materialize_leaves_no_staging_directory() {
        let p = TempProject::new("nostaging");
        let provider = TreeProvider::new(&p.root);
        materialize(&p.ws(), &provider, &full_request()).unwrap();
        assert!(
            !p.ws().work_dir().join("context.staging").exists(),
            "staging is consumed by the switch"
        );
    }

    #[test]
    fn mid_staging_failure_leaves_current_projection_untouched() {
        let p = TempProject::new("midstage");
        let provider = TreeProvider::new(&p.root);
        materialize(&p.ws(), &provider, &full_request()).unwrap();
        let before = dir_snapshot(&projection_dir(&p.ws()));

        // 來源改變後注入 staging 中途故障：現行投影逐位元不變、錯誤指出階段。
        speclink_core::util::write_file(
            &p.root.join("openspec/changes/add-payment/proposal.md"),
            "## Why\n\nChanged.\n",
        )
        .unwrap();
        let err =
            materialize_with_fault(&p.ws(), &provider, &full_request(), Some(Fault::MidStaging))
                .unwrap_err();
        assert!(format!("{err:#}").contains("staging"), "error names the stage: {err:#}");
        assert_eq!(
            before,
            dir_snapshot(&projection_dir(&p.ws())),
            "current projection is byte-identical after a staging failure"
        );
    }

    #[test]
    fn switch_failure_keeps_projection_and_a_complete_staging_for_retry() {
        let p = TempProject::new("switchfail");
        let provider = TreeProvider::new(&p.root);
        materialize(&p.ws(), &provider, &full_request()).unwrap();
        let before = dir_snapshot(&projection_dir(&p.ws()));

        speclink_core::util::write_file(
            &p.root.join("openspec/changes/add-payment/proposal.md"),
            "## Why\n\nChanged.\n",
        )
        .unwrap();
        let err =
            materialize_with_fault(&p.ws(), &provider, &full_request(), Some(Fault::BeforeSwitch))
                .unwrap_err();
        assert!(format!("{err:#}").contains("switch"), "error names the stage: {err:#}");
        assert_eq!(
            before,
            dir_snapshot(&projection_dir(&p.ws())),
            "current projection is byte-identical after a switch failure"
        );
        // 切換失敗保留完整 staging 供重試。
        let staging = p.ws().work_dir().join("context.staging");
        assert!(staging.join("manifest.json").is_file(), "staging kept complete for retry");
        assert!(staging.join("INDEX.md").is_file());

        // 重試（無故障）成功：staging 消化、投影更新為新內容。
        materialize(&p.ws(), &provider, &full_request()).unwrap();
        assert!(!staging.exists(), "retry consumes the staging directory");
        let rebuilt = std::fs::read_to_string(
            projection_dir(&p.ws()).join("openspec/changes/add-payment/proposal.md"),
        )
        .unwrap();
        assert_eq!(rebuilt, "## Why\n\nChanged.\n", "retry lands the new snapshot");
    }

    // --- 投影必為 gitignore 涵蓋：未涵蓋補寫並警告、已涵蓋不動且無警告 ---

    #[test]
    fn uncovered_gitignore_is_amended_with_a_warning() {
        let p = TempProject::new("gitignore-add");
        // 覆蓋掉 helper 寫入的涵蓋：模擬手動刪過 .speclink/ 條目的 workspace。
        std::fs::write(p.root.join(".gitignore"), "target/\n").unwrap();
        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(&p.root)
                .output()
                .expect("run git")
        };
        assert!(git(&["init", "-q"]).status.success());

        let out = materialize(&p.ws(), &TreeProvider::new(&p.root), &full_request()).unwrap();
        assert!(
            out.warnings.iter().any(|w| w.contains(".gitignore")),
            "amending warns, never silent: {:?}",
            out.warnings
        );
        let gitignore = std::fs::read_to_string(p.root.join(".gitignore")).unwrap();
        assert!(gitignore.starts_with("target/\n"), "existing entries preserved");
        assert!(gitignore.contains(".speclink/"), "projection covered");

        // git status 不顯示投影文件。
        let status = String::from_utf8(git(&["status", "--porcelain"]).stdout).unwrap();
        assert!(!status.contains(".speclink"), "projection is invisible to git: {status}");
    }

    #[test]
    fn covered_gitignore_is_left_untouched_without_warning() {
        let p = TempProject::new("gitignore-ok");
        let before = std::fs::read_to_string(p.root.join(".gitignore")).unwrap();
        let out = materialize(&p.ws(), &TreeProvider::new(&p.root), &full_request()).unwrap();
        assert!(out.warnings.is_empty(), "no warning when already covered: {:?}", out.warnings);
        assert_eq!(
            before,
            std::fs::read_to_string(p.root.join(".gitignore")).unwrap(),
            "covered .gitignore is not rewritten"
        );
    }

    // --- 完整性驗證 fail closed：digest 不符、manifest 缺失皆拒絕並要求 refresh ---

    /// 測試端解除唯讀（投影文件盡力唯讀，改檔前需先解除）。
    fn make_writable(p: &Path) {
        let mut perm = std::fs::metadata(p).unwrap().permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        perm.set_readonly(false);
        std::fs::set_permissions(p, perm).unwrap();
    }

    #[test]
    fn verify_accepts_an_untouched_projection() {
        let p = TempProject::new("verify-ok");
        let provider = TreeProvider::new(&p.root);
        materialize(&p.ws(), &provider, &full_request()).unwrap();
        verify_projection(&p.ws()).expect("an untouched projection verifies");
    }

    #[test]
    fn modified_projection_file_is_rejected_with_the_offending_path() {
        let p = TempProject::new("verify-mod");
        let provider = TreeProvider::new(&p.root);
        materialize(&p.ws(), &provider, &full_request()).unwrap();

        // 修改投影內某 spec 文件一個字元 → 拒絕、指出 digest 不符文件、要求 refresh。
        let target = projection_dir(&p.ws()).join("openspec/specs/payment/spec.md");
        make_writable(&target);
        let mut content = std::fs::read_to_string(&target).unwrap();
        content.replace_range(0..1, "X");
        std::fs::write(&target, content).unwrap();

        let err = verify_projection(&p.ws()).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("openspec/specs/payment/spec.md"),
            "error names the mismatching file: {msg}"
        );
        assert!(msg.contains("refresh"), "error demands a refresh: {msg}");

        // 遠端正典（來源樹）未被任何寫入觸及。
        assert_eq!(
            std::fs::read_to_string(p.root.join("openspec/specs/payment/spec.md")).unwrap(),
            "### Requirement: Pay\nPay SHALL work.\n",
            "verification never writes back"
        );
    }

    #[test]
    fn missing_manifest_is_rejected() {
        let p = TempProject::new("verify-nomanifest");
        let provider = TreeProvider::new(&p.root);
        materialize(&p.ws(), &provider, &full_request()).unwrap();

        let manifest = projection_dir(&p.ws()).join("manifest.json");
        make_writable(&manifest);
        std::fs::remove_file(&manifest).unwrap();

        let err = verify_projection(&p.ws()).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("refresh"), "missing manifest demands a refresh: {msg}");
    }

    // --- stale 標記與 refresh：marker 不偷換文件、refresh 全量重建 ---

    #[test]
    fn mark_stale_writes_the_marker_without_touching_documents() {
        let p = TempProject::new("stale");
        let provider = TreeProvider::new(&p.root);
        materialize(&p.ws(), &provider, &full_request()).unwrap();
        let before = dir_snapshot(&projection_dir(&p.ws()));

        mark_stale(&p.ws()).unwrap();
        assert!(is_stale(&p.ws()), "reader-side check sees the marker");
        assert!(projection_dir(&p.ws()).join(STALE_MARKER).is_file(), "fixed-name marker");

        // 文件逐位元不變（僅多出 marker）；驗證仍通過（marker 不是文件修改）。
        let mut after = dir_snapshot(&projection_dir(&p.ws()));
        after.remove(STALE_MARKER);
        assert_eq!(before, after, "no document content changed");
        verify_projection(&p.ws()).expect("the marker alone never fails verification");
    }

    #[test]
    fn refresh_rebuilds_clears_the_marker_and_updates_the_snapshot_id() {
        let p = TempProject::new("refresh");
        let provider = TreeProvider::new(&p.root);
        materialize(&p.ws(), &provider, &full_request()).unwrap();
        mark_stale(&p.ws()).unwrap();

        *provider.snapshot_id.borrow_mut() = "snap-0002".to_string();
        let out = refresh(&p.ws(), &provider, &full_request()).unwrap();
        assert_eq!(out.manifest.snapshot_id, "snap-0002");
        assert!(!is_stale(&p.ws()), "refresh clears the marker");

        let manifest_text =
            std::fs::read_to_string(projection_dir(&p.ws()).join("manifest.json")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&manifest_text).unwrap();
        assert_eq!(json["snapshotId"], "snap-0002", "manifest carries the new snapshot id");
        verify_projection(&p.ws()).expect("a refreshed projection verifies");
    }

    // --- 唯讀屬性盡力設定；完整性以 digest 為準 ---

    #[test]
    fn projected_files_are_read_only_best_effort() {
        let p = TempProject::new("readonly");
        let provider = TreeProvider::new(&p.root);
        materialize(&p.ws(), &provider, &full_request()).unwrap();
        let dir = projection_dir(&p.ws());
        for rel in ["manifest.json", "INDEX.md", "openspec/config.yaml"] {
            assert!(
                std::fs::metadata(dir.join(rel)).unwrap().permissions().readonly(),
                "{rel} is read-only"
            );
        }
    }

    // --- 依流程縮小 context：五種流程各得預設集合、未給流程為全量 ---

    fn flow_request(change: Option<&str>, flow: &str) -> ContextSnapshotRequest {
        ContextSnapshotRequest {
            change: change.map(str::to_string),
            flow: Some(flow.to_string()),
        }
    }

    /// 投影下實際存在的文件相對路徑集合（manifest.json 與 INDEX.md 除外）。
    fn projected_docs(ws: &Workspace) -> std::collections::BTreeSet<String> {
        dir_snapshot(&projection_dir(ws))
            .into_keys()
            .filter(|k| k != "manifest.json" && k != "INDEX.md")
            .collect()
    }

    #[test]
    fn discuss_flow_projects_config_language_and_a_specs_index() {
        let p = TempProject::new("flow-discuss");
        let provider = TreeProvider::new(&p.root);
        materialize(&p.ws(), &provider, &flow_request(None, "discuss")).unwrap();

        let docs = projected_docs(&p.ws());
        assert!(docs.contains("openspec/config.yaml"), "config projected: {docs:?}");
        assert!(docs.contains("openspec/LANGUAGE.md"), "LANGUAGE projected");
        assert!(
            docs.iter().all(|d| !d.starts_with("openspec/specs/")
                && !d.starts_with("openspec/changes/")
                && !d.starts_with("openspec/discussions/")),
            "discuss narrows away spec/change/discussion documents: {docs:?}"
        );
        // canonical specs 以索引呈現：INDEX.md 列出 spec 路徑供發現。
        let index =
            std::fs::read_to_string(projection_dir(&p.ws()).join("INDEX.md")).unwrap();
        assert!(
            index.contains("openspec/specs/payment/spec.md"),
            "INDEX carries the canonical specs index: {index}"
        );
    }

    #[test]
    fn propose_flow_projects_discussions_canonical_specs_and_schemas() {
        let p = TempProject::new("flow-propose");
        let provider = TreeProvider::new(&p.root);
        materialize(&p.ws(), &provider, &flow_request(None, "propose")).unwrap();

        let docs = projected_docs(&p.ws());
        assert!(docs.contains("openspec/discussions/demo-topic.md"), "discussion: {docs:?}");
        assert!(docs.contains("openspec/specs/payment/spec.md"), "canonical specs");
        assert!(docs.contains("openspec/specs/auth/spec.md"), "all canonical specs carried");
        assert!(docs.contains("openspec/schemas/custom.yaml"), "schema/template");
        assert!(
            docs.iter().all(|d| !d.starts_with("openspec/changes/")),
            "no change documents during propose: {docs:?}"
        );
    }

    #[test]
    fn apply_flow_projects_the_change_with_delta_and_matching_base_specs() {
        let p = TempProject::new("flow-apply");
        let provider = TreeProvider::new(&p.root);
        materialize(&p.ws(), &provider, &flow_request(Some("add-payment"), "apply")).unwrap();

        let docs = projected_docs(&p.ws());
        for rel in [
            "openspec/changes/add-payment/proposal.md",
            "openspec/changes/add-payment/design.md",
            "openspec/changes/add-payment/tasks.md",
            "openspec/changes/add-payment/specs/payment/spec.md",
            "openspec/specs/payment/spec.md",
        ] {
            assert!(docs.contains(rel), "{rel} is in the apply set: {docs:?}");
        }
        assert!(
            docs.iter().all(|d| !d.starts_with("openspec/changes/other-change/")),
            "unrelated changes stay out: {docs:?}"
        );
        assert!(
            !docs.contains("openspec/specs/auth/spec.md"),
            "base specs narrow to the delta's capabilities: {docs:?}"
        );
        assert!(
            docs.iter().all(|d| !d.starts_with("openspec/discussions/")),
            "no discussions in the apply set"
        );
    }

    #[test]
    fn verify_flow_is_the_apply_set_plus_validation_rules() {
        let p = TempProject::new("flow-verify");
        let provider = TreeProvider::new(&p.root);
        materialize(&p.ws(), &provider, &flow_request(Some("add-payment"), "verify")).unwrap();

        let docs = projected_docs(&p.ws());
        for rel in [
            "openspec/changes/add-payment/proposal.md",
            "openspec/changes/add-payment/design.md",
            "openspec/changes/add-payment/tasks.md",
            "openspec/changes/add-payment/specs/payment/spec.md",
            "openspec/specs/payment/spec.md",
            // 驗證規則：workflow config（rules 條目所在）。
            "openspec/config.yaml",
        ] {
            assert!(docs.contains(rel), "{rel} is in the verify set: {docs:?}");
        }
        assert!(
            docs.iter().all(|d| !d.starts_with("openspec/changes/other-change/")),
            "unrelated changes stay out: {docs:?}"
        );
    }

    #[test]
    fn archive_flow_projects_delta_base_and_tasks() {
        let p = TempProject::new("flow-archive");
        let provider = TreeProvider::new(&p.root);
        materialize(&p.ws(), &provider, &flow_request(Some("add-payment"), "archive")).unwrap();

        let docs = projected_docs(&p.ws());
        for rel in [
            "openspec/changes/add-payment/specs/payment/spec.md",
            "openspec/specs/payment/spec.md",
            "openspec/changes/add-payment/tasks.md",
        ] {
            assert!(docs.contains(rel), "{rel} is in the archive set: {docs:?}");
        }
        assert!(
            !docs.contains("openspec/changes/add-payment/proposal.md")
                && !docs.contains("openspec/changes/add-payment/design.md"),
            "archive narrows to delta/base/tasks: {docs:?}"
        );
        assert!(
            !docs.contains("openspec/specs/auth/spec.md"),
            "canonical base narrows to the delta's capabilities"
        );
    }

    #[test]
    fn absent_flow_projects_the_full_snapshot_and_unknown_flow_fails_closed() {
        let p = TempProject::new("flow-full");
        let provider = TreeProvider::new(&p.root);
        materialize(&p.ws(), &provider, &full_request()).unwrap();
        let docs = projected_docs(&p.ws());
        for rel in [
            "openspec/config.yaml",
            "openspec/specs/auth/spec.md",
            "openspec/changes/other-change/proposal.md",
            "openspec/discussions/demo-topic.md",
        ] {
            assert!(docs.contains(rel), "full projection carries {rel}");
        }

        let err = materialize(&p.ws(), &provider, &flow_request(None, "deploy")).unwrap_err();
        assert!(
            format!("{err:#}").contains("unknown context flow"),
            "a typo'd flow fails closed: {err:#}"
        );
    }

    // --- 邊界防護：路徑逃逸與保留檔名的 snapshot 文件拒收 ---

    #[test]
    fn hostile_or_reserved_document_paths_are_refused() {
        let p = TempProject::new("hostile");
        struct OneDoc(String);
        impl SnapshotProvider for OneDoc {
            fn snapshot(&self, _request: &ContextSnapshotRequest) -> Result<ContextSnapshot> {
                let content = "x".to_string();
                let digest = speclink_store::content_digest(&content);
                Ok(ContextSnapshot {
                    snapshot_id: "snap-h".to_string(),
                    policy_revision: None,
                    digest: digest.clone(),
                    documents: vec![ContextDocument {
                        path: self.0.clone(),
                        content,
                        revision: None,
                        digest,
                    }],
                })
            }
        }
        for path in ["../evil.md", "/abs.md", "", "a/../../evil.md", "manifest.json", "INDEX.md", "STALE"] {
            let err = materialize(&p.ws(), &OneDoc(path.to_string()), &full_request())
                .unwrap_err();
            let msg = format!("{err:#}");
            assert!(
                msg.contains("escapes the projection") || msg.contains("reserved projection file"),
                "{path:?} is refused: {msg}"
            );
        }
        assert!(!p.ws().work_dir().join("context").exists(), "nothing was materialized");
    }

    // --- 本地 fs 模式不建立投影；remote 模式才 materialize ---

    #[test]
    fn fs_mode_never_creates_a_projection() {
        let p = TempProject::new("fsmode");
        let provider = TreeProvider::new(&p.root);

        let out =
            materialize_for_mode(&p.ws(), &StoreMode::Fs, &provider, &full_request()).unwrap();
        assert!(out.is_none(), "fs mode materializes nothing");
        assert!(!p.ws().work_dir().exists(), "no work-dir writes in fs mode");

        let remote = StoreMode::Remote(RemoteConnection {
            url: "http://127.0.0.1:9/api/speclink/v1/projects/demo".to_string(),
            repo: None,
        });
        let out = materialize_for_mode(&p.ws(), &remote, &provider, &full_request()).unwrap();
        assert!(out.is_some(), "remote mode materializes");
        assert!(projection_dir(&p.ws()).join("manifest.json").is_file());
    }
}

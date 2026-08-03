//! Host-side drift collection: WorkspaceFacts gathering and DriftBundle
//! production.
//!
//! Local git/worktree facts are a client (Host) responsibility — the Engine's
//! drift computation is git-free and consumes the [`WorkspaceFacts`] this
//! module gathers (platform architecture §6.5's client/server split). The
//! DriftBundle fixes the basis a drift check runs against (project/repo
//! binding, spec/tasks/policy digests, created metadata, design and tasks
//! content, evidence summary), reusing the verify-evidence digest mechanism.
//! drift is diagnostic: nothing here writes to the workspace.

use crate::binding::ResolvedBinding;
use crate::bridge::{BridgeError, BridgeStore};
use serde::Serialize;
use speclink_core::command::{CommandError, ErrorCode};
use speclink_core::drift::{self, PathKind, WorkspaceFacts};
use speclink_core::model::{Change, ChangeMeta};
use speclink_core::store::Store;
use speclink_core::tasks::{self, BasisDigests};
use speclink_core::util;
use speclink_core::workspace::Workspace;
use speclink_protocol::drift as wire;
use speclink_store::{Scope, TeamStore};
use std::collections::BTreeMap;
use std::path::Path;

/// A drift check's fixed basis: the (project, repo) binding, the change's
/// spec/tasks/policy basis digests (verify-evidence mechanism), created
/// metadata, design and tasks content, and an evidence summary. Produced and
/// consumed locally by this change; the camelCase serialization is the Phase 2
/// transport shape (no transmission here). drift never writes it back.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriftBundle {
    pub project: String,
    pub repo: String,
    pub change: String,
    pub basis_digests: BasisDigests,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    pub design: String,
    pub tasks: String,
    /// Files recorded in the change's touched/evidence record.
    pub evidence_summary: Vec<String>,
    /// UTC RFC3339 production timestamp.
    pub produced_at: String,
}

/// Filesystem kind of a path, following symlinks (matching `exists`/`is_file`).
fn path_kind(p: &Path) -> PathKind {
    match std::fs::metadata(p) {
        Ok(m) if m.is_file() => PathKind::File,
        Ok(_) => PathKind::Other,
        Err(_) => PathKind::Missing,
    }
}

/// Gather the local git/worktree facts the workspace-side drift computation
/// consumes. Runs the same read-only git probes the monolithic `analyze` ran
/// (`git log`/`ls-files`/`grep HEAD`) plus filesystem stats, and records each
/// with its three-value semantics so the Engine reproduces the current output
/// without touching git. Nothing is written.
pub fn collect_workspace_facts(
    ws: &Workspace,
    store: &dyn Store,
    change: &Change,
) -> WorkspaceFacts {
    let git_ok = util::git_available(&ws.root);

    // Commit window: git log --since the created date. `None` when git is
    // unavailable or the repo has no commit history (log fails) — the same
    // signal the current code keys "git unavailable" fallbacks off.
    let since_arg = match change.meta.created.as_deref() {
        Some(c) if !c.is_empty() => format!("--since={c} 00:00:00"),
        _ => "--since=".to_string(),
    };
    let since_log = if git_ok {
        util::git(
            &ws.root,
            &["log", &since_arg, "--pretty=format:COMMIT|%H|%at|%s", "--name-only"],
        )
    } else {
        None
    };
    let commit_window = since_log.as_deref().map(drift::parse_commit_files);

    // The change's storage location as a git pathspec prefix — excluded from
    // the symbol corpus so a committed design.md cannot self-satisfy anchors.
    let exclude_prefix = change
        .dir
        .strip_prefix(&ws.root)
        .map(|rel| format!("{}/", util::to_slash(rel)))
        .unwrap_or_else(|_| format!("{}/changes/{}/", ws.spec_dir_name, change.name));

    // Tracked doc corpus (git ls-files → read *.md/*.txt, change dir excluded).
    // `None` when ls-files is unavailable.
    let tracked_docs = util::git(&ws.root, &["ls-files"]).map(|list| {
        let mut out = Vec::new();
        for line in list.lines() {
            let f = line.trim();
            if f.starts_with(&exclude_prefix) {
                continue;
            }
            if f.ends_with(".md") || f.ends_with(".txt") {
                if let Some(content) = util::read_opt(&ws.root.join(f)) {
                    out.push(content);
                }
            }
        }
        out
    });

    // Anchors drive symbol grep (non-path) and path stats (path anchors).
    let design = store.read_artifact(&change.name, "design.md").unwrap_or_default();
    let mut symbol_hits = Vec::new();
    let mut path_status: BTreeMap<String, PathKind> = BTreeMap::new();
    let exclude = format!(":(exclude){exclude_prefix}");
    for (name, is_path) in drift::design_anchors(&design) {
        if is_path {
            path_status
                .entry(name.clone())
                .or_insert_with(|| path_kind(&ws.root.join(&name)));
        } else if util::git(
            &ws.root,
            &["grep", "-q", "--word-regexp", "--fixed-strings", &name, "HEAD", "--", &exclude],
        )
        .is_some()
        {
            symbol_hits.push(name);
        }
    }
    let symbol_head_hits = git_ok.then_some(symbol_hits);

    // Task references (unchecked tasks) stated for the Tasks dimension probes.
    let task_list =
        tasks::parse(&store.read_artifact(&change.name, "tasks.md").unwrap_or_default());
    for t in &task_list {
        if t.done {
            continue;
        }
        for r in drift::task_file_refs(&t.description) {
            path_status
                .entry(r.clone())
                .or_insert_with(|| path_kind(&ws.root.join(&r)));
        }
    }

    let touched_files = tasks::TouchedRecord::load(ws, &change.name).all_files();

    WorkspaceFacts {
        commit_window,
        tracked_docs,
        symbol_head_hits,
        path_status,
        touched_files,
    }
}

/// Produce the DriftBundle fixing a drift check's basis for the change. The
/// basis digests come from the same core computation evidence recording uses,
/// so a bundle produced and consumed back-to-back never falsely reads stale.
pub fn produce_drift_bundle(
    store: &dyn Store,
    ws: &Workspace,
    change: &Change,
    binding: &ResolvedBinding,
) -> DriftBundle {
    DriftBundle {
        project: binding.project.as_str().to_string(),
        repo: binding.repo.as_str().to_string(),
        change: change.name.clone(),
        basis_digests: tasks::current_basis_digests(store, &change.name),
        created: change.meta.created.clone(),
        design: store.read_artifact(&change.name, "design.md").unwrap_or_default(),
        tasks: store.read_artifact(&change.name, "tasks.md").unwrap_or_default(),
        evidence_summary: tasks::TouchedRecord::load(ws, &change.name).all_files(),
        produced_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    }
}

/// Everything a Server can contribute to a drift report from one store
/// snapshot: the spec-side report, the basis it was computed against, and the
/// change's store-side inputs the client's workspace-side computation reads.
/// All of it comes from the same snapshot, so a report can never name a basis
/// or an input set it was not computed against.
pub struct SpecDriftView {
    pub spec: drift::SpecDriftReport,
    pub basis: BasisDigests,
    /// The change's `created` metadata — the Time dimension's input.
    pub created: Option<String>,
    /// design.md's content, or `None` when the change has no design.md. The
    /// distinction drives the Structure dimension, so it is never flattened to
    /// an empty string.
    pub design: Option<String>,
    /// tasks.md's content, or `None` when the change has no tasks.md.
    pub tasks: Option<String>,
}

/// Compute a change's spec-side drift over one TeamStore scope, together with
/// the basis and store-side inputs fixed by that same snapshot.
///
/// This is the Host's composition point for the Server's drift endpoint — the
/// bridged Engine `Store` view is materialized, used, and dropped inside, never
/// handed out. Nothing is written: the view's staged operations are discarded
/// and no unit of work is opened.
pub fn spec_drift(
    store: &dyn TeamStore,
    scope: &Scope,
    change: &str,
) -> Result<SpecDriftView, BridgeError> {
    let view = BridgeStore::materialize(store, scope).map_err(BridgeError::Store)?;
    let found = speclink_core::model::find_change(&view, change).ok_or_else(|| {
        BridgeError::Command(CommandError::new(
            ErrorCode::NotFound,
            format!("Change '{change}' not found."),
        ))
    })?;
    Ok(SpecDriftView {
        spec: drift::compute_spec_drift(&view, &found),
        basis: tasks::current_basis_digests(&view, &found.name),
        created: found.meta.created.clone(),
        design: view.read_artifact(&found.name, "design.md"),
        tasks: view.read_artifact(&found.name, "tasks.md"),
    })
}

/// A read-only Engine [`Store`] over one change's Server-supplied documents —
/// what lets a client with no local `openspec/` run the workspace-side drift
/// computation, whose signature takes a `Store`.
///
/// Only the surface drift actually reads is live: this change's design.md and
/// tasks.md (content and existence) and its `created` metadata. Everything else
/// is `unreachable!` on purpose (the `core::teststore` discipline) — the point
/// is to pin exactly which storage surface drift touches, so a future reader
/// that quietly widens it fails loudly instead of silently serving defaults.
pub struct RemoteDriftStore {
    change: String,
    created: Option<String>,
    design: Option<String>,
    tasks: Option<String>,
}

impl RemoteDriftStore {
    /// Wrap one change's Server-supplied store-side inputs.
    pub fn new(
        change: &str,
        created: Option<String>,
        design: Option<String>,
        tasks: Option<String>,
    ) -> RemoteDriftStore {
        RemoteDriftStore { change: change.to_string(), created, design, tasks }
    }

    /// The change as the Engine sees it. `dir` is the display location the
    /// documents would occupy — it is not a local path and nothing reads it
    /// from disk.
    pub fn change(&self) -> Change {
        Change {
            name: self.change.clone(),
            dir: std::path::PathBuf::from(format!("openspec/changes/{}", self.change)),
            meta: ChangeMeta { created: self.created.clone(), ..Default::default() },
            meta_error: None,
        }
    }

    /// The one artifact lookup this store serves, absence included.
    fn artifact(&self, change: &str, artifact: &str) -> Option<&String> {
        if change != self.change {
            return None;
        }
        match artifact {
            "design.md" => self.design.as_ref(),
            "tasks.md" => self.tasks.as_ref(),
            _ => None,
        }
    }
}

impl Store for RemoteDriftStore {
    fn find_change(&self, name: &str) -> Option<Change> {
        (name == self.change).then(|| self.change())
    }

    fn change_exists(&self, name: &str) -> bool {
        name == self.change
    }

    fn list_changes(&self) -> Vec<Change> {
        vec![self.change()]
    }

    fn read_artifact(&self, change: &str, artifact: &str) -> Option<String> {
        self.artifact(change, artifact).cloned()
    }

    fn artifact_exists(&self, change: &str, artifact: &str) -> bool {
        self.artifact(change, artifact).is_some()
    }

    // --- everything below is outside drift's storage surface ---

    fn create_change(&self, _name: &str, _meta_text: &str) -> anyhow::Result<std::path::PathBuf> {
        unreachable!("remote drift never creates a change")
    }
    fn updated_at_secs(&self, _name: &str) -> u64 {
        unreachable!("remote drift never sorts changes by mtime")
    }
    fn read_change_meta(&self, _name: &str) -> Option<String> {
        unreachable!("remote drift reads `created` off the change, not the raw metadata document")
    }
    fn write_change_meta(&self, _name: &str, _content: &str) -> anyhow::Result<()> {
        unreachable!("remote drift is diagnostic — it writes nothing")
    }
    fn delete_change(&self, _name: &str) -> anyhow::Result<()> {
        unreachable!("remote drift is diagnostic — it deletes nothing")
    }
    fn write_artifact(
        &self,
        _change: &str,
        _artifact: &str,
        _content: &str,
    ) -> anyhow::Result<std::path::PathBuf> {
        unreachable!("remote drift is diagnostic — it writes nothing")
    }
    fn delta_capabilities(&self, _change: &str) -> Vec<String> {
        unreachable!("the spec side is computed on the Server; this store serves the workspace side")
    }
    fn has_capability_dirs(&self, _change: &str) -> bool {
        unreachable!("the spec side is computed on the Server; this store serves the workspace side")
    }
    fn list_canonical_capabilities(&self) -> Vec<String> {
        unreachable!("the spec side is computed on the Server; this store serves the workspace side")
    }
    fn canonical_spec_exists(&self, _cap: &str) -> bool {
        unreachable!("the spec side is computed on the Server; this store serves the workspace side")
    }
    fn read_canonical_spec(&self, _cap: &str) -> Option<String> {
        unreachable!("the spec side is computed on the Server; this store serves the workspace side")
    }
    fn write_canonical_spec(&self, _cap: &str, _content: &str) -> anyhow::Result<()> {
        unreachable!("remote drift is diagnostic — it writes nothing")
    }
    fn canonical_spec_path(&self, _cap: &str) -> std::path::PathBuf {
        unreachable!("the spec side is computed on the Server; this store serves the workspace side")
    }
    fn list_archived_changes(&self) -> Vec<String> {
        // This adapter is permanently limited to one active change's drift
        // inputs; archive browsing belongs to the full BridgeStore surface.
        Vec::new()
    }
    fn archived_change_exists(&self, _dated_name: &str) -> bool {
        unreachable!("remote drift never touches the archive")
    }
    fn archive_change(&self, _name: &str, _dated_name: &str) -> anyhow::Result<()> {
        unreachable!("remote drift never touches the archive")
    }
    fn read_archived_meta(&self, _dated_name: &str) -> Option<String> {
        unreachable!("remote drift never touches the archive")
    }
    fn write_archived_meta(&self, _dated_name: &str, _content: &str) -> anyhow::Result<()> {
        unreachable!("remote drift never touches the archive")
    }
    fn live_discussion_exists(&self, _slug: &str) -> bool {
        unreachable!("remote drift never reads discussions")
    }
    fn archived_discussion_exists(&self, _slug: &str) -> bool {
        unreachable!("remote drift never reads discussions")
    }
    fn live_discussion_path(&self, _slug: &str) -> std::path::PathBuf {
        unreachable!("remote drift never reads discussions")
    }
    fn read_live_discussion(&self, _slug: &str) -> Option<String> {
        unreachable!("remote drift never reads discussions")
    }
    fn write_live_discussion(&self, _slug: &str, _content: &str) -> anyhow::Result<std::path::PathBuf> {
        unreachable!("remote drift is diagnostic — it writes nothing")
    }
    fn delete_live_discussion(&self, _slug: &str) -> anyhow::Result<()> {
        unreachable!("remote drift is diagnostic — it deletes nothing")
    }
    fn read_discussion(&self, _slug: &str) -> Option<speclink_core::store::DiscussionDoc> {
        unreachable!("remote drift never reads discussions")
    }
    fn list_live_discussions(&self) -> Vec<speclink_core::store::DiscussionDoc> {
        unreachable!("remote drift never reads discussions")
    }
    fn list_archived_discussions(&self) -> Vec<speclink_core::store::DiscussionDoc> {
        unreachable!("remote drift never reads discussions")
    }
    fn archive_discussion(&self, _slug: &str, _created: &str) -> anyhow::Result<Option<String>> {
        unreachable!("remote drift never touches the archive")
    }
    fn read_workflow_config(&self) -> Option<String> {
        unreachable!("the policy basis is computed on the Server; this store serves the workspace side")
    }
    fn read_language(&self) -> Option<String> {
        unreachable!("remote drift never reads the language document")
    }
}

// --- wire ↔ Engine mapping (server-drift-api spec「wire 與引擎型別映射單點往返」) ---
//
// The single place the drift wire DTOs and the Engine's spec-side types meet.
// It lives here because the Host is the only layer that depends on both the
// Engine and the protocol crate — and the only one both the Server and the
// typed client already depend on. The protocol crate must never depend on
// speclink-core, so the mapping cannot live there; splitting it across the
// Server (core → wire) and the client (wire → core) would be two half-mappings
// with no crate able to assert the round trip. The Engine's core types stay
// free of serde annotations: the wire's needs do not constrain their evolution.

/// A whole Server-side drift view → the wire response.
pub fn spec_drift_view_to_wire(view: &SpecDriftView) -> wire::SpecDriftResponse {
    wire::SpecDriftResponse {
        spec_drift: spec_drift_to_wire(&view.spec),
        basis: basis_to_wire(&view.basis),
        change: wire::DriftChangeInputs {
            created: view.created.clone(),
            design: view.design.clone(),
            tasks: view.tasks.clone(),
        },
    }
}

/// Engine spec-side report → wire DTO.
pub fn spec_drift_to_wire(report: &drift::SpecDriftReport) -> wire::SpecDrift {
    wire::SpecDrift {
        dimension: wire::DriftDimension {
            kind: report.dimension.kind.clone(),
            status: report.dimension.status.clone(),
            score: report.dimension.score,
            contributes_to_total: report.dimension.contributes_to_total,
        },
        spec_assumptions: report
            .spec_assumptions
            .iter()
            .map(|a| wire::SpecAssumption {
                capability: a.capability.clone(),
                operation: a.operation.clone(),
                requirement: a.requirement.clone(),
                reason: a.reason.clone(),
            })
            .collect(),
    }
}

/// Wire DTO → Engine spec-side report, for the merger on the client.
pub fn spec_drift_from_wire(w: &wire::SpecDrift) -> drift::SpecDriftReport {
    drift::SpecDriftReport {
        dimension: drift::DriftDimension {
            kind: w.dimension.kind.clone(),
            status: w.dimension.status.clone(),
            score: w.dimension.score,
            contributes_to_total: w.dimension.contributes_to_total,
        },
        spec_assumptions: w
            .spec_assumptions
            .iter()
            .map(|a| drift::SpecAssumption {
                capability: a.capability.clone(),
                operation: a.operation.clone(),
                requirement: a.requirement.clone(),
                reason: a.reason.clone(),
            })
            .collect(),
    }
}

/// Engine basis digests → wire DTO.
pub fn basis_to_wire(basis: &BasisDigests) -> wire::DriftBasisDigests {
    wire::DriftBasisDigests {
        spec: basis.spec.clone(),
        tasks: basis.tasks.clone(),
        policy: basis.policy.clone(),
    }
}

/// Wire DTO → Engine basis digests.
pub fn basis_from_wire(basis: &wire::DriftBasisDigests) -> BasisDigests {
    BasisDigests {
        spec: basis.spec.clone(),
        tasks: basis.tasks.clone(),
        policy: basis.policy.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding::local_default_binding;
    use speclink_core::store::Store;
    use speclink_core::workspace::Workspace;
    use speclink_fs::FsStore;
    use std::path::PathBuf;
    use std::process::Command;

    const META: &str = "schema: spec-driven\ncreated: 2026-07-13\n";

    /// Throwaway fs workspace for one change ("demo"). Optionally a git repo.
    struct TempProject {
        root: PathBuf,
    }

    impl TempProject {
        fn new(tag: &str) -> TempProject {
            let root = std::env::temp_dir()
                .join(format!("speclink-host-drift-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&root);
            let change = root.join("openspec").join("changes").join("demo");
            std::fs::create_dir_all(&change).unwrap();
            std::fs::write(change.join(".openspec.yaml"), META).unwrap();
            std::fs::write(
                change.join("design.md"),
                "Uses `Widget_kind`, `Missing_sym` and `src/app.rs`.",
            )
            .unwrap();
            std::fs::write(change.join("tasks.md"), "- [ ] 1.1 wire `src/app.rs`\n").unwrap();
            std::fs::create_dir_all(change.join("specs").join("auth")).unwrap();
            std::fs::write(
                change.join("specs").join("auth").join("spec.md"),
                "## ADDED Requirements\n\n### Requirement: A\n",
            )
            .unwrap();
            TempProject { root }
        }

        fn write(&self, rel: &str, content: &str) {
            let p = self.root.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, content).unwrap();
        }

        fn git(&self, args: &[&str]) {
            let ok = Command::new("git")
                .args(args)
                .current_dir(&self.root)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            assert!(ok, "git {args:?} failed");
        }

        fn init_git_with_commit(&self) {
            self.git(&["init", "-q"]);
            self.git(&["config", "user.name", "Sandbox Tester"]);
            self.git(&["config", "user.email", "sandbox@example.com"]);
            // A committed src file carries the symbol so `git grep HEAD` finds it — and
            // it is a .rs (never read into tracked_docs), isolating the grep-HEAD half.
            self.write("src/lib.rs", "pub struct Widget_kind;\n");
            self.write("src/app.rs", "// app\n");
            self.git(&["add", "-A"]);
            self.git(&["commit", "-q", "-m", "seed"]);
        }

        fn store(&self) -> FsStore {
            FsStore::new(&self.root, "openspec")
        }

        fn ws(&self) -> Workspace {
            Workspace { root: self.root.clone(), spec_dir_name: "openspec".to_string() }
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    fn walk(root: &std::path::Path) -> Vec<PathBuf> {
        // Workspace files only — git internals (.git) are irrelevant to "no write".
        speclink_core::util::walk_files(root)
            .into_iter()
            .filter(|p| !p.components().any(|c| c.as_os_str() == ".git"))
            .collect()
    }

    // --- WorkspaceFacts 蒐集器：git 可用與不可用分別產出正確三值 facts ---

    #[test]
    fn collector_on_git_unavailable_workspace_yields_unavailable_facts() {
        // 非 git 的 workspace：git 相關欄位皆 None（不可用），區別於「空值」。
        let p = TempProject::new("gitunavail");
        let store = p.store();
        let change = store.find_change("demo").unwrap();
        let facts = collect_workspace_facts(&p.ws(), &store, &change);
        assert!(facts.commit_window.is_none(), "commit window 不可用");
        assert!(facts.tracked_docs.is_none(), "tracked docs 不可用");
        assert!(facts.symbol_head_hits.is_none(), "symbol hits 不可用");
    }

    #[test]
    fn collector_on_git_available_workspace_resolves_three_value_facts() {
        let p = TempProject::new("gitavail");
        p.init_git_with_commit();
        let store = p.store();
        let change = store.find_change("demo").unwrap();
        let facts = collect_workspace_facts(&p.ws(), &store, &change);

        // git 可用 + 有 commit 歷史 → commit window 有值，含觸及 src/lib.rs 的 commit。
        let window = facts.commit_window.expect("commit window available");
        assert!(
            window.iter().flatten().any(|f| f == "src/lib.rs"),
            "commit window records the seeded commit's files: {window:?}"
        );

        // tracked docs 有值（Some），且不含被排除的 change 目錄 design.md 內容。
        let docs = facts.tracked_docs.expect("tracked docs available");
        assert!(
            !docs.iter().any(|d| d.contains("Missing_sym")),
            "change-dir design.md is excluded from tracked docs"
        );

        // 符號查詢：Widget_kind 由 git grep HEAD（在 src/lib.rs）命中；Missing_sym 未命中。
        let hits = facts.symbol_head_hits.expect("symbol hits available");
        assert!(hits.contains(&"Widget_kind".to_string()), "committed symbol found: {hits:?}");
        assert!(!hits.contains(&"Missing_sym".to_string()), "absent symbol not found");

        // 路徑錨點：src/app.rs 已提交 → File。
        assert_eq!(
            facts.path_status.get("src/app.rs"),
            Some(&speclink_core::drift::PathKind::File),
            "committed path anchor resolves to a file"
        );
    }

    // --- DriftBundle：內容齊備、basis digests 可重現 ---

    #[test]
    fn drift_bundle_is_content_complete_and_reproducible() {
        let p = TempProject::new("bundle");
        let store = p.store();
        let change = store.find_change("demo").unwrap();
        let binding = local_default_binding();

        let a = produce_drift_bundle(&store, &p.ws(), &change, &binding);
        let b = produce_drift_bundle(&store, &p.ws(), &change, &binding);

        // 同一狀態重複產生 → basis digests 逐項相同。
        assert_eq!(a.basis_digests, b.basis_digests, "basis digests reproducible");
        for d in [&a.basis_digests.spec, &a.basis_digests.tasks, &a.basis_digests.policy] {
            assert!(d.starts_with("sha256:"), "digest form: {d}");
        }

        // 內容齊備：binding、change 名、created metadata、design 與 tasks 內容、產生時間。
        assert_eq!(a.project, "default");
        assert_eq!(a.repo, "main");
        assert_eq!(a.change, "demo");
        assert_eq!(a.created.as_deref(), Some("2026-07-13"));
        assert!(a.design.contains("Widget_kind"), "design content carried");
        assert!(a.tasks.contains("src/app.rs"), "tasks content carried");
        assert!(a.produced_at.ends_with('Z'), "producedAt is UTC: {}", a.produced_at);

        // serde camelCase 形狀（Phase 2 傳輸載體）。
        let json = serde_json::to_value(&a).unwrap();
        assert!(json.get("basisDigests").is_some(), "camelCase basisDigests");
        assert!(json.get("producedAt").is_some(), "camelCase producedAt");
        assert!(json.get("evidenceSummary").is_some(), "camelCase evidenceSummary");
    }

    // --- wire ↔ 引擎型別：單點雙向映射往返結構相等 ---

    #[test]
    fn spec_drift_round_trips_through_the_wire() {
        // 樣本涵蓋規格假設的三種 operation 與各自的 reason 措辭。
        let report = drift::SpecDriftReport {
            dimension: drift::DriftDimension {
                kind: "Specs".to_string(),
                status: "3 stale assumptions".to_string(),
                score: 9,
                contributes_to_total: true,
            },
            spec_assumptions: vec![
                drift::SpecAssumption {
                    capability: "auth".to_string(),
                    operation: "ADDED".to_string(),
                    requirement: "Token rotation".to_string(),
                    reason: "already exists in the canonical spec — archive would refuse it"
                        .to_string(),
                },
                drift::SpecAssumption {
                    capability: "billing".to_string(),
                    operation: "MODIFIED".to_string(),
                    requirement: "Invoice export".to_string(),
                    reason: "target requirement no longer exists in the canonical spec".to_string(),
                },
                drift::SpecAssumption {
                    capability: "reporting".to_string(),
                    operation: "RENAMED".to_string(),
                    requirement: "Monthly rollup".to_string(),
                    reason: "canonical spec for this capability does not exist".to_string(),
                },
            ],
        };

        let back = spec_drift_from_wire(&spec_drift_to_wire(&report));

        // 引擎型別未為 wire 需求加 PartialEq／serde 標註，結構相等以 Debug 形狀斷言。
        assert_eq!(
            format!("{report:?}"),
            format!("{back:?}"),
            "core → wire → core 往返後結構相等"
        );
    }

    #[test]
    fn basis_digests_round_trip_through_the_wire() {
        let basis = BasisDigests {
            spec: "sha256:aaa".to_string(),
            tasks: "sha256:bbb".to_string(),
            policy: "sha256:ccc".to_string(),
        };
        assert_eq!(basis_from_wire(&basis_to_wire(&basis)), basis);
    }

    // --- RemoteDriftStore：以 server 供給的 store 面輸入服務引擎工作區面 ---

    #[test]
    fn remote_drift_store_serves_the_change_and_its_two_artifacts() {
        let store = RemoteDriftStore::new(
            "demo",
            Some("2026-07-13".to_string()),
            Some("## Context\n\nUses `Widget_kind`.\n".to_string()),
            Some("- [ ] 1.1 wire `src/app.rs`\n".to_string()),
        );

        let change = store.find_change("demo").expect("the change is found by name");
        assert_eq!(change.name, "demo");
        assert_eq!(change.meta.created.as_deref(), Some("2026-07-13"), "created 供 Time 維度");
        assert!(store.change_exists("demo"));

        assert!(store.read_artifact("demo", "design.md").unwrap().contains("Widget_kind"));
        assert!(store.artifact_exists("demo", "tasks.md"));

        // 另一個 change 名不屬於此 store —— 不冒充服務。
        assert!(store.find_change("other").is_none());
        assert!(!store.artifact_exists("other", "design.md"));
        assert!(
            store.list_archived_changes().is_empty(),
            "the drift-only adapter never exposes archive browsing"
        );
    }

    /// 缺席與空內容是兩件事：`artifact_exists` 驅動 Structure 維度的「no design」
    /// 分支，把 None 攤成 "" 會讓報告說謊。
    #[test]
    fn remote_drift_store_keeps_absence_distinct_from_emptiness() {
        let absent = RemoteDriftStore::new("demo", None, None, None);
        assert!(!absent.artifact_exists("demo", "design.md"), "缺席的 design 不存在");
        assert_eq!(absent.read_artifact("demo", "design.md"), None);
        assert_eq!(absent.change().meta.created, None);

        let empty = RemoteDriftStore::new("demo", None, Some(String::new()), None);
        assert!(empty.artifact_exists("demo", "design.md"), "空的 design 仍然存在");
        assert_eq!(empty.read_artifact("demo", "design.md"), Some(String::new()));
    }

    /// 引擎的工作區面計算跑得起來，且只碰得到活的儲存表面（碰到別處會 panic）。
    #[test]
    fn engine_workspace_drift_runs_over_the_remote_drift_store() {
        let store = RemoteDriftStore::new(
            "demo",
            Some("2026-07-13".to_string()),
            Some("Uses `Missing_sym` and `src/app.rs`.".to_string()),
            Some("- [ ] 1.1 wire `src/app.rs`\n".to_string()),
        );
        let change = store.change();

        // facts 不可得（無 checkout）→ 四個維度皆為不可得，不偽裝乾淨。
        let report = drift::compute_workspace_drift(&store, &change, None);
        assert_eq!(report.dimensions.len(), 4);
        assert!(
            report
                .dimensions
                .iter()
                .all(|d| matches!(d, drift::WorkspaceDimension::Unavailable { .. })),
            "無 facts 時工作區面全部不可得：{report:?}"
        );

        // facts 可得 → 引擎讀 design/tasks 算出錨點，不觸及 unreachable 表面。
        let facts = WorkspaceFacts {
            commit_window: Some(Vec::new()),
            tracked_docs: Some(Vec::new()),
            symbol_head_hits: Some(Vec::new()),
            path_status: BTreeMap::new(),
            touched_files: Vec::new(),
        };
        let report = drift::compute_workspace_drift(&store, &change, Some(&facts));
        assert!(
            report.broken_anchors.iter().any(|b| b.anchor == "Missing_sym"),
            "未命中的符號錨點被列為 broken：{:?}",
            report.broken_anchors
        );
    }

    // --- 完整流程結束後 workspace 無任何檔案被寫入 ---

    #[test]
    fn full_drift_flow_writes_no_workspace_files() {
        let p = TempProject::new("nowrite");
        p.init_git_with_commit();
        let store = p.store();
        let change = store.find_change("demo").unwrap();
        let binding = local_default_binding();

        let before = walk(&p.root);
        let _facts = collect_workspace_facts(&p.ws(), &store, &change);
        let _bundle = produce_drift_bundle(&store, &p.ws(), &change, &binding);
        let after = walk(&p.root);

        assert_eq!(before, after, "drift collection + bundle production write no files");
    }
}

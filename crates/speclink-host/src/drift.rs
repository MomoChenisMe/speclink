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
use serde::Serialize;
use speclink_core::drift::{self, PathKind, WorkspaceFacts};
use speclink_core::model::Change;
use speclink_core::store::Store;
use speclink_core::tasks::{self, BasisDigests};
use speclink_core::util;
use speclink_core::workspace::Workspace;
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

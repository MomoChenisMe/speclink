//! Spec directory migration (spec-dir-migration D4): `openspec/` → `speclink/`
//! and every change's `.openspec.yaml` → `.meta.yaml`, contents untouched.
//! `update` runs it when it is safe and defers otherwise; `init` runs the same
//! checks to import an OpenSpec project and refuses when they fail.

use crate::config::AppConfig;
use crate::model::{LEGACY_META_FILE, META_FILE};
use crate::util;
use crate::workspace::{resolve_spec_dir, SpecLayout, DEFAULT_SPEC_DIR, LEGACY_SPEC_DIR};
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

/// What a migration would do for a project right now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationPlan {
    /// No legacy layout or leftover data file — or `spec_dir` is explicit, or
    /// the project is in remote mode.
    Nothing,
    /// A migration is due but moving now is unsafe.
    Deferred(DeferReason),
    /// Rename `openspec/` to `speclink/`, then every data file inside it.
    MoveFolder,
    /// `speclink/` still holds data files under the legacy name.
    RenameLeftovers,
}

/// Why a due migration waits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeferReason {
    /// This checkout is a linked worktree; the move belongs to the main one.
    LinkedWorktree,
    /// Other worktrees exist (paths as git lists them); each holds its own copy.
    OtherWorktrees(Vec<PathBuf>),
    /// The named folder has uncommitted changes (untracked files included), or
    /// git could not report its status.
    Uncommitted(String),
    /// `openspec` is a symbolic link: renaming it would move the link, and the
    /// renamed data files would land in a folder other projects may share.
    SymlinkedFolder,
    /// Something that is not a spec folder already sits at `speclink`.
    SpecFolderTaken,
    /// No `.speclink.yaml`: an OpenSpec project, which `init` imports and
    /// `update` leaves alone.
    OpenSpecProject,
}

/// Why the move waits, with what to do about it — one wording for `update`'s
/// note and `init`'s refusal.
impl std::fmt::Display for DeferReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeferReason::LinkedWorktree => {
                write!(f, "this is a linked worktree — run it in the main checkout")
            }
            DeferReason::OtherWorktrees(paths) => {
                let paths: Vec<String> = paths.iter().map(|p| p.display().to_string()).collect();
                write!(
                    f,
                    "other worktrees still hold their own copy ({}) — merge or remove them first",
                    paths.join(", ")
                )
            }
            DeferReason::Uncommitted(folder) => {
                write!(f, "{folder}/ has uncommitted changes — commit them first")
            }
            DeferReason::SymlinkedFolder => write!(
                f,
                "{LEGACY_SPEC_DIR}/ is a symbolic link — move the folder it points to into this project by hand"
            ),
            DeferReason::SpecFolderTaken => write!(
                f,
                "something that is not a spec folder already sits at {DEFAULT_SPEC_DIR} — move it away first"
            ),
            DeferReason::OpenSpecProject => write!(
                f,
                "{LEGACY_SPEC_DIR}/ belongs to an OpenSpec project — run `speclink init` to import it"
            ),
        }
    }
}

/// The folder a migration works on: `openspec` moves to `speclink`, `speclink`
/// only renames leftover data files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SpecFolder {
    Openspec,
    Speclink,
}

impl SpecFolder {
    fn name(self) -> &'static str {
        match self {
            SpecFolder::Openspec => LEGACY_SPEC_DIR,
            SpecFolder::Speclink => DEFAULT_SPEC_DIR,
        }
    }
}

/// Why a due migration waits, as the skill-file probe reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DeferCode {
    LinkedWorktree,
    OtherWorktrees,
    Uncommitted,
    SymlinkedFolder,
    SpecFolderTaken,
}

/// The migration as the skill-file probe reports it (desktop-spec-dir-migration
/// D1): [`plan`] seen from the desktop prompt, `folder` being where the data
/// files sit now.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "state", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum MigrationProbe {
    None,
    Pending { folder: SpecFolder, meta_files: usize },
    Deferred { folder: SpecFolder, reason: DeferCode, worktrees: Vec<String> },
}

/// What `update` would do about the spec folder, for the probe. An OpenSpec
/// project is init's or adopt's to import, so it reports nothing here.
pub fn probe(root: &Path, app: &AppConfig) -> MigrationProbe {
    let folder = match resolve_spec_dir(root, None).1 {
        SpecLayout::Legacy => SpecFolder::Openspec,
        _ => SpecFolder::Speclink,
    };
    let (reason, worktrees) = match plan(root, app) {
        MigrationPlan::Nothing | MigrationPlan::Deferred(DeferReason::OpenSpecProject) => return MigrationProbe::None,
        MigrationPlan::MoveFolder | MigrationPlan::RenameLeftovers => {
            let meta_files = legacy_data_files(&root.join(folder.name())).len();
            return MigrationProbe::Pending { folder, meta_files };
        }
        MigrationPlan::Deferred(DeferReason::OtherWorktrees(paths)) => {
            (DeferCode::OtherWorktrees, paths.iter().map(|p| p.display().to_string()).collect())
        }
        MigrationPlan::Deferred(DeferReason::LinkedWorktree) => (DeferCode::LinkedWorktree, Vec::new()),
        MigrationPlan::Deferred(DeferReason::Uncommitted(_)) => (DeferCode::Uncommitted, Vec::new()),
        MigrationPlan::Deferred(DeferReason::SymlinkedFolder) => (DeferCode::SymlinkedFolder, Vec::new()),
        MigrationPlan::Deferred(DeferReason::SpecFolderTaken) => (DeferCode::SpecFolderTaken, Vec::new()),
    };
    MigrationProbe::Deferred { folder, reason, worktrees }
}

/// Whether the project at `root` is an OpenSpec project — `openspec/` and neither
/// `.speclink.yaml` nor `speclink/` — that enabling speclink imports first.
pub fn needs_import(root: &Path) -> bool {
    resolve_spec_dir(root, None).1 == SpecLayout::OpenSpec
}

/// What [`apply`] did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MigrationReport {
    /// `openspec/` was renamed to `speclink/`.
    pub moved_folder: bool,
    /// Data files renamed from `.openspec.yaml` to `.meta.yaml`.
    pub renamed: usize,
}

/// Decide what `update`'s migration would do for the project at `root`.
pub fn plan(root: &Path, app: &AppConfig) -> MigrationPlan {
    if app.remote.is_some() || app.spec_dir.is_some() {
        return MigrationPlan::Nothing;
    }
    match resolve_spec_dir(root, None).1 {
        SpecLayout::OpenSpec => MigrationPlan::Deferred(DeferReason::OpenSpecProject),
        SpecLayout::Legacy => move_plan(root),
        _ if !legacy_data_files(&root.join(DEFAULT_SPEC_DIR)).is_empty() => {
            if let Some(reason) = worktree_blocker(root) {
                return MigrationPlan::Deferred(reason);
            }
            // A speclink/ git tracks nothing of is a move not yet committed —
            // the migration's own state, which the renames finish.
            let tracked = util::git(root, &["ls-files", "--", DEFAULT_SPEC_DIR])
                .is_none_or(|files| !files.is_empty());
            if tracked && uncommitted(root, DEFAULT_SPEC_DIR) {
                return MigrationPlan::Deferred(DeferReason::Uncommitted(DEFAULT_SPEC_DIR.to_string()));
            }
            MigrationPlan::RenameLeftovers
        }
        _ => MigrationPlan::Nothing,
    }
}

/// Decide what `init`'s import of an OpenSpec project would do: the same move
/// and checks as `update`'s migration.
pub fn plan_import(root: &Path) -> MigrationPlan {
    match resolve_spec_dir(root, None).1 {
        SpecLayout::OpenSpec => move_plan(root),
        _ => MigrationPlan::Nothing,
    }
}

/// `openspec/` → `speclink/`, or why it waits.
fn move_plan(root: &Path) -> MigrationPlan {
    let reason = if root.join(LEGACY_SPEC_DIR).is_symlink() {
        Some(DeferReason::SymlinkedFolder)
    } else if root.join(DEFAULT_SPEC_DIR).symlink_metadata().is_ok() {
        // The layout read it as no spec folder, so it is something else.
        Some(DeferReason::SpecFolderTaken)
    } else if let Some(reason) = worktree_blocker(root) {
        Some(reason)
    } else if uncommitted(root, LEGACY_SPEC_DIR) {
        Some(DeferReason::Uncommitted(LEGACY_SPEC_DIR.to_string()))
    } else {
        None
    };
    reason.map_or(MigrationPlan::MoveFolder, MigrationPlan::Deferred)
}

/// Whether git reports changes under `folder`. Outside a git repo — or with git
/// missing from PATH — there is nothing to report; a status git cannot give
/// inside a repo counts as dirty, so the move waits rather than guess.
fn uncommitted(root: &Path, folder: &str) -> bool {
    util::git_available(root)
        && !util::git_raw(root, &["status", "--porcelain", "--untracked-files=all", "--", folder])
            .is_some_and(|status| status.trim().is_empty())
}

/// The worktree condition that holds a migration back. Outside a git repo —
/// or with git missing from PATH — there is nothing to check.
fn worktree_blocker(root: &Path) -> Option<DeferReason> {
    if !util::git_available(root) {
        return None;
    }
    let canonical = |p: PathBuf| std::fs::canonicalize(&p).unwrap_or(p);
    let linked = match (
        util::git(root, &["rev-parse", "--absolute-git-dir"]),
        util::git(root, &["rev-parse", "--git-common-dir"]),
    ) {
        (Some(git_dir), Some(common_dir)) => {
            canonical(PathBuf::from(git_dir)) != canonical(root.join(common_dir))
        }
        // An older git without --absolute-git-dir: a linked worktree's `.git` is a file.
        _ => root.join(".git").is_file(),
    };
    if linked {
        return Some(DeferReason::LinkedWorktree);
    }
    let listing = util::git(root, &["worktree", "list", "--porcelain"]).unwrap_or_default();
    // git lists the main worktree first.
    let others: Vec<PathBuf> = listing
        .lines()
        .filter_map(|l| l.strip_prefix("worktree "))
        .skip(1)
        .map(PathBuf::from)
        .collect();
    (!others.is_empty()).then_some(DeferReason::OtherWorktrees(others))
}

/// Every legacy data file under `spec_root` that still is a change's data file:
/// active changes first, then archived ones, each in name order. One beside a
/// `.meta.yaml` is stale — reads already ignore it and the next metadata write
/// drops it — so it is no leftover to rename over the newer file.
fn legacy_data_files(spec_root: &Path) -> Vec<PathBuf> {
    let changes = spec_root.join("changes");
    let mut dirs: Vec<PathBuf> = sub_dirs(&changes)
        .into_iter()
        .filter(|d| d.file_name().is_some_and(|n| n != "archive"))
        .collect();
    dirs.extend(sub_dirs(&changes.join("archive")));
    dirs.into_iter()
        .filter(|d| !d.join(META_FILE).is_file())
        .map(|d| d.join(LEGACY_META_FILE))
        .filter(|f| f.is_file())
        .collect()
}

fn sub_dirs(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|it| it.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect())
        .unwrap_or_default();
    out.sort();
    out
}

/// `path` relative to `root`, with forward slashes — the shape messages use.
fn shown(root: &Path, path: &Path) -> String {
    util::to_slash(path.strip_prefix(root).unwrap_or(path))
}

/// Windows closes a watched directory's handle only once the watcher thread wakes
/// up after the watch is dropped, and virus scanners hold files briefly: there, a
/// failed folder rename is retried for about a second before it counts.
const RENAME_ATTEMPTS: u32 = if cfg!(windows) { 10 } else { 1 };

/// Run `op` up to `attempts` times, 100 ms apart, until it succeeds.
fn retried<T>(attempts: u32, mut op: impl FnMut() -> std::io::Result<T>) -> std::io::Result<T> {
    let mut result = op();
    for _ in 1..attempts {
        if result.is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
        result = op();
    }
    result
}

/// Carry out `plan`. A failed folder rename moves nothing; a failed data-file
/// rename names that file and keeps the renames before it — the next run picks
/// up the rest as leftovers.
pub fn apply(root: &Path, plan: &MigrationPlan) -> Result<MigrationReport> {
    let mut report = MigrationReport::default();
    match plan {
        MigrationPlan::MoveFolder => {
            let (from, to) = (root.join(LEGACY_SPEC_DIR), root.join(DEFAULT_SPEC_DIR));
            // rename(2) would silently replace an empty directory at the target.
            if to.symlink_metadata().is_ok() {
                bail!("cannot move {LEGACY_SPEC_DIR}/ to {DEFAULT_SPEC_DIR}/: {DEFAULT_SPEC_DIR} already exists — nothing was moved");
            }
            retried(RENAME_ATTEMPTS, || std::fs::rename(&from, &to)).with_context(|| {
                format!("cannot move {LEGACY_SPEC_DIR}/ to {DEFAULT_SPEC_DIR}/ — nothing was moved (close programs that hold files in it, such as the desktop app, and retry)")
            })?;
            report.moved_folder = true;
        }
        MigrationPlan::RenameLeftovers => {}
        MigrationPlan::Nothing | MigrationPlan::Deferred(_) => return Ok(report),
    }
    for legacy in legacy_data_files(&root.join(DEFAULT_SPEC_DIR)) {
        let target = legacy.with_file_name(META_FILE);
        let failed = |why: String| {
            anyhow::anyhow!(
                "cannot rename {} to {META_FILE}: {why} — {} data file(s) renamed so far; run `speclink update` again after fixing it",
                shown(root, &legacy),
                report.renamed
            )
        };
        if target.symlink_metadata().is_ok() {
            return Err(failed(format!("{} already exists", shown(root, &target))));
        }
        std::fs::rename(&legacy, &target).map_err(|e| failed(e.to_string()))?;
        report.renamed += 1;
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    /// Throwaway project root, removed on drop. `git: true` makes it a repo
    /// with everything committed.
    struct Project {
        base: PathBuf,
        root: PathBuf,
    }

    impl Project {
        fn new(tag: &str) -> Project {
            let base = std::env::temp_dir().join(format!("speclink-migrate-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&base);
            let root = base.join("project");
            std::fs::create_dir_all(&root).unwrap();
            Project { base, root }
        }

        fn write(&self, rel: &str, content: &str) -> PathBuf {
            let path = self.root.join(rel.split('/').collect::<PathBuf>());
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, content).unwrap();
            path
        }

        fn git(&self, args: &[&str]) {
            let ok = Command::new("git")
                .arg("-C")
                .arg(&self.root)
                .args(["-c", "user.name=T", "-c", "user.email=t@example.com"])
                .args(args)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            assert!(ok, "git {args:?} failed");
        }

        fn commit_all(&self) {
            self.git(&["init", "-q"]);
            self.git(&["add", "-A"]);
            self.git(&["commit", "-qm", "init"]);
        }

        fn app(&self) -> AppConfig {
            AppConfig::load(&self.root.join(".speclink.yaml")).unwrap()
        }

        fn plan(&self) -> MigrationPlan {
            plan(&self.root, &self.app())
        }
    }

    impl Drop for Project {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.base);
        }
    }

    /// A legacy project: `.speclink.yaml` without `spec_dir` and one change.
    fn legacy(tag: &str) -> Project {
        let p = Project::new(tag);
        p.write(".speclink.yaml", "tools: [claude]\n");
        p.write("openspec/changes/demo/.openspec.yaml", "schema: spec-driven\n");
        p.write("openspec/changes/demo/tasks.md", "- [ ] 1.1 x\n");
        p
    }

    // --- plan ---

    #[test]
    fn a_settled_layout_plans_nothing() {
        let p = Project::new("settled");
        p.write(".speclink.yaml", "tools: [claude]\n");
        p.write("speclink/changes/demo/.meta.yaml", "schema: spec-driven\n");
        assert_eq!(p.plan(), MigrationPlan::Nothing);
    }

    #[test]
    fn an_explicit_spec_dir_or_remote_mode_plans_nothing() {
        let p = legacy("explicit");
        p.write(".speclink.yaml", "spec_dir: openspec\n");
        assert_eq!(p.plan(), MigrationPlan::Nothing, "explicit spec_dir");
        p.write(".speclink.yaml", "remote:\n  url: https://example.com\n");
        assert_eq!(p.plan(), MigrationPlan::Nothing, "remote mode");
    }

    #[test]
    fn a_legacy_layout_outside_git_moves() {
        let p = legacy("no-git");
        assert_eq!(p.plan(), MigrationPlan::MoveFolder);
    }

    #[test]
    fn a_committed_legacy_layout_moves_and_an_openspec_project_too() {
        let p = legacy("clean");
        p.commit_all();
        assert_eq!(p.plan(), MigrationPlan::MoveFolder);

        // An OpenSpec project (no .speclink.yaml) is init's to import, never update's.
        let openspec = Project::new("openspec-project");
        openspec.write("openspec/changes/demo/.openspec.yaml", "schema: spec-driven\n");
        openspec.commit_all();
        assert_eq!(openspec.plan(), MigrationPlan::Deferred(DeferReason::OpenSpecProject));
        assert_eq!(plan_import(&openspec.root), MigrationPlan::MoveFolder);
    }

    #[test]
    fn uncommitted_or_untracked_files_in_openspec_defer() {
        let p = legacy("dirty");
        p.commit_all();
        p.write("openspec/changes/demo/tasks.md", "- [x] 1.1 x\n");
        assert_eq!(p.plan(), MigrationPlan::Deferred(DeferReason::Uncommitted("openspec".into())), "modified");

        let p = legacy("untracked");
        p.commit_all();
        p.write("openspec/changes/demo/notes.md", "new\n");
        assert_eq!(p.plan(), MigrationPlan::Deferred(DeferReason::Uncommitted("openspec".into())), "untracked");

        // A dirty file outside openspec/ does not hold the move back.
        let p = legacy("dirty-elsewhere");
        p.commit_all();
        p.write("src/main.rs", "fn main() {}\n");
        assert_eq!(p.plan(), MigrationPlan::MoveFolder);
    }

    #[test]
    fn other_worktrees_defer_from_the_main_checkout_and_a_linked_one_defers_itself() {
        let p = legacy("worktrees");
        p.commit_all();
        let wt = p.base.join("wt");
        p.git(&["worktree", "add", "-q", wt.to_str().unwrap(), "-b", "side"]);
        let listed = crate::testkit::listed_worktrees(&p.root);
        assert_eq!(listed.len(), 1);
        assert_eq!(
            p.plan(),
            MigrationPlan::Deferred(DeferReason::OtherWorktrees(listed))
        );
        assert_eq!(
            plan(&wt, &p.app()),
            MigrationPlan::Deferred(DeferReason::LinkedWorktree)
        );
    }

    #[test]
    fn leftover_legacy_data_files_in_speclink_are_planned_for_renaming() {
        let p = Project::new("leftover");
        p.write(".speclink.yaml", "tools: [claude]\n");
        p.write("speclink/changes/done/.meta.yaml", "schema: spec-driven\n");
        p.write("speclink/changes/late/.openspec.yaml", "schema: spec-driven\n");
        p.commit_all();
        assert_eq!(p.plan(), MigrationPlan::RenameLeftovers);

        p.write("speclink/changes/late/tasks.md", "- [ ] 1.1 x\n");
        assert_eq!(p.plan(), MigrationPlan::Deferred(DeferReason::Uncommitted("speclink".into())));
    }

    // --- sharp edges (task 8.1) ---

    #[cfg(unix)]
    #[test]
    fn a_symlinked_openspec_waits_instead_of_renaming_the_link() {
        let p = Project::new("symlink");
        p.write(".speclink.yaml", "tools: [claude]\n");
        let shared = p.base.join("shared");
        std::fs::create_dir_all(shared.join("changes").join("demo")).unwrap();
        std::fs::write(shared.join("changes/demo/.openspec.yaml"), "schema: spec-driven\n").unwrap();
        std::os::unix::fs::symlink(&shared, p.root.join("openspec")).unwrap();
        assert_eq!(p.plan(), MigrationPlan::Deferred(DeferReason::SymlinkedFolder));
        std::fs::remove_file(p.root.join(".speclink.yaml")).unwrap();
        assert_eq!(plan_import(&p.root), MigrationPlan::Deferred(DeferReason::SymlinkedFolder));
        assert!(shared.join("changes/demo/.openspec.yaml").is_file(), "the shared target is untouched");
    }

    #[test]
    fn a_speclink_entry_that_is_no_spec_folder_defers_the_move() {
        // A file named speclink, or a folder of something else (a package that
        // happens to be named speclink): the move waits, and update carries on.
        let p = legacy("speclink-file");
        p.write("speclink", "not a folder\n");
        assert_eq!(p.plan(), MigrationPlan::Deferred(DeferReason::SpecFolderTaken));

        let p = legacy("speclink-package");
        p.write("speclink/__init__.py", "");
        assert_eq!(p.plan(), MigrationPlan::Deferred(DeferReason::SpecFolderTaken));
        // The race guard in apply still refuses to replace whatever sits there.
        let err = apply(&p.root, &MigrationPlan::MoveFolder).unwrap_err().to_string();
        assert!(err.contains("speclink already exists") && err.contains("nothing was moved"), "{err}");
        assert!(p.root.join("openspec/changes/demo/.openspec.yaml").is_file());
    }

    #[test]
    fn a_stale_legacy_file_beside_the_new_one_is_not_a_leftover() {
        // Reads already prefer .meta.yaml and the next metadata write drops the
        // stale file; renaming over it would lose the newer content.
        let p = Project::new("stale-legacy");
        p.write(".speclink.yaml", "tools: [claude]\n");
        p.write("speclink/changes/both/.meta.yaml", "schema: no-spec-change\n");
        p.write("speclink/changes/both/.openspec.yaml", "schema: spec-driven\n");
        assert_eq!(p.plan(), MigrationPlan::Nothing);
    }

    #[test]
    fn a_half_finished_move_in_git_resumes_the_renames() {
        // spec「再次執行 speclink update SHALL 繼續改名」：the folder moved, a data
        // file failed, and speclink/ is wholly untracked — that is the migration's
        // own uncommitted state, not the user's work.
        let p = legacy("half-move");
        p.write("openspec/changes/other/.openspec.yaml", "schema: spec-driven\n");
        p.commit_all();
        std::fs::rename(p.root.join("openspec"), p.root.join("speclink")).unwrap();
        std::fs::rename(
            p.root.join("speclink/changes/demo/.openspec.yaml"),
            p.root.join("speclink/changes/demo/.meta.yaml"),
        )
        .unwrap();
        assert_eq!(p.plan(), MigrationPlan::RenameLeftovers);

        // Once speclink/ is tracked, an uncommitted edit holds the tidy back again.
        p.git(&["add", "-A"]);
        p.git(&["commit", "-qm", "move"]);
        p.write("speclink/changes/demo/tasks.md", "- [x] 1.1 x\n");
        assert_eq!(p.plan(), MigrationPlan::Deferred(DeferReason::Uncommitted("speclink".into())));
    }

    #[cfg(unix)]
    #[test]
    fn a_failed_folder_rename_moves_nothing() {
        use std::os::unix::fs::PermissionsExt;
        let p = legacy("read-only-root");
        std::fs::set_permissions(&p.root, std::fs::Permissions::from_mode(0o555)).unwrap();
        let result = apply(&p.root, &MigrationPlan::MoveFolder);
        std::fs::set_permissions(&p.root, std::fs::Permissions::from_mode(0o755)).unwrap();
        let err = result.unwrap_err().to_string();
        assert!(err.contains("nothing was moved"), "{err}");
        assert!(p.root.join("openspec/changes/demo/.openspec.yaml").is_file());
        assert!(!p.root.join("speclink").exists());
    }

    #[test]
    fn change_folders_with_unusual_names_migrate_like_any_other() {
        let p = Project::new("odd-names");
        p.write(".speclink.yaml", "tools: [claude]\n");
        for name in ["with space", "-leading-dash", "中文名稱", "a.b"] {
            p.write(&format!("openspec/changes/{name}/.openspec.yaml"), name);
        }
        p.commit_all();
        assert_eq!(p.plan(), MigrationPlan::MoveFolder);
        let report = apply(&p.root, &MigrationPlan::MoveFolder).unwrap();
        assert_eq!(report.renamed, 4);
        for name in ["with space", "-leading-dash", "中文名稱", "a.b"] {
            let moved = p.root.join("speclink").join("changes").join(name).join(META_FILE);
            assert_eq!(std::fs::read_to_string(moved).unwrap(), name);
        }
    }

    #[test]
    fn a_failed_rename_is_retried_up_to_the_attempt_count() {
        // Windows 關監看的目錄 handle 是非同步的：改名失敗要在短時間內重試。
        let mut calls = 0;
        let result = retried(3, || {
            calls += 1;
            if calls < 3 { Err(std::io::Error::other("busy")) } else { Ok(calls) }
        });
        assert_eq!(result.unwrap(), 3);
        let mut calls = 0;
        let result: std::io::Result<()> = retried(1, || {
            calls += 1;
            Err(std::io::Error::other("busy"))
        });
        assert!(result.is_err());
        assert_eq!(calls, 1, "只試一次就不重試");
    }

    // --- apply ---

    #[test]
    fn moving_renames_the_folder_and_every_data_file_byte_for_byte() {
        let p = Project::new("move");
        p.write(".speclink.yaml", "tools: [claude]\n");
        let metas = [
            ("openspec/changes/a/.openspec.yaml", "schema: spec-driven\ncreated: 2026-10-01\n"),
            ("openspec/changes/b/.openspec.yaml", "schema: no-spec-change\r\n"),
            ("openspec/changes/archive/2026-01-01-c/.openspec.yaml", "created_by: C\n"),
            ("openspec/changes/archive/2026-01-02-d/.openspec.yaml", "created_by: D\n"),
            ("openspec/changes/archive/2026-01-03-e/.openspec.yaml", "created_by: E\n"),
        ];
        for (rel, text) in metas {
            p.write(rel, text);
        }
        p.write("openspec/specs/auth/spec.md", "# auth\n");

        let report = apply(&p.root, &MigrationPlan::MoveFolder).unwrap();
        assert_eq!(report, MigrationReport { moved_folder: true, renamed: 5 });
        assert!(!p.root.join("openspec").exists());
        assert_eq!(std::fs::read_to_string(p.root.join("speclink/specs/auth/spec.md")).unwrap(), "# auth\n");
        for (rel, text) in metas {
            let moved = p.root.join(rel.replace("openspec/", "speclink/").replace(LEGACY_META_FILE, META_FILE));
            assert_eq!(std::fs::read(&moved).unwrap(), text.as_bytes(), "{rel}");
            assert!(!moved.with_file_name(LEGACY_META_FILE).exists(), "{rel}");
        }
    }

    #[test]
    fn a_blocked_data_file_names_its_path_and_keeps_the_renames_before_it() {
        let p = Project::new("blocked");
        p.write("speclink/changes/a/.openspec.yaml", "schema: spec-driven\n");
        p.write("speclink/changes/b/.openspec.yaml", "schema: spec-driven\n");
        std::fs::create_dir_all(p.root.join("speclink/changes/b/.meta.yaml")).unwrap();

        let err = apply(&p.root, &MigrationPlan::RenameLeftovers).unwrap_err().to_string();
        assert!(err.contains("speclink/changes/b/.meta.yaml"), "{err}");
        assert!(p.root.join("speclink/changes/a/.meta.yaml").is_file(), "a stays renamed");
        assert!(p.root.join("speclink/changes/b/.openspec.yaml").is_file(), "b keeps its file");
    }
}

//! Host-side worktree discovery: the change → linked-worktree mapping that the
//! main checkout's `list` reads through.
//!
//! Local git/worktree facts are a client (Host) responsibility — the same split
//! [`crate::drift`] follows. git's own worktree registry is the ONLY source of
//! truth here: nothing is persisted, so removing a worktree retires its mapping
//! with no cleanup step. Discovery is fail-open throughout — an unavailable git,
//! an unparseable record, or a condition that does not hold yields no mapping
//! rather than an error, so the observation surface can never break `list`.

use speclink_core::store::Store;
use speclink_core::util;
use speclink_core::workspace::Workspace;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Branch naming convention that opts a linked worktree into the mapping:
/// `speclink/<change name>`.
const BRANCH_PREFIX: &str = "speclink/";

/// One linked worktree that maps to a change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeEntry {
    /// Absolute path of the worktree directory, as git reports it.
    pub path: PathBuf,
    /// Branch name with `refs/heads/` stripped (e.g. `speclink/add-dark-mode`).
    pub branch: String,
}

/// change name → its linked worktree. Empty means "no overlay": the policy is
/// off, git is unavailable, or nothing satisfied the three conditions.
pub type WorktreeFacts = BTreeMap<String, WorktreeEntry>;

/// Discover the change → worktree mapping for a main checkout.
///
/// A mapping is established only when all three hold: the worktree's branch is
/// `speclink/<change>`, an active (unarchived) change of that name exists in the
/// main workspace, and the worktree's own change directory is readable. git
/// failing at all yields an empty map — never an error.
pub fn discover(ws: &Workspace, store: &dyn Store) -> WorktreeFacts {
    let Some(text) = util::git(&ws.root, &["worktree", "list", "--porcelain"]) else {
        return WorktreeFacts::new();
    };
    facts_from_porcelain(&text, &ws.spec_dir_name, &|name| store.change_exists(name))
}

/// The observation facts for a main checkout, or an empty map.
///
/// Three gates, all of which must hold before git is spawned at all (D3): the
/// workspace root's `.git` is a DIRECTORY (a main checkout — inside a linked
/// worktree it is a file), `.speclink.yaml` loads, and the effective `worktree`
/// policy is true. A config that cannot load or parse counts as "off": the
/// observation surface must never start failing a read that used to succeed.
///
/// Every local surface that shows worktree facts (the CLI's `list`, the desktop
/// board) goes through this one gate, so they can never disagree about when the
/// overlay is in play. `get_env` is injected — the process-env read stays at the
/// caller's boundary.
pub fn observed_facts(
    ws: &Workspace,
    store: &dyn Store,
    get_env: impl Fn(&str) -> Option<String>,
) -> WorktreeFacts {
    if !ws.root.join(".git").is_dir() {
        return WorktreeFacts::new();
    }
    let policy =
        crate::policy::resolve_effective_policy(get_env, store.read_workflow_config().as_deref());
    match policy {
        Ok(p) if p.resolved().worktree => discover(ws, store),
        _ => WorktreeFacts::new(),
    }
}

/// The per-change `worktree` objects a change listing attaches — the payload
/// half of the observation surface, shared so the CLI and the desktop board
/// describe a worktree with the same fields.
pub fn payload_objects(
    facts: &WorktreeFacts,
) -> BTreeMap<String, speclink_core::listing::ListWorktreeJson> {
    facts
        .iter()
        .map(|(name, e)| {
            (
                name.clone(),
                speclink_core::listing::ListWorktreeJson {
                    path: e.path.to_string_lossy().to_string(),
                    branch: e.branch.clone(),
                },
            )
        })
        .collect()
}

/// One worktree standing in the way of turning the worktree policy off.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeardownBlocker {
    pub change: String,
    pub branch: String,
    pub path: PathBuf,
}

/// The worktrees that must be wrapped up before the worktree policy can be turned off.
///
/// Turning the policy off retires the whole worktree footprint — including the merge
/// skill — so a live worktree would be stranded with no wrap-up tool. Both write paths
/// (the CLI's `workflow-config set` and the desktop settings page) consult this one
/// answer, and both present it in the same order: by change name, as discovery keys it.
///
/// Empty means nothing blocks, which includes every fail-open case discovery already
/// covers (no git, unparseable records) — an environment without git must not become
/// one where the policy can never be turned off.
pub fn teardown_blockers(ws: &Workspace, store: &dyn Store) -> Vec<TeardownBlocker> {
    blockers_from_facts(discover(ws, store))
}

/// The facts → blockers step, split from the git call so it is testable without
/// spawning git (same split as [`facts_from_porcelain`]).
fn blockers_from_facts(facts: WorktreeFacts) -> Vec<TeardownBlocker> {
    facts
        .into_iter()
        .map(|(change, entry)| TeardownBlocker {
            change,
            branch: entry.branch,
            path: entry.path,
        })
        .collect()
}

/// The mapping step, split from the git call so the three conditions are
/// testable without spawning git.
fn facts_from_porcelain(
    text: &str,
    spec_dir_name: &str,
    change_exists: &dyn Fn(&str) -> bool,
) -> WorktreeFacts {
    let mut facts = WorktreeFacts::new();
    for entry in linked_worktrees(text) {
        let Some(change) = entry.branch.strip_prefix(BRANCH_PREFIX) else {
            continue;
        };
        // A bare `speclink/` prefix names no change; an unknown or archived name
        // fails `change_exists`. Both fall back to the main copy silently.
        if change.is_empty() || !change_exists(change) {
            continue;
        }
        if !change_dir(&entry.path, spec_dir_name, change).is_dir() {
            continue;
        }
        facts.insert(change.to_string(), entry);
    }
    facts
}

/// A change's directory inside a worktree copy — the readability probe of
/// condition (c), and the root the overlay store reads through.
fn change_dir(worktree: &Path, spec_dir_name: &str, change: &str) -> PathBuf {
    worktree.join(spec_dir_name).join("changes").join(change)
}

/// A read-redirecting decorator over the main checkout's store.
///
/// **Read-redirecting only.** Every read scoped to a MAPPED change is
/// answered by that change's worktree copy; every other read and *every* write
/// passes straight through to the main store. Its intended callers are batch
/// read surfaces (the CLI's `list`, the desktop board's listing and search),
/// and the decorator is built fresh per invocation. A write flow may READ
/// through it only when it also resolves a per-change write destination of its
/// own (the desktop board's reorder does) — writing through the overlay itself
/// always lands on the main copy, silently splitting a mapped change's read
/// and write.
pub struct WorktreeOverlay<'a> {
    inner: &'a dyn Store,
    overlays: BTreeMap<String, Box<dyn Store + 'a>>,
}

impl<'a> WorktreeOverlay<'a> {
    /// Wrap the main store. `overlays` maps a change name to the store rooted
    /// at its worktree copy — the assembly point builds those from
    /// [`WorktreeFacts`], keeping storage-layout knowledge out of the Host.
    pub fn new(
        inner: &'a dyn Store,
        overlays: BTreeMap<String, Box<dyn Store + 'a>>,
    ) -> WorktreeOverlay<'a> {
        WorktreeOverlay { inner, overlays }
    }

    /// The store a change's reads go through: its worktree copy, else the main
    /// one. The fallback decision is made HERE, once, for every redirected
    /// method: a mapped worktree that no longer holds the change (removed
    /// between discovery and this read) answers nothing — the whole read set
    /// falls back to the main copy together, never a half-worktree view.
    fn of(&self, change: &str) -> &dyn Store {
        match self.overlays.get(change) {
            Some(store) if store.change_exists(change) => &**store,
            _ => self.inner,
        }
    }
}

impl Store for WorktreeOverlay<'_> {
    // --- change-scoped reads: redirected as a set, so a mapped change never
    // presents a half-worktree view ---

    fn list_changes(&self) -> Vec<speclink_core::model::Change> {
        // The main copy owns the roster (which changes exist); a mapped entry's
        // VALUES come from its worktree — `of` already falls back wholesale
        // when the worktree no longer holds the change.
        self.inner
            .list_changes()
            .into_iter()
            .map(|c| match self.of(&c.name).find_change(&c.name) {
                Some(resolved) => resolved,
                None => c,
            })
            .collect()
    }

    fn find_change(&self, name: &str) -> Option<speclink_core::model::Change> {
        self.of(name).find_change(name)
    }

    fn change_exists(&self, name: &str) -> bool {
        self.of(name).change_exists(name)
    }

    fn updated_at_secs(&self, name: &str) -> u64 {
        self.of(name).updated_at_secs(name)
    }

    fn read_change_meta(&self, name: &str) -> Option<String> {
        self.of(name).read_change_meta(name)
    }

    fn read_artifact(&self, change: &str, artifact: &str) -> Option<String> {
        self.of(change).read_artifact(change, artifact)
    }

    fn artifact_exists(&self, change: &str, artifact: &str) -> bool {
        self.of(change).artifact_exists(change, artifact)
    }

    fn delta_capabilities(&self, change: &str) -> Vec<String> {
        self.of(change).delta_capabilities(change)
    }

    fn has_capability_dirs(&self, change: &str) -> bool {
        self.of(change).has_capability_dirs(change)
    }

    // --- everything else: straight through to the main store. Methods with a
    // trait default are delegated explicitly too — inheriting the default would
    // silently answer "empty"/"unsupported" instead of the real store. ---

    fn create_change(&self, name: &str, meta_text: &str) -> anyhow::Result<PathBuf> {
        self.inner.create_change(name, meta_text)
    }

    fn write_change_meta(&self, name: &str, content: &str) -> anyhow::Result<()> {
        self.inner.write_change_meta(name, content)
    }

    fn delete_change(&self, name: &str) -> anyhow::Result<()> {
        self.inner.delete_change(name)
    }

    fn write_artifact(&self, change: &str, artifact: &str, content: &str) -> anyhow::Result<PathBuf> {
        self.inner.write_artifact(change, artifact, content)
    }

    fn delete_artifact(&self, change: &str, artifact: &str) -> anyhow::Result<()> {
        self.inner.delete_artifact(change, artifact)
    }

    fn list_canonical_capabilities(&self) -> Vec<String> {
        self.inner.list_canonical_capabilities()
    }

    fn canonical_spec_exists(&self, cap: &str) -> bool {
        self.inner.canonical_spec_exists(cap)
    }

    fn read_canonical_spec(&self, cap: &str) -> Option<String> {
        self.inner.read_canonical_spec(cap)
    }

    fn write_canonical_spec(&self, cap: &str, content: &str) -> anyhow::Result<()> {
        self.inner.write_canonical_spec(cap, content)
    }

    fn canonical_spec_path(&self, cap: &str) -> PathBuf {
        self.inner.canonical_spec_path(cap)
    }

    fn list_archived_changes(&self) -> Vec<String> {
        self.inner.list_archived_changes()
    }

    fn archived_change_exists(&self, dated_name: &str) -> bool {
        self.inner.archived_change_exists(dated_name)
    }

    fn archive_change(&self, name: &str, dated_name: &str) -> anyhow::Result<()> {
        self.inner.archive_change(name, dated_name)
    }

    fn read_archived_meta(&self, dated_name: &str) -> Option<String> {
        self.inner.read_archived_meta(dated_name)
    }

    fn write_archived_meta(&self, dated_name: &str, content: &str) -> anyhow::Result<()> {
        self.inner.write_archived_meta(dated_name, content)
    }

    fn read_archived_artifact(&self, dated_name: &str, artifact: &str) -> Option<String> {
        self.inner.read_archived_artifact(dated_name, artifact)
    }

    fn archived_delta_capabilities(&self, dated_name: &str) -> Vec<String> {
        self.inner.archived_delta_capabilities(dated_name)
    }

    fn live_discussion_exists(&self, slug: &str) -> bool {
        self.inner.live_discussion_exists(slug)
    }

    fn archived_discussion_exists(&self, slug: &str) -> bool {
        self.inner.archived_discussion_exists(slug)
    }

    fn live_discussion_path(&self, slug: &str) -> PathBuf {
        self.inner.live_discussion_path(slug)
    }

    fn read_live_discussion(&self, slug: &str) -> Option<String> {
        self.inner.read_live_discussion(slug)
    }

    fn write_live_discussion(&self, slug: &str, content: &str) -> anyhow::Result<PathBuf> {
        self.inner.write_live_discussion(slug, content)
    }

    fn delete_live_discussion(&self, slug: &str) -> anyhow::Result<()> {
        self.inner.delete_live_discussion(slug)
    }

    fn read_discussion(&self, slug: &str) -> Option<speclink_core::store::DiscussionDoc> {
        self.inner.read_discussion(slug)
    }

    fn list_live_discussions(&self) -> Vec<speclink_core::store::DiscussionDoc> {
        self.inner.list_live_discussions()
    }

    fn list_archived_discussions(&self) -> Vec<speclink_core::store::DiscussionDoc> {
        self.inner.list_archived_discussions()
    }

    fn archive_discussion(&self, slug: &str, created: &str) -> anyhow::Result<Option<String>> {
        self.inner.archive_discussion(slug, created)
    }

    fn read_workflow_config(&self) -> Option<String> {
        self.inner.read_workflow_config()
    }

    fn read_language(&self) -> Option<String> {
        self.inner.read_language()
    }
}

/// One `git worktree list --porcelain` record before the conditions apply.
struct Record {
    path: PathBuf,
    branch: Option<String>,
    /// detached / bare / prunable — never eligible for a mapping.
    ineligible: bool,
}

/// Linked worktrees with a branch, in registry order. git lists the MAIN
/// worktree first, so that record is dropped: `list` running in the main
/// checkout must never overlay a change onto its own copy.
fn linked_worktrees(text: &str) -> Vec<WorktreeEntry> {
    parse_records(text)
        .into_iter()
        .skip(1)
        .filter(|r| !r.ineligible)
        .filter_map(|r| r.branch.map(|branch| WorktreeEntry { path: r.path, branch }))
        .collect()
}

/// Split porcelain output into records. Records are delimited by their
/// `worktree <path>` header line rather than by blank lines, so CRLF and
/// trailing-whitespace variations cannot merge two entries into one.
fn parse_records(text: &str) -> Vec<Record> {
    let mut out: Vec<Record> = Vec::new();
    for raw in text.lines() {
        let line = raw.trim_end();
        if let Some(path) = line.strip_prefix("worktree ") {
            out.push(Record {
                path: PathBuf::from(path),
                branch: None,
                ineligible: false,
            });
            continue;
        }
        let Some(cur) = out.last_mut() else {
            continue;
        };
        if let Some(branch) = line.strip_prefix("branch refs/heads/") {
            cur.branch = Some(branch.to_string());
        } else if line == "detached" || line == "bare" || line.starts_with("prunable") {
            cur.ineligible = true;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_fs::FsStore;

    /// A main checkout plus a worktree copy, both real `openspec/` layouts.
    struct Pair {
        dir: PathBuf,
    }

    impl Pair {
        fn new(tag: &str) -> Pair {
            let dir = std::env::temp_dir().join(format!(
                "speclink-host-overlay-{tag}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(dir.join("main")).unwrap();
            std::fs::create_dir_all(dir.join("wt")).unwrap();
            Pair { dir }
        }

        fn main_root(&self) -> PathBuf {
            self.dir.join("main")
        }

        fn wt_root(&self) -> PathBuf {
            self.dir.join("wt")
        }

        /// Write one file inside a copy's change directory.
        fn put(&self, copy: &str, change: &str, file: &str, body: &str) {
            let path = self
                .dir
                .join(copy)
                .join("openspec")
                .join("changes")
                .join(change)
                .join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, body).unwrap();
        }

        fn read(&self, copy: &str, change: &str, file: &str) -> Option<String> {
            std::fs::read_to_string(
                self.dir
                    .join(copy)
                    .join("openspec")
                    .join("changes")
                    .join(change)
                    .join(file),
            )
            .ok()
        }
    }

    impl Drop for Pair {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    /// The overlay under test: `mapped` reads through the worktree copy.
    fn overlay<'a>(
        inner: &'a dyn Store,
        wt_root: &Path,
        mapped: &str,
    ) -> WorktreeOverlay<'a> {
        let mut overlays: BTreeMap<String, Box<dyn Store + 'a>> = BTreeMap::new();
        overlays.insert(mapped.to_string(), Box::new(FsStore::new(wt_root, "openspec")));
        WorktreeOverlay::new(inner, overlays)
    }

    #[test]
    fn mapped_change_reads_come_from_the_worktree_copy() {
        let p = Pair::new("hit");
        p.put("main", "add-dark-mode", "tasks.md", "- [ ] one\n- [ ] two\n");
        p.put("wt", "add-dark-mode", "tasks.md", "- [x] one\n- [ ] two\n");
        let main = FsStore::new(&p.main_root(), "openspec");
        let ov = overlay(&main, &p.wt_root(), "add-dark-mode");
        assert_eq!(
            ov.read_artifact("add-dark-mode", "tasks.md").as_deref(),
            Some("- [x] one\n- [ ] two\n"),
            "the worktree copy answers the read"
        );
    }

    #[test]
    fn unmapped_change_reads_pass_through_to_the_main_copy() {
        let p = Pair::new("miss");
        p.put("main", "other", "tasks.md", "- [ ] main only\n");
        p.put("wt", "other", "tasks.md", "- [x] worktree\n");
        let main = FsStore::new(&p.main_root(), "openspec");
        let ov = overlay(&main, &p.wt_root(), "add-dark-mode");
        assert_eq!(
            ov.read_artifact("other", "tasks.md").as_deref(),
            Some("- [ ] main only\n"),
            "an unmapped change never sees the worktree"
        );
    }

    #[test]
    fn writes_always_land_in_the_main_copy() {
        let p = Pair::new("write");
        p.put("main", "add-dark-mode", "tasks.md", "- [ ] one\n");
        p.put("wt", "add-dark-mode", "tasks.md", "- [x] one\n");
        let main = FsStore::new(&p.main_root(), "openspec");
        let ov = overlay(&main, &p.wt_root(), "add-dark-mode");
        ov.write_artifact("add-dark-mode", "proposal.md", "written\n")
            .expect("write ok");
        assert_eq!(
            p.read("main", "add-dark-mode", "proposal.md").as_deref(),
            Some("written\n"),
            "the main copy receives the write"
        );
        assert_eq!(
            p.read("wt", "add-dark-mode", "proposal.md"),
            None,
            "the worktree copy is never written through the overlay"
        );
    }

    #[test]
    fn list_changes_reports_the_worktree_metadata_for_mapped_changes() {
        // Spec scenario worktree 副本中介資料損壞如實診斷: the diagnostic follows
        // the copy the values come from.
        let p = Pair::new("list");
        p.put("main", "add-dark-mode", ".openspec.yaml", "schema: spec-driven\n");
        p.put("main", "other", ".openspec.yaml", "schema: spec-driven\n");
        p.put("wt", "add-dark-mode", ".openspec.yaml", ": not yaml : [\n");
        let main = FsStore::new(&p.main_root(), "openspec");
        let ov = overlay(&main, &p.wt_root(), "add-dark-mode");
        let listed = ov.list_changes();
        let names: Vec<&str> = listed.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["add-dark-mode", "other"], "the main roster still drives the list");
        let mapped = listed.iter().find(|c| c.name == "add-dark-mode").unwrap();
        assert!(
            mapped.meta_error.is_some(),
            "the worktree copy's corruption surfaces: {:?}",
            mapped.meta_error
        );
        let plain = listed.iter().find(|c| c.name == "other").unwrap();
        assert!(plain.meta_error.is_none(), "unmapped entries are untouched");
    }

    #[test]
    fn every_redirected_read_falls_back_together_when_the_worktree_vanishes() {
        // 回退必須整組一致：worktree 在 discovery 後被移除時，meta、artifact、
        // 時間戳全部回讀主副本——不得出現「主副本的 meta ＋ 讀不到 tasks.md」
        // 的半套視圖。
        let p = Pair::new("vanish");
        p.put("main", "add-dark-mode", ".openspec.yaml", "schema: spec-driven\n");
        p.put("main", "add-dark-mode", "tasks.md", "- [ ] main one\n");
        std::fs::create_dir_all(p.wt_root().join("openspec").join("changes")).unwrap();
        let main = FsStore::new(&p.main_root(), "openspec");
        let ov = overlay(&main, &p.wt_root(), "add-dark-mode");
        assert_eq!(
            ov.read_artifact("add-dark-mode", "tasks.md").as_deref(),
            Some("- [ ] main one\n"),
            "artifact reads fall back with the rest"
        );
        assert_eq!(
            ov.read_change_meta("add-dark-mode").as_deref(),
            Some("schema: spec-driven\n"),
            "meta reads fall back with the rest"
        );
        assert!(
            ov.updated_at_secs("add-dark-mode") > 0,
            "the sort key falls back instead of dropping to 0"
        );
        assert!(ov.artifact_exists("add-dark-mode", "tasks.md"));
    }

    #[test]
    fn a_mapped_change_missing_from_the_worktree_falls_back_to_the_main_entry() {
        // Defence in depth: discovery guarantees the directory exists, but a
        // worktree removed between discovery and read must not drop the entry.
        let p = Pair::new("gone");
        p.put("main", "add-dark-mode", ".openspec.yaml", "schema: spec-driven\n");
        std::fs::create_dir_all(p.wt_root().join("openspec").join("changes")).unwrap();
        let main = FsStore::new(&p.main_root(), "openspec");
        let ov = overlay(&main, &p.wt_root(), "add-dark-mode");
        let listed = ov.list_changes();
        assert_eq!(listed.len(), 1, "the entry survives: {listed:?}");
        assert!(listed[0].meta_error.is_none());
    }

    /// A registry with the main checkout plus one conventional linked worktree.
    const PORCELAIN: &str = "\
worktree /repos/speclink
HEAD aaaa1111
branch refs/heads/main

worktree /repos/speclink.worktrees/add-dark-mode
HEAD bbbb2222
branch refs/heads/speclink/add-dark-mode
";

    fn always(_: &str) -> bool {
        true
    }

    fn never(_: &str) -> bool {
        false
    }

    /// Temp dir holding a worktree layout, removed on drop.
    struct TempTree {
        dir: PathBuf,
    }

    impl TempTree {
        /// A worktree root with `openspec/changes/<change>/` present for each name.
        fn with_changes(tag: &str, changes: &[&str]) -> TempTree {
            let dir = std::env::temp_dir().join(format!(
                "speclink-host-worktree-{tag}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            for change in changes {
                std::fs::create_dir_all(dir.join("openspec").join("changes").join(change)).unwrap();
            }
            std::fs::create_dir_all(&dir).unwrap();
            TempTree { dir }
        }

        /// Porcelain text mapping `branch` onto this tree (main checkout first).
        fn porcelain(&self, branch: &str) -> String {
            format!(
                "worktree /repos/main\nHEAD aaaa\nbranch refs/heads/main\n\n\
                 worktree {}\nHEAD bbbb\nbranch refs/heads/{branch}\n",
                self.dir.display()
            )
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    // --- porcelain parsing ---

    #[test]
    fn parses_linked_worktrees_and_drops_the_main_checkout() {
        let out = linked_worktrees(PORCELAIN);
        assert_eq!(
            out,
            vec![WorktreeEntry {
                path: PathBuf::from("/repos/speclink.worktrees/add-dark-mode"),
                branch: "speclink/add-dark-mode".to_string(),
            }],
            "only the linked record survives, with refs/heads/ stripped"
        );
    }

    #[test]
    fn skips_detached_prunable_and_bare_records() {
        let text = "\
worktree /repos/main
HEAD aaaa
branch refs/heads/main

worktree /repos/detached
HEAD bbbb
detached

worktree /repos/gone
HEAD cccc
branch refs/heads/speclink/gone
prunable gitdir file points to non-existent location

worktree /repos/bare
bare
";
        assert!(linked_worktrees(text).is_empty(), "got: {:?}", linked_worktrees(text));
    }

    #[test]
    fn tolerates_crlf_and_a_missing_trailing_newline() {
        let text = "worktree /repos/main\r\nHEAD aaaa\r\nbranch refs/heads/main\r\n\r\n\
                    worktree /repos/wt\r\nHEAD bbbb\r\nbranch refs/heads/speclink/x";
        assert_eq!(
            linked_worktrees(text),
            vec![WorktreeEntry {
                path: PathBuf::from("/repos/wt"),
                branch: "speclink/x".to_string(),
            }]
        );
    }

    #[test]
    fn empty_output_yields_no_worktrees() {
        assert!(linked_worktrees("").is_empty());
    }

    // --- the three mapping conditions ---

    #[test]
    fn maps_a_change_when_all_three_conditions_hold() {
        // Spec scenario 映射成立.
        let tree = TempTree::with_changes("all-hold", &["add-dark-mode"]);
        let facts = facts_from_porcelain(
            &tree.porcelain("speclink/add-dark-mode"),
            "openspec",
            &always,
        );
        let entry = facts.get("add-dark-mode").expect("mapped: {facts:?}");
        assert_eq!(entry.path, tree.dir);
        assert_eq!(entry.branch, "speclink/add-dark-mode");
    }

    #[test]
    fn skips_a_branch_outside_the_convention() {
        // Spec scenario 分支不合慣例即略過.
        let tree = TempTree::with_changes("no-prefix", &["add-dark-mode"]);
        let facts =
            facts_from_porcelain(&tree.porcelain("feature/add-dark-mode"), "openspec", &always);
        assert!(facts.is_empty(), "got: {facts:?}");
    }

    #[test]
    fn skips_a_change_absent_from_the_main_workspace() {
        // Spec scenario 同名 change 不存在即略過.
        let tree = TempTree::with_changes("ghost", &["ghost-change"]);
        let facts =
            facts_from_porcelain(&tree.porcelain("speclink/ghost-change"), "openspec", &never);
        assert!(facts.is_empty(), "got: {facts:?}");
    }

    #[test]
    fn skips_a_worktree_without_the_change_directory() {
        // Spec scenario worktree 內 spec 目錄不可讀即回讀主副本.
        let tree = TempTree::with_changes("no-dir", &[]);
        let facts =
            facts_from_porcelain(&tree.porcelain("speclink/add-dark-mode"), "openspec", &always);
        assert!(facts.is_empty(), "got: {facts:?}");
    }

    #[test]
    fn skips_a_bare_branch_prefix_naming_no_change() {
        let tree = TempTree::with_changes("bare-prefix", &[]);
        let facts = facts_from_porcelain(&tree.porcelain("speclink/"), "openspec", &always);
        assert!(facts.is_empty(), "got: {facts:?}");
    }

    // --- teardown blockers: what stands in the way of turning the policy off ---

    #[test]
    fn every_mapped_worktree_blocks_teardown_in_change_order() {
        let tree = TempTree::with_changes("blockers", &["add-dark-mode"]);
        let facts =
            facts_from_porcelain(&tree.porcelain("speclink/add-dark-mode"), "openspec", &always);

        let blockers = blockers_from_facts(facts);

        assert_eq!(blockers.len(), 1, "got: {blockers:?}");
        assert_eq!(blockers[0].change, "add-dark-mode");
        assert_eq!(blockers[0].branch, "speclink/add-dark-mode");
        assert_eq!(blockers[0].path, tree.dir);
    }

    #[test]
    fn no_mapping_means_nothing_blocks_teardown() {
        assert!(blockers_from_facts(WorktreeFacts::new()).is_empty());
    }

    #[test]
    fn teardown_is_unblocked_outside_a_git_repo() {
        // fail-open 沿用 discovery：git 不可用時不擋，否則沒有 git 的環境將永遠
        // 關不掉政策。
        let dir = std::env::temp_dir()
            .join(format!("speclink-host-wt-teardown-nogit-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let ws = Workspace { root: dir.clone(), spec_dir_name: "openspec".to_string() };
        let store = FsStore::new(&dir, "openspec");
        assert!(teardown_blockers(&ws, &store).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn discovery_outside_a_git_repo_yields_an_empty_map() {
        // Spec scenario git 失敗時 fail-open: `git worktree list` failing (here:
        // not a repo at all) must produce no mapping, never an error or a panic.
        let dir = std::env::temp_dir().join(format!("speclink-host-wt-nogit-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let ws = Workspace { root: dir.clone(), spec_dir_name: "openspec".to_string() };
        let store = FsStore::new(&dir, "openspec");
        assert!(discover(&ws, &store).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn honors_a_non_default_spec_dir_name() {
        let dir = std::env::temp_dir().join(format!("speclink-host-wt-specdir-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("docs").join("changes").join("x")).unwrap();
        let text = format!(
            "worktree /repos/main\nHEAD a\nbranch refs/heads/main\n\n\
             worktree {}\nHEAD b\nbranch refs/heads/speclink/x\n",
            dir.display()
        );
        assert!(facts_from_porcelain(&text, "docs", &always).contains_key("x"));
        assert!(facts_from_porcelain(&text, "openspec", &always).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

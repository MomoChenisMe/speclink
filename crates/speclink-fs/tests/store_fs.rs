//! Behavior tests for the filesystem Store adapter.
//!
//! Fixtures are built in a throwaway temp directory laid out like a real project
//! (`<root>/openspec/...`). Assertions pin the behaviors the engine relies on:
//! change enumeration and updated_at ordering (whole-second truncation, newest first),
//! artifact read/write/exists, empty-list/default fallbacks for missing files and
//! directories, default metadata for a corrupt `.openspec.yaml`, and discussion
//! creation/append/archive naming.

use speclink_core::store::Store;
use speclink_fs::FsStore;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Throwaway project root, removed on drop.
struct TempRoot {
    dir: PathBuf,
}

impl TempRoot {
    fn new(tag: &str) -> TempRoot {
        let dir = std::env::temp_dir().join(format!(
            "speclink-fs-test-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        TempRoot { dir }
    }

    fn store(&self) -> FsStore {
        FsStore::new(&self.dir, "openspec")
    }

    fn write(&self, rel: &str, content: &str) -> PathBuf {
        let path = self
            .dir
            .join(rel.split('/').collect::<PathBuf>());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, content).unwrap();
        path
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn set_mtime(path: &Path, secs_ago: u64) {
    let t = SystemTime::now() - Duration::from_secs(secs_ago);
    let f = File::options().write(true).open(path).unwrap();
    f.set_modified(t).unwrap();
}

// --- changes: enumeration, metadata, ordering ---

#[test]
fn missing_changes_dir_yields_empty_list() {
    let root = TempRoot::new("no-changes-dir");
    let store = root.store();
    assert!(store.list_changes().is_empty());
    assert!(store.find_change("anything").is_none());
    assert!(!store.change_exists("anything"));
}

#[test]
fn list_changes_excludes_archive_and_sorts_by_name() {
    let root = TempRoot::new("list-changes");
    root.write("openspec/changes/zeta/.openspec.yaml", "schema: spec-driven\ncreated: 2026-07-01\n");
    root.write("openspec/changes/alpha/.openspec.yaml", "schema: spec-driven\ncreated: 2026-07-02\n");
    root.write("openspec/changes/archive/2026-07-01-old/.openspec.yaml", "schema: spec-driven\n");
    let store = root.store();
    let names: Vec<String> = store.list_changes().into_iter().map(|c| c.name).collect();
    assert_eq!(names, vec!["alpha", "zeta"]);
}

#[test]
fn change_meta_is_parsed_and_corrupt_yaml_carries_meta_error_with_default_meta() {
    let root = TempRoot::new("meta");
    root.write(
        "openspec/changes/good/.openspec.yaml",
        "schema: custom-flow\ncreated: 2026-07-01\ncreated_by: Tester <t@example.com>\n",
    );
    root.write("openspec/changes/broken/.openspec.yaml", ": : :\n\t bad yaml [unclosed\n");
    // A change directory without .openspec.yaml at all also gets default meta.
    std::fs::create_dir_all(root.dir.join("openspec/changes/bare")).unwrap();
    let store = root.store();

    let good = store.find_change("good").unwrap();
    assert_eq!(good.meta.schema.as_deref(), Some("custom-flow"));
    assert_eq!(good.meta.created.as_deref(), Some("2026-07-01"));
    assert_eq!(good.meta.schema_name(), "custom-flow");
    assert!(good.meta_error.is_none());

    // 壞檔 fail closed（design 決策一）：meta 以預設值承載讓 list 照常列出，
    // meta_error 帶解析原因供守門與診斷。
    let broken = store.find_change("broken").unwrap();
    assert!(broken.meta.schema.is_none());
    assert_eq!(broken.meta.schema_name(), "spec-driven");
    let reason = broken.meta_error.expect("corrupt YAML must carry the parse reason");
    assert!(!reason.is_empty());
    let names: Vec<String> = store.list_changes().into_iter().map(|c| c.name).collect();
    assert_eq!(names, vec!["bare", "broken", "good"], "corrupt meta must not drop the change");

    // 缺檔維持既有預設行為，無診斷。
    let bare = store.find_change("bare").unwrap();
    assert!(bare.meta.created.is_none());
    assert_eq!(bare.meta.schema_name(), "spec-driven");
    assert!(bare.meta_error.is_none());
}

#[test]
fn create_change_writes_meta_and_reports_dir() {
    let root = TempRoot::new("create-change");
    let store = root.store();
    assert!(!store.change_exists("demo"));
    let dir = store.create_change("demo", "schema: spec-driven\ncreated: 2026-07-04\n").unwrap();
    assert!(store.change_exists("demo"));
    assert_eq!(dir, root.dir.join("openspec").join("changes").join("demo"));
    let meta = std::fs::read_to_string(dir.join(".openspec.yaml")).unwrap();
    assert_eq!(meta, "schema: spec-driven\ncreated: 2026-07-04\n");
    let c = store.find_change("demo").unwrap();
    assert_eq!(c.dir, dir);
}

// --- delete change (discard) ---

#[test]
fn delete_change_removes_the_whole_tree_and_drops_it_from_the_listing() {
    let root = TempRoot::new("delete-change");
    let store = root.store();
    store.create_change("demo", "schema: spec-driven\ncreated: 2026-07-09\n").unwrap();
    store.write_artifact("demo", "proposal.md", "## Why\n\nx\n").unwrap();
    store.write_artifact("demo", "tasks.md", "- [ ] 1.1 t\n").unwrap();
    // Nested subdirectories with several files must be removed wholesale.
    store.write_artifact("demo", "specs/cap-a/spec.md", "## ADDED Requirements\n").unwrap();
    store.write_artifact("demo", "specs/cap-b/spec.md", "## ADDED Requirements\n").unwrap();
    // A sibling change must survive.
    store.create_change("keep", "schema: spec-driven\n").unwrap();

    assert!(store.change_exists("demo"));
    store.delete_change("demo").unwrap();

    assert!(!store.change_exists("demo"));
    assert!(
        !root.dir.join("openspec/changes/demo".split('/').collect::<PathBuf>()).exists(),
        "the change directory (and its nested specs/) must be gone"
    );
    let names: Vec<String> = store.list_changes().into_iter().map(|c| c.name).collect();
    assert_eq!(names, vec!["keep"], "only the deleted change leaves the listing");
}

// --- active change metadata: raw read/write pair (symmetric with archived) ---

#[test]
fn change_meta_raw_read_returns_verbatim_text_and_none_for_missing_change() {
    let root = TempRoot::new("meta-raw-read");
    let store = root.store();
    assert!(store.read_change_meta("ghost").is_none());

    let raw = "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: Tester <t@example.com>\n";
    store.create_change("demo", raw).unwrap();
    assert_eq!(store.read_change_meta("demo").unwrap(), raw);
}

#[test]
fn change_meta_raw_write_roundtrip_preserves_existing_and_unknown_fields_verbatim() {
    let root = TempRoot::new("meta-raw-write");
    let store = root.store();
    // Unknown fields must survive a read → append → write cycle byte-for-byte
    // (the raw pair exists precisely so stamping never re-serializes YAML).
    let raw = "schema: spec-driven\ncreated: 2026-07-01\ncustom_field: keep me exactly\nfrom_discussion: 桌面即時刷新與封存瀏覽\n";
    store.create_change("demo", raw).unwrap();

    let mut text = store.read_change_meta("demo").unwrap();
    text.push_str("started_at: 2026-07-06\n");
    store.write_change_meta("demo", &text).unwrap();

    let after = store.read_change_meta("demo").unwrap();
    assert!(
        after.starts_with(raw),
        "existing and unknown fields must be preserved verbatim, got: {after}"
    );
    assert!(after.ends_with("started_at: 2026-07-06\n"));
}

#[test]
fn updated_at_is_newest_mtime_truncated_to_seconds_and_orders_newest_first() {
    let root = TempRoot::new("updated-at");
    let old_file = root.write("openspec/changes/older/.openspec.yaml", "schema: spec-driven\n");
    let old_task = root.write("openspec/changes/older/tasks.md", "- [ ] t\n");
    let new_file = root.write("openspec/changes/newer/.openspec.yaml", "schema: spec-driven\n");
    set_mtime(&old_file, 500);
    set_mtime(&old_task, 400);
    set_mtime(&new_file, 100);
    let store = root.store();

    let older = store.updated_at_secs("older");
    let newer = store.updated_at_secs("newer");
    // Newest mtime INSIDE the change wins (400s ago beats 500s ago).
    let expected_older = std::fs::metadata(&old_task)
        .unwrap()
        .modified()
        .unwrap()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    assert_eq!(older, expected_older, "updated_at must be whole-second truncated mtime");
    assert!(newer > older, "more recently touched change must sort newer");
    // Missing change reports 0 (the sort fallback).
    assert_eq!(store.updated_at_secs("missing"), 0);

    // Newest-first ordering as the CLI derives it from updated_at.
    let mut changes = store.list_changes();
    changes.sort_by(|a, b| {
        store
            .updated_at_secs(&b.name)
            .cmp(&store.updated_at_secs(&a.name))
            .then_with(|| a.name.cmp(&b.name))
    });
    let names: Vec<String> = changes.into_iter().map(|c| c.name).collect();
    assert_eq!(names, vec!["newer", "older"]);
}

// --- artifacts ---

#[test]
fn artifact_roundtrip_and_existence() {
    let root = TempRoot::new("artifacts");
    let store = root.store();
    store.create_change("demo", "schema: spec-driven\n").unwrap();

    assert!(!store.artifact_exists("demo", "proposal.md"));
    assert!(store.read_artifact("demo", "proposal.md").is_none());

    let path = store.write_artifact("demo", "proposal.md", "## Why\n\nBecause.\n").unwrap();
    assert_eq!(path, root.dir.join("openspec/changes/demo/proposal.md".split('/').collect::<PathBuf>()));
    assert!(store.artifact_exists("demo", "proposal.md"));
    assert_eq!(store.read_artifact("demo", "proposal.md").unwrap(), "## Why\n\nBecause.\n");

    // Nested artifact rel paths create parent directories.
    store.write_artifact("demo", "specs/cap-x/spec.md", "## ADDED Requirements\n").unwrap();
    assert!(store.artifact_exists("demo", "specs/cap-x/spec.md"));

    // An empty artifact still exists (done-ness is EXISTS-based).
    store.write_artifact("demo", "design.md", "").unwrap();
    assert!(store.artifact_exists("demo", "design.md"));
    assert_eq!(store.read_artifact("demo", "design.md").unwrap(), "");
}

// --- delta specs ---

#[test]
fn delta_capabilities_require_spec_md_and_sort() {
    let root = TempRoot::new("delta-caps");
    let store = root.store();
    store.create_change("demo", "schema: spec-driven\n").unwrap();

    // Missing specs/ directory: empty list, no capability dirs.
    assert!(store.delta_capabilities("demo").is_empty());
    assert!(!store.has_capability_dirs("demo"));

    // A capability directory WITHOUT spec.md counts for has_capability_dirs
    // but not for delta_capabilities (matches the validate warning logic).
    std::fs::create_dir_all(root.dir.join("openspec/changes/demo/specs/empty-cap")).unwrap();
    assert!(store.has_capability_dirs("demo"));
    assert!(store.delta_capabilities("demo").is_empty());

    store.write_artifact("demo", "specs/zeta-cap/spec.md", "## ADDED Requirements\n").unwrap();
    store.write_artifact("demo", "specs/alpha-cap/spec.md", "## ADDED Requirements\n").unwrap();
    assert_eq!(store.delta_capabilities("demo"), vec!["alpha-cap", "zeta-cap"]);
}

// --- canonical specs ---

#[test]
fn canonical_spec_roundtrip_listing_and_missing_defaults() {
    let root = TempRoot::new("canonical");
    let store = root.store();

    // Missing specs dir: empty list / None / false.
    assert!(store.list_canonical_capabilities().is_empty());
    assert!(store.read_canonical_spec("cap-x").is_none());
    assert!(!store.canonical_spec_exists("cap-x"));

    store.write_canonical_spec("cap-x", "# cap-x Specification\n").unwrap();
    assert!(store.canonical_spec_exists("cap-x"));
    assert_eq!(store.read_canonical_spec("cap-x").unwrap(), "# cap-x Specification\n");
    assert_eq!(
        store.canonical_spec_path("cap-x"),
        root.dir.join("openspec").join("specs").join("cap-x").join("spec.md")
    );

    store.write_canonical_spec("cap-a", "# cap-a\n").unwrap();
    // The trait returns capabilities unsorted (callers sort); readdir order is
    // filesystem-specific (NTFS sorts, APFS does not), so sort before comparing.
    let sorted = |mut caps: Vec<String>| {
        caps.sort();
        caps
    };
    assert_eq!(sorted(store.list_canonical_capabilities()), vec!["cap-a", "cap-x"]);

    // A capability directory without spec.md is not listed.
    std::fs::create_dir_all(root.dir.join("openspec/specs/no-spec-here")).unwrap();
    assert_eq!(sorted(store.list_canonical_capabilities()), vec!["cap-a", "cap-x"]);
}

// --- archive ---

#[test]
fn archive_change_moves_dir_and_meta_is_stampable() {
    let root = TempRoot::new("archive");
    let store = root.store();
    store.create_change("demo", "schema: spec-driven\ncreated: 2026-07-04\n").unwrap();
    store.write_artifact("demo", "tasks.md", "- [x] done\n").unwrap();

    assert!(!store.archived_change_exists("2026-07-04-demo"));
    store.archive_change("demo", "2026-07-04-demo").unwrap();
    store.create_change("older", "schema: spec-driven\n").unwrap();
    store.archive_change("older", "2026-06-01-older").unwrap();
    assert!(store.archived_change_exists("2026-07-04-demo"));
    assert!(!store.change_exists("demo"));
    assert_eq!(
        store.list_archived_changes(),
        vec!["2026-07-04-demo", "2026-06-01-older"]
    );
    // The whole tree moved.
    assert!(root
        .dir
        .join("openspec/changes/archive/2026-07-04-demo/tasks.md".split('/').collect::<PathBuf>())
        .is_file());

    // Stamp archived metadata (read → append → write).
    let mut meta = store.read_archived_meta("2026-07-04-demo").unwrap();
    meta.push_str("archived_at: 2026-07-04\n");
    store.write_archived_meta("2026-07-04-demo", &meta).unwrap();
    assert!(store
        .read_archived_meta("2026-07-04-demo")
        .unwrap()
        .ends_with("archived_at: 2026-07-04\n"));

    // Missing archived change: None.
    assert!(store.read_archived_meta("2026-01-01-ghost").is_none());
}

#[test]
fn archived_artifact_read_and_capability_listing() {
    let root = TempRoot::new("archive-artifacts");
    let store = root.store();
    store.create_change("demo", "schema: spec-driven\ncreated: 2026-07-04\n").unwrap();
    store.write_artifact("demo", "proposal.md", "## Why\n\nArchived body.\n").unwrap();
    store.write_artifact("demo", "tasks.md", "- [x] 1.1 done\n").unwrap();
    store.write_artifact("demo", "specs/cap-b/spec.md", "## ADDED Requirements\n").unwrap();
    store.write_artifact("demo", "specs/cap-a/spec.md", "## MODIFIED Requirements\n").unwrap();
    store.archive_change("demo", "2026-07-04-demo").unwrap();

    // 原文讀取以 dated_name＋output path 定址（read_archived_meta 的對稱擴充）。
    assert_eq!(
        store.read_archived_artifact("2026-07-04-demo", "proposal.md").unwrap(),
        "## Why\n\nArchived body.\n"
    );
    assert_eq!(
        store.read_archived_artifact("2026-07-04-demo", "specs/cap-a/spec.md").unwrap(),
        "## MODIFIED Requirements\n"
    );
    // 缺件 artifact 與不存在的 dated_name 都是 None，不是錯誤。
    assert!(store.read_archived_artifact("2026-07-04-demo", "design.md").is_none());
    assert!(store.read_archived_artifact("2026-01-01-ghost", "proposal.md").is_none());

    // 封存 delta capability 列舉（排序），供規格分頁載入。
    assert_eq!(
        store.archived_delta_capabilities("2026-07-04-demo"),
        vec!["cap-a", "cap-b"]
    );
    assert!(store.archived_delta_capabilities("2026-01-01-ghost").is_empty());
}

// --- workflow config ---

#[test]
fn workflow_config_read_is_optional() {
    let root = TempRoot::new("wf-config");
    let store = root.store();
    assert!(store.read_workflow_config().is_none());
    root.write("openspec/config.yaml", "schema: spec-driven\n");
    assert_eq!(store.read_workflow_config().unwrap(), "schema: spec-driven\n");
}

// --- discussions ---

#[test]
fn discussion_create_append_and_listing() {
    let root = TempRoot::new("discussions");
    let store = root.store();

    // Missing discussions dir: empty lists, no doc.
    assert!(store.list_live_discussions().is_empty());
    assert!(store.list_archived_discussions().is_empty());
    assert!(!store.live_discussion_exists("topic-a"));
    assert!(store.read_live_discussion("topic-a").is_none());
    assert!(store.read_discussion("topic-a").is_none());

    let path = store
        .write_live_discussion("topic-a", "---\nslug: topic-a\nstatus: open\n---\n\n## Rounds\n")
        .unwrap();
    assert_eq!(
        path,
        root.dir.join("openspec").join("discussions").join("topic-a.md")
    );
    assert_eq!(store.live_discussion_path("topic-a"), path);
    assert!(store.live_discussion_exists("topic-a"));

    // Append = read, extend, write back (round appending is engine logic).
    let mut text = store.read_live_discussion("topic-a").unwrap();
    text.push_str("\n### Round 1 — interview (2026-07-04)\n\ncontent\n");
    store.write_live_discussion("topic-a", &text).unwrap();
    let doc = store.read_discussion("topic-a").unwrap();
    assert!(!doc.archived);
    assert!(doc.text.contains("### Round 1"));

    let live = store.list_live_discussions();
    assert_eq!(live.len(), 1);
    assert_eq!(live[0].slug, "topic-a");
    assert!(!live[0].archived);

    // Non-.md files and subdirectories are ignored by the live listing.
    root.write("openspec/discussions/notes.txt", "not a discussion");
    assert_eq!(store.list_live_discussions().len(), 1);
}

#[test]
fn discussion_archive_naming_resolution_and_delete() {
    let root = TempRoot::new("discussion-archive");
    let store = root.store();

    // Archiving a missing discussion is a no-op (None).
    assert!(store.archive_discussion("ghost", "2026-07-04").unwrap().is_none());

    store.write_live_discussion("topic-a", "---\nslug: topic-a\n---\nfirst\n").unwrap();
    let name = store.archive_discussion("topic-a", "2026-07-01").unwrap().unwrap();
    assert_eq!(name, "2026-07-01-topic-a.md");
    assert!(!store.live_discussion_exists("topic-a"));
    assert!(store.archived_discussion_exists("topic-a"));

    // Same-day slug reuse gets a -N suffix instead of failing.
    store.write_live_discussion("topic-a", "---\nslug: topic-a\n---\nsecond\n").unwrap();
    let name2 = store.archive_discussion("topic-a", "2026-07-01").unwrap().unwrap();
    assert_eq!(name2, "2026-07-01-topic-a-2.md");

    // read_discussion resolves live first, then the NEWEST archived candidate.
    let doc = store.read_discussion("topic-a").unwrap();
    assert!(doc.archived);
    assert!(doc.text.contains("second"));
    store.write_live_discussion("topic-a", "---\nslug: topic-a\n---\nlive again\n").unwrap();
    let doc = store.read_discussion("topic-a").unwrap();
    assert!(!doc.archived);
    assert!(doc.text.contains("live again"));

    // Archived listing derives the slug from the date-prefixed stem; a same-day
    // `-N` reuse suffix is kept (matches the CLI's long-standing listing), and
    // entries come in stored-name order ("-2.md" sorts before ".md").
    let archived = store.list_archived_discussions();
    assert_eq!(archived.len(), 2);
    assert!(archived.iter().all(|d| d.archived));
    let slugs: Vec<&str> = archived.iter().map(|d| d.slug.as_str()).collect();
    assert_eq!(slugs, vec!["topic-a-2", "topic-a"]);

    // Deleting a live discussion removes only the live document.
    store.delete_live_discussion("topic-a").unwrap();
    assert!(!store.live_discussion_exists("topic-a"));
    assert!(store.archived_discussion_exists("topic-a"));
}

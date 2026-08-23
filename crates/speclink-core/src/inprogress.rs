//! In-progress change markers — the "started" station of the change lifecycle
//! (created → started → archived), stored in the change's own metadata
//! document so the marker travels with the repository and rides whatever
//! backend the [`Store`] provides. (The pre-migration SQLite database under
//! `.git/speclink-app/` was host-local and had zero readers; leftovers on
//! existing machines are harmless and intentionally not migrated.)
//!
//! The command surface is parity-sensitive: add is silent, idempotent, and
//! succeeds for unknown change names (measured pre-migration baseline —
//! exit 0, no output, and in that case nothing is written).

use crate::model::ChangeMeta;
use crate::store::Store;
use crate::util;
use anyhow::Result;

/// 守門拒絕:change 帶工作痕跡(已勾任務或 touched 記錄),in-progress 標記
/// 不可機械移除。結構化證據隨錯誤走——CLI stderr、server 409 載荷與 desktop
/// 對話框各自呈現;Display 即 CLI 的人眼文字。
#[derive(Debug, Clone)]
pub struct RevertBlocked {
    pub change: String,
    /// tasks.md 的已勾任務數(缺檔視為 0)。
    pub checked_tasks: usize,
    /// touched 記錄 v1 與 v2 兩清單的檔案聯集(去重、首見序)。
    pub touched_files: Vec<String>,
}

impl std::fmt::Display for RevertBlocked {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "cannot remove the in-progress marker for '{}': work traces exist",
            self.change
        )?;
        if self.checked_tasks > 0 {
            write!(
                f,
                "\n  checked tasks: {} — uncheck them (speclink task undone) and retry",
                self.checked_tasks
            )?;
        }
        if !self.touched_files.is_empty() {
            write!(f, "\n  touched files: {}", self.touched_files.join(", "))?;
            write!(
                f,
                "\n  touched records may mix work from other changes — have an agent or a human judge them (no mechanical cleanup)"
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for RevertBlocked {}

/// Remove the in-progress marker — the reverse verb of [`add`], for changes
/// started by mistake. Gated on zero work traces; see the change-lifecycle
/// spec「in-progress 標記可自 change meta 移除(零工作痕跡守門)」.
///
/// Returns whether THIS call removed the marker — `false` for the idempotent
/// not-started success (zero writes, no event). Unknown names error loudly,
/// deliberately asymmetric with [`add`]'s parity-frozen silence.
pub fn remove(store: &dyn Store, name: &str) -> Result<bool> {
    // Same single-path-segment guard as [`add`], but loud: a name with
    // separators is never an active change, and a correction verb aimed at the
    // wrong name must say so instead of silently writing outside changes/.
    let not_found = || anyhow::anyhow!("Change '{name}' not found.");
    if name.contains(['/', '\\', ':']) || name.contains("..") {
        return Err(not_found());
    }
    let Some(meta) = store.read_change_meta(name) else {
        return Err(not_found());
    };
    // Fail-closed gate: a corrupt document must not read as "not started" and
    // take the idempotent pass — refuse before any decision.
    let parsed = ChangeMeta::from_text(Some(&meta)).map_err(|reason| crate::model::MetaError {
        change: name.to_string(),
        reason,
    })?;
    // Zero-work-trace gate, judged before idempotence: a change whose stage
    // still derives as in-progress (checked tasks without a stamp) must hear
    // "blocked, traces exist", not "already proposed".
    let tasks = crate::tasks::parse(&store.read_artifact(name, "tasks.md").unwrap_or_default());
    let checked_tasks = tasks.iter().filter(|t| t.done).count();
    let touched_files = crate::tasks::TouchedRecord::load(store, name).all_files();
    if checked_tasks > 0 || !touched_files.is_empty() {
        return Err(RevertBlocked {
            change: name.to_string(),
            checked_tasks,
            touched_files,
        }
        .into());
    }
    if parsed.started_at.is_none() && parsed.started_by.is_none() && parsed.started_with.is_none() {
        return Ok(false);
    }
    // Line filter, mirroring add's append: drop exactly the three started_*
    // lines and keep every other byte as-is — never re-serialize.
    let kept: String = meta
        .split_inclusive('\n')
        .filter(|line| {
            !["started_at:", "started_by:", "started_with:"]
                .iter()
                .any(|field| line.starts_with(field))
        })
        .collect();
    store.write_change_meta(name, &kept)?;
    Ok(true)
}

/// Mark a change as in-progress by stamping `started_at` / `started_by` /
/// `started_with` into its metadata document (read → append → write, never
/// re-serialized). Identity and agent attribution follow the created_* rule:
/// what the caller cannot attribute is absent, not defaulted. A change already
/// carrying a started_* field keeps its first stamp verbatim.
///
/// Returns whether THIS call stamped the marker — `false` for the silent
/// no-op successes (unknown name, already stamped), so the command layer can
/// tell a mutation from an idempotent pass and only report an event for the
/// former. Output and exit behavior stay parity-frozen either way.
pub fn add(store: &dyn Store, name: &str, identity: Option<&str>, agent: Option<&str>) -> Result<bool> {
    // Active change names are single path segments by construction (directory
    // entries); anything else could address a metadata document outside
    // changes/ through the raw write pair. Baseline shape for any unknown
    // name is a silent success with zero file effects — keep it.
    if name.contains(['/', '\\', ':']) || name.contains("..") {
        return Ok(false);
    }
    let Some(mut meta) = store.read_change_meta(name) else {
        return Ok(false);
    };
    // Fail-closed gate: a corrupt document must not read as "not started" and
    // take the stamp append — refuse before any text surgery.
    let parsed = ChangeMeta::from_text(Some(&meta)).map_err(|reason| crate::model::MetaError {
        change: name.to_string(),
        reason,
    })?;
    if parsed.started_at.is_some() || parsed.started_by.is_some() || parsed.started_with.is_some() {
        return Ok(false);
    }
    if !meta.ends_with('\n') && !meta.is_empty() {
        meta.push('\n');
    }
    let clean = util::yaml_scalar;
    meta.push_str(&format!("started_at: {}\n", util::today()));
    if let Some(id) = identity {
        meta.push_str(&format!("started_by: {}\n", clean(id)));
    }
    if let Some(agent) = agent {
        meta.push_str(&format!("started_with: {}\n", clean(agent)));
    }
    store.write_change_meta(name, &meta)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use crate::store::Store;
    use crate::teststore::TestStore;

    const EXISTING_META: &str = "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: Base Line <base@example.com>\ncreated_with: claude\nfrom_discussion: 桌面即時刷新與封存瀏覽\n";

    #[test]
    fn add_appends_started_fields_and_preserves_existing_fields_verbatim() {
        let store = TestStore::with_meta("demo", EXISTING_META);
        super::add(&store, "demo", Some("Tester <t@example.com>"), Some("claude")).unwrap();

        let meta = store.meta("demo");
        assert!(
            meta.starts_with(EXISTING_META),
            "existing fields must be preserved byte-for-byte, got: {meta}"
        );
        let today = crate::util::today();
        assert_eq!(
            meta[EXISTING_META.len()..],
            format!("started_at: {today}\nstarted_by: Tester <t@example.com>\nstarted_with: claude\n")
        );
        // The stamped document parses and exposes the started fields.
        let parsed = crate::model::ChangeMeta::from_text(Some(&meta)).expect("stamped meta parses");
        assert_eq!(parsed.started_at.as_deref(), Some(today.as_str()));
        assert_eq!(parsed.started_by.as_deref(), Some("Tester <t@example.com>"));
        assert_eq!(parsed.started_with.as_deref(), Some("claude"));
    }

    #[test]
    fn add_preserves_existing_board_rank_verbatim() {
        // spec「meta 寫入路徑對 board_rank 互不破壞」反向：開工標記作用於
        // 含 board_rank 的 meta 時原樣保留該欄位，開工欄位如常寫入。
        let with_rank = "schema: spec-driven\ncreated: 2026-07-01\nboard_rank: n\n";
        let store = TestStore::with_meta("demo", with_rank);
        super::add(&store, "demo", None, None).unwrap();
        let meta = store.meta("demo");
        assert!(
            meta.starts_with(with_rank),
            "board_rank must survive the started stamp byte-for-byte: {meta}"
        );
        let parsed = crate::model::ChangeMeta::from_text(Some(&meta)).expect("stamped meta parses");
        assert_eq!(parsed.board_rank.as_deref(), Some("n"));
        assert!(parsed.started_at.is_some());
    }

    #[test]
    fn add_without_identity_or_agent_stamps_only_started_at() {
        // Same mechanism as created_by / created_with: fields the caller cannot
        // attribute are absent, not defaulted.
        let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
        super::add(&store, "demo", None, None).unwrap();

        let meta = store.meta("demo");
        assert!(meta.contains("started_at: "));
        assert!(!meta.contains("started_by"));
        assert!(!meta.contains("started_with"));
    }

    #[test]
    fn add_handles_meta_missing_trailing_newline() {
        let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01");
        super::add(&store, "demo", None, None).unwrap();
        let meta = store.meta("demo");
        assert!(
            meta.contains("created: 2026-07-01\nstarted_at: "),
            "a missing trailing newline must not glue fields together: {meta}"
        );
    }

    #[test]
    fn repeated_add_is_idempotent_and_keeps_the_first_stamp() {
        let store = TestStore::with_meta("demo", EXISTING_META);
        super::add(&store, "demo", Some("First <first@example.com>"), Some("claude")).unwrap();
        let after_first = store.meta("demo");

        // A second add — even with a different identity/agent — must not
        // rewrite the first work stamp, and must not write at all.
        super::add(&store, "demo", Some("Second <second@example.com>"), Some("codex")).unwrap();
        assert_eq!(store.meta("demo"), after_first, "first stamp must be kept verbatim");
        assert_eq!(*store.meta_writes.borrow(), 1, "idempotent re-add must not write");
    }

    #[test]
    fn add_sanitizes_control_characters_out_of_identity_and_agent() {
        // Sharp edge (Scoundrel): a git user.name carrying a newline would
        // otherwise inject arbitrary YAML fields into the metadata document.
        let store = TestStore::with_meta("demo", "schema: spec-driven\n");
        super::add(
            &store,
            "demo",
            Some("Evil\nstarted_with: forged\rname"),
            Some("agent\nfrom_discussion: fake"),
        )
        .unwrap();
        let meta = store.meta("demo");
        // 結構性斷言：不得出現被注入的獨立欄位行；文件仍可解析且值完整落在
        // 對應欄位內（風險值以引號包覆，不破壞整檔解析）。
        assert!(
            meta.lines().all(|l| !l.starts_with("from_discussion:")),
            "newline in agent must not inject a field line: {meta}"
        );
        let parsed = crate::model::ChangeMeta::from_text(Some(&meta)).expect("stamped meta parses");
        assert_eq!(parsed.started_by.as_deref(), Some("Evil started_with: forged name"));
        assert_eq!(parsed.started_with.as_deref(), Some("agent from_discussion: fake"));
        assert!(parsed.from_discussion.is_none());
        assert_eq!(parsed.schema.as_deref(), Some("spec-driven"), "document must keep parsing");
    }

    #[test]
    fn add_refuses_on_corrupt_meta_without_writing() {
        // spec「壞 metadata 使生命週期寫入 fail closed」：壞檔不得被解讀為
        // 未開工而疊寫 started_* 行——拒絕、指名檔案，.openspec.yaml 逐位元不變。
        const BAD: &str = ": : :\n\t bad yaml [unclosed\n";
        let store = TestStore::with_meta("demo", BAD);
        let err = super::add(&store, "demo", Some("T <t@example.com>"), Some("claude"))
            .expect_err("corrupt meta must refuse the stamp");
        assert!(
            err.to_string().contains("openspec/changes/demo/.openspec.yaml"),
            "error must name the metadata file: {err}"
        );
        assert_eq!(store.meta("demo"), BAD, "meta byte-identical");
        assert_eq!(*store.meta_writes.borrow(), 0, "refusal must not write");
    }

    #[test]
    fn add_refuses_to_address_outside_a_single_path_segment() {
        // Sharp edge (Scoundrel): `in-progress add "../evil"` must not stamp a
        // metadata document outside changes/. Baseline behavior for any unknown
        // name is a silent success with zero file effects — keep that shape.
        let store = TestStore::with_meta("../evil", "schema: spec-driven\n");
        assert!(super::add(&store, "../evil", None, None).is_ok());
        assert!(super::add(&store, "a/b", None, None).is_ok());
        assert!(super::add(&store, "C:\\evil", None, None).is_ok());
        assert_eq!(*store.meta_writes.borrow(), 0, "non-segment names must not write");
    }

    #[test]
    fn add_for_missing_change_is_a_silent_no_op() {
        // Pre-migration baseline (measured): unknown names succeed silently
        // with exit 0 and no output. The meta flow keeps that shape — no
        // error, and nothing written anywhere.
        let store = TestStore::default();
        assert!(super::add(&store, "ghost", Some("T <t@example.com>"), None).is_ok());
        assert_eq!(*store.meta_writes.borrow(), 0);
        assert!(store.read_change_meta("ghost").is_none());
    }

    // --- remove:反向動詞與零工作痕跡守門 ---

    use crate::tasks::{EvidenceEntry, TouchedEntry, TouchedRecord};

    /// 帶開工戳記的 meta:started_* 三行之後仍有欄位(board_rank),
    /// 逼出「行過濾」而非「截尾」的實作。
    const STARTED_META: &str = "schema: spec-driven\ncreated: 2026-07-01\ncreated_by: Base Line <base@example.com>\ncreated_with: claude\nfrom_discussion: 桌面即時刷新與封存瀏覽\nstarted_at: 2026-07-30\nstarted_by: Tester <t@example.com>\nstarted_with: claude\nboard_rank: n\n";

    fn v1_entry(files: &[&str]) -> TouchedEntry {
        TouchedEntry {
            task_id: "1".to_string(),
            task_desc: "1.1 first task".to_string(),
            files: files.iter().map(|f| f.to_string()).collect(),
        }
    }

    fn v2_entry(files: &[&str]) -> EvidenceEntry {
        EvidenceEntry {
            task_id: "tsk_0000000000000000".to_string(),
            task_desc: "1.1 first task".to_string(),
            actor: None,
            repo: None,
            head_commit: None,
            touched_files: files.iter().map(|f| f.to_string()).collect(),
            recorded_at: "2026-07-30T00:00:00Z".to_string(),
        }
    }

    fn save_touched(store: &TestStore, change: &str, v1: Vec<TouchedEntry>, v2: Vec<EvidenceEntry>) {
        let mut rec = TouchedRecord::load(store, change);
        rec.change = change.to_string();
        rec.touched = v1;
        rec.entries = v2;
        rec.save(store).unwrap();
    }

    fn blocked(err: &anyhow::Error) -> &super::RevertBlocked {
        err.downcast_ref::<super::RevertBlocked>()
            .unwrap_or_else(|| panic!("gate refusal must carry RevertBlocked evidence, got: {err}"))
    }

    #[test]
    fn remove_strips_started_lines_and_preserves_the_rest_verbatim() {
        // 零痕跡:無 tasks.md(缺檔視為 0)、無 touched 記錄(缺檔視為空)。
        let store = TestStore::with_meta("demo", STARTED_META);

        let removed = super::remove(&store, "demo").unwrap();

        assert!(removed, "a stamped change with zero traces must actually remove");
        assert_eq!(
            store.meta("demo"),
            format!("{EXISTING_META}board_rank: n\n"),
            "started_* lines removed, every other line byte-identical (no re-serialization)"
        );
        let parsed = crate::model::ChangeMeta::from_text(Some(&store.meta("demo"))).unwrap();
        assert!(parsed.started_at.is_none());
        assert!(parsed.started_by.is_none());
        assert!(parsed.started_with.is_none());
    }

    #[test]
    fn remove_refuses_when_tasks_are_checked() {
        let store = TestStore::with_meta("demo", STARTED_META);
        store.put_artifact("demo", "tasks.md", "## 1. G\n\n- [x] 1.1 a\n- [x] 1.2 b\n- [ ] 1.3 c\n");

        let err = super::remove(&store, "demo").expect_err("checked tasks must block");

        let evidence = blocked(&err);
        assert_eq!(evidence.change, "demo");
        assert_eq!(evidence.checked_tasks, 2);
        assert!(evidence.touched_files.is_empty());
        assert_eq!(store.meta("demo"), STARTED_META, "refusal must not touch the meta");
        assert_eq!(*store.meta_writes.borrow(), 0, "refusal must not write");
    }

    #[test]
    fn remove_refuses_when_touched_v1_list_is_nonempty() {
        let store = TestStore::with_meta("demo", STARTED_META);
        save_touched(&store, "demo", vec![v1_entry(&["src/a.rs"])], vec![]);

        let err = super::remove(&store, "demo").expect_err("v1 touched must block");

        let evidence = blocked(&err);
        assert_eq!(evidence.checked_tasks, 0);
        assert_eq!(evidence.touched_files, vec!["src/a.rs"]);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn remove_refuses_when_v2_evidence_entries_are_nonempty() {
        let store = TestStore::with_meta("demo", STARTED_META);
        save_touched(&store, "demo", vec![], vec![v2_entry(&["src/b.rs"])]);

        let err = super::remove(&store, "demo").expect_err("v2 entries must block");

        let evidence = blocked(&err);
        assert_eq!(evidence.touched_files, vec!["src/b.rs"]);
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn remove_evidence_unions_and_dedupes_both_touched_lists() {
        // spec Example「證據清單為兩版記錄的聯集去重」:v1 含 a、b,
        // v2 含 b、c → 恰為 a、b、c 三項,首見序,無重複。
        let store = TestStore::with_meta("demo", STARTED_META);
        save_touched(
            &store,
            "demo",
            vec![v1_entry(&["src/a.rs", "src/b.ts"])],
            vec![v2_entry(&["src/b.ts", "src/c.rs"])],
        );

        let err = super::remove(&store, "demo").expect_err("traces must block");

        assert_eq!(blocked(&err).touched_files, vec!["src/a.rs", "src/b.ts", "src/c.rs"]);
    }

    #[test]
    fn remove_for_unknown_change_errors_loudly() {
        // 與 add 的靜默刻意不對稱:修正動作打錯名字必須明確報錯。
        let store = TestStore::default();

        let err = super::remove(&store, "ghost").expect_err("unknown change must error");

        let msg = err.to_string();
        assert!(msg.contains("ghost") && msg.contains("not found"), "error names the change: {msg}");
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn remove_refuses_to_address_outside_a_single_path_segment() {
        // Sharp edge (Scoundrel): `in-progress remove "../evil"` must not write
        // a metadata document outside changes/ — non-segment names are never
        // active changes, so they take the loud not-found path.
        let store = TestStore::with_meta("../evil", STARTED_META);

        let err = super::remove(&store, "../evil").expect_err("non-segment name must error");

        assert!(err.to_string().contains("not found"), "not-found shape: {err}");
        assert_eq!(*store.meta_writes.borrow(), 0);
    }

    #[test]
    fn remove_on_an_unstarted_change_is_idempotent_with_zero_writes() {
        let store = TestStore::with_meta("demo", EXISTING_META);

        let removed = super::remove(&store, "demo").unwrap();

        assert!(!removed, "nothing to remove must report false (no event upstream)");
        assert_eq!(store.meta("demo"), EXISTING_META, "meta byte-identical");
        assert_eq!(*store.meta_writes.borrow(), 0, "idempotent pass must not write");
    }

    #[test]
    fn remove_refuses_on_corrupt_meta_without_writing() {
        // Fail-closed 與 add 同款:壞檔不得被解讀為未開工而冪等放行。
        const BAD: &str = ": : :\n\t bad yaml [unclosed\n";
        let store = TestStore::with_meta("demo", BAD);

        let err = super::remove(&store, "demo").expect_err("corrupt meta must refuse");

        assert!(
            err.to_string().contains("openspec/changes/demo/.openspec.yaml"),
            "error must name the metadata file: {err}"
        );
        assert_eq!(store.meta("demo"), BAD, "meta byte-identical");
        assert_eq!(*store.meta_writes.borrow(), 0, "refusal must not write");
    }
}

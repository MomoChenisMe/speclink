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

/// Mark a change as in-progress by stamping `started_at` / `started_by` /
/// `started_with` into its metadata document (read → append → write, never
/// re-serialized). Identity and agent attribution follow the created_* rule:
/// what the caller cannot attribute is absent, not defaulted. A change already
/// carrying a started_* field keeps its first stamp verbatim.
pub fn add(store: &dyn Store, name: &str, identity: Option<&str>, agent: Option<&str>) -> Result<()> {
    // Active change names are single path segments by construction (directory
    // entries); anything else could address a metadata document outside
    // changes/ through the raw write pair. Baseline shape for any unknown
    // name is a silent success with zero file effects — keep it.
    if name.contains(['/', '\\', ':']) || name.contains("..") {
        return Ok(());
    }
    let Some(mut meta) = store.read_change_meta(name) else {
        return Ok(());
    };
    let parsed = ChangeMeta::from_text(Some(&meta));
    if parsed.started_at.is_some() || parsed.started_by.is_some() || parsed.started_with.is_some() {
        return Ok(());
    }
    if !meta.ends_with('\n') && !meta.is_empty() {
        meta.push('\n');
    }
    // A newline inside the identity/agent string would inject arbitrary YAML
    // fields; a bare `:`/leading indicator would break the whole document's
    // parse (which silently falls back to defaults). Flatten control
    // characters and double-quote values a plain scalar cannot carry.
    let clean = |s: &str| {
        let flat = s.replace(|c: char| c.is_control(), " ");
        let risky = flat.is_empty()
            || flat.contains([':', '#', '"'])
            || flat.ends_with(' ')
            || flat.starts_with([' ', '[', '{', '\'', '&', '*', '!', '|', '>', '%', '@', '`', '-', '?']);
        if risky {
            format!("\"{}\"", flat.replace('\\', "\\\\").replace('"', "\\\""))
        } else {
            flat
        }
    };
    meta.push_str(&format!("started_at: {}\n", util::today()));
    if let Some(id) = identity {
        meta.push_str(&format!("started_by: {}\n", clean(id)));
    }
    if let Some(agent) = agent {
        meta.push_str(&format!("started_with: {}\n", clean(agent)));
    }
    store.write_change_meta(name, &meta)?;
    Ok(())
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
        let parsed = crate::model::ChangeMeta::from_text(Some(&meta));
        assert_eq!(parsed.started_at.as_deref(), Some(today.as_str()));
        assert_eq!(parsed.started_by.as_deref(), Some("Tester <t@example.com>"));
        assert_eq!(parsed.started_with.as_deref(), Some("claude"));
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
        let parsed = crate::model::ChangeMeta::from_text(Some(&meta));
        assert_eq!(parsed.started_by.as_deref(), Some("Evil started_with: forged name"));
        assert_eq!(parsed.started_with.as_deref(), Some("agent from_discussion: fake"));
        assert!(parsed.from_discussion.is_none());
        assert_eq!(parsed.schema.as_deref(), Some("spec-driven"), "document must keep parsing");
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
}

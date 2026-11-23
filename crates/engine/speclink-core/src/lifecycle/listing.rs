//! The `list --json` serialization path, shared by the CLI and the Node SDK so
//! the two surfaces cannot drift (their parity is additionally locked by the
//! SDK's fixture comparison tests).

use crate::command::{ClaimRecord, LandedRecord};
use crate::model::Change;
use crate::station::{self, StationStatus};
use crate::store::{DocRef, Store};
use serde::Serialize;
use std::collections::BTreeMap;

/// One change entry of `list --json` (frozen field order).
#[derive(Debug, Serialize)]
pub struct ListChangeJson {
    #[serde(rename = "completedTasks")]
    pub completed_tasks: usize,
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub summary: String,
    #[serde(rename = "totalTasks")]
    pub total_tasks: usize,
    /// Discussions this change reflected then went stale against — re-concluded after
    /// seal, pending re-ingest (speclink extension). Omitted when empty so the common-case
    /// `list --json` output stays byte-identical to the frozen baseline.
    #[serde(rename = "restaleFrom", skip_serializing_if = "Vec::is_empty")]
    pub restale_from: Vec<String>,
    /// Parse-failure reason when the change's `.openspec.yaml` is corrupt
    /// (fail-closed diagnostic). Omitted when valid/absent so every existing
    /// consumer's payload shape is untouched.
    #[serde(rename = "metaError", skip_serializing_if = "Option::is_none")]
    pub meta_error: Option<String>,
    /// Capability names of the change's delta specs, ascending, from the store's
    /// `delta_capabilities`. Under the main-checkout worktree overlay that store
    /// reads a mapped change from its worktree copy (like every other value here),
    /// while `show --json`'s `deltaSpecs` reads the main copy. Omitted when the
    /// change has none, so a delta-less change's payload stays byte-identical.
    #[serde(rename = "deltaCapabilities", skip_serializing_if = "Vec::is_empty")]
    pub delta_capabilities: Vec<String>,
    /// The creation date of the change's metadata, `YYYY-MM-DD`. Absent when
    /// the metadata has none or does not parse.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// When a document of the change was last modified, as an RFC 3339 UTC
    /// time in whole seconds — the store's `updated_at_secs`. Absent when the
    /// store cannot tell; never 0 or an empty string. On a local checkout it
    /// is a file time, which a checkout or a copy also moves: a hint for "how
    /// long has this been idle", not a fact to decide on.
    #[serde(rename = "modifiedAt", skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
    /// The change's free fields (host-free-fields D5). Omitted when the change
    /// has none, so a project that uses no free field keeps its output byte
    /// for byte.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub fields: crate::fields::Fields,
    /// The detail keys (`list --json --detail`): what the change has on
    /// record, each in the shape `show --json` gives it. All of them stay
    /// unset, and out of the output, unless the list was asked for detail;
    /// with detail each one is there only when the change has that record.
    ///
    /// The two quality stations. A station with no ticket and no stamp has
    /// nothing recorded and is not serialized.
    #[serde(skip_serializing_if = "station_unrecorded", serialize_with = "station_json")]
    pub review: Option<StationStatus>,
    #[serde(skip_serializing_if = "station_unrecorded", serialize_with = "station_json")]
    pub verify: Option<StationStatus>,
    /// Who holds the change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claimed: Option<ClaimRecord>,
    /// The branch recorded when work started.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// The landing record. The list carries the commit only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub landed: Option<ListLandedJson>,
    /// Who created the change, as its metadata records it.
    #[serde(rename = "createdBy", skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    /// The discussions the change came from, in the order of its metadata
    /// (the order `show --json` gives them).
    #[serde(rename = "fromDiscussions", skip_serializing_if = "Vec::is_empty")]
    pub from_discussions: Vec<String>,
    /// The linked worktree this change is being implemented in (local main
    /// checkout only, worktree policy on). Last in the field order and omitted
    /// when absent, so every existing consumer's payload stays byte-identical.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree: Option<ListWorktreeJson>,
}

/// The `worktree` object of a `list --json` entry: where the change is being
/// implemented and on which branch. Both field names are already camelCase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListWorktreeJson {
    /// Absolute path of the worktree directory, rendered by the assembly point
    /// (the engine never handles a storage path itself).
    pub path: String,
    /// Full branch name, e.g. `speclink/add-dark-mode`.
    pub branch: String,
}

/// The `landed` object of a `list --json --detail` entry: the commit alone.
/// The report time and the reporter are in `show --json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListLandedJson {
    pub commit: String,
}

fn station_unrecorded(status: &Option<StationStatus>) -> bool {
    !status.as_ref().is_some_and(StationStatus::is_recorded)
}

fn station_json<S: serde::Serializer>(status: &Option<StationStatus>, serializer: S) -> Result<S::Ok, S::Error> {
    status.as_ref().map(StationStatus::to_list_json).serialize(serializer)
}

/// Order changes for listing (frozen ordering contract):
/// - "name": alphabetical.
/// - "created": changes with a VALID metadata pair (schema AND created both present) come
///   first, created descending, mtime-then-name tiebreak; invalid-metadata changes follow
///   in modified order.
/// - everything else (default "modified", unknown values): newest file mtime inside the
///   change, whole seconds, newest first, name-ascending ties.
pub fn sort_changes(store: &dyn Store, changes: &mut [Change], sort: &str) {
    let mtime_desc = |x: &Change, y: &Change| {
        let mx = store.updated_at_secs(&x.name);
        let my = store.updated_at_secs(&y.name);
        my.cmp(&mx).then_with(|| x.name.cmp(&y.name))
    };
    match sort {
        "name" => changes.sort_by(|x, y| x.name.cmp(&y.name)),
        "created" => changes.sort_by(|x, y| {
            let valid = |c: &Change| match (&c.meta.schema, &c.meta.created) {
                (Some(_), Some(created)) => Some(created.clone()),
                _ => None,
            };
            match (valid(x), valid(y)) {
                (Some(a), Some(b)) => b.cmp(&a).then_with(|| mtime_desc(x, y)),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => mtime_desc(x, y),
            }
        }),
        _ => changes.sort_by(mtime_desc),
    }
}

fn truncate_summary(text: &str, limit: usize) -> String {
    let first_line = text.trim();
    if first_line.chars().count() <= limit {
        return first_line.to_string();
    }
    // Take the first `limit` characters verbatim (no word-boundary, no trim) and append an ellipsis.
    let head: String = first_line.chars().take(limit).collect();
    format!("{head}…")
}

/// The one-line change summary of `list`: first prose line after "## Why"
/// (fallback: first prose line anywhere), truncated to 30 characters.
pub fn proposal_summary(store: &dyn Store, change: &Change) -> String {
    let proposal = store.read_artifact(&change.name, "proposal.md").unwrap_or_default();
    // First non-empty, non-header line after "## Why" (or first prose line).
    let mut after_why = false;
    for line in proposal.lines() {
        let t = line.trim();
        if t.starts_with("## Why") {
            after_why = true;
            continue;
        }
        if after_why && !t.is_empty() && !t.starts_with('#') {
            return truncate_summary(t, 30);
        }
    }
    // Fallback: first prose line.
    for line in proposal.lines() {
        let t = line.trim();
        if !t.is_empty() && !t.starts_with('#') && !t.starts_with("<!--") {
            return truncate_summary(t, 30);
        }
    }
    String::new()
}

/// (complete, total) checkbox counts of a change's tasks.md.
pub fn task_counts(store: &dyn Store, change: &Change) -> (usize, usize) {
    let c = crate::tasks::counts_for(store, &change.name);
    (c.complete, c.total)
}

/// The `changes` items of `list --json`, in the given (already sorted) order.
pub fn changes_json(store: &dyn Store, changes: &[Change]) -> Vec<ListChangeJson> {
    changes_json_detail(store, changes, &BTreeMap::new(), false)
}

/// `changes_json` plus the local worktree observation surface: entries named in
/// `worktrees` gain the `worktree` object. Callers without worktree facts use
/// [`changes_json`] — an empty map is byte-for-byte the frozen output, by
/// construction rather than by convention.
pub fn changes_json_with(
    store: &dyn Store,
    changes: &[Change],
    worktrees: &BTreeMap<String, ListWorktreeJson>,
) -> Vec<ListChangeJson> {
    changes_json_detail(store, changes, worktrees, false)
}

/// [`changes_json_with`], and with `detail` the detail keys of every change
/// whose metadata reads. Each comes from where `show` takes it: the
/// stations from [`station::recorded_status`], the claim and the landing from
/// the same two records. Without `detail` nothing here runs, so the output and
/// what the store is asked for are the ones of [`changes_json_with`].
pub fn changes_json_detail(
    store: &dyn Store,
    changes: &[Change],
    worktrees: &BTreeMap<String, ListWorktreeJson>,
    detail: bool,
) -> Vec<ListChangeJson> {
    if detail {
        // Every open ticket is read for its rounds: one batch for a store that
        // pays per call. A change whose metadata is broken reports no station.
        let tickets: Vec<DocRef<'_>> = changes
            .iter()
            .filter(|c| c.meta_error.is_none())
            .flat_map(|c| {
                [&station::REVIEW, &station::VERIFY].map(|st| DocRef::Artifact { change: &c.name, artifact: st.doc })
            })
            .collect();
        store.prefetch(&tickets);
    }
    changes
        .iter()
        .map(|c| {
            let (complete, total) = task_counts(store, c);
            // A corrupt metadata document reads as the default one, which
            // holds no claim, branch or landing; `recorded_status` gives no
            // station for it, also when a ticket lies beside it.
            let meta = detail.then_some(&c.meta);
            ListChangeJson {
                completed_tasks: complete,
                name: c.name.clone(),
                // "done" only when every task is checked (and there is at least one).
                status: if total > 0 && complete == total {
                    "done".to_string()
                } else {
                    "in-progress".to_string()
                },
                summary: proposal_summary(store, c),
                total_tasks: total,
                restale_from: c.meta.restale_from(),
                meta_error: c.meta_error.clone(),
                delta_capabilities: store.delta_capabilities(&c.name),
                created: c.meta.created.clone(),
                modified_at: modified_at(store.updated_at_secs(&c.name)),
                fields: c.meta.fields.clone(),
                review: meta.and_then(|_| station::recorded_status(&station::REVIEW, store, c)),
                verify: meta.and_then(|_| station::recorded_status(&station::VERIFY, store, c)),
                claimed: meta.and_then(|m| {
                    ClaimRecord::new(m.claimed_by.clone(), m.claimed_by_id.clone(), m.claimed_at.clone())
                }),
                branch: meta.and_then(|m| m.branch.clone()),
                landed: meta
                    .and_then(|m| LandedRecord::new(m.landed_commit.clone(), m.landed_at.clone(), m.landed_by.clone()))
                    .map(|landed| ListLandedJson { commit: landed.commit }),
                created_by: meta.and_then(|m| m.created_by.clone()),
                from_discussions: meta.map(|m| m.from_discussions()).unwrap_or_default(),
                worktree: worktrees.get(&c.name).cloned(),
            }
        })
        .collect()
}

/// Whole seconds since the Unix epoch as an RFC 3339 UTC time. 0 is a store
/// that cannot tell, so it gives nothing; a value past what RFC 3339 can
/// state (year 9999) gives nothing either.
fn modified_at(secs: u64) -> Option<String> {
    use chrono::Datelike;
    if secs == 0 {
        return None;
    }
    let at = chrono::DateTime::from_timestamp(i64::try_from(secs).ok()?, 0)?;
    (at.year() <= 9999).then(|| at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
}

/// One item of `list --archived --json`: an archived change. The order of the
/// fields is the output order; every key past `verifyOutcome` is there only
/// when it has a value.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListArchivedJson {
    pub dated_name: String,
    /// `YYYY-MM-DD` from the start of `dated_name`; empty when the name has
    /// no date.
    pub date: String,
    /// `dated_name` without its date; the whole name when it has none.
    pub name: String,
    pub review_outcome: ListArchivedOutcome,
    pub verify_outcome: ListArchivedOutcome,
    /// The two task counts, both or neither: there when the archived change
    /// has a task list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks_total: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks_done: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub from_discussions: Vec<String>,
    /// The capabilities of the archived change's delta specs, ascending.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub delta_capabilities: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub fields: crate::fields::Fields,
}

/// A station's outcome on an archived change, as the archived list prints it:
/// `none`, `stamped` or `notPassed` for the outcomes this Engine names, and
/// the text as it came for one that only a newer server names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum ListArchivedOutcome {
    Known(station::ArchivedOutcome),
    Other(String),
}

/// The items of `list --archived --json`, newest dated name first. One
/// implementation for every entry point: the CLI and the host kit run it as
/// the `list` command, and the remote door's archived list calls it on its
/// store view.
///
/// Each item is judged by the archived documents alone: the task counts from
/// the task list, the two outcomes by [`station::ArchivedOutcome::of`] (the
/// station's ticket archived with the change, else a stamp in the metadata,
/// else nothing). Metadata that does not parse reads as having no stamp,
/// creator, source discussion or field; the item is still listed.
pub fn archived_json(store: &dyn Store) -> Vec<ListArchivedJson> {
    let mut names = store.list_archived_changes();
    names.sort_by(|a, b| b.cmp(a));
    // Every document an item reads, in one batch for a store that pays per call.
    let docs: Vec<DocRef<'_>> = names
        .iter()
        .flat_map(|dated_name| {
            [
                DocRef::ArchivedMeta { dated_name },
                DocRef::ArchivedArtifact { dated_name, artifact: "tasks.md" },
                DocRef::ArchivedArtifact { dated_name, artifact: station::REVIEW.doc },
                DocRef::ArchivedArtifact { dated_name, artifact: station::VERIFY.doc },
            ]
        })
        .collect();
    store.prefetch(&docs);
    names.iter().map(|dated_name| archived_item(store, dated_name)).collect()
}

fn archived_item(store: &dyn Store, dated_name: &str) -> ListArchivedJson {
    let (date, name) = split_dated_name(dated_name);
    let tasks = store.read_archived_artifact(dated_name, "tasks.md").map(|text| {
        let (total, done, _) = crate::tasks::progress(&crate::tasks::parse(&text));
        (total, done)
    });
    let meta = crate::model::ChangeMeta::from_text(store.read_archived_meta(dated_name).as_deref()).unwrap_or_default();
    let outcome = |st: &station::Station| {
        let ticket_archived = store.read_archived_artifact(dated_name, st.doc).is_some();
        ListArchivedOutcome::Known(station::ArchivedOutcome::of(ticket_archived, station::is_stamped(meta.anchors(st))))
    };
    ListArchivedJson {
        dated_name: dated_name.to_string(),
        date: date.to_string(),
        name: name.to_string(),
        review_outcome: outcome(&station::REVIEW),
        verify_outcome: outcome(&station::VERIFY),
        tasks_total: tasks.map(|(total, _)| total),
        tasks_done: tasks.map(|(_, done)| done),
        from_discussions: meta.from_discussions(),
        created_by: meta.created_by,
        delta_capabilities: store.archived_delta_capabilities(dated_name),
        fields: meta.fields,
    }
}

/// The date and the name of an archived change, by the rule the archived
/// list has always answered with: a `-` at bytes 4, 7 and 10, digits not
/// checked, and no date at all when the name does not fit. It is lenient on
/// purpose; changing it would change the `date` and `name` every archived
/// list returns.
pub fn split_dated_name(dated_name: &str) -> (&str, &str) {
    let bytes = dated_name.as_bytes();
    if bytes.len() > 11
        && bytes.get(4) == Some(&b'-')
        && bytes.get(7) == Some(&b'-')
        && bytes.get(10) == Some(&b'-')
        && dated_name.is_char_boundary(10)
        && dated_name.is_char_boundary(11)
    {
        (&dated_name[..10], &dated_name[11..])
    } else {
        ("", dated_name)
    }
}

/// The `specs` items of `list --specs --json`.
pub fn specs_json_items(store: &dyn Store) -> serde_json::Value {
    let mut specs = store.list_canonical_capabilities();
    specs.sort();
    serde_json::Value::Array(
        specs
            .iter()
            .map(|s| {
                // The listed path is the capability's directory (its spec.md parent).
                let dir = store
                    .canonical_spec_path(s)
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_default();
                serde_json::json!({
                    "id": s,
                    "path": dir.to_string_lossy(),
                })
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::{changes_json, changes_json_detail, changes_json_with, split_dated_name, ListWorktreeJson};
    use crate::teststore::TestStore;
    use std::collections::BTreeMap;

    fn demo_store(tag: &str) -> TestStore {
        let store = TestStore::with_meta(tag, "schema: spec-driven\ncreated: 2026-07-01\n");
        store.put_artifact(tag, "proposal.md", "## Why\n\nDemo.\n");
        store.put_artifact(tag, "tasks.md", "## 1. Group\n\n- [ ] 1.1 First task\n- [x] 1.2 Second task\n");
        store
    }

    #[test]
    fn a_dated_name_splits_into_its_date_and_its_name_and_other_names_have_no_date() {
        assert_eq!(split_dated_name("2026-07-20-demo"), ("2026-07-20", "demo"));
        assert_eq!(split_dated_name("demo"), ("", "demo"));
        // The digits are not checked; the three dashes and a name after them are.
        assert_eq!(split_dated_name("abcd-ef-gh-x"), ("abcd-ef-gh", "x"));
        assert_eq!(split_dated_name("2026-07-20-"), ("", "2026-07-20-"));
        assert_eq!(split_dated_name("2026-07-20"), ("", "2026-07-20"));
    }

    #[test]
    fn empty_worktree_facts_serialize_bit_identically_to_the_frozen_assembly() {
        // D5「加法、缺席即位元級不變」的釘子：沒有 facts 時新變體不得改動任何位元。
        let store = demo_store("demo");
        let changes = crate::model::list_changes(&store);
        assert_eq!(
            serde_json::to_string(&changes_json_with(&store, &changes, &BTreeMap::new())).unwrap(),
            serde_json::to_string(&changes_json(&store, &changes)).unwrap(),
        );
    }

    #[test]
    fn worktree_facts_add_a_camel_case_path_and_branch_object() {
        // Spec Example「計數與欄位形狀」的欄位面。
        let store = demo_store("add-dark-mode");
        let changes = crate::model::list_changes(&store);
        let facts = BTreeMap::from([(
            "add-dark-mode".to_string(),
            ListWorktreeJson {
                path: "/repos/speclink.worktrees/add-dark-mode".to_string(),
                branch: "speclink/add-dark-mode".to_string(),
            },
        )]);
        let json = serde_json::to_string(&changes_json_with(&store, &changes, &facts)).unwrap();
        assert!(
            json.contains(
                r#""worktree":{"path":"/repos/speclink.worktrees/add-dark-mode","branch":"speclink/add-dark-mode"}"#
            ),
            "got: {json}"
        );
    }

    #[test]
    fn an_unmapped_change_omits_the_worktree_key_entirely() {
        let store = demo_store("demo");
        let changes = crate::model::list_changes(&store);
        let facts = BTreeMap::from([(
            "other".to_string(),
            ListWorktreeJson { path: "/x".into(), branch: "speclink/other".into() },
        )]);
        let json = serde_json::to_string(&changes_json_with(&store, &changes, &facts)).unwrap();
        assert!(!json.contains("worktree"), "absent means not serialized: {json}");
    }

    #[test]
    fn list_json_payload_shape_is_unchanged_by_the_lifecycle_fields() {
        // Parity pin: the CLI `list --json` item must not gain started_*
        // fields (the desktop overlays them in its own layer), and a
        // pre-migration meta (no started_*) must serialize identically.
        let old_meta = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
        old_meta.put_artifact("demo", "proposal.md", "## Why\n\nDemo.\n");
        old_meta.put_artifact("demo", "tasks.md", "## 1. Group\n\n- [ ] 1.1 First task\n- [x] 1.2 Second task\n");

        let stamped = TestStore::with_meta(
            "demo",
            "schema: spec-driven\ncreated: 2026-07-01\nstarted_at: 2026-07-06\nstarted_by: W <w@example.com>\nstarted_with: claude\n",
        );
        stamped.put_artifact("demo", "proposal.md", "## Why\n\nDemo.\n");
        stamped.put_artifact("demo", "tasks.md", "## 1. Group\n\n- [ ] 1.1 First task\n- [x] 1.2 Second task\n");

        let expected = r#"{"completedTasks":1,"name":"demo","status":"in-progress","summary":"Demo.","totalTasks":2,"created":"2026-07-01"}"#;
        for store in [old_meta, stamped] {
            let changes = crate::model::list_changes(&store);
            let items = changes_json(&store, &changes);
            assert_eq!(items.len(), 1);
            assert_eq!(serde_json::to_string(&items[0]).unwrap(), expected);
        }
    }

    #[test]
    fn list_json_payload_is_unchanged_by_board_rank() {
        // spec「board_rank 不進 CLI 輸出且既有輸出逐位元不變」：含 board_rank 的
        // meta 序列化結果與移除該欄位後逐位元一致，且不出現 rank 相關欄位。
        let ranked = TestStore::with_meta(
            "demo",
            "schema: spec-driven\ncreated: 2026-07-01\nboard_rank: n\n",
        );
        ranked.put_artifact("demo", "proposal.md", "## Why\n\nDemo.\n");
        ranked.put_artifact("demo", "tasks.md", "## 1. Group\n\n- [ ] 1.1 First task\n- [x] 1.2 Second task\n");

        let bare = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
        bare.put_artifact("demo", "proposal.md", "## Why\n\nDemo.\n");
        bare.put_artifact("demo", "tasks.md", "## 1. Group\n\n- [ ] 1.1 First task\n- [x] 1.2 Second task\n");

        let json_of = |store: &TestStore| {
            let changes = crate::model::list_changes(store);
            serde_json::to_string(&changes_json(store, &changes)).unwrap()
        };
        let ranked_json = json_of(&ranked);
        assert_eq!(ranked_json, json_of(&bare), "board_rank must not affect list --json");
        assert!(!ranked_json.contains("board_rank") && !ranked_json.contains("boardRank"));
    }

    #[test]
    fn list_json_payload_is_unchanged_by_depends_on() {
        // spec change-lifecycle「depends_on 不影響既有輸出」：含 depends_on 的
        // meta 序列化結果與移除該行後逐位元一致，且不出現依賴相關欄位。
        let dependent = TestStore::with_meta(
            "demo",
            "schema: spec-driven\ncreated: 2026-07-01\ndepends_on: other\n",
        );
        dependent.put_artifact("demo", "proposal.md", "## Why\n\nDemo.\n");
        dependent.put_artifact("demo", "tasks.md", "## 1. Group\n\n- [ ] 1.1 First task\n- [x] 1.2 Second task\n");

        let bare = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
        bare.put_artifact("demo", "proposal.md", "## Why\n\nDemo.\n");
        bare.put_artifact("demo", "tasks.md", "## 1. Group\n\n- [ ] 1.1 First task\n- [x] 1.2 Second task\n");

        let json_of = |store: &TestStore| {
            let changes = crate::model::list_changes(store);
            serde_json::to_string(&changes_json(store, &changes)).unwrap()
        };
        let dependent_json = json_of(&dependent);
        assert_eq!(dependent_json, json_of(&bare), "depends_on must not affect list --json");
        assert!(
            !dependent_json.contains("depends_on") && !dependent_json.contains("dependsOn"),
            "no dependency field may leak into the CLI item: {dependent_json}"
        );
    }

    #[test]
    fn list_json_payload_is_unchanged_by_reviewed_fields() {
        // spec review-station「CLI 清單輸出的相容性釘住」：帶全套 reviewed 欄位的
        // change 與不帶者序列化同形（審查狀態僅進 desktop 協定，不進 CLI 輸出）。
        let reviewed = TestStore::with_meta(
            "demo",
            "schema: spec-driven\ncreated: 2026-07-01\nreviewed_at: 2026-08-01\nreviewed_by: Rev <r@example.com>\nreviewed_with: claude\nreviewed_tasks_total: 2\nreviewed_scope:\n  - path: crates/a/src/lib.rs\n    hash: 0f9c\n",
        );
        reviewed.put_artifact("demo", "proposal.md", "## Why\n\nDemo.\n");
        reviewed.put_artifact("demo", "tasks.md", "## 1. Group\n\n- [ ] 1.1 First task\n- [x] 1.2 Second task\n");

        let bare = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
        bare.put_artifact("demo", "proposal.md", "## Why\n\nDemo.\n");
        bare.put_artifact("demo", "tasks.md", "## 1. Group\n\n- [ ] 1.1 First task\n- [x] 1.2 Second task\n");

        let json_of = |store: &TestStore| {
            let changes = crate::model::list_changes(store);
            serde_json::to_string(&changes_json(store, &changes)).unwrap()
        };
        let reviewed_json = json_of(&reviewed);
        assert_eq!(reviewed_json, json_of(&bare), "reviewed_* must not affect list --json");
        assert!(
            !reviewed_json.contains("reviewed") && !reviewed_json.contains("reviewStatus"),
            "no review field may leak into the CLI item: {reviewed_json}"
        );
    }

    #[test]
    fn list_json_payload_is_unchanged_by_verified_fields() {
        // spec verify-station「CLI 清單輸出的驗證欄位釘住」：帶全套 verified 欄位
        // （單獨，以及與 reviewed 欄位並存）的 change 與不帶者序列化同形——驗證
        // 狀態同樣只進 desktop 協定，不進 CLI 輸出。
        const VERIFIED: &str = "verified_at: 2026-08-02\nverified_by: Ver <v@example.com>\nverified_with: claude\nverified_tasks_total: 2\nverified_scope:\n  - path: crates/a/src/lib.rs\n    hash: 0f9c\n";
        const REVIEWED: &str = "reviewed_at: 2026-08-01\nreviewed_by: Rev <r@example.com>\nreviewed_with: claude\nreviewed_tasks_total: 2\nreviewed_scope:\n  - path: crates/a/src/lib.rs\n    hash: 0f9c\n";
        let stamped = |extra: &str| {
            let store = TestStore::with_meta(
                "demo",
                &format!("schema: spec-driven\ncreated: 2026-07-01\n{extra}"),
            );
            store.put_artifact("demo", "proposal.md", "## Why\n\nDemo.\n");
            store.put_artifact(
                "demo",
                "tasks.md",
                "## 1. Group\n\n- [ ] 1.1 First task\n- [x] 1.2 Second task\n",
            );
            store
        };
        let json_of = |store: &TestStore| {
            let changes = crate::model::list_changes(store);
            serde_json::to_string(&changes_json(store, &changes)).unwrap()
        };
        let bare_json = json_of(&stamped(""));
        for extra in [VERIFIED, &format!("{REVIEWED}{VERIFIED}")] {
            let json = json_of(&stamped(extra));
            assert_eq!(json, bare_json, "verified_* must not affect list --json");
            assert!(
                !json.contains("verified") && !json.contains("verifyStatus"),
                "no verify field may leak into the CLI item: {json}"
            );
        }
    }

    // --- the two time fields (remote-read-verbs D5) ---

    /// 2026-11-12T03:00:00Z in whole seconds since the Unix epoch.
    const NOV_12_0300: u64 = 1_794_452_400;

    fn item_json(store: &TestStore) -> serde_json::Value {
        let changes = crate::model::list_changes(store);
        let items = changes_json(store, &changes);
        assert_eq!(items.len(), 1);
        serde_json::to_value(&items[0]).unwrap()
    }

    #[test]
    fn an_item_carries_the_creation_date_of_its_metadata() {
        let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-11-01\n");
        assert_eq!(item_json(&store)["created"], "2026-11-01");
        // A creation time stamp reads as its date.
        let stamped = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-11-01T08:30:00+08:00\n");
        assert_eq!(item_json(&stamped)["created"], "2026-11-01");
    }

    #[test]
    fn a_creation_value_that_is_no_date_goes_out_as_the_metadata_has_it() {
        // A hand-edited value is not corrected and not dropped: `show` prints
        // it the same way, and a reader that cannot parse it skips it.
        let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: someday\n");
        assert_eq!(item_json(&store)["created"], "someday");
    }

    #[test]
    fn an_item_carries_the_last_modified_time_as_rfc3339_utc() {
        let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-11-01\n");
        store.updated_at.borrow_mut().insert("demo".to_string(), NOV_12_0300);
        let item = item_json(&store);
        assert_eq!(item["modifiedAt"], "2026-11-12T03:00:00Z");
        assert!(item.get("modified_at").is_none(), "the key is camelCase: {item}");
    }

    #[test]
    fn a_store_that_reports_zero_gives_no_modified_at() {
        let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-11-01\n");
        let item = item_json(&store);
        assert!(item.get("modifiedAt").is_none(), "absent, never 0 or an empty string: {item}");
        assert_eq!(item["created"], "2026-11-01", "created does not depend on the store's clock");
    }

    #[test]
    fn a_change_with_no_usable_metadata_gives_no_created() {
        // Corrupt metadata, and metadata with no creation date.
        for meta in ["schema: [unclosed\n", "schema: spec-driven\n"] {
            let store = TestStore::with_meta("demo", meta);
            let item = item_json(&store);
            assert!(item.get("created").is_none(), "{meta:?}: {item}");
        }
    }

    #[test]
    fn the_time_fields_come_after_the_frozen_fields_and_before_worktree() {
        let store = demo_store("demo");
        store.updated_at.borrow_mut().insert("demo".to_string(), NOV_12_0300);
        let changes = crate::model::list_changes(&store);
        let facts = BTreeMap::from([(
            "demo".to_string(),
            ListWorktreeJson { path: "/x".into(), branch: "speclink/demo".into() },
        )]);
        assert_eq!(
            serde_json::to_string(&changes_json_with(&store, &changes, &facts)).unwrap(),
            r#"[{"completedTasks":1,"name":"demo","status":"in-progress","summary":"Demo.","totalTasks":2,"created":"2026-07-01","modifiedAt":"2026-11-12T03:00:00Z","worktree":{"path":"/x","branch":"speclink/demo"}}]"#
        );
    }

    #[test]
    fn a_time_no_rfc3339_text_can_state_gives_no_modified_at_and_no_panic() {
        // Year 9999 ends at 253402300799; a store can report anything in u64.
        for secs in [253_402_300_800_u64, i64::MAX as u64, i64::MAX as u64 + 1, u64::MAX] {
            let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-11-01\n");
            store.updated_at.borrow_mut().insert("demo".to_string(), secs);
            let item = item_json(&store);
            assert!(item.get("modifiedAt").is_none(), "{secs}: {item}");
        }
        let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-11-01\n");
        store.updated_at.borrow_mut().insert("demo".to_string(), 253_402_300_799);
        assert_eq!(item_json(&store)["modifiedAt"], "9999-12-31T23:59:59Z");
        store.updated_at.borrow_mut().insert("demo".to_string(), 1);
        assert_eq!(item_json(&store)["modifiedAt"], "1970-01-01T00:00:01Z");
    }

    // --- the detail keys (verb-output-for-embedded D1) ---

    const DETAIL_KEYS: [&str; 7] = ["review", "verify", "claimed", "branch", "landed", "createdBy", "fromDiscussions"];
    const REVIEW_STAMP: &str = "reviewed_at: 2026-08-01\nreviewed_by: Rev <r@example.com>\nreviewed_with: claude\nreviewed_tasks_total: 2\nreviewed_scope:\n  - path: crates/a/src/lib.rs\n    hash: 0f9c\n";
    const LANDED_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

    /// `demo_store` with `extra` appended to the metadata.
    fn recorded_store(extra: &str) -> TestStore {
        let store = TestStore::with_meta("demo", &format!("schema: spec-driven\ncreated: 2026-07-01\n{extra}"));
        store.put_artifact("demo", "proposal.md", "## Why\n\nDemo.\n");
        store.put_artifact("demo", "tasks.md", "## 1. Group\n\n- [ ] 1.1 First task\n- [x] 1.2 Second task\n");
        store
    }

    fn detail_item(store: &TestStore, detail: bool) -> serde_json::Value {
        let changes = crate::model::list_changes(store);
        let items = changes_json_detail(store, &changes, &BTreeMap::new(), detail);
        assert_eq!(items.len(), 1);
        serde_json::to_value(&items[0]).unwrap()
    }

    fn detail_keys_of(item: &serde_json::Value) -> Vec<&'static str> {
        DETAIL_KEYS.into_iter().filter(|key| item.get(key).is_some()).collect()
    }

    #[test]
    fn detail_adds_the_records_a_change_has() {
        // spec verb-contract「清單的詳細輸出」Scenario「有記錄的變更帶出四類狀態」。
        let store = recorded_store(&format!(
            "{REVIEW_STAMP}claimed_by: Alice\nclaimed_by_id: u_1\nclaimed_at: 2026-11-18T10:00:00+00:00\nbranch: speclink/demo\nlanded_commit: {LANDED_COMMIT}\nlanded_at: 2026-11-18T11:00:00+00:00\nlanded_by: Bob\n"
        ));
        store.put_artifact("demo", crate::station::VERIFY_DOC, "## Round 1\n");
        let item = detail_item(&store, true);
        assert_eq!(
            item["review"],
            serde_json::json!({
                "state": "stamped",
                "stampedAt": "2026-08-01",
                "stampedBy": "Rev <r@example.com>",
                "stampedWith": "claude",
                "rounds": 0,
                "tasksTotal": 2,
            }),
            "the shape of `show --json`, without the scope"
        );
        assert_eq!(item["verify"], serde_json::json!({ "state": "inProgress" }));
        assert_eq!(
            item["claimed"],
            serde_json::json!({ "by": "Alice", "byId": "u_1", "at": "2026-11-18T10:00:00+00:00" })
        );
        assert_eq!(item["branch"], "speclink/demo");
        assert_eq!(item["landed"], serde_json::json!({ "commit": LANDED_COMMIT }), "the commit only");

        // The fields that were there before are the ones of a list without detail.
        let mut without = item.clone();
        for key in DETAIL_KEYS {
            without.as_object_mut().unwrap().remove(key);
        }
        assert_eq!(without, detail_item(&store, false));
        assert_eq!(detail_keys_of(&detail_item(&store, false)), Vec::<&str>::new());
    }

    #[test]
    fn detail_gives_each_record_its_own_key() {
        // spec Example「各種記錄對應的鍵」，一列一個案例。
        let landed = format!("branch: speclink/demo\nlanded_commit: {LANDED_COMMIT}\nlanded_at: 2026-11-18T11:00:00+00:00\n");
        let unlanded_branchless = format!("landed_commit: {LANDED_COMMIT}\n");
        let two_rounds = "# Review — demo\n\n## Round 1\n\n**Scope**: src/a.rs\n\n## Round 2\n\n**Scope**: src/a.rs\n";
        let cases: [(&str, &str, bool, &[&str]); 9] = [
            ("no record", "", false, &[]),
            ("a review stamp only", REVIEW_STAMP, false, &["review"]),
            ("an open review ticket of two rounds, no stamp", "", true, &["review"]),
            ("a recorded creator", "created_by: Ann <ann@example.com>\n", false, &["createdBy"]),
            ("promoted from discussion two", "from_discussion: two\n", false, &["fromDiscussions"]),
            ("claimed, no identifier", "claimed_by: Alice\nclaimed_at: 2026-11-18T10:00:00+00:00\n", false, &["claimed"]),
            ("a branch, not landed", "branch: speclink/demo\n", false, &["branch"]),
            ("landed", &landed, false, &["branch", "landed"]),
            ("landed, no branch recorded", &unlanded_branchless, false, &["landed"]),
        ];
        for (case, extra, review_ticket, keys) in cases {
            let store = recorded_store(extra);
            if review_ticket {
                store.put_artifact("demo", crate::station::REVIEW_DOC, two_rounds);
            }
            let item = detail_item(&store, true);
            assert_eq!(detail_keys_of(&item), keys, "{case}: {item}");
        }
        // The values of the rows that say more than "the key is there".
        let ticket = recorded_store("");
        ticket.put_artifact("demo", crate::station::REVIEW_DOC, two_rounds);
        assert_eq!(detail_item(&ticket, true)["review"], serde_json::json!({ "state": "inProgress", "rounds": 2 }));
        // A ticket whose rounds cannot be counted is still an open one.
        ticket.put_artifact("demo", crate::station::REVIEW_DOC, "## Round 1\n");
        assert_eq!(detail_item(&ticket, true)["review"], serde_json::json!({ "state": "inProgress" }));
        let unkeyed = recorded_store("claimed_by: Alice\nclaimed_at: 2026-11-18T10:00:00+00:00\n");
        assert_eq!(
            detail_item(&unkeyed, true)["claimed"],
            serde_json::json!({ "by": "Alice", "at": "2026-11-18T10:00:00+00:00" })
        );
    }

    #[test]
    fn detail_adds_the_creator_and_the_source_discussions() {
        // spec verb-contract「清單的詳細輸出」Scenario「帶出建立的人與來源討論」。
        let store = recorded_store("created_by: Ann <ann@example.com>\nfrom_discussion: two, three\n");
        let item = detail_item(&store, true);
        assert_eq!(item["createdBy"], "Ann <ann@example.com>");
        assert_eq!(item["fromDiscussions"], serde_json::json!(["two", "three"]), "the order of the metadata");
        let plain = detail_item(&store, false);
        assert!(plain.get("createdBy").is_none() && plain.get("fromDiscussions").is_none(), "{plain}");
        // A change with neither record has neither key.
        let solo = detail_item(&recorded_store(""), true);
        assert!(solo.get("createdBy").is_none() && solo.get("fromDiscussions").is_none(), "{solo}");
        // The two keys come after the other detail keys and before worktree.
        let store = recorded_store("branch: speclink/demo\ncreated_by: Ann\nfrom_discussion: two\n");
        let changes = crate::model::list_changes(&store);
        let facts = BTreeMap::from([(
            "demo".to_string(),
            ListWorktreeJson { path: "/x".into(), branch: "speclink/demo".into() },
        )]);
        assert_eq!(
            serde_json::to_string(&changes_json_detail(&store, &changes, &facts, true)).unwrap(),
            r#"[{"completedTasks":1,"name":"demo","status":"in-progress","summary":"Demo.","totalTasks":2,"created":"2026-07-01","branch":"speclink/demo","createdBy":"Ann","fromDiscussions":["two"],"worktree":{"path":"/x","branch":"speclink/demo"}}]"#
        );
    }

    #[test]
    fn a_change_with_corrupt_metadata_has_no_creator_or_source_discussion() {
        let store = TestStore::with_meta("demo", "created_by: Ann\nfrom_discussion: two\nschema: [unclosed\n");
        let item = detail_item(&store, true);
        assert!(item.get("metaError").is_some(), "{item}");
        assert!(item.get("createdBy").is_none() && item.get("fromDiscussions").is_none(), "{item}");
    }

    #[test]
    fn a_change_with_no_record_serializes_the_same_with_and_without_detail() {
        let store = recorded_store("");
        let changes = crate::model::list_changes(&store);
        let text = |detail| serde_json::to_string(&changes_json_detail(&store, &changes, &BTreeMap::new(), detail)).unwrap();
        assert_eq!(text(true), text(false));
        assert_eq!(text(false), serde_json::to_string(&changes_json(&store, &changes)).unwrap());
    }

    #[test]
    fn a_change_with_corrupt_metadata_carries_no_detail_key() {
        // 損壞的 metadata 讀成預設值：不能從它報出「沒有記錄」以外的任何狀態，
        // 工單在也一樣。
        let store = TestStore::with_meta("demo", "schema: [unclosed\n");
        store.put_artifact("demo", crate::station::REVIEW_DOC, "## Round 1\n");
        store.put_artifact("demo", crate::station::VERIFY_DOC, "## Round 1\n");
        let item = detail_item(&store, true);
        assert!(item.get("metaError").is_some(), "{item}");
        assert_eq!(detail_keys_of(&item), Vec::<&str>::new(), "{item}");
        assert_eq!(item, detail_item(&store, false));
    }

    #[test]
    fn the_detail_keys_come_after_the_fields_and_before_worktree() {
        let store = recorded_store(&format!("branch: speclink/demo\nlanded_commit: {LANDED_COMMIT}\n"));
        let changes = crate::model::list_changes(&store);
        let facts = BTreeMap::from([(
            "demo".to_string(),
            ListWorktreeJson { path: "/x".into(), branch: "speclink/demo".into() },
        )]);
        assert_eq!(
            serde_json::to_string(&changes_json_detail(&store, &changes, &facts, true)).unwrap(),
            format!(
                r#"[{{"completedTasks":1,"name":"demo","status":"in-progress","summary":"Demo.","totalTasks":2,"created":"2026-07-01","branch":"speclink/demo","landed":{{"commit":"{LANDED_COMMIT}"}},"worktree":{{"path":"/x","branch":"speclink/demo"}}}}]"#
            )
        );
    }
}

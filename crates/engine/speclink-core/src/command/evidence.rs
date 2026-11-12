//! `evidence show` — read one change's completion evidence (design
//! remote-read-verbs D1). The name is looked up the way `artifact cat` does
//! it (`model::locate_change`): an active change first, else its archive.

use super::{CommandError, CommandOutcome, ErrorCode};
use crate::model::{locate_change, Located};
use crate::store::Store;
use crate::tasks::{EvidenceEntry, TouchedRecord};

/// The archived document that holds a change's evidence record.
const ARCHIVED_EVIDENCE: &str = ".evidence.json";

/// `evidence show` outcome: the change's recorded entries. `change` is the
/// change's own name (an archived name without its date), `archived_as` the
/// archived name when the record came from the archive. No record at all is
/// an empty `entries`.
#[derive(Debug)]
pub struct EvidenceShowOutcome {
    pub change: String,
    pub archived_as: Option<String>,
    pub entries: Vec<EvidenceEntry>,
}

pub(super) fn show(store: &dyn Store, name: &str) -> Result<CommandOutcome, CommandError> {
    let (change, archived_as, text) = match locate_change(store, name) {
        Some(Located::Active(change)) => {
            let text = store.read_evidence(&change.name);
            (change.name, None, text)
        }
        Some(Located::Archived(dated)) => {
            let text = store.read_archived_artifact(&dated, ARCHIVED_EVIDENCE);
            (crate::util::strip_date_prefix(&dated).to_string(), Some(dated), text)
        }
        None => {
            return Err(CommandError::new(ErrorCode::NotFound, format!("Change '{name}' not found.")));
        }
    };
    // No record is a normal state. A record that does not parse is a failure:
    // "no evidence" over a corrupt record would be a wrong answer. The message
    // carries the parser's position, never the record's content.
    let entries = match text {
        Some(text) => serde_json::from_str::<TouchedRecord>(&text)
            .map_err(|e| {
                CommandError::new(
                    ErrorCode::Error,
                    format!(
                        "evidence record of '{change}' cannot be parsed: {:?} error at line {} column {}",
                        e.classify(),
                        e.line(),
                        e.column()
                    ),
                )
            })?
            .entries,
        None => Vec::new(),
    };
    Ok(CommandOutcome::EvidenceShow(EvidenceShowOutcome { change, archived_as, entries }))
}

#[cfg(test)]
mod tests {
    use crate::command::{execute, Command, CommandOutcome, ErrorCode, EvidenceShowOutcome, ExecutionContext};
    use crate::teststore::TestStore;

    const META: &str = "schema: spec-driven\ncreated: 2026-11-01\n";

    /// Two entries: task 1.1 on a.rs and b.rs, task 1.2 on b.rs and c.rs.
    const TWO_ENTRIES: &str = r#"{"version":2,"change":"demo","entries":[
        {"taskId":"tsk_a","taskDesc":"1.1 First","touchedFiles":["src/a.rs","src/b.rs"],"recordedAt":"2026-11-12T03:00:00Z"},
        {"taskId":"tsk_b","taskDesc":"1.2 Second","actor":"Alice","touchedFiles":["src/b.rs","src/c.rs"],"recordedAt":"2026-11-12T04:00:00Z"}]}"#;

    fn one_entry(task_id: &str) -> String {
        format!(
            r#"{{"version":2,"change":"demo","entries":[{{"taskId":"{task_id}","taskDesc":"1.1 First","touchedFiles":["src/a.rs"],"recordedAt":"2026-11-12T03:00:00Z"}}]}}"#
        )
    }

    fn put_archived(store: &TestStore, dated: &str, evidence: Option<&str>) {
        store.archived_metas.borrow_mut().insert(dated.to_string(), META.to_string());
        if let Some(text) = evidence {
            store
                .archived_artifacts
                .borrow_mut()
                .insert((dated.to_string(), ".evidence.json".to_string()), text.to_string());
        }
    }

    fn show(store: &TestStore, name: &str) -> EvidenceShowOutcome {
        let (outcome, events) = execute(
            store,
            &ExecutionContext::default(),
            Command::EvidenceShow { change: name.to_string() },
        )
        .unwrap_or_else(|e| panic!("evidence show {name} executes: {}", e.message));
        assert!(events.is_empty(), "a query never produces events");
        match outcome {
            CommandOutcome::EvidenceShow(o) => o,
            other => panic!("expected an evidence-show outcome, got {other:?}"),
        }
    }

    fn task_ids(o: &EvidenceShowOutcome) -> Vec<&str> {
        o.entries.iter().map(|e| e.task_id.as_str()).collect()
    }

    #[test]
    fn an_active_change_returns_its_record() {
        let store = TestStore::with_meta("demo", META);
        store.put_evidence("demo", TWO_ENTRIES);
        let o = show(&store, "demo");
        assert_eq!(o.change, "demo");
        assert_eq!(o.archived_as, None);
        assert_eq!(task_ids(&o), ["tsk_a", "tsk_b"]);
        assert_eq!(o.entries[0].task_desc, "1.1 First");
        assert_eq!(o.entries[0].touched_files, ["src/a.rs", "src/b.rs"]);
        assert_eq!(o.entries[0].recorded_at, "2026-11-12T03:00:00Z");
        assert_eq!(o.entries[0].actor, None);
        assert_eq!(o.entries[1].actor.as_deref(), Some("Alice"));
    }

    #[test]
    fn a_change_with_no_record_is_empty() {
        let store = TestStore::with_meta("fresh", META);
        let o = show(&store, "fresh");
        assert_eq!(o.change, "fresh");
        assert_eq!(o.archived_as, None);
        assert!(o.entries.is_empty());
    }

    #[test]
    fn an_archived_change_reads_by_its_original_name_and_by_its_archived_name() {
        let store = TestStore::default();
        put_archived(&store, "2026-11-12-demo", Some(TWO_ENTRIES));
        for name in ["demo", "2026-11-12-demo"] {
            let o = show(&store, name);
            assert_eq!(o.change, "demo", "{name}");
            assert_eq!(o.archived_as.as_deref(), Some("2026-11-12-demo"), "{name}");
            assert_eq!(task_ids(&o), ["tsk_a", "tsk_b"], "{name}");
        }
    }

    #[test]
    fn an_archived_change_with_no_record_is_empty() {
        let store = TestStore::default();
        put_archived(&store, "2026-11-12-demo", None);
        let o = show(&store, "demo");
        assert_eq!(o.archived_as.as_deref(), Some("2026-11-12-demo"));
        assert!(o.entries.is_empty());
    }

    #[test]
    fn an_active_change_of_the_same_name_wins() {
        let store = TestStore::with_meta("demo", META);
        put_archived(&store, "2026-11-01-demo", Some(TWO_ENTRIES));
        let o = show(&store, "demo");
        assert_eq!(o.archived_as, None);
        assert!(o.entries.is_empty(), "the active change has no record");
        // The archived name still reaches the archived one.
        let archived = show(&store, "2026-11-01-demo");
        assert_eq!(archived.archived_as.as_deref(), Some("2026-11-01-demo"));
        assert_eq!(task_ids(&archived), ["tsk_a", "tsk_b"]);
    }

    #[test]
    fn a_name_archived_twice_reads_the_newest() {
        let store = TestStore::default();
        put_archived(&store, "2026-10-01-demo", Some(&one_entry("tsk_old")));
        put_archived(&store, "2026-11-12-demo", Some(&one_entry("tsk_new")));
        let o = show(&store, "demo");
        assert_eq!(o.archived_as.as_deref(), Some("2026-11-12-demo"));
        assert_eq!(task_ids(&o), ["tsk_new"]);
        assert_eq!(task_ids(&show(&store, "2026-10-01-demo")), ["tsk_old"]);
    }

    #[test]
    fn an_unknown_name_is_not_found() {
        let store = TestStore::with_meta("demo", META);
        put_archived(&store, "2026-11-12-other", Some(TWO_ENTRIES));
        let err = execute(
            &store,
            &ExecutionContext::default(),
            Command::EvidenceShow { change: "ghost".to_string() },
        )
        .expect_err("neither active nor archived");
        assert_eq!(err.code, ErrorCode::NotFound);
        assert_eq!(err.message, "Change 'ghost' not found.");
    }

    #[test]
    fn a_record_that_does_not_parse_is_an_error_not_an_empty_set() {
        let active = TestStore::with_meta("demo", META);
        active.put_evidence("demo", "not json");
        let archived = TestStore::default();
        put_archived(&archived, "2026-11-12-demo", Some("not json"));
        for store in [active, archived] {
            let err = execute(
                &store,
                &ExecutionContext::default(),
                Command::EvidenceShow { change: "demo".to_string() },
            )
            .expect_err("a corrupt record must not read as no record");
            assert_eq!(err.code, ErrorCode::Error);
            assert!(
                err.message.starts_with("evidence record of 'demo' cannot be parsed: "),
                "{}",
                err.message
            );
            assert!(!err.message.contains("not json"), "the record's content stays out: {}", err.message);
        }
    }

    #[test]
    fn it_writes_nothing() {
        let store = TestStore::with_meta("demo", META);
        store.put_evidence("demo", TWO_ENTRIES);
        put_archived(&store, "2026-11-12-other", Some(TWO_ENTRIES));
        show(&store, "demo");
        show(&store, "other");
        assert_eq!(*store.meta_writes.borrow(), 0);
        assert_eq!(*store.artifact_writes.borrow(), 0);
        assert_eq!(store.evidence.borrow().get("demo").map(String::as_str), Some(TWO_ENTRIES));
        assert_eq!(store.archived_metas.borrow().len(), 1);
        assert_eq!(store.archived_artifacts.borrow().len(), 1);
    }

    // --- artifact cat reads an archived change through the same lookup (D6) ---

    fn put_archived_doc(store: &TestStore, dated: &str, artifact: &str, content: &str) {
        store
            .archived_artifacts
            .borrow_mut()
            .insert((dated.to_string(), artifact.to_string()), content.to_string());
    }

    fn cat(store: &TestStore, artifact: &str, change: &str) -> Result<String, crate::command::CommandError> {
        let (outcome, events) = execute(
            store,
            &ExecutionContext::default(),
            Command::ArtifactCat { artifact: artifact.to_string(), change: Some(change.to_string()) },
        )?;
        assert!(events.is_empty(), "a query never produces events");
        match outcome {
            CommandOutcome::ArtifactCat(read) => Ok(read.content),
            other => panic!("expected an artifact-cat outcome, got {other:?}"),
        }
    }

    /// `demo` archived as 2026-11-12-demo with a proposal and a delta spec for auth.
    fn archived_demo() -> TestStore {
        let store = TestStore::default();
        put_archived(&store, "2026-11-12-demo", None);
        put_archived_doc(&store, "2026-11-12-demo", "proposal.md", "## Why\n\nArchived.\n");
        put_archived_doc(&store, "2026-11-12-demo", "specs/auth/spec.md", "## ADDED Requirements\n");
        store
    }

    #[test]
    fn artifact_cat_reads_an_archived_change_by_either_name() {
        let store = archived_demo();
        for name in ["demo", "2026-11-12-demo"] {
            assert_eq!(cat(&store, "proposal", name).unwrap(), "## Why\n\nArchived.\n", "{name}");
            assert_eq!(cat(&store, "specs/auth", name).unwrap(), "## ADDED Requirements\n", "{name}");
        }
    }

    #[test]
    fn artifact_cat_says_which_archive_it_read() {
        let store = archived_demo();
        let read = |change: &str| {
            let cmd = Command::ArtifactCat { artifact: "proposal".to_string(), change: Some(change.to_string()) };
            match execute(&store, &ExecutionContext::default(), cmd).expect("reads").0 {
                CommandOutcome::ArtifactCat(read) => read.archived_as,
                other => panic!("expected an artifact-cat outcome, got {other:?}"),
            }
        };
        assert_eq!(read("demo").as_deref(), Some("2026-11-12-demo"));
        assert_eq!(read("2026-11-12-demo").as_deref(), Some("2026-11-12-demo"));
        store.metas.borrow_mut().insert("demo".to_string(), META.to_string());
        store.put_artifact("demo", "proposal.md", "## Why\n\nActive.\n");
        assert_eq!(read("demo"), None, "an active change is no archive");
    }

    #[test]
    fn artifact_cat_reads_the_newest_archived_change_of_a_name() {
        let store = archived_demo();
        put_archived(&store, "2026-10-01-demo", None);
        put_archived_doc(&store, "2026-10-01-demo", "proposal.md", "## Why\n\nOlder.\n");
        assert_eq!(cat(&store, "proposal", "demo").unwrap(), "## Why\n\nArchived.\n");
        assert_eq!(cat(&store, "proposal", "2026-10-01-demo").unwrap(), "## Why\n\nOlder.\n");
    }

    #[test]
    fn artifact_cat_prefers_the_active_change_and_its_output_is_unchanged() {
        let store = archived_demo();
        store.metas.borrow_mut().insert("demo".to_string(), META.to_string());
        store.put_artifact("demo", "proposal.md", "## Why\n\nActive.\n");
        assert_eq!(cat(&store, "proposal", "demo").unwrap(), "## Why\n\nActive.\n");
        // The active change has no design: the archive is not a second chance.
        put_archived_doc(&store, "2026-11-12-demo", "design.md", "archived design");
        let err = cat(&store, "design", "demo").expect_err("the active change has no design");
        assert_eq!(err.code, ErrorCode::NotFound);
        assert_eq!(err.message, "artifact 'design' not found for change 'demo'");
    }

    #[test]
    fn artifact_cat_of_a_name_in_neither_place_keeps_its_message() {
        let store = archived_demo();
        let err = cat(&store, "proposal", "ghost").expect_err("neither active nor archived");
        assert_eq!(err.code, ErrorCode::NotFound);
        assert_eq!(err.message, "Change 'ghost' not found.");
    }

    #[test]
    fn artifact_cat_of_an_archived_change_without_the_document_keeps_its_message() {
        let store = archived_demo();
        let err = cat(&store, "design", "demo").expect_err("the archived change has no design");
        assert_eq!(err.code, ErrorCode::NotFound);
        assert_eq!(err.message, "artifact 'design' not found for change 'demo'");
        let err = cat(&store, "nonsense", "demo").expect_err("no such artifact id");
        assert_eq!(err.code, ErrorCode::InvalidArgv);
        assert_eq!(
            err.message,
            "Unknown artifact 'nonsense'. Use proposal, design, tasks, or specs/<capability>"
        );
    }

    #[test]
    fn artifact_cat_never_reads_an_archived_record_that_is_no_artifact() {
        // The artifact vocabulary is closed: the evidence record and the
        // metadata of an archived change are not reachable through it.
        let store = archived_demo();
        put_archived_doc(&store, "2026-11-12-demo", ".evidence.json", TWO_ENTRIES);
        for artifact in [".evidence.json", ".meta.yaml", "specs/../.evidence.json", "specs/auth/../x"] {
            let err = cat(&store, artifact, "demo").expect_err(artifact);
            assert_eq!(err.code, ErrorCode::InvalidArgv, "{artifact}");
        }
    }

    // --- a name that is no change name is never looked up (sharp-edges audit) ---

    #[test]
    fn a_name_that_is_not_one_path_segment_is_never_looked_up() {
        // A store that finds a change by probing a directory would answer for
        // these names: `..` is the folder above the changes, `archive/<name>`
        // an archived change read as an active one. This double answers for
        // every one of them, as such a store would.
        let store = TestStore::default();
        for name in ["..", ".", "", "archive", "archive/2026-11-12-demo", "demo/../other", "a\\b", "../discussions"] {
            store.metas.borrow_mut().insert(name.to_string(), META.to_string());
            store.put_evidence(name, TWO_ENTRIES);
            store.put_artifact(name, "proposal.md", "## Why\n\nNot a change.\n");
            let shown = execute(
                &store,
                &ExecutionContext::default(),
                Command::EvidenceShow { change: name.to_string() },
            )
            .expect_err(&format!("{name:?} is no change name"));
            assert_eq!(shown.code, ErrorCode::NotFound, "{name:?}");
            assert_eq!(shown.message, format!("Change '{name}' not found."), "{name:?}");
            let cat = cat(&store, "proposal", name).expect_err(&format!("{name:?} is no change name"));
            assert_eq!(cat.code, ErrorCode::NotFound, "{name:?}");
            assert_eq!(cat.message, format!("Change '{name}' not found."), "{name:?}");
        }
    }

    #[test]
    fn an_archived_name_without_a_date_reads_like_any_other() {
        // An older archive can hold a change under its bare name.
        let store = TestStore::default();
        put_archived(&store, "legacy", Some(TWO_ENTRIES));
        let o = show(&store, "legacy");
        assert_eq!(o.change, "legacy");
        assert_eq!(o.archived_as.as_deref(), Some("legacy"));
        assert_eq!(task_ids(&o), ["tsk_a", "tsk_b"]);
    }

    #[test]
    fn a_record_of_the_first_format_reads_as_no_entries() {
        // The first format holds file lists with no actor or time; this read
        // never makes those up.
        let store = TestStore::with_meta("demo", META);
        store.put_evidence(
            "demo",
            r#"{"change":"demo","touched":[{"task_id":"1","task_desc":"1.1 a","files":["src/a.rs"]}]}"#,
        );
        assert!(show(&store, "demo").entries.is_empty());
    }
}

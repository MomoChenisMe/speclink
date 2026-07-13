//! Evidence adjudication: VerifyBundle production and staleness judgment.
//!
//! The Host owns this adjudication (design 決策五): a VerifyBundle fixes the
//! basis a verification ran against (spec / tasks / policy digests plus the
//! task identity list), and `judge_staleness` rejects evidence whose recorded
//! basis no longer matches — mixed-basis evidence is never accepted silently.
//! Errors are host-layer types (the binding-error precedent); the command
//! layer's closed code set is untouched, and no CLI verb is wired here.

use speclink_core::store::Store;
use speclink_core::tasks::{self, EvidenceEntry};

/// A fixed verification basis for one change: task identities plus the three
/// basis digests the verification ran against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyBundle {
    pub change: String,
    /// Task identities in file order: stable IDs, ordinal strings for
    /// unstamped legacy lines.
    pub task_ids: Vec<String>,
    pub spec_digest: String,
    pub tasks_digest: String,
    pub policy_digest: String,
    /// UTC RFC3339 production timestamp.
    pub produced_at: String,
}

/// Why a bundle could not be produced. Host-layer error type (the
/// binding-error precedent) — never a command-layer code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceError {
    ChangeNotFound { change: String },
    TasksMissing { change: String },
}

impl std::fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvidenceError::ChangeNotFound { change } => {
                write!(f, "Change '{change}' not found")
            }
            EvidenceError::TasksMissing { change } => {
                write!(f, "tasks.md not found for change '{change}'")
            }
        }
    }
}

impl std::error::Error for EvidenceError {}

/// One of the three verification bases, in the fixed spec → tasks → policy
/// order staleness reports use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasisItem {
    Spec,
    Tasks,
    Policy,
}

impl std::fmt::Display for BasisItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            BasisItem::Spec => "spec",
            BasisItem::Tasks => "tasks",
            BasisItem::Policy => "policy",
        })
    }
}

/// Verdict of judging one evidence entry against a bundle: valid only when
/// every basis digest matches; otherwise stale with ALL mismatched items
/// listed — mixed-basis evidence is never accepted silently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StalenessVerdict {
    Valid,
    Stale { mismatched: Vec<BasisItem> },
}

/// Produce the VerifyBundle for a change from its current workspace state.
/// The basis digests come from the same core computation evidence recording
/// uses, so recorded and judged bases always agree.
pub fn produce_verify_bundle(
    store: &dyn Store,
    change: &str,
) -> Result<VerifyBundle, EvidenceError> {
    if !store.change_exists(change) {
        return Err(EvidenceError::ChangeNotFound { change: change.to_string() });
    }
    let tasks_text = store
        .read_artifact(change, "tasks.md")
        .ok_or_else(|| EvidenceError::TasksMissing { change: change.to_string() })?;
    let digests = tasks::current_basis_digests(store, change);
    let task_ids = tasks::parse(&tasks_text)
        .into_iter()
        .map(|t| t.stable_id.unwrap_or_else(|| t.id.to_string()))
        .collect();
    Ok(VerifyBundle {
        change: change.to_string(),
        task_ids,
        spec_digest: digests.spec,
        tasks_digest: digests.tasks,
        policy_digest: digests.policy,
        produced_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    })
}

/// Judge one evidence entry against a bundle's basis.
pub fn judge_staleness(evidence: &EvidenceEntry, bundle: &VerifyBundle) -> StalenessVerdict {
    let mut mismatched = Vec::new();
    if evidence.basis_digests.spec != bundle.spec_digest {
        mismatched.push(BasisItem::Spec);
    }
    if evidence.basis_digests.tasks != bundle.tasks_digest {
        mismatched.push(BasisItem::Tasks);
    }
    if evidence.basis_digests.policy != bundle.policy_digest {
        mismatched.push(BasisItem::Policy);
    }
    if mismatched.is_empty() {
        StalenessVerdict::Valid
    } else {
        StalenessVerdict::Stale { mismatched }
    }
}

/// Why the archive gate rejects a change's evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveGateRejection {
    /// The bundle itself could not be produced (change or tasks.md missing).
    Bundle(EvidenceError),
    /// Not every task is checked.
    TasksIncomplete { remaining: usize },
    /// No per-task evidence recorded for the change.
    EvidenceMissing,
    /// The latest evidence's basis no longer matches the current bundle.
    StaleEvidence { mismatched: Vec<BasisItem> },
}

impl std::fmt::Display for ArchiveGateRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchiveGateRejection::Bundle(e) => write!(f, "{e}"),
            ArchiveGateRejection::TasksIncomplete { remaining } => {
                write!(f, "archive gate: {remaining} task(s) still open")
            }
            ArchiveGateRejection::EvidenceMissing => {
                f.write_str("archive gate: no completion evidence recorded")
            }
            ArchiveGateRejection::StaleEvidence { mismatched } => {
                let names: Vec<String> = mismatched.iter().map(|m| m.to_string()).collect();
                write!(f, "archive gate: stale evidence — mismatched basis: {}", names.join(", "))
            }
        }
    }
}

impl std::error::Error for ArchiveGateRejection {}

/// Archive-gate evidence check (spec verify-evidence): every task checked,
/// evidence present, and the latest entry unstale against the current
/// bundle. Advisory only — the local archive flow never calls it;
/// enforcement belongs to the remote Host (Phase 2).
pub fn check_archive_evidence(
    store: &dyn Store,
    ws: &speclink_core::workspace::Workspace,
    change: &str,
) -> Result<(), ArchiveGateRejection> {
    let bundle = produce_verify_bundle(store, change).map_err(ArchiveGateRejection::Bundle)?;
    let tasks_text = store.read_artifact(change, "tasks.md").unwrap_or_default();
    let remaining = tasks::parse(&tasks_text).iter().filter(|t| !t.done).count();
    if remaining > 0 {
        return Err(ArchiveGateRejection::TasksIncomplete { remaining });
    }
    let record = tasks::TouchedRecord::load(ws, change);
    let Some(latest) = record.entries.last() else {
        return Err(ArchiveGateRejection::EvidenceMissing);
    };
    match judge_staleness(latest, &bundle) {
        StalenessVerdict::Valid => Ok(()),
        StalenessVerdict::Stale { mismatched } => {
            Err(ArchiveGateRejection::StaleEvidence { mismatched })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speclink_core::tasks::{BasisDigests, EvidenceEntry};
    use speclink_fs::FsStore;
    use std::path::PathBuf;

    const TID_A: &str = "tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV";
    const TID_B: &str = "tsk_01BX5ZZKBKACTAV9WEVGEMMVRZ";

    /// Throwaway fs workspace with one change ("demo"): stamped tasks.md and
    /// one delta spec; removed on drop.
    struct TempProject {
        root: PathBuf,
    }

    impl TempProject {
        fn new(tag: &str) -> TempProject {
            let root = std::env::temp_dir().join(format!(
                "speclink-host-evidence-{tag}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&root);
            let change = root.join("openspec").join("changes").join("demo");
            std::fs::create_dir_all(change.join("specs").join("auth")).unwrap();
            std::fs::write(change.join(".openspec.yaml"), "schema: spec-driven\n").unwrap();
            std::fs::write(
                change.join("tasks.md"),
                format!(
                    "- [x] 1.1 first <!-- speclink-task:{TID_A} -->\n- [ ] 1.2 second <!-- speclink-task:{TID_B} -->\n"
                ),
            )
            .unwrap();
            std::fs::write(
                change.join("specs").join("auth").join("spec.md"),
                "## ADDED Requirements\n\n### Requirement: A\n",
            )
            .unwrap();
            TempProject { root }
        }

        fn store(&self) -> FsStore {
            FsStore::new(&self.root, "openspec")
        }

        fn write_change_file(&self, rel: &str, content: &str) {
            let p = rel
                .split('/')
                .fold(self.root.join("openspec").join("changes").join("demo"), |p, c| p.join(c));
            std::fs::write(p, content).unwrap();
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    /// Evidence recorded exactly at the given bundle's basis.
    fn entry_at(bundle: &VerifyBundle) -> EvidenceEntry {
        EvidenceEntry {
            task_id: TID_A.to_string(),
            task_desc: "1.1 first".to_string(),
            actor: Some("Tester <t@example.com>".to_string()),
            repo: Some("main".to_string()),
            head_commit: None,
            touched_files: vec!["src/app.rs".to_string()],
            basis_digests: BasisDigests {
                spec: bundle.spec_digest.clone(),
                tasks: bundle.tasks_digest.clone(),
                policy: bundle.policy_digest.clone(),
            },
            recorded_at: "2026-07-13T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn bundle_basis_is_reproducible_with_stable_task_ids() {
        let p = TempProject::new("repro");
        let store = p.store();
        let a = produce_verify_bundle(&store, "demo").expect("bundle produced");
        let b = produce_verify_bundle(&store, "demo").expect("bundle produced");
        assert_eq!(a.spec_digest, b.spec_digest, "spec digest reproducible");
        assert_eq!(a.tasks_digest, b.tasks_digest, "tasks digest reproducible");
        assert_eq!(a.policy_digest, b.policy_digest, "policy digest reproducible");
        assert_eq!(a.change, "demo");
        assert_eq!(a.task_ids, vec![TID_A.to_string(), TID_B.to_string()]);
        for d in [&a.spec_digest, &a.tasks_digest, &a.policy_digest] {
            assert!(d.starts_with("sha256:"), "digest form: {d}");
        }
        assert!(a.produced_at.ends_with('Z'), "producedAt is UTC: {}", a.produced_at);
    }

    #[test]
    fn delta_spec_edit_changes_only_the_spec_digest() {
        let p = TempProject::new("specdigest");
        let store = p.store();
        let before = produce_verify_bundle(&store, "demo").unwrap();
        p.write_change_file("specs/auth/spec.md", "## ADDED Requirements\n\n### Requirement: B\n");
        let after = produce_verify_bundle(&store, "demo").unwrap();
        assert_ne!(before.spec_digest, after.spec_digest, "spec digest follows delta content");
        assert_eq!(before.tasks_digest, after.tasks_digest);
        assert_eq!(before.policy_digest, after.policy_digest);
    }

    #[test]
    fn tasks_edit_stales_evidence_listing_only_the_tasks_digest() {
        let p = TempProject::new("stale");
        let store = p.store();
        let evidence = entry_at(&produce_verify_bundle(&store, "demo").unwrap());
        p.write_change_file(
            "tasks.md",
            &format!(
                "- [x] 1.1 first <!-- speclink-task:{TID_A} -->\n- [ ] 1.2 second edited <!-- speclink-task:{TID_B} -->\n"
            ),
        );
        let fresh = produce_verify_bundle(&store, "demo").unwrap();
        match judge_staleness(&evidence, &fresh) {
            StalenessVerdict::Stale { mismatched } => {
                assert_eq!(mismatched, vec![BasisItem::Tasks], "only the tasks basis mismatches");
            }
            other => panic!("expected a stale verdict, got {other:?}"),
        }
    }

    #[test]
    fn matching_basis_judges_valid() {
        let p = TempProject::new("valid");
        let store = p.store();
        let bundle = produce_verify_bundle(&store, "demo").unwrap();
        let evidence = entry_at(&bundle);
        assert_eq!(judge_staleness(&evidence, &bundle), StalenessVerdict::Valid);
    }

    // --- archive gate 檢查函式（spec verify-evidence: gate 檢查回報原因）---

    fn ws_of(p: &TempProject) -> speclink_core::workspace::Workspace {
        speclink_core::workspace::Workspace {
            root: p.root.clone(),
            spec_dir_name: "openspec".to_string(),
        }
    }

    fn write_record(p: &TempProject, entry: &EvidenceEntry) {
        let ws = ws_of(p);
        std::fs::create_dir_all(ws.touched_dir()).unwrap();
        let record = speclink_core::tasks::TouchedRecord {
            version: Some(2),
            change: "demo".to_string(),
            touched: Vec::new(),
            entries: vec![entry.clone()],
        };
        std::fs::write(
            ws.touched_dir().join("demo.json"),
            serde_json::to_string_pretty(&record).unwrap(),
        )
        .unwrap();
    }

    /// All-done tasks so gate checks reach the evidence judgment.
    fn all_done(p: &TempProject) {
        p.write_change_file(
            "tasks.md",
            &format!(
                "- [x] 1.1 first <!-- speclink-task:{TID_A} -->\n- [x] 1.2 second <!-- speclink-task:{TID_B} -->\n"
            ),
        );
    }

    #[test]
    fn archive_gate_passes_when_everything_aligns() {
        let p = TempProject::new("gate-ok");
        all_done(&p);
        let store = p.store();
        let bundle = produce_verify_bundle(&store, "demo").unwrap();
        write_record(&p, &entry_at(&bundle));
        assert_eq!(check_archive_evidence(&store, &ws_of(&p), "demo"), Ok(()));
    }

    #[test]
    fn archive_gate_rejects_stale_evidence_with_the_mismatched_basis() {
        let p = TempProject::new("gate-stale");
        all_done(&p);
        let store = p.store();
        let mut evidence = entry_at(&produce_verify_bundle(&store, "demo").unwrap());
        evidence.basis_digests.tasks = "sha256:deadbeef".to_string();
        write_record(&p, &evidence);
        match check_archive_evidence(&store, &ws_of(&p), "demo") {
            Err(ArchiveGateRejection::StaleEvidence { mismatched }) => {
                assert_eq!(mismatched, vec![BasisItem::Tasks], "rejection names the stale basis");
            }
            other => panic!("expected a stale-evidence rejection, got {other:?}"),
        }
    }

    #[test]
    fn archive_gate_rejects_open_tasks_and_missing_evidence() {
        // 預設 fixture 的 1.2 未勾 → TasksIncomplete。
        let p = TempProject::new("gate-open");
        let store = p.store();
        assert_eq!(
            check_archive_evidence(&store, &ws_of(&p), "demo"),
            Err(ArchiveGateRejection::TasksIncomplete { remaining: 1 })
        );
        // 全勾但無任何 evidence 記錄 → EvidenceMissing。
        all_done(&p);
        assert_eq!(
            check_archive_evidence(&store, &ws_of(&p), "demo"),
            Err(ArchiveGateRejection::EvidenceMissing)
        );
    }
}

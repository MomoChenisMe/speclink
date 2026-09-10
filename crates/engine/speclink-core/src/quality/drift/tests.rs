use super::*;
use crate::teststore::TestStore;

const META: &str = "schema: spec-driven\ncreated: 2026-07-13\n";

// --- Task 1: compute_spec_drift 純函式（只吃 Store 規格事實、確定性、無 git）---

#[test]
fn spec_drift_reports_specs_dimension_and_assumptions_deterministically() {
    // 規格面運算只吃 Store：delta 有一筆 ADDED 但正典已存在該需求 → 一條 stale
    // assumption（archive 會靜默 skip，drift 必須浮出）。
    let store = TestStore::with_meta("demo", META);
    store.put_artifact(
        "demo",
        "specs/auth/spec.md",
        "## ADDED Requirements\n\n### Requirement: Login\n",
    );
    store
        .write_canonical_spec("auth", "## Purpose\n\n### Requirement: Login\n")
        .unwrap();
    let change = store.find_change("demo").unwrap();

    let a = compute_spec_drift(&store, &change);
    let b = compute_spec_drift(&store, &change);

    assert_eq!(a.dimension.kind, "Specs");
    assert_eq!(a.dimension.status, "1 stale assumptions");
    assert_eq!(a.dimension.score, 4);
    assert!(a.dimension.contributes_to_total);
    assert_eq!(a.spec_assumptions.len(), 1);
    assert_eq!(a.spec_assumptions[0].operation, "ADDED");
    assert_eq!(a.spec_assumptions[0].requirement, "Login");

    // 相同輸入重複呼叫逐欄相同（純函式、無隱藏狀態、無時間依賴）。
    assert_eq!(a.dimension.status, b.dimension.status);
    assert_eq!(a.dimension.score, b.dimension.score);
    assert_eq!(a.spec_assumptions.len(), b.spec_assumptions.len());
}

#[test]
fn spec_drift_no_delta_specs_is_a_distinct_status() {
    let store = TestStore::with_meta("demo", META);
    let change = store.find_change("demo").unwrap();
    let r = compute_spec_drift(&store, &change);
    assert_eq!(r.dimension.status, "no delta specs");
    assert_eq!(r.dimension.score, 0);
    assert!(r.spec_assumptions.is_empty());
}

#[test]
fn spec_drift_holds_when_delta_targets_are_consistent() {
    // delta 有 ADDED 且正典尚無該需求 → 假設成立、零分。
    // 新開 capability 的 delta 自帶合格 Purpose：Purpose 守門與 archive 共用
    // 同一支判定（spec archive-merge「過期判定單源共用」），缺席會在這裡記一筆。
    let store = TestStore::with_meta("demo", META);
    store.put_artifact(
        "demo",
        "specs/auth/spec.md",
        "## Purpose\n\n本 capability 負責登入與登出的可觀察行為，涵蓋工作階段的建立、續期與撤銷三段流程及其失敗處置。\n\n## ADDED Requirements\n\n### Requirement: Logout\n",
    );
    let change = store.find_change("demo").unwrap();
    let r = compute_spec_drift(&store, &change);
    assert_eq!(r.dimension.status, "delta assumptions hold");
    assert_eq!(r.dimension.score, 0);
    assert!(r.spec_assumptions.is_empty());
}

// --- 過期判定單源共用（design「判定共用」；spec archive-merge「過期判定單源共用」）---

#[test]
fn drift_bulk_precheck_and_single_archive_share_one_verdict() {
    // spec Scenario「四處判定一致」：同一過期 MODIFIED 之下，drift 的 spec
    // assumption、bulk 預檢讀的違規清單與單筆 archive 的拒絕逐欄指向同一
    // capability 與需求名——共用 merge_violations 這一支判定。第四條腿
    // （validate）由 crates/adapters/speclink-cli/tests/it/validate_specs.rs 覆蓋。
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "tasks.md", "- [x] 1.1 done\n");
    store.put_artifact(
        "demo",
        "specs/auth/spec.md",
        "## MODIFIED Requirements\n\n### Requirement: Rotate tokens\n\nIt SHALL rotate.\n",
    );
    store
        .write_canonical_spec("auth", "## Purpose\n\nAuth.\n\n### Requirement: Login\n")
        .unwrap();
    let change = store.find_change("demo").unwrap();

    // bulk 預檢的來源＝引擎守門的來源。
    let violations = crate::archive::merge_violations(&store, "demo");
    assert_eq!(violations.len(), 1, "the pre-check sees exactly one violation");
    assert_eq!(violations[0].capability, "auth");
    assert_eq!(violations[0].operation, "MODIFIED");
    assert_eq!(violations[0].requirement, "Rotate tokens");

    // drift 的 Specs 維度逐欄同源。
    let assumptions = spec_assumptions(&store, &change);
    assert_eq!(assumptions.len(), violations.len(), "drift reports the same count");
    assert_eq!(assumptions[0].capability, violations[0].capability);
    assert_eq!(assumptions[0].operation, violations[0].operation);
    assert_eq!(assumptions[0].requirement, violations[0].requirement);
    assert_eq!(assumptions[0].reason, violations[0].reason, "one reason string, one source");

    // 單筆 archive 拒絕，訊息指向同一 capability 與需求名。
    let ws = Workspace {
        root: std::env::temp_dir().join("speclink-drift-single-source-ghost-root"),
        spec_dir_name: "openspec".to_string(),
    };
    let opts = crate::archive::ArchiveOptions { no_validate: true, ..Default::default() };
    let err = crate::archive::archive(&ws, &store, &change, &opts, None)
        .expect_err("the same stale delta refuses a single archive");
    let msg = err.to_string();
    assert!(msg.contains("auth"), "capability named: {msg}");
    assert!(msg.contains("Rotate tokens"), "requirement named: {msg}");
    assert!(msg.contains(&violations[0].reason), "the shared reason is rendered: {msg}");
}

#[test]
fn drift_reason_speaks_refusal_not_skip() {
    // design「判定共用」：reason 文案改為拒絕語意，欄位結構不變。
    let store = TestStore::with_meta("demo", META);
    store.put_artifact(
        "demo",
        "specs/auth/spec.md",
        "## ADDED Requirements\n\n### Requirement: Login\n",
    );
    store
        .write_canonical_spec("auth", "## Purpose\n\n### Requirement: Login\n")
        .unwrap();
    let change = store.find_change("demo").unwrap();
    let assumptions = spec_assumptions(&store, &change);
    assert_eq!(assumptions.len(), 1);
    assert!(
        assumptions[0].reason.contains("archive would refuse it"),
        "refusal wording, not skip: {}",
        assumptions[0].reason
    );
}

// --- Task 1: WorkspaceFacts 封閉結構逐欄可表達 有值／空值／不可用 ---

#[test]
fn workspace_facts_express_three_value_semantics_per_field() {
    use std::collections::BTreeMap;

    // 不可用：git 相關欄位皆 None（區別於「空值」）。
    let unavailable = WorkspaceFacts::default();
    assert!(unavailable.commit_window.is_none(), "commit window 不可用");
    assert!(unavailable.tracked_docs.is_none(), "tracked docs 不可用");
    assert!(unavailable.symbol_head_hits.is_none(), "symbol hits 不可用");

    // 空值：git 可用但無內容（Some(empty) 明確有別於 None）。
    let empty = WorkspaceFacts {
        commit_window: Some(Vec::new()),
        tracked_docs: Some(Vec::new()),
        symbol_head_hits: Some(Vec::new()),
        path_status: BTreeMap::new(),
        touched_files: Vec::new(),
    };
    assert_eq!(empty.commit_window.as_deref(), Some(&[][..]), "空 commit window");
    assert!(empty.tracked_docs.as_deref().unwrap().is_empty());

    // 有值：commit window 一筆、符號命中、路徑三態俱全。
    let mut path_status = BTreeMap::new();
    path_status.insert("a.rs".to_string(), PathKind::File);
    path_status.insert("gone.rs".to_string(), PathKind::Missing);
    path_status.insert("dir".to_string(), PathKind::Other);
    let present = WorkspaceFacts {
        commit_window: Some(vec![vec!["a.rs".to_string()]]),
        tracked_docs: Some(vec!["# doc\nLogin\n".to_string()]),
        symbol_head_hits: Some(vec!["Login".to_string()]),
        path_status,
        touched_files: vec!["a.rs".to_string()],
    };
    assert_eq!(present.commit_window.as_ref().unwrap().len(), 1);
    assert_eq!(present.path_status.get("a.rs"), Some(&PathKind::File));
    assert_eq!(present.path_status.get("gone.rs"), Some(&PathKind::Missing));
    assert_eq!(present.path_status.get("dir"), Some(&PathKind::Other));
}

// --- Task 2: compute_workspace_drift（四維度只讀 facts、缺席即 unavailable）---

fn assert_available(dim: &WorkspaceDimension, expected: &DriftDimension) {
    match dim {
        WorkspaceDimension::Available(d) => {
            assert_eq!(d.kind, expected.kind, "kind");
            assert_eq!(d.status, expected.status, "status of {}", expected.kind);
            assert_eq!(d.score, expected.score, "score of {}", expected.kind);
            assert_eq!(
                d.contributes_to_total, expected.contributes_to_total,
                "contributes of {}", expected.kind
            );
        }
        WorkspaceDimension::Unavailable { kind } => {
            panic!("expected {} available, got unavailable {kind}", expected.kind)
        }
    }
}

#[test]
fn workspace_facts_absent_marks_all_four_dimensions_unavailable_without_scores() {
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "design.md", "Uses `DriftReport`.");
    store.put_artifact("demo", "tasks.md", "- [ ] 1.1 edit `src/app.rs`\n");
    let change = store.find_change("demo").unwrap();

    let r = compute_workspace_drift(&store, &change, None);

    assert_eq!(r.dimensions.len(), 4);
    for (i, kind) in ["Time", "Structure", "Tasks", "Environment"].iter().enumerate() {
        match &r.dimensions[i] {
            WorkspaceDimension::Unavailable { kind: k } => assert_eq!(k, kind),
            other => panic!("dim {i} expected unavailable {kind}, got {other:?}"),
        }
    }
    // 缺席不得帶任何工作區訊號（不是 clean、不是零分）。
    assert!(r.broken_anchors.is_empty());
    assert!(r.tasks_maybe_resolved.is_empty());
    assert!(r.tasks_blocked_external.is_empty());
    assert_eq!(r.commits_since_created, 0);
}

#[test]
fn git_unavailable_facts_match_current_analyze_fallback_byte_for_byte() {
    // 「有 checkout 但 git 不可用」：facts 存在但 git 相關欄位 None。以非 git 的
    // 臨時 workspace 跑現行 analyze（git_available=false → 同一 fallback 路徑），
    // 與 compute_workspace_drift(facts=git 不可用) 的四維度逐欄對照。同一時鐘 →
    // 日數依賴的字串兩邊一致，測試與執行日期無關。
    let tmp = std::env::temp_dir()
        .join(format!("speclink-drift-wsparity-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let ws = Workspace { root: tmp.clone(), spec_dir_name: "openspec".to_string() };

    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "design.md", "Uses `DriftReport` and helper_fn.");
    store.put_artifact("demo", "tasks.md", "- [ ] 1.1 edit `src/app.rs`\n");
    let change = store.find_change("demo").unwrap();

    let expected = analyze(&ws, &store, &change);
    // git 不可用、checkout 存在：所有 git 欄位 None、fs 探測皆缺（tmp 為空）。
    let facts = WorkspaceFacts::default();
    let got = compute_workspace_drift(&store, &change, Some(&facts));

    // expected.dimensions 序：Time, Structure, Tasks, Specs, Environment。
    // got.dimensions 序：Time, Structure, Tasks, Environment。
    assert_available(&got.dimensions[0], &expected.dimensions[0]); // Time
    assert_available(&got.dimensions[1], &expected.dimensions[1]); // Structure
    assert_available(&got.dimensions[2], &expected.dimensions[2]); // Tasks
    assert_available(&got.dimensions[3], &expected.dimensions[4]); // Environment

    let got_broken: Vec<_> = got.broken_anchors.iter().map(|b| &b.anchor).collect();
    let exp_broken: Vec<_> = expected.broken_anchors.iter().map(|b| &b.anchor).collect();
    assert_eq!(got_broken, exp_broken, "broken anchors identical under git-unavailable");
    assert_eq!(got.tasks_maybe_resolved, expected.tasks_maybe_resolved);
    assert_eq!(got.tasks_blocked_external, expected.tasks_blocked_external);
    assert_eq!(got.commits_since_created, expected.commits_since_created);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn valid_facts_produce_the_git_available_four_dimensions_per_field() {
    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "design.md", "Refers to `DriftReport` and helper_fn here.");
    store.put_artifact(
        "demo",
        "tasks.md",
        "- [ ] 1.1 wire `src/app.rs`\n- [ ] 1.2 remove `src/old.rs`\n",
    );
    let change = store.find_change("demo").unwrap();

    let mut path_status = std::collections::BTreeMap::new();
    path_status.insert("src/app.rs".to_string(), PathKind::File);
    path_status.insert("src/old.rs".to_string(), PathKind::Missing);
    let facts = WorkspaceFacts {
        commit_window: Some(vec![
            vec!["src/app.rs".to_string()],
            vec!["README.md".to_string()],
        ]),
        tracked_docs: Some(Vec::new()),
        symbol_head_hits: Some(vec!["DriftReport".to_string()]),
        path_status,
        touched_files: Vec::new(),
    };
    let r = compute_workspace_drift(&store, &change, Some(&facts));

    // Structure: DriftReport 命中、helper_fn 未命中 → 1/2 broken → score 4。
    assert_available(
        &r.dimensions[1],
        &DriftDimension {
            kind: "Structure".to_string(),
            status: "1/2 anchors broken".to_string(),
            score: 4,
            contributes_to_total: true,
        },
    );
    assert_eq!(r.broken_anchors.len(), 1);
    assert_eq!(r.broken_anchors[0].anchor, "helper_fn");

    // Tasks: git 可用 → 依 commit window 判定 1 maybe-done、0 blocked。
    assert_available(
        &r.dimensions[2],
        &DriftDimension {
            kind: "Tasks".to_string(),
            status: "0 blocked, 1 maybe-done".to_string(),
            score: 0,
            contributes_to_total: true,
        },
    );
    assert_eq!(r.tasks_maybe_resolved.len(), 1);
    assert!(r.tasks_maybe_resolved[0].contains("src/app.rs"));
    assert!(r.tasks_blocked_external.is_empty());

    // Environment: 2 commits，其中 1 筆碰到本 change 檔案（src/app.rs）。
    assert_available(
        &r.dimensions[3],
        &DriftDimension {
            kind: "Environment".to_string(),
            status: "2 commits (1 touching this change's files)".to_string(),
            score: 0,
            contributes_to_total: false,
        },
    );
    assert_eq!(r.commits_since_created, 2);

    // Time: git 可用（無 "git unavailable" 後綴）；score 由 days_old 決定。
    let days = crate::preflight::days_old(change.meta.created.as_deref());
    let expected_time_score = if days > 14 { 3 } else if days > 5 { 1 } else { 0 };
    match &r.dimensions[0] {
        WorkspaceDimension::Available(d) => {
            assert_eq!(d.kind, "Time");
            assert!(!d.status.contains("git unavailable"), "git available: {}", d.status);
            assert_eq!(d.score, expected_time_score);
        }
        other => panic!("Time must be available, got {other:?}"),
    }
}

// --- Task 3: merge_drift_reports（單一 merger、coverage、stale）---

fn digests(spec: &str, tasks: &str, policy: &str) -> crate::tasks::BasisDigests {
    crate::tasks::BasisDigests {
        spec: spec.to_string(),
        tasks: tasks.to_string(),
        policy: policy.to_string(),
    }
}

#[test]
fn full_coverage_merge_equals_current_drift_report_field_for_field() {
    // 以非 git 臨時 workspace 對照：現行 analyze 的整份 DriftReport，應與
    // compute_spec_drift + compute_workspace_drift(git 不可用 facts) + merger
    // 逐欄（並逐位元 JSON）一致；coverage 與 stale 於 full/非 stale 時不出現。
    let tmp = std::env::temp_dir()
        .join(format!("speclink-drift-mergeparity-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let ws = Workspace { root: tmp.clone(), spec_dir_name: "openspec".to_string() };

    let store = TestStore::with_meta("demo", META);
    store.put_artifact("demo", "design.md", "Uses `DriftReport` and helper_fn.");
    store.put_artifact("demo", "tasks.md", "- [ ] 1.1 edit `src/app.rs`\n");
    store.put_artifact(
        "demo",
        "specs/auth/spec.md",
        "## MODIFIED Requirements\n\n### Requirement: Gone\n",
    );
    store.write_canonical_spec("auth", "## Purpose\n\n### Requirement: Other\n").unwrap();
    let change = store.find_change("demo").unwrap();

    let expected = analyze(&ws, &store, &change);

    let facts = WorkspaceFacts::default(); // git 不可用、checkout 存在
    let spec = compute_spec_drift(&store, &change);
    let workspace = compute_workspace_drift(&store, &change, Some(&facts));
    let combined = merge_drift_reports(&change, spec, workspace, None);

    assert_eq!(combined.coverage, Coverage::Full, "facts present → full coverage");
    assert!(combined.stale.is_none(), "no basis → not stale");

    let got = serde_json::to_value(&combined).unwrap();
    let want = serde_json::to_value(&expected).unwrap();
    assert_eq!(got, want, "combined full report is byte-identical to current DriftReport");

    // 選填欄位不得洩漏到 full 路徑輸出。
    let obj = got.as_object().unwrap();
    assert!(!obj.contains_key("coverage"), "coverage omitted on full path");
    assert!(!obj.contains_key("stale"), "stale omitted when not stale");

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn purpose_only_assumptions_route_the_recommendation_to_the_delta_not_ingest() {
    // Purpose 違規的正典不存在，ingest 修不了它——主建議改指向補寫
    // `## Purpose` 的 validate 指引（spec archive-merge「新 capability 缺
    // Purpose 的違規呈現三處一致」）。
    let store = TestStore::with_meta("demo", META);
    store.put_artifact(
        "demo",
        "specs/auth/spec.md",
        "## ADDED Requirements\n\n### Requirement: Login\n",
    );
    let change = store.find_change("demo").unwrap();
    let spec = compute_spec_drift(&store, &change);
    assert_eq!(spec.spec_assumptions.len(), 1, "premise: the purpose gate fires");
    let workspace = compute_workspace_drift(&store, &change, None);
    let combined = merge_drift_reports(&change, spec, workspace, None);
    let rec = &combined.report.primary_recommendation;
    assert!(rec.contains("## Purpose"), "recommendation names the missing section: {rec}");
    assert!(!rec.contains("ingest"), "ingest cannot fix a missing Purpose: {rec}");
}

#[test]
fn mixed_assumptions_still_route_to_ingest() {
    // 摻雜過期操作時 ingest 仍是對的第一步——只有純 Purpose 違規才改道。
    let store = TestStore::with_meta("demo", META);
    store.put_artifact(
        "demo",
        "specs/auth/spec.md",
        "## MODIFIED Requirements\n\n### Requirement: Gone\n",
    );
    store.put_artifact(
        "demo",
        "specs/token/spec.md",
        "## ADDED Requirements\n\n### Requirement: Fresh\n",
    );
    store.write_canonical_spec("auth", "## Purpose\n\n### Requirement: Other\n").unwrap();
    let change = store.find_change("demo").unwrap();
    let spec = compute_spec_drift(&store, &change);
    assert!(spec.spec_assumptions.len() >= 2, "premise: stale + purpose both fire");
    let workspace = compute_workspace_drift(&store, &change, None);
    let combined = merge_drift_reports(&change, spec, workspace, None);
    assert!(
        combined.report.primary_recommendation.contains("ingest"),
        "mixed violations keep the ingest route: {}",
        combined.report.primary_recommendation
    );
}

#[test]
fn workspace_absent_merges_to_spec_only_keeping_unavailable_dimensions() {
    let store = TestStore::with_meta("demo", META);
    store.put_artifact(
        "demo",
        "specs/auth/spec.md",
        "## ADDED Requirements\n\n### Requirement: Login\n",
    );
    store.write_canonical_spec("auth", "## Purpose\n\n### Requirement: Login\n").unwrap();
    let change = store.find_change("demo").unwrap();

    let spec = compute_spec_drift(&store, &change);
    let workspace = compute_workspace_drift(&store, &change, None);
    let combined = merge_drift_reports(&change, spec, workspace, None);

    assert_eq!(combined.coverage, Coverage::SpecOnly);
    // 五維度序仍為 Time, Structure, Tasks, Specs, Environment；四工作區維度標 unavailable。
    let dims = &combined.report.dimensions;
    assert_eq!(dims.len(), 5);
    for (i, kind) in ["Time", "Structure", "Tasks"].iter().enumerate() {
        assert_eq!(dims[i].kind, *kind);
        assert_eq!(dims[i].status, "unavailable", "{kind} 標 unavailable");
        assert!(!dims[i].contributes_to_total, "{kind} 不計分");
    }
    assert_eq!(dims[3].kind, "Specs", "Specs 仍在第 4 位且有值");
    assert_eq!(dims[4].kind, "Environment");
    assert_eq!(dims[4].status, "unavailable");
    // total 只含 Specs（缺席四維度不計為零分或任何分數）。
    assert_eq!(combined.report.total_score, combined.report.dimensions[3].score);

    // spec-only 時 coverage 出現於 JSON。
    let obj = serde_json::to_value(&combined).unwrap();
    assert_eq!(obj["coverage"], serde_json::json!("spec-only"));
}

#[test]
fn basis_mismatch_marks_stale_listing_only_the_mismatched_item() {
    let store = TestStore::with_meta("demo", META);
    let change = store.find_change("demo").unwrap();
    let spec = compute_spec_drift(&store, &change);
    let workspace = compute_workspace_drift(&store, &change, Some(&WorkspaceFacts::default()));

    // 僅 tasks digest 不符 → stale 只列 Tasks。
    let basis = DriftBasis {
        expected: digests("sha256:s", "sha256:t-old", "sha256:p"),
        current: digests("sha256:s", "sha256:t-new", "sha256:p"),
    };
    let combined = merge_drift_reports(&change, spec, workspace, Some(&basis));

    let stale = combined.stale.as_ref().expect("basis mismatch marks stale");
    assert_eq!(stale.mismatched, vec![DriftBasisItem::Tasks], "只列不符項");

    let obj = serde_json::to_value(&combined).unwrap();
    assert_eq!(obj["stale"]["mismatched"], serde_json::json!(["tasks"]));
}

#[test]
fn matching_basis_is_not_stale() {
    let store = TestStore::with_meta("demo", META);
    let change = store.find_change("demo").unwrap();
    let spec = compute_spec_drift(&store, &change);
    let workspace = compute_workspace_drift(&store, &change, Some(&WorkspaceFacts::default()));
    let same = digests("sha256:s", "sha256:t", "sha256:p");
    let basis = DriftBasis { expected: same.clone(), current: same };
    let combined = merge_drift_reports(&change, spec, workspace, Some(&basis));
    assert!(combined.stale.is_none(), "相同 basis 不標 stale");
}

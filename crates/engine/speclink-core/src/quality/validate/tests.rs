use super::*;
use crate::teststore::TestStore;

/// 一份合格的 Purpose 內容（60 字元以上，spec Scenario「新開 capability 的
/// Purpose 合格則通過」的取值）。
const GOOD_PURPOSE: &str = "本 capability 管理權杖的輪替與撤銷，涵蓋簽發、驗證與失效三段生命週期的可觀察行為，以及逾期權杖的清理時機。";

fn delta(purpose: Option<&str>, operation: &str) -> String {
    match purpose {
        Some(p) => format!("## Purpose\n\n{p}\n\n{operation}"),
        None => operation.to_string(),
    }
}

const ADDED: &str = "## ADDED Requirements\n\n### Requirement: Fresh\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";
const MODIFIED: &str = "## MODIFIED Requirements\n\n### Requirement: R1\n\nIt SHALL work harder.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";
const CANON: &str = "# auth Specification\n\n## Purpose\n\nAuth.\n\n## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n";

fn store_for(deltas: &[(&str, &str)], canon: &[(&str, &str)]) -> TestStore {
    let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
    for (cap, text) in deltas {
        store.put_artifact("demo", &crate::model::delta_spec_artifact(cap), text);
    }
    for (cap, text) in canon {
        store.canonical.borrow_mut().insert((*cap).to_string(), (*text).to_string());
    }
    store
}

fn result_for(deltas: &[(&str, &str)], canon: &[(&str, &str)]) -> ValidationResult {
    let store = store_for(deltas, canon);
    let change = crate::model::find_change(&store, "demo").expect("change resolves");
    validate_change(&store, &change, &crate::schema::spec_driven(), false)
}

// --- 新開 capability 的 Purpose 早期檢查（design D2；spec spec-validation
//     「新開 capability 的 change 驗證早期檢查」）---

#[test]
fn new_capability_without_purpose_is_an_error() {
    // spec Scenario「新開 capability 缺 Purpose 驗證報 error」。
    let r = result_for(&[("token", &delta(None, ADDED))], &[]);
    assert!(!r.valid, "驗證結果必須 invalid");
    let joined = r.errors.join("\n");
    assert!(joined.contains("token"), "error 指名該 capability: {joined}");
    assert!(joined.contains("## Purpose"), "error 附範例骨架: {joined}");
}

#[test]
fn new_capability_with_a_qualified_purpose_passes() {
    // spec Scenario「新開 capability 的 Purpose 合格則通過」。
    let r = result_for(&[("token", &delta(Some(GOOD_PURPOSE), ADDED))], &[]);
    assert!(r.valid, "合格 Purpose 不得報 error: {:?}", r.errors);
    assert!(r.warnings.is_empty(), "也不得報 warning: {:?}", r.warnings);
}

#[test]
fn new_capability_with_a_too_short_purpose_is_an_error() {
    // 判準三態的第三態：區段在、內容非空但不足門檻——同樣擋在 change 驗證。
    let r = result_for(&[("token", &delta(Some("管權杖。"), ADDED))], &[]);
    assert!(!r.valid, "過短 Purpose 必須 invalid: {:?}", r.errors);
    let joined = r.errors.join("\n");
    assert!(
        joined.contains(&crate::model::MIN_PURPOSE_LENGTH.to_string()),
        "error 報出門檻字元數: {joined}"
    );
}

#[test]
fn existing_capability_without_purpose_reports_nothing() {
    // spec Scenario「既有 capability 的 delta 不受 Purpose 檢查影響」：
    // 既有 capability 的 delta Purpose 屬忽略語意，缺席不構成違規。
    let r = result_for(&[("auth", &delta(None, MODIFIED))], &[("auth", CANON)]);
    assert!(r.valid, "既有 capability 不得因 Purpose 報 error: {:?}", r.errors);
    assert!(r.warnings.is_empty(), "也不得報 warning: {:?}", r.warnings);
}

// --- 新開 capability 的近似名 warning（design D5；spec spec-validation
//     「新開 capability 的近似名 warning」）---

#[test]
fn a_near_named_new_capability_warns_but_still_passes() {
    // spec Scenario「近似新名報 warning 且驗證仍通過」。
    let r = result_for(
        &[("authentication", &delta(Some(GOOD_PURPOSE), ADDED))],
        &[("auth", CANON)],
    );
    assert!(r.valid, "warning 不改變驗證結果: {:?}", r.errors);
    assert_eq!(r.warnings.len(), 1, "一筆近似名 warning: {:?}", r.warnings);
    let w = &r.warnings[0];
    assert!(w.contains("authentication"), "點名新目錄: {w}");
    assert!(w.contains("auth"), "含近似名: {w}");
    assert!(w.contains("existing name"), "指引一：沿用既有名: {w}");
    assert!(w.contains("ignore"), "指引二：確為新 capability 可忽略: {w}");
}

#[test]
fn the_warning_pool_includes_in_flight_deltas_of_other_changes() {
    // spec：建議池＝正典＋其他未封存 change 的 delta，與主閘同一份。
    let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
    store.metas.borrow_mut().insert(
        "add-sso".to_string(),
        "schema: spec-driven\ncreated: 2026-07-01\n".to_string(),
    );
    store.put_artifact(
        "add-sso",
        &crate::model::delta_spec_artifact("user-auth"),
        &delta(Some(GOOD_PURPOSE), ADDED),
    );
    store.put_artifact(
        "demo",
        &crate::model::delta_spec_artifact("user-authentication"),
        &delta(Some(GOOD_PURPOSE), ADDED),
    );
    let change = crate::model::find_change(&store, "demo").expect("change resolves");
    let r = validate_change(&store, &change, &crate::schema::spec_driven(), false);
    assert!(r.valid, "warning 不改變驗證結果: {:?}", r.errors);
    let joined = r.warnings.join("\n");
    assert!(joined.contains("user-auth"), "in-flight delta 進建議: {joined}");
    assert!(joined.contains("add-sso"), "標注來源 change: {joined}");
}

#[test]
fn sibling_new_capabilities_in_the_same_change_warn_each_other() {
    // 池含本 change 的其他 delta：同一個 change 內同時新開兩個近似名，
    // 兩筆 warning 互相點名——濾掉的只有受檢 capability 自身。
    let r = result_for(
        &[
            ("user-auth", &delta(Some(GOOD_PURPOSE), ADDED)),
            ("user-authentication", &delta(Some(GOOD_PURPOSE), ADDED)),
        ],
        &[],
    );
    assert!(r.valid, "warning 不改變驗證結果: {:?}", r.errors);
    assert_eq!(r.warnings.len(), 2, "兩個新名各得一筆: {:?}", r.warnings);
    let joined = r.warnings.join("\n");
    assert!(
        joined.contains("user-authentication (in-flight: demo)")
            && joined.contains("user-auth (in-flight: demo)"),
        "互相出現在對方的建議裡: {joined}"
    );
}

#[test]
fn a_case_variant_delta_counts_as_new_and_warns() {
    // 正典收錄與否是清單逐字比對——`Auth` 是新 capability：Purpose 早
    // 檢查照跑（合格即過），naming warning 折疊大小寫後點名 `auth`。
    let r = result_for(&[("Auth", &delta(Some(GOOD_PURPOSE), ADDED))], &[("auth", CANON)]);
    assert!(r.valid, "合格 Purpose 不報 error: {:?}", r.errors);
    assert_eq!(r.warnings.len(), 1, "近似名 warning 一筆: {:?}", r.warnings);
    assert!(r.warnings[0].contains("auth (canonical)"), "折疊後建議 auth: {:?}", r.warnings);
}

#[test]
fn a_same_named_delta_does_not_trigger_the_naming_warning() {
    // spec Scenario「既有 capability 的 delta 不觸發」。
    let r = result_for(&[("auth", &delta(None, MODIFIED))], &[("auth", CANON)]);
    assert!(r.valid, "同名 delta 照常通過: {:?}", r.errors);
    assert!(r.warnings.is_empty(), "同名不報近似 warning: {:?}", r.warnings);
}

#[test]
fn a_new_capability_without_near_names_stays_silent() {
    // spec Scenario「無近似名的新 capability 不報」：建議池空即不報，
    // 既有的 Purpose 早檢查照常執行。
    let clean = result_for(
        &[("zzz-unrelated", &delta(Some(GOOD_PURPOSE), ADDED))],
        &[("auth", CANON)],
    );
    assert!(clean.valid, "毫無交集不報: {:?}", clean.errors);
    assert!(clean.warnings.is_empty(), "無近似零 warning: {:?}", clean.warnings);

    let broken = result_for(&[("zzz-unrelated", &delta(None, ADDED))], &[("auth", CANON)]);
    assert!(!broken.valid, "Purpose 早檢查照常: {:?}", broken.errors);
}

// --- validate --specs 的正典規格驗證（design D4；spec spec-validation
//     「validate --specs 驗證正典規格」）---

fn canonical(purpose: Option<&str>) -> String {
    let head = "# auth Specification\n\n";
    let tail = "## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n";
    match purpose {
        Some(p) => format!("{head}## Purpose\n\n{p}\n\n{tail}"),
        None => format!("{head}{tail}"),
    }
}

#[test]
fn canonical_spec_missing_purpose_is_an_error() {
    // spec Scenario「缺 Purpose 區段報 error」。
    for text in [canonical(None), canonical(Some("   "))] {
        let r = validate_canonical_spec("auth", &text, false);
        assert!(!r.valid, "缺段／空內容必須 invalid: {text}");
        assert!(
            r.errors.iter().any(|e| e.contains("specs/auth/spec.md")),
            "error 帶邏輯路徑: {:?}",
            r.errors
        );
    }
}

#[test]
fn canonical_spec_too_short_purpose_only_warns_under_strict() {
    // spec Scenario「過短 Purpose 僅 strict 報 warning」。
    let text = canonical(Some("短短一句。"));
    let lenient = validate_canonical_spec("auth", &text, false);
    assert!(lenient.valid && lenient.warnings.is_empty(), "非 strict 不報過短: {lenient:?}");
    let strict = validate_canonical_spec("auth", &text, true);
    assert!(strict.valid, "過短只是 warning，不使規格 invalid: {:?}", strict.errors);
    assert_eq!(strict.warnings.len(), 1, "strict 報一筆 warning: {:?}", strict.warnings);
}

#[test]
fn canonical_spec_placeholder_purpose_warns_regardless_of_strict() {
    // spec Scenario「佔位 Purpose 以 warning 顯形」：佔位句長度恆超過門檻，
    // 長度判準攔不到它，需獨立的前綴判準；且不依附 strict。
    let placeholder = format!(
        "{} change 'demo'. Update Purpose after archive.",
        crate::model::PURPOSE_TBD_PREFIX
    );
    assert!(
        placeholder.chars().count() >= crate::model::MIN_PURPOSE_LENGTH,
        "前提：佔位句本身長於門檻"
    );
    for strict in [false, true] {
        let r = validate_canonical_spec("auth", &canonical(Some(&placeholder)), strict);
        assert!(r.valid, "佔位不是 error: {:?}", r.errors);
        assert_eq!(r.warnings.len(), 1, "佔位報 warning（strict={strict}）: {:?}", r.warnings);
    }
}

#[test]
fn canonical_spec_with_a_qualified_purpose_is_clean() {
    let r = validate_canonical_spec("auth", &canonical(Some(GOOD_PURPOSE)), true);
    assert!(r.valid && r.warnings.is_empty(), "合格規格零報: {r:?}");
}

#[test]
fn purpose_error_carries_the_repair_guidance() {
    // spec 需求段：error 訊息自帶修復指引與範例骨架，不得只報缺失。
    let r = result_for(&[("token", &delta(None, ADDED))], &[]);
    let joined = r.errors.join("\n");
    for key in ["## Purpose", "50"] {
        assert!(joined.contains(key), "修復指引缺關鍵內容 {key:?}: {joined}");
    }
    assert!(
        joined.lines().count() > 1,
        "範例骨架應以獨立行呈現，而非塞成一行: {joined}"
    );
}

// --- 手動標記位置檢查（design D3；spec manual-task-marker
//     「標記位置的 change 驗證檢查」）---

fn result_with_tasks(tasks_md: &str) -> ValidationResult {
    let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-08-12\n");
    store.put_artifact("demo", "tasks.md", tasks_md);
    store.put_artifact("demo", &crate::model::delta_spec_artifact("auth"), MODIFIED);
    store.canonical.borrow_mut().insert("auth".to_string(), CANON.to_string());
    let change = crate::model::find_change(&store, "demo").expect("change resolves");
    validate_change(&store, &change, &crate::schema::spec_driven(), false)
}

#[test]
fn task_number_before_the_marker_is_an_error() {
    // spec Scenario「編號在前報 error」。
    let r = result_with_tasks("- [ ] 6.2 [M] 手動驗收\n");
    assert!(!r.valid, "誤置必須使驗證 invalid: {r:?}");
    let joined = r.errors.join("\n");
    // design D3:路徑是含 change 目錄的邏輯路徑(正斜線),與零操作 parse error
    // 的 spec_path 同慣例——TestStore 的 change.dir 為 changes/demo。
    for key in ["changes/demo/tasks.md", "Task 1", "6.2 [M] 手動驗收", "- [ ] [M] 6.2 手動驗收"]
    {
        assert!(joined.contains(key), "error 缺 {key:?}: {joined}");
    }
}

#[test]
fn guidance_keeps_the_checkbox_state_and_stable_id_of_the_original_line() {
    // 已勾誤置行的修復例須忠實重建原行:`- [x]` 不得退成 `- [ ]`,尾部
    // ID 註解不得被抹掉——照訊息逐字改行是代理的常態。
    let r = result_with_tasks(
        "- [x] 5.2 [M] 手動驗收 <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->\n",
    );
    assert!(!r.valid);
    let joined = r.errors.join("\n");
    for key in [
        "- [x] [M] 5.2 手動驗收 <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->",
        "- [x] 5.2 [M] 手動驗收 <!-- speclink-task:tsk_01ARZ3NDEKTSV4RRFFQ69G5FAV -->",
    ] {
        assert!(joined.contains(key), "修復例缺 {key:?}: {joined}");
    }
    assert!(!joined.contains("- [ ]"), "不得出現退勾的 checkbox: {joined}");
}

#[test]
fn guidance_for_a_bare_marker_skips_the_example_lines() {
    // 描述僅為 [M](無內文)時,正誤例兩行重建不出有意義的內容——
    // 略去兩行,留下規則句本身。
    let r = result_with_tasks("- [ ]  [M]\n");
    assert!(!r.valid, "空內文的行首殘留仍須報 error: {r:?}");
    let joined = r.errors.join("\n");
    assert!(joined.contains("exactly one space"), "規則句仍在: {joined}");
    assert!(!joined.contains("Write:"), "空內文不給重建例: {joined}");
}

#[test]
fn marker_that_missed_the_prefix_slot_names_the_single_space_rule() {
    // spec Scenario「行首殘留報 error」：checkbox 後兩個空格,前綴槽漏接。
    let r = result_with_tasks("- [ ]  [M] 手測匯入\n");
    assert!(!r.valid, "誤置必須使驗證 invalid: {r:?}");
    let joined = r.errors.join("\n");
    assert!(joined.contains("exactly one space"), "error 須點名恰一個空格: {joined}");
}

#[test]
fn correct_prefix_and_mid_description_mentions_report_nothing() {
    // spec Scenario「正確前綴與中段字面提及不報」。
    let r = result_with_tasks(
        "- [ ] [M] 手測匯入\n- [x] 1.1 前綴剝除迴圈同時接受 `[P]` 與 `[M]` 的說明文字\n",
    );
    assert!(r.valid, "正確寫法不得報 error: {:?}", r.errors);
    assert!(r.warnings.is_empty(), "也不得報 warning: {:?}", r.warnings);
}

#[test]
fn a_change_without_tasks_validates_exactly_as_before() {
    // tasks.md 缺席時本檢查零輸出——既有驗證結果逐位元不變。
    let with_empty = result_with_tasks("");
    let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-08-12\n");
    store.put_artifact("demo", &crate::model::delta_spec_artifact("auth"), MODIFIED);
    store.canonical.borrow_mut().insert("auth".to_string(), CANON.to_string());
    let change = crate::model::find_change(&store, "demo").expect("change resolves");
    let absent = validate_change(&store, &change, &crate::schema::spec_driven(), false);
    assert_eq!(
        (absent.valid, absent.errors, absent.warnings),
        (with_empty.valid, with_empty.errors, with_empty.warnings),
        "有無 tasks.md 的驗證結果須全等"
    );
}

#[test]
fn existing_errors_are_listed_before_the_marker_ones() {
    // 凍結順序慣例同 Purpose 檢查：既有錯誤先列,本檢查後附。
    let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-08-12\n");
    store.put_artifact("demo", "tasks.md", "- [ ] 6.2 [M] 手動驗收\n");
    store.put_artifact("demo", &crate::model::delta_spec_artifact("token"), &delta(None, ADDED));
    let change = crate::model::find_change(&store, "demo").expect("change resolves");
    let r = validate_change(&store, &change, &crate::schema::spec_driven(), false);
    assert_eq!(r.errors.len(), 2, "兩類錯誤各一筆: {:?}", r.errors);
    assert!(r.errors[0].contains("## Purpose"), "既有 Purpose 錯誤在前: {:?}", r.errors);
    assert!(r.errors[1].contains("tasks.md"), "標記錯誤在後: {:?}", r.errors);
}
// --- change 驗證納入合併守門（design D1–D3；spec spec-validation
//     「change 驗證納入合併守門」）---

/// 正典 `R1` 帶兩個 scenario，用來驗「未宣告的 scenario 移除」。
const CANON_TWO_SCENARIOS: &str = "# auth Specification\n\n## Purpose\n\nAuth.\n\n## Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n\n#### Scenario: fine\n\n- **WHEN** idle\n- **THEN** rests\n";
/// 正典只有 `R1`，delta 卻 MODIFIED 一個已不存在的 `R9`。
const MODIFIED_R9: &str = "## MODIFIED Requirements\n\n### Requirement: R9\n\nIt SHALL work harder.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";

#[test]
fn the_structural_entry_point_carries_no_merge_gate_errors() {
    // spec Scenario「archive 的拒絕輸出不變」對應的引擎面：archive 家族的
    // 前置只看結構層，同一份會被守門拒收的 change 在那裡零報。
    let store = store_for(&[("auth", MODIFIED_R9)], &[("auth", CANON)]);
    let change = crate::model::find_change(&store, "demo").expect("change resolves");
    let schema = crate::schema::spec_driven();
    let structural = validate_change_structural(&store, &change, false);
    let full = validate_change(&store, &change, &schema, false);
    assert!(structural.valid, "結構層對守門違規零報: {:?}", structural.errors);
    assert!(structural.errors.is_empty(), "結構層 error 全空: {:?}", structural.errors);
    assert!(!full.valid, "validate 必須因守門違規而 invalid: {full:?}");
    assert_eq!(full.errors.len(), 1, "恰一條守門 error: {:?}", full.errors);
}

#[test]
fn a_modified_target_that_is_gone_reads_as_the_gate_worded_error() {
    // spec Scenario「MODIFIED 目標不存在時 validate 報 error」與 Example 表第一列。
    let r = result_for(&[("auth", MODIFIED_R9)], &[("auth", CANON)]);
    assert_eq!(
        r.errors,
        vec![
            "specs/auth/spec.md: MODIFIED 'R9': target requirement no longer exists in \
the canonical spec (see: speclink drift demo)"
                .to_string()
        ],
        "組字逐字對應 Example 表"
    );
    assert!(!r.valid, "守門違規使結果 invalid");
}

#[test]
fn an_added_name_the_canon_already_carries_reads_as_the_gate_worded_error() {
    // Example 表第二列。
    const ADDED_R1: &str = "## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";
    let r = result_for(&[("auth", ADDED_R1)], &[("auth", CANON)]);
    assert_eq!(
        r.errors,
        vec![
            "specs/auth/spec.md: ADDED 'R1': already exists in the canonical spec — \
archive would refuse it (see: speclink drift demo)"
                .to_string()
        ]
    );
}

#[test]
fn a_renamed_without_a_to_target_reads_as_the_gate_worded_error() {
    // Example 表第三列。ADDED 一併寫著，delta 才有可套用的操作——本例要測的
    // 是懸空 RENAMED 的組字，不是零操作的 parse error。
    const DANGLING_RENAME: &str = "## ADDED Requirements\n\n### Requirement: Fresh\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n\n## RENAMED Requirements\n\n- FROM: `### Requirement: R1`\n";
    let r = result_for(&[("auth", DANGLING_RENAME)], &[("auth", CANON)]);
    assert_eq!(
        r.errors,
        vec![
            "specs/auth/spec.md: RENAMED 'R1': RENAMED operation names no TO: target \
(see: speclink drift demo)"
                .to_string()
        ]
    );
}

#[test]
fn an_undeclared_scenario_removal_is_caught_and_clears_once_declared() {
    // spec Scenario「未宣告的 scenario 移除被 validate 抓到」。
    let dropped = result_for(&[("auth", MODIFIED)], &[("auth", CANON_TWO_SCENARIOS)]);
    assert_eq!(dropped.errors.len(), 1, "恰一條守門 error: {:?}", dropped.errors);
    assert!(
        dropped.errors[0].starts_with(
            "specs/auth/spec.md: MODIFIED 'R1': drops canonical scenario(s) fine"
        ),
        "reason 逐字沿用守門字串: {}",
        dropped.errors[0]
    );
    assert!(
        dropped.errors[0].ends_with("(see: speclink drift demo)"),
        "每行自帶補救指向: {}",
        dropped.errors[0]
    );

    let declared = format!("{MODIFIED}\n<!-- REMOVED-SCENARIO: fine -->\n");
    let r = result_for(&[("auth", &declared)], &[("auth", CANON_TWO_SCENARIOS)]);
    assert!(r.valid, "補上宣告後通過: {:?}", r.errors);
}

#[test]
fn the_purpose_gate_and_duplicate_names_are_not_listed_twice() {
    // spec Scenario「Purpose 與重複需求名不重複列」：errors 恰 2 條。
    const DUP_ADDED: &str = "## ADDED Requirements\n\n### Requirement: Fresh\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n\n### Requirement: Fresh\n\nIt SHALL work twice.\n\n#### Scenario: again\n\n- **WHEN** reused\n- **THEN** works\n";
    let r = result_for(&[("token", DUP_ADDED)], &[]);
    assert_eq!(r.errors.len(), 2, "Purpose 與重複名各一條，無第三條: {:?}", r.errors);
    assert!(r.errors[0].contains("## Purpose"), "既有 Purpose error: {:?}", r.errors);
    assert!(
        r.errors[1].contains("Duplicate requirement 'Fresh' in ADDED section"),
        "既有重複名 error: {:?}",
        r.errors
    );

    // 跨區段同名走同一條去重規則：結構層已報 appears in both，守門不再補刀。
    const CROSS_SECTION: &str = "## MODIFIED Requirements\n\n### Requirement: R1\n\nIt SHALL work harder.\n\n## REMOVED Requirements\n\n### Requirement: R1\n\nIt SHALL go.\n";
    let cross = result_for(&[("auth", CROSS_SECTION)], &[("auth", CANON)]);
    assert_eq!(cross.errors.len(), 1, "只留結構層那一條: {:?}", cross.errors);
    assert!(
        cross.errors[0].contains("appears in both MODIFIED and REMOVED sections"),
        "留下的是結構層的文字: {:?}",
        cross.errors
    );
}

#[test]
fn a_renamed_endpoint_collision_is_only_visible_to_the_gate() {
    // spec Scenario「RENAMED 端點撞名只有守門看得到」：結構層的重複名掃描
    // 不看 RENAMED 區段，這條撞名只有守門報得出來。
    const RENAME_COLLIDES: &str = "## ADDED Requirements\n\n### Requirement: R2\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n\n## RENAMED Requirements\n\n- FROM: `### Requirement: R1`\n- TO: `### Requirement: R2`\n";
    let r = result_for(&[("auth", RENAME_COLLIDES)], &[("auth", CANON)]);
    // 撞名違規的 operation 是逗號串——這是守門唯一偏離「單一 operation」
    // 形狀的組字，整行凍結。
    assert_eq!(
        r.errors,
        vec![
            "specs/auth/spec.md: ADDED, RENAMED 'R2': appears more than once across this \
delta's operation sections (see: speclink drift demo)"
                .to_string()
        ]
    );
}

#[test]
fn a_name_written_twice_that_also_collides_with_the_canon_lists_one_gate_line() {
    // 守門逐筆掃描 delta 的需求：同名在同一區段寫兩次、又撞正典時會產出兩筆
    // 相同的 ADDED_EXISTS 違規——validate 同字只列一行。
    const ADDED_R1_TWICE: &str = "## ADDED Requirements\n\n### Requirement: R1\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n\n### Requirement: R1\n\nIt SHALL work twice.\n\n#### Scenario: again\n\n- **WHEN** reused\n- **THEN** works\n";
    let r = result_for(&[("auth", ADDED_R1_TWICE)], &[("auth", CANON)]);
    assert_eq!(
        r.errors,
        vec![
            "specs/auth/spec.md: Duplicate requirement 'R1' in ADDED section".to_string(),
            "specs/auth/spec.md: ADDED 'R1': already exists in the canonical spec — \
archive would refuse it (see: speclink drift demo)"
                .to_string(),
        ],
        "結構層一條、守門一條，沒有重複行"
    );
}

#[test]
fn a_duplicated_name_that_is_also_a_rename_endpoint_keeps_the_gate_line() {
    // spec「未被結構檢查涵蓋的撞名（含 RENAMED 端點）仍 SHALL 列出」：同名既在
    // ADDED 重複、又是 RENAMED 的 TO 端點——結構層只報得出 ADDED 那一半，
    // 守門那筆 `ADDED, RENAMED` 撞名不得被去重吞掉。
    const DUP_AND_RENAME: &str = "## ADDED Requirements\n\n### Requirement: R2\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n\n### Requirement: R2\n\nIt SHALL work twice.\n\n#### Scenario: again\n\n- **WHEN** reused\n- **THEN** works\n\n## RENAMED Requirements\n\n- FROM: `### Requirement: R1`\n- TO: `### Requirement: R2`\n";
    let r = result_for(&[("auth", DUP_AND_RENAME)], &[("auth", CANON)]);
    assert_eq!(
        r.errors,
        vec![
            "specs/auth/spec.md: Duplicate requirement 'R2' in ADDED section".to_string(),
            "specs/auth/spec.md: ADDED, RENAMED 'R2': appears more than once across this \
delta's operation sections (see: speclink drift demo)"
                .to_string(),
        ]
    );
}

#[test]
fn a_requirement_under_a_purpose_headed_section_is_not_swallowed() {
    // Purpose 類的去重只認真正的 Purpose 守門（operation 與 requirement 都要
    // 對上）：新 capability 的 delta 若寫了 `## PURPOSE Requirements` 標頭，
    // 底下的需求拿到的是 CANON_ABSENT 違規，validate 必須照列——否則
    // validate 全綠而 archive 仍拒收。
    const PURPOSE_HEADED: &str = "## PURPOSE Requirements\n\n### Requirement: X\n\nIt SHALL.\n\n## ADDED Requirements\n\n### Requirement: Fresh\n\nIt SHALL work.\n\n#### Scenario: ok\n\n- **WHEN** used\n- **THEN** works\n";
    let r = result_for(
        &[("token", &delta(Some(GOOD_PURPOSE), PURPOSE_HEADED))],
        &[],
    );
    assert_eq!(
        r.errors,
        vec![
            "specs/token/spec.md: PURPOSE 'X': canonical spec for this capability does not \
exist (see: speclink drift demo)"
                .to_string()
        ]
    );
}

#[test]
fn gate_errors_are_listed_after_every_structural_one() {
    // design D2：守門 error 一律排在既有 error 之後，含 `[M]` 標記 error。
    let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-08-12\n");
    store.put_artifact("demo", "tasks.md", "- [ ] 6.2 [M] 手動驗收\n");
    store.put_artifact("demo", &crate::model::delta_spec_artifact("auth"), MODIFIED_R9);
    store.canonical.borrow_mut().insert("auth".to_string(), CANON.to_string());
    let change = crate::model::find_change(&store, "demo").expect("change resolves");
    let r = validate_change(&store, &change, &crate::schema::spec_driven(), false);
    assert_eq!(r.errors.len(), 2, "標記與守門各一條: {:?}", r.errors);
    assert!(r.errors[0].contains("tasks.md"), "既有標記 error 在前: {:?}", r.errors);
    assert!(r.errors[1].contains("MODIFIED 'R9'"), "守門 error 在後: {:?}", r.errors);
}

#[test]
fn the_gate_reports_regardless_of_strict() {
    // design Non-Goals：守門違規不依附 `--strict`，兩種模式都是 error。
    let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-08-12\n");
    store.put_artifact("demo", &crate::model::delta_spec_artifact("auth"), MODIFIED_R9);
    store.canonical.borrow_mut().insert("auth".to_string(), CANON.to_string());
    let change = crate::model::find_change(&store, "demo").expect("change resolves");
    for strict in [false, true] {
        let r = validate_change(&store, &change, &crate::schema::spec_driven(), strict);
        assert_eq!(r.errors.len(), 1, "strict={strict} 一樣報: {:?}", r.errors);
    }
}

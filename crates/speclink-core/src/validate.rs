//! Structural validation of changes and specs.

use crate::model::{self, Change};
use crate::schema::Schema;
use crate::store::Store;
use crate::tasks;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ValidationResult {
    pub change: String,
    pub errors: Vec<String>,
    pub valid: bool,
    pub warnings: Vec<String>,
}

/// validate 的目標集（design D4）：兩個獨立的開關，由旗標組合解出。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidateTargets {
    pub changes: bool,
    pub specs: bool,
}

/// 旗標組合 → 目標集的單一定義：`--specs` 單獨＝只驗規格；`--all`＝兩邊都驗；
/// `--specs --changes` 同傳＝聯集（與 `--all` 等效）；兩旗標皆缺席＝只驗 changes
/// （現行行為不變）。`--specs` 與 item 同傳是參數錯誤：--specs 驗的是全部正典
/// 規格、無法指定單一份，聯集語意只會讓人以為指定生效了——大聲拒絕。fs 與
/// remote 兩條路徑讀同一支，旗標語意與錯誤措辭不會各自漂移。
pub fn validate_targets(
    item: Option<&str>,
    all: bool,
    changes: bool,
    specs: bool,
) -> Result<ValidateTargets, String> {
    if item.is_some() && specs {
        return Err(
            "--specs validates the canonical specs and cannot be combined with a name; \
run `speclink validate --specs` alone, or `speclink validate --all` for both sides"
                .to_string(),
        );
    }
    Ok(ValidateTargets {
        changes: !specs || changes || all || item.is_some(),
        specs: specs || all,
    })
}

/// 一份正典規格的驗證（design D4）：缺 `## Purpose` 區段或內容為空＝error；
/// 內容不足門檻＝warning（僅 strict 報）；內容仍為 archive 佔位＝warning（不依附
/// strict——佔位句恆長於門檻，長度判準抓不到它）。fs 與 remote 兩模式共用這一支：
/// 前者從 Store 讀正典，後者由 client 取回內容後本地執行，輸出因此同形。
pub fn validate_canonical_spec(cap: &str, text: &str, strict: bool) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    // 訊息裡的路徑是邏輯路徑（openspec 相對），一律正斜線。
    let at = format!("specs/{cap}/spec.md");
    match model::purpose_content(text) {
        None => errors.push(format!("{at}: {}", model::PurposeDefect::Missing.reason())),
        Some(content) => {
            let len = content.chars().count();
            if content.starts_with(model::PURPOSE_TBD_PREFIX) {
                warnings.push(format!(
                    "{at}: Purpose is still the placeholder written by archive — \
replace it with what this capability actually covers"
                ));
            } else if strict && len < model::MIN_PURPOSE_LENGTH {
                warnings.push(format!("{at}: {}", model::PurposeDefect::TooShort(len).reason()));
            }
        }
    }
    ValidationResult {
        change: cap.to_string(),
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

/// 全部正典規格的驗證結果，依 capability id 排序（沿 listing 的既有慣例——
/// Store 的列舉順序是檔案系統順序，不可依賴）。
pub fn validate_specs(store: &dyn Store, strict: bool) -> Vec<ValidationResult> {
    let mut caps = store.list_canonical_capabilities();
    caps.sort();
    caps.iter()
        .map(|cap| {
            let text = store.read_canonical_spec(cap).unwrap_or_default();
            validate_canonical_spec(cap, &text, strict)
        })
        .collect()
}

/// 新開 capability 的 Purpose 不合格時的 error 訊息（design D2）：說明規則、
/// 點名不合格原因，並附可直接照抄的範例骨架——propose 收尾跑 validate 失敗即修，
/// 錯誤訊息本身就是教材。
fn purpose_guidance(cap: &str, defect: &model::PurposeDefect) -> String {
    format!(
        "specs/{cap}/spec.md: new capability '{cap}' — {reason}. \
A capability the canonical specs do not carry yet must open its delta with a \
`## Purpose` section of one or two sentences ({min} characters or more); it becomes \
the new canonical spec's Purpose when the change is archived.\n    \
Add this above the first operation section:\n      ## Purpose\n\n      \
<what this capability covers and where its boundary lies — one or two sentences>",
        reason = defect.reason(),
        min = model::MIN_PURPOSE_LENGTH,
    )
}

fn misplaced_marker_guidance(path: &str, m: &tasks::Misplaced) -> String {
    let marker = tasks::MANUAL_MARKER;
    let cause = match m.kind {
        tasks::MisplacedMarker::AfterNumber => "the task number took the marker slot",
        tasks::MisplacedMarker::PrefixSlotMissed => {
            "the marker slot takes exactly one space after the checkbox"
        }
    };
    let mut msg = format!(
        "{path}: Task {id} (\"{desc}\"): misplaced `{marker}` marker — {cause}, so the engine \
reads `{marker}` as description text and counts the task as code work that never completes.",
        id = m.task_id,
        desc = m.description,
    );
    // The repair examples reproduce the original line faithfully — checkbox state and
    // trailing stable-ID comment included — so following them verbatim cannot uncheck a
    // done task or sever its identity. A description that is nothing but the marker has
    // no line worth rebuilding; the rule sentence above already says it all.
    let body = m.description.replacen(marker, "", 1);
    let body = body.split_whitespace().collect::<Vec<_>>().join(" ");
    if !body.is_empty() {
        let check = if m.done { 'x' } else { ' ' };
        let tail = m
            .stable_id
            .as_deref()
            .map(|id| format!(" <!-- speclink-task:{id} -->"))
            .unwrap_or_default();
        // The wrong line is rebuilt rather than quoted — the double space of a missed
        // prefix slot would otherwise be invisible in the example.
        let wrong = match m.kind {
            tasks::MisplacedMarker::AfterNumber => m.description.clone(),
            tasks::MisplacedMarker::PrefixSlotMissed => format!(" {}", m.description),
        };
        msg.push_str(&format!(
            "\n    Write:  - [{check}] {marker} {body}{tail}\n    Not:    - [{check}] {wrong}{tail}"
        ));
    }
    msg
}

/// 新開 capability 撞近似既有名的 warning 文字（design D5）：近似名各附來源
/// 標注與 Purpose 首行，指引兩條路——同一 capability 就改用既有名、確為新
/// capability 可忽略本警告。
fn naming_warning(cap: &str, suggestions: &[crate::capname::KnownName]) -> String {
    let mut msg = format!(
        "specs/{cap}/spec.md: new capability '{cap}' is close to existing names:\n"
    );
    for s in suggestions {
        msg.push_str(&format!("      - {}\n", crate::capname::suggestion_line(s)));
    }
    msg.push_str(
        "    If this is the same capability, rename the delta directory to the existing name; \
if it really is new, ignore this warning.",
    );
    msg
}

/// Validate a change's artifacts structurally.
pub fn validate_change(store: &dyn Store, change: &Change, _schema: &Schema, strict: bool) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // validate is lenient: a missing proposal is NOT an error, and a scenario-less
    // requirement is NOT an error. The one hard error is an EXISTING delta spec file that parses
    // to zero applied operations (empty, RENAMED-only, or an operation-less requirement). The
    // informational "No delta specs found" warning fires only when there is not even a capability
    // directory under specs/.
    let caps = store.delta_capabilities(&change.name);
    // 正典收錄與否以清單逐字比對（同建立點主閘）——canonical_spec_exists 走
    // 檔案系統，大小寫不敏感的 fs 會把 `Auth` 當 `auth` 而讓兩張網同時靜默。
    let canon = store.list_canonical_capabilities();
    // 近似名建議池與 cap 無關，整個 change 建一次；只有存在新開 capability
    // 時才需要（池會讀全部正典規格取 Purpose 首行，白讀太貴）。
    let pool = if caps.iter().any(|cap| !canon.contains(cap)) {
        crate::capname::suggestion_pool(store)
    } else {
        Vec::new()
    };
    for cap in &caps {
        let spec_path = change.dir.join("specs").join(cap).join("spec.md");
        let text = store
            .read_artifact(&change.name, &model::delta_spec_artifact(cap))
            .unwrap_or_default();
        if !model::has_delta_operation(&text) {
            // 訊息裡的路徑是邏輯路徑（openspec 相對），凍結的正典形式是正斜線；
            // Windows 的 PathBuf::join 會補反斜線，渲染時統一回正斜線。
            errors.push(format!(
                "{}: Parse error: Invalid format: Delta spec must contain at least one operation (ADDED, MODIFIED, REMOVED, or RENAMED)",
                spec_path.to_string_lossy().replace('\\', "/")
            ));
        }
        // 新開 capability（正典尚無同名規格）的 Purpose 早期檢查（design D2）：
        // 不合格即 error，訊息自帶修復指引。既有 capability 的 delta Purpose 屬
        // 忽略語意，這裡零報——向後相容。既有錯誤先列，這條後補，凍結項的順序不動。
        if !canon.contains(cap) {
            if let Some(defect) = model::purpose_defect(&text) {
                errors.push(purpose_guidance(cap, &defect));
            }
            // 近似名第二網（design D5）：與建立點主閘同一建議池與排序。有建議
            // 即 warning——不改變 valid，涵蓋 ingest 或手寫檔案繞過 CLI 的入口。
            // 池含本 change 的其他 delta（同 change 內兩個近似新名也要互相看見），
            // 濾掉的只有受檢 capability 自身。
            let known: Vec<crate::capname::KnownName> = pool
                .iter()
                .filter(|k| {
                    !(k.name == *cap
                        && k.source == crate::capname::Source::InFlight(change.name.clone()))
                })
                .cloned()
                .collect();
            let suggestions = crate::capname::suggest(cap, &known);
            if !suggestions.is_empty() {
                warnings.push(naming_warning(cap, &suggestions));
            }
        }
        // Duplicate requirement names are hard errors: the same
        // name twice inside one ADDED/MODIFIED/REMOVED section, or the same name across
        // two different sections. Reported with the change-relative path.
        let rel = spec_path
            .strip_prefix(&change.dir)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| spec_path.to_string_lossy().to_string())
            .replace('\\', "/");
        let mut section = "";
        let mut seen_in: Vec<(String, Vec<&str>)> = Vec::new(); // name -> sections (ordered)
        let mut reported_dup: Vec<(String, &str)> = Vec::new();
        let mut reported_cross: Vec<String> = Vec::new();
        for line in text.lines() {
            let t = line.trim_start();
            if let Some(rest) = t.strip_prefix("## ") {
                let head = rest.split_whitespace().next().unwrap_or("");
                if rest.trim_end().ends_with("Requirements")
                    && matches!(head, "ADDED" | "MODIFIED" | "REMOVED")
                {
                    section = match head {
                        "ADDED" => "ADDED",
                        "MODIFIED" => "MODIFIED",
                        _ => "REMOVED",
                    };
                } else {
                    section = "";
                }
            } else if let Some(name) = t.strip_prefix("### Requirement:") {
                if section.is_empty() {
                    continue;
                }
                let name = name.trim().to_string();
                let entry = match seen_in.iter_mut().find(|(n, _)| *n == name) {
                    Some(e) => e,
                    None => {
                        seen_in.push((name.clone(), Vec::new()));
                        seen_in.last_mut().unwrap()
                    }
                };
                if entry.1.contains(&section) {
                    if !reported_dup.iter().any(|(n, s)| *n == name && *s == section) {
                        errors.push(format!(
                            "{rel}: Duplicate requirement '{name}' in {section} section"
                        ));
                        reported_dup.push((name.clone(), section));
                    }
                } else {
                    if let Some(first) = entry.1.first() {
                        if !reported_cross.contains(&name) {
                            errors.push(format!(
                                "{rel}: Requirement '{name}' appears in both {first} and {section} sections"
                            ));
                            reported_cross.push(name.clone());
                        }
                    }
                    entry.1.push(section);
                }
            }
        }
    }
    // 手動標記位置檢查（design D3）：`[M]` 寫在前綴槽外時解析不到,任務被靜默算成
    // 寫碼任務。既有錯誤先列,這條後補,凍結項的順序不動。路徑與零操作 parse error
    // 同慣例:含 change 目錄的邏輯路徑,渲染統一正斜線。
    let tasks_md = store.read_artifact(&change.name, "tasks.md").unwrap_or_default();
    let misplaced = tasks::misplaced_markers(&tasks::parse(&tasks_md));
    if !misplaced.is_empty() {
        let path = change.dir.join("tasks.md").to_string_lossy().replace('\\', "/");
        errors.extend(misplaced.iter().map(|m| misplaced_marker_guidance(&path, m)));
    }

    let has_cap_dirs = store.has_capability_dirs(&change.name);
    if caps.is_empty() && !has_cap_dirs {
        warnings.push("No delta specs found".to_string());
    }
    let _ = strict;

    let valid = errors.is_empty();
    ValidationResult {
        change: change.name.clone(),
        errors,
        valid,
        warnings,
    }
}

#[cfg(test)]
mod tests {
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

    fn result_for(deltas: &[(&str, &str)], canon: &[(&str, &str)]) -> ValidationResult {
        let store = TestStore::with_meta("demo", "schema: spec-driven\ncreated: 2026-07-01\n");
        for (cap, text) in deltas {
            store.put_artifact("demo", &crate::model::delta_spec_artifact(cap), text);
        }
        for (cap, text) in canon {
            store.canonical.borrow_mut().insert((*cap).to_string(), (*text).to_string());
        }
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
}


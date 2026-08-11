//! Structural validation of changes and specs.

use crate::model::{self, Change};
use crate::schema::Schema;
use crate::store::Store;
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
/// （現行行為不變）。fs 與 remote 兩條路徑讀同一支，旗標語意不會各自漂移。
pub fn validate_targets(item: Option<&str>, all: bool, changes: bool, specs: bool) -> ValidateTargets {
    ValidateTargets {
        changes: !specs || changes || all || item.is_some(),
        specs: specs || all,
    }
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
        if !store.canonical_spec_exists(cap) {
            if let Some(defect) = model::purpose_defect(&text) {
                errors.push(purpose_guidance(cap, &defect));
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
}


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

/// 一份正式規格的驗證（design D4）：缺 `## Purpose` 區段或內容為空＝error；
/// 內容不足門檻＝warning（僅 strict 報）；內容仍為 archive 佔位＝warning（不依附
/// strict——佔位句恆長於門檻，長度判準抓不到它）。fs 與 remote 兩模式共用這一支：
/// 前者從 Store 讀正式規格，後者由 client 取回內容後本地執行，輸出因此同形。
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

/// 全部正式規格的驗證結果，依 capability id 排序（沿 listing 的既有慣例——
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

/// 只含結構檢查的 change 驗證（design D1）：archive 家族的驗證前置專用。
/// 那裡的前置後面緊接自己的合併守門與 `merge_refusal` 聚合文案；前置若也含守門，
/// 違規會先以「Validation failed:」的形狀跳出來，等於改了凍結輸出。輸出與本
/// change 之前的 `validate_change` 逐位元相同。
pub(crate) fn validate_change_structural(
    store: &dyn Store,
    change: &Change,
    strict: bool,
) -> ValidationResult {
    structural_pass(store, change, strict).result
}

/// change 驗證的入口（design D1）：結構檢查的結果，加上與 archive 相同的合併
/// 守門違規。守門判斷沿用 `archive::capability_violations`（`merge_violations`
/// 的逐 capability 本體；結構層讀過的 delta 內文直接傳入，不重讀）——validate
/// 是 spec archive-merge「過期判定單源共用」的第四個共用者，不平行實作。
///
/// 每筆違規化為一條 error（design D2，組字見 `MergeViolation::validation_error`），
/// 一律排在既有結構 error 之後，`valid` 仍為 `errors.is_empty()`，且不依附 `strict`。
pub fn validate_change(
    store: &dyn Store,
    change: &Change,
    _schema: &Schema,
    strict: bool,
) -> ValidationResult {
    let StructuralPass { mut result, deltas } = structural_pass(store, change, strict);
    for d in &deltas {
        let canonical = store.read_canonical_spec(&d.capability);
        for v in crate::archive::capability_violations(&d.capability, &d.text, canonical.as_deref())
        {
            // 去重（design D3）：Purpose 類一律略過——結構層的 Purpose error 已含
            // 範例骨架，比守門的 reason 更完整。
            if v.is_purpose_gate() {
                continue;
            }
            // 撞名類只在結構層已報過該需求名時略過；含 RENAMED 端點的撞名結構層
            // 看不到那一端，即使同名已被報過仍列出。
            if v.is_section_collision()
                && !v.involves_rename()
                && d.reported_names.contains(&v.requirement)
            {
                continue;
            }
            // 守門逐筆掃描 delta 的需求：同名在同一區段寫兩次、又撞正典時會產出
            // 兩筆相同的違規——同字只列一行。
            let line = v.validation_error(&change.name);
            if !result.errors.contains(&line) {
                result.errors.push(line);
            }
        }
    }
    result.valid = result.errors.is_empty();
    result
}

/// 結構層對一份 delta 讀到、守門追加時還會用到的事實（design D1／D3）。
struct DeltaFacts {
    capability: String,
    /// delta 內文——守門直接用，不再讀一次 Store。
    text: String,
    /// 已被 Duplicate／appears in both error 點名的需求名，供撞名去重。
    reported_names: Vec<String>,
}

struct StructuralPass {
    result: ValidationResult,
    /// 依 Store 列舉順序，與 `merge_violations` 的走訪順序相同。
    deltas: Vec<DeltaFacts>,
}

/// 結構檢查本體。
fn structural_pass(store: &dyn Store, change: &Change, strict: bool) -> StructuralPass {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut deltas: Vec<DeltaFacts> = Vec::new();

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
    // 時才需要（池會讀全部正式規格取 Purpose 首行，白讀太貴）。
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
        deltas.push(DeltaFacts {
            capability: cap.clone(),
            reported_names: reported_dup
                .iter()
                .map(|(n, _)| n.clone())
                .chain(reported_cross.iter().cloned())
                .collect(),
            text,
        });
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
    StructuralPass {
        result: ValidationResult {
            change: change.name.clone(),
            errors,
            valid,
            warnings,
        },
        deltas,
    }
}

#[cfg(test)]
mod tests;

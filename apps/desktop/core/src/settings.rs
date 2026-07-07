//! 設定讀寫橋接：`.speclink.yaml`（tools）與 `openspec/config.yaml`（政策欄位）
//! 的載入快照與雙重驗證寫入（design D5）。純函式部分在 speclink-core（config.rs
//! 的 update_*_text）；檔案讀寫、寫前後驗證與技能同步在本層。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;
use speclink_core::config::{AppConfig, ToolEntry, WorkflowConfig};
use speclink_core::skills::Tool;
use speclink_core::workspace::Workspace;

// Tauri 殼經本 crate 取用政策欄位與 context 三態型別，不直接依賴 speclink-core。
pub use speclink_core::config::{ContextEdit, WorkflowPolicyFields};

/// 設定頁載入快照：兩檔各自的欄位現值與可選的解析錯誤。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsSnapshot {
    pub app: AppSettings,
    pub workflow: WorkflowSettings,
}

/// `.speclink.yaml` 面：內建工具選集與自訂描述子的存在標記。
/// `parse_error` 有值＝檔案存在但解析失敗——前端停用該檔表單並拒寫。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub tools: Vec<String>,
    pub custom_tools: Vec<String>,
    pub parse_error: Option<String>,
}

/// `openspec/config.yaml` 面：呈現檔案內的原始欄位值（非四層 resolve 後的
/// 有效政策——GUI 編輯的對象是這份檔案）。未設定的欄位呈預設值狀態。
/// `context`／`rules` 為「專案說明」「產出規則」的現值；`schema_artifacts`
/// 為活躍 schema 的 artifact id（引擎顯示序），是產出規則分節的固定鍵來源
/// ——解析失敗時給空（表單停用，不在壞檔上呈現猜測的分節）。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSettings {
    pub locale: Option<String>,
    pub spec_locale: Option<String>,
    pub tdd: bool,
    pub audit: bool,
    pub context: Option<String>,
    pub rules: BTreeMap<String, Vec<String>>,
    pub schema_artifacts: Vec<String>,
    pub parse_error: Option<String>,
}

/// 讀取設定頁快照。三態各自獨立：檔案缺席＝預設值狀態（無 parse_error）；
/// 可解析＝實際欄位值；解析失敗＝parse_error 單行訊息（刻意浮出，對比引擎
/// `WorkflowConfig::from_text`／`AppConfig::load` 的靜默 fallback）。
pub fn read_settings_at(root: &Path) -> Result<SettingsSnapshot, String> {
    let ws = discover(root)?;
    let app = match read_opt(&ws.app_config()) {
        None => AppSettings { tools: Vec::new(), custom_tools: Vec::new(), parse_error: None },
        Some(text) => match serde_yaml::from_str::<AppConfig>(&text) {
            Ok(cfg) => {
                let mut tools = Vec::new();
                let mut custom_tools = Vec::new();
                for entry in &cfg.tools {
                    match entry {
                        ToolEntry::Builtin(name) => {
                            if let Some(t) = Tool::parse(name) {
                                let canonical = t.name().to_string();
                                if !tools.contains(&canonical) {
                                    tools.push(canonical);
                                }
                            }
                        }
                        ToolEntry::Descriptor(d) => custom_tools
                            .push(d.name.clone().unwrap_or_else(|| "(unnamed)".to_string())),
                    }
                }
                AppSettings { tools, custom_tools, parse_error: None }
            }
            Err(e) => AppSettings {
                tools: Vec::new(),
                custom_tools: Vec::new(),
                parse_error: Some(single_line(&e.to_string())),
            },
        },
    };
    let workflow = match read_opt(&workflow_config_path(&ws)) {
        None => WorkflowSettings {
            locale: None,
            spec_locale: None,
            tdd: false,
            audit: false,
            context: None,
            rules: Default::default(),
            schema_artifacts: schema_artifact_ids(&ws, &WorkflowConfig::default()),
            parse_error: None,
        },
        Some(text) => match serde_yaml::from_str::<WorkflowConfig>(&text) {
            Ok(cfg) => WorkflowSettings {
                locale: cfg.locale.clone(),
                spec_locale: cfg.spec_locale.clone(),
                tdd: cfg.tdd.unwrap_or(false),
                audit: cfg.audit.unwrap_or(false),
                context: cfg.context.clone(),
                rules: cfg.rules.clone(),
                schema_artifacts: schema_artifact_ids(&ws, &cfg),
                parse_error: None,
            },
            Err(e) => WorkflowSettings {
                locale: None,
                spec_locale: None,
                tdd: false,
                audit: false,
                context: None,
                rules: Default::default(),
                // 壞檔無從得知活躍 schema——不呈現猜測的分節（表單一併停用）。
                schema_artifacts: Vec::new(),
                parse_error: Some(single_line(&e.to_string())),
            },
        },
    };
    Ok(SettingsSnapshot { app, workflow })
}

/// 活躍 schema 的 artifact id（引擎顯示序＝`status::display_order` 的拓撲序，
/// 與 status 輸出一致）。schema 解析失敗或找不到時給空——與壞檔同語意：不在
/// 未知 schema 上呈現猜測的固定鍵。
fn schema_artifact_ids(ws: &Workspace, cfg: &WorkflowConfig) -> Vec<String> {
    match speclink_core::schema::resolve_with(Some(ws), &cfg.schema_name()) {
        Some(Ok(schema)) => speclink_core::status::display_order(&schema)
            .into_iter()
            .map(|a| a.id.clone())
            .collect(),
        _ => Vec::new(),
    }
}

/// 寫入 `openspec/config.yaml` 的政策欄位（design D5 雙重驗證）：core 純函式
/// 改寫（寫前解析原文失敗即中止）→ 驗證新文字可解析且目標欄位值正確 →
/// 寫檔 → 回讀再驗。任一步失敗回指明檔案與階段的單行 Err，磁碟檔案維持原內容
/// ——絕不留下不可解析的設定檔。
///
/// `fields` 是四欄位的完整目標狀態（非 patch）：呼叫端必須先以
/// `read_settings_at` 取得現值再改寫，否則留在預設的欄位會被清掉。
pub fn write_workflow_fields_at(
    root: &Path,
    fields: &speclink_core::config::WorkflowPolicyFields,
) -> Result<(), String> {
    let ws = discover(root)?;
    let file = format!("{}/config.yaml", ws.spec_dir_name);
    let path = workflow_config_path(&ws);
    let original = read_opt(&path).unwrap_or_default();
    let new_text = speclink_core::config::update_workflow_config_text(
        &original,
        fields,
        &speclink_core::config::ContextEdit::Keep,
        None,
    )
    .map_err(|e| single_line(&e.to_string()))?;
    verify_workflow_text(&new_text, fields, &file, "pre-write verification")?;
    std::fs::write(&path, &new_text).map_err(|e| format!("{file}: write failed: {e}"))?;
    let reread = read_opt(&path)
        .ok_or_else(|| format!("{file}: verify after write failed: file unreadable"))?;
    verify_workflow_text(&reread, fields, &file, "verify after write")
}

/// 寫入 `openspec/config.yaml` 的「專案說明」與「產出規則」（design D4：與政策
/// 欄位同一套雙重驗證流程）：解析原文取政策現值（壞檔 loud 中止、檔案未動）→
/// core 純函式改寫（政策欄位回填現值、不受內容寫入波及）→ 驗證新文字 → 寫檔 →
/// 回讀再驗。任一步失敗回指明檔案與階段的單行 Err，磁碟檔案維持原內容。
///
/// `rules` 為 `None` 不動 rules；`Some` 為整份代換（節序即寫入序，條目 trim、
/// 空條目滌除、空節移除、全空移除 rules 鍵——清理語意在 core 純函式落實）。
pub fn write_workflow_content_at(
    root: &Path,
    context: &ContextEdit,
    rules: Option<&[(String, Vec<String>)]>,
) -> Result<(), String> {
    let ws = discover(root)?;
    let file = format!("{}/config.yaml", ws.spec_dir_name);
    let path = workflow_config_path(&ws);
    let original = read_opt(&path).unwrap_or_default();
    let current: WorkflowConfig = if original.trim().is_empty() {
        WorkflowConfig::default()
    } else {
        serde_yaml::from_str(&original).map_err(|e| {
            format!("{file}: pre-write verification failed: {}", single_line(&e.to_string()))
        })?
    };
    let fields = WorkflowPolicyFields {
        locale: current.locale.clone(),
        spec_locale: current.spec_locale.clone(),
        tdd: current.tdd.unwrap_or(false),
        audit: current.audit.unwrap_or(false),
    };
    // 期望效果（驗證基準）——獨立於 core 的清理實作再算一次，作為第二道防線。
    let want_context = match context {
        ContextEdit::Keep => current.context.clone(),
        ContextEdit::Set(v) if !v.trim().is_empty() => Some(v.clone()),
        ContextEdit::Set(_) | ContextEdit::Remove => None,
    };
    let want_rules: BTreeMap<String, Vec<String>> = match rules {
        None => current.rules.clone(),
        Some(sections) => sections
            .iter()
            .filter_map(|(k, v)| {
                let cleaned: Vec<String> = v
                    .iter()
                    .map(|e| e.trim().to_string())
                    .filter(|e| !e.is_empty())
                    .collect();
                (!cleaned.is_empty()).then(|| (k.clone(), cleaned))
            })
            .collect(),
    };
    let new_text =
        speclink_core::config::update_workflow_config_text(&original, &fields, context, rules)
            .map_err(|e| single_line(&e.to_string()))?;
    verify_workflow_content_text(&new_text, &want_context, &want_rules, &fields, &file, "pre-write verification")?;
    std::fs::write(&path, &new_text).map_err(|e| format!("{file}: write failed: {e}"))?;
    let reread = read_opt(&path)
        .ok_or_else(|| format!("{file}: verify after write failed: file unreadable"))?;
    verify_workflow_content_text(&reread, &want_context, &want_rules, &fields, &file, "verify after write")
}

/// 驗證一份 config.yaml 文字可解析，且 context／rules 與政策欄位皆為期望值。
fn verify_workflow_content_text(
    text: &str,
    want_context: &Option<String>,
    want_rules: &BTreeMap<String, Vec<String>>,
    fields: &WorkflowPolicyFields,
    file: &str,
    stage: &str,
) -> Result<(), String> {
    let cfg: WorkflowConfig = serde_yaml::from_str(text)
        .map_err(|e| format!("{file}: {stage} failed: {}", single_line(&e.to_string())))?;
    if cfg.context == *want_context
        && cfg.rules == *want_rules
        && cfg.locale == fields.locale
        && cfg.spec_locale == fields.spec_locale
        && cfg.tdd.unwrap_or(false) == fields.tdd
        && cfg.audit.unwrap_or(false) == fields.audit
    {
        Ok(())
    } else {
        Err(format!("{file}: {stage} failed: rewritten values do not match the request"))
    }
}

/// 驗證一份 config.yaml 文字可解析且政策欄位值與請求一致。
fn verify_workflow_text(
    text: &str,
    fields: &speclink_core::config::WorkflowPolicyFields,
    file: &str,
    stage: &str,
) -> Result<(), String> {
    let cfg: WorkflowConfig = serde_yaml::from_str(text)
        .map_err(|e| format!("{file}: {stage} failed: {}", single_line(&e.to_string())))?;
    if cfg.locale == fields.locale
        && cfg.spec_locale == fields.spec_locale
        && cfg.tdd.unwrap_or(false) == fields.tdd
        && cfg.audit.unwrap_or(false) == fields.audit
    {
        Ok(())
    } else {
        Err(format!("{file}: {stage} failed: rewritten values do not match the request"))
    }
}

/// 寫入 `.speclink.yaml` 的內建工具選集（同一套雙重驗證），成功後呼叫
/// `speclink_core::init::update` 全同步技能（新選工具生成、取消工具清理殘留），
/// 同步失敗浮出錯誤而非靜默。未知工具名在任何寫入之前被拒。
pub fn write_tools_at(root: &Path, tools: &[String]) -> Result<(), String> {
    let ws = discover(root)?;
    let mut selected: Vec<Tool> = Vec::new();
    for name in tools {
        let t = Tool::parse(name)
            .ok_or_else(|| format!("unknown tool '{name}' (supported: claude, codex)"))?;
        if !selected.contains(&t) {
            selected.push(t);
        }
    }
    let file = ".speclink.yaml";
    let path = ws.app_config();
    let original = read_opt(&path).unwrap_or_default();
    let new_text = speclink_core::config::update_app_config_tools_text(&original, &selected)
        .map_err(|e| single_line(&e.to_string()))?;
    verify_app_text(&new_text, &selected, file, "pre-write verification")?;
    std::fs::write(&path, &new_text).map_err(|e| format!("{file}: write failed: {e}"))?;
    let reread = read_opt(&path)
        .ok_or_else(|| format!("{file}: verify after write failed: file unreadable"))?;
    verify_app_text(&reread, &selected, file, "verify after write")?;
    speclink_core::init::update(&ws.root).map_err(|e| single_line(&e.to_string()))?;
    Ok(())
}

/// 驗證一份 .speclink.yaml 文字可解析且內建工具選集與請求一致（順序無關）。
fn verify_app_text(text: &str, selected: &[Tool], file: &str, stage: &str) -> Result<(), String> {
    let cfg: AppConfig = serde_yaml::from_str(text)
        .map_err(|e| format!("{file}: {stage} failed: {}", single_line(&e.to_string())))?;
    let mut got: Vec<Tool> = Vec::new();
    for entry in &cfg.tools {
        if let ToolEntry::Builtin(name) = entry {
            if let Some(t) = Tool::parse(name) {
                if !got.contains(&t) {
                    got.push(t);
                }
            }
        }
    }
    let mut want = selected.to_vec();
    want.sort_by_key(Tool::name);
    got.sort_by_key(Tool::name);
    if want == got {
        Ok(())
    } else {
        Err(format!("{file}: {stage} failed: rewritten tool selection does not match the request"))
    }
}

fn discover(root: &Path) -> Result<Workspace, String> {
    Workspace::discover(root).ok_or_else(|| format!("not a speclink project: {}", root.display()))
}

fn workflow_config_path(ws: &Workspace) -> PathBuf {
    ws.spec_dir().join("config.yaml")
}

/// 檔案缺席回 None；讀取失敗以外的內容原文回傳。
fn read_opt(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// serde_yaml 錯誤訊息壓成單行（多行時取首行），符合 Err 單行訊息契約。
fn single_line(msg: &str) -> String {
    msg.lines().next().unwrap_or(msg).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testfixture::FixtureRoot;

    // --- read_settings_at：三態讀取（檔案缺席／可解析／解析失敗） ---

    #[test]
    fn read_settings_absent_files_report_defaults_without_parse_error() {
        // FixtureRoot 只建 openspec/changes——兩份設定檔皆缺席＝預設值狀態。
        let fx = FixtureRoot::new("settings-absent");
        let s = read_settings_at(fx.root()).expect("read ok");
        assert_eq!(s.app.tools, Vec::<String>::new());
        assert_eq!(s.app.custom_tools, Vec::<String>::new());
        assert_eq!(s.app.parse_error, None);
        assert_eq!(s.workflow.locale, None);
        assert_eq!(s.workflow.spec_locale, None);
        assert!(!s.workflow.tdd);
        assert!(!s.workflow.audit);
        assert_eq!(s.workflow.parse_error, None);
    }

    #[test]
    fn read_settings_parses_actual_values_and_flags_custom_descriptors() {
        let fx = FixtureRoot::new("settings-values");
        fx.write(
            ".speclink.yaml",
            "tools:\n  - claude\n  - name: wad-harness\n    skills_dir: .wad/skills\n    instructions_file: WAD.md\n",
        );
        fx.write(
            "openspec/config.yaml",
            "schema: spec-driven\nlocale: tw\nspec_locale: auto\ntdd: true\naudit: true\n",
        );
        let s = read_settings_at(fx.root()).expect("read ok");
        assert_eq!(s.app.tools, vec!["claude".to_string()]);
        assert_eq!(s.app.custom_tools, vec!["wad-harness".to_string()]);
        assert_eq!(s.app.parse_error, None);
        assert_eq!(s.workflow.locale.as_deref(), Some("tw"));
        assert_eq!(s.workflow.spec_locale.as_deref(), Some("auto"));
        assert!(s.workflow.tdd);
        assert!(s.workflow.audit);
        assert_eq!(s.workflow.parse_error, None);
    }

    #[test]
    fn read_settings_surfaces_parse_errors_instead_of_silent_defaults() {
        // 對比引擎的靜默 fallback：GUI 載入必須把解析失敗浮出（spec 需求
        // 「設定寫入具解析驗證且失敗浮出」的載入面）。兩檔各自獨立。
        let fx = FixtureRoot::new("settings-badwf");
        fx.write(".speclink.yaml", "tools:\n  - claude\n");
        fx.write("openspec/config.yaml", "rules: [unclosed\n");
        let s = read_settings_at(fx.root()).expect("read ok");
        assert_eq!(s.app.parse_error, None);
        let msg = s.workflow.parse_error.expect("workflow parse error surfaced");
        assert!(!msg.is_empty() && !msg.contains('\n'), "single line: {msg:?}");

        let fx = FixtureRoot::new("settings-badapp");
        fx.write(".speclink.yaml", "tools: [unclosed\n");
        fx.write("openspec/config.yaml", "locale: tw\n");
        let s = read_settings_at(fx.root()).expect("read ok");
        let msg = s.app.parse_error.expect("app parse error surfaced");
        assert!(!msg.is_empty() && !msg.contains('\n'), "single line: {msg:?}");
        assert_eq!(s.workflow.parse_error, None);
        assert_eq!(s.workflow.locale.as_deref(), Some("tw"));
    }

    #[test]
    fn read_settings_outside_a_project_is_an_error() {
        let dir = std::env::temp_dir();
        assert!(read_settings_at(&dir).is_err());
    }

    // --- 寫入：雙重驗證與失敗浮出（spec 需求「設定寫入具解析驗證且失敗浮出」） ---

    use speclink_core::config::WorkflowPolicyFields;
    use std::path::Path;

    fn read(path: &Path) -> String {
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    fn set_readonly(path: &Path, ro: bool) {
        let mut perms = std::fs::metadata(path).unwrap().permissions();
        perms.set_readonly(ro);
        std::fs::set_permissions(path, perms).unwrap();
    }

    #[test]
    fn write_workflow_fields_replaces_targets_and_keeps_untouched_keys() {
        // spec Scenario「寫入政策欄位且未觸及鍵原樣保留」。
        let fx = FixtureRoot::new("wf-write");
        let doc = "schema: spec-driven\ncontext: |\n  keep me\nrules:\n  proposal:\n    - keep rule\n";
        fx.write("openspec/config.yaml", doc);
        let fields = WorkflowPolicyFields { tdd: true, ..Default::default() };
        write_workflow_fields_at(fx.root(), &fields).expect("write ok");
        let text = read(&fx.root().join("openspec/config.yaml"));
        let new: WorkflowConfig = serde_yaml::from_str(&text).expect("output parses");
        let orig: WorkflowConfig = serde_yaml::from_str(doc).unwrap();
        assert_eq!(new.tdd, Some(true));
        assert_eq!(new.context, orig.context);
        assert_eq!(new.rules, orig.rules);
        assert_eq!(new.schema, orig.schema);
    }

    #[test]
    fn write_workflow_fields_creates_config_when_absent() {
        let fx = FixtureRoot::new("wf-write-fresh");
        let fields = WorkflowPolicyFields { locale: Some("tw".into()), ..Default::default() };
        write_workflow_fields_at(fx.root(), &fields).expect("write ok");
        let text = read(&fx.root().join("openspec/config.yaml"));
        let new: WorkflowConfig = serde_yaml::from_str(&text).expect("output parses");
        assert_eq!(new.locale.as_deref(), Some("tw"));
    }

    #[test]
    fn write_workflow_fields_refuses_unparseable_original_and_leaves_file_intact() {
        // spec Scenario「解析失敗的檔案拒絕寫入」＋「寫入驗證失敗檔案不變」的寫前面。
        let fx = FixtureRoot::new("wf-write-bad");
        let bad = "rules: [unclosed\n";
        fx.write("openspec/config.yaml", bad);
        let err = write_workflow_fields_at(fx.root(), &WorkflowPolicyFields::default())
            .expect_err("must refuse");
        assert!(err.contains("config.yaml"), "must name the file: {err}");
        assert!(!err.contains('\n'), "single line: {err:?}");
        assert_eq!(read(&fx.root().join("openspec/config.yaml")), bad, "file must be untouched");
    }

    #[test]
    fn write_workflow_fields_surfaces_write_failure_with_file_and_stage() {
        // readonly 檔案觸發寫檔階段失敗：Err 指明檔案與階段、內容逐字元不變。
        let fx = FixtureRoot::new("wf-write-ro");
        let doc = "locale: tw\n";
        fx.write("openspec/config.yaml", doc);
        let path = fx.root().join("openspec/config.yaml");
        set_readonly(&path, true);
        let err = write_workflow_fields_at(fx.root(), &WorkflowPolicyFields::default())
            .expect_err("must fail");
        set_readonly(&path, false);
        assert!(err.contains("config.yaml"), "must name the file: {err}");
        assert!(err.contains("write"), "must name the stage: {err}");
        assert!(!err.contains('\n'), "single line: {err:?}");
        assert_eq!(read(&path), doc, "file must be untouched");
    }

    #[test]
    fn write_tools_syncs_skills_for_newly_selected_codex() {
        // spec Scenario「tools 變更後技能同步」：加選 codex 生成 AGENTS.md marker
        // 與 .agents/skills/。從 init 過的 claude 專案出發。
        let fx = FixtureRoot::new("tools-write-add");
        std::fs::remove_dir_all(fx.root().join("openspec")).unwrap();
        crate::project::init_project_at(fx.root(), &["claude".into()]).expect("init ok");
        write_tools_at(fx.root(), &["claude".into(), "codex".into()]).expect("write ok");
        let app = read(&fx.root().join(".speclink.yaml"));
        assert!(app.contains("claude") && app.contains("codex"), "tools recorded: {app}");
        assert!(read(&fx.root().join("AGENTS.md")).contains("<!-- SPECLINK:START"));
        assert!(fx.root().join(".agents").join("skills").is_dir());
    }

    #[test]
    fn write_tools_prunes_deselected_tool_leftovers() {
        let fx = FixtureRoot::new("tools-write-prune");
        std::fs::remove_dir_all(fx.root().join("openspec")).unwrap();
        crate::project::init_project_at(fx.root(), &["claude".into(), "codex".into()])
            .expect("init ok");
        assert!(fx.root().join(".agents").join("skills").is_dir(), "precondition");
        write_tools_at(fx.root(), &["claude".into()]).expect("write ok");
        let app = read(&fx.root().join(".speclink.yaml"));
        assert!(app.contains("claude") && !app.contains("codex"), "tools recorded: {app}");
        // prune：speclink-* 技能目錄移除、marker 區塊自 AGENTS.md 剝除。
        let agents_skills = fx.root().join(".agents").join("skills");
        if agents_skills.is_dir() {
            let leftover: Vec<_> = std::fs::read_dir(&agents_skills)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().starts_with("speclink-"))
                .collect();
            assert!(leftover.is_empty(), "speclink-* skills must be pruned: {leftover:?}");
        }
        if let Ok(agents_md) = std::fs::read_to_string(fx.root().join("AGENTS.md")) {
            assert!(!agents_md.contains("<!-- SPECLINK:START"), "marker must be stripped");
        }
    }

    #[test]
    fn write_tools_refuses_unparseable_original_and_leaves_file_intact() {
        let fx = FixtureRoot::new("tools-write-bad");
        let bad = "tools: [unclosed\n";
        fx.write(".speclink.yaml", bad);
        let err = write_tools_at(fx.root(), &["claude".into()]).expect_err("must refuse");
        assert!(err.contains(".speclink.yaml"), "must name the file: {err}");
        assert!(!err.contains('\n'), "single line: {err:?}");
        assert_eq!(read(&fx.root().join(".speclink.yaml")), bad, "file must be untouched");
    }

    #[test]
    fn write_tools_rejects_unknown_tool_name_before_touching_disk() {
        let fx = FixtureRoot::new("tools-write-unknown");
        let doc = "tools:\n  - claude\n";
        fx.write(".speclink.yaml", doc);
        let err = write_tools_at(fx.root(), &["claude".into(), "vscode".into()])
            .expect_err("must reject");
        assert!(err.contains("vscode"), "must name the offender: {err}");
        assert_eq!(read(&fx.root().join(".speclink.yaml")), doc, "file must be untouched");
    }

    // --- context／rules／schemaArtifacts 讀取擴充（design D3） ---

    const SPEC_DRIVEN_IDS: [&str; 4] = ["proposal", "design", "specs", "tasks"];

    fn sections(pairs: &[(&str, &[&str])]) -> Vec<(String, Vec<String>)> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
            .collect()
    }

    #[test]
    fn read_settings_includes_context_rules_and_schema_artifacts() {
        let fx = FixtureRoot::new("settings-content");
        fx.write(
            "openspec/config.yaml",
            "schema: spec-driven\ncontext: |\n  第一行\n  第二行\nrules:\n  tasks:\n    - 先寫失敗測試\n    - 更新文件\n  proposal:\n    - 提案必須列出影響的 crates\n",
        );
        let s = read_settings_at(fx.root()).expect("read ok");
        assert_eq!(s.workflow.context.as_deref(), Some("第一行\n第二行\n"));
        assert_eq!(
            s.workflow.rules.get("tasks").map(Vec::as_slice),
            Some(["先寫失敗測試".to_string(), "更新文件".to_string()].as_slice()),
            "entry order preserved"
        );
        assert_eq!(
            s.workflow.rules.get("proposal").map(Vec::as_slice),
            Some(["提案必須列出影響的 crates".to_string()].as_slice())
        );
        assert_eq!(s.workflow.schema_artifacts, SPEC_DRIVEN_IDS);
        // 橋接 payload 一律 camelCase。
        let json = serde_json::to_value(&s).expect("serialize");
        assert!(json["workflow"].get("schemaArtifacts").is_some(), "camelCase key");
        assert!(json["workflow"].get("context").is_some());
        assert!(json["workflow"].get("rules").is_some());
    }

    #[test]
    fn read_settings_absent_workflow_reports_unset_content_with_default_schema_artifacts() {
        // 檔案缺席＝未設定狀態；活躍 schema 為預設 spec-driven，固定鍵分節仍可呈現。
        let fx = FixtureRoot::new("settings-content-absent");
        let s = read_settings_at(fx.root()).expect("read ok");
        assert_eq!(s.workflow.context, None);
        assert!(s.workflow.rules.is_empty());
        assert_eq!(s.workflow.schema_artifacts, SPEC_DRIVEN_IDS);
    }

    #[test]
    fn read_settings_parse_error_empties_content_fields() {
        // 解析失敗浮出 parseError；內容欄位不得以猜測值呈現（表單將整份停用）。
        let fx = FixtureRoot::new("settings-content-bad");
        fx.write("openspec/config.yaml", "rules: [unclosed\n");
        let s = read_settings_at(fx.root()).expect("read ok");
        assert!(s.workflow.parse_error.is_some());
        assert_eq!(s.workflow.context, None);
        assert!(s.workflow.rules.is_empty());
        assert!(s.workflow.schema_artifacts.is_empty(), "no schema known for a broken file");
    }

    // --- context／rules 寫入（design D4 雙重驗證沿用） ---

    #[test]
    fn write_workflow_content_sets_context_and_rules_and_keeps_policy() {
        // spec Scenario「編輯專案說明並儲存」＋政策欄位不受內容寫入波及。
        let fx = FixtureRoot::new("wf-content-write");
        let doc = "schema: spec-driven\nlocale: tw\ntdd: true\nrules:\n  proposal:\n    - 舊規則\n";
        fx.write("openspec/config.yaml", doc);
        let rules = sections(&[("proposal", &["舊規則"]), ("tasks", &["@完成後執行全部測試"])]);
        write_workflow_content_at(
            fx.root(),
            &ContextEdit::Set("新的專案說明\n跨兩行\n".into()),
            Some(&rules),
        )
        .expect("write ok");
        let text = read(&fx.root().join("openspec/config.yaml"));
        let new: WorkflowConfig = serde_yaml::from_str(&text).expect("output parses");
        assert_eq!(new.context.as_deref(), Some("新的專案說明\n跨兩行\n"));
        assert_eq!(
            new.rules.get("tasks").map(Vec::as_slice),
            Some(["@完成後執行全部測試".to_string()].as_slice()),
            "reserved-char entry round-trips"
        );
        assert_eq!(new.locale.as_deref(), Some("tw"), "policy keys untouched");
        assert_eq!(new.tdd, Some(true));
        assert_eq!(new.schema.as_deref(), Some("spec-driven"));
    }

    #[test]
    fn write_workflow_content_clearing_removes_keys() {
        // spec Scenario「清空即移除鍵」。
        let fx = FixtureRoot::new("wf-content-clear");
        fx.write(
            "openspec/config.yaml",
            "schema: spec-driven\ncontext: 舊說明\nrules:\n  tasks:\n    - t1\n",
        );
        write_workflow_content_at(fx.root(), &ContextEdit::Set("   ".into()), Some(&sections(&[("tasks", &[])])))
            .expect("write ok");
        let text = read(&fx.root().join("openspec/config.yaml"));
        let m: serde_yaml::Mapping = serde_yaml::from_str(&text).expect("mapping");
        assert!(!m.contains_key("context"), "blank context removes the key: {text}");
        assert!(!m.contains_key("rules"), "all-empty rules removes the key: {text}");
        assert!(m.contains_key("schema"));
    }

    #[test]
    fn write_workflow_content_refuses_unparseable_original_and_leaves_file_intact() {
        let fx = FixtureRoot::new("wf-content-bad");
        let bad = "rules: [unclosed\n";
        fx.write("openspec/config.yaml", bad);
        let err = write_workflow_content_at(fx.root(), &ContextEdit::Set("x".into()), None)
            .expect_err("must refuse");
        assert!(err.contains("config.yaml"), "must name the file: {err}");
        assert!(!err.contains('\n'), "single line: {err:?}");
        assert_eq!(read(&fx.root().join("openspec/config.yaml")), bad, "file must be untouched");
    }

    #[test]
    fn write_workflow_content_surfaces_write_failure_with_file_and_stage() {
        let fx = FixtureRoot::new("wf-content-ro");
        let doc = "context: old\n";
        fx.write("openspec/config.yaml", doc);
        let path = fx.root().join("openspec/config.yaml");
        set_readonly(&path, true);
        let err = write_workflow_content_at(fx.root(), &ContextEdit::Set("new".into()), None)
            .expect_err("must fail");
        set_readonly(&path, false);
        assert!(err.contains("config.yaml"), "must name the file: {err}");
        assert!(err.contains("write"), "must name the stage: {err}");
        assert!(!err.contains('\n'), "single line: {err:?}");
        assert_eq!(read(&path), doc, "file must be untouched");
    }
}

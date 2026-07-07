//! 設定讀寫橋接：`.speclink.yaml`（tools）與 `openspec/config.yaml`（政策欄位）
//! 的載入快照與雙重驗證寫入（design D5）。純函式部分在 speclink-core（config.rs
//! 的 update_*_text）；檔案讀寫、寫前後驗證與技能同步在本層。

use std::path::{Path, PathBuf};

use serde::Serialize;
use speclink_core::config::{AppConfig, ToolEntry, WorkflowConfig};
use speclink_core::skills::Tool;
use speclink_core::workspace::Workspace;

// Tauri 殼經本 crate 取用政策欄位型別，不直接依賴 speclink-core。
pub use speclink_core::config::WorkflowPolicyFields;

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
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSettings {
    pub locale: Option<String>,
    pub spec_locale: Option<String>,
    pub tdd: bool,
    pub audit: bool,
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
            parse_error: None,
        },
        Some(text) => match serde_yaml::from_str::<WorkflowConfig>(&text) {
            Ok(cfg) => WorkflowSettings {
                locale: cfg.locale,
                spec_locale: cfg.spec_locale,
                tdd: cfg.tdd.unwrap_or(false),
                audit: cfg.audit.unwrap_or(false),
                parse_error: None,
            },
            Err(e) => WorkflowSettings {
                locale: None,
                spec_locale: None,
                tdd: false,
                audit: false,
                parse_error: Some(single_line(&e.to_string())),
            },
        },
    };
    Ok(SettingsSnapshot { app, workflow })
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
    let new_text = speclink_core::config::update_workflow_config_text(&original, fields)
        .map_err(|e| single_line(&e.to_string()))?;
    verify_workflow_text(&new_text, fields, &file, "pre-write verification")?;
    std::fs::write(&path, &new_text).map_err(|e| format!("{file}: write failed: {e}"))?;
    let reread = read_opt(&path)
        .ok_or_else(|| format!("{file}: verify after write failed: file unreadable"))?;
    verify_workflow_text(&reread, fields, &file, "verify after write")
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
}

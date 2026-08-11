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
    pub worktree: bool,
    pub context: Option<String>,
    pub rules: BTreeMap<String, Vec<String>>,
    pub schema_artifacts: Vec<String>,
    pub parse_error: Option<String>,
}

/// 讀取設定頁快照。三態各自獨立：檔案缺席＝預設值狀態（無 parse_error）；
/// 可解析＝實際欄位值；解析失敗＝parse_error 單行訊息。載入經引擎下沉後的
/// typed 函式（`AppConfig::load`／`WorkflowConfig::from_text` 已 fail-closed，
/// 本層不再自行嚴格解析繞道）。
pub fn read_settings_at(root: &Path) -> Result<SettingsSnapshot, String> {
    let (ws, app) = match Workspace::discover(root) {
        Ok(None) => return Err(format!("not a speclink project: {}", root.display())),
        Ok(Some(ws)) => {
            let app = match AppConfig::load(&ws.app_config()) {
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
                    parse_error: Some(single_line(&e.reason)),
                },
            };
            (ws, app)
        }
        // 壞 .speclink.yaml：discover fail-closed。設定頁沿用既有 UI——app 面
        // 浮出 parse_error（表單停用），workflow 面以預設佈局照常呈現。
        Err(e) => (
            Workspace { root: root.to_path_buf(), spec_dir_name: "openspec".to_string() },
            AppSettings {
                tools: Vec::new(),
                custom_tools: Vec::new(),
                parse_error: Some(single_line(&e.reason)),
            },
        ),
    };
    let workflow = workflow_settings_from_text(
        read_opt(&workflow_config_path(&ws)).as_deref(),
        Some(&ws),
    );
    Ok(SettingsSnapshot { app, workflow })
}

/// 從 server `/config` 或本機檔案原文建立 workflow 設定快照。此入口不讀檔、
/// 不依賴 checkout，讓 local/remote 共用完全相同的欄位預設與 parse-error 語意。
pub fn read_workflow_settings_from_text(text: Option<&str>) -> WorkflowSettings {
    workflow_settings_from_text(text, None)
}

fn workflow_settings_from_text(
    text: Option<&str>,
    workspace: Option<&Workspace>,
) -> WorkflowSettings {
    match WorkflowConfig::from_text(text) {
        // 缺席與可解析同一 arm：缺檔的 typed 載入即預設值狀態。
        Ok(cfg) => WorkflowSettings {
            locale: cfg.locale.clone(),
            spec_locale: cfg.spec_locale.clone(),
            tdd: cfg.tdd.unwrap_or(false),
            audit: cfg.audit.unwrap_or(false),
            worktree: cfg.worktree.unwrap_or(false),
            context: cfg.context.clone(),
            rules: cfg.rules.clone(),
            schema_artifacts: schema_artifact_ids(workspace, &cfg),
            parse_error: None,
        },
        Err(e) => WorkflowSettings {
            locale: None,
            spec_locale: None,
            tdd: false,
            audit: false,
            worktree: false,
            context: None,
            rules: Default::default(),
            // 壞檔無從得知活躍 schema——不呈現猜測的分節（表單一併停用）。
            schema_artifacts: Vec::new(),
            parse_error: Some(single_line(&e.reason)),
        },
    }
}

/// 活躍 schema 的 artifact id（引擎顯示序＝`status::display_order` 的拓撲序，
/// 與 status 輸出一致）。schema 解析失敗或找不到時給空——與壞檔同語意：不在
/// 未知 schema 上呈現猜測的固定鍵。
fn schema_artifact_ids(ws: Option<&Workspace>, cfg: &WorkflowConfig) -> Vec<String> {
    match speclink_core::schema::resolve_with(ws, Some(&speclink_host::context::global_config_dir()), &cfg.schema_name()) {
        Some(Ok(schema)) => speclink_core::status::display_order(&schema)
            .into_iter()
            .map(|a| a.id.clone())
            .collect(),
        _ => Vec::new(),
    }
}

/// 純文字政策欄位改寫 seam。代換 locale/spec_locale/tdd/audit/worktree，其他鍵的
/// parsed value 保持不變；設回預設值時移除鍵。輸出在回傳前再經 typed 驗證。
pub fn rewrite_workflow_fields_text(
    original: &str,
    fields: &WorkflowPolicyFields,
) -> Result<String, String> {
    rewrite_workflow_fields_text_for(original, fields, "config.yaml")
}

fn rewrite_workflow_fields_text_for(
    original: &str,
    fields: &WorkflowPolicyFields,
    file: &str,
) -> Result<String, String> {
    let new_text = speclink_core::config::update_workflow_config_text(
        original,
        fields,
        &ContextEdit::Keep,
        None,
    )
    .map_err(|e| single_line(&e.to_string()))?;
    verify_workflow_text(&new_text, fields, file, "rewrite verification")?;
    Ok(new_text)
}

/// 寫入 `openspec/config.yaml` 的政策欄位（design D5 雙重驗證）：core 純函式
/// 改寫（寫前解析原文失敗即中止）→ 驗證新文字可解析且目標欄位值正確 →
/// 寫檔 → 回讀再驗。任一步失敗回指明檔案與階段的單行 Err，磁碟檔案維持原內容
/// ——絕不留下不可解析的設定檔。
///
/// `fields` 是設定頁可編輯欄位的完整目標狀態（非 patch）：呼叫端必須先以
/// `read_settings_at` 取得現值再改寫，否則留在預設的欄位會被清掉。
///
/// worktree 牽動技能足跡，故與 CLI 的 `workflow-config set` 同序（design D2）：
/// 開→關先查活躍 worktree（有則整體不動），寫入成功後同步足跡。
pub fn write_workflow_fields_at(
    root: &Path,
    fields: &speclink_core::config::WorkflowPolicyFields,
) -> Result<(), String> {
    let ws = discover(root)?;
    let file = format!("{}/config.yaml", ws.spec_dir_name);
    let path = workflow_config_path(&ws);
    let original = read_opt(&path).unwrap_or_default();
    let new_text = rewrite_workflow_fields_text_for(&original, fields, &file)?;
    let worktree_was_on = WorkflowConfig::from_text(Some(&original))
        .ok()
        .and_then(|c| c.worktree)
        .unwrap_or(false);
    if worktree_was_on && !fields.worktree {
        refuse_teardown_with_active_worktrees(&ws)?;
    }
    speclink_core::util::write_file(&path, &new_text)
        .map_err(|e| format!("{file}: write failed: {e}"))?;
    let reread = read_opt(&path)
        .ok_or_else(|| format!("{file}: verify after write failed: file unreadable"))?;
    verify_workflow_text(&reread, fields, &file, "verify after write")?;
    if worktree_was_on != fields.worktree {
        speclink_core::init::update(&ws.root, false).map_err(|e| {
            format!(
                "{file} written, but the skill footprint did not sync: {} — fix the cause above, then re-run `speclink update` to rebuild it",
                single_line(&e.to_string())
            )
        })?;
    }
    Ok(())
}

/// 關閉 worktree 政策會連 merge 技能一起收走，掛著的 worktree 就沒有收尾工具了。
/// 與 CLI 走同一個 host 判定，訊息列出每個 worktree 的 change 名、分支與路徑。
/// git 不可用時判定回空清單（fail-open），不擋。
fn refuse_teardown_with_active_worktrees(ws: &Workspace) -> Result<(), String> {
    let store = speclink_fs::FsStore::new(&ws.root, &ws.spec_dir_name);
    let blockers = speclink_host::worktree::teardown_blockers(ws, &store);
    if blockers.is_empty() {
        return Ok(());
    }
    let list: Vec<String> = blockers
        .iter()
        .map(|b| format!("{} ({}) at {}", b.change, b.branch, b.path.display()))
        .collect();
    Err(format!(
        "worktree 仍在使用中，關閉後將移除收尾用的 merge 技能：{}。請先對每個 worktree 執行 speclink-worktree-merge 收尾，再關閉此開關。",
        list.join("；")
    ))
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
    let new_text = rewrite_workflow_content_text_for(&original, context, rules, &file)?;
    let expected = WorkflowConfig::from_text(Some(&new_text)).map_err(|e| {
        format!("{file}: pre-write verification failed: {}", single_line(&e.reason))
    })?;
    let fields = workflow_policy_fields(&expected);
    speclink_core::util::write_file(&path, &new_text)
        .map_err(|e| format!("{file}: write failed: {e}"))?;
    let reread = read_opt(&path)
        .ok_or_else(|| format!("{file}: verify after write failed: file unreadable"))?;
    verify_workflow_content_text(
        &reread,
        &expected.context,
        &expected.rules,
        &fields,
        &file,
        "verify after write",
    )
}

/// 純文字 context/rules targeted-key 改寫 seam。政策欄位從原文讀出再原樣
/// 回填，讓 remote 與 local 共用同一份未觸及鍵與清空移除語意。
pub fn rewrite_workflow_content_text(
    original: &str,
    context: &ContextEdit,
    rules: Option<&[(String, Vec<String>)]>,
) -> Result<String, String> {
    rewrite_workflow_content_text_for(original, context, rules, "config.yaml")
}

fn rewrite_workflow_content_text_for(
    original: &str,
    context: &ContextEdit,
    rules: Option<&[(String, Vec<String>)]>,
    file: &str,
) -> Result<String, String> {
    let current = WorkflowConfig::from_text(Some(original)).map_err(|e| {
        format!("{file}: rewrite verification failed: {}", single_line(&e.reason))
    })?;
    let fields = workflow_policy_fields(&current);
    let want_context = match context {
        ContextEdit::Keep => current.context.clone(),
        ContextEdit::Set(value) if !value.trim().is_empty() => Some(value.clone()),
        ContextEdit::Set(_) | ContextEdit::Remove => None,
    };
    let want_rules: BTreeMap<String, Vec<String>> = match rules {
        None => current.rules.clone(),
        Some(sections) => sections
            .iter()
            .filter_map(|(key, entries)| {
                let cleaned: Vec<String> = entries
                    .iter()
                    .map(|entry| entry.trim().to_string())
                    .filter(|entry| !entry.is_empty())
                    .collect();
                (!cleaned.is_empty()).then(|| (key.clone(), cleaned))
            })
            .collect(),
    };
    let new_text =
        speclink_core::config::update_workflow_config_text(original, &fields, context, rules)
            .map_err(|e| single_line(&e.to_string()))?;
    verify_workflow_content_text(
        &new_text,
        &want_context,
        &want_rules,
        &fields,
        file,
        "rewrite verification",
    )?;
    Ok(new_text)
}

fn workflow_policy_fields(config: &WorkflowConfig) -> WorkflowPolicyFields {
    WorkflowPolicyFields {
        locale: config.locale.clone(),
        spec_locale: config.spec_locale.clone(),
        tdd: config.tdd.unwrap_or(false),
        audit: config.audit.unwrap_or(false),
        // 讀回現值而非留 default：目標狀態是「完整」的，漏帶會讓設定頁的任一次存檔
        // 靜默刪掉使用者的 worktree 鍵。桌面尚無此欄位的 UI（屬第二刀），這裡只保值。
        worktree: config.worktree.unwrap_or(false),
    }
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
        && cfg.worktree.unwrap_or(false) == fields.worktree
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
        && cfg.worktree.unwrap_or(false) == fields.worktree
    {
        Ok(())
    } else {
        Err(format!("{file}: {stage} failed: rewritten values do not match the request"))
    }
}

/// 寫入 `.speclink.yaml` 的內建工具選集並同步受管產物。編排全在
/// `speclink_core::init::reconcile_builtin_tools`——設定頁、CLI init 與 checkout 綁定
/// 共用同一入口，因此三者對相同選集產生相同結果。未知工具名、空選集與無法解析的
/// 原檔都在任何寫入之前被拒。
pub fn write_tools_at(root: &Path, tools: &[String]) -> Result<(), String> {
    let ws = discover(root)?;
    let selected =
        speclink_core::init::parse_tool_names(tools).map_err(|e| single_line(&e.to_string()))?;
    speclink_core::init::reconcile_builtin_tools(&ws.root, &selected)
        .map_err(|e| single_line(&e.to_string()))?;
    Ok(())
}

fn discover(root: &Path) -> Result<Workspace, String> {
    Workspace::discover(root)
        .map_err(|e| single_line(&e.to_string()))?
        .ok_or_else(|| format!("not a speclink project: {}", root.display()))
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

    #[test]
    fn read_workflow_settings_from_text_parses_every_remote_editable_field() {
        let settings = read_workflow_settings_from_text(Some(
            "schema: spec-driven\nlocale: tw\nspec_locale: auto\ntdd: true\naudit: true\ncontext: 專案說明\nrules:\n  tasks:\n    - 完成後驗證\n",
        ));
        assert_eq!(settings.locale.as_deref(), Some("tw"));
        assert_eq!(settings.spec_locale.as_deref(), Some("auto"));
        assert!(settings.tdd);
        assert!(settings.audit);
        assert_eq!(settings.context.as_deref(), Some("專案說明"));
        assert_eq!(settings.rules["tasks"], ["完成後驗證"]);
        assert_eq!(settings.schema_artifacts, SPEC_DRIVEN_IDS);
        assert_eq!(settings.parse_error, None);
    }

    // spec Example「政策欄位寫入效果」row 1。
    #[test]
    fn text_rewrite_adds_tdd_true_when_the_key_is_absent() {
        let fields = WorkflowPolicyFields { tdd: true, ..Default::default() };
        let output = rewrite_workflow_fields_text("schema: spec-driven\n", &fields)
            .expect("rewrite");
        let parsed = WorkflowConfig::from_text(Some(&output)).expect("parse output");
        assert_eq!(parsed.tdd, Some(true));
    }

    // spec Example「政策欄位寫入效果」row 2。
    #[test]
    fn text_rewrite_removes_tdd_when_reset_to_the_default() {
        let output = rewrite_workflow_fields_text(
            "schema: spec-driven\ntdd: true\n",
            &WorkflowPolicyFields::default(),
        )
        .expect("rewrite");
        let mapping: serde_yaml::Mapping = serde_yaml::from_str(&output).expect("mapping");
        assert!(!mapping.contains_key("tdd"), "default false removes the key: {output}");
    }

    // spec Example「政策欄位寫入效果」row 3。
    #[test]
    fn text_rewrite_adds_spec_locale_and_preserves_locale_and_rules_values() {
        let original = "locale: tw\nrules:\n  proposal:\n    - keep\n";
        let fields = WorkflowPolicyFields {
            locale: Some("tw".into()),
            spec_locale: Some("auto".into()),
            ..Default::default()
        };
        let output = rewrite_workflow_fields_text(original, &fields).expect("rewrite");
        let before = WorkflowConfig::from_text(Some(original)).expect("parse before");
        let after = WorkflowConfig::from_text(Some(&output)).expect("parse after");
        assert_eq!(after.spec_locale.as_deref(), Some("auto"));
        assert_eq!(after.locale, before.locale);
        assert_eq!(after.rules, before.rules);
    }

    #[test]
    fn text_rewrite_writes_the_worktree_value_the_page_sent() {
        // 設定頁有了 worktree 開關，送來的值就是目標狀態（不再從原檔回填）：
        // 開→關要真的把鍵拿掉，否則畫面上關了、檔案裡還開著。
        let off = rewrite_workflow_fields_text(
            "schema: spec-driven\nworktree: true\n",
            &WorkflowPolicyFields { tdd: true, ..Default::default() },
        )
        .expect("rewrite");
        let parsed = WorkflowConfig::from_text(Some(&off)).expect("parse output");
        assert_eq!(parsed.worktree, None, "關閉時鍵應被移除（false＝預設）：{off}");
        assert_eq!(parsed.tdd, Some(true));

        let on = rewrite_workflow_fields_text(
            "schema: spec-driven\n",
            &WorkflowPolicyFields { worktree: true, ..Default::default() },
        )
        .expect("rewrite");
        let parsed = WorkflowConfig::from_text(Some(&on)).expect("parse output");
        assert_eq!(parsed.worktree, Some(true), "開啟時鍵應寫入：{on}");
    }

    #[test]
    fn content_text_rewrite_preserves_policy_and_the_untouched_rules_value() {
        let original = "schema: spec-driven\nlocale: tw\ntdd: true\ncontext: old\nrules:\n  tasks:\n    - keep me\n";
        let output = rewrite_workflow_content_text(
            original,
            &ContextEdit::Set("new context".into()),
            None,
        )
        .expect("rewrite");
        let before = WorkflowConfig::from_text(Some(original)).expect("parse before");
        let after = WorkflowConfig::from_text(Some(&output)).expect("parse after");
        assert_eq!(after.context.as_deref(), Some("new context"));
        assert_eq!(after.rules, before.rules);
        assert_eq!(after.locale, before.locale);
        assert_eq!(after.tdd, before.tdd);
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

    /// 讓寫檔階段必定失敗。unix 上原子寫的暫存檔看目錄權限、退回的直寫看檔案
    /// 權限——兩者都鎖才逼得出失敗；Windows 的 rename 覆蓋唯讀檔與直寫唯讀檔
    /// 都會被拒，鎖檔案即可。
    fn block_writes(path: &Path, blocked: bool) {
        #[cfg(unix)]
        set_readonly(path.parent().unwrap(), blocked);
        set_readonly(path, blocked);
    }

    #[test]
    fn write_workflow_fields_replaces_targets_and_keeps_untouched_keys() {
        // spec Scenario「寫入政策欄位且未觸及鍵原樣保留」。
        let fx = FixtureRoot::new("wf-write");
        let doc = "schema: spec-driven\ncontext: |\n  keep me\nrules:\n  proposal:\n    - keep rule\n";
        fx.write("openspec/config.yaml", doc);
        let fields = WorkflowPolicyFields { tdd: true, ..Default::default() };
        write_workflow_fields_at(fx.root(), &fields).expect("write ok");
        // 手術改寫：tdd 插於 schema 之下、前後恰一空行，其餘行逐位元保留（不再整檔重排）。
        assert_eq!(
            read(&fx.root().join("openspec/config.yaml")),
            "schema: spec-driven\n\ntdd: true\n\ncontext: |\n  keep me\nrules:\n  proposal:\n    - keep rule\n"
        );
    }

    #[test]
    fn write_workflow_fields_leaves_no_temp_residue() {
        // spec Scenario「設定寫入走同一原子入口」的觀察面：寫入成功後目錄無暫存
        // 殘留、內容為完整全文（收編一旦回歸普通 fs::write 前者仍綠，但殘留斷言
        // 釘住原子入口的成功路徑契約）。
        let fx = FixtureRoot::new("wf-write-atomic-face");
        fx.write("openspec/config.yaml", "schema: spec-driven\n");
        let fields = WorkflowPolicyFields { tdd: true, ..Default::default() };
        write_workflow_fields_at(fx.root(), &fields).expect("write ok");
        let residue: Vec<String> = std::fs::read_dir(fx.root().join("openspec"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".tmp"))
            .collect();
        assert!(residue.is_empty(), "temp residue left behind: {residue:?}");
        let text = read(&fx.root().join("openspec/config.yaml"));
        assert!(
            text.contains("schema: spec-driven") && text.contains("tdd: true"),
            "content is the complete new document: {text}"
        );
    }

    #[test]
    fn write_refuses_to_turn_worktree_off_while_one_is_active() {
        // spec Scenario「關閉遇活躍 worktree 浮出擋下」：訊息列出 change 名、分支
        // 與路徑並指出收尾方式，config 檔逐位元不變。
        // fixture 的 attach_worktree 會寫入 worktree: true 的 config 並回報 git
        // 拼法的路徑——訊息裡的路徑同樣來自 git，期望與其同源才對得上。
        let fx = FixtureRoot::new("wf-write-wt-block");
        fx.add_change("add-auth", "");
        let wt = fx.attach_worktree("add-auth");
        let before = read(&fx.root().join("openspec/config.yaml"));

        let err = write_workflow_fields_at(fx.root(), &WorkflowPolicyFields::default())
            .expect_err("有活躍 worktree 時必須拒絕");

        assert!(err.contains("add-auth"), "須列 change 名: {err}");
        assert!(err.contains("speclink/add-auth"), "須列分支: {err}");
        assert!(err.contains(wt.path().to_str().unwrap()), "須列路徑: {err}");
        assert!(err.contains("worktree-merge"), "須指出收尾方式: {err}");
        assert_eq!(read(&fx.root().join("openspec/config.yaml")), before, "檔案不得變動");
    }

    #[test]
    fn write_turns_worktree_off_when_none_is_active() {
        // 無活躍 worktree（此 fixture 連 git repo 都不是）＝ fail-open，照常寫入。
        let fx = FixtureRoot::new("wf-write-wt-free");
        fx.write("openspec/config.yaml", "schema: spec-driven\nworktree: true\n");
        write_workflow_fields_at(fx.root(), &WorkflowPolicyFields::default()).expect("write ok");
        let text = read(&fx.root().join("openspec/config.yaml"));
        let parsed = WorkflowConfig::from_text(Some(&text)).expect("parse");
        assert_eq!(parsed.worktree, None, "關閉後鍵應被移除: {text}");
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
        // 擋下寫入觸發寫檔階段失敗：Err 指明檔案與階段、內容逐字元不變。
        let fx = FixtureRoot::new("wf-write-ro");
        let doc = "locale: tw\n";
        fx.write("openspec/config.yaml", doc);
        let path = fx.root().join("openspec/config.yaml");
        block_writes(&path, true);
        let err = write_workflow_fields_at(fx.root(), &WorkflowPolicyFields::default())
            .expect_err("must fail");
        block_writes(&path, false);
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
        block_writes(&path, true);
        let err = write_workflow_content_at(fx.root(), &ContextEdit::Set("new".into()), None)
            .expect_err("must fail");
        block_writes(&path, false);
        assert!(err.contains("config.yaml"), "must name the file: {err}");
        assert!(err.contains("write"), "must name the stage: {err}");
        assert!(!err.contains('\n'), "single line: {err:?}");
        assert_eq!(read(&path), doc, "file must be untouched");
    }
}

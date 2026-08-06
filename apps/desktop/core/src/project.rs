//! 開啟專案：所選目錄的四態判定（本機專案／remote binding／未初始化／錯誤）與未初始化目錄的
//! 初始化。僅回報判定結果或執行 init——root 的切換由 Tauri 層的狀態持有者決定。

use std::path::Path;

use serde::Serialize;
use speclink_core::workspace::Workspace;

/// `open_project_at` 的判定結果——僅回報，不切換任何狀態。
/// 序列化為 `{ "status": "project" | "remoteBinding" | "uninitialized", … }` 供前端分流。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "status")]
pub enum ProjectProbe {
    /// 向上探索命中 speclink 專案：`root` 為探索到的專案根、`name` 為其目錄名。
    Project { root: String, name: String },
    /// 命中 `.speclink.yaml` 的 remote section；Desktop 必須走 remote handshake，
    /// 並以呼叫端原始 path 作為 checkoutRoot，不得把 marker 當成本機專案。
    RemoteBinding {
        url: String,
        repo: Option<String>,
        has_local_openspec: bool,
    },
    /// 目錄存在且可讀，但不屬於任何 speclink 專案（前端據此開初始化確認框）。
    Uninitialized { dir: String },
    /// 向上探索命中且為本地檔案模式、但專案根無 `.speclink.yaml`——有規格資料
    /// 但未啟用 speclink（前端據此開啟用確認框）。`root` 為命中的專案根。
    Unadopted { root: String },
}

/// 對所選目錄做開啟專案的四態判定：以該目錄為起點沿用
/// `Workspace::discover` 向上探索（與 app 啟動語意一致）。零寫入。
/// 不存在或非目錄的路徑是單行 Err——必須在探索前攔下，否則已刪除路徑
/// 會經祖先誤命中別的專案（分頁失效態依賴這個判定）。
pub fn open_project_at(path: &Path) -> Result<ProjectProbe, String> {
    if !path.is_dir() {
        return Err(format!(
            "cannot open '{}': not an existing directory",
            path.display()
        ));
    }
    match Workspace::discover(path) {
        Ok(Some(ws)) => {
            let resolution = ws.resolve_mode_with(None).map_err(|e| e.to_string())?;
            match resolution.mode {
                speclink_core::workspace::StoreMode::Remote(remote) => {
                    Ok(ProjectProbe::RemoteBinding {
                        url: remote.url,
                        repo: remote.repo,
                        has_local_openspec: resolution.coexists,
                    })
                }
                // 第四態（desktop-enable-speclink-prompt 決策 1）：Fs 模式且 root 無
                // .speclink.yaml → 未啟用。舊版 remote 標記檔的遷移警告路徑凍結——
                // 帶 leftover 標記者照舊判 Project，不得誤判成未啟用。
                speclink_core::workspace::StoreMode::Fs
                    if !ws.root.join(".speclink.yaml").is_file()
                        && !resolution.leftover_remote_file =>
                {
                    Ok(ProjectProbe::Unadopted {
                        root: ws.root.display().to_string(),
                    })
                }
                speclink_core::workspace::StoreMode::Fs => Ok(ProjectProbe::Project {
                    name: project_name(&ws.root),
                    root: ws.root.display().to_string(),
                }),
            }
        }
        Ok(None) => Ok(ProjectProbe::Uninitialized {
            dir: path.display().to_string(),
        }),
        // 壞 .speclink.yaml：fail-closed 為單行 Err——不得誤判為 Uninitialized
        // （否則前端會對既有專案開初始化確認框）。
        Err(e) => Err(e.to_string()),
    }
}

/// 專案顯示名＝root 目錄名（分頁與頂欄用）；磁碟根等無目錄名時退回完整路徑。
pub fn project_name(root: &Path) -> String {
    root.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.display().to_string())
}

/// 唯讀回報指定專案 root 的待收尾數（spec-archive-drawer design D6 背景分頁徽章快照）。
/// 不切換任何狀態。待收尾＝等使用者執行動詞的卡片：已就緒（任務全完成，與看板
/// ready 欄派生一致）變更數＋concluded 未轉出討論數（promoted 已轉出、open 仍在
/// 推進，皆不計）。路徑失效回單行 Err；且探索命中的 root 必須就是 `path` 本身——
/// openspec/ 已刪但目錄還在時，向上探索會誤命中祖先專案，不得回報別人的數字。
pub fn project_stats_at(path: &Path) -> Result<serde_json::Value, String> {
    match open_project_at(path)? {
        ProjectProbe::Project { root, .. } if root == path.display().to_string() => {}
        _ => {
            return Err(format!(
                "'{}' is no longer a speclink project root",
                path.display()
            ))
        }
    }
    let payload = crate::query::list_changes_at(path);
    let ready = payload["changes"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter(|c| {
                    let total = c["totalTasks"].as_u64().unwrap_or(0);
                    let done = c["completedTasks"].as_u64().unwrap_or(0);
                    total > 0 && done >= total
                })
                .count()
        })
        .unwrap_or(0);
    let discussions = crate::discussions::list_discussions_at(path);
    let concluded = discussions["active"]
        .as_array()
        .map(|items| items.iter().filter(|d| d["status"] == "concluded").count())
        .unwrap_or(0);
    Ok(serde_json::json!({ "pendingWrapUp": ready + concluded }))
}

/// 對未初始化目錄執行與 `speclink init` 等效的初始化（design D3：消費
/// `speclink_core::init::init`，force=false、spec_dir 固定 openspec），成功後
/// 重跑三態判定回報命中的專案。`tools` 為內建工具名（claude／codex）；
/// 未知名在任何寫入之前被拒，單行 Err。
pub fn init_project_at(path: &Path, tools: &[String]) -> Result<ProjectProbe, String> {
    let selected = speclink_core::init::parse_tool_names(tools).map_err(|e| e.to_string())?;
    speclink_core::init::init(path, &selected, false, "openspec").map_err(|e| e.to_string())?;
    open_project_at(path)
}

/// 對未啟用目錄執行工作區補齊（消費 `speclink_core::init::adopt`，決策 2），
/// 成功後重跑探測回報命中的專案。`path` 為探測回報的專案根；未知工具名在
/// 任何寫入之前被拒，單行 Err。
pub fn adopt_project_at(path: &Path, tools: &[String]) -> Result<ProjectProbe, String> {
    let selected = speclink_core::init::parse_tool_names(tools).map_err(|e| e.to_string())?;
    speclink_core::init::adopt(path, &selected).map_err(|e| e.to_string())?;
    open_project_at(path)
}

/// 唯讀的指令檔過期探測（desktop-instruction-staleness-prompt 決策 4）：單行委派
/// `speclink_core::init::probe_instructions`，回報序列化為 camelCase JSON 供前端
/// 裁決是否顯示提示。零寫入、無 Err——探測失敗以 status=unknown 表達，開專案
/// 不得被探測擋下。
pub fn probe_instructions_at(path: &Path) -> serde_json::Value {
    let probe = speclink_core::init::probe_instructions(path);
    serde_json::to_value(probe).expect("probe payload serializes")
}

/// 指令檔整套再生（決策 5）：委派引擎既有的 `update()`——與 CLI 同一入口、冪等，
/// 依 `.speclink.yaml` 記錄的 store mode 維持 marker 措辭。回報沿用 UpdateOutcome
/// 形狀；失敗為單行 Err，由前端於提示原位呈現並可重試。
pub fn update_instructions_at(path: &Path) -> Result<serde_json::Value, String> {
    let outcome = speclink_core::init::update(path).map_err(|e| single_line(&e.to_string()))?;
    Ok(serde_json::json!({
        "updated": outcome.updated,
        "pruned": outcome.pruned,
        "notes": outcome.notes,
    }))
}

/// 引擎錯誤壓成單行（多行時取首行），符合 Err 單行訊息契約。
fn single_line(msg: &str) -> String {
    msg.lines().next().unwrap_or(msg).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testfixture::FixtureRoot;
    use std::path::{Path, PathBuf};

    // --- 指令檔探測與更新包裝（desktop-instruction-staleness-prompt 決策 4、5） ---

    /// 以引擎 init 生成一個現版工作區（claude 單工具）。
    fn init_workspace(fx: &FixtureRoot, tools: &[&str]) {
        let owned: Vec<String> = tools.iter().map(|t| (*t).to_string()).collect();
        let selected = speclink_core::init::parse_tool_names(&owned).unwrap();
        speclink_core::init::init(fx.root(), &selected, true, "openspec").unwrap();
    }

    #[test]
    fn probe_instructions_serializes_camel_case_fields() {
        // 前端消費的欄位名（決策 3 回報形狀）：currentVersion／stale／missing／
        // differingFiles／workspaceVersion 皆為 camelCase。
        let fx = FixtureRoot::new("probe-camel");
        init_workspace(&fx, &["claude"]);
        let marker = std::fs::read_to_string(fx.root().join("CLAUDE.md")).unwrap();
        std::fs::write(
            fx.root().join("CLAUDE.md"),
            marker.replace(speclink_core::init::MARKER_VERSION, "v0.9.0"),
        )
        .unwrap();

        let value = probe_instructions_at(fx.root());
        assert_eq!(value["status"], "stale", "{value}");
        assert_eq!(value["currentVersion"], speclink_core::init::MARKER_VERSION);
        let tool = &value["tools"][0];
        assert_eq!(tool["tool"], "claude");
        assert_eq!(tool["workspaceVersion"], "v0.9.0");
        assert_eq!(tool["stale"], true);
        assert_eq!(tool["newer"], false);
        assert_eq!(tool["missing"], false);
        assert!(
            value["differingFiles"]
                .as_array()
                .expect("differingFiles 為陣列")
                .iter()
                .any(|f| f == "CLAUDE.md"),
            "{value}"
        );
    }

    #[test]
    fn probe_instructions_reports_newer_when_the_workspace_leads_the_engine() {
        // 較新態（app 本體是舊版）：status 值與 per-tool 的 newer 布林都是前端契約
        // ——前端不重算方向，橫幅據此拿掉所有改寫動作。
        let fx = FixtureRoot::new("probe-camel-newer");
        init_workspace(&fx, &["claude"]);
        let major: u64 = speclink_core::init::MARKER_VERSION
            .trim_start_matches('v')
            .split('.')
            .next()
            .and_then(|s| s.parse().ok())
            .expect("MARKER_VERSION 主版號可解析");
        let ahead = format!("v{}.0.0", major + 1);
        let marker = std::fs::read_to_string(fx.root().join("CLAUDE.md")).unwrap();
        std::fs::write(
            fx.root().join("CLAUDE.md"),
            marker.replace(speclink_core::init::MARKER_VERSION, &ahead),
        )
        .unwrap();

        let value = probe_instructions_at(fx.root());
        assert_eq!(value["status"], "newer", "{value}");
        let tool = &value["tools"][0];
        assert_eq!(tool["newer"], true);
        assert_eq!(tool["stale"], false);
        assert_eq!(tool["workspaceVersion"], ahead);
    }

    #[test]
    fn probe_instructions_reports_missing_for_an_uninstalled_tool() {
        // 缺失態（從未安裝）也走同一形狀，前端據此改用安裝文案。
        let fx = FixtureRoot::new("probe-camel-missing");
        init_workspace(&fx, &["claude", "codex"]);
        std::fs::remove_file(fx.root().join("AGENTS.md")).unwrap();

        let value = probe_instructions_at(fx.root());
        assert_eq!(value["status"], "missing", "{value}");
        let codex = value["tools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["tool"] == "codex")
            .expect("codex 在列");
        assert_eq!(codex["missing"], true);
        assert_eq!(codex["workspaceVersion"], serde_json::Value::Null);
    }

    #[test]
    fn update_instructions_regenerates_and_reports_updated_tools() {
        // 決策 5：更新委派既有 update() 整套再生；回報沿用 UpdateOutcome 形狀。
        let fx = FixtureRoot::new("update-instructions");
        init_workspace(&fx, &["claude"]);
        std::fs::remove_file(fx.root().join("CLAUDE.md")).unwrap();

        let outcome = update_instructions_at(fx.root()).expect("update ok");
        assert_eq!(outcome["updated"][0], "claude", "{outcome}");
        assert!(outcome["pruned"].as_array().unwrap().is_empty(), "{outcome}");
        // 再生後探測回到現版。
        assert_eq!(probe_instructions_at(fx.root())["status"], "current");
    }

    #[test]
    fn update_instructions_on_a_broken_config_is_a_single_line_error() {
        let fx = FixtureRoot::new("update-instructions-bad");
        init_workspace(&fx, &["claude"]);
        std::fs::write(fx.root().join(".speclink.yaml"), "tools: [\n").unwrap();

        let err = update_instructions_at(fx.root()).expect_err("must fail");
        assert!(!err.contains('\n'), "single line: {err:?}");
    }

    /// 不含任何 speclink 標記的空目錄（自動清除）。系統 temp 目錄本身不是
    /// speclink 專案，向上探索不會誤命中。
    struct PlainDir(PathBuf);

    impl PlainDir {
        fn new(tag: &str) -> PlainDir {
            let dir = std::env::temp_dir().join(format!("speclink-dtplain-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            PlainDir(dir)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for PlainDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn entries(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    // --- open_project_at：三態判定（spec 需求「執行期切換專案 root」） ---

    /// 已啟用專案標記（spec Scenario「已啟用專案不出現啟用對話框」的判準）：
    /// 專案根有 `.speclink.yaml` 才照舊判 Project。
    fn mark_adopted(fx: &FixtureRoot) {
        fx.write(".speclink.yaml", "tools:\n  - claude\n");
    }

    #[test]
    fn open_project_at_project_root_reports_root_and_name() {
        let fx = FixtureRoot::new("open-root");
        mark_adopted(&fx);
        let probe = open_project_at(fx.root()).expect("probe ok");
        match probe {
            ProjectProbe::Project { root, name } => {
                assert_eq!(root, fx.root().display().to_string());
                assert_eq!(name, fx.root().file_name().unwrap().to_string_lossy());
            }
            other => panic!("expected Project, got {other:?}"),
        }
    }

    #[test]
    fn open_project_at_subdir_walks_up_to_project_root() {
        // spec Scenario「自子目錄向上探索至專案根」：子目錄本身不含 openspec/。
        let fx = FixtureRoot::new("open-subdir");
        mark_adopted(&fx);
        let sub = fx.root().join("src").join("nested");
        std::fs::create_dir_all(&sub).unwrap();
        let probe = open_project_at(&sub).expect("probe ok");
        match probe {
            ProjectProbe::Project { root, .. } => {
                assert_eq!(root, fx.root().display().to_string());
            }
            other => panic!("expected Project, got {other:?}"),
        }
    }

    // --- 第四態：未啟用（spec 需求「未啟用資料夾經確認後補齊啟用」；決策 1） ---

    #[test]
    fn open_project_at_openspec_without_app_config_reports_unadopted_with_zero_writes() {
        // 判準＝專案根 .speclink.yaml 存在與否：有 openspec/ 但無 .speclink.yaml
        // 的資料夾（自其他體系遷移、或隊友未提交）不再靜默判 Project。
        let fx = FixtureRoot::new("open-unadopted");
        let before = entries(fx.root());
        let probe = open_project_at(fx.root()).expect("probe ok");
        assert_eq!(
            serde_json::to_value(probe).unwrap(),
            serde_json::json!({
                "status": "unadopted",
                "root": fx.root().display().to_string(),
            })
        );
        assert_eq!(entries(fx.root()), before, "probe must not write anything");
    }

    #[test]
    fn open_project_at_unadopted_subdir_anchors_the_walked_up_root() {
        // spec Scenario「子目錄開啟錨定專案根」：判定錨在 discover 命中的 root，
        // 非使用者所選子目錄。
        let fx = FixtureRoot::new("open-unadopted-subdir");
        let sub = fx.root().join("src").join("nested");
        std::fs::create_dir_all(&sub).unwrap();
        let probe = open_project_at(&sub).expect("probe ok");
        match probe {
            ProjectProbe::Unadopted { root } => {
                assert_eq!(root, fx.root().display().to_string());
            }
            other => panic!("expected Unadopted, got {other:?}"),
        }
    }

    #[test]
    fn open_project_at_leftover_remote_file_keeps_reporting_project() {
        // design 風險「誤把舊 remote 標記檔資料夾判成未啟用」：舊版
        // .speclink.remote.yaml 的遷移警告路徑凍結——不得變成 unadopted。
        let fx = FixtureRoot::new("open-leftover-remote");
        fx.write(".speclink.remote.yaml", "url: https://legacy.example.test\n");
        let probe = open_project_at(fx.root()).expect("probe ok");
        match probe {
            ProjectProbe::Project { root, .. } => {
                assert_eq!(root, fx.root().display().to_string());
            }
            other => panic!("expected Project, got {other:?}"),
        }
    }

    #[test]
    fn open_project_at_non_project_reports_uninitialized_with_zero_writes() {
        let plain = PlainDir::new("open-uninit");
        let before = entries(plain.path());
        let probe = open_project_at(plain.path()).expect("probe ok");
        match probe {
            ProjectProbe::Uninitialized { dir } => {
                assert_eq!(dir, plain.path().display().to_string());
            }
            other => panic!("expected Uninitialized, got {other:?}"),
        }
        assert_eq!(entries(plain.path()), before, "probe must not write anything");
    }

    #[test]
    fn open_project_at_remote_marker_reports_binding_without_local_openspec() {
        let plain = PlainDir::new("open-remote");
        std::fs::write(
            plain.path().join(".speclink.yaml"),
            "remote:\n  url: https://spec.example.test/team/\n  repo: desktop\n",
        )
        .unwrap();

        let probe = open_project_at(plain.path()).expect("probe ok");
        assert_eq!(
            serde_json::to_value(probe).unwrap(),
            serde_json::json!({
                "status": "remoteBinding",
                "url": "https://spec.example.test/team/",
                "repo": "desktop",
                "hasLocalOpenspec": false,
            })
        );
    }

    #[test]
    fn open_project_at_remote_marker_and_openspec_reports_coexistence() {
        let fx = FixtureRoot::new("open-remote-coexists");
        fx.write(
            ".speclink.yaml",
            "remote:\n  url: https://spec.example.test\n  repo: desktop\n",
        );

        let probe = open_project_at(fx.root()).expect("probe ok");
        assert_eq!(
            serde_json::to_value(probe).unwrap(),
            serde_json::json!({
                "status": "remoteBinding",
                "url": "https://spec.example.test",
                "repo": "desktop",
                "hasLocalOpenspec": true,
            })
        );
    }

    #[test]
    fn open_project_at_malformed_remote_marker_fails_closed() {
        let plain = PlainDir::new("open-remote-bad-yaml");
        std::fs::write(plain.path().join(".speclink.yaml"), "remote: [\n").unwrap();

        let err = open_project_at(plain.path()).expect_err("broken marker must fail");
        assert!(err.contains(".speclink.yaml"), "error names marker: {err}");
        assert!(!err.contains('\n'), "single line: {err:?}");
    }

    #[test]
    fn open_project_at_missing_path_is_a_single_line_error() {
        // 已刪除的路徑必須是 Err——即使其上層目錄是一個 speclink 專案，
        // 也不得向上探索誤命中（分頁指向已消失路徑的錯誤態依賴這點）。
        let fx = FixtureRoot::new("open-missing");
        let gone = fx.root().join("gone");
        let err = open_project_at(&gone).expect_err("must fail");
        assert!(!err.contains('\n'), "single line: {err:?}");
        assert!(err.contains("gone"), "must name the path: {err}");
    }

    #[test]
    fn open_project_at_file_path_is_an_error() {
        // 選擇器理論上只回目錄；防型別混淆——傳進檔案路徑不得 panic 也不得誤判。
        let fx = FixtureRoot::new("open-file");
        fx.write("note.txt", "not a dir");
        let err = open_project_at(&fx.root().join("note.txt")).expect_err("must fail");
        assert!(!err.contains('\n'), "single line: {err:?}");
    }

    // --- init_project_at：確認後自動初始化的檔案效果（spec 需求「未初始化目錄經確認後自動初始化」） ---

    fn read(path: &Path) -> String {
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    #[test]
    fn init_project_at_default_claude_creates_workspace_files() {
        // spec Scenario「確認後初始化並切入新專案」的檔案效果逐項。
        let plain = PlainDir::new("init-claude");
        let probe = init_project_at(plain.path(), &["claude".into()]).expect("init ok");
        match probe {
            ProjectProbe::Project { root, .. } => {
                assert_eq!(root, plain.path().display().to_string());
            }
            other => panic!("expected Project after init, got {other:?}"),
        }
        let p = plain.path();
        assert!(p.join("openspec").join("specs").is_dir());
        assert!(p.join("openspec").join("changes").join("archive").is_dir());
        assert!(p.join("openspec").join("config.yaml").is_file());
        let app = read(&p.join(".speclink.yaml"));
        assert!(app.contains("claude"), "tools must record claude: {app}");
        assert!(read(&p.join("CLAUDE.md")).contains("<!-- SPECLINK:START"));
        let skills = p.join(".claude").join("skills");
        assert!(skills.is_dir());
        assert!(std::fs::read_dir(&skills).unwrap().next().is_some(), "skills dir must not be empty");
        // 未加選 codex 時不得生成 codex 檔。
        assert!(!p.join("AGENTS.md").exists());
        assert!(!p.join(".agents").exists());
    }

    #[test]
    fn init_project_at_with_codex_adds_agents_files() {
        // spec Scenario「勾選 codex 時生成對應工具檔」。
        let plain = PlainDir::new("init-codex");
        init_project_at(plain.path(), &["claude".into(), "codex".into()]).expect("init ok");
        let p = plain.path();
        assert!(read(&p.join("AGENTS.md")).contains("<!-- SPECLINK:START"));
        assert!(p.join(".agents").join("skills").is_dir());
        let app = read(&p.join(".speclink.yaml"));
        assert!(app.contains("claude") && app.contains("codex"), "tools must record both: {app}");
    }

    #[test]
    fn init_project_at_unwritable_target_is_a_single_line_error() {
        // 目標路徑是檔案 → 建骨架必然失敗；Err 單行且不得 panic。
        let fx = FixtureRoot::new("init-unwritable");
        fx.write("blocker.txt", "occupied");
        let err = init_project_at(&fx.root().join("blocker.txt"), &["claude".into()])
            .expect_err("must fail");
        assert!(!err.contains('\n'), "single line: {err:?}");
    }

    #[test]
    fn init_project_at_already_initialized_is_an_error() {
        // GUI 只對 uninitialized 判定開確認框；防禦：對既有專案誤呼叫不得覆寫（force=false）。
        let fx = FixtureRoot::new("init-already");
        assert!(init_project_at(fx.root(), &["claude".into()]).is_err());
    }

    // --- project_stats_at：唯讀待收尾數（spec-archive-drawer design D6 背景分頁徽章快照） ---

    /// 鷹架版討論記錄（比照 discussions.rs 測試）。
    fn discussion_doc(slug: &str, topic: &str, status: &str, extra_fm: &str) -> String {
        format!(
            "---\ntopic: {topic}\nslug: {slug}\nstatus: {status}\n{extra_fm}created: 2026-01-02\n---\n\n\
             # Discussion: {topic}\n\n\
             ## Context\n\nFixture context.\n\n\
             ## Rounds\n\n\
             ## Conclusion\n\n**Decision**: something\n"
        )
    }

    #[test]
    fn project_stats_reports_pending_wrap_up_count() {
        // design D6：待收尾數＝已就緒（任務全完成）變更數＋concluded 未轉出討論數；
        // 進行中／proposed 變更與 promoted／open 討論不計；inProgressChanges 欄位移除
        //（唯一消費者 store.ts 同變更內改寫）。
        let fx = FixtureRoot::new("stats-wrapup");
        mark_adopted(&fx);
        fx.add_change("ready-a", "schema: spec-driven\ncreated: 2026-07-01\n");
        fx.write("openspec/changes/ready-a/tasks.md", "- [x] 1.1 one\n- [x] 1.2 two\n");
        fx.add_change("ready-b", "schema: spec-driven\ncreated: 2026-07-01\n");
        fx.write("openspec/changes/ready-b/tasks.md", "- [x] 1.1 one\n");
        fx.add_change(
            "started-c",
            "schema: spec-driven\ncreated: 2026-07-01\nstarted_at: 2026-07-02T00:00:00Z\n",
        );
        fx.write("openspec/changes/started-c/tasks.md", "- [ ] 1.1 one\n- [ ] 1.2 two\n");
        fx.add_change("proposed-d", "schema: spec-driven\ncreated: 2026-07-01\n");
        fx.write("openspec/changes/proposed-d/tasks.md", "- [ ] 1.1 one\n");
        fx.write(
            "openspec/discussions/alpha-concluded.md",
            &discussion_doc("alpha-concluded", "Alpha", "concluded", ""),
        );
        fx.write(
            "openspec/discussions/beta-open.md",
            &discussion_doc("beta-open", "Beta", "open", ""),
        );
        fx.write(
            "openspec/discussions/gamma-promoted.md",
            &discussion_doc("gamma-promoted", "Gamma", "promoted", "promoted_to: cut-a\n"),
        );
        let stats = project_stats_at(fx.root()).expect("stats ok");
        assert_eq!(stats["pendingWrapUp"], 3, "ready-a + ready-b + alpha-concluded: {stats}");
        assert!(
            stats.get("inProgressChanges").is_none(),
            "inProgressChanges removed (design D6): {stats}"
        );
    }

    #[test]
    fn project_stats_pending_wrap_up_is_zero_when_nothing_awaits_the_user() {
        // 全部收尾後歸零（Implementation Contract）：只剩進行中／proposed 與 open 討論。
        let fx = FixtureRoot::new("stats-wrapup-zero");
        mark_adopted(&fx);
        fx.add_change("proposed-a", "schema: spec-driven\ncreated: 2026-07-01\n");
        fx.write("openspec/changes/proposed-a/tasks.md", "- [ ] 1.1 one\n");
        fx.write(
            "openspec/discussions/beta-open.md",
            &discussion_doc("beta-open", "Beta", "open", ""),
        );
        let stats = project_stats_at(fx.root()).expect("stats ok");
        assert_eq!(stats["pendingWrapUp"], 0, "{stats}");
    }

    #[test]
    fn project_stats_missing_path_is_an_error() {
        let fx = FixtureRoot::new("stats-missing");
        assert!(project_stats_at(&fx.root().join("gone")).is_err());
    }

    #[test]
    fn project_stats_rejects_a_root_that_stopped_being_a_project() {
        // 分頁記錄的 root 其 openspec/ 已刪但目錄還在：探索會向上命中祖先
        // 專案——stats 必須拒絕（Err），不得回報別的專案的數字。
        let fx = FixtureRoot::new("stats-stale");
        let stale = fx.root().join("was-a-project");
        std::fs::create_dir_all(&stale).unwrap();
        assert!(project_stats_at(&stale).is_err());
        // 非專案的獨立目錄同樣是 Err（uninitialized 不是可統計狀態）。
        let plain = PlainDir::new("stats-plain");
        assert!(project_stats_at(plain.path()).is_err());
    }

    #[test]
    fn init_project_at_unknown_tool_is_rejected_with_zero_writes() {
        // 注入非法工具名被拒，且拒絕發生在任何寫入之前。
        let plain = PlainDir::new("init-badtool");
        let before = entries(plain.path());
        let err = init_project_at(plain.path(), &["claude".into(), "vscode".into()])
            .expect_err("must fail");
        assert!(err.contains("vscode"), "must name the offender: {err}");
        assert!(!err.contains('\n'), "single line: {err:?}");
        assert_eq!(entries(plain.path()), before, "no writes on rejection");
    }

    // --- adopt_project_at：確認後補齊啟用的檔案效果（決策 2 消費；design 驗收） ---

    #[test]
    fn adopt_project_at_fills_workspace_and_reports_project() {
        // 呼叫 core adopt 後重跑探測回報 Project；既有 openspec/ 文件不動。
        let fx = FixtureRoot::new("adopt-fill");
        fx.write("openspec/specs/auth/spec.md", "## Purpose\n既有規格。\n");
        let probe = adopt_project_at(fx.root(), &["claude".into()]).expect("adopt ok");
        match probe {
            ProjectProbe::Project { root, .. } => {
                assert_eq!(root, fx.root().display().to_string());
            }
            other => panic!("expected Project after adopt, got {other:?}"),
        }
        let p = fx.root();
        assert!(read(&p.join(".speclink.yaml")).contains("claude"));
        assert!(read(&p.join("CLAUDE.md")).contains("<!-- SPECLINK:START"));
        assert!(p.join(".claude").join("skills").is_dir());
        assert_eq!(
            read(&p.join("openspec").join("specs").join("auth").join("spec.md")),
            "## Purpose\n既有規格。\n"
        );
    }

    #[test]
    fn adopt_project_at_unknown_tool_is_rejected_with_zero_writes() {
        let fx = FixtureRoot::new("adopt-badtool");
        let before = entries(fx.root());
        let err = adopt_project_at(fx.root(), &["claude".into(), "vscode".into()])
            .expect_err("must fail");
        assert!(err.contains("vscode"), "must name the offender: {err}");
        assert!(!err.contains('\n'), "single line: {err:?}");
        assert_eq!(entries(fx.root()), before, "no writes on rejection");
    }
}

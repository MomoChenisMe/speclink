//! 開啟專案：所選目錄的三態判定（命中專案／未初始化／錯誤）與未初始化目錄的
//! 初始化。僅回報判定結果或執行 init——root 的切換由 Tauri 層的狀態持有者決定。

use std::path::Path;

use serde::Serialize;
use speclink_core::skills::Tool;
use speclink_core::workspace::Workspace;

/// `open_project_at` 的判定結果——僅回報，不切換任何狀態。
/// 序列化為 `{ "status": "project" | "uninitialized", … }` 供前端分流。
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum ProjectProbe {
    /// 向上探索命中 speclink 專案：`root` 為探索到的專案根、`name` 為其目錄名。
    Project { root: String, name: String },
    /// 目錄存在且可讀，但不屬於任何 speclink 專案（前端據此開初始化確認框）。
    Uninitialized { dir: String },
}

/// 對所選目錄做開啟專案的三態判定（design D3）：以該目錄為起點沿用
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
        Some(ws) => Ok(ProjectProbe::Project {
            name: project_name(&ws.root),
            root: ws.root.display().to_string(),
        }),
        None => Ok(ProjectProbe::Uninitialized {
            dir: path.display().to_string(),
        }),
    }
}

/// 專案顯示名＝root 目錄名（分頁與頂欄用）；磁碟根等無目錄名時退回完整路徑。
pub fn project_name(root: &Path) -> String {
    root.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.display().to_string())
}

/// 唯讀回報指定專案 root 的進行中變更數（design D11 背景分頁徽章快照）。
/// 不切換任何狀態。計數與看板欄位派生一致：全完成（總數>0）＝ready 不計；
/// 有開工章或任務完成數>0 計入；其餘 proposed 不計。
/// 路徑失效回單行 Err；且探索命中的 root 必須就是 `path` 本身——openspec/
/// 已刪但目錄還在時，向上探索會誤命中祖先專案，不得回報別人的數字。
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
    let n = payload["changes"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter(|c| {
                    let total = c["totalTasks"].as_u64().unwrap_or(0);
                    let done = c["completedTasks"].as_u64().unwrap_or(0);
                    let started = c["startedAt"].as_str().is_some();
                    let ready = total > 0 && done >= total;
                    !ready && (started || done > 0)
                })
                .count()
        })
        .unwrap_or(0);
    Ok(serde_json::json!({ "inProgressChanges": n }))
}

/// 對未初始化目錄執行與 `speclink init` 等效的初始化（design D3：消費
/// `speclink_core::init::init`，force=false、spec_dir 固定 openspec），成功後
/// 重跑三態判定回報命中的專案。`tools` 為內建工具名（claude／codex）；
/// 未知名在任何寫入之前被拒，單行 Err。
pub fn init_project_at(path: &Path, tools: &[String]) -> Result<ProjectProbe, String> {
    let mut selected: Vec<Tool> = Vec::new();
    for name in tools {
        let t = Tool::parse(name)
            .ok_or_else(|| format!("unknown tool '{name}' (supported: claude, codex)"))?;
        if !selected.contains(&t) {
            selected.push(t);
        }
    }
    speclink_core::init::init(path, &selected, false, "openspec").map_err(|e| e.to_string())?;
    open_project_at(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testfixture::FixtureRoot;
    use std::path::{Path, PathBuf};

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

    #[test]
    fn open_project_at_project_root_reports_root_and_name() {
        let fx = FixtureRoot::new("open-root");
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

    // --- project_stats_at：唯讀進行中變更數（design D11 背景分頁徽章快照） ---

    #[test]
    fn project_stats_counts_in_progress_changes_like_the_kanban() {
        // 與看板欄位派生一致：全完成＝ready 不計；有開工章或任務完成數>0 計入；
        // 其餘 proposed 不計。
        let fx = FixtureRoot::new("stats");
        fx.add_change("proposed-a", "schema: spec-driven\ncreated: 2026-07-01\n");
        fx.write(
            "openspec/changes/proposed-a/tasks.md",
            "- [ ] 1.1 one\n- [ ] 1.2 two\n",
        );
        fx.add_change(
            "started-b",
            "schema: spec-driven\ncreated: 2026-07-01\nstarted_at: 2026-07-02T00:00:00Z\n",
        );
        fx.write(
            "openspec/changes/started-b/tasks.md",
            "- [ ] 1.1 one\n- [ ] 1.2 two\n",
        );
        // progressed-c 用 fixture 預設 tasks（1/2 完成）、無開工章。
        fx.add_change("progressed-c", "schema: spec-driven\ncreated: 2026-07-01\n");
        fx.add_change("ready-d", "schema: spec-driven\ncreated: 2026-07-01\n");
        fx.write(
            "openspec/changes/ready-d/tasks.md",
            "- [x] 1.1 one\n- [x] 1.2 two\n",
        );
        let stats = project_stats_at(fx.root()).expect("stats ok");
        assert_eq!(stats["inProgressChanges"], 2, "started-b + progressed-c: {stats}");
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
}

//! 測試共用的暫存專案 fixture：真實 openspec/ 目錄佈局，測試結束自動清除。
//! 取代早期直接指向本 repo 的自參照 fixture——repo 內容會隨 change 歸檔而變，
//! 自參照測試會無聲腐化（desktop-shell-and-browser 歸檔後即斷）。

use std::path::{Path, PathBuf};

pub(crate) struct FixtureRoot(PathBuf);

/// [`FixtureRoot::attach_worktree`] 建立的 worktree 副本：路徑帶著往 change
/// 目錄的導航，測試不再以裸 `PathBuf` 流轉、逐處手拼 `openspec/changes/<名>`。
pub(crate) struct WorktreeCopy(PathBuf);

impl WorktreeCopy {
    /// worktree 根（git 回報的正規化路徑）。
    pub fn path(&self) -> &Path {
        &self.0
    }

    /// 一個 change 在這份副本內的目錄——寫入「只存在於 worktree」的內容用。
    pub fn change_dir(&self, change: &str) -> PathBuf {
        self.0.join("openspec").join("changes").join(change)
    }
}

impl FixtureRoot {
    pub fn new(tag: &str) -> FixtureRoot {
        let dir = std::env::temp_dir().join(format!(
            "speclink-dtcore-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("openspec").join("changes")).unwrap();
        FixtureRoot(dir)
    }

    pub fn root(&self) -> &Path {
        &self.0
    }

    pub fn write(&self, rel: &str, content: &str) {
        let path = self.0.join(rel.split('/').collect::<PathBuf>());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, content).unwrap();
    }

    /// 主 checkout ＋ 一個 speclink/<change> worktree，change 兩份副本皆在——
    /// 探索成立映射的三個條件。
    ///
    /// 呼叫前寫入的檔案都會進 seed commit，因此也存在於 worktree 副本；之後兩邊
    /// 各自改檔即可製造「主 checkout 與 worktree 相異」的情境。
    pub fn attach_worktree(&self, change: &str) -> WorktreeCopy {
        let git = |args: &[&str]| {
            let out = std::process::Command::new("git")
                .args(args)
                .current_dir(self.root())
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@example.test")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@example.test")
                .output()
                .expect("run git");
            assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
        };
        self.write(".speclink.yaml", "tools:\n  - claude\n");
        self.write("openspec/config.yaml", "schema: spec-driven\nworktree: true\n");
        git(&["init", "-q", "-b", "main"]);
        git(&["add", "-A"]);
        git(&["commit", "-qm", "seed"]);
        let wt = self.0.join("wt");
        git(&["worktree", "add", "-q", "-b", &format!("speclink/{change}"), wt.to_str().unwrap()]);
        // 期望值與產品同源：產品逐字取 `git worktree list --porcelain` 的路徑，這裡
        // 也拿 git 回報的拼法。自行 canonicalize 逼近不了——macOS 它解 /var symlink
        // 剛好一致，Windows 卻會加上 \\?\ 前綴，而 git 回報的是正斜線＋8.3 短名
        // （RUNNER~1）展開後的長名，兩種拼法指同一目錄仍對不上。
        let out = std::process::Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .current_dir(self.root())
            .output()
            .expect("git worktree list");
        let text = String::from_utf8_lossy(&out.stdout).into_owned();
        let reported = text
            .lines()
            .filter_map(|line| line.strip_prefix("worktree "))
            .next_back()
            .expect("porcelain lists the new worktree");
        WorktreeCopy(PathBuf::from(reported))
    }

    /// 建一個含 proposal 與 tasks 的 change；meta 原文由呼叫端給（測 started_* 疊加）。
    pub fn add_change(&self, name: &str, meta: &str) {
        self.write(&format!("openspec/changes/{name}/.openspec.yaml"), meta);
        self.write(
            &format!("openspec/changes/{name}/proposal.md"),
            "## Why\n\nDemo change.\n\n## What Changes\n\n- something\n",
        );
        self.write(
            &format!("openspec/changes/{name}/tasks.md"),
            "## 1. Group\n\n- [ ] 1.1 First task\n- [x] 1.2 Second task\n",
        );
    }
}

/// 非 speclink 專案的空暫存目錄（向上探索找不到 openspec/ 的情境）。
pub(crate) fn fresh_non_project_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("speclink-dtcore-nonproject-{tag}-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    dir
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

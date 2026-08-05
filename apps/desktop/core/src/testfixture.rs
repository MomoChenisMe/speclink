//! 測試共用的暫存專案 fixture：真實 openspec/ 目錄佈局，測試結束自動清除。
//! 取代早期直接指向本 repo 的自參照 fixture——repo 內容會隨 change 歸檔而變，
//! 自參照測試會無聲腐化（desktop-shell-and-browser 歸檔後即斷）。

use std::path::{Path, PathBuf};

pub(crate) struct FixtureRoot(PathBuf);

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
    /// 探索成立映射的三個條件。回傳 worktree 路徑（正規化）。
    ///
    /// 呼叫前寫入的檔案都會進 seed commit，因此也存在於 worktree 副本；之後兩邊
    /// 各自改檔即可製造「主 checkout 與 worktree 相異」的情境。
    pub fn attach_worktree(&self, change: &str) -> PathBuf {
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
        // git 回報的是正規化路徑；macOS 的 /var 是 /private/var 的 symlink，
        // 不正規化就會拿 symlink 路徑去比對 git 的實體路徑。
        wt.canonicalize().expect("worktree path")
    }

    /// 一個 change 在 worktree 副本內的目錄——寫入「只存在於 worktree」的內容用。
    pub fn worktree_change_dir(wt: &Path, change: &str) -> PathBuf {
        wt.join("openspec").join("changes").join(change)
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

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

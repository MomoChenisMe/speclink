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

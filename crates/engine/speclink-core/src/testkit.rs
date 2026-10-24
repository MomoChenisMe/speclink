//! Test-support helpers shared by this workspace's test suites (doc-hidden, not API).
//!
//! 「把工作區技能檔的版號改成指定值」在 core、CLI 與 desktop 的測試各需要一份——
//! 集中在這裡，frontmatter 的讀寫邏輯才不會抄三份、改一處漏兩處。

use std::path::{Path, PathBuf};

/// `root` 之外的 worktree 路徑，逐字取 `git worktree list --porcelain` 的拼法——產品
/// 也是這樣取的。期望值不得自行 canonicalize：Windows 上它會加 `\\?\` 前綴，git
/// 回報的卻是正斜線路徑，兩種拼法指同一目錄仍對不上。
pub fn listed_worktrees(root: &Path) -> Vec<PathBuf> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["worktree", "list", "--porcelain"])
        .output()
        .expect("git worktree list");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.strip_prefix("worktree "))
        .skip(1)
        .map(PathBuf::from)
        .collect()
}

/// 把 `skills_root` 下每份 speclink-*/SKILL.md 的 frontmatter 版號字串改成
/// `version`（模擬以別版引擎生成的工作區——舊值模擬落後、新值模擬領先）。
pub fn set_skill_version(skills_root: &Path, version: &str) {
    for entry in std::fs::read_dir(skills_root).expect("skills 目錄存在").flatten() {
        let file = entry.path().join("SKILL.md");
        if !file.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&file)
            .unwrap()
            .replace(crate::init::ASSET_VERSION, version);
        std::fs::write(&file, text).unwrap();
    }
}

/// 在 `skills_root` 下寫一份 `<name>/SKILL.md`，frontmatter 帶 `version`——不經
/// `speclink update` 生成技能時（例如 remote 模式的測試專案）模擬某版引擎的產物。
pub fn write_skill_file(skills_root: &Path, name: &str, version: &str) {
    let dir = skills_root.join(name);
    std::fs::create_dir_all(&dir).unwrap();
    let text = format!("---\nname: {name}\nmetadata:\n  version: \"{version}\"\n---\n");
    std::fs::write(dir.join("SKILL.md"), text).unwrap();
}

/// 讀一份 SKILL.md frontmatter 的版號值（不含引號）；測試斷言用。
pub fn skill_frontmatter_version(skill_file: &Path) -> String {
    let text = std::fs::read_to_string(skill_file).expect("SKILL.md exists");
    let line = text
        .lines()
        .find(|l| l.trim_start().starts_with("version:"))
        .expect("frontmatter carries a version line");
    line.split_once("version:").unwrap().1.trim().trim_matches('"').to_string()
}

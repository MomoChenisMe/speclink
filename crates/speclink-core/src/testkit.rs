//! Test-support helpers shared by this workspace's test suites (doc-hidden, not API).
//!
//! 「把工作區技能檔的版號改成指定值」在 core、CLI 與 desktop 的測試各需要一份——
//! 集中在這裡，frontmatter 的讀寫邏輯才不會抄三份、改一處漏兩處。

use std::path::Path;

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

/// 讀一份 SKILL.md frontmatter 的版號值（不含引號）；測試斷言用。
pub fn skill_frontmatter_version(skill_file: &Path) -> String {
    let text = std::fs::read_to_string(skill_file).expect("SKILL.md exists");
    let line = text
        .lines()
        .find(|l| l.trim_start().starts_with("version:"))
        .expect("frontmatter carries a version line");
    line.split_once("version:").unwrap().1.trim().trim_matches('"').to_string()
}

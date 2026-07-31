## 1. 移除 settings.json 生成（TDD）

- [x] 1.1 撰寫紅燈測試釘死規格需求「工具檔生成不寫入 AI 工具的使用者設定檔」的三場景：crates/speclink-core/src/init.rs 的 #[cfg(test)] 新增——(a)「init 不產生使用者設定檔」：init(tools=[claude]) 後 .claude/settings.json 不存在且 .claude/skills/speclink-propose/SKILL.md 存在；(b)「既有使用者設定檔在工具同步後位元級不變」：以規格 Example 的 JSON（{"enabledPlugins":{"frontend-design":true},"includeGitInstructions":false}）預置 .claude/settings.json，執行 update 後該檔位元級不變且技能檔照常再生；(c)「工作區補齊不產生使用者設定檔」：adopt 後 .claude/settings.json 不存在。同批把 crates/speclink-cli/tests/remote_section.rs 既有的「Claude settings written」斷言反轉為「settings.json 不存在」。驗證：cargo test -p speclink-core 與 cargo test -p speclink-cli remote_section 紅燈。 <!-- speclink-task:tsk_01KYVK4F4HG6X2DEGG88ZPBRK7 -->
- [x] 1.2 移除生成：刪除 crates/speclink-core/src/init.rs 的 CLAUDE_SETTINGS 常數與 generate_tool 內 Claude 分支寫入 settings.json 的呼叫；不動 CLAUDE.md marker、技能檔、prune 與 .gitignore 的既有行為。驗證：1.1 測試綠燈。 <!-- speclink-task:tsk_01KYVK4F4H7BVAKA7M8E63S79C -->

## 2. 收尾驗證

- [x] 2.1 全套測試：cargo test（workspace 全量，含 render golden 與 CLI 整合測試）。驗證：全綠、無其他測試殘留 settings.json 假設。 <!-- speclink-task:tsk_01KYVK4F4HG7Y3JY8G96H6WZEA -->
- [x] 2.2 speclink validate remove-claude-settings-write 通過。驗證：無 Critical 與 Warning。 <!-- speclink-task:tsk_01KYVK4F4HX79HYY4N9YDF8HW5 -->

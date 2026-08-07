## 1. 正典資產調整（紅→綠）

- [x] 1.1 撰寫 config 技能內容測試（紅）：對規格「技能規定固定輸入來源與四條內容判準」的三個新場景與「技能規定任務驗證測試範圍的第五問」的全部場景，於 crates/speclink-core/tests/it/render_golden.rs 增內容斷言——claude 與 codex 兩 flavor 的生成 config 技能檔皆含：判準四靜態核實與不執行引用指令字樣（並明示 instructions payload 探測除外）、刪除理由限定原則句（非固定輸入來源可導出不構成刪除理由）、scope hint 收窄語意段（判準一至三收窄、判準四恆全文件）、第五問段（任務驗證測試範圍、現值帶入、答受影響面自 manifests 組規則落 rules 的 tasks 段、答全量不寫規則）。驗證:cargo test -p speclink-core --test it render_golden:: 新斷言全紅、既有斷言不受影響 <!-- speclink-task:tsk_01KZDDH9RH7Y6QXDAW7TN2RCH0 -->
- [x] 1.2 實作資產調整（綠）：crates/speclink-core/assets/skills/config.md 落 A–D 四處文字（對照 proposal What Changes 的 A–D 對照表與來源討論結論），crates/speclink-core/src/init.rs 的 MARKER_VERSION 自 v1.18.1 升至 v1.18.2。驗證:1.1 的新斷言全綠 <!-- speclink-task:tsk_01KZDDH9RH57SF6DW2GZ9PQE8Z -->
- [x] 1.3 同步生成面：乾淨樹再生 crates/speclink-core/tests/golden 全部快照與 assets.lock，並以 checkout CLI 執行 speclink update 再生 .claude/skills 與 .agents/skills 的技能檔及 CLAUDE.md、AGENTS.md 注入區塊版號。驗證:cargo test -p speclink-core 全綠（含 golden 位元一致與 assets.lock 鎖定測試），git diff 確認生成檔變動僅限版號與 config 技能內容 <!-- speclink-task:tsk_01KZDDH9RHF2KYCB0Y0ZVTW1FB -->
## 2. 端到端與收尾驗證

- [x] 2.1 E2E 走查：以更新後的 .claude/skills/speclink-config/SKILL.md 實跑一次整理流程——政策詢問含第五問且帶現值、判準四核實全程無測試或建置執行、rules 汰留判斷含刪除理由限定。驗證(只跑受影響面):cargo test -p speclink-core -p speclink-cli 全綠、crates/speclink-node 的 napi build 與 npm test 全綠（binding 與 CLI 版本一致面）、speclink validate config-skill-rule-alignment 通過；全量 npm run test:all 由 CI 守門 <!-- speclink-task:tsk_01KZDDH9RHE98VK5QHWBV39RYF -->

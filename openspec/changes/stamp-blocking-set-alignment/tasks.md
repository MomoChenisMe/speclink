## 1. 引擎守門收斂到必修分界（TDD）

- [ ] 1.1 [Red] 為兩站 stamp 守門新增失敗測試：末輪僅 SUGGESTION 級 findings 時 `verify stamp`／`review stamp`（無 `--accept`）成功蓋章（五欄寫入＋工單刪除）；末輪含 WARNING 或 CRITICAL 時拒絕且 stderr 點名未解必修數；`--accept` 照常放行必修——覆蓋「驗證蓋章守門與蓋章效果」與「蓋章守門與蓋章效果」兩 delta 的新 Scenario。測試落在 crates/speclink-core/src/verify.rs 與 crates/speclink-core/src/review.rs 的 `#[cfg(test)]` 模組。驗證：cargo test -p speclink-core 顯示新測試為紅、既有測試不誤傷。 <!-- speclink-task:tsk_01KZH2APP18HKB3YHHB4AAS9DP -->
- [ ] 1.2 [Green] 共用守門的未解計數改為只算必修（過濾 Severity::Suggestion），拒絕訊息點名未解必修數（含 CRITICAL/WARNING 字樣），其餘守門（任務完成、fail-closed、原子寫入）不變——crates/speclink-core/src/station.rs。驗證：cargo test -p speclink-core 全綠。 <!-- speclink-task:tsk_01KZH2APP1F8WH63C6GZKEC1DF -->
- [ ] 1.3 CLI 整合測試補齊兩站行為：SUGGESTION-only 末輪 stamp 的 exit code 0 與蓋章效果、必修末輪 stamp 的 exit code 非零與 stderr 措辭斷言，既有拒絕訊息斷言同步更新——crates/speclink-cli/tests/it/verify_verbs.rs、crates/speclink-cli/tests/it/review_verbs.rs。驗證：cargo test -p speclink-cli --test it 全綠。 <!-- speclink-task:tsk_01KZH2APP10GS45QRH4D39PHK9 -->

## 2. 技能資產收窄與衍生物再生

- [ ] 2.1 verify 技能資產把「驗證收尾迴圈」的收窄落進文字：triage 阻斷分界＝嚴重度（可裁一律記 SUGGESTION、不入 WARNING、不進接受機制）、三選項詢問僅必修觸發、僅 SUGGESTION 的輪直接乾淨蓋章、loop 規則的 accepted 收窄為必修——crates/speclink-core/assets/skills/verify.md。驗證：內容審閱對照本 change 的 verify-skill delta spec 逐條相符。 <!-- speclink-task:tsk_01KZH2APP1NNZXFZD09RVXMVYX -->
- [ ] 2.2 review 技能資產把「審查後的迴圈與收尾」「審查結果的裁量分類」「已接受事項的續輪前饋」的收窄落進文字：可裁一律 SUGGESTION、WARNING 保留給必修級判定、僅剩可裁不詢問直接蓋、接受機制限必修級——crates/speclink-core/assets/skills/review.md。驗證：內容審閱對照本 change 的 review-skill delta spec 逐條相符。 <!-- speclink-task:tsk_01KZH2APP1YTB7KT6PPZ7RJ78B -->
- [ ] 2.3 quality 技能資產把「兩站時序的編排行為」的帶保留章定義收窄：僅使用者裁示不修的必修構成保留、SUGGESTION 殘留落乾淨章——crates/speclink-core/assets/skills/quality.md。驗證：內容審閱對照本 change 的 quality-skill delta spec 相符。 <!-- speclink-task:tsk_01KZH2APP1DEADA3EXMFXV3QSD -->
- [ ] 2.4 三連動同批完成：MARKER_VERSION 遞增（crates/speclink-core/src/init.rs）、golden 快照與 crates/speclink-core/tests/golden/assets.lock 再生、以重建後的 CLI 執行 speclink update 再生 claude 與 codex 兩個 render target 的技能檔（含 .claude/skills/speclink-verify/SKILL.md、.claude/skills/speclink-review/SKILL.md、.claude/skills/speclink-quality/SKILL.md）。驗證：cargo test -p speclink-core --test it 全綠（render_golden 含新內容）、git status 確認資產與衍生物同批入列。 <!-- speclink-task:tsk_01KZH2APP1WQ6M51X8RJHQS5CQ -->

## 3. 收尾

- [ ] 3.1 全量回歸與一致性收尾：cargo test -p speclink-core 與 cargo test -p speclink-cli 全綠，./target/debug/speclink validate stamp-blocking-set-alignment 通過；人工斷言：以測試工單重現「末輪僅 SUGGESTION → stamp 直接成功」的端到端行為與 `--json` 欄位 shape 不變（camelCase 欄位名無增減）。 <!-- speclink-task:tsk_01KZH2APP1PG6Q2PVWYBEFFFEJ -->

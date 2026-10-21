## 1. quality 技能

- [ ] 1.1 （Red）crates/engine/speclink-core/tests/it/render_golden.rs 新增測試 `quality_skill_prevents_stale_stamps`，釘住需求「蓋章前的預防提醒」：claude 與 codex 生成的 speclink-quality 在準備兩道關卡的步驟含「不在主分支時以 git 找出主分支在分岔點之後改過的檔、與本分支改過的檔取交集，有交集就停下列出檔案並建議先 rebase」、交集為空／在主分支上／找不到主分支時直接繼續；每輪詢問的「進入收尾蓋章」選項在 repo 有 CI 設定時附註先確認 CI 在所有平台通過；並寫明不自動 rebase、合併、push 或查詢 CI。驗證：`cargo test -p speclink-core --test it quality_skill_prevents_stale_stamps` 失敗 <!-- speclink-task:tsk_01M5EQHEFXMMKZGTTK6NDZ4105 -->
- [ ] 1.2 （Green）修改 crates/engine/speclink-core/assets/skills/quality.md 的「Prepare both stations」與「Stop and ask — the round's pause」兩步，加入上述兩條提醒；不新增大寫強調字。驗證：`cargo test -p speclink-core --test it quality_skill_prevents_stale_stamps` 通過，`cargo test -p speclink-core --test it skill_authoring` 通過（quality.md 強調字仍為 3） <!-- speclink-task:tsk_01M5EQHEFXPT1YVMGEPJ2W712P -->
- [ ] 1.3 把 crates/engine/speclink-core/src/workspace/init.rs 的 ASSET_VERSION 由 v1.51.0 升為 v1.52.0，以 `UPDATE_GOLDEN=1 cargo test -p speclink-core --test it render_golden` 再生 golden snapshot、以 `UPDATE_ASSETS_LOCK=1 cargo test -p speclink-core --test it render_golden` 再生 crates/engine/speclink-core/tests/golden/assets.lock，再 `cargo build -p speclink-cli` 並以 `./target/debug/speclink update` 再生 repo 自用技能檔。驗證：`git diff crates/engine/speclink-core/tests/golden` 只有 quality 技能內文與版號行的差異；`./target/debug/speclink --version` 顯示 engine v1.52.0；再生的 SKILL.md 不經 task done 記入 evidence，commit 前以 git status 盤點 <!-- speclink-task:tsk_01M5EQHEFX3Q2N4M7BXH89G6K7 -->

## 2. workflow 文件

- [ ] 2.1 docs/workflow.zh-TW.md 與 docs/workflow.md 的「品質關卡的共同規則」新增兩條規矩並附原因（需求「workflow 文件載明蓋章前的預防規矩」）：先 rebase 或合併最新的主分支再跑關卡；專案有 CI 時先等 CI 在所有平台通過再蓋章；同一節「蓋章後檔案又被改」改寫為重新蓋章要從首輪重跑兩道關卡（工單在蓋章時已刪除）。驗證：內容檢查中英兩版一一對應、該節不再出現「再跑一輪」的說法；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過 <!-- speclink-task:tsk_01M5EQHEFXQGTEWNV61QXAMSA2 -->

## 3. 收尾

- [ ] 3.1 受影響面測試與文件守門全部通過。驗證：`cargo test -p speclink-core --test it render_golden skill_authoring`、`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 皆通過，且 `speclink validate stamp-order-rules` 通過 <!-- speclink-task:tsk_01M5EQHEFXK3XFYBE3PS4AV4D2 -->

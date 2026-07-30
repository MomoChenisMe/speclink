## 1. 更新內嵌技能事實來源（assets）

- [x] 1.1 重寫 crates/speclink-core/assets/skills/discuss.md 的 interview 提問紀律為決策樹遍歷（對應 Requirement: interview 模式以決策樹遍歷提問）：How to Discuss 區改為「開場攤開決策空間（根節點＝這題到底在決定什麼，展開子決策與依賴邊）、依依賴順序一次一題、上游決策先解」，並明定停止條件維持使用者主導（使用者喊停時未走分支記入結論 Deferred）。驗證：檢視該檔含上述規則字句，且 Convergence 區的 one nudge maximum 與 Deferred 段落保留、未出現「所有分支解完才可收斂」的規定。 <!-- speclink-task:tsk_01KYSHN8M3JJ7JE5WDKEZ2S5YE -->
- [x] 1.2 於 crates/speclink-core/assets/skills/discuss.md 的 interview 提問規則明定兩條硬規則（對應 Requirement: interview 每題附建議答案與 Evidence、Requirement: 事實與決策分診及逐節點查證）：每題附建議答案且必附 Evidence（檔案路徑或查證結果，使用者僅需同意或修正）；事實／決策分診——環境查得到的事實沿樹逐節點以工具自行查證，不得問使用者、不得憑印象作答，僅決策交由使用者裁定。驗證：檢視該檔兩條規則以硬規則語氣表述（非軟性建議），且開場 codebase scout 段落維持淺掃時間盒（數秒、至多 5 檔）與「僅用於選模式」的用途不變。 <!-- speclink-task:tsk_01KYSHN8M39H8C557E4ENEK96E -->
- [x] 1.3 於 crates/speclink-core/assets/skills/discuss.md 的記錄規則加入樹慣例（對應 Requirement: 討論記錄的樹慣例與格式不變）：首輪 Position 攤開初始決策空間（得含 ASCII 樹）、之後每輪解一個節點、中途發現的新分支記入該輪 Open。驗證：檢視記錄規則段落含樹慣例三點，且 Context／Rounds／Conclusion 骨架與 Focus／Position／Ruled out／Open 欄位定義未變。 <!-- speclink-task:tsk_01KYSHN8M3BPWRB1M3757TXVMN -->

## 2. 再生 render golden（刻意更新）

- [x] 2.1 以 git status 確認除本變更的 assets 編輯外，無其他未提交改動會影響渲染輸入，然後帶 UPDATE_GOLDEN=1 執行 cargo test -p speclink-core --test render_golden 再生 crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md。驗證：git diff 顯示四份 snapshot 僅含 speclink-discuss 技能文字的預期變更，無其他技能內容或未提交狀態滲入。 <!-- speclink-task:tsk_01KYSHN8M3Z5VXXGD7ZQWHG12B -->
- [x] 2.2 不帶 UPDATE_GOLDEN 重跑 cargo test -p speclink-core --test render_golden。驗證：測試綠燈，golden 與渲染輸出一致。 <!-- speclink-task:tsk_01KYSHN8M3E9B08Y31KERVN26F -->

## 3. 同步工具技能實例

- [x] 3.1 將任務 1.1–1.3 的相同內容變更套用至 .claude/skills/speclink-discuss/SKILL.md 與 .agents/skills/speclink-discuss/SKILL.md（兩檔與 assets 版僅 frontmatter／工具殼差異；涵蓋 Requirement: interview 模式以決策樹遍歷提問、interview 每題附建議答案與 Evidence、事實與決策分診及逐節點查證、討論記錄的樹慣例與格式不變 的相同文字）。驗證：以 diff 比對三處檔案的 interview 紀律、Evidence 硬規則、分診規則、樹慣例段落，內文一致。 <!-- speclink-task:tsk_01KYSHN8M3VTYSTBEKQN5DTRRV -->

## 4. 收尾走查

- [x] 4.1 對照 delta spec（openspec/changes/discuss-decision-tree-interview/specs/discuss-skill/spec.md）逐場景走查：四個 Requirement（interview 模式以決策樹遍歷提問、interview 每題附建議答案與 Evidence、事實與決策分診及逐節點查證、討論記錄的樹慣例與格式不變）的六個 Scenario 均能在渲染產物（golden snapshot）與三處技能檔中指認對應文字。驗證：執行 speclink validate discuss-decision-tree-interview 通過，走查清單六項全數指認成功。 <!-- speclink-task:tsk_01KYSHN8M3DMJ88XZGMZC11SA4 -->

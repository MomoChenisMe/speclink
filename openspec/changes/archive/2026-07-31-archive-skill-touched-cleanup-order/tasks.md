## 1. 更新 archive 技能事實來源（assets）

- [x] 1.1 於 crates/speclink-core/assets/skills/archive.md 將「Clean up tracking file」步驟從 speclink archive 之前移到之後（對應 Requirement: touched 記錄的刪除排在封存與提交之後）：刪除 .speclink/touched/<change>.json 的指示改列於執行封存的步驟之後，並註明須待該次封存的提交完成；同步寫明兩項理由——該記錄是 @trace 檔案清單來源（封存前刪除會退回掃描工作樹髒檔而混入無關檔案）、該記錄是 commit 技能的檔案歸屬來源（提交前刪除會使檔案清單消失）；其餘步驟編號因移動而位移者一併更新，檔內所有交叉引用（例如 bulk 段的「as in step above」指涉）同步改指正確步驟。驗證：檢視該檔中刪除指示位於封存指令之後、含上述兩項理由，且全檔不再有任何在封存前刪除該檔的指示；步驟編號與交叉引用無殘留舊值。 <!-- speclink-task:tsk_01KYTZQ3B5KNYHF92X38CR4CFG -->
- [x] 1.2 於 crates/speclink-core/assets/skills/archive.md 的 bulk archive 段修正 @trace 來源敘述（對應 Requirement: @trace 來源敘述與引擎行為一致）：將「工作樹髒檔集就是 @trace 來源」改為條件式——存在 evidence 記錄時清單由記錄聚合建立、記錄缺席時才退回髒檔集；保留整潔工作樹的要求，理由改敘為避免記錄缺席時退路取得無關檔案。驗證：檢視該段敘述含來源優先序兩種情況、未將髒檔集寫成無條件唯一來源，且整潔工作樹要求仍在並帶新理由。 <!-- speclink-task:tsk_01KYTZQ3B5PVZQEVMXJ7KQDH0Y -->

## 2. 同步工具技能實例

- [x] 2.1 將任務 1.1–1.2 的相同內容變更套用至 .claude/skills/speclink-archive/SKILL.md 與 .agents/skills/speclink-archive/SKILL.md（各檔與 assets 版僅 frontmatter 與工具殼呼叫前綴差異；涵蓋 Requirement: touched 記錄的刪除排在封存與提交之後、@trace 來源敘述與引擎行為一致 的相同文字）。驗證：以 diff 比對三處檔案的清理步驟位置、兩項理由文字與 bulk 段 @trace 來源敘述，內文一致；兩實例間差異僅剩既有的工具殼呼叫前綴。 <!-- speclink-task:tsk_01KYTZQ3B5R87J1YDASF984C8A -->

## 3. 再生 render golden（刻意更新）

- [x] 3.1 以 git status 確認除本變更的 assets 與技能實例編輯外，無其他未提交改動會影響渲染輸入，然後帶 UPDATE_GOLDEN=1 執行 cargo test -p speclink-core --test render_golden 再生 crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md。驗證：git diff 顯示四份 snapshot 僅含 speclink-archive 技能文字的預期變更，無其他技能內容或未提交狀態滲入。 <!-- speclink-task:tsk_01KYTZQ3B5H7ZSWG1SYTP18AX6 -->
- [x] 3.2 不帶 UPDATE_GOLDEN 重跑 cargo test -p speclink-core --test render_golden。驗證：測試綠燈，golden 與渲染輸出一致。 <!-- speclink-task:tsk_01KYTZQ3B5A5R9XYWVNBM8NGTG -->

## 4. 收尾走查

- [x] 4.1 對照 delta spec（openspec/changes/archive-skill-touched-cleanup-order/specs/archive-skill/spec.md）逐場景走查：兩個 Requirement 的五個 Scenario 均能在渲染產物（golden snapshot）與三處技能檔中指認對應文字。驗證：執行 speclink validate archive-skill-touched-cleanup-order 通過，走查清單五項全數指認成功。 <!-- speclink-task:tsk_01KYTZQ3B55DY65VQVN5FAATM3 -->

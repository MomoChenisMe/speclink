## Why

透過 AI 代理跑規格驅動開發的開發者、PO、PM，在討論、提案、實作、品質關卡、封存每一個 workflow 階段，都靠 speclink 產生的 19 個技能指揮模型。這些技能會被 Claude、Codex（GPT）、Copilot 可選的模型，以及自訂描述子接上的任意模型讀取。原稿經過一次次補丁：最長的 discuss 有 484 行；全部原稿約有 290 個全大寫強調字；同一條規則在一份技能裡寫了好幾次。這讓模型容易漏讀規則，也容易過度觸發。OpenAI 與 Anthropic 的官方提示詞指南對這些問題有共同建議，但專案目前沒有寫法規範，沒有守門擋住原稿退化，也沒有方法量測改寫後模型是否更照做。

## What Changes

- 新增一份跨模型的技能寫法規範，共十條：每條規則只完整陳述一次，寫在模型做那件事的步驟裡；規則之間不矛盾，有例外時寫明哪一條優先；強調字只用在鐵律；一個概念一個詞；依工具而不同的能力都附退路；多步驟流程用編號步驟、判斷點寫成「條件 → 動作」、固定格式的產出附樣板、結尾附自我檢查清單；不寫歷史；單一原稿檔不超過 500 行；判斷標準的原文不濃縮；技能互相引用一律寫成 /speclink:<名稱>，--agent 的值一律寫成 {{TOOL}}，提到 plan mode 的內容獨立成一行（Claude、Codex、Copilot 與自訂描述子共用同一份原稿，只靠這些記號區分）。
- 新增守門測試，機械檢查五項：原稿行數、強調字上限（以改寫後的次數登記，只能降低）、程式碼以外的日期與 design 決策編號、工具名的退路句、寫死的技能引用（/speclink-<名稱>、$speclink-）與寫死的 --agent 值。
- 新增技能評測套件與執行腳本：19 個技能每個至少一題單輪評測。舊版與新版 speclink 執行檔各跑一次，模型為 claude-opus-5-5 與 claude-haiku-5-5，每題 3 次，不跑無技能對照組，再逐題比較分數。
- 依寫法規範一次改寫 19 個技能的 20 份原稿（apply-with-worktree 由 apply-worktree-pre、apply、apply-worktree-post 三份組成）。行為不變（下一點的引用修正除外）：逐技能規格要求的內容全部保留，技能的 description 不變。
- 修正 drift、archive、apply、commit 四份原稿中 14 處寫死的 /speclink-<名稱> 引用，改為 /speclink:<名稱>。Claude 與 Copilot 版的輸出不變；Codex 版從錯誤的 /speclink-<名稱> 改為正確的 $speclink-<名稱>；自訂描述子版從 speclink-<名稱> 改為與其他引用一致的 speclink <名稱>。這是刻意修正。
- 調整規格的字面要求：逐技能規格的 scenario 逐字釘住重複或措辭、而與寫法規範衝突時，以 MODIFIED 改寫該 scenario 的字面要求，行為要求與 scenario 名稱不變。目前已知的是 discuss-skill「討論記錄的樹慣例與格式不變」：結論條列規則改為只在 conclude 步驟完整陳述；以及 improve-skill「conclude 範例與討論結論條列規則同形」的「技能其餘段落逐位元不變」一句。改寫期間找到的其他同類 scenario，依同一原則補進 delta。
- ASSET_VERSION 只升一次，golden 快照與 assets.lock 只重產一次。
- 在 Codex 與 Copilot 上手動抽查 propose、apply、discuss 三份技能。

## Capabilities

### New Capabilities

- `skill-authoring`: 技能原稿的跨模型寫法規範、機械守門與技能評測套件。最接近的既有規格有三類：skill-routing 只管跨技能的路由職責（description 觸發句與交棒句），並明言不管單一技能的內文；verb-contract 只有「技能資產不含直接讀檔指示」一條讀取規則；17 份逐技能規格各自只管一個技能的行為。沒有任何規格約束技能文字本身的寫法，也沒有規格要求量測技能的效果。

### Modified Capabilities

- `discuss-skill`: 「討論記錄的樹慣例與格式不變」的結論條列規則，改為只在 conclude 步驟完整陳述一次；文件規則清單與 Capture decisions 摘要改用一句話指向它。規則內容與結論的形狀不變。
- `improve-skill`: 「conclude 範例與討論結論條列規則同形」刪掉「改動只落在兩個範例、技能其餘段落逐位元不變」一句——那是當初那次變更的範圍限制，會擋下依寫法規範的改寫。兩個 conclude 範例的條列形要求不變。

## Impact

- 目標使用者與情境：透過 AI 代理跑 SDD 的開發者／PO／PM，涵蓋所有 workflow 階段（19 個技能全部）。
- 影響的 crate 與目錄：speclink-core（技能原稿、ASSET_VERSION、測試）、integrations/claude-code（新增評測套件）、scripts/claude-code（評測執行腳本與守門測試）。
- 影響的技能與工具：19 個技能在 claude、codex、copilot 與自訂描述子四種渲染目標的內文全部改變；frontmatter 與 description 不變。三個內部技能（sync、clarify、tdd，經 instructions --skill 取用）不改寫，但同樣受守門測試檢查，強調字上限以現況登記。
- 相容性影響：
  - 不新增或變更任何 CLI 指令、旗標、stdin 或 exit code；人眼輸出與 --json 輸出不變。
  - ASSET_VERSION 升版後，既有工作區的技能檔會被探測為過期（desktop 顯示更新提示）。使用者執行 speclink update 即可重產，不需要遷移。
  - golden 快照與 assets.lock 的改變屬刻意變更。
  - Codex 與自訂描述子版的 drift、archive、apply、apply-with-worktree、commit 五個技能中，14 處技能引用的寫法改變（見 What Changes）；Claude 與 Copilot 版不受這項修正影響。
  - 不涉及 openspec/config.yaml 或 .speclink.yaml 的欄位。
- 前置變更：add-copilot-tool 同樣修改技能渲染、golden 與 assets.lock，並新增 Copilot 渲染目標；它合併後本變更才改原稿。
- Affected specs: skill-authoring（新增）、discuss-skill（修改）、improve-skill（修改）
- Affected code:
  - New: crates/engine/speclink-core/tests/it/skill_authoring.rs
  - New: integrations/claude-code/skill-evals/.claude-plugin/plugin.json
  - New: integrations/claude-code/skill-evals/evals/（每題一個目錄，內含 prompt.md、case.yaml、scaffold.sh 與 graders；各題 scaffold 共用的 evals/fixtures.sh）
  - New: scripts/claude-code/skill-evals.mjs
  - New: scripts/claude-code/skill-evals.test.mjs
  - Modified: crates/engine/speclink-core/assets/skills/analyze.md
  - Modified: crates/engine/speclink-core/assets/skills/apply.md
  - Modified: crates/engine/speclink-core/assets/skills/apply-worktree-pre.md
  - Modified: crates/engine/speclink-core/assets/skills/apply-worktree-post.md
  - Modified: crates/engine/speclink-core/assets/skills/archive.md
  - Modified: crates/engine/speclink-core/assets/skills/audit.md
  - Modified: crates/engine/speclink-core/assets/skills/baseline.md
  - Modified: crates/engine/speclink-core/assets/skills/commit.md
  - Modified: crates/engine/speclink-core/assets/skills/config.md
  - Modified: crates/engine/speclink-core/assets/skills/discuss.md
  - Modified: crates/engine/speclink-core/assets/skills/drift.md
  - Modified: crates/engine/speclink-core/assets/skills/improve.md
  - Modified: crates/engine/speclink-core/assets/skills/ingest.md
  - Modified: crates/engine/speclink-core/assets/skills/manual.md
  - Modified: crates/engine/speclink-core/assets/skills/propose.md
  - Modified: crates/engine/speclink-core/assets/skills/quality.md
  - Modified: crates/engine/speclink-core/assets/skills/review.md
  - Modified: crates/engine/speclink-core/assets/skills/trace.md
  - Modified: crates/engine/speclink-core/assets/skills/verify.md
  - Modified: crates/engine/speclink-core/assets/skills/worktree-merge.md
  - Modified: crates/engine/speclink-core/src/workspace/init.rs
  - Modified: crates/engine/speclink-core/tests/it/main.rs
  - Modified: crates/engine/speclink-core/tests/it/render_golden.rs
  - Modified: crates/engine/speclink-core/tests/golden/assets.lock
  - Modified: crates/engine/speclink-core/tests/golden/claude.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/codex.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - Modified: crates/engine/speclink-core/tests/golden/copilot.snapshot.md
  - Modified: 本 repo 以 speclink update 重產的 Claude 與 Codex 技能檔（.claude/skills 與 .agents/skills 兩個目錄下）

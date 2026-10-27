## Why

透過 AI 代理跑 SDD 的開發者（以及替團隊訂標準的 PO／PM）用 `/speclink-config` 整理 openspec/config.yaml 時，技能的判準不夠用：技能沒有「一條規矩該放審查規則還是產出規則」的分流；審查規則的重複檢查只比對 12 條 code smell，沒擋掉已有測試或 lint 守門、或其實屬於 audit／verify 的規則；嚴重度依措辭建議，實際整理時兩個專案（speclink 與另一個專案）都是先定嚴重度、再回頭改措辭配合。另外，審查規則是選用的，現行流程卻預設要問要不要加規則；每次執行又把政策欄位逐題重問一輪，而 worktree 欄位從未被問過。

本變更是討論 config-skill-rule-routing 規劃的第 2 刀（最後一刀），只改 config 技能文字；第 1 刀 workflow-config-write-feedback（CLI 寫入回饋）已封存。前置的 review-rules-config 與 quality-gate-speedup 都已封存，本變更直接改寫正式規格。

## What Changes

- **審查規則改為選用，一題三選**：原本的第六問（review.baseline）與「要不要新增審查規則」合併成一題——不設定（預設）／附加（extend：12 條 code smell 照舊，再加專案規則）／覆蓋（replace：只用專案規則，清單可以為空＝只依 repo 文件）。
  - 不設定：不動 review 區塊；提醒不分流到審查規則、不建議搬移；只適合當審查規則的提醒，在最後的報告列為「屬審查規則，未寫入」。
  - 已有審查規則時題目帶現值，選不設定＝維持現狀；既有規則仍照判準 3、4 檢查（會過時的內容、失效的引用）。
- **規則分流**（選了附加或覆蓋才分流到審查規則）：每條提醒先問三題——違反的證據在程式還是文件、審查員只看 patch 能不能判斷、太晚發現的代價大不大——分到審查規則、某個 artifact 的產出規則、兩邊都放、或不收。
  - 兩邊都放時兩句描述不同對象（產出規則要求計畫寫出什麼，審查規則檢查程式本身），而且只在兩個階段各自有用時才兩邊放，因為產出規則每次產出該 artifact 都會注入。
  - 審查規則不得寫成「程式要照 artifact 做」，那是 verify 的工作。
  - 既有產出規則中其實在檢查程式的句子，技能建議拆一半到審查規則；搬移要使用者確認，被拒絕的建議下次執行會再提。
- **重複檢查多兩種不收**：已有測試或 lint 守門的規矩（用判準 4 同類的針對性靜態查核核實，不掃原始碼）；屬於其他關卡的規矩（安全類在 audit 開啟時歸 audit，對照 artifact 歸 verify）。audit 開關改變時，既有安全類審查規則的去留跟著變，經 diff 確認。
- **嚴重度改以「有沒有正當例外」判斷**：沿用既定分格意義（critical＝違反一定錯、沒有例外；warning＝通常要修、偶有正當例外；suggestion＝偏好），措辭降為提示。
- **時機**：規則依賴的行為還沒進程式（討論已定案但變更未落地）時先不寫入，並告訴使用者。
- **提醒清單當輸入來源**：技能明寫可以收使用者提供的提醒清單（貼上或給檔案路徑），每條走同一套分流；技能不自己去讀各工具的對話紀錄。
- **政策欄位一次確認**：先列一張表顯示所有政策欄位的現值（未設定的標「未設定，預設 X」），只問一題「全部沿用，還是要改其中幾項」，選要改才逐項問被點名的欄位；第一次執行同樣攤開每個值、改為一題確認。表中的欄位：locale、spec_locale、tdd、audit、worktree（新納入，現行技能從未問過）、任務驗證的測試範圍、審查規則的模式。
- 文件：docs/configuration.md 與 docs/configuration.zh-TW.md 描述 config 技能的段落，改寫成新的問法與選用的審查規則。

### 相容性影響

- 只改技能文字，不改任何 CLI 指令、旗標、`--json` 輸出或 config 欄位；config.yaml 的欄位與預設值不變（選「不設定」不產生新狀態：沒有 review 區塊＝引擎照舊讀成 extend 加三個空清單）。
- 生成的技能：speclink-config 內文改變，影響所有已啟用工具（claude、codex、copilot）的技能檔。引擎的技能版號（ASSET_VERSION）升一個 minor 版號，所以所有受管技能檔的版號行都會變；既有專案執行 `speclink update` 取得新版，沒更新的專案照舊版技能的問法運作，不會壞。
- 回歸對照：golden 快照與 assets.lock 同批刻意再生；render_golden 中 config 技能的第五問測試隨新結構同批改寫。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `config-skill`: 改寫「技能規定審查規則的第六問與規則接收」（一題三選、依正當例外判嚴重度、兩種新的不收）、「技能規定 diff 先行與收斂驗收」與「技能規定任務驗證測試範圍的第五問」的逐項詢問字面；新增「技能規定政策欄位一次確認現值」與「技能規定提醒的分流」兩條需求。

步驟 3 的規格掃描命中 config-skill（本變更的正典）、workflow-config（review 區塊與 set 寫入的正典，本變更只引用、不改它的需求）與 review-skill（審查站讀 review 區塊的方式不變，不改）。

## Impact

- Affected specs: config-skill
- Affected crates／apps：speclink-core（config 技能範本、技能版號、golden 與 assets.lock、render 測試）；speclink-cli、speclink-server、apps/desktop 不受影響
- 生成的技能：speclink-config，影響 claude、codex、copilot 三種工具；其餘技能只有版號行改變。本 repo 自身的技能檔由 `speclink update` 再生
- 設定欄位：無新增或變更（worktree 欄位本來就由 workflow-config set 接受，本變更只讓技能開始詢問它）
- Affected code:
  - New: (none)
  - Modified:
    - crates/engine/speclink-core/assets/skills/config.md
    - crates/engine/speclink-core/src/workspace/init.rs
    - crates/engine/speclink-core/tests/it/render_golden.rs
    - crates/engine/speclink-core/tests/golden/assets.lock
    - crates/engine/speclink-core/tests/golden/claude.snapshot.md
    - crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md
    - crates/engine/speclink-core/tests/golden/codex.snapshot.md
    - crates/engine/speclink-core/tests/golden/copilot.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md
    - crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md
    - docs/configuration.md
    - docs/configuration.zh-TW.md
  - Removed: (none)

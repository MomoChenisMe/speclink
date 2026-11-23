## Why

內嵌引擎的宿主用 `skills.render` 產生給自己 Agent 的技能文字，用 `agentTool.description` 當 `speclink` 工具的說明。兩者都不能依宿主的實際情況調整：

- 宿主只提供六支技能（discuss、trace、propose、ingest、analyze、config），但產生的文字會指到宿主沒有的技能（propose、ingest、analyze 指到 `speclink-apply`，config 要讀 `speclink-review` 的技能檔）。
- 使用宿主 Agent 的人沒有程式碼的 checkout，也沒有 git 歷史，但 discuss 的文字要模型用 Grep／Glob 找程式碼，trace 要模型跑 `git log`。
- `skills.render` 的選項多給或打錯鍵名時不報錯也不生效，宿主無從知道選項沒有被採用。
- 工具說明約 7,100 個字元，列出引擎全部的動詞，包含宿主的掛鉤一律拒絕的動詞。

目標使用者是內嵌 `@speclink/engine`、把技能與工具交給自己 Agent 框架的宿主開發者；最終受益的是在宿主的對話裡討論需求與提案的 PO 與 PM。對應的 workflow 階段是 discuss、propose、ingest、analyze、trace、config 六支技能在沒有 checkout 的環境裡執行。

這是討論 embedded-host-needs-7 五個變更裡的第 3 個（G3）。討論決定先做小的：只在技能開頭的前言加標準說明，不改技能內文，因為目前沒有實際觀察到模型做錯。

## What Changes

- `skills.render` 的 options 新增 `skills`（字串陣列，宿主提供的技能名稱）：渲染結果在前言多一段，說明這個環境只提供清單裡的技能、清單外的技能不要載入也不要建議使用者執行。不認得的技能名稱、空陣列、或清單不含正在渲染的那一支技能時拋出錯誤。
- `skills.render` 的 options 新增 `checkout`（布林，預設 true）：為 false 時前言多一段，說明這個環境沒有程式碼的 checkout 與 git 歷史、跳過讀程式碼與讀 git 的步驟、事實只用 speclink 的動詞查證。`checkout` 為 false 時 `specInCheckout` 省略視同 false；明確給 true 則拋出錯誤。
- `skills.render` 的 options 含不認得的鍵時拋出錯誤，訊息指出那個鍵並列出合法的鍵。
- 新增匯出 `agentToolDescription(options)`：產生 `speclink` 工具的說明，`options.exclude` 列出不要出現的動詞（例如 `archive`、`review`、`discuss archive`）；`review` 會排除它底下的全部動詞。不帶 options 時回傳值等於 `agentTool.description`。清單裡沒有的動詞拋出錯誤。
- 不改的部分：`agentTool` 的三個欄位、技能原稿（assets）的內文、`skills.list()`、CLI 的 `speclink init`／`speclink update` 產生的檔案、入口的 `POST /skills`。不新增 CLI 指令與設定欄位。

影響的技能與工具：兩個新選項對每一支技能、每一個 target 都可用，但只有呼叫端明確給了才會多出前言。CLI 為 claude、codex、copilot、pi、cursor 與自訂工具產生的技能檔逐位元不變，內嵌資產的版本不變。

三種形態下的結果：

| 項目 | 本機 repo | 連到遠端的 checkout | 沒有本機規格檔的內嵌宿主 |
| --- | --- | --- | --- |
| `skills` 與 `checkout` 選項 | 不適用：CLI 產生全部技能，而且一定有 checkout；SDK 的呼叫端要用也可以 | 同左 | 可用，這是它們的用途 |
| 不認得的選項鍵報錯 | SDK 的呼叫端適用 | 同左 | 可用 |
| `agentToolDescription` | 不適用：工具定義只給內嵌宿主的 Agent 框架 | 同左 | 可用 |

相容性影響：

- **BREAKING（只影響傳了多餘選項的呼叫端）**：之前 `skills.render` 默默忽略不認得的鍵，現在拋出錯誤。來源文件記錄過宿主多給 `skills: ['discuss']` 而沒有作用；這個鍵現在是合法的，而且會生效。
- 不帶新選項時，`skills.render` 的回傳字串逐位元不變；`agentTool` 不變。
- 人眼輸出與 `--json` 輸出不變；線上格式不變；golden 不受影響。

掃描過的相關規格（沒有列進下面清單的）：skill-authoring 的寫法規範與條件區塊只管技能原稿，這次不動原稿；workspace-tools 與 skill-routing 規定 CLI 產生技能檔的位置與交棒，這次不變；host-kit「入口提供這個範圍的技能」的回應不變。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `node-sdk`: 新增「渲染 API 的宿主限制選項」、「渲染 API 拒絕不認得的選項鍵」與「Agent 工具說明的動詞篩選」三條需求

## Impact

- Affected specs: `node-sdk`
- Affected code:
  - Modified:
    - crates/engine/speclink-core/src/workspace/skills.rs
    - crates/adapters/speclink-verbs/src/tool.rs
    - crates/adapters/speclink-node/src/render.rs
    - crates/adapters/speclink-node/src/host.rs
    - crates/adapters/speclink-node/index.js
    - crates/adapters/speclink-node/index.d.ts
    - crates/adapters/speclink-node/binding.js
    - crates/adapters/speclink-node/binding.d.ts
    - crates/adapters/speclink-node/__test__/render.spec.ts
    - crates/adapters/speclink-node/__test__/agent-tool.spec.ts
    - docs/sdk-node.md
    - docs/sdk-node.zh-TW.md
  - New: （無）
  - Removed: （無）

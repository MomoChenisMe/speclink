# propose-skill Specification

## Purpose

/speclink-propose 技能的輸入來源處理：--from-discussion 時跟隨討論的 Source doc 引用與疊加語意、--from-doc 以文件為直接輸入的入口，以及起草任務時替需使用者親手操作的項目標上手動任務標記。本 capability 保證提案能完整承接討論或文件裡已經談定的內容，不必使用者重述一遍。

## Requirements

### Requirement: from-discussion 跟隨 Source doc 引用與疊加語意

內嵌 speclink-propose 技能（事實來源 crates/speclink-core/assets/skills/propose.md，經 init 與 update 渲染至 claude 與 codex 工具技能目錄）SHALL 規定：以 --from-discussion 讀取討論記錄時，若記錄的 Context 含 Source doc: <路徑> 行，代理人 SHALL 讀取該原始文件，並以疊加語意合成提案內容——文件為底層、討論為勝出層：討論有決定的 SHALL 以討論為準；討論未觸及的文件內容 SHALL 補位採用；討論記錄 Ruled out 的內容 SHALL NOT 出現在提案中。本能力屬 Speclink 自身延伸；渲染產物內容由 speclink-core 的 render_golden 測試（cargo test）保護，golden 快照更新屬刻意變更。

#### Scenario: 渲染產物含疊加語意三規則

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** 產出的 speclink-propose 技能檔 SHALL 含疊加語意三規則：討論決定優先、討論未觸及者以文件補位、Ruled out 內容不得復活於提案

#### Scenario: Ruled out 內容不復活

- **WHEN** 原始文件主張作法 X，而討論記錄的 Ruled out 否決了 X 並於 Conclusion 決定作法 Y
- **THEN** 技能檔 SHALL 規定產出的提案採 Y，且 X SHALL NOT 以任何形式出現於提案內容

#### Scenario: 記錄無 Source doc 行時照舊

- **WHEN** 討論記錄的 Context 無 Source doc 行
- **THEN** 技能檔 SHALL 規定 from-discussion 流程與現行相同，無額外文件讀取步驟


<!-- @trace
source: discuss-propose-from-docs
updated: 2026-07-31
code:
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/assets/skills/propose.md
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - docs/workflow.md
  - docs/workflow.zh-TW.md
-->

---
### Requirement: from-doc 直接文件入口

技能檔 SHALL 規定 propose 認得 --from-doc <路徑> 引數慣例：使用者以該引數指定文件時，代理人 SHALL 讀取該文件作為需求來源建立提案，無需既存討論。requirement source 優先序 SHALL 為：明確需求描述引數 → --from-doc 指定文件 → 討論記錄 → plan 檔偵測 → 對話上下文；既有 plan 檔偵測（plan mode 對話觸發、限 plan 檔目錄）SHALL 保留並存。技能檔並 SHALL 規定出處記錄：以 --from-doc 建立的提案，其 proposal 的 Why 或 Impact 段 SHALL 含一行 Source doc: <路徑>，留存來源文件出處。本慣例與出處記錄均屬技能文字約定，SHALL NOT 要求引擎新增旗標或改變任何 CLI 語法。

#### Scenario: 渲染產物含 from-doc 入口與優先序

- **WHEN** 檢視渲染產出的 speclink-propose 技能檔的需求來源段落
- **THEN** 技能檔 SHALL 含 --from-doc <路徑> 慣例與五級優先序（明確引數、--from-doc、討論記錄、plan 檔偵測、對話上下文），且既有 plan 檔偵測段落保留

#### Scenario: from-doc 提案留存出處

- **WHEN** 檢視渲染產出的 speclink-propose 技能檔的 --from-doc 段落
- **THEN** 技能檔 SHALL 規定以 --from-doc 建立的提案於 proposal 的 Why 或 Impact 段含一行 Source doc: <路徑>，留存來源文件出處

#### Scenario: 未給 from-doc 時行為照舊

- **WHEN** 使用者未帶 --from-doc 引數
- **THEN** 技能檔 SHALL 規定需求來源判定與現行相同，無額外文件讀取步驟

<!-- @trace
source: discuss-propose-from-docs
updated: 2026-07-31
code:
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/assets/skills/propose.md
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - docs/workflow.md
  - docs/workflow.zh-TW.md
-->

---
### Requirement: 手動任務的起草標記

propose 技能文字 SHALL 指示 tasks 起草時對手動任務(agent 無法代行、需使用者親手操作的任務——人工驗收、建立外部服務帳號、放置金鑰等,不限於測試)加 `[M]` 前綴標記,使寫碼與人工操作的完成度得以分開判讀;agent 做得到的任務(寫碼與自動化測試)SHALL NOT 加此標記。指引 SHALL 以正誤例並列(對比對)呈現:正例(`[M]` 緊接 checkbox、編號在後)與誤例(編號在前)並列,附後果說明(引擎不認、任務被算成寫碼任務而卡住完成度)與「checkbox 後恰一個空格」規則,SHALL NOT 僅給正例。技能模板 SHALL 於 claude 與 codex 兩工具正典化生成,golden 對照涵蓋。

#### Scenario: 起草含人工操作的 tasks

- **WHEN** 提案含「開啟文件實際操作確認」類的人工驗收項與「至外部服務建立帳號」類的人工前置項,propose 流程產出 tasks.md
- **THEN** 該等任務行皆帶 `[M]` 前綴且緊接 checkbox(編號在標記之後),agent 可自行執行的寫碼與自動化測試行不帶

#### Scenario: 對比對指引呈現

- **WHEN** 閱讀 propose 技能檔的 `[M]` 起草指引
- **THEN** 正例與誤例並列可見,並載明誤寫後果與 checkbox 後恰一個空格的規則

#### Scenario: 技能模板生成

- **WHEN** 執行 speclink update
- **THEN** claude 與 codex 的 propose 技能檔含 `[M]` 對比對起草指引,與 golden 對照一致

<!-- @trace
source: manual-marker-scope-beyond-tests
updated: 2026-08-14
-->

---
### Requirement: 收尾盤點提案中變更的執行順序

技能檔 SHALL 規定：propose 完成、給出下一步建議之前，代理人 SHALL 以 list 動詞的 JSON 輸出列出變更名，並以各變更 metadata（.openspec.yaml）的 started_* 標記缺席判定提案中（未開工）——list 輸出本身分不出已開工與未開工，SHALL NOT 以其 status 或任務數判定。提案中數量 ≥2 時 SHALL 展開執行順序判定——硬信號為 delta capability 重疊（兩個變更的 delta 目錄含同一 capability 即判須依序：delta 重寫同一份正典規格，亂序封存可能觸發合併閘拒絕），軟信號為讀 proposal 與 tasks 推測的程式碼重疊或依賴；僅 1 個時 SHALL 維持既有出邊、SHALL NOT 展開盤點段。有效 worktree 政策（含 SPECLINK_WORKTREE 環境覆寫層）開啟時 SHALL 分「可平行——各開一個 session 以 apply-with-worktree 執行，沿用多 session 配方」與「須依序」兩組呈現；政策關閉時 SHALL 給單一建議順序。盤點為僅建議：SHALL NOT 自動呼叫任何技能，SHALL NOT 依賴引擎新增指令。

#### Scenario: 多提案且 worktree 開啟時分組

- **WHEN** propose 完成、提案中變更 ≥2 且有效 worktree 政策為開啟
- **THEN** 技能檔指示列出全部提案中變更，以 delta capability 重疊與內容推測判定順序，並分「可平行（各開 session 走 apply-with-worktree）」與「須依序」兩組呈現

#### Scenario: 多提案且 worktree 關閉時給單一順序

- **WHEN** propose 完成、提案中變更 ≥2 且有效 worktree 政策為關閉
- **THEN** 技能檔指示給出單一建議執行順序，不出現平行分組

#### Scenario: 單一提案不盤點

- **WHEN** propose 完成且提案中變更僅 1 個
- **THEN** 技能檔維持既有下一步建議，不展開盤點段

##### Example: delta capability 重疊判序

| 變更 A 的 delta 目錄 | 變更 B 的 delta 目錄 | 判定 |
| --- | --- | --- |
| specs/board-card-order/ | specs/board-card-order/ 與 specs/tray-status-menu/ | 須依序（board-card-order 重疊） |
| specs/discuss-skill/ | specs/archive-skill/ | 可平行（無重疊） |

<!-- @trace
source: propose-apply-handoff-updates
updated: 2026-08-27
-->
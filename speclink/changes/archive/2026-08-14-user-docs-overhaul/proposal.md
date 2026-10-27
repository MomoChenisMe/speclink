## Summary

重寫全部面向使用者的文件（docs/ 扣除兩份架構文件）與中英 README，改以「Speclink 是什麼、看得到畫面、照著做得完」為主軸，並首度加入 desktop 與 server 後台的截圖。

## Motivation

現行文件是隨能力逐次增修堆疊出來的，對首次接觸的人有三個缺口：

- **沒有畫面**：Speclink 的核心價值是變更看板、規格與討論的視覺化管理，但整份文件零截圖，讀者無法在安裝前知道自己會得到什麼。
- **工作流說明分散**：SDD 十一個站（onboard／discuss／improve／propose／apply／ingest／quality／review／verify／archive／worktree）散落在 README 摘要與 workflow 文件，沒有一處把「站 → 對應技能 → 完成判準」講完整。
- **事實已過時**：product-status 的查核日停在 2026-07-17，且把 `docs/verb-contract.md` 記為不存在的缺口，但該文件現已存在；本地與遠端能力的對照也沒有單一入口。

另外，使用者面的「未來會有什麼」目前只能從內部的實作重構路線圖推敲，那份文件是給維護者看的交付順序，不適合當作對外承諾。

## Proposed Solution

**截圖基建**：新增一支腳本，備份使用者的 desktop app 狀態目錄、以 speclink init 建出一個乾淨的示範 workspace（含數張處於不同欄位的示範變更與一則討論）、指向該 workspace 啟動 app，截圖完成後還原原狀態。截圖以人工擷取（版面與時機無法可靠自動化），腳本只負責可重現的乾淨場景與安全的備份還原。

**文件重寫**：以漸進揭露重排——README 是入口與定位、getting-started 是第一輪、workflow 是工作流正典、其餘為參考文件。每份文件維持單一責任與中英對等，並符合 GitHub 上的 markdown 呈現（表格、程式碼區塊語言標註、相對路徑連結、圖片以相對路徑內嵌）。

**新增使用者面路線圖**：獨立於內部實作路線圖，只寫對使用者有意義的方向（Node SDK 發布、Copilot／MCP 等 agent 工具整合、系統整合能力），並明示為方向而非日期承諾。

**能力對照**：本地與遠端兩條路徑的能力差異集中成一張對照表，取代目前散落各文件的零星說明。

## Non-Goals

- 不改寫 `docs/platform-architecture.zh-TW.md` 與 `docs/implementation-refactor-roadmap.zh-TW.md`——兩者是維護者導向的目標架構與交付順序正典，本次維持原狀，僅校正指向它們的連結文字。
- 不自動化截圖擷取本身：版面穩定度與時機判斷不可靠，強行自動化會產出品質不一的圖片。
- 不新增或改動任何產品功能、CLI 動詞、GUI 行為——本次只動文件與截圖資產。
- 不建立文件網站（靜態網站產生器、GitHub Pages）：本次目標是 repo 內直接可讀。
- 不翻譯品牌資產說明文件。

## Alternatives Considered

- **只補截圖、不重寫文字**：截圖會落進與現行敘事不合的段落，且工作流分散與事實過時的問題仍在。
- **以錄影或 GIF 取代靜態截圖**：檔案大、在 GitHub 上不易掃讀，且每次介面微調都要重錄。
- **把使用者面路線圖併入現有實作路線圖**：兩者受眾與承諾強度不同，混寫會讓對外方向讀起來像內部排程。

## Impact

- Affected specs: `user-documentation`
- Affected code:
  - New: `scripts/docs-screenshots.mjs`、`scripts/docs-screenshots.test.mjs`、`docs/roadmap.zh-TW.md`、`docs/roadmap.md`、`docs/assets/screenshots/`（desktop 與 server 後台截圖）
  - Modified: `README.md`、`README.en.md`、`docs/getting-started.zh-TW.md`、`docs/getting-started.md`、`docs/workflow.zh-TW.md`、`docs/workflow.md`、`docs/configuration.zh-TW.md`、`docs/configuration.md`、`docs/development.zh-TW.md`、`docs/development.md`、`docs/product-status.zh-TW.md`、`docs/product-status.md`、`docs/remote-getting-started.zh-TW.md`、`docs/remote-getting-started.md`、`docs/sdk-node.zh-TW.md`、`docs/sdk-node.md`、`docs/verb-contract.zh-TW.md`、`docs/verb-contract.md`、`docs/server-deployment.zh-TW.md`、`docs/server-store-drivers.zh-TW.md`、`docs/server-backup.zh-TW.md`
  - Removed: 無
- 排序前提：本 change 與進行中的 `release-signing-and-channels` 在 README、getting-started、sdk-node、product-status 四處重疊。該 change 的文件任務只補安裝區塊與發布狀態，屬 v0.1.0 發版前置；本 change SHALL 在其合併後才開始文件改寫，避免同檔對撞。

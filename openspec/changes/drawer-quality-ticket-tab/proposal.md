## Why

桌面 app 的變更詳情面板在審查／驗證進行中只顯示「審查中」「驗證中」章籤，使用者（透過 AI 代理跑 `/speclink-review`、`/speclink-verify`、`/speclink-quality` 的開發者與 PO）看不到工單裡目前卡在哪些 findings，得回終端機或開檔案才知道。工單（`review.md`／`verify.md`）是引擎擁有、只增不改的文件，蓋章或放棄時會被刪除；面板要在工單存在時給一個唯讀的結構化檢視，工單消失時檢視也跟著消失。來源討論：drawer-quality-ticket-tab。

## What Changes

- 變更詳情面板（`packages/ui` 的 `RichDetailDrawer`）於「排程」分頁之後新增兩個條件式分頁「審查」「驗證」：清單項 `reviewStatus` 為 `inReview` 時出現「審查」分頁，`verifyStatus` 為 `inVerify` 時出現「驗證」分頁；皆非時分頁列維持原五個分頁。分頁名與狀態列章籤的站名一致。
- 已封存詳情面板（`ArchivedDrawer`）同法新增兩個條件式分頁：`reviewStatus` 為 `reviewedNotPassed`（帶走的化石審查工單）出「審查」、`verifyStatus` 為 `verifiedNotPassed` 出「驗證」。
- 分頁內容為結構化渲染的工單（新元件 `TicketView`）：段標題列站名、目前輪數、末輪三級計數（CRITICAL／WARNING／SUGGESTION）；輪次可收合，末輪展開、舊輪收合；每輪標階段——`discovery` 顯示「首輪」、`validation` 顯示「複驗」、legacy 輪（無 Phase）不標——與「範圍 N 檔」（展開列出檔案路徑）；finding 一列一條：嚴重度色章（CRITICAL 紅、WARNING 琥珀、SUGGESTION 灰）、路徑等寬字、描述原文不翻譯；描述行尾的結構 token `(accepted)` 不直出，改以「已接受」籤呈現。
- 載入三態沿用「抽屜文件載入以 skeleton 呈現」：載入中 skeleton、載入完成無工單顯示分頁空態、有內容渲染。使用者停在該分頁時工單被刪（蓋章／放棄）：狀態隨看板刷新翻為非進行中，分頁消失，選中分頁退回「提案」。
- 資料來源介面（`packages/ui/src/adapter.ts` 的 `DataSource`）新增 `getStationTicket(change, station)` 與 `getArchivedStationTicket(datedName, station)`，回傳結構化工單 `{ rounds: [{ index, phase, patchHash, scope, findings: [{ severity, path, text }] }] }` 或 `null`（無工單）。形狀與 CLI `speclink review show --json` 的 `rounds` 及 server `GET /changes/{name}/review` 的 `rounds` 同構。
- `apps/desktop/core` 新增查詢：本機活工單（經 change 的 store context 讀 `review.md`／`verify.md`，含 worktree 覆蓋層）與本機封存工單（`read_archived_artifact`）各解析成上述 JSON；`apps/desktop/src-tauri` 新增四個 Tauri 指令——本機活、本機封存、遠端活（走 `speclink-remote` 既有的 `station_ticket_if_any`，直接取 `rounds`）、遠端封存（走既有 `archived_artifact` 取原文後以引擎解析）。不放寬引擎 `artifact cat` 的 artifact 名白名單。
- `speclink-core` 的 `quality::station::parse_ticket` 由私有升為 `pub`：純函式「站別＋工單文字 → `Ticket`」，供 desktop core 對封存原文與遠端封存原文解析。引擎行為不變，只是曝露既有解析。
- `openspec/LANGUAGE.md` 新增詞條「首輪」（discovery 輪）與「複驗」（validation 輪），avoid「驗證輪」（與驗證站撞名）、「發現輪」；並於「換頁」詞條補註面板 tab 稱「分頁」不稱「頁籤」。
- i18n（`packages/ui/src/i18n.tsx`）新增 tw／en 詞條：分頁標籤、階段詞、「範圍 N 檔」、「已接受」、「第 N 輪」、分頁空態。

相容性影響：無 CLI 指令新增或改動，人眼輸出與 `--json` 皆不變；Tauri 指令為純新增，既有指令簽名不變；`DataSource` 介面新增兩個方法，`apps/desktop` 的 local 與 remote 兩個資料源皆實作。無設定欄位、無技能或 Agent 指令異動。

## Capabilities

### New Capabilities

（無）。步驟 3 掃描到的相關規格：`desktop-app`（詳情抽屜的審查／驗證資訊列、已封存側標示、排程分頁、skeleton 載入）已涵蓋面板與章籤，工單內容檢視是它的新增需求而非新 capability；`review-station`／`verify-station` 規定工單的建立、蓋章刪檔與封存帶走，本變更只讀不寫，不改其需求；`client-protocol` 的 `GET /changes/{name}/review`／`verify` 回應形狀已含 `rounds`，本變更直接消費，不改 wire。

### Modified Capabilities

- `desktop-app`：新增「詳情抽屜的工單分頁」與「已封存抽屜的工單分頁」兩條需求（條件式分頁、結構化渲染、階段文案、載入三態、工單消失時退場）。

## Impact

- Affected specs: `desktop-app`（modified）
- Affected code:
  - New:
    - packages/ui/src/components/TicketView.tsx（結構化工單元件：段標題、可收合輪次、finding 列）
    - packages/ui/src/__tests__/ticketView.test.tsx
  - Modified:
    - crates/engine/speclink-core/src/quality/station.rs（`parse_ticket` 升為 pub）
    - apps/desktop/core/src/query.rs（本機活工單與封存工單查詢、`Ticket` → JSON）
    - apps/desktop/src-tauri/src/lib.rs（四個 Tauri 指令與註冊）
    - apps/desktop/src/adapter/tauriDataSource.ts（local 資料源實作兩個方法）
    - apps/desktop/src/adapter/remoteDataSource.ts（remote 資料源實作兩個方法）
    - apps/desktop/src/App.tsx（把兩個載入器接進兩個面板）
    - apps/desktop/src/__tests__/App.test.tsx（資料源假件補兩個方法）
    - packages/ui/src/adapter.ts（`DataSource` 介面與 `StationTicket` 型別）
    - packages/ui/src/components/RichDetailDrawer.tsx（條件式「審查」「驗證」分頁、載入與退場）
    - packages/ui/src/components/ArchivedDrawer.tsx（條件式分頁）
    - packages/ui/src/index.ts（匯出 `TicketView` 與型別）
    - packages/ui/src/i18n.tsx（tw／en 詞條）
    - packages/ui/src/__tests__/richDrawer.test.tsx（分頁出現／消失／退場）
    - packages/ui/src/__tests__/archivedDrawer.test.tsx（封存側分頁）
    - openspec/LANGUAGE.md（「首輪」「複驗」詞條、「換頁」詞條補註）
  - Removed: （無）

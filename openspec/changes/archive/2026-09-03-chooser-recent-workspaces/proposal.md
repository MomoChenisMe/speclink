## Why

桌面 app 目前只記「正在開著的分頁」：分頁一關，那個 workspace 的路徑就從 app 的記憶裡消失，下次要再開得重新走一次資料夾選擇器或 server 的 scope 清單。目標使用者是透過 AI 代理跑 SDD 的開發者，情境是在桌面 app 切換專案的時刻（propose／apply 之前先開對 workspace）；他們要的是 VS Code 式的「最近開啟」：分頁關掉可以忘，但開過的 workspace 要記得。2026-09-02 討論「chooser-recent-workspaces」裁定推翻 2026-07-06 討論「專案選擇對齊-spectra」的「分頁列即最近清單、不另設最近選單」決定，本變更落實該裁決。

## What Changes

- 「新增 Workspace」chooser 第一步的兩張來源卡（本機資料夾／Speclink Server）下方新增「最近開啟」清單：列出曾經成功開啟過、但目前不在分頁列上的 workspace。
- 記錄面：apps/desktop 新增一個純函式模組，以 localStorage 獨立新鍵持久化最近開啟清單（條目為 WorkspaceLocator＋顯示名；最新在前、同 locator key 去重、上限 20）。每次本機或 remote workspace 成功開啟（含分頁點擊、chooser、remote marker 探測、本機轉 remote）即記入；關閉分頁不動這份記錄。
- 顯示面：清單在 chooser 顯示時以 locator key 過濾掉目前分頁列上已開著的 workspace；過濾後為空則整個「最近開啟」區段不顯示。本機項目顯示資料夾名稱與路徑，remote 項目顯示 server 名稱與 workspace 顯示名。
- 開啟面：點本機項目先經既有 openProject 探測再沿既有本機開啟流程（未初始化資料夾仍走 init 確認、帶 remote marker 的資料夾仍走既有分流）；點 remote 項目沿既有 remote 開啟流程。
- 失效面：本機資料夾不存在、remote 連線已自連線登錄移除、或 remote handshake 失敗時，該項目轉錯誤態並顯示原因，點擊不再開啟。每筆項目滑過顯示移除鈕，可自清單移除（錯誤態項目同）。
- 升級補種：升級後第一次啟動、localStorage 尚無最近開啟鍵時，以既有分頁列的條目補種清單，避免清單一開始是空的。
- 正典措辭：desktop-config 的「專案分頁列存於 app 本機」需求改寫為「分頁列呈現目前開著的專案」，並明示關閉分頁不影響最近開啟清單；「關閉分頁即自分頁持久化清單移除」與其餘分頁行為不變。
- 影響範圍只在 apps/desktop 前端（React／zustand／localStorage）。不新增 Tauri command，不動任何 Rust crate，不動 CLI 指令與其輸出，不動 openspec/config.yaml 與 .speclink.yaml 欄位，不動生成的技能與 Agent 指令（claude／codex 均無影響）。

## Capabilities

### New Capabilities

（無——規格掃描命中 workspace-chooser（新增 Workspace 的來源分流）、workspace-session（WorkspaceLocator 身分與分頁持久化 v2）與 desktop-config（專案分頁列存於 app 本機）三份既有規格；最近開啟清單是 chooser 第一步的新需求，落在 workspace-chooser 之下，不另立 capability。workspace-session 的 locator 身分與分頁持久化 v2 格式本變更均不改動。）

### Modified Capabilities

- `workspace-chooser`: 新增需求「最近開啟清單」——記錄規則（成功開啟即記入、去重、上限 20、關閉分頁不影響）、顯示過濾（已開著的分頁不顯示、空清單不顯示區段）、開啟路徑（沿既有本機／remote 流程）、失效錯誤態與移除、升級補種。
- `desktop-config`: 需求「專案分頁列存於 app 本機」措辭調整——分頁列呈現的是「目前開著的專案」而非「開啟過的專案」，關閉分頁自分頁持久化清單移除但不影響最近開啟清單；五個既有 scenario 全數保留、行為不變。

## Impact

- Affected specs: `workspace-chooser`（ADDED 需求）、`desktop-config`（MODIFIED 需求）。
- Affected code:
  - New: `apps/desktop/src/recents.ts`（最近開啟清單的純函式模組：upsert／remove／persist／read／顯示過濾）、`apps/desktop/src/__tests__/recents.test.ts`
  - Modified: `apps/desktop/src/store.ts`（最近開啟狀態、成功開啟尾聲記入、移除動作、啟動讀取與補種）、`apps/desktop/src/components/WorkspaceChooser.tsx`（第一步的最近開啟區段）、`apps/desktop/src/App.tsx`（chooser 的最近開啟 props 接線）、`apps/desktop/src/tabs.ts`（檔頭「分頁列即最近開啟清單」註解改寫、匯出 locator 形狀驗證供 recents 共用）、`apps/desktop/src/i18n/messages.ts`（zh-TW／en 新增最近開啟文案）、`apps/desktop/src/__tests__/workspaceChooser.test.tsx`、`apps/desktop/src/__tests__/store.test.ts`、`apps/desktop/src/__tests__/tabs.test.ts`
  - Removed: 無
- 相容性影響：無 CLI 人眼或 `--json` 輸出變動；localStorage 新增一個鍵，既有分頁鍵 `speclink.projectTabs` 的格式與內容不變，舊版使用者升級後分頁列照常還原，最近開啟清單自分頁補種。
- 不影響：Rust crate、Tauri command、server、CLI、設定欄位、生成技能。

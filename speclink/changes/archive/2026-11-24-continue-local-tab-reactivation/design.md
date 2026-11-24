## Context

Desktop 對「本機規格資料夾與 `.speclink.yaml` remote 區段並存」的資料夾，在第一次開啟時讓使用者三選一。選「繼續本機」後，Desktop 建立本機分頁（locator 為 `local:{root}`），讀取一律經 `speclink-desktop-core` 的 `init_core_context`，它只用 `Workspace::discover` 加 `FsStore`，不看 remote 區段，所以本機看板本身讀得到資料。

問題在於本機分頁之後的兩個檢查點都重新探測，並且只認「純本機專案」：

- 前端 `activateTab` 的本機分支：探測結果不是 `project` 就寫入 `store.tabInvalid`。切回分頁、Ctrl+數字、系統匣切換、app 啟動時的作用中分頁都走這裡。
- `speclink-desktop-core` 的 `project_stats_at`：只接受 `ProjectProbe::Project` 且專案根等於路徑。app 啟動時，背景本機分頁經 Tauri command `project_stats` 走這裡，失敗就寫入分頁錯誤。
- 前端 `requestMigration` 原本列為第三個檢查點（範圍調整，品質關卡第一輪後）。查證後它唯一的呼叫端是新增專案選擇器的「遷移到 Server…」按鈕，而那顆按鈕只在探測結果是 `project` 時出現；並存資料夾走不到它，專案設定也沒有這個入口。所以它不在這次的修正範圍，維持只接受 `project`。並存資料夾要遷移時，走三出口對話框的「遷移本機內容」（`migrateLocalFromConflict`）。

探測本身（`open_project_at`）對並存資料夾回 `RemoteBinding { has_local_openspec: true }`，這與 CLI「並存時 remote 生效」的模式解析一致，不改。

## Goals / Non-Goals

**Goals:**

- 「繼續本機」開出的本機分頁，在切回、app 重開（作用中或背景）時都以本機開啟，不轉錯誤態，也不再跳三出口對話框。
- 分頁路徑不是並存專案根時（子目錄或巢狀專案），切回照舊轉錯誤態，不開出上層專案的內容。
- 修法只動這兩個檢查點，探測結果的形狀與 IPC 契約不變。

**Non-Goals:**

- 不在本機分頁加「終端機裡的 speclink 會走遠端」提示：三出口對話框已寫明 remote marker 保持不動（來源討論已排除）。
- 不記住使用者的選擇：沒有設定檔欄位、沒有 localStorage 鍵。本機分頁存在本身就是選擇的紀錄。
- 不改明確的開啟動作：經「新增專案」、最近開啟清單、新增專案選擇器開啟並存資料夾，照舊停在三出口對話框。
- 不改首次啟動時以啟動目錄自動開專案的行為：沒有分頁就沒有先前的選擇，並存的啟動目錄照舊不自動開。
- 不改「只有 remote 區段、沒有本機規格資料夾」的本機分頁：照舊轉錯誤態。這類分頁要不要提供「改用遠端開啟」是另一件事。
- 不處理圖示列對沒點過的遠端分頁的連線狀態：那是來源討論的第二刀。
- 不改 `requestMigration`，也不新增從專案設定遷移的入口：Desktop 的專案設定沒有「遷移到 Server…」，並存資料夾要遷移時走三出口對話框的「遷移本機內容」。
- 不改 `continueLocalFromConflict` 記錄的路徑：從並存專案的子目錄選「繼續本機」時，分頁照舊記成那個子目錄，之後切回或重開會一致地轉錯誤態。

## Decisions

### 本機分頁切回時把並存視為本機專案

`activateTab` 的本機分支新增一種可接受結果：`status === "remoteBinding"` 且 `hasLocalOpenspec === true`。這時以分頁 locator 的 `root` 與分頁的 `name` 呼叫既有的 `enterProject`，與 `continueLocalFromConflict` 第一次開啟用的是同一組值（對話框的 `path` 與它的目錄名）。探測結果不帶根，所以進入前先以 `projectStats` 確認分頁路徑就是專案根，與背景啟動檢查是同一道比對；不是根時（分頁路徑是並存專案的子目錄，或巢狀專案的規格資料夾已刪而上層是並存資料夾）照舊寫入 `store.tabInvalid`，不開出上層專案的內容。

替代方案：

- 每次切回都重跳三出口對話框 — 每切一次分頁就被強制選一次；來源討論已排除。
- 把選擇記在設定或 localStorage — 多一份要同步的狀態；本機分頁的 locator 已經說明使用者選了本機。

### 判斷依據沿用探測結果，不新增探測狀態

前端只讀既有的 `hasLocalOpenspec` 旗標，不在 `ProjectProbe` 新增「本機加 marker」之類的狀態。新增狀態會改 `open_project` 的序列化形狀，連帶影響 `openProjectAt` 的分流與它的測試，而這次的需求用既有旗標就能判斷。

### 背景啟動檢查接受並存且根等於路徑的資料夾

`project_stats_at` 在現有的「`Project` 且根等於路徑」之外，再接受「探測為 `RemoteBinding` 且 `has_local_openspec` 為 true，而且 `Workspace::discover(path)` 命中的專案根等於 `path`」。根等於路徑的檢查沿用既有理由：規格資料夾刪掉但目錄還在時，向上探索會命中祖先專案，不得回報別人的數字。

通過檢查後照常以 `list_changes_at` 與 `list_discussions_at` 計算待收尾數，這兩者經 `init_core_context` 讀本機檔案。

替代方案：給 `RemoteBinding` 加一個 `root` 欄位讓 `project_stats_at` 直接比對 — 會改 IPC payload 與前端型別，只為一個內部檢查不值得。

## Implementation Contract

- **行為**：
  - 本機分頁對應的資料夾同時有本機規格資料夾與 remote 區段時，切回這個分頁會開出本機看板，`tabErrors` 沒有它的鍵，`pendingRemoteConflict` 維持 `null`，不發出任何 remote handshake。
  - app 重開時，這個分頁不論是上次作用中或在背景，都沒有分頁錯誤；背景時 `project_stats` 回 `{ pendingWrapUp: <number> }`。
- **介面與資料形狀**：`ProjectProbe`（Rust 與 TypeScript 兩側）、Tauri command `open_project` 與 `project_stats` 的回應形狀都不變。`project_stats_at` 與 `activateTab` 的簽名不變；`requestMigration` 的行為與簽名都不變。
- **失敗模式**：
  - 本機分頁探測為 `remoteBinding` 且 `hasLocalOpenspec` 為 false → `activateTab` 照舊寫入 `store.tabInvalid`。
  - 探測為 `uninitialized` 或 `unadopted` → 同上，行為不變。
  - 探測丟出錯誤（路徑不存在、壞 `.speclink.yaml`、remote 區段缺 url）→ 照舊把錯誤原文寫入分頁錯誤（`activateTab`）。
  - 本機分頁探測為並存，但 `projectStats` 對分頁路徑回錯誤（分頁路徑不是專案根）→ `activateTab` 寫入 `store.tabInvalid`，不呼叫 `enterProject`。
  - `project_stats_at` 對「並存但探索命中的根不是 `path`」→ 照舊回錯誤。
  - `requestMigration` 對並存資料夾 → 照舊拋出「不再是本機 Speclink 專案」，`migrationRoot` 維持 `null`（行為不變）。
- **驗收**：
  - `apps/desktop/src/__tests__/remoteOpen.test.ts` 新增測試：並存資料夾「繼續本機」後切到另一個本機分頁再 `activateTab` 切回，`activeKey` 為 `local:/work/coexists`、沒有分頁錯誤、`pendingRemoteConflict` 為 `null`、`openRemote` 沒被呼叫。
  - `apps/desktop/src/__tests__/workspace.test.ts`（`restoreTabs` 的既有測試所在）新增測試：持久化分頁含這個並存資料夾且它是 activeKey，`restoreTabs` 後它是作用中、沒有分頁錯誤；它在背景時，`restoreTabs` 後也沒有分頁錯誤。
  - `remoteOpen.test.ts` 的測試：對並存資料夾呼叫 `requestMigration` 照舊 reject（訊息含「不再是本機 Speclink 專案」），`migrationRoot` 為 `null`。
  - `remoteOpen.test.ts` 新增測試：並存資料夾的分頁在 `projectStats` 回錯誤（分頁路徑不是專案根）時，`activateTab` 後分頁錯誤為「已不是 speclink 專案」，`activeKey` 不變，`openRemote` 沒被呼叫。
  - `remoteOpen.test.ts` 新增測試：本機分頁探測為 `remoteBinding` 且 `hasLocalOpenspec` 為 false 時，`activateTab` 後分頁錯誤為「已不是 speclink 專案」。
  - `apps/desktop/core/src/project.rs` 的測試模組新增：並存資料夾（有 `speclink/`，`.speclink.yaml` 有含 url 的 remote 區段）`project_stats_at` 回 `pendingWrapUp`；對它的子目錄呼叫仍回錯誤。
  - `cargo test -p speclink-desktop-core` 與 `npm test -w @speclink/desktop` 都通過。
- **範圍**：只改 `activateTab` 的本機分支與 `project_stats_at`，以及它們的測試。`requestMigration`、`openProjectAt`、`continueLocalFromConflict`、`migrateLocalFromConflict`、`open_project_at`、首次啟動的啟動目錄探測、三出口對話框的文案都不改。

## Risks / Trade-offs

- [使用者在 Desktop 看本機看板，同一個資料夾裡終端機與 AI 工具跑的 `speclink` 走 Server] → 這是並存的既有性質，三出口對話框已說明；來源討論決定這次不加提示。
- [背景啟動時對並存資料夾多做一次 `Workspace::discover` 與模式解析] → 每個背景分頁一次，成本是讀一個 YAML 檔。
- [切回並存分頁時多呼叫一次 `projectStats`] → 只在探測結果為並存時多跑，代價是讀一次本機的變更與討論清單，換來與背景檢查一致的根比對。
- [資料夾在本機分頁建立後才加上 remote 區段（例如 git pull 拉進隊友改過的 `.speclink.yaml`）] → 切回時同樣以本機開啟、不跳三出口對話框；Desktop 讀本機檔案，終端機的 `speclink` 走 Server。本機分頁存在即視為選擇的紀錄，這次接受這個取捨。
- [回歸對照] → 不動 CLI 與引擎，golden 與 CLI 測試不受影響；路徑比對沿用既有的字串比對方式，Windows 與 macOS 的行為和現有 `project_stats_at` 相同。

## Context

討論 herdr-speclink-panel 定案「Herdr 管執行、speclink 本體管規劃與閱讀」，兩邊靠 `speclink://` 網址連接。現況：

- 桌面 app（apps/desktop）沒有任何對外入口：Info.plist 沒有 CFBundleURLTypes，apps/desktop/src-tauri/Cargo.toml 沒有 deep-link 或 single-instance 外掛，Tauri 殼也不讀啟動參數
- 前端 store 已有開專案與開詳情的動作：`openProjectAt(path)`（探測資料夾後進入專案、加成分頁並整批載入）、`openDetail(name)`、`openDiscussion(slug)`；系統匣的「開啟此變更」就是「喚起主視窗＋openDetail」，喚起主視窗的函式 `openMainWindow` 在 apps/desktop/src/tray.ts（先收合面板，再 unminimize、show、setFocus）
- 啟動時 App 以 `restoreTabs()` 還原分頁列；在它完成前 store 還沒有活躍專案
- `openDetail`／`openDiscussion` 只在目標已在目前專案的清單裡時才動作；不可取消的浮層開著時不改狀態（desktop-app「詳情頁互斥」）
- 桌面已用 sonner 的 `toast` 顯示提示（例：更新可用）

## Goals / Non-Goals

**Goals:**

- 外部程式送出 `speclink://change/<name>?project=<path>` 或 `speclink://discussion/<slug>?project=<path>`，桌面 app 打開該專案並開啟對應詳情頁
- app 未執行時由網址啟動；已執行時由執行中的那一個處理
- 格式錯誤、找不到專案或目標時，主視窗照常顯示並以 toast 說明

**Non-Goals:**

- remote 專案的網址；linked worktree 路徑換算成主 checkout；開路線圖的網址（留給 desktop-roadmap）
- CLI 動詞或 `speclink-desktop` 啟動參數當入口
- 任何經由網址觸發的寫入動詞
- Herdr 端的實作

## Decisions

### 網址格式：種類放 host、識別放 path、專案放 query

`speclink://change/<name>?project=<百分比編碼的絕對路徑>` 與 `speclink://discussion/<slug>?project=<…>`。以標準 URL 解析：host 是種類（`change` 或 `discussion`），path 只有一段且是名稱，query 的 `project` 是專案資料夾的絕對路徑（POSIX 以 `/` 開頭；Windows 以磁碟代號加 `:\` 或 `:/` 開頭）。其餘 host、多段 path、缺 `project`、相對路徑一律視為格式錯誤。路線圖將以新增 host（例：`roadmap`）擴充，不動既有兩種。

替代方案：`speclink://open?kind=change&name=…&project=…`——全部塞 query 可讀性差、和討論提到的形狀不符；把專案路徑放 path——Windows 路徑的反斜線與冒號要另外處理，比 query 參數麻煩。

### 解析與派送落在前端模組，Tauri 殼只註冊外掛

新增前端模組 deepLink.ts，內含兩部分：純函式 `parseSpeclinkUrl(url)`（字串 → `{ kind: "change" | "discussion", name, project }` 或錯誤原因），與 `installDeepLinks(store, deps)`（訂閱外掛事件、排隊、派送）。理由：要呼叫的 `openProjectAt`、`openDetail`、`openDiscussion` 都是前端 store 動作；外掛的 JS API（`@tauri-apps/plugin-deep-link` 的 `getCurrent`、`onOpenUrl`）直接把網址交給前端，不需要新增 Tauri command。沒有領域邏輯要放 speclink-desktop-core，Rust 端只做外掛註冊與 single-instance 的視窗喚起。這是唯一的網址處理落點，App.tsx 只呼叫一次 `installDeepLinks`。

替代方案：Rust 端解析後 emit 事件——多一層轉手且仍要回前端呼叫 store 動作，是零深度的轉接層。

### 啟動時的網址排隊到分頁還原之後

App.tsx 保留 `restoreTabs()` 的 promise，傳給 `installDeepLinks` 當「就緒」訊號。就緒前收到的網址（含 `getCurrent()` 取得的啟動網址與 `onOpenUrl` 事件）依到達順序排隊；就緒後逐一處理，同一時間只處理一個，前一個完成（或失敗）才處理下一個。沒有 workspace（非 Tauri 環境、測試）時不安裝。

### 專案一律走 openProjectAt，詳情等載入後再開

處理一個網址的順序：1. `openMainWindow()` 喚起主視窗（先做，讓後續的確認對話框出現在使用者眼前）；2. `openProjectAt(project)`——已是分頁的專案會被 upsert 成活躍分頁並重新整批載入，未開過的會加成分頁，未初始化、未啟用、帶 remote 標記的資料夾照既有流程跳出對話框或選擇器；3. 第 2 步進入了本機專案時，執行 `openDetail(name)` 或 `openDiscussion(slug)`；4. 目標不在已載入的清單裡時以 toast 說明找不到。對話框或選擇器出現的情況（第 2 步沒有進入專案）不開詳情、不出 toast。

為了讓第 3 步知道第 2 步的結果，apps/desktop/src/store.ts 的 `openProjectAt(path)` 由回傳 `Promise<void>` 改為回傳 `Promise<string | null>`：探測結果為本機專案、且整批載入完成時回傳該分頁的 locator key，其餘情況（未初始化、未啟用、帶 remote 標記、探測失敗）回傳 null。既有呼叫端都忽略回傳值，行為不受影響；把它當 `onOpenLocal` 往下傳的三個元件（WorkspaceChooser、EmptyWorkspace、RecentList）把該 prop 的型別由 `Promise<void>` 放寬為 `Promise<unknown>`。

替代方案：已是分頁時改用 `activateTab`——要自己比對 locator 與路徑正規化；`openProjectAt` 已能處理「已存在的分頁」，少一條分支。不改 store、改由 deepLink.ts 自己比對活躍分頁與 pending 對話框狀態——要同時盯住四種 pending 欄位，日後新增探測分支時容易漏，不如由 `openProjectAt` 直接說出結果。

### 主視窗喚起沿用系統匣的 openMainWindow

把 apps/desktop/src/tray.ts 的 `openMainWindow` 改為匯出，deepLink.ts 直接呼叫，不另寫一份。

### Windows、Linux 以 single-instance 轉交網址並喚起視窗

Rust 端在 Windows、Linux 先註冊 tauri-plugin-single-instance（開 deep-link 功能，且依外掛要求排在所有外掛之前），再註冊 tauri-plugin-deep-link；第二次啟動時外掛把網址轉給執行中的 app 的 `onOpenUrl`，callback 另外把主視窗 unminimize、show、set_focus。Linux 的 AppImage 不經安裝程式，setup 時以 deep-link 外掛的執行期註冊 API 補登網址格式（失敗只記錄、app 照常）。macOS 由系統確保單一實例，只註冊 deep-link 外掛。

結果：在 Windows、Linux 再次啟動 app（不帶網址）也改為把已開的視窗帶到前景，而不是開第二個 app。

### 網址格式的註冊與權限

tauri.conf.json 的 `plugins` 加上 deep-link 設定（desktop 的 schemes 為 `speclink`），macOS 由 bundler 寫進 Info.plist 的 CFBundleURLTypes，Windows 由 NSIS 安裝程式登錄；capabilities 的 default.json 加入 deep-link 外掛的預設權限，讓主視窗能呼叫 `getCurrent`、`onOpenUrl`。

### 提示文字走既有 i18n

格式錯誤、找不到變更、找不到討論三種 toast 的文字加進 apps/desktop/src/i18n/messages.ts（tw、en），以 `appT` 取用，不寫死。

## Implementation Contract

**行為**

- 網址 `speclink://change/<name>?project=<path>`：主視窗出現並取得焦點，活躍分頁是 `<path>` 探測到的專案，主區是看板且開著變更 `<name>` 的詳情頁
- 網址 `speclink://discussion/<slug>?project=<path>`：同上，開著討論 `<slug>` 的詳情頁
- app 未執行時，網址啟動 app，分頁還原後依上兩條處理；app 已執行時由同一個 app 處理，不出現第二個視窗或第二個 Dock／工作列圖示
- Windows、Linux 再次啟動 app（不帶網址）時，既有主視窗被帶到前景

**介面**

- `parseSpeclinkUrl(url: string)` → `{ ok: true, link: { kind: "change" | "discussion"; name: string; project: string } }` 或 `{ ok: false, reason: "scheme" | "kind" | "name" | "project" }`；`name`、`project` 為百分比解碼後的字串
- `installDeepLinks(store, { ready, getCurrent, onOpenUrl, openMainWindow, toast })` → 回傳卸載函式；依賴以參數注入，測試以假物件替換
- store 的 `openProjectAt(path)` 回傳 `Promise<string | null>`：進入本機專案時為分頁的 locator key，其餘為 null
- 不新增 Tauri command；前端只用 `@tauri-apps/plugin-deep-link` 的 `getCurrent`、`onOpenUrl`

**失敗模式**

- 格式錯誤：主視窗照常喚起，toast「無法開啟這個 speclink 網址」，不切換專案
- 專案資料夾探測失敗或不是專案：沿用 `openProjectAt` 既有的對話框、選擇器或錯誤處理，不另出 toast、不開詳情
- 目標不在清單裡（打錯名稱、已封存、在別的分支）：活躍專案已切過去，toast「在 <專案名稱> 找不到變更 <name>」或「找不到討論 <slug>」
- 不可取消的浮層開著：詳情頁不開（與系統匣同語意），主視窗仍喚起
- 網址只能開啟與檢視：處理流程不呼叫任何寫入動詞；未初始化、未啟用的資料夾只出現既有確認框，確認前零寫入

**驗收**

- vitest：`parseSpeclinkUrl` 的合法與不合法網址表（含 Windows 路徑、百分比編碼、多段 path、缺 project、未知 host）
- vitest：`installDeepLinks` 以假 store 驗證——就緒前的網址會排隊到就緒後依序處理；成功時依序呼叫 openMainWindow → openProjectAt → openDetail／openDiscussion；目標不在清單時呼叫 toast 且不呼叫 openDetail；格式錯誤時只喚起視窗與 toast、不呼叫 openProjectAt
- vitest：讀 tauri.conf.json 與 capabilities/default.json，確認網址格式為 `speclink`、預設權限已列入
- `cargo build -p speclink-desktop` 在三平台 CI 通過（Windows、Linux 會編入 single-instance）
- 手動（macOS 安裝版）：app 關閉與開啟兩種情況下，以終端機執行 open 指令送出網址，詳情頁正確開啟；送出格式錯誤的網址會出現 toast

**範圍**

- 範圍內：apps/desktop 的 Tauri 殼（外掛註冊、設定、權限）與前端（deepLink.ts、App.tsx 的安裝、tray.ts 匯出 openMainWindow、store.ts 的 openProjectAt 回傳值、i18n 文字）
- 範圍外：speclink-core、speclink-cli、server、Herdr mod；路線圖網址；remote 專案

## Risks / Trade-offs

- [macOS 開發模式不註冊網址格式：Launch Services 只認安裝的 bundle] → 手動驗收一律用安裝版（desktop-install 腳本）；自動測試只測解析與派送
- [Windows、Linux 沒有實機驗收] → 解析與派送由 vitest 涵蓋、外掛註冊由三平台 CI 編譯涵蓋；風險在作業系統登錄層，由外掛負責
- [single-instance 改變 Windows、Linux「再次啟動」的行為：不再能開第二個 app] → 這是網址轉交的必要條件；callback 把既有視窗帶到前景，使用者看到的是熟悉的單一 app 行為（與 macOS 一致）
- [任何本機程式或網頁都能送出網址] → 處理流程只開啟與檢視，不寫入；未初始化資料夾只會出現確認框；瀏覽器開啟自訂網址格式前通常會先詢問
- [openProjectAt 對已是分頁的專案會重掛監看並重新整批載入，切換比 activateTab 慢一點] → 接受；換來少一條比對路徑的分支
- [回歸對照] → 不動 CLI、core 與 golden；desktop 既有 vitest（tray、store、App）需維持通過，tray.ts 只多一個 export

## Migration Plan

- 新增入口，無資料遷移；舊版 app 不認得網址，作業系統會回報無法開啟
- 退版：移除外掛註冊與設定即可，前端模組在沒有外掛事件時不會被觸發

## Open Questions

(none)

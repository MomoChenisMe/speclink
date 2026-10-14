## Context

本變更服務桌面閱讀與專案辨識。現有 packages/ui 共用色票以系統媒體查詢切換，Tailwind 的 dark 變體也依系統；只改背景色會造成徽章等狀態色仍停在另一模式。通知元件另固定傳入 system。macOS 面板是獨立 WebView，既有 tray-snapshot／tray-panel-ready 已提供主視窗投影與首次同步通道。

專案根目錄已在 WorkspaceLocator：本機 root、遠端可選 checkoutRoot。ProjectSettingsView 目前僅接收設定 provider，因此須從 App 的作用中專案傳入位置，而非由設定檔相對路徑反推。

## Goals / Non-Goals

**Goals:**
- 範圍內：裝置級三選偏好、保存與即時套用、全桌面網頁介面和 macOS 面板一致配色、專案目錄唯讀展示、中英文文案。
- 沿用色票與前端資料，不新增引擎、Host、Protocol、Server 或 Rust command；純呈現與瀏覽器偏好邏輯置於桌面 TypeScript，無桌面核心領域演算法需移入 Rust。

**Non-Goals:**
- 不做各專案配色、任意色票、自訂主題、排程或跨裝置同步。
- 不增加路徑編輯、搬移、資料夾開啟或額外複製按鈕；使用文字選取複製。
- 不取得 Server 實體目錄、不變更 remote 政策、連線或認證。
- 系統管理的原生選單、檔案對話框、選單列 template 圖示與毛玻璃材質維持原生行為；app 自繪面板的補光、文字與控制項套用外觀偏好。
- server-web 不新增外觀設定；無覆寫時共用樣式維持系統深淺行為。

## Decisions

### 裝置外觀偏好

新增 apps/desktop/src/theme.ts，集中 ThemePreference（system／light／dark）、ResolvedTheme（light／dark）、讀寫、系統偏好解析與訂閱。localStorage 使用 speclink.uiTheme；缺鍵、非法值或讀取失敗視為 system，system 以移除鍵保存，另兩值直接保存字串，不改既有語言鍵。

主視窗是唯一偏好擁有者：main 啟動時先讀取並把有效值放到 html 的 data-theme，再渲染 React。App 訂閱同一控制器，設定 UI 顯示偏好，Toaster 接收有效值。系統變更只在 system 下改有效配色；不得因工作區切換重建偏好。保存先寫成功再更新狀態與通知訂閱者，失敗保留原選擇及配色、於外觀卡顯示單行錯誤。

此模組隱藏儲存、解析、DOM 套用與媒體監聽，供主視窗、tray 及面板套用函式共用，具有實際行為；不再包多層 service 或建立通用偏好框架。React 使用既有訂閱慣例或 useSyncExternalStore 連接；生命週期提供清理，避免 StrictMode／重新掛載累積監聽。

### 共用配色與通知

packages/ui/src/theme.css 保持一份 light 與 dark 色票。CSS 同時支援明示 data-theme 和沒有該屬性時的系統預設；Tailwind dark 變體與 token 使用同一判定：明示 dark 啟用、明示 light 停用，未明示才讀系統。以 Tailwind 的變體展開共用深色色票，避免複製兩組值；實作時以本地已安裝版本驗證輸出 CSS。color-scheme 與有效值一致，原生 HTML 控制項不與自繪內容混色。

桌面 index.css 的 Markdown 繼續由共用 token 取色，不另建文件主題。共用 Toaster 接受呼叫端 theme，缺省仍 system；桌面傳入已解析的 light／dark，既有單槽、逾時及錯誤樣式不變。server-web 無 data-theme 時維持自動模式，建置與瀏覽器驗證涵蓋此回歸。

### 系統匣面板配色同步

延伸既有 TraySnapshot，加入 effectiveTheme: light／dark。tray 的 pushSnapshot 在發送時從主視窗控制器取得有效值；偏好或系統造成有效值改變時立即推送，毋須等 workspace store 變動或去抖。既有 buildTraySnapshot 的純資料投影可接收明確有效值參數，測試不得讀全域 DOM。

面板先等 tray-snapshot 監聽完成，再發 tray-panel-ready，主視窗回覆帶有效配色的最新快照；收到時先套用 html data-theme，再呈現面板內容。首次快照前維持透明空容器，避免先顯示相反配色；尺寸觀察器在首份快照使內容節點存在後再掛載並量測，後續沿用頁首／中段／頁尾觀察，避免原本空節點使高度永遠未更新；掛載與取消期間須清理監聽，不遺留 ready 競態。已有快照的面板保留最後值並隨後續快照更新，隱藏期間仍收更新，重建時重新握手。不依賴不同 WebView 共用 localStorage 或 storage 事件，不新增第二條偏好儲存與 Rust 指令。

### 專案目錄展示

App 從作用中專案的 locator 取得本機 root 或遠端 checkoutRoot，傳入 ProjectSettingsView 的 projectDirectory: string | null；切換時同步更新。無 session 的恢復頁維持原路由，不因本需求新增設定查詢。

ProjectSettingsView 頂端、Tabs 之外顯示帶「專案目錄」標籤的等寬唯讀文字，使用可選取、任意長字串可換行的樣式，完整值不以省略號截掉。保留原始平台路徑分隔，不縮寫家目錄、不從設定檔字串拼接。null 顯示「未連結本機目錄」。外觀／目錄文字使用 APP_MESSAGES 中英文詞條，與 UI 語言同步；不受 AI 產出語言設定影響。

## Implementation Contract

- 偏好資料只有一個 localStorage 字串鍵；不涉及 serde 結構或專案 YAML。舊版資料缺鍵即 system，讀取未知值不自動寫回。寫入失敗保留上次成功值並顯示錯誤，允許重試。
- 新內部 payload 欄位 effectiveTheme 在主視窗及面板同版更新；不改對外 Protocol。tray 訂閱清理時一併取消主題訂閱。訂閱先完成再 ready，首次與後續快照皆帶配色。
- 外觀設定即使沒有開啟專案也可操作。system、light、dark 都必須在系統淺／深兩種條件下驗證；固定模式下系統切換不改畫面。
- 本機與遠端設定展示共用同一元件；連線離線但仍有 session 時使用既存 locator 顯示目錄。認證失效且無 session 時保留恢復畫面；設定 revision 衝突時目錄仍保持正確，不觸發任何寫入或查詢。
- AppSettingsView 的選項可由鍵盤操作，提供選中狀態與中英文名稱；保存錯誤以 role=alert 呈現。
- 驗收：theme 單元測試涵蓋解析矩陣、缺／壞值、讀寫失敗、媒體變更與清理；App 與設定頁測試涵蓋選取、重掛載恢復、無專案、路徑切換與遠端無副本；tray 接線測試涵蓋無工作區刷新時的配色推送、初次握手與清理；Toaster 測試覆蓋明示 theme 和缺省 system。
- 實際渲染驗收需用桌面 WebView 或瀏覽器檢查計算樣式：強制淺色配上系統深色，及反向組合，背景、狀態徽章、Markdown、toast 皆同模式；server-web 無覆寫仍自動；長 Windows／POSIX 路徑完整換行與可複製。另以 macOS 實窗確認面板首次、隱藏後重開與主視窗隱藏期間的系統切換。不以 jsdom 的 class 字串斷言取代渲染驗證。

## Risks / Trade-offs

- 回歸對照：不修改 CLI、引擎或人眼／JSON 輸出，因此本次無 golden 更新；執行桌面與共用 UI 相關測試及兩個 app 建置，避免無關全套測試掩蓋 UI 驗證重點。
- 跨平台：Tailwind dark 變體與 CSS 預設分歧會造成混色 → 同一判定與系統／手動矩陣渲染驗證；macOS 原生材質仍依系統，保留既有半透明補光並檢查可讀性。
- 首次面板事件競態 → 先 await 監聽再 ready，第一份快照前不渲染內容；主視窗隱藏仍為權威來源，無跨窗儲存假設。
- CSS 共用使 server-web 受影響 → 未標記根元素繼續系統模式，驗證明暗兩種計算樣式，不新增其偏好狀態。
- 瀏覽器儲存被拒 → 保存前置、失敗不改 UI 狀態，錯誤明確顯示；不加備份檔或第二儲存來源。

## Migration Plan

無資料遷移。新鍵缺省維持舊有自動配色；rollback 舊版時忽略新增偏好鍵，專案與 Server 文件均未改動。

## Open Questions

無，需求已確認；實作時依編譯後 CSS 與實窗結果驗證變體輸出，不需新增產品決策。

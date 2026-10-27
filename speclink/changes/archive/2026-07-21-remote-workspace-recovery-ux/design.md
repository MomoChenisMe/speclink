## Context

Desktop 目前把持久化分頁、已建立的 `WorkspaceSession` 與 `tabErrors` 分開保存。App 重啟後只還原 remote locator；再次選取時必須重走 handshake。該 handshake 失敗時，store 只寫入 `tabErrors` 而不更新 `activeKey`，所以使用者仍停在上一個 workspace 或設定頁；分頁只以驚嘆號與原生 `title` 顯示底層錯誤。這與已建立 remote session 後才發生的 offline 不同：後者有最後成功 snapshot、connection state、stale 橫幅與自動收斂，前者沒有 session 或可信 snapshot。

Tray 目前只接收分頁 `key`／`name`、作用中 key 與作用中資料，沒有分頁來源或復原狀態。macOS 面板與原生選單雖能以 locator key 切換 remote 分頁，切換失敗仍依正典刻意靜默；面板因此可能繼續顯示上一個 workspace 的內容。stakeholders 是在 Phase 3 Desktop remote workflow 中切換專案、檢視 SDD 進度並透過 AI 代理工作的開發者、PO 與 PM。

本變更只落在 `apps/desktop` React／Tauri adapter 邊界。`speclink-core`、`speclink-cli`、server API、TeamStore 與 Local Repo 路徑不變；沒有 ANSI、git、儲存 driver 或 CLI parity 影響。

## Goals / Non-Goals

**Goals:**

- 讓 remote 分頁在 restoring 或 handshake error 時仍是可選取、可理解、可復原的 navigation destination。
- 以同一 store 真相驅動主視窗、macOS Tray Panel 與原生 Tray 選單，且保留各 surface 的空間與焦點慣例。
- 將一般使用者文案、復原動作與底層 technical detail 分層；loading、error 與 recovery 皆可測且可存取。
- 保留已建立 session 的 offline stale snapshot、自動重連與 write mask；不把無 session 的錯誤分頁偽裝成 stale。

**Non-Goals:**

- 不跨 app 重啟持久化 remote 資料 snapshot 或復原狀態。
- 不新增 retry worker、離線寫入、背景重放、衝突合併或通知中心。
- 不變更 Workspace Chooser、登入協定、Keychain、connection registry、Tray 內容分區、CLI、設定檔或 skills。
- 不讓 macOS 非 key panel 改為會奪取焦點的完整鍵盤視窗；鍵盤完整操作由主視窗與 OS 原生選單承擔。

## Decisions

### Decision 1：作用中分頁與可用 session 分離，例外狀態不持久化

store 新增以 locator key 索引的 remote tab recovery 狀態，只表示尚無可用 session 的例外期：`restoring`，或 `error` 加上 failure kind 與 technical detail。狀態缺席代表由現有 session／local 開啟流程處理，不另複製 ready 真相。`activeKey` 可指向仍存在於 tabs 但尚無 session 的 remote 分頁；所有資料操作仍只經 `activeSession()`，缺少 session 時不得讀取上一分頁資料或發出 workspace command。

選取無 session 的 remote 分頁時，store 先同步設定 `activeKey`、看板目的地與 `restoring`，再發 handshake；成功後採用 session 並清除 recovery，失敗後保留 active 與 tab，轉為 `error`。重新連線重用同一路徑；關閉分頁同時清除 recovery。recovery 狀態不寫入 localStorage，重啟仍只持久化既有 v2 locator／activeKey，避免新的 migration 與過期錯誤跨重啟殘留。

每次 activate／retry 取得遞增 request generation。較舊請求完成時可更新其對應 session／error，但 SHALL NOT 把 `activeKey` 從使用者後來選取的分頁搶回；同一 key 的較舊 completion SHALL NOT 覆蓋較新的 retry 結果。

替代方案是把 ready／offline／error 全部寫入 `ProjectTab` 並持久化；被否決，因為會複製 `WorkspaceSession.connectionState`、要求 v3 migration，並把瞬時網路狀態變成過期磁碟真相。另一替代是維持失敗不切換；這正是「點擊無反應」的根因。

### Decision 2：remote_open 保留 machine-readable error，UI 不比對英文訊息

Tauri `remote_open` 的成功 payload 維持 `RemoteOpenInfo`；失敗改為可序列化物件，至少含 `message: string`、`reason: string | null`、`status: number | null`。欄位沿用 remote protocol 已存在的 reason／HTTP status，不改 server HTTP API、不含 token、credential、header 或 filesystem secret。Desktop adapter 將其正規化為封閉 failure kind：`unreachable`、`needs-reauth`、`access-denied`、`not-found`、`unknown`；`message` 只作 technical detail，使用者摘要由 i18n key 產生。

401 或 runtime 已進 needs-reauth 對應 `needs-reauth`；403 對應 `access-denied`；404 對應 `not-found`；無 HTTP response 的 transport unavailable 對應 `unreachable`；其他值 fail-closed 為 `unknown`。無法解析的舊字串 rejection 亦歸 `unknown`，讓測試注入與升級邊界不崩潰。

替代方案是以前綴或英文句子比對 `String(error)`；被否決，因為 locale、remote client 文案或環境變數說明一變就會誤分類。另一替代是新增 server endpoint；被否決，現有 reason／status 已足夠且此 UX 不需改 wire protocol。

### Decision 3：主視窗以 recovery destination 取代 tooltip-only 錯誤

`ProjectTabs` 對 remote presentation state 使用單一一致狀態圖示：ready cloud、restoring spinner、已建立 session 的 offline cloud-off、needs-reauth 或 handshake error 的語意圖示；錯誤分頁維持可點外觀與清楚 focus／selected state，不使用整體低透明度暗示 disabled。短 tooltip 只顯示「狀態＋點擊查看與修復」，不承載 raw error。

當 active remote tab 有 recovery 狀態且沒有 session，`App` 在既有主內容區呈現 `RemoteWorkspaceRecovery`，內容包含 workspace 名稱、server 顯示名／origin、繁中摘要、主要動作與可展開 technical detail。`unreachable`／`unknown` 的主要動作為重新連線並提供伺服器設定；`needs-reauth` 主要動作為前往該 connection 的重新登入；`access-denied`／`not-found` 提供重試、伺服器設定與移除分頁。移除沿用 `closeTab`，不刪 connection 或 server 資料。

restoring 超過 300ms 時顯示 spinner 與「正在重新連線」，按鈕在該請求完成前停用；成功自動換回正常內容。錯誤容器使用 `role=alert` 或等價 live region，狀態轉換不奪取任意輸入焦點；動作為真正 button 並有可見 focus ring。現有 session 轉 offline 時仍呈 stale 橫幅與最後內容，不進空 recovery page。

替代方案是只換成 Radix Tooltip 或 Sonner toast；被否決，因為 hover 不是主要互動、toast 會消失，兩者都沒有穩定復原目的地。Modal 亦被否決，因為一個壞掉的背景 workspace 不得阻塞其他分頁。

### Decision 4：TraySnapshot 投影共用狀態，Panel 與原生選單各自降階

`TraySnapshot.tabs` 在既有 key／name 上增加 source 與 presentation state，remote 錯誤另攜帶非敏感的 failure kind、connectionId、server label／origin；不攜帶 token 或完整 technical detail。投影優先序固定為：無 session 的 recovery restoring／error優先；有 session 時 needs-reauth 優先於 offline；其餘為 ready。此投影只讀同一 Zustand store，不新增 remote query、panel store 或第二份狀態機。

macOS Panel 的每個 remote tab 呈現來源／狀態，不以首字母 avatar 隱藏錯誤。active tab 為無 session restoring／error 時，原討論與生命週期分區整體改為一張精簡 recovery card，避免顯示上一 workspace 的資料；card 提供重試，以及明確開啟主視窗詳情／設定／重新登入的動作。直接重試透過既有 panel action event 回流 store、面板保持開啟且不喚起主視窗；顯式詳情、設定或重新登入才沿用 `openIn` 取得前景。active tab 是已有 session 的 offline 時，Panel 顯示 stale 狀態列並保留最後成功內容，等待既有 worker 自動收斂。

原生選單中 ready workspace 維持 check item；restoring 顯示 disabled 的「正在連線」狀態；error／needs-reauth workspace 以 submenu 顯示狀態摘要、重新連線或重新登入、開啟問題詳情與伺服器設定。OS 原生選單自行提供鍵盤導覽；正常專案切換仍不喚起主視窗。macOS 進行中數文字徽章維持原契約，不拿錯誤數覆蓋或混用。

替代方案是讓 Tray 只開主視窗顯示錯誤；被否決，因為在 Tray 內點擊仍會像失敗且違反就地回饋。另一替代是 Panel 自行呼叫 remote API；被否決，這會破壞現有「主視窗 store 單一真相＋panel 薄渲染」承重牆。

### Decision 5：surface focus 與無障礙邊界維持平台慣例

主視窗 recovery 與原生選單須完整支援鍵盤；圖示、狀態文字與 `aria-selected`／live announcement 共同表意，不依賴顏色。macOS Panel 保留非 key window、不自動 focus、失焦收合的既有行為；新增 card 動作與 tab 提供語意 label 與足夠 pointer hit area，但本變更不把 Panel 改造成搶焦點的鍵盤視窗。動畫只使用 150–300ms 的 opacity／color 狀態轉換，spinner 尊重 reduced motion；不新增依賴或另造色票。

替代方案是令 Panel 成為 key window 以取得完整 Tab 導覽；被否決，會改變 menu bar utility 不奪取前景 app 焦點的核心使用情境，範圍也遠超本次錯誤復原。

## Implementation Contract

**Behavior**

- 使用者點擊持久化 remote tab 後，分頁在 100ms 內呈 selected／restoring；handshake 失敗後 selected 不回退，主內容與 Tray 均有可見錯誤摘要和復原動作。
- retry 成功後同一分頁原地取得 session 並顯示 server 資料；retry 失敗更新同一 recovery 狀態，不新增分頁、不退 local、不清除 connection。
- 使用者在 handshake 期間切到另一分頁時，舊請求完成不得搶回 active；同一分頁連續 retry 只接受最新 generation 結果。
- 已建立 remote session 的 offline／needs-reauth 繼續顯示 stale snapshot；重啟後無 session 的 error 不顯示任何上一 workspace 或偽造 snapshot。
- local tab、正常 remote tab、Tray 變更／討論區、macOS 徽章與 CLI 行為逐項維持既有語意。

**Interface / data shape**

- `remote_open` rejection 為 camelCase 的 `{ message, reason, status }`；TS 正規化後 failure kind 為 `unreachable | needs-reauth | access-denied | not-found | unknown`。
- store 暴露 locator-keyed recovery state、retry action 與只讀 presentation projection；recovery state 不持久化。
- `TraySnapshot.tabs` 攜帶 `key`、`name`、`source`、`status`，僅 remote error 狀態攜帶復原所需的非敏感 connection／server metadata；panel action 增加 retry、open recovery、open settings／reauth，所有 action 仍回流主視窗 store。

**Failure modes**

- 無法解析 structured rejection 時呈 `unknown`，保留 technical detail 並仍提供 retry／settings；UI 不崩潰。
- Tray retry 失敗時 Panel／native menu 更新原位狀態；不得靜默、不得強制喚起主視窗。
- panel 建立失敗仍退回原生選單；原生選單重建失敗沿既有 tray controller 失敗邊界，不影響主視窗。
- technical detail 不得包含 credential；使用者摘要不得顯示 `.speclink.yaml` 或 `SPECLINK_STORE_URL` 作為 Desktop 的主要修復指示。

**Acceptance criteria**

- Vitest 先以紅燈覆蓋：active error destination、restoring／retry、latest-wins、主視窗復原頁、tooltip 非唯一資訊、TraySnapshot 投影、Panel recovery card、native submenu、focus 行為與 local regression。
- Rust 測試先以紅燈覆蓋 `remote_open` structured error 的 401／403／404／transport／unknown 映射及 secret absence。
- `npm test -w apps/desktop`、`npm run build -w apps/desktop`、相關 Desktop Rust tests、`speclink validate remote-workspace-recovery-ux` 與 Critical／Warning analyze gate 全部通過。
- 真實 macOS GUI 手動全鏈：重啟 app 還原 remote tab並令 server 不可達；主視窗與 Panel 逐項驗證 selected、loading、error、retry；原生 fallback 選單驗證 submenu；server 恢復後原地成功；needs-reauth 驗證顯式動作才喚起主視窗；local tab 全程正常。截圖或 process 存活不得替代直接互動斷言。

**Scope boundaries**

- In scope：Desktop remote tab navigation／recovery、Tauri open error payload、主視窗 recovery、Tray snapshot／Panel／native menu、繁中 i18n 與相關測試。
- Out of scope：server route、remote protocol wire format、CLI、core、storage、設定、登入協定、snapshot persistence、offline writes、全域通知與 Tray 全面鍵盤化。

## Risks / Trade-offs

- [activeKey 可指向無 session，既有 view 誤讀上一份陣列] → App 先以 active recovery branch 截斷所有 workspace view；資料 action 繼續只取 `activeSession()`，測試斷言錯誤頁不渲染上一分頁名稱或資料。
- [並行 handshake completion 導致分頁跳回或舊錯誤覆蓋新成功] → locator-keyed generation 與 active ownership 檢查；以 deferred promise 測試跨 tab 與同 tab retry 競態。
- [Tauri object rejection 與既有測試 string rejection 不相容] → adapter 正規化同時接受 structured object 與 unknown/string fallback；IPC payload 加 Rust serialization tests。
- [Tray debounce 使狀態看似延遲] → store 先同步進 restoring，panel push 在既有去抖上限內到達；超過 300ms 才顯示持續 loading，測試使用可控 timer。
- [狹窄 Panel 增加動作後高度膨脹] → error 時以單張 card 取代資料分區，不與完整內容疊加；維持既有 320px 寬與 640px 上限。
- [現有 tray spec 與新行為衝突] → 同一 change 完整 MODIFIED `選單專案切換` 與 `面板樣式（macOS）`，archive 後不留下雙重真相。

## Migration Plan

不改持久化 schema、資料庫或 server。部署後既有 v1／v2 tabs 照常讀取；瞬時 recovery state 由新版 app 啟動時重建。回滾只需還原 Desktop code 與正典 delta，既有 localStorage、connection registry 與 Keychain 無需轉換。

## Open Questions

無；復原動作、surface focus 與狀態邊界已由本設計定案。

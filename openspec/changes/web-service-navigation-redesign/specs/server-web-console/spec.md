## ADDED Requirements

### Requirement: 全部 browser route 由單一 SPA 提供可發現導覽

Server SHALL 以 Vite、React 與 TypeScript SPA 提供 `/`、`/setup`、`/invite/:token`、`/login`、`/activate`、`/account`、`/admin` 與 `/admin/*`；每個頁面 SHALL 可由應用內導覽或當前流程中的明確動作到達，且 SHALL 支援直接開啟與重新整理。未登入流程、一般成員與管理員 SHALL 分別使用專注流程殼、帳號殼與管理殼；管理殼 SHALL 提供總覽、使用者、專案與儲存庫、憑證、資料操作、系統狀態、稽核紀錄七個目的地。SPA SHALL NOT 提供 changes、specs 或 discussions 的檢視與編輯。

#### Scenario: 管理員不需手打 URL 走訪管理功能

- **WHEN** 已登入管理員從 `/` 進入服務並依可見導覽走訪全部管理目的地
- **THEN** 管理員可到達七個管理頁面、帳號頁與登出動作，且不需修改瀏覽器網址

#### Scenario: 一般成員不顯示管理導覽

- **WHEN** 已登入但無 admin 旗標的成員開啟 `/account`
- **THEN** 頁面顯示帳號、PAT、Web session 與裝置資訊，且不顯示管理目的地

#### Scenario: 深連結可直接重新整理

- **WHEN** 管理員直接開啟或重新整理 `/admin/audit`
- **THEN** Server 回傳 SPA shell，SPA 完成 session 檢查後呈現稽核紀錄頁

### Requirement: 導向遵守伺服器裁決與安全優先序

SPA SHALL 以 browser session API 回傳的 `home` 與 mutation 回傳的 `destination` 導向，SHALL NOT 依客戶端角色自行計算安全敏感目的地。登入成功的導向優先序 SHALL 為有效 device `userCode`、通過白名單的 `returnTo`、角色 home；`returnTo` SHALL 僅接受以單一 `/` 開頭、無 scheme 或 authority，且首段為 `/account`、`/activate` 或 `/admin` 的站內路徑。未登入使用者進入受保護 route 時 SHALL 前往帶安全 `returnTo` 的登入頁；route 切換完成後 focus SHALL 移至 `<main>` 標題。

#### Scenario: 裝置核准優先於一般返回路徑

- **WHEN** 使用者以有效 device `userCode` 與 `/account` 的 `returnTo` 完成登入
- **THEN** Server 回傳 activation destination，SPA 先呈現裝置核准流程

#### Scenario: 外部 returnTo 不形成 open redirect

- **WHEN** 登入請求帶入 `https://evil.example/path` 或 `//evil.example/path` 作為 `returnTo`
- **THEN** Server 忽略該值並回傳角色 home，SPA 不導向外部 origin

#### Scenario: 一般成員不可用 returnTo 進入管理面

- **WHEN** 一般成員以 `/admin` 作為 `returnTo` 完成登入
- **THEN** Server 回 403，SPA 呈現無權限狀態且不降級導向 `/account`

### Requirement: SPA 資產與 fallback 具可驗證的安全邊界

Production SPA SHALL 以 compile-time asset embedding 進入 `speclink-server` binary，runtime SHALL NOT 依賴相鄰 `dist`、Node、CDN 或外部字型服務。`/assets/*` SHALL 只回傳 build manifest 內的資產與正確 MIME；內容雜湊資產 SHALL 回 `Cache-Control: public, max-age=31536000, immutable`，SPA shell SHALL 回 `Cache-Control: no-cache` 與 self-only Content Security Policy。SPA fallback SHALL 僅匹配已定義 browser GET route；未知 browser path、asset、`/api/*`、`/auth/*`、health、readiness 與下載 route SHALL NOT 回傳 `index.html`。

#### Scenario: 無外部靜態檔仍可載入 SPA

- **WHEN** 在沒有 Node、`dist` 目錄與外網連線的 runtime 啟動 release binary 並請求 `/login`
- **THEN** 回應載入內嵌 index、hashed JavaScript、CSS、字型與圖示，且所有資產來自相同 origin

#### Scenario: 拼錯 API 不被 SPA fallback 吞掉

- **WHEN** client GET `/api/speclink/v1/web/unknown`
- **THEN** Server 回 JSON 404，回應內容不是 SPA index

#### Scenario: 未知資產不回 shell

- **WHEN** browser GET `/assets/missing.js`
- **THEN** Server 回 404，回應內容不是 SPA index

### Requirement: 共用設計系統維持高密度可存取體驗

Server SPA SHALL 復用 `packages/ui` 的 shadcn/ui 原語、共用 semantic theme、Noto Sans TC 與青綠 focus token，SHALL NOT 建立第二套 theme 或 import Desktop 的 Tauri、Zustand store 與 SDD board 元件。寬度至少 1024px 時管理殼 SHALL 顯示 icon 加 label 的固定側欄；更窄時 SHALL 以有可見 trigger 的 Sheet 提供相同目的地。每頁 SHALL 只有一個視覺 primary action，手機版資料 SHALL 轉為可換行 row 或 card 且 SHALL NOT 造成整頁水平捲動。

所有互動 SHALL 可由鍵盤完成，並包含 skip link、至少 2px focus ring、連續 heading、icon-only control 的 aria-label、輸入 label／helper text／autocomplete、鄰近欄位且以 `role=alert` 宣告的錯誤、至少 44×44px 互動目標、至少 16px 正文與至少 4.5:1 正常文字對比。動畫 SHALL 限於 150–250ms opacity 或 transform；`prefers-reduced-motion` 啟用時 SHALL 停用動畫。

#### Scenario: 375px 完成主要流程

- **WHEN** 使用者以 375px viewport、200% zoom 與鍵盤操作 login、setup 或 invite 流程
- **THEN** 主要內容與操作保持可見、focus 順序合理、沒有整頁水平捲動，且可完成提交

#### Scenario: 窄螢幕管理導覽使用 Sheet

- **WHEN** 管理員以 768px viewport 開啟 `/admin`
- **THEN** 固定側欄收合、可見 trigger 開啟含七個目的地的 Sheet，關閉後 focus 回到 trigger

#### Scenario: reduced motion 停用轉場

- **WHEN** 作業系統設定 `prefers-reduced-motion: reduce`
- **THEN** SPA 不執行 route、Sheet、toast 或狀態切換動畫，且功能狀態仍完整可辨識

### Requirement: Browser API 互動狀態一致且可恢復

SPA SHALL 只透過 `/api/speclink/v1/web` same-origin JSON API 讀寫 server 資料，所有 raw HTTP 呼叫 SHALL 集中於 typed client。成功 envelope SHALL 為 `{data: T}`；錯誤 envelope SHALL 為 `{error:{code,message,fieldErrors?}}`，欄位 SHALL 使用 camelCase。每個 route SHALL 表示 loading、success、empty、forbidden 與 unexpected error；route chunk 或 render 失敗 SHALL 由 error boundary 顯示重試入口而非白屏。Mutation 期間 SHALL 停用重複提交並顯示進度；成功 SHALL 以 `aria-live=polite` 回饋；失敗 SHALL 保留輸入與原頁資料。停權、撤銷與資料遷移等破壞性操作 SHALL 在送出前以 AlertDialog 顯示確切對象並要求確認。

#### Scenario: 欄位驗證失敗保留輸入

- **WHEN** 使用者提交表單後收到含 `fieldErrors` 的 400 回應
- **THEN** SPA 保留所有非祕密輸入、把錯誤放在對應欄位附近並以 `role=alert` 宣告，且允許修正後重送

#### Scenario: Session 過期回到登入並保留安全路徑

- **WHEN** 已載入的受保護 route 呼叫 browser API 收到 401
- **THEN** SPA 前往 `/login` 並只保留通過白名單的當前 route 作為 `returnTo`

#### Scenario: 破壞性操作阻止重複提交

- **WHEN** 管理員確認撤銷一組憑證且請求仍在進行
- **THEN** 確認按鈕停用並顯示進度，第二次提交不會送出；完成後以可存取訊息回報結果

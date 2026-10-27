## Context

Speclink Server 的 browser surface 目前由 `crates/speclink-server/src/web.rs`、`setup.rs` 與 `admin.rs` 直接組合 HTML 字串；共用 `page()` 只輸出 document shell，沒有 stylesheet、應用導覽或一致的載入／成功／失敗狀態。Router 已有 `/setup`、`/invite/{token}`、`/login`、`/activate`、`/account` 與 `/admin/*` 深連結，但根路徑沒有入口，登入後固定落 `/account`，管理員必須自己知道 `/admin`。

`apps/desktop` 已使用 React、Tailwind CSS、shadcn/ui 原語、Noto Sans TC 與青綠語意 token；`packages/ui` 的既定目的就是 Desktop／Web 共用呈現與資料源解耦。Server 的認證、identity、admin single-point action、audit、setup token 與 device authorization 已有 Rust 實作與測試，這次不得把權限裁決搬進 React，也不得讓 Web UI 直碰 TeamStore 或 identity SQLite。

主要使用者是透過 AI 代理執行 Remote SDD 的開發者、PO、PM，以及負責首次 setup、邀請、憑證與服務維運的自架管理員。使用情境位於 Remote workflow 進入 `propose`／`apply` 前的環境建立與成員接入，以及日常帳號、裝置、registry、audit、資料與健康狀態管理。

## Goals / Non-Goals

**Goals:**

- 以單一、可深連結、依角色導覽的 React SPA 取代全部 server-rendered browser pages。
- 維持單一 `speclink-server` binary／Docker image 交付，SPA 與 API 永遠 same-origin、同版本。
- 讓 Rust/Axum 繼續擁有 session、Origin、admin、token、audit 與資料驗證；React 只消費 browser JSON contract。
- 復用 `packages/ui` 的 shadcn 原語與語意 theme，不建立第二套元件庫，且不改變 Desktop 可觀察外觀。
- 以可測試的 loading、success、error、empty、forbidden 與 destructive confirmation 狀態達到鍵盤、觸控與 WCAG AA 基線。
- 讓 root validation、三平台 CI、Docker 與 release 對前端測試／production assets 提供同一綠燈門檻。

**Non-Goals:**

- 不在 Server Web 加入 changes、specs、discussions 看板或內容編輯。
- 不獨立部署前端，不做 CORS、PWA、service worker、離線快取或第二個公開 origin。
- 不改 `speclink-core`、`speclink-cli`、Client Protocol、TeamStore contract、identity schema、設定欄位、技能或 CLI 輸出。
- 不把 Desktop Zustand store、Tauri adapter 或 Desktop 領域元件搬進 Server Web；只共用無宿主依賴的 UI 原語、theme 與 i18n 基礎。
- 不永久保留舊 HTML／form 與 SPA 兩套 browser interface。

## Decisions

### D1：`apps/server-web` 擁有 SPA，`packages/ui` 擁有共用設計原語

新增 npm workspace `apps/server-web`，以 Vite、React、TypeScript 與 React Router 組成三個殼層：未登入／token-gated 的專注流程殼、登入成員的帳號殼、管理員的管理殼。Route module 以 lazy import 切分管理頁面；路由 loader／action 管理首載、提交與錯誤邊界，不新增全域 server-data store。只有跨路由且非伺服器真相的 UI 狀態可留在 React context 或 component state。

`packages/ui` 持有 Button、Input、Sheet、AlertDialog、Sonner 等既有原語，並補齊 Sidebar、Table、Label、Skeleton、Separator、DropdownMenu 等實際被 Web Console 使用的 shadcn 原語。青綠色彩、radius、字體與 focus token 抽成 `packages/ui/src/theme.css`，Desktop 與 Web 各自 import；server 專屬 Sidebar 組合、頁面文案與 HTTP 型別留在 `apps/server-web`。

替代方案：在 Web app 內執行一套獨立 shadcn registry——會複製 token、修正與可存取性；把整個 Desktop app 元件直接搬入 Web——會帶入 Tauri／SDD 看板領域，超出 Server 管理面；新增 Zustand 或 TanStack Query——目前 route loader/action 足以表示頁面資料與 mutation，先不增加狀態框架。

### D2：browser JSON API 使用獨立 same-origin session 邊界

新增 `/api/speclink/v1/web/*` browser API。此 namespace 只接受 bundled SPA 的 session-cookie 流程：所有 mutation 先做 Origin／Referer 同源檢查，再解析 active session；admin 子樹再做 admin flag 檢查。既有 bearer `/api/speclink/v1/admin/*` 與 Client Protocol 不改，兩個 adapter 呼叫相同 identity／admin single-point action，禁止複製領域邏輯。

成功回應使用 camelCase JSON 的 `{ "data": ... }`；錯誤使用 `{ "error": { "code": string, "message": string, "fieldErrors"?: Record<string,string> } }`。Validation 回 400、未登入或 invalid credentials 回 401、已登入但權限不足回 403、無效／過期／已使用 token 維持既有不可區分的 404 或 401 分類、衝突回 409、非預期錯誤回不含內部細節的 500。`fieldErrors` 只含可公開且能對應表單欄位的修正訊息。

API family 包含：session／login／logout、setup state 與兩個提交節點、invitation lookup／accept、account summary／PAT／device family、activation lookup／decision，以及 admin overview／users／registry／credentials／data／system／audit 的讀取與 mutation。帳號摘要 SHALL 分別回傳 user、PAT metadata、web sessions 與 device families；PAT 建立回應的 plaintext 僅在該次 `data` 出現。

替代方案：直接讓 React POST 舊 HTML form endpoint——無穩定 JSON 錯誤與 loading contract；讓現有 bearer admin API 同時接受 cookie——會把版本化外部 API 與 browser CSRF 邊界混在同一 extractor；為每個頁面各寫 fetch——會重複 credentials、error parsing 與 session-expired 行為。

### D3：導向由伺服器計算，深連結與裝置核准優先

`GET /api/speclink/v1/web/session` 回 `{ authenticated, user, home }`；`user` 未登入時為 null，登入時至少含 `id`、`email`、`display`、`admin`，`home` 只能是 `/login`、`/account` 或 `/admin`。Login、setup completion 與 invitation acceptance 的成功回應皆由 Rust 回 `destination`，React 只執行站內 navigate。

導向優先序固定為：有效 device `userCode` → 經白名單驗證的 `returnTo` → 伺服器依角色計算的 home。白名單只接受以單一 `/` 開頭、無 scheme／authority、且首段屬 `/account`、`/activate`、`/admin` 的路徑；一般成員的 admin destination 在伺服器端回 403，不降級成 `/account`。未登入載入受保護 SPA route 時導向 `/login?returnTo=...`；route change 後 focus 移至 `<main>` 標題。

Setup 建立第一個 admin、registry、耗用 token、record audit 與建立 session 的既有交易邊界保持；完成回應另帶 `connection: { publicUrl, projectKey, repoKey }` 與 `/admin?welcome=1`。Invitation acceptance 在原子建立 user／耗用 token 後建立 session，admin invitation 進 `/admin`，一般 invitation 進 `/account`。若 session 建立失敗，成功建立的帳號不得偽裝成已登入，回可重試登入的 500 recovery message。

替代方案：React 讀 `admin` 自己決定 destination——客戶端會成為安全敏感控制流；所有 login 固定進 `/account`——中斷裝置核准與深連結；接受任意 `returnTo`——形成 open redirect。

### D4：管理 SPA 補齊 view-model API，所有 mutation 重用既有 domain action

Admin overview 聚合管理導航所需的低成本摘要：active／suspended user 數、project／repo 數、active credential 數、store health、identity schema version，以及 setup welcome 所需 connection fields。Users、registry、credentials、audit、system、data 各自有獨立 GET view model；清單回傳穩定 id、顯示欄位與 action eligibility，不把 PAT hash、refresh credential、password hash 或 setup／invite token 放進 payload。

SPA mutation 包含邀請、停權／復權、membership role、admin flag、project／repo 建立與顯示名、PAT／device family 撤銷、scope export 與 store migrate。它們直接呼叫目前 CLI／admin API 使用的 single-point identity function；audit source 維持 `web`。舊 server-rendered form handler 在相對應 browser API 與 React route 測試綠後刪除，外部 bearer admin API 與 CLI 不變。

Store health 失敗時，overview／system／data 回傳可得資料與明確 `storeHealthy: false`／`storeHealthError`，users、credentials 與 identity 管理仍可用。危險動作在 UI 以 AlertDialog 明示對象；請求期間 disable submit 並顯示進度，成功以 aria-live polite toast 回饋，失敗保留輸入與原頁資料。

替代方案：讓 Rust 回頁面專用 HTML fragment——重新引入雙呈現層；由 Web app 直接組合多個 identity/store endpoint——造成 waterfall 與領域資料外洩；重做 admin domain service——現有 single-point action 已滿足 transaction/audit，新增一層無深度。

### D5：Vite hashed assets 內嵌 binary，fallback 僅服務明確 browser routes

Production build 先產生 `apps/server-web/dist`，Rust 使用 compile-time asset embedding 將 `index.html`、hashed JS/CSS、字型與圖示放進 binary。Release／Docker／CI 的建置順序固定為 npm install → Web production build → Rust server build；缺少 `index.html` 或 manifest 時 release build 直接失敗並指出先建 Web workspace，禁止產生只有 API、沒有 UI 的成功 server artifact。

`/assets/*` 只回 manifest 中的資產並帶正確 MIME；內容雜湊資產使用 `Cache-Control: public, max-age=31536000, immutable`，SPA shell 使用 `Cache-Control: no-cache`。Browser GET allowlist 為 `/`、`/setup`、`/invite/*`、`/login`、`/activate`、`/account`、`/admin` 與 `/admin/*`；allowlist route 回 `index.html`，未知 browser path 回 404。`/api/*`、`/auth/*`、`/healthz`、`/readyz` 與下載 route 永不進 SPA fallback。Response 採 self-only CSP，不從 CDN 或 Google Fonts 載入資產。

替代方案：獨立靜態服務／CDN——破壞單一交付與 same-origin；執行期讀相鄰 `dist`——binary 不再自包含且易版本錯配；catch-all fallback——會把拼錯的 API 變成 200 HTML；將 dist 全量提交 git——生成物噪音與 source/build 漂移。

### D6：Speclink 維運主控台採高密度、低動態、可存取設計

視覺沿用 Desktop 的 light/dark semantic tokens：青綠為 primary／focus，neutral surface 表示層級，success／warning／destructive 除顏色外皆附 icon 與文字。Noto Sans TC 隨資產打包，URL、token prefix、版本與時間使用等寬字體與 tabular numbers。Desktop viewport 顯示 icon＋label 的固定側欄；窄於 1024px 以 shadcn Sheet 取代側欄並保留可見 trigger，不在同一層混用 bottom navigation。

管理導覽固定為總覽、使用者、專案與儲存庫、憑證、資料操作、系統狀態、稽核紀錄；帳號入口與登出空間上分離。專注流程殼只顯示 Speclink identity、步驟／任務、主要表單與返回路徑。每頁恰有一個視覺 primary action；資料以 semantic Table 呈現，手機改為可換行 row/card，不產生整頁水平捲動。

所有互動可用鍵盤完成；第一個元素為 skip link，focus ring 2px 以上，heading 連續，icon-only control 有 aria-label，輸入保留 label／helper text／autocomplete，錯誤靠近欄位且透過 role=alert 宣告。互動目標至少 44×44px，正文至少 16px，正常文字對比至少 4.5:1。動畫限 150–250ms 的 opacity／transform 狀態連續，`prefers-reduced-motion` 下停用；不使用玻璃、裝飾漸層、浮動動作鈕、脈動點或自動播放。

替代方案：照 `ui-ux-pro-max` 的 generic high-tech boutique 套用 glass／gradient／FAB——會與高風險維運資訊競爭；完全複製 Desktop 看板 layout——Server admin 的資訊架構與 SDD 看板不同；只做桌面寬度——setup 與 invite URL 常由手機開啟，會重現不可用入口。

### D7：TDD 與交付 gate 同時涵蓋 Web、Rust、Desktop 共用 theme

每個行為依紅→綠→重構：React route／form／a11y 先寫 Vitest + Testing Library 失敗測試；browser API、session、fallback 與 asset headers 先寫 `speclink-server` integration test；production build、Docker 與 release 先寫腳本／workflow contract test 或可重現 smoke。React error boundary 分隔 app shell 與 route，route lazy import 保護初載 bundle。

Root `test:all` 納入 `packages/ui`、`apps/desktop`、`apps/server-web` 與 Rust workspace，且在 Rust release／asset integration 前先完成 Web production build。主 CI 三平台執行三個 React workspace 測試、Web build、Rust workspace 與現有 Node SDK gate；`apps/server-web` 與共用 UI 測試輸出不得含未等待的 React act warning。Docker multi-stage 以 Node stage 產 dist，再由 Rust stage embed，runtime 仍只有 non-root server binary。

替代方案：只在 release workflow 建 Web——PR 可合入壞掉 UI；只跑 jsdom 不做 production build——無法抓 Tailwind source、route chunk 或 asset manifest 問題；每個 workspace 各自複製 theme——無法證明 Desktop 沒有設計漂移。

### D8：依功能面漸進切換，最終刪除舊 HTML

遷移順序固定為：(1) workspace、shared theme、SPA asset serving；(2) session／login／root routing；(3) setup／invite 自動登入；(4) account／PAT／device family／activation；(5) admin view models 與 mutations；(6) 全路由切換、移除 HTML renderer／form handler、release／docs 終驗。每一步只在對應 Rust integration、React behavior、security regression 綠後切換該 browser route。

沒有 identity schema、TeamStore 或設定 migration。Rollback 使用上一版 server binary／image；資料與 session schema 相容。若新 SPA release 發生資產或呈現問題，可回退 binary，不需要資料修復。舊 HTML 不以 feature flag 永久保留；切換前的 git revision 就是短期 rollback surface。

替代方案：一次刪除全部 HTML 後補 API／頁面——紅燈面過大且難定位；永久 feature flag 雙跑——擴大安全與測試矩陣；先做漂亮畫面再補 API——會迫使 mock contract 反向塑造 Rust 安全邊界。

## Implementation Contract

**Behavior**

- 所有已定義 browser URL 可直接開啟與重新整理，不要求使用者手打隱藏 endpoint；未登入、一般成員、管理員與 token-gated flow 看到相符殼層與導覽。
- Setup／invite 成功後 session cookie 已存在並直接進角色目的地；login 依 device code、safe return、role home 的固定優先序導向。
- Admin 的七個目的地均可由側欄到達；一般成員沒有管理導覽，直接開 admin route 得到明確 403。
- 每個 mutation 有 loading、單次提交、success 或可恢復 error；破壞性動作先確認，PAT 明文只顯示一次。
- Server binary 與 Docker image 在無 Node、無外部靜態檔服務與無外網字型的 runtime 環境仍可載入完整 SPA。

**Interface / data shape**

- Browser API base 為 `/api/speclink/v1/web`；成功 envelope 為 `{data: T}`，失敗 envelope 為 `{error:{code,message,fieldErrors?}}`，所有 JSON 欄位 camelCase。
- Session payload 為 `{authenticated:boolean,user:null|{id,email,display,admin},home:string}`；成功 login／setup／invite mutation 回 `destination`，setup completion 另回 `{connection:{publicUrl,projectKey,repoKey}}`。
- Account payload 分開回 user、PAT metadata、web sessions、device families；PAT create 只有首次 response 含 plaintext。
- Admin view models 只回頁面需要的 metadata 與 eligibility；祕密、hash、token、password、refresh credential 永不出現在讀取 payload。
- `packages/ui` 不 import Tauri、browser API 或 app store；`apps/server-web/src/api/client.ts` 是唯一 raw HTTP 呼叫入口，route／page 只呼叫其 typed operations。

**Failure modes**

- Same-origin 檢查失敗回 403；未登入 browser API 回 401；非 admin 回 403；invalid credentials 保持 email 不可枚舉的統一 message。
- Invalid／expired／consumed setup 或 invite token 維持不可區分；失敗不建立 session、不消耗仍有效 token、不回傳內部 reason。
- Store 不健康只降級 overview／system／data 的 store 區塊，不使 identity 管理失效。
- 未知 API／asset／browser route 回真正 404；asset manifest 缺漏使 build 失敗，不允許 runtime 半成品。
- Route chunk 或 render error 由 route error boundary 顯示可重試訊息，不白屏；401 使 SPA 回 login 並保留 safe return path。

**Acceptance criteria**

- `apps/server-web` tests 覆蓋 root／role／returnTo／device priority、setup、invite、account、admin navigation、loading、field errors、403、404、focus、keyboard、mobile Sheet、reduced motion 與 destructive confirmation；production build 成功且 chunks 帶 hash。
- `speclink-server` integration tests 覆蓋 browser API status／envelope、cookie flags、Origin、admin gate、token indistinguishability、single-point action/audit、PAT plaintext、fallback allowlist、MIME、cache headers 與 CSP。
- `packages/ui` 與 `apps/desktop` tests／build 證明 theme 抽取與新增原語沒有 Desktop 視覺 token／互動退化。
- Root `test:all`、三平台 CI、server Docker smoke、release binary smoke 全綠；smoke 於無 dist 目錄的 runtime 啟動 binary，GET `/login` 載入 index 與 hashed assets，GET 未知 `/api` 回 404。
- 手動 viewport 驗收至少包含 375、768、1024、1440px；light／dark、keyboard-only、200% zoom、reduced-motion 均可完成 login、setup、invite 與一項 admin mutation。

**Scope boundaries**

- In scope：`apps/server-web`、`packages/ui` 共用原語/theme、`speclink-server` browser API／assets／route、相關 tests、root／CI／Docker／release／部署文件。
- Out of scope：SDD board、Desktop 領域重設、core／cli／protocol／store／identity schema、獨立前端部署、PWA／offline、設定與技能。
- Storage decoupling：Web 只經 Axum adapter 與既有 identity／admin／store boundary 取資料，不新增 filesystem 或 database adapter，也不讓 UI 依賴特定 TeamStore driver。

## Risks / Trade-offs

- [Cargo release 依賴先完成 Web build] → root、CI、Docker 與 release 都以同一順序建置；缺 manifest 立即 fail closed 並提供可執行的修正訊息。
- [Browser API 與既有 bearer API 出現行為漂移] → 兩者只做 auth／view-model adapter，mutation 共用 single-point action；跨入口 integration test 比對效果與 audit。
- [SPA fallback 吞掉 API 拼字錯誤] → path allowlist 與 `/api` 排除測試鎖定 404，禁止無條件 catch-all。
- [自動登入擴大 token 成功交易] → 只在帳號／setup transaction 成功後建立 session；cookie flags、token consumption 與 failure rollback 有 integration tests。
- [共用 theme 抽取使 Desktop 外觀退化] → token snapshot／元件測試與 Desktop build 先行，theme 值不在本 change 改色，只改所有權。
- [大範圍切換造成難以回歸] → 依 D8 六階段 TDD 切面推進，每面可獨立驗證；最終才刪舊 renderer。
- [Admin 表格在窄螢幕資訊過密] → 優先內容欄保留，次要 metadata 轉 stacked row，禁止整頁橫向捲動並以 375px 行為測試。

## Migration Plan

1. 建立 Web workspace、共用 theme／原語與 production asset build，不切換既有 browser route。
2. 建 browser session API 與 SPA shell，先切 `/`／`/login`；保留既有 setup/admin HTML。
3. 建 setup／invite JSON API 與 React flow，驗證 auto-session／token 語意後切換對應 route。
4. 建 account／activation flow，驗證 PAT／device security 後切換。
5. 建 admin read／mutation API 與七頁 SPA，逐頁驗證後切換 `/admin/*`。
6. 移除舊 HTML renderer／form handler，跑全量 security、CI、Docker、release 與 viewport 驗收並更新部署文件。
7. 發布後若需 rollback，部署上一版 binary／image；因沒有 schema 或設定 migration，既有 identity／TeamStore 資料可直接沿用。

## Open Questions

無；框架、部署、覆蓋範圍、自動登入與導向優先序皆已由 `web-service-navigation-redesign` 討論結論化。

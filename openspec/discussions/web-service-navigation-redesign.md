---
topic: 目前前端專案的設計，是完全純文字的web服務，我希望可以設計一下，因為有好幾個端點都要自己打URL才能夠進去，完全不符合人性，所以請你幫我重新設計一下
slug: web-service-navigation-redesign
status: promoted
promoted_to: web-service-navigation-redesign
created: 2026-07-24
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 目前前端專案的設計，是完全純文字的web服務，我希望可以設計一下，因為有好幾個端點都要自己打URL才能夠進去，完全不符合人性，所以請你幫我重新設計一下

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

畫面顯示 Speclink Server 初始設定完成頁只有純文字，管理員還必須記住 /admin、/account、/activate 等端點。採 assumptions 模式，因為已找到 crates/speclink-server/src/web.rs、admin.rs、setup.rs、app.rs 與 apps/desktop/src/index.css，可判斷現有路由、角色與視覺基礎。現有 Web 以 Rust/Axum 直接拼接 HTML；沒有同主題的既有 change 或 discussion。第一輪使用者修正原先維持 server-rendered HTML 的假設，明確希望評估 React SPA、shadcn/ui 與 Tailwind CSS。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-24)

**Focus**: Server Web 是否改採 React SPA、shadcn/ui 與 Tailwind CSS
**Position**: 採 React SPA 作為重新設計方向，保留 Rust/Axum 為權限與資料邊界。
- 現有多頁管理流程已有角色導覽、表格、表單、深連結與回饋需求，SPA 能用共用應用殼層與路由消除手打 URL。
- shadcn/ui 提供可控且具無障礙基礎的元件原語，Tailwind CSS 可沿用 Desktop 的語意色彩 token；兩者不應成為套版外觀，仍須建立 Speclink 專屬設計系統。
- `/setup`、`/invite/:token`、`/login`、`/activate` 與 `/admin/*` 仍保留可直接開啟的 URL，SPA 只改善導覽，不犧牲深連結。
- 現有 session cookie、same-origin 與 admin gate 必須留在伺服器端，React 不承擔真正的授權判斷。
**Ruled out**: 繼續擴充 Rust 字串拼接 HTML；共用殼層、狀態回饋與響應式導覽會持續重複且難以維護。
**Open**: SPA 要由 Rust binary 內嵌靜態資產或獨立部署；現有 HTML form handler 與新 JSON API 的遷移邊界；初始設定完成後的首次導向。

### Round 2 — assumptions (2026-07-24)

**Focus**: React SPA 的部署與交付邊界
**Position**: Vite 產物內嵌進 speclink-server，由同一個 Rust 執行檔提供 SPA 與 JSON API。
- 單一執行檔符合現有自架 Server 的交付方式，不增加獨立前端服務、反向代理或資產版本協調。
- SPA 與 API 維持 same-origin，沿用 HttpOnly session cookie、Origin 檢查與伺服器端管理員權限判斷。
- Rust 路由須區分 API／下載等實體端點與 SPA browser routes；合法的 SPA 深連結重新整理時回傳 index.html，未知 API 仍維持正確 404。
- 前端建置必須成為 speclink-server release 的可重現步驟，執行檔只內嵌已完成且帶內容雜湊的靜態產物。
**Ruled out**: 前端獨立部署；會引入 CORS、cookie、CSRF、反向代理、前後端版本錯配與第二份部署說明。
**Open**: 現有 Rust HTML form handlers 與新 JSON API 的遷移邊界；初始設定完成及登入後的角色導向。

### Round 3 — assumptions (2026-07-24)

**Focus**: SPA 覆蓋範圍與舊 Rust HTML 的生命週期
**Position**: 所有瀏覽器頁面最終全面改由 React SPA 呈現，實作採分階段切換。
- `/setup`、`/invite/:token`、`/login`、`/activate`、`/account` 與 `/admin/*` 共用同一設計系統、路由與互動回饋。
- 各頁先補齊 same-origin JSON API、React 頁面與行為測試，再切換 browser route；切換前可暫留原 handler 降低一次性風險。
- 功能等價後移除 `web.rs`、`admin.rs`、`setup.rs` 中只為舊 HTML 存在的字串 renderer 與 form handler，保留底層 single-point domain actions、權限 gate 與 API。
- `/api/*`、下載與認證協定端點不進 SPA fallback；只有已定義的 browser route 與前端子路徑回傳內嵌 index.html。
**Ruled out**: 永久保留登入／初始設定的 Rust HTML 與管理 SPA 並存；會形成兩套設計、表單驗證、錯誤文案與無障礙維護面。
**Open**: 建立管理員或接受邀請後是否自動登入；登入與根路徑的角色導向。

### Round 4 — assumptions (2026-07-24)

**Focus**: 初始設定與接受邀請完成後的登入體驗
**Position**: 成功建立管理員或受邀帳號後，由 Rust 立即建立 HttpOnly session 並依角色進入 SPA。
- 第一位管理員完成 setup 後直接進入 `/admin`；初始 Public URL、Project、Repo 與邀請下一步改在首次管理總覽的歡迎區塊呈現並可複製。
- 接受邀請並設定密碼後，一般成員進入 `/account`，具管理權的成員進入 `/admin`。
- session 仍由伺服器建立並設定 Secure、SameSite 與 HttpOnly cookie；React 只接收成功狀態與下一個同來源路徑。
- setup token 或 invite token 在成功交易中立即消耗；無效、過期、已使用或驗證失敗時不得建立 session。
**Ruled out**: 設定完密碼後再要求使用者回到登入頁重輸一次；增加摩擦但沒有新增身分證明價值。
**Open**: 根路徑與一般登入後的角色導向，以及裝置核准 return path 的優先順序。

### Round 5 — assumptions (2026-07-24)

**Focus**: 根路徑、一般登入、受保護深連結與裝置核准的導向優先順序
**Position**: 採伺服器驗證角色的可預測導向，深連結與裝置核准目的地優先於角色首頁。
- 未登入開啟 `/` 進入 `/login`；已登入管理員進入 `/admin`，一般成員進入 `/account`。
- 一般登入成功後依管理員／成員角色進入 `/admin` 或 `/account`。
- 登入請求若帶有效裝置代碼，優先返回 `/activate?user_code=XXXX-XXXX` 完成核准，不被角色首頁覆蓋。
- 未登入開啟受保護深連結時，登入後返回原頁；`returnTo` 只接受站內白名單路徑，禁止外部或協定相對 URL。
- 非管理員開啟 `/admin/*` 顯示明確 403 權限頁，不默默跳走；導覽同時不顯示無權使用的管理項目。
**Ruled out**: 所有登入一律跳固定首頁；會中斷裝置核准與使用者原本的深連結意圖。非管理員默默跳回帳號頁；會隱藏權限問題。
**Open**: 無。

## Conclusion

**Decision**: 將 Speclink Server 的所有瀏覽器介面重建為 Vite + React + TypeScript SPA，使用 shadcn/ui 與 Tailwind CSS，建置產物內嵌進 `speclink-server` 單一執行檔；Rust/Axum 保留 session、權限、same-origin、資料驗證與 JSON API 的唯一權威。
- 新增 `apps/server-web` 擁有路由、應用殼層、互動狀態、表單回饋與響應式呈現；`crates/speclink-server` 擁有靜態資產服務、SPA fallback、認證授權與 API。
- 只設一個前端 HTTP 資料層，統一處理 credentials、JSON 錯誤、登入失效、loading 與 mutation 後更新；不得讓各元件散落 fetch 或再疊一層無行為的 Rust proxy。
- `/setup`、`/invite/:token`、`/login`、`/activate`、`/account`、`/admin/*` 全面 SPA，分階段補齊 JSON API 並切換；達成功能等價後移除舊 Rust HTML renderer 與 form handler。
- 設定與邀請成功後自動建立 HttpOnly session；角色首頁、受保護深連結與裝置核准依已確認的優先順序導向，`returnTo` 僅允許站內白名單。
- 視覺採高資訊密度、低動態的 Speclink 維運主控台：沿用青綠語意 token、亮暗色主題、Noto Sans TC 與等寬資料字體；桌面側欄、行動版 Sheet，狀態色含文字／圖示，不使用玻璃質感、裝飾漸層、浮動動作鈕或脈動效果。
- 無障礙為驗收條件：鍵盤操作、跳至主要內容、路由換頁焦點、可見標籤與欄位旁錯誤、44px 觸控目標、4.5:1 文字對比、reduced-motion、危險操作確認與非靜默成功／失敗回饋。
**Rationale**: 多個角色化頁面、表格與表單需要一致導覽和狀態回饋；SPA 解決必須記 URL 與重複殼層，內嵌資產維持自架 Server 的單一交付、same-origin 安全與前後端版本一致。介面深度成立：刪除 `apps/server-web` 會失去整個 Web 體驗但 API/CLI 仍可運作；刪除單一 HTTP 資料層則會失去統一的 session、錯誤與更新語意。
**Rejected alternatives**: 繼續拼接 Rust HTML，因共用導覽、互動與無障礙成本會持續重複；前端獨立部署，因增加 CORS、CSRF、cookie、反向代理與版本錯配；永久混合 SPA/HTML，因形成兩套設計與驗證；只在 React 隱藏管理功能，因客戶端不是授權邊界；通用高科技 SaaS 裝飾，因干擾維運資訊層級。
**Deferred**: 無。
**Capture to**: proposal.md、design.md、specs、tasks.md。
**Next**: $speclink-propose --from-discussion web-service-navigation-redesign

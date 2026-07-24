## Why

Speclink Server 目前以 Rust 字串直接輸出無樣式 HTML，初始設定、帳號、裝置核准與六個管理頁面缺乏共同導覽，使用者必須記住並手動輸入 URL。對透過 AI 代理執行 Remote SDD 的開發者、PO、PM 與自架管理員而言，這使首次 setup、邀請成員、建立憑證及維運 Remote workflow 的入口難以發現，也讓錯誤、載入、成功與權限狀態無法形成一致體驗。

## What Changes

- 新增完整的 Speclink Server Web Console：以 Vite、React、TypeScript、Tailwind CSS 與 `packages/ui` 既有 shadcn/ui 原語實作單頁應用程式，統一承載 `/setup`、`/invite/:token`、`/login`、`/activate`、`/account` 與 `/admin/*`。
- 建立持續可見且依角色裁切的導覽：根路徑與登入成功後由伺服器驗證角色，管理員進入 `/admin`、一般成員進入 `/account`；受保護深連結與有效裝置碼回程優先，`returnTo` 僅接受站內白名單路徑；非管理員開啟 `/admin/*` 明確回 403。
- 將 setup、邀請、登入、帳號自助與管理介面所需資料／動作補成 same-origin JSON API；React 只負責呈現，HttpOnly session、同源檢查、admin gate、資料驗證、audit 與 single-point domain action 仍由 Rust/Axum 掌權。
- setup 與邀請成功後立即建立 session 並依角色進入 SPA；setup 的 Public URL、Project、Repo 與邀請下一步改在首次管理總覽的歡迎區塊呈現並可複製。
- 將 Vite 產物以內容雜湊資產內嵌進 `speclink-server` 單一執行檔；合法 browser route 的重新整理回傳 SPA shell，API、下載、認證協定與未知 API 不得被 SPA fallback 吞掉。
- 以一個 Web HTTP adapter 統一 credentials、JSON 錯誤、登入失效、載入狀態與 mutation 後更新；不得在 React 元件散落 `fetch`，也不新增只轉送呼叫的 Rust proxy。
- 沿用 Speclink 青綠語意色彩、Noto Sans TC、等寬資料字體與亮／暗主題，採高資訊密度、低動態的維運主控台；桌面使用側欄、窄螢幕使用 Sheet。鍵盤導覽、skip link、路由換頁焦點、可見表單標籤、欄位旁錯誤、44px 觸控目標、4.5:1 文字對比、reduced-motion、危險操作確認及非靜默回饋皆為驗收條件。
- 分階段以測試先行補齊 JSON API 與 SPA 頁面；每一面功能等價後切換 browser route，最終移除舊 Rust HTML renderer 與 form handler，不永久維護雙介面。
- **BREAKING（僅舊 Web 呈現面）**：既有 browser route 不再保證傳回原本的 server-rendered HTML body，舊 HTML form POST 由 JSON API 取代；既有 URL、CLI、Client Protocol 與正式 API 的安全／領域語意維持。

## Non-Goals

- 不把 changes、specs、discussions 看板或編輯功能加入 Server 管理面；規格內容仍由 Desktop／既有 Client Protocol 消費。
- 不獨立部署前端，不新增 CORS 模式、第二個公開服務、PWA、service worker 或離線快取。
- 不以 React 的 route guard 或隱藏導覽取代伺服器授權；客戶端永遠不是權限權威。
- 不重新設計 Desktop 領域畫面；只把跨 Desktop／Web 共用的 shadcn 原語與語意 theme token 留在 `packages/ui`，Desktop 可觀察外觀不得退化。
- 不變更 `speclink-core`、`speclink-cli`、CLI 子指令／旗標／stdin／exit code、人眼輸出或 `--json` shape；不新增 `.speclink.yaml`、`openspec/config.yaml` 欄位，也不修改 Claude／Codex 技能與注入區塊。
- 不採玻璃質感、裝飾漸層、浮動動作鈕、脈動狀態點或與維運資訊無關的動畫。

## Capabilities

### New Capabilities

- `server-web-console`: 內嵌 React SPA 的應用殼層、角色導覽、深連結、共用設計系統、響應式行為、可存取性與 browser/API fallback 邊界。

### Modified Capabilities

- `server-admin`: 管理頁由 HTML form 入口改為 SPA 呼叫完整 same-origin admin JSON API，保留 admin gate、single-point action、憑證祕密邊界、audit 與 store 失聯降級語意。
- `server-setup`: setup 改為可續作的 JSON 驅動 SPA，完成時原子耗用 token、建立管理員 session，並把連線資訊帶到首次管理總覽。
- `server-identity`: 邀請接受後自動建立 session；登入、根路徑、受保護深連結與角色首頁採安全導向，帳號／PAT／session 自助面改由 JSON API 支援。
- `server-release`: server binary、Docker image 與 release workflow 必須包含同版本、已建置且可離線提供的 SPA 資產。
- `delivery-baseline`: root 全量驗證與三平台 CI 納入 `apps/server-web` 的測試與 production build，React act warning gate 同時涵蓋新的 Web workspace。

## Impact

- Affected specs: `server-web-console`（新增）；`server-admin`、`server-setup`、`server-identity`、`server-release`、`delivery-baseline`（修改）。`server-device-auth` 的既有裝置碼保留與明確確認語意不變，由新 SPA 實作回歸保護。
- Affected systems: Server browser surface、same-origin session API、管理 JSON API、npm workspace、共用 UI theme、server binary／Docker／release build 與 CI。
- Compatibility: `speclink-core` 與 `speclink-cli` 不受影響；CLI 人眼／`--json`、Client Protocol、正式 API reason、identity schema 與 TeamStore contract 均不變。舊 Web HTML body 與 form submission 不是保留介面，browser URL 則維持可深連結。
- Affected code:
  - New: `apps/server-web/package.json`、`apps/server-web/index.html`、`apps/server-web/vite.config.ts`、`apps/server-web/vitest.config.ts`、`apps/server-web/tsconfig.json`、`apps/server-web/src/main.tsx`、`apps/server-web/src/App.tsx`、`apps/server-web/src/index.css`、`apps/server-web/src/api/client.ts`、`apps/server-web/src/routes/router.tsx`、`apps/server-web/src/__tests__/app.test.tsx`、`packages/ui/src/theme.css`，以及所需的共用 shadcn/ui 原語檔。
  - Modified: `package.json`、`package-lock.json`、`packages/ui/package.json`、`packages/ui/src/index.ts`、`apps/desktop/src/index.css`、`crates/speclink-server/Cargo.toml`、`crates/speclink-server/src/app.rs`、`crates/speclink-server/src/web.rs`、`crates/speclink-server/src/setup.rs`、`crates/speclink-server/src/admin.rs`、相關 `crates/speclink-server/tests/` 測試、`crates/speclink-server/Dockerfile`、`.github/workflows/ci.yml`、`.github/workflows/release.yml`、`docs/server-deployment.zh-TW.md`。
  - Removed: 無整檔刪除；舊 HTML renderer 與 form-only handler 會自上述 Rust 模組移除。

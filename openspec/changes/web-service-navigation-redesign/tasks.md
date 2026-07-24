<!--
Each task description MUST state:
- the behavior or contract being delivered (what is observably true when the
  task is complete), and
- the verification target that proves completion (test, CLI invocation,
  analyzer check, manual assertion, or content review on a generated artifact).

File paths are supporting context for locating the work, never the task
itself. "Edit file X" is not a valid task — it is missing both behavior and
verification.
-->

## 1. Web workspace、共用 theme 與內嵌資產

- [x] 1.1 RED：在 `packages/ui`、`apps/server-web` 與 `crates/speclink-server/tests/web_assets.rs` 先寫共用 theme 不改變 Desktop token、Vite chunks 具 hash、合法 browser route 回 shell、未知 API／asset 回 404、MIME／cache／CSP 正確的失敗測試；分別執行 `npm test -w packages/ui`、`npm test -w apps/server-web`、`cargo test -p speclink-server --test web_assets`，確認因 workspace、shared theme 或 embedded assets 尚未存在而以預期原因失敗。 <!-- speclink-task:tsk_01KY98N6287J1SAGNWZJHSWE0N -->
- [x] 1.2 GREEN／REFACTOR：依「D1：`apps/server-web` 擁有 SPA，`packages/ui` 擁有共用設計原語」與「D5：Vite hashed assets 內嵌 binary，fallback 僅服務明確 browser routes」建立 React＋Vite＋TypeScript＋Tailwind workspace、抽出 `packages/ui/src/theme.css` 並補最小 shadcn/ui 原語，在 `crates/speclink-server` 實作「SPA 資產與 fallback 具可驗證的安全邊界」；執行 1.1 三組測試、`npm run build -w apps/server-web` 與 `npm run build -w apps/desktop`，確認 hashed assets 可內嵌、Desktop theme／互動無退化且全部轉綠。 <!-- speclink-task:tsk_01KY98N62846QQHYGT9NTBTRPT -->

## 2. Browser API、session 與安全導向

- [x] 2.1 RED：在 `crates/speclink-server/tests/web_session.rs` 先寫 `{data}`／`{error}` camelCase envelope、cookie flags、同源 mutation、email 不可枚舉、active session、device code／safe `returnTo`／role home 優先序、外部 redirect 與一般成員 admin destination 拒絕的失敗 integration tests；執行 `cargo test -p speclink-server --test web_session`，確認因 `/api/speclink/v1/web` contract 尚未實作而失敗。 <!-- speclink-task:tsk_01KY98N6286Y9YKPYZ88HHKW28 -->
- [x] 2.2 GREEN／REFACTOR：依「D2：browser JSON API 使用獨立 same-origin session 邊界」與「D3：導向由伺服器計算，深連結與裝置核准優先」在 `crates/speclink-server/src/web.rs`、`auth.rs` 與 router 實作 session／login／logout typed response、origin-first guard 與 Server-owned destination，使「導向遵守伺服器裁決與安全優先序」及「本機密碼登入與 session 安全屬性」成立，且 bearer admin API 不接受 cookie；執行 2.1 測試與既有 `auth_pat`、`auth_device`、`web_account` tests，確認新舊認證邊界全綠。 <!-- speclink-task:tsk_01KY98N628QDG7HRGQ6QA67BJH -->

## 3. SPA 殼層、導覽與互動狀態

- [x] 3.1 RED：在 `apps/server-web/src/__tests__/app.test.tsx` 先寫三種殼層、七個 admin 目的地、一般成員裁切、direct deep link、route focus、loading／empty／403／error boundary、field error 保留、單次提交、AlertDialog、mobile Sheet、skip link、keyboard 與 reduced-motion 的失敗 Vitest＋Testing Library tests；執行 `npm test -w apps/server-web`，確認因 router、typed client 與 layouts 尚未完成而失敗。 <!-- speclink-task:tsk_01KY98N628JFDH1HW70MB6K8SB -->
- [x] 3.2 GREEN／REFACTOR：依「D6：Speclink 維運主控台採高密度、低動態、可存取設計」實作 `apps/server-web/src/api/client.ts` 唯一 raw HTTP 入口、lazy routes、專注／帳號／管理殼與 responsive navigation，使「全部 browser route 由單一 SPA 提供可發現導覽」、「共用設計系統維持高密度可存取體驗」與「Browser API 互動狀態一致且可恢復」成立；執行 3.1 tests、`npm run build -w apps/server-web` 與 axe 可存取性 assertions，確認沒有白屏、重複提交、整頁水平捲動或未標示 control。 <!-- speclink-task:tsk_01KY98N628ARAP219YWWJWWRD9 -->

## 4. Setup 與 invite 自動登入

- [x] 4.1 RED：在 `crates/speclink-server/tests/web_setup.rs`、`web_invite.rs` 與對應 React route tests 先寫 setup 可續作四要素、同源拒絕、完成回 connection＋`/admin?welcome=1`、重送不複製資料、invite token 不可區分、一般／admin destination、自動 session 與 session failure recovery 的失敗測試；執行指定 Rust tests 與 `npm test -w apps/server-web -- setup invite`，確認舊 HTML／form 行為無法滿足 JSON 與自動登入契約。 <!-- speclink-task:tsk_01KY98N628T38RM41DFD9REASA -->
- [x] 4.2 GREEN／REFACTOR：依「D8：依功能面漸進切換，最終刪除舊 HTML」的第三階段，在 `crates/speclink-server/src/setup.rs`、identity browser adapter 與 SPA routes 實作「setup 流程完成開箱四要素」及「邀請一次性且到期失效」，保持既有交易、token、audit 與 public URL 邊界，只在 4.1 Rust／React tests 全綠後切換 `/setup`、`/invite/:token`；再執行既有 `setup`、`invite`、`admin_e2e` regression tests 確認 CLI invite 與 binding 未退化。 <!-- speclink-task:tsk_01KY98N628SRGHGV58ZC5QFMVM -->

## 5. Account、PAT 與 device activation

- [ ] 5.1 RED：在 `crates/speclink-server/tests/web_account.rs`、device authorization tests 與 SPA account／activation tests 先寫 account summary 僅含本人 metadata、PAT plaintext 只出現一次、PAT／Web session／device family 撤銷即時生效、activation 明確確認、401 safe return 與 secrets 不出現在 read payload 的失敗測試；執行指定 Rust tests 與 `npm test -w apps/server-web -- account activate`，確認 browser JSON account surface 尚未滿足契約。 <!-- speclink-task:tsk_01KY98N628HDAVA6C91VJZ1Z5N -->
- [ ] 5.2 GREEN／REFACTOR：在 `crates/speclink-server` identity adapter 與 `apps/server-web` account／activation routes 實作「帳號 browser API 保持憑證祕密邊界」，復用既有 PAT、session 與 device family domain actions，不新增 refresh credential 或 hash 的讀取路徑；執行 5.1、既有 `server-device-auth` 與 `auth_pat` tests，確認 plaintext、撤銷、confirmation 與登入失效行為全綠。 <!-- speclink-task:tsk_01KY98N628HXFBPB0JHTPMYVHT -->

## 6. Admin view models 與 single-point mutations

- [ ] 6.1 RED：在 `crates/speclink-server/tests/admin_web_api.rs`、`admin_three_entry.rs` 與 SPA admin tests 先寫 session-only 401／403、origin-first guard、七個獨立 view models、secret exclusion、action eligibility、Store 不健康降級、三入口等效 mutation／audit source、最後 admin 保護、registry key 不可改與破壞性確認的失敗測試；執行指定 Rust tests 與 `npm test -w apps/server-web -- admin`，確認 browser admin API 與七頁尚未存在而失敗。 <!-- speclink-task:tsk_01KY98N628A5NJ3W0A1HZETF0M -->
- [ ] 6.2 GREEN／REFACTOR：依「D4：管理 SPA 補齊 view-model API，所有 mutation 重用既有 domain action」在 `crates/speclink-server/src/admin.rs`、browser adapter 與 SPA admin routes 實作「admin 門禁前置且非 admin 一律 403」、「管理動作三入口同一實作且功能完備」與「管理 browser API 提供最小且完整的頁面 view model」；執行 6.1、既有 `admin_api`、`admin_pages`、`admin_system`、`audit` tests，確認 bearer API／CLI 不變、audit source 正確且 store degradation 不阻斷 identity 管理。 <!-- speclink-task:tsk_01KY98N62893WB91FFDWE5SKW5 -->

## 7. 全面切換、交付 gate 與安全驗收

- [ ] 7.1 RED：先在 root scripts、workflow contract tests 與 release smoke fixture 寫入三個 React workspace tests、Web production build 先於 Rust、缺 assets fail closed、Docker 最終層無 Node、release binary 無 `dist` 仍載入 `/login`、act warning 零命中的失敗驗證；執行 contract tests、`npm run test:all`、Docker／release smoke，確認舊 build 順序與交付物尚未滿足新 gate。 <!-- speclink-task:tsk_01KY98N628AFBFQY3DKZK02D8E -->
- [ ] 7.2 GREEN／REFACTOR：依「D7：TDD 與交付 gate 同時涵蓋 Web、Rust、Desktop 共用 theme」與「D8：依功能面漸進切換，最終刪除舊 HTML」更新 root `package.json`、CI、release、`crates/speclink-server/Dockerfile` 與部署文件，移除已被 SPA 覆蓋的 HTML renderer／form-only handlers，使「Server 交付物內嵌同版本 SPA 資產」、「root 單一指令全量驗證」、「CI 執行完整測試」及「測試輸出無 React act 警告」成立；執行 7.1 驗證與 `cargo test --workspace`，確認單一 binary／image、五面 root gate、三平台定義與 rollback 文件一致。 <!-- speclink-task:tsk_01KY98N628ETRC3BKW9R0F9HYH -->
- [ ] 7.3 AUDIT／終驗：對 browser API、token、`returnTo`、Origin、admin、secret payload、fallback 與 build defaults 執行 `speclink instructions --skill audit` 的 Scoundrel／Lazy／Confused checklist並修正 Critical／High sharp edges；接著執行 `npm run test:all`、`cargo build --release -p speclink-server`、Docker smoke，並以 375／768／1024／1440px、light／dark、keyboard-only、200% zoom、reduced-motion 手動完成 login、setup、invite 與一項 admin mutation，確認 D1–D8、全部 delta scenarios 與可存取性驗收均有綠燈證據。 <!-- speclink-task:tsk_01KY98N628QZ2RMX4BEKE29TFR -->

<!--
Each task description MUST state:
- the behavior or contract being delivered (what is observably true when the
  task is complete), and
- the verification target that proves completion (test, CLI invocation,
  analyzer check, manual assertion, or content review).

File paths are supporting context for locating the work, never the task
itself. "Edit file X" is not a valid task — it is missing both behavior and
verification.
-->

## 1. 結構化遠端開啟錯誤

- [x] 1.1 RED — 依「Decision 2：remote_open 保留 machine-readable error，UI 不比對英文訊息」為「remote_open 失敗保留 machine-readable reason」撰寫失敗測試：`apps/desktop/src-tauri/tests/remote_data.rs` 必須先證明 transport、401、403、404 與未知失敗尚未穩定輸出 camelCase `message`／`reason`／`status`，且 payload 不得包含 token、PAT、authorization header 或 Keychain 值；執行 `cargo test -p speclink-desktop --test remote_data`，確認測試因缺少新契約而 RED。 <!-- speclink-task:tsk_01KY1JSWBZEYV7M1D36DMN8GEQ -->
- [x] 1.2 GREEN — 在 `apps/desktop/src-tauri/src/remote.rs`、`apps/desktop/src-tauri/src/lib.rs`、`apps/desktop/src/main.tsx` 與 `apps/desktop/src/session.ts` 實作最小 structured rejection 與封閉的 `unreachable`／`needs-reauth`／`access-denied`／`not-found`／`unknown` 映射，使 UI 分類只依 `reason`／`status` 與 runtime 狀態、從不比對英文 `message`；執行 `cargo test -p speclink-desktop --test remote_data` 及 `npm test -w apps/desktop -- --run src/__tests__/remoteOpen.test.ts`，確認 1.1 與舊版未知 rejection 降階案例轉為 GREEN。 <!-- speclink-task:tsk_01KY1JSWBZGX62K0Z43901VTWF -->
- [x] 1.3 REFACTOR／AUDIT — 對上述 IPC 邊界套用 sharp-edges audit 的 Scoundrel／Lazy Developer／Confused Developer 檢查，收斂共用錯誤型別並確保錯誤不靜默成功、欄位不可混淆、預設降階安全且不洩漏 credential；以 `cargo test -p speclink-desktop --test remote_data`、`npm test -w apps/desktop -- --run src/__tests__/remoteOpen.test.ts` 與人工檢視序列化欄位清單維持 GREEN。 <!-- speclink-task:tsk_01KY1JSWBZY1K6JHZM6SVYKWVB -->

## 2. 可選取復原分頁與 session 邊界

- [x] 2.1 RED — 依「Decision 1：作用中分頁與可用 session 分離，例外狀態不持久化」為「可選取的 remote 復原分頁與 session 邊界」、「handshake 成功後才建立 remote session」及「stale snapshot 與無 session 復原頁依 session 存在性分流」撰寫失敗測試：`apps/desktop/src/__tests__/session.test.ts`、`store.test.ts`、`remoteOpen.test.ts`、`remoteResilience.test.tsx` 必須覆蓋 restoring／error 可成為 active destination、無 session 不洩漏上一分頁資料、retry 原地成功、關閉清理、跨 tab 不搶 active、同 tab latest-wins、既有 session offline 保留 stale，以及 local tab 不回歸；執行這四個 Vitest 檔並確認新案例因缺少 recovery state 而 RED。 <!-- speclink-task:tsk_01KY1JSWBZ3AARG62KV5HAPGH0 -->
- [x] 2.2 GREEN — 在 `apps/desktop/src/session.ts`、`apps/desktop/src/store.ts` 與 `apps/desktop/src/main.tsx` 以 locator key 加入不持久化的 restoring／error 狀態與 request generation，允許 `activeKey` 指向無 session 分頁，但所有資料操作仍只經 `activeSession()`；retry 成功須在同 key 建立 session、不新增分頁，舊請求不得覆寫最新結果；重跑 2.1 的四個 Vitest 檔，確認所有案例 GREEN。 <!-- speclink-task:tsk_01KY1JSWBZ7AQQB3BMZK51J9ZM -->
- [x] 2.3 REFACTOR — 抽出單一 recovery state 投影與生命週期 helper，刪除重複分支並維持 local watcher、持久化 activeKey 與既有 remote worker 行為不變；執行 `npm test -w apps/desktop -- --run src/__tests__/session.test.ts src/__tests__/store.test.ts src/__tests__/remoteOpen.test.ts src/__tests__/remoteResilience.test.tsx`，確認重構後仍全綠且無 session 路徑不產生偽造 stale 資料。 <!-- speclink-task:tsk_01KY1JSWBZPM1A1QNP4K6WH8NF -->

## 3. 主視窗復原目的地

- [x] 3.1 RED — 依「Decision 3：主視窗以 recovery destination 取代 tooltip-only 錯誤」與「Decision 5：surface focus 與無障礙邊界維持平台慣例」撰寫失敗元件測試：`apps/desktop/src/__tests__/projectTabs.test.tsx` 與新增的 `apps/desktop/src/__tests__/remoteWorkspaceRecovery.test.tsx` 必須驗證錯誤 tab 仍可選取且 selected、不呈 disabled、只顯示一個狀態、tooltip 僅為短摘要，復原頁有繁中摘要、workspace／server、重新連線、設定或重新登入、可展開 technical detail、移除分頁、live region 與鍵盤可操作控制項；執行兩個測試檔，確認新案例 RED。 <!-- speclink-task:tsk_01KY1JSWBZVM316RV7PMNJMFXP -->
- [x] 3.2 GREEN — 新增 `apps/desktop/src/components/RemoteWorkspaceRecovery.tsx`，並調整 `App.tsx`、`ProjectTabs.tsx`、`i18n.tsx` 與 store actions，使 active no-session remote 分頁呈完整復原目的地；「重新認證原地復活不退 local」須從主視窗明確導向對應 connection 的伺服器設定／登入，retry 留在原頁且 technical detail 預設收合；執行 3.1 的元件測試與 `npm run build -w apps/desktop`，確認行為 GREEN 且 TypeScript build 成功。 <!-- speclink-task:tsk_01KY1JSWBZWYXX7HYM0YDZB060 -->
- [x] 3.3 REFACTOR — 統一 tab、復原頁與既有 remote banner 的狀態圖示、繁中語彙、focus-visible、ARIA live region、深淺色與 reduced-motion 表現，不新增阻塞式 modal 或長篇 tooltip；執行 `npm test -w apps/desktop -- --run src/__tests__/projectTabs.test.tsx src/__tests__/remoteWorkspaceRecovery.test.tsx src/__tests__/remoteResilience.test.tsx`，並人工以鍵盤巡覽 light／dark mode，確認狀態不只靠顏色且焦點順序可預期。 <!-- speclink-task:tsk_01KY1JSWBZV9X4C77YBTR54JQA -->

## 4. Tray 共用復原體驗

- [x] 4.1 RED — 依「Decision 4：TraySnapshot 投影共用狀態，Panel 與原生選單各自降階」、「Decision 5：surface focus 與無障礙邊界維持平台慣例」、「選單專案切換」與「面板樣式（macOS）」撰寫失敗測試：`apps/desktop/src/__tests__/tray.test.ts`、`trayPanel.test.tsx` 與 `apps/desktop/src-tauri/tests/tray_menu.rs`（若原檔不存在則新增）須覆蓋 ready／restoring／offline／needs-reauth／error、locator-key 切換、原生 recovery submenu、macOS compact recovery card、no-session 不顯示舊討論／變更、existing-session offline 保留 stale、直接 retry 不喚起主視窗，以及詳情／設定／登入才顯示並聚焦主視窗；執行相關 Vitest 與 `cargo test -p speclink-desktop --test tray_menu`，確認新案例 RED。 <!-- speclink-task:tsk_01KY1JSWBZSVWMK06D3QWKSDDE -->
- [x] 4.2 GREEN — 在 `apps/desktop/src/tray.ts`、`TrayPanel.tsx`、`store.ts`、`i18n.tsx`、`apps/desktop/src-tauri/src/tray.rs` 與 `lib.rs` 實作單一 `TraySnapshot` 狀態投影：macOS Panel 的 active no-session workspace 以復原卡取代討論／生命週期內容，原生選單以 recovery submenu 降階；直接 retry 保持 Panel 與主視窗焦點狀態，明確詳情／設定／登入才顯示並聚焦主視窗；執行 4.1 的測試，確認全部 GREEN。 <!-- speclink-task:tsk_01KY1JSWBZSFABD71HYACZV57D -->
- [x] 4.3 REFACTOR／AUDIT — 移除 Tray 的第二條查詢或平行狀態推斷，讓 Panel 與原生選單只消費同一 store snapshot，並以 sharp-edges audit 檢查 locator key、action payload、未知狀態與 focus side effect 均 fail-safe；執行 `npm test -w apps/desktop -- --run src/__tests__/tray.test.ts src/__tests__/trayPanel.test.tsx`、`cargo test -p speclink-desktop --test tray_menu` 及 `npm run build -w apps/desktop`，確認維持 GREEN。 <!-- speclink-task:tsk_01KY1JSWBZV1F8BC1D9Q1Q61EG -->

## 5. 整合與人工驗收

- [x] 5.1 執行完整 Desktop 回歸：`npm test -w apps/desktop`、`cargo test -p speclink-desktop` 與 `npm run build -w apps/desktop` 必須全數通過；驗證 CLI human／JSON、server HTTP API、`.speclink.yaml`、`speclink-core` 與 `speclink-cli` 均未改動，且既有 local workspace、ready remote 與 established-session offline 行為保持相容。 <!-- speclink-task:tsk_01KY1JSWBZKWQ467VJ9SKSR26M -->
- [x] 5.2 以實際 macOS Desktop 完成主視窗、Tray Panel 與原生 fallback 選單全鏈 GUI 驗收：從已儲存 remote tab 啟動，在 server down→error→retry→server up→原地 ready、401→needs-reauth→原地復活，以及切換 local／remote 情境逐一直接點擊；確認 warning tab 可開、無 session 時沒有舊資料、retry 不奪焦、詳情／設定／登入才聚焦主視窗、Panel 不成為 key window，並保存逐步操作與畫面證據。 <!-- speclink-task:tsk_01KY1JSWBZG0AA0W57E6TG2W1P -->
- [x] 5.3 執行 `speclink analyze remote-workspace-recovery-ux --json` 與 `speclink validate remote-workspace-recovery-ux`，Critical／Warning 必須為零且 strict validation 通過；逐項核對四份 delta spec 的所有 Scenario 已由自動測試或 5.2 GUI 證據覆蓋，完成後才可進入 archive 判定。 <!-- speclink-task:tsk_01KY1JSWBZWNNFSRJVX0CJ10TV -->

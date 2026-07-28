---
topic: 遠端登入與裝置授權的 UX 斷點（專案歸屬、裝置碼、授權後銜接）
slug: remote-login-ux-gaps
status: promoted
promoted_to: remote-login-ux-gaps
created: 2026-07-27
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 遠端登入與裝置授權的 UX 斷點（專案歸屬、裝置碼、授權後銜接）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者以截圖回報遠端（Remote Store）路徑上三個 UX 斷點：(1) 一般使用者在 web 端看不到自己隸屬哪些專案；(2) 從 desktop 登入時瀏覽器要求輸入裝置碼，但 desktop 端從未顯示該碼；(3) 完成核准後回到畫面沒有任何自動銜接（不自動選取、不自動開工作區），流程出現斷點。

模式：assumptions——codebase scout 找到遠超三個相關檔（apps/server-web 的 AppRoutes/ActivatePage/LoginPage/AccountPage/api client、apps/desktop 的 ServersPanel/store.ts、apps/desktop/src-tauri/src/connections.rs、crates/speclink-server 的 web.rs/admin.rs/identity.rs、crates/speclink-remote/src/device.rs），足以直接形成立場。

限制：截圖在 assistant 端只收到佔位標記，畫面內容無法檢視，所有現況判讀均由原始碼推得並經使用者確認。

相關規格：server-identity、server-device-auth、desktop-connections、remote-data-source。目前無進行中的變更（speclink list 為空）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-27)

**Focus**: 三個回報斷點在程式碼中的實際缺口各是什麼？
**Position**: 三者都成立，但根因與使用者直覺不同——碼其實一路預填、真正缺的是呈現與銜接：
- 議題 1（看不到隸屬專案）＝ web 端無此面。`apps/server-web/src/routes/AppRoutes.tsx` 非 admin 只有 `/account`；`api/client.ts` 的 `AccountSummary` 僅 user/pats/sessions/deviceFamilies。後端已有能力：`crates/speclink-server/src/admin.rs:698` 呼叫 `list_memberships`，但只餵 `/admin/users`。使用者補充：版面有 admin 與一般使用者兩種，需求要同時對這兩種版面成立，不是只補一般使用者頁。
- 議題 2（裝置碼）＝碼其實有預填。`connections.rs:285-296` 開瀏覽器時 URL 已帶 `?user_code=`；`web.rs:44/331` 的 `compute_destination` 在登入後導回 `/activate?user_code=…`；`ActivatePage.tsx:20-21` 讀 URL 預填。痛點在 `ActivatePage` 第一階段呈現為待填輸入欄（`Field id="user-code"`），語氣是「請輸入」而非「請確認這組碼」。
- 議題 2 的第二層＝desktop 全程不呈現 device flow 狀態。`ServersPanel.tsx` 的 `ConnectionPhase` 只有 idle/busy/error/notice/patInput，無任何狀態承載 user_code；等待授權整段只顯示 `t("servers.busy")`，無碼、無驗證網址、無取消。瀏覽器開錯 profile 或想改用另一台裝置授權時完全無路。
- 議題 3（授權後無銜接）＝兩側對稱缺口。desktop 側 `store.ts:1078-1097` 的 `loginConnection` 成功後只做 refreshConnections → recoverRemoteSessions → phase 歸 idle，進工作區仍需手動按 `ServersPanel.tsx:163-173`；web 側 `ActivatePage.tsx:67-76` 核准後只留一行「已核准」，未告知可返回 app。
**Open**: 議題 1 在 admin 與一般使用者兩種版面上分別長什麼樣（同一元件雙進入點，或 admin 沿用既有 registry 頁）？議題 2 要做到「顯示可複製的碼」還是「連驗證網址一起顯示以支援換裝置授權」？議題 3 自動開工作區是否會干擾「一次登入多台 server」的操作流？

### Round 2 — assumptions (2026-07-27)

**Focus**: 第一輪留下的三個問題——版面歸屬、裝置碼要顯示到什麼程度、授權後是否自動開工作區。
**Position**: 三題都有明確答案，且都不需要新的後端能力：
- **議題 1：「我的專案」放 `/account` 的一個區塊，admin 與一般使用者共用同一元件**。`HeaderAccount.tsx:18` 的右上帳號入口對任何登入者都連到 `/account`，admin 同樣到得了，不需要兩套。admin 的 `/admin/registry` 顯示 `data.projects`（全部專案，且可建立）是**治理視角**，與「我隸屬哪些」是不同語意，不該合併——admin 也有 memberships，也需要看到自己的隸屬，而 `role_home`（`web.rs:258`）把 admin 導向 `/admin`、成員導向 `/account`，admin 若不點右上入口就永遠看不到。後端零新增：`admin.rs:698` 已在用的 `list_memberships` 餵進 `/account` 的 `AccountSummary` 即可。
- **議題 2：碼與驗證網址一起顯示，並補取消**。`DeviceAuthorizationResponse`（`protocol/src/device.rs:19-27`）四個欄位 `user_code` / `verification_uri` / `expires_in` / `interval` desktop 全部拿到了，卻一個都沒呈現——`ConnectionPhase` 沒有承載它們的變體。只顯示碼而不顯示網址，換另一台裝置授權時使用者不知道要開哪個 URL；`expires_in` 在手上也可直接做倒數。成本幾乎相同（同一個 phase 變體多帶幾個欄位），故一併做。取消同理：目前等待中只能等到 `expires_in` 逾時，中途無路可退。
- **議題 3：不自動開工作區，改成登入成功後的顯眼行動呼籲；web 側補「可返回 app」**。`loginConnection`（`store.ts:1078-1097`）成功後緊接著跑 `recoverRemoteSessions`，此時強制彈 chooser 會打斷既有 session 的恢復；且「登入」與「開工作區」是兩個意圖，一次登入多台 server 是真實情境。斷點真正該補的是 web 側——`ActivatePage.tsx:67-76` 核准後只留一行「已核准」，未告知可以回到 Speclink app，使用者不知道流程已完成。
**Ruled out**: 為 admin 另做一套「我的專案」面（治理視角與個人視角語意不同，但個人視角本身兩種版面完全一致，重複實作無收益）；desktop 只顯示碼不顯示網址（省不了成本，卻堵死換裝置授權）；授權成功自動彈出工作區選擇器（與多 server 登入流衝突）。
**Open**: 「我的專案」區塊要顯示到什麼程度——僅專案名與角色，或含 repo 清單與直接跳轉？desktop 的碼呈現要不要附複製鈕（LANGUAGE.md 已有 slug 複製的既定語彙可循）？

### Round 3 — assumptions (2026-07-27)

**Focus**: 使用者確認三個立場後，收掉最後兩個細節（區塊顯示深度、複製鈕）。
**Position**: 兩者都取最小可用形：
- 「我的專案」區塊只顯示專案名與角色。repo 清單對一般成員需要新的查詢面（registry API 是 admin-only），而 web 端沒有可跳轉的工作區概念，跳轉無意義——YAGNI。`list_memberships` 回的 projectKey＋role 就是完整資料。
- desktop 的裝置碼附複製鈕，循 LANGUAGE.md 既有的複製語彙（系統匣「複製 slug」先例）；驗證網址同樣可複製。
**Ruled out**: 在 /account 內嵌 repo 清單與跳轉（需 admin-only API 下放＋web 端無對應目的地）。

## Conclusion

**Decision**: 三個斷點各以最小形補齊，全部不需新後端能力：(1) web 的 `/account` 新增「我的專案」區塊，顯示專案名與角色，admin 與一般使用者共用同一元件（經右上帳號入口到達）；server 端把既有 `list_memberships` 餵進 `AccountSummary`。(2) desktop 的連線互動狀態新增「等待授權」變體，承載 `user_code`／`verification_uri`／`expires_in`（倒數）並附複製鈕（循 LANGUAGE.md 複製語彙）與取消；資料已全數在 `DeviceAuthorizationResponse` 手上，僅缺呈現。(3) 授權後銜接：desktop 不自動開工作區（與多 server 登入流及 `recoverRemoteSessions` 衝突），改為登入成功後顯眼的「開啟工作區」行動呼籲；web 的核准完成頁補「可返回 Speclink app」的明確收尾文案。

**Rationale**: 逐檔查證後三個斷點的共同根因是「資料都到位、呈現層沒接」——裝置碼一路預填（URL 帶 user_code、登入後 server 導回 activate）、memberships 後端已在算、授權結果 desktop 已輪詢到。修呈現而非改流程，成本最低且不動既有認證語意。

**Rejected alternatives**: 為 admin 另做一套個人專案面（個人視角在兩種版面完全一致；/admin/registry 是治理視角、語意不同不合併）；desktop 只顯示碼不顯示驗證網址（省不了成本卻堵死換裝置授權）；授權成功自動彈工作區選擇器（打斷 session 恢復、與一次登入多台 server 的真實情境衝突）；/account 內嵌 repo 清單與跳轉（需 admin-only API 下放且 web 無跳轉目的地，YAGNI）。

**Deferred**: 無。

**Capture to**: proposal（新變更；三個斷點同屬「遠端登入鏈的呈現層」，先併一個變更，propose 時視 tasks 規模再決定是否拆分）

**Next**: /speclink-propose --from-discussion remote-login-ux-gaps

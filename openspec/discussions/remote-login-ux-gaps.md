---
topic: 遠端登入與裝置授權的 UX 斷點（專案歸屬、裝置碼、授權後銜接）
slug: remote-login-ux-gaps
status: open
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

## Conclusion

<!-- Written by `speclink discuss conclude`:
**Decision** / **Rationale** / **Rejected alternatives** / **Deferred** / **Capture to** / **Next** -->

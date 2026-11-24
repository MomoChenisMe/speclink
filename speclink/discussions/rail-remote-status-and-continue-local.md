---
topic: 如截圖，目前左側sidebar如果連線不了，必須點下去後才會知道，並不會即時反應，這個可以怎麼修正？然後若選擇繼續以本機，切換左側sidebar tab後再切回來就會變成已經不是speclink專案，為什麼？
slug: rail-remote-status-and-continue-local
status: promoted
created: 2026-11-24
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: continue-local-tab-reactivation, rail-remote-connection-check
---

# Discussion: 如截圖，目前左側sidebar如果連線不了，必須點下去後才會知道，並不會即時反應，這個可以怎麼修正？然後若選擇繼續以本機，切換左側sidebar tab後再切回來就會變成已經不是speclink專案，為什麼？

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者回報 Speclink Desktop 圖示列的兩個問題：(1) 遠端專案的 Server 連不上時，圖示列不會先標出來，要點下去才看到「無法連線到伺服器」；(2) 開啟一個同時有本機規格資料夾與 `.speclink.yaml` remote 區段的資料夾，在三出口對話框選「繼續本機」後，切到別的專案再切回來，方塊轉成錯誤態，提示「已不是 speclink 專案」。兩個目標都可以直接驗證（不點也看得到連不上、切回來不轉錯誤態），不需要先釐清需求，直接列假設。相關規格：desktop-config（圖示列方塊的錯誤態與琥珀點）、remote-resilience（離線狀態機由 Rust 單一判定、存取被拒跟著分頁）、workspace-session（無 session 的 remote 分頁呈 restoring／error）、workspace-chooser（並存衝突三出口）、remote-connection（並存時 CLI 以 remote 為準並警告）、tray-status-menu（系統匣專案列與圖示列共用狀態判斷）、client-protocol（`/api/speclink/v1/whoami`）；目前沒有進行中的變更。
Prior discussions: 專案選擇對齊-spectra

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-11-24)

**Focus**: 兩個問題的原因，與各自的修法、顯示方式和拆刀
**Position**: 使用者確認八條假設全部成立：① 是 bug，切回時把「本機與遠端並存」照樣當本機開；② 由 Rust 對每台有開著分頁的 Server 跑一條輕量的定期檢查；拆成兩個變更，先修 ①。
- ① 原因：`activateTab` 對本機分頁每次都重新探測（`apps/desktop/src/store.ts:2503`），只接受 `project` 一種結果，其他一律標「已不是 speclink 專案」（`store.ts:2514-2517`）；探測遇到 remote 區段一律回 `remoteBinding`，並存只多帶 `hasLocalOpenspec`（`apps/desktop/core/src/project.rs:43-53`）；「繼續本機」第一次開啟繞過探測（`store.ts:1942`），所以要切走再切回才發作；重開 app 時若它是上次作用中的分頁，也走同一段（`store.ts:2744`）
- ① 修法：本機分頁切回時，探測結果是 `remoteBinding` 且 `hasLocalOpenspec` 為 true，照樣以本機開啟；分頁身分是 `local:{root}`，使用者已經選過本機
- ① 不加「終端機的 speclink 走遠端」提示：對話框已寫明 remote marker 保持不動
- ② 原因：啟動時只探測背景的本機分頁，遠端分頁跳過（`store.ts:2763`）；只有作用中 session 訂閱事件、啟動背景輪詢（`session.ts:503`、`App.tsx:511`）；沒有 session 也沒有復原狀態的遠端分頁，圖示列一律畫成正常（`ProjectRail.tsx:113-116`）
- ② 檢查方式：Rust 端對有開著分頁的每台 Server 一條定期檢查，送一個很輕的請求（例如 `/api/speclink/v1/whoami`）；時機為啟動、視窗回到前景、之後每 60 秒；失敗沿用連續 3 次才轉離線的規則，失敗後快速重試；結果走既有的 `remote-connection-state` 事件；前端改為每條連線記一份狀態，沒有 session 的分頁也讀得到。連線健康狀態本來就以連線為單位存在 Rust（`apps/desktop/src-tauri/src/remote.rs:110-128`）
- ② 顯示：琥珀點表示離線或需要重新登入，紅點留給開啟失敗與存取被拒（`ProjectRail.tsx:141-147`）；系統匣共用同一個判斷函式，一起更新
- ② Server 恢復後只拿掉點，不自動連線；只偵測整台 Server 連不上與需要重新登入，單一 repo 的存取被拒仍由該分頁自己的輪詢判定
- 規格對照：① 規格沒寫（workspace-chooser「並存衝突三出口」只寫第一次開啟）；② 規格寫了一部分（remote-resilience 只涵蓋有 session 的分頁，desktop-config 只寫背景本機分頁啟動時探測）；先前討論 `專案選擇對齊-spectra` 排除的是「每個分頁各自監看與快取」，理由是範圍太大，這次的定期檢查不載入資料、每台 Server 一條，不是那個方向
**Ruled out**: 每次切回都重跳三出口對話框 — 每切一次分頁就被強制選一次；啟動時逐個遠端分頁握手 — N 個分頁 N 次完整連線，而且只檢查一次，之後 Server 停掉一樣看不到；沒點過的分頁連不上時用紅點 — 和「這個 repo 本身有問題」同色，分不出來；Server 恢復後自動連線 — 等於替每個背景分頁建立 session 並載入資料；兩件事合成一個變更 — 沒有共用程式碼，小 bug 要等大功能
**Open**: none

## Conclusion

**Decision**: 拆成兩個變更，先修「繼續本機」切回失效的 bug，再讓圖示列不用點也標出連不上的 Server。
- **cut 1 `continue-local-tab-reactivation`**: 本機分頁切回或重開 app 時，資料夾同時有本機規格資料夾與 remote 區段，照樣以本機開啟
  - `activateTab` 對本機分頁的探測結果是 `remoteBinding` 且 `hasLocalOpenspec` 為 true 時，走 `enterProject`，不轉錯誤態
  - 探測結果是 `remoteBinding` 但沒有本機規格資料夾、`uninitialized`、`unadopted` 時，行為不變
  - 不加「終端機的 speclink 走遠端」提示
  - 例：`~/acme` 同時有 `speclink/` 與 `.speclink.yaml` 的 remote 區段，選「繼續本機」，切到別的專案再切回，看板照常顯示本機內容、方塊沒有紅點；關掉 app 再開、上次停在這個分頁時，同樣以本機開啟
  - 規格：workspace-chooser「並存衝突三出口」補切回與重開的情境；desktop-config 圖示列錯誤態的判定一併對齊
- **cut 2 `rail-remote-connection-check`**: Rust 端對有開著分頁的每台 Server 跑一條輕量的定期檢查，圖示列與系統匣不用點就標出離線或需要重新登入
  - 每台 Server（以連線為單位）一條，不是每個分頁一條；只送很輕的請求（例如 `/api/speclink/v1/whoami`），不載入任何專案資料
  - 時機：app 啟動、視窗回到前景、之後每 60 秒
  - 失敗沿用連續 3 次才轉離線的規則，失敗後快速重試，幾秒內就會亮燈；401 走既有的換發與需要重新登入流程
  - 結果走既有的 `remote-connection-state` 事件；前端改為每條連線記一份狀態，沒有 session 的遠端分頁也讀得到
  - 顯示：離線與需要重新登入用琥珀點，紅點只留給開啟失敗與存取被拒；系統匣共用同一個判斷函式，一起更新
  - Server 恢復後只拿掉琥珀點，不自動連線；點下去才建立 session，點下去之後的流程不變
  - 只偵測整台 Server 連不上與需要重新登入；單一 repo 的存取被拒仍由該分頁自己的輪詢判定
  - 例：開著本機專案 A，與同一台 Server 上沒點過的遠端專案 B、C；Server 停著時啟動 app，幾秒內 B、C 的方塊亮琥珀點、提示「離線」，A 不變；Server 恢復後，最遲 60 秒或視窗回到前景時琥珀點消失；這時點 C 才建立連線
  - 規格：desktop-config 圖示列（琥珀點涵蓋沒點過的遠端分頁）、remote-resilience（定期檢查是連線狀態的來源之一，仍由 Rust 單一判定）、tray-status-menu（系統匣專案列跟著顯示）
**Rationale**: 兩個問題的原因都是「沒點過的分頁沒有任何真實狀態」：本機分頁切回時重新探測卻只認一種結果，遠端分頁在點下去之前完全沒有人檢查。連線健康狀態本來就以連線為單位存在 Rust，也已經有廣播事件，所以每台 Server 一條定期檢查只是多一個請求來源，比逐分頁握手便宜，也能在 Server 中途停掉時跟著更新；① 的修法只是把使用者已經做過的選擇延續下去。
**Rejected alternatives**:
- 每次切回都重跳三出口對話框 — 每切一次分頁就被強制選一次
- 啟動時逐個遠端分頁握手 — N 個分頁 N 次完整連線，而且只檢查一次，之後 Server 停掉一樣看不到
- 沒點過的分頁連不上時用紅點 — 和「這個 repo 本身有問題」同色，分不出來
- Server 恢復後自動連線 — 等於替每個背景分頁建立 session 並載入資料
- 兩件事合成一個變更 — 沒有共用程式碼，小 bug 要等大功能
**Deferred**:
- 琥珀點的分頁點下去後，握手失敗會進復原頁並轉紅點，前後顏色不同 — 先沿用現有的復原頁流程，實作後看實際畫面再決定要不要統一
**Capture to**: proposal | design | spec
**Next**: /speclink-propose --from-discussion rail-remote-status-and-continue-local

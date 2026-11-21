---
topic: 內嵌宿主開放 HTTP 入口給 CLI 與 Speclink Desktop 0.14.0 之後的實測結果：F1 專案有多個 repo 時 speclink link 與 auth status 被 409 拒絕、F2 Speclink Desktop 的背景輪詢收到 403 時不顯示訊息，另有五個觀察
slug: embedded-host-needs-5
status: promoted
created: 2026-11-20
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: project-whoami-and-denied-state
---

# Discussion: 內嵌宿主開放 HTTP 入口給 CLI 與 Speclink Desktop 0.14.0 之後的實測結果：F1 專案有多個 repo 時 speclink link 與 auth status 被 409 拒絕、F2 Speclink Desktop 的背景輪詢收到 403 時不顯示訊息，另有五個觀察

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

一個內嵌宿主把 `/api/speclink/v1/` 交給 `host.handleRequest`，並用 `speclink` CLI 0.14.0 與 Speclink Desktop 0.14.0 實測之後，提出兩項不足（F1、F2）與五個觀察（O1–O5）。這份討論接在 embedded-host-needs-4 之後。使用者要在結論之後修正 F1 與 F2，宿主再用新版重新實測。那個宿主是私有系統，記錄只談通用的情境。
需求已經明確（每一項都有現況、實測結果與需求），沒有先做需求釐清；輸入是文件，所以把文件的每個主張對照 `main`（與 v0.14.0 只差兩份 README）逐項分類。
相關規格：host-kit（「範圍與能力由宿主的函式決定」「握手、輪詢與範圍清單由入口回答」「登入方式由宿主宣告」）、remote-connection（「remote 初始化與連接指令」「repo 身分攜帶與歸屬防呆」）、remote-resilience（「離線狀態機單一真相且明確呈現」）、remote-auth（「憑證解析階梯」「憑證失效的處理」）、reference-server（「binding 與認證前置 fail closed」）、client-protocol。目前沒有進行中的變更。
Source doc: ../speclink-embedded-host-needs-5.md
Prior discussions: embedded-host-needs, embedded-host-needs-2, embedded-host-needs-3, embedded-host-needs-4

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-11-20)

**Focus**: 文件的每個主張對照程式碼是否成立，以及 F1 要改用戶端還是改伺服端
**Position**: F1 與 F2 都成立；F1 改伺服端——專案底下的 `/whoami` 不選 repo，CLI 另補一條檢查。
- F1 成立：`connection.rs` 的 `cmd_link` 與 `auth_status` 以 `RemoteClient::new(.., None).whoami()` 呼叫；入口在 `http/mod.rs` 的 `project_request` 先執行 `choose_repo` 才進路由，專案有多個 repo 而沒有指名時回 409。
- 文件漏掉第三處：`speclink init --store remote` 走 `remote_base.rs` 的 `validate_or_defer`，寫法相同，也會被拒絕。
- 文件的「請確認」：Speclink Desktop 不呼叫專案底下的 `/whoami`，「新增專案」的清單來自 API 根的 `/scopes`（`remote.rs` 的 `list_scopes`），多個 repo 的專案不受影響。
- 修法的理由：專案底下的 `/whoami` 回的 `repos` 是整個專案的清單（`routes.rs` 的 `whoami`），它的用途就是讓用戶端知道有哪些 repo；要求先選 repo 才能列 repo 是循環。這個路由只為了算 ETag 才用到 repo 的範圍，而 CLI 不讀這個 ETag。
- 示範用 Server 的專案路由已經全部經過同一個入口（`forward.rs` 的 `through_door`），改入口一處就涵蓋兩者。
- CLI 補一條：專案有多個 repo 而 `link`、`init` 沒帶 `--repo` 時拒絕並列出代號。
- 代價（使用者已接受）：宿主重測時要連 `@speclink/engine` 一起升級，不只是 CLI 與 Speclink Desktop。
- F2 成立：`event_manager.rs` 的 worker 只處理 `sync_state()` 成功；連線狀態只有 online、offline、needs-reauth 三種，403 不屬於任何一種。連不上時連續 3 次後有離線提示列；回 503 時沒有提示（`is_transport_failure` 只認沒有狀態碼的失敗）。
- 觀察：O1 是預期（回應是固定形狀，`protocol/auth.rs` 的測試釘住 `scopes`）；O2 成立（`CredentialSource::describe` 的文字寫死 device login）；O3 成立（`translate_protocol_error` 對 500 以上一律丟掉信封）；O4 讀碼確認兩支 worktree 技能會在 `worktree: true` 之後出現（`http/skills.rs` 與 `worktree_skills_on`）；O5 是預期。
**Ruled out**: CLI 在已知 repo 時帶上 `X-Speclink-Repo` — `--repo` 拼錯時入口回 404 且不列代號，和 remote-connection 的情境「列出可用註冊名」衝突；而且沒有修掉「列 repo 要先選 repo」的循環
**Open**: F2 的狀態跟著分頁還是跟著連線；503 是否算離線；O2、O3 是否併入這次修正；O1 是否補文件；一個變更還是兩個；入口的 409 訊息要 CLI 使用者指定他不能設的標頭，要不要改字

### Round 2 — assumptions (2026-11-20)

**Focus**: F2 的狀態範圍、503 的判定、五個觀察的去向、變更的數量，以及入口 409 訊息的用字
**Position**: 使用者確認六項假設全部成立，全部放進一個變更。
- F2：「存取被拒」的狀態跟著分頁（連線＋專案＋repo），不跟著連線——同一台伺服器上使用者可能被移出專案 A 而仍在專案 B，事件管理器的 worker 本來就以這三者為鍵。只認輪詢（`/sync-state`）的 403；寫入被拒的 403 在操作當下已有訊息。提示列放在離線提示列的位置，顯示宿主的原因，輪詢恢復後自動消失並重新載入。
- 已知邊界：推送串流正常時 Speclink Desktop 不輪詢，被移出的人要等串流斷線才看到提示。
- 503：伺服器有回應但不能服務（500 以上）也算離線。remote-resilience 的字面是「sync-state 亦失敗即 offline」，現在的判定只認連不上。
- O2：`auth status` 的憑證來源文字不再指名登入方式（鑰匙圈不記是哪一種登入）。
- O3：伺服器以信封回 `unavailable` 時，用戶端顯示信封的原因，不再請使用者檢查網址；連不上時維持檢查網址的提示。
- O1：不改程式，在 SDK 文件補一句「回應的 `browser` 一定帶 `scopes`，沒有宣告時是空陣列」。O4、O5 不動。
- 入口的 409 訊息：同時寫出 `X-Speclink-Repo` 標頭與 `.speclink.yaml` 的 `remote.repo`，候選清單照舊。專案變成多個 repo 之後，沒有設定 repo 的資料夾每個動詞都會看到這句，而 CLI 使用者不能設標頭。
**Ruled out**: 「存取被拒」做成連線層級的狀態 — 使用者仍有權限的其他專案分頁會誤報；F1 與 F2 分成兩個變更 — 兩項都小，宿主要一起重測
**Open**: Speclink Desktop 提示列的畫面（使用者要先看示意）

### Round 3 — assumptions (2026-11-20)

**Focus**: 「存取被拒」在 Speclink Desktop 的畫面要怎麼符合既有的設計，以及這個狀態下能做什麼
**Position**: 使用者看過示意後同意三處調整與「期間只能讀」；畫面跟著既有的錯誤呈現，不另立樣式。
- 底色用紅色（錯誤）而不是琥珀色：desktop-app「介面狀態語意色分層」把遠端工作區的 access-denied 列為錯誤，並規定錯誤不得以琥珀呈現。琥珀留給使用者自己能解決的離線與重新登入。
- 圖示用警示三角形：`RemoteWorkspaceRecovery.tsx` 與系統匣面板對 access-denied 都用它；Speclink Desktop 沒有用鎖頭表示權限的先例。
- 提示出現在三個地方：看板上方的提示列（離線提示列的位置，標題沿用既有文案「沒有存取權限」，說明是宿主的原因加上「目前顯示最後成功載入的內容」，保留 stale 標記，沒有按鈕）、專案圖示列的狀態點（紅點，提示文字沿用「存取遭拒」）、系統匣面板（分頁下方的狀態文字與面板上方的提示卡）。remote-resilience 要求主視窗與面板一起標示。
- 期間只能讀：沿用離線時的做法——`session.ts` 的 `maskRemoteWrites` 把寫入能力全部關掉，Rust 端的寫入命令立即拒絕，不排隊、不重放。已載入的內容照常可以看。
- 重開 app 後帳號仍被拒絕時，既有的紅色復原頁已經處理（沒有 session 的那條路）；這次只補「看板開著時被拒絕」。
- 示意畫面：使用者帳號底下的 artifact「看板提示列示意」（四個畫面：存取被拒、連線中斷、需要重新登入、系統匣面板）。
**Ruled out**: 琥珀色提示列加鎖頭圖示（第一版示意）— 和「介面狀態語意色分層」衝突，也和既有的 access-denied 呈現不一致
**Open**: 無

### Round 4 — assumptions (2026-11-20)

**Focus**: 第一次結論延後的項目能不能放進同一個變更
**Position**: 使用者決定把三項納入同一個變更；第四項是宿主環境的實測，不是可以修的缺陷，維持延後。
- 專案底下的 `/whoami` 回應不帶 ETag：不選 repo 就沒有範圍的版本號；CLI 與 Speclink Desktop 都不讀這個 ETag（`speclink-remote` 的 `client.rs` 的 `get` 只取本文）。
- 離線提示列顯示伺服器回的原因：伺服器以信封回 `unavailable` 時，提示列的說明用它的原因；連不上時維持現有的文字。連線狀態的事件本來就帶訊息欄位，提示列已經會顯示它。
- 推送串流正常時的偵測由伺服端負責：示範用 Server 在送出心跳時（預設 15 秒）重新檢查帳號仍可用、仍是專案成員，不成立就關閉串流。Speclink Desktop 重新訂閱時收到 403，接著的輪詢也是 403，就走「存取被拒」的路徑，不需要另外的用戶端邏輯。文件寫明自己提供串流的宿主也要在使用者失去權限時關閉串流。
- 檢查的是帳號與成員資格，不是 token 的有效期限：token 到期由既有的換新流程處理，不該讓串流每次到期都斷線。
- 讀碼確認宿主沒看到的一項：「遷移到 Server」被拒時，`MigrationDialog.tsx` 用紅色訊息列顯示錯誤文字。
**Ruled out**: 串流正常時 Speclink Desktop 也定期輪詢 — 每個開著的分頁每 30 秒多一個請求，推送的好處少一半；用戶端讀串流時會一直等到下一個事件（`events.rs` 的 `EventStream::next`），要插入輪詢得改讀串流的方式
**Open**: 無

## Conclusion

**Decision**: F1 與 F2 都成立，用一個變更 `project-whoami-and-denied-state` 修正，並一併處理 O1、O2、O3 與推送串流的偵測；O4、O5 不動。
- 入口（`speclink-host` 的 HTTP 入口，示範用 Server 的專案路由也經過它）：
  - 專案底下的 `/whoami` 不再選 repo。專案有多個 repo 而請求沒有 `X-Speclink-Repo` 時照樣回 200，`repos` 是整個專案的清單。身分、專案是否在這個人的範圍清單裡、API 版本標頭三項檢查照舊。
  - 這個回應不帶 ETag：不選 repo 就沒有範圍的版本號，CLI 與 Speclink Desktop 都不讀它。
  - 其他專案路由的規則不變：多個 repo 而沒有指名時回 409，不自動選。
  - 409 的訊息同時寫出 `X-Speclink-Repo` 標頭與 `.speclink.yaml` 的 `remote.repo`，候選清單照舊。專案變成多個 repo 之後，沒有設定 repo 的資料夾每個動詞都會看到這句，而 CLI 使用者不能設標頭。
- CLI：
  - `speclink link`、`speclink init --store remote`、`speclink auth status` 三個動詞的呼叫方式不用改（它們問專案底下的 `/whoami` 而不帶 repo），入口修正後就能用。
  - 已有憑證、專案有多個 repo、而 `link` 或 `init --store remote` 沒帶 `--repo` 時：以非 0 結束、訊息請使用者加 `--repo` 並列出代號、不寫入 remote 區段。沒有憑證時照舊直接寫入並提示登入。
  - `--repo` 指定的名稱不在專案裡時，沿用既有訊息 `repo '<名稱>' is not registered in this project (available: …)`。
  - `auth status` 的憑證來源文字不再指名登入方式（鑰匙圈不記是瀏覽器登入還是裝置登入）；`--json` 的 `credentialSource` 值不變。
  - 伺服器以錯誤信封回 `unavailable` 時，用戶端顯示信封的原因，不再請使用者檢查網址；連不上時維持檢查網址的提示。
- Speclink Desktop：
  - 新增「存取被拒」狀態，跟著分頁（連線＋專案＋repo），不跟著連線。只由背景輪詢（`/sync-state`）的 403 觸發；由 Rust 端判定並廣播，前端不自行推斷。
  - 進入狀態後，已載入的內容保留可讀，畫面照常可以瀏覽；寫入能力全部關掉、Rust 端的寫入命令立即拒絕、不排隊也不重放（和離線相同）。下一次輪詢成功時狀態消失並重新載入。
  - 呈現在三個地方，都用紅色（錯誤）與警示三角形圖示：看板上方的提示列（離線提示列的位置；標題沿用既有文案「沒有存取權限」，說明是宿主的原因原樣顯示，再接「目前顯示最後成功載入的內容」；保留 stale 標記；沒有按鈕）、專案圖示列的狀態點（紅點，提示文字沿用「存取遭拒」）、系統匣面板（分頁下方的狀態文字與面板上方的提示卡）。
  - 伺服器有回應但不能服務（500 以上）也計入離線的連續失敗次數，達到既有的門檻（3 次）就顯示既有的離線提示列。
  - 離線提示列的說明：伺服器以信封回 `unavailable` 時用它的原因；連不上時維持現有的文字。
  - 重開 app 後帳號仍被拒絕的情況不改：既有的紅色復原頁已經處理。
- 推送串流：
  - 示範用 Server 在送出心跳時（預設 15 秒）重新檢查帳號仍可用、仍是專案成員；不成立就關閉串流。檢查的是帳號與成員資格，不是 token 的有效期限。
  - Speclink Desktop 不加邏輯：串流關閉後重新訂閱收到 403，接著的輪詢也是 403，就進入「存取被拒」。這條路徑要有測試。
  - 文件寫明：用 `pushTransports` 宣告自己的串流的宿主，要在使用者失去權限時關閉串流，Speclink Desktop 才會顯示提示。
- 文件：SDK 文件補一句「`auth-methods` 回應的 `browser` 一定帶 `scopes`，宿主沒有宣告時是空陣列」；使用者文件與 docs/product-status 照流程設定的規則更新，並寫明宿主要升級 `@speclink/engine`、CLI、Speclink Desktop 三樣才能重測。
- 查證後不用改的項目：Speclink Desktop 的「新增專案」不受多個 repo 影響（清單來自 API 根的 `/scopes`）；O4 讀碼確認流程設定的 `worktree` 為 true 之後 `speclink update` 會帶出兩支 worktree 技能；O5 符合 remote-auth「憑證失效的處理」；「遷移到 Server」被拒時對話框以紅色訊息列顯示錯誤文字。
- 涉及的規格（提案時逐條確認）：host-kit「範圍與能力由宿主的函式決定」「HTTP 入口處理原版用戶端的請求」、reference-server「binding 與認證前置 fail closed」、remote-connection「remote 初始化與連接指令」、remote-resilience「離線狀態機單一真相且明確呈現」「最後 snapshot 唯讀且寫入即拒無佇列」「恢復自動收斂並清除 stale」、server-event-stream、desktop-app「介面狀態語意色分層」、tray-status-menu、client-protocol、user-documentation。
- 範例：專案 `team` 有 `erp`、`crm` 兩個 repo。已登入時 `speclink link <專案網址> --repo crm` 成功並寫入；`--repo nope` 顯示 `repo 'nope' is not registered in this project (available: erp, crm)`；不帶 `--repo` 時以非 0 結束、列出 `erp` 與 `crm`、不寫入。已連好的資料夾執行 `speclink auth status` 顯示身分與憑證來源。看板開著時帳號被停用：最長 30 秒內看板上方出現紅色提示列「沒有存取權限」與宿主的原因，勾選任務等寫入被停用；帳號恢復後下一次輪詢提示列消失、看板重新載入。連到示範用 Server（有推送）的看板開著時使用者被移出專案：下一次心跳時串流關閉，Speclink Desktop 重新訂閱被拒，出現同一個提示列。
**Rationale**: 專案底下的 `/whoami` 的用途就是告訴用戶端專案有哪些 repo，要求先選 repo 才能列 repo 是循環；改入口一處就同時修好三個 CLI 動詞，也保住「repo 拼錯時列出可用代號」的既有行為，代價是宿主要連引擎一起升級。Speclink Desktop 的部分沿用既有的狀態呈現（語意色、既有文案、唯讀的做法），只補上「看板開著時被拒絕」這一段。推送串流的偵測交給伺服端，是因為只有伺服端知道權限何時變了；用戶端因此不用在串流正常時多送請求。
**Rejected alternatives**:
- CLI 在已知 repo 時帶上 `X-Speclink-Repo` — `--repo` 拼錯時入口回 404 且不列代號，和 remote-connection 的情境「列出可用註冊名」衝突；也沒有修掉「列 repo 要先選 repo」的循環
- 「存取被拒」做成連線層級的狀態 — 同一台伺服器上使用者仍有權限的其他專案分頁會誤報
- 琥珀色提示列加鎖頭圖示 — desktop-app「介面狀態語意色分層」把 access-denied 列為錯誤且禁止用琥珀；既有的 access-denied 呈現都用警示三角形
- 串流正常時 Speclink Desktop 也定期輪詢 — 每個開著的分頁每 30 秒多一個請求，推送的好處少一半，還要改用戶端讀串流的方式
- F1 與 F2 分成兩個變更 — 兩項都小，宿主要一起重測
- `auth-methods` 的回應在沒有宣告時省略 `scopes` — 回應是固定形狀，用戶端把缺少與空陣列視為相同，改了沒有好處
**Deferred**:
- 宿主自己列為還沒確認的三個部分（成功換新 access token、掛在路徑前綴底下、內部 CA 簽的憑證）— 是宿主環境的實測，不是已知的缺陷；等宿主部署後實測再提
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion embedded-host-needs-5

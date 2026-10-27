---
topic: CLI 與 desktop 的登入憑證共享與驗證架構
slug: cli-desktop-credential-sharing
status: promoted
promoted_to: cli-desktop-credential-sharing
created: 2026-07-28
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: CLI 與 desktop 的登入憑證共享與驗證架構

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在 remote 模式下發現：desktop 已完成裝置授權登入，CLI 卻報「Not logged in」，要求跑 `speclink auth login`。追問兩點：(1) CLI 為何吃不到 desktop 的登入；(2) desktop 是否內建自己的引擎、是否該改走 CLI、驗證如何共享。

模式：assumptions——掃碼找到大量直接相關原始碼（crates/speclink-remote/src/auth.rs、apps/desktop/src-tauri/src/credentials.rs、connections.rs、remote.rs、crates/speclink-server/src/identity.rs、device.rs）。

碼庫事實：CLI 憑證解析只有 SPECLINK_TOKEN 環境變數 → credentials.yaml 明文檔（auth.rs:89）；desktop 走 OS Keychain 存 refresh／PAT，refresh 換發編排在 app 層（connections.rs:446）。desktop 以 library 直接連結與 CLI 相同的引擎 crates（Cargo.toml:23-28）——一顆引擎、兩個殼，唯獨驗證這塊沒下沉到共用層。server 端 refresh 為 single-use＋reuse 偵測，重用即整族撤銷（identity.rs:495-499）。

相關舊討論：remote-login-ux-gaps（已轉出）——處理裝置授權的 UX 斷點；本討論是其後續，聚焦跨前端憑證共享的架構歸屬。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-28)

**Focus**: 斷點的根因為何——是引擎分裂還是驗證儲存分岔？該修哪一層？
**Position**: 使用者確認走「下沉共用」路線——把 CredentialStore＋refresh 編排下沉到 speclink-remote，新增約束：不得犧牲純 CLI（不裝 desktop）的使用者。
- CLI 讀不到 desktop 登入的根因：兩套獨立憑證儲存（CLI＝credentials.yaml 明文 PAT；desktop＝Keychain refresh/PAT），CLI 解析路徑根本沒有 Keychain（auth.rs:89）
- 引擎未分裂：desktop 以 library 連結與 CLI 相同 crates（Cargo.toml:23-28），驗證是唯一沒下沉的部分——缺口在「驗證編排放錯層」
- 天真共享（CLI 直接讀 Keychain refresh）不可行：server 端 single-use＋reuse 偵測會整族撤銷，兩行程併發換發＝互相登出（identity.rs:495-499）
- 介面深度檢查通過：seam 落在 speclink-remote；adapter 收斂為一個 CredentialStore（desktop 已有 keyring 生產＋in-memory 測試實作，且刻意無 tauri 依賴，下沉障礙低）；背後藏解析順序／rotation 回寫／reuse 保護／跨行程序列化，非 pass-through
**Ruled out**: desktop 改走 CLI subprocess——失去型別安全、解析 stdout、錯誤語意糊化、需打包 CLI 執行檔，全面退步；短期解（desktop 登入後代 CLI 佈建 PAT）——使用者直接選長期解，且該解對純 CLI 使用者無效
**Open**: 共用一個 credential family（真正自動共享，需跨行程序列化）還是每客戶端一族（無競態但 CLI 仍須登入一次）？headless／CI 環境（無 keyring）的 fallback 怎麼收？macOS Keychain ACL 對第二個 binary 的首次存取提示如何處理？

### Round 2 — assumptions (2026-07-28)

**Focus**: 共用族裁定；CLI 登入是否讓使用者自選 PAT／裝置授權？
**Position**: 共用一族＋檔案鎖拍板；CLI `auth login` 走雙軌，使用者可自選。
- 共用一族：desktop 與 CLI 在同一台機器、同一 origin 共用一個 Keychain refresh 條目；換發（讀 refresh → POST /auth/refresh → 回寫）全程持 config dir 檔案鎖序列化，Keychain 單機性質使檔案鎖即足夠
- CLI 登入雙軌：互動 TTY 預設走裝置授權（開瀏覽器／印 URL＋code）；`--pat` 互動貼 PAT；`--token-stdin` 原樣保留（CI）
- PAT 路徑完全不動：儲存仍在 credentials.yaml，行為、旗標、headless 體驗零變化——向下相容的底線
- 解析階梯：SPECLINK_TOKEN → Keychain refresh 換發 → Keychain PAT → credentials.yaml PAT；任一層不可用即靜默下探
- headless（無 keyring）：裝置授權的 refresh 不落明文檔，引導使用 PAT／環境變數（現狀）——refresh 需 rotation 回寫且 reuse 偵測會整族撤銷，落檔後被備份／複製即觸發 teardown，PAT 無此狀態性
**Ruled out**: 每客戶端一族——沒解掉「desktop 登入過 CLI 還要再登入」的原始抱怨，只是把第二次登入變快；換發不序列化——兩行程併發 refresh 觸發 reuse teardown 互相登出
**Open**: 無——進入結論

## Conclusion

**Decision**: 把憑證儲存（CredentialStore）與 refresh 換發編排從 desktop app 層下沉到 speclink-remote，CLI 與 desktop 共用。同機同 origin 共用一個 credential family，換發全程以 config dir 檔案鎖序列化。CLI `auth login` 雙軌：互動 TTY 預設裝置授權（開瀏覽器或印 URL＋裝置碼），`--pat` 互動貼 PAT、`--token-stdin` 原樣保留（儲存仍為 credentials.yaml，行為零變化）。CLI 憑證解析階梯：SPECLINK_TOKEN → Keychain refresh 換發 → Keychain PAT → credentials.yaml PAT，任一層不可用即靜默下探。headless（無 keyring）維持 PAT／環境變數現狀，裝置授權 refresh 不落明文檔。
**Rationale**: 「一顆引擎、兩個殼」架構已成立（desktop 以 library 連結相同 crates），驗證是唯一未下沉的層——修架構歸屬而非貼 UX 補丁。共用一族才真正消除「desktop 登入過、CLI 還要再登入」；Keychain 單機性質使檔案鎖足以序列化換發。階梯向下相容保證純 CLI／headless 使用者零退化，裝置授權下沉後純 CLI 使用者反而獲得免手建 PAT 的登入。
**Rejected alternatives**: desktop 改走 CLI subprocess（失去型別安全、解析 stdout、錯誤語意糊化、需打包 CLI）；desktop 代 CLI 佈建 PAT（對純 CLI 使用者無效）；每客戶端一族（無競態但沒解掉重複登入的原始抱怨）；共享 refresh 不序列化（server 端 single-use＋reuse 偵測整族撤銷，兩行程併發換發互相登出）。
**Deferred**: macOS Keychain 首次存取提示（非建立者 binary）的錯誤訊息文案；PAT 是否也寫入 keyring（維持 credentials.yaml，未來再議）。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion cli-desktop-credential-sharing

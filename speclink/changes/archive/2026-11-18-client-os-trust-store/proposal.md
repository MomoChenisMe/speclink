## Why

組織把宿主（內嵌了引擎的系統）或示範用 server 架在內部網路時，憑證通常由組織自己的 CA（簽發憑證的單位）簽發。今天 CLI 與 Speclink Desktop 的 HTTP 用戶端只信任編進程式裡的公開 CA 清單，不讀作業系統的信任庫（系統記錄「信任哪些 CA」的清單）。結果是這類 Server 完全連不上：登入方式的查詢、瀏覽器登入換 token、一般 API、事件串流全部失敗，而且錯誤訊息只說「server unreachable，請檢查網址」，使用者看不出問題在憑證。

受影響的人是連到遠端專案的工程師：他們在自己的 checkout 用 CLI 跑動詞與技能，或用 Speclink Desktop 看板。瀏覽器沒有這個問題，因為瀏覽器看的是系統的信任庫，而 IT 會把組織的 CA 裝進那裡。來源討論 embedded-host-needs-2 把這一項列為「不解決就無法導入」，並裁定只做「信任作業系統的信任庫」。

正式規格目前沒有任何一條談用戶端的 TLS 信任。掃過的相關規格：remote-connection（連接語意，最接近）、remote-auth（憑證的存放與解析，談的是登入憑證不是 TLS）、remote-resilience（失聯與恢復）、client-protocol（線上格式）。

## What Changes

- `speclink-remote` 的 HTTPS 連線改由作業系統的憑證檢查決定是否信任 Server：macOS 用系統的憑證服務、Windows 用系統的憑證存放區、Linux 用系統的 CA 憑證檔。IT 裝進系統的 CA 自動生效，使用者不用設定。
- 作業系統完全沒有提供任何 CA 時（例如沒有安裝 CA 憑證套件的精簡 Linux 容器），退回今天內建的公開 CA 清單，所以這類環境連公開 Server 的行為不變。
- 建立 HTTP 用戶端的六處收成一個共用函式：一般 API、事件串流、狀態查詢（sync-state）、裝置登入的請求、登入方式與身分的查詢、瀏覽器登入的說明文件查詢與換 token。CLI 與 Speclink Desktop 都經過 `speclink-remote`，所以行為相同。
- 憑證不被信任時，錯誤訊息改成指出原因與處理方法。訊息仍以 `server unreachable` 開頭，接著說明「這台電腦不信任 Server 的憑證」，並請使用者把簽發憑證的 CA 裝進作業系統的信任庫。
- 使用者文件補上這條規則：遠端專案的設定說明、遠端專案的上手指南（連線失敗的排除）、Node SDK 文件裡給宿主的 HTTP 入口說明，中英兩版同步。

相容性影響：

- 沒有新增或變更任何 CLI 子指令、旗標、stdin 或 exit code。
- 人眼輸出只有一處改變：憑證不被信任時 stderr 的訊息變長、變具體，exit code 不變（仍是非零）。其他連線失敗（拒絕連線、逾時、DNS）的訊息逐字不變。`--json` 的輸出不受影響。
- Speclink Desktop 用「訊息以 `server unreachable` 開頭」判斷連線中斷。新訊息保留這個開頭，所以 Desktop 的離線判斷與畫面不變。
- 原本就連得上的 Server（憑證由公開 CA 簽發）照常連得上：公開 CA 也在作業系統的信任庫裡。
- 一種情況會從「連得上」變成「連不上」：組織用系統設定明確移除或不信任某個公開 CA。這是組織的政策，用戶端跟著系統走是預期的結果。

三種形態：

- 本機 repo：不適用。本機專案不建立任何連線。
- 連到遠端的 checkout：可用，這是本變更服務的對象。CLI 與 Speclink Desktop 連到宿主或示範用 server 時都生效。
- 沒有本機規格檔的內嵌宿主：不適用。宿主在自己的程序裡呼叫引擎，不經過這個 HTTP 用戶端；它的憑證由連進來的用戶端檢查，也就是上一項。

不涉及設定欄位（`speclink/config.yaml`、`.speclink.yaml`、環境變數都沒有新增），不涉及生成的技能與 Agent 指令。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `remote-connection`: 新增一條需求，規定用戶端以作業系統的信任庫判斷 Server 的憑證、系統沒有任何 CA 時退回內建清單，以及憑證不被信任時的錯誤訊息。

## Impact

- Affected specs: remote-connection
- Affected crates: speclink-remote（實作）；speclink-cli 與 apps/desktop 經由它取得新行為，自己的程式碼不改
- Affected code:
  - New:
    - crates/protocol/speclink-remote/src/http.rs（共用的建立函式、連線層錯誤的翻譯，以及它們的單元測試）
  - Modified:
    - crates/protocol/speclink-remote/Cargo.toml
    - Cargo.lock
    - crates/protocol/speclink-remote/src/lib.rs
    - crates/protocol/speclink-remote/src/client.rs
    - crates/protocol/speclink-remote/src/events.rs
    - crates/protocol/speclink-remote/src/device.rs
    - crates/protocol/speclink-remote/src/oauth.rs
    - docs/configuration.md
    - docs/configuration.zh-TW.md
    - docs/remote-getting-started.md
    - docs/remote-getting-started.zh-TW.md
    - docs/sdk-node.md
    - docs/sdk-node.zh-TW.md
  - Removed: (none)
- Dependencies: speclink-remote 直接依賴 rustls 與 rustls-platform-verifier（兩者都已經在 Cargo.lock 裡，由其他 crate 帶進來）；測試用的依賴新增 rcgen（產生測試用的 CA 與憑證）

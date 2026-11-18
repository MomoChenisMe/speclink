## 1. 共用的用戶端建立函式（D1）

- [x] 1.1 先寫失敗的守門測試：在 crates/protocol/speclink-remote/src/http.rs 的測試區塊掃描這個 crate 的 src 目錄，斷言 `AgentBuilder::new` 字樣只出現在 http 模組。同時用全 workspace 搜尋（crates 與 apps/desktop 兩處的 Rust 原始碼與 Cargo.toml，搜尋 ureq 與 reqwest）確認 speclink-remote 以外沒有自己建立 HTTP 用戶端的地方，並把搜尋的寫法與結果記在測試的註解。驗證：`cargo test -p speclink-remote http::` 這時失敗，失敗訊息列出 client.rs、events.rs、device.rs、oauth.rs。 <!-- speclink-task:tsk_01M7PMT62NK5KKJ4EA1YGNY722 -->
- [x] 1.2 依 D1 共用的用戶端建立函式放在 speclink-remote 的 http 模組：新增回傳 `ureq::AgentBuilder` 的函式（這一步先回傳 ureq 預設的建立器），並把六處建立用戶端的地方（crates/protocol/speclink-remote/src/client.rs、events.rs 的兩處、device.rs 的兩處、oauth.rs）改成呼叫它；各處原有的逾時數值與 oauth.rs 的不跟隨轉址維持不變。驗證：1.1 的守門測試通過，`cargo test -p speclink-remote` 全部通過。 <!-- speclink-task:tsk_01M7PMT62N4JSXXSS8MBN05BNQ -->

## 2. 信任來源（D2、D3、D5）

- [x] 2.1 先寫失敗的測試（D5 測試用函式層注入的額外信任根）：在 crates/protocol/speclink-remote/Cargo.toml 的 dev-dependencies 加 rcgen，在 http 模組的測試區塊用 rcgen 產生測試 CA 與一張對象是 127.0.0.1 的憑證（含 serverAuth 延伸金鑰用途、IP SAN、有效期不超過 825 天），用 rustls 的伺服器端在本機隨機埠起 HTTPS 服務。兩個測試：信任根含測試 CA 時請求成功並讀到回應內容；信任根不含測試 CA 時請求失敗。驗證：`cargo test -p speclink-remote http::` 這時因為接受額外信任根的函式還不存在而編譯失敗或測試失敗。 <!-- speclink-task:tsk_01M7PMT62NRSFYY1NB04PSJW52 -->
- [x] 2.2 實作需求「用戶端以作業系統的信任庫驗證 Server 的憑證」的信任來源（D2 信任來源用 rustls-platform-verifier）：crates/protocol/speclink-remote/Cargo.toml 直接宣告 rustls（關閉預設功能，只開 ring、std、tls12）與 rustls-platform-verifier；http 模組用明確指定的 ring 加密提供者建立 `rustls::ClientConfig`，以接受額外信任根的私有函式實作，正式路徑傳空清單，結果用 `OnceLock` 在程序內只建立一次並套進 1.2 的建立函式。驗證：2.1 的兩個測試通過；`cargo tree -p speclink-remote -i aws-lc-rs` 回報這個 crate 沒有依賴 aws-lc-rs；`cargo test -p speclink-remote` 全部通過（既有的 http 測試伺服器照常可連，證明 http 連線不受影響）。 <!-- speclink-task:tsk_01M7PMT62NNV3QHGRG0GADVP2W -->
- [x] 2.3 先寫測試再實作 D3 作業系統沒有任何 CA 時退回內建的公開 CA 清單：把「建立信任設定的結果 → 用自訂設定還是 ureq 預設」寫成一個私有函式；測試把一個失敗的結果交給它，斷言回傳的建立器沒有套用自訂設定，而且用它對本機的 http 測試伺服器發請求會成功。驗證：`cargo test -p speclink-remote http::` 通過。 <!-- speclink-task:tsk_01M7PMT62PSCZ0VRY1MHSS00T2 -->

## 3. 憑證被拒絕的訊息（D4）

- [x] 3.1 先寫失敗的測試（D4 憑證被拒絕時的訊息與其他連線失敗分開）：在 http 模組的測試區塊斷言 (a) 信任根不含測試 CA 時，翻譯出的 `RemoteError` 訊息以 `server unreachable — the server's TLS certificate was rejected (` 開頭、含 `UnknownIssuer` 與 `trust store`，`status`、`reason`、`evidence` 都是 `None`；(b) 連到沒有程序在聽的埠時，翻譯出的訊息與 `translate_transport()` 的訊息逐字相同、不含 `certificate`。驗證：`cargo test -p speclink-remote http::` 這時因翻譯函式不存在而失敗。 <!-- speclink-task:tsk_01M7PMT62PNJ090BY4PXE1H41R -->
- [x] 3.2 實作 D4：在 http 模組新增接受 ureq 連線層錯誤的翻譯函式，沿錯誤的來源鏈找 `rustls::Error::InvalidCertificate`；crates/protocol/speclink-remote/src/client.rs、events.rs、device.rs 裡每一處把連線層錯誤翻成 `translate_transport()` 的地方改成呼叫它；crates/protocol/speclink-remote/src/lib.rs 的 `translate_transport` 保留且文字不變。驗證：3.1 的測試通過；`cargo test -p speclink-remote` 全部通過；`cargo test -p speclink-cli --test it remote_read_path` 通過（既有的 `server unreachable` 斷言不變）。 <!-- speclink-task:tsk_01M7PMT62PED3JZYW9QN8WBTP9 -->
- [x] 3.3 先寫測試再實作登入服務的訊息：crates/protocol/speclink-remote/src/oauth.rs 的三處連線層錯誤，在憑證被拒絕時回傳 `the login service's TLS certificate was rejected (<原因>); if your organisation's own CA issued it, add that CA to the operating system's trust store`，其他連線失敗維持各自既有的句子；訊息的後半段與 3.2 共用同一個函式。測試放在 oauth 模組的測試區塊或 http 模組的測試區塊，斷言憑證被拒絕時的句子含 `certificate`、原因與 `trust store`，以及拒絕連線時三處的句子逐字不變。驗證：`cargo test -p speclink-remote` 全部通過，其中 crates/protocol/speclink-remote/tests/it/browser_login.rs 的既有斷言不變。 <!-- speclink-task:tsk_01M7PMT62PBAK4JH9CNHS6AAJ7 -->

## 4. 稽核

- [x] 4.1 對這次新增的函式套用 sharp-edges 稽核清單（用 `speclink instructions --skill audit` 取得），逐項確認並把結果寫在 http 模組的模組註解：沒有任何路徑會關閉憑證檢查（rustls 的 `dangerous()` 在這個 crate 的原始碼只出現一次，只用來裝上作業系統的驗證器；這個 crate 沒有自己實作的憑證驗證器）；D3 的退回只在建立信任設定失敗時發生，不會因為單次請求的憑證被拒絕而退回；翻譯出的訊息只含 rustls 的原因名稱，不含 token、網址的查詢字串或憑證內容；oauth.rs 的不跟隨轉址仍然生效。驗證：`cargo test -p speclink-remote` 全部通過，稽核結果在模組註解中逐項可讀。 <!-- speclink-task:tsk_01M7PMT62P5AQBQ1SKPKX3VETM -->

## 5. 手動驗證

- [x] [M] 5.1 在一台電腦上用本機 CA 工具（例如 mkcert）產生 CA 並裝進作業系統的信任庫，用它簽發的憑證讓一台示範用 server 或宿主以 https 服務。確認：CLI 的 `speclink auth login` 與 `speclink list` 成功；Speclink Desktop 開啟同一個遠端專案後看板載入、事件串流連上（改一個任務的勾選，畫面不用手動重新整理就更新）。記下 `speclink list` 的耗時，與變更前的版本連同一台 server 的耗時比較。 <!-- speclink-task:tsk_01M7PMT62PG7ZAW1VZF7Z0AK87 -->
- [x] [M] 5.2 把 5.1 的 CA 從作業系統的信任庫移除後重試。確認：`speclink list` 以非零 exit code 結束，stderr 的訊息以 `server unreachable` 開頭並指出憑證被拒絕與處理方法；Speclink Desktop 顯示連線中斷、寫入動作停用。 <!-- speclink-task:tsk_01M7PMT62PBY8191MSWBSGDXKE -->

## 6. 使用者文件與收尾（D6）

- [x] 6.1 依 D6 使用者文件寫明信任規則，中英兩版同步：docs/configuration.md 與 docs/configuration.zh-TW.md 的「遠端專案」一節寫明用戶端信任作業系統的信任庫、組織的 CA 要裝進系統、沒有另外的 CA 設定；docs/remote-getting-started.md 與 docs/remote-getting-started.zh-TW.md 的連線失敗說明加上憑證被拒絕時的訊息與處理方法；docs/sdk-node.md 與 docs/sdk-node.zh-TW.md 的 HTTP 入口一節告訴整合者用戶端如何檢查宿主的憑證。驗證：逐一對照規格 delta 的訊息文字與文件引用的訊息一致；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全部通過。 <!-- speclink-task:tsk_01M7PMT62PZX1KBMEYWS1Z7JDM -->

## Context

`speclink-remote` 是 CLI 與 Speclink Desktop 連到 Server 的唯一 HTTP 用戶端。它用 ureq 2（Cargo.lock 是 2.12.1）搭配 rustls。ureq 預設的信任來源是 webpki-roots：一份編進程式裡的公開 CA 清單，不讀作業系統的信任庫。

建立 HTTP 用戶端的地方有六處，全部只設逾時：

- 一般 API：`client.rs` 的 `Client::new`（逾時 30 秒，用戶端物件留著重用）
- 事件串流：`events.rs` 的 `subscribe`（連線逾時 10 秒，讀取逾時是中止輪詢的間隔）
- 狀態查詢：`events.rs` 的 `sync_state`（逾時 10 秒）
- 裝置登入的 POST：`device.rs` 的 JSON POST 函式
- 登入方式與身分的 GET：`device.rs` 的 `get_json`
- 瀏覽器登入：`oauth.rs` 的 `agent`（另外設定不跟隨轉址）

連線層的失敗（拒絕連線、逾時、DNS、TLS 握手失敗）今天全部經 `lib.rs` 的 `translate_transport` 翻成同一句 `server unreachable — check the connection url …`。瀏覽器登入的三處另有自己的句子（`could not reach the login service …`）。Speclink Desktop 用「訊息以 `server unreachable` 開頭、沒有 status、沒有 reason」判斷連線中斷（`apps/desktop/src-tauri/src/remote.rs` 的 `is_transport_failure`）。

Cargo.lock 已經有 rustls 0.23.41 與 rustls-platform-verifier 0.7.0（後者由桌面更新外掛帶進來的 reqwest 使用）。ureq 2.12.1 的 `AgentBuilder::tls_config` 接受 `Arc<rustls::ClientConfig>`，rustls 版本相同，所以不必升級 ureq。

## Goals / Non-Goals

**Goals:**

- 憑證由組織自己的 CA 簽發的 Server，在 CA 已裝進作業系統信任庫的電腦上可以連線，使用者不用設定。
- 六處建立用戶端的地方共用同一份信任設定，之後新增的連線不會漏掉。
- 憑證被拒絕時，訊息指出是憑證的問題與處理方法。
- 原本連得上的環境（含沒有系統 CA 的精簡容器、http 連線）行為不變。

**Non-Goals:**

- 不加指定額外 CA 檔的設定（旗標、設定檔欄位、環境變數）。來源討論已否決：目前沒有人遇到不能改系統信任庫的機器。
- 不提供略過憑證檢查的開關。
- 不升級 ureq 到 3.x，不改用 native-tls。
- 不改 Speclink Desktop 的離線判斷與畫面，不改示範用 server。
- 不承諾 `SSL_CERT_FILE` 與 `SSL_CERT_DIR`。Linux 上底層函式庫會讀這兩個環境變數，那是函式庫的行為；本變更不把它寫進規格與使用者文件，也不為它寫測試。
- 內嵌宿主的 Node SDK 不建立對外的 HTTP 連線，不在範圍內。

## Decisions

### D1 共用的用戶端建立函式放在 speclink-remote 的 http 模組

`speclink-remote` 新增 `http` 模組，提供一個回傳 `ureq::AgentBuilder` 的函式（暫名 `agent_builder`）。它已經套好信任設定，呼叫端只接著設定各自的逾時與轉址規則。六處建立用戶端的地方都改用它，各自原有的逾時數值與 `oauth.rs` 的「不跟隨轉址」一律不變。

唯一落點是 `speclink-remote`：CLI、Speclink Desktop 與桌面的事件串流都經過這個 crate，所以不需要在其他 crate 寫任何程式。

用一個守門測試固定這件事：掃描 `speclink-remote` 的 `src` 目錄，`AgentBuilder::new` 這個字樣只允許出現在 `http` 模組。之後有人新增連線而沒有用共用函式，測試會失敗。

替代方案：在六處各自呼叫 `tls_config`。否決，因為第七處很容易漏掉，而且漏掉時沒有任何測試會發現。

### D2 信任來源用 rustls-platform-verifier

信任設定用 rustls-platform-verifier 0.7 建立：macOS 交給系統的憑證服務，Windows 交給系統的憑證存放區，Linux 讀系統的 CA 憑證檔。建立 `rustls::ClientConfig` 時明確指定 ring 這個加密提供者（和 ureq 目前用的相同），不依賴程序層級的預設提供者，避免和同一個程序裡其他用 rustls 的套件互相影響。

設定在程序內只建立一次（`OnceLock`），之後每個用戶端共用同一個 `Arc`。Linux 上建立時要讀系統的 CA 憑證檔，重複建立會讓每個 CLI 指令多做幾次檔案讀取。

`speclink-remote` 的 Cargo.toml 直接宣告 rustls（關閉預設功能，只開 `ring`、`std`、`tls12`，避免帶進另一個加密提供者）與 rustls-platform-verifier。

替代方案：

- ureq 的 `native-certs` 功能。它在啟動時把系統的 CA 讀成一份清單再自己驗證。否決：macOS 與 Windows 上，管理者用系統設定加上的信任與不信任規則不會生效，結果和瀏覽器不一致。
- 升級到 ureq 3（內建平台驗證的功能）。否決：ureq 3 的 API 與 2 不相容，六處呼叫與全部錯誤處理都要改寫，超出這個變更的範圍。
- 改用 native-tls。否決：Linux 上要連結 OpenSSL，四個發行通路的建置都會變複雜。

### D3 作業系統沒有任何 CA 時退回內建的公開 CA 清單

rustls-platform-verifier 在 Linux 上讀不到任何系統 CA 時，建立驗證器會回傳錯誤（`No CA certificates were loaded from the system`）。這時共用函式不套用自訂的信任設定，直接回傳 ureq 預設的建立器，也就是今天的 webpki-roots。

這樣做的理由：沒有安裝 CA 憑證套件的精簡容器今天連得上公開的 Server，不能因為這個變更變成全部連不上。另外，ureq 的建立器不分 http 與 https 都會帶著信任設定，所以建立設定失敗絕不能讓 http 連線跟著失敗。

退回是無聲的：這個 crate 沒有記錄檔機制，CLI 的 stderr 也不該多出一行。macOS 與 Windows 的驗證器不會因為「沒有 CA」而建立失敗，所以這條退回路徑實際上只在 Linux 出現。

替代方案：建立失敗就讓每個請求報錯。否決：會把今天能用的環境弄壞。

### D4 憑證被拒絕時的訊息與其他連線失敗分開

新增一個翻譯函式，接受 ureq 的連線層錯誤，沿著錯誤的來源鏈找 rustls 的「憑證無效」錯誤（`rustls::Error::InvalidCertificate`）：

- 找到：回傳的 `RemoteError` 訊息是 `server unreachable — the server's TLS certificate was rejected (<原因>); if your organisation's own CA issued it, add that CA to the operating system's trust store`。`<原因>` 是 rustls 給的原因名稱，例如 `UnknownIssuer`、`Expired`、`NotValidForName`。`status`、`reason`、`evidence` 都是空的，和其他連線失敗相同。
- 沒找到：回傳既有的 `translate_transport` 結果，文字逐字不變。

原因名稱由各平台的驗證器決定，同一種情況在不同平台、或 Server 送出的憑證鏈不同時，名稱可能不同。實作時在 macOS 實測：簽發的 CA 不在信任庫時，Server 只送出自己的憑證是 `UnknownIssuer`；連根憑證一起送出時，系統回報的是「不被信任」，rustls 沒有對應的名稱，括號內是 `Other`。驗證器沒有給出具體名稱的其他失敗也是 `Other`。處理方法那半句不受影響，每一種情況都會出現。

訊息只放名稱本身，不放名稱附帶的資料（主機名稱、時間、作業系統給的說明文字）：那些文字有一部分來自對方的憑證。rustls 把帶資料的版本叫 `ExpiredContext`、`NotValidForNameContext`，訊息去掉結尾的 `Context`，所以讀起來仍是 `Expired`、`NotValidForName`。

`client.rs`、`events.rs`、`device.rs` 裡把連線層錯誤翻成 `translate_transport` 的每一處，都改成把錯誤物件交給新函式。

訊息保留 `server unreachable` 開頭有兩個理由：Speclink Desktop 的離線判斷靠這個開頭，保留它就不必改桌面的程式；CLI 的既有測試與使用者文件也用這個開頭辨識連線失敗。

`oauth.rs` 的三處回傳的是字串不是 `RemoteError`，而且對象是登入服務。它們在憑證被拒絕時改成：`the login service's TLS certificate was rejected (<原因>); if your organisation's own CA issued it, add that CA to the operating system's trust store`。其他連線失敗維持各自既有的句子。兩種訊息的後半段由同一個函式產生，不各寫一份。

替代方案：全部連線失敗都附上底層的錯誤文字。否決：會改掉既有訊息的位元級輸出，而且拒絕連線、逾時這些情況的底層文字對使用者沒有幫助。

### D5 測試用函式層注入的額外信任根，不加對外的接縫

真實的作業系統信任庫不能在測試裡修改，所以測試不經過它。`http` 模組內部把「建立信任設定」寫成一個接受額外信任根的私有函式；正式路徑傳空的清單，單元測試傳測試用的 CA。rustls-platform-verifier 在三個平台都提供帶額外信任根的建構函式（`Verifier::new_with_extra_roots`），所以這條測試路徑在 macOS、Windows、Linux 的 CI 都會跑到各平台真正的驗證器。

測試寫在 `http` 模組自己的 `#[cfg(test)]` 區塊，因此私有函式不必變成公開 API，也不新增環境變數或設定。測試內容：

- 用 rcgen（新增的測試用依賴）產生一個 CA 與一張由它簽發、對象是 `127.0.0.1` 的憑證；用 rustls 的伺服器端在本機的隨機埠起一個只回一行 JSON 的 HTTPS 服務。
- 信任根含測試 CA：請求成功。
- 信任根不含測試 CA：請求失敗，翻譯後的訊息符合 D4。測試伺服器送出的憑證鏈依平台而不同，目的是讓系統回報 `UnknownIssuer`：Windows 連根憑證一起送出（它要能把鏈建到根，才回報根不被信任），其他平台只送伺服器憑證（macOS 在根憑證一起送出時回報的是沒有對應名稱的「不被信任」）。
- 沒有程序在聽的埠：翻譯後的訊息和既有的 `translate_transport` 逐字相同。
- 建立信任設定失敗時（把失敗的結果直接交給決定「用自訂設定還是 ureq 預設」的那個函式），回傳的是 ureq 預設的建立器。

測試憑證的注意事項：macOS 的憑證服務會拒絕不合規範的伺服器憑證。測試用的憑證要有 serverAuth 的延伸金鑰用途、有 SAN（IP `127.0.0.1`），而且有效期不超過 825 天；rcgen 的預設有效期遠超過這個上限，要明確設定起訖時間。

### D6 使用者文件寫明信任規則

三份文件各補一段，中英兩版同步：

- 設定文件的「遠端專案」一節：用戶端信任作業系統的信任庫；組織自己的 CA 要裝進系統；沒有另外的 CA 設定。
- 遠端專案上手指南的連線失敗說明：新增憑證被拒絕時的訊息長相與處理方法。
- Node SDK 文件的 HTTP 入口一節：告訴宿主的整合者，用戶端以作業系統的信任庫檢查宿主的憑證，用組織的 CA 時要請使用者（或 IT）把 CA 裝進系統。

docs/product-status 的已知缺口沒有這一項，所以不需要移走任何一列。

## Implementation Contract

**行為**

- CLI 與 Speclink Desktop 連 https 的 Server 或登入服務時，憑證是否被信任由作業系統決定。CA 在系統信任庫裡就連得上，使用者不用設定。
- 作業系統沒有任何 CA 時，信任來源是內建的公開 CA 清單，和今天相同。
- http 連線的行為不變。六處用戶端原有的逾時與轉址規則不變。

**介面與資料形狀**

- `speclink-remote` 內部新增 `http` 模組：一個回傳已套好信任設定的 `ureq::AgentBuilder` 的函式，以及一個把 ureq 連線層錯誤翻成 `RemoteError` 的函式。兩者只在 crate 內使用，不是對外 API。
- `RemoteError` 的欄位不變。憑證被拒絕時：`message` 以 `server unreachable — the server's TLS certificate was rejected (` 開頭，括號內是 rustls 的原因名稱，之後是把 CA 加進作業系統信任庫的說明；`status`、`reason`、`evidence` 都是 `None`。
- 既有的 `translate_transport` 保留，文字不變。
- 沒有新增 CLI 旗標、設定欄位、環境變數、線上格式欄位。

**失敗模式**

- 憑證被拒絕：CLI 非零結束，stderr 是上面的訊息；Speclink Desktop 顯示連線中斷。不讀快取、不重試。
- 登入服務的憑證被拒絕：登入指令非零結束，訊息是 D4 的登入服務版本。
- 其他連線失敗：訊息逐字維持既有輸出。
- 建立信任設定失敗：無聲退回 ureq 預設，不輸出任何訊息。

**驗收**

- `cargo test -p speclink-remote` 全部通過，包含 D5 列的測試與 D1 的守門測試。
- `cargo test -p speclink-cli --test it remote_read_path` 通過：既有的 `server unreachable` 斷言不變。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過：文件的中英對照、連結與詞彙守門。
- 手動：在一台把測試 CA 裝進系統信任庫的電腦上，CLI 與 Speclink Desktop 連得上用該 CA 簽發憑證的 Server；移除 CA 後，CLI 印出憑證被拒絕的訊息，Speclink Desktop 顯示連線中斷。

**範圍**

- 範圍內：`speclink-remote` 的六處用戶端建立、連線層錯誤的翻譯、該 crate 的 Cargo.toml 與 Cargo.lock、三份使用者文件的中英兩版。
- 範圍外：`speclink-cli`、`apps/desktop`、`speclink-host`、`speclink-server`、`speclink-node` 的程式碼；額外 CA 的設定；略過憑證檢查的開關；ureq 升級；用戶端憑證（mTLS）。

## Risks / Trade-offs

- [macOS 與 Windows 的系統驗證可能為了查撤銷狀態而連網，讓第一次握手變慢] → `Client` 物件在一次指令或一個桌面連線期間重用同一個用戶端，連線會被重用；手動驗收時記下 `speclink list` 在變更前後的耗時，差距明顯時在這個變更裡回報並決定是否處理。
- [三個平台各走不同的驗證器，行為可能不一致] → D5 的測試在 CI 的 macOS、Windows、Linux 都跑真正的平台驗證器；測試憑證照 D5 的注意事項產生，避免只在 macOS 失敗。
- [原因名稱在某些平台與憑證鏈下是 `Other`，使用者看不出細節] → 使用者決定維持只印名稱（不印作業系統的說明文字，避免帶出對方憑證上的文字）；處理方法那半句照常出現。D5 測試裡 Windows 那一半的憑證鏈是依 rustls-platform-verifier 的原始碼判讀的，本機無法執行，要由 CI 的 Windows 工作確認。
- [既有輸出的回歸] → 憑證以外的連線失敗訊息逐字不變，由 D5 的「沒有程序在聽」測試與 `speclink-cli` 既有的 `remote_read_path` 測試固定；引擎的 golden 不涉及連線訊息，不需要更新。
- [組織用系統設定移除某個公開 CA 時，原本連得上的 Server 會連不上] → 這是跟隨系統政策的預期結果，寫進提案的相容性影響；訊息會指出是憑證被拒絕。
- [新增 rcgen 這個測試用依賴，Cargo.lock 會多幾個套件] → 只在 `dev-dependencies`，不進發行的 binary。
- [`speclink-remote` 直接宣告 rustls 時若開了預設功能，會帶進第二個加密提供者並拉長建置時間] → 依 D2 關閉預設功能，實作後用 `cargo tree -p speclink-remote -i aws-lc-rs` 確認這個 crate 沒有因此帶進它。
- [Linux 上 `SSL_CERT_FILE` 會改變信任來源，這不是本變更承諾的行為] → 不寫進規格與文件；之後若要正式支援指定 CA 檔，另開變更。

## 1. 蓋章事件帶 commit（D1）

- [x] 1.1 先寫失敗的測試，涵蓋需求「蓋章事件帶 commit 與流程設定變動的事件」的蓋章部分，寫在 crates/engine/speclink-core/src/command/tests.rs 與 crates/host/speclink-host/src/commit.rs 的測試區塊：審查蓋章與驗證蓋章帶 commit 時，結果與事件都帶同一個 commit，且與 metadata 記下的值相同；沒有帶 commit 時事件沒有 commit；`event_payload` 對有 commit 的蓋章事件輸出 `{change, commit}`，對沒有的輸出與現在逐位元相同的 `{change}`。驗證：`cargo test -p speclink-core command::tests` 與 `cargo test -p speclink-host commit::` 這時失敗。 <!-- speclink-task:tsk_01M7PP21Q5CCRMGJ4CDTT9RMWV -->
- [x] 1.2 實作 D1 蓋章事件的 commit 來自蓋章的結果：crates/engine/speclink-core/src/command/mod.rs 的蓋章結果型別與 `ReviewStamped`、`VerifyStamped` 新增選填的 commit，值取自實際寫進 metadata 的 commit；crates/host/speclink-host/src/commit.rs 的 `event_payload` 在有值時輸出 `commit`。同批更新手寫蓋章結果字面值的地方讓 workspace 編譯通過。驗證：1.1 的測試通過；`cargo build --workspace --tests` 通過；`cargo test -p speclink-cli --test it review_verbs` 與 `verify_verbs` 通過（CLI 的輸出不變）。 <!-- speclink-task:tsk_01M7PP21Q53J5ACJ20B3BK2SZ8 -->

## 2. 流程設定變動的事件（D2）

- [x] 2.1 先寫失敗的測試，涵蓋規格 delta 範例表的六列，寫在 crates/engine/speclink-core/src/command/workflow_config.rs 或 command/tests.rs 的測試區塊：改一個鍵回報一筆 keys 為 `["tdd"]` 的事件；整份取代同時改兩個鍵時 keys 依寫入結果的順序；只改註解時回報 keys 為空陣列的事件；設成現在的值、dry-run、不合法的 locale 都沒有事件；事件不含設定的值。另在 crates/host/speclink-host/tests/kit.rs 與 crates/host/speclink-host/tests/http_door.rs 斷言：經宿主套件寫入時，儲存的 commit 與提交後掛鉤都收到 `workflow-config-changed` 與同樣的 keys；推送的對照把它歸為 unknown，推送的訊息不含 keys。驗證：`cargo test -p speclink-core` 與 `cargo test -p speclink-host` 這時失敗。 <!-- speclink-task:tsk_01M7PP21Q56566S6H923SWZ66B -->
- [x] 2.2 實作 D2 流程設定的寫入在真的寫入時回報 workflow-config-changed：crates/engine/speclink-core/src/command/mod.rs 新增事件種類與名稱，事件清單的產生處對兩個寫入結果在 `written` 為真時回傳一筆；crates/host/speclink-host/src/commit.rs 的 `event_payload` 輸出 `{keys}`，資源對照不新增類別。同批更新既有測試中斷言流程設定寫入沒有事件的地方（搜尋 crates/host/speclink-host/tests 與 crates/adapters/speclink-node/__test__ 裡對流程設定寫入的事件斷言，逐一改成新的內容）。驗證：2.1 的測試通過；`cargo test -p speclink-core`、`cargo test -p speclink-host` 全部通過。 <!-- speclink-task:tsk_01M7PP21Q5AA7NJ1EHS5R7BV5A -->
- [x] 2.3 確認 Node 的宿主收到新的事件內容：在 crates/adapters/speclink-node/__test__/host.spec.ts 與 crates/adapters/speclink-node/__test__/workflow-config-verbs.spec.ts 新增或更新測試——帶 `headCommit` 的蓋章讓 JS 的 commit 函式與 afterCommit 收到帶 commit 的事件；流程設定的寫入讓兩者收到 `workflow-config-changed` 與 keys。驗證：`npm --prefix crates/adapters/speclink-node run build` 後 `npm --prefix crates/adapters/speclink-node test` 全部通過。 <!-- speclink-task:tsk_01M7PP21Q5BW11VKNHT0AFBZV9 -->

## 3. 串流網址的解析（D3）

- [x] 3.1 先寫失敗的測試，寫在 crates/protocol/speclink-remote/src/events.rs 的測試區塊：`stream_url` 對規格 delta 範例表七列的結果（相對路徑接在專案的 API 位址之後、同 origin 的完整網址原樣回傳、不同主機／協定／埠回傳沒有）；另加四列——專案的 API 位址帶結尾斜線、預設埠寫與不寫視為相同（`https://h` 與 `https://h:443`）、主機大小寫不同視為相同、`https://specs.example.com@evil.example/x` 回傳沒有；空字串與不以 `/` 開頭的相對路徑回傳沒有。驗證：`cargo test -p speclink-remote events::` 這時因函式不存在而編譯失敗。 <!-- speclink-task:tsk_01M7PP21Q58EHATPN97CCQJYAH -->
- [x] 3.2 實作 D3 串流網址的解析是 speclink-remote 的一個純函式：新增 `stream_url`，用 `url` 套件解析並比較 origin；`subscribe` 的第一個參數改成串流的完整網址、不再自己接 `/events`，其餘行為不變；crates/protocol/speclink-remote/tests/it/events_sse.rs 的既有測試改成先經 `stream_url` 取得網址再訂閱。驗證：3.1 的測試通過；`cargo test -p speclink-remote` 全部通過。 <!-- speclink-task:tsk_01M7PP21Q5B4C4B1GDTTA3MWMN -->

## 4. Speclink Desktop 的訂閱（D4）

- [x] 4.1 先寫失敗的測試，涵蓋需求「事件串流的網址取自握手的宣告」，寫在 apps/desktop/src-tauri/src/remote.rs 的測試區塊：握手宣告 `/events` 時保存的串流網址是專案的 API 位址加 `/events`；宣告同 origin 的完整網址時原樣保存；宣告不同 origin、只宣告 websocket、沒有宣告 transports 時保存的是沒有；宣告多個 sse 時用第一個；`live_updates` 的值與現在相同。驗證：先補齊桌面測試需要的 sidecar 與 server-web dist，`cargo test -p speclink-desktop remote::` 這時失敗。 <!-- speclink-task:tsk_01M7PP21Q5WE9NCGA481DTHJYG -->
- [x] 4.2 實作 D4 Speclink Desktop 把握手宣告的第一個 SSE 網址交給訂閱：apps/desktop/src-tauri/src/remote.rs 在握手之後算出選填的串流網址並保存；apps/desktop/src-tauri/src/lib.rs 註冊訂閱函式時有網址就訂閱它，沒有網址就不發請求、回傳串流不可用的錯誤，交給事件管理器既有的輪詢路徑。驗證：4.1 的測試通過；`cargo test -p speclink-desktop` 全部通過；跑完後執行 `cargo build -p speclink-cli` 讓 CLI 的 binary 回到最新。 <!-- speclink-task:tsk_01M7PP21Q5WQ151AJBMX5Y9255 -->
- [x] [M] 4.3 用 root 的 `npm run dev:desktop` 開啟 Speclink Desktop 並連到本機的示範用 server：在 CLI 勾選一個任務，確認看板不用手動重新整理就更新（即時更新照常運作）；停掉 server 再啟動，確認畫面顯示連線中斷後自己恢復。 <!-- speclink-task:tsk_01M7PP21Q5SQQ0TDRE37FJ111Q -->

## 5. 稽核

- [x] 5.1 對串流網址的解析與兩種事件套用 sharp-edges 稽核清單（用 `speclink instructions --skill audit` 取得），把結論寫在 crates/protocol/speclink-remote/src/events.rs 的 `stream_url` 說明註解：憑證只會送到與 Server 同 origin 的網址，比較用解析後的協定、主機與埠；無法解析的宣告一律當成沒有；`workflow-config-changed` 與蓋章事件都不含設定的值、token 或文件內容；推送給用戶端的訊息不含事件的內容。驗證：`cargo test -p speclink-remote` 與 `cargo test -p speclink-host` 全部通過。 <!-- speclink-task:tsk_01M7PP21Q5K61AE4G8AQBDRKXM -->

## 6. 文件與收尾（D5）

- [x] 6.1 依 D5 文件寫明事件的設計、網址的寫法與心跳，中英兩版同步（需求「宣告的串流網址的寫法與心跳的契約」）：docs/verb-contract.md 與 docs/verb-contract.zh-TW.md 的事件一節加上事件設計的說明、url 的兩種寫法與同 origin 的限制、15 秒與 45 秒的規則、兩種事件的新內容；docs/sdk-node.md 與 docs/sdk-node.zh-TW.md 的 `pushTransports` 段落與四個步驟一節同步，crates/adapters/speclink-node/index.d.ts 的 `HostPushTransport` 說明同步；docs/product-status.md 與 docs/product-status.zh-TW.md 推送事件那一列更新。驗證：文件的數字與 crates/protocol/speclink-remote/src/events.rs 的斷線時限、示範用 server 的心跳預設值一致；文件的事件內容與 1.1、2.1 的測試斷言一致；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全部通過。 <!-- speclink-task:tsk_01M7PP21Q51KXRZ4WC6CCCR60G -->

## 7. 連線用解析後的網址（D3 修訂）

- [x] 7.1 先改測試，涵蓋需求「事件串流的網址取自握手的宣告」修訂後的正規寫法，寫在 crates/protocol/speclink-remote/src/events.rs 的測試區塊：主機大小寫不同的宣告 `https://SPECS.Example.com/push/acme` 回傳 `https://specs.example.com/push/acme`；寫出預設埠的 `https://specs.example.com:443/push` 回傳 `https://specs.example.com/push`；規格 delta 範例表新增的那一列 `https://SPECS.Example.com:443/push/acme` 回傳 `https://specs.example.com/push/acme`；相對路徑 `/a/../events` 回傳專案的 API 位址加 `/events`；已經是正規寫法的宣告（範例表其餘各列）回傳的文字與現在逐字相同。驗證：`cargo test -p speclink-remote events::` 這時失敗（大小寫與預設埠兩列仍回傳宣告的原始字串）。 <!-- speclink-task:tsk_01M7RR93FRDTR66XD5X66BW4PY -->
- [x] 7.2 實作 D3 的修訂：crates/protocol/speclink-remote/src/events.rs 的 `stream_url` 回傳的 `StreamUrl` 改存解析結果的正規寫法（`url::Url` 的序列化），`subscribe` 連線用的就是它；`stream_url` 與 `StreamUrl` 的說明註解同步（不再寫「照宣告的原樣使用」）。驗證：7.1 的測試通過；`cargo test -p speclink-remote` 全部通過；`cargo test -p speclink-desktop` 全部通過（apps/desktop/src-tauri/src/remote.rs 的握手表格測試用的都是正規寫法，期望值不變），跑完後執行 `cargo build -p speclink-cli`。 <!-- speclink-task:tsk_01M7RR93FR4MYBNWC28TBYRPDF -->

## 8. 示範用 server 拒絕 45 秒以上的心跳（D6）

- [x] 8.1 先寫失敗的測試，涵蓋規格 delta 範例表「各種 heartbeat_secs 的結果」五列，寫在 crates/host/speclink-server/src/config.rs 的測試區塊：沒有 `events` 區段時心跳是 15 秒；`heartbeat_secs: 5` 與 `44` 照常讀取；`45` 與 `60` 回 `ConfigError::InvalidValue`，訊息含 `'events.heartbeat_secs' must be less than 45`。另在 crates/protocol/speclink-remote/src/events.rs 的測試區塊斷言 `STALL_LIMIT` 等於 `speclink_protocol::events::STALL_LIMIT_SECS` 秒。驗證：`cargo test -p speclink-server --lib config::` 這時失敗（45 與 60 被接受）；`cargo test -p speclink-remote events::` 這時因常數不存在而編譯失敗。 <!-- speclink-task:tsk_01M7RR93FR9RQGJ4TPXMGQ1G60 -->
- [x] 8.2 實作 D6 45 秒的斷線時限只有一個來源，示範用 server 拒絕不小於它的心跳：crates/protocol/speclink-protocol/src/events.rs 新增 `STALL_LIMIT_SECS`；crates/protocol/speclink-remote/src/events.rs 的 `STALL_LIMIT` 改由它算出；crates/host/speclink-server/src/config.rs 在 `events.heartbeat_secs` 大於或等於它時回 `InvalidValue`（欄位 `events.heartbeat_secs`、規則 `must be less than 45`）。先搜尋 crates/host/speclink-server 與 apps/desktop/src-tauri/tests 裡寫了 `heartbeat_secs` 的設定內容，確認沒有 45 以上的值會被新規則擋下。驗證：8.1 的測試通過；`cargo test -p speclink-protocol`、`cargo test -p speclink-remote`、`cargo test -p speclink-server --lib` 全部通過。 <!-- speclink-task:tsk_01M7RR93FRVF91BP5MR4A106BJ -->

## 9. 文件與稽核結論的位置（D7）

- [x] 9.1 文件與註解同步：docs/server-deployment.md 與 docs/server-deployment.zh-TW.md 的 `events` 那一列寫明 `heartbeat_secs` 必須小於 45、45 以上時 server 不啟動；docs/verb-contract.md 與 docs/verb-contract.zh-TW.md 心跳那一條寫明示範用 server 的 `events.heartbeat_secs` 必須小於 45。依 D7 稽核結論各放在它講的程式旁邊：把 crates/protocol/speclink-remote/src/events.rs 的 `stream_url` 說明註解中「事件內容不含設定的值、token 或文件內容，推送不含事件的內容」那一點搬到 crates/host/speclink-host/src/commit.rs 的 `event_payload` 說明註解，`stream_url` 的註解留一句指向它。驗證：文件的數字與 `STALL_LIMIT_SECS`、示範用 server 的預設值一致；`cargo test -p speclink-host --lib commit::` 通過；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全部通過。 <!-- speclink-task:tsk_01M7RR93FRDQBHKR0CNFVD0GBM -->

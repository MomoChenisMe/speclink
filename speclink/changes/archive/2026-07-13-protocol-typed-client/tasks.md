## 1. protocol crate 與 DTO（design 決策一：protocol 為獨立 crate，依賴方向 remote → protocol、未來 server → protocol；決策二：Rust 型別為正典、JSON Schema 為匯出物）

- [x] 1.1 建立 crate 並撰寫失敗測試，覆蓋「protocol 型別是 wire contract 的唯一定義」：Command／Query／Context DTO 的序列化往返（欄位全 camelCase）、JSON Schema 匯出成功、API version 常數存在且進 handshake 回應型別（crates/speclink-protocol/src/command.rs、crates/speclink-protocol/src/query.rs、crates/speclink-protocol/src/context.rs 的 #[cfg(test)]；根 Cargo.toml 追加 workspace 成員、schemars 依賴僅限本 crate）。cargo test -p speclink-protocol 觀察紅燈。
- [x] 1.2 實作 protocol DTO 模組（command／query／context／events 宣告型別，events 對齊 transports 陣列與 polling 宣告形狀），1.1 轉綠；驗證 cargo build --release 全 workspace 編譯通過且 speclink-protocol 不依賴 speclink-core、speclink-host 或 speclink-store。

## 2. error reason registry（design 決策三：error reason registry 正式化既有 mapping 經驗）

- [x] 2.1 撰寫失敗測試，覆蓋「標準 error reason registry」：錯誤回應為 status／reason／message 三元組；reason 封閉八值（not_found、permission_denied、revision_conflict、invalid_argument、invalid_config、refused、unavailable、internal）；未知 reason 字串反序列化不失敗、可作一般錯誤處理（crates/speclink-protocol/src/error.rs 測試）。紅燈。
- [x] 2.2 實作 error 型別與 registry，2.1 轉綠；驗證 cargo test -p speclink-protocol 全綠。

## 3. binding handshake（design 決策四：binding handshake 為 client 連線前置，fail closed）

- [x] 3.1 撰寫失敗測試，覆蓋「binding handshake 前置且 fail closed」：handshake 回應型別含 actor／project／repo／apiVersion／engineVersion／capabilities.events；stub server 宣告不相容 apiVersion 時 client 回帶版本原因的拒絕且不送後續請求；binding 多義回拒絕列候選；capabilities 的 sse 與 polling 宣告解析保存且不建立任何事件連線（crates/speclink-protocol/src/binding.rs 與 crates/speclink-remote/src/client.rs 的 stub 對測）。紅燈。
- [x] 3.2 實作 handshake DTO 與 speclink-remote 的 handshake 呼叫（沿既有 twin stub server 基建擴充 handshake 端點），3.1 轉綠；驗證 cargo test -p speclink-remote 全綠。

## 4. typed client 重構與攔截層薄化（design 決策五：remote 攔截層收編為薄轉譯層、輸出凍結；決策六：client 對測沿 twin harness 的 stub server 基建）

- [x] 4.1 撰寫失敗測試，覆蓋「typed client 全面取代 raw JSON 旁路」的 client 面：各動詞請求路徑與 body 符合 protocol DTO；帶 If-Match 的寫入在 stub 判定 revision 前進時收到 revision_conflict 且對映現行衝突訊息逐位元一致；每個 registry reason 對映的 CLI 訊息與現行 remote error translation 逐位元一致（crates/speclink-remote/src/client.rs 的 stub 對測；先為每動詞留存現行 stub 回應樣本）。紅燈。
- [x] 4.2 實作 speclink-remote typed 化：client 收發全改 protocol DTO、error translation 遷入 reason 對映、auth 與 project-scoped URL 語意保留（crates/speclink-remote/src/client.rs、crates/speclink-remote/src/lib.rs、crates/speclink-remote/src/auth.rs），4.1 轉綠。
- [x] 4.3 撰寫並轉綠攔截層測試，覆蓋「remote 動詞經 handshake 建立的連線語境執行」：handshake 因 binding 多義失敗時任一 remote 動詞非零 exit、stderr 列候選、無動詞請求送出；現行 .speclink.yaml remote 區段無需修改即可運作；隨後把 crates/speclink-cli/src/remote_commands.rs 薄化為「argv → typed client → 與 fs 相同渲染」並移除 raw payload 重組碼。驗證 twin harness 8 情境全綠（remote 與 fs 輸出逐位元一致）。
- [x] 4.4 撰寫並轉綠殘留盤點斷言：crates/speclink-remote 與 crates/speclink-cli/src/remote_commands.rs 的 wire payload 處理零 serde_json::Value 命中（測試化 grep 或 CI 步驟）。

## 5. 全量收尾

- [x] 5.1 對新公開 API 跑 sharp-edges 稽核檢查表（speclink instructions --skill audit）並修正發現；驗證 cargo test --workspace 與 npm run test:all 全綠；parity 31 項／color 16 項全綠；git diff --stat 對照 proposal Impact 清單檢查改動面無溢出（fs 模式路徑零改動）。

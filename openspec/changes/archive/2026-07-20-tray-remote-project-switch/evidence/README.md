# macOS 真實視窗驗證（2026-07-20）

驗證程式：`target/release/speclink-desktop`（先執行 `npm run build -w apps/desktop` 與 `cargo build --release --bin speclink-desktop`）。

驗證環境含兩個 local 分頁與一個 `Demo 專案/backend` remote 分頁；remote server 為本機 `127.0.0.1:8787` 測試服務。

## 結果

1. [主視窗 remote 基線](01-main-remote-baseline.png)：三個分頁成功還原，remote handshake 成功並顯示 server 看板資料。
2. [tray 切至 local](02-tray-local-switch.png)：`speclink-test-b` 轉為實心主色，面板內容切為 local 專案；面板保持開啟，背景仍為 ChatGPT，Speclink 主視窗未出現。
3. [tray 切回 remote](03-tray-remote-switch.png)：`Demo 專案/backend` 轉為實心主色，面板內容切回 remote 資料；面板保持開啟，背景仍為 ChatGPT，Speclink 主視窗未出現。
4. [remote 失敗時 tray 靜默](04-tray-remote-failure-silent.png)：短暫停止測試 server 後由 local 點 remote；remote tab 成為作用中，tray 未新增錯誤 UI、面板未收合、app 未崩潰。
5. [看板分頁錯誤態](05-board-remote-error.png)：明確點「開啟 Speclink」後，remote 分頁顯示橘色警示；hover 顯示 `server unreachable`，錯誤沿用既有 `tabErrors` 呈現。
6. [server 恢復後](06-tray-remote-recovered.png)：以原命令恢復 server 並確認 `/healthz` 成功；再次由 tray 點 remote 後資料與徽章恢復，分頁警示清除。

測試期間只短暫停止已確認啟動命令與設定路徑的本機測試 server；驗證結束前已恢復服務。

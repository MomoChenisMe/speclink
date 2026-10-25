## MODIFIED Requirements

### Requirement: 單獨啟動 desktop

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

repo root SHALL 提供 npm run dev:desktop：啟動 desktop 的 tauri dev，其前端由 tauri dev 自身啟動的 Vite dev server 供應，SHALL NOT 由編排 script 於啟動前另行建置前端，SHALL NOT 啟動 speclink-server、SHALL NOT 要求任何 remote 設定。設定驗證 SHALL 與 npm run dev 共用——.env 不合法時（例如 postgres 缺 SPECLINK_POSTGRES_URL）SHALL 以非零 exit code 拒絕啟動。Vite dev server 無法啟動時 SHALL 以非零結束且不留下已開啟的 dev 視窗。

#### Scenario: 修改前端後視窗呈現最新畫面

- **WHEN** 修改 desktop 前端原始碼後執行 npm run dev:desktop
- **THEN** tauri dev 開啟的視窗呈現本次修改後的畫面，且後續再次修改前端原始碼並存檔時，視窗同樣反映最新內容而不需重啟

#### Scenario: dev server 無法啟動即拒絕啟動

- **WHEN** Vite dev server 以非零狀態結束或無法啟動（例如連接埠被占用）
- **THEN** npm run dev:desktop 以非零 exit code 結束，錯誤訊息輸出至終端，不留下已開啟的 dev 視窗

#### Scenario: 無 server 亦可用

- **WHEN** 機器上沒有任何 speclink-server 在跑時執行 npm run dev:desktop
- **THEN** desktop 視窗以本地模式開啟並可瀏覽本地 speclink/ 看板，不因 remote 不可達而阻擋啟動

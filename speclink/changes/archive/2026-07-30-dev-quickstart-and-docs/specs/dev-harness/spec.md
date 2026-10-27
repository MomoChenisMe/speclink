## ADDED Requirements

### Requirement: 單獨啟動 server

repo root SHALL 提供 npm run dev:server：只驗證 dev 設定並啟動 speclink-server，SHALL NOT 建置 CLI、SHALL NOT 建置 desktop 前端、SHALL NOT 啟動 desktop。設定來源與預設值（.env 合併 process env、sqlite、.dev/store.db、identity .dev/identity.db、127.0.0.1:8080）、輸出直通（server 首跑的一次性 /setup 連結原樣可見）、SIGINT/SIGTERM 收束與 .dev 持久化 SHALL 與 npm run dev 完全一致。

#### Scenario: 全新 checkout 零設定啟動後端

- **WHEN** 在沒有 .env、沒有 .dev/ 的全新 checkout 執行 npm run dev:server
- **THEN** server 以全預設啟動、終端出現含 /setup?token= 的連結行，過程中沒有 CLI 建置、沒有前端建置、沒有 desktop 視窗

#### Scenario: 設定不合法即拒絕啟動

- **WHEN** SPECLINK_STORE_DRIVER=postgres 且未設 SPECLINK_POSTGRES_URL 時執行 npm run dev:server
- **THEN** script 以非零 exit code 結束並顯示與 npm run dev 相同的錯誤訊息，server 未啟動

#### Scenario: 中斷收束無殘留

- **WHEN** npm run dev:server 執行中收到 SIGINT
- **THEN** server process 終止且無任何 process 殘留

### Requirement: 單獨啟動 desktop

repo root SHALL 提供 npm run dev:desktop：先建置 desktop 前端（vite 產出 dist）再啟動 desktop 的 tauri dev，SHALL NOT 啟動 speclink-server、SHALL NOT 要求任何 remote 設定。設定驗證 SHALL 與 npm run dev 共用——.env 不合法時（例如 postgres 缺 SPECLINK_POSTGRES_URL）SHALL 以非零 exit code 拒絕啟動。前端建置失敗時 SHALL 以非零結束且不啟動 tauri dev——tauri dev 載入靜態 dist，跳過建置會靜默沿用過期畫面。

#### Scenario: 前端先建置再啟動

- **WHEN** 修改 desktop 前端原始碼後執行 npm run dev:desktop
- **THEN** 前端建置先完成，tauri dev 開啟的視窗呈現本次修改後的畫面，而非過期 dist

#### Scenario: 前端建置失敗即拒絕啟動

- **WHEN** desktop 前端建置以非零狀態結束
- **THEN** npm run dev:desktop 以非零 exit code 結束，tauri dev 未啟動

#### Scenario: 無 server 亦可用

- **WHEN** 機器上沒有任何 speclink-server 在跑時執行 npm run dev:desktop
- **THEN** desktop 視窗以本地模式開啟並可瀏覽本地 openspec/ 看板，不因 remote 不可達而阻擋啟動

## MODIFIED Requirements

### Requirement: checkout 內 CLI 測試入口

<!-- BEFORE: binary 不存在時於 stderr 報錯並以非零結束（提示先跑 npm run dev 或 cargo build），不自動建置 -->

repo root SHALL 提供 npm run cli -- <args>，固定執行同一 checkout 的 target/debug/speclink；Windows SHALL 使用 target/debug/speclink.exe。該 binary 不存在時，wrapper SHALL 先於 checkout root 建置 speclink-cli 再執行建置產物；建置進度輸出 SHALL NOT 寫入 stdout；建置失敗時 SHALL 於 stderr 顯示原因並以非零 exit code 結束。wrapper SHALL NOT 查詢或 fallback 到 PATH 中的 speclink，SHALL 原序轉送 `<args>`、繼承 environment 與 stdin/stdout/stderr，並回傳既有 CLI 的 exit code。child 工作目錄 SHALL 優先採用 npm 的 INIT_CWD，該值不存在時 SHALL 採用 wrapper 的 process.cwd()；自動建置 SHALL 於 checkout root 執行、不受呼叫端工作目錄影響。wrapper 不新增子指令、旗標、stdin 格式、輸出 envelope 或檔案系統效果（target/debug 的建置產物除外）；既有 --json camelCase payload、--no-color 與人眼輸出行為 SHALL 保持不變。

#### Scenario: PATH 中舊版 CLI 不影響 checkout binary

- **WHEN** PATH 中已有另一版 speclink，且目前 checkout 的 target/debug/speclink 已由 npm run dev 建置後執行 npm run cli -- status
- **THEN** wrapper 只執行目前 checkout 的 debug binary，並將 status 參數原序傳入

#### Scenario: 從外部測試 repo 保留呼叫端工作目錄

- **WHEN** 使用者位於 /tmp/remote-client，透過 npm --prefix <speclink-checkout> run cli -- list 呼叫 wrapper，且 INIT_CWD 為 /tmp/remote-client
- **THEN** CLI child 的工作目錄為 /tmp/remote-client，而 binary 仍來自 <speclink-checkout>/target/debug

#### Scenario: 互動輸入輸出與成功狀態透明轉送

- **WHEN** CLI 子指令讀取 stdin、寫入 stdout/stderr 並以 exit code 0 結束
- **THEN** wrapper 以 inherit 模式轉送 stdin/stdout/stderr，且 npm CLI script 以 exit code 0 結束

#### Scenario: CLI 失敗狀態透明轉送

- **WHEN** checkout CLI 因錯誤輸入、找不到變更或驗證失敗而以非零 exit code 結束
- **THEN** wrapper 保留 CLI 寫入 stdout/stderr 的內容並回傳相同的非零 exit code

#### Scenario: checkout binary 不存在時自動建置且禁止 fallback

- **WHEN** target/debug/speclink（Windows 為 speclink.exe）不存在，且 PATH 中存在可執行的 speclink，執行 npm run cli -- status
- **THEN** wrapper 先於 checkout root 建置 speclink-cli，再執行建置出的 debug binary 並將 status 原序傳入，SHALL NOT 執行 PATH 中的 speclink

#### Scenario: 自動建置失敗以非零收場

- **WHEN** binary 不存在且自動建置以非零狀態結束
- **THEN** wrapper 於 stderr 顯示建置失敗原因、以非零 exit code 結束，且 SHALL NOT 執行 PATH 中的 speclink

#### Scenario: machine-readable 輸出維持既有契約

- **WHEN** 使用 npm run --silent cli -- <args> 傳入既有 --json 或 --no-color 旗標（含觸發自動建置的情況）
- **THEN** wrapper 與自動建置皆不增加 stdout 內容，CLI 的 --json camelCase payload、--no-color 人眼文字與 exit code 維持既有位元級輸出契約

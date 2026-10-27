## ADDED Requirements

### Requirement: dev 模式前端由 dev server 供應且變更免重編

desktop 的 tauri dev SHALL 由 Vite dev server 供應前端，而非載入編譯期嵌入的靜態產物。修改 apps/desktop 前端原始碼後，變更 SHALL 反映於已開啟的 dev 視窗，且 SHALL NOT 要求重編 Rust binary、SHALL NOT 要求重啟 tauri dev。此契約同時適用於 npm run dev 與 npm run dev:desktop 兩條啟動路徑，並涵蓋 desktop 的兩個 HTML 進入點（主視窗與系統匣面板）。

Vite dev server SHALL 固定使用連接埠 1420 並啟用嚴格連接埠模式：該埠被占用時 SHALL 以非零狀態結束並將錯誤輸出至終端，SHALL NOT 改用其他連接埠。release 與 bundle 路徑 SHALL 維持讀取靜態產物，不受此需求影響。

#### Scenario: 修改前端後視窗即時反映

- **WHEN** npm run dev 執行中，修改 apps/desktop 前端原始碼中一處可見文字並存檔
- **THEN** dev 視窗顯示修改後的文字，終端未出現 Rust 重編輸出，且 tauri dev process 未重啟

#### Scenario: 系統匣面板同樣由 dev server 供應

- **WHEN** dev 視窗執行中開啟系統匣面板
- **THEN** 面板視窗正常顯示內容，其前端同樣來自 dev server，修改面板前端原始碼後存檔亦即時反映

#### Scenario: 連接埠被占用即明確失敗

- **WHEN** 連接埠 1420 已被其他 process 占用時執行 npm run dev:desktop
- **THEN** 指令以非零 exit code 結束，終端出現連接埠被占用的錯誤訊息，SHALL NOT 靜默改用其他連接埠而開出載入失敗的空白視窗

#### Scenario: release 產出不受影響

- **WHEN** 執行 desktop 的 release bundle 建置
- **THEN** bundle 內的前端仍取自靜態產物目錄，產物內容與本需求導入前一致

## MODIFIED Requirements

### Requirement: 一鍵啟動 remote 開發環境

<!-- BEFORE: CLI build 成功後先建置 Desktop 前端，再同時啟動 server 與 tauri dev -->

repo root 的 npm run dev SHALL 先驗證 dev 設定並建置目前 checkout 的 speclink-cli；CLI build 成功後，才 SHALL 同時啟動 speclink-server（依 env 驅動的 dev 設定）與 desktop 的 tauri dev，且不依賴 docker 或 PATH 中已安裝的 speclink。Desktop 前端 SHALL NOT 由編排 script 於啟動前另行建置——前端由 tauri dev 自身啟動的 Vite dev server 供應。編排 script SHALL 將兩個長時間 child process 的輸出直通終端——server 首跑印出的一次性 /setup 連結必須原樣可見。收到 SIGINT/SIGTERM 或任一 child 先退出時，script SHALL 終止另一個 child 一併收束，不留殘留 process。npm run dev:reset SHALL 保持只執行重置，不觸發 CLI 或 Desktop build。

#### Scenario: 全新 checkout 且未安裝 CLI 仍可啟動

- **WHEN** 在沒有 .env、沒有 .dev/ 且 PATH 中沒有 speclink 的全新 checkout 執行 npm run dev
- **THEN** script 先於 target/debug 建置目前 checkout 的 speclink-cli，再讓 server 以全預設（sqlite、.dev/store.db、identity .dev/identity.db、127.0.0.1:8080）啟動，終端出現含 /setup?token= 的連結行，desktop dev 視窗同時開啟

#### Scenario: CLI build 失敗即拒絕啟動

- **WHEN** 目前 checkout 的 speclink-cli build 以非零狀態結束或無法啟動
- **THEN** npm run dev 以非零 exit code 結束，且 speclink-server 與 desktop dev 的長時間 process 皆未啟動

#### Scenario: Ctrl+C 同殺兩個 child

- **WHEN** npm run dev 執行中於終端按 Ctrl+C
- **THEN** server 與 desktop dev 兩個 process 皆終止，無任一 process 殘留，且 CLI 不在長時間 child 清單中

#### Scenario: 啟動前不另行建置前端

- **WHEN** 執行 npm run dev 並觀察啟動至 desktop 視窗開啟之間的終端輸出
- **THEN** 輸出中不含編排 script 發起的 desktop 前端建置步驟，前端改由 tauri dev 啟動的 dev server 供應

### Requirement: 單獨啟動 desktop

<!-- BEFORE: 先以 vite 建置 dist 再啟動 tauri dev，前端建置失敗即拒絕啟動 -->

repo root SHALL 提供 npm run dev:desktop：啟動 desktop 的 tauri dev，其前端由 tauri dev 自身啟動的 Vite dev server 供應，SHALL NOT 由編排 script 於啟動前另行建置前端，SHALL NOT 啟動 speclink-server、SHALL NOT 要求任何 remote 設定。設定驗證 SHALL 與 npm run dev 共用——.env 不合法時（例如 postgres 缺 SPECLINK_POSTGRES_URL）SHALL 以非零 exit code 拒絕啟動。Vite dev server 無法啟動時 SHALL 以非零結束且不留下已開啟的 dev 視窗。

<!-- REMOVED-SCENARIO: 前端先建置再啟動 -->
<!-- REMOVED-SCENARIO: 前端建置失敗即拒絕啟動 -->

#### Scenario: 修改前端後視窗呈現最新畫面

- **WHEN** 修改 desktop 前端原始碼後執行 npm run dev:desktop
- **THEN** tauri dev 開啟的視窗呈現本次修改後的畫面，且後續再次修改前端原始碼並存檔時，視窗同樣反映最新內容而不需重啟

#### Scenario: dev server 無法啟動即拒絕啟動

- **WHEN** Vite dev server 以非零狀態結束或無法啟動（例如連接埠被占用）
- **THEN** npm run dev:desktop 以非零 exit code 結束，錯誤訊息輸出至終端，不留下已開啟的 dev 視窗

#### Scenario: 無 server 亦可用

- **WHEN** 機器上沒有任何 speclink-server 在跑時執行 npm run dev:desktop
- **THEN** desktop 視窗以本地模式開啟並可瀏覽本地 openspec/ 看板，不因 remote 不可達而阻擋啟動

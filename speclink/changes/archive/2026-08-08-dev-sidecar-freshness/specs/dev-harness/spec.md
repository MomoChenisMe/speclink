## ADDED Requirements

### Requirement: dev 啟動自動佈署當前 checkout 的 sidecar

desktop 的 dev 啟動流程 SHALL 於前端 dev server 與 Rust 編譯開始前，將當前 checkout 建置的 speclink CLI（debug 建置，與 npm run cli 驗證所用同一顆）佈署至 apps/desktop/src-tauri/binaries/speclink-<triple>。此佈署 SHALL 涵蓋所有 dev 啟動入口——repo root 的 npm run dev、npm run dev:desktop，以及直接於 apps/desktop 執行 tauri dev。

sidecar 內容與已佈署檔案相同時，佈署 SHALL NOT 改寫該檔案——該路徑是 Rust 重編的觸發來源，無謂改寫會使每次啟動都多一輪重編。sidecar 建置或佈署失敗時，dev 啟動 SHALL 以非零狀態中止且 dev 視窗不開啟，SHALL NOT 以缺檔或過期檔繼續。本機安裝與 release 的佈署 SHALL 維持 release 建置與現行行為不變。

#### Scenario: 全新 checkout 首次啟動不因缺 sidecar 失敗

- **WHEN** 在沒有 apps/desktop/src-tauri/binaries/ 的全新 checkout 執行 npm run dev:desktop
- **THEN** 終端先出現 sidecar 建置與佈署輸出，該檔案就位後編譯繼續，dev 視窗正常開啟

#### Scenario: 修改 CLI 原始碼後啟動佈署新內容

- **WHEN** 修改 CLI 相關原始碼後執行任一 dev 啟動入口
- **THEN** apps/desktop/src-tauri/binaries/speclink-<triple> 與 target/debug/speclink 內容一致，皆為當前 checkout 的建置結果

#### Scenario: 內容未變不改寫亦不觸發重編

- **WHEN** CLI 原始碼未變動且 sidecar 已與當前建置一致時再次啟動 dev
- **THEN** sidecar 檔案未被改寫，終端未因 sidecar 出現 speclink-desktop 的重編輸出

#### Scenario: 佈署失敗即中止啟動

- **WHEN** sidecar 佈署腳本以非零狀態結束（例如收到白名單外的 profile 值，stderr 點名該值）
- **THEN** dev 啟動以非零 exit code 中止，dev 視窗未開啟

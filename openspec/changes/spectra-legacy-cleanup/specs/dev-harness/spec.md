## MODIFIED Requirements

### Requirement: checkout 內 CLI 測試入口

repo root SHALL 提供 npm run cli -- <args>，固定執行同一 checkout 的 target/debug/speclink；Windows SHALL 使用 target/debug/speclink.exe。wrapper SHALL NOT 查詢或 fallback 到 PATH 中的 speclink，SHALL 原序轉送 `<args>`、繼承 environment 與 stdin/stdout/stderr，並回傳既有 CLI 的 exit code。child 工作目錄 SHALL 優先採用 npm 的 INIT_CWD，該值不存在時 SHALL 採用 wrapper 的 process.cwd()。wrapper 不新增子指令、旗標、stdin 格式、輸出 envelope 或檔案系統效果；既有 --json camelCase payload、--no-color 與人眼輸出行為 SHALL 保持不變。

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

#### Scenario: checkout binary 不存在時禁止 fallback

- **WHEN** target/debug/speclink（Windows 為 speclink.exe）不存在或無法執行，且 PATH 中存在可執行的 speclink
- **THEN** wrapper 在 stderr 顯示 checkout CLI 無法執行，以非零 exit code 結束，且 SHALL NOT 執行 PATH 中的 speclink

#### Scenario: machine-readable 輸出維持既有契約

- **WHEN** 使用 npm run --silent cli -- <args> 傳入既有 --json 或 --no-color 旗標
- **THEN** wrapper 不增加 stdout 內容，CLI 的 --json camelCase payload、--no-color 人眼文字與 exit code 維持既有位元級輸出契約

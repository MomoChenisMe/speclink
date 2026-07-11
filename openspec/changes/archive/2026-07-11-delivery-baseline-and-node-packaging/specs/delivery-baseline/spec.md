## ADDED Requirements

### Requirement: Node 套件安裝確定性

`crates/speclink-node` SHALL 在乾淨環境下以 npm ci 成功安裝依賴：package.json 與 package-lock.json 保持同步，且 committed 的 package.json SHALL NOT 宣告未發佈至 npm registry 的套件依賴。

#### Scenario: 乾淨環境 npm ci 成功

- **WHEN** 在乾淨 checkout（無 node_modules）的 crates/speclink-node 目錄執行 npm ci
- **THEN** 指令以 exit code 0 完成，且後續 napi build 與 vitest 測試可正常執行

#### Scenario: lock file 與 package.json 同步

- **WHEN** package.json 的依賴宣告有任何變更
- **THEN** package-lock.json 同步更新，npm ci 不出現 Missing from lock file 錯誤

### Requirement: root 單一指令全量驗證

repo root SHALL 提供單一指令，依序執行 Rust workspace、`packages/ui`、`apps/desktop` 與 `crates/speclink-node` 四個測試面的測試，並在任一面失敗時以非零 exit code 中止。

#### Scenario: 全部通過

- **WHEN** 四個測試面皆通過時於 root 執行該指令
- **THEN** 指令依序完成四面並以 exit code 0 結束

#### Scenario: 任一面失敗即中止

- **WHEN** 任一測試面存在失敗測試時於 root 執行該指令
- **THEN** 指令於該測試面以非零 exit code 中止，不繼續執行後續測試面

### Requirement: CI 執行完整測試

主 CI SHALL 在三個作業系統（Windows、macOS、Linux）上執行 cargo test --workspace 與 npm workspace 測試（`packages/ui`、`apps/desktop`），而非僅 build 與 smoke；測試步驟 SHALL NOT 設定 continue-on-error。

#### Scenario: 測試失敗使 CI 紅燈

- **WHEN** 任一平台上任一測試失敗
- **THEN** 該 CI workflow 以失敗狀態結束，不得標記為允許失敗

#### Scenario: push 觸發完整測試

- **WHEN** push 或 pull request 觸發主 CI
- **THEN** workflow 執行 Rust workspace 測試與 npm workspace 測試，全數通過才回報成功

### Requirement: Node native 套件全平台交付驗證

Node SDK 的 CI SHALL 在五個宣告平台（x86_64-pc-windows-msvc、x86_64-apple-darwin、aarch64-apple-darwin、x86_64-unknown-linux-gnu、aarch64-unknown-linux-gnu）完成 native module build，且 SHALL 在可原生執行的平台上以測試驗證 build 產物可載入。

#### Scenario: 五平台 build 成功

- **WHEN** 變更觸發 Node SDK workflow
- **THEN** 五個平台的 native module build job 全數成功並上傳 binary artifact

#### Scenario: 可執行平台的載入驗證

- **WHEN** build 完成於可原生執行的平台（win32-x64、darwin-arm64、linux-x64）
- **THEN** 該平台對 build 產物執行測試套件並全數通過

### Requirement: 測試輸出無 React act 警告

`packages/ui` 與 `apps/desktop` 的測試輸出 SHALL NOT 含 React act(...) 警告（"not wrapped in act" 字樣）；async 狀態更新 SHALL 在測試中被明確等待，而非被壓制或過濾。

#### Scenario: 測試輸出檢查零命中

- **WHEN** 執行 packages/ui 與 apps/desktop 的完整測試套件並檢查輸出
- **THEN** 輸出不含 "not wrapped in act" 字樣，且所有測試通過

#### Scenario: 禁止以壓制方式清零

- **WHEN** 檢視測試設定與測試檔的警告處理方式
- **THEN** 不存在對 act 警告的 console 過濾或全域壓制設定，警告消除一律來自測試側的明確等待

## MODIFIED Requirements

### Requirement: CI 執行完整測試

主 CI SHALL 在三個作業系統（Windows、macOS、Linux）上執行 `scripts` 測試面、`packages/ui`、`apps/desktop`、`apps/server-web` 測試、Server Web production build 與 `cargo test --workspace`，而非僅 build 與 smoke；測試與 Web build 步驟 SHALL NOT 設定 `continue-on-error`。`scripts` 測試面守的是設定檔本身的契約（workflow 步驟順序、release 產物組裝、簽章閘門），SHALL 排在其他測試面之前，且其執行方式 SHALL 相容於本 workflow 釘選的 Node 版本與三平台的預設 shell——glob 於傳入 Node 之前完成展開。Rust 測試 SHALL 在 Web production build 後執行；既有 Node SDK 與其他 delivery gate SHALL 保持啟用。

#### Scenario: 測試失敗使 CI 紅燈

- **WHEN** 任一平台上任一 `scripts` 測試、React workspace 測試、Web production build 或 Rust 測試失敗
- **THEN** 該 CI workflow 以失敗狀態結束，不得標記為允許失敗

#### Scenario: push 觸發完整測試

- **WHEN** push 或 pull request 觸發主 CI
- **THEN** workflow 依序完成 `scripts` 測試面、三個 React workspace 測試、Server Web production build 與 Rust workspace 測試，全數通過才回報成功

#### Scenario: scripts 測試面在釘選 Node 版本上實際執行

- **WHEN** 於本 workflow 釘選的 Node 版本與三平台預設環境執行 `scripts` 測試步驟
- **THEN** 該步驟實際載入並執行全部 `scripts` 測試檔，不因 glob 未展開而以「找不到檔案」結束

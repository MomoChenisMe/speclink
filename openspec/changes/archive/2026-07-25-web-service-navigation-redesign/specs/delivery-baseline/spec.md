## MODIFIED Requirements

### Requirement: root 單一指令全量驗證
<!-- BEFORE: root 指令依序執行 Rust workspace、`packages/ui`、`apps/desktop` 與 `crates/speclink-node` 四個測試面。 -->

repo root SHALL 提供單一指令，依序執行 `packages/ui`、`apps/desktop`、`apps/server-web`、Rust workspace 與 `crates/speclink-node` 五個測試面的測試；`apps/server-web` 測試後 SHALL 執行 production build，使後續 Rust asset integration 以本次 source 產生的 index 與 manifest 驗證。任一測試面或 Web production build 失敗時，指令 SHALL 以非零 exit code 中止。

#### Scenario: 全部通過

- **WHEN** 五個測試面與 Web production build 皆通過時於 root 執行該指令
- **THEN** 指令依序完成 Web build 與五面驗證並以 exit code 0 結束

#### Scenario: 任一面失敗即中止

- **WHEN** 任一測試面或 Web production build 存在失敗時於 root 執行該指令
- **THEN** 指令於失敗步驟以非零 exit code 中止，不繼續執行後續驗證

#### Scenario: Rust 測試使用當次 Web build

- **WHEN** `apps/server-web` source 有未建置變更並執行 root 全量驗證
- **THEN** 指令先重建 production assets，再執行 Rust workspace 的 embedded asset 與 route tests

### Requirement: CI 執行完整測試
<!-- BEFORE: 三平台主 CI 執行 Rust workspace 與 `packages/ui`、`apps/desktop` 測試，不含 Server Web。 -->

主 CI SHALL 在三個作業系統（Windows、macOS、Linux）上執行 `packages/ui`、`apps/desktop`、`apps/server-web` 測試、Server Web production build 與 `cargo test --workspace`，而非僅 build 與 smoke；測試與 Web build 步驟 SHALL NOT 設定 `continue-on-error`。Rust 測試 SHALL 在 Web production build 後執行；既有 Node SDK 與其他 delivery gate SHALL 保持啟用。

#### Scenario: 測試失敗使 CI 紅燈

- **WHEN** 任一平台上任一 React workspace 測試、Web production build 或 Rust 測試失敗
- **THEN** 該 CI workflow 以失敗狀態結束，不得標記為允許失敗

#### Scenario: push 觸發完整測試

- **WHEN** push 或 pull request 觸發主 CI
- **THEN** workflow 依序完成三個 React workspace 測試、Server Web production build與 Rust workspace 測試，全數通過才回報成功

### Requirement: 測試輸出無 React act 警告
<!-- BEFORE: 只要求 `packages/ui` 與 `apps/desktop` 測試輸出不含未等待的 React act 警告。 -->

`packages/ui`、`apps/desktop` 與 `apps/server-web` 的測試輸出 SHALL NOT 含 React `act(...)` 警告（`not wrapped in act` 字樣）；async 狀態更新 SHALL 在測試中被明確等待，而非被壓制或過濾。

#### Scenario: 測試輸出檢查零命中

- **WHEN** 執行 `packages/ui`、`apps/desktop` 與 `apps/server-web` 的完整測試套件並檢查輸出
- **THEN** 輸出不含 `not wrapped in act` 字樣，且所有測試通過

#### Scenario: 禁止以壓制方式清零

- **WHEN** 檢視三個 React workspace 的測試設定與測試檔警告處理方式
- **THEN** 不存在對 act 警告的 console 過濾或全域壓制設定，警告消除一律來自測試側的明確等待

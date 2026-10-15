## ADDED Requirements

### Requirement: 開發建置不保留除錯中間檔

workspace 根目錄的 Cargo.toml SHALL 以 `[profile.dev]` 區段設定 `debug = false`，使 dev 與 test profile 的建置不產生除錯資訊，且不在 target/debug/deps 留下 .o 中間檔——macOS 預設把除錯資訊留在 .o 且重新編譯不刪舊檔，目錄累積到數百萬個檔案後，rustc 每次啟動讀取相依目錄都要付出分鐘級成本。release profile SHALL NOT 因此改變；dev 建置的 debug_assertions 與整數溢位檢查 SHALL 維持開啟；CLI 的人眼輸出與 `--json` 輸出 SHALL 維持既有位元級輸出。測試失敗時的 panic 訊息 SHALL 仍帶檔名與行號。scripts 測試面的 delivery gate SHALL 以靜態解析守住此設定：根 Cargo.toml 缺少 `[profile.dev]` 區段，或該區段的 `debug` 不是關閉值（`false` 或 `0`）時，該測試 SHALL 失敗，失敗訊息 SHALL 指出 dev profile 必須關閉除錯資訊及其原因。

##### Example: delivery gate 對 dev profile 的判定

| 根 Cargo.toml 的 dev profile | delivery gate |
| ---------------------------- | ------------- |
| `[profile.dev]` 內有 `debug = false` | 通過 |
| `[profile.dev]` 內有 `debug = 0` | 通過 |
| `[profile.dev]` 內有 `debug = "line-tables-only"` | 失敗 |
| `[profile.dev]` 內有 `debug = true` | 失敗 |
| `[profile.dev]` 區段存在但沒有 `debug` 鍵 | 失敗 |
| 沒有 `[profile.dev]` 區段 | 失敗 |

#### Scenario: dev 建置後不留 .o 中間檔

- **WHEN** 在 macOS 以 dev 或 test profile 建置任一 workspace 成員（例如 cargo test --no-run）
- **THEN** target/debug/deps 內沒有任何 .o 檔

#### Scenario: 重新編譯不累積

- **WHEN** 改動一個 workspace crate 的原始碼後再次以 test profile 建置
- **THEN** target/debug/deps 內的 .o 檔數量仍為 0，不因重新編譯而增加

#### Scenario: 測試失敗訊息仍帶位置

- **WHEN** dev 建置的測試因 assert 失敗而 panic
- **THEN** 測試輸出的 panic 訊息帶有出錯處的檔名、行號與欄號

#### Scenario: 設定被移除時守門轉紅

- **WHEN** 根 Cargo.toml 的 `[profile.dev]` 區段被移除，或其 `debug` 被改成 `true`、`"line-tables-only"` 等開啟除錯資訊的值，然後執行 scripts 測試面
- **THEN** delivery gate 的 dev profile 測試失敗，scripts 測試步驟以非零 exit code 結束，失敗訊息指出 dev profile 必須關閉除錯資訊以避免 .o 累積

#### Scenario: 設定存在時守門通過

- **WHEN** 根 Cargo.toml 的 `[profile.dev]` 區段設定 `debug = false`，執行 scripts 測試面
- **THEN** delivery gate 的 dev profile 測試通過

#### Scenario: release 建置不受影響

- **WHEN** 以 release profile 建置 CLI 或 server
- **THEN** 建置設定與本需求生效前相同，輸出行為不變

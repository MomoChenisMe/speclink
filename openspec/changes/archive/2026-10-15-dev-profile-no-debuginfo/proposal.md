## Summary

關掉 Rust 開發設定（dev profile）的除錯資訊，並一次清空本機累積的編譯中間檔，讓本機完整測試從 40 分鐘以上回到約 10 分鐘。

## Motivation

- 目標使用者：在本 repo 透過 AI 代理跑 SDD 的開發者。apply 的任務驗證、品質關卡的修補輪、push 前的自我驗證，都要跑 cargo 建置與測試。
- 現況：本機 cargo test --workspace 要 40 分鐘以上；speclink-desktop 的 lib 測試在主 checkout 要 1000–1800 秒，在全新的 worktree 只要 5 秒。同一套 Rust 測試在 CI 只要 4–5 分鐘（macOS／Linux）。
- 根因（討論 test-speed-and-wording 實測）：macOS 的 dev profile 預設 split-debuginfo=unpacked，把除錯資訊留在每個 codegen unit 的 .o 中間檔裡，重新編譯時不刪舊檔。主 checkout 的 target/debug/deps 已累積 2,076,545 個 .o，target 目錄共 173 GB。rustc 每次啟動都把 -L dependency 目錄整個讀一遍（對編譯中的 rustc 取樣，2346 次中 2339 次落在讀目錄的系統呼叫），每編一個 crate 付一次，rustc 只用到 6–18% CPU。CI 每次從快取起建、不累積，所以沒有這個問題。

## Proposed Solution

- workspace 根目錄的 Cargo.toml 新增 `[profile.dev]` 區段，設 `debug = false`。test profile 繼承 dev，所以 cargo test 一併生效。scratchpad 小專案實測：此設定下編譯不留任何 .o。
- delivery-baseline 新增需求「開發建置不保留除錯中間檔」，並在 delivery gate 的靜態契約測試加一條守門：有人移除此設定時 scripts 測試面轉紅，避免累積在幾個月後靜默復發。
- 一次性清空主 checkout 的 target 目錄（cargo clean）。清空前先確認沒有其他 session 正在使用 cargo。
- 清空後在本機跑一次 npm run test:all，記錄耗時與 target/debug/deps 內的 .o 數量。

## Alternatives Considered

- `debug = "line-tables-only"`：實測仍留下 .o，不解決累積。
- macOS 限定改 `split-debuginfo=packed`：保留完整除錯資訊（收進 .dSYM）且不留 .o，但每支執行檔連結後要多跑一次 dsymutil。本專案目前不用逐行除錯，保留為日後需要時的退路。
- macOS 改 `split-debuginfo=off`：一樣失去除錯資訊，卻仍付產生除錯資訊的編譯成本，不如直接關掉。
- 只靠定期 cargo clean：治標，幾個月又堆回兩百萬個。
- 改用 cargo-nextest：不減少測試執行檔數，討論 slow-global-test-suite 已否決它當主解，理由仍成立。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `delivery-baseline`：新增需求「開發建置不保留除錯中間檔」——workspace 的 dev profile 關閉除錯資訊，並由 delivery gate 靜態守門。

## Impact

- Affected specs: delivery-baseline（新增一條需求）
- Affected crates/apps: 全部 workspace 成員的 dev 與 test 建置（speclink-core、speclink-cli、speclink-host、speclink-server、speclink-node、apps/desktop 等）。程式行為不變：debug_assertions 與整數溢位檢查是另外的設定，dev 建置維持開啟；release 建置不受影響。
- 相容性影響：CLI 的人眼輸出與 `--json` 輸出不變；測試失敗訊息仍帶檔名與行號；失去 lldb 逐行除錯，以及 RUST_BACKTRACE 呼叫堆疊上的行號（只剩函式名稱）。不涉及 openspec/config.yaml 或 .speclink.yaml 的設定欄位，不涉及生成的技能。
- Affected code:
  - Modified: Cargo.toml、scripts/release/delivery-gate.test.mjs
- repo 外的一次性動作：清空主 checkout 的 target 目錄（gitignored，不進 commit）。

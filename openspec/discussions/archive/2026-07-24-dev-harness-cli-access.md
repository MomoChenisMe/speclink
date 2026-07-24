---
topic: npm run dev 如何保證使用目前 checkout 的測試 CLI
slug: dev-harness-cli-access
status: promoted
promoted_to: dev-harness-cli-access
created: 2026-07-24
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: npm run dev 如何保證使用目前 checkout 的測試 CLI

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在 Remote Server＋Desktop 手動測試時，擔心本機沒有安裝 `speclink` CLI，或 PATH 指向與目前 checkout 不同版本的 CLI。採 assumptions mode，因 repo root `package.json`、`scripts/dev.mjs` 與 `crates/speclink-cli/Cargo.toml` 已足以確認現行 dev harness、process 生命週期與 CLI binary 產物。相關正典為 dev-harness、remote-connection 與 phase3-acceptance；本討論只收斂測試 CLI 的建置與呼叫入口，不實作。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-24)

**Focus**: 測試期間如何消除 CLI 未安裝或版本錯置
**Position**: dev harness 應使用目前 checkout 的 CLI binary，而非依賴 PATH 中已安裝的 `speclink`。
- CLI 是一次性 command，不加入 Server／Desktop 的常駐 child process 集合；否則其正常結束會觸發現有同殺語意。
- `cargo build -p speclink-cli` 可產生 `target/debug/speclink`，不覆寫 `~/.cargo/bin/speclink`。
- root npm wrapper 應提供固定呼叫目前 checkout CLI 的入口，使沒有全域安裝與已安裝舊版得到相同行為。
**Ruled out**: `cargo install --path crates/speclink-cli` 會覆寫本機安裝；修改 parent shell PATH 或 shell profile 具有隱性、不可攜狀態；把 CLI 當常駐 child 會破壞現有收束生命週期。
**Open**: `npm run dev` 是否必須先建置 CLI 且 build 失敗即整體 fail-fast，或由 npm CLI wrapper 首次使用時才建置。

### Round 2 — assumptions (2026-07-24)

**Focus**: CLI build 失敗時 dev harness 是否仍啟動 Server／Desktop
**Position**: `npm run dev` 必須先完成目前 checkout 的 CLI build，失敗即 fail-fast，不啟動任何常駐 process。
- 成功後 `target/debug/speclink`（Windows 為對應 `.exe`）成為本次測試的唯一 CLI binary。
- `npm run cli -- <args>` 固定轉送到該 binary，保留 stdin、stdout、stderr 與 exit code；不得解析或改寫 CLI 參數。
- 例如 PATH 即使有舊版 `speclink`，`npm run cli -- list` 仍執行目前 checkout 的 binary；若 CLI 無法編譯，`npm run dev` 非零結束且 Server／Desktop 都不啟動。
**Ruled out**: lazy build 會讓 `npm run dev` 顯示可用但第一次 CLI 操作才失敗，形成假成功環境。
**Open**: none

## Conclusion

**Decision**: `npm run dev` 在啟動 Server／Desktop 前先建置目前 checkout 的 `speclink-cli`，build 失敗即非零結束且不啟動任何常駐 process；root 另提供 `npm run cli -- <args>`，跨平台固定呼叫同一 checkout 的 `target/debug/speclink`（Windows 對應 `.exe`），透明轉送 stdin、stdout、stderr、參數與 exit code。
**Rationale**: 測試環境必須保證 Server、Desktop、CLI 來自同一 checkout；以 build gate 加固定 wrapper 可同時消除 CLI 未安裝與 PATH 舊版問題，又不覆寫使用者既有安裝。
**Rejected alternatives**: `cargo install --path` 會覆寫本機 CLI；修改 PATH／shell profile 有隱性且不可攜狀態；把 CLI 當常駐 child 會因正常結束觸發同殺；lazy build 會讓 dev harness 假成功。
**Deferred**: none
**Capture to**: proposal, design, specs/dev-harness/spec.md, tasks
**Next**: $speclink-propose --from-discussion dev-harness-cli-access

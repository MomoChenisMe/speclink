## Why

平台重構（Phase 1–4）啟動前，交付基線有三個實測為真的缺口：`crates/speclink-node` 的 npm ci 因 lock file 缺少五個未發佈 `@speclink/engine-*` 平台套件的條目而失敗，導致 Node SDK workflow（安裝步驟即用 npm ci）對任何碰到 `crates/**` 的 push 必紅；主 CI 只做 release build 加 CLI smoke，cargo test --workspace 與 npm workspace 測試都不在 CI 內；root 的 package.json 沒有任何 scripts，無法用單一指令執行全部 build 與測試。缺了這個基線，`engine-typed-core` 的 Node dispatch 遷移段與全量回歸段無法驗收，後續每一把重構刀在 CI 上都沒有回歸網。

目標使用者是 Speclink 的開發者與 AI 代理——在 apply／verify 階段執行重構刀、需要可信紅綠訊號的人。使用情境對應 implementation-refactor-roadmap 的 G0 交付 Gate：G0 完成前，禁止進入 Node 遷移與全量回歸。

## What Changes

- 修復 `crates/speclink-node` 的 npm 安裝確定性：乾淨環境下 npm ci 成功，package.json 與 package-lock.json 對五個未發佈平台套件的宣告方式一致且可重現。
- root 新增單一指令執行全部 build 與測試：涵蓋 Rust workspace、`packages/ui`、`apps/desktop` 與 `crates/speclink-node` 四個測試面。
- 主 CI 從 build＋smoke 擴為完整測試：至少執行 cargo test --workspace 與 npm workspace 測試。
- Node native 套件在五個宣告平台完成 build，可原生執行的平台跑 load smoke 與測試（沿用既有 Node SDK workflow 矩陣，恢復綠燈）。
- Desktop 測試輸出的 React act(...) warnings 清零，消除 async 更新未被測試等待的假綠風險。
- 上述五條驗收條件固化為新 capability `delivery-baseline` 的正典規格，作為後續每把重構刀的交付前提。

## Non-Goals

- 不動 `crates/speclink-core`、`crates/speclink-cli`、`crates/speclink-node/src` 的產品程式碼，也不動 `packages/ui`、`apps/desktop` 的元件原始碼——本變更只碰 packaging 檔、CI 設定、root scripts 與測試檔；驗收時以 git diff --stat 檢查改動面。**單點豁免**（apply 期間經使用者裁定）：crates/speclink-node/src/store_bridge.rs 的 ChangeMeta 初始化補兩行欄位映射——此為 apply 揭露的既有編譯斷裂（main 上 cargo test --workspace 已紅），不修則 G0 驗收不可能成立，詳見 design 決策五。
- 不改 Engine 語意與 CLI 行為：人眼輸出與 --json 輸出零變動，parity／color／twin 回歸對照不受影響（相容性影響：無）。
- 不發佈任何 npm 套件、不改 release 發佈行為（現有 release workflow 只發佈 CLI binary，無 npm 發佈步驟）。
- CI 全量測試若揭露平台性既有紅燈，修復逾出 packaging／設定／測試檔範圍者另開 change 處理，不在本變更內擴 scope。
- act(...) 清零過程若揭露元件本身的 async 真 bug，修元件另開小刀，本變更僅處理測試檔側的等待缺失。

## Capabilities

### New Capabilities

- `delivery-baseline`: 儲存庫的交付基線要求——Node 套件安裝確定性、root 單一指令全量驗證、CI 完整測試、Node native 全平台 build 與可執行平台 load smoke、測試輸出無 React act(...) warnings。此規格是 G0 交付 Gate 的正典化，後續 Phase 的變更以它為交付前提。

### Modified Capabilities

（無）——既有規格的需求不變，僅其驗證管道被補強。

## Impact

- Affected specs: 新增 `delivery-baseline`（openspec/specs/delivery-baseline/spec.md 於 archive 時建立）。
- Affected code:
  - Modified: crates/speclink-node/package.json、crates/speclink-node/package-lock.json、crates/speclink-node/src/store_bridge.rs（單點豁免，兩行欄位映射）、crates/speclink-node/__test__/helpers.ts（fixture 路徑 realpath 正規化）、crates/speclink-cli/tests/discuss_promote_snapshot.rs（macOS tempdir symlink 正規化）、crates/speclink-core/src/config.rs（僅 #[cfg(test)] 測試模組內的平台條件斷言，詳見 design 決策五）、crates/speclink-core/tests/golden 的兩個 snapshot 檔（main 既有失同步，乾淨樹再生、純空白行）、crates/speclink-fs/tests/store_fs.rs（readdir 順序正規化）、.github/workflows/ci.yml、.github/workflows/node-sdk.yml（可測平台補一步 release CLI build）、package.json、apps/desktop 既有測試檔（act warnings 清零，實際檔案於執行時依測試輸出定位）。
  - New: 無（root 指令加在既有 package.json 的 scripts，不新增檔案）。
  - Removed: 無。
- 影響的 crate：`speclink-core` 與 `speclink-cli` 原始碼零改動（其測試開始在 CI 執行）；`crates/speclink-node` 僅 packaging 層。
- 不涉及 CLI 子指令、設定欄位（.speclink.yaml／openspec/config.yaml）與技能注入區塊。

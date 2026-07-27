# drift-computation Specification

## Purpose

TBD - created by archiving change 'drift-client-server-split'. Update Purpose after archive.

## Requirements

### Requirement: drift 運算拆分為規格面與工作區面純函式

drift 的五維度運算 SHALL 拆分為兩段無副作用的純函式：規格面運算只消費 Store 的規格事實（delta、正典規格、created metadata）產出 Specs 維度與規格假設；工作區面運算只消費明確傳入的 WorkspaceFacts 結構產出 Time、Structure、Tasks、Environment 四維度。兩段運算 SHALL NOT 呼叫 git、讀取 process 環境或執行任何 I/O 副作用；本機 git 與 worktree 事實的蒐集 SHALL 由 Host 側蒐集器承擔。

#### Scenario: 規格面運算不觸本機事實

- **WHEN** 以測試 Store 與固定 WorkspaceFacts 分別呼叫兩段運算
- **THEN** 相同輸入重複呼叫得到逐欄相同的結果；無 git 程序被執行

---
### Requirement: WorkspaceFacts 缺席時四維度標 unavailable

工作區面運算 SHALL 接受 WorkspaceFacts 缺席（無 code checkout）：此時 Time、Structure、Tasks、Environment 四維度 SHALL 標為 unavailable，SHALL NOT 視為 clean、SHALL NOT 計為零分或任何分數。WorkspaceFacts 存在但 git 不可用時，SHALL 沿用現行的 git-unavailable fallback 語意與輸出字串（與 unavailable 為不同狀態）。

#### Scenario: 缺席不是 clean

- **WHEN** 以缺席的 WorkspaceFacts 執行工作區面運算並合併報告
- **THEN** 四維度標 unavailable 且不帶分數；合併報告的 coverage 為 spec-only

#### Scenario: git 不可用沿現行 fallback

- **WHEN** 以「有 checkout 但 git 不可用」的 WorkspaceFacts 執行運算
- **THEN** 各維度輸出與現行 git-unavailable fallback 字串與分數逐位元一致

---
### Requirement: 單一 merger 裁決合併、coverage 與 stale

合併 SHALL 由唯一的 merger 函式實作：full coverage 時合併結果與現行 DriftReport 逐欄一致；工作區面缺席時標 coverage 為 spec-only；傳入的 basis digests 與合併時規格狀態不符時 SHALL 標 stale 並列出不符項，SHALL NOT 靜默輸出混用基準的報告。coverage 與 stale 標示 SHALL 為僅於非常態情境出現的選填欄位。CLI、Node 與桌面 SHALL NOT 各自實作 scoring 或合併規則。

#### Scenario: basis 改變即標 stale

- **WHEN** 以某 bundle 的 basis 運算後，於合併前修改該 change 的 tasks.md 再執行合併
- **THEN** 合併報告帶 stale 標示並列出 tasks basis 不符；未變動的 basis 項不列入

---
### Requirement: DriftBundle 固定漂移檢查的基準

Host SHALL 提供 DriftBundle 產生：內容含 project 與 repo binding、change 名、spec 與 tasks 與 policy 的 basis digests、created metadata、design 與 tasks 內容、task evidence 摘要與產生時間（序列化欄位 camelCase）。同一 workspace 狀態下重複產生 SHALL 得到相同的 basis digests。drift 為診斷結果，SHALL NOT 寫回正典或任何規格文件。

#### Scenario: bundle 基準可重現且不寫檔

- **WHEN** 對同一 change 連續產生兩份 DriftBundle 並執行完整 drift 流程
- **THEN** 兩份 bundle 的 basis digests 逐項相同；流程結束後 workspace 內無任何檔案被寫入或修改

---
### Requirement: 本地 drift 路徑輸出凍結

本地 cmd_drift 改為「蒐集 WorkspaceFacts → 兩段運算 → merger」三段串接後，speclink drift 的人眼輸出、--json 輸出與 exit code SHALL 與拆分前逐位元一致，含 git 可用、git 不可用、無 design 與 broken anchors 等既有情境。

#### Scenario: 重構前後輸出逐位元一致

- **WHEN** 對同一樣本 workspace（涵蓋 git 可用、git 不可用、無 design、broken anchors 情境）於拆分前後執行 speclink drift 與 speclink drift --json
- **THEN** stdout、stderr 與 exit code 逐位元一致；`crates/speclink-cli/tests/` 的整合測試（含 `--no-color` 人眼輸出斷言與 fs／remote 對照）與 `crates/speclink-core/tests/render_golden.rs` 全綠


<!-- @trace
source: stale-verification-vehicles
updated: 2026-07-27
code:
  - docs/implementation-refactor-roadmap.zh-TW.md
-->
## Why

drift 目前是單體運算：核心的 drift 模組在一個 analyze 函式內交織五個維度——Specs 維度讀 Store 的 delta 與正典規格，Time／Structure／Tasks／Environment 四維度直接呼叫本機 git（ls-files、log、grep）與 worktree。平台架構藍圖 §6.5 明定遠端模式必須拆分：Specs 在 Server 以一致 snapshot 與 revisions 運算，其餘四維度只能在有 code checkout 的 client 運算——「不能把現有依賴 Git/worktree 的五個維度整體搬到 server」；重構路線圖 §3.3 的責任表同樣把 spec drift 歸 Server/Engine、code/git drift 歸 RD client。若不先拆，Phase 2 的 Server 要嘛讀不到 RD 本機事實而砍掉四個維度、要嘛 shell-out 使用者的 git（藍圖 §16 明列非目標）。藍圖 §6.5 並要求共用 Rust merger：「CLI、Node SDK 與 Desktop 不會各自重寫 scoring 與合併規則」，以及三條硬規則——無 checkout 時四維度標 unavailable 不得視為 clean 或零分、合併報告帶 basis 且 revision 改變標 stale、drift 是診斷不寫回正典。

目標使用者：執行 drift 技能的開發者與 AI 代理（resume 前檢查漂移的 workflow 階段）——本地行為與輸出完全不變；以及順位 7 與 Phase 2 的實作者——他們以本刀的 DriftBundle 與三段運算作遠端 drift 的地基。

## What Changes

- **drift 運算拆成三段純函式**（核心 drift 模組內重構）：compute_spec_drift（只吃 Store 規格面事實：delta、正典、created metadata）產出 SpecDriftReport；compute_workspace_drift（只吃 WorkspaceFacts：本機 commit window、tracked 文件、worktree 符號、HEAD 與 dirty 狀態、task evidence）產出四個 client 維度；merge_drift_reports 合併為現行 DriftReport 形狀——scoring 與合併規則單一實作。
- **WorkspaceFacts 蒐集器落 speclink-host**：drift 的 git 讀取從核心運算中抽離為 host 側蒐集器（延續 host 刀「Engine 規格面不讀本機事實」的邊界方向）；核心 drift 運算成為無 git 副作用的純函式。
- **DriftBundle 型別與產生落 speclink-host**：固定 project/repo binding、change 與 spec 與 config 的 basis digest（沿 verify-evidence 的 digest 機制）、created metadata、design、tasks 與 task evidence——遠端流程 prepareDrift 的載體，本刀只產生與消費於本地，不做傳輸。
- **無 checkout 與部分報告語意**：WorkspaceFacts 缺席時四維度 SHALL 標 unavailable（不視為 clean 或零分）；合併報告型別支援 coverage 標示（full 或 spec-only）與 workspace_required 拒絕；本刀不新增 CLI 旗標（遠端 CLI 的 --spec-only 屬 Phase 2 接線）。
- **stale 標示**：合併報告攜 basis digests；merger 偵測 bundle basis 與合併時規格狀態不符時標 stale。
- **本地路徑輸出凍結**：cmd_drift 改為三段串接（蒐集 → 兩段運算 → 合併），人眼與 --json 輸出、exit code 與現行逐位元一致；既有「git unavailable」的本地 fallback 語意與輸出原樣保留。

## Non-Goals

- 不做 prepareDrift 的 HTTP 端點、遠端傳輸與 remote CLI 的 drift 接線（順位 7 protocol-client-context 與 Phase 2 reference-server）。
- 不做 Context Materializer 與 Context Projection（順位 7）。
- 不改 drift 的人眼與 --json 輸出、不新增 CLI 旗標、不改五維度的 scoring 規則與門檻。
- 不把 drift 結果寫回正典或任何檔案（維持診斷性質；handoff/evidence 另走明確 command 屬後續）。
- 不動 Node dispatch 動詞覆蓋與桌面 UI（drift 無桌面呈現）。

## Capabilities

### New Capabilities

- `drift-computation`: drift 運算的拆分契約——spec 面與 workspace 面的純函式邊界、WorkspaceFacts 與 DriftBundle 的形狀、共用 merger 的單一實作、unavailable 與 coverage 與 stale 語意、本地三段串接的輸出凍結。

### Modified Capabilities

（無）——drift 動詞的外部行為與輸出不變；命令層覆蓋表與錯誤分類不受影響。

## Impact

- 影響的 crate：`speclink-core`（drift 模組拆為三段純函式、WorkspaceFacts 消費端）、`speclink-host`（WorkspaceFacts 蒐集器、DriftBundle 產生）、`speclink-cli`（cmd_drift 串接改動，輸出不變）；`speclink-node` 經 runtime 自動沿用（dispatch 不含 drift，無變更）。
- 相容性影響：drift 的人眼與 --json 輸出、exit code 逐位元不變；parity／color／twin 對照全綠；「git unavailable」的既有 fallback 輸出原樣保留。新型別（SpecDriftReport、WorkspaceFacts、DriftBundle、coverage 與 stale 標示）只在程式介面層存在，不進現有輸出。
- Affected specs: `drift-computation`（新增）。
- Affected code:
  - New: crates/speclink-host/src/drift.rs
  - Modified: crates/speclink-core/src/drift.rs、crates/speclink-core/src/command/mod.rs、crates/speclink-core/src/util.rs、crates/speclink-cli/src/commands.rs、crates/speclink-host/src/lib.rs
  - Removed: 無

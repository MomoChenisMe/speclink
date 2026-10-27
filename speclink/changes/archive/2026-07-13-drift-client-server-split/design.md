## Context

crates/speclink-core/src/drift.rs 約 536 行，單一 analyze(ws, store, change) 交織五維度：Specs 維度走 Store 讀 delta 與正典（spec_assumptions 函式已獨立）；Time 吃 git log 的 commit window；Structure 吃 design anchors 對 git ls-files 的 tracked 文件與符號（git grep 語意）；Tasks 吃 tasks 與 commit/worktree 狀態；Environment 吃 git log、HEAD、dirty 狀態與 touched/evidence。git 不可用時各維度有既定 fallback 字串與分數（如「fresh (Nd), git unavailable」），屬輸出凍結面。host 刀已確立「Engine 規格面不讀本機事實」邊界與 ExecutionContext；verify-evidence 刀提供 basis digest 機制（spec／tasks／policy）與 v2 evidence。藍圖 §6.5 是本刀的正典：五維度拆分表、prepareDrift 序列、DriftBundle 內容、unavailable／coverage／stale 三條硬規則、共用 merger。

## Goals / Non-Goals

**Goals:**

- drift 運算拆成三段純函式：compute_spec_drift（Store 規格面）、compute_workspace_drift（WorkspaceFacts）、merge_drift_reports（單一 scoring 與合併實作）。
- git／worktree 事實蒐集抽離為 host 側 WorkspaceFacts 蒐集器；核心運算零 git 副作用。
- DriftBundle（host 產生）固定 binding、basis digests、created metadata、design、tasks、task evidence。
- unavailable（不視為 clean）、coverage（full／spec-only）、workspace_required、stale 的型別語意落地。
- 本地 cmd_drift 三段串接後輸出逐位元凍結。

**Non-Goals:**

- 不做 HTTP prepareDrift、遠端傳輸、remote CLI drift 接線與 --spec-only 旗標（順位 7／Phase 2）。
- 不做 Context Materializer；不改 scoring 規則與門檻；不改任何輸出；不做 drift 結果的持久化。

## Decisions

### 決策一：三段純函式留在 core 的 drift 模組，蒐集器落 host

compute_spec_drift 與 compute_workspace_drift 與 merge_drift_reports 是領域運算（scoring 規則屬流程語意），留在 speclink-core；兩者的輸入改為明確資料（Store 快照事實與 WorkspaceFacts 結構），不再接觸 git。WorkspaceFacts 蒐集器（跑 git ls-files／log／grep、讀 HEAD 與 dirty、讀 evidence）落 crates/speclink-host/src/drift.rs——與 git identity 先例一致：本機事實的取得屬 host（client）職責。util 中僅供 drift 使用的 git 輔助函式隨蒐集器遷移。替代方案：運算也搬 host——scoring 是「同一份 Rust Engine」的流程語意（藍圖 §6.5 要求 CLI／Node／Desktop 共用），搬到 host 會讓未來 Server 的 Specs 維度運算依賴 host crate，分層倒置，被拒。

### 決策二：WorkspaceFacts 為封閉輸入結構，缺席即 unavailable

WorkspaceFacts 含：commit window（since log 有無與筆數）、tracked 文件清單與內容（drift 用的 md／txt）、worktree 符號查詢結果、HEAD 與 dirty 狀態、task evidence 摘要；每個欄位保留「git 不可用」的現行三值語意（有值／空值／不可用），使既有 fallback 輸出逐位元重現。compute_workspace_drift 接受 Option 的 WorkspaceFacts：None（無 checkout）時四維度回 unavailable 標示——不是 clean、不是零分，型別上與「git 不可用但有 checkout」的既行 fallback 區分。替代方案：以空 facts 代替缺席——四維度會誤算成「無 commit、無檔案」的分數，正是藍圖禁止的「視為 clean 或零分」，被拒。

### 決策三：merger 是唯一合併與 coverage／stale 裁決點

merge_drift_reports(spec_report, workspace_report_or_unavailable, basis) 產出 CombinedDriftReport：本地 full coverage 時內容與現行 DriftReport 逐欄一致（人眼與 --json 渲染不變）；workspace 側缺席時標 coverage: spec-only 並保留四維度的 unavailable 條目；basis digest 與合併時規格狀態不符時標 stale。coverage 與 stale 欄位只在非 full 或 stale 情境存在（選填），本地現行路徑不出現、輸出凍結不破。替代方案：CLI 層自行拼裝兩份報告——Node 與 Desktop 未來各自重寫合併，正是 §6.5 點名要避免的分叉，被拒。

### 決策四：DriftBundle 由 host 產生，內容對齊藍圖 §6.5

produce_drift_bundle(change)：project／repo binding（ExecutionContext）、change 名、spec／tasks／policy basis digests（沿 verify-evidence 的 digest 機制，config 即 policy digest）、created metadata、design 與 tasks 內容、task evidence 摘要、產生時間。本刀 bundle 只在本地產生與消費（compute_workspace_drift 的 basis 來源與 stale 比對輸入）；序列化形狀（serde、camelCase）先定，傳輸屬 Phase 2。替代方案：bundle 延後到 Phase 2 一併定——workspace 運算的 basis 輸入會先以臨時參數散裝，Phase 2 再收攏成 bundle 時破壞已凍結的函式邊界，被拒。

### 決策五：本地 cmd_drift 三段串接、輸出逐位元凍結

CLI 的 drift 路徑改為：host 蒐集 WorkspaceFacts → compute_spec_drift ＋ compute_workspace_drift → merge → 現行渲染。以樣本 workspace（含 git 可用、git 不可用、無 design、broken anchors 等既有情境）在重構前後對照人眼與 --json 逐位元一致；parity／color／twin 全綠。runtime 的 drift 查詢動詞經同一路徑（命令層簽名已攜 ExecutionContext）。

## Implementation Contract

- **行為**：speclink drift 的一切現行輸出（人眼、--json、exit code、git-unavailable fallback 字串）逐位元不變。程式介面新增：core 的 compute_spec_drift／compute_workspace_drift／merge_drift_reports 三個純函式（無 git、無 env 副作用）；host 的 WorkspaceFacts 蒐集器與 produce_drift_bundle。無 checkout（facts 缺席）時合併報告四維度為 unavailable 且 coverage 為 spec-only；basis 不符時報告帶 stale 標示。
- **介面／資料形狀**：WorkspaceFacts 封閉結構（欄位保留有值／空值／不可用三值語意）；DriftBundle { project, repo, change, basisDigests（spec／tasks／policy）, createdMeta, design, tasks, evidenceSummary, producedAt }（serde camelCase）；CombinedDriftReport 於 full coverage 時與現行 DriftReport 逐欄一致，coverage 與 stale 為僅非常態時出現的選填欄位。
- **失敗模式**：facts 缺席≠clean（四維度 unavailable）；basis 不符→stale 標示不靜默；蒐集器的 git 失敗沿現行「git unavailable」語意進 facts，不改錯誤分類。
- **驗收**：cargo test -p speclink-core 與 -p speclink-host 全綠（三段函式單元測試、unavailable／coverage／stale 情境、bundle 內容）；樣本 workspace 重構前後 drift 輸出逐位元對照；parity／color／twin 全綠；npm run test:all 全綠。
- **範圍邊界**：in scope——drift.rs 拆分、WorkspaceFacts 與蒐集器、DriftBundle、merger 語意、cmd_drift 串接；out of scope——HTTP 端點、remote CLI 接線、--spec-only 旗標、Context Materializer、scoring 規則調整、結果持久化。

## Risks / Trade-offs

- [拆分時 fallback 字串與分數語意遺漏，輸出漂移] → 既有 drift 測試全數保留；樣本 workspace 涵蓋 git 可用／不可用／無 design／broken anchors 情境做前後逐位元對照。
- [WorkspaceFacts 欄位設計過寬或過窄，Phase 2 遠端接線時不敷使用] → 欄位嚴格對齊現行 analyze 實際消費的輸入（從現碼逆推），不臆測遠端需求；不足時 Phase 2 以新增選填欄位演進。
- [三值語意（有值／空值／不可用）在型別上混淆] → 以 enum 明確表達不可用，禁止以空集合暗示；測試逐欄斷言三值行為。
- [util 的 git 輔助函式遷移波及其他呼叫者] → 遷移前 git grep 盤點呼叫者；僅 drift 專用者隨遷，共用者（如 archive 的 git 呼叫）不動。
- [跨平台 git 輸出差異（換行、路徑分隔）] → 蒐集器原樣搬移現行實作不改邏輯；既有跨平台護欄與 .gitattributes 的 LF 強制不動。

## Migration Plan

純內部重構加新型別：無資料、設定或輸出遷移；回滾即還原 commit。後續採用：順位 7 的 Context Materializer 與 remote CLI 以 DriftBundle 與 spec-only coverage 接線；Phase 2 的 Server 消費 compute_spec_drift 與 prepareDrift 流程。

## Open Questions

（無）

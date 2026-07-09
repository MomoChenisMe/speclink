---
topic: desktop 動詞的結果呈現與範圍（analyze 富面板、結果進抽屜、撤 promote）
slug: desktop-sdd-verb-scope
status: promoted
promoted_to: desktop-verb-drawer-surface
created: 2026-07-09
---

# Discussion: desktop 動詞的結果呈現與範圍（analyze 富面板、結果進抽屜、撤 promote）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

桌面看板與 Spectra 的差異盤點（第三批 UI）中屬「SDD 動詞」的三項：(3) 分析/驗證按鈕是否沒實作、(4) 為何缺 Spectra 的四維度分析、(5) desktop 的「轉為變更」是否該撤（因整個 discuss 流程都依賴 LLM）。

模式：assumptions（掃到 RichDetailDrawer.tsx、store.ts、App.tsx、analyzer.rs、adapter.ts）。

程式碼盤點：
- 分析/驗證按鈕已接 `onRunVerb`（RichDetailDrawer:297-302）→ desktop 呼叫同名 Tauri command → 跑**確定性引擎**（非 LLM）。`Verb = validate|analyze|archive`（adapter.ts:69）。
- `crates/speclink-core/src/analyzer.rs` 就是四維度（Coverage/Consistency/Ambiguity/Gaps）確定性分析，回 `AnalyzeReport { findings }`；完整 findings 已回到前端，但 `store.ts:120 formatVerbResult` 只數 `.length` 成一行摘要就丟掉結構。
- verbResult 僅在 `App.tsx:254` 視窗頂列渲染一行——使用者實測按鈕感覺「沒反應」＝結果跑到視窗最上緣（Image #11 佐證）。
- promote：`discuss promote` 是確定性 scaffold＋prefill，但產出的 change 是 stub，需 LLM `/speclink-propose` 補完；desktop 已刻意不提供 conclude/add-round（DiscussionDrawer 註解「GUI 不提供 conclude 等寫入」）。

介面深度：analyze 富面板無新 IPC（結果已在前端）；撤 promote 為移除。無新 seam。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-09)

**Focus**: 分析/驗證是否實作、結果為何感覺沒反應、該怎麼呈現
**Position**: 已實作且確定性，缺的是「結果的家」——
- 兩按鈕都接 onRunVerb 跑確定性 Tauri 動詞（RichDetailDrawer:297-302），非 LLM；analyze 就是 analyzer.rs 的四維度分析。
- 富面板要的 `AnalyzeReport.findings` 已回到前端，store.ts:120 只 stringify 成「N findings」丟掉結構——無新 IPC/LLM 即可渲染。
- 「沒反應」根因＝verbResult 僅在 App.tsx:254 視窗頂列一行渲染，離抽屜按鈕很遠。
- 定案：change 動詞結果移進抽屜、人性化呈現——validate 綠✓/紅✗＋首則錯誤；analyze 四維度＋逐條 finding（比照 Spectra）。頂列 verbResult 留給看板全域操作（刪除/封存/拖排失敗）。
**Ruled out**: 「分析/驗證沒實作」的前提（實測有跑，只是結果被壓成一行且放錯位置）；把 analyze 當 LLM 功能（它是確定性引擎）
**Open**: 撤不撤 promote（議題 5）

### Round 2 — assumptions (2026-07-09)

**Focus**: desktop 該不該提供「轉為變更」(promote)
**Position**: 撤掉——
- promote 操作本身確定性、可靠（scaffold＋prefill Why），但**產出一個只有 LLM 能補完的 stub**（change 仍需 /speclink-propose 生 delta specs/design/tasks）。
- desktop 已刻意不提供 conclude/add-round 等 LLM 依賴寫入（DiscussionDrawer 註解）；promote 是唯一破例（其輸出需 LLM 收尾）。
- 撤掉後 desktop 定位一致＝檢視器＋確定性且自足的動詞（validate/analyze/archive）。衍生變更分頁、已轉出分組維持唯讀，只拿掉 promote 動作鈕（DiscussionCard 轉為變更鈕、DiscussionDrawer promote pane 的鈕）。
**Ruled out**: 保留 promote（「操作可靠」不足以蓋過「輸出是 LLM 才能補完的 stub、與既有 no-LLM-write 線不一致」）
**Open**: none

## Conclusion

**Decision**: (3+4) change 動詞（validate/analyze）結果從視窗頂列（App.tsx:254）移進詳情抽屜、人性化呈現——validate＝按鈕旁綠✓valid／紅✗＋首則錯誤；analyze＝四維度（Coverage/Consistency/Ambiguity/Gaps）富面板＋逐條 finding（比照 Spectra Image #8）。資料 `AnalyzeReport.findings` 已回前端，停止 stringify、改保留結構渲染，無新 IPC、無 LLM。頂列 verbResult 保留給看板全域操作（刪除/封存/拖排失敗）。(5) 撤掉 desktop 的「轉為變更」(promote) 動作鈕（DiscussionCard、DiscussionDrawer promote pane）；衍生變更分頁與已轉出分組維持唯讀。
**Rationale**: 分析/驗證本是確定性引擎、結果早已到前端，只是被壓成一行放在遠處視窗頂列——搬進抽屜富呈現是純 UI、高價值。promote 雖操作可靠卻產出 LLM 才能補完的 stub，與 desktop 既有「不提供 conclude/add-round 等 LLM 寫入」不一致，撤掉使定位一致（檢視器＋自足確定性動詞）。
**Rejected alternatives**: 「分析/驗證沒實作」前提（實測有跑）；把 analyze 當 LLM 功能（是確定性 analyzer.rs）；保留 promote（輸出需 LLM 收尾，破壞 no-LLM-write 一致性）；把 verbResult 留在視窗頂列（離動作太遠、感覺沒反應）
**Deferred**: validate 結果在抽屜的確切版式、analyze 面板與現有四分頁的位置關係留 propose/design
**Capture to**: proposal（新變更：analyze 抽屜富面板＋validate 結果進抽屜＋撤 promote）
**Next**: /speclink-propose --from-discussion desktop-sdd-verb-scope

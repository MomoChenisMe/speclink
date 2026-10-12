# Verify — desktop-spec-manual-pages

## Round 1

**Phase**: discovery
**Patch**: sha256:ddf8fc3e4d4b20582f2856499303c8abdb3ef193656216912c1c3e62418df36a
**Scope**: apps/desktop/core/src/manual.rs, apps/desktop/src-tauri/src/lib.rs, apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/fsActions.test.ts, apps/desktop/src/adapter/fsActions.ts, packages/ui/src/__tests__/deltaSpecView.test.tsx, packages/ui/src/__tests__/manualPage.test.tsx, packages/ui/src/__tests__/specDetailPage.test.tsx, packages/ui/src/__tests__/specSections.test.ts, packages/ui/src/__tests__/specSummary.test.ts, packages/ui/src/__tests__/ticketView.test.tsx, packages/ui/src/components/DetailHeader.tsx, packages/ui/src/components/ManualPage.tsx, packages/ui/src/components/ManualTree.tsx, packages/ui/src/components/Markdown.tsx, packages/ui/src/components/RequirementBlock.tsx, packages/ui/src/components/SpecDetailPage.tsx, packages/ui/src/components/TicketView.tsx, packages/ui/src/i18n.tsx, packages/ui/src/index.ts, packages/ui/src/manualDoc.ts, packages/ui/src/specSections.ts, packages/ui/src/specSummary.ts

- [WARNING] packages/ui/src/__tests__/specDetailPage.test.tsx — Correctness: Scenario「需求段之外的內容照樣呈現」這次新增了「「附錄」列入本頁目錄」，但 :531 的測試只驗證附錄有內容、排序正確，沒驗證目錄。程式其實做得到：`SpecDetailPage.tsx:322` 把附錄（含 `## 附錄`）交給 Markdown，會被 `.markdown h2` 選進目錄。建議在該題補上 `await waitFor(() => expect(tocAnchors()).toContain("附錄"))`。
- [WARNING] packages/ui/src/__tests__/specDetailPage.test.tsx — Correctness: 「溯源」閱讀卡的標題區有兩處沒測到。一是 desktop-app「手冊頁的開啟選單」THEN 要求「規格詳情頁的「溯源」文件無選單」，這是新行為（舊版兩個分頁共用同一個「開啟 ▾」）；二是「溯源分頁依封存日期排序」要求「閱讀卡標題「溯源」帶籤 3」，目前只驗了節點徽章。建議在溯源排序題或「「開啟 ▾」收到…」題切到溯源，斷言 `[data-doc-header]` 內沒有「開啟 ▾」、且有數字籤 3（對應 `SpecDetailPage.tsx:334-337`）。
- [WARNING] packages/ui/src/__tests__/manualPage.test.tsx — Correctness: desktop-manual-page Scenario「帶錨定的出處只列 capability 一次」改寫後，要求出處行與頁尾出處列各恰有一顆可點的 `desktop-app`，且數字磚「出處規格」為 2。既有的 :395 測試只查頁尾 `[data-manual-sources]`，出處行和磚都沒斷言。建議在該題加上 `docProvenance()` 內 `getAllByText("desktop-app")` 長度為 1、`policy-config` 不可點，以及 `tiles()[3] === "2出處規格"`（對應 `ManualPage.tsx:254`）。
- [WARNING] packages/ui/src/__tests__/manualPage.test.tsx — Correctness: Scenario「過期提示框在規格讀不到時退回」涵蓋兩種情況：讀取失敗，或規格「沒有任何 `@trace`」。:815 的測試名稱寫了兩者，實際只測 reject。`ManualPage.tsx:192` 中 `md` 為 null、或解析後 `latest` 為 null 的分支都沒被測到。建議同一題再跑一次 `loadSpec` 回傳無 `@trace` 的規格（或回傳 `null`），並斷言出現「出處 x 在本頁產生後更新過」。
- [SUGGESTION] packages/ui/src/components/RequirementBlock.tsx — Coherence: 使用者在手動驗收時要求的 hover 回饋，讓 delta 收合鈕（:125）加上了 `-ml-2`、內距、`cursor-pointer` 與淡底；`TicketView.tsx:203` 的輪卡頭也一起改。但 design D3 寫的是「delta 畫法不變」，proposal 的 Impact 也沒列 `TicketView.tsx`、`ticketView.test.tsx`。建議在 D3 補一句「兩種畫法的收合列皆有手指游標與 hover 淡底（驗收期使用者要求）」，並把這兩個檔加進 Impact 的 Modified，讓規劃文件跟實際改動一致。
- [SUGGESTION] packages/ui/src/components/ManualPage.tsx — Coherence: desktop-app「手冊頁的開啟選單」THEN 寫「remote 無 checkout 的手冊頁…只有「複製內容」」，但 Non-Goals 寫「不做 remote 模式的手冊」，而且 :227 遇到 `reason === "remote"` 只顯示空狀態，根本沒有閱讀卡和選單，所以這句規格實際走不到。建議把該子句改成「remote 模式顯示既有空狀態、無選單」，或直接刪掉，免得封存後的正式規格寫著做不到的行為。

## Round 2

**Phase**: validation
**Patch**: sha256:2ece81715ad85d3d742d1b14b7ce59673620fb81baa15ecec47532b58cc7e755
**Scope**: apps/desktop/src/App.tsx, apps/desktop/src/__tests__/App.test.tsx, packages/ui/src/__tests__/manualPage.test.tsx, packages/ui/src/__tests__/specDetailPage.test.tsx, packages/ui/src/__tests__/specSummary.test.ts, packages/ui/src/components/ManualPage.tsx, packages/ui/src/components/ManualTree.tsx, packages/ui/src/components/RequirementBlock.tsx, packages/ui/src/components/SpecDetailPage.tsx, packages/ui/src/components/ui/tabs.tsx, packages/ui/src/i18n.tsx, packages/ui/src/index.ts, packages/ui/src/specSummary.ts

## Round 3

**Phase**: validation
**Patch**: sha256:7f0715e1eea90cb09799cc982959e1c7c67a5cf28f12a54b5d9f90d9f0be0e3c
**Scope**: packages/ui/src/__tests__/specDetailPage.test.tsx, packages/ui/src/components/ManualToc.tsx

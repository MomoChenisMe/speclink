## Summary

桌面把 validate／analyze 的結果從視窗頂列搬進詳情抽屜並人性化呈現（analyze 為四維度富面板），並撤除 desktop 端的「轉為變更」（promote）動作。

## Motivation

- validate／analyze 為確定性引擎、結果早已送達前端，但目前只在視窗頂列顯示一行摘要（離抽屜按鈕很遠、使用者實測感覺沒反應），且 analyze 的四維度發現項被壓成一個計數丟棄。
- promote 操作雖確定性，卻產出只有 LLM 能補完的 stub，與 desktop 既有「不提供 conclude／add-round 等 LLM 依賴寫入」不一致；desktop 定位應為檢視器＋自足的確定性動詞（validate／analyze／archive）。

## Proposed Solution

- validate／analyze 結果改於該 change 的詳情抽屜內呈現：validate 於動作列近處以通過／失敗呈現（失敗附首則錯誤），analyze 以 Coverage／Consistency／Ambiguity／Gaps 四維度面板＋逐條發現項呈現（沿用引擎已回傳的 AnalyzeReport，不新增 IPC）。視窗頂列狀態列保留給看板全域操作（刪除／封存／拖排失敗）之結果。
- 撤除 desktop 的 promote 動作：移除討論卡的「轉為變更」鈕與討論抽屜衍生變更分頁的轉出鈕（轉為變更／再轉出）；衍生變更分頁與已轉出分組維持唯讀（列子變更與跳轉）。promote 併入「GUI 不提供的寫入動詞」清單。

## Non-Goals

見 design.md 的 Goals/Non-Goals。

## Alternatives Considered

見 design.md 的 Decisions（各決策附替代與否決理由）。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`：三需求——「桌面 app 提供動詞操作面」（validate／analyze 結果進抽屜富呈現）、「討論抽屜檢視與轉出變更」（撤 promote 動作、衍生變更維持唯讀）、「討論於看板第 0 欄兩級呈現」（concluded 卡移除轉為變更動詞）。

## Impact

- Affected specs: desktop-app（modified）
- Affected code:
  - Modified:
    - packages/ui/src/components/RichDetailDrawer.tsx
    - packages/ui/src/components/DiscussionColumn.tsx
    - packages/ui/src/components/DiscussionDrawer.tsx
    - apps/desktop/src/store.ts
    - apps/desktop/src/App.tsx
    - packages/ui/src/i18n.tsx
    - packages/ui/src/__tests__/richDrawer.test.tsx
    - packages/ui/src/__tests__/discussionColumn.test.tsx
    - packages/ui/src/__tests__/discussionDrawer.test.tsx
    - apps/desktop/src/__tests__/store.test.ts
  - New:
    - packages/ui/src/components/AnalyzePanel.tsx
    - packages/ui/src/__tests__/analyzePanel.test.tsx
  - Removed: (none)

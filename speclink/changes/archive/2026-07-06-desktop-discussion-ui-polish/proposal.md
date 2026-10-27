## Summary

依討論「討論看板 UI 與詞彙打磨」結論，把討論相關 GUI 的自造詞全面白話化（促轉→轉為變更等，詞彙表已落 openspec/LANGUAGE.md），並做三項資訊架構調整：已轉出討論細列改衍生樹、討論抽屜加生命週期階梯與預設分頁改結論、確認框文案使用者語言化。

## Motivation

desktop-discussion-board 落地後使用者回饋：討論 UI 不易理解與操作、中文用詞不直觀。目標使用者（透過桌面 GUI 追蹤 SDD 流程的開發者與 PO/PM）看到「促轉」「已促轉」無法從字面推出行為；已促轉細列把 topic 換成 slug 顯示（「變臉」）、pill 標籤堆看不出父子關係；確認框直接暴露 from_discussion、kebab-case 等工程詞。對應 workflow 階段：discuss 的看板呈現與 promote 操作。

## Proposed Solution

- **A 詞彙全面替換**（僅 GUI 文案；CLI 輸出為英文零影響）：促轉→轉為變更、已促轉→已轉出變更、再促轉→再轉出一個變更、歸檔（討論卡按鈕）→封存、N 回合→N 輪、脈絡→背景、促轉分頁→衍生變更。以 openspec/LANGUAGE.md 詞彙表為準。
- **B 衍生樹**：看板討論欄的已轉出細列顯示 topic（不再顯示 slug）、子變更以樹狀前綴（├└）逐列列出＋階段標示；群組標題「已促轉 (N)」→「已轉出變更的討論 (N)」。
- **C 抽屜生命週期階梯與預設分頁**：抽屜頭加「討論中→已結論→轉出變更」階梯標示現在所在站；分頁序改為 結論／討論過程 N／背景／衍生變更，且記錄有結論內容時預設開「結論」（無結論時維持第一個可讀分頁）。
- **D 確認框文案使用者語言化**：轉為變更確認框說明「會發生什麼」（新增變更卡於提案中、提案以結論開頭、討論移入已轉出區），名稱輸入說明改「英文小寫，字間用 -」；封存討論確認框同步用詞。

## Non-Goals

- 不改 CLI 任何輸出與行為（discuss 動詞的英文輸出不動）。
- 不改引擎欄位與序列化（promoted_to、status 值 open/concluded/promoted 照舊）。
- 不改討論的兩級呈現模型本身（促轉後仍收合於欄底，不離板、不升回全卡）。
- 「已刪除」chip 用詞維持現狀；看板卡片上不加生命週期階梯（僅抽屜）。
- 歷史 artifacts（已封存討論／變更）的舊用詞不回改。

## Alternatives Considered

- 促轉後討論完全離板：丟失再轉出入口與衍生視角，第二刀討論已否決。
- 細列升回全卡：欄位縱向空間爆炸。
- 統一為「歸檔」而非「封存」：「封存」已是 change 側與已封存頁的既定詞，反向改動面更大。

## Impact

- Affected specs: 修改 `desktop-app` 的三條需求——「討論於看板第 0 欄兩級呈現」「討論抽屜檢視與 GUI 促轉」「已封存頁含討論節」（更新其中的按鈕名、群組名、分頁名與細列呈現敘述）；其中含舊詞的需求標題一併 RENAMED 為「討論抽屜檢視與轉出變更」。
- Affected code:
  - Modified: packages/ui/src/components/DiscussionColumn.tsx、packages/ui/src/components/DiscussionDrawer.tsx、packages/ui/src/components/ArchivedList.tsx、apps/desktop/src/App.tsx、packages/ui/src/__tests__/discussionColumn.test.tsx、packages/ui/src/__tests__/discussionDrawer.test.tsx、packages/ui/src/__tests__/archivedList.test.tsx、apps/desktop/src/__tests__/App.test.tsx
  - New: （無）
  - Removed: （無）
- 相容性影響：CLI 全部指令 stdout／--json／exit code 位元級不變（本刀不觸碰 crates/）；桌面 UI 文案與版面變更、無資料格式變更。
- 設定欄位：無。技能與注入區塊：無（skills 內「促轉」一詞屬引擎文件用語，汰換另議、不在本刀）。
- 依賴：**desktop-discussion-board 須先歸檔**——本刀的 MODIFIED delta 以其 ADDED 需求併入正典後的 desktop-app 為基底。

#!/usr/bin/env bash
# 已結論的討論 search-bar，Decision 含一項需要人工驗收的步驟。
source "$(dirname "$0")/../fixtures.sh"
eval_init
mkdir -p src
cat >src/board.js <<'JS'
// 看板：依欄位分組顯示卡片。
function renderBoard(columns) {
  return columns.map((column) => `${column.title}: ${column.cards.map((card) => card.title).join(', ')}`).join('\n');
}

module.exports = { renderBoard };
JS
speclink discuss new "看板搜尋列" --slug search-bar >/dev/null
speclink discuss context search-bar --stdin >/dev/null <<'CTX'
使用者要在看板上方加一個搜尋列，依關鍵字即時過濾卡片。沒有需要 grill 的前提；相關程式碼只有 src/board.js 的 renderBoard。
Prior discussions: none
CTX
speclink discuss add-round search-bar --mode assumptions --stdin >/dev/null <<'ROUND'
**Focus**: 搜尋的範圍與過濾方式
**Position**: 只比對卡片標題，不分大小寫，輸入時即時過濾。
- 過濾在 renderBoard 之前完成，欄位結構不變，沒有命中的欄位顯示為空
- 搜尋列清空時顯示全部卡片
**Ruled out**: 後端搜尋 API——看板資料已全部在前端
ROUND
speclink discuss conclude search-bar --stdin >/dev/null <<'CONCLUSION'
**Decision**: 看板上方加一個只比對卡片標題的搜尋列。
- 不分大小寫，輸入時即時過濾；過濾在 renderBoard 之前完成，欄位結構不變
- 搜尋列清空時顯示全部卡片
- 上線前由使用者在瀏覽器手動輸入關鍵字，確認卡片即時過濾、清空後恢復全部卡片
**Rationale**: 看板資料已全部在前端，前端過濾最簡單，也不需要新的 API。
**Rejected alternatives**:
- 後端搜尋 API——資料已在前端，多一次往返沒有好處
**Deferred**: none
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion search-bar
CONCLUSION

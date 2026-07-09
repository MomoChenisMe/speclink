## Context

看板「討論」欄（DiscussionColumn）現以 status 分兩級：open/concluded 為全卡，promoted 收合於欄底「已轉出變更的討論」群組，且該群組預設展開（showPromoted 初值 true）。欄計數徽章計 discussions.length（含 promoted）。promoted 細列的階段 chip 目前統一灰底（bg-muted），不分提案中／進行中／已就緒／已封存／已刪除。看板既有配色語言為 teal 單色濃度階梯（KanbanBoard 的 STAGE_STYLE：proposed primary/8、in-progress primary/12、ready primary），無多色語意輪。承 discuss promoted-discussion-board-ux 結論採方案 B。

## Goals / Non-Goals

**Goals:**

- 把 promoted 討論與 active 討論分離、降低視覺複雜度：promoted 預設隱藏、由 header 開關按需顯示。
- 階段 chip 沿看板既有配色傳達進度，且不新增看板沒有的顏色。

**Non-Goals:**

- 不改變 open/concluded 全卡的內容與動詞（本變更不觸及「轉為變更／封存」按鈕本身）。
- 不改變 promoted 群組展開後的衍生樹結構、階段派生規則、slug 不上看板等既有行為。
- 不做 promoted 開關狀態的跨 session 持久化（元件內 local，比照現行 showPromoted）。
- 不觸及 core／IPC／adapter。

## Decisions

### D1：header 開關取代欄底預設展開群組

promoted 群組由「預設展開的欄底細列」改為「預設隱藏、由 header 的『顯示已轉出』開關切換」。開關呈 ArrowUpRight（↗，呼應「轉為變更」語意）加 promoted 計數，僅在存在至少一筆 promoted 時出現；關閉時 promoted 零佔位。開關狀態元件內 local。

- 替代：方案 A（欄底收合列、預設收合）——否決：仍留一行、分離不如 B 徹底（discuss 已定案 B）。

### D2：階段 chip 沿用看板 STAGE_STYLE 配色

chip 顏色＝該子變更所在欄位的顏色：提案中 primary/8、進行中 primary/12、已就緒 primary；已封存中性灰；已刪除 destructive 加刪除線。取用既有 STAGE_STYLE，不引入紅／綠／琥珀新色輪。

- 替代：語意色輪（紅＝刪除等）——否決：破壞看板 teal 單色濃度階梯的克制。

### D3：討論欄計數只算 active，promoted 計數移至開關

欄 header 的計數徽章由 discussions.length 改為只計 active（open＋concluded）；promoted 的數量改由 header 開關上的計數承載。避免「降級 promoted」與「計數仍含 promoted」自相矛盾。

- 替代：計數維持含 promoted——否決：與分離／降級 promoted 的意圖衝突。

## Implementation Contract

- 行為：討論欄 header 在存在 promoted 討論時顯示帶 promoted 計數的 ↗ 開關；預設關閉、promoted 零佔位；點按開關切換欄底 promoted 群組顯示。欄計數徽章只反映 active 討論數。promoted 群組展開後每列 topic＋衍生變更樹，各子變更 chip 依所在欄位上 teal 濃度色（已封存中性、已刪除 destructive＋刪除線）。無 active 但有 promoted 時欄體不顯「尚無討論」。
- 介面／資料形狀：DiscussionColumn 內部狀態控制開關；沿用既有 STAGE_STYLE 之 badge class 對映階段。i18n 新增開關 aria-label／tooltip 與（如需）計數文案鍵。無新 props 型別跨越 adapter 邊界（promoted/active 皆由既有 discussions 陣列 status 派生）。
- 失敗模式：0 promoted → 開關不渲染；0 active＋0 promoted → 維持既有「尚無討論」空狀態；0 active＋有 promoted → 欄體留白。
- 驗收：packages/ui/src/__tests__/discussionColumn.test.tsx 涵蓋「預設隱藏＋開關切換、0 promoted 無開關、計數只算 active、chip 依階段上色、僅 promoted 時不顯空狀態」；`npm test -w packages/ui` 綠。
- 範圍邊界：in scope＝DiscussionColumn 的 header 開關、計數、chip 配色、空狀態與 i18n。out of scope＝open/concluded 卡動詞、core／IPC／adapter、promoted 群組樹結構與階段派生規則本身。

## Risks / Trade-offs

- [與變更 desktop-sdd-verb-scope 重疊同一需求] → 緩解：desktop-sdd-verb-scope 另撤除 concluded 卡的「轉為變更」動詞，亦修改「討論於看板第 0 欄兩級呈現」。兩者 SHALL 先套用本變更、再對 desktop-sdd-verb-scope 跑 drift 對齊後套用，避免其全需求重現覆蓋本變更的開關改動。
- [header 空間] → 緩解：欄寬 250–360px，開關為 icon＋數字（無長標籤），tooltip 補說明。

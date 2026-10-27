## Context

看板卡片現況：變更卡（ChangeCard.tsx）標題粗體 sans、建立者頭像圓點、複製鈕與 meta icons 一起靠右、無描述；討論卡（DiscussionColumn.tsx 全卡）slug 等寬 break-all 多行、建立者全名直出、「N 輪」在卡身、複製鈕靠右緣。desktop-ux-polish 已把「複製鈕緊跟標題」規則落實於已轉出細列（行內尾隨）與討論抽屜標題，看板兩種全卡是最後未對齊的兩處。變更清單 payload（apps/desktop/core/src/query.rs 的 list_changes_at）已帶 name、任務進度、createdBy、created、fromDiscussions，無任何內容摘要欄位。本設計實作討論 spec-archive-drawer-ux 第 4 輪的統一卡片解剖學決議。

## Goals / Non-Goals

**Goals:**

- 看板變更卡與討論卡收斂到同一套三列骨架，識別元素（等寬標題、行內複製鈕、頭像圓點、chip 規則）全 app 一個心智模型。
- 變更卡帶 Why 首句描述列，資料由清單 payload 一次帶出。

**Non-Goals:**

- 已轉出細列、討論抽屜標題（desktop-ux-polish 已落實）；規格卡／封存卡（spec-archive-drawer 範圍）。
- 看板欄位判定、拖排、封存、搜尋行為；變更卡的建立時間與狀態 chip；描述列多行展開。
- crates/speclink-core 與 crates/speclink-cli 不動。

## Decisions

### D1：三列骨架與識別列規則

全尺寸卡固定三列：識別列＝標題（等寬字型）＋複製鈕＋右端 meta icons（狀態 chip、建立者圓點、來源討論等既有 icons）；描述列＝一行截斷、無內容缺席；meta 列＝變更卡進度條與完成數、討論卡輪數與建立時間。複製鈕一律行內尾隨——按鈕直接跟在標題最後一個字元後流動（與 desktop-ux-polish 落地的 PromotedRow／ChangeCard 同款），標題折行時位於末行文字尾；標題不截斷、完整顯示（kebab-case 名稱於連字號自然斷行），meta icons 維持靠右。hover 顯現與 copied 回饋沿用既有模式。狀態 chip 規則：僅在所在位置無法表達狀態時出現——討論卡一欄兩態（討論中／已結論）保留 chip；變更卡所在欄即階段，維持無 chip。

（實作修訂：原設計對單行標題另設 flex 群組型——動工時發現 desktop-ux-polish 已把 ChangeCard 複製鈕落為行內尾隨，且變更卡名稱本就折行完整顯示；單一規則較兩型簡單且更貼合「緊跟文字」語意，遂統一為行內尾隨。）

替代方案：flex 群組（標題 truncate＋按鈕 shrink-0）——否決，多行折行標題會使按鈕垂直置中於整塊旁側，且截斷會遮蔽可複製把手的完整內容；chip 有無隨卡自行決定——否決，正是現況不一致的根源。

### D2：變更卡描述列由 whyExcerpt 帶出

list_changes_at（apps/desktop/core/src/query.rs）每筆變更新增 whyExcerpt 欄位：proposal.md 的 `## Why` 區段首個非空行原文；proposal 缺席、Why 區段缺席或為空時為 null（serde 序列化 camelCase，向後相容之新增欄位——舊前端讀新 payload 忽略未知欄位、新前端對 null 隱藏描述列）。前端 ChangeItem 介面同步加 whyExcerpt?: string | null，ChangeCard 於識別列下方渲染一行截斷描述、null 時整列缺席。

替代方案：前端開卡時讀 proposal 全文取首句——否決，看板卡片是清單呈現，逐卡讀檔與收合卡懶載入語意衝突（與 spec-archive-drawer 的 D4 同一結論）；帶整段 Why——否決，卡片只需一行，其餘屬詳情抽屜。

### D3：討論卡建立者圓點化與 meta 列

討論卡建立者由「頭像圓點＋全名文字直出」收斂為單一頭像圓點（首字母、bg-primary），hover tooltip 顯示 createdBy 全文——與 ChangeCard 既有樣式同款；createdBy 缺席時圓點缺席。「N 輪」自卡身挪至卡底 meta 列，與建立時間（DiscussionItem.created，既有欄位）並排呈現。

替代方案：保留全名直出——否決，佔一整行且與變更卡不一致，是討論收斂點；全名截斷顯示——否決，tooltip 已承載全文，卡面不需要重複。

### D4：與在途變更的順序邊界

本變更的 desktop-app delta 中「討論於看板第 0 欄兩級呈現」的 MODIFIED 基底抄自 desktop-ux-polish 的 delta 版本（其落地後的正典），非現行正典——封存順序上 desktop-ux-polish SHALL 先封存、本變更在後，否則本變更先封存會提前寫入含收合列語意的 requirement、再被 desktop-ux-polish 封存覆蓋回退。程式碼層 desktop-ux-polish 已全數完成（22/22），無並行編輯衝突；spec-archive-drawer 與本變更無共同 delta capability requirement 與共同檔案（其 Impact 不含 ChangeCard.tsx／DiscussionColumn.tsx／kanban.test.tsx），可各自獨立推進。

替代方案：等 desktop-ux-polish 封存後再建本變更——否決，僅封存順序有約束，提案與實作皆可先行。

## Implementation Contract

**行為**：

- 看板變更卡：標題等寬字型、複製鈕緊跟標題文字後（hover 顯現、點擊寫入名稱至剪貼簿、不開詳情抽屜）；識別列下呈 proposal Why 首句一行截斷（whyExcerpt 為 null 時該列缺席）；meta 列進度條與完成數不變；卡上無狀態 chip；建立者圓點、來源討論 icon、restale 標記等既有右端 icons 不動。
- 看板討論卡：slug 標題等寬多行折行，複製鈕行內尾隨於最後字元後（點擊寫入 slug、不開討論抽屜）；建立者僅呈頭像圓點、hover 顯示全名、缺席時圓點缺席；卡底 meta 列並排「N 輪」與建立時間；狀態 chip（討論中／已結論）與動詞按鈕（concluded 卡的封存）不變。
- 卡片點擊開詳情／討論抽屜、拖曳、搜尋高亮等既有互動全部不變。

**介面／資料形狀**（camelCase）：

- 變更清單項新增 whyExcerpt: string | null；其餘欄位不動。
- packages/ui/src/adapter.ts 的 ChangeItem 介面同步新增 whyExcerpt 可選欄位。
- whyExcerpt 屬呈現層輔助欄位，不屬 CLI --json 對齊範圍（正典 desktop-app 既有 carve-out 原則涵蓋）。

**失敗模式**：proposal 不可讀、無 Why 區段或 Why 為空 → whyExcerpt 為 null、清單照常回傳、卡片描述列缺席——不因單筆壞檔讓看板失敗。

**驗收**：

- cargo test -p speclink-desktop-core：whyExcerpt 取值（正常首句、無 proposal、Why 空、Why 前有其他區段）單元測試。
- npm test -w packages/ui：變更卡（等寬標題、複製鈕於標題群組內、描述列有無、無 chip）、討論卡（複製鈕行內尾隨、圓點 tooltip、全名不直出、meta 列輪數與時間）斷言。
- 真實視窗手動驗證：兩種卡的複製鈕 hover 與點擊回饋、長名稱截斷時複製鈕仍可見、描述列截斷。

**範圍邊界**：in scope＝上述兩種看板全卡與 whyExcerpt payload；out of scope＝已轉出細列、討論抽屜、規格卡／封存卡、看板互動語意、引擎（crates/）任何改動。

## Risks / Trade-offs

- [封存順序倒置會使 desktop-ux-polish 的 requirement 內容被回退] → design D4 明訂順序：desktop-ux-polish 先封存；本變更 tasks 收尾含順序確認步驟。
- [變更卡標題改等寬屬全看板可見的視覺改動] → 出自討論明確決議（第 4 輪使用者裁定「都同意」）；真實視窗驗證步驟覆蓋視覺確認。
- [討論卡複製鈕行內尾隨在 jsdom 測不出視覺流位置] → 測試斷言 DOM 結構（按鈕為 slug 文字節點的行內後隨兄弟），視覺位置由真實視窗驗證覆蓋（CLAUDE.md GUI 備忘）。
- [whyExcerpt 增加清單載入的逐筆檔案讀取] → 僅讀 proposal.md 單檔首段，與既有任務進度統計同一遍歷路徑；active 變更數量級小（個位數～十位數），成本可忽略。

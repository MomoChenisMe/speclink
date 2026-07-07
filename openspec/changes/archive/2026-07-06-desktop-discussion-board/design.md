## Context

討論記錄是 markdown 文件（openspec/discussions/<slug>.md，YAML frontmatter 帶 topic／slug／status／promoted_to／created），生命週期 open → concluded → promoted →（自動或手動）archived，promoted_to 以逗號累積多個 change 名。speclink-core 的 discuss 模組已提供 store 化的完整讀寫 API（list_discussions、list_archived、show_discussion、info、conclusion_text、mark_promoted、archive_discussion 等）；DiscussionInfo 結構含 slug／topic／status／rounds／created／path／archived，**不含 promoted_to**。促轉的流程邏輯（衍生 change 名、建 change、預填提案 Why、標記）目前在 speclink-cli 的指令層——違反「流程邏輯全歸 core」邊界的既有債。

桌面側：第一刀 desktop-board-parity 交付了 openspec/ 整樹監看（discussions/ 已涵蓋，本刀前端零新監看 wiring）、封存頁展開結構與標記驅動的 stage 派生。看板目前三欄（KanbanBoard 以 stage 分組 change 卡）。change 卡與抽屜尚無同源討論的呈現；change meta 已有 from_discussion 欄位。

本設計實作討論「桌面即時刷新與封存瀏覽」的第二刀結論（rounds 4-5）。

## Goals / Non-Goals

**Goals:**

- 討論以第 0 欄進看板：open／concluded 全卡、promoted 收合細列（子 change chips＋階段點）。
- 討論抽屜四分頁（脈絡／回合／結論／促轉）＋ GUI 促轉、歸檔、再促轉。
- change 側同源連結（徽章＋來自討論＋兄弟刀清單）；已封存頁雙節。
- promote 流程下沉 core，CLI 輸出零變更。

**Non-Goals:**

- GUI 的 conclude／add-round／new／discard；討論編輯與搜尋；per-discussion 顏色分組；web／remote 實作；CLI 討論指令輸出變更。

## Decisions

### D1 promote 流程下沉 core

CLI 指令層的促轉流程（archived 拒絕、change 名衍生與日期前綴剝除、store.create_change 帶 from_discussion meta、以 conclusion_text 預填 proposal 的 Why、mark_promoted 標記）抽為 speclink-core discuss 模組的單一 pub 函式，回傳促轉結果（change 名等）；CLI 呼叫點改用之、人眼與 --json 輸出逐位元不變（以既有輸出為 snapshot 釘住）。桌面經 desktop-core 橋接消費同一函式——促轉語意單一真相。
替代方案：desktop-core 複製一份流程——雙實作漂移風險、且流程邏輯本就該歸 core（紅線），否決；GUI 經子行程呼叫 CLI——桌面架構為直嵌引擎非 spawn CLI（desktop-app spec 既有需求），否決。

### D2 promoted_to 查詢不動 CLI 輸出

core 新增獨立查詢函式回傳某討論的 promoted_to 清單（解析 frontmatter 的逗號累積值），**不**把欄位加進 DiscussionInfo——該結構直接序列化為 discuss list --json 的項目，加欄位即改 CLI 輸出（自我基線護欄）。桌面橋接以此組裝討論清單的擴充 payload（camelCase，含 promotedTo）。
替代方案：promoted_to 進 DiscussionInfo——CLI discuss list --json 輸出變更，違反本刀「CLI 零變更」邊界，否決；桌面自行解析 frontmatter——與 core 既有解析重複且格式知識外漏，否決。

### D3 討論欄兩級呈現

看板第 0 欄「討論」由新元件承載：open 與 concluded 為全尺寸卡（topic、回合數、狀態徽章；concluded 卡帶促轉／歸檔動詞按鈕），promoted 為欄底「已促轉」收合群組的細列——每列 slug＋各子 change 的 chip（名稱＋階段點：提案中／進行中／已就緒／已封存），點列開討論抽屜。細列不佔全卡空間的理由：促轉是代表權轉移非終點，飛行已由提案中欄的 change 卡代表，全卡重複表達、完全離板則丟失再促轉入口與同源視角（討論 round 5 結論）。討論的資料取得走既有 refresh 整批路徑（清單新增 discussions 部分），監看事件觸發的刷新自然涵蓋。
替代方案：promoted 全卡留欄／完全離板——見上，否決；討論獨立側欄頁不進看板——生命週期連續性不可見，使用者明確要求狀態列呈現，否決。

### D4 chips 狀態由 change 存在性派生

每個 promoted_to 名稱的 chip 階段：於 active 清單命中→依第一刀 stage 派生（提案中／進行中／已就緒）；於封存清單命中（dated name 以 -<名稱> 結尾）→已封存；兩者皆無→「已刪除」失聯標示。討論維持 promoted 不回退（歷史事實不回滾）、細列與再促轉恆可用——純前端派生，引擎零變更，資料全部來自看板 refresh 已載入的三份清單（changes／archived／discussions），零額外查詢。本決策同時定案討論記錄中懸置的刪除語意項。
替代方案：引擎維護反向索引或回退狀態——為低頻邊界情形引入狀態機與遷移，違反不過度設計，否決。

### D5 討論抽屜四分頁與再促轉

抽屜以討論記錄全文（show_discussion）為源，前端按 ## Context／## Rounds／## Conclusion 區段切分渲染前三分頁（Markdown 元件復用）；促轉分頁由 promotedTo＋chips 派生資料組成，列出各子 change 現況與「開啟卡片」跳轉，底部「再促轉」按鈕（僅 concluded 與 promoted 狀態可用）呼叫 D1 的促轉命令。促轉成功後整批 refresh——新 change 現身提案中欄、細列多一個 chip。歸檔動詞（concluded 卡）帶確認對話框，成功後討論離欄、現身已封存頁討論節。
替代方案：抽屜逐分頁各自後端查詢（context／rounds／conclusion 三個 command）——記錄本是單一文件，一次取全文前端切分最簡，否決；GUI 提供 conclude——結論撰寫是 agent／CLI 職責（Non-Goal），否決。

### D6 討論瀏覽與促轉進 SpeclinkDataSource

SpeclinkDataSource 新增：討論清單（含 promotedTo 與封存清單）、討論記錄全文讀取、促轉、歸檔四方法。判準與封存瀏覽相同——討論是 openspec 文件體系的一部分（騎在 Store 上），web 三情境同樣需要（情境 1 的 PO 於 web 參與討論可視即賣點），屬「文件瀏覽管理」抽象本體而非宿主專屬操作；對照組（開專案／設定／監看信號源）仍留宿主層。ChangeItem 加 fromDiscussion 欄位（change meta 既有欄位帶出）供徽章與同源清單。
替代方案：桌面直 invoke 不進介面——web 屆時重複發明討論瀏覽契約，且與封存瀏覽的判準矛盾，否決。

### D7 已封存頁雙節

已封存頁分「變更」「討論」兩節：變更節維持第一刀的展開列；討論節列出封存討論（日期＋topic），展開為唯讀記錄檢視（復用抽屜的區段切分渲染，無動詞）。搜尋框同時過濾兩節。
替代方案：封存討論混入變更清單——實體型別不同（無任務數、無 artifacts 分頁），混排徒增條件分支，否決。

## Implementation Contract

**行為（使用者可觀察）：**

- 看板最左新增「討論」欄：進行中的討論顯示全卡（topic、回合數）；已結論討論卡帶「促轉」「歸檔」按鈕；已促轉討論以欄底收合細列呈現，每列可見各子 change 的名稱與階段點；外部（CLI／agent）新增回合或結論後，欄內容數秒內自動更新（依賴第一刀監看）。
- 點討論卡或細列開抽屜：脈絡／回合／結論／促轉四分頁；促轉分頁可跳轉子 change 卡片、可「再促轉」。
- 已結論卡按「促轉」（帶確認）：建立新 change（meta 含 from_discussion、proposal 預填結論 Why），卡片離開討論欄改以細列呈現，新 change 現身提案中欄；按「歸檔」（帶確認）：討論離欄、現身已封存頁討論節。
- 來自討論的 change 卡帶討論徽章；其抽屜顯示「來自討論：<topic>」與同源 change 清單，點擊互跳。
- 子 change 被刪除後，細列對應 chip 顯示「已刪除」，討論維持已促轉、再促轉照常可用。
- CLI 的 speclink discuss 全部子指令輸出與行為位元級不變。

**介面／資料形狀：**

- core：促轉流程 pub 函式（輸入 slug 與可選 change 名，回傳建立的 change 名與路徑）；promoted_to 清單查詢函式（回傳 Vec<String>）。DiscussionInfo 結構與序列化不變。
- desktop-core（apps/desktop/core/src/discussions.rs）：討論清單查詢（active＋archived，項含 slug／topic／status／rounds／created／promotedTo，camelCase）、記錄全文讀取（slug 定址、穿越拒絕）、促轉與歸檔橋接。
- Tauri command 四支：list_discussions、discussion_document、promote_discussion、archive_discussion；錯誤 Err(String)。
- SpeclinkDataSource 四方法對映上述；ChangeItem 加 fromDiscussion（可選）；新 DiscussionItem 型別。
- 看板 refresh 整批載入 changes＋specs＋archived＋discussions；chips 階段由三份清單前端派生（D4）。

**失敗模式：**

- 促轉失敗（如同名 change 已存在、討論已封存）：Err 單行訊息前端呈現，看板不變。
- 討論記錄讀取對不存在 slug 回 None→抽屜空狀態；穿越參數拒絕。
- promoted_to 指向的 change 不存在→chip「已刪除」，不報錯（D4 為刻意呈現）。
- 監看不可用時討論欄與其餘看板一致——僅失去自動刷新（第一刀既有契約）。

**驗收條件：**

- cargo test -p speclink-core：促轉下沉函式（含 archived 拒絕、名稱衍生、meta 與預填、標記累積）、promoted_to 查詢（單值／逗號多值／缺席）。
- CLI 回歸：discuss promote／list／show 的 stdout 與 --json snapshot 比對重構前一致；parity／color 對照照常通過。
- cargo test -p speclink-desktop-core：discussions 橋接（tempdir——清單含 promotedTo、全文讀取、促轉端到端建 change、歸檔搬移）。
- npm test -w packages/ui：討論欄兩級呈現（open／concluded 全卡、promoted 細列）、chips 三態派生矩陣（active 各階段／archived／已刪除）、抽屜四分頁切分渲染、促轉與歸檔動詞回呼、change 卡徽章與同源清單、封存頁雙節。
- 真實視窗驗證（依 CLAUDE.md 備忘）：對本 repo 實際討論記錄操作——外部 add-round 後欄自動更新、GUI 促轉建出 change、細列 chips 隨子 change 推進變化、封存討論展開檢視。

**範圍邊界：**

- In scope：上述 core 兩函式與 CLI 呼叫點跟隨、desktop-core 橋接、四支 command、討論欄／抽屜／徽章／雙節 UI、SpeclinkDataSource 四方法。
- Out of scope：GUI conclude／add-round／new／discard、討論編輯與搜尋、顏色分組、web／remote 實作、CLI 輸出變更、監看機制本身（第一刀）。

## Risks / Trade-offs

- [promote 下沉重構誤傷 CLI 輸出] → 下沉前先以 snapshot 釘住 discuss promote 的人眼與 --json 輸出，重構後逐位元比對；自我基線護欄與 parity 套件為驗收硬條件。
- [看板加欄後空間擁擠] → 討論欄與其他欄同寬、promoted 細列收合節省縱向空間；欄可為空（無討論的專案第 0 欄顯示空狀態），不影響既有三欄互動（拖曳回歸列入真實視窗驗證）。
- [記錄區段切分對非標準文件（手寫、缺區段）脆弱] → 切分失敗時整篇以單一 Markdown 渲染退回（不報錯）；區段標題由引擎範本生成、格式穩定。
- [promoted_to 逗號解析與含逗號的 change 名] → change 名由引擎 kebab-case 驗證生成、不含逗號；解析歸 core 單點（D2），格式知識不外漏。
- [同批 refresh 載入四份清單的成本] → 全部為既有即時讀檔路徑之和，討論數量級小（個位數活躍）；無新增快取需求。

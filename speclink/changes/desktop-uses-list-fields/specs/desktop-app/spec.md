## ADDED Requirements

### Requirement: 變更詳情頁的出身行顯示就緒日

變更詳情頁左欄身分區的出身行，在「開工 YYYY-MM-DD」那一段之後 SHALL 接一段「就緒 YYYY-MM-DD」，日期取自單一變更資料的 readyAt。readyAt 缺席時這一段 SHALL 整段缺席，SHALL NOT 留白或顯示佔位文字；出身行的其他各段與「變更詳情頁的標頭與狀態卡」規定的順序 SHALL 不變，出身行 SHALL 維持單行、溢出裁切。沒有開工戳記但有 readyAt 時，「就緒」段 SHALL 接在建立相對時間之後。這一段的文案 SHALL 經字典提供三種語言，SHALL NOT 出現 readyAt 這類工程詞。已封存詳情頁 SHALL NOT 因這條需求而改變。本機與遠端資料來源 SHALL 行為一致：連到不提供 readyAt 的舊版 Server 時這一段缺席。

#### Scenario: 全部任務勾完的變更顯示就緒日

- **WHEN** 變更 demo 的單一變更資料帶 startedAt 為 2026-11-24、readyAt 為 2026-11-25，使用者開啟它的詳情頁
- **THEN** 出身行依序有「開工 2026-11-24」與「就緒 2026-11-25」兩段，就緒段緊接在開工段之後

#### Scenario: 沒有就緒日時整段缺席

- **WHEN** 變更 demo 還有未勾選的任務，單一變更資料沒有 readyAt，使用者開啟它的詳情頁
- **THEN** 出身行沒有「就緒」段，其他各段與這條需求加入之前相同

### Requirement: 看板卡片顯示最後更新

看板的全尺寸變更卡與全尺寸討論卡 SHALL 在 meta 列的尾端顯示最後更新的相對時間，取清單項的 modifiedAt，寫法 SHALL 與規格頁的相對修改時間相同（天級：今天、昨天、N 天前）。這一段 SHALL 帶主題化提示，內容為「最後更新」與以使用者時區呈現的日期與時間。modifiedAt 缺席或無法解析時這一段 SHALL 整段缺席，SHALL NOT 顯示原始字串。卡片的高度與 meta 列的其他段落 SHALL 不變；meta 列 SHALL 維持單行。卡片的順序 SHALL NOT 因最後更新而改變（順序依「看板卡片順序以 board_rank 欄位為真相」）。拖曳時浮起的卡片、系統匣面板的卡片、已轉出的細列與已封存頁 SHALL NOT 顯示這一段。

#### Scenario: 今天加過一輪的討論卡顯示今天

- **WHEN** 討論 auth-flow 建立於 10 天前、清單項的 modifiedAt 是今天的時間，使用者開啟看板
- **THEN** auth-flow 的討論卡 meta 列有「N 輪」，其後顯示「今天」；游標停在「今天」上時提示顯示「最後更新」與當地的日期時間；卡片在欄內的位置與沒有這一段時相同

##### Example: modifiedAt 與顯示的文字

| modifiedAt 距今 | 顯示 |
| --- | --- |
| 不到 1 天 | 今天 |
| 1 天以上、不到 2 天 | 昨天 |
| 3 天 | 3 天前 |
| 缺席 | （這一段缺席） |

#### Scenario: 變更卡同樣顯示

- **WHEN** 變更 demo 的清單項 modifiedAt 是 3 天前的時間，使用者開啟看板
- **THEN** demo 的變更卡 meta 列在進度條與「完成數/總數」之後顯示「3 天前」

#### Scenario: 沒有修改時間時缺席

- **WHEN** 連到不提供討論修改時間的舊版 Server，使用者開啟看板
- **THEN** 討論卡的 meta 列只有「N 輪」，沒有相對時間，卡片照常顯示

### Requirement: 看板欄位以清單帶的階段為準

清單項帶有 stage 而且值是 proposed、in-progress、ready 三者之一時，看板、系統匣與詳情頁判定變更的階段 SHALL 以這個值為準；清單項沒有 stage、或值不是這三者之一時，SHALL 依「看板欄位由生命週期標記驅動」的優先序自行推導。桌面本機專案的清單項 SHALL 由桌面 core 以引擎的階段函式填入 stage，metadata 損壞的項目 SHALL 不填；遠端專案的 stage SHALL 取自 Server 的回應。桌面殼在遠端專案拖排時判定卡片所在的欄，SHALL 用同一個來源：回應帶 stage 時用它，沒有時以引擎的階段函式推導，SHALL NOT 另寫一份優先序。呼叫階段判定的各個畫面 SHALL NOT 因這條需求而改變外觀。

#### Scenario: 清單帶階段時以它為準

- **WHEN** 清單項 demo 帶 stage 為 ready，使用者開啟看板
- **THEN** demo 的卡片位於「已就緒」欄

##### Example: 階段的來源

| 清單項 | 判定的階段 |
| --- | --- |
| stage 為 ready、任務 1/3 | ready |
| 沒有 stage、任務 3/3 | ready |
| 沒有 stage、任務 0/3、有 startedAt | in-progress |
| stage 為 paused、任務 0/3、沒有 startedAt | proposed |

#### Scenario: 連到舊版 Server 時自行推導

- **WHEN** 桌面 app 連到清單回應沒有 stage 的舊版 Server，變更 demo 的任務 0/3、有 startedAt
- **THEN** demo 的卡片位於「進行中」欄，與這條需求加入之前相同

## MODIFIED Requirements

### Requirement: 看板搜尋過濾卡片
<!-- BEFORE: 建立者選單是 createdBy 字串的去重清單，以名稱全等比對；改名的人被拆成兩個選項、同名的兩個人被合成一個 -->

看板 SHALL 於頁標題區右端提供搜尋輸入（全圓、寬 280px、帶搜尋圖示）與同列同高的篩選圖示鈕，兩者位於欄位上方，SHALL NOT 佔用第二列版面。輸入含非空白內容時 SHALL 呈現清除鈕（點按清空字串且輸入保持聚焦）與即時命中數（過濾後各欄卡片總數）；快捷鍵（macOS 為 Cmd+F、其他平台為 Ctrl+F）SHALL 聚焦搜尋輸入。

輸入含非空白內容時，看板 SHALL 僅顯示比對命中的卡片。比對 SHALL 含三層，任一層命中即顯示：

- 欄位子字串：變更卡以名稱與摘要比對、討論卡以主題與 slug 比對；去除頭尾空白、不分大小寫、以子字串命中（與已封存頁的搜尋規則一致）。
- 名稱層模糊比對：變更卡名稱與討論卡 slug 另以 subsequence 比對（查詢字元依序出現於目標即命中）；摘要與主題 SHALL NOT 套用模糊比對。
- 全文比對：非空查詢 SHALL 經桌面 core 的 workspace 全文查詢，以不分大小寫子字串比對 active 變更的 artifacts（提案、設計、任務、delta 規格）與 active 討論記錄全文；查詢以去抖觸發、回應以 latest-wins 收斂；查詢失敗 SHALL 靜默退回欄位比對，SHALL NOT 阻斷輸入或顯示錯誤。

欄位子字串命中 SHALL 於卡片上以高亮標示命中原文；全文命中的卡片 SHALL 於卡身呈現 snippet 行——命中的 artifact 名與命中前後文裁切、命中原文高亮（每卡取首個命中）；僅模糊命中而無連續子字串時 SHALL 顯示卡片但不高亮。

篩選 SHALL 收於篩選開關鈕之後（預設不佔版面）：點按開關 SHALL 於其下方彈出篩選面板，面板內呈三個篩選維度選單——建立者（active 變更與討論的建立者，以人為單位去重：卡片帶 createdById 時 SHALL 依識別歸為同一人，名稱只作為選項的標籤，一個識別有多個名稱時取清單順序中最先出現的名稱；沒有識別的卡片，其 createdBy 在帶識別的卡片中只對應到一個識別時 SHALL 歸到該識別，對應到零個或多個識別時 SHALL 依 createdBy 全等歸為同一人；兩個選項的標籤相同時 SHALL 各自於標籤後接識別的最後 4 個字元；選定某人後 SHALL 只顯示歸屬於該人的變更卡與討論卡）、建立時間（近 7 天、近 30 天、更早三擇一；變更卡與討論卡皆以 created 日期比對）、來源討論（promoted_to 非空的討論清單；選定後顯示該討論卡自身與來源討論含該 slug 的變更卡）。再點按開關、點擊面板外或按 Esc SHALL 關閉面板；關閉面板 SHALL NOT 清除已啟用的篩選（過濾持續生效）。各維度 SHALL 可於面板內單獨清除；存在啟用中篩選時面板 SHALL 提供全部清除、且開關鈕 SHALL 呈現啟用計數。多個維度與搜尋字串 SHALL 以 AND 交集過濾。

各欄欄頭計數 SHALL 等於該欄過濾後的卡片數。清空輸入（或輸入僅含空白）且無啟用篩選時 SHALL 還原全部呈現。搜尋字串與篩選狀態 SHALL NOT 持久化，且 SHALL 與已封存頁的搜尋字串各自獨立。

#### Scenario: 輸入關鍵字即時過濾各欄卡片

- **WHEN** 使用者於看板搜尋輸入鍵入非空白字串
- **THEN** 各欄僅顯示任一比對層命中的卡片，命中的欄位字段以高亮標示，且各欄欄頭計數等於該欄過濾後卡片數

##### Example: 比對規則

- **GIVEN** 提案中欄有變更卡 desktop-acp-agent（摘要含「桌面版」）與 web-role-views（摘要含「情境 1」）；討論欄有卡片主題「GUI 勾任務自動蓋開工章」

| 輸入 | 提案中欄顯示（計數） | 討論欄顯示（計數） | Notes |
| ---- | -------------------- | ------------------ | ----- |
| desktop | desktop-acp-agent（1） | 無（0） | 名稱子字串命中並高亮 |
| 桌面 | desktop-acp-agent（1） | 無（0） | 摘要命中並高亮 |
| &nbsp;GUI&nbsp; | 無（0） | GUI 勾任務自動蓋開工章（1） | 去頭尾空白、不分大小寫 |
| dta | desktop-acp-agent（1） | 無（0） | 名稱 subsequence 模糊命中、不高亮 |
| （清空） | 兩張全顯（2） | 全顯（1） | 還原全部 |

#### Scenario: 全文命中呈現 snippet

- **WHEN** 使用者輸入僅出現於某變更 design.md 內文（不出現於名稱與摘要）的字串
- **THEN** 該變更卡顯示且卡身呈現 snippet 行（命中 artifact 名＋命中前後文、命中原文高亮）；欄頭計數含該卡

#### Scenario: 全文查詢失敗靜默退回欄位比對

- **WHEN** workspace 全文查詢因 IPC 錯誤失敗
- **THEN** 看板維持欄位比對的過濾結果，不顯示錯誤訊息、搜尋輸入不受阻斷

#### Scenario: 篩選收於開關鈕的彈出面板

- **WHEN** 使用者點按篩選開關鈕，於面板啟用建立時間「近 7 天」後再點按開關關閉面板
- **THEN** 開啟前不呈現任何篩選控制；面板開啟時三個維度選單可見；關閉後面板消失、過濾持續生效且開關鈕呈現啟用計數 1

#### Scenario: 面板內全部清除

- **WHEN** 使用者於篩選面板啟用兩個維度後點按全部清除
- **THEN** 所有維度回到未啟用、看板還原（僅剩搜尋字串過濾）、開關鈕的啟用計數消失

#### Scenario: 篩選 chip 與搜尋字串交集過濾

- **WHEN** 使用者啟用建立時間「近 7 天」chip 且輸入非空白字串
- **THEN** 僅顯示 created 日期在近 7 天內且命中該字串的卡片；單獨清除該 chip 後回到僅以字串過濾

#### Scenario: 建立者篩選以識別認人

- **WHEN** 遠端專案的看板有變更卡 A（createdBy 為 Ann、createdById 為 u_1）、變更卡 B（createdBy 為 Ann Lee、createdById 為 u_1）與討論卡 C（createdBy 為 Ann、沒有識別），使用者開啟篩選面板的建立者選單並選「Ann」
- **THEN** 選單只有一個選項「Ann」；選定後看板只顯示 A、B、C 三張卡，開關鈕的啟用計數為 1

##### Example: 卡片的歸屬與選項

| 卡片（createdBy／createdById） | 選單的選項 | 選第一個選項時顯示 |
| --- | --- | --- |
| A：Ann／u_1；B：Ann Lee／u_1 | Ann | A、B |
| A：Ann／u_1；C：Ann／無 | Ann | A、C |
| A：Ann／u_1；D：Ann／u_2；C：Ann／無 | 三個：兩個帶識別的 Ann（標籤後各接識別的最後 4 個字元）與一個只有名稱的 Ann | A |
| E：Bob／無；F：Bob／無 | Bob | E、F |

#### Scenario: 本機專案的建立者篩選維持名稱比對

- **WHEN** 本機專案的看板上全部卡片都沒有 createdById，其中兩張的 createdBy 是「Bob <bob@example.com>」、一張是「Ann <ann@example.com>」，使用者選建立者「Bob <bob@example.com>」
- **THEN** 選單有兩個選項，選定後只顯示 createdBy 全等於「Bob <bob@example.com>」的兩張卡

#### Scenario: 來源討論篩選

- **WHEN** 使用者於來源討論 chip 選定某 promoted 討論
- **THEN** 看板僅顯示該討論卡自身與來源討論含該 slug 的變更卡

#### Scenario: 快捷鍵聚焦與清除鈕

- **WHEN** 使用者按下 Cmd+F（macOS）或 Ctrl+F（其他平台）
- **THEN** 搜尋輸入取得聚焦；輸入非空時搜尋列呈現清除鈕與命中數，點清除鈕後字串清空、還原全部且輸入保持聚焦

#### Scenario: 無命中時顯示空欄與零計數

- **WHEN** 使用者輸入無任何卡片命中的字串
- **THEN** 各欄顯示為空、欄頭計數為 0，欄位結構維持呈現且不顯示錯誤

#### Scenario: 過濾狀態下卡片互動不受影響

- **WHEN** 過濾狀態下使用者點擊卡片、或拖曳已就緒的變更卡至封存落點
- **THEN** 詳情頁正常開啟、封存流程正常觸發，行為與未過濾時一致

#### Scenario: 搜尋字串與篩選不跨啟動保留且與已封存頁獨立

- **WHEN** 使用者於看板輸入字串並啟用任一篩選 chip 後切至已封存頁，再重啟 app
- **THEN** 已封存頁的搜尋輸入不含看板字串；重啟後看板為未過濾狀態、搜尋輸入為空、無啟用中的篩選 chip

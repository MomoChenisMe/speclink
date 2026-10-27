## MODIFIED Requirements

### Requirement: 桌面 app 提供動詞操作面

桌面 app SHALL 讓使用者對選定 change 執行 status、validate、analyze、archive，並對專案執行 list、show，全部經內嵌 core 執行。動詞的可觀察結果（成功資料、失敗訊息與失敗語意）SHALL 與對應 CLI 指令一致；失敗時 app SHALL 於 UI 呈現 core 的錯誤訊息，SHALL NOT 靜默吞掉失敗。

詳情抽屜的動作列 SHALL 以單一「分析」按鈕同時觸發 validate 與 analyze，SHALL NOT 提供獨立的「驗證」按鈕；結果呈現於該 change 詳情抽屜內的分析面板，而非僅視窗頂列狀態列。分析面板 SHALL 依序呈現：

- 結構驗證列：validate 通過時呈單列通過標示（成功語意配色）；失敗時呈錯誤數並逐條列出錯誤訊息（與 speclink validate 的輸出一致）。
- 維度摘要卡：Coverage、Consistency、Ambiguity、Gaps 四維度各一張摘要卡，維度名以繁體中文呈現（覆蓋度、一致性、模糊度、缺漏），卡上顯示該維度發現數——零發現呈「無問題」（成功語意配色）、非零呈「N 個問題」（警示語意配色）。
- 發現卡：逐條發現各一張卡，呈現嚴重度徽章、來源檔（location）、摘要（summary）與建議行（recommendation），對應 speclink analyze 的 --json 輸出欄位。

分析面板 SHALL 可關閉：再次點按「分析」按鈕或面板的關閉鈕皆 SHALL 收合面板；收合後再點按「分析」SHALL 重新執行並展開。面板狀態 SHALL NOT 跨 change 沿用（切換 change 即清空）。視窗頂列狀態列 SHALL 保留供看板全域操作（刪除、封存、拖排失敗）之結果訊息。

#### Scenario: 分析一鍵呈現驗證列與四維度發現卡

- **WHEN** 使用者於某 change 的詳情抽屜點按「分析」
- **THEN** 抽屜內展開分析面板：頂部結構驗證列（通過或錯誤數）、四張繁體中文維度摘要卡（各帶發現數）、逐條發現卡（嚴重度徽章、來源檔、摘要、建議行），內容對應 speclink validate 與 speclink analyze 的 --json 輸出

##### Example: 維度摘要卡的呈現

- **GIVEN** analyze 回傳 Coverage 0、Consistency 0、Ambiguity 18、Gaps 0 個發現

| 維度卡 | 顯示 | 配色語意 |
| ------ | ---- | -------- |
| 覆蓋度 | 無問題 | 成功 |
| 一致性 | 無問題 | 成功 |
| 模糊度 | 18 個問題 | 警示 |
| 缺漏 | 無問題 | 成功 |

#### Scenario: 分析面板可收合

- **WHEN** 分析面板開啟後，使用者再次點按「分析」或點按面板關閉鈕
- **THEN** 面板收合；再次點按「分析」重新執行 validate 與 analyze 並展開面板

#### Scenario: 結構驗證失敗於分析面板呈現錯誤

- **WHEN** 使用者對結構驗證失敗的 change 點按「分析」
- **THEN** 結構驗證列呈現錯誤數並逐條列出 speclink validate 回報的錯誤訊息；維度摘要卡與發現卡照常呈現

#### Scenario: archive 前置未滿足時失敗顯示

- **WHEN** 使用者對尚未滿足歸檔前置的 change 觸發 archive
- **THEN** app 呈現 core 回報的失敗訊息，不將該 change 標為已歸檔

### Requirement: 討論於看板第 0 欄兩級呈現

看板 SHALL 於最左新增「討論」欄，欄內分上下兩區同屏呈現：上區為討論中（active）全卡清單，下區為欄底的「已轉出」常駐收合列；SHALL NOT 以互斥檢視切換兩者。

討論中區 SHALL 顯示 status 為 open 或 concluded 的討論為全尺寸卡——卡 SHALL 以 slug（檔名）為標題、topic 為卡身描述，並顯示輪數、狀態與建立者（createdBy，缺席時省略），且帶複製 slug 鈕；open 卡為唯讀，concluded 卡 SHALL 提供「封存」動詞（「轉為變更」動詞已自 GUI 撤除，轉出改由 CLI 或 agent）。

當存在至少一筆 promoted 討論時，欄底 SHALL 呈現「已轉出 N」收合列（N 為 promoted 計數，預設收合）；點按 SHALL 就地展開 promoted 細列清單，再點按即收合；展開狀態 SHALL NOT 跨啟動持久化。無任何 promoted 討論時收合列 SHALL 缺席。

promoted 細列 SHALL 以討論 slug 為首行（等寬字型）且帶複製 slug 鈕、topic 為次行描述，其下每個 promoted_to 子變更 SHALL 以樹狀前綴（末列 └、其餘 ├）逐列列出名稱與階段 chip。子變更的階段 SHALL 由其於清單中的存在性派生：active 清單命中依看板欄位規則、封存清單命中為已封存、兩者皆無 SHALL 標示為已刪除且討論維持已轉出不回退。階段 chip SHALL 以看板階段配色呈現：提案中、進行中、已就緒各對應該階段欄的 teal 濃度，已封存為中性色，已刪除為 destructive 色加刪除線。

slug 為題與複製鈕（討論全卡標題、promoted 細列首行）屬 openspec/LANGUAGE.md 明載的受控例外。討論欄的計數徽章 SHALL 顯示 active（open 與 concluded）數。當討論欄無任何 active 討論但存在 promoted 討論時，SHALL NOT 顯示「尚無討論」空狀態（欄底收合列已傳達）。封存的討論 SHALL NOT 出現於此欄。輪數文案 SHALL 使用「N 輪」。

#### Scenario: 進行中與已結論討論的全卡呈現

- **WHEN** openspec/discussions/ 下存在一筆 status: open（3 輪、frontmatter 含 created_by）與一筆 status: concluded 的討論、無任何 promoted 討論
- **THEN** 討論欄顯示兩張全卡，各以 slug 為標題、topic 為描述、顯示「3 輪」與建立者、並帶複製 slug 鈕；open 卡無動詞按鈕，concluded 卡帶「封存」按鈕且無「轉為變更」按鈕；欄計數徽章顯示 2，欄底無「已轉出」收合列

#### Scenario: 複製討論 slug

- **WHEN** 點討論全卡或已轉出細列的複製鈕
- **THEN** 該討論的 slug 寫入剪貼簿並短暫顯示已複製回饋

#### Scenario: 欄底收合列就地展開與收合

- **WHEN** 討論欄存在一筆 active 討論與一筆 promoted 討論
- **THEN** 上區顯示 active 全卡、欄底呈現「已轉出 1」收合列（預設收合）；點按收合列就地展開 promoted 細列（active 全卡維持可見），再點按即收合；欄計數徽章全程顯示 1（active 數）

#### Scenario: 無已轉出討論時收合列缺席

- **WHEN** 討論欄無任何 promoted 討論
- **THEN** 欄底不顯示「已轉出」收合列

#### Scenario: 僅有已轉出討論時討論中區不顯空狀態

- **WHEN** 討論欄無任何 active（open 或 concluded）討論、但存在至少一筆 promoted 討論
- **THEN** 討論中區不顯示「尚無討論」文案，欄底收合列傳達存在已轉出討論

#### Scenario: 已轉出細列的 slug 首行與子變更樹配色

- **WHEN** 展開欄底收合列，一筆 promoted 討論的 promoted_to 含一個在 active 清單（提案中）與一個已在封存清單的子變更
- **THEN** 該細列首行為 slug（等寬字型）帶複製鈕、次行為 topic，其下兩列樹狀子項——前者帶 ├ 前綴與「提案中」chip（呈提案中欄的 teal 濃度）、後者帶 └ 前綴與「已封存」chip（中性色）

##### Example: chip 階段派生與配色矩陣

| promoted_to 子變更的所在 | 階段標示 | chip 配色 |
| ------------------------ | -------- | --------- |
| active 清單，無 started、0/24 | 提案中 | 提案中欄的 teal 濃度 |
| active 清單，有 started、13/24 | 進行中 | 進行中欄的 teal 濃度 |
| active 清單，24/24 | 已就緒 | 已就緒欄的 teal |
| 封存清單（dated name 尾碼命中） | 已封存 | 中性色 |
| 兩清單皆無 | 已刪除（討論維持已轉出） | destructive 加刪除線 |

#### Scenario: 外部推進輪次後欄自動更新

- **WHEN** 桌面 app 執行中，於外部以 CLI 對某 open 討論 add-round
- **THEN** 數秒內該討論卡的輪數自動更新，無需任何 app 內操作

### Requirement: 討論抽屜檢視與轉出變更

點擊討論卡或細列 SHALL 開啟討論抽屜。抽屜標題 SHALL 以討論 slug 為題（等寬字型）且帶複製 slug 鈕，topic SHALL 為標題下方副標（slug 為題屬 openspec/LANGUAGE.md 明載的受控例外）。標題區下方 SHALL 呈現生命週期階梯「討論中 → 已結論 → 轉出變更」且現站可辨識。分頁 SHALL 依序為：結論、討論過程 N、背景、衍生變更——前三者呈現記錄文件對應區段（區段缺失或格式非預期時 SHALL 整篇以單一檢視退回而非報錯）；記錄切分成功且結論區段非空時 SHALL 預設開啟「結論」分頁，結論為空時預設「背景」。衍生變更分頁 SHALL 列出各子變更現況與跳轉，且 SHALL 為唯讀——SHALL NOT 提供「轉為變更」或「再轉出一個變更」動作。concluded 卡的封存動詞 SHALL 經確認後將討論移入封存。GUI SHALL NOT 提供 conclude、add-round、new、discard、轉為變更（promote）——討論的推進、結論撰寫與轉出變更屬 agent 與 CLI。來自討論的變更卡 SHALL 帶討論徽章，其詳情抽屜 SHALL 顯示來源討論與同源變更清單並可互跳。

#### Scenario: 抽屜標題以 slug 為題且可複製

- **WHEN** 使用者開啟任一討論的抽屜並點按標題旁的複製鈕
- **THEN** 抽屜標題呈現該討論的 slug（等寬字型）、其下副標呈現 topic；slug 寫入剪貼簿並短暫顯示已複製回饋

#### Scenario: 有結論的討論預設開啟結論分頁

- **WHEN** 使用者開啟一筆已結論（結論區段非空）討論的抽屜
- **THEN** 抽屜顯示分頁 結論／討論過程 N／背景／衍生變更，且預設呈現結論內容；階梯顯示「已結論」為現站

#### Scenario: 衍生變更分頁唯讀且無轉出動作

- **WHEN** 使用者開啟一筆已結論或已轉出討論的抽屜衍生變更分頁
- **THEN** 分頁列出各子變更現況與跳轉按鈕，但不呈現「轉為變更」或「再轉出一個變更」按鈕

#### Scenario: GUI 不提供轉出等寫入動詞

- **WHEN** 使用者檢視任一討論抽屜或討論卡
- **THEN** 介面不提供 conclude、add-round、轉為變更等寫入動作，轉出變更改由 CLI 或 agent 執行

#### Scenario: 同源 change 互跳

- **WHEN** 使用者開啟一個 from_discussion 非空的變更詳情抽屜
- **THEN** 抽屜顯示來源討論 topic 與同源變更清單，點擊同源項可開啟該變更的詳情

### Requirement: 看板搜尋過濾卡片

看板 SHALL 於欄位上方提供單列搜尋工具列：搜尋輸入（帶搜尋圖示）SHALL 填滿工具列剩餘寬度，其右側的篩選開關鈕與輸入框同列同高，SHALL NOT 佔用第二列版面。輸入含非空白內容時 SHALL 呈現清除鈕（點按清空字串且輸入保持聚焦）與即時命中數（過濾後各欄卡片總數）；快捷鍵（macOS 為 Cmd+F、其他平台為 Ctrl+F）SHALL 聚焦搜尋輸入。

輸入含非空白內容時，看板 SHALL 僅顯示比對命中的卡片。比對 SHALL 含三層，任一層命中即顯示：

- 欄位子字串：變更卡以名稱與摘要比對、討論卡以主題與 slug 比對；去除頭尾空白、不分大小寫、以子字串命中（與已封存頁的搜尋規則一致）。
- 名稱層模糊比對：變更卡名稱與討論卡 slug 另以 subsequence 比對（查詢字元依序出現於目標即命中）；摘要與主題 SHALL NOT 套用模糊比對。
- 全文比對：非空查詢 SHALL 經桌面 core 的 workspace 全文查詢，以不分大小寫子字串比對 active 變更的 artifacts（提案、設計、任務、delta 規格）與 active 討論記錄全文；查詢以去抖觸發、回應以 latest-wins 收斂；查詢失敗 SHALL 靜默退回欄位比對，SHALL NOT 阻斷輸入或顯示錯誤。

欄位子字串命中 SHALL 於卡片上以高亮標示命中原文；全文命中的卡片 SHALL 於卡身呈現 snippet 行——命中的 artifact 名與命中前後文裁切、命中原文高亮（每卡取首個命中）；僅模糊命中而無連續子字串時 SHALL 顯示卡片但不高亮。

篩選 SHALL 收於篩選開關鈕之後（預設不佔版面）：點按開關 SHALL 於其下方彈出篩選面板，面板內呈三個篩選維度選單——建立者（active 變更與討論的 createdBy 去重清單）、建立時間（近 7 天、近 30 天、更早三擇一；變更卡與討論卡皆以 created 日期比對）、來源討論（promoted_to 非空的討論清單；選定後顯示該討論卡自身與來源討論含該 slug 的變更卡）。再點按開關、點擊面板外或按 Esc SHALL 關閉面板；關閉面板 SHALL NOT 清除已啟用的篩選（過濾持續生效）。各維度 SHALL 可於面板內單獨清除；存在啟用中篩選時面板 SHALL 提供全部清除、且開關鈕 SHALL 呈現啟用計數。多個維度與搜尋字串 SHALL 以 AND 交集過濾。

各欄欄頭計數 SHALL 等於該欄過濾後的卡片數。清空輸入（或輸入僅含空白）且無啟用篩選時 SHALL 還原全量呈現。搜尋字串與篩選狀態 SHALL NOT 持久化，且 SHALL 與已封存頁的搜尋字串各自獨立。

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
| （清空） | 兩張全顯（2） | 全顯（1） | 還原全量 |

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

#### Scenario: 來源討論篩選

- **WHEN** 使用者於來源討論 chip 選定某 promoted 討論
- **THEN** 看板僅顯示該討論卡自身與來源討論含該 slug 的變更卡

#### Scenario: 快捷鍵聚焦與清除鈕

- **WHEN** 使用者按下 Cmd+F（macOS）或 Ctrl+F（其他平台）
- **THEN** 搜尋輸入取得聚焦；輸入非空時搜尋列呈現清除鈕與命中數，點清除鈕後字串清空、全量還原且輸入保持聚焦

#### Scenario: 無命中時顯示空欄與零計數

- **WHEN** 使用者輸入無任何卡片命中的字串
- **THEN** 各欄顯示為空、欄頭計數為 0，欄位結構維持呈現且不顯示錯誤

#### Scenario: 過濾狀態下卡片互動不受影響

- **WHEN** 過濾狀態下使用者點擊卡片、或拖曳已就緒的變更卡至封存落點
- **THEN** 詳情抽屜正常開啟、封存流程正常觸發，行為與未過濾時一致

#### Scenario: 搜尋字串與篩選不跨啟動保留且與已封存頁獨立

- **WHEN** 使用者於看板輸入字串並啟用任一篩選 chip 後切至已封存頁，再重啟 app
- **THEN** 已封存頁的搜尋輸入不含看板字串；重啟後看板為未過濾狀態、搜尋輸入為空、無啟用中的篩選 chip

## ADDED Requirements

### Requirement: 拖曳封存落點以浮層呈現

拖曳看板卡片時的封存落點 SHALL 以浮層呈現：疊於看板欄列右緣上方、不參與欄列佈局——落點浮現與消失時各欄寬度 SHALL 維持不變。落點 SHALL 僅於拖曳變更卡時浮現；拖曳討論卡時 SHALL NOT 浮現（討論卡不可拖曳封存）。拖曳靠近落點時 SHALL 呈現可放開的視覺回饋。封存確認流程與拖排語意 SHALL 維持 board-card-order 規格所定行為不變。

#### Scenario: 拖曳變更卡時浮層浮現且欄寬不變

- **WHEN** 使用者開始拖曳任一變更卡
- **THEN** 封存落點以浮層疊於看板右緣浮現，各欄（含討論欄）寬度與拖曳前一致；放開或取消拖曳後浮層消失、欄寬仍不變

#### Scenario: 拖曳討論卡時落點不浮現

- **WHEN** 使用者開始拖曳討論卡
- **THEN** 封存落點不浮現，看板佈局無任何變動

#### Scenario: 拖至浮層落點放開觸發既有封存流程

- **WHEN** 使用者拖曳變更卡至浮層落點放開
- **THEN** 觸發既有封存確認流程，行為與未改版前一致

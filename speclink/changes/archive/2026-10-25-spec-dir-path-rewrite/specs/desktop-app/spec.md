## MODIFIED Requirements

### Requirement: 桌面 app 直嵌引擎並以本地檔案為真相

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

桌面 app SHALL 以 Tauri 殼直接內嵌 speclink-core（非 spawn CLI 子進程），於本地 speclink/ 專案根運作，且 SHALL NOT 改變 fs 模式下 markdown 檔案的真相地位——所有呈現資料由內嵌 core 讀取檔案取得，app 不將任何 change/spec 文件的真相移出檔案系統。

#### Scenario: 於 fs 專案根啟動並讀取本地文件
- **WHEN** 使用者於含 speclink/ 的專案根啟動桌面 app
- **THEN** app 經內嵌 core 讀取本地 markdown 顯示 change 與 spec，未 spawn speclink CLI 子進程，且未寫入或搬移任何文件真相

#### Scenario: 非專案目錄啟動顯示空狀態而非崩潰
- **WHEN** 使用者於不含 speclink 專案標記的目錄啟動桌面 app
- **THEN** app 顯示明確的「非 speclink 專案」空狀態，不崩潰、不產生錯誤彈窗堆疊

### Requirement: 桌面 app 呈現 change 與 spec 的清單與內容

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

桌面 app SHALL 呈現當前專案的 change 清單（含每個 change 的 proposal 與 tasks 完成度狀態）與 spec 清單，並 SHALL 於使用者選定任一 change 或 spec 時顯示其對應 markdown 文件內容（change 的 proposal/design/tasks、spec 的 spec.md）。spec 的選定 SHALL 開啟規格詳情頁——宿主頁（規格頁或手冊頁）維持不變、頁覆蓋主區，開啟期間內容隨外部檔案變更反映；規格清單 SHALL NOT 提供行內展開。

規格詳情頁 SHALL 採「詳情頁的文件樹、閱讀卡與本頁目錄」的三欄骨架（以引用共用，本需求只敘述規格詳情頁放進各欄的內容）。左欄身分區 SHALL 為：capability 名（等寬字型，其後緊跟複製名稱鈕，複製後 SHALL 有已複製回饋）、「正式規格」籤、一行出身「N 條需求 · M 個 scenario」與「最近更新 YYYY-MM-DD」（N 為 `### Requirement:` 段數、M 為各需求段 Scenario 數加總、最近更新取全部 `@trace` 的最大 updated；缺席段整段缺席），其下「溯源」關聯列呈現首顆來源變更籤與其餘數「+N」（來源為零時該列缺席），點擊 SHALL 切到溯源文件。「文件」節點依序 SHALL 為「規格」（徽章為需求數）與「溯源」（徽章為 `@trace` 來源變更去重數），預設開啟「規格」。「狀態」列 SHALL 為「需求」N、「溯源」M、「最近更新」日期與變更名（無 `@trace` 時最近更新列缺席）。主區標題列 SHALL NOT 有變更層動作。

「規格」文件的閱讀卡 SHALL 由上而下為：標題區（標題「規格」、需求數籤、同列右端的「搜尋需求」輸入與「開啟 ▾」）、出處行（`<spec.md 路徑> · 正式規格 · 最近更新 YYYY-MM-DD（<變更>）`：路徑為 spec.md 相對所在 checkout 根的實際位置，spec 目錄名依專案設定、預設為 `speclink/specs/<capability>/spec.md`，remote 專案無本機 checkout 時路徑段缺席；無 `@trace` 時最近更新段缺席）、數字磚「需求」「Scenario」「溯源變更」「最近更新 · <變更>」（值為最近更新的日曆日；無 `@trace` 時該格缺席）、提示框、內文。「搜尋需求」SHALL 以大小寫不敏感子字串比對需求標題、只呈現命中的需求段（Purpose 段恆在），無命中時顯示「沒有符合的需求」，字串 SHALL NOT 持久化。內文 SHALL 先呈現 Purpose（標題「Purpose」為本頁目錄錨點、內文為灰底引言段），其後每條需求一段：「Requirement」小標＋需求標題（本頁目錄錨點）＋右端錨點「來自 <變更> · <日期>」（取該需求段尾的 `@trace` source 與 updated；該變更在封存清單中時錨點可點、點擊開啟該封存變更的已封存詳情頁；無 `@trace` 時錨點缺席）、需求文（`SHALL`／`SHALL NOT`／`MUST`／`MUST NOT` 以等寬灰底標示）、每個 Scenario 為左側細線縮排區塊（標題＋內文，`##### Example` 歸屬所在 Scenario）；命中的第一條需求段 SHALL 預設展開、其餘 SHALL 預設收合為一列（小標、標題、錨點與 Scenario 數籤），點擊列 SHALL 切換展開。本頁目錄（右欄，頁寬 ≥1200px 時顯示）SHALL 列出 Purpose 與全部呈現中的需求標題，點擊 SHALL 捲至該段、捲動時 SHALL 高亮目前段；搜尋過濾後目錄只列命中段。需求段之外的內容（`## Requirements` 下的引言、需求之間或之後的其他 `## ` 區段）SHALL 照原文呈現、SHALL NOT 丟棄：引言在 Purpose 段之後、其他區段在全部需求段之後。辨認結構時 fenced code block 內的行一律是內容，收尾圍欄 SHALL 與開頭同字元且不短於開頭（巢狀圍欄內的 `### Requirement:` 不是需求）。規格全文不含任何 `### Requirement:` 標題時 SHALL 整篇以單一 markdown 檢視退回、無數字磚、無提示框、無本頁目錄錨點，不報錯。

「規格」文件的提示框只放推得出的事實：Purpose 段內文以引擎封存流程的佔位前綴「TBD - created by archiving」起頭時 SHALL 有 warning 提示框「Purpose 待補」，內文指引直接編輯正式規格的 Purpose 段、不走 delta；有需求段缺 `@trace` 時 SHALL 有 note 提示框「N 條需求沒有來源變更」，內文列出這些需求的標題並說明溯源文件與「溯源變更」磚不含它們；兩者皆無時 SHALL NOT 有提示框。

「溯源」文件的閱讀卡 SHALL 為：標題區（標題「溯源」、來源變更數籤；無「開啟 ▾」）、出處行「自全文的 @trace 註解推導，不是檔案 · 封存清單命中者在前、依封存日期升冪」、數字磚「來源變更」「最早 · <變更>」「最近 · <變更>」（分別為最早與最近 `@trace` updated 的日曆日與其 source）、「有來源的需求」（`有 @trace 的需求數 ／ 需求數`；全文無 `@trace` 時數字磚整列缺席），其後列出正式規格全文 `@trace` 來源變更名的去重清單，一列一個變更：封存清單中有對應者為可點列、副標為封存日期、依封存日期升冪在前，點擊 SHALL 開啟該封存變更的已封存詳情頁（規格詳情頁依「詳情頁互斥」關閉、宿主頁 SHALL 維持不變）；無對應者為不可點列、副標「無封存記錄」，排在所有可點列之後、依文件首次出現序。全文無 `@trace` 來源時節點徽章為 0 且卡內顯示「此規格沒有來源變更」。清單與狀態資料的欄位與值 SHALL 與對應 CLI `--json` 輸出一致；自檔案系統衍生的呈現層輔助欄位（如 spec 的最後修改時間、需求數、Purpose 摘要、溯源變更數）不屬此對齊範圍，SHALL NOT 出現在 CLI 輸出對照要求中。

#### Scenario: 顯示 change 清單與狀態

- **WHEN** app 於含多個 active change 的專案啟動
- **THEN** 每個 change 以其名稱與 proposal/tasks 狀態呈現，欄位與值對應 speclink list 與 speclink status 的 --json 輸出

#### Scenario: 選定 change 顯示其文件內容

- **WHEN** 使用者在清單中選定一個 change
- **THEN** app 顯示該 change 的 proposal 內容，並可切換檢視其 design 與 tasks（若存在）

#### Scenario: 選定 spec 以詳情頁顯示需求分段

- **WHEN** 使用者於規格頁點一列含 Purpose、三條需求（各帶兩個 Scenario 與 `@trace`）的規格
- **THEN** 主區換成三欄骨架的規格詳情頁：左欄身分區為等寬名稱＋複製鈕、「正式規格」籤、「3 條需求 · 6 個 scenario」與「最近更新 …」、「溯源」關聯列；「文件」節點為規格 3／溯源 2、「規格」作用中；閱讀卡依序為 Purpose 引言段與三個需求段，第一段展開（「Requirement」小標、標題、右端「來自 <變更> · 日期」錨點、SHALL 關鍵字等寬灰底、Scenario 左線縮排區塊）、第二與第三段收合為一列並帶「2 個 scenario」籤；規格頁清單無行內展開、宿主維持規格頁

#### Scenario: 需求目錄與搜尋需求

- **WHEN** 視窗寬 1600px 時使用者於規格詳情頁閱讀卡標題列的「搜尋需求」輸入只命中一條需求標題的字串
- **THEN** 右欄本頁目錄只列 Purpose 與該條需求，閱讀卡只呈現 Purpose 段與該需求段且該段展開；清空後全部還原（第一條展開、其餘收合）；視窗縮至 1100px 時本頁目錄缺席、閱讀卡照常

#### Scenario: 溯源分頁依封存日期排序

- **WHEN** 使用者開啟一份正式規格全文含三個 `@trace` 來源變更、且三者皆在封存清單中的規格詳情頁並點左欄的「溯源」節點
- **THEN** 節點徽章為 3，閱讀卡標題「溯源」帶籤 3，三列依封存日期升冪排列，每列主行為變更名、副標為封存日期

##### Example: 三個來源變更的排序

| 正式規格全文出現序 | 變更名 | 封存日期 | 呈現位置 |
| --- | --- | --- | --- |
| 1 | drawer-polish | 2026-08-04 | 第 2 列 |
| 2 | spec-archive-drawer | 2026-07-11 | 第 1 列 |
| 3 | desktop-archived-parity | 2026-08-11 | 第 3 列 |

#### Scenario: 點擊溯源列開啟已封存詳情頁且宿主不切頁

- **WHEN** 使用者於手冊頁經出處開啟規格詳情頁，再於「溯源」文件點一個可點列
- **THEN** 規格詳情頁關閉、該封存變更的已封存詳情頁開啟，宿主頁面維持手冊頁；按 Esc 回到手冊頁

#### Scenario: 無封存記錄的來源變更不可點

- **WHEN** 使用者開啟一份正式規格全文含兩個 `@trace` 來源變更、其中一個不在封存清單中的規格詳情頁並點「溯源」節點
- **THEN** 有對應者為第一列可點，無對應者排在其後、副標「無封存記錄」且不可點，點擊無任何效果

#### Scenario: 無溯源來源時溯源分頁空態

- **WHEN** 使用者開啟一份正式規格全文不含任何 `@trace` 註解的規格詳情頁
- **THEN** 身分區出身行無「最近更新」段且無「溯源」關聯列，「狀態」列無最近更新列；「規格」文件的出處行無最近更新段、數字磚無「最近更新」格、各需求段無錨點，note 提示框「N 條需求沒有來源變更」列出全部需求；「溯源」節點徽章為 0、文件無數字磚且顯示「此規格沒有來源變更」

#### Scenario: 需求段之外的內容照樣呈現

- **WHEN** 使用者開啟一份規格詳情頁，其 `## Requirements` 下有一段引言、最後一條需求之後另有 `## 附錄` 區段
- **THEN** 「規格」文件依序為 Purpose 段、引言、需求段、附錄區段，引言與附錄的內文都可見，「附錄」列入本頁目錄

#### Scenario: 無需求標題的規格整篇退回

- **WHEN** 使用者開啟一份不含任何 `### Requirement:` 標題的規格
- **THEN** 「規格」文件整篇以單一 markdown 檢視呈現、無數字磚、無提示框、右欄無本頁目錄、無錯誤訊息

#### Scenario: 複製名稱鈕

- **WHEN** 使用者於規格詳情頁點擊左欄身分區標題旁的複製名稱鈕
- **THEN** capability 名寫入剪貼簿並短暫顯示已複製回饋

#### Scenario: 規格文件的出處行、數字磚與狀態列

- **WHEN** 使用者開啟 tray-status-menu（15 條需求、74 個 scenario、9 個 `@trace` 來源變更、最近更新 2026-10-12 來自 tray-panel-restyle）的規格詳情頁
- **THEN** 「規格」文件出處行為 `speclink/specs/tray-status-menu/spec.md · 正式規格 · 最近更新 2026-10-12（tray-panel-restyle）`；數字磚依序為 15「需求」、74「Scenario」、9「溯源變更」、2026-10-12「最近更新 · tray-panel-restyle」；左欄「狀態」列為需求 15、溯源 9、最近更新 2026-10-12；無提示框

#### Scenario: 溯源關聯列切到溯源文件

- **WHEN** 使用者於上述規格詳情頁點左欄「溯源」關聯列的首顆變更籤或「+8」籤
- **THEN** 「溯源」節點作用中、閱讀卡換成溯源文件，出處行為「自全文的 @trace 註解推導，不是檔案 · 封存清單命中者在前、依封存日期升冪」，數字磚為 9「來源變更」、最早與最近的日曆日各附變更名、「15 ／ 15」「有來源的需求」

#### Scenario: Purpose 待補提示框

- **WHEN** 使用者開啟一份 Purpose 段內文以「TBD - created by archiving」起頭的規格詳情頁
- **THEN** 「規格」文件的數字磚之下有 warning 提示框「Purpose 待補」，內文含該規格 spec.md 的實際相對路徑（預設 spec 目錄下為 `speclink/specs/<capability>/spec.md`；remote 無本機 checkout 時不列路徑、改指引直接編輯這份規格檔）並指引直接編輯 Purpose 段；Purpose 引言段仍呈現佔位原文

#### Scenario: 缺來源變更的需求提示框

- **WHEN** 使用者開啟一份 15 條需求中「選單內容與看板同源」「分區溢出摺疊」兩條沒有 `@trace` 的規格詳情頁
- **THEN** note 提示框標題「2 條需求沒有來源變更」、內文列出兩個標題；兩條需求段無錨點；溯源文件的「有來源的需求」磚為「13 ／ 15」

#### Scenario: 收合的需求段展開

- **WHEN** 使用者於規格詳情頁點第二條（收合中）需求段的列
- **THEN** 該段展開呈現需求文與 Scenario 區塊、列的 `aria-expanded` 為 true，第一段維持展開；再點一次收合；右欄本頁目錄的錨點數不變

### Requirement: 外部變更即時反映

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

桌面 app SHALL 監看目前專案的 speclink/ 目錄樹：app 之外的寫者（CLI、agent、手動編輯器）修改其下任何文件後，看板、詳情頁、討論詳情頁與已封存頁 SHALL 在短時間內（秒級）自動更新呈現；已開啟檢視中「已載入的內容」——任務清單勾選狀態、proposal／design／specs 文件原文、meta 開工歸屬、討論記錄各分頁——SHALL 重載至磁碟現況，SHALL NOT 要求重啟、重開詳情頁或任何 app 內操作。使用者互動進行中（任務勾選寫回、拖曳排序）時外部觸發的內容重載 SHALL 讓路，互動結束後 SHALL 補一次重載至磁碟現況——SHALL NOT 打斷或蓋掉進行中的操作；重載回應交錯時 SHALL 以最新一次為準，較舊回應 SHALL NOT 覆蓋較新內容。app 內操作（勾任務、拖曳、動詞）後 SHALL 重載受影響的已載入內容——含任務清單與 meta。監看不可用時（如檔案系統權限）app SHALL 照常提供其餘功能——僅失去自動刷新，SHALL NOT 崩潰或反覆彈出錯誤。

#### Scenario: 外部勾選任務後看板自動更新

- **WHEN** 桌面 app 執行中，於外部終端執行 speclink task done 勾掉某 change 的一項任務
- **THEN** 數秒內該 change 的看板卡片任務數與進度條更新；詳情頁若開啟，標頭計數與任務清單中該項的核取方塊皆同步至磁碟狀態，全程無任何 app 內操作

#### Scenario: 外部蓋開工章後詳情頁出現開工歸屬

- **WHEN** 某 change 的詳情頁開啟中，於外部終端執行 speclink in-progress add 該 change
- **THEN** 數秒內詳情頁出現開工者與開工日，無需重開詳情頁

#### Scenario: 外部推進討論後頁內容更新

- **WHEN** 某討論的詳情頁開啟中，於外部終端執行 speclink discuss add-round 該討論
- **THEN** 數秒內詳情頁的回合分頁出現新回合內容，標頭回合數與其一致

#### Scenario: 互動進行中外部重載讓路

- **WHEN** 使用者正在拖曳某 change 的任務（尚未放開），外部寫者同時修改該 change 的文件
- **THEN** 拖曳互動不被打斷、拖曳視覺不重置；放開完成後數秒內，頁內容重載至磁碟現況

#### Scenario: 外部新增與歸檔反映到清單

- **WHEN** 於外部以 CLI 建立新 change，隨後將另一 change 歸檔
- **THEN** 數秒內看板出現新 change 卡片，被歸檔者自看板消失並出現於已封存頁

#### Scenario: 監看不可用時功能照常

- **WHEN** 檔案監看因環境因素無法建立
- **THEN** app 啟動與所有查詢、操作照常運作，錯誤僅記錄於日誌，畫面無錯誤彈窗堆疊

### Requirement: 討論於看板第 0 欄兩級呈現

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

看板 SHALL 於最左新增「討論」欄，欄內分上下兩區同屏呈現：上區為討論中（active）全卡清單，下區為欄底的「已轉出」常駐收合列；SHALL NOT 以互斥檢視切換兩者。分區判準 SHALL 為「引擎會自動收走的討論才收合」：已轉出（promoted）、已有結論（concluded 為 true）且未保留（hold 非 true）的討論收進欄底收合列；已轉出但尚無結論（concluded 為 false）的討論 SHALL 留在上區為全尺寸卡；已轉出、已有結論但保留在途（hold 為 true）的討論 SHALL 同樣留在上區為全尺寸卡。討論清單資料的 concluded 欄位缺席（向後相容路徑）時 SHALL 退回既有判準——promoted 一律收進收合列，SHALL NOT 把缺席當成尚無結論；hold 欄位缺席時 SHALL 視同未保留，SHALL NOT 把缺席當成保留。看板拖排落點清單 SHALL 與上區卡集共用同一判準（上區卡集與可拖集恆等）。

討論中區 SHALL 顯示 status 為 open 或 concluded 的討論、已轉出但尚無結論的討論、以及已轉出且保留在途的討論為全尺寸卡——卡 SHALL 以 slug（檔名）為標題（等寬字型）、topic 為卡身描述；複製 slug 鈕 SHALL 行內尾隨於標題最後一個字元後（版面規則見「看板卡片統一解剖學」）；建立者（createdBy，缺席時省略）SHALL 以頭像圓點呈現且 hover 顯示全名；輪數與建立時間 SHALL 並排於卡底 meta 列，狀態 chip 保留；已轉出但尚無結論的卡 SHALL 帶「已轉出・尚無結論」狀態標；已轉出且保留在途的卡 SHALL 帶「已轉出・保留中」狀態標（英文介面為「Promoted · on hold」），與「已轉出・尚無結論」同為 warning 語意色；promoted 且 concluded 為 false 的卡不論 hold 值 SHALL 標「已轉出・尚無結論」。open 卡為唯讀，concluded 卡 SHALL 提供「封存」動詞（「轉為變更」動詞已自 GUI 撤除，轉出改由 CLI 或 agent）；已轉出但尚無結論的卡與已轉出且保留在途的卡 SHALL NOT 提供任何動詞按鈕（前者結論未寫；後者的收尾由使用者以 CLI 的 discuss archive 明示解除）。上區全卡 SHALL NOT 列出衍生變更清單（衍生進度於討論詳情頁檢視）。

當存在至少一筆已有結論且未保留的 promoted 討論時，欄底 SHALL 呈現「已轉出 N ›」收合列（白底、上緣細線、右端箭頭依「收合時一律 ›、展開時箭頭指向展開出來的內容」：收合時為 ›，展開時朝上、指向展開於該列上方的 promoted 細列，SHALL NOT 朝下；N 為已有結論且未保留的 promoted 計數，預設收合）；點按 SHALL 就地展開 promoted 細列清單（白底細框列），再點按即收合；桌面 app SHALL 把展開狀態存於 app 本機並跨啟動保留，SHALL NOT 寫入任何專案目錄。無任何已有結論且未保留的 promoted 討論時收合列 SHALL 缺席。

promoted 細列 SHALL 以討論 slug 為首行（等寬字型）且帶複製 slug 鈕、topic 為次行描述，其下每個 promoted_to 子變更 SHALL 以樹狀前綴（末列 └、其餘 ├）逐列列出名稱與階段 chip。子變更的階段 SHALL 由其於清單中的存在性派生：active 清單命中依看板欄位規則、封存清單命中為已封存、兩者皆無 SHALL 標示為已刪除且討論維持已轉出不回退。階段 chip SHALL 以看板階段配色呈現：提案中、進行中、已就緒各對應該階段欄的 teal 濃度，已封存為中性色，已刪除為 destructive 色加刪除線。

slug 為題與複製鈕（討論全卡標題、promoted 細列首行）屬 speclink/LANGUAGE.md 明載的受控例外。討論欄的計數徽章 SHALL 顯示上區全卡數（open、concluded、已轉出但尚無結論、已轉出且保留在途）。當討論欄上區無任何全卡但存在已有結論且未保留的 promoted 討論時，SHALL NOT 顯示「尚無討論」空狀態（欄底收合列已傳達）。封存的討論 SHALL NOT 出現於此欄。輪數文案 SHALL 使用「N 輪」。

#### Scenario: 進行中與已結論討論的全卡呈現

- **WHEN** speclink/discussions/ 下存在一筆 status: open（3 輪、frontmatter 含 created_by 與 created）與一筆 status: concluded 的討論、無任何 promoted 討論
- **THEN** 討論欄顯示兩張全卡，各以 slug 為標題（等寬字型）、topic 為描述，複製 slug 鈕行內尾隨於標題末字元後，建立者呈頭像圓點且 hover 顯示全名、卡面無全名直出文字，卡底 meta 列並排「3 輪」與建立時間；open 卡無動詞按鈕，concluded 卡帶「封存」按鈕且無「轉為變更」按鈕；欄計數徽章顯示 2，欄底無「已轉出」收合列

#### Scenario: 未結論的已轉出討論留在上區並帶標示

- **WHEN** 討論欄存在一筆已轉出但尚無結論（promoted 且 concluded 為 false）的討論與一筆已轉出且已有結論且無 hold 的討論
- **THEN** 前者呈上區全尺寸卡、帶「已轉出・尚無結論」狀態標、無任何動詞按鈕；後者收進欄底「已轉出 1」收合列；欄計數徽章顯示 1（上區卡數）

#### Scenario: 保留中的已轉出討論留在上區並帶標示

- **WHEN** 討論欄存在一筆已轉出、已有結論且 hold 為 true 的討論（promoted_to 含兩個已封存變更與一個進行中變更），與一筆已轉出、已有結論且 hold 為 false 的討論
- **THEN** 前者呈上區全尺寸卡、帶「已轉出・保留中」狀態標、無任何動詞按鈕、卡面不列衍生變更清單；後者收進欄底「已轉出 1」收合列；欄計數徽章顯示 1（上區卡數）；拖排落點清單含前者不含後者

#### Scenario: concluded 資料缺席時退回既有分區

- **WHEN** 討論清單資料無 concluded 欄位（向後相容路徑），存在一筆 promoted 討論
- **THEN** 該討論收進欄底「已轉出」收合列（既有行為），上區不出現「已轉出・尚無結論」標示

#### Scenario: hold 資料缺席時退回既有分區

- **WHEN** 討論清單資料無 hold 欄位（舊 server），存在一筆已轉出且已有結論的討論
- **THEN** 該討論收進欄底「已轉出」收合列，上區不出現「已轉出・保留中」標示

#### Scenario: 複製討論 slug

- **WHEN** 點討論全卡或已轉出細列的複製鈕
- **THEN** 該討論的 slug 寫入剪貼簿並短暫顯示已複製回饋，且不開啟討論詳情頁

#### Scenario: 欄底收合列就地展開與收合

- **WHEN** 討論欄存在一筆 active 討論與一筆已有結論且無 hold 的 promoted 討論
- **THEN** 上區顯示 active 全卡、欄底呈現「已轉出 1」收合列（預設收合）；點按收合列就地展開 promoted 細列（active 全卡維持可見），再點按即收合；欄計數徽章全程顯示 1（上區卡數）；展開後重啟桌面 app，收合列仍為展開

#### Scenario: 無已轉出討論時收合列缺席

- **WHEN** 討論欄無任何已有結論且無 hold 的 promoted 討論
- **THEN** 欄底不顯示「已轉出」收合列

#### Scenario: 僅有已轉出討論時討論中區不顯空狀態

- **WHEN** 討論欄上區無任何全卡、但存在至少一筆已有結論且無 hold 的 promoted 討論
- **THEN** 討論中區不顯示「尚無討論」文案，欄底收合列傳達存在已轉出討論

#### Scenario: 已轉出細列的 slug 首行與子變更樹配色

- **WHEN** 展開欄底收合列，一筆已有結論且無 hold 的 promoted 討論的 promoted_to 含一個在 active 清單（提案中）與一個已在封存清單的子變更
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

#### Scenario: 欄底收合列箭頭指向展開內容

- **WHEN** 看板討論欄底部有「已轉出 2 ›」收合列，使用者點按展開、再點按收合
- **THEN** 展開後 2 筆 promoted 細列出現在收合列上方、箭頭朝上；收合後細列消失、箭頭恢復 ›

### Requirement: GUI 勾任務與 CLI 完成語意一致

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

桌面看板勾選任務為完成 SHALL 產生與 speclink task done 相同的檔案效果：tasks.md 該任務勾章、touched-files 記錄（有未被先前任務認領的 git dirty 檔時追加，無則不追加——與 CLI 同語意）、該 change 首次有任務完成且 meta 無 started_* 時蓋開工章（started_at 為當日、started_by 依 git 身分可得性、started_with 缺席）。對已完成任務的重複完成請求 SHALL 視為冪等成功，SHALL NOT 寫入任何檔案、SHALL NOT 對使用者報錯。取消勾選與拖曳排序 SHALL 僅寫 tasks.md，SHALL NOT 寫入 meta 或 touched 記錄。

#### Scenario: 勾選首任務蓋章並移欄

- **WHEN** 使用者於看板勾選某 meta 無 started_* 的 change 的第一項任務
- **THEN** tasks.md 該任務成 [x]，.meta.yaml 新增 started_at（git 身分可得時含 started_by），看板刷新後卡片移入「進行中」欄且詳情頁顯示開工列

#### Scenario: 取消勾選不動 meta 與 touched

- **WHEN** 使用者取消勾選一項已完成任務
- **THEN** 僅 tasks.md 該行標記變為 [ ]；.meta.yaml 與 touched 記錄逐字元不變

#### Scenario: 拖曳排序不觸發完成語意

- **WHEN** 使用者拖曳任務改變順序（含跨群組重編號）
- **THEN** 僅 tasks.md 變動；.meta.yaml 與 touched 記錄逐字元不變

### Requirement: 側欄導覽結構

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

桌面 app 的左側 SHALL 分兩層:全域的圖示列(約 56px)與每專案的專案欄(固定 200px,SHALL NOT 可收合)。圖示列自左側標題列下方起由上而下 SHALL 為品牌標記、每個開著的專案一個方塊、「新增專案」的「＋」、底部的「設定」;專案欄自左側標題列下方起由上而下 SHALL 為變更、已封存、規格、手冊,底部 SHALL 固定為專案設定,中間為彈性空間。「設定」SHALL NOT 出現在專案欄,其入口為圖示列底部的齒輪。專案欄 SHALL NOT 含備忘項、SHALL NOT 呈現常駐 app 版號文字;app 版號的呈現位置 SHALL 為應用程式設定的更新頁版本卡與關於頁品牌卡。已封存導覽項 SHALL 帶封存變更數量徽章,數字 SHALL 隨封存清單變動即時更新;其無障礙標籤 SHALL 為「已封存」。手冊導覽項的無障礙標籤 SHALL 為「手冊」,點擊進入手冊頁(行為見 desktop-manual-page)。導覽項點擊 SHALL 為單純切頁——切至對應頁並以視覺高亮(主色淡底、主色字)標示現行項,SHALL NOT 具再次點擊返回看板的 toggle 行為。左側頂端 48px SHALL 為一條連續的標題列(圖示列與專案欄共用、同側欄底色、48px 內無直向分隔線,下緣 SHALL 有與主區頂列同高的橫向細線,使頂列橫貫整個視窗、圖示列分隔線自此線往下),承載 macOS 紅綠燈的留白、作用中專案的純文字名稱(左緣 SHALL 與下方專案欄導覽項文字左緣對齊;hover 顯示完整路徑)與右端的「⋯」專案動作鈕;設定模式時左端 SHALL 為「← 設定」返回鈕且無「⋯」;專案設定模式時左端 SHALL 為「← 專案設定」返回鈕、右端為等寬小字的專案名;兩個返回鈕點擊皆 SHALL 回到進入設定前所在的頁面(兩種設定模式之間切換不改變此頁;零專案時為空狀態);零專案且非設定模式時左側標題列 SHALL 留白(只剩圖示列寬,放不下文字)。

設定(齒輪)SHALL 開啟應用程式設定且 SHALL NOT 依賴任何已開啟的專案——零專案時點擊 SHALL 照常進入且頁內功能可操作;設定模式下齒輪 SHALL 為作用中,專案欄 SHALL 換成設定分類欄(200px;頂部全圓搜尋框,以分類名與各頁列標籤做大小寫不敏感比對過濾,無命中時顯示「沒有符合的設定」;其下群組「應用程式」:一般、更新、CLI、伺服器,再下群組「關於」:關於 Speclink;群組內沒有命中的分類時連群組標題一併隱藏),左側因此恆為圖示列加分類欄的寬度(零專案亦然),主區麵包屑為「設定 / 分類名」(「設定」可點回一般頁)。設定模式與專案設定模式下,Esc、⌘[、⌘←(非 macOS 為 Alt+←)、滑鼠側鍵「上一頁」與 macOS 上觸控板兩指往返回方向(手指向右)的水平滑動 SHALL 等效於左側標題列的返回鈕;快捷鍵的不作用條件(焦點在文字輸入、按鍵已被浮層處理)與觸控板滑動返回的平台限制、返回指示、門檻、結算時機與不觸發條件同需求「詳情頁的導覽與返回」。專案設定有未儲存的修改時,返回 SHALL 先詢問是否捨棄,確認後才離開;取消時 SHALL 留在原頁且修改保留。有對話框開著時,所有返回操作 SHALL NOT 作用,對話框 SHALL NOT 被連帶收掉。詳情頁、設定模式與專案設定模式以外的頁面沒有返回去處,上述操作 SHALL NOT 作用。專案設定項 SHALL 進入專案設定模式:專案欄換成專案設定分類欄(群組「speclink/config.yaml」:專案說明、產出規則、審查規則、產出流程;群組「專案」:.speclink.yaml,遠端專案無此組;解析失敗的檔所掛分類項帶警示點),主區麵包屑為「專案名 / 專案設定 / 分類名」(前兩段可點:專案名回看板、專案設定回專案說明頁)。兩種分類欄 SHALL 記住上次所在分類。零專案時 SHALL NOT 渲染專案欄,圖示列 SHALL 只有品牌標記、「＋」與設定。有專案且非設定模式時專案欄五個導覽項 SHALL 恆常渲染,SHALL NOT 依頁面狀態顯隱。

#### Scenario: 專案欄順序與設定在圖示列底部

- **WHEN** 開啟任一專案進入桌面 app
- **THEN** 圖示列由上而下為品牌標記、專案方塊、「＋」、底部齒輪;專案欄頂部由上而下依序為變更、已封存、規格、手冊,底部為專案設定(兩者之間為彈性空白);專案欄不存在「設定」與備忘項;左側標題列顯示作用中專案名與「⋯」

#### Scenario: 側欄無常駐版號

- **WHEN** 開啟任一專案檢視圖示列與專案欄(含專案設定下方區域)
- **THEN** 任何位置不存在版號文字;應用程式設定的更新頁版本卡與關於頁品牌卡顯示目前版本

#### Scenario: 零分頁仍可進入應用程式設定

- **WHEN** app 於零專案狀態點擊圖示列底部的齒輪
- **THEN** 進入應用程式設定,左側為圖示列加設定分類欄、麵包屑為「設定 / 一般」、齒輪為作用中;一般頁的顯示語言列與伺服器頁的連線操作皆可正常使用

#### Scenario: 零分頁點專案設定見空狀態

- **WHEN** app 於零專案狀態
- **THEN** 專案欄不存在,專案設定項不可達;主區為「開啟一個專案開始」空狀態

#### Scenario: 已封存徽章隨封存動作更新

- **WHEN** 看板上一個已就緒的變更被封存
- **THEN** 已封存導覽項的徽章數量在數秒內增加一、無需重啟或手動重新整理

#### Scenario: 已封存導覽項為切頁而非 toggle

- **WHEN** 使用者位於已封存頁並再次點擊已封存導覽項
- **THEN** 畫面停留在已封存頁;隨後點擊變更導覽項才返回看板,且變更項高亮、已封存項恢復未選取樣式

#### Scenario: 手冊導覽項切頁與零分頁空狀態

- **WHEN** 使用者點擊專案欄「手冊」項
- **THEN** 切至手冊頁且該項高亮、其他項恢復未選取樣式,麵包屑顯示「專案名 / 手冊」

#### Scenario: 零專案時只剩圖示列

- **WHEN** 使用者關閉最後一個專案
- **THEN** 專案欄消失、左側只剩圖示列(品牌標記、「＋」、齒輪),左側標題列與主區頂列只剩留白,主區呈現空狀態;此時點齒輪,左側擴為圖示列加設定分類欄

#### Scenario: 專案設定模式的分類欄與返回

- **WHEN** 使用者於 local 專案點專案欄「專案設定」,再點左側標題列的「← 專案設定」
- **THEN** 進入時專案欄換成專案設定分類欄(專案說明、產出規則、審查規則、產出流程;.speclink.yaml)、左側標題列為「← 專案設定」與等寬專案名、麵包屑「專案名 / 專案設定 / 專案說明」;點返回後回到看板,專案欄恢復五個導覽項

#### Scenario: 離開設定模式

- **WHEN** 使用者於規格頁點齒輪進入設定模式,再點左側標題列的「← 設定」,或按 Esc、⌘[、滑鼠側鍵「上一頁」,或於 macOS 以觸控板兩指向右滑過門檻後放開
- **THEN** 回到規格頁,專案欄恢復五個導覽項、齒輪恢復未作用;自手冊頁進專案設定模式再以同樣操作返回則回到手冊頁;零專案時回到空狀態,左側只剩圖示列;焦點在分類欄搜尋框時按 Esc 不離開設定模式

#### Scenario: 專案設定有未存修改時返回先確認

- **WHEN** 使用者於專案設定的專案說明頁編輯並改了內容但未儲存,接著按 Esc 或點「← 專案設定」
- **THEN** 出現「要捨棄未儲存的修改嗎？」確認框;選取消則留在專案說明頁且修改仍在,選捨棄則回到進入專案設定前的頁面;只開了編輯而未改動時直接返回、不詢問

#### Scenario: 設定分類欄搜尋

- **WHEN** 使用者於設定分類欄搜尋框輸入「外觀」
- **THEN** 分類欄只剩一般一項(外觀列只在一般頁),「關於」群組標題隱藏;輸入無命中字串時顯示「沒有符合的設定」;清空後兩群組共五項還原(一般、更新、CLI、伺服器;關於 Speclink)

#### Scenario: 非 macOS 的設定模式不接觸控板返回

- **WHEN** 使用者於 Windows 自規格頁點齒輪進入設定模式,以觸控板兩指向右滑動超過門檻後放開,接著按 Alt+←
- **THEN** 滑動後仍停在設定模式、左緣不出現「←」指示;按 Alt+← 後回到規格頁

### Requirement: raw HTML 不以原文呈現

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

桌面 app 渲染 markdown 內容時，來源中的 raw HTML（含 HTML 註解）SHALL NOT 以原始文字出現在渲染結果；渲染 SHALL NOT 修改任何來源檔案。

#### Scenario: 討論記錄的 scaffold 註解不顯示

- **WHEN** 開啟討論詳情頁的討論過程分頁，其來源在 Rounds 區段含 CLI scaffold 產生的 HTML 註解行
- **THEN** 渲染結果不含該註解的任何文字，speclink/ 下的討論記錄檔案內容位元不變

#### Scenario: 程式碼區塊內的 HTML 原文照常顯示

- **WHEN** 檢視在 code fence 內含 HTML 標籤範例的文件
- **THEN** code fence 內容以原文完整顯示（過濾僅及於 code fence 之外的 raw HTML）

### Requirement: 討論結論以欄位標籤呈現

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

桌面 app 的討論結論呈現（討論詳情頁的結論文件、已封存討論頁的結論文件）SHALL 將行首的 Decision／Rationale／Rejected alternatives／Deferred／Capture to／Next 粗體前綴切成六個欄位（前綴原文 SHALL NOT 出現在渲染結果；一個欄位 SHALL 涵蓋其標籤行起至下一個標籤行或結論結尾的全部內容；非六詞白名單的粗體前綴行 SHALL 按一般內文歸屬當前欄位），並以 16px 粗體的區段標題呈現各欄位（標籤依語系呈現，zh-TW 為決定／理由／否決替代案／擱置／記錄去向／下一步；每個區段標題 SHALL 為本頁目錄錨點）。來源缺席的欄位 SHALL NOT 渲染空區段；結論不含任何白名單欄位時 SHALL 整篇以單一 markdown 檢視退回，不報錯。渲染 SHALL NOT 修改任何來源檔案。

各欄位的畫法 SHALL 為：

- 決定：第一段散文為定論句，SHALL 以大於內文的字級呈現；其後的頂層列點為定案細節，SHALL 以細線列呈現且預設只展開前 4 條，其餘以「展開其餘 N 項定案細節」鈕一次展開；區段標題右端 SHALL 顯示細節總數。
- 分刀：決定的列點中，首行以粗體開頭、粗體內（去掉反引號片段後）以 `cut <編號>`、「刀」或「第 N 刀」起頭的列 SHALL 自細節列抽出，成為「分刀」區段（有任一刀時才出現、位於決定之後）；每刀一張卡，卡頭為刀的識別籤（粗體內去掉反引號片段後開頭的 `cut <編號>`、「刀…」或「第 N 刀」）、範圍句與該刀的變更數。範圍句：粗體內在識別籤之後還有文字時為該文字，否則為粗體之後（去掉開頭冒號）、第一個「——」之前的文字；首行其餘的文字（範圍句取自粗體時為粗體之後的全部文字、外圍整組括號拿掉，否則為「——」之後的文字）SHALL 為卡內第一條細線列，不丟內容。粗體內的反引號片段為該刀的變更名籤，粗體內沒有反引號片段時取同一行粗體之後的反引號片段（含空白的片段不算）——名稱在 `promoted_to` 中時籤 SHALL 附階段籤（階段派生與配色同「討論於看板第 0 欄兩級呈現」）且點擊 SHALL 開啟該變更的詳情頁（已封存者開啟已封存詳情頁），不在 `promoted_to` 中時為不可點籤；列的巢狀子項 SHALL 接在其後為卡內的細線列。刀是否轉出 SHALL 只依該刀粗體內的變更名籤是否在 `promoted_to` 判定（變更名只寫在粗體之後的刀列，反引號片段可能是識別字或檔名，視同沒有變更名籤）：刀至少有一個變更名籤且全部不在 `promoted_to` 時為「整刀未轉出」，卡 SHALL 以橄欖色（手動任務語意色）虛線外框與淡橄欖底呈現，卡頭的識別籤之後 SHALL 有橄欖色「尚未轉出」籤；變更名籤部分在、部分不在 `promoted_to` 時，每個不在其中的變更名籤之後 SHALL 有同款「尚未轉出」籤，卡的外框照舊；刀沒有任何變更名籤時 SHALL NOT 有任何轉出標示。
- 理由：照 markdown 呈現。
- 否決替代案與擱置：內文有列點時每條列點一列、否則以「；」切列；每列以第一個分隔號（「——」或前後帶空白的「 — 」，取先出現者；discuss 技能檔的寫法為「方案 — 理由」）拆成「方案」與「落敗理由」兩行，切不到時整條為方案；擱置的每列 SHALL 帶時鐘圖示；兩個區段標題右端 SHALL 顯示列數。
- 記錄去向：內文以「、」或逗號切成籤；籤文字為正式規格名時 SHALL 為可點籤，點擊開啟該規格的規格詳情頁；其餘為不可點籤；切不出籤時照 markdown 呈現。
- 下一步：照 markdown 呈現，其後 SHALL 另列內文中每個 `/speclink-*` 指令（含其旗標與參數）的等寬籤，每顆附複製鈕，點擊 SHALL 把指令原文寫入剪貼簿。

結論文件的閱讀卡標題下 SHALL 有數字磚：「定案細節」（細節數）、「分刀」（刀數，標籤附變更數；無分刀時缺席）、「否決的替代方案」（列數；欄位缺席時缺席）、「擱置」（列數；欄位缺席時缺席）。討論詳情頁的結論文件有任一刀整刀未轉出時，數字磚之下 SHALL 有橄欖色（手動任務語意色）提示框「N 刀尚未轉出」（N 為整刀未轉出的刀數）：每刀一列，呈現識別籤、範圍句與「前往」鈕，點擊 SHALL 把該刀卡捲到閱讀區可見處；提示框末行 SHALL 說明這幾刀還沒有變更，並附一顆可複製的等寬指令籤 `/speclink-propose --from-discussion <slug>`（樣式與「下一步」的指令籤相同）。沒有整刀未轉出的刀時 SHALL NOT 有提示框；已封存討論頁的結論文件 SHALL NOT 有提示框（刀卡的轉出標示照常）。結論文件 SHALL NOT 有其他提示框。

#### Scenario: 標準結論欄位成標籤區塊

- **WHEN** 開啟含 scaffold 格式結論（Decision、Rationale、Capture to、Next 各佔一行起頭）的結論文件
- **THEN** 四個欄位以 16px 粗體區段標題呈現並列入本頁目錄，「**Decision**:」等粗體前綴原文不出現在渲染結果

#### Scenario: 結論缺席欄位不渲染空標籤

- **WHEN** 某結論來源只有 Decision 與 Rationale 行，無 Deferred 行
- **THEN** 結論文件呈現決定與理由兩個區段，無擱置區段，數字磚無「擱置」格

#### Scenario: 自由格式結論整篇退回

- **WHEN** 結論來源為手寫自由段落、不含任何白名單粗體前綴行
- **THEN** 結論文件整篇以單一 markdown 檢視呈現，無區段標題、無數字磚、無錯誤訊息，來源檔案位元不變

#### Scenario: 決定主句放大與細節收合

- **WHEN** 開啟一份 Decision 為「一句定論＋27 條頂層列點（其中 7 條為分刀列）」的結論文件
- **THEN** 定論句以大於內文的字級呈現；其下細線列只列前 4 條非分刀細節，鈕文字為「展開其餘 16 項定案細節」，點擊後 20 條全部可見且鈕消失；「決定」區段標題右端計數為 20

#### Scenario: 分刀卡與變更籤

- **WHEN** 開啟 detail-content-redesign 的結論文件，其 Decision 列點含「- **cut 1 `desktop-detail-doc-tree`**: 骨架——……」等三條分刀列，且 `promoted_to` 含前兩刀的變更而第三刀尚未轉出
- **THEN** 「分刀」區段有三張卡，卡頭依序為 cut 1、cut 2、cut 3 的識別籤與範圍句（「骨架」等，「——」之後的文字不在卡頭）；前兩張卡的變更籤附「已封存」階段籤且點擊開啟該變更的已封存詳情頁，第三張的變更籤不可點且無階段籤，該卡為橄欖色虛線外框、卡頭識別籤之後有「尚未轉出」籤；前兩張卡為一般外框、無「尚未轉出」籤；每張卡內列出該列的巢狀子項；數字磚「分刀」為 3 且標籤含「3 個變更」

##### Example: 分刀列的解析

| 列首粗體 | 識別籤 | 變更名籤 | 範圍句 |
| --- | --- | --- | --- |
| `**cut 1 \`desktop-detail-doc-tree\`**: 骨架——三種詳情頁改三欄` | cut 1 | desktop-detail-doc-tree | 骨架 |
| `**cut 2 \`desktop-shell-redesign\` \`desktop-notice-relocation\`**: 外殼 7 項` | cut 2 | desktop-shell-redesign、desktop-notice-relocation | 外殼 7 項 |
| `**刀三**：系統匣面板` | 刀三 | （無） | 系統匣面板 |
| `**cut 2 外殼**（propose 期拆兩個 change：\`desktop-shell-redesign\` 外殼結構、\`desktop-native-menu\` 原生選單）` | cut 2 | desktop-shell-redesign、desktop-native-menu | 外殼（卡內第一條細線列為括號內的說明） |
| `**範圍與驗收**：只改 UI 渲染` | （不是分刀，留在定案細節） | — | — |

#### Scenario: 否決與擱置拆方案與理由

- **WHEN** 開啟一份 Rejected alternatives 有 14 條「方案 — 理由」列點、Deferred 有 5 條（其中一條沒有分隔號「——」或「 — 」）的結論文件
- **THEN** 「否決替代案」區段計數 14、每列方案一行、理由灰字一行；「擱置」區段計數 5、每列帶時鐘圖示，沒有分隔號的那列只有方案行

#### Scenario: 記錄去向籤與下一步指令籤

- **WHEN** 開啟一份 Capture to 為「proposal、design、spec（desktop-app MODIFIED）、tasks」且 Next 含「/speclink-propose --from-discussion detail-content-redesign」的結論文件，正式規格存在 desktop-app
- **THEN** 「記錄去向」呈現四顆籤，點 desktop-app 籤開啟該規格的規格詳情頁，其餘籤不可點；「下一步」的 markdown 之後有一顆等寬指令籤「/speclink-propose --from-discussion detail-content-redesign」，點複製鈕後剪貼簿為該指令原文

#### Scenario: 結論文件的數字磚與出處行

- **WHEN** 開啟一份 11 輪、Decision 有 20 條細節與 3 刀、Rejected alternatives 有 11 條、Deferred 有 4 條、三刀的變更皆在 `promoted_to` 的 live 討論結論文件
- **THEN** 閱讀卡出處行為 `speclink/discussions/<slug>.md · 11 輪之後寫入`；數字磚依序為 20「定案細節」、3「分刀 · 3 個變更」、11「否決的替代方案」、4「擱置」；無提示框

#### Scenario: 未轉出刀的提示框與前往

- **WHEN** 使用者於討論詳情頁開啟 wad-plugins-merge-or-split 的結論文件，其 Decision 有 cut 1 `add-team-plugin-library`（在 `promoted_to`、進行中）與 cut 2 `remove-builtin-wad-work`（不在 `promoted_to`）兩條分刀列
- **THEN** 數字磚之下有橄欖色提示框，標題「1 刀尚未轉出」，恰一列為 cut 2 的識別籤、範圍句與「前往」鈕；點「前往」後 cut 2 的刀卡捲到閱讀區可見處；提示框末行有指令籤「/speclink-propose --from-discussion wad-plugins-merge-or-split」，點其複製鈕後剪貼簿為該指令原文；cut 2 卡為橄欖色虛線外框且卡頭有「尚未轉出」籤，cut 1 卡為一般外框、變更名籤附「進行中」階段籤

##### Example: 刀的轉出判定

| 刀列粗體內的變更名籤 | 在 `promoted_to` 的 | 判定 | 刀卡 | 提示框列出 |
| --- | --- | --- | --- | --- |
| `a`、`b` | `a`、`b` | 已轉出 | 一般外框、各附階段籤 | 否 |
| `a`、`b` | `a` | 部分轉出 | 一般外框、`b` 之後有「尚未轉出」籤 | 否 |
| `a` | （無） | 整刀未轉出 | 橄欖色虛線外框、卡頭「尚未轉出」籤 | 是 |
| （無變更名籤） | — | 無從判定 | 一般外框、無轉出標示 | 否 |
| （粗體內沒有；粗體之後有 `KeyLines`、`util.rs`） | （無） | 無從判定 | 一般外框、無轉出標示 | 否 |

#### Scenario: 沒有整刀未轉出的刀時無提示框

- **WHEN** 開啟一份分刀全為「已轉出」「部分轉出」或「無變更名籤」的 live 討論結論文件
- **THEN** 結論文件沒有提示框；部分轉出的刀只在未轉出的變更名籤之後有「尚未轉出」籤

#### Scenario: 已封存討論頁不出未轉出提示框

- **WHEN** 使用者於已封存頁開啟一筆已封存討論的結論文件，其中一刀的變更名不在 `promoted_to`
- **THEN** 該刀卡為橄欖色虛線外框且卡頭有「尚未轉出」籤；結論文件沒有提示框

### Requirement: 桌面刪除變更走 discard 全語意

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

桌面 app 的本地刪除變更 SHALL 執行引擎 discard 全語意(不帶強制):開工痕跡守門、來源討論解鏈與 touched 紀錄清理——SHALL NOT 直接刪除目錄繞過任一環節。刪除一個由討論轉出的 change 後,來源討論的已轉出清單 SHALL 同步移除該 change 名,清單空時討論狀態 SHALL 回復。對有開工痕跡(meta 含 started_at 或任一任務已勾)的 change,刪除 SHALL 被引擎拒絕且無任何檔案改動。remote session 的刪除 SHALL 以不帶強制的語意呼叫 server 刪除端點,已開工 change 的刪除由 server 拒絕,依既有失敗 toast 語意呈現。

#### Scenario: 刪除轉出 change 連帶解鏈來源討論

- **WHEN** 使用者刪除一個由討論轉出、無開工痕跡的提案中 change(該討論僅轉出此一 change)
- **THEN** change 目錄消失、其 touched 紀錄清除;來源討論的已轉出清單移除該名稱且狀態回復,看板重載後討論卡如實呈現

#### Scenario: 有開工痕跡的本地刪除被拒

- **WHEN** 使用者對 meta 含 started_at 的 change 觸發刪除並確認(UI 停用未及反應的併發情境)
- **THEN** 引擎拒絕,speclink/ 下任何檔案逐位元不變,app 依既有失敗 toast 語意呈現拒絕訊息

#### Scenario: remote 刪除已開工 change 被 server 拒絕

- **WHEN** 使用者於 remote 分頁對已勾任務的 change 觸發刪除並確認
- **THEN** server 以需要強制的拒絕語意回應,scope 內容零改動,app 依既有失敗 toast 語意呈現

### Requirement: 指令檔過期提示

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

桌面 app SHALL 於本地專案成為作用中時執行指令檔過期探測,並於更新動作完成後與 workspace-changed 事件到達時重新探測。探測回報過期、缺失或較新、且該專案未略過當前產物層版號時,SHALL 於專案欄底部、專案設定項上方呈現一張琥珀淡底的提示卡(琥珀圖示、一句標題、一句說明、右端 ›);提示卡 SHALL 於該專案的所有頁面(變更、已封存、規格、手冊、專案設定)常駐,SHALL NOT 佔用主區、SHALL NOT 以 modal 阻斷操作,SHALL NOT 於 remote 專案觸發探測。探測回報現版或無法判定時 SHALL 無任何提示;無法判定 SHALL NOT 記入略過、SHALL NOT 視同現版。其他開著的本地專案上次探測為過期、缺失或較新時,其圖示列方塊角落 SHALL 亮琥珀點,提示附「技能檔需要處理」;背景專案 SHALL NOT 主動探測。

點提示卡 SHALL 開啟確認框:琥珀圖示、標題、說明,一張細框卡三列(這個專案的技能檔版本、你的 Speclink 版本、會改寫的檔案數——較新態第三列為會被換回舊內容的檔案數),按鈕依探測狀態:過期為「稍後」「保留現狀」「更新技能檔」、缺失為「稍後」「保留現狀」「安裝技能檔」(從未安裝的專案不以「更新」稱之)、較新為「保留現狀」「更新 Speclink」。較新時 SHALL 以「app 本體是舊版、專案檔案較新」語意呈現,SHALL NOT 提供任何改寫專案檔案的動作,「更新 Speclink」SHALL 開啟應用程式設定的更新頁。「稍後」SHALL 只關閉確認框、提示卡維持。

「更新技能檔」與「安裝技能檔」SHALL 經引擎既有的指令檔再生入口整套再生受管檔,成功後提示卡與確認框消失;失敗時錯誤 SHALL 呈現於確認框內原位且可重試,SHALL NOT 阻斷專案其他功能。「保留現狀」SHALL 將該專案與當前產物層版號記入 app 本機持久化,SHALL NOT 寫入專案內任何檔案;同版不再提示(缺失、過期與較新共用同一略過記憶),產物層版號變動後 SHALL 重新提示。提示文案 SHALL 遵循 speclink/LANGUAGE.md 詞彙原則,SHALL NOT 出現工程詞。主區各視圖的高度 SHALL NOT 因提示存在而改變。

#### Scenario: 開啟過期專案出現提示並更新

- **WHEN** 以新版 desktop 開啟指令檔為舊版的本地專案,點專案欄底部的提示卡,於確認框點「更新技能檔」
- **THEN** 提示卡先於專案欄底部呈現(標題「技能檔是舊版」、說明含將被改寫的檔案數);確認框三列顯示專案技能檔版本、Speclink 版本與檔案數;更新後受管檔再生為現版、確認框與提示卡消失

#### Scenario: 檔案領先 app 時不提供改寫動作

- **WHEN** 以舊版 desktop(引擎產物層版號舊於專案指令檔標記版號)開啟該本地專案並點提示卡
- **THEN** 提示卡以「技能檔比 Speclink 新」呈現;確認框說明以「app 本體是舊版」語意引導更新 app,按鈕只有「保留現狀」與「更新 Speclink」,無「更新技能檔」「安裝技能檔」;「更新 Speclink」開設定的更新頁;專案內無任何檔案因此變動

#### Scenario: 較新提示保留現狀後同版不再問

- **WHEN** 使用者於較新態的確認框點「保留現狀」,其後關閉並重新開啟同一專案
- **THEN** 提示卡不再出現;app 升版使產物層版號變動後重新探測、依新狀態重新裁決

#### Scenario: 保留現狀同版不再問

- **WHEN** 使用者於過期態的確認框點「稍後」再點提示卡改點「保留現狀」,其後關閉並重新開啟同一專案
- **THEN** 「稍後」後提示卡仍在;「保留現狀」後提示卡消失且重開不再出現,專案內無任何檔案因此變動

#### Scenario: 產物層版號變動後重新提示

- **WHEN** 曾保留現狀的專案,在 desktop 升版且產物層版號變動後再次開啟
- **THEN** 提示卡重新出現

#### Scenario: 標記移除不提示

- **WHEN** 開啟指令檔已整塊移除 SPECLINK 標記的本地專案
- **THEN** 無任何提示卡

#### Scenario: 從未安裝的專案提示安裝

- **WHEN** 開啟 tools 清單宣告 claude 但 CLAUDE.md 不存在(如 clone 後指令檔未進版控)的本地專案,點提示卡並於確認框點「安裝技能檔」
- **THEN** 提示卡以「還沒安裝技能檔」呈現、說明含將被新建的檔案數;安裝後受管檔生成為現版、提示卡消失

#### Scenario: 更新失敗原位呈現錯誤

- **WHEN** 更新動作因受管檔無法寫入而失敗
- **THEN** 錯誤訊息呈現於確認框內容卡下方、動作鍵恢復可按可重試,專案其餘功能不受影響

#### Scenario: remote 分頁不觸發

- **WHEN** 開啟 remote 專案
- **THEN** 不執行過期探測、專案欄無提示卡

#### Scenario: 外部更新自然消提示

- **WHEN** 提示卡顯示期間,使用者於終端執行 speclink update 使受管檔更新為現版
- **THEN** workspace-changed 事件觸發重新探測後提示卡消失,無需重開專案

#### Scenario: 提示卡跨頁常駐且主區不扣高度

- **WHEN** 技能檔提示卡存在時依序切到規格頁、手冊頁、已封存頁、專案設定頁
- **THEN** 提示卡始終在專案欄底部同一位置;主區各視圖的高度與無提示時相同,看板欄底緣仍貼齊主區底緣

#### Scenario: 其他專案的狀況以琥珀點提示

- **WHEN** 開著專案 A(技能檔舊版)與 B(現版),作用中切到 B
- **THEN** A 的圖示列方塊角落亮琥珀點、提示附「技能檔需要處理」;B 無點;專案欄無提示卡;切回 A 後重新探測並顯示提示卡

### Requirement: 討論詳情頁檢視與轉出變更

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

點擊討論卡或細列 SHALL 開啟討論詳情頁，採「詳情頁的文件樹、閱讀卡與本頁目錄」的三欄骨架。左欄身分區 SHALL 以討論 slug 為題（等寬 14px）、緊跟複製 slug 鈕與狀態籤（討論中／已結論／已轉出／已轉出・尚無結論／已轉出・保留中，判準與看板討論卡一致），topic SHALL 為其下最多三行、溢出截斷的文字（slug 為題屬 speclink/LANGUAGE.md 明載的受控例外）；出身 SHALL 一行依序呈現建立者頭像＋名字、「N 輪」與建立日，缺席的段整段缺席；其下「關聯」列呈現先前討論數（自背景的 `Prior discussions:` 行計數；`none` 時該列缺席），點擊 SHALL 切到背景文件。主區標題列右端 SHALL 為動作區：concluded 且未封存的討論、以及已轉出且保留在途（hold 為 true）的討論呈現「封存」動作（outline）；保留在途時「封存」SHALL 停用並以提示說明結論保留在途、最後一刀轉出後自動封存；其餘（討論中、已轉出且未保留、已封存）的討論 SHALL NOT 呈現封存動作。

左欄「狀態」列 SHALL 為：「階段」列為步驟列「討論中 → 已結論 → 已轉出 → 已封存」且現站可辨識（promoted 為已轉出、concluded 為已結論、其餘為討論中；已封存站保留給已封存討論頁）；現站籤的色調 SHALL 與身分區狀態籤一致——狀態籤為「已轉出・保留中」或「已轉出・尚無結論」時現站為 warning 語意色，其餘為主色；現站之前的站 SHALL 呈現為已過站（打勾），但已轉出且尚無結論（concluded 為 false）時「已結論」站 SHALL 呈現為未到站（灰圈、無勾）；hold 為 true 時 SHALL 另有「保留在途」列呈現說明；promoted_to 非空時 SHALL 另有「衍生變更」列，每個子變更一顆可點等寬籤並附階段籤（階段派生與配色同「討論於看板第 0 欄兩級呈現」的 chip 矩陣），點籤 SHALL 開啟該變更的詳情頁；無衍生變更時該列缺席。

左欄「文件」節點依序 SHALL 為：結論、討論過程 N、背景、衍生變更 N——前三者呈現記錄文件對應區段（區段缺失或格式非預期時 SHALL 整篇以單一檢視退回而非報錯，此時節點為「記錄」與「衍生變更」）；記錄切分成功且結論區段非空時 SHALL 預設開啟「結論」，結論為空時預設「背景」。結論與討論過程的畫法見「討論結論以欄位標籤呈現」與「討論輪以卡片呈現」。

背景文件 SHALL 先把 Context 區段收尾的 `Related specs:`／`Related changes:`／`Prior discussions:`／`Source doc:` 四行自內文剝離，其餘分三段（各為本頁目錄錨點）：「起因與範圍」——第一段散文為引言、以灰底引言段呈現，其餘照 markdown，內文中與正式規格名、作用中變更名、已封存變更名、其他討論 slug 完全相同的字詞（前後皆非英數、連字號與底線；反引號內的不算）SHALL 自動成為可點籤，點擊分別開啟該規格的規格詳情頁、該變更的詳情頁、該封存變更的已封存詳情頁、該討論的詳情頁；「關聯」——鍵值列「正式規格」「在途變更」「已封存變更」「先前討論」「來源文件」，值為前述四行解析出的名稱（去掉括號備註、以逗號、頓號或分號切項、`none` 為空）與內文命中名稱的聯集籤（可點規則同上），值為空的列 SHALL 缺席（來源文件為 `none` 時顯示「無」、行缺席時列缺席），全部缺席時「關聯」段缺席；「記錄」——「建立」（建立日與建立者名字）、「種類」（kind 為 improve 時「改進討論」、否則「一般討論」）、「檔案」（live 討論為 `speclink/discussions/<slug>.md` 等寬文字附複製鈕；已封存討論頁該列缺席）。背景文件的數字磚 SHALL 為「命中的正式規格」「在途變更」「相關已封存變更」（各為關聯列的籤數，0 時缺席）與「先前討論」（`Prior discussions:` 行缺席時缺席，`none` 為 0）；出處行為「記錄建立時寫入一次」與建立日；首輪 mode 為 `assumptions` 時 SHALL 有 note 提示框「沒有 grill 階段」，否則無提示框。

衍生變更文件 SHALL 列出各子變更現況與跳轉且 SHALL 為唯讀——SHALL NOT 提供「轉為變更」或「再轉出一個變更」動作。結論的 Decision 列點可解析出分刀（見「討論結論以欄位標籤呈現」）時 SHALL 依刀分組：每組組頭為刀的識別籤、範圍句與該組變更數（組頭為本頁目錄錨點），組內列為該刀變更名籤中存在於 `promoted_to` 的變更、依籤序；不屬任何刀的子變更接在最後、無組頭。解析不出分刀時 SHALL 退回依轉出順序的平面清單、無組頭。每列 SHALL 呈現：階段籤（作用中變更依看板階段；已封存為「已封存」中性；兩者皆無為「已刪除」警示色加刪除線）、等寬變更名、任務進度（細進度條與 n/m；作用中取其任務完成數／總數、已封存取封存清單的任務數；無資料時缺席）、審查章與驗證章（作用中變更帶審查章或驗證章、已封存變更的結局為已審查或已驗證時各自出現）、日期（已封存為封存日期；作用中為最近蓋章日；皆無時缺席）、開啟鈕（作用中開啟該變更的詳情頁、已封存開啟其已封存詳情頁；已刪除無鈕）。衍生變更文件的數字磚 SHALL 為「變更」（總數）、「已封存」、「作用中」、「任務」（有任務資料的列加總 n/m；皆無資料時缺席）；出處行為「依結論的分刀分組」（有分刀時）或「依轉出順序」；討論為已轉出、hold 為 false 且恰有一個作用中子變更時 SHALL 有 tip 提示框「只差最後一刀」，內文點名該變更與其階段並說明它封存後討論會自動隨之封存；其他情況 SHALL NOT 有提示框。無衍生變更時衍生變更文件顯示既有空態文案、無數字磚。已封存討論頁的衍生變更文件 SHALL 同款（無「只差最後一刀」提示框）。

concluded 且未封存討論的封存動詞 SHALL 於討論卡與討論詳情頁皆可及，兩處 SHALL 走同一確認流程，經確認後將討論移入封存。GUI SHALL NOT 提供 conclude、add-round、new、discard、轉為變更（promote）——討論的推進、結論撰寫與轉出變更屬 agent 與 CLI。來自討論的變更卡 SHALL 帶討論徽章，其詳情頁 SHALL 顯示來源討論與同源變更清單並可互跳。

#### Scenario: 頁標題以 slug 為題且可複製

- **WHEN** 使用者開啟任一討論的詳情頁並點按左欄身分區 slug 旁的複製鈕
- **THEN** 身分區呈現該討論的 slug（等寬字型）、其下呈現 topic；slug 寫入剪貼簿並短暫顯示已複製回饋

#### Scenario: 有結論的討論預設開啟結論分頁

- **WHEN** 使用者開啟一筆已結論（結論區段非空）討論的詳情頁
- **THEN** 左欄「文件」節點為 結論／討論過程 N／背景／衍生變更 N，且預設閱讀卡呈現結論內容；「狀態」列的階段以「已結論」為現站

#### Scenario: 狀態卡的保留與衍生列

- **WHEN** 使用者開啟一筆已轉出、已有結論、hold 為 true、promoted_to 含一個已封存與一個進行中子變更的討論詳情頁
- **THEN** 身分區狀態籤為「已轉出・保留中」；「狀態」列的階段現站為「已轉出」且為 warning 語意色、與狀態籤同色，「討論中」與「已結論」兩站打勾；「保留在途」列呈現說明；「衍生變更」列兩顆籤分別附「已封存」與「進行中」階段籤，點進行中那顆開啟該變更的詳情頁；標題列右端的「封存」停用並有提示

#### Scenario: 尚無結論的已轉出討論的步驟列

- **WHEN** 使用者開啟一筆已轉出、concluded 為 false 的討論詳情頁
- **THEN** 身分區狀態籤為 warning 語意色的「已轉出・尚無結論」；「狀態」列的步驟列「討論中」打勾、「已結論」為灰圈且無勾、現站「已轉出」為 warning 語意色、「已封存」為未到站

#### Scenario: 已轉出且未保留的討論現站為主色

- **WHEN** 使用者開啟一筆已轉出、已有結論、hold 為 false 的討論詳情頁
- **THEN** 身分區狀態籤為主色的「已轉出」；步驟列現站「已轉出」為主色，「討論中」與「已結論」兩站打勾

#### Scenario: 衍生變更分頁唯讀且無轉出動作

- **WHEN** 使用者開啟一筆已結論或已轉出討論的衍生變更文件
- **THEN** 閱讀卡列出各子變更現況與開啟鈕，但不呈現「轉為變更」或「再轉出一個變更」按鈕

#### Scenario: GUI 不提供轉出等寫入動詞

- **WHEN** 使用者檢視任一討論詳情頁或討論卡
- **THEN** 介面不提供 conclude、add-round、轉為變更等寫入動作，轉出變更改由 CLI 或 agent 執行

#### Scenario: 同源 change 互跳

- **WHEN** 使用者開啟一個 from_discussion 非空的變更詳情頁
- **THEN** 左欄身分區的「來自」列顯示來源討論的 slug 籤（topic 於提示呈現）、「同源」列顯示同源變更籤，點擊同源籤開啟該變更的詳情頁

#### Scenario: 頁內封存 concluded 討論

- **WHEN** 使用者開啟一筆已結論、未封存且無 hold 討論的詳情頁，點標題列右端的「封存」並於確認對話框確認
- **THEN** 該討論移入封存：自討論欄消失、出現於已封存頁討論節，與自討論卡封存的效果一致；詳情頁關閉回看板

#### Scenario: 背景文件的引言、關聯與記錄列

- **WHEN** 使用者開啟一筆 live 討論（2026-10-08 由 MomoChen 建立、無 kind）的背景文件，其 Context 內文提到正式規格 desktop-app 與 tray-status-menu、作用中變更 add-copilot-tool、已封存變更 tray-panel-card-design，收尾行為「Related specs: desktop-app（79 條）、tray-status-menu」「Prior discussions: card-drawer-header-colors, admin-console-redesign」「Source doc: none」
- **THEN** 「起因與範圍」的第一段為灰底引言，內文中 desktop-app、tray-status-menu、add-copilot-tool、tray-panel-card-design 各成可點籤且收尾四行不出現在內文；「關聯」列為正式規格（desktop-app、tray-status-menu）、在途變更（add-copilot-tool）、已封存變更（tray-panel-card-design）、先前討論（兩顆）、來源文件「無」；「記錄」列為建立「2026-10-08 · MomoChen」、種類「一般討論」、檔案 `speclink/discussions/<slug>.md` 附複製鈕；數字磚為 2「命中的正式規格」、1「在途變更」、1「相關已封存變更」、2「先前討論」

#### Scenario: 背景的名稱籤跳轉與沒有 grill 階段提示框

- **WHEN** 使用者於上述背景文件點內文的 desktop-app 籤，再回到該討論點 add-copilot-tool 籤；該討論第一輪 mode 為 assumptions
- **THEN** 第一次開啟 desktop-app 的規格詳情頁（討論詳情頁依「詳情頁互斥」關閉），第二次開啟 add-copilot-tool 的變更詳情頁；背景文件有 note 提示框「沒有 grill 階段」，出處行為「記錄建立時寫入一次 · 2026-10-08」

#### Scenario: 背景無關聯行時關聯段缺席

- **WHEN** 使用者開啟一筆 Context 為兩段純散文、無收尾四行且內文未提及任何規格、變更或討論名稱的討論背景文件
- **THEN** 只有「起因與範圍」與「記錄」兩段，無「關聯」段；數字磚整列缺席；無名稱籤

#### Scenario: 衍生變更依分刀分組

- **WHEN** 使用者開啟 desktop-ui-redesign（結論含 cut 0～cut 5 共 7 張分刀卡、promoted_to 13 個變更，12 個已封存、1 個已就緒且兩站蓋章）的衍生變更文件
- **THEN** 列依 7 組呈現，各組組頭為 cut 識別籤、範圍句與變更數（cut 2 為 3 個變更），組頭列入本頁目錄；每列有階段籤、等寬名稱、任務進度 n/m、「審」「驗」章、日期與開啟鈕；數字磚為 13「變更」、12「已封存」、1「作用中」與「133 / 133」「任務」；出處行為「依結論的分刀分組」

##### Example: 列的欄位來源

| 子變更所在 | 階段籤 | 任務 | 審查章／驗證章 | 日期 | 開啟鈕 |
| --- | --- | --- | --- | --- | --- |
| 作用中（已就緒、已審查、已驗證） | 已就緒 | completedTasks/totalTasks | 兩章皆有 | 最近蓋章日 | 開啟變更詳情頁 |
| 作用中（進行中、審查中） | 進行中 | completedTasks/totalTasks | 無 | 缺席 | 開啟變更詳情頁 |
| 已封存（結局 reviewed、verified） | 已封存 | tasksDone/tasksTotal | 兩章皆有 | 封存日期 | 開啟已封存詳情頁 |
| 已封存（結局 reviewedNotPassed） | 已封存 | tasksDone/tasksTotal | 無審查章 | 封存日期 | 開啟已封存詳情頁 |
| 兩者皆無 | 已刪除（刪除線） | 缺席 | 無 | 缺席 | 無 |

#### Scenario: 讀不到分刀時退回平面清單

- **WHEN** 使用者開啟一筆結論的 Decision 沒有任何以 cut 或刀起頭的粗體列點、promoted_to 含兩個變更的討論衍生變更文件
- **THEN** 兩列依轉出順序平面呈現、無組頭；出處行為「依轉出順序」；數字磚照常

#### Scenario: 只差最後一刀提示框

- **WHEN** 使用者開啟一筆已轉出、hold 為 false、promoted_to 含兩個已封存與一個已就緒變更 tray-panel-restyle 的討論衍生變更文件
- **THEN** tip 提示框標題「只差最後一刀」、內文點名 tray-panel-restyle 與「已就緒」並說明它封存後討論會自動隨之封存；同一討論的 hold 為 true 時、或作用中子變更有兩個時，無此提示框

### Requirement: 詳情頁的排程分頁

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

change 詳情頁 SHALL 於左欄「文件」節點「規格」之後提供「排程」節點（tw 標籤「排程」、en「Plan」），點擊 SHALL 於閱讀卡呈現排程文件。清單項帶 `wave` 時閱讀卡 SHALL 依序呈現四張卡片（與審查／驗證文件同一種邊框卡片：卡頭與內容區以分隔線分格；卡頭為 16px 粗體標題帶本頁目錄錨點，重疊卡與封存順序卡的卡頭右端附筆數籤），data-plan-section 依序為 wave、depends、overlaps、archive：波次卡（「第 N 波」＋同波其他變更名稱清單，無其他時顯示「本波只有這個變更」）；前置卡（卡頭右側 SHALL 有狀態徽章——`blockedBy` 空時「可以開工」、非空時「等 N 項」且 title 列出前置名；內容為 dependsOn 每項一列，資料源 capability `setDepends` 為 true 時每列附移除鈕、底部附下拉選單列出候選變更供新增，選擇即寫入；空時「尚無前置」；`blockedBy` 含 `dependsOn` 以外的名稱（舊 server 以目錄級重疊判定阻擋）時 SHALL 另以一列唯讀列出這些名稱（tw「另需等待（server 依重疊判定）：」，不附移除鈕），讓徽章數字對得上卡片內容）；重疊卡（`requirementOverlap` 每項一列：對方變更名、`capability › requirement` 文字、雙方操作各一枚小標籤（用詞與配色同規格文件的 delta 區段，如 MODIFIED 顯示「修改」；title 為該操作所屬的變更名），`conflict` 為 true 時加「同名衝突」標籤；空時「無重疊」；SHALL NOT 顯示目錄級 `overlaps`）；封存順序卡（`archiveAfter` 每項變更名，下方一句「先封存它們，再對照正式規格重寫同名 requirement 後封存本變更」；其中有變更也把本變更列入它的 `archiveAfter`（互相等待）時 SHALL 另加一句警示（tw「與 {names} 互相等待：兩邊都要先等對方封存，須調整拆分」）；`requirementOverlap` 有 `conflict` 為 true 的列時 SHALL 加一句同名衝突警示（tw「與 {names} 同名衝突：先改掉其中一邊，較晚封存的一方才不會被拒」）；`archiveAfter` 為空且無同名衝突時「可直接封存」）。排程文件 SHALL NOT 渲染獨立的「阻擋」段。新增候選 SHALL 為該變更所在名冊中的其他作用中變更、扣除已宣告的前置：資料源提供名冊查詢時（local）SHALL 以查詢結果為準——該變更有 worktree 映射時名冊為其 worktree 副本、無映射時為主 checkout，與前置寫入作用的 store 相同；名冊查詢未完成時 SHALL NOT 渲染新增下拉；資料源未提供名冊查詢（remote，scope 只有一份名冊）或首次查詢（開啟詳情頁、切換變更）失敗時 SHALL 自作用中變更清單派生；看板刷新後的重新查詢失敗時 SHALL 沿用前一次的查詢結果。新增或移除前置 SHALL 經資料源 `setDepends` 寫回該變更 meta 的 depends_on，成功後看板與排程文件刷新；引擎拒絕（自依賴、不存在、已封存、成環）時 SHALL 以單行錯誤呈現引擎訊息、畫面不變。capability 為 false 時 SHALL NOT 渲染移除鈕與下拉選單。清單項缺 `wave` 且缺 `dependsOn`（remote 清單未取得 plan：舊 server、依賴成環或請求失敗）時閱讀卡 SHALL 只顯示一句說明（tw「此模式尚未提供排程資訊」），不渲染四卡；缺 `wave` 但帶 `dependsOn`（local plan 成環）時 SHALL 顯示一句說明（tw「依賴成環，先移除一條前置才有排程資訊」）並只渲染前置卡（無狀態徽章；`dependsOn` 為空時照樣顯示「尚無前置」；capability 為 true 時附移除鈕、SHALL NOT 渲染新增下拉），讓使用者解環。`requirementOverlap` 與 `archiveAfter` 缺席但 `wave` 存在（舊 server）時重疊卡與封存順序卡 SHALL 顯示各自的空態文案。判定 SHALL 收斂於 packages/ui 階段派生模組的 planWave／planBlockedBy／planRequirementOverlap／planArchiveAfter 單一入口。

清單項帶 `wave` 時閱讀卡 SHALL 另有：籤槽為前置卡同一枚狀態徽章（「可以開工」或「等 N 項」）；數字磚依序「波次」（N）、「宣告的前置」（`dependsOn` 筆數）、「同名需求重疊」（`requirementOverlap` 筆數）、「封存前要等」（`archiveAfter` 筆數）；一個提示框，標題由兩個子句組成——`blockedBy` 空為「可以開工」、非空為「等 N 項前置」；`archiveAfter` 空且無同名衝突為「可直接封存」、`archiveAfter` 非空為「封存前要等 M 個變更」、有同名衝突為「有同名衝突要先改」——兩個子句皆為綠態時提示框為 tip、否則為 warning；內文列出前置名、要先封存的變更名與同名衝突的對方變更名，並說明排程於看板刷新時重算。缺 `wave` 的兩種情況 SHALL NOT 渲染籤槽、數字磚與提示框。

#### Scenario: 排程分頁四段

- **WHEN** 開啟 wave=2、dependsOn=["add-a"]、blockedBy=["add-a"]、requirementOverlap=[{change:"add-b",capability:"desktop-app",requirement:"看板與任務",ownOperation:"MODIFIED",otherOperation:"MODIFIED",conflict:false}]、archiveAfter=["add-b"] 的變更詳情頁並點左欄「排程」節點
- **THEN** 閱讀卡依序顯示波次卡「第 2 波」、前置卡（徽章「等 1 項」、前置列 add-a 附移除鈕、新增下拉）、重疊卡一列（add-b、`desktop-app › 看板與任務`、兩枚「修改」標籤）、封存順序卡（add-b 與提示句），頁面無 data-plan-section="blocked"；四卡頭為 16px 粗體且進本頁目錄

#### Scenario: 新增前置寫回

- **WHEN** 於排程文件的下拉選單選擇 add-c
- **THEN** 該變更 .meta.yaml 的 depends_on 含 add-c，前置卡出現 add-c，看板順序刷新

#### Scenario: 引擎拒絕新增

- **WHEN** 於排程文件選擇會成環的變更
- **THEN** 顯示引擎的成環單行錯誤，前置卡不變，meta 檔不變

#### Scenario: 成環時可移除前置

- **WHEN** 清單 planError 非 null，開啟 dependsOn=["add-a"]（無 wave）的變更詳情頁並點「排程」節點
- **THEN** 顯示「依賴成環，先移除一條前置才有排程資訊」與前置卡的 add-a 列（附移除鈕），無狀態徽章、無新增下拉、無波次／重疊／封存順序卡；按移除鈕即寫回

#### Scenario: remote 分頁唯讀

- **WHEN** 於 capability setDepends 為 false 的資料源開啟詳情頁的排程文件
- **THEN** 不出現移除鈕與下拉選單，四卡照常顯示

#### Scenario: worktree 映射時候選來自副本名冊

- **WHEN** 主 checkout 且 worktree 政策開啟，add-dark-mode 已映射到 worktree，add-auth 在分支前建立、add-late 在分支後才於主 checkout 建立，開啟 add-dark-mode 的排程文件
- **THEN** 新增下拉選單含 add-auth、不含 add-late，看板清單仍列出 add-late

#### Scenario: 重新查詢名冊失敗沿用前次結果

- **WHEN** 排程文件已以名冊 add-a、add-b 呈現候選（作用中變更清單另有 add-d，add-a 已是前置），看板刷新後名冊查詢失敗
- **THEN** 新增下拉的候選仍只有 add-b，不出現 add-d

#### Scenario: 可開工徽章與同名衝突標籤

- **WHEN** 開啟 wave=1、blockedBy=[]、requirementOverlap=[{change:"add-x",capability:"auth",requirement:"登入",ownOperation:"ADDED",otherOperation:"ADDED",conflict:true}]、archiveAfter=[] 的變更詳情頁並點「排程」節點
- **THEN** 前置卡徽章為「可以開工」，重疊卡的 add-x 列帶「同名衝突」標籤與兩枚「新增」標籤，封存順序卡顯示「與 add-x 同名衝突：先改掉其中一邊，較晚封存的一方才不會被拒」且不顯示「可直接封存」

#### Scenario: 舊 server 缺新欄位的空態

- **WHEN** remote 清單項帶 wave 與 blockedBy 但無 requirementOverlap 與 archiveAfter
- **THEN** 重疊卡顯示「無重疊」、封存順序卡顯示「可直接封存」，其餘兩卡照常

#### Scenario: 舊 server 的重疊阻擋列在前置卡

- **WHEN** remote 清單項 wave=2、dependsOn=[]、blockedBy=["add-a"]（舊 server 把先配置、共用 capability 的 add-a 算進阻擋），無 requirementOverlap 與 archiveAfter
- **THEN** 前置卡徽章「等 1 項」、內容「尚無前置」與一列「另需等待（server 依重疊判定）：add-a」，該列無移除鈕；本機與新 server 的 blockedBy 只含宣告前置，不出現這一列

#### Scenario: 互相等待時封存卡警示

- **WHEN** add-a 的 archiveAfter=["add-b"]、add-b 的 archiveAfter=["add-a"]，開啟 add-a 的排程文件
- **THEN** 封存順序卡列 add-b 與提示句，另顯示「與 add-b 互相等待：兩邊都要先等對方封存，須調整拆分」；單向的 archiveAfter 不出現這句警示

#### Scenario: 成環時環外變更的前置卡空態

- **WHEN** 清單 planError 非 null，開啟 dependsOn=[]（無 wave）的環外變更詳情頁並點「排程」節點
- **THEN** 顯示「依賴成環，先移除一條前置才有排程資訊」與前置卡「尚無前置」，無狀態徽章、無新增下拉

#### Scenario: 排程的數字磚與提示框兩態

- **WHEN** 開啟排程分頁四段 scenario 的變更（wave=2、dependsOn 與 blockedBy 各一、重疊一列、archiveAfter 一個）並點「排程」節點；再開啟 wave=1、dependsOn=[]、blockedBy=[]、requirementOverlap=[]、archiveAfter=[] 的變更
- **THEN** 第一個變更的籤槽為「等 1 項」、數字磚「波次 2」「宣告的前置 1」「同名需求重疊 1」「封存前要等 1」、warning 提示框標題「等 1 項前置，封存前要等 1 個變更」且內文列出 add-a 與 add-b；第二個變更的籤槽為「可以開工」、數字磚「波次 1」「宣告的前置 0」「同名需求重疊 0」「封存前要等 0」、tip 提示框標題「可以開工，也可直接封存」

##### Example: 提示框標題的兩個子句

| blockedBy | archiveAfter | 同名衝突 | 標題 | 型 |
| --------- | ------------ | -------- | ---- | -- |
| [] | [] | 無 | 可以開工，也可直接封存 | tip |
| [add-a] | [] | 無 | 等 1 項前置，可直接封存 | warning |
| [] | [add-b] | 無 | 可以開工，封存前要等 1 個變更 | warning |
| [add-a] | [add-b] | 無 | 等 1 項前置，封存前要等 1 個變更 | warning |
| [] | [] | add-x | 可以開工，有同名衝突要先改 | warning |

#### Scenario: 缺 wave 時無數字磚與提示框

- **WHEN** 開啟 remote 清單項缺 wave 且缺 dependsOn 的變更詳情頁並點「排程」節點
- **THEN** 閱讀卡只顯示「此模式尚未提供排程資訊」，籤槽、數字磚與提示框皆缺席

### Requirement: 詳情頁的文件動作

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

變更詳情頁、討論詳情頁、規格詳情頁與手冊頁的閱讀卡標題列右端 SHALL 提供「開啟 ▾」選單（outline 樣式、與標題同列），對目前文件對應的檔案提供：以預設 App 開啟、在 Finder 顯示（標籤隨平台：Finder／檔案總管／檔案管理員）、複製路徑；選單首列 SHALL 以等寬灰字顯示該檔案相對所在 checkout 根的路徑。文件與檔案的對應 SHALL 為：提案 `speclink/changes/<name>/proposal.md`、設計 `design.md`、任務 `tasks.md`、規格 `specs/<capability>/spec.md`（目前選定的 capability）、審查 `review.md`、驗證 `verify.md`；討論各文件皆為該討論的記錄檔；正式規格為 `speclink/specs/<capability>/spec.md`；手冊頁為 `speclink/manual/<slug>.md`（spec 目錄名依專案設定）。檔案的實際位置 SHALL 與詳情頁讀取內容的位置相同：綁定 worktree 的變更指向 worktree 副本、spec 目錄名依專案設定、已封存的討論指向 `speclink/discussions/archive/<封存日>-<slug>.md`。沒有對應檔案的文件（排程、規格詳情頁的溯源）SHALL NOT 顯示選單。remote 專案無本機 checkout 時選單 SHALL 只有「複製內容」（該文件已載入的原文；未載完時項目停用）且首列路徑缺席；工單文件（審查、驗證）不載原文，此時 SHALL NOT 顯示選單。「以預設 App 開啟」SHALL 只開啟 markdown 文件：傳入的檔名與符號連結的實際指向皆須為 markdown。動作失敗時 SHALL 以既有失敗 toast 語意呈現錯誤，SHALL NOT 彈出錯誤視窗。「開啟 ▾」SHALL NOT 出現在主區標題列，也 SHALL NOT 收進「⋯」選單。

#### Scenario: 提案分頁的開啟選單

- **WHEN** 使用者於本機專案 /work/app 的變更 add-auth 詳情頁提案文件展開閱讀卡標題列的「開啟 ▾」並選「複製路徑」
- **THEN** 選單首列為 `speclink/changes/add-auth/proposal.md`，其後三項為以預設 App 開啟、在 Finder 顯示（macOS）、複製路徑；剪貼簿寫入 `/work/app/speclink/changes/add-auth/proposal.md`

#### Scenario: worktree 中的變更指向 worktree 副本

- **WHEN** 變更 add-auth 綁定 worktree /work/app-wt/add-auth，使用者於其詳情頁任務文件展開「開啟 ▾」並選「在 Finder 顯示」
- **THEN** 選單首列為 `speclink/changes/add-auth/tasks.md`，Finder 顯示 `/work/app-wt/add-auth/speclink/changes/add-auth/tasks.md`（詳情頁所讀的那一份）

#### Scenario: 已封存討論指向封存檔

- **WHEN** 使用者自變更詳情頁的來源討論籤開啟已封存的討論 old-topic（封存檔為 `speclink/discussions/archive/2026-01-03-old-topic.md`）並展開「開啟 ▾」
- **THEN** 選單首列為 `speclink/discussions/archive/2026-01-03-old-topic.md`，複製路徑得到該封存檔的絕對路徑

#### Scenario: 排程分頁無選單

- **WHEN** 使用者點左欄的「排程」節點
- **THEN** 閱讀卡標題列右端不顯示「開啟 ▾」，主區標題列也沒有「開啟 ▾」

#### Scenario: remote 只剩複製內容

- **WHEN** 使用者於無本機 checkout 的 remote 專案開啟討論詳情頁並展開「開啟 ▾」
- **THEN** 選單只有「複製內容」且無路徑首列；選後剪貼簿為該討論記錄的原文

#### Scenario: 在 Finder 顯示失敗以 toast 呈現

- **WHEN** 使用者選「在 Finder 顯示」而該檔案已被外部刪除
- **THEN** 以失敗 toast 呈現 opener 回報的錯誤，詳情頁維持開啟，無錯誤視窗

#### Scenario: 手冊頁的開啟選單

- **WHEN** 使用者於本機專案 /work/app 的手冊頁閱讀〈系統匣選單〉（檔案 `speclink/manual/tray.md`），展開閱讀卡標題列的「開啟 ▾」並選「複製路徑」
- **THEN** 選單首列為 `speclink/manual/tray.md`，其後三項為以預設 App 開啟、在 Finder 顯示（macOS）、複製路徑；剪貼簿寫入 `/work/app/speclink/manual/tray.md`；規格詳情頁的「溯源」文件無選單；remote 模式的手冊頁顯示既有空狀態，無閱讀卡與選單

### Requirement: 已封存項目以詳情頁檢視

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

已封存頁的變更列與討論列 SHALL 以列式清單呈現，點擊列 SHALL 開啟唯讀的已封存詳情頁（宿主維持已封存頁），SHALL NOT 提供行內展開。已封存詳情頁 SHALL 與變更／討論詳情頁同骨架（「詳情頁的文件樹、閱讀卡與本頁目錄」：左欄身分區、文件節點、「狀態」列，中欄閱讀卡，右欄本頁目錄）且唯讀——SHALL NOT 提供任務勾選、動詞執行或任何寫入操作；身分區 SHALL NOT 呈現進度條，標題列 SHALL NOT 有動作區——封存是唯讀定格；開啟期間內容 SHALL 隨外部檔案變更反映；所請求的文件不存在時對應閱讀卡 SHALL 顯示空狀態而非錯誤。

封存變更的詳情頁 SHALL 至少含提案、設計、任務、規格文件節點（內容來自封存目錄的實體文件），閱讀卡標題列右端提供「開啟 ▾」（對應 `speclink/changes/archive/<封存目錄名>/` 下的檔案）；任務文件的勾選框不可點擊、無拖曳把手、無批次工具列、整列不回應點擊。身分區 SHALL 為變更名（等寬）緊跟複製鈕（複製含日期前綴的封存目錄名，複製後 SHALL 有已複製回饋）與「已封存」籤，審查／驗證結局籤 SHALL NOT 出現在身分區——結局只由「狀態」列的品質關卡結局承載；出身 SHALL 一行依序呈現建立者（首字母圓標＋名字，完整識別收 hover 提示）、建立日期、「封存 YYYY-MM-DD」，其下「來自」列呈現來源討論籤（有來源討論時；點擊 SHALL 開啟該討論的已封存詳情頁，記錄以 live 優先、封存後備載入），出身行恆定單行、溢出裁切，任一欄位資料缺席時該欄位 SHALL 缺席、其餘照常。「狀態」列 SHALL 為三列：任務 n/m（無 tasks.md 時「—」）、品質關卡結局（皆無時「未經品質關卡」）、封存日期。

封存討論的詳情頁身分區 SHALL 以 slug 為題（等寬）緊跟複製 slug 鈕與「已封存」籤（kind 為 improve 時另帶改進籤），topic 為其下最多三行；出身為建立者、建立日期與「封存 YYYY-MM-DD」，任一欄位資料缺席時該欄位 SHALL 缺席、其餘照常（清單資料不帶封存討論的封存日期時，「封存」欄缺席）。「狀態」列 SHALL 含階段列（步驟列與討論詳情頁相同、現站為「已封存」）；promoted_to 非空時 SHALL 另有「衍生變更」列：每顆籤依子變更所在呈現三態——在封存清單中者為可點籤、副標為封存日期、點擊開啟該封存變更的已封存詳情頁；在活躍變更清單中者為可點籤、副標為其看板階段詞、點擊開啟該變更的詳情頁（宿主依「變更與討論詳情頁的宿主為看板」切回看板）；兩者皆無者為不可點籤、副標「無封存記錄」；順序依轉出順序；無衍生變更時該列缺席，封存變更的詳情頁 SHALL NOT 顯示該列。文件節點 SHALL 為「結論」「討論過程」「背景」「衍生變更」；「開啟 ▾」SHALL 只有「複製內容」。

#### Scenario: 點擊封存變更列開啟四分頁詳情頁

- **WHEN** 使用者於已封存頁點擊一個含完整 artifacts 的封存變更列
- **THEN** 主區換成唯讀的已封存詳情頁，左欄文件節點為提案／設計／任務／規格，各文件的閱讀卡呈現封存目錄內對應文件的內容，任務文件的勾選框不可點擊、無把手與工具列；宿主維持已封存頁，清單無行內展開

#### Scenario: 點擊封存討論列開啟唯讀詳情頁

- **WHEN** 使用者於已封存頁討論節點擊一筆封存討論列
- **THEN** 主區換成該討論的已封存詳情頁：身分區有「已封存」籤、「狀態」列的階段現站為「已封存」、文件節點為結論／討論過程／背景／衍生變更，無轉為變更、封存或任何寫入按鈕

#### Scenario: 缺件文件顯示空狀態

- **WHEN** 使用者開啟一個無 design.md 的封存變更詳情頁並點「設計」節點
- **THEN** 閱讀卡顯示空狀態文字，無錯誤彈窗，其餘文件照常可用

#### Scenario: 自封存變更詳情頁跳轉來源討論

- **WHEN** 使用者開啟一個帶來源討論的封存變更詳情頁，點擊身分區「來自」列的來源討論籤
- **THEN** 該討論的已封存詳情頁取代顯示（記錄以 live 優先、封存後備載入），宿主維持已封存頁；無來源討論的封存變更詳情頁不顯示此列

#### Scenario: 封存變更詳情頁的標題列、出身列與狀態卡

- **WHEN** 使用者開啟一筆建立者與建立日期俱在、20/21 任務、reviewStatus 為 reviewed 的封存變更詳情頁並點擊身分區變更名旁的複製鈕
- **THEN** 剪貼簿寫入含日期前綴的封存目錄名並顯示已複製回饋；身分區只帶「已封存」籤、無「已審查」籤；出身行顯示建立者首字母圓標＋名字、建立日期與「封存 YYYY-MM-DD」；「狀態」列三列為「20/21」、「已審查」籤、封存日期；無進度條，標題列無動作區

#### Scenario: 出身列欄位缺席時其餘照常

- **WHEN** 使用者開啟一筆建立日期不可得、建立者可得的封存變更詳情頁
- **THEN** 出身行顯示建立者與封存日期，建立日期欄位缺席，無錯誤與空欄佔位

#### Scenario: 封存討論詳情頁列出衍生變更並跳轉封存變更

- **WHEN** 使用者開啟一筆 promoted_to 含兩個已封存子變更的封存討論詳情頁，點擊「狀態」列中「衍生變更」的第一顆籤
- **THEN** 列有兩顆依轉出順序的籤、副標各為封存日期；點擊後該子變更的已封存詳情頁取代顯示，宿主頁面維持已封存頁

#### Scenario: 衍生變更籤的三態

- **WHEN** 使用者開啟一筆 promoted_to 含三個子變更的封存討論詳情頁——一個已封存、一個仍在看板進行中、一個已刪除
- **THEN** 三顆籤依轉出順序呈現，各自的副標與可點性如下表；點擊活躍者開啟其詳情頁且宿主切回看板，點擊已刪除者無任何動作

##### Example: 三態對照

| 子變更所在 | 副標 | 可點 | 點擊結果 |
| --- | --- | --- | --- |
| 封存清單 | 封存日期（如 2026-09-02） | 是 | 開啟該封存變更的已封存詳情頁，宿主不切頁 |
| 活躍變更清單 | 看板階段詞（如「進行中」） | 是 | 開啟該變更的詳情頁，宿主落回看板 |
| 兩者皆無 | 無封存記錄 | 否 | 無 |

#### Scenario: 無衍生變更時衍生列缺席

- **WHEN** 使用者開啟一筆 promoted_to 為空的封存討論詳情頁
- **THEN** 「狀態」列只有階段列，無「衍生變更」列；封存變更詳情頁亦無該列

#### Scenario: 已封存討論的文件動作只有複製內容

- **WHEN** 使用者於封存討論詳情頁展開閱讀卡標題列的「開啟 ▾」
- **THEN** 選單只有「複製內容」，選後剪貼簿為該記錄原文

### Requirement: 詳情頁的文件樹、閱讀卡與本頁目錄

<!-- BEFORE: 以 openspec/ 與 .openspec.yaml 描述規格資料夾與變更資料檔 -->

變更詳情頁、討論詳情頁與已封存詳情頁 SHALL 共用同一個三欄骨架，SHALL NOT 有底部固定列：

- **左欄「文件樹」**（固定 240px、獨立捲動）由上而下為身分區、「文件」節點清單、「狀態」列。身分區 SHALL 呈現名稱或 slug（等寬 14px，過長時折行、SHALL NOT 撐寬欄）、階段籤（SHALL NOT 含審查／驗證章籤——品質關卡由「狀態」列承載）、一行出身（建立者首字母圓標＋名字、產生工具、建立日、「開工 YYYY-MM-DD」；缺席的段整段缺席）、關聯列各自成行——變更為「來自」＋來源討論籤（首顆直出、其餘收「+N」浮層）與「規格」＋delta 能力籤（點擊切到規格文件）；討論為 topic（最多三行、溢出截斷）與「關聯」一列（先前討論數，自背景的 `Prior discussions:` 行計數，`none` 時該列缺席；點擊切到背景文件）。建立者與開工者含 email 的完整識別 SHALL 收進主題化提示。「文件」節點 SHALL 每份文件一個，含圖示、名稱與徽章，點擊 SHALL 切換閱讀卡內容，作用中節點 SHALL 以主色淡底標示；節點 SHALL NOT 列出文件內的段落——頁內清單一律由本頁目錄承接。「狀態」列 SHALL 承接原「狀態」卡的內容（左標籤右內容，細線分隔）。
- **中欄閱讀區**：內容欄置中、寬 768px，一張閱讀卡。閱讀卡 SHALL 由上而下為標題區（24px 一般字重文件名、籤槽、同列右端的「開啟 ▾」）、出處行（灰字：變更與已封存變更的文件為該檔案相對變更目錄的路徑，如 `proposal.md`、`specs/<capability>/spec.md`；排程為推算來源「由 speclink plan 推算」；討論與已封存討論的文件為自記錄推導的出處——結論為記錄路徑與「N 輪之後寫入」、討論過程為日期範圍與各 mode 輪數、背景為「記錄建立時寫入一次」與建立日、衍生變更為「依結論的分刀分組」或「依轉出順序」；已封存討論的文件 SHALL NOT 出記錄路徑段）、數字磚列容器（0 到 4 格、磚缺席時整列缺席）、提示框容器、內文。切換文件時閱讀區 SHALL 自頂端呈現；文件載入三態沿用「詳情頁文件載入以 skeleton 呈現」。
- **右欄「本頁目錄」**（200px）SHALL 自閱讀卡渲染後的 DOM 推導目錄：列入帶目錄錨點的標題，點擊 SHALL 捲至該標題，捲動時 SHALL 高亮目前段；沒有錨點時 SHALL NOT 渲染。頁寬 ≥1200px 時 SHALL 顯示，窄於 1200px 時 SHALL 隱藏且閱讀區維持置中。各文件的目錄錨點為其渲染器標出的區段標題：提案與設計的章節標籤、任務的群組標題、規格 delta 的色標區段標頭與需求標題、排程的四卡標題、工單的輪次標題、結論的欄位區段標題（含「分刀」）、討論過程的輪次籤（依輪卡的畫面順序，由新到舊）、背景的三個區段標題、衍生變更的分刀組頭。

變更的文件節點依序 SHALL 為提案、設計、任務（徽章 n/m）、規格（delta 計數徽章）、排程（有 `wave` 時徽章「第 N 波」）、審查、驗證（後兩者的出現條件與「詳情頁的工單分頁」相同，徽章為末輪序號）；討論的文件節點依序 SHALL 為結論、討論過程（徽章為輪數）、背景、衍生變更（徽章為數量），預設文件依「討論詳情頁檢視與轉出變更」；已封存變更為提案、設計、任務、規格與條件式的審查、驗證，已封存討論為結論、討論過程、背景、衍生變更。已封存詳情頁 SHALL 為同一骨架的唯讀變體：身分區帶「已封存」籤與「封存 YYYY-MM-DD」，標題列 SHALL NOT 有動作。

#### Scenario: 三欄骨架與左欄內容

- **WHEN** 使用者於寬 1440px 的視窗開啟 reviewStatus 為 reviewed、verifyStatus 為 verified、來源討論一份、delta 能力一個、9/9 任務的已就緒變更詳情頁
- **THEN** 主區為左欄文件樹、中欄置中 768px 的閱讀卡、右欄本頁目錄，無底部固定列；左欄身分區顯示等寬變更名、「已就緒」籤（無「已審查」「已驗證」章籤）、一行出身、「來自」列的討論籤與「規格」列的能力籤；「文件」節點為提案、設計、任務 9/9、規格、排程、無審查與驗證節點；「狀態」列為任務進度、品質關卡（「已審查」「已驗證」章籤）、排程三列

#### Scenario: 點文件節點切換閱讀卡

- **WHEN** 使用者於變更詳情頁點左欄的「任務」節點，再點「規格」節點
- **THEN** 「任務」節點以主色淡底標示且閱讀卡標題為「任務」、內文為任務清單；再點後「規格」節點標示、閱讀卡自頂端呈現規格 delta，左欄捲動位置不變

#### Scenario: 閱讀卡的出處行

- **WHEN** 使用者於變更詳情頁依序看「提案」與「排程」兩份文件，再開啟一筆 11 輪、2026-10-12 建立的 live 討論詳情頁依序看結論、討論過程、背景、衍生變更，最後開啟一筆已封存討論的結論文件
- **THEN** 提案的出處行為 `proposal.md`，排程的出處行為「由 speclink plan 推算」；討論的結論出處行為 `speclink/discussions/<slug>.md · 11 輪之後寫入`、討論過程為日期範圍與各 mode 輪數、背景為「記錄建立時寫入一次 · 2026-10-12」、衍生變更為「依結論的分刀分組」；已封存討論的結論出處行只有「N 輪之後寫入」、無路徑段

#### Scenario: 本頁目錄的錨點與高亮

- **WHEN** 使用者於寬 1440px 的視窗開啟含 Why、What Changes、Impact 三個模板章節的提案文件並往下捲到「影響」區段
- **THEN** 右欄本頁目錄列出「為什麼」「變更內容」「影響」三個錨點，捲到「影響」後該錨點高亮；點擊「為什麼」閱讀區捲回該章節

#### Scenario: 窄於 1200px 時無本頁目錄

- **WHEN** 使用者把視窗縮到寬 1100px 再開啟同一份文件
- **THEN** 右欄本頁目錄不顯示，左欄文件樹仍在，閱讀卡維持置中 768px，頁無水平捲軸

#### Scenario: 已封存詳情頁的唯讀變體

- **WHEN** 使用者於已封存頁開啟一筆封存變更的詳情頁
- **THEN** 同一三欄骨架；左欄身分區帶「已封存」籤與「封存 YYYY-MM-DD」；標題列右端只有 ‹ n／m ›、無分析、封存與「⋯」；「文件」節點為提案、設計、任務、規格；任務文件的勾選框不可點

#### Scenario: 討論文件的本頁目錄錨點

- **WHEN** 使用者於寬 1440px 的視窗開啟一筆 20 輪、結論含六欄位與 7 刀的討論詳情頁，依序看結論、討論過程、衍生變更
- **THEN** 結論的本頁目錄依序為決定、分刀、理由、否決替代案、擱置、記錄去向、下一步；討論過程的本頁目錄依序為 Round 20 到 Round 1；衍生變更的本頁目錄為 7 個分刀組頭；點任一錨點閱讀區捲至該處

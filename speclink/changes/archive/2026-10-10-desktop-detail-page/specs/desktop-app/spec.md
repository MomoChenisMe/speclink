## ADDED Requirements

### Requirement: 詳情頁互斥

桌面 app SHALL 保證同時僅一個詳情頁（變更詳情、規格、已封存、討論）開啟：任一詳情頁開啟中，自任何入口（看板卡片、規格清單、已封存頁、系統匣選單與面板、頁內跳轉、‹ › 切換）開啟另一個詳情頁時，先開者 SHALL 關閉、由後開者取代顯示，SHALL NOT 疊加顯示、SHALL NOT 拒開。此互斥 SHALL 由狀態層的開啟動作保證——每個開啟動作於設定自身選定狀態時同時清除其他詳情頁的選定狀態；SHALL NOT 依賴各呼叫端入口自行先關再開。

變更詳情頁與討論詳情頁的開啟動作 SHALL 同時收掉可取消浮層：新增專案對話框、初始化確認框、啟用確認框、封存確認框（含品質關卡工單三選項框）、封存討論確認框、刪除確認框、退回提案中確認框、退回被擋說明框、更新日誌對話框、技能檔提示確認框。收掉的語意 SHALL 與使用者按該浮層的「取消」或「關閉」相同——SHALL NOT 送出任何未確認的動作；更新日誌於 whatsNew 模式被收掉時 SHALL NOT 記為已看過。此保證 SHALL 同樣由狀態層的開啟動作承擔，自任何入口開啟結果 SHALL 一致。

不可取消浮層（遷移對話框、遠端衝突對話框）開啟中，變更詳情頁與討論詳情頁的開啟動作 SHALL NOT 改變任何狀態：浮層維持、頁不開、宿主頁面不切換；自系統匣觸發時主視窗 SHALL 仍顯示到前景。規格詳情頁與已封存詳情頁的開啟動作 SHALL NOT 受本段約束（其宿主頁為規格頁與已封存頁）。

#### Scenario: 討論詳情頁開啟中自系統匣開啟變更詳情

- **WHEN** 討論詳情頁開啟中，使用者自系統匣開啟某 change 的詳情頁
- **THEN** 討論詳情頁關閉，僅變更詳情頁可見

#### Scenario: 任兩種詳情頁交錯開啟

- **WHEN** 任一詳情頁開啟中，使用者自任何入口開啟另一種詳情頁
- **THEN** 先開者關閉、後開者顯示，同時僅一個詳情頁可見

##### Example: 四種詳情頁互斥狀態轉移

| 開啟中的詳情頁 | 開啟動作         | 結果                       |
| -------------- | ---------------- | -------------------------- |
| 討論           | 開啟變更詳情     | 討論關閉，僅變更詳情可見   |
| 變更詳情       | 開啟規格         | 變更詳情關閉，僅規格可見   |
| 規格           | 開啟已封存       | 規格關閉，僅已封存可見     |
| 已封存         | 開啟討論         | 已封存關閉，僅討論可見     |

#### Scenario: 新增專案對話框開啟中自系統匣開啟變更詳情

- **WHEN** 新增專案對話框開啟中，使用者自系統匣面板或原生選單選「開啟此變更」
- **THEN** 對話框關閉（未開啟任何專案）、變更詳情頁開啟、宿主為看板，畫面沒有任何浮層

#### Scenario: 可取消確認框開啟中開啟討論詳情頁

- **WHEN** 初始化確認、啟用確認、封存確認（含工單三選項框）、封存討論確認、刪除確認、退回提案中確認、退回被擋說明或技能檔提示確認任一開啟中，使用者自任何入口開啟某討論
- **THEN** 該確認框關閉且未送出任何動作，討論詳情頁開啟

##### Example: 可取消浮層與開啟動作的狀態轉移

| 開啟中的浮層 | 開啟動作     | 結果                                   |
| ------------ | ------------ | -------------------------------------- |
| 新增專案     | 開啟變更詳情 | 對話框關閉、詳情頁開啟、無專案被開啟   |
| 封存確認     | 開啟討論     | 確認框關閉、詳情頁開啟、無封存發生     |
| 工單三選項框 | 開啟變更詳情 | 三選項框關閉、詳情頁開啟、工單維持未結 |
| 刪除確認（change A） | 開啟討論 | 確認框關閉、詳情頁開啟、change A 未被刪除 |
| 更新日誌（whatsNew） | 開啟討論 | 對話框關閉、詳情頁開啟、下次啟動仍彈出 |
| 技能檔提示確認 | 開啟變更詳情 | 確認框關閉、詳情頁開啟、技能檔未改寫 |

#### Scenario: 更新日誌對話框開啟中自系統匣開啟變更詳情

- **WHEN** 更新日誌對話框開啟中，使用者自系統匣開啟某 change 的詳情頁
- **THEN** 更新日誌對話框關閉、變更詳情頁開啟；whatsNew 模式下 SHALL NOT 記為已看過

#### Scenario: 遷移進行中自系統匣開啟變更詳情

- **WHEN** 遷移對話框開啟中，使用者自系統匣面板選「開啟此變更」
- **THEN** 主視窗顯示到前景、遷移對話框維持、變更詳情頁不開、宿主頁面不變

#### Scenario: 遠端衝突對話框開啟中開啟討論

- **WHEN** 遠端衝突對話框開啟中，使用者自任何入口開啟某討論
- **THEN** 衝突對話框維持、討論詳情頁不開、任何狀態不變

### Requirement: 討論詳情頁檢視與轉出變更

點擊討論卡或細列 SHALL 開啟討論詳情頁。頁標題 SHALL 以討論 slug 為題（等寬字型）、緊跟複製 slug 鈕與狀態籤（討論中／已結論／已轉出／已轉出・尚無結論／已轉出・保留中，判準與看板討論卡一致），topic SHALL 為標題下方的一行文字（slug 為題屬 openspec/LANGUAGE.md 明載的受控例外）；出身列 SHALL 依序呈現建立者頭像＋名字、「N 輪」與建立日，缺席的段整段缺席。標題列右端 SHALL 為動作區：concluded 且未封存的討論、以及已轉出且保留在途（hold 為 true）的討論呈現「封存」動作；保留在途時「封存」SHALL 停用並以提示說明結論保留在途、最後一刀轉出後自動封存；其餘（討論中、已轉出且未保留、已封存）的討論 SHALL NOT 呈現封存動作。

標題區之下 SHALL 為「狀態」卡：「階段」列為步驟列「討論中 → 已結論 → 已轉出 → 已封存」且現站可辨識（promoted 為已轉出、concluded 為已結論、其餘為討論中；已封存站保留給已封存討論頁）；hold 為 true 時 SHALL 另有「保留在途」列呈現說明；promoted_to 非空時 SHALL 另有「衍生變更」列，每個子變更一顆可點等寬籤並附階段籤（階段派生與配色同「討論於看板第 0 欄兩級呈現」的 chip 矩陣），點籤 SHALL 開啟該變更的詳情頁；無衍生變更時該列缺席。

分頁 SHALL 為內容卡的標頭，依序為：結論、討論過程 N、背景、衍生變更 N——前三者呈現記錄文件對應區段（區段缺失或格式非預期時 SHALL 整篇以單一檢視退回而非報錯）；記錄切分成功且結論區段非空時 SHALL 預設開啟「結論」分頁，結論為空時預設「背景」。衍生變更分頁 SHALL 列出各子變更現況與跳轉，且 SHALL 為唯讀——SHALL NOT 提供「轉為變更」或「再轉出一個變更」動作。concluded 且未封存討論的封存動詞 SHALL 於討論卡與討論詳情頁皆可及，兩處 SHALL 走同一確認流程，經確認後將討論移入封存。GUI SHALL NOT 提供 conclude、add-round、new、discard、轉為變更（promote）——討論的推進、結論撰寫與轉出變更屬 agent 與 CLI。來自討論的變更卡 SHALL 帶討論徽章，其詳情頁 SHALL 顯示來源討論與同源變更清單並可互跳。

#### Scenario: 頁標題以 slug 為題且可複製

- **WHEN** 使用者開啟任一討論的詳情頁並點按標題旁的複製鈕
- **THEN** 頁標題呈現該討論的 slug（等寬字型）、其下一行呈現 topic；slug 寫入剪貼簿並短暫顯示已複製回饋

#### Scenario: 有結論的討論預設開啟結論分頁

- **WHEN** 使用者開啟一筆已結論（結論區段非空）討論的詳情頁
- **THEN** 內容卡標頭顯示分頁 結論／討論過程 N／背景／衍生變更，且預設呈現結論內容；狀態卡的階段列以「已結論」為現站

#### Scenario: 狀態卡的保留與衍生列

- **WHEN** 使用者開啟一筆已轉出、已有結論、hold 為 true、promoted_to 含一個已封存與一個進行中子變更的討論詳情頁
- **THEN** 標題列狀態籤為「已轉出・保留中」；階段列現站為「已轉出」；「保留在途」列呈現說明；「衍生變更」列兩顆籤分別附「已封存」與「進行中」階段籤，點進行中那顆開啟該變更的詳情頁；「封存」動作停用並有提示

#### Scenario: 衍生變更分頁唯讀且無轉出動作

- **WHEN** 使用者開啟一筆已結論或已轉出討論的衍生變更分頁
- **THEN** 分頁列出各子變更現況與跳轉按鈕，但不呈現「轉為變更」或「再轉出一個變更」按鈕

#### Scenario: GUI 不提供轉出等寫入動詞

- **WHEN** 使用者檢視任一討論詳情頁或討論卡
- **THEN** 介面不提供 conclude、add-round、轉為變更等寫入動作，轉出變更改由 CLI 或 agent 執行

#### Scenario: 同源 change 互跳

- **WHEN** 使用者開啟一個 from_discussion 非空的變更詳情頁
- **THEN** 出身列顯示來源討論的 slug 籤（topic 於提示呈現）與同源變更籤，點擊同源籤開啟該變更的詳情頁

#### Scenario: 頁內封存 concluded 討論

- **WHEN** 使用者開啟一筆已結論、未封存且無 hold 討論的詳情頁，點「封存」並於確認對話框確認
- **THEN** 該討論移入封存：自討論欄消失、出現於已封存頁討論節，與自討論卡封存的效果一致；詳情頁關閉回看板

### Requirement: 詳情頁標頭標記受寬度約束且頁不產生水平捲軸

變更詳情頁與已封存詳情頁的標頭中,來源討論標記 SHALL 以討論 slug 直出(等寬字型),topic SHALL NOT 於標記面直出。標記 SHALL 設寬度上限:slug 超出上限時 SHALL 以截斷呈現,SHALL NOT 使標記寬度超出頁內容區。標記 SHALL 以主題化提示呈現完整 slug 與 topic,使用者 SHALL 能取得未截斷的原始內容。多份來源討論 SHALL NOT 各自換行——僅第一份直接顯示,其餘收進「+N」數字籤與其浮層(呈現規則見「變更的來源討論多值呈現」)。詳情頁 SHALL NOT 於水平方向產生捲軸——不論標頭標記、動作列或內文包含多長的單一元素,詳情頁的標頭、動作列與內文 SHALL 維持在可視寬度內。此約束 SHALL 同時套用於變更詳情頁與已封存詳情頁,SHALL NOT 只在其中一處成立。

來源討論標記的資料來源、數量、排序,以及點擊(標記或浮層項)後開啟對應討論的行為 SHALL 維持不變。

#### Scenario: 長討論 topic 不撐寬詳情頁

- **WHEN** 使用者開啟一個來源討論的 topic 長度遠超過詳情頁寬度的變更,其詳情頁
- **THEN** 該來源討論標記顯示 slug 而非 topic,寬度不超出頁內容區,詳情頁沒有水平捲軸,標頭、動作列與內文皆維持在可視寬度內

#### Scenario: 截斷後仍可取得完整 topic

- **WHEN** 使用者將指標停留在來源討論標記上
- **THEN** 主題化提示呈現該討論完整的 slug 與 topic 文字

#### Scenario: 已封存詳情頁適用同一約束

- **WHEN** 使用者於已封存頁開啟同一個變更的詳情頁
- **THEN** 來源討論標記同樣以 slug 直出、多份時溢出收「+N」籤,詳情頁沒有水平捲軸

#### Scenario: 超長 slug 截斷呈現

- **WHEN** 使用者開啟來源討論 slug 長於標記寬度上限的變更,其詳情頁
- **THEN** 該標記以截斷呈現且寬度不超出上限,提示保有完整 slug 與 topic;點擊標記開啟對應討論的詳情頁

##### Example: 標記呈現與詳情頁捲軸

| 情境                 | 標記面呈現                  | 提示           | 詳情頁水平捲軸 |
| -------------------- | --------------------------- | -------------- | ------------ |
| slug 短於寬度上限    | slug 完整                   | slug 與 topic  | 無           |
| slug 超過寬度上限    | slug 截斷                   | slug 與 topic  | 無           |
| 多份來源討論         | 首份 slug+「+N」籤,單行     | 各籤 slug 與 topic | 無       |

### Requirement: 詳情頁的封存與刪除依階段守門

變更詳情頁「⋯」選單的「封存」項 SHALL 僅於該 change 派生階段為已就緒時可選；非已就緒時 SHALL 呈現停用並於項目下方以灰字說明原因（載明任務進度與「完成後才能封存」的出路）。已就緒時標題列另 SHALL 呈現「封存」主要鈕，與選單項同一守門。「刪除」項 SHALL 置於選單底部、以警示色呈現，僅於派生階段為提案中時可選；非提案中時 SHALL 呈現停用並說明原因（載明已有開工痕跡與「先退回提案中」的出路）。remote session 下對應能力缺失時，能力缺失原因 SHALL 優先於階段原因呈現。守門為呈現層過濾：可按期間階段已於外部改變（併發寫入）時，引擎拒絕 SHALL 為最終裁決，依既有失敗 toast 語意呈現。桌面 SHALL NOT 提供任何略過守門的強制通道。

#### Scenario: 非已就緒的封存項停用附原因

- **WHEN** 使用者開啟提案中（0/11）與進行中（5/19）change 的詳情頁並展開「⋯」選單
- **THEN** 兩者「封存」項皆停用，項目下方各載明任務進度與完成後才能封存；標題列無「封存」主要鈕。已就緒 change 的「封存」項與標題列主要鈕照常可按並走既有確認流程

#### Scenario: 非提案中的刪除項停用附原因

- **WHEN** 使用者開啟進行中 change 的詳情頁並展開「⋯」選單
- **THEN** 「刪除」項停用，項目下方載明已有開工痕跡並指向先退回提案中；提案中 change 的「刪除」項照常可按

#### Scenario: 併發階段變化由引擎拒絕保底

- **WHEN** 封存可按期間另一 session 於 tasks.md 取消勾選一個任務，使用者隨後確認封存
- **THEN** 引擎拒絕封存，app 依既有失敗 toast 語意呈現拒絕訊息，change 仍在看板、詳情頁維持開啟

### Requirement: 變更詳情頁的標頭與狀態卡

變更詳情頁的內容欄 SHALL 置中、寬 768px，由上而下為：標題列、出身列、「狀態」卡、分析結果卡（有分析結果時）、內容卡（分頁列為其標頭）。標題列 SHALL 為 22px 等寬變更名（單行截斷）緊跟複製名稱鈕與章籤，同列右端為動作區：「分析」、依派生階段的鈕（進行中「退回提案中」、已就緒「封存」、remote 可認領時「認領」）與「⋯」選單。審查或驗證狀態非 `none` 時 SHALL 以狀態籤呈現章籤（審查在前、驗證在後，圖示＋狀態詞，i18n 沿用既有詞條）；蓋章日期與蓋章者完整識別（含 email）SHALL 收進主題化提示，可視文字 SHALL NOT 直出 email 與日期；狀態為 `none` 時不渲染章籤。標題列與出身列 SHALL NOT 顯示任務計數——任務進度由狀態卡與任務分頁徽章承載。

出身列 SHALL 為單行灰字，依序呈現：建立者頭像＋名字、產生工具、建立相對時間、「開工 YYYY-MM-DD」（有開工戳記時）、「來自」與來源討論籤（有來源討論時）、「同源」與同源變更籤（有同源變更時）；缺席的資訊段 SHALL 整段缺席而非留白。建立者與開工者的完整識別（含 email）SHALL 收進主題化提示，可視文字 SHALL NOT 直出 email。標題列與出身列於任何資料組合下 SHALL 維持單行，SHALL NOT 折行、SHALL NOT 使頁出現水平捲軸。

「狀態」卡 SHALL 為三列細線分隔、左標籤右內容：「任務進度」列呈現進度條與 `n/m · p%`；「品質關卡」列呈現各站章籤各帶蓋章日期（`inReview`／`inVerify` 僅狀態詞；兩站皆 `none` 時灰字「尚未經過品質關卡」）；「排程」列於清單項帶 `wave` 時呈現「第 N 波」與前置說明（`blockedBy` 空為「可以開工」、非空為「等 N 項」），缺 `wave` 時沿用排程分頁的說明詞條。

#### Scenario: 標頭結構與任務計數位置

- **WHEN** 使用者開啟任一變更的詳情頁
- **THEN** 內容欄由上而下為標題列、出身列、「狀態」卡、內容卡；標題列與出身列可視文字不含任務計數，狀態卡「任務進度」列與任務分頁徽章呈現任務進度

#### Scenario: 章籤與品質關卡列

- **WHEN** 使用者開啟 reviewStatus 為 reviewed 且 verifyStatus 為 verified、兩站蓋章者皆含 email 的變更詳情頁
- **THEN** 標題列顯示「已審查」「已驗證」兩枚章籤；狀態卡「品質關卡」列顯示兩枚章籤各帶蓋章日期；將指標停留於標題列章籤時，主題化提示顯示該站蓋章日期與含 email 的完整識別；標題列維持單行、頁無水平捲軸

#### Scenario: 品質關卡列的空態與進行中

- **WHEN** 使用者分別開啟 reviewStatus 與 verifyStatus 皆為 none、以及 reviewStatus 為 inReview 的變更詳情頁
- **THEN** 前者標題列無章籤、「品質關卡」列顯示「尚未經過品質關卡」；後者標題列與「品質關卡」列顯示「審查中」且無日期

#### Scenario: 排程列

- **WHEN** 使用者開啟 wave=2、blockedBy=["add-a"] 的變更詳情頁
- **THEN** 「排程」列顯示「第 2 波」與「等 1 項」；wave 缺席的變更改顯排程分頁的說明詞條

#### Scenario: email 收進提示

- **WHEN** 使用者開啟 created_by 含 email 的變更詳情頁，並將指標停留於建立者名字
- **THEN** 出身列可視文字僅顯示頭像與名字，主題化提示呈現含 email 的完整識別；開工者的完整識別亦僅於提示呈現

#### Scenario: 出身列恆定單行

- **WHEN** 使用者開啟一個有 4 份來源討論、有開工戳記且有同源變更的變更詳情頁
- **THEN** 出身列維持單行——直接顯示出身討論籤與「+N」籤，不折行，頁無水平捲軸

### Requirement: 討論詳情頁的改進標示

討論詳情頁 SHALL 於 kind 為 improve 時顯示改進標示,文案與卡片 tooltip 使用同一 i18n 詞條;一般討論的詳情頁 SHALL NOT 顯示該標示。

#### Scenario: 改進討論詳情頁

- **WHEN** 開啟 kind 為 improve 的討論詳情頁(語言 tw)
- **THEN** 詳情頁顯示「改進討論」標示

#### Scenario: 一般討論詳情頁無標示

- **WHEN** 開啟無 kind 欄位的討論詳情頁
- **THEN** 詳情頁不顯示改進標示,與本變更引入前的渲染一致

### Requirement: 變更與討論詳情頁的宿主為看板

變更詳情頁與討論詳情頁的開啟動作 SHALL 於設定選定狀態的同一次狀態更新內，把宿主頁切回看板——不論開啟入口（看板卡片、系統匣選單與面板、頁內跳轉、封存前確認對話框的「去蓋章」）與開啟當下所在頁面（規格頁、已封存頁、專案設定頁、應用程式設定頁）。此歸位 SHALL 由狀態層的開啟動作保證，SHALL NOT 依賴各呼叫端入口自行切頁。規格詳情頁與已封存詳情頁 SHALL NOT 在此列——其宿主為開啟當下所在的規格頁、已封存頁或手冊頁，開啟時宿主頁面 SHALL 維持不變。

詳情頁開啟期間宿主頁 SHALL 保持掛載但不可見、不可互動（鍵盤與指標皆不可達）；關閉詳情頁後宿主頁 SHALL 以開啟前的捲動位置、搜尋字串與篩選狀態呈現，SHALL NOT 重新載入或重置。

#### Scenario: 已封存頁自系統匣開啟變更詳情

- **WHEN** 使用者身處已封存頁，自系統匣點擊某活躍 change 的「開啟此變更」
- **THEN** 主視窗前置、宿主頁切回看板、該 change 的詳情頁開啟——關閉詳情頁後所見為看板而非已封存頁

#### Scenario: 規格頁開啟討論詳情頁落回看板

- **WHEN** 使用者身處規格頁，經任一入口開啟某活躍討論的詳情頁
- **THEN** 宿主頁切回看板、討論詳情頁開啟

#### Scenario: 已封存頁開啟已封存詳情頁不切頁

- **WHEN** 使用者於已封存頁點擊一筆封存變更列
- **THEN** 已封存詳情頁開啟，宿主頁面維持已封存頁

#### Scenario: 關閉詳情頁後看板狀態保留

- **WHEN** 使用者於看板輸入搜尋字串、啟用一個篩選並把「進行中」欄捲到底，點一張卡開詳情頁，再按 Esc
- **THEN** 看板重新可見且搜尋字串、篩選與該欄捲動位置與開頁前相同；詳情頁開啟期間看板不可見且無法以 Tab 鍵聚焦

### Requirement: 詳情頁文件載入以 skeleton 呈現

詳情頁（變更、規格、討論、已封存）的各文件分頁 SHALL 統一區分文件三態：載入中 SHALL 渲染文件 skeleton（標題條與數行內文條），載入完成且檔案不存在才 SHALL 顯示該分頁的空態文案，載入完成且有內容 SHALL 渲染內容。任何文件分頁 SHALL NOT 於載入中顯示「無文件」類空態文案。

#### Scenario: 文件載入中顯示骨架

- **WHEN** 開啟詳情頁的任一文件分頁且文件內容尚未抵達
- **THEN** 該分頁顯示文件 skeleton，不顯示任何空態文案

#### Scenario: 載入完成檔案不存在

- **WHEN** 文件載入完成且該檔案不存在
- **THEN** skeleton 消失，顯示該分頁既有的空態文案

#### Scenario: 載入完成有內容

- **WHEN** 文件載入完成且有內容
- **THEN** skeleton 消失，內容以既有渲染路徑呈現

### Requirement: 詳情頁與浮層的進出動畫

桌面 app 的詳情頁（變更、討論、規格、已封存）開啟時 SHALL 以 150ms 淡入並自下方 4px 位移到位；關閉時 SHALL 直接移除節點，無退場動畫。浮層（popover、tooltip、對話框、選單）SHALL 沿用元件庫既有的進出動畫。使用者系統偏好 prefers-reduced-motion 時，上述動畫 SHALL 停用，詳情頁與浮層立即出現與消失。動畫 SHALL NOT 改變詳情頁的焦點、關閉與內容行為。

#### Scenario: 詳情頁進場

- **WHEN** 使用者在看板開啟變更詳情頁，再按 Esc 關閉
- **THEN** 詳情頁淡入並自下方微位移到位；關閉時立即消失、看板立即可見

#### Scenario: 減少動態偏好下無動畫

- **WHEN** 系統設定 prefers-reduced-motion 後開啟任一詳情頁
- **THEN** 詳情頁立即出現、無淡入與位移；浮層亦立即出現與消失

### Requirement: 詳情頁的排程分頁

change 詳情頁 SHALL 於「規格」分頁之後提供「排程」分頁（tw 標籤「排程」、en「Plan」）。清單項帶 `wave` 時分頁 SHALL 依序呈現四張卡片（與審查／驗證分頁同一種邊框卡片：標題列與內容區以分隔線分格），data-plan-section 依序為 wave、depends、overlaps、archive：波次卡（「第 N 波」＋同波其他變更名稱清單，無其他時顯示「本波只有這個變更」）；前置卡（標題列右側 SHALL 有狀態徽章——`blockedBy` 空時「可以開工」、非空時「等 N 項」且 title 列出前置名；內容為 dependsOn 每項一列，資料源 capability `setDepends` 為 true 時每列附移除鈕、底部附下拉選單列出候選變更供新增，選擇即寫入；空時「尚無前置」；`blockedBy` 含 `dependsOn` 以外的名稱（舊 server 以目錄級重疊判定阻擋）時 SHALL 另以一列唯讀列出這些名稱（tw「另需等待（server 依重疊判定）：」，不附移除鈕），讓徽章數字對得上卡片內容）；重疊卡（`requirementOverlap` 每項一列：對方變更名、`capability › requirement` 文字、雙方操作各一枚小標籤（用詞與配色同規格分頁的 delta 區段，如 MODIFIED 顯示「修改」；title 為該操作所屬的變更名），`conflict` 為 true 時加「同名衝突」標籤；空時「無重疊」；SHALL NOT 顯示目錄級 `overlaps`）；封存順序卡（`archiveAfter` 每項變更名，下方一句「先封存它們，再對照正式規格重寫同名 requirement 後封存本變更」；其中有變更也把本變更列入它的 `archiveAfter`（互相等待）時 SHALL 另加一句警示（tw「與 {names} 互相等待：兩邊都要先等對方封存，須調整拆分」）；`requirementOverlap` 有 `conflict` 為 true 的列時 SHALL 加一句同名衝突警示（tw「與 {names} 同名衝突：先改掉其中一邊，較晚封存的一方才不會被拒」）；`archiveAfter` 為空且無同名衝突時「可直接封存」）。分頁 SHALL NOT 渲染獨立的「阻擋」段。新增候選 SHALL 為該變更所在名冊中的其他作用中變更、扣除已宣告的前置：資料源提供名冊查詢時（local）SHALL 以查詢結果為準——該變更有 worktree 映射時名冊為其 worktree 副本、無映射時為主 checkout，與前置寫入作用的 store 相同；名冊查詢未完成時 SHALL NOT 渲染新增下拉；資料源未提供名冊查詢（remote，scope 只有一份名冊）或首次查詢（開啟詳情頁、切換變更）失敗時 SHALL 自作用中變更清單派生；看板刷新後的重新查詢失敗時 SHALL 沿用前一次的查詢結果。新增或移除前置 SHALL 經資料源 `setDepends` 寫回該變更 meta 的 depends_on，成功後看板與分頁刷新；引擎拒絕（自依賴、不存在、已封存、成環）時 SHALL 以單行錯誤呈現引擎訊息、畫面不變。capability 為 false 時 SHALL NOT 渲染移除鈕與下拉選單。清單項缺 `wave` 且缺 `dependsOn`（remote 清單未取得 plan：舊 server、依賴成環或請求失敗）時分頁 SHALL 只顯示一句說明（tw「此模式尚未提供排程資訊」），不渲染四卡；缺 `wave` 但帶 `dependsOn`（local plan 成環）時 SHALL 顯示一句說明（tw「依賴成環，先移除一條前置才有排程資訊」）並只渲染前置卡（無狀態徽章；`dependsOn` 為空時照樣顯示「尚無前置」；capability 為 true 時附移除鈕、SHALL NOT 渲染新增下拉），讓使用者解環。`requirementOverlap` 與 `archiveAfter` 缺席但 `wave` 存在（舊 server）時重疊卡與封存順序卡 SHALL 顯示各自的空態文案。判定 SHALL 收斂於 packages/ui 階段派生模組的 planWave／planBlockedBy／planRequirementOverlap／planArchiveAfter 單一入口。

#### Scenario: 排程分頁四段

- **WHEN** 開啟 wave=2、dependsOn=["add-a"]、blockedBy=["add-a"]、requirementOverlap=[{change:"add-b",capability:"desktop-app",requirement:"看板與任務",ownOperation:"MODIFIED",otherOperation:"MODIFIED",conflict:false}]、archiveAfter=["add-b"] 的變更詳情頁並切到排程分頁
- **THEN** 依序顯示波次卡「第 2 波」、前置卡（徽章「等 1 項」、前置列 add-a 附移除鈕、新增下拉）、重疊卡一列（add-b、`desktop-app › 看板與任務`、兩枚「修改」標籤）、封存順序卡（add-b 與提示句），頁面無 data-plan-section="blocked"

#### Scenario: 新增前置寫回

- **WHEN** 於排程分頁的下拉選單選擇 add-c
- **THEN** 該變更 .openspec.yaml 的 depends_on 含 add-c，分頁前置卡出現 add-c，看板順序刷新

#### Scenario: 引擎拒絕新增

- **WHEN** 於排程分頁選擇會成環的變更
- **THEN** 顯示引擎的成環單行錯誤，前置卡不變，meta 檔不變

#### Scenario: 成環時可移除前置

- **WHEN** 清單 planError 非 null，開啟 dependsOn=["add-a"]（無 wave）的變更詳情頁並切到排程分頁
- **THEN** 顯示「依賴成環，先移除一條前置才有排程資訊」與前置卡的 add-a 列（附移除鈕），無狀態徽章、無新增下拉、無波次／重疊／封存順序卡；按移除鈕即寫回

#### Scenario: remote 分頁唯讀

- **WHEN** 於 capability setDepends 為 false 的資料源開啟詳情頁排程分頁
- **THEN** 不出現移除鈕與下拉選單，四卡照常顯示

#### Scenario: worktree 映射時候選來自副本名冊

- **WHEN** 主 checkout 且 worktree 政策開啟，add-dark-mode 已映射到 worktree，add-auth 在分支前建立、add-late 在分支後才於主 checkout 建立，開啟 add-dark-mode 的排程分頁
- **THEN** 新增下拉選單含 add-auth、不含 add-late，看板清單仍列出 add-late

#### Scenario: 重新查詢名冊失敗沿用前次結果

- **WHEN** 排程分頁已以名冊 add-a、add-b 呈現候選（作用中變更清單另有 add-d，add-a 已是前置），看板刷新後名冊查詢失敗
- **THEN** 新增下拉的候選仍只有 add-b，不出現 add-d

#### Scenario: 可開工徽章與同名衝突標籤

- **WHEN** 開啟 wave=1、blockedBy=[]、requirementOverlap=[{change:"add-x",capability:"auth",requirement:"登入",ownOperation:"ADDED",otherOperation:"ADDED",conflict:true}]、archiveAfter=[] 的變更詳情頁並切到排程分頁
- **THEN** 前置卡徽章為「可以開工」，重疊卡的 add-x 列帶「同名衝突」標籤與兩枚「新增」標籤，封存順序卡顯示「與 add-x 同名衝突：先改掉其中一邊，較晚封存的一方才不會被拒」且不顯示「可直接封存」

#### Scenario: 舊 server 缺新欄位的空態

- **WHEN** remote 清單項帶 wave 與 blockedBy 但無 requirementOverlap 與 archiveAfter
- **THEN** 重疊卡顯示「無重疊」、封存順序卡顯示「可直接封存」，其餘兩卡照常

#### Scenario: 舊 server 的重疊阻擋列在前置卡

- **WHEN** remote 清單項 wave=2、dependsOn=[]、blockedBy=["add-a"]（舊 server 把先配置、共用 capability 的 add-a 算進阻擋），無 requirementOverlap 與 archiveAfter
- **THEN** 前置卡徽章「等 1 項」、內容「尚無前置」與一列「另需等待（server 依重疊判定）：add-a」，該列無移除鈕；本機與新 server 的 blockedBy 只含宣告前置，不出現這一列

#### Scenario: 互相等待時封存卡警示

- **WHEN** add-a 的 archiveAfter=["add-b"]、add-b 的 archiveAfter=["add-a"]，開啟 add-a 的排程分頁
- **THEN** 封存順序卡列 add-b 與提示句，另顯示「與 add-b 互相等待：兩邊都要先等對方封存，須調整拆分」；單向的 archiveAfter 不出現這句警示

#### Scenario: 成環時環外變更的前置卡空態

- **WHEN** 清單 planError 非 null，開啟 dependsOn=[]（無 wave）的環外變更詳情頁並切到排程分頁
- **THEN** 顯示「依賴成環，先移除一條前置才有排程資訊」與前置卡「尚無前置」，無狀態徽章、無新增下拉

### Requirement: 詳情頁的工單分頁

change 詳情頁 SHALL 於「排程」分頁之後提供兩個條件式分頁：清單項 `reviewStatus` 為 `inReview` 時 SHALL 出現「審查」分頁（tw「審查」、en「Review」），`verifyStatus` 為 `inVerify` 時 SHALL 出現「驗證」分頁（tw「驗證」、en「Verify」），審查分頁在前；兩者皆非時 SHALL NOT 渲染這兩個分頁，分頁列維持提案／設計／任務／規格／排程。分頁內容 SHALL 為該站工單的結構化呈現：段標題含站名、「第 N 輪」（N 為末輪序號）、末輪階段詞與末輪三級計數（CRITICAL／WARNING／SUGGESTION 各自的條數，標籤維持英文）；輪次依序號升序逐輪可收合，末輪預設展開、其餘預設收合；每輪標題含「第 N 輪」、階段詞（`phase` 為 `discovery` 顯示 tw「首輪」en「First pass」、`validation` 顯示 tw「複驗」en「Re-check」、`null` 不顯示階段詞）、「範圍 N 檔」（展開後列出 scope 路徑，原樣顯示、不做平台轉換）與 findings 條數；每條 finding 一列，含嚴重度色章（CRITICAL 紅、WARNING 琥珀、SUGGESTION 灰）、路徑（等寬字）與描述原文（不翻譯、不截斷）；描述以結構 token `(accepted)` 結尾時 SHALL 去掉 token 並附「已接受」籤，token 出現於行中時 SHALL 原樣顯示；零 findings 的輪 SHALL 顯示「本輪無發現」。工單 SHALL 於分頁出現時載入，並隨外部檔案變更重新載入；載入三態依「詳情頁文件載入以 skeleton 呈現」；載入完成而無工單（不存在、遠端 404、讀取或解析失敗）時 SHALL 顯示分頁空態「工單尚未抵達或已被刪除」而非錯誤。工單資料 SHALL 為 `{ rounds: [{ index, phase, patchHash, scope, findings: [{ severity, path, text }] }] }`，欄位名與型別與 CLI `speclink review show --json` 的 `rounds` 相同；本機（含 worktree 覆蓋層）與遠端、活變更與已封存 SHALL 回同一形狀。遠端活變更 SHALL 以 `GET /changes/{name}/review`／`verify` 取得結構化輪次，SHALL NOT 放寬 `artifact cat` 的 artifact 名白名單。看板刷新後對應狀態翻為非進行中時，該分頁 SHALL 消失；使用者正停在該分頁時 SHALL 切回「提案」分頁。分頁 SHALL 為唯讀，SHALL NOT 提供蓋章、放棄或追加輪的操作。

#### Scenario: 審查工單分頁出現並結構化呈現

- **WHEN** 開啟 `reviewStatus: "inReview"`、`verifyStatus: "none"` 的變更詳情頁，工單含 Round 1（`phase: "discovery"`，scope 3 檔，findings 兩條 WARNING）與 Round 2（`phase: "validation"`，scope 3 檔，findings 一條 WARNING 描述結尾 `(accepted)`、一條 SUGGESTION）
- **THEN** 分頁列為提案／設計／任務／規格／排程／審查，無「驗證」分頁；切到「審查」分頁後段標題顯示「審查 · 第 2 輪（複驗）CRITICAL 0 · WARNING 1 · SUGGESTION 1」；第 2 輪展開列兩條 finding，WARNING 那條附「已接受」籤且描述不含 `(accepted)`；第 1 輪收合、標題顯示「第 1 輪 · 首輪 · 範圍 3 檔 · 2 條」

##### Example: 階段詞與 accepted token

| round.phase | 顯示階段詞（tw） | finding.text | 顯示描述 | 已接受籤 |
| ----------- | ---------------- | ------------ | -------- | -------- |
| discovery | 首輪 | Correctness: 未處理空清單 | Correctness: 未處理空清單 | 無 |
| validation | 複驗 | Standards: 命名不一致 (accepted) | Standards: 命名不一致 | 有 |
| null | （不顯示） | Standards: (accepted) 語意不清 | Standards: (accepted) 語意不清 | 無 |

#### Scenario: 兩站皆進行中出現兩個分頁

- **WHEN** 開啟 `reviewStatus: "inReview"` 且 `verifyStatus: "inVerify"` 的變更詳情頁
- **THEN** 分頁列於「排程」後依序為「審查」「驗證」，各自載入對應站的工單，兩分頁內容互不混入

#### Scenario: 無工單時分頁缺席

- **WHEN** 開啟 `reviewStatus: "reviewed"`、`verifyStatus: "none"` 的變更詳情頁
- **THEN** 分頁列維持提案／設計／任務／規格／排程五個，無「審查」與「驗證」分頁，且未發出工單讀取請求

#### Scenario: 工單被刪時分頁退場

- **WHEN** 使用者停在 `inReview` 變更的「審查」分頁，其後工單被蓋章刪除、看板刷新後該變更 `reviewStatus` 翻為 `reviewed`
- **THEN** 「審查」分頁自分頁列消失，詳情頁切回「提案」分頁，狀態列章籤改為「已審查」

#### Scenario: 遠端工單 404 顯示空態

- **WHEN** 於 remote 資料源開啟 `inReview` 變更詳情頁並切到「審查」分頁，server 對 `GET /changes/{name}/review` 回 404
- **THEN** 分頁仍在，skeleton 消失後顯示「工單尚未抵達或已被刪除」，無錯誤彈窗

#### Scenario: 工單追加輪後分頁更新

- **WHEN** 「審查」分頁開啟中，外部以 `speclink review add-round` 追加 Round 3
- **THEN** 檔案變更觸發重新載入，段標題改為「第 3 輪」，第 3 輪展開、第 2 輪與第 1 輪收合

### Requirement: 詳情頁的導覽與返回

詳情頁開啟時主區頂列的麵包屑 SHALL 為「專案名 / 頁名 / 項目名」三段：專案名與頁名 SHALL 可點、點擊 SHALL 關閉詳情頁回宿主頁；項目名 SHALL 以等寬字型呈現且不可點。麵包屑右端 SHALL 呈現 ‹ n／m ›：m 為宿主清單目前呈現順序的項目數、n 為目前項目的序位，順序 SHALL 為宿主清單的呈現順序含搜尋與篩選——看板的變更依欄序（提案中、進行中、已就緒）再依欄內順序，看板的討論為上區全卡依序、其後接欄底「已轉出」細列；到頭時對應鈕 SHALL 停用，SHALL NOT 循環；目前項目不在序列中時整組 SHALL 缺席。快捷鍵 ⌥↑／⌥↓（非 macOS 為 Alt+↑／Alt+↓）SHALL 等效於 ‹ ›，Esc SHALL 關閉詳情頁；三者於焦點在文字輸入（input、textarea、contenteditable）時 SHALL NOT 作用，且 SHALL NOT 攔截已被浮層（選單、對話框、提示）處理掉的 Esc。切換項目 SHALL 經與開啟相同的狀態層動作（互斥與浮層收掉規則同樣適用），切換後頁 SHALL 自頂端呈現；封存、刪除或退回的確認在途時切換 SHALL NOT 作用。詳情頁開啟時經專案欄、應用程式選單或系統匣切換主區頁面 SHALL 先關閉詳情頁再呈現該頁。

#### Scenario: 麵包屑三段與返回

- **WHEN** 使用者於專案 speclink 的看板點開變更 add-auth 的詳情頁，再點麵包屑的「變更」
- **THEN** 開頁時麵包屑為「speclink / 變更 / add-auth」且 add-auth 為等寬不可點；點「變更」後詳情頁關閉、看板可見

#### Scenario: ‹ › 依看板順序含過濾

- **WHEN** 看板搜尋後提案中欄剩 a、b，進行中欄剩 c，使用者開啟 b 的詳情頁
- **THEN** 麵包屑右端顯示「2 / 3」；按 › 切到 c、顯示「3 / 3」且 › 停用；按 ‹ 兩次回到 a 且 ‹ 停用

#### Scenario: 快捷鍵與焦點在輸入時不作用

- **WHEN** 使用者於變更詳情頁按 ⌥↓、再把焦點放進排程分頁的下拉輸入後按 ⌥↓ 與 Esc
- **THEN** 第一次 ⌥↓ 切到下一個變更；焦點在輸入時 ⌥↓ 與 Esc 皆不切換、不關頁

#### Scenario: Esc 先給浮層

- **WHEN** 使用者於詳情頁開啟「⋯」選單後按 Esc，再按一次 Esc
- **THEN** 第一次 Esc 只關閉選單、詳情頁維持；第二次 Esc 關閉詳情頁

#### Scenario: 切換主區頁面關閉詳情頁

- **WHEN** 使用者停在變更 add-auth 的詳情頁，點專案欄的「規格」
- **THEN** 詳情頁關閉、主區為規格頁，麵包屑為「專案名 / 規格」

#### Scenario: 目前項目被外部刪除

- **WHEN** 使用者停在變更 x 的詳情頁，外部以 CLI 刪除 x 後看板刷新
- **THEN** 詳情頁關閉、看板可見且不含 x；過程無錯誤彈窗

### Requirement: 詳情頁的文件動作

變更詳情頁與討論詳情頁的內容卡標頭右端 SHALL 提供「開啟 ▾」選單，對目前分頁對應的檔案提供：以預設 App 開啟、在 Finder 顯示（標籤隨平台：Finder／檔案總管／檔案管理員）、複製路徑；選單首列 SHALL 以等寬灰字顯示該檔案相對所在 checkout 根的路徑。分頁與檔案的對應 SHALL 為：提案 `openspec/changes/<name>/proposal.md`、設計 `design.md`、任務 `tasks.md`、規格 `specs/<capability>/spec.md`（目前選定的 capability）、審查 `review.md`、驗證 `verify.md`；討論各分頁皆為該討論的記錄檔。檔案的實際位置 SHALL 與詳情頁讀取內容的位置相同：綁定 worktree 的變更指向 worktree 副本、spec 目錄名依專案設定、已封存的討論指向 `openspec/discussions/archive/<封存日>-<slug>.md`。沒有對應檔案的分頁（排程）SHALL NOT 顯示選單。remote 專案無本機 checkout 時選單 SHALL 只有「複製內容」（該分頁已載入的文件原文；未載完時項目停用）且首列路徑缺席；工單分頁（審查、驗證）不載原文，此時 SHALL NOT 顯示選單。「以預設 App 開啟」SHALL 只開啟 markdown 文件：傳入的檔名與符號連結的實際指向皆須為 markdown。動作失敗時 SHALL 以既有失敗 toast 語意呈現錯誤，SHALL NOT 彈出錯誤視窗。

#### Scenario: 提案分頁的開啟選單

- **WHEN** 使用者於本機專案 /work/app 的變更 add-auth 詳情頁提案分頁展開「開啟 ▾」並選「複製路徑」
- **THEN** 選單首列為 `openspec/changes/add-auth/proposal.md`，其後三項為以預設 App 開啟、在 Finder 顯示（macOS）、複製路徑；剪貼簿寫入 `/work/app/openspec/changes/add-auth/proposal.md`

#### Scenario: worktree 中的變更指向 worktree 副本

- **WHEN** 變更 add-auth 綁定 worktree /work/app-wt/add-auth，使用者於其詳情頁任務分頁展開「開啟 ▾」並選「在 Finder 顯示」
- **THEN** 選單首列為 `openspec/changes/add-auth/tasks.md`，Finder 顯示 `/work/app-wt/add-auth/openspec/changes/add-auth/tasks.md`（詳情頁所讀的那一份）

#### Scenario: 已封存討論指向封存檔

- **WHEN** 使用者自變更詳情頁的來源討論籤開啟已封存的討論 old-topic（封存檔為 `openspec/discussions/archive/2026-01-03-old-topic.md`）並展開「開啟 ▾」
- **THEN** 選單首列為 `openspec/discussions/archive/2026-01-03-old-topic.md`，複製路徑得到該封存檔的絕對路徑

#### Scenario: 排程分頁無選單

- **WHEN** 使用者切到排程分頁
- **THEN** 內容卡標頭右端不顯示「開啟 ▾」

#### Scenario: remote 只剩複製內容

- **WHEN** 使用者於無本機 checkout 的 remote 專案開啟討論詳情頁並展開「開啟 ▾」
- **THEN** 選單只有「複製內容」且無路徑首列；選後剪貼簿為該討論記錄的原文

#### Scenario: 在 Finder 顯示失敗以 toast 呈現

- **WHEN** 使用者選「在 Finder 顯示」而該檔案已被外部刪除
- **THEN** 以失敗 toast 呈現 opener 回報的錯誤，詳情頁維持開啟，無錯誤視窗

### Requirement: 任務列整列可點

變更詳情頁任務分頁的每個任務列 SHALL 整列為其勾選框的 label：點擊列上任一處（勾選框、編號、文字、空白）SHALL 切換該任務的完成狀態並依既有語意寫回；拖曳把手 SHALL NOT 觸發切換；拖選列上文字後放開 SHALL NOT 觸發切換（直接點勾選框不受此限）；雙擊選字時兩下點擊 SHALL 互相抵銷、勾選狀態不變。唯讀檢視（已封存）與批次操作或拖排寫回進行中的列 SHALL NOT 回應點擊；單一任務的勾選寫回不鎖列（勾選後立即取消不遺失）。勾選框 SHALL 保留 checkbox 角色、「任務 N」標籤與空白鍵切換；拖曳仍 SHALL 僅由把手啟動，拖曳進行中按 Esc SHALL 只取消拖曳、SHALL NOT 關閉詳情頁。有寫入能力時工具列 SHALL 附一行灰字提示「點整列即可勾選；拖曳把手改順序」；無拖排能力（不顯示把手）時提示 SHALL 為「點整列即可勾選」。

#### Scenario: 點任務文字切換

- **WHEN** 使用者點某未完成任務列的文字部分
- **THEN** 該任務切換為完成並寫回 tasks.md，效果與點勾選框相同

#### Scenario: 點把手不切換

- **WHEN** 使用者點某任務列的拖曳把手
- **THEN** 勾選狀態不變

#### Scenario: 選取任務文字不切換

- **WHEN** 使用者在某任務列的文字上拖選一段文字後放開
- **THEN** 勾選狀態不變、不發出寫回，選取的文字可複製

#### Scenario: 唯讀與批次寫回中不回應

- **WHEN** 使用者於已封存變更的任務分頁點任務列，或於「全部已完成」等批次操作、拖排的寫回尚未完成時點任務列
- **THEN** 勾選狀態不變、不發出寫回

## MODIFIED Requirements

### Requirement: 桌面 app 提供動詞操作面

桌面 app SHALL 讓使用者對選定 change 執行 status、validate、analyze、archive，並對專案執行 list、show，全部經內嵌 core 執行。動詞的可觀察結果（成功資料、失敗訊息與失敗語意）SHALL 與對應 CLI 指令一致；失敗時 app SHALL 於 UI 呈現 core 的錯誤訊息，SHALL NOT 靜默吞掉失敗。

詳情頁的動作區 SHALL 以單一「分析」按鈕同時觸發 validate 與 analyze，SHALL NOT 提供獨立的「驗證」按鈕；結果呈現於該 change 詳情頁內、「狀態」卡之下的分析結果卡（分析面板）。分析面板 SHALL 依序呈現：

- 結構驗證列：validate 通過時呈單列通過標示（成功語意配色）；失敗時呈錯誤數並逐條列出錯誤訊息（與 speclink validate 的輸出一致）。
- 維度摘要卡：Coverage、Consistency、Ambiguity、Gaps 四維度各一張摘要卡，維度名以繁體中文呈現（覆蓋度、一致性、模糊度、缺漏），卡上顯示該維度發現數——零發現呈「無問題」（成功語意配色）、非零呈「N 個問題」（警示語意配色）。
- 發現卡：逐條發現各一張卡，呈現嚴重度徽章、來源檔（location）、摘要（summary）與建議行（recommendation），對應 speclink analyze 的 --json 輸出欄位。

分析面板 SHALL 可關閉：再次點按「分析」按鈕或面板的關閉鈕皆 SHALL 收合面板；收合後再點按「分析」SHALL 重新執行並展開。面板狀態 SHALL NOT 跨 change 沿用（切換 change 即清空）。視窗頂欄 SHALL NOT 承載任何操作結果訊息面；看板全域操作的結果回饋 SHALL 依「看板全域操作成功靜默、失敗以 toast 浮層呈現」需求所定語意呈現。

#### Scenario: 分析一鍵呈現驗證列與四維度發現卡

- **WHEN** 使用者於某 change 的詳情頁點按「分析」
- **THEN** 頁內展開分析面板：頂部結構驗證列（通過或錯誤數）、四張繁體中文維度摘要卡（各帶發現數）、逐條發現卡（嚴重度徽章、來源檔、摘要、建議行），內容對應 speclink validate 與 speclink analyze 的 --json 輸出

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
- **THEN** app 以 toast 呈現 core 回報的失敗訊息，不將該 change 標為已歸檔

### Requirement: 外部變更即時反映

<!-- REMOVED-SCENARIO: 外部蓋開工章後抽屜出現開工歸屬 -->
<!-- REMOVED-SCENARIO: 外部推進討論後抽屜內容更新 -->

桌面 app SHALL 監看目前專案的 openspec/ 目錄樹：app 之外的寫者（CLI、agent、手動編輯器）修改其下任何文件後，看板、詳情頁、討論詳情頁與已封存頁 SHALL 在短時間內（秒級）自動更新呈現；已開啟檢視中「已載入的內容」——任務清單勾選狀態、proposal／design／specs 文件原文、meta 開工歸屬、討論記錄各分頁——SHALL 重載至磁碟現況，SHALL NOT 要求重啟、重開詳情頁或任何 app 內操作。使用者互動進行中（任務勾選寫回、拖曳排序）時外部觸發的內容重載 SHALL 讓路，互動結束後 SHALL 補一次重載至磁碟現況——SHALL NOT 打斷或蓋掉進行中的操作；重載回應交錯時 SHALL 以最新一次為準，較舊回應 SHALL NOT 覆蓋較新內容。app 內操作（勾任務、拖曳、動詞）後 SHALL 重載受影響的已載入內容——含任務清單與 meta。監看不可用時（如檔案系統權限）app SHALL 照常提供其餘功能——僅失去自動刷新，SHALL NOT 崩潰或反覆彈出錯誤。

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

### Requirement: 看板欄位由生命週期標記驅動

看板欄位 SHALL 依下列優先序判定：任務全完成（總數大於 0 且完成數等於總數）＝已就緒；meta 含 started_at 或任務完成數大於 0＝進行中；其餘＝提案中。剛完成 propose（有任務、全未勾、未標記開工）的 change SHALL 顯示於提案中欄。詳情頁 SHALL 於 meta 含 started_at 時顯示開工者與開工日（started_by、started_at）；meta 無 started_at 時 SHALL NOT 顯示開工資訊——即使該 change 因任務進度列於進行中欄（派生管顯示，歸屬缺席維持缺席）。此判定 SHALL 對 fs 與 remote 資料來源一致：remote 清單 payload 的 startedAt SHALL 進入看板與系統匣的欄位推導，SHALL NOT 以任務完成數替代開工判定（完成數大於 0 的 fallback 保留，涵蓋繞過工具的寫入路徑）。

#### Scenario: 未開工的 change 留在提案中

- **WHEN** 某 change 的 tasks.md 含 28 項任務全未勾、meta 無 started_at
- **THEN** 看板將其顯示於「提案中」欄，卡片任務數為 0/28

#### Scenario: 標記開工後移入進行中

- **WHEN** 對上述 change 執行 speclink in-progress add 後看板更新
- **THEN** 該卡片移至「進行中」欄，詳情頁標頭顯示開工者與開工日

#### Scenario: 無章而有任務進度列於進行中

- **WHEN** 某 change 的 meta 無 started_at，其 tasks.md 經任意途徑（如編輯器直接修改後 git pull 或本機儲存）成為 3/28 已勾，看板刷新
- **THEN** 該卡片顯示於「進行中」欄，詳情頁不顯示開工者與開工日

#### Scenario: remote 已開工零進度列於進行中

- **WHEN** remote 連線的 change 清單中某 change 帶 startedAt 且任務進度為 0/15，看板與系統匣刷新
- **THEN** 該卡片顯示於「進行中」欄、系統匣列於進行中分區——不因完成數為 0 而落回提案中

##### Example: 欄位判定矩陣

| meta started_at | 任務進度 | 看板欄 |
| --------------- | -------- | ------ |
| 無 | 0 任務 | 提案中 |
| 無 | 0/28 | 提案中 |
| 無 | 3/28 | 進行中（詳情頁無開工資訊） |
| 有 | 0/28 | 進行中 |
| 有 | 13/28 | 進行中 |
| 無 | 28/28 | 已就緒（全完成優先） |
| 有 | 28/28 | 已就緒 |

### Requirement: 任務清單拖放排序與自動重編號

詳情頁任務分頁的每個任務列 SHALL 於列首提供拖曳把手，使用者拖曳把手放開後 SHALL 一次到位完成排序並回寫 tasks.md——僅 checkbox 行被搬移，群組標題與其他行 SHALL NOT 被搬動。寫回時 SHALL 重寫受影響任務的編號前綴：文字以「數字.數字」＋空白開頭、且位於帶「數字.」前綴標題之群組下的任務行，其前綴 SHALL 重寫為「群組編號.組內序」；不符該樣式的任務行與所有非 checkbox 行 SHALL 逐字元保留。拖曳 SHALL 僅由把手觸發——點擊核取方塊或任務文字 SHALL NOT 啟動拖曳。群組標題 SHALL 參與拖曳中的讓位序列——被越過任務的讓位視覺 SHALL NOT 穿越其群組標題（群組歸屬在拖曳預覽中保持可讀）；群組標題為「組界槽」落點——把手放開於群組標題上時：來自標題上方的任務 SHALL 成為該群組的第一個任務；來自標題下方的任務 SHALL 移到標題之前、成為上一群組的末任務（雙向皆重編號；標題該側無任務可錨定時不提供落點）。逐格上下移動按鈕 SHALL NOT 再提供。封存唯讀檢視 SHALL NOT 渲染把手。

#### Scenario: 組內拖放後編號重寫

- **WHEN** 使用者把群組 1 的首任務拖到該群組末位放開
- **THEN** tasks.md 該群組的 checkbox 行依新順序排列，編號前綴依序重寫為 1.1、1.2、…（被拖任務取得末位編號），群組標題與其他群組逐字元不變

##### Example: 組內移動重編號

| 拖放前（群組 1） | 拖放後（把 1.1 甲拖到末位） |
| ---------------- | --------------------------- |
| - [ ] 1.1 甲 | - [x] 1.1 乙 |
| - [x] 1.2 乙 | - [ ] 1.2 丙 |
| - [ ] 1.3 丙 | - [ ] 1.3 甲 |

#### Scenario: 跨群組拖放取得新群組編號

- **WHEN** 使用者把群組 1 的末任務拖到群組 2 首任務之上放開（落點為該任務）
- **THEN** 該任務插於其後、前綴重寫為 2.2，群組 2 其餘任務依序後移（2.3…），群組 1 剩餘任務依序重排（1.1、1.2…）

#### Scenario: 拖放到群組標題成為組首

- **WHEN** 使用者把群組 1 的任務拖放到「## 2.」群組標題上
- **THEN** 該任務成為群組 2 的第一個任務（前綴重寫為 2.1），原組首任務後移為 2.2，群組 1 剩餘任務依序重排

##### Example: 標題落點成組首

| 拖放前 | 拖放後（把 1.2 乙拖到「## 2. 後段」標題上） |
| ------ | ------------------------------------------- |
| ## 1. 前段：1.1 甲、1.2 乙 | ## 1. 前段：1.1 甲 |
| ## 2. 後段：2.1 丙、2.2 丁 | ## 2. 後段：2.1 乙、2.2 丙、2.3 丁 |

#### Scenario: 組首任務拖回標題之前回到上一群組末位

- **WHEN** 使用者把群組 2 的第一個任務（原自群組 1 拖入）拖放到「## 2.」群組標題上
- **THEN** 該任務移到標題之前、成為群組 1 的末任務並重編號（如群組 1 原有 5 項則成為 1.6），群組 2 其餘任務依序前移補位

##### Example: 標題落點雙向

| 操作 | 結果 |
| ---- | ---- |
| 把 1.2 乙拖到「## 2. 後段」標題上（來自上方） | 乙成為 2.1 組首 |
| 再把 2.1 乙拖回「## 2. 後段」標題上（來自下方） | 乙回到群組 1 末位（1.2） |

#### Scenario: 拖曳讓位不穿越群組標題

- **WHEN** 使用者拖曳任務向下越過相鄰群組的首任務（拖曳進行中、尚未放開）
- **THEN** 該首任務的讓位視覺仍位於其群組標題之下方——SHALL NOT 出現在標題上方（上一群組的區域內）

#### Scenario: 無編號前綴的任務文字不被改寫

- **WHEN** 使用者拖動一個文字不以「數字.數字」開頭的任務到新位置
- **THEN** 該任務僅位置改變、文字逐字元保留，其他符合樣式的任務仍正常重編號

#### Scenario: 點擊核取方塊不誤觸拖曳

- **WHEN** 使用者在任務核取方塊上按下並於拖曳啟動閾值內放開
- **THEN** 勾選狀態切換且任務順序不變，無拖曳視覺出現

#### Scenario: 封存唯讀檢視無把手

- **WHEN** 使用者展開已封存變更的任務分頁
- **THEN** 任務列無拖曳把手、無法拖動，核取方塊維持不可點擊

### Requirement: 討論於看板第 0 欄兩級呈現

看板 SHALL 於最左新增「討論」欄，欄內分上下兩區同屏呈現：上區為討論中（active）全卡清單，下區為欄底的「已轉出」常駐收合列；SHALL NOT 以互斥檢視切換兩者。分區判準 SHALL 為「引擎會自動收走的討論才收合」：已轉出（promoted）、已有結論（concluded 為 true）且未保留（hold 非 true）的討論收進欄底收合列；已轉出但尚無結論（concluded 為 false）的討論 SHALL 留在上區為全尺寸卡；已轉出、已有結論但保留在途（hold 為 true）的討論 SHALL 同樣留在上區為全尺寸卡。討論清單資料的 concluded 欄位缺席（向後相容路徑）時 SHALL 退回既有判準——promoted 一律收進收合列，SHALL NOT 把缺席當成尚無結論；hold 欄位缺席時 SHALL 視同未保留，SHALL NOT 把缺席當成保留。看板拖排落點清單 SHALL 與上區卡集共用同一判準（上區卡集與可拖集恆等）。

討論中區 SHALL 顯示 status 為 open 或 concluded 的討論、已轉出但尚無結論的討論、以及已轉出且保留在途的討論為全尺寸卡——卡 SHALL 以 slug（檔名）為標題（等寬字型）、topic 為卡身描述；複製 slug 鈕 SHALL 行內尾隨於標題最後一個字元後（版面規則見「看板卡片統一解剖學」）；建立者（createdBy，缺席時省略）SHALL 以頭像圓點呈現且 hover 顯示全名；輪數與建立時間 SHALL 並排於卡底 meta 列，狀態 chip 保留；已轉出但尚無結論的卡 SHALL 帶「已轉出・尚無結論」狀態標；已轉出且保留在途的卡 SHALL 帶「已轉出・保留中」狀態標（英文介面為「Promoted · on hold」），與「已轉出・尚無結論」同為 warning 語意色；promoted 且 concluded 為 false 的卡不論 hold 值 SHALL 標「已轉出・尚無結論」。open 卡為唯讀，concluded 卡 SHALL 提供「封存」動詞（「轉為變更」動詞已自 GUI 撤除，轉出改由 CLI 或 agent）；已轉出但尚無結論的卡與已轉出且保留在途的卡 SHALL NOT 提供任何動詞按鈕（前者結論未寫；後者的收尾由使用者以 CLI 的 discuss archive 明示解除）。上區全卡 SHALL NOT 列出衍生變更清單（衍生進度於討論詳情頁檢視）。

當存在至少一筆已有結論且未保留的 promoted 討論時，欄底 SHALL 呈現「已轉出 N ›」收合列（白底、上緣細線、右端 › 於展開時旋轉；N 為已有結論且未保留的 promoted 計數，預設收合）；點按 SHALL 就地展開 promoted 細列清單（白底細框列），再點按即收合；桌面 app SHALL 把展開狀態存於 app 本機並跨啟動保留，SHALL NOT 寫入任何專案目錄。無任何已有結論且未保留的 promoted 討論時收合列 SHALL 缺席。

promoted 細列 SHALL 以討論 slug 為首行（等寬字型）且帶複製 slug 鈕、topic 為次行描述，其下每個 promoted_to 子變更 SHALL 以樹狀前綴（末列 └、其餘 ├）逐列列出名稱與階段 chip。子變更的階段 SHALL 由其於清單中的存在性派生：active 清單命中依看板欄位規則、封存清單命中為已封存、兩者皆無 SHALL 標示為已刪除且討論維持已轉出不回退。階段 chip SHALL 以看板階段配色呈現：提案中、進行中、已就緒各對應該階段欄的 teal 濃度，已封存為中性色，已刪除為 destructive 色加刪除線。

slug 為題與複製鈕（討論全卡標題、promoted 細列首行）屬 openspec/LANGUAGE.md 明載的受控例外。討論欄的計數徽章 SHALL 顯示上區全卡數（open、concluded、已轉出但尚無結論、已轉出且保留在途）。當討論欄上區無任何全卡但存在已有結論且未保留的 promoted 討論時，SHALL NOT 顯示「尚無討論」空狀態（欄底收合列已傳達）。封存的討論 SHALL NOT 出現於此欄。輪數文案 SHALL 使用「N 輪」。

#### Scenario: 進行中與已結論討論的全卡呈現

- **WHEN** openspec/discussions/ 下存在一筆 status: open（3 輪、frontmatter 含 created_by 與 created）與一筆 status: concluded 的討論、無任何 promoted 討論
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

### Requirement: GUI 勾任務與 CLI 完成語意一致

桌面看板勾選任務為完成 SHALL 產生與 speclink task done 相同的檔案效果：tasks.md 該任務勾章、touched-files 記錄（有未被先前任務認領的 git dirty 檔時追加，無則不追加——與 CLI 同語意）、該 change 首次有任務完成且 meta 無 started_* 時蓋開工章（started_at 為當日、started_by 依 git 身分可得性、started_with 缺席）。對已完成任務的重複完成請求 SHALL 視為冪等成功，SHALL NOT 寫入任何檔案、SHALL NOT 對使用者報錯。取消勾選與拖曳排序 SHALL 僅寫 tasks.md，SHALL NOT 寫入 meta 或 touched 記錄。

#### Scenario: 勾選首任務蓋章並移欄

- **WHEN** 使用者於看板勾選某 meta 無 started_* 的 change 的第一項任務
- **THEN** tasks.md 該任務成 [x]，.openspec.yaml 新增 started_at（git 身分可得時含 started_by），看板刷新後卡片移入「進行中」欄且詳情頁顯示開工列

#### Scenario: 取消勾選不動 meta 與 touched

- **WHEN** 使用者取消勾選一項已完成任務
- **THEN** 僅 tasks.md 該行標記變為 [ ]；.openspec.yaml 與 touched 記錄逐字元不變

#### Scenario: 拖曳排序不觸發完成語意

- **WHEN** 使用者拖曳任務改變順序（含跨群組重編號）
- **THEN** 僅 tasks.md 變動；.openspec.yaml 與 touched 記錄逐字元不變

### Requirement: 看板搜尋過濾卡片

看板 SHALL 於頁標題區右端提供搜尋輸入（全圓、寬 280px、帶搜尋圖示）與同列同高的篩選圖示鈕，兩者位於欄位上方，SHALL NOT 佔用第二列版面。輸入含非空白內容時 SHALL 呈現清除鈕（點按清空字串且輸入保持聚焦）與即時命中數（過濾後各欄卡片總數）；快捷鍵（macOS 為 Cmd+F、其他平台為 Ctrl+F）SHALL 聚焦搜尋輸入。

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
- **THEN** 詳情頁正常開啟、封存流程正常觸發，行為與未過濾時一致

#### Scenario: 搜尋字串與篩選不跨啟動保留且與已封存頁獨立

- **WHEN** 使用者於看板輸入字串並啟用任一篩選 chip 後切至已封存頁，再重啟 app
- **THEN** 已封存頁的搜尋輸入不含看板字串；重啟後看板為未過濾狀態、搜尋輸入為空、無啟用中的篩選 chip

### Requirement: markdown 內容保留文件結構呈現

桌面 app 渲染 markdown 內容（變更詳情頁的提案／設計／規格分頁、討論詳情頁各分頁、已封存檢視）SHALL 保留來源的文件結構：無序清單 SHALL 顯示列表符號、有序清單 SHALL 顯示編號、段落之間 SHALL 有可辨識的垂直間距、來源中的單一換行 SHALL 呈現為換行。內容基準字級 SHALL 為 16px，任務分頁的任務文字 SHALL 同為 16px。排版與內容色彩 SHALL 於淺色與深色主題（跟隨系統偏好）一致生效。

#### Scenario: 清單顯示列表符號與編號

- **WHEN** 檢視含無序清單與有序清單的提案文件
- **THEN** 無序清單項目前顯示列表符號、有序清單項目前顯示編號，清單相對內文有縮排

#### Scenario: 單一換行呈現為換行

- **WHEN** 檢視討論記錄的討論過程分頁，其中一輪的 Focus 與 Position 行在來源中各佔一行、以單一換行分隔
- **THEN** 兩行在渲染結果中分行呈現，不塌成同一段連續文字

#### Scenario: 內容基準字級為 16px

- **WHEN** 檢視變更詳情頁的提案分頁與任務分頁
- **THEN** markdown 內文與任務清單文字的計算字級皆為 16px

#### Scenario: 深色主題下排版一致

- **WHEN** 系統偏好為深色時檢視同一份文件
- **THEN** 列表符號、段落間距、字級與淺色主題一致，內容色彩取自深色 token 且可讀

### Requirement: raw HTML 不以原文呈現

桌面 app 渲染 markdown 內容時，來源中的 raw HTML（含 HTML 註解）SHALL NOT 以原始文字出現在渲染結果；渲染 SHALL NOT 修改任何來源檔案。

#### Scenario: 討論記錄的 scaffold 註解不顯示

- **WHEN** 開啟討論詳情頁的討論過程分頁，其來源在 Rounds 區段含 CLI scaffold 產生的 HTML 註解行
- **THEN** 渲染結果不含該註解的任何文字，openspec/ 下的討論記錄檔案內容位元不變

#### Scenario: 程式碼區塊內的 HTML 原文照常顯示

- **WHEN** 檢視在 code fence 內含 HTML 標籤範例的文件
- **THEN** code fence 內容以原文完整顯示（過濾僅及於 code fence 之外的 raw HTML）

### Requirement: 任務分頁提供批次操作工具列

變更詳情頁的任務分頁 SHALL 於任務清單頂部提供工具列：「全部已完成」SHALL 將該變更全部任務標為完成並以單次檔案寫入回寫 tasks.md；「重置任務」SHALL 將全部任務取消勾選並以單次檔案寫入回寫；「下一個未完成」SHALL 使第一個未完成任務捲入可視範圍並短暫高亮，n 快捷鍵 SHALL 等效。批次完成的開工章語意 SHALL 與逐一勾選一致——未開工變更首次完成任務時蓋開工章；重置 SHALL NOT 蓋開工章、SHALL NOT 記錄 touched。目標狀態已達成時重跑 SHALL 為冪等成功且不改檔。全部任務已完成時「全部已完成」與「下一個未完成」SHALL 呈現不可用；唯讀封存檢視 SHALL NOT 顯示工具列。

#### Scenario: 全部已完成單次寫回

- **WHEN** 對含未完成任務的變更按「全部已完成」
- **THEN** tasks.md 全部任務標為完成且僅發生單次檔案寫入，詳情頁進度顯示 100%

##### Example: 批次完成的進度變化

| 操作前 | 操作 | 操作後 |
| ------ | ---- | ------ |
| 3/10 任務完成 | 全部已完成 | 10/10、進度 100% |
| 10/10 任務完成 | 全部已完成 | 不改檔（冪等）、按鈕不可用 |

#### Scenario: 重置任務不蓋開工章

- **WHEN** 對已有完成任務的變更按「重置任務」
- **THEN** tasks.md 全部任務取消勾選且僅發生單次檔案寫入，變更 meta 未新增開工章與 touched 記錄

#### Scenario: 批次完成沿用開工章語意

- **WHEN** 對尚未開工的變更按「全部已完成」
- **THEN** 變更 meta 蓋上開工章（與逐一勾選首次完成任務的行為一致）

#### Scenario: 下一個未完成定位

- **WHEN** 任務清單有多個任務且第一個未完成任務在可視範圍外，按「下一個未完成」或按 n 鍵
- **THEN** 該任務捲入可視範圍並短暫高亮

#### Scenario: 唯讀封存檢視不顯示工具列

- **WHEN** 於已封存頁展開封存變更的任務檢視
- **THEN** 不顯示批次操作工具列，checkbox 維持唯讀

### Requirement: 表單控制項與按鈕以主題化元件呈現

桌面 app 的互動控制項 SHALL 以元件庫的主題化原語呈現：勾選控制項（任務分頁勾選框、初始化對話框的工具多選）SHALL 呈現主題化外觀——未勾選為主題邊框空框、勾選為主色底加勾選圖示，SHALL NOT 呈現作業系統原生控制項外觀；多行文字輸入（設定頁）SHALL 取用設計 token 的邊框、底色與 focus 樣式；動作按鈕 SHALL 統一取用元件庫按鈕變體，鍵盤聚焦時 SHALL 顯示一致的 focus 可視環、disabled 態呈現一致。上述控制項於淺色與深色主題 SHALL 一致取自設計 token。替換 SHALL NOT 改變控制項的行為與無障礙語意：勾選框 SHALL 保留 checkbox 角色、既有標籤與空白鍵切換；按鈕 SHALL 保留既有無障礙名稱、回呼與 disabled 條件；唯讀封存檢視的勾選框 SHALL 維持不可互動。

#### Scenario: 任務勾選框主題化外觀

- **WHEN** 檢視含已完成與未完成任務的任務分頁
- **THEN** 未完成任務的勾選框為主題邊框空框、已完成為主色底加勾選圖示，非作業系統原生繪製，深淺主題下外觀一致

#### Scenario: 勾選框無障礙語意保留

- **WHEN** 以鍵盤或輔助技術操作任務分頁的勾選框
- **THEN** 每個勾選框曝露 checkbox 角色與「任務 N」標籤，空白鍵可切換勾選且觸發與滑鼠點擊相同的寫回

#### Scenario: 初始化對話框工具多選主題化

- **WHEN** 開啟初始化確認對話框
- **THEN** claude 與 codex 選項以主題化勾選框呈現且可獨立勾選，預設勾選狀態與替換前相同

#### Scenario: 設定頁多行輸入主題化

- **WHEN** 於深色主題開啟設定頁的專案說明或產出規則編輯區
- **THEN** 多行輸入的邊框、底色與 focus 樣式取自設計 token，輸入與儲存行為與替換前相同

#### Scenario: 按鈕 focus 可視環一致

- **WHEN** 以 Tab 鍵依序聚焦任務工具列、詳情頁動作列與側欄導覽的按鈕
- **THEN** 每個按鈕顯示一致的 focus 可視環，無障礙名稱與點擊行為與替換前相同

### Requirement: 討論輪以卡片呈現

桌面 app 的討論過程呈現（討論詳情頁的討論過程分頁、已封存討論檢視的討論過程分頁）SHALL 將符合 scaffold 格式（輪標題形如「### Round N — <mode> (<date>)」）的記錄逐輪呈現為獨立卡片：卡頭 SHALL 顯示輪次、mode 與日期；卡身 SHALL 將行首的 Focus／Position／Ruled out／Open 粗體前綴呈現為欄位標籤區塊，前綴原文 SHALL NOT 出現在欄位內文；一個欄位 SHALL 涵蓋其標籤行起至下一個標籤行（或輪結尾）的全部內容。來源缺席的欄位 SHALL NOT 渲染空標籤；mode 值 SHALL 按來源字串透傳呈現（不設白名單）。任一輪標題不符 scaffold 格式時 SHALL 整篇以單一 markdown 檢視退回，不報錯。渲染 SHALL NOT 修改任何來源檔案。

#### Scenario: 標準記錄逐輪成卡

- **WHEN** 開啟含四輪 scaffold 格式記錄的討論過程分頁
- **THEN** 呈現四張輪卡片，各卡頭顯示輪次、mode 與日期；卡身的 Focus 與 Position 以標籤區塊呈現，來源的粗體前綴原文不出現在渲染結果

##### Example: 輪標題解析

- **GIVEN** 來源輪標題「### Round 1 — assumptions (2026-07-08)」
- **WHEN** 開啟討論過程分頁
- **THEN** 該卡片卡頭顯示輪次 Round 1、mode assumptions、日期 2026-07-08

#### Scenario: 缺席欄位不渲染空標籤

- **WHEN** 某輪來源只有 Focus、Position 與 Open 行，無 Ruled out 行
- **THEN** 該卡片呈現 Focus、Position、Open 三個標籤區塊，無 Ruled out 標籤

#### Scenario: 欄位涵蓋後續多行內容

- **WHEN** 某輪的 Position 標籤行之後接數行列點、其後才是 Ruled out 標籤行
- **THEN** 該數行列點全數呈現於 Position 欄位區塊內，列表符號保留

#### Scenario: 非標準格式整篇退回

- **WHEN** 記錄的 Rounds 區段含不符 scaffold 格式的輪標題（手寫或 pre-scaffold 記錄）
- **THEN** 討論過程分頁整篇以單一 markdown 檢視呈現，無卡片、無錯誤訊息，來源檔案位元不變

### Requirement: 討論結論以欄位標籤呈現

桌面 app 的討論結論呈現（討論詳情頁的結論分頁、已封存討論檢視的結論區）SHALL 將行首的 Decision／Rationale／Rejected alternatives／Deferred／Capture to／Next 粗體前綴呈現為欄位標籤區塊（標籤依語系呈現，zh-TW 為決定／理由／否決替代案／擱置／記錄去向／下一步），前綴原文 SHALL NOT 出現在欄位內文；一個欄位 SHALL 涵蓋其標籤行起至下一個標籤行（或結論結尾）的全部內容。來源缺席的欄位 SHALL NOT 渲染空標籤；非六詞白名單的粗體前綴行 SHALL 按一般內文歸屬當前欄位。結論不含任何白名單欄位時 SHALL 整篇以單一 markdown 檢視退回，不報錯。渲染 SHALL NOT 修改任何來源檔案。

#### Scenario: 標準結論欄位成標籤區塊

- **WHEN** 開啟含 scaffold 格式結論（Decision、Rationale、Capture to、Next 各佔一行起頭）的結論分頁
- **THEN** 各欄位以標籤區塊呈現，「**Decision**:」等粗體前綴原文不出現在渲染結果

#### Scenario: 結論缺席欄位不渲染空標籤

- **WHEN** 某結論來源只有 Decision 與 Rationale 行，無 Deferred 行
- **THEN** 結論分頁呈現決定與理由兩個標籤區塊，無擱置標籤

#### Scenario: 自由格式結論整篇退回

- **WHEN** 結論來源為手寫自由段落、不含任何白名單粗體前綴行
- **THEN** 結論分頁整篇以單一 markdown 檢視呈現，無標籤區塊、無錯誤訊息，來源檔案位元不變

### Requirement: markdown 文件內容行寬有上限

<!-- REMOVED-SCENARIO: 全螢幕下行寬不增長且內容欄置中 -->

桌面 app 經共用 markdown 渲染呈現的文件內容（變更詳情頁的提案／設計／規格分頁、討論詳情頁各分頁、規格詳情頁、已封存詳情頁）SHALL 有固定行寬上限；主區寬度改變時內文行寬 SHALL NOT 隨之增長；容器寬於行寬上限時內容欄 SHALL 水平置中、留白均分兩側。同一分頁內的區段標籤、討論輪卡片與任務清單 SHALL 與內文同欄對齊。超過行寬的表格 SHALL 於容器內橫向捲動，版面 SHALL NOT 橫向溢出。

#### Scenario: 主區加寬時行寬不增長且內容欄置中

- **WHEN** 視窗拉寬至 1600px 檢視變更詳情頁的提案分頁
- **THEN** 內文行寬維持固定上限，內容欄水平置中、左右留白均分，區段標籤與內文同欄對齊

#### Scenario: 寬表格於容器內橫捲

- **WHEN** 檢視含超過行寬上限的寬表格的文件
- **THEN** 表格於容器內橫向捲動，頁版面不橫向溢出

### Requirement: 規格分頁 delta 區段以色標呈現

變更詳情頁規格分頁與已封存變更檢視的規格分頁 SHALL 將 delta spec 的區段標題（ADDED／MODIFIED／REMOVED／RENAMED Requirements）呈現為色標區段標頭，原始區段標題行 SHALL NOT 以標題文字直出；區段內的 requirement 與 scenario 內文 SHALL 照 prose 排版呈現。不含 delta 區段標題的規格文件 SHALL 整篇照常渲染。色標配色 SHALL 與 delta 計數徽章（DeltaBadges）一致。

#### Scenario: delta 區段呈現色標標頭

- **WHEN** 檢視含 ADDED 與 MODIFIED 區段的 delta spec
- **THEN** 呈現綠色「新增」與琥珀色「修改」區段標頭，原始「ADDED Requirements」「MODIFIED Requirements」標題文字不出現在渲染結果

##### Example: 四種 delta 區段的色標對應

| 來源區段標題 | 標頭文字 | 色系 |
| ------------ | -------- | ---- |
| ## ADDED Requirements | 新增 | 綠（emerald） |
| ## MODIFIED Requirements | 修改 | 琥珀（amber） |
| ## REMOVED Requirements | 移除 | 紅（red） |
| ## RENAMED Requirements | 更名 | 藍（sky） |

#### Scenario: 無 delta 標記的規格照常渲染

- **WHEN** 檢視不含任何 delta 區段標題的規格文件
- **THEN** 內容整篇照現行 markdown 渲染呈現，無色標標頭

### Requirement: 提案與設計章節以中文標籤呈現

變更詳情頁與已封存檢視的提案／設計分頁 SHALL 將已知模板章節標題（提案側 Why／What Changes／Non-Goals／Capabilities／New Capabilities／Modified Capabilities／Impact／Problem／Root Cause／Proposed Solution／Success Criteria／Summary／Motivation／Alternatives Considered；設計側 Context／Goals / Non-Goals／Decisions／Implementation Contract／Risks / Trade-offs／Migration Plan／Open Questions）呈現為中文標籤區塊，英文模板標題 SHALL NOT 以標題文字直出；標籤款式 SHALL 為粗體大標題——計算字級 SHALL 大於內文基準字級（16px），且 SHALL 與討論側欄位標籤（輪的焦點／立場、結論的決定／理由）及規格分頁色標區段標頭同款式（色標標頭保留各 delta 色彩）。白名單以外的章節標題 SHALL 連同內文照 prose 排版呈現。整份文件無任何白名單章節時 SHALL 整篇以單一 markdown 檢視退回，不報錯。渲染 SHALL NOT 修改任何來源檔案。

#### Scenario: 提案模板章節成中文標籤

- **WHEN** 檢視含 Why、What Changes、Non-Goals、Capabilities、Impact 章節的提案分頁
- **THEN** 呈現「為什麼」「變更內容」「非目標」「能力」「影響」標籤區塊，Why 等英文標題文字不出現在渲染結果

##### Example: 章節對照

| 來源章節標題 | 呈現標籤 |
| ------------ | -------- |
| ## Why | 為什麼 |
| ## What Changes | 變更內容 |
| ## Non-Goals | 非目標 |
| ## Context | 背景 |
| ## Decisions | 決策 |
| ## Risks / Trade-offs | 風險與取捨 |

#### Scenario: 標籤為大標題且字級大於內文

- **WHEN** 檢視提案分頁的章節標籤與討論詳情頁結論分頁的欄位標籤
- **THEN** 兩者款式一致，皆為粗體且計算字級大於內文的 16px

#### Scenario: 白名單外章節照排

- **WHEN** 檢視設計分頁，其 Decisions 章節內含自訂決策標題（如 D1 起頭的三級標題）
- **THEN** 決策標題照 prose 標題樣式渲染，不被標籤化、不被翻譯

#### Scenario: 無白名單章節整篇退回

- **WHEN** 檢視手寫自由格式（無任何模板章節標題）的提案文件
- **THEN** 內容整篇照現行 markdown 渲染呈現，無標籤區塊、無錯誤訊息

### Requirement: 任務群組標題與章節標籤同款式

變更詳情頁任務分頁的群組標題 SHALL 以標籤家族的次級款式呈現——粗體、計算字級與內文基準（16px）一致，與 Capabilities 次級標籤同款、與章節主標題同族但小一級；群組標題文字 SHALL 照來源呈現（不翻譯、不改寫）；任務勾選、拖曳排序與工具列行為 SHALL 不受款式變更影響。

#### Scenario: 群組標題款式一致

- **WHEN** 檢視含群組標題的任務分頁
- **THEN** 群組標題為粗體、計算字級為 16px（與任務文字同級、小於章節主標題），標題文字與來源一致

#### Scenario: 互動行為不變

- **WHEN** 在款式調整後的任務分頁勾選任務並拖曳排序
- **THEN** 勾選與排序行為與調整前一致，寫回結果正確

### Requirement: 變更的來源討論多值呈現

<!-- REMOVED-SCENARIO: 詳情抽屜列出全部來源討論 -->

變更連結多份討論時(meta 的 from_discussion 為逗號清單),變更卡 SHALL 維持單一討論徽章、以清單第一個(出身)討論為代表,徽章提示 SHALL 列出全部來源討論;變更詳情頁的來源討論籤 SHALL 固定顯示清單第一個(出身)討論,其餘 SHALL 收進「+N」數字籤(N 為未直接顯示的數量),點擊 +N SHALL 開啟浮層列出其餘全部來源討論——浮層項 SHALL 呈現該討論的 slug 與 topic,點擊任一項 SHALL 開啟該討論的詳情頁;全部來源討論 SHALL 經直接顯示的籤或浮層可及並可互跳。同源變更清單 SHALL 以「雙方來源討論集合交集非空」判定收錄,其呈現 SHALL 比照來源討論籤(顯示第一顆,其餘收「+N」浮層,點擊浮層項開啟該變更的詳情)。僅一份來源討論的變更 SHALL NOT 出現來源討論的「+N」籤。

#### Scenario: 多來源徽章以出身討論為代表

- **WHEN** 看板呈現來源討論清單為兩份(出身在前)的變更卡
- **THEN** 卡片帶單一討論徽章、代表清單第一份(出身)討論,徽章提示列出全部兩份來源討論

#### Scenario: 詳情頁列出全部來源討論

- **WHEN** 開啟該變更的詳情頁
- **THEN** 來源討論區直接顯示出身討論籤與「+N」籤;點擊 +N 開啟浮層,列出其餘來源討論(各項含 slug 與 topic),點擊任一項開啟該討論的詳情頁

#### Scenario: 同源以來源討論交集判定

- **WHEN** 變更 A 的來源討論清單為 d1, d2、變更 B 的來源討論清單僅含 d2
- **THEN** A 與 B 互為同源變更,出現在彼此詳情頁的同源變更清單

##### Example: 交集判定表

| 變更 A 來源 | 變更 B 來源 | 是否同源 |
| ----------- | ----------- | -------- |
| d1, d2      | d2          | 是       |
| d1          | d1          | 是       |
| d1, d2      | d3          | 否       |
| (無)        | d1          | 否       |

#### Scenario: 單一來源無溢出籤

- **WHEN** 看板呈現僅一份來源討論的變更卡並開啟其詳情頁
- **THEN** 卡片維持單一討論徽章;詳情頁來源討論區僅顯示該討論的籤,無「+N」籤

### Requirement: 看板卡片統一解剖學

看板全尺寸卡（變更卡與討論卡）SHALL 採統一三列骨架：識別列（標題＋複製鈕＋右端 meta icons）、描述列（一行截斷，無內容時缺席）、meta 列。標題 SHALL 以等寬字型呈現（變更名稱與討論 slug 同為可複製把手）。標題 SHALL 恆為單行——長於可用寬度時 SHALL 就地截斷，SHALL NOT 折行、SHALL NOT 強制斷字；截斷處 SHALL 以省略號收尾，SHALL NOT 以漸層淡出或硬切呈現；標題未被截斷時 SHALL 完整顯示，SHALL NOT 出現省略號。複製鈕 SHALL 與標題同列尾隨於標題文字之後，SHALL NOT 因標題過長而落至次行、SHALL NOT 被壓縮，meta icons 維持靠右；SHALL NOT 將複製鈕推至列右緣。變更卡描述列 SHALL 顯示 proposal Why 首句（一行截斷）；proposal 缺席、Why 區段缺席或為空時描述列 SHALL 缺席。描述資料 SHALL 由變更清單 payload 一次帶出，SHALL NOT 逐卡讀取 proposal 全文；該欄位屬呈現層輔助欄位，不屬 CLI --json 對齊範圍。建立者 SHALL 以頭像圓點呈現且 hover 顯示全名，SHALL NOT 於卡面直出全名文字；createdBy 缺席時圓點缺席。狀態 chip SHALL 僅在所在位置無法表達狀態時出現：討論卡（討論欄一欄兩態）帶狀態 chip，變更卡（所在欄即階段）SHALL NOT 帶狀態 chip。

#### Scenario: 變更卡三列骨架

- **WHEN** 看板載入一個 proposal Why 首句非空、createdBy 存在、任務 5/21 的變更
- **THEN** 變更卡識別列以等寬字型顯示名稱且複製鈕緊跟名稱文字後（hover 顯現、點擊寫入名稱至剪貼簿且不開詳情頁）、右端呈建立者圓點；描述列一行截斷顯示 Why 首句；meta 列顯示進度條與 5/21；卡上無狀態 chip

#### Scenario: 變更卡無 Why 內容時描述列缺席

- **WHEN** 某變更無 proposal.md（或 Why 區段為空）
- **THEN** 該變更卡不顯示描述列，識別列與 meta 列照常呈現，看板不因該筆缺件而報錯

#### Scenario: 長標題單行截斷時複製鈕仍在同列

- **WHEN** 變更名稱長於卡片可用寬度
- **THEN** 標題維持單行並於可用寬度處截斷、以省略號收尾，複製鈕緊跟標題文字之後留在同一列且維持可點，右端 meta icons 不被擠出卡外

#### Scenario: 討論卡長 slug 與變更卡同一收尾

- **WHEN** 討論 slug 長於卡片可用寬度
- **THEN** slug 維持單行截斷並以省略號收尾、不強制斷字折行，複製 slug 鈕留在同一列，收尾行為與變更卡標題一致

#### Scenario: 短標題完整顯示

- **WHEN** 變更名稱短於卡片可用寬度
- **THEN** 標題完整顯示且不出現省略號，複製鈕緊跟標題最後一個字元後

### Requirement: 看板全域操作成功靜默、失敗以 toast 浮層呈現

<!-- REMOVED-SCENARIO: 抽屜開啟中封存失敗仍完整可見 -->
<!-- REMOVED-SCENARIO: 封存成功關閉詳情抽屜且不呈現文字訊息 -->

桌面 app 的看板全域操作——刪除變更、封存變更、封存討論、拖排卡片、開啟專案、初始化專案——成功時 SHALL NOT 呈現任何文字結果訊息；操作結果由畫面自身表達（卡片自看板消失、側欄「已封存」計數遞增、卡片位於新位置、切換至該專案）。視窗頂欄 SHALL NOT 呈現任何操作結果文字。

上述任一操作失敗時，app SHALL 以 toast 浮層呈現失敗訊息。訊息 SHALL 包含操作對象的主詞與 core 回報的原始錯誤內容，SHALL NOT 吞掉或改寫該錯誤內容。toast SHALL 疊於詳情頁與確認對話框之上——詳情頁或確認框開啟中發生失敗時，toast SHALL 完整可見且不被遮蔽。toast SHALL 於固定時距後自動消失，並 SHALL 提供關閉鈕供提前關閉。同時 SHALL 僅呈現一則 toast；新訊息 SHALL 取代前一則並重置其自動消失時距。

toast 的表面、前景、邊框、圓角、陰影與字體 SHALL 沿用桌面 app 的既有語意設計 tokens。失敗 toast SHALL 保持中性 card 表面與可讀前景，並以 destructive token 強調錯誤圖示與邊框；SHALL NOT 使用獨立於主介面的內建明暗色票或另建 raw 色票。作業系統切換淺色或深色模式時，toast SHALL 由同一組語意 tokens 取得該模式的對應值。

失敗後看板 SHALL 刷新回磁碟現況，SHALL NOT 保留未落檔的假象順序或狀態。

封存變更成功時 SHALL 關閉詳情頁回看板。成功之所以能靜默，前提是畫面可見地表達了結果；詳情頁開啟時看板不可見，該前提即不成立。

#### Scenario: 詳情頁開啟中封存失敗仍完整可見

- **WHEN** 使用者於某 change 的詳情頁點按「封存」，而封存因前置未滿足失敗
- **THEN** toast 疊於詳情頁之上完整呈現，內容含該 change 名稱與 core 回報的錯誤訊息；詳情頁維持開啟

#### Scenario: 失敗 toast 沿用主介面語意色彩

- **WHEN** 桌面 app 在淺色或深色模式呈現任一看板全域操作的失敗 toast
- **THEN** toast 使用該模式的 card 表面、可讀前景、border、radius、陰影與 host 字體，錯誤圖示與邊框使用 destructive 語意；toast 不呈現獨立於主介面的內建明暗色票

#### Scenario: 系統明暗模式切換後 toast 使用對應 tokens

- **WHEN** 作業系統由淺色切換為深色或由深色切換為淺色，之後看板全域操作失敗
- **THEN** 新呈現的 toast 使用同一組語意 token 名稱在目前模式下的值，訊息內容、逾時、關閉與單槽取代行為不變

#### Scenario: 刪除成功不呈現任何文字訊息

- **WHEN** 使用者確認刪除某變更且刪除成功
- **THEN** 該卡片自看板消失、詳情頁關閉；視窗頂欄與 toast 皆無任何文字訊息

#### Scenario: 封存成功關閉詳情頁且不呈現文字訊息

- **WHEN** 使用者於某 change 的詳情頁點按「封存」且封存成功
- **THEN** 詳情頁關閉、卡片自看板消失、側欄「已封存」計數遞增；無任何文字訊息

#### Scenario: toast 自動消失且可提前關閉

- **WHEN** 失敗 toast 呈現後使用者未操作
- **THEN** toast 於固定時距後自動消失；該時距內使用者點按關閉鈕時 toast 立即關閉

#### Scenario: 開啟專案失敗帶主詞呈現且不切換專案

- **WHEN** 使用者選定的資料夾無法開啟為專案
- **THEN** toast 呈現該路徑與失敗原因；目前專案維持不變

#### Scenario: 拖排成功不呈現文字訊息

- **WHEN** 使用者於看板同一欄內拖動卡片至新位置放開且寫回成功
- **THEN** 卡片呈於新位置；無任何文字訊息

##### Example: 六項全域操作的回饋面對照

| 操作 | 成功 | 失敗 |
| ---- | ---- | ---- |
| 刪除變更 | 靜默：卡片消失、詳情頁關閉 | toast：`<變更名稱> · 刪除失敗 ✗ <core 錯誤>` |
| 封存變更 | 靜默：卡片消失、「已封存」計數遞增、詳情頁關閉 | toast：`<變更名稱> · 封存失敗 ✗ <core 錯誤>` |
| 封存討論 | 靜默：卡片消失、詳情頁關閉 | toast：`<討論 slug> · 討論封存失敗 ✗ <core 錯誤>` |
| 拖排卡片 | 靜默：卡片位於新位置 | toast：`<卡片識別> · 排序寫回失敗 ✗ <core 錯誤>` |
| 開啟專案 | 靜默：切換至該專案 | toast：`<選定路徑> · 開啟專案失敗 ✗ <core 錯誤>` |
| 初始化專案 | 靜默：進入該專案 | toast：`<選定目錄> · 初始化失敗 ✗ <core 錯誤>` |

### Requirement: 進行中變更可自看板退回提案中

派生階段為進行中的變更,其看板卡片與詳情頁 SHALL 呈現「退回提案中」動作(樣式沿討論卡封存按鈕的動作呈現);非進行中的變更 SHALL NOT 呈現此動作。點擊 SHALL 先經確認,確認後直接呼叫引擎的退回動詞——UI SHALL NOT 預判守門條件(守門僅由引擎裁決一次)。退回成功後 SHALL NOT 手動搬移卡片:重載後開工戳記為空、已勾任務為 0,卡片依階段派生自然回到提案中欄。引擎守門拒絕時 SHALL 開啟守門對話框,列出引擎回傳的證據(已勾任務數與 touched 記錄的檔案清單)與出路說明——已勾任務可於任務分頁取消後重試,touched 需請 agent 判斷;GUI SHALL NOT 提供清理 touched 或批次取消勾選的機械出路。本地與 remote 資料模式下,動作的出現條件、確認流程、成功效果與守門對話框行為 SHALL 一致。跨欄拖曳 SHALL 維持不改變變更階段的既有行為。

#### Scenario: 退回動作僅於進行中變更出現

- **WHEN** 使用者檢視看板上一張提案中的變更卡與一張進行中的變更卡(含各自的詳情頁)
- **THEN** 僅進行中的卡片與其詳情頁呈現「退回提案中」動作,提案中的卡片與詳情頁皆不呈現

#### Scenario: 零痕跡變更確認後退回提案中欄

- **WHEN** 使用者對一張零工作痕跡(無已勾任務、無 touched 記錄)的進行中變更卡點「退回提案中」並於確認對話框確認
- **THEN** 退回成功,卡片自進行中欄消失並出現於提案中欄,詳情頁的開工歸屬顯示清空

#### Scenario: 有工作痕跡時顯示守門對話框

- **WHEN** 使用者對一張已有 3 個已勾任務且 touched 記錄含 2 個檔案的進行中變更點「退回提案中」並確認
- **THEN** 出現守門對話框,列出已勾任務數 3 與該 2 個檔案的清單,並說明取消勾選可重試、touched 需請 agent 判斷;卡片停留在進行中欄,對話框內無任何清理或強制退回按鈕

#### Scenario: remote 模式行為一致

- **WHEN** 使用者於 remote 資料模式對零痕跡的進行中變更執行同一退回操作
- **THEN** 動作出現條件、確認流程、成功後卡片回提案中欄、守門對話框行為皆與本地模式一致

### Requirement: 封存入口的未結工單三選項

desktop 的封存入口（卡片封存鈕、詳情頁封存動作、拖曳封存）SHALL 於目標 change 為 `inReview` 時彈出三選項對話框：前往完成蓋章（導引執行審查收尾）、放棄審查（等同 review discard 後封存）、照樣帶走（等同 `--carry-review` 封存，並說明將永久顯示「曾審查未通過」）。非 `inReview` 的封存行為 SHALL 維持現行不變。

#### Scenario: 封存審查中的 change

- **WHEN** 對 `reviewStatus: "inReview"` 的 change 觸發封存
- **THEN** 出現三選項對話框，未選擇前不執行封存

#### Scenario: 選擇照樣帶走

- **WHEN** 於對話框選擇「照樣帶走」
- **THEN** change 封存成功且已封存側顯示「曾審查未通過」

### Requirement: 品質站蓋章配色與主色分離

品質站狀態標示的配色 SHALL 與主題主色分離:「已審查」蓋章 SHALL 以紫色系呈現,SHALL NOT 引用主題主色 token;「審查中」「已審查·其後有變動」「曾審查未通過」三態維持藍/琥珀/紅系不變。紫色系 SHALL 為品質站蓋章專屬——品質站標示以外的介面元素 SHALL NOT 使用紫色系。此約束 SHALL 套用於呈現審查標示的全部表面——「已審查」蓋章出現於看板卡片、變更詳情頁、已封存清單、已封存詳情頁四處;封存對話框共用同一份配色對照,但僅取用「曾審查未通過」,不呈現已審查蓋章。各表面 SHALL 引用同一份配色對照,SHALL NOT 各自定義。

#### Scenario: 已審查蓋章與主色可區辨

- **WHEN** 使用者開啟 reviewStatus 為 reviewed 的變更卡片與其詳情頁
- **THEN** 「已審查」蓋章以紫色系呈現,與同視野的主色元素(進度條、分頁、連結籤)可區辨;深淺主題下皆然

#### Scenario: 其餘三態配色不變

- **WHEN** 使用者分別檢視 inReview、reviewedStale、reviewedNotPassed 的審查標示
- **THEN** 三態仍以藍、琥珀、紅系呈現,詞條與圖示與本變更前一致

#### Scenario: 紫色系為蓋章專屬

- **WHEN** 使用者檢視看板、詳情頁與已封存各表面
- **THEN** 紫色系僅出現於品質站蓋章,不出現在其他介面元素

### Requirement: 介面狀態語意色分層

桌面 app 的狀態呈現 SHALL 依語意上色:進行中(分頁還原、遷移進行、更新下載、看板卡片 worktree 標示)為藍系;成功結果(遷移完成)為綠系;警示(登入過期 stale、needs-reauth、政策衝突、待重新反映)為琥珀系;錯誤(分頁 error、更新檢查失敗、設定檔解析錯誤、遠端工作區 access-denied/not-found/unknown)與危險動作為紅系。主題主色 SHALL NOT 用於表達狀態;錯誤 SHALL NOT 以琥珀呈現;靜態小標與裝飾面 SHALL 為中性。

生命週期與討論 SHALL 各有一個色相,由單一來源供給並為看板與系統匣面板共用:提案中為主色(teal)、進行中為藍系(與進行中語意同色)、已就緒為綠系(與成功語意同色)、討論為桃紫系(fuchsia);看板欄頂條、欄標頭圖示、欄計數徽章、卡片波次章與系統匣面板的生命週期分區、討論分區的圖示與計數徽章 SHALL 取所在階段或分區的色相;「已轉出」分區、收合列及其計數 SHALL 為中性。琥珀、紅與紫色系 SHALL NOT 用於任何生命週期或討論的欄位與分區。豁免項 SHALL 維持主色:複製成功勾號(互動即時回饋)、可點連結籤與導航元素。

色值 SHALL 只存在於共用 theme 樣式表(基底 token、狀態語意 token、品質站蓋章 token、改進標示 token、生命週期 token),淺色與深色各一套;集中常數檔 SHALL 只保存「狀態→token class」的對照;元件原始碼 SHALL 只以 token class 取色。原生色階字面(含語意色階與中性色階)SHALL NOT 出現在共用元件庫、桌面 app 與 server web 的任何元件或常數檔原始碼;主題守門測試 SHALL 於違規時失敗並指名檔案與 class,SHALL NOT 設白名單。

#### Scenario: 錯誤態以紅呈現

- **WHEN** 專案分頁進入 error、更新檢查失敗或設定檔解析錯誤
- **THEN** 對應訊息與圖示以紅系呈現,非琥珀

#### Scenario: 進行中以藍呈現

- **WHEN** 分頁還原中或遷移進行中
- **THEN** spinner 與狀態面以藍系呈現

#### Scenario: worktree 標示以藍呈現

- **WHEN** 看板卡片的 change 有活躍 worktree 映射
- **THEN** 該卡片的 worktree 圖示以藍系(進行中語意)呈現,非主色;變更詳情頁的分支與路徑維持中性

#### Scenario: 成功以綠呈現

- **WHEN** 工作區遷移完成
- **THEN** 成功卡與勾號圓標以綠系呈現

#### Scenario: 生命週期四欄各一色相

- **WHEN** 檢視看板的討論、提案中、進行中、已就緒四欄與系統匣面板的對應分區
- **THEN** 欄頂條、標頭圖示與計數徽章分別為桃紫、主色、藍、綠;系統匣面板的討論分區與三個生命週期分區同色;「已轉出」分區與收合列為中性

##### Example: 生命週期與討論的色相對照

| 欄位／分區 | 色相 | 與之同色的既有語意 |
| ---------- | ---- | ------------------ |
| 討論       | 桃紫 | 無(專屬)          |
| 提案中     | 主色 | 連結、主要操作      |
| 進行中     | 藍   | 進行中狀態          |
| 已就緒     | 綠   | 成功結果            |
| 已轉出     | 中性 | 靜態 meta          |

#### Scenario: 卡片波次章隨所在欄色相

- **WHEN** 檢視提案中欄與進行中欄各一張帶波次章的變更卡
- **THEN** 提案中卡的波次章為主色、進行中卡的波次章為藍,與各自欄的計數徽章同色

#### Scenario: 換一個色值全面生效

- **WHEN** 開發者只改共用 theme 樣式表中已就緒的生命週期 token 值
- **THEN** 看板已就緒欄、卡片、系統匣面板已就緒分區與 server web 任何取用該 token 的元件同步變色,不需改動任何 TS 或 TSX 原始碼

#### Scenario: 深色主題取深色 token

- **WHEN** 系統偏好為深色時檢視看板與系統匣面板
- **THEN** 生命週期、狀態與蓋章色取自深色 token 值,元件原始碼不含任何 dark: 色階成對寫法

#### Scenario: 原生色階守門無白名單

- **WHEN** 共用元件庫、桌面 app 或 server web 任一元件或常數檔原始碼出現原生語意色階或中性色階字面
- **THEN** 主題守門測試失敗並列出違規檔名與 class,任何檔案皆不豁免

### Requirement: 主題化提示統一延遲

<!-- REMOVED-SCENARIO: 詳情抽屜與卡片同一延遲 -->

共用 UI 元件庫的主題化提示 SHALL 採單一共用延遲預設：指標停留達 300ms 顯示提示；個別介面 SHALL NOT 另行覆寫延遲值。此預設 SHALL 同源套用於 desktop 與 server-web console（兩者共用同一 UI 元件庫）。於同一提示群組內的多個觸發點間連續移動時，後續提示 SHALL 立即顯示（沿用元件庫既有 skipDelay 行為；跨群組移動時延遲重新計算）。系統匣面板的原生 title 提示不屬主題化提示，SHALL 維持原生行為、不受本延遲約束。

#### Scenario: 卡片章提示於統一延遲後顯示

- **WHEN** 使用者將指標停留於看板卡片的審查章達 300ms
- **THEN** 主題化提示顯示狀態詞（tw 為正典詞「已審查」等）

#### Scenario: 詳情頁與卡片同一延遲

- **WHEN** 使用者分別將指標停留於變更詳情頁的章籤與看板卡片的章
- **THEN** 兩處提示皆於停留達 300ms 時顯示，無任一處立即顯示或採用其他延遲值

## REMOVED Requirements

### Requirement: detail 抽屜互斥

**Reason**: 詳情以頁取代右側面板

**Migration**: 改以「詳情頁互斥」承載同一互斥與浮層收掉規則，字面改為頁語意並補列新增專案對話框與技能檔提示確認框

### Requirement: 討論抽屜檢視與轉出變更

**Reason**: 討論詳情改為主區的詳情頁，標頭與狀態卡結構不同

**Migration**: 改以「討論詳情頁檢視與轉出變更」承載：階梯改為狀態卡的階段列、衍生變更籤列與保留在途說明

### Requirement: 抽屜標頭標記受寬度約束且抽屜不產生水平捲軸

**Reason**: 面板退場

**Migration**: 改以「詳情頁標頭標記受寬度約束且頁不產生水平捲軸」承載同一約束

### Requirement: 詳情抽屜的封存與刪除依階段守門

**Reason**: 封存與刪除移入「⋯」選單，停用原因改在項目下方

**Migration**: 改以「詳情頁的封存與刪除依階段守門」承載

### Requirement: 詳情抽屜的審查資訊列

**Reason**: 審查資訊併入標題列章籤與狀態卡品質關卡列

**Migration**: 改以「變更詳情頁的標頭與狀態卡」承載

### Requirement: 變更詳情抽屜標頭的四層結構

**Reason**: 標頭改為標題列、出身列與狀態卡

**Migration**: 改以「變更詳情頁的標頭與狀態卡」承載

### Requirement: 詳情抽屜的驗證資訊列

**Reason**: 驗證資訊併入標題列章籤與狀態卡品質關卡列

**Migration**: 改以「變更詳情頁的標頭與狀態卡」承載

### Requirement: 討論抽屜的改進標示

**Reason**: 面板退場

**Migration**: 改以「討論詳情頁的改進標示」承載

### Requirement: 變更與討論抽屜開啟時底層落回看板

**Reason**: 面板退場

**Migration**: 改以「變更與討論詳情頁的宿主為看板」承載，並補宿主頁狀態保留

### Requirement: 抽屜文件載入以 skeleton 呈現

**Reason**: 面板退場

**Migration**: 改以「詳情頁文件載入以 skeleton 呈現」承載

### Requirement: 抽屜與浮層的開關動畫

**Reason**: 滑入滑出與遮罩隨面板退場

**Migration**: 改以「詳情頁與浮層的進出動畫」承載

### Requirement: 詳情抽屜的排程分頁

**Reason**: 面板退場

**Migration**: 改以「詳情頁的排程分頁」承載，內容不變

### Requirement: 詳情抽屜的工單分頁

**Reason**: 面板退場

**Migration**: 改以「詳情頁的工單分頁」承載，內容不變

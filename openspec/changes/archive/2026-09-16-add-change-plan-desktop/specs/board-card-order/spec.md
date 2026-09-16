## MODIFIED Requirements

### Requirement: 看板卡片順序以 board_rank 欄位為真相

本地 session 的看板欄內顯示順序 SHALL 為引擎 plan 配置順序（見 change-plan 能力）投影到該欄的結果；plan 的輸入之一是卡片自身 meta 的 `board_rank` 欄位：變更卡讀自 change 目錄的 .openspec.yaml、討論卡讀自 openspec/discussions/<slug>.md 的 frontmatter（討論卡不參與 plan，維持 rank 升冪、缺 rank 置頂依 slug 序）。remote session 的順序真相 SHALL 為 scope 的 board resource 文件，SHALL NOT 讀寫卡片 meta 的 `board_rank`；remote 的排序語意（含回退序與是否套用 plan）SHALL 依 remote-board-order 能力的規定，本需求不另行規定。rank 值 SHALL 為小寫英文字母組成的字串，以位元組字典序升冪排列。本地變更卡的同欄基底序 SHALL 為：具 rank 的卡在前依字典序升冪，缺 rank 的卡在後依 created 升冪、缺 created 者殿後；同值以變更名字典序決斷；此基底序再經宣告依賴修正（宣告前置一律在其依賴者之前）。repo 內所有變更卡皆缺 rank 時，欄內顯示 SHALL 為 created 升冪（刻意變更：原為修改時間序），plan 成環時退回此基底序。

<!-- REMOVED-SCENARIO: 新建的卡落欄頂 -->
<!-- REMOVED-SCENARIO: 依 rank 升冪且缺值置頂 -->

#### Scenario: 依 rank 升冪且缺值置後

- **WHEN** 同一欄內存在具 rank 與缺 rank 的變更卡
- **THEN** 具值卡依 rank 字典序升冪排在前，其後接缺值卡依 created 升冪

##### Example: 四卡混排

- **GIVEN** 同欄四卡：W（rank: b）、X（rank: f）、Y（rank: n）、Z（無 rank）
- **WHEN** 看板渲染該欄
- **THEN** 顯示順序為 W、X、Y、Z

##### Example: 同值以名稱決斷

- **GIVEN** 同欄兩卡 beta 與 alpha 的 rank 皆為 n
- **WHEN** 看板渲染該欄
- **THEN** alpha 排在 beta 之前（名稱字典序），且兩台機器上的順序相同

##### Example: 宣告依賴修正基底序

- **GIVEN** 同欄兩卡 c（rank: b）與 a（rank: f），c 的 depends_on 含 a
- **WHEN** 看板渲染該欄
- **THEN** 顯示順序為 a、c（依賴修正蓋過 rank 序）

#### Scenario: 新建的卡落欄底

- **WHEN** 使用者新建一個變更（無 rank）且看板刷新
- **THEN** 該卡顯示於所屬欄缺 rank 群組的末端（同欄全員缺 rank 時即欄底）

#### Scenario: remote 拖排不寫卡片 meta

- **WHEN** editor 於 remote 分頁拖排一張變更卡
- **THEN** 順序變動記錄於 board resource，該變更的 .openspec.yaml 內容不變

### Requirement: 欄內拖排以中點 rank 單檔寫回

使用者於看板同一欄內拖動卡片到新位置放開時，系統 SHALL 計算落點前後鄰居 `board_rank` 的字典序中點作為被拖卡的新 `board_rank` 並寫回其 meta 檔；落點為欄頂或欄底時 SHALL 以單側鄰居推導嚴格較小或較大的鍵。中點鍵 SHALL 嚴格介於兩鄰居之間；兩鄰居間無可用縫隙時 SHALL 延長鍵長產生新鍵，SHALL NOT 改寫鄰居的 `board_rank`。變更卡寫回前 SHALL 以新鍵執行引擎的 rank 移動依賴檢查（見 change-plan 能力）：新位置會排在任一宣告前置之前、或任一宣告依賴它的變更之後時，SHALL 拒絕寫回被拖卡的 rank、以單行錯誤訊息呈現並刷新回磁碟現況；僅因 delta 重疊而依序的夥伴 SHALL NOT 構成拒絕。目標欄內全員具 `board_rank` 且 rank 序與顯示序一致時，一次拖排 SHALL 只修改被拖卡的一個檔案，且該檔除 `board_rank` 一行外其餘內容 SHALL 逐位元組不變；欄內 rank 序與顯示序不一致時（宣告依賴修正過基底序，例如加了前置但 rank 未動），SHALL 先依顯示序整欄重派 rank 再套用移動——與缺 rank 時的整欄補章同一路徑，否則反序的兩鄰居之間沒有可用的中點鍵。寫回完成後看板 SHALL 刷新至磁碟現況；寫回失敗（如檔案不可寫）時 SHALL 以單行錯誤訊息呈現並刷新回磁碟現況，SHALL NOT 保留未落檔的假象順序。看板搜尋過濾中拖排 SHALL 沿同一語意：新鍵介於可見前後鄰居之間，被過濾隱藏的卡與其相對序不受本次寫回影響。

#### Scenario: 穩態拖排只改一檔

- **WHEN** 使用者於全員具 `board_rank` 且 rank 序與顯示序一致的欄內拖動一張卡到兩鄰居之間放開
- **THEN** 只有被拖卡的 meta 檔被修改（變更卡＝該 change 的 .openspec.yaml、討論卡＝該討論的 .md），diff 僅含 `board_rank` 一行的增改，重啟 app 後順序不變

##### Example: 中點與延長

| 前鄰居 rank | 後鄰居 rank | 新 rank 性質 | Notes |
| ----------- | ----------- | ------------ | ----- |
| b | f | 嚴格介於 b 與 f（如 d） | 有縫隙取中點 |
| ab | ac | 以 ab 為前綴延長（如 abn） | 無縫隙延長鍵長 |
| （欄頂） | b | 嚴格小於 b（如 an） | 單側推導 |
| n | （欄底） | 嚴格大於 n（如 t） | 單側推導 |

#### Scenario: 寫回失敗不留假象

- **WHEN** 拖排寫回因檔案不可寫而失敗
- **THEN** 錯誤以單行訊息呈現，看板刷新後顯示磁碟上的實際順序

#### Scenario: 鄰居於寫回前消失

- **WHEN** 拖排落點的鄰居卡在寫回前被封存或刪除
- **THEN** 系統以現存鄰居重新推導新鍵完成寫回或將卡置於欄頂／欄底，不損壞任何 meta 檔、不崩潰

#### Scenario: 依賴修正後的欄拖排

- **WHEN** c 的 depends_on 含 a、c 的 rank 小於 a 的 rank（顯示序 a、c），使用者把 x 拖到 a 與 c 之間放開
- **THEN** 整欄先依顯示序重派 rank，x 的新鍵介於 a 與 c 之間，刷新後顯示 a、x、c

#### Scenario: 跨越宣告前置被拒

- **WHEN** c 的 depends_on 含 a、兩者同欄，使用者把 c 拖到 a 之前放開
- **THEN** 被拖卡 c 的 meta 檔 `board_rank` 不變，畫面顯示單行錯誤並刷新回 a 在 c 前的順序

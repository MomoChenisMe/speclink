## MODIFIED Requirements

### Requirement: 看板卡片順序以 board_rank 欄位為真相

<!-- BEFORE: 缺 rank 的卡只依 created 升冪；全員缺 rank 時欄內為 created 升冪 -->

本地 session 的看板欄內顯示順序 SHALL 為引擎 plan 配置順序（見 change-plan 能力）投影到該欄的結果；plan 的輸入之一是卡片自身 meta 的 `board_rank` 欄位：變更卡讀自 change 目錄的 .openspec.yaml、討論卡讀自 openspec/discussions/<slug>.md 的 frontmatter（討論卡不參與 plan，維持 rank 升冪、缺 rank 置頂依 slug 序）。remote session 的順序真相 SHALL 為 scope 的 board resource 文件，SHALL NOT 讀寫卡片 meta 的 `board_rank`；remote 的排序語意（含回退序與是否套用 plan）SHALL 依 remote-board-order 能力的規定，本需求不另行規定。rank 值 SHALL 為小寫英文字母組成的字串，以位元組字典序升冪排列。本地變更卡的同欄基底序 SHALL 為：具 rank 的卡在前依字典序升冪，缺 rank 的卡在後依 change-plan「執行順序的基底與拓樸修正」規定的缺 rank 順序（被依賴數多者在前 → task 總數少者在前 → created 升冪、缺 created 者殿後）；同值以變更名字典序決斷；此基底序再經宣告依賴修正（宣告前置一律在其依賴者之前）。repo 內所有變更卡皆缺 rank 時，欄內顯示 SHALL 為上述缺 rank 順序，plan 成環時退回此基底序。`board_rank` 的寫入來源 SHALL 為桌面拖排與 CLI 的 change rank 動詞（見 change-plan 能力），兩者寫同一欄位、同一中點演算。

#### Scenario: 依 rank 升冪且缺值置後

- **WHEN** 同一欄內存在具 rank 與缺 rank 的變更卡
- **THEN** 具值卡依 rank 字典序升冪排在前，其後接缺值卡依 change-plan 的缺 rank 順序

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

##### Example: 缺 rank 的卡依被依賴數與 task 數

- **GIVEN** 同欄三卡皆無 rank：x 被兩張卡依賴、y 有 12 個 task、z 有 5 個 task，y 與 z 無人依賴
- **WHEN** 看板渲染該欄
- **THEN** 顯示順序為 x、z、y

#### Scenario: 新建的卡落欄底

- **WHEN** 使用者新建一個變更（無 rank、無人依賴、tasks 已建立）且看板刷新
- **THEN** 該卡顯示於所屬欄缺 rank 群組中依 task 總數與 created 決定的位置；同欄其他缺 rank 卡皆無人依賴且 task 數不多於它時即欄底

#### Scenario: remote 拖排不寫卡片 meta

- **WHEN** editor 於 remote 分頁拖排一張變更卡
- **THEN** 順序變動記錄於 board resource，該變更的 .openspec.yaml 內容不變

### Requirement: 欄內存在缺 rank 卡時整欄補章

<!-- BEFORE: 被拖卡也算入「缺 rank」判定；不合生成格式的 rank 不觸發補章；依賴檢查在補章寫入之後，被拒時補章已落盤不回滾 -->

拖排落點所在欄內（被拖卡除外）存在缺 `board_rank`、或 `board_rank` 不合生成格式（非空、只含小寫英文字母、不以 a 結尾；手改的鍵不合時算不出中點）的卡時，該次寫回 SHALL 先依當前顯示序對該欄其餘卡片派發 `board_rank`（鍵列嚴格遞增且兩兩留有可再分縫隙），再以落點鄰居的中點寫被拖卡；被拖卡本身缺 rank SHALL NOT 觸發補章（它的新鍵由本次移動寫入）。補章 SHALL 只涵蓋該欄，SHALL NOT 波及其他欄的卡片。補章、中點與依賴檢查 SHALL 與 CLI 的 change rank 為同一份引擎實作（見 change-plan「change rank 動詞寫入看板順序鍵」），所有檢查（宣告依賴檢查、要寫入的卡皆有 .openspec.yaml）SHALL 在第一筆寫入之前完成：被拒時補章與被拖卡 SHALL NOT 寫入任何一檔。

#### Scenario: 首次拖排補章整欄

- **WHEN** 使用者首次於某欄拖排（該欄全員缺 `board_rank`）
- **THEN** 該欄每張卡的 meta 檔皆寫入 `board_rank`，欄序等於拖放後的視覺序，其他欄的檔案不變

##### Example: 三卡補章後移動

- **GIVEN** 某欄依顯示序有 A、B、C 三卡皆缺 `board_rank`
- **WHEN** 使用者把 C 拖到 A 之前放開
- **THEN** 三卡皆獲得 `board_rank` 且字典序滿足 C < A < B，任兩鍵之間仍可取中點插入新卡

#### Scenario: 只有被拖卡缺 rank 時只寫被拖卡

- **WHEN** 欄內其餘卡皆有 `board_rank` 且與顯示序一致，使用者把缺 rank 的 x 拖到其中兩張之間
- **THEN** 只有 x 的 meta 檔被寫入，其餘卡的 meta 檔逐位元不變

#### Scenario: 被拒的拖排零寫入

- **WHEN** c 的 depends_on 含 a、兩者同欄且皆缺 `board_rank`，使用者把 c 拖到 a 之前放開
- **THEN** a 與 c 的 meta 檔皆逐位元不變，畫面顯示單行錯誤並刷新回 a 在 c 前的順序

#### Scenario: 手改出不合格式的 rank 視同缺 rank

- **WHEN** 同欄 m1 的 `board_rank` 被手改成 N（大寫），m3 缺 rank，執行 speclink change rank m3 --after m1
- **THEN** 該欄先整欄補章（m1 改寫為合格式的鍵），m3 的新鍵大於 m1 的新鍵，動詞成功結束而不中止

## MODIFIED Requirements

### Requirement: 看板卡片順序以 board_rank 欄位為真相

本地 session 的看板欄內顯示順序 SHALL 由卡片自身 meta 的 `board_rank` 欄位決定：變更卡讀自 change 目錄的 .openspec.yaml、討論卡讀自 openspec/discussions/<slug>.md 的 frontmatter。remote session 的順序真相 SHALL 為 scope 的 board resource 文件（見 remote-board-order 能力），SHALL NOT 讀寫卡片 meta 的 `board_rank`。兩種模式的排序語意 SHALL 同構：rank 值 SHALL 為小寫英文字母組成的字串，以位元組字典序升冪排列；缺 rank 的卡 SHALL 排在同欄所有具 rank 的卡之前（欄頂），彼此間 SHALL 維持該模式的回退序（本地：變更卡＝修改時間序、討論卡＝slug 序；remote：server 回傳序）；rank 相同的卡 SHALL 以變更名／討論 slug 的字典序決斷，使同欄順序為全序且跨機器確定。repo 內所有卡皆缺 rank 時，看板顯示 SHALL 與引入排序能力前的行為完全一致。

#### Scenario: 依 rank 升冪且缺值置頂

- **WHEN** 同一欄內存在具 rank 與缺 rank 的卡片
- **THEN** 缺值卡依該模式回退序排在欄頂，其後接具值卡依 rank 字典序升冪

##### Example: 四卡混排

- **GIVEN** 同欄四卡：W（rank: b）、X（rank: f）、Y（rank: n）、Z（無 rank）
- **WHEN** 看板渲染該欄
- **THEN** 顯示順序為 Z、W、X、Y

##### Example: 同值以名稱決斷

- **GIVEN** 同欄兩卡 beta 與 alpha 的 rank 皆為 n
- **WHEN** 看板渲染該欄
- **THEN** alpha 排在 beta 之前（名稱字典序），且兩台機器上的順序相同

#### Scenario: 新建的卡落欄頂

- **WHEN** 使用者新建一個變更或討論（無 rank）且看板刷新
- **THEN** 該卡顯示於所屬欄的欄頂

#### Scenario: remote 拖排不寫卡片 meta

- **WHEN** editor 於 remote 分頁拖排一張變更卡
- **THEN** 順序變動記錄於 board resource，該變更的 .openspec.yaml 內容不變

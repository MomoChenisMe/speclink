## MODIFIED Requirements

### Requirement: 看板卡片的波次與阻擋標示

change 卡片 SHALL 於清單項帶 `wave` 欄位時，在進度列（進度條與任務數所在的那一列）最左渲染一枚波次行內小章：圓圈內為波次數字（同波同號即可並行）；`blockedBy` 為空時 tooltip 為「第 N 波」（tw），非空時 tooltip 為「第 N 波 · 等待：a、b」（tw；前置名稱以「、」相連，en 以「, 」）。`blockedBy` 非空時卡片 SHALL 整卡降透明度並帶 `data-blocked="true"` 屬性（與系統匣面板被擋列同一語意：淡＝現在不能做），SHALL NOT 另渲染「等 N 項」文字章。排程資訊 SHALL NOT 在標題列（名稱所在列）新增任何章——名稱的空間不被排程資訊佔用。判定 SHALL 收斂於 packages/ui 的階段派生模組單一入口（planWave／planBlockedBy），卡片元件只讀結果。清單項缺 `wave`（remote 清單未取得 plan：舊 server、請求失敗或成環；或 local plan 成環）時波次章 SHALL 缺席、卡片 SHALL NOT 變淡，其餘呈現 SHALL 與本需求引入前逐位元一致。波次章 SHALL NOT 增加文字列、SHALL NOT 影響卡片所在欄位。

#### Scenario: 第一波無阻擋

- **WHEN** 看板載入一個 wave=1、blockedBy=[] 的變更
- **THEN** 該卡片進度列最左顯示圓圈「1」波次章，tooltip 為「第 1 波」，卡片不變淡且無 data-blocked

#### Scenario: 第二波被兩項擋住

- **WHEN** 看板載入一個 wave=2、blockedBy=["add-a","add-b"] 的變更
- **THEN** 該卡片進度列最左顯示圓圈「2」，tooltip 為「第 2 波 · 等待：add-a、add-b」，整卡降透明度並帶 data-blocked="true"，標題列沒有任何排程章

#### Scenario: 缺欄位無章

- **WHEN** remote 看板載入變更清單，但 server 沒有 plan 端點（清單項不含 wave）
- **THEN** 卡片無波次章、不變淡，其餘呈現不變

### Requirement: 依賴成環時看板提示

清單 payload 的 `planError` 非 null 時，看板 SHALL 於欄位上方顯示一行提示：tw「依賴成環：」加訊息原文（如 a -> b -> a），en「Dependency cycle: 」加原文；卡片 SHALL 無波次章與阻擋章；欄內順序為 plan 退回序（local 為基底序；remote 為 board resource overlay 序，見 remote-board-order）；拖排 SHALL 照常可用。planError 為 null 時 SHALL NOT 渲染此提示列。

#### Scenario: 成環提示

- **WHEN** 看板載入 planError 為 `dependency cycle: a -> b -> a` 的清單
- **THEN** 欄位上方出現「依賴成環：dependency cycle: a -> b -> a」一行，所有卡片無波次章，拖排可用

#### Scenario: 無成環不提示

- **WHEN** 看板載入 planError 為 null 的清單
- **THEN** 不出現提示列，呈現與本需求引入前一致

### Requirement: 詳情抽屜的排程分頁

change 詳情抽屜 SHALL 於「規格」分頁之後提供「排程」分頁（tw 標籤「排程」、en「Plan」）。清單項帶 `wave` 時分頁 SHALL 依序呈現四段：波次（「第 N 波」＋同波其他變更名稱清單，無其他時顯示「本波只有這個變更」）、前置（dependsOn 每項一列；資料源 capability `setDepends` 為 true 時每列附移除鈕、底部附下拉選單列出候選變更供新增，選擇即寫入）、重疊（overlaps 每項顯示變更名與 capability 清單，唯讀）、阻擋（blockedBy 清單，空時顯示「可以開工」）。新增候選 SHALL 為該變更所在名冊中的其他作用中變更、扣除已宣告的前置：資料源提供名冊查詢時（local）SHALL 以查詢結果為準——該變更有 worktree 映射時名冊為其 worktree 副本、無映射時為主 checkout，與前置寫入作用的 store 相同；名冊查詢未完成時 SHALL NOT 渲染新增下拉；資料源未提供名冊查詢（remote，scope 只有一份名冊）或首次查詢（開啟抽屜、切換變更）失敗時 SHALL 自作用中變更清單派生；看板刷新後的重新查詢失敗時 SHALL 沿用前一次的查詢結果。新增或移除前置 SHALL 經資料源 `setDepends` 寫回該變更 meta 的 depends_on，成功後看板與分頁刷新；引擎拒絕（自依賴、不存在、已封存、成環）時 SHALL 以單行錯誤呈現引擎訊息、畫面不變。capability 為 false 時 SHALL NOT 渲染移除鈕與下拉選單。清單項缺 `wave` 且缺 `dependsOn`（remote 清單未取得 plan：舊 server、依賴成環或請求失敗）時分頁 SHALL 只顯示一句說明（tw「此模式尚未提供排程資訊」），不渲染四段；缺 `wave` 但帶 `dependsOn`（local plan 成環）時 SHALL 顯示一句說明（tw「依賴成環，先移除一條前置才有排程資訊」）並只渲染前置段（capability 為 true 時附移除鈕、SHALL NOT 渲染新增下拉），讓使用者解環。

#### Scenario: 排程分頁四段

- **WHEN** 開啟 wave=2、dependsOn=["add-a"]、overlaps=[{change:"add-b",capabilities:["desktop-app"]}]、blockedBy=["add-a","add-b"] 的變更抽屜並切到排程分頁
- **THEN** 依序顯示「第 2 波」、前置列 add-a（附移除鈕）與新增下拉、重疊列 add-b（desktop-app）、阻擋列 add-a 與 add-b

#### Scenario: 新增前置寫回

- **WHEN** 於排程分頁的下拉選單選擇 add-c
- **THEN** 該變更 .openspec.yaml 的 depends_on 含 add-c，分頁前置段出現 add-c，看板順序刷新

#### Scenario: 引擎拒絕新增

- **WHEN** 於排程分頁選擇會成環的變更
- **THEN** 顯示引擎的成環單行錯誤，前置段不變，meta 檔不變

#### Scenario: 成環時可移除前置

- **WHEN** 清單 planError 非 null，開啟 dependsOn=["add-a"]（無 wave）的變更抽屜並切到排程分頁
- **THEN** 顯示「依賴成環，先移除一條前置才有排程資訊」與前置列 add-a（附移除鈕），無新增下拉、無波次／重疊／阻擋段；按移除鈕即寫回

#### Scenario: remote 分頁唯讀

- **WHEN** 於 capability setDepends 為 false 的資料源開啟抽屜排程分頁
- **THEN** 不出現移除鈕與下拉選單

#### Scenario: worktree 映射時候選來自副本名冊

- **WHEN** 主 checkout 且 worktree 政策開啟，add-dark-mode 已映射到 worktree，add-auth 在分支前建立、add-late 在分支後才於主 checkout 建立，開啟 add-dark-mode 的排程分頁
- **THEN** 新增下拉選單含 add-auth、不含 add-late，看板清單仍列出 add-late

#### Scenario: 重新查詢名冊失敗沿用前次結果

- **WHEN** 排程分頁已以名冊 add-a、add-b 呈現候選（作用中變更清單另有 add-d，add-a 已是前置），看板刷新後名冊查詢失敗
- **THEN** 新增下拉的候選仍只有 add-b，不出現 add-d

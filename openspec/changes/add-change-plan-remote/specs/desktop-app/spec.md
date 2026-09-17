## MODIFIED Requirements

### Requirement: 詳情抽屜的排程分頁

change 詳情抽屜 SHALL 於「規格」分頁之後提供「排程」分頁（tw 標籤「排程」、en「Plan」）。清單項帶 `wave` 時分頁 SHALL 依序呈現四段：波次（「第 N 波」＋同波其他變更名稱清單，無其他時顯示「本波只有這個變更」）、前置（dependsOn 每項一列；資料源 capability `setDepends` 為 true 時每列附移除鈕、底部附下拉選單列出候選變更供新增，選擇即寫入）、重疊（overlaps 每項顯示變更名與 capability 清單，唯讀）、阻擋（blockedBy 清單，空時顯示「可以開工」）。新增候選 SHALL 為該變更所在名冊中的其他作用中變更、扣除已宣告的前置：資料源提供名冊查詢時（local）SHALL 以查詢結果為準——該變更有 worktree 映射時名冊為其 worktree 副本、無映射時為主 checkout，與前置寫入作用的 store 相同；資料源未提供名冊查詢（remote，scope 只有一份名冊）或查詢失敗時 SHALL 自作用中變更清單派生。新增或移除前置 SHALL 經資料源 `setDepends` 寫回該變更 meta 的 depends_on，成功後看板與分頁刷新；引擎拒絕（自依賴、不存在、已封存、成環）時 SHALL 以單行錯誤呈現引擎訊息、畫面不變。capability 為 false 時 SHALL NOT 渲染移除鈕與下拉選單。清單項缺 `wave` 且缺 `dependsOn`（remote 清單未取得 plan：舊 server、依賴成環或請求失敗）時分頁 SHALL 只顯示一句說明（tw「此模式尚未提供排程資訊」），不渲染四段；缺 `wave` 但帶 `dependsOn`（local plan 成環）時 SHALL 顯示一句說明（tw「依賴成環，先移除一條前置才有排程資訊」）並只渲染前置段（capability 為 true 時附移除鈕、SHALL NOT 渲染新增下拉），讓使用者解環。

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

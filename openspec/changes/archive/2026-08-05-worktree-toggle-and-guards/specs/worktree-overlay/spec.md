## ADDED Requirements

### Requirement: desktop 看板的 worktree 呈現

local workspace 的 desktop 看板 SHALL 經與 CLI list 同一觀察面組裝取得 worktree facts：有 worktree 映射的 change，卡片 SHALL 帶 worktree 標示，變更抽屜 SHALL 顯示分支名（speclink/<change名>）與 worktree 路徑（OS 原生路徑形式）。文案於 zh-TW 與 en 介面語言下均直出「worktree」一詞。worktree 的增減、或其內 openspec/changes/<change名>/ 目錄的變動，SHALL 使看板自動更新（無需手動重整）；worktree 移除（merge 收尾）後標示與抽屜資訊 SHALL 退場。git 不可用時看板照常呈現且無任何 worktree 標示（fail-open，沿用 discovery 慣例）。remote 工作區不適用本需求。

#### Scenario: 卡片標示與抽屜資訊

- **WHEN** 存在分支 speclink/add-auth 的活躍 worktree 且 add-auth 為未封存 change 時開啟 desktop 看板
- **THEN** add-auth 卡片帶 worktree 標示，開啟其變更抽屜可見分支 speclink/add-auth 與該 worktree 的路徑

#### Scenario: worktree 內進度即時反映

- **WHEN** 於 worktree 副本內將一個任務勾為完成
- **THEN** 主看板該 change 卡片的任務計數自動更新，無需手動重整

#### Scenario: 收尾後標示退場

- **WHEN** worktree 經 merge 收尾流程移除後看板刷新
- **THEN** 卡片不再帶 worktree 標示，抽屜不再顯示分支與路徑

#### Scenario: git 不可用時看板照常

- **WHEN** git 於環境中不可用時開啟 desktop 看板
- **THEN** 看板照常列出 changes 且無任何 worktree 標示，不顯示錯誤

### Requirement: worktree 掛著時的 desktop 動詞防護

對有 worktree 映射的 change，desktop 的「封存」與「退回提案中」動詞 SHALL 拒絕執行，並提示先執行 worktree-merge 收尾；唯讀呈現（抽屜、diff 檢視）不受影響。此防護僅及 desktop 動詞層；CLI 對應動詞不在此限。

#### Scenario: 封存被擋

- **WHEN** 對有活躍 worktree 映射的 change 於 desktop 觸發封存
- **THEN** 動詞拒絕執行，訊息含先收尾的指引，change 目錄與看板狀態不變

#### Scenario: 收尾後解禁

- **WHEN** 該 change 的 worktree 經 merge 收尾移除後，於 desktop 再次觸發封存
- **THEN** 封存照常執行

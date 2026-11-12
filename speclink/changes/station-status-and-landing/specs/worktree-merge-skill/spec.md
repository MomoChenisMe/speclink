## ADDED Requirements

### Requirement: 合併後回報落地

worktree-merge 技能 SHALL 在變更分支合併進主分支成功之後，指示執行 speclink change landed <change-name> --branch speclink/<change-name> 回報落地（commit 為合併後主分支的 HEAD）。合併失敗或中止時 SHALL NOT 回報。回報被拒絕時，技能 SHALL 指示把訊息告訴使用者並繼續收尾，SHALL NOT 還原合併。

#### Scenario: 合併成功後回報

- **WHEN** 檢視產生的 worktree-merge 技能中合併成功之後的步驟
- **THEN** 有一步執行 speclink change landed，帶 --branch speclink/<change-name>

#### Scenario: 合併失敗不回報

- **WHEN** 檢視合併發生衝突而中止的分支
- **THEN** 該分支沒有回報落地的步驟

#### Scenario: 回報被拒絕時繼續收尾

- **WHEN** 檢視回報那一步的失敗處理
- **THEN** 它指示告訴使用者並繼續，沒有指示還原合併

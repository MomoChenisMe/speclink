## ADDED Requirements

### Requirement: 提交後回報落地

commit 技能 SHALL 在 git commit 成功之後、且不在 worktree 的變更分支上時，指示執行 speclink change landed <change-name> 回報落地（以提交後的 HEAD）。在 worktree 的變更分支上提交時 SHALL NOT 回報：落地由合併的技能回報。回報被拒絕時（例如連到沒有這個端點的 Server），技能 SHALL 指示把訊息告訴使用者並繼續，SHALL NOT 取消或重做已完成的提交。提交失敗或被使用者取消時 SHALL NOT 回報。

#### Scenario: 一般提交後回報

- **WHEN** 檢視產生的 commit 技能中 git commit 成功之後的步驟
- **THEN** 有一步執行 speclink change landed <change-name>

#### Scenario: worktree 分支上不回報

- **WHEN** 檢視同一段步驟的條件
- **THEN** 它寫明在 worktree 的變更分支上提交時略過這一步，並指出由合併的技能回報

#### Scenario: 回報被拒絕時繼續

- **WHEN** 檢視回報那一步的失敗處理
- **THEN** 它指示把訊息告訴使用者並繼續，沒有指示還原提交

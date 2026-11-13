## ADDED Requirements

### Requirement: 提交後回報落地

commit 技能 SHALL 在 git commit 成功之後、且不在 worktree 的變更分支上時，指示執行 speclink change landed <change-name> 回報落地（以提交後的 HEAD）。技能 SHALL 指示先以 git 取得提交後的 HEAD，並以 --commit 明確帶上：技能的動詞可能在沒有 checkout 的宿主那一端執行，那裡讀不到 HEAD。在 worktree 的變更分支上提交時 SHALL NOT 回報：落地由合併的技能回報。封存子流程已經封存這個變更時 SHALL NOT 回報：落地回報需要作用中的變更，這時執行只會得到找不到變更的錯誤。回報被拒絕時（例如連到沒有這個端點的 Server），技能 SHALL 指示把訊息告訴使用者並繼續，SHALL NOT 取消或重做已完成的提交。提交失敗或被使用者取消時 SHALL NOT 回報。規格資料夾在 checkout 裡時，回報會寫進這個變更的資料檔；技能 SHALL 指出該檔在這一步之後有改動、留給這個變更的下一次提交。

#### Scenario: 一般提交後回報

- **WHEN** 檢視產生的 commit 技能中 git commit 成功之後的步驟
- **THEN** 有一步執行 speclink change landed <change-name>，並以 --commit 帶上提交後的 HEAD

#### Scenario: worktree 分支上不回報

- **WHEN** 檢視同一段步驟的條件
- **THEN** 它寫明在 worktree 的變更分支上提交時略過這一步，並指出由合併的技能回報

#### Scenario: 已封存時不回報

- **WHEN** 檢視同一段步驟的條件
- **THEN** 它寫明封存子流程執行過時略過這一步

#### Scenario: 回報被拒絕時繼續

- **WHEN** 檢視回報那一步的失敗處理
- **THEN** 它指示把訊息告訴使用者並繼續，沒有指示還原提交

#### Scenario: 本機專案指出資料檔的改動

- **WHEN** 分別檢視為「規格資料夾在 checkout 裡」與「不在 checkout 裡」產生的 commit 技能
- **THEN** 前者寫明變更的資料檔在回報之後有改動、留給下一次提交；後者沒有這段說明

## ADDED Requirements

### Requirement: 合併後回報落地

worktree-merge 技能 SHALL 在變更分支合併進主分支成功之後，指示執行 speclink change landed <change-name> --branch speclink/<change-name> 回報落地（commit 為合併後主分支的 HEAD）。技能 SHALL 指示先以 git 取得合併後主分支的 HEAD，並以 --commit 明確帶上：技能的動詞可能在沒有 checkout 的宿主那一端執行，那裡讀不到 HEAD。合併失敗或中止時 SHALL NOT 回報。回報被拒絕時，技能 SHALL 指示把訊息告訴使用者並繼續收尾，SHALL NOT 還原合併。規格資料夾在 checkout 裡時，回報會寫進這個變更的資料檔；技能 SHALL 指出主 checkout 從這一步起多了這一個改動的檔、不代為提交，並在交棒時點名它。

#### Scenario: 合併成功後回報

- **WHEN** 檢視產生的 worktree-merge 技能中合併成功之後的步驟
- **THEN** 有一步執行 speclink change landed，帶 --branch speclink/<change-name>，並以 --commit 帶上合併後主分支的 HEAD

#### Scenario: 合併失敗不回報

- **WHEN** 檢視合併發生衝突而中止的分支
- **THEN** 該分支沒有回報落地的步驟

#### Scenario: 回報被拒絕時繼續收尾

- **WHEN** 檢視回報那一步的失敗處理
- **THEN** 它指示告訴使用者並繼續，沒有指示還原合併

#### Scenario: 本機專案指出資料檔的改動

- **WHEN** 分別檢視為「規格資料夾在 checkout 裡」與「不在 checkout 裡」產生的 worktree-merge 技能
- **THEN** 前者寫明主 checkout 多了一個改動的資料檔、不代為提交；後者沒有這段說明

## ADDED Requirements

### Requirement: worktree 技能的規格檔步驟依 checkout 產生

apply-with-worktree 技能的前置指示中，「變更的 artifact 尚未 commit 時先 commit」的步驟與其 git 指令 SHALL 只在規格資料夾位於這個 checkout 時產生；不位於時 SHALL 以一句說明取代：規格文件在 Server 上，沒有東西要先 commit。前置指示檢查「這個變更已經改到的檔案在主 checkout 是否乾淨」時，檔案清單 SHALL 取自 speclink evidence show <change> --json 的 files。收尾指示要 stage 的檔案 SHALL 是 evidence 的檔案；規格資料夾位於這個 checkout 時 SHALL 另外加上這個變更在規格資料夾底下的檔案。對應的守則句子 SHALL 在兩種情況各自與步驟一致。

#### Scenario: 本機專案保留先 commit 的步驟

- **WHEN** 以「規格資料夾在這個 checkout」產生 apply-with-worktree 技能
- **THEN** 前置指示含先 commit 變更的 artifact 的步驟與對規格資料夾的 git 指令

#### Scenario: remote 專案沒有先 commit 的步驟

- **WHEN** 以「規格資料夾不在這個 checkout」產生 apply-with-worktree 技能
- **THEN** 前置指示不含對規格資料夾的 git 指令，並含一句說明規格文件在 Server 上

#### Scenario: 已改到的檔案以動詞取得

- **WHEN** 檢視兩種情況產生的前置指示中讀取變更記錄的步驟
- **THEN** 步驟以 speclink evidence show 表述，不含 cat 檔案的指令

#### Scenario: 收尾的 stage 範圍

- **WHEN** 檢視兩種情況產生的收尾指示
- **THEN** 「不在」的版本只 stage evidence 的檔案；「在」的版本另外 stage 這個變更在規格資料夾底下的檔案

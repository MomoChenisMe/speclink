## MODIFIED Requirements

### Requirement: worktree-merge 技能的收尾流程指示

<!-- BEFORE: 合併步驟為單一的一般 merge：於主資料夾直接 merge speclink 分支，衝突即中止回報；主分支曾前進時每次收尾必產生合併節點。 -->

生成的技能內文 SHALL 指示執行代理依序收尾指定 change 的 worktree：(1) preflight——確認主資料夾當前分支非 speclink/* 且非 detached（合併目標分支 SHALL 於合併前向使用者宣告，SHALL NOT 代為切換分支）、主資料夾工作樹乾淨（無未提交變更）、且該 change 的 worktree 分支已全數提交，任一不成立 SHALL 停止並向使用者說明缺什麼，SHALL NOT 代為 stash 或代為提交主資料夾的變更；(2) rebase-first 合併階梯——先於 worktree 內將分支 speclink/<change名> rebase 到合併目標分支，成功後於主資料夾以 fast-forward 限定方式合併該分支（git merge --ff-only，線圖不產生合併節點）；rebase 發生衝突時 SHALL 中止 rebase（git rebase --abort，分支完整復原）並退回一般 merge 於主資料夾執行；(3) 一般 merge 發生衝突時 SHALL 立即停止並回報衝突檔案清單，SHALL NOT 代編衝突內容、SHALL NOT 留下未完成的合併狀態（中止合併後回報）；rebase 與 merge 的衝突處置合併後，最壞情況的可觀察行為 SHALL 與單一 merge 流程相同；(4) 合併成功後 SHALL 移除該 worktree 並刪除分支；(5) 向使用者確認收尾完成，並提示該 change 可續走品質站或封存流程。

#### Scenario: 內文含 preflight 停止指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「主樹不乾淨或分支未提交即停止說明」的指示，且明示不代 stash、不代提交

#### Scenario: 內文含合併目標分支確認指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「確認主資料夾當前分支」的指示（branch --show-current），且明示停在 speclink/* 或 detached、不代為切換分支

#### Scenario: 內文含 rebase-first 階梯指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「worktree 內先 rebase 合併目標分支」與「成功後主資料夾以 --ff-only 合併」的指示，且明示 rebase 衝突時以 rebase --abort 復原分支後退回一般 merge

#### Scenario: 內文含衝突即停指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「一般 merge 衝突立即停止、回報衝突檔案、不代編、中止合併後回報」的指示

#### Scenario: 內文含清理與交棒指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「合併成功後移除 worktree 並刪除分支」與「提示續走品質站或封存」的指示

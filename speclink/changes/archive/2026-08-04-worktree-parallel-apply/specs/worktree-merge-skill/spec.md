## ADDED Requirements

### Requirement: worktree-merge 技能的生成

技能再生 SHALL 產出 worktree-merge 技能：claude 目標生成 .claude/skills/speclink-worktree-merge/SKILL.md，codex 目標生成對應產物。此技能 SHALL 為獨立完整模板（不組合其他技能本體）。

#### Scenario: claude 目標生成技能檔

- **WHEN** 於已初始化 claude 工具的 workspace 執行技能再生
- **THEN** .claude/skills/speclink-worktree-merge/SKILL.md 存在

### Requirement: worktree-merge 技能的收尾流程指示

生成的技能內文 SHALL 指示執行代理依序收尾指定 change 的 worktree：(1) preflight——確認主資料夾當前分支非 speclink/* 且非 detached（合併目標分支 SHALL 於合併前向使用者宣告，SHALL NOT 代為切換分支）、主資料夾工作樹乾淨（無未提交變更）、且該 change 的 worktree 分支已全數提交，任一不成立 SHALL 停止並向使用者說明缺什麼，SHALL NOT 代為 stash 或代為提交主資料夾的變更；(2) 於主資料夾將分支 speclink/<change名> 合併回主分支；(3) 合併發生衝突時 SHALL 立即停止並回報衝突檔案清單，SHALL NOT 代編衝突內容、SHALL NOT 留下未完成的合併狀態（中止合併後回報）；(4) 合併成功後 SHALL 移除該 worktree 並刪除分支；(5) 向使用者確認收尾完成，並提示該 change 可續走品質站或封存流程。

#### Scenario: 內文含 preflight 停止指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「主樹不乾淨或分支未提交即停止說明」的指示，且明示不代 stash、不代提交

#### Scenario: 內文含合併目標分支確認指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「確認主資料夾當前分支」的指示（branch --show-current），且明示停在 speclink/* 或 detached、不代為切換分支

#### Scenario: 內文含衝突即停指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「衝突立即停止、回報衝突檔案、不代編、中止合併後回報」的指示

#### Scenario: 內文含清理與交棒指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「合併成功後移除 worktree 並刪除分支」與「提示續走品質站或封存」的指示

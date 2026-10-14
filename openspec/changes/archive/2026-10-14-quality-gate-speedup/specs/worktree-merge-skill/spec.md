## MODIFIED Requirements

### Requirement: worktree-merge 技能的收尾流程指示
<!-- BEFORE: 合併後直接移除 worktree；交棒稱主 checkout 補跑品質關卡為無 Apply baseline 的降級路徑 -->

生成的技能內文 SHALL 指示執行代理依序收尾指定 change 的 worktree：(1) preflight——確認主資料夾當前分支非 speclink/* 且非 detached（合併目標分支 SHALL 於合併前向使用者宣告，SHALL NOT 代為切換分支）、主資料夾工作樹乾淨（無未提交變更）、且該 change 的 worktree 分支已全數提交，任一不成立 SHALL 停止並向使用者說明缺什麼，SHALL NOT 代為 stash 或代為提交主資料夾的變更；(2) rebase-first 合併階梯——合併開始前 SHALL 先記下主資料夾的 HEAD commit（合併前 HEAD），再於 worktree 內將分支 speclink/<change名> rebase 到合併目標分支，成功後於主資料夾以 fast-forward 限定方式合併該分支（git merge --ff-only，線圖不產生合併節點）；rebase 發生衝突時 SHALL 中止 rebase（git rebase --abort，分支完整復原）並退回一般 merge 於主資料夾執行；fast-forward 被拒（合併目標於 rebase 與合併之間前進，例如另一個 worktree 先合回）時 SHALL 走與 rebase 衝突相同的出口——退回一般 merge 並告知使用者本次留下合併節點；(3) 一般 merge 發生衝突時 SHALL 立即停止並回報衝突檔案清單，SHALL NOT 代編衝突內容、SHALL NOT 留下未完成的合併狀態（中止合併後回報）；「不代解 rebase 衝突」SHALL 與「不代解 merge 衝突」同列於守則清單；rebase 與 merge 的衝突處置合併後，最壞情況的可觀察行為 SHALL 與單一 merge 流程相同；(4) 合併成功後（fast-forward 或合併節點皆同）、移除 worktree 之前，SHALL 於主資料夾執行 `speclink review prepare <change> --base <合併前 HEAD> --from <worktree 路徑>`，把品質關卡的 Apply baseline 寫到主資料夾並承接 worktree 內的兩站快照；該指令失敗時 SHALL 回報並停止，SHALL NOT 移除 worktree；成功後 SHALL 移除該 worktree 並刪除分支；(5) 向使用者確認收尾完成（成功輸出 SHALL 標示本次以 fast-forward 或合併節點落地），並依變更狀態交棒：品質關卡已於 worktree 內完成→下一步為主 checkout 封存；品質關卡尚未執行→可直接於主 checkout 執行，基準已承接，SHALL NOT 稱之為降級路徑；尚有未勾 `[M]` 手動任務→於主 checkout 執行品質關卡、完成手動任務後再封存。

技能內文的前提敘述 SHALL NOT 宣稱所有步驟都不在 worktree 內執行——步驟 (2) 的 rebase 即以 worktree 為對象。

#### Scenario: 內文含 preflight 停止指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「主樹不乾淨或分支未提交即停止說明」的指示，且明示不代 stash、不代提交

#### Scenario: 內文含合併目標分支確認指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「確認主資料夾當前分支」的指示（branch --show-current），且明示停在 speclink/* 或 detached、不代為切換分支

#### Scenario: 內文含 rebase-first 階梯指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「worktree 內先 rebase 合併目標分支」與「成功後主資料夾以 --ff-only 合併」的指示，且明示 rebase 衝突時以 rebase --abort 復原分支後退回一般 merge

#### Scenario: 內文含 fast-forward 被拒的出口

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「fast-forward 被拒時退回一般 merge、並告知使用者本次留下合併節點」的指示，且前提敘述未宣稱所有步驟都不在 worktree 內執行

#### Scenario: 守則清單含 rebase 紅線與落地方式標示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 守則清單含「不代解 rebase 衝突、以 rebase --abort 退回一般 merge」，且成功輸出區塊標示本次以 fast-forward 或合併節點落地

#### Scenario: 內文含衝突即停指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「一般 merge 衝突立即停止、回報衝突檔案、不代編、中止合併後回報」的指示

#### Scenario: 內文含清理與正典順序交棒指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「合併成功後移除 worktree 並刪除分支」的指示；交棒段依狀態分岔——品質關卡已完成者以主 checkout 封存為下一步、未執行者可直接於主 checkout 執行品質關卡（基準已承接）、有未勾手動任務者先跑品質關卡與手動任務再封存；內文不含「降級路徑」的說法

#### Scenario: 內文含合併後寫入基準指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「合併開始前記下主資料夾 HEAD」與「合併成功後、移除 worktree 前執行 `speclink review prepare <change> --base <合併前 HEAD> --from <worktree 路徑>`」的指示，並明示該指令失敗時停止、不移除 worktree

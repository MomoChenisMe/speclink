# worktree-merge-skill Specification

## Purpose

/speclink-worktree-merge 技能的內容：技能檔的生成，以及收尾流程的指示——確認主樹乾淨、把 worktree 分支合併回主分支，成功後移除 worktree 並刪掉分支。本 capability 保證合併是使用者顯式觸發的一步，衝突時停下來交回給人判斷而不自行裁決。

## Requirements

### Requirement: worktree-merge 技能的生成

技能再生 SHALL 產出 worktree-merge 技能：claude 目標生成 .claude/skills/speclink-worktree-merge/SKILL.md，codex 目標生成對應產物。此技能 SHALL 為獨立完整模板（不組合其他技能本體）。

#### Scenario: claude 目標生成技能檔

- **WHEN** 於已初始化 claude 工具的 workspace 執行技能再生
- **THEN** .claude/skills/speclink-worktree-merge/SKILL.md 存在


<!-- @trace
source: worktree-parallel-apply
updated: 2026-08-04
-->

---
### Requirement: worktree-merge 技能的收尾流程指示

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


<!-- @trace
source: quality-gate-speedup
updated: 2026-10-14T21:31:03+08:00
-->

---
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

<!-- @trace
source: station-status-and-landing
updated: 2026-11-13T09:30:51+08:00
-->
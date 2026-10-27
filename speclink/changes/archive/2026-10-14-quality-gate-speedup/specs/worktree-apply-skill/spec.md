## MODIFIED Requirements

### Requirement: apply-with-worktree 技能的收尾指示
<!-- BEFORE: 提交後直接交棒，一律建議先於 worktree 內執行品質關卡再合併 -->

生成的技能內文 SHALL 指示執行代理於 apply 本體完成後：於 worktree 內完成該 change 的提交（沿用 commit 技能的歸屬慣例），SHALL NOT 執行合併回主分支，SHALL NOT 移除 worktree。

提交之後、交棒之前，技能內文 SHALL 指示把分支 rebase 到主分支最新位置並重設品質關卡的起點：

1. 審查與驗證兩站都沒有工單時才進行（以 `speclink review show`、`speclink verify show` 判斷）；任一站已有工單時 SHALL 略過整步——不在品質迴圈中途 rebase。
2. 合併目標取主 checkout 目前所在的分支；該分支為 speclink/* 或 detached 時 SHALL 略過並說明。
3. 記下合併目標當下的 commit，於 worktree 內 rebase 到該 commit 所在的分支；發生衝突時 SHALL 執行 `git rebase --abort`、回報衝突檔案並略過重設起點（分支未變，原起點仍有效），SHALL NOT 代解衝突。
4. rebase 成功後 SHALL 於 worktree 內執行 `speclink review prepare <change> --base <記下的 commit>`。

交棒 SHALL 以明確文字依變更狀態建議兩條路徑之一，由使用者判斷是否執行品質關卡，且交棒句 SHALL 明列三個入口：review（工藝品質）∥ verify（規格符合度），或 quality（兩站合跑）：

- 沒有未勾的 `[M]` 手動任務：於 worktree 內執行品質關卡，蓋章寫入的 meta 變更 SHALL 提示補提交，再以 worktree-merge 技能收尾。
- 尚有未勾的 `[M]` 手動任務（`speclink instructions apply` 的 `remaining` 減 `codeRemaining` 大於零），或使用者想在蓋章前看到 CI 結果（worktree 分支只在本機、從不 push，CI 看不到它）：先以 worktree-merge 技能收尾，再於主 checkout 執行品質關卡、完成手動任務後封存——合併時基準會承接到主 checkout。

#### Scenario: 內文含停點與正典順序交棒指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「不合併、不移除 worktree」的停點指示，交棒段的品質關卡建議明列 review、verify、quality 三個入口，並依有無未勾手動任務分成「worktree 內執行、蓋章後補提交、再 worktree-merge」與「先 worktree-merge、再於主 checkout 執行」兩條路徑，點名 worktree-merge 技能

#### Scenario: 內文含提交後 rebase 與重設起點指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「兩站皆無工單才 rebase」「rebase 衝突即 abort、略過重設起點、不代解」與「成功後執行 `speclink review prepare <change> --base <記下的 commit>`」的指示

#### Scenario: 交棒提及 CI 看不到 worktree 分支

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 交棒段載明想在蓋章前看到 CI 結果時走「先合併」路徑，理由為 worktree 分支只在本機、從不 push

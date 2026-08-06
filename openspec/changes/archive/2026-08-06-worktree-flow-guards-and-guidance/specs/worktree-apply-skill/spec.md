## MODIFIED Requirements

### Requirement: apply-with-worktree 技能的收尾指示

<!-- REMOVED-SCENARIO: 內文含停點與交棒指示 -->

生成的技能內文 SHALL 指示執行代理於 apply 本體完成後：於 worktree 內完成該 change 的提交（沿用 commit 技能的歸屬慣例），SHALL NOT 執行合併回主分支，SHALL NOT 移除 worktree，並以明確文字依正典順序交棒：建議先於 worktree 內執行品質站（review ∥ verify，由使用者判斷是否執行；蓋章寫入的 meta 變更 SHALL 提示補提交），再以 worktree-merge 技能收尾。

#### Scenario: 內文含停點與正典順序交棒指示

- **WHEN** 技能再生後讀取 SKILL.md
- **THEN** 內文含「不合併、不移除 worktree」的停點指示，交棒段含品質站建議（於 worktree 內執行、蓋章後補提交）並點名 worktree-merge 技能為後續步驟

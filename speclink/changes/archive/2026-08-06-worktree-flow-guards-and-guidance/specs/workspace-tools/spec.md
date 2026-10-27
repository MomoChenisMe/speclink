## MODIFIED Requirements

### Requirement: marker 技能指引跟隨 worktree 政策

<!-- REMOVED-SCENARIO: 政策開啟時 marker 含 worktree 技能 -->

內建工具指令檔（CLAUDE.md／AGENTS.md）的 SPECLINK marker 區塊，其 worktree 指引內容 SHALL 與技能檔生成套用同一個 worktree 政策閘：政策為 true 時輸出，false 或未設時不輸出。worktree 指引內容包含三項：(1) 技能清單段的兩行 worktree 技能指引（apply-with-worktree 與 worktree-merge）；(2) Workflow 段主流程線之下的一條 worktree 流程線，依序載明 apply-with-worktree ⇄ ingest → 品質站（並列站依工具目標：claude 為 review 與 verify 並列，codex 僅 review）→ worktree-merge → 主 checkout 封存；(3) Workflow 段 bullet 清單的一條品質站指引，敘明品質站建議於 worktree 內完成（Apply baseline 在 worktree）、封存僅在主 checkout 執行、worktree 內封存會被引擎拒絕。marker 的其餘內容（其他技能指引、主流程線與其餘 Workflow bullet）SHALL NOT 受此政策影響。自訂描述子的中性 marker 本就不列 worktree 技能，不受此需求影響。

#### Scenario: 政策關閉時 marker 不提 worktree 技能

- **WHEN** worktree 政策關閉的專案執行 speclink update
- **THEN** CLAUDE.md 的 marker 區塊不含 apply-with-worktree 與 worktree-merge 兩行，Workflow 段不含 worktree 流程線與品質站指引 bullet，其餘技能指引與 Workflow 段照舊

#### Scenario: 政策開啟時 marker 含 worktree 指引

- **WHEN** worktree 政策開啟的專案執行 speclink update
- **THEN** CLAUDE.md 的 marker 區塊含兩行 worktree 技能指引，Workflow 段含 worktree 流程線與品質站指引 bullet，且與政策關閉版本的差異僅為上述 worktree 指引內容

#### Scenario: 流程線載明正典順序

- **WHEN** worktree 政策開啟後檢視 CLAUDE.md marker 區塊的 Workflow 段
- **THEN** worktree 流程線依序含 apply-with-worktree、品質站、worktree-merge、主 checkout 封存四個階段；品質站指引 bullet 敘明封存僅在主 checkout 與 worktree 內封存會被引擎拒絕

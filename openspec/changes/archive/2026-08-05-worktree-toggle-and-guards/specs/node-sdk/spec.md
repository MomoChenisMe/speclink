## ADDED Requirements

### Requirement: 渲染 API 的 worktree 軸

instructions.render(options) 的 options SHALL 涵蓋 worktree 布林軸（未給定時視為 false），用以選擇 marker 區塊是否含兩行 worktree 技能指引；回傳內容 SHALL 與 CLI 於對等 worktree 政策下生成的 marker 一致。skills.render 不受此軸影響——技能檔內容與政策無關，政策只決定該技能是否被生成。

#### Scenario: worktree 軸切換 marker 內容

- **WHEN** 以同組 target 與 store 分別呼叫 instructions.render({ worktree: true }) 與 instructions.render({ worktree: false })
- **THEN** 前者的 marker 含 apply-with-worktree 與 worktree-merge 兩行，後者不含，其餘內容逐字相同

#### Scenario: 未給定 worktree 時取預設

- **WHEN** 呼叫 instructions.render 而未給定 worktree 選項
- **THEN** 回傳內容與 worktree: false 的結果相同

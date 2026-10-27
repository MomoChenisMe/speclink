## ADDED Requirements

### Requirement: worktree 欄位寫入的技能同步與關閉擋下

speclink workflow-config set worktree <value> SHALL 於 config 寫入成功後同步技能足跡（生成與清理範圍等同 speclink update）。由 true 改 false 時，若 local workspace 存在活躍 linked worktree（分支慣例 speclink/<change名> 映射到未封存 change），SHALL 拒絕寫入：exit code 非 0，stderr 逐列每個活躍 worktree 的 change 名、分支與路徑並提示先收尾（worktree-merge），openspec/config.yaml SHALL 維持位元組不變、技能足跡不動。git 不可用時視為無活躍 worktree（fail-open，沿用 discovery 慣例）。config 寫入成功但技能同步失敗時，寫入仍成立（config 為正典），SHALL 於 stderr 回報同步錯誤並提示重跑 speclink update 重建足跡。workflow-config show 的人眼與 --json 輸出形狀不變（worktree 欄位既已存在，此處為維持既有輸出的相容性聲明）。

#### Scenario: set true 寫入並注入技能

- **WHEN** 於無活躍 worktree 的專案執行 speclink workflow-config set worktree true
- **THEN** exit code 0，openspec/config.yaml 的 worktree 鍵為 true，各工具 skills 目錄出現兩顆 worktree 技能，stdout 確認寫入與同步結果

#### Scenario: set false 無活躍 worktree 時寫入並清理

- **WHEN** worktree: true 且無活躍 linked worktree 的專案執行 speclink workflow-config set worktree false
- **THEN** exit code 0，config 鍵改為 false，兩顆 worktree 技能目錄被移除

#### Scenario: set false 遇活躍 worktree 拒絕

- **WHEN** 存在分支 speclink/add-auth 的 linked worktree 且 add-auth 為未封存 change 時執行 speclink workflow-config set worktree false
- **THEN** exit code 非 0，stderr 列出 add-auth 的 change 名、分支 speclink/add-auth 與 worktree 路徑並提示先收尾，openspec/config.yaml 位元組不變，技能足跡不動

#### Scenario: 技能同步失敗時 config 寫入仍成立

- **WHEN** set worktree true 的 config 寫入成功、技能足跡生成因檔案系統故障失敗
- **THEN** openspec/config.yaml 的 worktree 鍵已為 true，exit code 非 0 且 stderr 報同步錯誤並提示重跑 speclink update

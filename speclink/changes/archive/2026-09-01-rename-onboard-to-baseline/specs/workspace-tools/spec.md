## ADDED Requirements

### Requirement: update 清除孤兒技能目錄

`speclink update` 於各生成目標（內建工具與自訂描述子）完成技能生成後，SHALL 清除該目標 skills 目錄下名稱以 speclink- 為前綴、且不屬於該目標本次應生成集合的目錄。本次應生成集合 SHALL 依既有規則計算：claude 為 registry 全集、codex 與自訂描述子為 for_codex 子集，worktree 政策關閉時排除兩顆 worktree 技能。名稱非 speclink- 前綴的目錄 SHALL NOT 被移除。任一目錄刪除失敗時 update SHALL 以非零 exit code 結束，已生成的檔案保留；重跑 update SHALL 收斂到同一終態。本清理 SHALL 與既有三條清理路徑（工具自 tools 下架、自訂描述子移除、worktree 政策關閉）並存，不改變其行為。

#### Scenario: 技能改名後舊目錄被清除

- **WHEN** 工作區的 skills 目錄含舊版生成的 speclink-onboard 目錄，執行 speclink update
- **THEN** speclink-onboard 目錄不存在，speclink-baseline 目錄存在，兩份技能不並存

#### Scenario: 非前綴目錄不受清理影響

- **WHEN** skills 目錄含使用者自建、名稱非 speclink- 前綴的技能目錄（如 conventional-commit），執行 speclink update
- **THEN** 該目錄與其內容位元級不變

#### Scenario: 前綴保留給生成物

- **WHEN** skills 目錄含名稱以 speclink- 為前綴、但不在本次應生成集合內的目錄，執行 speclink update
- **THEN** 該目錄被清除——speclink- 前綴的目錄一律視為引擎生成物

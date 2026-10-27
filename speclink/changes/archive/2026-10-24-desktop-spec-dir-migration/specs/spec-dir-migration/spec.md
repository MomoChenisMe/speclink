## ADDED Requirements

### Requirement: 技能足跡同步不遷移規格資料夾

切換 worktree 政策後的技能足跡同步（CLI `speclink workflow-config set worktree <true|false>`，以及桌面產出流程頁 worktree 開關的存檔）SHALL 只同步技能檔，SHALL NOT 移動規格資料夾或改名變更資料檔；符合遷移條件的舊佈局專案 SHALL 照常以 `openspec/` 同步技能檔，stdout SHALL NOT 出現遷移相關的提示。規格資料夾的遷移 SHALL 只由 `speclink update` 與桌面技能檔提示的「更新」執行，兩者都會說明改名內容與提醒。

#### Scenario: CLI 切換 worktree 不遷移

- **WHEN** 已 commit、沒有其他 worktree、`.speclink.yaml` 沒有 `spec_dir` 的舊佈局專案執行 `speclink workflow-config set worktree true`
- **THEN** exit code 為 0，`openspec/` 原封不動、沒有建立 `speclink/`，兩顆 worktree 技能生成且其中的規格路徑為 `openspec/`，stdout 沒有遷移相關的提示

#### Scenario: 桌面切換 worktree 不遷移

- **WHEN** 使用者在符合遷移條件的舊佈局專案打開產出流程頁的 worktree 開關並存檔
- **THEN** `openspec/` 原封不動，技能足跡出現兩顆 worktree 技能，看板持續自動更新

## ADDED Requirements

### Requirement: 蓋章前的預防提醒

quality 技能 SHALL 在兩條會讓章失效的情境上，於事前提醒使用者：

- **跑關卡前比對主分支**：準備兩道關卡、凍結第一輪範圍之前，若目前分支不是 repo 的主分支（例如 worktree 或功能分支），技能 SHALL 以 git 找出主分支在分岔點之後改過的檔案，並與本分支自分岔點以來改過的檔案取交集。交集不為空時，技能 SHALL 停下來列出這些檔案，說明之後再 rebase 會讓章失效、重蓋要從首輪重跑兩道關卡，並建議先 rebase（或合併）主分支，等使用者回答後才繼續；交集為空、目前就在主分支上、或找不到主分支時，SHALL 直接繼續，不停頓。
- **收尾蓋章前提醒 CI**：每輪停下來詢問時，若 repo 有 CI 設定（例如 `.github/workflows/` 下有工作流程檔），「進入收尾蓋章」選項的說明 SHALL 附註「建議先確認目前 commit 的 CI 已在所有平台通過——蓋章後為了 CI 再修會讓章失效」；技能 SHALL NOT 為此新增問題，也 SHALL NOT 自動 push 或查詢 CI 狀態。

技能 SHALL NOT 自動執行 rebase、合併或 push。

#### Scenario: 主分支改過本變更的檔時先停下

- **WHEN** 在 worktree 分支上對變更執行 `/speclink-quality`，主分支在分岔點之後改過 `crates/engine/speclink-core/src/util.rs`，而本分支也改過這個檔
- **THEN** 技能在凍結第一輪範圍之前停下，列出 `crates/engine/speclink-core/src/util.rs`，建議先 rebase 主分支並說明原因；使用者回答前沒有任何工單或範圍被建立

#### Scenario: 沒有交集時直接開始

- **WHEN** 在功能分支上執行 `/speclink-quality`，主分支在分岔點之後只改過本分支沒碰的檔
- **THEN** 技能不停頓，直接凍結第一輪範圍並送出檢查

#### Scenario: 在主分支上不比對

- **WHEN** 目前就在主分支上執行 `/speclink-quality`
- **THEN** 技能不做分支比對，直接凍結第一輪範圍

#### Scenario: 有 CI 設定時收尾選項附註

- **WHEN** repo 有 `.github/workflows/ci.yml`，某一輪兩道關卡的必修 finding 皆為零，技能停下來詢問
- **THEN** 「進入收尾蓋章」選項的說明附註先確認 CI 已在所有平台通過的建議，選項數量與沒有 CI 設定時相同

#### Scenario: 沒有 CI 設定時不附註

- **WHEN** repo 沒有任何 CI 設定檔，某一輪兩道關卡的必修 finding 皆為零，技能停下來詢問
- **THEN** 「進入收尾蓋章」選項的說明沒有 CI 附註

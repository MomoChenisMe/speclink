## ADDED Requirements

### Requirement: 既有 artifact 的更新經動詞整份寫回

ingest 技能 SHALL 以動詞讀寫它更新的每一份 artifact：更新前以 speclink artifact cat 讀出全文，在全文上合併新內容，再以 speclink new artifact 帶 --stdin --force 整份寫回；delta spec 以 speclink new artifact spec <capability> 寫回。技能 SHALL 寫明合併時不移除既有內容。被更新的變更在 speclink list --json 的項目帶 worktree 時，讀取、整份寫回與修正的指令 SHALL 指示從它的 worktree.path 執行，使更新落在 apply 讀取的那一份副本。更新任務清單時，技能 SHALL 寫明既有的每一行任務原樣保留——勾選狀態與行尾的識別註解都不變——新任務加在既有任務之外。讀取其他進行中變更的提案與 spec 撰寫語言的說明 SHALL 與 propose 技能相同。整份寫回被引擎拒絕時，技能 SHALL 指示停下來告訴使用者原因，SHALL NOT 指示改用直接編輯檔案。技能全文 SHALL NOT 提及 config.yaml。

#### Scenario: 更新以讀出再整份寫回表述

- **WHEN** 檢視生成的 ingest 技能中更新既有提案的步驟
- **THEN** 該步驟指名 speclink artifact cat 與 speclink new artifact 帶 --stdin --force

#### Scenario: 已完成的任務原樣保留

- **WHEN** 依 ingest 技能對一個已完成兩項任務的變更新增一項任務
- **THEN** 寫回的任務清單裡，原有兩項仍是勾選的且行尾的識別註解與寫回前相同；新任務是未勾選的

#### Scenario: 寫回被拒絕時停下來

- **WHEN** 檢視生成的 ingest 技能中整份寫回的說明
- **THEN** 它寫明引擎拒絕覆寫時要停下來告訴使用者，沒有指示直接編輯檔案

#### Scenario: remote 模式下更新提案

- **WHEN** 於 remote 模式依 ingest 技能把新需求併進既有的提案
- **THEN** server 上的提案含原有內容與新內容；過程沒有讀寫 checkout 內的任何規格檔

#### Scenario: worktree 裡的變更寫回 worktree 副本

- **WHEN** 檢視生成的 ingest 技能中讀取既有 artifact 與整份寫回的步驟
- **THEN** 它寫明變更帶 worktree 時，這些指令從該變更的 worktree.path 執行

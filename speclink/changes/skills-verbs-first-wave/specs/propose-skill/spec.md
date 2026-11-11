## ADDED Requirements

### Requirement: artifact 的讀取與改寫一律經動詞

propose 技能 SHALL 以動詞讀寫它需要的每一份規格文件。討論記錄的結論與 frontmatter SHALL 取自 speclink discuss show <slug> --json 的 content。自我檢查、analyze 與 validate 之後對既有 artifact 的修正 SHALL 以「speclink artifact cat 讀出全文、修改後以 speclink new artifact 帶 --stdin --force 整份寫回」表述。讀取其他進行中變更的提案 SHALL 以 speclink artifact cat proposal --change <other> 表述；該變更在清單中帶 worktree 時 SHALL 指示從它的 worktree.path 執行。spec 的撰寫語言 SHALL 指向 speclink workflow-config show --json 的 specLocale，技能全文 SHALL NOT 提及 config.yaml。指示 JSON 的 outputPath SHALL 被說明為引擎存放的位置、僅供顯示。把已封存的討論記錄搬回進行中的復原步驟 SHALL 帶與 discuss 技能相同的條件。整份寫回被引擎拒絕時，技能 SHALL 指示停下來告訴使用者，SHALL NOT 指示改用直接編輯檔案。

#### Scenario: 修正以整份寫回表述

- **WHEN** 檢視生成的 propose 技能中 analyze 之後修正 artifact 的步驟
- **THEN** 該步驟指向 speclink artifact cat 與 speclink new artifact 帶 --stdin --force 的做法

#### Scenario: 其他變更的提案以動詞讀取

- **WHEN** 檢視生成的 propose 技能中判斷前置變更的步驟
- **THEN** 讀取其他變更 Impact 的指示是 speclink artifact cat proposal --change <other>，並寫明帶 worktree 的變更從 worktree.path 執行

#### Scenario: 不提及設定檔路徑

- **WHEN** 掃描生成的 propose 技能全文
- **THEN** 不含 config.yaml；spec 語言的說明指向 speclink workflow-config show --json 的 specLocale

#### Scenario: remote 模式下修正 artifact

- **WHEN** 於 remote 模式依 propose 技能的步驟修正一份已寫入的 design
- **THEN** agent 執行的是 speclink artifact cat design 與 speclink new artifact design 帶 --stdin --force；server 上的 design 是修正後的全文

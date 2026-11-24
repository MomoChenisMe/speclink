## MODIFIED Requirements

### Requirement: artifact 的讀取與改寫一律經動詞

<!-- BEFORE: spec 語言的說明只講結構標記與規範關鍵字維持英文；讀前置產出物一律寫成 speclink artifact cat <artifact-id>，前置產出物是 specs 時照字面執行會被拒絕 -->

propose 技能 SHALL 以動詞讀寫它需要的每一份規格文件。讀取候選正式規格 SHALL 以 speclink show <capability> --item-type spec 表述。討論記錄的結論與 frontmatter SHALL 取自 speclink discuss show <slug> --json 的 content。自我檢查、analyze 與 validate 之後對既有 artifact 的修正 SHALL 以「speclink artifact cat 讀出全文、修改後以 speclink new artifact 帶 --stdin --force 整份寫回」表述。讀取其他進行中變更的提案 SHALL 以 speclink artifact cat proposal --change <other> 表述；該變更在清單中帶 worktree 時 SHALL 指示從它的 worktree.path 執行。spec 的撰寫語言 SHALL 指向 speclink instructions 回應的 locale：技能 SHALL 說明這個欄位是該份產出物的語言、引擎已依產出物解析（specs 產出物得到的是規格的語言），SHALL NOT 另外指示以其他命令判斷 spec 的撰寫語言；同一段說明 SHALL 指出 specs 的指示寫明哪些需求與情境名稱照正式規格的原文、要照指示做；技能全文 SHALL NOT 提及 config.yaml 與 specLocale。指示 JSON 的 outputPath SHALL 被說明為引擎存放的位置、僅供顯示。讀取已完成的前置產出物的步驟 SHALL 寫明前置產出物是 specs 時的讀法：依 speclink show <name> --json 的 deltaSpecs，逐一以 speclink artifact cat specs/<capability> 讀取。把已封存的討論記錄搬回進行中的復原步驟 SHALL 帶與 discuss 技能相同的條件。整份寫回被引擎拒絕時，技能 SHALL 指示停下來告訴使用者，SHALL NOT 指示改用直接編輯檔案。

#### Scenario: 修正以整份寫回表述

- **WHEN** 檢視生成的 propose 技能中 analyze 之後修正 artifact 的步驟
- **THEN** 該步驟指向 speclink artifact cat 與 speclink new artifact 帶 --stdin --force 的做法

#### Scenario: 其他變更的提案以動詞讀取

- **WHEN** 檢視生成的 propose 技能中判斷前置變更的步驟
- **THEN** 讀取其他變更 Impact 的指示是 speclink artifact cat proposal --change <other>，並寫明帶 worktree 的變更從 worktree.path 執行

#### Scenario: 不提及設定檔路徑

- **WHEN** 掃描生成的 propose 技能全文
- **THEN** 不含 config.yaml 與 specLocale；spec 語言的說明指向 speclink instructions 回應的 locale，並寫明 specs 產出物得到的是規格的語言

#### Scenario: remote 模式下修正 artifact

- **WHEN** 於 remote 模式依 propose 技能的步驟修正一份已寫入的 design
- **THEN** agent 執行的是 speclink artifact cat design 與 speclink new artifact design 帶 --stdin --force；server 上的 design 是修正後的全文

#### Scenario: 候選正式規格指名類型

- **WHEN** 檢視生成的 propose 技能中讀取候選正式規格 Purpose 的步驟
- **THEN** 指令是 speclink show <capability> --item-type spec

#### Scenario: 語言說明指向指示裡的名稱規則

- **WHEN** 檢視生成的 propose 技能中說明指示 JSON 的 locale 欄位的段落
- **THEN** 它寫明 specs 的指示會說哪些需求與情境名稱照正式規格的原文，並要求照指示做

#### Scenario: 前置產出物是 specs 時逐一讀 delta 規格

- **WHEN** 檢視生成的 propose 技能中讀取已完成的前置產出物的步驟
- **THEN** 它寫明前置產出物是 specs 時，依 speclink show <name> --json 的 deltaSpecs 逐一以 speclink artifact cat specs/<capability> 讀取

#### Scenario: 寫任務前讀得到 delta 規格

- **WHEN** 變更已有兩份 delta 規格，依 propose 技能在寫任務清單之前讀取前置產出物 specs
- **THEN** agent 執行兩次 speclink artifact cat specs/<capability>，各得到一份 delta 規格的全文；沒有執行會被拒絕的 speclink artifact cat specs

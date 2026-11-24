## MODIFIED Requirements

### Requirement: 既有 artifact 的更新經動詞整份寫回

<!-- BEFORE: 取指示的步驟只寫 speclink instructions <artifact-id>，沒有分清 delta 規格在取指示、讀取、寫回三個動詞的寫法；摘要不列改到的名稱 -->

ingest 技能 SHALL 以動詞讀寫它更新的每一份 artifact：更新前以 speclink artifact cat 讀出全文，在全文上合併新內容，再以 speclink new artifact 帶 --stdin --force 整份寫回；delta spec 以 speclink new artifact spec <capability> 寫回。技能 SHALL 寫明合併時不移除既有內容。被更新的變更在 speclink list --json 的項目帶 worktree 時，讀取、整份寫回與修正的指令 SHALL 指示從它的 worktree.path 執行，使更新落在 apply 讀取的那一份副本。更新任務清單時，技能 SHALL 寫明既有的每一行任務原樣保留——勾選狀態與行尾的識別註解都不變——新任務加在既有任務之外。讀取其他進行中變更的提案與 spec 撰寫語言的說明 SHALL 與 propose 技能相同（含「specs 的指示寫明哪些名稱照正式規格的原文」這一句）。取得指示的步驟 SHALL 寫明 delta 規格的三種寫法：指示以 speclink instructions specs 取得，整個變更只有一份、不帶 capability；讀取以 speclink artifact cat specs/<capability>；寫回以 speclink new artifact spec <capability>。最後的摘要 SHALL 有一項列出這次更新改到的既有需求名稱與情境名稱，每一筆寫成「舊名稱 → 新名稱」；沒有改到任何名稱時 SHALL NOT 列這一項。整份寫回被引擎拒絕時，技能 SHALL 指示停下來告訴使用者原因，SHALL NOT 指示改用直接編輯檔案。技能全文 SHALL NOT 提及 config.yaml。

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

#### Scenario: 取指示的步驟分清三種寫法

- **WHEN** 檢視生成的 ingest 技能中取得指示的步驟
- **THEN** 它寫明 delta 規格的指示是 speclink instructions specs、不帶 capability，並指名讀取用 speclink artifact cat specs/<capability>、寫回用 speclink new artifact spec <capability>

#### Scenario: 摘要步驟列出改到的名稱

- **WHEN** 檢視生成的 ingest 技能中最後的摘要步驟
- **THEN** 摘要的項目含改到的需求與情境名稱，寫明每一筆是舊名稱到新名稱，並寫明沒有改到名稱時不列

#### Scenario: 把 ADDED 的 delta 規格改成規格語言時回報改名

- **WHEN** 規格語言是繁體中文，變更有一份內文與名稱都是英文的 ADDED delta 規格，依 ingest 技能把它改成規格語言
- **THEN** 寫回的 delta 規格裡需求與情境的名稱是繁體中文；回給使用者的摘要逐一列出每個改掉的名稱，格式是舊名稱 → 新名稱

#### Scenario: MODIFIED 的需求名稱不翻譯

- **WHEN** 規格語言是繁體中文，正式規格有一條名稱是英文的需求，變更的 delta 規格以 MODIFIED 改寫它，依 ingest 技能把這份 delta 規格的內文改成規格語言
- **THEN** 寫回的 delta 規格裡這條需求的名稱與正式規格逐字相同；speclink validate 該變更的結果是 valid；摘要不列這條需求的改名

##### Example: 改名的摘要

- **GIVEN** delta 規格有 ADDED 的需求「Issue tokens at the token endpoint」，底下有情境「Issue a token for a valid client」
- **WHEN** 依 ingest 技能把這份 delta 規格改成繁體中文
- **THEN** 摘要含兩筆：Issue tokens at the token endpoint → 在 Token 端點簽發 Token，以及 Issue a token for a valid client → 為合法的用戶端簽發 Token

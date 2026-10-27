## ADDED Requirements

### Requirement: 投影佈局與 manifest

remote 模式的 Context Projection SHALL 位於 workspace 的 speclink 工作目錄下 context 子目錄，含 manifest.json（snapshot ID、policy revision（有值時）、逐文件 digest 與 revision，欄位 camelCase——即 protocol ContextSnapshot 既有欄位）、INDEX.md 與 openspec 鏡像佈局（config、LANGUAGE、canonical specs、change 文件）。投影 SHALL 可隨時整目錄刪除並重建；本地 fs 模式 SHALL NOT 建立投影。

#### Scenario: materialize 產出完整佈局

- **WHEN** 以測試 snapshot provider 對含一個 change 與兩個 canonical specs 的 scope 執行 materialize
- **THEN** 投影含 manifest.json（digest 逐文件齊備）、INDEX.md 與對應鏡像文件；刪除整目錄後重新 materialize 得到等價內容

### Requirement: staging 產生後原子切換

materialize SHALL 先於 staging 目錄產生完整 snapshot（含 manifest 與全部文件）再原子切換為現行投影；SHALL NOT 逐檔覆寫既有投影。staging 或切換失敗時，既有投影 SHALL 完整保留且錯誤明確。

#### Scenario: 失敗不留半套

- **WHEN** 以故障注入使 staging 產生於中途失敗，隨後檢視現行投影
- **THEN** 現行投影與 materialize 前逐位元一致；錯誤指出失敗階段

### Requirement: 完整性驗證 fail closed

Host SHALL 提供投影驗證：任一文件內容與 manifest digest 不符、或 manifest 缺失時，SHALL 以「投影已被修改或不完整、需要 refresh」的錯誤拒絕，SHALL NOT 把投影的直接修改解讀為遠端寫入。materializer SHALL 盡力設定文件唯讀屬性，完整性判定 SHALL 以 digest 為準。

#### Scenario: 被修改的投影拒絕

- **WHEN** 修改投影內某 spec 文件一個字元後執行投影驗證
- **THEN** 驗證回拒絕並指出 digest 不符的文件；遠端正典未被任何寫入觸及

### Requirement: stale 標記與 refresh

Host SHALL 提供將投影標記 stale 的操作：寫入固定名稱的 marker 檔、SHALL NOT 改動任何投影文件內容；讀取端見 marker SHALL 提示 refresh。refresh SHALL 以新 snapshot 全量重建投影並清除 marker。

#### Scenario: stale 不偷換文件

- **WHEN** 對現行投影執行 stale 標記
- **THEN** 投影文件逐位元不變且 marker 存在；執行 refresh 後 marker 消失、manifest 的 snapshot ID 更新

### Requirement: 投影必為 gitignore 涵蓋

materialize 寫入前 SHALL 驗證投影目錄被 gitignore 涵蓋；未涵蓋時 SHALL 補寫 gitignore 並輸出警告，SHALL NOT 靜默寫入未被忽略的投影。

#### Scenario: 缺 gitignore 時補寫並警告

- **WHEN** 於 gitignore 不含 speclink 工作目錄的 workspace 執行 materialize
- **THEN** gitignore 被補寫涵蓋投影、stderr 出現警告；git status 不顯示投影文件

### Requirement: 依流程縮小 context

materialize SHALL 接受流程參數並依預設集合挑選文件：discuss（config、LANGUAGE、canonical specs 索引）、propose（discussion、相關 canonical specs、schema 與 template）、apply（proposal、design、tasks、delta specs、base specs）、verify（apply 集合加最新 tasks 與驗證規則）、archive（delta specs、canonical base、tasks、revision）；未給流程參數 SHALL 為全量。挑選規則 SHALL 為 materializer 的單一實作。

#### Scenario: apply 流程集合齊備

- **WHEN** 以流程參數 apply 對某 change 執行 materialize
- **THEN** 投影含該 change 的 proposal、design、tasks、delta specs 與對應 base specs；不含無關 change 的文件

### Requirement: remote skill 讀投影且禁止寫回

remote 模式的 instructions SHALL 把 contextFiles 指向投影下的對應路徑（key 與集合邏輯不變）；apply 與 verify 技能 SHALL 指示 Agent 自投影讀取規格、投影為唯讀、任何規格修改必須經 speclink 動詞而 SHALL NOT 直接編輯投影。技能內容變更 SHALL 三處同步（內嵌資產、倉庫技能實例、render golden），golden SHALL 於乾淨樹再生。本地模式的 instructions 輸出 SHALL 逐位元不變。

#### Scenario: remote instructions 指向投影

- **WHEN** 於 remote 模式取得 apply 階段 instructions
- **THEN** contextFiles 的每個值為投影下存在的路徑；本地模式同動詞的輸出與現行逐位元一致

#### Scenario: 技能三處同步

- **WHEN** 比對內嵌技能資產、倉庫技能實例與 render golden 中 apply 與 verify 技能的 remote 段落
- **THEN** 三處內容一致；render golden 測試全綠

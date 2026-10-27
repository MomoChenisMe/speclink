## MODIFIED Requirements

### Requirement: 設定頁編輯專案說明與產出規則

<!-- BEFORE: 兩區段為永遠可編輯的表單——專案說明多行文字區、產出規則逐項輸入框(新增/編輯/刪除/上下移按鈕),無唯讀態與就地編輯 -->

設定頁 SHALL 於頂部呈現「專案設定」卡(標註 config.yaml),含「專案說明」與「產出規則」兩分頁,預設唯讀。專案說明唯讀 SHALL 以 markdown 渲染呈現 context 內容,超過固定高度 SHALL 收合並提供顯示更多展開;未設定時 SHALL 顯示空狀態提示。產出規則唯讀 SHALL 僅列出有條目的 artifact 鍵,鍵名為小節標題、條目為清單。卡右上編輯鈕 SHALL 就地切換為編輯態並將按鈕列改為取消與儲存:專案說明為 raw markdown 多行文字區;產出規則 SHALL 以活躍 schema 的 artifact id 為固定鍵各渲染一個多行文字區,SHALL NOT 提供自由鍵輸入;一行即一條規則,行序 SHALL 即為寫入檔案的條目順序(亦即指令注入順序)。儲存時 SHALL 逐行去除頭尾空白,空行 SHALL NOT 寫入;專案說明清空儲存 SHALL 移除 context 鍵,某鍵文字區清空 SHALL 移除該 artifact 鍵,全部鍵皆空 SHALL 移除 rules 鍵。取消 SHALL 還原唯讀呈現且 SHALL NOT 寫入檔案。寫入 SHALL 經與政策欄位相同的雙重解析驗證流程;序列化 SHALL 為以 YAML 保留字元(反引號、at 符號等)開頭的條目自動加引號——寫出檔案 SHALL 可被引擎解析且條目值逐字元還原。config.yaml 解析失敗時,卡片 SHALL 浮出解析失敗說明並停用編輯。

#### Scenario: 唯讀優先與就地編輯切換

- **WHEN** 使用者開啟設定頁
- **THEN** 頂部專案設定卡為唯讀:專案說明以 markdown 渲染(超長時收合並可顯示更多),產出規則僅列有條目的鍵;點編輯後同一卡就地變為可編輯,按鈕列為取消與儲存

#### Scenario: 編輯專案說明並儲存

- **WHEN** 使用者進入編輯,於專案說明文字區輸入多行 markdown 並儲存
- **THEN** 重新讀取 config.yaml 解析出的 context 值與輸入逐字元一致、其餘鍵原樣保留,卡片回唯讀並渲染新內容,重開設定頁呈現同一文字

#### Scenario: 以保留字元開頭的規則條目寫入後仍可解析

- **WHEN** 使用者於「產出規則」某鍵文字區新增一行以 YAML 保留字元開頭的規則並儲存
- **THEN** 寫出的 config.yaml 可被引擎解析(必要引號由寫入自動加上)、該條目值逐字元還原,整份工作流政策未退回預設

##### Example: 保留字元條目自動加引號

- **GIVEN** rules 原含 proposal 節一條「提案必須列出影響的 crates」
- **WHEN** 於 tasks 文字區新增一行「@完成後執行全部測試」並儲存
- **THEN** 重讀 config.yaml 可解析,rules 的 tasks 節含值「@完成後執行全部測試」(逐字元一致),proposal 節與 schema 等其餘鍵原樣保留

#### Scenario: 清空即移除鍵

- **WHEN** 使用者進入編輯,清空專案說明文字區與某鍵文字區的全部行後儲存
- **THEN** 重新讀取 config.yaml 已無 context 鍵與該 artifact 鍵;其餘鍵原樣保留

##### Example: 鍵移除語意

| 操作前檔案狀態 | 編輯操作 | 寫入後檔案效果 |
| -------------- | -------- | -------------- |
| context: 舊說明、rules 含 tasks 兩行 | 清空專案說明文字區 | context 鍵被移除,rules.tasks 原樣保留 |
| rules 含 proposal 與 tasks 兩節 | 清空 tasks 文字區 | rules 僅餘 proposal 節 |
| rules 僅含 tasks 一節 | 清空該文字區 | rules 鍵整個被移除 |

#### Scenario: 行序即寫入順序

- **WHEN** 使用者於某鍵文字區將第二行搬至第一行並儲存
- **THEN** 重新讀取 config.yaml 該節條目順序對調,後續該 artifact 的指令注入依新順序呈現規則

##### Example: 行對調

- **GIVEN** tasks 節依序含條目「先寫失敗測試」「更新文件」
- **WHEN** 於 tasks 文字區將「更新文件」一行搬到「先寫失敗測試」之前並儲存
- **THEN** config.yaml 的 tasks 節依序為「更新文件」「先寫失敗測試」

#### Scenario: 固定鍵分節不可自由輸入

- **WHEN** 使用者於使用 spec-driven schema 的專案進入產出規則編輯
- **THEN** 文字區恰為 proposal、design、specs、tasks 四個固定鍵各一,介面不提供自由新增分節鍵的輸入;回唯讀後僅有條目的鍵被列出

#### Scenario: 取消放棄編輯

- **WHEN** 使用者進入編輯、修改兩分頁內容後點取消
- **THEN** 卡片還原為編輯前的唯讀呈現,config.yaml 逐字元未變

#### Scenario: 解析失敗停用編輯

- **WHEN** config.yaml 被外部改壞為無法解析後使用者開啟設定頁
- **THEN** 專案設定卡浮出解析失敗說明、編輯鈕停用,不提供任何寫入途徑

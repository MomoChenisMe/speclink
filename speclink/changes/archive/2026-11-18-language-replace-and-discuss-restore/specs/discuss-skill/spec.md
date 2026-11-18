## MODIFIED Requirements

### Requirement: 正式規格與變更文件一律經動詞讀取
<!-- BEFORE: 搬回已封存討論的步驟帶條件——規格在這個 checkout 時手動搬檔，否則停下來請使用者到 Server 處理 -->

discuss 技能 SHALL 以動詞讀取它需要的每一份規格文件，使同一份技能文字在本機專案、連到遠端的 checkout 與沒有本機規格檔的內嵌宿主都能執行。讀取候選正式規格的 Purpose 與全文 SHALL 以 speclink show <capability> --item-type spec 表述；技能 SHALL 寫明規格清單回傳的 path 欄位只供識別，SHALL NOT 指示以它開啟檔案。使用者指名某個變更時，讀取該變更的文件 SHALL 以 speclink show <name> --json 與 speclink artifact cat 表述；該變更在 speclink list --json 的項目帶 worktree 時，技能 SHALL 指示從它的 worktree.path 執行，並寫明 deltaSpecs 列出的 <capability>/spec.md 以 specs/<capability> 讀取。把已封存的討論記錄搬回進行中的復原步驟 SHALL 以 speclink discuss restore <slug> 表述，SHALL NOT 指示搬動規格資料夾裡的檔案，SHALL NOT 依規格資料夾是否位於這個 checkout 分成兩種寫法。

#### Scenario: 候選規格的 Purpose 以 show 取得

- **WHEN** 檢視生成的 discuss 技能的 canon pass 步驟
- **THEN** 讀取 Purpose 的指示是對每個候選執行 speclink show <capability> --item-type spec；步驟寫明清單的 path 不拿來開檔

#### Scenario: 指名的變更以動詞讀取

- **WHEN** 檢視生成的 discuss 技能中「使用者提到某個變更」的步驟
- **THEN** 讀取該變更文件的指示指名 speclink show 與 speclink artifact cat，並寫明變更帶 worktree 時從它的 worktree.path 執行

<!-- REMOVED-SCENARIO: 搬回記錄的步驟帶條件 -->

#### Scenario: 搬回記錄的步驟用動詞

- **WHEN** 分別以「規格資料夾在這個 checkout」與「規格資料夾不在這個 checkout」渲染 discuss 技能，檢視把已封存的討論記錄搬回進行中的步驟
- **THEN** 兩種渲染的這一步文字相同，都指示執行 speclink discuss restore <slug>；都不含搬動 discussions/archive/ 底下檔案的指示，也不含「到 Server 那一端處理」的指示

#### Scenario: tool-call 形式同樣成立

- **WHEN** 以自訂工具的 tool-call 形式渲染 discuss 技能
- **THEN** 上述三個步驟的文字與 CLI 形式相同，只有開頭的呼叫說明不同

## ADDED Requirements

### Requirement: 用詞偏移以動詞寫回詞彙表

discuss 技能在結論把用詞偏移記進共用詞彙表的步驟 SHALL 以動詞表述：以 speclink language show 讀出全文（文件不存在時從空白開始），加入或修改詞條後，以 speclink language replace --stdin 整份寫回。技能 SHALL NOT 指示直接編輯規格資料夾裡的檔案，SHALL NOT 依規格資料夾是否位於這個 checkout 分成兩種寫法，SHALL NOT 宣稱沒有動詞可以寫入詞彙表。寫回 SHALL 沿用技能既有的「預設記錄、使用者可以拒絕」的規則：使用者拒絕時 SHALL NOT 寫入。Conclusion 的 **Capture to** 值 `LANGUAGE.md` SHALL 維持不變。

#### Scenario: 寫回詞彙表的步驟用動詞

- **WHEN** 分別以「規格資料夾在這個 checkout」與「規格資料夾不在這個 checkout」渲染 discuss 技能，檢視把用詞偏移記進詞彙表的步驟
- **THEN** 兩種渲染的這一步文字相同，都指名 speclink language show 與 speclink language replace --stdin；都不含規格資料夾的路徑，也不含「沒有動詞可以寫入」的說法

#### Scenario: tool-call 形式同樣成立

- **WHEN** 以自訂工具的 tool-call 形式渲染 discuss 技能
- **THEN** 寫回詞彙表的步驟指名同樣的兩個動詞，只有呼叫的寫法不同

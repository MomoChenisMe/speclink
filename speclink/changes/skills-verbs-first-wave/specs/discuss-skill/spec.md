## ADDED Requirements

### Requirement: 正式規格與變更文件一律經動詞讀取

discuss 技能 SHALL 以動詞讀取它需要的每一份規格文件，使同一份技能文字在本機專案、連到遠端的 checkout 與沒有本機規格檔的內嵌宿主都能執行。讀取候選正式規格的 Purpose 與全文 SHALL 以 speclink show <capability> 表述；技能 SHALL 寫明規格清單回傳的 path 欄位只供識別，SHALL NOT 指示以它開啟檔案。使用者指名某個變更時，讀取該變更的文件 SHALL 以 speclink show <name> --json 與 speclink artifact cat 表述。技能中需要直接搬動規格儲存內檔案的復原步驟（把已封存的討論記錄搬回進行中）SHALL 帶條件：規格資料夾位於這個 checkout 時才執行；否則 SHALL 指示停下來並告訴使用者這一步要在持有規格的 Server 上處理。

#### Scenario: 候選規格的 Purpose 以 show 取得

- **WHEN** 檢視生成的 discuss 技能的 canon pass 步驟
- **THEN** 讀取 Purpose 的指示是對每個候選執行 speclink show <capability>；步驟寫明清單的 path 不拿來開檔

#### Scenario: 指名的變更以動詞讀取

- **WHEN** 檢視生成的 discuss 技能中「使用者提到某個變更」的步驟
- **THEN** 讀取該變更文件的指示指名 speclink show 與 speclink artifact cat

#### Scenario: 搬回記錄的步驟帶條件

- **WHEN** 檢視生成的 discuss 技能中把已封存的討論記錄搬回進行中的步驟
- **THEN** 該步驟寫明只在規格資料夾位於這個 checkout 時執行，並寫明其他情況要停下來告訴使用者

#### Scenario: tool-call 形式同樣成立

- **WHEN** 以自訂工具的 tool-call 形式渲染 discuss 技能
- **THEN** 上述三個步驟的文字與 CLI 形式相同，只有開頭的呼叫說明不同

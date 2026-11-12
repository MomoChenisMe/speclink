## MODIFIED Requirements

### Requirement: remote 模式的限制敘述
<!-- BEFORE: 只靠執行時讀 .speclink.yaml 判斷；內嵌引擎的宿主沒有這個檔，擋不到 -->

技能 SHALL 於生成模式開始前判斷專案是否綁定 remote store（.speclink.yaml 含 remote 區段）：綁定時 SHALL 明示 remote 模式尚不支援手冊生成並停止，零檔案寫入；導覽模式不受此限。技能檔 SHALL 敘明此限制。以「規格資料夾不在這個 checkout」渲染的技能檔（remote 專案，或內嵌引擎而沒有本機規格檔的宿主）SHALL 另外在生成模式的開頭直接寫明這份技能檔的專案不支援手冊生成、輸出同一句說明後結束且零檔案寫入，SHALL NOT 依賴 .speclink.yaml 的讀取結果；導覽模式同樣不受此限。以「規格資料夾在這個 checkout」渲染的技能檔 SHALL NOT 含這一段，執行時的判斷照舊。

#### Scenario: remote 綁定專案的生成被擋

- **WHEN** 於 remote 綁定的專案呼叫生成模式
- **THEN** 技能輸出不支援的說明並停止，speclink/manual/ 未被建立或改動

#### Scenario: remote 綁定專案的導覽照常

- **WHEN** 於 remote 綁定的專案呼叫導覽模式
- **THEN** 導覽照常進行（以規格或既有手冊為索引），零檔案寫入

#### Scenario: 沒有 .speclink.yaml 的內嵌宿主也被擋

- **WHEN** 以「規格資料夾不在這個 checkout」渲染 manual 技能，檢視生成模式的第一個步驟
- **THEN** 步驟在讀取 .speclink.yaml 的判斷之前就寫明不支援手冊生成並結束，不寫入任何檔案；導覽模式不受影響

#### Scenario: 本機專案的 manual 技能沒有多出來的拒絕

- **WHEN** 以「規格資料夾在這個 checkout」渲染 manual 技能
- **THEN** 生成模式的第一個步驟只有執行時讀取 .speclink.yaml 的判斷，內容和原本相同

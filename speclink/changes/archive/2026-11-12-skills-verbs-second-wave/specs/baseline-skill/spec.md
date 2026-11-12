## ADDED Requirements

### Requirement: 規格不在 checkout 時先拒絕

以「規格資料夾不在這個 checkout」渲染的 speclink-baseline 技能檔（remote 專案，或內嵌引擎而沒有本機規格檔的宿主）SHALL 在 Step 1 之前有一個拒絕的步驟：說明 baseline 會把規格檔直接寫進規格資料夾、寫出的檔只會留在這個 checkout 而到不了持有規格的 Server，所以這個專案還不支援 baseline；告知使用者後結束，SHALL NOT 寫入任何檔案。這個步驟 SHALL 寫在技能檔裡、不依賴執行時讀取 `.speclink.yaml`。以「規格資料夾在這個 checkout」渲染的技能檔 SHALL NOT 含這個步驟，其餘內容不變。

#### Scenario: remote 專案的 baseline 技能先拒絕

- **WHEN** 以「規格資料夾不在這個 checkout」渲染 baseline 技能
- **THEN** 技能檔在 Step 1 之前含拒絕的步驟：說明不支援、寫出的檔到不了 Server，並指示結束且不寫入任何檔案

#### Scenario: 本機專案的 baseline 技能沒有拒絕的步驟

- **WHEN** 以「規格資料夾在這個 checkout」渲染 baseline 技能
- **THEN** 技能檔不含拒絕的步驟，Step 1 起的內容和原本相同

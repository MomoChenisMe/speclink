## ADDED Requirements

### Requirement: 引擎命令有穩定的名稱與可序列化的參數

每一種引擎命令 SHALL 有一個固定的名稱：小寫、以連字號分隔，由命令的種類決定，不隨參數改變（例如建立變更是 `new-change`、封存是 `archive`、新增討論輪是 `discuss-add-round`、流程設定的編輯是 `workflow-config-edit`）。兩種不同的命令 SHALL NOT 有相同的名稱。每個命令 SHALL 能把自己的參數表示成一個 JSON 物件：鍵是 camelCase 的參數名稱，值是參數的值；沒有參數的命令 SHALL 表示成空物件。名稱與參數的鍵 SHALL 是對外契約：引擎新增命令時 SHALL 同時給它名稱，否則建置失敗；既有命令的名稱或參數的鍵改變時，SHALL 有一份逐命令的對照測試失敗。這個名稱 SHALL 用在宿主的掛鉤與儲存的提交記錄；CLI 的動詞文字、人眼輸出與 `--json` SHALL 維持既有的位元級輸出，不因此改變。

#### Scenario: 名稱只由命令的種類決定

- **WHEN** 取得「建立變更 a」與「建立變更 b 並指定 schema」兩個命令的名稱
- **THEN** 兩者都是 `new-change`

#### Scenario: 參數以 camelCase 的鍵表示

- **WHEN** 取得「封存變更 demo 並帶未通過審查也放行的旗標」這個命令的參數
- **THEN** 得到一個 JSON 物件，含變更名稱 demo 與值為 true 的 `carryReview`；鍵都是 camelCase

#### Scenario: 沒有參數的命令是空物件

- **WHEN** 取得「顯示詞彙表」這個命令的參數
- **THEN** 得到空的 JSON 物件

#### Scenario: 每一種命令都在對照裡

- **WHEN** 執行逐命令的對照測試
- **THEN** 對照的筆數等於引擎命令的種類數，每一筆的名稱都不重複

#### Scenario: 寫入命令的提交記錄用同一個名稱

- **WHEN** 經儲存接口執行驗證的蓋章命令成功
- **THEN** 儲存收到的提交記錄命令欄位是 `verify-stamp`

#### Scenario: CLI 的輸出不變

- **WHEN** 這個變更前後對同一個專案執行 `speclink list --json` 與 `speclink status`
- **THEN** stdout、stderr 與 exit code 逐位元相同

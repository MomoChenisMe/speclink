## ADDED Requirements

### Requirement: 詞彙文件的寫入動詞

CLI SHALL 提供 `speclink language replace --stdin`：自 stdin 讀入內容，把它整份寫成專案的共用詞彙文件（LANGUAGE 文件）。文件不存在時 SHALL 建立；存在時 SHALL 以 stdin 的內容整份取代，不做合併。stdin 的內容是空的或只有空白時 SHALL 拒絕且零寫入。沒有帶 `--stdin` 時 SHALL 以用法錯誤拒絕。寫入之後，`speclink language show` SHALL 輸出剛寫入的內容。

動詞 SHALL 是雙模式的，並在內嵌宿主可用：本機專案寫規格資料夾裡的 `LANGUAGE.md`；連到遠端的 CLI 經契約端點寫入；宿主套件經引擎命令（穩定名稱 `language-replace`）寫入，寫入之前 SHALL 先問執行前掛鉤。成功的寫入 SHALL 隨一次提交落盤並發布事件 `language-replaced`。這個動詞 SHALL 只需要一般的寫入權限，SHALL NOT 被當成流程設定的政策鍵。

輸出：成功時 stdout SHALL 是 `✓ Replaced the LANGUAGE document`；這次呼叫建立了原本不存在的文件時 SHALL 是 `✓ Created the LANGUAGE document`。exit code SHALL 是 0。`--no-color` 下文字相同、不含色碼。`--json` SHALL 輸出一個物件，含 `created`（布林：這次呼叫是否建立了原本不存在的文件）。失敗時訊息走 stderr、exit code 非零、stdout 沒有輸出。人眼輸出在本機、remote 與宿主套件三種執行方式 SHALL 逐位元一致，且 SHALL 不隨 `locale`（`tw`、`ja`、`en` 或未設定）改變。

遠端路徑：撞上 revision 衝突時 SHALL 沿既有的寫入動詞行為失敗並請使用者重跑，文件維持衝突前的內容；離線或認證失效時 SHALL 直接失敗、不排隊；沒有寫入權限的身分 SHALL 被拒絕且文件不變。

生成技能資產中改寫詞彙文件的指示 SHALL 以「`speclink language show` 讀出全文、修改後以 `speclink language replace --stdin` 整份寫回」表述，SHALL NOT 指示直接編輯規格資料夾裡的檔案。

#### Scenario: 取代既有的詞彙文件

- **WHEN** 專案已有詞彙文件，使用者把修改後的全文經 stdin 交給 `speclink language replace --stdin`，接著執行 `speclink language show`
- **THEN** 第一個指令 exit code 0、stdout 是 `✓ Replaced the LANGUAGE document`；第二個指令輸出的內容與 stdin 的內容相同

#### Scenario: 文件不存在時建立

- **WHEN** 專案沒有詞彙文件（`speclink language show` 會失敗），使用者執行 `speclink language replace --stdin --json` 並從 stdin 給內容
- **THEN** exit code 0，stdout 是 `{"created":true}`；之後 `speclink language show` 成功並輸出該內容

#### Scenario: 空內容被拒絕

- **WHEN** stdin 是空的或只有空白與換行，執行 `speclink language replace --stdin`
- **THEN** 指令以非零 exit code 結束，stderr 指出內容是空的，既有的詞彙文件逐位元不變

##### Example: 輸入與結果

| 既有文件 | stdin | 結果 | `created` |
| --- | --- | --- | --- |
| 有 | `# Language\n\n## 詞彙\n` | 整份取代 | false |
| 沒有 | `# Language\n` | 建立 | true |
| 有 | （空字串） | 拒絕，文件不變 | 不輸出 |
| 有 | `   \n\n` | 拒絕，文件不變 | 不輸出 |

#### Scenario: 三種執行方式結果相同

- **WHEN** 以相同的內容分別在本機專案執行、連到 Server 執行、以及經宿主套件執行 `language replace --stdin`
- **THEN** 三者的人眼輸出逐位元相同；三邊之後的 `language show` 都輸出該內容

#### Scenario: 宿主的掛鉤擋下寫入

- **WHEN** 宿主的執行前掛鉤對命令 `language-replace` 回答拒絕、原因「只有維護者可以改詞彙表」，使用者經宿主套件執行 `language replace --stdin`
- **THEN** 呼叫以 `refused` 失敗，訊息是「只有維護者可以改詞彙表」，詞彙文件不變

#### Scenario: 沒有寫入權限時被拒絕

- **WHEN** 連到遠端專案、只有讀取權限的使用者執行 `speclink language replace --stdin`
- **THEN** 指令以非零 exit code 結束，詞彙文件不變

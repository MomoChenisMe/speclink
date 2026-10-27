## ADDED Requirements

### Requirement: 技能檔提示承載規格資料夾遷移

引擎的技能檔過期探測 SHALL 多回報規格資料夾的遷移狀態：無、待遷移（附待改名的資料檔數）或暫緩（附原因：其他 worktree 的路徑清單、`openspec/` 有未提交修改、或位於 linked worktree），判定與 `speclink update` 相同。桌面 app 開啟專案時 SHALL 沿用既有的技能檔過期提示卡與對話框承載遷移，SHALL NOT 另建流程：

- 待遷移時，對話框 SHALL 寫明 `openspec/` 會改名為 `speclink/`、N 個資料檔會改名為 `.meta.yaml`、不會幫使用者 commit；按「更新」SHALL 呼叫與 CLI 相同的引擎 update。
- 暫緩時，對話框 SHALL 列出暫緩原因；按「更新」只更新技能檔，資料夾不動。
- 遷移完成後，對話框 SHALL 顯示與 CLI 相同的提醒：要 commit 的路徑、團隊成員先升級再 pull、自己的腳本與 CI 設定裡的 `openspec/` 要自己改；桌面 SHALL 重新解析規格資料夾並以 `speclink/` 重啟檔案監看，看板與詳情持續自動更新。
- 只有遷移狀態需要提示、技能檔本身不過期時，提示卡與對話框 SHALL 照樣出現。
- 「略過」SHALL 沿用既有語意，只記住這一版；略過後專案照常讀 `openspec/`。

桌面 app 自己升版時 SHALL NOT 在背景遷移任何專案；只有使用者開啟該專案並在對話框按「更新」時才遷移。遠端工作區 SHALL NOT 出現遷移提示。

#### Scenario: 待遷移的提示與更新

- **WHEN** 使用者以桌面 app 開啟已 commit、沒有其他 worktree、仍是 openspec/ 的舊專案（含 4 個變更資料檔），技能檔過期提示出現，使用者在對話框按「更新」
- **THEN** 對話框在按下前寫明 openspec/ 會改名為 speclink/、4 個資料檔會改名、不會 commit；按下後 openspec/ 改名為 speclink/，對話框顯示要 commit 的路徑與兩則提醒；之後於外部修改 speclink/changes/ 下的文件，看板數秒內自動更新

#### Scenario: 暫緩時只更新技能檔

- **WHEN** 舊專案另有一個 worktree，使用者開啟專案並在提示對話框按「更新」
- **THEN** 對話框列出該 worktree 路徑作為暫緩原因，按下後只更新技能檔，openspec/ 原封不動

#### Scenario: 技能檔不過期也提示遷移

- **WHEN** 舊專案的技能檔已是目前版本，但仍符合遷移條件
- **THEN** 開啟專案時技能檔提示卡照樣出現，對話框寫明待遷移的內容

#### Scenario: app 升版不在背景遷移

- **WHEN** 桌面 app 自動更新到新版並重啟，使用者尚未開啟任何舊專案
- **THEN** 所有舊專案的 openspec/ 原封不動

#### Scenario: 略過後照常使用

- **WHEN** 使用者在待遷移的提示按「略過」
- **THEN** 專案照常以 openspec/ 開啟與自動更新，這一版不再出現提示

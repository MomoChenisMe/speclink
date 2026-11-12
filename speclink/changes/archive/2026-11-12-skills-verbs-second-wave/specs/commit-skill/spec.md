## ADDED Requirements

### Requirement: commit 的檔案清單取自 evidence 動詞

commit 技能 SHALL 以 speclink evidence show <change> --json 取得這個變更改到的程式檔（`files` 與逐任務的 `entries`），SHALL NOT 指示直接讀取 evidence 檔。任務進度 SHALL 取自 speclink list --json；先封存子流程中 delta 的檢查與改寫 SHALL 與 archive 技能用相同的動詞做法。規格資料夾底下的變動（變更的 artifact、封存造成的搬移、手冊資料夾的提醒）SHALL 只在規格資料夾位於這個 checkout 時列入檔案清單與提醒；不位於時技能 SHALL 說明這次 commit 只含程式碼的檔案、規格的變動已經在 Server 上。確認閘門顯示的檔案清單 SHALL 等於實際會 stage 的檔案，兩種情況都成立。逐檔 stage 與不使用整批 stage 的既有規則不變。

#### Scenario: 檔案清單以動詞取得

- **WHEN** 檢視產生的 commit 技能中收集這個變更的程式檔的步驟
- **THEN** 該步驟以 speclink evidence show 表述，全文不在條件區塊以外提到 evidence 的檔名

#### Scenario: remote 專案的 commit 只含程式碼

- **WHEN** 以「規格資料夾不在這個 checkout」產生 commit 技能
- **THEN** 檔案清單的步驟只列 evidence 的檔案與使用者確認的其他髒檔，並說明規格的變動已經在 Server 上；不含規格資料夾的路徑

#### Scenario: 本機專案的 commit 含規格檔

- **WHEN** 以「規格資料夾在這個 checkout」產生 commit 技能
- **THEN** 檔案清單的步驟另外列入這個變更在規格資料夾底下的變動，並保留封存後的搬移一併提交的步驟

#### Scenario: 確認閘門與實際一致

- **WHEN** 檢視兩種情況產生的 commit 技能的確認閘門
- **THEN** 閘門顯示的檔案清單來源與之後 stage 的檔案來源相同

## ADDED Requirements

### Requirement: 動詞參考文件載明任務行的切分規則

動詞參考文件（docs/verb-contract 的中英兩版）SHALL 各有一節說明讀取端怎麼把 tasks.md 的一個任務行切成序號、標題、內文與驗證，錨點 SHALL 為 `task-line-shape`。這一節 SHALL 寫明：行首的勾選框與選用的 `[M]` 標記；行尾的 stable ID 註解在切分前先移除；三個驗證標記「驗證：」「検証：」「Verify: 」取最先出現的一個；有全形「：」時以首個全形「：」切標題與內文，沒有時才以首個「半形冒號後接一個空白」切；「Verify: 」與半形冒號在行內程式碼內不算標記；切不到時整段為標題。這一節 SHALL 附一張含繁體中文、日文、英文各一個任務的例子表，並 SHALL 說明這個形狀是 tasks 指示給寫入端的建議、既有的任務不保證符合。文件寫的規則 SHALL 與 packages/ui 的任務切分結果一致。

#### Scenario: 讀者查得到切分規則

- **WHEN** 自己畫任務頁的讀者在動詞參考文件查 `show --json` 的 `tasks` 怎麼解析
- **THEN** 可見連到 `task-line-shape` 這一節的連結；該節列出三個驗證標記、兩種冒號的優先序、行內程式碼的例外與行尾 stable ID 註解的處理，並有三種語言的例子表

#### Scenario: 中英兩版都有這一節

- **WHEN** 分別開啟 docs/verb-contract.md 與 docs/verb-contract.zh-TW.md
- **THEN** 兩份都有錨點為 `task-line-shape` 的一節，例子表的列數相同，標記的寫法逐字相同

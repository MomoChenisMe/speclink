## ADDED Requirements

### Requirement: 動詞文字的解析只有一份

把動詞文字（argv）轉成引擎命令的解析 SHALL 只有一份定義，CLI 與 Node SDK dispatch SHALL 共用它：同一串 argv 在兩個入口 SHALL 得到相同的接受或拒絕，被拒絕時的錯誤文字 SHALL 相同。這份解析 SHALL NOT 結束程序、SHALL NOT 印出任何內容、SHALL NOT 讀取 stdin、檔案、環境變數或呼叫 git。命令需要而 argv 帶不了的資料——stdin 的內容、worktree 的事實、改了哪些檔與 HEAD commit、看板的順序——SHALL 由呼叫端收集後當成輸入交給解析。沒有對應引擎命令的動詞（drift、change rank、review prepare、review scope、verify scope、帶 --skill 的 instructions）SHALL 被明確回報為「只能在有 checkout 的一端執行」，SHALL NOT 回報成不認得的動詞。每個動詞要不要讀 stdin SHALL 由解析結果回報，分三種：不讀、帶 --stdin 才讀、帶 --stdin 或 stdin 不是終端機時讀。CLI 既有的指令、旗標、--help 文字、人眼輸出、--json 與 exit code SHALL 維持位元級一致。

#### Scenario: 兩個入口對同一串 argv 的結果相同

- **WHEN** 對同一個 workspace 分別執行 speclink list --sort name --json 與 engine.dispatch(['list', '--sort', 'name'])
- **THEN** 兩者得到語意相同的 changes 清單

#### Scenario: 不認得的旗標在兩個入口都被拒絕

- **WHEN** 分別執行 speclink status --no-such-flag 與 engine.dispatch(['status', '--no-such-flag'])
- **THEN** CLI 以非零 exit code 結束、stderr 是解析錯誤；dispatch 以 Error 拒絕、code 為 invalid_argv、message 與 CLI 的錯誤文字相同（不含 ANSI 色碼）

#### Scenario: 解析不結束程序

- **WHEN** 把不合法的 argv、空的 argv 或 --help 交給解析
- **THEN** 解析回傳錯誤或說明文字給呼叫端，呼叫它的程序繼續執行，stdout 與 stderr 沒有任何輸出

#### Scenario: 需要 checkout 的動詞被明確回報

- **WHEN** 執行 engine.dispatch(['drift', 'add-x'])
- **THEN** Promise 以 Error 拒絕，code 為 invalid_argv，message 說明這個動詞只能在有 checkout 的一端執行

#### Scenario: 呼叫端提供的資料進入命令

- **WHEN** 呼叫端把 stdin 的內容交給解析，動詞是 new artifact proposal --change demo --stdin
- **THEN** 得到的引擎命令帶有這段內容；呼叫端沒有提供內容時，解析回報 invalid_argv

##### Example: stdin 的三種規則

| 動詞 | 規則 |
| ---- | ---- |
| list | 不讀 |
| new artifact proposal --change demo --stdin | 帶 --stdin 才讀 |
| review stamp demo --stdin | 帶 --stdin 才讀 |
| discuss add-round demo | 帶 --stdin 或 stdin 不是終端機時讀 |
| discuss conclude demo | 帶 --stdin 或 stdin 不是終端機時讀 |

#### Scenario: CLI 的說明文字不變

- **WHEN** 於這個變更前後，對讀寫規格的每個動詞家族與每個末端動詞執行 --help
- **THEN** stdout 逐位元一致，頂層 speclink --help 的子指令順序相同

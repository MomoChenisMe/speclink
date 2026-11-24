## ADDED Requirements

### Requirement: instructions 對 specs 產出物回報解析後的規格語言

speclink instructions specs --change <名> --json 的 payload 的 `locale` 欄位（字串）SHALL 是規格語言的顯示名稱。規格語言 SHALL 由 spec_locale 的有效值決定，有效值沿用工作流政策的三層解析（SPECLINK_SPEC_LOCALE ＞ speclink/config.yaml ＞ 內建預設）：

- spec_locale 是 `tw` 時 SHALL 為 `Traditional Chinese (繁體中文)`，是 `ja` 時 SHALL 為 `Japanese (日本語)`，是 `en` 時 SHALL 為 `English`。
- spec_locale 是 `auto` 時 SHALL 為 locale 有效值的顯示名稱；locale 未設定時 SHALL 為 `English`。
- spec_locale 未設定時 SHALL 為 `English`，不論 locale 的值。

其他產出物（proposal、design、tasks）與 speclink instructions apply 的 `locale` SHALL 維持 locale 有效值的顯示名稱。payload 的欄位名稱與型別 SHALL 不變。

同一份 payload 的 `instruction` SHALL 以一段寫明規格語言的文字結尾，規格語言是 English 時也寫：這一段 SHALL 含 `Write spec prose in <顯示名稱>`，`<顯示名稱>` 與同一份 payload 的 `locale` 相同；SHALL 說明結構標記（`## ADDED/MODIFIED/REMOVED/RENAMED Requirements`、`### Requirement:`、`#### Scenario:`、`- **WHEN**`／`- **THEN**`）與規範關鍵字（SHALL、MUST、SHALL NOT、MUST NOT）維持英文。規格語言是 `tw` 時，這一段 SHALL 另含中文模糊用語（應該、可能、也許、或許、大概、考慮、盡量、待定）會被 analyzer 標記的提醒；規格語言是 `ja` 或 English 時 SHALL NOT 含這項提醒。`instruction` SHALL NOT 含 `config.yaml`、`English by default` 與 `spec_locale` 這三段文字。schema 的 specs 產出物沒有 instruction 時，這一段 SHALL 是整個 `instruction`。

人眼輸出（不帶 --json）SHALL 維持不顯示 `locale` 欄位，它的 Instruction 區段 SHALL 是同一段 `instruction` 文字（因此以同一段語言說明結尾）；`--no-color` 下文字相同、不含色碼。本機 repo、連到遠端的 CLI（payload 由 Server 以同一個入口計算，CLI 原樣輸出）與宿主套件的 dispatch SHALL 對相同的設定得到相同的 `locale` 與 `instruction`。speclink/config.yaml 無法解析時 SHALL 照「工作流政策的正典歸屬與三層解析順序」以非零 exit code 失敗，SHALL NOT 以預設語言繼續；找不到變更時 SHALL 以非零 exit code 失敗、stderr 指出找不到該變更，stdout 不輸出 payload。

這是刻意的輸出變更。變更前：`locale` 對每個產出物都是 locale 有效值的顯示名稱；specs 的 `instruction` 固定含「English by default. If the project sets spec_locale in speclink/config.yaml…」一段，只有 spec_locale 解析成非英文時才在最後補上語言說明。

#### Scenario: 沒有設定 spec_locale 時規格語言是 English

- **WHEN** speclink/config.yaml 設定 locale: tw 且未設定 spec_locale，執行 speclink instructions specs --change 某 change --json
- **THEN** exit code 為 0；payload 的 locale 為 English；instruction 的最後一段含 Write spec prose in English，不含中文模糊用語的提醒

#### Scenario: auto 跟著 locale

- **WHEN** speclink/config.yaml 設定 locale: tw 與 spec_locale: auto，執行 speclink instructions specs --change 某 change --json
- **THEN** payload 的 locale 為 Traditional Chinese (繁體中文)；instruction 的最後一段含 Write spec prose in Traditional Chinese (繁體中文) 與中文模糊用語的提醒；instruction 不含 spec_locale 這段文字

#### Scenario: 規格語言與 locale 不同時各自回報

- **WHEN** speclink/config.yaml 設定 locale: tw 與 spec_locale: ja，分別執行 speclink instructions specs 與 speclink instructions proposal（同一個 change，帶 --json）
- **THEN** specs 的 locale 為 Japanese (日本語)，proposal 的 locale 為 Traditional Chinese (繁體中文)

#### Scenario: 環境變數覆寫規格語言

- **WHEN** speclink/config.yaml 設定 spec_locale: en，設定環境變數 SPECLINK_SPEC_LOCALE=ja，執行 speclink instructions specs --change 某 change --json
- **THEN** payload 的 locale 為 Japanese (日本語)；instruction 的最後一段含 Write spec prose in Japanese (日本語)

#### Scenario: 指示不要求讀設定檔

- **WHEN** 以任一合法的 locale 與 spec_locale 組合執行 speclink instructions specs --change 某 change --json
- **THEN** instruction 不含 config.yaml，也不含 English by default

#### Scenario: 經宿主套件得到相同的回應

- **WHEN** 宿主套件的範圍設定 locale: tw、未設定 spec_locale，以 dispatch 執行 instructions specs --change 某 change --json
- **THEN** 回傳物件的 locale 為 English，instruction 的最後一段含 Write spec prose in English，與本機 CLI 對相同設定的 payload 相同

#### Scenario: 設定檔無法解析時不回報語言

- **WHEN** speclink/config.yaml 含 YAML 語法錯誤，執行 speclink instructions specs --change 某 change --json
- **THEN** exit code 非 0，stderr 指出該檔的路徑與解析原因，stdout 不輸出 payload

##### Example: 規格語言的解析

| locale | spec_locale | SPECLINK_SPEC_LOCALE | specs 的 locale | proposal 的 locale |
| ------ | ----------- | -------------------- | --------------- | ------------------ |
| tw     | 未設定      | 未設定               | English         | Traditional Chinese (繁體中文) |
| tw     | en          | 未設定               | English         | Traditional Chinese (繁體中文) |
| tw     | auto        | 未設定               | Traditional Chinese (繁體中文) | Traditional Chinese (繁體中文) |
| tw     | ja          | 未設定               | Japanese (日本語) | Traditional Chinese (繁體中文) |
| 未設定 | auto        | 未設定               | English         | English            |
| 未設定 | tw          | 未設定               | Traditional Chinese (繁體中文) | English |
| tw     | en          | ja                   | Japanese (日本語) | Traditional Chinese (繁體中文) |

### Requirement: tasks 指示的紀律說明不指向帶 --skill 的 instructions

tdd 的有效值為開啟時，speclink instructions tasks --change <名> --json 的 `instruction` SHALL 含一段 TDD 的說明：寫明任務依 Red-Green-Refactor 的順序安排（每個工作單位先有一項寫出失敗測試的任務，再有一項讓它通過的實作任務），並寫明完整的 TDD 紀律由 apply 階段提供、寫任務時不需要取得。audit 的有效值為開啟時，`instruction` SHALL 含一段 audit 的說明：寫明任務新增 API、設定選項或參數處理時要有一個套用 sharp-edges audit checklist 的步驟，並寫明檢查清單由 apply 階段提供、寫任務時不需要取得。

這兩段 SHALL NOT 含 `instructions --skill`，SHALL NOT 指示執行任何經宿主套件的 dispatch 會被拒絕的命令。兩個開關都關閉時，`instruction` SHALL 維持既有的位元級輸出。這兩段是英文，內容不隨 locale（tw、ja、en 或未設定）改變。本機 repo、連到遠端的 CLI 與宿主套件的 dispatch SHALL 得到相同的文字。

這是刻意的輸出變更。變更前：兩段的結尾分別是「fetch the full discipline with speclink instructions --skill tdd」與「fetch it with speclink instructions --skill audit」。

#### Scenario: tdd 開啟時指示不要求取得技能文字

- **WHEN** speclink/config.yaml 設定 tdd: true，執行 speclink instructions tasks --change 某 change --json
- **THEN** exit code 為 0；instruction 含 Red-Green-Refactor 與「The apply step supplies the full TDD discipline」；instruction 不含 instructions --skill

#### Scenario: audit 開啟時指示不要求取得技能文字

- **WHEN** speclink/config.yaml 設定 audit: true，執行 speclink instructions tasks --change 某 change --json
- **THEN** instruction 含 sharp-edges audit checklist 與「The apply step supplies the checklist」；instruction 不含 instructions --skill

#### Scenario: 兩個開關都關閉時指示不變

- **WHEN** speclink/config.yaml 未設定 tdd 與 audit，執行 speclink instructions tasks --change 某 change --json
- **THEN** instruction 不含 TDD 與 audit 的說明，與本需求之前的輸出逐位元相同

#### Scenario: 經宿主套件讀任務指示

- **WHEN** 宿主套件的範圍設定 tdd: true 與 audit: true，以 dispatch 執行 instructions tasks --change 某 change --json
- **THEN** 回傳物件的 instruction 含兩段說明，不含 instructions --skill

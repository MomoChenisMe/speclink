## MODIFIED Requirements

### Requirement: instructions 對 specs 產出物回報解析後的規格語言

<!-- BEFORE: 語言說明只講結構標記與規範關鍵字維持英文，沒有說需求與情境的名稱要用哪一種語言、哪些名稱不能改 -->

speclink instructions specs --change <名> --json 的 payload 的 `locale` 欄位（字串）SHALL 是規格語言的顯示名稱。規格語言 SHALL 由 spec_locale 的有效值決定，有效值沿用工作流政策的三層解析（SPECLINK_SPEC_LOCALE ＞ speclink/config.yaml ＞ 內建預設）：

- spec_locale 是 `tw` 時 SHALL 為 `Traditional Chinese (繁體中文)`，是 `ja` 時 SHALL 為 `Japanese (日本語)`，是 `en` 時 SHALL 為 `English`。
- spec_locale 是 `auto` 時 SHALL 為 locale 有效值的顯示名稱；locale 未設定時 SHALL 為 `English`。
- spec_locale 未設定時 SHALL 為 `English`，不論 locale 的值。

其他產出物（proposal、design、tasks）與 speclink instructions apply 的 `locale` SHALL 維持 locale 有效值的顯示名稱。payload 的欄位名稱與型別 SHALL 不變。

同一份 payload 的 `instruction` SHALL 以一段寫明規格語言的文字結尾，規格語言是 English 時也寫：這一段 SHALL 含 `Write spec prose in <顯示名稱>`，`<顯示名稱>` 與同一份 payload 的 `locale` 相同；SHALL 說明結構標記（`## ADDED/MODIFIED/REMOVED/RENAMED Requirements`、`### Requirement:`、`#### Scenario:`、`- **WHEN**`／`- **THEN**`）與規範關鍵字（SHALL、MUST、SHALL NOT、MUST NOT）維持英文。規格語言是 `tw` 時，這一段 SHALL 另含中文模糊用語（應該、可能、也許、或許、大概、考慮、盡量、待定）會被 analyzer 標記的提醒；規格語言是 `ja` 或 English 時 SHALL NOT 含這項提醒。`instruction` SHALL NOT 含 `config.yaml`、`English by default` 與 `spec_locale` 這三段文字。

同一段語言說明 SHALL 寫明需求與情境名稱的規則，三種規格語言（`tw`、`ja`、English）都寫，位置在結構標記與規範關鍵字的說明之後、中文模糊用語的提醒之前：

- 名稱是內文，跟著規格語言寫：這一段 SHALL 含 `Requirement and scenario names are prose too: write them in <顯示名稱>`，`<顯示名稱>` 與同一份 payload 的 `locale` 相同。
- 指向正式規格的名稱照正式規格的原文，不論原文是哪一種語言：這一段 SHALL 含 `exactly as the canonical spec writes it`，並 SHALL 逐一點名四種名稱——`## MODIFIED Requirements` 與 `## REMOVED Requirements` 底下的需求名稱、RENAMED 項目的 `FROM:` 名稱、MODIFIED 區塊沿用的正式規格情境名稱、`<!-- REMOVED-SCENARIO: … -->` 宣告裡的名稱。
- 這一段 SHALL 說明改掉這些名稱會讓變更的驗證失敗。

其他產出物（proposal、design、tasks）與 speclink instructions apply 的 `instruction` SHALL NOT 含名稱的規則。schema 的 specs 產出物沒有 instruction 時，這一段 SHALL 是整個 `instruction`。

人眼輸出（不帶 --json）SHALL 維持不顯示 `locale` 欄位，它的 Instruction 區段 SHALL 是同一段 `instruction` 文字（因此以同一段語言說明結尾）；`--no-color` 下文字相同、不含色碼。本機 repo、連到遠端的 CLI（payload 由 Server 以同一個入口計算，CLI 原樣輸出）與宿主套件的 dispatch SHALL 對相同的設定得到相同的 `locale` 與 `instruction`。speclink/config.yaml 無法解析時 SHALL 照「工作流政策的正典歸屬與三層解析順序」以非零 exit code 失敗，SHALL NOT 以預設語言繼續；找不到變更時 SHALL 以非零 exit code 失敗、stderr 指出找不到該變更，stdout 不輸出 payload。

這是刻意的輸出變更。變更前：specs 的 `instruction` 最後一段只說明結構標記與規範關鍵字維持英文（規格語言是 `tw` 時另有中文模糊用語的提醒），沒有名稱的規則；`locale` 欄位的值不變。

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

#### Scenario: 語言說明寫明名稱的規則

- **WHEN** speclink/config.yaml 設定 locale: tw 與 spec_locale: tw，執行 speclink instructions specs --change 某 change --json
- **THEN** exit code 為 0；instruction 的最後一段含 Requirement and scenario names are prose too: write them in Traditional Chinese (繁體中文)、exactly as the canonical spec writes it、## MODIFIED Requirements、## REMOVED Requirements、FROM: 與 REMOVED-SCENARIO；這些文字都出現在中文模糊用語的提醒之前

#### Scenario: 規格語言是 English 或日文時也寫名稱的規則

- **WHEN** speclink/config.yaml 未設定 spec_locale，以及設定 spec_locale: ja，分別執行 speclink instructions specs --change 某 change --json
- **THEN** 兩次的 instruction 最後一段都含 exactly as the canonical spec writes it 與 REMOVED-SCENARIO；未設定時含 write them in English，ja 時含 write them in Japanese (日本語)；兩次都不含中文模糊用語的提醒

#### Scenario: 其他產出物的指示沒有名稱的規則

- **WHEN** 以 spec_locale: tw 分別執行 speclink instructions proposal、design、tasks 與 apply（同一個 change，帶 --json）
- **THEN** 四份 payload 的 instruction 都不含 Requirement and scenario names are prose

#### Scenario: 人眼輸出帶同一段名稱規則

- **WHEN** 以 spec_locale: tw 執行 speclink instructions specs --change 某 change（不帶 --json），再加上 --no-color 執行一次
- **THEN** 兩次的 Instruction 區段都以含 Write spec prose in Traditional Chinese (繁體中文) 與 exactly as the canonical spec writes it 的同一段文字結尾；加 --no-color 的輸出不含色碼

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

## ADDED Requirements

### Requirement: instructions 找不到產出物代號時列出可用的代號

speclink instructions <代號> --change <名> 的 `<代號>` 不是 `apply`、也不是該變更的產出流程定義的產出物代號時，命令 SHALL 以非零 exit code 失敗，stdout SHALL NOT 輸出任何 payload，stderr 的錯誤訊息 SHALL 是 `Artifact '<代號>' not found in schema. Use <清單>, or apply`。`<清單>` SHALL 是該變更的產出流程定義的全部產出物代號，以 `, ` 分隔，順序與 speclink status --change <名> --json 的 `artifacts` 相同。`apply` 在訊息裡 SHALL 只出現一次：產出流程定義了代號為 `apply` 的產出物時，`<清單>` SHALL NOT 再列它；產出流程沒有定義任何產出物時，訊息 SHALL 是 `Artifact '<代號>' not found in schema. Use apply`。帶 --json 與不帶 --json 時錯誤訊息 SHALL 相同；--no-color 下文字相同、不含色碼。錯誤訊息是英文，SHALL NOT 隨 locale 或 spec_locale（`tw`、`ja`、`en` 或未設定）改變。

命令 SHALL NOT 把清單以外的寫法當成某個產出物接受：`specs/<capability>`（speclink artifact cat 讀 delta 規格的寫法）與 `spec`（speclink new artifact 寫 delta 規格的寫法）都 SHALL 得到上述錯誤。清單內的代號與 `apply` 的行為 SHALL 不變。

本機 repo、連到遠端的 CLI（錯誤由 Server 以同一個入口產生，CLI 原樣輸出）與宿主套件的 dispatch SHALL 得到相同的訊息文字；經 dispatch 時錯誤碼 SHALL 維持 `not_found`。找不到變更時 SHALL 維持既有的錯誤（stderr 指出找不到該變更），SHALL NOT 改成列出產出物代號。這個命令不寫入任何檔案。

這是刻意的輸出變更。變更前：錯誤訊息只有 `Artifact '<代號>' not found in schema`，不列可用的代號。

#### Scenario: 把讀取用的寫法拿來取指示

- **WHEN** 對一個 spec-driven 的變更執行 speclink instructions specs/auth --change 該變更 --json
- **THEN** exit code 非 0；stdout 沒有輸出；stderr 含 Artifact 'specs/auth' not found in schema. Use proposal, design, specs, tasks, or apply

#### Scenario: 沒有 specs 產出物的變更不列 specs

- **WHEN** 對一個 no-spec-change 的變更執行 speclink instructions specs --change 該變更
- **THEN** exit code 非 0；stderr 含 Artifact 'specs' not found in schema. Use proposal, design, tasks, or apply

#### Scenario: 清單內的代號照常回應

- **WHEN** 對一個 spec-driven 的變更分別執行 speclink instructions specs 與 speclink instructions apply（帶 --change 與 --json）
- **THEN** 兩次的 exit code 都是 0，stdout 各輸出一份 payload

#### Scenario: 經宿主套件得到相同的錯誤

- **WHEN** 宿主套件以 dispatch 執行 instructions specs/auth --change 某 spec-driven 變更 --json
- **THEN** dispatch 以錯誤結束，錯誤碼為 not_found，訊息含 Artifact 'specs/auth' not found in schema. Use proposal, design, specs, tasks, or apply

#### Scenario: 找不到變更時維持既有錯誤

- **WHEN** 執行 speclink instructions specs/auth --change 不存在的名稱 --json
- **THEN** exit code 非 0；stderr 指出找不到該變更，不含 Use proposal；stdout 沒有輸出

##### Example: 錯誤訊息的清單

| 變更的產出流程 | 執行的代號 | stderr 的訊息 |
| -------------- | ---------- | ------------- |
| spec-driven | specs/auth | Artifact 'specs/auth' not found in schema. Use proposal, design, specs, tasks, or apply |
| spec-driven | spec | Artifact 'spec' not found in schema. Use proposal, design, specs, tasks, or apply |
| no-spec-change | specs | Artifact 'specs' not found in schema. Use proposal, design, tasks, or apply |

## ADDED Requirements

### Requirement: tasks 指示說明短標題的任務形狀

內建 schema `spec-driven` 與 `no-spec-change` 的 tasks instruction SHALL 各含一條指引，說明每個任務的形狀為「短標題、分隔符號、行為、最後一段為驗證方式」，並 SHALL 列出三組固定的標記與各自的形狀：繁體中文為全形「：」與「驗證：」，日文為全形「：」與「検証：」，英文為「半形冒號後接一個空白」與「Verify: 」。指引 SHALL 說明：依 instructions JSON 的 `locale` 選用一組，三者以外的語言用英文那一組；標題為短語而非句子，且不含冒號；標記本身不翻譯、不改寫。instruction 的 Example 區塊裡的每個任務 SHALL 符合英文的形狀。兩個 schema 共用的 tasks 範本，其每個任務的佔位文字 SHALL 為 `<!-- Short title: behavior/contract delivered. Verify: verification target -->`，範本開頭的說明 SHALL 提到這個形狀。

這條指引 SHALL NOT 成為格式守門：任務文字不符合這個形狀時，`speclink new artifact tasks`、`speclink validate` 與 `speclink analyze` 的結果 SHALL 與符合時相同。`no-spec-change` 的 tasks instruction 既有的要求（最後一個任務以既有測試確認行為未變、不要求引用 requirement 名稱）SHALL 維持不變。`speclink schema fork` 的輸出 SHALL 仍與正典 YAML 逐位元組相同。專案層與使用者層的自訂 schema SHALL NOT 被改寫。

#### Scenario: spec-driven 的 tasks 指示列出三組標記

- **WHEN** 對 schema 為 spec-driven 的變更 `demo` 執行 `speclink instructions tasks --change demo --json`
- **THEN** exit code 為 0；`instruction` 含「驗證：」「検証：」與「Verify: 」三個驗證標記，以及繁體中文、日文、英文三種形狀各一行；`instruction` 的 Example 區塊裡每個任務都含「: 」與「Verify: 」；`template` 的四個任務佔位文字都是 `<!-- Short title: behavior/contract delivered. Verify: verification target -->`，不含舊的佔位文字 `<!-- Behavior/contract delivered + verification target -->`

#### Scenario: no-spec-change 的 tasks 指示含同一條指引

- **WHEN** 對 schema 為 no-spec-change 的變更 `tidy` 執行 `speclink instructions tasks --change tidy --json`
- **THEN** exit code 為 0；`instruction` 含同樣的三組標記與三種形狀；`instruction` 仍要求最後一個任務以既有測試確認行為未變；`template` 與 spec-driven 的相同

#### Scenario: 不照形狀寫的任務清單照常通過

- **WHEN** 以 `speclink new artifact tasks --change demo --stdin` 寫入只有一個任務「- [ ] 1.1 一句沒有冒號也沒有驗證標記的長句」的任務清單，接著執行 `speclink validate demo`
- **THEN** 寫入的 exit code 為 0；`validate` 的輸出沒有任何一條訊息提到任務的形狀、標題或標記

#### Scenario: fork 的輸出帶著新的指引

- **WHEN** 執行 `speclink schema fork spec-driven my-flow` 並讀取產出的 schema.yaml
- **THEN** tasks 的 instruction 含三組標記的指引，檔案內容與內建的正典 YAML 逐位元組相同

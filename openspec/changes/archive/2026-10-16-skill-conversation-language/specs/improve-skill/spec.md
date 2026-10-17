## ADDED Requirements

### Requirement: 改進討論記錄的撰寫語言依 locale

渲染產出的 speclink-improve 技能檔 SHALL 在記錄 candidates 的步驟規定記錄的撰寫語言：以 `speclink workflow-config show --json` 讀取 `locale` 欄位，記錄的 topic、Context、各輪內容與結論散文 SHALL 以該語言撰寫——`tw` 為繁體中文、`ja` 為日文、`en` 或 null 為英文；指令失敗時 SHALL 以英文撰寫。improve 的 topic 由模型撰寫，因此同樣依 `locale`。欄位標記（`**Focus**:`、`**Position**:`、`**Ruled out**:`、`**Open**:` 與結論各欄位標籤）、固定標記（單字 `none`、``**cut N `change-name`**:`` 刀頭與 **Capture to** 的值）、slug（`improve-<scope>`）、mode 標籤 `scan`、指令行與程式碼 SHALL 維持英文。因記錄散文依 `locale` 撰寫，防重提檢查的 `speclink discuss search` SHALL 同時以 `locale` 語言的關鍵字與英文識別字（crate、模組、指令名）搜尋。結尾自我檢查清單 SHALL 以一句話指回此規則。對話本身的語言依共用對話語言段落，SHALL NOT 由 `locale` 決定。

此為刻意變更：變更前技能未規定記錄語言；變更後依 `locale`，未設定 `locale` 時為英文。

#### Scenario: 渲染產物含記錄語言規則

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** speclink-improve 技能檔在記錄 candidates 的步驟含讀取 `speclink workflow-config show --json` 的 `locale` 欄位、依該語言撰寫 topic 與記錄散文、欄位標記與固定標記及 slug 維持英文的規定；防重提檢查步驟含以 `locale` 語言與英文識別字一起搜尋的規定

#### Scenario: 防重提檢查用兩種語言的關鍵字

- **WHEN** 專案設 `locale: tw`，improve 在掃描前執行防重提檢查
- **THEN** 傳給 `speclink discuss search` 的關鍵字同時含繁體中文詞與英文識別字（例如 crate 名），以命中以繁體中文撰寫的既有否決行

#### Scenario: locale ja 時記錄為日文

- **WHEN** 專案設 `locale: ja`，使用者以繁體中文執行 improve 並完成掃描
- **THEN** Agent 以繁體中文跟使用者對話；新建記錄的 topic、Context 與 Round 1 的 candidates 內容為日文，slug 為 `improve-<scope>` 的英文形式

##### Example: 記錄的撰寫語言

| locale | 記錄的 topic 與散文 | 欄位標記、slug 與 mode 標籤 |
| ------ | ------------------- | --------------------------- |
| tw | 繁體中文 | 英文 |
| ja | 日文 | 英文 |
| en | 英文 | 英文 |
| 未設定 | 英文 | 英文 |

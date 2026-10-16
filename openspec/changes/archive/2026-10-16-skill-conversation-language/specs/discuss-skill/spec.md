## ADDED Requirements

### Requirement: 討論記錄的撰寫語言依 locale

渲染產出的 speclink-discuss 技能檔 SHALL 在建立討論記錄的步驟規定記錄的撰寫語言：以 `speclink workflow-config show --json` 讀取 `locale` 欄位，新建記錄的 Context、各輪 Focus／Position／Ruled out／Open 的內容與結論散文 SHALL 以該語言撰寫——`tw` 為繁體中文、`ja` 為日文、`en` 或 null 為英文；指令失敗時 SHALL 以英文撰寫。topic SHALL 維持使用者輸入的原文（沿用「討論技能指示要求英文 slug」的 topic 規定）。欄位標記（`**Focus**:`、`**Position**:`、`**Ruled out**:`、`**Open**:` 與結論的 `**Decision**:`、`**Rationale**:`、`**Rejected alternatives**:`、`**Deferred**:`、`**Capture to**:`、`**Next**:`）、固定標記（`Prior discussions:`、`Source doc:`、單字 `none`、``**cut N `change-name`**:`` 刀頭與 **Capture to** 的值）、slug、指令行與程式碼 SHALL 維持英文。恢復已存在的記錄時 SHALL 沿用該記錄原本的語言，SHALL NOT 依 `locale` 改寫既有內容或在新增的輪次改用另一種語言；恢復流程（Resume ritual）SHALL 以一句話指回此規則。對話本身的語言依共用對話語言段落，SHALL NOT 由 `locale` 決定。

此為刻意變更：變更前技能未規定記錄語言，記錄通常跟著對話語言；變更後新建記錄依 `locale`，未設定 `locale` 時為英文。

#### Scenario: 渲染產物含記錄語言規則

- **WHEN** 執行 speclink init 或 speclink update 渲染 claude 與 codex 工具的技能檔
- **THEN** speclink-discuss 技能檔在建立記錄的步驟含讀取 `speclink workflow-config show --json` 的 `locale` 欄位、依該語言撰寫記錄散文、欄位標記與固定標記（`Prior discussions:`、`Source doc:`、`none`、cut 刀頭、Capture to 的值）及 slug 維持英文，以及恢復既有記錄沿用原語言的規定；Resume ritual 與結尾自我檢查清單各以一句話指回此規則

#### Scenario: locale 與對話語言不同時記錄依 locale

- **WHEN** 專案 openspec/config.yaml 設 `locale: tw`，使用者以英文開啟一個新討論
- **THEN** Agent 以英文跟使用者對話；新建記錄的 Context、各輪內容與結論為繁體中文，`**Focus**:` 等欄位標記與 slug 為英文，topic 為使用者輸入的英文原文

#### Scenario: 恢復既有記錄沿用原語言

- **WHEN** 專案未設定 `locale`，使用者恢復一份以繁體中文撰寫、status 為 open 的既有記錄並新增一輪
- **THEN** 新增輪次的內容為繁體中文，既有輪次內容不變

##### Example: 新建記錄的散文語言

| locale | `workflow-config show --json` 結果 | 新建記錄的散文 | 欄位標記與 slug |
| ------ | ---------------------------------- | -------------- | --------------- |
| tw | `"locale": "tw"` | 繁體中文 | 英文 |
| ja | `"locale": "ja"` | 日文 | 英文 |
| en | `"locale": "en"` | 英文 | 英文 |
| 未設定 | `"locale": null` | 英文 | 英文 |
| 任一 | 指令失敗 | 英文 | 英文 |

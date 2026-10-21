## ADDED Requirements

### Requirement: 依是否改到正式規格選擇 schema

propose 技能 SHALL 在建立變更之前決定它的 schema，依下列順序取第一個成立的規則：

1. 使用者在需求或參數中指定 schema → 用指定的 schema。
2. 專案預設 schema（`speclink workflow-config show --json` 的 `schema`）是自訂 schema → 用專案預設，不做判定。
3. 需求來自討論（`--from-discussion`）→ 依討論結論判定：結論寫明不改任何正式規格承諾的行為、或點名 `no-spec-change` → `no-spec-change`；結論會新增、修改或移除行為 → `spec-driven`。
4. 其餘 → 依需求內容與步驟 3 的正式規格掃描判定「這個變更會不會改到正式規格承諾的行為？」：不會 → `no-spec-change`；會 → `spec-driven`。修 bug 預設為 `spec-driven`，因為通常要寫 delta 把正確行為釘住；只有能說明「沒有任何正式規格承諾被改到」時才選 `no-spec-change`。

規則 3 或 4 判斷不了時，技能 SHALL 以兩個選項詢問使用者（建議的選項放第一個並寫出理由），SHALL NOT 自行猜測。技能 SHALL 以 `speclink new change "<name>" --schema <選定的 schema>` 建立變更。選定 `no-spec-change` 時，proposal SHALL 依該 schema 的 template 撰寫（不含 Capabilities），Why SHALL 寫明為什麼不改正式規格承諾的行為並列出查過的正式規格；收尾摘要 SHALL 寫出選定的 schema，`no-spec-change` 以「不改規格（no-spec-change）」呈現。

#### Scenario: 討論結論寫明不改行為

- **WHEN** 使用者以 `--from-discussion` 執行 propose，討論結論寫明「純重構，不改任何正式規格承諾的行為」，使用者沒有指定 schema，專案預設 schema 為 spec-driven
- **THEN** 技能以 `--schema no-spec-change` 建立變更，proposal 沒有 Capabilities 段，Why 列出查過的正式規格，收尾摘要寫「不改規格（no-spec-change）」

#### Scenario: 修 bug 預設寫 delta

- **WHEN** 使用者以「修正封存時日期錯一天」為需求執行 propose，沒有指定 schema，該行為由正式規格承諾
- **THEN** 技能以 `--schema spec-driven` 建立變更，並寫出修正該行為的 delta spec

#### Scenario: 使用者指定優先

- **WHEN** 使用者執行 propose 時寫明「用 spec-driven」，需求內容看起來是不改行為的重構
- **THEN** 技能以 `--schema spec-driven` 建立變更，不再詢問

#### Scenario: 專案預設為自訂 schema

- **WHEN** 專案 `openspec/config.yaml` 的 `schema` 為自訂的 `my-flow`，使用者沒有指定 schema
- **THEN** 技能以專案預設 `my-flow` 建立變更，不做 spec-driven 與 no-spec-change 的判定

#### Scenario: 判斷不了時詢問

- **WHEN** 需求為「調整匯出檔的欄位順序」，技能無法從正式規格判斷欄位順序是否為承諾的行為
- **THEN** 技能以兩個選項詢問使用者，建議的選項放第一個並附理由；使用者回答前不建立變更

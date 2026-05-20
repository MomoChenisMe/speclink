## Instruction

撰寫 `specs/<capability>/spec.md`（delta 形式）：以 SHALL / MUST 等 normative 字眼釘住「change 套用後該 capability 的契約變動」。這是 archive 時會 merge 入主 spec 的 delta。

思考順序：
1. 決定 delta heading 類型：`## ADDED Requirements` / `## MODIFIED Requirements` / `## REMOVED Requirements` / `## RENAMED Requirements`
2. 每個 requirement 以 `### Requirement: <Requirement name>` 標題
3. requirement body 使用 SHALL / MUST / SHOULD / MAY；避免 should / may（小寫含糊）
4. 為 requirement 加 `#### Scenario:` 區塊描述具體情境（GIVEN/WHEN/THEN）
5. Spec 一律使用英文撰寫（與 locale 無關），因為 normative 詞彙專有

邊界：
- 不寫實作步驟（屬於 `design.md` / `tasks.md`）
- 不引入未在 proposal `Capabilities` 中列出的 capability
- `MODIFIED` / `REMOVED` / `RENAMED` 中提及的 requirement 必須已存在於主 spec

## Template

## ADDED Requirements

### Requirement: <Requirement name>

The <subject> SHALL <observable behavior>.

#### Scenario: <Scenario name>

- **GIVEN** <precondition>
- **WHEN** <action>
- **THEN** <observable outcome>

## MODIFIED Requirements

### Requirement: <Existing requirement name>

<新的 requirement body — 必須以 normative 字眼描述變動後行為>

#### Scenario: <Scenario name>

- **GIVEN** <precondition>
- **WHEN** <action>
- **THEN** <observable outcome>

## REMOVED Requirements

### Requirement: <Existing requirement name>

<被移除的 requirement 名稱列出即可；body 可空白或寫移除理由>

## RENAMED Requirements

- FROM: `<Old requirement name>`
- TO: `<New requirement name>`

## Rules

- [error] spec.must_use_shall_must: Requirement body SHALL 使用 SHALL / MUST 而非小寫的 should / may，以保持 normative 強度。
- [error] spec.delta_heading_must_be_known: Delta heading SHALL 限於 `## ADDED Requirements` / `## MODIFIED Requirements` / `## REMOVED Requirements` / `## RENAMED Requirements` 四種。
- [warning] spec.should_include_scenario: 每個 requirement SHOULD 至少包含一個 `#### Scenario:` 區塊以利驗證。

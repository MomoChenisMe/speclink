## Instruction

撰寫 `design.md`：說明「HOW」— 怎麼把 proposal 中的 What 落地。讀者是負責實作的 AI agent 或工程師。

思考順序：
1. `## Context`：繼承自前一個 change 或專案的設計約束（不要重複 proposal 已寫過的）
2. `## Goals / Non-Goals`：清晰列出本 change 必須達成 vs 明確排除的事項
3. `## Decisions`：每個重要設計決策一節，包含「理由」與「替代方案」
4. `## Implementation Contract`：釘住 observable behavior、interface 命名、failure modes、acceptance criteria、scope boundaries
5. `## Risks / Trade-offs`、`## Migration Plan`、`## Open Questions`

邊界：
- 不寫具體 task 切割（屬於 `tasks.md`）
- 不寫單一 requirement 的 normative 字眼（屬於 `specs/<capability>/spec.md`）
- Implementation Contract 是 task 完成後可驗證的契約，不是實作步驟

## Template

## Context

<!-- 設計約束、繼承自前一 change 的決策 -->

## Goals / Non-Goals

**Goals:**

- <goal 1>
- <goal 2>

**Non-Goals:**

- <non-goal 1>

## Decisions

### <Decision title>

<!-- 決策描述 -->

**理由**：<理由>

**替代方案**：

- **<alt 1>**：拒絕原因
- **<alt 2>**：拒絕原因

## Implementation Contract

**Observable behavior**: <可觀察行為>

**Interface (命名)**: <型別/函式/檔案命名>

**Failure modes**: <錯誤對應 exit code / error code 表格>

**Acceptance criteria**: <可驗證條件>

**Scope boundaries**: <in scope / out of scope>

## Risks / Trade-offs

- <風險與緩解>

## Migration Plan

<!-- 對既有資料 / behaviour 的遷移策略；N/A 若純新增 -->

## Open Questions

<!-- 尚未決定的開放問題 -->

## Rules

- [error] design.must_include_context: Design SHALL 包含 `## Context` heading 說明設計約束來源。
- [error] design.must_include_implementation_contract: Design SHALL 包含 `## Implementation Contract` heading 釘住 observable behavior 與 acceptance criteria。
- [warning] design.should_list_alternatives: Design SHOULD 對每個重要決策列出替代方案與拒絕理由。

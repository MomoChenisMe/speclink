## Instruction

撰寫 `proposal.md`：以 SDD 的角度說明「為什麼要做這個 change」與「具體會改變什麼」。讀者是另一個 AI agent 或工程師，目的是在不讀程式碼前就能評估提案的價值與風險。

思考順序：
1. 先寫 `## Why`：用 1-3 段說明動機；連結到具體的痛點、bug、或新需求
2. 再寫 `## What Changes`：以 bullet list 列出對使用者或開發者可觀察的差異
3. 列出 `## Capabilities`：標明 New / Modified；capability 名稱需符合 kebab-case
4. 最後 `## Impact`：影響到的 specs、crates、檔案（new / modified / removed）

邊界：
- 不寫實作步驟（屬於 `design.md` 範疇）
- 不寫具體任務切割（屬於 `tasks.md` 範疇）
- 不引入外部設定檔（如 YAML / TOML）除非在 `Non-Goals` 中明確列出

## Template

## Why

<!-- 為什麼需要這個 change：痛點、機會、外部需求 -->

## What Changes

<!-- 對使用者或開發者可觀察的差異 -->

## Non-Goals (optional)

<!-- 範圍排除與被拒絕的替代方案 -->

## Capabilities

### New Capabilities

- `<capability-name>`: <brief description>

### Modified Capabilities

(none)

## Impact

- Affected specs: <new or modified capabilities>
- Affected code:
  - New: <paths to be created, relative to project root>
  - Modified: <paths that already exist>
  - Removed: <paths to be deleted>

## Rules

- [error] proposal.must_include_why: Proposal SHALL 包含 `## Why` heading，且內容非空。
- [error] proposal.must_list_capabilities: Proposal SHALL 包含 `## Capabilities` heading 並至少列出一個 New 或 Modified capability。
- [warning] proposal.should_list_impact: Proposal SHOULD 包含 `## Impact` heading 列出被影響的 specs / code。

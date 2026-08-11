<!-- SPECLINK:START v1.19.10 -->

# Speclink Instructions

This project uses Speclink for Spec-Driven Development(SDD). Specs live in `openspec/specs/`, change proposals in `openspec/changes/`, discussion records in `openspec/discussions/`.

## Use `$speclink-*` skills when:

- Requirements are fuzzy or worth debating → `$speclink-discuss` (recorded as a document; promote turns it into a change)
- User asks for improvements without naming a topic → `$speclink-improve` (user-initiated only; scans the codebase and records the candidates as a discussion)
- User wants to plan, propose, or design a change → `$speclink-propose` (`--from-discussion <slug>` seeds it from a concluded discussion)
- Adopting Speclink on an existing codebase → `$speclink-onboard`
- Tasks are ready to implement → `$speclink-apply`
- Implementing several independent changes at once → `$speclink-apply-with-worktree` (one git worktree per change)
- A worktree change is committed and ready to land → `$speclink-worktree-merge` (merge back, then clean up)
- Resuming a change that sat idle → run `$speclink-drift` first
- Requirements change mid-work → `$speclink-ingest`
- Implementation is done, before archiving → optional quality stations `$speclink-review` (craft quality) ∥ `$speclink-verify` (spec compliance; user's call), then `$speclink-archive`
- Both quality stations over one change → `$speclink-quality` (both checks first without stamping, then it stops after every round for your call on what to fix and when to stamp); only one station → call `$speclink-review` or `$speclink-verify` directly
- Commit only files related to a specific change → `$speclink-commit`

## Workflow

discuss?/improve? → propose → apply ⇄ ingest → (quality? | review? ∥ verify?) → archive

worktree: apply-with-worktree ⇄ ingest → (quality? | review? ∥ verify?) → worktree-merge → archive (main checkout)

- `discuss` is optional — skip if requirements are clear; conclude and archive it even when the outcome is "don't do it"
- A promoted discussion is archived automatically with its last remaining change (one discussion can fan out into several changes)
- Resuming after a pause? Run `drift` first — stale delta assumptions route to `ingest`
- Quality stations belong inside the worktree (the Apply baseline lives there); archive runs only from the main checkout — archiving inside a linked worktree is refused by the engine
- Requirements change mid-work? `ingest` → resume `apply`

<!-- SPECLINK:END -->

---

## 工作指引

### 專案慣例

- **開發一律走 TDD**：新功能、修 bug、重構都用 `tdd-workflow` 技能（先測試 → 實作 → 重構），覆蓋率目標 80% 以上（unit / integration / E2E）。這是硬規定，不是選項。
- **Commit 用 `conventional-commit` 技能**：description 與 body 寫繁體中文。例：`feat(auth): 新增 GitHub OAuth 登入`、`fix(collab): 修正 Y.js 同步衝突`。
- **查資料順序**：專案內既有規格與程式碼（搭配對應 Skills）→ `context7` 等已接的 MCP → 最後才網路搜尋。不要憑印象猜 API 簽名。

### 程式碼取捨

- 不為一次性使用的程式碼建立抽象層；不加沒被要求的「彈性」或「可配置性」。
- 錯誤處理只在系統邊界驗證（使用者輸入、外部 API），內部程式碼當成可信；不為不可能發生的情境寫防護。
- 寫了 200 行但 50 行能搞定就重寫。判準：資深工程師會不會覺得這太複雜？
- 死碼不對稱處理：清掉「因為這次改動」而孤兒化的 imports／變數／函式；原本就存在的死碼提一句就好，不要刪。
- 有更簡單的做法直接說出來，並講清楚你的假設。

### 表達習慣

- 用「白話」、「白話」、「白話」！很重要！
- 請不要過多的術語，若有術語請附上說明。

<!-- 這份檔案刻意保持精簡。已內建於 Claude Code 的通則不重寫（配合現有風格、不擴張任務範圍、驗證前不宣稱成功、
     模糊處自行判斷而非預設停下來問）——重寫只會與內建指引打架。
     專案地雷寫在根目錄 CLAUDE.md；某個情境需要更細的規範時，做成技能後在此處 @ 引用，不要把規則塞回這裡。 -->


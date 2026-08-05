<!-- SPECLINK:START v1.10.0 -->

# Speclink Instructions

This project uses Speclink for Spec-Driven Development(SDD). Specs live in `openspec/specs/`, change proposals in `openspec/changes/`, discussion records in `openspec/discussions/`.

## Use `$speclink-*` skills when:

- Requirements are fuzzy or worth debating → `$speclink-discuss` (recorded as a document; promote turns it into a change)
- User wants to plan, propose, or design a change → `$speclink-propose` (`--from-discussion <slug>` seeds it from a concluded discussion)
- Adopting Speclink on an existing codebase → `$speclink-onboard`
- Tasks are ready to implement → `$speclink-apply`
- Implementing several independent changes at once → `$speclink-apply-with-worktree` (one git worktree per change)
- A worktree change is committed and ready to land → `$speclink-worktree-merge` (merge back, then clean up)
- Resuming a change that sat idle → run `$speclink-drift` first
- Requirements change mid-work → `$speclink-ingest`
- Implementation is done, before archiving → optionally `$speclink-review` (craft quality; user's call), then `$speclink-archive`
- Commit only files related to a specific change → `$speclink-commit`

## Workflow

discuss? → propose → apply ⇄ ingest → review? → archive

- `discuss` is optional — skip if requirements are clear; conclude and archive it even when the outcome is "don't do it"
- A promoted discussion is archived automatically with its last remaining change (one discussion can fan out into several changes)
- Resuming after a pause? Run `drift` first — stale delta assumptions route to `ingest`
- Requirements change mid-work? `ingest` → resume `apply`

<!-- SPECLINK:END -->

---

## 開發流程

### TDD 開發

所有開發一律採用 TDD 流程，使用 `tdd-workflow` 技能引導：

- 先寫測試 → 再寫實作 → 最後重構
- 目標覆蓋率 80% 以上（unit、integration、E2E）

### 基本 Git 提交規範

Git commit 訊息使用 `conventional-commit` 技能，遵循 Conventional Commits v1.0.0：

- description 和 body 使用繁體中文
- 範例：`feat(auth): 新增 GitHub OAuth 登入`、`fix(collab): 修正 Y.js 同步衝突`

---

## 核心原則

### 語氣控制

- ✅「這可能解決問題，測試看看...」
- ❌「完美！問題已解決！」（驗證前禁止宣稱成功）

### 動手前先思考

**不要假設。不要隱藏困惑。把取捨攤開來談。**

在開始實作之前：

- 明確說出你的假設。不確定就問。
- 如果有多種解讀方式，列出來討論——不要默默選一個。
- 如果有更簡單的做法，直接說。該反對就反對。
- 如果有不清楚的地方，停下來。指出哪裡讓你困惑，然後問。

### 保持簡單

**用最少的程式碼解決問題。不做任何臆測性的設計。**

- 不做超出需求的功能。
- 不為一次性使用的程式碼建立抽象層。
- 不加沒被要求的「彈性」或「可配置性」。
- 不為不可能發生的情境做錯誤處理。只在系統邊界驗證（用戶輸入、外部 API），信任內部程式碼。
- 如果寫了 200 行但 50 行就能搞定，就重寫。

問自己一句：「資深工程師會不會覺得這太複雜？」如果會，就簡化。

### 外科手術式的變更

**只動該動的地方。只清理自己造成的混亂。**

修改現有程式碼時：

- 不要「順手改進」旁邊的程式碼、註解或排版。
- 不要重構沒壞的東西。
- 配合現有風格，即使你的習慣不同。
- 看到不相關的死碼，提一下就好——不要刪。

當你的變更造成孤兒程式碼時：

- 移除「因為你的變更」而變成沒人用的 imports、變數、函式。
- 不要刪除原本就存在的死碼，除非有人要求。

判準：每一行改動都應該能直接對應到使用者的需求。

### 文檔查詢優先順序

| 優先級 | 來源     | 說明                                            |
| ------ | -------- | ----------------------------------------------- |
| 1      | 專案本地 | 搜尋現有規格、程式碼、搭配對應 Skills |
| 2      | MCP 工具 | `context7` 等已整合的 MCP server                |
| 3      | 網路搜尋 | 僅當以上無法解決時                              |

> 先看專案規格和程式碼怎麼做，再查官方文檔，最後才 websearch。不假設 API 簽名。

### 溝通與表達

- 儘量使用簡單明的中文表達，避免使用過於複雜的英文術語，例如：「YAGNI（You Aren't Gonna Need It，意指「你不會需要它」）」、「KISS（Keep It Simple, Stupid，意指「保持簡單」）」等。
- 若對話中提到的專有名詞或任何較深的英文技術名詞和縮寫，請附上中文解釋或註解。

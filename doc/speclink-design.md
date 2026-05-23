# SpecLink 設計文件

日期：2026-05-20
版本：重設計（撤回 2026-05-19 初版，捨棄原 22-change roadmap）

## Status

本文是 SpecLink 重設計後唯一的 vision doc，整合 2026-05-20 設計討論的所有結論。

---

## 1. 定位

**SpecLink 是 Spec-Driven Development (SDD) workflow engine + bridge**：

- AI Agent 透過 5 個 skill 觸發 SDD 流程；agent 走 **Bash binding**（spawn CLI subprocess）或 **Tool binding**（typed tool call）任一條路徑
- `speclink-core` library 持有所有 SDD logic、state machine、artifact DAG、validation、analysis
- 對外兩個 deliverable：
  - **`speclink` CLI binary**（給 Claude Code、Codex、shell scripts 等 Bash binding host 用）
  - **`@speclink/client` npm package**（給 GitHub Copilot SDK、CopilotKit、OpenAI function calling、LangChain 等 Tool binding host 用）
- 兩個 Provider（**LocalProvider** 個人本地 / **HttpProvider** 對接 webapp server）；MVP 只實作 LocalProvider
- 設計開放讓他人建立自己的 frontend（規格縱覽 web app、IDE plugin、自訂 chatbot）
- 學習 spectra-cli 但不追求功能對齊；不對齊 openspec 配置

## 1.1 Walking skeleton slice naming

Walking skeleton 開發以小切片（slice）依序落地 SpecLink MVP；每片獨立可 ship、不引入未完成 slice 的依賴。slice 命名一旦凍住即不再變動，後續 design / spec / task 全以下列字串對照：

| Slice id | Change name | 涵蓋範圍 |
|---|---|---|
| A1 | `add-project-bootstrap` | LocalProvider 骨架 + two-root storage (`.speclink/` + `<git-common-dir>/speclink/`) + `project` 表 + `init/status/link/unlink` + `state.db` v1 migration |
| A2 | `add-change-and-artifact-io` | `change` 表 + `state.db` v2 migration + artifact filesystem I/O + `new change` / `show change` / `delete change` / `list --changes` / `list --specs --change` / `new artifact` / `artifact read` + sha256-based Etag + atomic rename |
| A3 | `add-state-machine-and-apply` | 6-state lifecycle + `state.db` v3 migration（`actor_json` / `all_tasks_done` 欄位 + `state_transition` audit 表）+ `apply start` / `apply pause` / `task list` / `task done` / `task undo` 5 個 CLI op + `artifact.write` 後 DAG evaluator hook + `StateMachineStore` trait + `change.version` CAS + walking-skeleton 4-state mode（hard-coded `require_*_review=false`） |
| A4 | `add-archive` | `state.db` v4 migration（`archived_at` 欄位）+ `archive.run` op + `ArchiveStore` trait + spec delta dumb merge + `.speclink/changes/archive/<YYYY-MM-DD>-<id>/` rename + `in_progress + all_tasks_done=1 → archived` transition + 2 新 error code（`change.tasks_incomplete` / `validation.archive_failed` 後者保留）+ `archive.specs_skipped` warning carrier |
| A5 | `add-config-rw` | `state.db` v5 migration（`config_state` singleton + `config_change` audit）+ `config.read` / `config.write` 兩個 op + `ConfigStore` trait + A3 review-flag hardcode 改讀 config + walking-skeleton fallback semantics for malformed config + external-edit detection |

A1–A4 已 archive；A5 為本文件當前主題。後續 slice（review / locking / schema / ingest 等）皆另行命名，不溯及修改本表。

## 2. 設計原則

1. **個人專用 + 團隊可擴展** — 命名、lifecycle、skill 數從 SDD 本質設計；個人 RD 走 LocalProvider，團隊走 HttpProvider 對接自家 webapp
2. **Local-first MVP** — 本地檔案 + SQLite。HttpProvider impl + SDK 整合 + 跨機器同步延後到 LocalProvider 完成
3. **Library + thin clients** — `speclink-core` library 持有所有 SDD 邏輯；CLI / npm package 都是 thin client；未來其他語言 client / web 後端 binding 都可加入
4. **Provider trait 抽象保留** — MVP 只實作 `provider-local`，但 trait 設計到位，未來 HttpProvider 不必重構
5. **Engine 內建分析能力 + AI 加工 skill 層** — analyze / validate / drift 是 CLI/SDK 指令（engine 程式檢查）；analyze / verify / drift 同時也有對應的 AI overlay skill（語意檢查 + 互動 layer）。MVP 內 8 skill：discuss / propose / apply / ingest / archive / analyze / verify / drift
6. **Role 概念只用於 discuss** — 不橫跨整個 engine；不影響 artifact ownership、不影響 permission
7. **單檔 config** — `.speclink/config.yaml` 一檔到底；不引入 openspec 相容包袱
8. **同機多 agent 並發安全** — SQLite WAL + advisory locks（LocalProvider）；HTTP `If-Match` optimistic locking（HttpProvider）
9. **Operation catalogue 為 SOT** — CLI subcommand、describe-tools output、SDK typed methods、MCP tools、skill bindings 全部從 `doc/protocol/operations.md` 衍生
10. **Skill source = workflow + bindings** — host-agnostic workflow 邏輯 + host-specific invocation 指引（bash / tool）；deploy 時拼合
11. **Malformed config = warning + fallback** — 不靜默吃錯
12. **TDD 開發流程** — 所有實作嚴格紅燈 → 綠燈 → 重構

## 3. 角色與責任

| 角色 | 責任 |
|---|---|
| 人類 | 安裝 CLI / npm package、`speclink init` 或 SDK 設定、編輯 config、決定何時 review approve |
| AI Agent | 依 skill 指示觸發 operation（Bash binding spawn CLI / Tool binding call tool）、產生 artifact、修正分析發現的問題 |
| Skill | AI 的 workflow 腳本；source = `workflow.md` + `bindings/{bash,tool}.md`；deploy 時拼合 |
| CLI | thin Bash-binding frontend，只負責 clap parse、JSON serialize、stdin/stdout |
| `@speclink/client` (npm) | thin Tool-binding frontend；內部 spawn CLI subprocess；對外暴露 typed methods + tool helpers + skill installer |
| Core library | `speclink-core` — SDD engine、artifact DAG、schema、role、lifecycle、provider trait、operation dispatcher |
| Provider (local) | `provider-local` — filesystem + SQLite 實作；唯一 MVP provider 實作 |
| Provider (http) | `provider-http` — HTTP-based provider；trait skeleton 在 §19、impl 延後 |

### 3.1 Agent Host

**Agent Host = 任何能讓 LLM agent 載入 skill prompt + spawn shell subprocess 的執行環境**。SpecLink 的對外契約是 `skill + CLI`，所以任何符合契約的 host 都自動接得上，host 之間是對稱的。

| Agent Host | 跑哪裡 | 主要使用者 | 怎麼接 SpecLink |
|---|---|---|---|
| Claude Code | RD 本機 | RD | 讀 `.claude/skills/speclink-*/`、spawn `speclink` CLI |
| Codex | RD 本機 | RD | 讀 `.agents/skills/speclink-*/`、spawn `speclink` CLI |
| GitHub Copilot | RD 本機 IDE | RD | 讀 `.github/prompts/speclink-*.prompt.md` + skill |
| CopilotKit-based agent | 伺服器（Next.js 等） | PM/SA/QA | 安裝 skill 到 SDK 認識的 path、伺服器上裝 `speclink` CLI、agent 自然 spawn |
| 其他 LLM SDK | 任何環境 | 任何 | 同上 — 只要 host 能 load prompt + run shell |

**設計含義**：
- **不為任何 host 寫專屬包裝**（例如不做 CopilotKit-specific tool function）。Tool 包裝會把 SDD 邏輯洩漏到 host 端、變成 host 綁定。
- **Skill body 一份到底**。同一個 skill 給所有 host 用；§20.1 部署機制只負責「複製到對的 path、做語法替換」，不分 host 改 skill 內容。
- **CLI subprocess 是唯一邏輯入口**。Server-side host（CopilotKit）也透過 spawn `speclink` 跑 SDD，不直接寫 backend DB。寫 DB 的事透過 CLI → core → provider 完成；backend 只 read DB 顯示 UI。
- **PM/SA/QA 流程 = RD 流程**。前者透過 CopilotKit chat agent、後者透過 coding agent，最終都走「agent → skill → CLI → core → provider」。

### 3.2 Agent Host × Provider 正交性

Agent Host（誰在跑 agent）跟 Provider（資料存哪）是 **兩個獨立維度**。任何 host × 任何 provider 自由組合：

```
                  LocalProvider             HttpProvider              BYO / None
                  ──────────────────────    ──────────────────────    ──────────────────────
RD coding agent   個人 RD + git 走動         團隊 RD 透過 HTTP          少見：RD 端走 SDK
(在 RD 本機)      (spectra-style)            連到自家或公開              Tier 1 helpers-only
                                             SpecLink service           
CopilotKit-       少見但合法                 主流：網頁工具 backend     主流：webapp 已有自家
server agent      (server 上跑個人模式)      連到 SpecLink service       spec DB、走 Tier 1/2

其他 LLM SDK      ...                        ...                        ...
host
```

含義：
- 單一 RD 在不同公司／不同團隊裡，可能在三個欄位之間漂移。speclink 不假設「個人 RD = LocalProvider」。
- 「當前這台機器走哪種 Provider」由 `.speclink/link.yaml`（CLI 模式）或 `@speclink/client` constructor（SDK 模式）決定，跟 agent host 是誰無關。
- **Provider 軸不限於 bundled 兩類**：
  - **Tier 2 BYO**（§22.1）— caller impl 自家 `Provider` trait，engine logic 仍是 SpecLink 跑
  - **Tier 1 None**（§22.1）— SDK 走 helpers-only entry，連 `Provider` 概念都沒有；engine + storage 都 caller 自管
- CLI 模式 **只支援 bundled provider**（LocalProvider / HttpProvider）。BYO / None 是 SDK-only 概念。

### 3.3 Delivery surface

Agent Host × Provider 之外、再加一個正交維度 — **Delivery surface**：agent 怎麼觸發 SpecLink operations。兩類：

| Delivery surface | 機制 | Host 範例 | SpecLink 端工具 |
|---|---|---|---|
| **Bash binding** | host 用 Bash tool spawn `speclink` CLI subprocess、讀 stdout JSON | Claude Code、Codex、shell scripts、CI runners | `speclink` CLI binary |
| **Tool binding** | host 用 typed function call 觸發、SpecLink-supplied handler 內呼 client SDK | GitHub Copilot SDK、CopilotKit、OpenAI function calling、LangChain tools | `@speclink/client` npm package（其他語言 client 未來加） |

底層 logic 都過 `speclink-core`；上層 invocation 機制不同。Skill body 在兩個 binding 下 **workflow 邏輯一樣、invoke verb 不同**（見 §4.3）。

| 維度 | 取值 |
|---|---|
| **Agent Host** | Claude Code / Codex / GitHub Copilot SDK / CopilotKit / 其他 LLM SDK |
| **Provider** | LocalProvider / HttpProvider |
| **Delivery surface** | Bash binding / Tool binding |

三維正交。任意組合都合法。

## 4. Skill 設計

### 4.1 五個 skill

| Skill | 角色感知 | 觸發場景 | 修改 state |
|---|---|---|---|
| `discuss` | **role-aware**（唯一） | 任何階段需要結構化討論 | 產生／更新 discussion.md；**不改 change state** |
| `propose` | role-neutral | 開始新需求 | 建立 change、寫 proposal/spec/design/tasks |
| `apply` | role-neutral | 開始實作 | `task done`；可能觸發 drift 警告 |
| `ingest` | role-neutral | apply 中需求變動 | 重新讀 artifact、可能回頭改 spec/tasks |
| `archive` | role-neutral | 實作完 + review pass | 套用 spec delta、收檔；最後 sub-flow 詢問是否 commit |

### 4.2 不存在的 skill（明確列出）

| Skill | 為什麼不做 |
|---|---|
| `commit` | 變 archive 的 sub-flow（`speclink instructions commit --change X` 回 commit 結構） |
| `ask` | 純對話、不需 CLI 包裝；AI 自己讀檔案 |
| `audit` / `verify` | 與 engine 內建 `validate` / `analyze` 重複 |
| `debug` | 改 CLI 一條 `speclink doctor`，非 skill |
| `drift` | drift 是 CLI 指令，engine 在 `task done` 後自動跑、AI 也可手動跑；不獨立 skill |
| `analyze` / `validate` | 同上，CLI 指令而非 skill |

### 4.3 Skill source 結構（workflow + bindings）

每個 skill 在 source 內**不是單一 SKILL.md**，而是 **workflow + bindings** 拆開：

```
crates/speclink-core/embedded/skills/
├── propose/
│   ├── workflow.md          # ★ host-agnostic SDD workflow 邏輯
│   │                        #   - 概念順序（先 proposal 再 spec 再 tasks 等）
│   │                        #   - AI 該想什麼、convergence 規則
│   │                        #   - Capture / Guardrails 等共同段落
│   │   
│   ├── bindings/
│   │   ├── bash.md          # ▶ Bash binding：用 `speclink` CLI subprocess 觸發
│   │   │                    #   "Step 1: run `speclink instructions proposal --change <name> --json`"
│   │   │   
│   │   └── tool.md          # ▶ Tool binding：用 typed tool call 觸發
│   │                        #   "Step 1: call tool `instructions_proposal({ changeName })`"
│   │   
│   └── frontmatter.yaml     # 通用 metadata（name, description, version）
├── apply/  (同結構)
├── archive/  (同結構)
├── ingest/  (同結構)
├── discuss/  (同結構)
├── analyze/  (同結構 — AI overlay on `analyze.run` op)
├── verify/   (同結構 — AI-only QA review；無對應 CLI op)
└── drift/    (同結構 — AI overlay on `drift.run` op)
```

**SDD workflow skills**（5 個 — 主流程）：discuss / propose / apply / ingest / archive
**AI overlay / QA skills**（3 個）：
- `analyze`：補 `analyze.run` 結構分析之上的語意層（design vs spec 衝突、scope drift、risk gap）；propose 結束時 **Passive Trigger Contract** 建議自動跑
- `verify`：純 AI 的 QA review（Completeness / Correctness / Coherence 三維度，跨 codebase grep + spec / tasks 追溯）；reviewer 在 code_reviewing state 之前可選用、降低 review approve/reject 主觀性
- `drift`：補 `drift.run` 之上的「conclusion-first 報告 + 互動式選下一步」layer；apply Step 3d dormancy 條件成立時 inline 直接呼 `drift.run`（不 invoke skill），user 主動呼用 skill

**Deploy 時組裝**：

- CLI `speclink init --tools claude` → workflow + `bindings/bash.md` 拼成 `.claude/skills/speclink-<skill>/SKILL.md`
- CLI `speclink init --tools codex` → workflow + `bindings/bash.md` 拼成 `.agents/skills/speclink-<skill>/SKILL.md`
- SDK `speclink.installSkills(dir, { host: 'copilot-sdk' })` → workflow + `bindings/tool.md` 拼成 `<dir>/speclink-<skill>/SKILL.md`

**Legacy 單檔 drafts**：2026-05-21 拆分前的 single-file skill drafts 位於 `doc/skill-drafts/legacy/`（含 README 說明）。**僅供歷史對照，deploy pipeline 不讀**。詳見該目錄內 README.md。

Workflow 邏輯永遠對齊；bindings 在 deploy 時分流。Source 內的 operation 引用統一用 canonical operation ID（如 `change.create`，見 §21）；bindings 文件負責把 canonical ID 翻譯成具體 verb。

### 4.4 Skill 跑的 operation 序列

每個 skill 對應到一連串 SpecLink operations（canonical ID 表示）：

```
propose skill:
  change.create
  instructions.proposal
  artifact.write (kind=proposal)
  status.change
  instructions.spec
  artifact.write (kind=spec)
  ... (repeat for design, tasks)
  
apply skill:
  apply.start
  instructions.apply
  ... AI 改 code ...
  task.done
  ... (loop)
  
archive skill:
  archive.run
  instructions.commit    ← 若使用者要 commit
  ... AI 組 commit message + git add + git commit
  
ingest skill:
  instructions.ingest
  drift.check
  analyze.run
  ... AI 修 artifact ...
  artifact.write (force)
  
discuss skill:
  discuss.new
  instructions.discuss
  ... AI 帶出討論
  discuss.patch (section)
  ... (loop, append rounds)
  discuss.conclude
```

每個 operation 的 bash binding（CLI 子指令）跟 tool binding（tool 名稱 + JSON schema）見 §21 Operation Catalogue。

## 5. Engine 內建 CLI 指令

這些是 CLI 指令、不是 skill。三條觸發路徑：

1. **Engine 自動**：state transition 觸發、artifact 寫入後觸發、task done 後觸發
2. **Skill 內 AI 主動**：AI 自檢、修正前驗證
3. **人類除錯**：直接從 terminal 呼叫

| 指令 | 用途 | 自動觸發時機 |
|---|---|---|
| `speclink analyze <change>` | 跨 artifact 一致性檢查（Coverage / Consistency / Ambiguity / Gaps） | `proposing → reviewing` 前；ingest 時 |
| `speclink validate <change>` | artifact 結構與 delta spec 解析驗證 | 每次 `new artifact` 之後；archive 前 strict |
| `speclink drift <change>` | spec ↔ code 漂移 | `task done` 之後；ingest 時 |
| `speclink doctor` | 一次跑全部診斷（status / analyze / validate / drift / 環境檢查） | **不自動**，人類除錯用 |

## 6. Lifecycle States

### 6.1 Discussion (2 states)

```
iterating ───► converged (terminal)
   (反覆 round)    (有 Conclusion + linked_changes)
```

進入 `converged`：使用者 / AI 顯式呼叫 `speclink discuss conclude --topic <id>`。

**`converged` 是 terminal 狀態，不會自動刪除**：

- Discussion 是 SDD 的「為什麼這樣決定」推理軌跡，與 change 為「要做什麼」並列；對未來 ingest / audit 有獨立價值
- 一個 discussion 可 spawn 多個 change（一對多），所以「change 用完了 → 刪 discussion」是錯誤推論
- Engine **永遠不會自動刪除 discussion**（任何 skill / archive / 時間到期都不會）
- 使用者顯式 `speclink discuss delete <id> [--force]` 才會真的刪除
- `speclink discuss list` 預設**只顯示 iterating**；converged 需用 `--status converged` / `--all` 才顯示

**Engine 在 propose 用 discussion 當 source 時**自動把新建立的 change 加進 discussion frontmatter 的 `linked_changes[]`（bidirectional 連結），但 discussion 內容不動、狀態不變。

### 6.2 Change (6 states)

```
proposing ──► reviewing ──► ready ⇌ in_progress ──► code_reviewing ──► archived
                              ▲          │              ▲     │
                              │          │              │     │
                          apply pause  apply start     auto  review reject
                          (顯式)       (skill auto)   (last  (engine 退回
                                                      task   + 自動加 feedback
                                                      done)  task to tasks.md)
   ingest 可從後續任何 state 退回 proposing 或 ready（依需求變動範圍）
```

**設計核心原則：使用者只該顯式做兩件事** — (1) 跑 skill（skill 內部呼 CLI），(2) reviewer approve / reject（人類判斷）。其他 transition engine 全自動，避免「工程師忘了手動 trigger」造成流程卡住。

| State | 進入條件 | 退出條件 | 退出觸發者 |
|---|---|---|---|
| `proposing` | `new change` 建立 | proposal + spec + tasks 寫完 + engine auto validate pass | engine auto |
| `reviewing` | proposing 完成 | reviewer 顯式 approve | `speclink review approve --change X --reviewer <id> --phase artifact` |
| `ready` | reviewing approve OR `apply pause` | 顯式 `apply start`（skill auto 呼叫） | `speclink apply start <change>` |
| `in_progress` | 顯式 `apply start` OR `review reject --phase code`（退回） | 所有 task `[x]`（engine auto） OR 顯式 `apply pause` | engine auto（all done → code_reviewing）／`apply pause`（顯式退回 ready） |
| `code_reviewing` | engine auto（在 `require_code_review: true` 且所有 task `[x]` 時）；或 re-entry（rejection 後 task 又全 done） | reviewer approve（→ archive 可走） OR reviewer reject（→ in_progress） | `review approve --phase code` OR `review reject --phase code` |
| `archived` | code review approve（若 required）+ `archive` 指令 | （終態） | `/speclink-archive` |

> **Slice 標註**：A3=`add-state-machine-and-apply` 落實本表 6-state transition table + walking-skeleton 4-state mode（review flags 硬編 false，走 `proposing → ready → in_progress` 主路徑）。Review / archive transition 在後續 slice 接通。

**Apply start / pause 語意（雙向 idempotent + ensure-actor）**：

`apply start` 採「ensure actor 進入 apply 階段」語意（不只是 transition）；`apply pause` 對稱：

| 當前 state | `apply start` 行為 | `apply pause` 行為 |
|---|---|---|
| `proposing` / `reviewing` | ✗ `state.transition_invalid`（沒 approve 不能 apply） | ✗ `state.transition_invalid` |
| `ready` | → `in_progress` + assign actor、回 `{ state: "in_progress" }` | **no-op idempotent**、回 `{ state: "ready" }` |
| `in_progress` | **no-op idempotent** + assign actor、回 `{ state: "in_progress" }` | → `ready` + 清 actor、回 `{ state: "ready" }` |
| `code_reviewing` | **不轉移**、回 `{ state: "code_reviewing", message: "Already in code review; nothing to apply." }` | ✗ `state.transition_invalid` |
| `archived` | **不轉移**、回 `{ state: "archived", message: "Change is archived." }` | ✗ `state.transition_invalid` |

**設計重點**：

1. **雙向 idempotency** — start 在 in_progress / pause 在 ready 都是 no-op；對 race / retry / concurrent agent 友善
2. **Ensure-actor 而非 transition-only** — start 永遠 assign actor（除非 state 阻擋）；skill 不必先 status 後再決定，直接呼 start 看回應即可
3. **Apply start 對 post-ready state 回 success + state 描述**（不 error）— skill 可依 response.state 判斷：in_progress → 正常 apply 流程；code_reviewing/archived → 跳過 apply、提示 user 已過此階段
4. **Apply pause 仍嚴格** — 只用於 in_progress → ready；其他狀態 error（使用者顯式呼、不會誤打）

`task done` 不觸發 `ready → in_progress`（那是 apply start 的責任）；只負責 mark checkbox + 記 touched-file。**完成最後一個 task** 時 engine **auto-trigger**（含 feedback task 驗證，見下方）：

  - `require_code_review: true` → `in_progress → code_reviewing`
  - `require_code_review: false` → 仍 `in_progress`，但設 `all_tasks_done: true` flag，`/speclink-archive` 此時可進

**Code review reject 的 re-entry 機制（含 synthetic task 防呆）**：

當 reviewer 跑 `speclink review reject --change X --reviewer Y --phase code --reason "<feedback>"`：

1. Engine 將 state 從 `code_reviewing` 退回 `in_progress`
2. Engine 清掉 code review approval 紀錄
3. Engine 產生 `feedback_id`（如 `fb-2026-05-21-001`）並紀錄到 state.db `feedback_tasks` 表：
   ```
   feedback_tasks (
     change_id      TEXT,
     feedback_id    TEXT PRIMARY KEY,
     reviewer       TEXT,
     phase          TEXT,        -- 'artifact' or 'code'
     reason         TEXT,
     created_at     TIMESTAMP,
     status         TEXT         -- 'pending' or 'done'
   )
   ```
4. Engine **自動**在 `.speclink/changes/<name>/tasks.md` 追加 synthetic task，**帶 HTML comment marker**：
   ```markdown
   - [ ] <!-- speclink:feedback id=fb-2026-05-21-001 reviewer=alice phase=code -->[Review feedback by alice, 2026-05-21] auth check missing in middleware
   ```
5. Engineer 走正常 `/speclink-apply` 流程處理（與一般 task 沒差，AI 看不到差別）
6. **完成所有 task 後（含 feedback task），engine 在 auto-transition 前驗證**：
   - 對每個 `feedback_tasks.status = 'pending'` 的紀錄，搜 tasks.md 內是否仍存在對應 marker
   - **缺少** → `error: tasks.feedback_task_removed`，state **不變**；engine 自動把該 feedback task **re-append** 到 tasks.md（帶警告訊息）
   - 全部 marker 存在且 `[x]` → 標 `feedback_tasks.status = 'done'`，**auto re-entry to `code_reviewing`**
7. Reviewer 再 review；reviewer 第二次 reject 用語意接近的 reason → engine warning「相同 feedback 在 X 分鐘前 reject 過，確認 engineer 真的修正了嗎？」（不阻止）

**整個 reject ↔ fix ↔ re-review 循環全靠 `task done` 一個動作驅動**。`review reject --reason` 是 **required**（empty reason 報錯）。

**Synthetic task marker 規格**：

- 格式：`<!-- speclink:feedback id=<id> reviewer=<name> phase=<artifact|code> -->`
- 緊跟在 markdown checkbox `- [ ]` 後、reason 文字前
- Engine parse tasks.md 時 regex 抓 marker；任何「marker 不見了」都算 user error
- `reason` 在 task 內顯示為純文字、長度上限 200 char（超過截斷顯示，全文存 state.db）
- 多行 reason → 第一行進 task、`\n` 之後存 state.db `reason` 欄位

**`feedback_tasks` 表 unique constraint**：

```sql
CREATE TABLE feedback_tasks (
  change_id      TEXT,
  feedback_id    TEXT PRIMARY KEY,
  reviewer       TEXT,
  phase          TEXT,
  reason         TEXT,
  reason_hash    TEXT,                                          -- sha256(normalized reason) for dedup
  created_at     TIMESTAMP,
  status         TEXT
);
CREATE UNIQUE INDEX idx_feedback_tasks_pending_dedup
  ON feedback_tasks (change_id, reason_hash)
  WHERE status = 'pending';
```

**Partial unique index** 防同 reviewer 反覆 reject 同樣理由 → 第二次 insert 違反 constraint → engine 偵測「重複 reject 相同 reason」、merge 到既有 row（更新 `created_at`）+ 不重複 append marker。

**Auto-transition 前驗證**（補完 step 6）：

1. **必須跟 state transition 在同 SQLite transaction 內**（LocalProvider）或 server-side transaction（HttpProvider）— 防 crash mid-write
2. **Audit event 寫入也同 transaction**（`change.state_changed` 跟 state mutation 一起 commit，見 §17.6 寫入時機保證）
3. 缺漏 marker 時 doctor finding `doctor.state.feedback_task_orphan` 觸發 + auto-fix 規則：**孤兒 row** → engine re-append marker；**孤兒 marker（row 不存在）** → 移除 marker（engineer 不該手動加）

#### 6.2.1 Ingest 退回 state 的 cleanup spec

`ingest` 可從後續 state 退回 `proposing` / `ready`，但必須做 cascade cleanup：

| 來源 state | 目標 state | 必須清理 |
|---|---|---|
| `in_progress` | `proposing` / `ready` | (1) 所有 `feedback_tasks` rows 標 `status='cleared'` (2) `actor` 欄位清空 (3) tasks.md 改動保留但 unmark `[x]`（engineer 可選擇重做）|
| `code_reviewing` | `proposing` / `ready` | 同上 + (4) `review history` 記號標「pre-ingest, may need re-review」 |
| `code_reviewing` | `in_progress` | (1) `feedback_tasks` 不動 (2) `actor` 不動 — 純 state 退一格 |
| `archived` | 任何 | ✗ **拒絕** — archived terminal、需開新 change |

`ingest` skill 完成 cleanup 前不釋放 lock；失敗 rollback；audit event `change.state_changed` 紀錄 `from_state` / `to_state` / `ingest_reason`。

### 6.3 Review optionality

```yaml
# .speclink/config.yaml
require_artifact_review: true   # false → proposing 完直接 ready
require_code_review: true       # false → tasks 全完直接可 archive
```

個人專案 / solo dev 可全關，跑成 `proposing → ready → in_progress → archived` 4 state。

### 6.4 Backlog / icebox 為什麼不用 parked flag 處理

評估後**不**引入 parked metadata flag（無 `change.park` / `change.unpark` / `discuss.park` / `discuss.unpark` ops、無 `parked` column）。理由：

- **SDD lifecycle 已天然提供兩層 backlog buffer**：
  - `proposing` state = idea backlog（spec drafts，可長期擱置、多份共存、engineer 無法 apply）
  - `ready` state = to-do queue（approved specs，可多份共存、engineer 從中挑一個 apply）
- **真實 kanban 工具（Jira / Linear / GitHub Projects）靠 priority + ordering 處理 backlog**，不靠 parked flag
- **`apply pause`（in_progress → ready）已覆蓋「做到一半暫停」場景**，與 backlog 概念正交
- **徹底放棄的 change**：用 `change.delete`（destructive、要 force confirm）；若需要「放棄但留歷史」未來可加 `cancelled` state（archived 的 sibling），不是 parked flag

**Future enhancements（記錄但不入 MVP）**：
- 若 ready 排隊出現 priority 區分需求 → 加 `priority: low / medium / high` field
- 若需要「放棄但留歷史」 → 加 `cancelled` state

## 7. Artifact DAG（spec-driven schema）

```
        ┌────────────┐
        │  proposal  │  ← 為什麼做、做什麼
        └─────┬──────┘
              │
       ┌──────┴──────┐
       ▼             ▼
  ┌────────┐    ┌────────────────┐
  │ design │    │ specs/<cap>/   │  ← 一 capability 一份 delta spec
  │(optional)│  │   spec.md      │
  └────┬───┘    └────────┬───────┘
       │                 │
       └────────┬────────┘
                ▼
          ┌─────────┐
          │  tasks  │  ← 引用 design 決策與 spec requirement
          └─────────┘
```

| Artifact | 依賴 | 必填 | 多份 |
|---|---|---|---|
| `proposal` | （無） | ✅ | 1 |
| `spec` | proposal | ✅ | N（一 capability 一份） |
| `design` | proposal | optional | 1 |
| `tasks` | proposal + spec（+ design if exists） | ✅ | 1 |

## 8. Discussion Entity

### 8.1 獨立於 Change

Discussion 是獨立 entity，不一定變 change。三種使用情境：

```
情境 A：純探索（一對零）
  discuss → 結論「先不做」→ 結束，不建 change

情境 B：前向（一對一或一對多）
  discuss → 結論 capture to: change → propose 開該 change

情境 C：後向（多對一）
  apply 中發現問題 → discuss --about <change> → 結論 ingest 或開新 change
```

### 8.2 檔案格式

```markdown
---
id: order-export-discussion
topic: 訂單匯出流程怎麼設計
status: iterating              # iterating | converged
created: 2026-05-20
updated: 2026-05-22
participants: [pm, sa]
linked_changes:
  - id: add-order-export
    state_at_link: in_progress
    relevant_artifacts: [tasks, design]
linked_specs: []
---

## Background
（背景，不隨 round 變動，會被修訂）

## Open Questions
（活清單 — 每 round 可加可消）
- [ ] 匯出格式：CSV / Excel / JSON?
- [x] 觸發時機 → on-demand

## Decisions Made
（累積，append-only）
- ✅ [2026-05-20 pm] 主要客戶要 CSV
- ✅ [2026-05-21 sa] 走 background job + S3

## Rounds
（append-only）

### Round 1 — 2026-05-20 — pm
...

### Round 2 — 2026-05-21 — sa
...

## Conclusion
（converged 時填）
**Decision**: ...
**Rationale**: ...
**Capture to**: change `add-order-export`、spec `order-export`
```

### 8.3 檔案位置

```
.speclink/discussions/<id>/discussion.md
```

目錄是預留以後可能加 transcripts / attachments。

### 8.4 雙向連結

```bash
# 純探索
speclink discuss new explore-payments

# 對既有 change 延伸
speclink discuss new fix-export-retry --about add-order-export

# 連結多個 change
speclink discuss new redesign-export --about add-order-export --about add-export-formats
```

Engine 在 `instructions discuss --topic X --role <r>` 回傳時自動帶 `linked_changes` 的 snapshot 給 AI 當 context。

### 8.5 寫入機制（Section patch）

Discussion 文件採 **section-level patch**：discuss skill 不一次重寫整份 doc，而是針對 5 個 section 各自呼叫對應 endpoint。Engine 強制部分 section 為 append-only，保護討論歷史不被覆蓋。

| Section | API 行為 | Engine 規則 |
|---|---|---|
| `background` | replace（覆寫） | 每次完整重寫，允許修訂 |
| `open_questions` | replace | 每次完整列出 open/closed 狀態 |
| `decisions_made` | **append-only** | Engine 拒絕修改既有條目，只能加新 |
| `rounds` | **append-only** | 每 round 加一節；無法改既有 round 內容 |
| `conclusion` | replace（僅 converged 時呼叫） | 收斂後填一次 |

**寫入時機**：由 AI 在 discuss skill 內主動觸發，engine 不自動寫。Skill 應指示 AI：

```
- 每 round 結束時呼叫 patch --section rounds --append
- 做出新決策時呼叫 patch --section decisions_made --append
- Open question 變動時呼叫 patch --section open_questions（replace 整 section）
- Background 需修訂時呼叫 patch --section background
- 收斂時呼叫 conclude（內含 conclusion patch）
```

**Invariant**：round 結束點之後必須立即 flush 到盤，**不能等到 skill 結束才 flush**，避免 crash 丟失。

## 9. Role 機制

### 9.1 只用於 discuss

```
discuss skill：唯一接受 --role 的 skill
其他 skill：完全 role-neutral，engine 不知道呼叫者是誰
```

```bash
speclink instructions discuss --topic <id> --role pm
speclink instructions discuss --topic <id> --role security
speclink instructions discuss --topic <id>          # 用 default_role
```

### 9.2 內建 4 個

| Role | 預設 focus |
|---|---|
| `pm` | 使用者價值、商業 trade-off、風險 |
| `sa` | 架構、依賴、替代方案 |
| `rd` | 實作可行性、邊界條件、效能 |
| `qa` | Test scenarios、acceptance criteria、edge cases |

Binary 內建 prompt，從程式碼讀。

### 9.3 自訂機制（inline in config.yaml）

```yaml
# .speclink/config.yaml
default_role: engineer       # 沒帶 --role 時 discuss 用這個
                              # 不設 → 沒 --role 就 error

roles:
  # Override built-in pm（同 id 蓋掉 binary 內建）
  pm:
    display_name: 產品經理
    prompt: |
      You are a Product Manager...
  
  # 新增自訂 role
  security:
    extends: sa                # 繼承 sa prompt 為基底
    display_name: Security Engineer
    prompt: |
      Additional focus beyond SA:
      - Threat modeling
      - Secrets handling
```

### 9.4 Precedence

```
1. .speclink/config.yaml#roles.<id>   ← 使用者 inline 定義（最高）
2. binary 內建 pm/sa/rd/qa           ← built-in fallback
3. 不存在 → engine 回 error code `role.unknown`
```

### 9.5 `extends` 機制

`extends: <existing-role-id>` 讓使用者繼承既有 prompt 為基底，body 內容自動 merge：parent prompt + 子 role 的 prompt 拼接。避免「每次新加 role 都要整套重寫」。

## 10. Schema 抽象

### 10.1 保留 + 簡化

| Layer | 範圍 |
|---|---|
| 內建 | binary 內建 `spec-driven` schema（4 artifact：proposal/spec/design/tasks） |
| 自訂 | 使用者可 `speclink schema fork spec-driven my-flow` 產 `.speclink/schemas/my-flow/` |
| Scope | Schema 只定義 artifact DAG，**不定義 lifecycle**（6 state 是 engine 固定） |

### 10.2 Schema 檔案

```yaml
# .speclink/schemas/spec-driven/schema.yaml
id: spec-driven
name: SDD spec-driven workflow
version: 1.0.0
description: Default workflow with proposal, design, spec, tasks

artifacts:
  - id: proposal
    output_path: proposal.md
    required: true
    dependencies: []

  - id: spec
    output_path: specs/<capability>/spec.md
    required: true
    dependencies: [proposal]
    multi: true

  - id: design
    output_path: design.md
    required: false
    dependencies: [proposal]

  - id: tasks
    output_path: tasks.md
    required: true
    dependencies: [proposal, spec]
```

### 10.3 Templates

```
.speclink/schemas/<schema-id>/templates/<artifact-id>.md
```

`templates/proposal.md` 覆寫 binary 內建 template。Built-in `spec-driven` 沒 fork 過時，templates 全部來自 binary。

### 10.4 Schema 設計約束

- `schema id` = 目錄名（不可變、識別用）
- `schema name` = `schema.yaml` 內 `name` 欄位（display label）
- 不允許 schema 覆寫 binary 內建 `spec-driven`（fork 後必須改 id）
- ArtifactKind 從固定 enum 改為動態 string id（由 schema 定義）

## 11. Config 結構

### 11.0 A5 落地對照（`add-config-rw`）

A5 slice 把本節結構落實到 LocalProvider 與 state.db v5。對應實作細節：

- **etag 公式**：`v<version>.<sha256[:12]>`；fallback 走 literal `v0.malformed-fallback`。
  對齊 design decision「Config etag 命名格式對齊 artifact etag」（slice change `add-config-rw/design.md`）。
- **state.db v5 兩表 schema**：
  - `config_state`：singleton row（CHECK id=1）、`content_sha256` / `size_bytes` / `mtime_ns` /
    `version` / `updated_at` / `written_by`。
  - `config_change` audit log：`change_seq` / `changed_at` / `mode` (`set` / `edit` /
    `external_edit`) / `keys_changed` (JSON array) / `etag_before` / `etag_after` /
    `actor_json` / `reason` (`config_write` / `config_external_edit`)。
- **External-edit reconcile 流程**：read path 偵測檔案 sha 與 `config_state.content_sha256`
  不一致 → 開 SQLite tx → UPDATE `config_state`（version+1）+ INSERT `config_change`
  (mode='external_edit') → commit → 回新 etag + envelope warning
  `config.external_edit_detected`。Read path SHALL NOT raise error。
- **Walking-skeleton fallback semantics**：config.yaml 缺失 / YAML 解析失敗 → read path
  回 defaults + etag = `v0.malformed-fallback` + warning `config.malformed_using_defaults`，
  **不** 寫 audit row、**不** raise error。Write path 對 malformed content 仍抛
  `config.malformed` (exit 3)。
- **state.db v5 cache vs YAML SOT 取捨**：config.yaml 為 source of truth、可隨意外部
  編輯；state.db v5 表為 cache（CAS token + audit log），任何外部編輯由 read path 透過
  sha 比對偵測並 reconcile。

完整 op 觀察行為見 `doc/protocol/operations.md` `config.read` / `config.write` 兩節
（兩 op 均標記 `implemented (A5)`）。

### 11.1 兩個檔的分工

speclink 有 **兩個** 設定相關檔案，目的完全不同：

| 檔 | 目的 | 內容 | 物理位置 |
|---|---|---|---|
| `link.yaml` | **連線元資料**（pointer） | provider type、endpoint、project_id、auth ref | 永遠在 `.speclink/link.yaml`（per-working-dir）、被 .gitignore 排除、不隨 git push 走 |
| `config.yaml` | **SDD 規則本身** | tools、roles、rules（proposal/design/task instructions）、schema 選擇、locale 等 | 邏輯上永遠存在；物理位置由 provider 持有（見 §11.5） |

不存在 `.speclink.yaml`（root）、不存在 `.speclink/roles/<id>/role.md` 外部檔、不存在 user-global config（MVP 階段）。

### 11.2 link.yaml schema

```yaml
# .speclink/link.yaml
# per-working-dir（per-clone）連線設定
# 物理上住在 git 專案的 .speclink/ 內、被 .gitignore 排除、不隨 git push 走
# 同一台機器上不同 clone 各自一份；不是 global ~/.config 設定

provider: local              # local | http
project_id: speclink         # provider 內部 project 識別（local 模式下 = config.yaml#project.id）

# HttpProvider default mode 才需要：
# baseUrl: https://speclink.team.internal
# auth: ${SPECLINK_TOKEN}    # env var / OS keychain reference

# HttpProvider custom mode 才需要（連到既有 webapp endpoints）：
# baseUrl: https://my-webapp.example.com/api
# auth: ${SPECLINK_TOKEN}
# customMapping: ./speclink-mapping.yaml  # 或 inline mapping object，見 §19.4
```

**設計約束**：
- 永遠在 `.speclink/link.yaml`，CLI 啟動時必讀
- LocalProvider 模式下可省略整個檔（缺檔 = 預設 local）
- Credential 永遠走 env var，不存明文 → engine 偵測明文密碼回 `error`（非 warning）
- 入 `.gitignore`（見 §14.1）— `link.yaml` 是本機環境設定，不跟 git 走
- **僅 CLI 模式使用**。SDK 模式（`@speclink/client`）的設定來自 constructor / env / fromConfig，見 §22.5

#### 11.2.1 `${VAR}` Interpolation 規範（嚴格）

link.yaml 內 **僅以下 secret 欄位** 接受 `${VAR}` interpolation：

| 欄位 | 允許 interpolation? |
|---|---|
| `auth` | ✓ |
| `auth.token` / `auth.password`（巢狀 secret 欄）| ✓ |
| `baseUrl` | ✗ **完全禁止** — 防 SSRF / token leak |
| `customMapping`（檔名 path）| ✗ |
| `project_id` | ✗ |
| 其他所有欄位 | ✗ |

**語法**：

- **僅支援** `${VAR_NAME}` exact-match：整個欄位 value 必須匹配 `^\${[A-Z_][A-Z0-9_]*}$`
- **不**支援 partial interpolation（`Bearer ${TOKEN}` 拒收 — 用 token 欄位直接拿）
- **不**支援 default value（`${VAR:-fallback}` 拒收）
- **不**支援 `$VAR` 無 braces 形式（拒收）
- env var 不存在 → **hard error** `config.auth.unresolved` + exit code 1（**不**走 empty string）
- env var resolve 結果在 audit / log / error message **永不印出**；只印欄位名稱

**`baseUrl` 禁 interpolation 的理由**：若允許 `baseUrl: https://api/${ENV}/v1` → attacker 改 `ENV` env var 把 SDD requests + auth token 引去任意 host = SSRF + token exfiltration。`baseUrl` 必須是字面 URL。

**OS keychain reference 標 [deferred]**：design doc 早期提及，但 MVP 階段不實作（沒明確語法、AI workflow 無法 prompt 解鎖）。MVP 嚴格只支援 `${ENV_VAR}`。Keychain reference 等未來 `speclink auth login/logout` 子命令一起做。

### 11.3 config.yaml 完整範例

```yaml
# config.yaml — SDD 規則本身（LocalProvider 模式下物理位於 .speclink/config.yaml）

# ── Project identity ──
project:
  id: speclink
  created: 2026-05-20

# ── SDD workflow ──
schema: spec-driven
default_role: engineer
require_artifact_review: true
require_code_review: true

# ── Runtime / engine 行為 ──
locale: tw                       # tw / en / ja / zh / ko ...
tdd: true
parallel_tasks: true
skill_effort:                    # Claude Code skill effort levels
  propose: xhigh
  apply: xhigh
  archive: low
  ingest: xhigh
  discuss: xhigh
tools:
  - claude

# ── Project context（注入 instructions JSON 的 context 欄位）──
context: |
  ## 專案
  SpecLink — Spec-Driven Development workflow engine。
  ...

# ── Per-artifact rules（注入 instructions JSON 的 rules 欄位）──
rules:
  proposal:
    - 必須明確指出影響的 crate
    - 必須包含 Non-Goals 段落
  spec:
    - 使用 WHEN/THEN 格式
    - SHALL / MUST 表達規範性
  design:
    - 每個決策必須列出至少一個替代方案
  tasks:
    - 嚴格遵循 TDD（紅燈 → 綠燈 → 重構）

# ── Roles（discuss 用）──
roles:
  # Built-in 4 個（pm/sa/rd/qa）engine 從 binary 讀，不寫也能用
  # 可在此 override / 擴充
  
  security:
    extends: sa
    display_name: Security Engineer
    prompt: |
      You are a Security Engineer.
      Focus: threat modeling, secrets handling, data flow.

# ── Providers（MVP 為空，遠端階段才填）──
# providers: {}
```

### 11.4 CLI 統一介面（讀寫 config.yaml）

不管底層 provider 是哪種，user 操作 config 的指令一致：

```bash
speclink config show [--key <path>] [--json]
speclink config set <key.path> <value>
speclink config edit                          # 開 $EDITOR、儲存後自動 push 回 provider
speclink config rules set proposal --stdin
speclink config rules clear proposal
speclink config roles add <id> --from <path>
```

CLI 透過 Provider trait 把讀寫導到對的位置（見 §11.5）。對 user 透明 — 本地 RD 跟團隊 RD 用同一組指令。

### 11.5 不同 Provider 下 config.yaml 物理位置

| Provider | config.yaml 物理位置 |
|---|---|
| `LocalProvider` | `.speclink/config.yaml`（檔案，跟 git 走） |
| `HttpProvider` | webapp server 內部（server 自己決定怎麼存）；CLI 或 SDK 透過 HTTP API 讀寫 |

**Provider trait 必須提供 `read_config` / `write_config` method**（見 §19）。

### 11.6 Malformed 行為

```bash
$ speclink instructions proposal --change my-feature --json
```

```json
{
  "data": { "instruction": "...", "locale": "English" },
  "warnings": [
    {
      "code": "config.workflow.locale.invalid",
      "field": "locale",
      "value": "ttww",
      "message": "Invalid locale code 'ttww'. Expected one of: tw, en, ja, zh, ko. Falling back to 'en'."
    }
  ]
}
```

**永遠回 warning。Engine 不靜默吃錯。**

### 11.7 Rules 注入機制

```
AI 在 propose skill 內呼叫：
  speclink instructions proposal --change my-feature --json
                                                ▼
Engine 讀取（透過 provider trait，物理位置由 provider 決定）：
  1. provider.read_config() → schema 欄位 → 從 schema 拿 template
  2. provider.read_config() → context 欄位
  3. provider.read_config() → rules.proposal[]
  4. provider.read_config() → locale 欄位
  5. 從 schema 拿該 artifact kind 的 instruction body + dependency list
                                                ▼
回傳 JSON（完整 schema 見 operations.md §`instructions.get`）：
  {
    "kind": "proposal",
    "schema_id": "spec-driven",
    "instruction": "<schema-specific guidance markdown>",
    "template": "<artifact skeleton>",
    "context": "<from config.context>",
    "rules": [<from config.rules.proposal[]>],
    "dependencies": [],
    "output_path": "proposal.md",
    "locale": "Traditional Chinese (繁體中文)",
    "available_roles": null,
    "linked_changes_context": null
  }
```

**結構化欄位的設計用意**：

- `instruction` + `template` 是 AI 該**填入 / 跟隨**的內容（output 的一部分）
- `context` + `rules` 是 AI 該**遵守**的約束（**不**進 output content）
- `dependencies` 是 AI 該**先讀**的前置 artifact（context 不入 output）
- AI 看 `kind` 判斷是 artifact kind 或 workflow phase kind：workflow phase 的 `template` / `output_path` 為 null

Engine 跟 caller 之間有明確 contract：**5 個 input field + 11 個 output field 是 stable surface**，caller 不需要解析 markdown body 區分這幾類。

### 11.8 i18n Scope（locale 影響範圍）

`config.yaml#locale` 影響範圍**僅限**：

| 範圍 | locale 是否生效 |
|---|---|
| Artifact body（proposal.md / spec.md / tasks.md / design.md 等內容） | ✓ — AI 依 locale 產生 |
| `instructions.*` JSON 內的 `locale` 欄位（給 AI 看的指引） | ✓ |
| Discussion content（discussion.md 內 Rounds / Conclusion 文字） | ✓ |
| `role.display_name`（user 自訂 role）| ✓ — user 自己用 locale 寫 |

**locale 不影響**（永遠英文，AI / 機器可讀）：

- CLI output（stdout / stderr message）
- Error envelope `message` / `hint`
- Doctor finding `message` / `fix_message` / `fix_command`
- Audit event 內容
- JSON schema descriptions
- Built-in role display_name（`Project Manager` / `Systems Analyst` / `Engineer` / `QA Engineer`）
- `speclink describe-tools` 輸出

**理由**：機器可讀內容多語言會破壞 AI / SDK / CI script grep / log aggregation 一致性。Artifact body 是 user-facing 產出物、locale 才有意義。

未來 user-facing CLI output 多語化視為獨立 effort，需另開 change；MVP 嚴格英文。

## 12. 並發模型（同機多 agent）

### 12.1 場景

同一 repo 同時跑：
- Agent A：apply change `add-export`
- Agent B：propose 新 change `fix-auth`
- 人類：discuss 第三個 topic

三者同時呼叫 CLI。Engine 必須保證資料一致。

### 12.2 Lock 設計

#### 12.2.1 Lock 種類

| Lock | 位置 | 用途 | scope |
|---|---|---|---|
| Global lock | `.git/speclink/locks/global.lock` | init / migration | 全 project |
| Per-change lock | `.git/speclink/locks/changes/<id>.lock` | state mutation | 單一 change |

Storage 在 `.git/speclink/` 內（§13.9）→ 跨 worktree 自動共用、跨 branch 自動共用、不被 git 追蹤。

#### 12.2.2 Lock 階層與取得順序（防 deadlock）

明文寫死兩層 lock 的取得順序：

```
Level 1: .git/speclink/locks/global.lock           (global)
Level 2: .git/speclink/locks/changes/<id>.lock     (per-change)
```

**規則**：任何需要兩層 lock 的操作 **必須先 acquire global、後 acquire per-change**；release 反向（per-change 先 release）。

MVP 內目前只有 `init` / migration 需要 global lock，操作類只要 per-change — 沒有「同時要兩層」的場景。但明文寫死防後續加新 operation 時意外反向 acquire 造成 deadlock。

#### 12.2.3 Lock 檔 schema

```yaml
# .git/speclink/locks/global.lock 或 .git/speclink/locks/changes/<id>.lock
pid: 12345
host: <hostname>
acquired_at: 2026-05-21T10:30:00Z
operation: task.done
instance_id: <uuid from .git/speclink/state.db speclink_meta>
```

- `pid` / `host`：擁有者 process 識別
- `acquired_at`：UTC ISO 8601
- `operation`：lock 是哪個 operation 拿的（debug / `doctor` 顯示用）
- `instance_id`：對應 §13.6 LocalProvider 的 instance UUID — 跨機器 traceability

#### 12.2.4 Stale lock 偵測與接管

Acquire lock 時若檔已存在：

| Holder pid | 持有時間 | host 比對 | 行為 |
|---|---|---|---|
| alive | — | — | 正常等 §12.4 timeout |
| dead | < 5 min | match | 等 timeout（保守 — 防快速 crash-restart 雙寫）|
| dead | ≥ 5 min | match | **強制接管**、log warning + audit event `lock.stale_takeover` |
| dead | — | mismatch | 拒絕接管、error `lock.foreign_host` 提示 user 手動清 |

**5 分鐘 threshold**：hardcode、MVP 不開放配置；若實際運用發現過短/過長再加 `config.yaml#concurrency.stale_lock_after` 欄位。

**Pid alive 偵測**：`kill -0 <pid>` (Unix) / `OpenProcess` (Windows)；判斷不出（permission denied 等）→ **保守視為 alive**。

`speclink doctor` 偵測 stale lock 但 **不自動清**（§16.12.5 already）；接管邏輯只在 acquire 路徑跑。

### 12.3 SQLite WAL mode

State.db 位於 `.git/speclink/state.db`（§13.9）；WAL / shm 同層（`state.db-wal` / `state.db-shm`）。

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
```

讀取免 lock。寫入用 BEGIN IMMEDIATE transaction。

### 12.4 操作 vs lock 對照

| 操作 | 需要的 lock | timeout |
|---|---|---|
| `list` / `show` / 查詢類 | 無（SQLite WAL） | — |
| `new artifact` | per-change exclusive | 5s |
| `task done` | per-change exclusive | 5s |
| `review approve` | per-change exclusive | 5s |
| `archive` | per-change exclusive | 5s |
| `new change` | global short（建目錄） | 1s |
| `init` / migration | global exclusive | 30s |

### 12.5 Conflict 處理

#### 12.5.1 Lock 取不到 → error shape

取不到 lock → 等待 ≤ timeout → 仍取不到：

```json
{
  "ok": false,
  "error": {
    "code": "change.locked",
    "message": "Change 'add-export' is being modified by another process. Try again.",
    "details": {
      "lockHolderPid": 12345,
      "heldSince": "2026-05-20T15:30:00Z",
      "operation": "task.done"
    }
  }
}
```

Exit code `7`。

#### 12.5.2 Jittered backoff 規範（全 skill 共用）

對 `change.locked` / `state.lock_timeout` / HTTP 503 / HTTP 408 等暫態 error：

| 嘗試 | base delay | jitter | 累積最差 |
|---|---|---|---|
| 1 | 0.5s | + random(0, 0.5s) | ~1s |
| 2 | 1s | + random(0, 1s) | ~3s |
| 3 | 2s | + random(0, 2s) | ~7s |
| 4 | — | — | bubble up（exit 7）|

**Hardcode 4 次嘗試（含第一次）**、MVP 不開放 `--retry-max` flag；CI 真有需要再加。Jitter 是 uniform random，避免多 agent lockstep collision。

**Aggregate retry budget**（避免長 skill flow 累積 retry 燒時間）：

- 同一個 skill invocation 內、跨多個 CLI subprocess、累積 retry budget 上限 **20 秒**（透過 env var `SPECLINK_RETRY_BUDGET_MS` 預設 20000）
- Budget 由 skill 起始時設定、每次 CLI invoke 帶 `--retry-budget-ms <remaining>` 傳遞；CLI 內部走 §12.5.2 backoff 但不得 wait 超過剩餘 budget
- Budget 用盡後 → CLI 直接 bubble up `change.locked` / `state.lock_timeout`，**不再 retry**
- 對 `state.etag_mismatch` 的 read-then-retry（§12.5.4）也算進 budget
- AI Bash binding host 可選擇性傳 budget；不傳則 default 20s
- Tool binding host（SDK）由 `@speclink/client` 自動算 + 傳

**永不重試的 error**（重試一定再失敗、需要 caller 重讀 state）：

#### 12.5.3 `state.transition_invalid`（不做盲目 retry）

- 此 error **代表 state 不符預期、不是暫態問題**，retry 無意義
- AI 應立即跑 `speclink status --change X --json` 重讀當前 state
- 依當前 state 決定行動：
  - 若 state 是預期的（如 apply skill 預期 ready/in_progress，看到 `in_progress`）→ 之前的 fail 可能來自過期 state cache，繼續流程
  - 若 state 是後續階段（apply skill 看到 `code_reviewing` / `archived`）→ 提示 user 「此 change 已過 apply 階段」、skill 退出
  - 若 state 是前面階段（apply skill 看到 `proposing` / `reviewing`）→ 提示 user 「此 change 尚未 review approve、無法 apply」、skill 退出
- 不論哪種情況 **都不應該盲目 retry** — 重新讀 state 才能下正確判斷

#### 12.5.4 `state.etag_mismatch`（與 `state.transition_invalid` 同類）

- 此 error **代表 read-modify-write 期間有人改了同一筆**（config / change / artifact / discussion 任一），retry 同樣 payload 一定再失敗
- AI 應立即 **重新 read** 對應實體（如 `speclink config show --json` / `speclink show <change> --json` / `speclink artifact read --json`）拿到新 etag
- 比對 caller 自己手上的 value 是否需要重新合併：
  - **change state transition**（如 `apply start` ready → in_progress）：用 §12.5.3 `state.transition_invalid` 段的判斷邏輯（state 已往前/退後/吻合 caller 預期）
  - **artifact / config / discussion**：read 後重新 apply 自己這次想要的修改，再帶新 etag 寫回；若 caller 是 AI、可把 diff 顯示給 user 確認
- AI 重試規則：**最多一次自動重讀 + 重寫**；第二次仍 mismatch → bubble up 給 user 並顯示「此資源在你操作期間被其他 process 修改、請手動 review」

#### 12.5.5 `tasks.feedback_task_removed`

- engineer 不小心刪了 synthetic feedback task → engine 已 re-append；AI surface 給 user「feedback task 已重建、請補完」，skill 不自動 retry

#### 12.5.6 `lock.foreign_host`（stale lock 屬於他機）

- 偵測到 lock 檔 `host` 不是本機（§12.2.4）→ 視為 user error（如不該共用的 NFS 掛載 / 跨機 sync 工具誤同步 `.speclink/`）
- 不自動接管；提示 user 確認來源後手動刪 lock 檔
- 不重試

### 12.6 HttpProvider 並發對應

HttpProvider 沒 client-side file lock — server 端自管 transaction / lock。Client retry 走同一套 §12.5.2 backoff 規範，依 HTTP status code 對應 SpecLink error code：

| HTTP status | 對應 error code | Retry 策略 |
|---|---|---|
| `400 Bad Request` | 對應 operation 的 input error（如 `change.invalid_name`） | no（input 錯一定再錯）|
| `401 Unauthorized` | `auth.required` | **no**（token 缺失 retry 必再 401）|
| `403 Forbidden` | `auth.invalid` | **no**（token 不對 retry 必再 403）|
| `404 Not Found` | `*.not_found`（依 operation 推斷）| no |
| `408 Request Timeout` | `state.lock_timeout` | §12.5.2 backoff |
| `409 Conflict`（per-change 衝突）| `change.locked` | §12.5.2 backoff |
| `412 Precondition Failed`（If-Match 失敗）| `state.etag_mismatch` | **不重試**（§12.5.4 read-then-retry）|
| `423 Locked`（WebDAV 慣例，可選）| `change.locked` | §12.5.2 backoff |
| `429 Too Many Requests` | `change.locked` 或 `provider.unreachable`（依 server）| §12.5.2 backoff + **尊重 `Retry-After` header**（若帶，覆蓋 backoff curve；上限 30s，超出 bubble up） |
| `500 Internal Server Error` | `provider.unreachable` | **no**（server bug、retry 通常無效）|
| `502 Bad Gateway` | `provider.unreachable` | §12.5.2 backoff（gateway 暫態）|
| `503 Service Unavailable` | `change.locked` 或 `provider.unreachable` | §12.5.2 backoff + 尊重 `Retry-After` |
| `504 Gateway Timeout` | `state.lock_timeout` | §12.5.2 backoff |

**Network-level error 對應**（非 HTTP status）：

| Error 類型 | 對應 error code | Retry 策略 |
|---|---|---|
| Connect timeout / DNS 失敗 | `provider.unreachable` | §12.5.2 backoff |
| Connection reset mid-response | `provider.unreachable` | §12.5.2 backoff |
| TLS handshake 失敗 | `provider.unreachable` + `doctor.security.tls_handshake_failed` finding | no（cert 問題不會自動恢復）|

**Redirect 規則**：HttpProvider client 不 follow HTTP→HTTPS downgrade redirect；HTTPS→HTTPS redirect 限同 host。違反 → `mapping.host_mismatch`。

Server 端 implementation 自定（DB transaction / Redis lock / 自家機制），但回 client 的 status code 必須跟 SpecLink error model 對齊；HttpProvider custom mode 的 mapping template 可宣告 status code 對應（見 §19.4）。

### 12.7 Schema Migration Flow

LocalProvider state.db（`.git/speclink/state.db`，§13.9）schema 升級走 forward-only migration（SQLite + `schema_migrations` table）：

```sql
CREATE TABLE schema_migrations (
  version    INTEGER PRIMARY KEY,
  name       TEXT NOT NULL,
  applied_at TIMESTAMP NOT NULL,
  checksum   TEXT NOT NULL          -- sha256 of migration SQL body
);
```

**Migration acquire 流程**：

1. CLI 啟動讀 `schema_migrations` 對比 binary embedded migrations
2. 若需 apply：**先取 `.git/speclink/locks/global.lock` global lock（timeout 30s）**
3. 取到 lock 後再 check 一次 — 可能其他 process 已 apply 完（double-check）
4. 對 pending migrations 逐一 apply：每個 migration 一個 SQLite transaction，BEGIN → run SQL → INSERT schema_migrations row → COMMIT
5. 任一失敗 → ROLLBACK + 不繼續後續 migration + return `migration.failed` 帶 `version` / `error_detail`
6. 全部成功 → release lock

**Multi-process 互鎖**：global lock 30s timeout 對小 migration 夠；大 migration（>30s）需手動 `speclink doctor --check project` 偵測 + 提示 user 設長 timeout（未來加 `--migration-timeout` flag）。

**Rollback 規範**：MVP 不支援 downgrade — 一旦 apply 就不可逆。Binary downgrade（user 裝舊版）後讀 state.db 看到比自己新的 schema_version → `migration.version_too_new` error + 提示 user 升級 binary。

**Checksum 驗證**：每個 migration apply 後 checksum 寫進 `schema_migrations`；後續 binary 啟動對比 — 不一致 → `migration.checksum_mismatch` 表示 state.db 跟 binary 不匹配（可能 user 手改 state.db）。

新增 error code：
- `migration.failed`（已列 §17.4）
- `migration.version_too_new`
- `migration.checksum_mismatch`

### 12.8 Lock vs Etag — 角色分清

兩個 concurrency primitive 並存、各司其職、**不冗餘**：

| Primitive | 角色 | 何時用 |
|---|---|---|
| **File lock**（per-change / global）| Single-writer mutex within one machine | LocalProvider 把 SQLite 寫 + 檔案寫綁進同一 atomicity 封套；HttpProvider 不用 |
| **Etag**（`Versioned<T>.etag`，§19.2.2）| Cross-writer optimistic concurrency | 所有 read-modify-write 都帶；trait 層 primitive |

```
LocalProvider operation flow:
  acquire per-change lock
    ▼
  read with etag                              ─┐
    ▼                                          │ 在 lock 內、etag check
  write with expected_etag                    ─┘ 必然通過（redundant 但保留）
    ▼
  release lock

HttpProvider operation flow:
  read → server 回 ETag header
    ▼
  write with If-Match (沒 client-side lock)
    ▼
  server 端自己處理 transaction / lock
```

**LocalProvider 看似冗餘但保留 etag 的理由**：

1. **Trait abstraction 統一** — `speclink-core::Engine<R: ProviderRegistry>` 不必 `if-LocalProvider-else-HttpProvider` 分支寫兩套 retry / read-modify-write 邏輯
2. **未來放寬 lock 邊界** — 如果 LocalProvider 為了 perf 把「整個 operation 進 lock」放寬成「只 lock 寫入瞬間」，etag check 馬上接上、不必補實作
3. **Audit / debug** — etag 帶在 audit event 內、跨 provider 一致

## 13. Project Identity 與 Boundary

### 13.1 Boundary

`.speclink/` 目錄存在 = 這是 speclink project。Storage 拆兩個 root，見 §13.9。

CLI 啟動時：

1. 從 CWD 開始往上找 `.speclink/` → 取得 **artifacts root**
2. 找不到 → error `project.not_initialized`，提示 `speclink init <name>`
3. 讀 `.speclink/link.yaml`（若存在）決定 provider；不存在則預設 LocalProvider
4. **若 provider = local**：跑 `git rev-parse --git-common-dir` 取得 git common dir → **state root** = `<common-dir>/speclink/`
    - 失敗（非 git 專案）→ error `project.requires_git`（§13.9.2 / §17.4），提示 `git init` 後重跑
    - 成功 → state root 確定（worktree / submodule 自動 resolve、見 §13.9.1）
5. 走 `ProviderRegistry.open(project_id)`（見 §19.2）拿到單 project 的 `Provider` instance，後續操作對該 instance 跑

跟 `.git/` 同設計：artifacts root 在 working tree、state root 在 git dir 內、各自定位。

**Identity 來源依 provider 不同**：

| Provider | Identity 來源（CLI 模式） | Identity 來源（SDK 模式）|
|---|---|---|
| LocalProvider | `.speclink/config.yaml#project.id`（跟 git 走） | constructor `project` 參數（或 fromConfig/fromEnv） |
| HttpProvider | `.speclink/link.yaml#project_id` + remote server 內部紀錄 | constructor `project` 參數 + provider.baseUrl |

### 13.2 三種 Provider 場景對照（具體流程）

#### 場景 1 — 個人 RD（LocalProvider，= spectra 行為）

```
git repo: billing-system/
└── .speclink/
    ├── config.yaml         ← 規則（跟 git 走）
    ├── state.db            ← SQLite（gitignore）
    └── changes/<id>/...    ← 規格檔（跟 git 走）

操作：
  $ speclink init billing-system
  $ /speclink-propose
  $ git push   # 規格 + code 一起走
```

`project_id` 純 label，沒實際功能（working dir 本身就是 boundary，1:1）。

#### 場景 2 — 規格縱覽 webapp + 工程師 git project（HttpProvider）

```
git repo: billing-system/      git repo: onboarding-flow/    規格縱覽 webapp 伺服器
└── .speclink/                 └── .speclink/                ┌─────────────────────┐
    └── link.yaml                  └── link.yaml             │ Web UI (PM/SA/QA)    │
       provider: http                 provider: http         │ + CopilotKit agent   │
       baseUrl: https://...           baseUrl: https://...   │   + Tool binding     │
       project_id: billing            project_id: onboard    │ + Bash binding via   │
            │                                       │        │   CLI on same machine │
            └────────────────┬──────────────────────┘        │ + speclink-core      │
                             │ HTTP                          │   in webapp backend  │
                             ▼                               │   (或 spawn CLI 子   │
                  ┌──────────────────────────┐               │   process)           │
                  │  webapp HTTP API          │◄──────────►│ + 自家 DB (規格存這)  │
                  │  POST /api/changes        │              └─────────────────────┘
                  │  PATCH /api/changes/<id>  │
                  │  GET /api/changes         │              
                  └──────────────────────────┘
                  
操作（負責人在 webapp 上初次建立 project）：
  PM 透過網頁 chat agent (Tool binding) → 呼 `register_project` → webapp 寫自家 DB

操作（新 RD clone repo 加入既有 project）：
  $ git clone team/billing-system
  $ speclink link https://specs.team.internal --project billing-system
  # → CLI 寫 .speclink/link.yaml；後續 operation 走 HTTP API
```

`project_id` = webapp 端 server 上的 project 識別。HttpProvider 有 **default mode**（webapp 實作 SpecLink 標準 endpoints）與 **custom mode**（webapp 已有自己 endpoints，靠 link.yaml 宣告式 HTTP template 映射，見 §19.4）。

跨組織 / SaaS / 跨網場景**用同一個 HttpProvider**，差別只在 baseUrl 指向哪個 server + 信任域邊界由 server 端 access control 控制。

### 13.3 規格在哪 — 關鍵 Trade-off

| 項目 | LocalProvider | HttpProvider |
|---|---|---|
| `proposal.md` / `spec.md` / `tasks.md` 在哪 | git repo 內 | webapp server 端 storage |
| `git push` 帶走規格？ | ✓ | ✗ workflow 中規格在 server；archive 後**累積版 spec** (`.speclink/specs/<capability>/spec.md`) 進 git 跟 code 一起 PR |
| PM/SA/QA 怎麼參與 | 必須 clone repo + 用 coding agent | **網頁工具透過 chat agent**（CopilotKit / Tool binding），不碰 git |
| 規格版本控制 | git history | server 端 audit log + 累積 spec 進 git 後也有 history |
| 規格搜尋 | `git grep` | webapp UI / API |
| 離線工作 | ✓ | ✗（網路不通做不了 SDD）|

**累積 spec 進 git** 是讓 PR reviewer 看得到「這次 PR 對 spec 的修改」的關鍵 — 工程師 archive 完、`.speclink/specs/<capability>/spec.md` 自動更新到 git tracked 位置、跟 code 一起 commit。

**這是設計上必須面對的取捨**：要 PM/SA/QA 介入 SDD 流程 → 規格必須跑到雙方都能存取的地方（脫離 git）。spectra 沒這個能力。

MVP 只實作 LocalProvider，所以這個 trade-off 不會立刻浮現；Shared/Remote 階段才會。

### 13.4 project_id 分配機制

| Provider | 機制 |
|---|---|
| LocalProvider | user 自取，純 label，撞不撞名都不影響 |
| HttpProvider | user 自取 slug，**server 端 unique constraint**；撞名 → error `project.id_taken`，建議改名（未來可能加 namespace `<org>/<slug>`，MVP 不引入） |

**Slug 規範**：lowercase + hyphen + digits、長度 1–64、開頭必字母、保留字（`init`、`config`、`link` 等）禁用。

### 13.5 Project 生命週期指令

兩個指令分開、職責互斥：

| 指令 | 用途 |
|---|---|
| `speclink init <name>` | **建立新 project**。Local: 建 `.speclink/{config.yaml, state.db}`。Http: 連 provider + register 新 project + 建 `.speclink/link.yaml` |
| `speclink link <provider-url> [--project <id>]` | **連到既有 project**（HttpProvider 用）。純寫 `.speclink/link.yaml`，不動 provider 內資料。沒 `--project` 則互動式列出可選 project |
| `speclink unlink` | 解綁但不刪 provider 端 project — 刪掉 `.speclink/link.yaml` |

SDK 模式（`@speclink/client`）對應 method：`new SpecLink({...})` + `await speclink.init()` / `await speclink.link({...})`（見 §22）。

範例：

```bash
# 個人 RD（MVP）
$ speclink init billing-system

# 團隊負責人在 webapp 上新建（HttpProvider，[deferred]）
$ speclink init billing-system --provider http --baseUrl https://specs.team.internal

# 新 RD clone repo 加入既有 project（HttpProvider，[deferred]）
$ speclink link https://specs.team.internal --project billing-system

# 不再連這個 project（[deferred]，跟 link 一起延後）
$ speclink unlink
```

**MVP 範圍**：只 `init` 純 LocalProvider 一條路；`init --provider http` / `link` / `unlink` 全屬 [deferred]（HttpProvider 一起做）。SDK `@speclink/client.init()` / `.link()` / `.unlink()` 同步 [deferred]，MVP SDK 只支援 `fromWorkspace` / `fromConfig` / `fromEnv` 連到既有 LocalProvider（見 §22.5）。

### 13.6 Working dir 綁定強度

LocalProvider 跟 HttpProvider 在此點 **本質不同**：

| 情境 | LocalProvider 影響 | HttpProvider 影響 |
|---|---|---|
| Working dir 改名 / 搬家（單 RD）| 零影響 | 零影響 |
| 同 RD 換機器（重新 clone） | git clone 拿不到 `.git/speclink/state.db`（在 `.git/` 內、不隨 clone）→ 跑 `speclink restore --from-artifacts`（§16.13）從 artifact 重建；historical audit / instance_id lost | 拿 link.yaml + 連 server 即可；狀態仍在 server |
| **`git checkout <other-branch>`** | state.db 跨 branch 共用（`.git/speclink/` 在 git dir 內、不跟 branch）→ 看得到其他 branch 開的 in_progress changes；但若該 change 的 artifacts 在當前 branch fs 不存在 → op 走 `change.artifact_missing` 一律 reject（§13.9.4 / §17.4） | server 端 state 不變、自然跨 branch 一致 |
| **`git worktree add`**（同機開新 worktree） | `.git/speclink/` 在 GIT_COMMON_DIR、所有 worktree 自動共用 → state.db / lock / touched 跨 worktree 一致；§12 file lock 自然生效（§13.9.4） | 同上、server 端 single SoT |
| 手改 `link.yaml#project_id`（Http 模式）| n/a（local 不用 link.yaml） | CLI 下次啟動連到新 project；不做變更偵測 |
| `.speclink/` 整個刪除（artifacts root） | 失去 git tracked artifacts；git checkout 還原；`.git/speclink/state.db` 不受影響、但 op 多半走 `change.artifact_missing` | 不影響 server 端資料；重新 `speclink link` 即可恢復 |
| `.git/speclink/` 整個刪除（state root） | state.db / lock / touched 全失；下次 init 重建（或跑 `speclink restore --from-artifacts`）；instance_id rotate | 不影響（state 在 server） |
| **多 RD 同時 clone + 各自跑 apply** | ⚠️ **不支援**：每份 `.git/speclink/state.db` 互不知；divergence silent | ✓ server 端 lock + state 一致 |
| **同 RD 同機多 clone**（非 worktree、各自獨立 git dir）| 跟「多 RD」同類；推薦改用 `git worktree` 共用 `.git/`，state.db 自動共用 | ✓ 各 clone 都連同一 server、state 一致 |

**LocalProvider 是 single-RD-single-git-dir 的設計**（同 git dir 下的多 worktree 自動共用 state.db、屬支援場景）。多 RD 協作 / 同 RD 多獨立 clone 場景**請走 HttpProvider 或改用 worktree** — engine 不能也不應該防 multi-clone divergence，那是 user error。

在 HttpProvider 模式下，**provider 才是 source of truth**；working dir 是「我這台機器對應到 provider 哪個 project」的指引、完全 disposable。跟 git 的 `.git/config#remote.origin.url` 同邏輯。

**`instance_id` UUID（給 audit traceability）**：

LocalProvider init 時 engine 在 `.git/speclink/state.db` 內生成一個 UUID `instance_id`：

```sql
CREATE TABLE speclink_meta (
  key   TEXT PRIMARY KEY,
  value TEXT
);
INSERT INTO speclink_meta (key, value) VALUES ('instance_id', '<uuid>');
```

- **永遠不寫進 git 跟 config.yaml**（在 state.db、且 state.db 住在 `.git/speclink/` 內、git 自然不追蹤）
- 每筆 audit event 紀錄含 `actor.instance_id`
- `speclink doctor` 印出當前 instance_id；如果未來 export/import 跨 clone 合併資料，可追溯 event 來源
- **不阻止** multi-clone divergence；只提供「事後溯源」能力

**Rotation 行為**：

| 觸發 | 行為 |
|---|---|
| `speclink init --force` | 生成新 instance_id；audit log 內舊 instance_id **不 rewrite**、變 historical actor reference |
| 重 clone 後第一次 `speclink init` | 同上（新 instance_id；舊 ID 已隨舊 `.git/speclink/` lost） |
| `speclink restore --from-artifacts` | 生成新 instance_id；restore 完寫一筆 `audit.restored` event 紀錄 previous_instance_id（若可從 artifact 推、否則 null） |

`speclink doctor` 顯示當前 instance_id；rotation 不影響 op 正常進行。

### 13.6.1 LocalProvider init 推薦團隊改 HttpProvider 的提示

`speclink init <name>` 預設走 LocalProvider，print 提示：

```
✓ Initialized LocalProvider at /path/.speclink/
  instance_id: 9c5a3f2b-...

Note: LocalProvider stores SDD state locally per checkout. For multi-developer
collaboration with shared spec state, use HttpProvider instead:

  speclink init billing-system --provider http --baseUrl https://specs.team.internal
```

### 13.7 多 working dir 跨 project（同機）

每個 working dir 各有自己的 `.speclink/link.yaml`，各指各的 `project_id` — 這是 feature。每次 CLI invocation 都會獨立 `registry.open(project_id)` 拿 fresh `Provider` instance（CLI 是 short-lived process，不存 instance cache；instance cache 是 SDK / webapp 場景才有的概念，見 §22.6）。

連線資訊重複的 UX 痛點靠環境變數共用：

```bash
# ~/.bashrc / ~/.zshrc 設一次
export SPECLINK_TOKEN=...
export SPECLINK_HTTP_BASE_URL=https://specs.team.internal

# 各 working dir 的 link.yaml 都 reference 同一個
provider: http
baseUrl: ${SPECLINK_HTTP_BASE_URL}
auth: ${SPECLINK_TOKEN}
project_id: <各自的 id>
```

**未來加 alias（明確 deferred）**：`~/.speclink/providers.yaml` 用 user-global alias（如 `team-prod`），各 working dir link.yaml 只寫 `provider_ref: team-prod`。跟 git remote alias 同邏輯。MVP 不做。

### 13.8 MVP 不存在的指令

| 指令 | 狀態 |
|---|---|
| `speclink project bind` | ❌ 延後（被 `init` / `link` 取代）|
| `speclink project status` | ❌ 延後 |
| `speclink auth login/logout/status` | ❌ 延後 |
| `speclink provider add/list/use` | ❌ 延後（user-global alias 機制延後）|

### 13.9 State storage layout

State 拆兩個 root，各司其職：

- **`.speclink/`**（在 working tree）— **shared, git-tracked artifacts**
- **`.git/speclink/`**（在 git dir 內）— **local-only state；git 不追蹤 `.git/` 自己**

| Path | 內容 | git tracked? | 跨 branch? | 跨 worktree? |
|---|---|---|---|---|
| `.speclink/config.yaml` | SDD rules | ✓ tracked | per-branch | per-worktree |
| `.speclink/changes/<id>/...` | artifacts（proposal / spec / tasks / design / metadata）| ✓ tracked | per-branch | per-worktree |
| `.speclink/discussions/<id>/discussion.md` | discussions | ✓ tracked | per-branch | per-worktree |
| `.speclink/schemas/<id>/...` | schema forks | ✓ tracked | per-branch | per-worktree |
| `.speclink/archive/<date>-<id>/...` | archived changes | ✓ tracked | per-branch | per-worktree |
| `.speclink/audit/<change-id>.log` | audit log（user 自己決定 track）| optional | per-branch | per-worktree |
| `.speclink/link.yaml` | provider 連線設定 | gitignored | per-clone | per-clone |
| `.git/speclink/state.db` | SQLite state | implicit excluded（`.git/` 內）| **✓ shared** | **✓ shared** |
| `.git/speclink/state.db-wal` / `-shm` | SQLite WAL / shm | implicit excluded | ✓ shared | ✓ shared |
| `.git/speclink/locks/global.lock` | §12 global advisory lock | implicit excluded | ✓ shared | ✓ shared |
| `.git/speclink/locks/changes/<id>.lock` | §12 per-change lock | implicit excluded | ✓ shared | ✓ shared |
| `.git/speclink/touched/<change>.json` | task done 觸碰檔案紀錄 | implicit excluded | ✓ shared | ✓ shared |

#### 13.9.1 Path resolution

CLI / SDK 解析 storage 路徑時走 git 標準 API、**不自己解析 `.git` file**：

```
let common_dir   = run("git", &["rev-parse", "--git-common-dir"])?;
let state_root   = common_dir.join("speclink");        // .git/speclink/（worktree 自動 resolve 到主 repo）
let artifacts_root = find_speclink_dir_upward()?;      // .speclink/（從 cwd 往上找）
```

`git rev-parse --git-common-dir` 是 git 1.8+ 標準 API，自動處理：

| 場景 | `--git-common-dir` 回傳 | `.git/speclink/` 落點 |
|---|---|---|
| 一般 repo | `.git` | `.git/speclink/` |
| Git worktree（`.git` 是 file）| 主 repo 的 `.git/` | 主 repo 的 `.git/speclink/`（worktree 自然共用）|
| Submodule（superproject 的 `.git/modules/<name>/`）| `.git/modules/<name>/` | `.git/modules/<name>/speclink/` |
| Bare repo | git dir 本身 | n/a（bare repo 不該跑 LocalProvider）|

**設計重點**：worktree / submodule / repo rename 全部交給 git 自己處理；SpecLink 不維護獨立 path metadata、不寫死字串。

#### 13.9.2 Non-git project：require_git

`speclink init` 偵測無 `.git/`（`git rev-parse --git-common-dir` 失敗或 cwd 不在 git working tree）→ 拒絕 init：

```
$ cd /tmp/no-git-project
$ speclink init billing-system
✗ Error: project.requires_git
  SpecLink LocalProvider requires a git repository for state storage.
  Run `git init` first, then re-run `speclink init`.
```

**理由**：
- SpecLink LocalProvider 對 SQLite 的依賴比 spectra cli 強烈很多（feedback_tasks / etag version / instance_id / audit / review history 全在 state.db）；不適合 spectra「部分 op 純檔案、SQLite-依賴 op 默默 fail」的曖昧 policy。
- 個人 RD workflow 99% 已在 git project 內、強制 git 對使用者不痛。
- 比起「init 在純資料夾建 `.speclink/state.db`」，落在 git dir 內可享受 worktree / branch / sync 工具不碰到等所有免費 benefit。

**Future**：`speclink init --no-git` fallback 到 `.speclink/state.db` + gitignore 已標 §18.2 [deferred]；現階段不做。

#### 13.9.3 `.gitignore` 寫入

`speclink init` 自動 append `.gitignore`（idempotent；存在則 skip）：

```
# .gitignore（init 自動 append）
.speclink/link.yaml
```

僅一行。其他 local-only 內容（state.db / locks / touched）已在 `.git/speclink/` 內、git 自然不追蹤，**無需** gitignore 條目。

**選 `.gitignore` 而非 `.git/info/exclude`** 的理由：
- `.gitignore` 跟 commit 走 → team 內其他 RD clone 後自動排除 link.yaml、不會誤 commit 自己的 token。
- `info/exclude` 是 per-clone local、要求每個 RD 各自 setup、易遺漏。
- 學 spectra cli 同 pattern（`spectra init` 也寫 `.gitignore`），user mental model 一致。

#### 13.9.4 P0 議題對應

| 議題 | 解決方式 |
|---|---|
| **Branch switch** | state.db 跨 branch 共用 = feature、**不是** bug。Branch A 開的 in_progress change 切到 branch B 後 state.db 仍看得到；artifacts mismatch（B branch fs 沒 `changes/<id>/`）走 `change.artifact_missing` op error **一律 reject** + doctor finding `state.artifact_missing` warning 提示 user 切回對應 branch（見 §17.4 / §16.12）。 |
| **Git worktree** | `.git/speclink/` 在 GIT_COMMON_DIR 內、所有 worktree 自動共用；§12 file lock 自然跨 worktree 生效；**不需任何 worktree-specific 邏輯**（不需 marker file、不需 symlink、不需 `worktree init` 指令）。 |
| **State.db recovery** | `speclink restore --from-artifacts`（§16.13）從 `.speclink/changes/*/` + `discussions/*/` + `archive/*/` 重建；換筆電 / SSD 損壞 / `migration.checksum_mismatch` 都走同條 path；新 `instance_id` 自動生成（舊變 historical actor reference）。 |

#### 13.9.5 Cross-tool namespace

`.git/<tool-namespace>/` convention 對其他 SDD 工具友善、互不干擾：

| Namespace | 工具 |
|---|---|
| `.git/spectra-app/` | spectra cli |
| `.git/speclink/` | SpecLink |

各 tool 自己 namespace；user 同機並用 spectra + speclink 不衝突。

### 13.10 Migration: LocalProvider → HttpProvider [deferred]

**情境**：個人 RD 用 LocalProvider 6 個月、後來加入團隊要把 state 搬到 team server。

**MVP scope**：未實作；本節僅紀錄方向，作為「LocalProvider 設計不鎖死個人 → 團隊路徑」的設計保證。

**Artifacts migration**（單向、靠 git）：

- `.speclink/changes/` / `discussions/` / `schemas/` / `archive/` 已 git tracked → user `git push` 後 team 端從 git 拉，artifacts 自然到位。
- 若 team webapp 走 §19.4.1 default mode → webapp server 端從 git 撈 artifact 或 user manual 上傳。
- 若走 §19.4.2 custom mode → webapp 既有 schema 不變、靠 mapping engine 接。

**State migration（`.git/speclink/state.db` 內容）**：

| state.db 內容 | 搬遷方式 |
|---|---|
| Change rows（state, etag）| Server 端從 artifact + audit log 重建（同 `speclink restore` 邏輯）|
| `feedback_tasks` | Server 端 parse tasks.md HTML marker 重建 |
| `audit` events | 若 `.speclink/audit/<change-id>.log` git tracked → import；否則 lost |
| `instance_id` | 不搬遷（by design，§13.6）；server 端紀錄 actor 各自 |
| Lock state | 不搬遷（runtime 暫態 / 跟新 server 端機制）|

**未來指令**（[deferred]，跟 HttpProvider 一起做）：

```bash
speclink migrate-to-http --baseUrl <url> --auth <token> [--dry-run] [--json]
```

預期行為：
- artifacts 已在 git → git push 即可、不需此指令額外做
- 從 local state.db 序列化 audit / feedback / review history → 推 server import endpoint
- 寫 `.speclink/link.yaml` 切換 provider 設定
- 印 migration report（哪些 state 成功搬、哪些 lost）

**設計約束**：
- 一次性、不雙寫（不維護 local + remote 同步）
- 失敗可重跑（idempotent on server 端）
- Migration 期間 local 進入 read-only mode、防 divergence

**為什麼 MVP 不做**：HttpProvider impl 整體 [deferred]（§18.2）；migration 屬於 HttpProvider 的 user journey 一環、一起延後合理。本節寫成 stub、是為了在設計層級確認「個人 → 團隊路徑可走」、避免 LocalProvider 設計時把 user 鎖死。

## 14. 檔案結構

**Two-root layout**（§13.9）：`.speclink/` 為 git-tracked artifacts；`.git/speclink/` 為 local-only state。

```
專案根/
  .speclink/                            # ── working tree、git-tracked artifacts ──
    config.yaml                         # tracked — SDD 規則（LocalProvider 模式）
    link.yaml                           # gitignored — per-clone provider 連線設定
    
    changes/
      <change-id>/
        proposal.md                     # tracked
        design.md                       # tracked（optional）
        tasks.md                        # tracked
        specs/<capability>/
          spec.md                       # tracked
        metadata.json                   # tracked
    
    discussions/
      <discussion-id>/
        discussion.md                   # tracked
    
    schemas/
      <schema-id>/                      # tracked — 使用者 fork 的 schema
        schema.yaml
        templates/
          <artifact-id>.md
    
    archive/
      <YYYY-MM-DD>-<change-id>/         # tracked — archived changes
        ...
    
    audit/
      <change-id>.log                   # tracked 由使用者決定 — review / commit audit
  
  .git/speclink/                        # ── local-only state（git 自然不追蹤 .git/ 內部） ──
                                        #    路徑由 `git rev-parse --git-common-dir` 解析；
                                        #    worktree 自動共用主 repo 的 .git/speclink/
    state.db                            # SQLite — change rows / etag / feedback_tasks /
                                        #          audit / review history / instance_id
    state.db-wal                        # SQLite WAL（§12.3）
    state.db-shm                        # SQLite shared mem
    
    locks/
      global.lock                       # §12 global advisory lock（init / migration）
      changes/
        <change-id>.lock                # §12 per-change advisory lock
    
    touched/
      <change-id>.json                  # task done 寫的 touched files
```

**HttpProvider 模式下** `.speclink/config.yaml` 不在本地（由 server 持有，§11.5）；`.speclink/changes/<id>/` 等 artifacts workflow 中也不在本地、archive 後才同步進 git（§13.3）。`.git/speclink/state.db` 在 HttpProvider 模式下用途有限（可能僅存 cache / actor identity），詳見 §19.4 規劃。

### 14.1 預設 .gitignore

`speclink init` 自動 append `.gitignore`（idempotent；存在則 skip）。**僅一條**：

```
.speclink/link.yaml
```

**為什麼只有一條**：

- `state.db` / `state.db-wal` / `state.db-shm` / `locks/` / `touched/` 全部住在 `.git/speclink/` 內、git 自然不追蹤 `.git/` 自己，**無需** gitignore 條目
- `.speclink/changes/` / `discussions/` / `schemas/` / `archive/` / `audit/` / `config.yaml` 全部是 tracked artifacts，**不該** ignore

**`link.yaml` 入 gitignore 的理由**：per-clone 連線設定（可能含 endpoint / credential ref）、不同 RD 的 clone 可能連不同 provider、可能含敏感資訊，不該跟著 git push 走。物理上仍住在 `.speclink/` 內，只是 git 不追蹤。

**Non-git 專案**（`speclink init --no-git`，[deferred]、§18.2）下 fallback 行為：

- state.db / locks / touched 改放 `.speclink/state.db`、`.speclink/locks/`、`.speclink/touched/`
- init 寫入 gitignore 對應條目（即使無 .git/、寫一份備未來 `git init`）
- 本 MVP 不實作此 fallback；本段僅紀錄方向

## 15. Crate 拆分

```
crates/
  speclink-core/        # ★ SDD engine library
                        #    - Provider trait
                        #    - artifact DAG / schema / role / lifecycle
                        #    - instructions resolver
                        #    - analyze / validate / drift 邏輯
  speclink-cli/         # thin frontend
                        #    - clap parse
                        #    - JSON serialize
                        #    - stdin/stdout
  speclink-provider-local/  # filesystem + SQLite 實作（MVP 唯一 impl）
                        #    - LocalProvider : Provider
                        #    - state.db schema + migration
                        #    - advisory file lock
```

未來加（trait skeleton 在 §19 預留，但 crate 跟實作都延後到 LocalProvider 完成後）：

```
  speclink-provider-http/   # HTTP-based provider 實作（延後）
                        #    - default mode: SpecLink 標準 endpoints
                        #    - custom mode: 宣告式 HTTP template mapping（見 §19.4）
                        #    - optional Unix socket transport（同 protocol，未來加）
```

### 15.1 Library 介面（other frontends 用）

```rust
// speclink-core
pub trait Provider { ... }
pub struct Engine<P: Provider> { ... }

impl<P: Provider> Engine<P> {
    pub fn list_changes(&self, filter: ChangeFilter) -> Result<Vec<ChangeSummary>>;
    pub fn get_instructions(&self, request: InstructionsRequest) -> Result<Instructions>;
    pub fn write_artifact(&self, request: WriteArtifactRequest) -> Result<WriteResult>;
    pub fn task_done(&self, request: TaskDoneRequest) -> Result<TaskDoneResult>;
    pub fn review_approve(&self, request: ReviewApproveRequest) -> Result<()>;
    pub fn archive(&self, change_id: &str, options: ArchiveOptions) -> Result<ArchiveResult>;
    pub fn analyze(&self, change_id: &str) -> Result<AnalyzeReport>;
    pub fn validate(&self, change_id: &str, strict: bool) -> Result<ValidateReport>;
    pub fn drift(&self, change_id: &str) -> Result<DriftReport>;
    
    // Discussion
    pub fn discuss_new(&self, request: DiscussNewRequest) -> Result<DiscussionId>;
    pub fn discuss_capture(&self, request: DiscussCaptureRequest) -> Result<()>;
    pub fn discuss_conclude(&self, request: DiscussConcludeRequest) -> Result<()>;
    pub fn discuss_get(&self, id: &str) -> Result<Discussion>;
    
    // Schema
    pub fn schema_list(&self) -> Result<Vec<SchemaSummary>>;
    pub fn schema_get(&self, id: &str) -> Result<Schema>;
    pub fn schema_fork(&self, source: &str, target: &str) -> Result<()>;
    pub fn schema_validate(&self, id: &str) -> Result<ValidateReport>;
}
```

任何想建自己 frontend 的人 link `speclink-core`，提供自己的 Provider 實作（或用 `speclink-provider-local`），即可建 web UI / IDE plugin / 自訂 chatbot 等。

### 15.2 npm package（給 Tool binding host 用）

```
clients/typescript/        # @speclink/client npm package
                           #   - TypeScript / JavaScript SDK
                           #   - 內部 spawn `speclink` CLI subprocess（未來可換 N-API 直連 core）
                           #   - 三層 API：見 §22
                           #   - typed methods + JSON schemas + Type definitions exports
                           #   - getCopilotSdkTools / getOpenAITools / getLangChainTools / etc.
                           #   - installSkills（with aliases）
                           #   - 設定來源：constructor / fromEnv / fromConfig / fromWorkspace
```

未來加（依 adoption signal）：

```
clients/python/            # speclink-client pip package
clients/go/                # speclink-client Go module
```

## 16. CLI 指令完整面

### 16.0 全域規範

**子指令命名**：

| 動詞 | 形式 | 例 |
|---|---|---|
| **生產動詞**（`create` / `new` / `write`）| `new <noun>`（verb-first 例外）| `speclink new change <name>` / `speclink new artifact <kind>` |
| **其他動詞** | `<noun> <verb>`（subject-first 主流）| `speclink change show <id>` / `speclink apply start` / `speclink task done <id>` |

**Verb-first `new` 例外的理由**：對齊 spectra 慣性 + 對齊「動詞前置」自然語序。Catalogue 內以 `cli_verb_first: true` flag 標記（§21.4）。AI / 文件 / `describe-tools text` 都依此規則 derive。

**全域 flag**（每個 subcommand 都支援）：

| Flag | 語意 |
|---|---|
| `--json` | 強制 JSON 輸出（即使無 payload 也回 `{ok:true, data:null}`）；非互動 mode |
| `--non-interactive` | 拒絕任何互動式 prompt，遇到回 `cli.requires_tty` error |
| `--no-color` | 禁色彩 |
| `--retry-budget-ms <n>` | 同一 skill flow 累積 retry budget（§12.5.2）|
| `-h` / `--help` | 印 usage |
| `-V` / `--version` | 印版本 |

**`--json` 強制**：SDK 內部所有 spawn 都帶 `--json`。所有 subcommand **必須**支援 `--json`（包含 destructive ops 如 `unlink` / `delete`）— 無 payload 時回 `{ok: true, data: null}`。Skill drafts 內所有 CLI invoke **必須**帶 `--json`。

**主要 entity 用 positional、次要用 flag**：

```
speclink task done <task-id> --change <change-id>     ✓
speclink review approve --change <id> --phase code    ✓
speclink apply start <change-id>                      ✓（單一 entity）
speclink show change <change-id>                      ✓
```

唯一 entity → positional；多 entity → primary positional + secondary flag。

**Query verb 區分**：

| Verb | 對象 | 例 |
|---|---|---|
| `show <noun> <id>` | 單一 entity 完整內容 | `speclink show change billing` |
| `status [--change <id>]` | 當前 project / change 的 lifecycle 快照 | `speclink status` |
| `list [--changes\|--specs\|--discussions]` | 索引 / 列表 | `speclink list --changes --state ready` |
| `<noun> list` | 同 entity-namespace list 別名 | `speclink discuss list` ≡ `speclink list --discussions` |

**Skill 別名**：`speclink propose` / `speclink apply` / `speclink discuss` / `speclink archive` / `speclink ingest` 作為單獨子命令時印「This is a skill, run via your AI host: `/speclink-propose` (Claude Code) / `$speclink-propose` (Codex) / tool `speclink_propose` (Tool binding). For individual operations see `speclink --help`.」並 exit 0。

### 16.1 Project 管理

```bash
# 新建 project（MVP）
speclink init <name> [--force] [--json]              # LocalProvider，建 .speclink/config.yaml + .git/speclink/state.db
speclink init <name> --provider http --baseUrl <url> # HttpProvider 新 project [deferred]

# 連到既有 project [deferred — HttpProvider 一起]
speclink link <provider-url> [--project <id>] [--auto-suffix] [--non-interactive] [--json]
speclink unlink [--json]                             # 刪 .speclink/link.yaml，不動 provider 端資料

# 從 artifacts 重建 state.db（MVP，§16.13）
speclink restore [--from-artifacts] [--dry-run] [--overwrite] [--json]

# 診斷（detail 見 §16.12）
speclink doctor [--json] [--quick] [--fix]
  [--check <category>]              # 可多次：--check provider --check skill
  [--check-mapping] [--live]        # HttpProvider custom mode mapping dry-run
```

**`speclink init` 行為**（LocalProvider）：

1. 偵測 cwd 在 git working tree → 跑 `git rev-parse --git-common-dir`
    - 失敗 → error `project.requires_git`、訊息「SpecLink LocalProvider requires a git repository. Run `git init` first.」；exit code 對應 `2`（input error，§17.2）
    - 成功 → state root = `<common-dir>/speclink/`
2. 若 `.speclink/config.yaml` 已存在 + 無 `--force` → error `project.already_initialized`
3. 建 `.speclink/`、寫入 `.speclink/config.yaml`（embedded template）
4. 建 `.git/speclink/`、init SQLite `state.db`（含 `schema_migrations` + `speclink_meta.instance_id`）
5. Append `.gitignore`（idempotent，僅一條 `.speclink/link.yaml`，§14.1）
6. 印初始化完成訊息 + HttpProvider 推薦提示（§13.6.1）

**MVP scope 明確**：只 `init` 純 LocalProvider + `restore` + `doctor`。所有帶 `--provider http` / `link` / `unlink` 屬 [deferred]，CLI 偵測到會回 `provider.not_supported` + 訊息「HttpProvider not implemented in this build」。

`speclink link` 沒帶 `--project` 時走互動式 list（CLI 透過 `ProviderRegistry::list_projects()` 拿選項，見 §19.2）— 但若 stdin **非 TTY** 或帶 `--json` / `--non-interactive` flag → 回 `cli.requires_tty` error 並提示「Pass `--project <id>` to skip interactive prompt」。AI Bash binding host **永遠**該帶 `--project` + `--json` 避免 hang。

`--auto-suffix`（僅 `init` 用）：撞名時自動加短 suffix（如 `billing-system-7k2a`），給 CI / script 用；互動式預設不啟用，要 user 主動決定。

### 16.2 Tool metadata 與 MCP

```bash
# 印 operation catalogue 各種格式（給 SDK adapter / 文件 / debugging 用）
speclink describe-tools [--format <claude|copilot-sdk|copilotkit|openai|langchain|mcp|json|text>]

# 啟 MCP server 模式（延後到 MVP 後）
speclink mcp serve [--stdio | --port <port>]
```

`describe-tools` 從內建 operation catalogue 衍生（見 §21），輸出多種格式，給：

- `@speclink/client` 內部 `getXxxTools()` helper 用
- 文件生成
- Debugging / 手動驗證

`mcp serve` 把每個 operation 包成 MCP tool，給 MCP-compatible host 接（延後實作）。

### 16.3 Discussion

```bash
# CRUD
speclink discuss new <topic-id> [--about <change>...] [--description "..."]
speclink discuss list [--json] [--status iterating|converged] [--all] [--about <change>]
                              # 預設 (無 flag) 只顯示 iterating；converged 需顯式列
speclink discuss show <topic-id> [--json]
speclink discuss link --topic <id> --about <change>
speclink discuss delete <topic-id> [--force]
                              # 預設互動確認；若 linked_changes 非空，警告再要求二次確認
                              # --force 跳過互動但仍會在 stderr 印 linked_changes 警告

# Section-level patch（採 Section patch 模式，見 §8.5）
speclink discuss patch --topic <id> --section background --stdin [--json]
speclink discuss patch --topic <id> --section open_questions --stdin [--json]
speclink discuss patch --topic <id> --section decisions_made --append --stdin [--json]
speclink discuss patch --topic <id> --section rounds --append --round-name "..." --role <role> --stdin [--json]
speclink discuss patch --topic <id> --section conclusion --stdin [--json]

# 收斂（內含 conclusion patch + status 轉 converged）
speclink discuss conclude --topic <id> --stdin [--capture-to <change>...] [--json]
```

### 16.4 Change CRUD

```bash
speclink new change <name> [--schema <schema-id>] [--description "..."] [--json]
speclink list [--changes|--specs|--discussions] [--state <s>] [--sort <name|created|modified>] [--json]
speclink show change <change-id> [--json] [--requirements] [--deltas-only]
speclink show spec <capability> [--json] [--requirements] [--item-type change|spec]
speclink status --change <id> [--json]
```

### 16.5 Instructions（給 AI 看的 prompt + structure）

```bash
speclink instructions <artifact-or-step> [選項] --json
```

`<artifact-or-step>` 可為：

| 值 | 用途 |
|---|---|
| `proposal` / `spec` / `design` / `tasks` | 寫 artifact 的 prompt + template + rules |
| `discuss` | discuss 的 prompt（搭配 `--role`） |
| `apply` | apply skill 的 prompt（讀 proposal/spec/tasks） |
| `ingest` | ingest skill 的 prompt（讀 artifact + drift report） |
| `commit` | archive sub-flow 用，回 commit message structure + touched files |

通用選項：`--change <id>` `--topic <id>` `--role <id>` `--capability <name>` `--json`

### 16.6 Artifact 寫入

```bash
speclink new artifact <kind> --change <id> [--capability <name>] --stdin [--overwrite] [--json]
```

`<kind>`：`proposal` / `spec` / `design` / `tasks`（依 schema 動態）。

**`--overwrite`** 是「覆寫既有 artifact body」的明示語意（rewrite use case，如 ingest 流程），**不是 `--force`**。`--force` 在 SpecLink 一律保留給「bypass safety check」的 destructive ops（`init --force` / `discuss delete --force` / `change delete --force` / `schema delete --force` / `uninstall --force`），AI skill 永不主動帶 `--force`；catalogue 標 `destructive: true` op 走 §17.6 audit。Artifact rewrite 的合法路徑用 `--overwrite`，AI 在 ingest / fix flow 內可主動帶。

### 16.7 Apply / Task

```bash
# State transition（apply skill 自動呼，使用者通常不直接打）
speclink apply start <change-id> [--actor <id>] [--json]
speclink apply pause <change-id> [--json]

# Task 操作（不觸發 state transition）
speclink task done <task-id> --change <id> [--json]
speclink task list --change <id> [--json]
speclink task undo <task-id> --change <id> [--json]
```

**`apply start` 採 ensure-actor 語意**（見 §6.2 詳細表）：

- 對 `ready` → transition `in_progress` + assign actor；回 `{ state: "in_progress" }`
- 對 `in_progress` → no-op idempotent + assign actor；回 `{ state: "in_progress" }`
- 對 `code_reviewing` / `archived` → **不轉移** + 回 `{ state: <當前 state>, message: "..." }`（不 error）；skill 端依 response.state 判斷退出 apply
- 對 `proposing` / `reviewing` → `state.transition_invalid` error

回傳 JSON：

```json
{
  "ok": true,
  "data": {
    "change_id": "billing-export",
    "state": "in_progress",       // 永遠是當前最新 state
    "actor": { "agent_host": "claude-code", "os_user": "alice" },
    "message": null               // 或 "Already in code review; nothing to apply." 等
  }
}
```

**`apply pause` 採對稱 idempotency**：

- 對 `in_progress` → transition `ready` + 清 actor
- 對 `ready` → **no-op idempotent**（已在 ready）
- 對其他 state → `state.transition_invalid`

`task done` 完成最後一個 task 時 engine 在 auto-transition 前驗證 **synthetic feedback task 完整性**（見 §6.2）：所有 `feedback_tasks.status='pending'` 紀錄的 marker 必須仍在 tasks.md 且 `[x]`。否則 `tasks.feedback_task_removed` + 自動 re-append + state 不變。

> **Slice 標註**：A3=`add-state-machine-and-apply` 落實 `apply start` / `apply pause` / `task list` / `task done` / `task undo` 5 個 CLI op。Feedback task 完整性驗證留給 review slice 接通。

### 16.8 Review

```bash
speclink review approve --change <id> --reviewer <id> --phase artifact|code [--note "..."] [--json]
speclink review reject  --change <id> --reviewer <id> --phase artifact|code --reason "..." [--json]
speclink review status  --change <id> [--json]
speclink review history --change <id> [--json]
```

**規則**：

- `--phase` **required**：`artifact`（artifact review，退回 reviewing → 修 artifact）或 `code`（code review，退回 in_progress + 自動加 synthetic feedback task）
- `review reject` 的 `--reason` **required**；empty reason 回 `review.reason_required` error
- `review approve` 對應 phase 必須是當前 state（`artifact` 對 `reviewing`、`code` 對 `code_reviewing`）；mismatch 回 `review.wrong_phase`
- `review reject --phase code` 觸發 **synthetic feedback task auto-append**（見 §6.2 reject re-entry 機制 + marker 規格）
- **重複 reject 防呆**：若 30 分鐘內同 reviewer 對同 change 用語意接近的 reason（cosine similarity > 0.8 或 substring match）reject → warning「相同 feedback 在 X 分鐘前 reject 過、engineer 真的修正了嗎？」（不阻止；engine 可選擇性實作 — MVP 用簡單 substring 比對）

**`review history` 規格**：

```bash
speclink review history --change <id> [--json] [--phase artifact|code]
```

回傳該 change 全部 review approve/reject 紀錄（依時間排序），含：

```json
{
  "data": [
    {
      "feedback_id": "fb-2026-05-21-001",
      "reviewer": "alice",
      "phase": "code",
      "action": "reject",
      "reason": "auth check missing in middleware",
      "timestamp": "2026-05-21T10:15:00Z",
      "feedback_task_status": "done"   // 或 "pending"，若是 reject
    },
    {
      "reviewer": "alice",
      "phase": "code",
      "action": "approve",
      "note": "now LGTM",
      "timestamp": "2026-05-21T11:42:00Z"
    }
  ]
}
```

reviewer 用此查詢比對「engineer 是否真的修正前次回饋」。

### 16.9 Archive

```bash
speclink archive <change-id> [--skip-specs] [--no-validate] [--mark-tasks-complete] [--yes] [--json]
```

### 16.10 Engine 內建分析

```bash
speclink analyze <change-id> [--json]
speclink validate <change-id> [--strict] [--all] [--changes] [--specs] [--json]
speclink drift <change-id> [--json]
```

### 16.11 Schema 管理

```bash
speclink schemas [--json]                                              # 列出可用 schema
speclink schema show <id> [--json]                                     # 顯示 schema 內容
speclink schema fork <source-id> <target-id> [--force]                 # fork built-in 或既有
speclink schema validate <id> [--strict] [--json]                      # 驗證 schema.yaml
speclink schema delete <id>                                            # 刪除（拒絕刪 built-in）
speclink templates [--schema <id>] [--json]                            # 列出 template
```

### 16.12 Doctor 檢查項目

`speclink doctor` 是 **聚合診斷指令** — 對「我有問題不知從哪下手」的使用者一鍵抓全。**不**自動觸發、不在 hot path 跑（見 §5）。

#### 16.12.1 9 個檢查類別

預設全跑；可用 `--check <category>` 篩子集（多次累加），或 `--quick` 跳過 expensive checks。

| # | Category（`--check` 參數）| 範例 checks |
|---|---|---|
| 1 | `cli` | speclink CLI 在 PATH、CLI version vs SDK package version 匹配、embedded schema migrations applied |
| 2 | `project` | `.speclink/` 存在；`config.yaml` parseable + schema valid；`link.yaml` parseable（若存在）；**cwd 在 git working tree**（LocalProvider，§13.9.2）；`.git/speclink/state.db` `PRAGMA integrity_check` |
| 3 | `provider` | LocalProvider: `.git/speclink/state.db` 可開 + WAL mode + advisory lock try-acquire（`.git/speclink/locks/global.lock`）；HttpProvider: baseUrl 可達 + auth 試打 health endpoint（若 server 提供）|
| 4 | `provider-mapping` | （HttpProvider custom mode）對 catalogue 每個 operation 驗證 mapping shape（template 語法 / `$field` 對應 input / `$.field` 對應 output schema）；缺漏 mapping → error。**預設 dry-run 不打網路**；`--live` 才實際打 server |
| 5 | `security` | link.yaml 不含明文 secret（§19.5）；link.yaml 不在 git index；baseUrl 是 `https://` / `localhost` / `unix://` 之一 |
| 6 | `config` | locale 在 allowlist；rules sections 結構合法；roles `extends` 可解析；schema 引用存在 |
| 7 | `skill` | 每個 installed skill 的 `template_hash` vs binary embedded body（drift 偵測，§20.2）；`speclink_version` vs CLI version；AGENTS.md / CLAUDE.md markers 完整 |
| 8 | `state` | 所有 in_progress changes 有 `actor` 欄位；`feedback_tasks` table 行 ↔ tasks.md HTML marker 雙向 orphan check（§6.2）；changes table state enum 合法；`speclink_meta.instance_id` 存在；**state.db change row ↔ `.speclink/changes/<id>/` filesystem 雙向 cross-check**（artifact missing → `doctor.state.artifact_missing` warning，§13.9.4）|
| 9 | `artifacts` | （**預設 on、`--quick` 跳過**）對每個 active change 跑 `analyze` + `validate` + `drift`，aggregate 結果 |

`--check` 是 `category` 字面值（如 `--check security --check skill`）。`--check-mapping` 是 `--check provider-mapping` 的別名（沿用 §19.5 已預告的命名）。

#### 16.12.2 Diagnostic Levels 與 Exit Code

| Level | 語意 | 例子 |
|---|---|---|
| `error` | 阻擋 SpecLink 運作 / 安全漏洞 | state.db 損壞、link.yaml 含明文 secret |
| `warning` | 不阻擋但建議修 | HTTP baseUrl（建議升級 HTTPS）、skill template_hash drift |
| `info` | 純資訊 | instance_id、CLI 版本、provider 模式 |

| Exit code | 條件 |
|---|---|
| `0` | clean（沒有 error / warning） |
| `1` | warnings only |
| `2` | 有 errors |

對齊 §17.2 exit code 慣例。

#### 16.12.3 `--fix` Auto-fix Allowlist

MVP 只 auto-fix 四條 — **safe，無破壞性**（restore 雖然 substantial、但 idempotent + 不動 artifacts、視為 safe）：

| Finding | Auto-fix 動作 |
|---|---|
| `.gitignore` 缺漏 `.speclink/link.yaml`（§14.1 唯一條目） | append `.speclink/link.yaml`（idempotent；不重寫既有條目） |
| AGENTS.md / CLAUDE.md `SPECLINK:START` markers 缺漏或損壞 | 同 `speclink update` 內部 marker re-inject 邏輯 |
| `tasks.md` 缺漏對應 `feedback_tasks` 行的 HTML marker（§6.2 `tasks.feedback_task_removed`）| re-append synthetic feedback task 區塊 |
| `doctor.state.db_missing`（`.git/speclink/state.db` 不存在）| 跑 `speclink restore --from-artifacts`（§16.13）；idempotent；不動 artifacts |

**Non-auto-fix**：
- 不刪 state.db
- **不**自動 fix `doctor.state.db_corrupted`（要 user 顯式跑 `restore --overwrite` 確認）
- **不**自動 fix `doctor.state.artifact_missing`（cross-branch / 過期 state — user 須自己決定切 branch 還是刪 change）
- 不重置 lock file（即使 process 已死 — 由 user 自己確認）
- 不改 config.yaml（即使欄位明顯錯）
- 不動 link.yaml（特別是 secret 移到 env var 這種改動，user 必須自己決定）

`--fix` 跟 diagnostic level 互動：每筆 finding 多一個 `auto_fixable: bool`；非 fixable 的 finding 印 `fix:` 文字提示給 user 手動處理。

#### 16.12.4 JSON 輸出 shape

依 §17.1 通用結構：

```json
{
  "ok": false,
  "data": {
    "summary": { "errors": 1, "warnings": 2, "info": 5 },
    "findings": [
      {
        "category": "security",
        "level": "error",
        "code": "doctor.security.link_yaml.plaintext_secret",
        "message": "Plaintext token detected in .speclink/link.yaml line 4.",
        "details": { "file": ".speclink/link.yaml", "line": 4, "field": "auth" },
        "fix_message": "Move the token to an env var: auth: ${SPECLINK_TOKEN}",
        "fix_command": null,
        "auto_fixable": false
      },
      {
        "category": "skill",
        "level": "warning",
        "code": "doctor.skill.template_drift",
        "message": "Skill 'speclink-apply' body has been edited locally.",
        "details": {
          "tool": "claude",
          "skill": "apply",
          "expected_hash": "sha256:...",
          "actual_hash": "sha256:..."
        },
        "fix_message": "Run the update command to overwrite local changes.",
        "fix_command": "speclink update --force",
        "auto_fixable": false
      }
    ]
  },
  "warnings": []
}
```

**Finding 欄位**：

| 欄位 | 型別 | 語意 |
|---|---|---|
| `code` | string | `doctor.<category>.<code>` 命名（見 §17.5）|
| `category` | string | 9 大類之一（§16.12.1）|
| `level` | `"error" \| "warning" \| "info"` | §16.12.2 |
| `message` | string | 人類可讀 |
| `details` | object | 結構化補充 |
| `fix_message` | string \| null | 人類描述如何修（含 placeholder）|
| `fix_command` | string \| null | 機器可動作的指令（AI 可直接執行；無 placeholder） |
| `auto_fixable` | boolean | `doctor --fix` 是否能自動修（§16.12.3 allowlist）|

`fix_message` 跟 `fix_command` 分開 — AI host 拿 `fix_command` 直接 spawn、不必 NLP 解析 backtick；人類看 `fix_message` 解釋。

#### 16.12.5 不在 doctor 範圍

- **不**自動觸發（§5 已寫明）
- **不**做 cross-machine network probing（如 ping team server）
- **不**對隔壁 working dir / 其他 git repo 做檢查（boundary = 本 `.speclink/`）
- **不**做 performance benchmarking
- **不**自動清理 orphan files / 重置 state — 要 user 顯式跑

### 16.13 State recovery (`restore`)

`.git/speclink/state.db`（§13.9）損壞 / 遺失時，從 git tracked artifacts 重建。

```bash
speclink restore [--from-artifacts] [--dry-run] [--overwrite] [--json]
```

| Flag | 行為 |
|---|---|
| `--from-artifacts`（預設、MVP 唯一 mode）| 從 `.speclink/changes/*/` + `discussions/*/` + `archive/*/` +（若存在）`.speclink/audit/*.log` 重建 `.git/speclink/state.db` |
| `--dry-run` | 只印「會做什麼」、不寫 state.db |
| `--overwrite` | state.db 已存在 + 健康時強制覆寫；無此 flag 時 state.db 存在 → 拒絕（要 user 顯式選擇） |
| 預設無 flag 時 | state.db 不存在 / SQLite open 失敗 → 自動跑、無需 flag |

**觸發場景**：

| 場景 | 行為 |
|---|---|
| 換筆電 / 新 clone（state.db 不存在） | 任何 CLI 指令偵測到 → 提示 user 跑 `speclink restore`（doctor finding `state.db_missing`）|
| SQLite open 失敗（corruption） | 自動先備份到 `.git/speclink/state.db.bak-<timestamp>`，然後**拒絕**繼續、提示 user 跑 `speclink restore --overwrite`（doctor finding `state.db_corrupted`，**不**自動 fix）|
| `migration.checksum_mismatch` / `migration.version_too_new`（§12.7）| user 手動跑 restore（migration error 不自動觸發 restore；user 須確認 binary 版本 / state.db 來源）|
| 個人 RD 想刻意重置 audit history | 跑 `restore --overwrite`（destructive、要二次確認） |

**重建邏輯**（pseudo）：

1. 取得 global lock `.git/speclink/locks/global.lock`（timeout 30s）
2. 若 state.db 存在但 corrupted → 備份到 `state.db.bak-<timestamp>`
3. 建立新 state.db、apply 全部 migration（同 §12.7）
4. 生成新 `instance_id` 寫入 `speclink_meta`
5. 掃 `.speclink/changes/*/metadata.json` → 重建 change rows（state, etag = v1）
6. 掃 `.speclink/changes/*/tasks.md` → 解析 `<!-- speclink:feedback id=... -->` HTML marker → 重建 `feedback_tasks` 表
7. 掃 `.speclink/discussions/*/discussion.md` frontmatter → 重建 discussion rows
8. 掃 `.speclink/archive/<date>-<id>/` → 重建已 archived change rows（state = archived）
9. 若 `.speclink/audit/*.log` 存在 → import 為 audit events（`actor.instance_id` 保留舊值、變 historical reference）
10. 寫一筆 `audit.restored` event（含 `previous_instance_id`：從 audit log 推、無則 null）
11. 寫 markdown 報告 `.speclink/restore-<timestamp>.md`（內容同 JSON output 但人類可讀）
12. Release lock；印 JSON envelope

**輸出 envelope**：

```json
{
  "ok": true,
  "data": {
    "instance_id": "<新 UUID>",
    "previous_instance_id": "<從 audit log 推、否則 null>",
    "report_path": ".speclink/restore-2026-05-22T03-15-00Z.md",
    "restored": {
      "changes": 6,
      "discussions": 3,
      "archived_changes": 12,
      "feedback_tasks": 2,
      "audit_events_imported": 47
    },
    "lost": {
      "review_history": "若 audit log 未 tracked",
      "etag_history": "全部重置為 v1",
      "feedback_reason_hash": "tasks.md 不含 hash、無法重建"
    },
    "inconsistencies": [
      { "kind": "tasks_done_in_proposing_state", "change": "add-export", "hint": "tasks.md 顯示已勾 [x] 但 state=proposing；建議 user manually verify" }
    ]
  }
}
```

**規則**：

- **MVP 不支援** `--from-export <file>`（從顯式 backup 還原）；只走 from-artifacts、未來 `speclink export` 進來再加
- **不修補 `.speclink/` artifacts**（讀-only；restore 是把 artifact 推回 state.db、反向）
- **Etag 全重置為 v1**：caller 若手上有 stale etag → 下次 write 撞 `state.etag_mismatch`、依 §12.5.4 重讀即可恢復
- **Lock 必須 acquired**：restore 期間 reject 其他 CLI invocation（global lock 等待 timeout 30s）
- **Idempotent**：同 artifacts 跑兩次 restore 結果相同（除了 `instance_id` 會 rotate 兩次、各自一個新 UUID）
- **Audit 寫入**：restore 結束寫 `audit.restored` event（type, timestamp, restored counts, previous_instance_id, inconsistencies count）

**對應 doctor finding**（§16.12 增訂、見 §17.5）：

| Finding code | Default level | Auto-fixable |
|---|---|---|
| `doctor.state.db_missing` | error | ✓ 跑 `speclink restore` |
| `doctor.state.db_corrupted` | error | ✗（auto-fix 危險；要 user 手動確認） |
| `doctor.state.db_schema_invalid` | error | ✗（指向 migration error） |
| `doctor.state.artifact_missing` | warning | ✗ |

## 17. JSON 介面 Contract

### 17.1 通用結構

成功：

```json
{
  "ok": true,
  "data": { ... },
  "warnings": [],
  "requestId": "..."
}
```

失敗：

```json
{
  "ok": false,
  "error": {
    "code": "change.not_found",
    "message": "Change 'foo' not found.",
    "details": {},
    "retryable": "no",
    "hint": "List existing changes with `speclink list --changes`."
  },
  "requestId": "..."
}
```

**Error envelope 必填欄位**（機器可讀，給 AI / SDK 用）：

| 欄位 | 型別 | 語意 |
|---|---|---|
| `code` | string | 點分隔 canonical ID（§17.4 列表）|
| `message` | string | 人類可讀訊息，**不**寫「Try again」這類重試指引（指引走 `retryable`）|
| `details` | object | 結構化補充欄位（如 `lockHolderPid`、`expected_etag` 等）|
| `retryable` | `"backoff" \| "read-then-retry" \| "read-then-bail" \| "no"` | 對齊 §12.5 retry 四值；caller 直接 dispatch 不用 NLP 解析 message |
| `hint` | string \| null | 機器可動作建議（≤120 字、可選），如 `change.invalid_name` 給 slug regex |
| `retry_after_ms` | number \| null | `retryable=backoff` 時可選；對應 HTTP `Retry-After` 或 server 端建議 wait |

`retryable` 由 catalogue 每個 operation × error code 對應（§17.4）；CLI / Provider 不該回不在 §17.4 的 code。

### 17.2 Exit Codes

| Code | 意義 |
|---|---|
| 0 | 成功 |
| 1 | 一般錯誤 |
| 2 | 使用者輸入錯誤 |
| 3 | validation failed |
| 4 | analyzer found blocking issues |
| 5 | provider unavailable（MVP 不會出現） |
| 6 | auth required（MVP 不會出現） |
| 7 | conflict（lock 取不到 / version 衝突） |

### 17.3 Error Code Category Summary

點分隔命名 `<category>.<code>`。完整 reference 見 §17.4（user-facing errors）/ §17.5（doctor findings）/ §17.6（audit events）。

| 領域 | 範例 |
|---|---|
| project | `project.not_initialized` / `project.already_initialized` / `project.id_taken` / `project.requires_git` |
| change | `change.not_found` / `change.locked` / `change.invalid_name` / `change.artifact_missing` |
| artifact | `artifact.not_found` / `artifact.validation_failed` / `artifact.already_exists` |
| state | `state.transition_invalid` / `state.lock_timeout` / `state.etag_mismatch` |
| lock | `lock.foreign_host`（§12.5.6）/ `lock.stale_takeover`（audit event）|
| schema | `schema.not_found` / `schema.invalid` / `schema.cannot_override_builtin` |
| role | `role.unknown` |
| review | `review.unauthorized` / `review.wrong_phase` / `review.reason_required` |
| config | `config.malformed` / `config.<field>.invalid` / `config.missing` |
| discussion | `discussion.not_found` / `discussion.already_converged` / `discussion.has_linked_changes` / `discussion.section_not_found` |
| tasks | `tasks.not_found` / `tasks.feedback_task_removed` / `tasks.parse_failed` |
| cli | `cli.not_found` / `cli.version_mismatch` |
| provider | `provider.unreachable` / `provider.not_supported` |
| auth | `auth.required` / `auth.invalid` |
| mapping | `mapping.invalid` |
| archive | `archive.no_active_change` / `archive.spec_merge_conflict` |
| migration | `migration.failed` |
| doctor | `doctor.<category>.<code>`（完整列表見 §17.5）|

### 17.4 Complete User-Facing Error Code Reference

**Retry 欄位語意**（對照 §12.5）：

| 標記 | 行為 |
|---|---|
| `backoff` | §12.5.2 jittered backoff（最多 4 次嘗試、累積最差 ~7s）|
| `read-then-retry` | §12.5.4 — caller 重 read 後一次重試；二次仍 fail bubble up |
| `read-then-bail` | §12.5.3 — 不重試、重 read 後依當前 state 決定退出 |
| `no` | 不重試、直接 bubble up |

**MVP 標記**：`[MVP]` = 在 MVP scope 內可觸發；`[deferred]` = HttpProvider / auth 機制完成後才會被觸發，但 trait skeleton 階段預先寫進 contract。

| Code | Exit | Retry | Description | Thrown by | MVP |
|---|---|---|---|---|---|
| `project.not_initialized` | 2 | no | `.speclink/` 不存在 | CLI 啟動所有指令 | [MVP] |
| `project.already_initialized` | 1 | no | `init` 時 `.speclink/` 已存在且未帶 `--force` | `init` | [MVP] |
| `project.requires_git` | 2 | no | LocalProvider 偵測到 cwd 非 git working tree（`git rev-parse --git-common-dir` 失敗）；hint：「Run `git init` first.」（§13.9.2）| `init` / 任何 CLI 啟動 | [MVP] |
| `project.id_taken` | 2 | no | slug 撞 server unique constraint | `init` / `link` | [deferred] |
| `change.not_found` | 2 | no | 指定的 change id 不存在 | 多數 change-targeted 指令 | [MVP] |
| `change.locked` | 7 | backoff | per-change lock 取不到 | `task done` / `apply start` / `archive` / `review` / `new artifact` 等 | [MVP] |
| `change.invalid_name` | 2 | no | name 不符 §13.4 slug 規範 | `new change` | [MVP] |
| `change.artifact_missing` | 1 | no | state.db 有 change row 但 `.speclink/changes/<id>/` 在 fs 不存在（cross-branch / 殘留 / artifacts root 被刪）；hint：「Switch to the branch where this change was created, or run `speclink change delete --force` to remove it from state.」**一律 reject、不 fallback**（§13.9.4） | 多數 change-targeted 指令 | [MVP] |
| `artifact.not_found` | 2 | no | 指定 artifact 不存在 | `artifact read` / `instructions` | [MVP] |
| `artifact.already_exists` | 2 | no | `new artifact` 重複建立同 kind | `new artifact` | [MVP] |
| `artifact.validation_failed` | 3 | no | artifact 結構 / delta 驗證失敗 | `validate` / `new artifact` / `archive`（strict）| [MVP] |
| `state.transition_invalid` | 1 | read-then-bail | 當前 state 不允許此 transition | `apply start/pause` / `task done` / `review` / `archive` | [MVP] |
| `state.lock_timeout` | 7 | backoff | global / per-change lock 等待超時 | `init` / `migration` / 寫操作 | [MVP] |
| `state.etag_mismatch` | 7 | read-then-retry | If-Match / version 版本衝突 | 所有 `write_*` operation | [MVP] |
| `lock.foreign_host` | 7 | no | lock 檔 host 不是本機（§12.5.6）| lock acquire | [MVP] |
| `schema.not_found` | 2 | no | 指定 schema id 不存在 | `schema show` / `schema fork` 等 | [MVP] |
| `schema.invalid` | 3 | no | schema.yaml 結構錯 | `schema validate` / runtime load | [MVP] |
| `schema.cannot_override_builtin` | 1 | no | 試圖覆寫 binary 內建 schema | `schema fork` / `schema delete` | [MVP] |
| `role.unknown` | 1 | no | role id 不存在於 builtin + config.roles | `discuss` role 解析 | [MVP] |
| `review.unauthorized` | 1 | no | reviewer 不在 schema 允許名單 | `review approve/reject` | [MVP] |
| `review.wrong_phase` | 1 | no | review phase 跟當前 state 不符 | `review approve/reject` | [MVP] |
| `review.reason_required` | 2 | no | `review reject` 沒帶 `--reason` 或空字串 | `review reject` | [MVP] |
| `config.malformed` | 3 | no | config.yaml 整體 parse 失敗 | runtime load | [MVP] |
| `config.<field>.invalid` | 3 | no | 單一欄位驗證失敗（如 `config.workflow.locale.invalid`，warning level 也走此命名）| runtime load / `instructions` | [MVP] |
| `config.missing` | 1 | no | SDK 模式下完全找不到設定來源（無 constructor / env / fromConfig / fromWorkspace） | SDK constructor | [MVP] |
| `discussion.not_found` | 2 | no | discussion id 不存在 | `discuss show` / `discuss patch` 等 | [MVP] |
| `discussion.already_converged` | 1 | no | 試圖 patch / conclude 已 converged 的 discussion | `discuss patch` / `discuss conclude` | [MVP] |
| `discussion.has_linked_changes` | 1 | no | `discuss delete` 沒帶 `--force` 但有 linked changes | `discuss delete` | [MVP] |
| `discussion.section_not_found` | 2 | no | Section patch 找不到目標 section | `discuss patch --section` | [MVP] |
| `tasks.not_found` | 2 | no | 指定 task id 不存在於 tasks.md | `task done` | [MVP] |
| `tasks.feedback_task_removed` | 1 | no | synthetic feedback task marker 已被刪除（已 re-append）| `task done` 前驗證 | [MVP] |
| `tasks.parse_failed` | 3 | no | tasks.md 結構壞、無法解析 | `task done` / `validate` | [MVP] |
| `cli.not_found` | 1 | no | SDK 端找不到 `speclink` binary | `@speclink/client` constructor / 第一個 operation | [MVP] |
| `cli.version_mismatch` | 1 | no | SDK package version 跟 CLI binary version 不一致（warning level）| `@speclink/client` init 偵測 | [MVP] |
| `cli.requires_tty` | 2 | no | 互動式 prompt 但 stdin 非 TTY 或帶 `--json` / `--non-interactive` | `link` / `config edit` / `discuss delete` confirm 等 | [MVP] |
| `config.auth.unresolved` | 1 | no | link.yaml `${VAR}` 對應 env var 不存在或解析失敗 | runtime load | [MVP] |
| `mapping.host_mismatch` | 3 | no | custom mapping `path` resolve 後 host 不等於 `baseUrl` | mapping eval / `doctor --check-mapping` | [deferred] |
| `mapping.reserved_header` | 3 | no | custom mapping 宣告 `Authorization` / `Cookie` / `X-Speclink-*` header | mapping eval / `doctor --check-mapping` | [deferred] |
| `provider.unreachable` | 5 | backoff | LocalProvider 開不開 state.db / HttpProvider 連不上 server | runtime open | [MVP]（state.db 開不開）/ [deferred]（網路）|
| `provider.not_supported` | 1 | no | 當前 provider 不支援此 operation（如 LocalProvider 收到跨 project 操作） | trait dispatch | [deferred] |
| `auth.required` | 6 | no | HttpProvider 沒帶 auth header（HTTP 401）| HttpProvider request | [deferred] |
| `auth.invalid` | 6 | no | HttpProvider token 過期 / 拒絕（HTTP 403）| HttpProvider request | [deferred] |
| `mapping.invalid` | 3 | no | HttpProvider custom mode mapping shape 錯（template 語法 / 變數對不上 catalogue）| `doctor --check-mapping` / runtime mapping eval | [deferred] |
| `archive.no_active_change` | 1 | no | archive 但沒處於 `in_progress` / `code_reviewing` / `ready`（review optional 場景）| `archive` | [MVP] |
| `archive.spec_merge_conflict` | 3 | no | archive 時 delta spec 跟累積 spec 衝突 | `archive` spec delta merge | [MVP] |
| `migration.failed` | 1 | no | state.db schema migration 失敗（rollback 後）| `init` / `open` | [MVP] |
| `migration.version_too_new` | 1 | no | state.db schema_version 比 binary 預期新（user 裝舊版 binary）| `open` | [MVP] |
| `migration.checksum_mismatch` | 1 | no | `schema_migrations` checksum 對不上（state.db 被手改）| `open` | [MVP] |

### 17.5 Doctor Finding Code Reference

`speclink doctor` 走聚合報告（§16.12.4 JSON shape），每筆 finding 是 `{ code, level, message, fix, auto_fixable }` — 不對應 exit code（doctor 整體走 §16.12.2 的 0/1/2）。

| Code | Category | Default level | Auto-fixable | Description |
|---|---|---|---|---|
| `doctor.cli.version_mismatch` | cli | warning | no | CLI binary version 跟 SDK package version 不一致 |
| `doctor.cli.not_in_path` | cli | error | no | `speclink` binary 不在 PATH 中 |
| `doctor.project.not_initialized` | project | error | no | `.speclink/` 不存在 |
| `doctor.project.requires_git` | project | error | no | LocalProvider 但 cwd 不在 git working tree（§13.9.2）|
| `doctor.project.config_invalid` | project | error | no | `config.yaml` 結構錯 / 缺必要欄位 |
| `doctor.project.link_yaml_invalid` | project | error | no | `link.yaml` 結構錯 |
| `doctor.provider.unreachable` | provider | error | no | LocalProvider 開不開 / HttpProvider 連不上 |
| `doctor.provider_mapping.shape_mismatch` | provider-mapping | error | no | HttpProvider custom mapping 對 catalogue operation shape 不符 |
| `doctor.provider_mapping.missing_operation` | provider-mapping | warning | no | mapping 缺漏某個 catalogue operation（runtime 才會觸發）|
| `doctor.security.link_yaml.plaintext_secret` | security | error | no | link.yaml 偵測到明文密碼 / token |
| `doctor.security.link_yaml.in_git_index` | security | error | no | link.yaml 被 git index 追蹤 |
| `doctor.security.baseurl_not_https` | security | warning | no | baseUrl 非 https / localhost / unix:// |
| `doctor.config.locale_invalid` | config | warning | no | locale 不在 allowlist |
| `doctor.config.role_unresolved` | config | warning | no | role extends 找不到 base |
| `doctor.config.schema_missing` | config | error | no | config.yaml 引用的 schema id 不存在 |
| `doctor.skill.template_drift` | skill | warning | no | installed SKILL.md body 跟 binary embedded 不一致 |
| `doctor.skill.markers_missing` | skill | warning | yes | AGENTS.md / CLAUDE.md `SPECLINK:START` markers 缺漏或損壞 |
| `doctor.skill.version_mismatch` | skill | warning | no | SKILL.md frontmatter `speclink_version` 跟 CLI version 不符 |
| `doctor.state.actor_missing` | state | warning | no | in_progress change 沒 `actor` 欄位 |
| `doctor.state.feedback_task_orphan` | state | error | yes | `feedback_tasks` 表 ↔ tasks.md HTML marker 雙向 orphan |
| `doctor.state.instance_id_missing` | state | error | no | `speclink_meta.instance_id` UUID 缺失 |
| `doctor.state.invalid_enum` | state | error | no | changes table state 欄位值不在合法 enum 內 |
| `doctor.state.db_missing` | state | error | yes | `.git/speclink/state.db` 不存在；auto-fix 跑 `speclink restore --from-artifacts`（§16.13）|
| `doctor.state.db_corrupted` | state | error | no | SQLite open 失敗 / `PRAGMA integrity_check` 失敗；engine 自動備份壞檔到 `.git/speclink/state.db.bak-<timestamp>`，user 須跑 `speclink restore --overwrite` 確認 |
| `doctor.state.db_schema_invalid` | state | error | no | `schema_migrations` checksum / version 對不上（§12.7）；指向 migration error |
| `doctor.state.artifact_missing` | state | warning | no | state.db 有 change row 但 `.speclink/changes/<id>/` 在 fs 不存在（§13.9.4）；通常表示 user 切到沒有對應 artifact 的 branch；提示切回對應 branch 或刪 change |
| `doctor.gitignore.missing_entry` | project | warning | yes | `.gitignore` 缺漏 `.speclink/link.yaml`（§14.1 唯一條目）|
| `doctor.security.env_leak_risk` | security | warning | no | 當前 process env 含 `*_TOKEN` / `*_KEY` / `*_SECRET` — 子 process 雖會 sanitize、仍提示審查 |
| `doctor.security.link_yaml.env_var_unresolved` | security | error | no | link.yaml `${VAR}` 對應 env var 不存在 |
| `doctor.provider_mapping.absolute_url_disallowed` | provider-mapping | error | no | custom mapping `path:` 含 `://` |
| `doctor.provider_mapping.host_mismatch` | provider-mapping | error | no | mapping `path` resolve 後 host 不等於 `baseUrl` |
| `doctor.provider_mapping.reserved_header` | provider-mapping | error | no | mapping 宣告保留 header（`Authorization` / `Cookie` / `X-Speclink-*`）|
| `doctor.artifacts.analyze_failed` | artifacts | error | no | aggregate 對某 change 跑 `analyze` 失敗 |
| `doctor.artifacts.validate_failed` | artifacts | error | no | aggregate 對某 change 跑 `validate` 失敗 |
| `doctor.artifacts.drift_detected` | artifacts | warning | no | aggregate 對某 change 跑 `drift` 偵測到漂移 |

### 17.6 Audit Event Code Reference

Audit event 寫入 state.db 的 `events` table（見 §19.2 `Provider::record_event`）；不對應 exit code、不是 user-facing error，但跟 error code 同 namespace 規則命名以便 grep。

| Event | Trigger | 寫入欄位 |
|---|---|---|
| `lock.stale_takeover` | acquire 時強制接管 stale lock（§12.2.4）| previous_holder_pid / previous_holder_host / previous_acquired_at / takeover_at |
| `project.created` | `init` 完成 | project_id / created_by / instance_id |
| `change.created` | `new change` 完成 | change_id / created_by / state |
| `change.state_changed` | 任何 state transition 完成 | change_id / from_state / to_state / actor |
| `change.archived` | `archive` 完成 | change_id / archived_by / spec_delta_applied |
| `change.deleted` | `change delete` 完成 | change_id / deleted_by / state_at_deletion |
| `review.approved` | `review approve` 完成 | change_id / reviewer / phase |
| `review.rejected` | `review reject` 完成 | change_id / reviewer / phase / reason |
| `config.changed` | `config set` / `config edit` / `config rules set` 任一寫入 | actor / changed_keys[] / prev_etag / new_etag |
| `discussion.deleted` | `discuss delete` 完成（即使帶 `--force`） | discussion_id / deleted_by / linked_changes[] |
| `schema.forked` | `schema fork` 完成 | source_id / target_id / forked_by |
| `auth.failed` | HttpProvider 收到 401/403、CLI/SDK 偵測明文 token 等 | provider_baseurl / reason_code（如 `expired_token` / `missing_credential`）|
| `skill.alias_applied` | `installSkills({ aliases })` 對 destructive op 應用 alias | host / canonical_op / alias / destructive_flag |

**寫入時機保證**：state-mutating operation 的 audit event 必須跟 state mutation **同一 SQLite transaction**（LocalProvider）或 server-side transaction（HttpProvider）— 見 §19.2 trait method 規範（`update_*` / `record_*` method 內部包 audit event 寫入，caller 不應分開呼 `record_event`；`record_event` 僅給 audit-only 事件如 `lock.stale_takeover` / `auth.failed` 用）。

未來新增 audit event 沿用同 `<category>.<past-tense-action>` 命名。

## 18. MVP 範圍

### 18.1 MVP 必做

**Core + CLI**：

1. ✅ Crate workspace（`speclink-core` + `speclink-cli` + `speclink-provider-local`）
2. ✅ `speclink init` + `.speclink/config.yaml` 讀寫 — `config.read` ✅ Implemented (A5)、`config.write` ✅ Implemented (A5)
3. ✅ `discuss new` / `patch` / `conclude` / `show` + discussion.md 格式
4. ✅ `new change` + state machine（6 states）
5. ✅ `instructions <artifact|step>`（含 role-aware discuss）
6. ✅ `new artifact <kind>`（含 spec multi=true）
7. ✅ `status` + artifact DAG
8. ✅ `task done` + state transition
9. ✅ `analyze` / `validate` / `drift` 三條 CLI
10. ✅ `review approve` / `reject`
11. ✅ `archive` + spec delta merge + commit sub-flow（`instructions commit`）
12. ✅ `ingest` skill 對應 CLI 配套
13. ✅ Schema fork / validate（binary built-in `spec-driven`，user 可 fork）
14. ✅ 並發 safety（SQLite WAL + 雙層 lock）
15. ✅ Doctor 指令 — 9 個檢查類別（cli / project / provider / provider-mapping / security / config / skill / state / artifacts）+ `--quick` / `--check <cat>` / `--fix` flags + 4 條 auto-fix allowlist（含 `state.db_missing` → 跑 restore）+ diagnostic levels exit code 對齊（§16.12）
16. ✅ Skill source 拆 `workflow.md` + `bindings/{bash,tool}.md` 結構（§4.3）
17. ✅ `speclink init --tools <list>` 部署 Bash binding 版本 SKILL.md
18. ✅ `speclink describe-tools --format <format>` 印 operation catalogue 各格式
19. ✅ `doc/protocol/operations.md` 完整 catalogue（手寫；codegen pipeline 延後）
19a. ✅ `speclink-analyze` skill — `analyze.run` op + AI 語意 overlay；passive trigger contract（propose 結束 + state ∈ {reviewing, ready}）；fork context
19b. ✅ `speclink-verify` skill — 純 AI QA review（3 維度：Completeness / Correctness / Coherence）；無對應 CLI op；reviewer 在 code_reviewing 之前可選用
19c. ✅ `speclink-drift` skill — `drift.run` op + conclusion-first 報告 + 互動式 next-step；dual invocation path（user 主動 OR apply Step 3d inline）

**Lifecycle fixes（P0）**：

20. ✅ `apply start` / `apply pause` 雙向 idempotency + ensure-actor 語意 + return current state（§6.2 / §16.7）
21. ✅ AI retry 規則涵蓋 `state.transition_invalid`（不盲目 retry，重讀 state 後判斷）（§12.5）
22. ✅ Synthetic feedback task HTML marker + `feedback_tasks` 表 + auto-transition 前驗證 + `tasks.feedback_task_removed` error / re-append（§6.2 / §16.7 / §17.3）
23. ✅ `review history` query 補規範；重複 reject substring 比對 warning（§16.8）
24. ✅ LocalProvider `state.db.speclink_meta.instance_id` UUID（給 audit traceability）；§13.6 誠實標明 single-RD 場景 + init 印 HttpProvider 推薦提示

**Trait fixes（P1）**：

25. ✅ Provider trait 拆 `ProviderRegistry` + `Provider`（後者 constructor bind 單一 project_id）（§19.2 / §19.2.1）
26. ✅ `Versioned<T>` wrapper + `expected_etag: Option<Etag>` 套用於所有 read-modify-write method（config / change state / artifact / discussion）（§19.2 / §19.2.2）
27. ✅ LocalProvider state.db 每張表加 `version INTEGER NOT NULL DEFAULT 1` column + UPDATE `WHERE rowid = ? AND version = ?`（§19.2.2）
28. ✅ AI retry 規則涵蓋 `state.etag_mismatch`（最多一次自動重讀 + 重寫；二次仍 fail bubble up）（§12.5）
29. ✅ HttpProvider custom mapping template 加 `$.response.headers.ETag` + `If-Match: $expected_etag` 標準語法（§19.4.2）

**Concurrency 邊角（Topic 5）**：

30. ✅ Lock 階層 + 取得順序明文（global → per-change，杜絕 deadlock）（§12.2.2）
31. ✅ Lock 檔 schema（pid / host / acquired_at / operation / instance_id）（§12.2.3）
32. ✅ Stale lock 接管規則（pid dead + ≥ 5 min hardcode threshold；host mismatch → `lock.foreign_host`）（§12.2.4 / §12.5.6）
33. ✅ Jittered backoff curve 定型（4 次嘗試、累積最差 ~7s、uniform jitter）（§12.5.2）
34. ✅ HttpProvider HTTP status code ↔ SpecLink error code 對照（§12.6）
35. ✅ Lock vs Etag 角色分清明文（§12.8）
36. ✅ `lock.*` error code + audit event（§17.3）

**Error code 完整 reference（Topic 6）**：

37. ✅ §17.3 改 category summary + §17.4 user-facing error 完整表（含 retry semantics / exit code / thrown by / MVP-vs-deferred 標記）
38. ✅ §17.5 doctor finding code 完整表（含 default level / auto-fixable）
39. ✅ §17.6 audit event code 表（含 trigger / 寫入欄位）
40. ✅ 新增缺漏 code（`provider.unreachable` / `provider.not_supported` / `auth.required` / `auth.invalid` / `mapping.invalid` / `archive.no_active_change` / `archive.spec_merge_conflict` / `discussion.section_not_found` / `migration.failed` / `tasks.parse_failed`）

**Final audit 補完（2026-05-21）**：

41. ✅ Error envelope 結構：加 `retryable` / `hint` / `retry_after_ms` 機器可讀欄位（§17.1）；doctor finding 拆 `fix_message` / `fix_command`（§16.12.4）
42. ✅ §17.4 stale `discuss capture` → `discuss patch` 修正；§18.1 #3 同步
43. ✅ §13.5 init/link 範例移除 SharedProvider 語法、改 HttpProvider；標 [deferred]
44. ✅ §16.1 init/link/unlink MVP scope 清楚標 [deferred]；SDK init()/link()/unlink() 同步
45. ✅ §11.2.1 `${VAR}` interpolation 嚴格規範 — exact-match、`baseUrl` 完全禁、env 缺失 hard error；OS keychain 標 [deferred]
46. ✅ §19.4.2 mapping engine hard rules — path 不可 absolute URL、host 必須等於 baseUrl、保留 header 不可 override
47. ✅ §19.5 加 7 個 threat row（SSRF / supply-chain / env-leak / alias / stale-lock spoof / AI auto --force / mapping 安全）+ §19.5.1 child process env sanitize 規範
49. ✅ §6.2 `feedback_tasks` 表加 `reason_hash` + partial unique index；auto-transition 跟 state mutation 同 transaction 規範
50. ✅ §6.2.1 Ingest 退回 state cleanup spec（feedback_tasks clear、actor 清空、review 標記）；archived → 任何 拒絕
51. ✅ §12.5.2 加 aggregate retry budget（`SPECLINK_RETRY_BUDGET_MS` 預設 20s）
52. ✅ §12.6 HTTP status 完整對應 — 補 401/403/404/408/423/429/500/502/504 + `Retry-After` + network-level error + redirect 規則
53. ✅ §12.7 Schema migration flow（forward-only / global lock 30s / checksum 驗證 / no downgrade）+ `migration.version_too_new` / `migration.checksum_mismatch`
54. ✅ §19.2 trait 補 `write_tasks_md` / `analyze_change` / `validate_change` / `drift_change` / `archive_change` / `ingest_change` / `fork_schema` method
55. ✅ §21.4 命名映射規則 + §21.5 35 ops × 5 surface 完整 mapping 表；destructive flag 標記
56. ✅ §22.2 Layer 1 預設 curated 12 ops + categories/phases filter；SDK init/link MVP scope 清楚
57. ✅ §16.0 全域 CLI 規範（命名規則 / 全域 flag / query verb 區分 / skill alias）；`--json` 強制全域；`speclink propose` etc. 印 skill alias hint
58. ✅ §16.2 describe-tools format 加 `copilotkit` + `text`
59. ✅ §19.4.4 TLS / mTLS / cert pinning 規範 [deferred]；§19.5 redirect 規則
60. ✅ §11.8 i18n scope（machine 英文、artifact body locale）
61. ✅ §20.7 `uninstall` 加雙重 confirm（`--force` + `--confirm-project`）；標 `delivery: cli_only`
62. ✅ §22.4 aliases destructive 加 audit event `skill.alias_applied`；canonical_op 回傳機制
63. ✅ §16.6 `new artifact --force` → `--overwrite`（rewrite 語意），`--force` 保留給 destructive ops；5 個 skill draft 同步
64. ✅ Skill drafts task done arg 順序對齊 §16.7（`<task-id> --change <name>`）；apply pause ready 行為改為 idempotent

**Walking skeleton slice-A 出貨（2026-05-22，add-change-and-artifact-io）**：

64a. ✅ `change.create` / `change.list` / `change.show` / `change.delete` — CLI shipped via `speclink new change` / `list --changes` / `show change` / `delete change --confirm-name`
64b. ✅ `artifact.write` / `artifact.read` — CLI shipped via `speclink new artifact <kind> --stdin` / `artifact read <kind>`，sha256 etag 並發控制完整覆蓋 spec example concurrency matrix 5 列
64c. ✅ `spec.list-in-change` — CLI shipped via `speclink list --specs --change <name>`，新 op、待補進 §21.5 catalogue
64d. ✅ state.db v2 migration（新增 `change` 表）；`LocalProjectStore::open_state_db()` 統一 `migrate(2)`；bootstrap staging 同步升 v2
64e. ✅ `ProviderError` 加 7 個 variant + 7 個 `pub const` error code；`RuntimeError` 同步加 7 個 variant 與 exit code 對照；`output::error_code_to_exit` 兩處對照表保持一致

**SDK（npm package）**：

65. ✅ `@speclink/client` 薄版（spawn CLI subprocess）— **Tier 3 Bundled Provider**（§22.1）
    - `SpecLink` class + constructor / `fromEnv` / `fromConfig` / `fromWorkspace`
    - typed methods（`changes`, `artifacts`, `tasks`, `review`, `archive`, ...）對應 catalogue
    - `getCopilotSdkTools()` / `getCopilotKitActions()` / `getOpenAITools()` / `getLangChainTools()`
    - `installSkills(dir, { host, aliases })`
    - `loadSkillPrompt(name)` / `loadAllSkills()`
    - Layer 2 `filter` / `decorate` / `hooks` options
    - Layer 3 `schemas.<op>` + TypeScript types exports

66. ✅ `@speclink/client/helpers` sub-entry — **Tier 1 Helpers-only**（§22.1、§22.3）
    - `schemas.*`（35 ops JSON Schema 物件）
    - `installSkills(dir, opts)` / `loadSkillPrompt(name)` / `loadAllSkills()`
    - `makeToolDescriptors(opts)` 通用 + `makeCopilotSdkToolDescriptors` / `makeCopilotKitActionDescriptors` / `makeOpenAIToolDescriptors` / `makeLangChainToolDescriptors`（純 descriptor 產生器、不含 dispatcher）
    - TypeScript types exports（`ChangesCreateArgs` 等）
    - **零 Provider 依賴**：載入這個 sub-entry 不需要 `new SpecLink()`、不需要 provider config；caller 完全自管 engine logic 與 storage

67. ✅ `Provider` trait TypeScript interface export（from main entry）— **Tier 2 BYO Provider** trait surface
    - `Provider` interface 對應 §19.2 method signature
    - `Versioned<T>` / `Etag` / `CreateChangeRequest` / `WriteArtifactRequest` 等 trait-related types
    - MVP 只 export type 定義；in-process engine 接受 caller-impl `Provider` 屬 [deferred]（見 §22.1 Tier 2 MVP 限制）
    - Tier 2 MVP 實際可行路徑 = HttpProvider custom mode（Tier 2 via HTTP boundary）；in-process impl 等 HttpProvider 完成後再開

**State storage layout（P0，2026-05-22）**：

68. ✅ Two-root layout — `.speclink/` artifacts（git tracked）+ `.git/speclink/` state（git 自然不追蹤 `.git/`）；`git rev-parse --git-common-dir` path resolution；worktree / submodule 自動共用（§13.9）
69. ✅ `project.requires_git` 強制 git 專案：LocalProvider 偵測非 git → init 拒絕（§13.9.2 / §13.1 / §16.1 / §17.4）；non-git fallback 標 [deferred]
70. ✅ `.gitignore` 簡化為單行 `.speclink/link.yaml`（§14.1）；`speclink init` 自動 idempotent append
71. ✅ `speclink restore --from-artifacts` CLI — 從 `.speclink/changes/*/` 重建 state.db、自動備份壞檔、寫 markdown 報告 + JSON envelope；doctor `state.db_missing` auto-fix 對接（§16.13 / §16.12.3）
72. ✅ `change.artifact_missing` cross-branch detection — state.db change row ↔ filesystem 雙向 cross-check；**op 一律 reject**、不 fallback；doctor `state.artifact_missing` warning（§13.9.4 / §17.4 / §17.5）
73. ✅ Doctor finding 補 `state.db_missing` / `state.db_corrupted` / `state.db_schema_invalid` / `state.artifact_missing` / `project.requires_git`（§17.5）+ instance_id rotation 行為明文（§13.6）
74. ✅ Lock 位置改 `.git/speclink/locks/{global.lock, changes/<id>.lock}`（§12.2.1 / §12.2.3）
75. ✅ Migration LocalProvider → HttpProvider stub（§13.10，標 [deferred]、保證個人 → 團隊路徑可走）

### 18.2 Deferred

| 能力 | 為什麼延後 |
|---|---|
| HttpProvider impl（`speclink-provider-http` crate） | LocalProvider 完成後再做；trait skeleton + threat model 已在 §19 預留 |
| HttpProvider custom mode（HTTP template mapping） | 同上 |
| `speclink auth login/logout/status` | 跟 HttpProvider 一起延後 |
| `speclink provider add/list/use`（user-global alias） | 同上 |
| Unix socket transport for HttpProvider | 同 protocol 不同 transport；HTTP 路徑跑起來再加 |
| Python / Go client | TypeScript client 跑起來、看 adoption signal 再加 |
| `@speclink/client` codegen pipeline | MVP 階段 client 手寫；catalogue 跑穩後再加 codegen |
| `speclink mcp serve` | MCP server mode；MVP 後加 |
| 跨機器 sync / pack / unpack | 個人本機開發無需求 |
| Spectra 雙向 import / export | 不對齊 spectra-cli，無相容需求 |
| Vector search / documents 知識庫 / spectra-ask clone | 非 SDD 核心 |
| Feedback network command | 純產品 telemetry，個人專案不需要 |
| AI tool 多元支援（cursor / windsurf / cline / …） | MVP 只支援 claude / codex / github-copilot；其他延後 |
| `speclink completion` | 對 workflow 無必要，純舒適性指令 |
| Worktree / isolated branch | 對個人 + multi-agent 同機器無實際需求 |
| `speclink init --no-git` fallback（`.speclink/state.db` + gitignore）| LocalProvider 強制 git 是 MVP 規定（§13.9.2）；non-git fallback 對個人 RD workflow 罕見、未來真有需求再加 |
| `speclink list --all-projects` cross-project state | 需要 user-global registry 機制；個人 RD MVP 不痛 |
| `speclink change prune --state proposing --older-than <duration>` | proposing backlog 過期清理；MVP 不痛 |
| `speclink export <file>` / `speclink import <file>` | state.db 顯式 backup / 跨機器搬遷；MVP 用 `restore --from-artifacts` 已足夠（§16.13）|
| Schema user-global 共享（`~/.speclink/schemas/`）| 跨 project schema reuse；MVP 每 project 各自 fork 即可 |
| Worktree-isolated state（`.git/speclink/.worktree-ref` marker）| `.git/speclink/` 透過 git common dir 自動跨 worktree 共用（§13.9.4）；未來若有「想隔離 per-worktree state」反向需求才加 |
| `speclink migrate-to-http` | 跟 HttpProvider impl 一起延後（§13.10）|

### 18.3 明確 Non-Goals（不只是 deferred，是不打算做）

1. 與 spectra-cli 命名相容
2. 與 openspec 配置相容
3. Web UI（讓別人建，我們只 ship library + CLI）
4. 多人即時協作 lock（cross-machine）
5. SaaS 後端、telemetry
6. 與所有 Git provider 深度整合

## 19. Provider Trait

### 19.1 抽象軸：兩類 provider

Provider trait 按 **CLI / SDK 跟儲存講話的方式** 分類。Agent Host × Provider × Delivery surface 三維正交（見 §3.2、§3.3）。

| Provider | Transport | 適用情境 | MVP 狀態 |
|---|---|---|---|
| `LocalProvider` | filesystem + embedded SQLite（CLI 獨佔） | 個人 RD、SDD 狀態跟 git 走動（spectra-style）| **實作** |
| `HttpProvider` | HTTP / HTTPS | 規格縱覽 webapp、團隊共用、跨組織、SaaS — 任何想透過 server 集中管理規格的場景 | trait skeleton, no impl |
| Caller-impl（BYO） | TS / Rust trait impl | webapp 已有自家 DB schema、走 SDK Tier 2（§22.1） | trait export only；in-process engine [deferred] |

**廢除說明**：之前曾設計 `SharedProvider`（CLI 直連 external DB）跟 `RemoteProvider`（HTTP）兩類分離。2026-05-21 收斂為單一 `HttpProvider`，理由：

1. **Raw SQL custom mapping 維護地獄** — DB dialect / schema 耦合 / cross-cutting concern 重複實作；放棄。
2. **Loopback「繞圈」實測 sub-ms** — webapp server 上 CLI 走 localhost HTTP 到自家 backend 的開銷可接受；如果未來真有極速需求，加 Unix socket transport（同 protocol 不同 transport，Docker daemon pattern），仍歸 HttpProvider 範疇。
3. **「webapp 已有自己 schema 不想改」case** 改走 HttpProvider custom mode + 宣告式 HTTP template 映射（Hasura Remote Schema 風格），業界 pattern。

底層儲存技術（SQLite / Postgres / MySQL / 任何）是各 provider impl 的選擇，**不是 provider 分類本身**。

#### 19.1.1 Engine vs Provider 責任劃分

理想的乾淨切法：

```
┌─────────────────────────────────────────────────────────┐
│ Engine（speclink-core）= pure logic                     │
│  - State machine（6 state transitions、ingest 退回）     │
│  - Validation（proposal/spec/tasks 完整性）              │
│  - Drift detection（spec vs implementation）              │
│  - Lock acquisition orchestration（§12 階層）            │
│  - Audit event emission（state mutation 同 txn）         │
│  - Operation catalogue dispatch（§21）                   │
│  - Skill body 載入（embedded templates、§20.6）          │
│  - Schema fork/validate（§10、§16.11）                   │
│  - JSON envelope shaping（§17.1）                        │
│  - Jittered backoff / retry budget（§12.5.2）            │
│  - Etag conflict guidance（§12.5.4、§12.8）              │
└─────────────────────────────────────────────────────────┘
                          ▼ 透過 Provider trait（純 CRUD）呼
┌─────────────────────────────────────────────────────────┐
│ Provider = pure storage                                 │
│  - CRUD entity（change / artifact / discussion / config）│
│  - Atomic transaction                                    │
│  - Etag 產生（write-time monotonic）                     │
│  - Lock primitive（file lock / row lock / If-Match）     │
│  - Event log persist                                     │
└─────────────────────────────────────────────────────────┘
```

**意義**：所有業務邏輯不需要知道資料存哪裡；Provider 不需要知道業務規則。Tier 2 BYO Provider（§22.1）只需 impl 純 CRUD 即可拿到完整 SpecLink engine logic。

**目前 design 的偏離**：§19.2 Provider trait 仍含 `analyze_change` / `validate_change` / `drift_change` / `archive_change` / `ingest_change` / `fork_schema` / `write_tasks_md` 這些**邏輯 method**。當初擺在 Provider 是因為「HttpProvider custom mode 下、webapp 也許想 server-side 跑」，但這把責任邊界搞糊。

**收斂計畫（post-MVP design debt）**：

| Method | 現狀 | 收斂後 |
|---|---|---|
| `analyze_change` / `validate_change` / `drift_change` | Provider | Engine method，組合 `get_change` + `read_artifact` 純 local 計算 |
| `archive_change` / `ingest_change` | Provider | Engine method，組合 state transition + artifact write + audit |
| `fork_schema` | Provider | Engine method，組合 `list_schemas` + write 新 schema entry |
| `write_tasks_md`（synthetic feedback） | Provider | Engine method，組合 `read_artifact` + `write_artifact` |

收斂後 Provider trait 大幅瘦身（剩 ~12 個純 CRUD method），HttpProvider impl 工作量明顯降低，Tier 2 BYO 門檻也降。

**為什麼 MVP 不做這個收斂**：HttpProvider 本身 MVP `[trait skeleton only]`、Tier 2 BYO MVP `[deferred]`，所以這個 design debt 對 MVP 的 LocalProvider 唯一實作沒有 user-facing 影響。等 HttpProvider 正式做時一起重切，避免「現在改 trait、之後又改」雙重成本。標記為 [P1 post-MVP design debt]。

### 19.2 共通 Trait Method

Trait 拆兩層：

- **`ProviderRegistry`** — 跨 project 的探索與註冊（init / link 互動式列表 / 透過 `project_id` 開出單 project 的 `Provider`）。
- **`Provider`** — 單一 project 操作 instance（construction 時就綁定一個 `project_id`、所有 method 都對該 project，**不再每個 method 傳 project_id**）。

`Versioned<T>` wrapper + `expected_etag: Option<Etag>` 統一處理 optimistic concurrency；details 見 §19.2.1。

```rust
pub type Etag = String;

pub struct Versioned<T> {
    pub value: T,
    pub etag: Etag,
}

// ── ProviderRegistry：跨 project 的 pre-binding 操作 ──
pub trait ProviderRegistry {
    fn capabilities(&self) -> Capabilities;

    // Project lifecycle（§13.4 / §13.5）
    fn register_project(&self, request: RegisterProjectRequest) -> Result<ProjectId>;  // init 用
    fn list_projects(&self) -> Result<Vec<ProjectSummary>>;                              // link 互動式 list 用

    // 拿單 project 的 Provider instance
    fn open(&self, project_id: &str) -> Result<Box<dyn Provider>>;
}

// ── Provider：單一 project 操作 instance ──
pub trait Provider {
    fn project_id(&self) -> &str;
    fn capabilities(&self) -> Capabilities;

    // Config（§11.5 — 物理位置由 provider 決定）
    fn read_config(&self) -> Result<Versioned<Config>>;
    fn write_config(&self, request: WriteConfigRequest) -> Result<Versioned<Config>>;

    // Change
    fn create_change(&self, request: CreateChangeRequest) -> Result<Versioned<Change>>;
    fn list_changes(&self, filter: ChangeFilter) -> Result<Vec<ChangeSummary>>;
    fn get_change(&self, id: &str) -> Result<Versioned<Change>>;
    fn update_change_state(&self, request: UpdateChangeStateRequest) -> Result<Versioned<Change>>;

    // Artifact
    fn read_artifact(&self, request: ReadArtifactRequest) -> Result<Versioned<Artifact>>;
    fn write_artifact(&self, request: WriteArtifactRequest) -> Result<Versioned<Artifact>>;

    // Discussion
    fn list_discussions(&self, filter: DiscussionFilter) -> Result<Vec<DiscussionSummary>>;
    fn get_discussion(&self, id: &str) -> Result<Versioned<Discussion>>;
    fn write_discussion(&self, request: WriteDiscussionRequest) -> Result<Versioned<Discussion>>;

    // State（lifecycle、review、task）
    fn record_review(&self, request: RecordReviewRequest) -> Result<()>;
    fn record_task_done(&self, request: TaskDoneRequest) -> Result<Versioned<Change>>;
    fn write_tasks_md(&self, request: WriteTasksRequest) -> Result<()>;     // synthetic feedback re-append、ingest cleanup

    // Engine 內建分析
    fn analyze_change(&self, id: &str) -> Result<AnalysisReport>;
    fn validate_change(&self, id: &str, strict: bool) -> Result<ValidationReport>;
    fn drift_change(&self, id: &str) -> Result<DriftReport>;
    fn archive_change(&self, request: ArchiveRequest) -> Result<ArchiveResult>;
    fn ingest_change(&self, request: IngestRequest) -> Result<Versioned<Change>>;

    // Schema
    fn list_schemas(&self) -> Result<Vec<SchemaSummary>>;
    fn get_schema(&self, id: &str) -> Result<Schema>;
    fn fork_schema(&self, request: ForkSchemaRequest) -> Result<()>;

    // Audit / event log（caller 不該對 state-mutating event 直接呼；那些走對應 method 內隱含寫入）
    fn record_event(&self, event: Event) -> Result<()>;
}

// ── 帶 expected_etag 的 write request ──
pub struct WriteConfigRequest {
    pub config: Config,
    pub expected_etag: Option<Etag>,  // None = blind overwrite；Some = If-Match
}

pub struct UpdateChangeStateRequest {
    pub change_id: String,
    pub new_state: ChangeState,
    pub expected_etag: Option<Etag>,
}

pub struct WriteArtifactRequest {
    pub change_id: String,
    pub artifact_id: String,
    pub content: String,
    pub expected_etag: Option<Etag>,
}

pub struct WriteDiscussionRequest {
    pub discussion_id: String,
    pub patch: DiscussionPatch,
    pub expected_etag: Option<Etag>,
}
```

`speclink-core::Engine<R: ProviderRegistry>` 啟動時走 `registry.open(project_id)` 拿 `Box<dyn Provider>`，後續操作對 `Provider` 跑；對 Local / Http 實作無差別。

#### 19.2.1 為什麼拆兩層

| 問題 | Single-trait 痛點 | Split-trait 解法 |
|---|---|---|
| `register_project` / `list_projects` 跨 project，其他 method 又是 single-project | 同一 trait 內語意混雜，caller 不知道哪個 method 對哪個 project 操作 | Registry 跨 project；Provider instance 構造後綁定一個，method signature 乾淨 |
| Webapp serving N projects 每次都要傳 `project_id` | method-level param 每個 call site 重複 | instance cache `Map<ProjectId, Box<dyn Provider>>`，request 只查一次 |
| LocalProvider 物理上就是 `.speclink/` per project | 1:1 自然映射 | constructor bind 對齊現實 |

**Webapp instance cache pattern** 範例見 §22.6。

#### 19.2.2 Optimistic concurrency（Versioned<T> + expected_etag）

所有 **read-modify-write** 操作走統一模式：

```
1. read_X()        → Versioned<T> { value, etag: "v42" }
2. caller 改 value
3. write_X(WriteXRequest { value: new, expected_etag: Some("v42") })
4a. provider 比對：current etag == "v42" → 通過、回新 Versioned<T> { etag: "v43" }
4b. 不符      → error code `state.etag_mismatch`、附帶當前 etag
```

**Etag 來源**：

| Provider | Etag 實作 |
|---|---|
| LocalProvider | state.db 內每個 row 加 `version INTEGER NOT NULL DEFAULT 1` column；每次 update 走 `WHERE rowid = ? AND version = ?`；string 化為 `"v<n>"` |
| HttpProvider | 透傳 HTTP `ETag` response header → 下次 `If-Match: <etag>` request header；server 端負責生成（content hash / sequence / whatever） |

**Blind overwrite**：caller 傳 `expected_etag: None` 表示「不管當前版本、強制寫」。MVP 階段保留語意但慎用 — 預設應該都帶 etag。CLI 大部分情境（`speclink config edit` 是先讀後寫、`task done` 先讀 state 再 transition）都自然帶。

**Etag 衝突 retry**：見 §12.5 — 跟 `state.transition_invalid` 同類，**不盲目 retry、重讀後判斷**。

### 19.3 LocalProvider（MVP 唯一實作）

- 底層：filesystem + SQLite + WAL + advisory file lock
- **Storage layout 兩個 root**（§13.9）：
  - artifacts root：`.speclink/`（git tracked、跟 working dir 走）
  - state root：`.git/speclink/`（在 git common dir 內、git 自然不追蹤、跨 worktree / 跨 branch 共用）
  - Path resolution：`git rev-parse --git-common-dir`（自動處理 worktree / submodule）
- Identity：working dir 內的 `.speclink/`；`project.id` 從 `.speclink/config.yaml#project.id` 取
- `link.yaml` 可省略（缺檔 = 預設 local）
- **Require git**：非 git 專案 → `project.requires_git` error（§13.9.2 / §17.4）；MVP 不支援 `--no-git` fallback
- 並發模型：§12 已詳述（lock 路徑 `.git/speclink/locks/`）

### 19.4 HttpProvider（trait skeleton only, MVP 不實作）

底層：HTTP / HTTPS（未來可加 Unix socket transport，同 protocol 不同 transport）。

**兩個 sub-mode**：

#### 19.4.1 Default mode — webapp 實作 SpecLink 標準 protocol

```yaml
# .speclink/link.yaml
provider: http
baseUrl: https://specs.team.internal
auth: ${SPECLINK_TOKEN}
project_id: billing-system
```

Webapp server 實作 **SpecLink 標準 HTTP API**（endpoints 規格見未來 `doc/protocol/default-http-api.md`）。CLI / SDK 不必額外設定。

對應 use case：user 願意 adopt SpecLink 規格 + 從零開始蓋 webapp（或已有 webapp 願意加 SpecLink endpoints）。

#### 19.4.2 Custom mode — 對應到 webapp 既有 endpoints

```yaml
# .speclink/link.yaml
provider: http
baseUrl: https://my-webapp.example.com/api
auth: ${SPECLINK_TOKEN}
project_id: billing-system
customMapping: ./speclink-mapping.yaml   # 或 inline
```

`speclink-mapping.yaml` 是 **宣告式 HTTP template**（Hasura Remote Schema 風格、業界 pattern）：

```yaml
# speclink-mapping.yaml — 宣告式 operation → HTTP request 映射
mappings:
  change.create:
    method: POST
    path: /specs
    body:
      spec_id: $change_id
      title: $name
      summary: $description
      project_id: $project_id
    response:
      change_id: $.spec_id
      created_at: $.created_at
  
  change.state_change:
    method: PATCH
    path: /specs/{change_id}/status
    body:
      status: $new_state
    headers:
      If-Match: $expected_etag
    response:
      etag: $.response.headers.ETag

  change.get_by_id:
    method: GET
    path: /specs/{change_id}
    response:
      change_id: $.spec_id
      name: $.title
      description: $.summary
      state: $.status
      etag: $.response.headers.ETag
```

**Template 語法**：
- `$field` — operation params 內的欄位（body 或 path param）
- `$.field` — response JSON path 抽取（response 物件 → 標準 operation result 欄位）
- `$.response.headers.X` — 抽 HTTP response header 進 operation result（如 `ETag` → `Versioned<T>.etag`）
- `{change_id}` — path placeholder 由 `$change_id` 自動填入
- `If-Match: $expected_etag` — optimistic locking 走標準 HTTP semantics、對應 trait 的 `expected_etag` 欄位（見 §19.2）

**Mapping 安全 hard rules**（mapping engine MUST enforce）：

| 規則 | 違反 → |
|---|---|
| `path:` 不得為 absolute URL（不含 `://`） | `mapping.invalid` + finding `doctor.provider_mapping.absolute_url_disallowed` |
| `path:` resolve 後的 final URL host **必須等於 `baseUrl` host** | reject + `mapping.host_mismatch`；防 user mapping 把 request 送去他處 |
| `headers:` 不得宣告 `Authorization` / `Cookie` / `X-Speclink-*`（保留 namespace）| `mapping.reserved_header` + finding `doctor.provider_mapping.reserved_header` |
| `auth: ${SPECLINK_TOKEN}` 自動注入為 `Authorization: Bearer <token>` header | 注入時機在 user-defined headers **之後**、且只在通過上面 host check 才注入 |
| `$var` / `$.path` interpolation 結果 URL-encode 進 path、JSON-encode 進 body | 防 path traversal / JSON injection |
| Mapping YAML 不得 reference 任意 file（如 `!include`）| YAML loader 走 safe mode |

`doctor --check-mapping` 必須對每條 mapping 跑全部規則 dry-run；失敗 → finding `doctor.provider_mapping.*`（見 §17.5）。

**安全特性**：
- 純資料 template、無 code execution、無 SQL injection、無 SSRF（mapping engine enforce 上述 rules）
- DB-agnostic 完全
- Webapp 既有 middleware（auth / audit / validation）自然套上
- Core 不需綁任何 DB driver

對應 use case：webapp 已有自家 schema + endpoints，不想配合 SpecLink schema 重蓋。

#### 19.4.3 共通設計

- Identity：`.speclink/link.yaml#project_id` + server 內部紀錄
- Auth：HTTP 標準（Bearer token / mTLS / OIDC）— MVP 階段預留設計、選 token 為預設
- 並發 / lock：HTTP-level optimistic locking 走 `If-Match: <etag>` header（對應 trait 的 `Versioned<T>.etag`，見 §19.2.2）；server 端負責 transaction
- **Loopback 場景**：webapp server 上 CLI / SDK 走 localhost HTTP 到自家 backend，sub-ms overhead；未來可加 Unix socket transport（`baseUrl: unix:///var/run/speclink.sock`）走同樣 protocol

#### 19.4.4 TLS 規範

**強制 HTTPS**（§19.5 已寫）— `baseUrl` 必須 `https://` / `localhost` / `127.0.0.1` / `unix://` 之一。HTTP 路徑 → `mapping.invalid` error + 建議升級。

**Cert 驗證**：

| 情境 | MVP 行為 | Deferred |
|---|---|---|
| 公開 CA 簽的 server cert | ✓ 走 system trust store（OS 預設）| n/a |
| Self-signed cert | ✗ reject（runtime + doctor finding `doctor.security.tls_handshake_failed`）| [deferred] `link.yaml` 加 `tls.ca_cert: <path>` 指 user-provided CA bundle |
| mTLS client cert | [deferred] | `link.yaml` 加 `tls.client_cert` / `tls.client_key`（paths 或 `${VAR}`）|
| Cert pinning（pin specific cert hash）| [deferred] | `link.yaml` 加 `tls.pinned_cert: sha256:<hash>` |

**Redirect 規則**（已於 §12.6 簡述、此處正式）：

- HTTP → HTTPS downgrade redirect：拒絕
- HTTPS → HTTPS 跨 host redirect：拒絕（`mapping.host_mismatch`）
- HTTPS → HTTPS 同 host redirect：follow（上限 3 跳）

**doctor `--check-mapping --live`** 不 follow 任何 redirect（直接探 baseUrl 本身）；TLS handshake 失敗 fire `doctor.security.tls_handshake_failed` 不繼續 dry-run。

### 19.5 安全 Threat Model

HttpProvider 的安全責任**主要在 server 端**（access control / authn / authz / audit）；CLI / SDK 是 client。

| Threat | 風險點 | Mitigation |
|---|---|---|
| **明文密碼 / token 進 link.yaml** | credential 易誤洩漏（log、screenshot、git push 意外） | engine 偵測明文 secret 回 **error**（非 warning）；強制 env var / OS keychain reference |
| **link.yaml 被誤 push 到 git** | .gitignore 預設排除，但人為 force-add 可能發生 | gitignore 預設、`speclink init` 印 reminder；engine 啟動偵測 git index 內含 link.yaml → warning |
| **跨網路 MITM** | HTTP 明文傳輸 SDD 資料 + credential | 強制 HTTPS（baseUrl 須 `https://` 或 `unix://`、`localhost` 例外）；HTTP 路徑回 warning 並建議升級 |
| **Token replay / 永久 token** | 一次洩漏永久有效 | 建議 short-lived（OIDC ID token、IAM-issued、Vault dynamic）；trait 預留 token refresh hook |
| **Server 端 admin tamper config**（shared 場景） | 任何能寫 server config 的 client 都能改 `rules.*`、`roles.*`，影響全團隊 SDD 流程 | server 端做 admin scope 控制；CLI/SDK 層 `speclink config set` 需明確 admin scope；audit log 紀錄誰改了什麼。**MVP trait 預留 hook，admin 機制延後** |
| **Schema fork 污染**（shared 場景） | 改 schema → 全團隊 artifact 結構跟著變 | schema 改動視同 config 改動，需 admin scope + audit |
| **Audit / accountability** | 多 user 同 token → 不知誰幹的 | application-level event log 紀錄 `actor`（agent host id + OS user + project id）；server 端 token 應 per-user 簽發 |
| **Custom mapping 失敗難 debug** | mapping 寫錯只能 runtime 報錯 | `speclink doctor --check-mapping` dry-run 每個 operation 驗證 mapping shape；engine init 時試打一次 health endpoint |
| **Mapping SSRF / token exfiltration** | user mapping 把 request 送去 `baseUrl` 以外 host、把 auth header 改寫 | §19.4.2 mapping engine hard rules（host match / reserved header / `path:` 不可 absolute URL）；doctor `--check-mapping` 強制偵測 |
| **`${VAR}` interpolation 濫用** | `baseUrl` 含 `${ENV}` → attacker 改 env var 引導 request 到他處 + 帶走 token | §11.2.1 嚴格 interpolation 規範（僅 `auth:` 接受、`baseUrl` 完全禁、exact-match 規則）|
| **Child process env leak** | CLI spawn git / editor / hook 時繼承整個 env，token 洩到子 process | CLI spawn child 時 env sanitize — 只傳 `SPECLINK_*` whitelist + `PATH` / `HOME` / `USER` / `LANG` 等必要；其餘清空（§19.5.1）|
| **Binary supply-chain tamper** | attacker 改 binary 同時改 embedded skill body + frontmatter `template_hash`，drift 偵測失效 | MVP 接受個人用 risk；deferred mitigation：signed release / SLSA attestation / `cargo install --git <repo> --rev <hash>` + verify。`template_hash` 改名為 `template_version` 加 binary-side signed manifest |
| **Skill alias confusion**（destructive op）| developer 用 `installSkills({ aliases })` 把 `change.delete` rename 成 `safe_remove_item`，AI 看 tool name 認知偏差 | catalogue 標 `destructive: true` 的 operation 套 alias 時必寫 audit event `skill.alias_applied`；dispatcher 回傳 result 內帶 canonical operation id 讓 AI cross-check |
| **Stale lock spoof**（同 host）| attacker 偽造 lock 檔 `pid` + `acquired_at` 為過去時間 + 同 host name → 強迫接管別人 lock | MVP 接受個人用 risk；deferred mitigation：lock 檔加 HMAC signature with `instance_id` 派生 key |
| **AI 自動 `--force` 觸發 destructive op** | AI hallucinate 帶 `--force` 跑 `discuss delete --force` / `uninstall --force` / `schema delete` 等 | (1) destructive ops 從 Bash binding 拿掉、只走人工 CLI（catalogue 標 `delivery: cli_only`） (2) Tool binding catalogue 標 `destructive: true` flag 讓 host 自決 (3) `--force` 帶 audit event `*.deleted` 紀錄 actor (4) skill drafts 明文「AI **永不**主動加 `--force`」 |

### 19.5.1 Child process env sanitize

CLI 在 spawn 子 process（git commit、$EDITOR、hooks 等）時必須走 env whitelist：

**Whitelist**（傳給子 process）：
- `SPECLINK_*` 全 prefix
- `PATH` / `HOME` / `USER` / `LOGNAME` / `SHELL` / `LANG` / `LC_*` / `TERM` / `TMPDIR`
- `EDITOR` / `VISUAL`（給 `config edit`）
- `NO_COLOR` / `CLICOLOR*`

**Blacklist**（**永遠**從子 process env 移除）：
- 任何含 `TOKEN` / `SECRET` / `KEY` / `PASSWORD` substring 的 env var
- `AWS_*` / `GCP_*` / `AZURE_*` / cloud credential prefix
- `OPENAI_API_KEY` / `ANTHROPIC_API_KEY` / `GITHUB_TOKEN` 等已知 LLM / VCS credential

Whitelist + blacklist 兩條同時跑：whitelist match 但 substring 命中 blacklist → 仍移除。Doctor `security` 類別加 finding `doctor.security.env_leak_risk` 偵測當前 process env 含敏感 prefix 是否會誤傳。

對 LocalProvider 安全責任較少：本地檔案系統權限 + 各 RD 自管 `.speclink/`。

## 20. Skill 部署機制（init / update）

對齊 spectra 的 init / update 設計，但簡化 MVP scope。

### 20.1 MVP 支援的部署 target

按 Delivery surface（§3.3）分兩類：

#### Bash binding hosts（CLI subprocess）

`speclink init --tools <list>` 部署 `workflow.md + bindings/bash.md` 拼接的 SKILL.md：

| Tool | Skill 路徑 | 通用入口檔案 |
|---|---|---|
| `claude` | `.claude/skills/speclink-<skill>/SKILL.md` | `CLAUDE.md` |
| `codex` | `.agents/skills/speclink-<skill>/SKILL.md` | `AGENTS.md` |
| `github-copilot` | `.github/prompts/speclink-<skill>.prompt.md` + `.github/skills/speclink-<skill>/SKILL.md` | `AGENTS.md` |

5 個 skill × 3 個 tool ≈ 15-20 個檔案。

#### Tool binding hosts（SDK 內 typed tool）

`@speclink/client` 的 `installSkills(dir, { host })` 部署 `workflow.md + bindings/tool.md` 拼接的 SKILL.md：

| Host | install host 值 | Skill 部署目標 | tool 機制 |
|---|---|---|---|
| GitHub Copilot SDK | `copilot-sdk` | 使用者指定 dir，傳給 `skillDirectories` | `defineTool` + handler 內呼 `@speclink/client` |
| CopilotKit | `copilotkit` | 使用者指定 dir + `useCopilotReadable` | `useCopilotAction` + handler 內呼 `@speclink/client` |
| OpenAI function calling | `openai` | system prompt 注入 | tool 註冊到 Assistant + handler 同上 |
| LangChain | `langchain` | system prompt 注入 | `Tool` class + handler 同上 |

Tool binding skill body 不直接寫 CLI 指令、而是寫 tool name + 參數 shape（依 §21 catalogue 的 Tool binding 欄）。Tool name 可被 `installSkills` aliases option 覆寫（見 §22.4 Layer 3）。

### 20.2 SKILL.md 結構

```markdown
---
name: speclink-discuss
description: Have a focused, role-aware discussion that produces an iterative discussion document.
speclink_version: 0.1.0
template_hash: sha256:<hash of body>
---

# /speclink-discuss

<skill 內容 — AI 看的 prompt body>
```

- `speclink_version`：binary 部署當下的 CLI 版本，update 對比版本決定是否要更新
- `template_hash`：body sha256，用於 customization 偵測

### 20.3 AGENTS.md / CLAUDE.md markers

對齊 spectra `<!-- SPECTRA:START v1.0.2 -->` 設計：

```
<!-- SPECLINK:START v<ver> -->
（speclink 自動產生的內容，每次 update 覆寫）
<!-- SPECLINK:END -->
```

**注入規則：**
- markers 以外的內容**永遠保留**
- markers 以內的內容**每次 update 覆寫**
- markers 不存在時，`init` append 一段；`update` fail 並要求 `init --force`
- AGENTS.md 用 `$speclink-<name>` 風（codex convention）、CLAUDE.md 用 `/speclink-<name>` 風（Claude Code convention） — engine 依目標檔案做語法替換

### 20.4 `speclink init` 流程

```bash
speclink init <project-name> [--force] [--tools claude,codex,github-copilot] [--path <root>]
```

```
1. Pre-check  → .speclink/ 已存在 → 沒 --force 就拒絕（error: project.already_initialized）
2. Build state → 建 .speclink/{config.yaml, state.db}（state.db 含初始 schema_migrations）
3. Update .gitignore → 加 .speclink/state.db、.speclink/state.db-wal、.speclink/lock、.speclink/touched/
4. Write config → config.yaml#tools 紀錄 --tools 選擇
5. Deploy skills → 依 --tools 部署到對應路徑
6. Inject markers → 寫 AGENTS.md / CLAUDE.md（存在就 markers-in，不存在就建檔）
7. Print summary
```

`--tools` 預設值 `claude`；可指定多個 comma-separated。

### 20.5 `speclink update` 流程

```bash
speclink update [--force] [--check]
```

```
1. Verify        → .speclink/ 存在；讀 config.yaml#tools
2. Read installed → 每個 SKILL.md 的 frontmatter（speclink_version、template_hash）
3. Detect changes → body sha256 vs frontmatter template_hash
4. Compare bin   → binary bundled speclink_version vs installed
5. --check       → 列出所有 diff、不寫
6. 無 customization OR --force → 直接覆寫
7. 有 customization 且沒 --force → exit code 7、error skill.customized
8. Re-inject markers → AGENTS.md / CLAUDE.md（markers 內部覆寫）
```

### 20.6 模板來源

Skill templates 內嵌進 binary（以 `include_str!` 或類似機制）。Binary 內結構：

```
embedded/
  skills/
    propose/
      claude.md       # claude SKILL.md body
      codex.md        # codex SKILL.md body（多數情況同 claude）
      copilot.md      # github copilot prompt.md body
    apply/
      ...
    archive/
      ...
    ingest/
      ...
    discuss/
      ...
  markers/
    agents.md         # AGENTS.md markers 內容
    claude.md         # CLAUDE.md markers 內容（同 agents.md，僅指令前綴 / vs $ 差異）
```

實作上多數 tool 內容可共用，差異僅 frontmatter 與檔名。

### 20.7 解除安裝

```bash
speclink uninstall [--force] [--confirm-project <name>]
```

```
1. 警告：將刪除 .speclink/ 全部內容（含 state.db 與 changes、不可復原 audit log）
2. 需 BOTH --force AND --confirm-project <project-name>（必須匹配 config.yaml#project.id）
   - 雙重確認防 AI / script 誤觸
   - 互動式（TTY）模式下 --confirm-project 缺失 → 互動 prompt 要 user 輸入完整 project name
   - 非 TTY（CI / SDK）必須兩 flag 都帶；缺一回 cli.requires_tty 並提示
3. 移除 .claude/skills/speclink-*、.agents/skills/speclink-*、.github/prompts/speclink-*.prompt.md
4. 移除 AGENTS.md / CLAUDE.md 的 SPECLINK markers（保留外圍內容）
5. 不動 .gitignore（使用者自行決定）
```

**AI skill drafts 永不**自動跑 `uninstall`；catalogue 標 `delivery: cli_only` 代表 Bash binding 也不出現此 op、Tool binding 不暴露。只有人類在終端機手動下指令。

## 21. Operation Catalogue（single source of truth）

SpecLink 的所有 surface — CLI subcommands、`describe-tools` 輸出、`@speclink/client` typed methods、MCP server tool 定義、skill bindings 內 invocation 語法 — 全部 **derive from operation catalogue**。

### 21.1 角色

Operation catalogue 是 `doc/protocol/operations.md`（MVP 階段手寫；之後 codegen pipeline）。每個 operation 紀錄：

| 欄位 | 範例 |
|---|---|
| **Canonical ID** | `change.create` |
| **Inputs schema** | JSON schema：`{ name: string, description: string, ... }` |
| **Output schema** | JSON schema：`{ change_id, name, state, created_at, ... }` |
| **Semantics** | "Create a new change in proposing state. Reject if same id exists." |
| **Idempotency** | `non-idempotent` / `idempotent` / `idempotent-with-version` |
| **Lock requirement** | `none` / `change-exclusive` / `global-short` / `global-exclusive` |
| **Bash binding** | `speclink new change <name> --description "..."` |
| **Tool binding** | tool name `new_change`; handler input shape `{ name, description }` |
| **HTTP default endpoint** | `POST /api/projects/<project_id>/changes` |
| **Provider trait method** | `Provider::create_change(req)` |

### 21.2 MVP 階段的 operation 概貌（會在 operations.md 詳列）

按 skill 切分：

```
Project / config / schema
  project.init       (CLI: init, SDK: speclink.init())
  project.link       (CLI: link, SDK: speclink.link())
  project.unlink     (CLI: unlink)
  project.status     (CLI: status, SDK: speclink.status())
  config.read        (CLI: config show, SDK: speclink.config.read())
  config.write       (CLI: config set / edit, SDK: speclink.config.write())
  schema.list        (CLI: schema list)
  schema.fork        (CLI: schema fork)
  schema.validate    (CLI: schema validate)

Change CRUD + lifecycle
  change.create      (propose skill)
  change.list
  change.get
  change.delete
  change.state_change       (engine 自動，不暴露給 skill)

Artifact
  instructions.<kind>       (propose / apply / ingest / archive / discuss / commit)
  artifact.write
  artifact.read

Apply / Task
  apply.start
  apply.pause
  task.done
  task.add                  (engine 自動，feedback task 用)

Review
  review.approve            (--phase artifact | code)
  review.reject             (--phase artifact | code, --reason required)
  review.history

Archive
  archive.run               (spec delta merge + transition)
  instructions.commit       (commit sub-flow)

Spec (canonical capability spec — archived merge target)
  spec.list                 (列舉所有 capability)
  spec.show                 (讀單一 capability 完整 spec)

Engine analysis
  analyze.run
  validate.run
  drift.check
  doctor.run

Discussion
  discuss.new
  discuss.list
  discuss.show
  discuss.patch (section)
  discuss.conclude
  discuss.delete
```

預計 ~37 個 operation（含 `spec.list` / `spec.show` 等 read-only / metadata ops）。

### 21.3 衍生機制

```
operations.md
  │
  ├──► CLI subcommand parser (clap，每個 op 一個 subcommand)
  ├──► describe-tools output (multi-format)
  ├──► @speclink/client typed methods + schemas + types
  ├──► MCP server tool registrations
  ├──► Skill bindings/{bash,tool}.md 內的 invocation syntax
  └──► doc/protocol/ 自動生成 reference docs
```

MVP 階段這些都是手寫（單一 source 文件 + 手動同步各 surface）；之後加 codegen pipeline 防 drift。

### 21.4 命名映射規則

Catalogue ID `<noun>.<verb>` 在不同 surface 對應規則：

| Surface | 規則 | 例 |
|---|---|---|
| **Catalogue ID** | `<noun>.<verb>` 點分隔；snake_case | `change.create` / `task.done` / `apply.start` / `discuss.patch` |
| **CLI (Bash binding)** | `speclink <noun> <verb>`；hyphen-case | `speclink change new` / `speclink task done` / `speclink apply start` |
| **Tool binding** | `<verb>_<noun>` 或 `<noun>_<verb>`（依 catalogue 標記）；snake_case | `new_change` / `task_done` / `apply_start` |
| **SDK method** | `speclink.<noun_plural>.<verb>()`；camelCase namespace | `speclink.changes.create()` / `speclink.tasks.done()` |
| **HTTP default endpoint** | `<method> /api/projects/<project_id>/<noun_plural>[/<id>][/<verb>]` | `POST /api/projects/x/changes` / `POST /api/projects/x/tasks/<id>/done` |

**動詞前置例外**：`<verb>` 為 `create` / `new` / `write` 等「生產」動詞時，CLI 用 `new <noun>` 取代 `<noun> create` 對齊 spectra 慣性（如 `speclink new change` / `speclink new artifact`）。Catalogue 內以 `cli_verb_first: true` 欄位標記。Tool binding `<verb>_<noun>`（`new_change`）對齊。

**所有命名 derive 自 catalogue**：任何 surface 對不上 catalogue → `cli.unknown_command` / `tool.unknown` / `sdk.unknown_method` error。Catalogue 是 single source of truth（§21.3）；MVP 手動同步、未來 codegen。

### 21.5 完整 mapping 表（37 個 catalogue ops 對五 surface）

下表 derive 自 `doc/protocol/operations.md`（MVP 待寫）；以下是當前設計暫定值（實作前以 operations.md 為準）：

| Catalogue ID | CLI | Tool binding | SDK method | HTTP endpoint | Category | MVP |
|---|---|---|---|---|---|---|
| `project.init` | `init <name>` | `init` | `speclink.init()` | `POST /api/projects` | project | ✓ |
| `project.link` | `link <url>` | `link` | `speclink.link()` | n/a（client 端寫 link.yaml）| project | [deferred] |
| `project.unlink` | `unlink` | `unlink` | `speclink.unlink()` | n/a | project | [deferred] |
| `project.status` | `status` | `project_status` | `speclink.status()` | `GET /api/projects/<id>` | project | ✓ |
| `config.read` | `config show` | `read_config` | `speclink.config.read()` | `GET /api/projects/<id>/config` | config | ✓ |
| `config.write` | `config set` / `config edit` | `write_config` | `speclink.config.write()` | `PATCH /api/projects/<id>/config` | config | ✓ |
| `discuss.new` | `discuss new <id>` | `new_discussion` | `speclink.discussions.create()` | `POST /api/projects/<id>/discussions` | discuss | ✓ |
| `discuss.patch` | `discuss patch <id>` | `patch_discussion` | `speclink.discussions.patch()` | `PATCH /api/projects/<id>/discussions/<id>` | discuss | ✓ |
| `discuss.conclude` | `discuss conclude <id>` | `conclude_discussion` | `speclink.discussions.conclude()` | `POST /api/projects/<id>/discussions/<id>/conclude` | discuss | ✓ |
| `discuss.list` | `discuss list` | `list_discussions` | `speclink.discussions.list()` | `GET /api/projects/<id>/discussions` | discuss | ✓ |
| `discuss.show` | `discuss show <id>` | `show_discussion` | `speclink.discussions.get()` | `GET /api/projects/<id>/discussions/<id>` | discuss | ✓ |
| `discuss.delete` | `discuss delete <id>` | `delete_discussion`（**destructive**）| `speclink.discussions.delete()` | `DELETE /api/projects/<id>/discussions/<id>` | discuss | ✓ |
| `change.create` | `new change <name>` | `new_change` | `speclink.changes.create()` | `POST /api/projects/<id>/changes` | change | ✓ |
| `change.list` | `list --changes` | `list_changes` | `speclink.changes.list()` | `GET /api/projects/<id>/changes` | change | ✓ |
| `change.show` | `show change <id>` | `show_change` | `speclink.changes.get()` | `GET /api/projects/<id>/changes/<id>` | change | ✓ |
| `change.delete` | `delete change <id>` | `delete_change`（**destructive**）| `speclink.changes.delete()` | `DELETE /api/projects/<id>/changes/<id>` | change | ✓ |
| `artifact.write` | `new artifact <kind>` | `write_artifact` | `speclink.artifacts.write()` | `PUT .../artifacts/<kind>` | artifact | ✓ |
| `artifact.read` | `artifact read <kind>` | `read_artifact` | `speclink.artifacts.read()` | `GET .../artifacts/<kind>` | artifact | ✓ |
| `apply.start` | `apply start <id>` | `apply_start` | `speclink.apply.start()` | `POST .../apply/start` | apply | ✓ |
| `apply.pause` | `apply pause <id>` | `apply_pause` | `speclink.apply.pause()` | `POST .../apply/pause` | apply | ✓ |
| `task.done` | `task done <task-id> --change <id>` | `task_done` | `speclink.tasks.done()` | `POST .../tasks/<task_id>/done` | apply | ✓ |
| `review.approve` | `review approve --change <id> --phase <p>` | `review_approve` | `speclink.review.approve()` | `POST .../review/approve` | review | ✓ |
| `review.reject` | `review reject --change <id> --phase <p> --reason <s>` | `review_reject` | `speclink.review.reject()` | `POST .../review/reject` | review | ✓ |
| `review.history` | `review history --change <id>` | `review_history` | `speclink.review.history()` | `GET .../review/history` | review | ✓ |
| `archive.run` | `archive <id>` | `archive_change` | `speclink.archive.run()` | `POST .../archive` | archive | ✓ |
| `spec.list` | `list --specs` | `list_specs` | `speclink.specs.list()` | `GET .../specs` | spec | ✓ |
| `spec.show` | `show spec <cap>` | `show_spec` | `speclink.specs.get()` | `GET .../specs/<cap>` | spec | ✓ |
| `instructions.get` | `instructions <artifact> --change <id>` | `get_instructions` | `speclink.instructions.get()` | `GET .../instructions/<artifact>` | meta | ✓ |
| `analyze.run` | `analyze <id>` | `analyze_change` | `speclink.analyze.run()` | `GET .../analyze` | analyze | ✓ |
| `validate.run` | `validate <id>` | `validate_change` | `speclink.validate.run()` | `GET .../validate` | analyze | ✓ |
| `drift.run` | `drift <id>` | `drift_change` | `speclink.drift.run()` | `GET .../drift` | analyze | ✓ |
| `schema.list` | `schemas` | `list_schemas` | `speclink.schemas.list()` | `GET .../schemas` | schema | ✓ |
| `schema.show` | `schema show <id>` | `show_schema` | `speclink.schemas.get()` | `GET .../schemas/<id>` | schema | ✓ |
| `schema.fork` | `schema fork <src> <dst>` | `fork_schema` | `speclink.schemas.fork()` | `POST .../schemas/<dst>` | schema | ✓ |
| `schema.delete` | `schema delete <id>` | `delete_schema`（**destructive**）| `speclink.schemas.delete()` | `DELETE .../schemas/<id>` | schema | ✓ |
| `doctor.run` | `doctor` | `run_doctor` | `speclink.doctor.run()` | n/a（local 才有；HTTP 跑各別 check）| meta | ✓ |
| `tool.describe` | `describe-tools --format <fmt>` | n/a（meta-op）| `speclink.describeTools()` | `GET .../tool-catalogue?format=<fmt>` | meta | ✓ |

**Destructive 標記**：`change.delete` / `discuss.delete` / `schema.delete` 必須在 catalogue 標 `destructive: true`、Tool binding 標 `destructive: true`、`installSkills({ aliases })` 套用 alias 時觸發 audit event `skill.alias_applied`（§17.6）。AI skill drafts **永不**主動帶 `--force`。

## 22. `@speclink/client` 設計

給 Tool binding host（CopilotKit、GitHub Copilot SDK、OpenAI function calling、LangChain 等）整合用的 npm package。其他語言 client（Python / Go）未來依 adoption signal 加。

SDK 有 **兩個正交軸**：

- **Integration Tier**（§22.1）— 你跟 SpecLink 的關係：純 metadata 載入？接 engine logic 但 BYO 儲存？走 bundled provider？
- **Tool API Layer**（§22.2）— 你怎麼把 operation 暴露給 agent host：用內建 tool helper？filter/decorate？自寫 `defineTool`？

任 Tier × 任 Layer 自由組合。開發者依場景挑兩個軸，不互相強制。

### 22.1 Integration Tiers — 三 tier 整合模型

SpecLink SDK 對「engine logic」與「storage 媒介」的責任歸屬提供三個 tier，**開發者按場景自行挑**。三個 tier MVP 都做。

| Tier | Engine logic | Storage 媒介 | 主要 use case |
|---|---|---|---|
| **1. Helpers-only** | 你自己（或不做） | 你自己 | 已有自家完整 spec 系統、只想拿 SpecLink 的 prompt + JSON Schema + tool descriptor 當資產 |
| **2. BYO Provider** | SpecLink | 你 impl `Provider` trait | webapp 已有自家 DB，想要 SpecLink 的 state machine / validation / drift / audit、但儲存走自家 schema |
| **3. Bundled Provider** | SpecLink | SpecLink（LocalProvider / HttpProvider）| 個人 RD、團隊共用 spec 系統 — 從零開始、最低門檻 |

**對應到 Copilot SDK `defineTool` shape**（其他 Tool binding host 同理）：

#### Tier 1 — Helpers-only

純粹拿 SpecLink 當「prompt + schema 的 npm package」。Engine、Provider、state machine、validation、audit **全部你自己負責或選擇不做**。

```typescript
import { CopilotClient, defineTool } from "@github/copilot-sdk";
import { schemas, installSkills, makeToolDescriptors } from "@speclink/client/helpers";  // 無 Provider

// 1. 用 SpecLink 的 skill prompt
await installSkills("./skills", { host: "copilot-sdk" });

// 2. 用 SpecLink 的 JSON Schema 當 defineTool parameters
const createSpec = defineTool("create_spec", {
  description: "Create a new SDD change",
  parameters: schemas.changes.create,
  handler: async (args) => {
    // ▼ 100% 你自己的儲存與邏輯 — SpecLink core 不參與
    const id = await myWebappDb.insert("specs", { name: args.name, ... });
    await myAudit.log("spec.created", { id, args });
    return { changeId: id, state: "proposed" };
  },
});

const session = await client.createSession({
  model: "gpt-4.1",
  skillDirectories: ["./skills"],
  tools: [createSpec],
});
```

開發者拿到：
- ✅ SpecLink skill prompt body
- ✅ SpecLink JSON Schema（tool parameters）
- ✅ Tool 命名約定（catalogue 對齊）
- ❌ 沒有 SpecLink engine — state machine / validation / drift / audit / lock 全由你負責

#### Tier 2 — BYO Provider

接 SpecLink engine logic，但儲存層由你 impl `Provider` trait（§19.2）。Engine 跑 state machine + validation + audit，**只有 CRUD 落到你的 Provider**。

```typescript
import { CopilotClient, defineTool } from "@github/copilot-sdk";
import { SpecLink, schemas, type Provider, type Versioned, type CreateChangeRequest } from "@speclink/client";

class MyWebappProvider implements Provider {
  async createChange(req: CreateChangeRequest): Promise<Versioned<Change>> {
    const row = await myDb.insert("specs", { name: req.name, ... });
    return { value: row, etag: row.updated_at };
  }
  async readArtifact(req) { /* read from my DB */ }
  async writeArtifact(req) { /* write to my DB, honor expected_etag */ }
  // ... 其他純 CRUD method（依 §19.2 slim trait）
}

const speclink = new SpecLink({ provider: new MyWebappProvider() });

const createSpec = defineTool("create_spec", {
  description: "Create a new SDD change",
  parameters: schemas.changes.create,
  handler: async (args) => {
    return await speclink.changes.create(args);  // Engine 跑 state machine + validation + audit
  },
});
```

**注意**：Tier 2 在 MVP 階段有兩個實作路徑：
- **HTTP 邊界**（MVP 可走）— 用 HttpProvider custom mode + mapping.yaml，把 SpecLink CRUD ops 對到自家 endpoints；本質就是 BYO Provider via HTTP transport（§19.4.2）
- **In-process trait impl**（[deferred]）— 直接 impl `Provider` TS interface 塞給 `new SpecLink({ provider: ... })`；需要 SDK 端不走 CLI subprocess（§22.7）、改 in-process engine、屬 post-MVP

#### Tier 3 — Bundled Provider

預設 LocalProvider 或 HttpProvider default mode，「從零開始」最省力。

```typescript
import { CopilotClient, defineTool } from "@github/copilot-sdk";
import { SpecLink, schemas } from "@speclink/client";

const speclink = new SpecLink({
  provider: { type: "local", workspaceRoot: "/var/data/billing" },
});

const createSpec = defineTool("create_spec", {
  description: "Create a new SDD change",
  parameters: schemas.changes.create,
  handler: async (args) => {
    return await speclink.changes.create(args);  // 全套 SpecLink
  },
});
```

#### Tier 選擇判斷流程

| 問題 | 是 | 否 |
|---|---|---|
| 想要 SpecLink 的 6-state lifecycle？ | Tier 2 / 3 | Tier 1 |
| 已有自家 DB schema 且不想改？ | Tier 1 / 2 | Tier 3 |
| 想要 SpecLink 的 validation / drift / audit？ | Tier 2 / 3 | Tier 1 |
| 從零開始、無歷史包袱？ | Tier 3 | 看上面 |

#### MVP 範疇與限制

| Tier | MVP 狀態 | 備註 |
|---|---|---|
| Tier 1 — Helpers-only | ✅ 完整實作 | `@speclink/client/helpers` sub-entry，純 export、無 Provider 依賴 |
| Tier 2 — BYO Provider（HTTP） | ⚠️ 隨 HttpProvider 一起做 | HttpProvider 本身 MVP `[trait skeleton only, no impl]`；正式 ship 屬 [deferred] |
| Tier 2 — BYO Provider（in-process） | ❌ [deferred] | 需要 SDK in-process engine、N-API binding 或 pure TS engine impl |
| Tier 3 — Bundled LocalProvider | ✅ 完整實作 | MVP 唯一 ship 完整的 provider impl |
| Tier 3 — Bundled HttpProvider | ❌ [deferred] | 隨 Provider impl 一起做 |

### 22.2 三層 API

| Layer | 用途 | 例 |
|---|---|---|
| **Layer 1** — 一行式 | 標準整合、不客製 | `await speclink.getCopilotSdkTools()` |
| **Layer 2** — 選擇性客製 | filter / 改 name 描述 | `getCopilotSdkTools({ filter, decorate, categories, phases })` |
| **Layer 3** — 完全自寫 | hook / 合 operation / 自訂 schema | developer 自己 `defineTool` + handler 內呼 `speclink.changes.create()` |

三層**都自然走 core**（透過 spawn CLI subprocess，未來 N-API 直連）。

**Layer 1 預設 curated subset**：`getCopilotSdkTools()` 不帶任何 option 時 **不**返回全部 35+ ops（context bloat、AI selection accuracy 過 20 就掉），只返回 12 個 5 個 skill 真正用到的核心 op：

```
discuss.new / discuss.patch / discuss.conclude
change.create / change.show
artifact.write / artifact.read
apply.start / task.done
review.approve / review.reject
archive.run
```

要全集合呼 `getCopilotSdkTools({ full: true })`。要篩特定 category / phase 呼 `getCopilotSdkTools({ categories: ['change', 'review'] })` 或 `getCopilotSdkTools({ phases: ['propose'] })`。

`categories` 對應 §21.4 表內 Category 欄；`phases` 對應 8 skill 名稱（`discuss` / `propose` / `apply` / `archive` / `ingest` / `analyze` / `verify` / `drift`）。

### 22.3 Exported symbols

npm package 拆兩個 entry point，對應 Tier 1 vs Tier 2/3：

#### `@speclink/client/helpers` — Tier 1 entry（zero Provider 依賴）

純 metadata exports，不需 `new SpecLink()`、不需要任何 Provider 配置。Tier 1 整合場景的唯一入口。

```typescript
// @speclink/client/helpers

// JSON Schema 物件（給 developer 寫 defineTool 用）
export const schemas: {
  changes: { create: JSONSchema, list: JSONSchema, ... };
  artifacts: { write: JSONSchema, read: JSONSchema };
  tasks: { done: JSONSchema, add: JSONSchema };
  // ... 對應 §21.5 全部 35 ops
};

// Skill deployment — 純檔案複製 + bindings 拼裝，不需 Provider
export function installSkills(dir: string, opts: InstallSkillsOptions): Promise<void>;
export function loadSkillPrompt(name: SkillName): Promise<string>;
export function loadAllSkills(): Promise<{ combined: string; byName: Record<SkillName, string> }>;

// Tool descriptor 產生器 — 純靜態組合 catalogue + schema，產出 tool descriptor object
//   注意：不含 dispatcher（dispatcher 屬 Tier 2/3 的 Engine）
//   Tier 1 user 自己寫 handler，這些 helper 只給他 name + description + parameters 三件套
export function makeToolDescriptors(opts: MakeToolDescriptorsOptions): ToolDescriptor[];
//   各 host 專屬包裝（純 type 層 sugar，內部都是 makeToolDescriptors）
export function makeCopilotSdkToolDescriptors(opts?: ToolHelperOptions): CopilotSdkToolDescriptor[];
export function makeCopilotKitActionDescriptors(opts?: ToolHelperOptions): CopilotKitActionDescriptor[];
export function makeOpenAIToolDescriptors(opts?: ToolHelperOptions): OpenAIToolDescriptor[];
export function makeLangChainToolDescriptors(opts?: ToolHelperOptions): LangChainToolDescriptor[];

// TypeScript types（給 developer 寫 typed handler 用）
export type ChangesCreateArgs = { name: string; description: string };
export type ChangesCreateResult = { changeId: string; state: ChangeState; createdAt: string };
// ...

export interface MakeToolDescriptorsOptions {
  format: 'copilot-sdk' | 'copilotkit' | 'openai' | 'langchain';
  filter?: OperationId[];
  decorate?: Record<OperationId, { name?: string; description?: string }>;
  categories?: string[];
  phases?: SkillName[];
  full?: boolean;
}

export interface InstallSkillsOptions {
  host: 'claude' | 'codex' | 'copilot-sdk' | 'copilotkit' | 'openai' | 'langchain';
  aliases?: Record<OperationId, string>;
  force?: boolean;
}
```

#### `@speclink/client` — Tier 2/3 entry（含 Engine + Provider）

主 entry。Re-exports helpers，再加 `SpecLink` class 跟 `Provider` trait。

```typescript
// @speclink/client

// Re-export 所有 helpers — Tier 2/3 user 仍可從這裡用 schemas / installSkills 等
export * from '@speclink/client/helpers';

// 主類 — 提供 engine + provider
export class SpecLink {
  constructor(config: SpecLinkConfig);
  
  // 設定來源 helpers
  static fromEnv(): SpecLink;                    // 讀 SPECLINK_* 環境變數
  static fromConfig(path: string): Promise<SpecLink>;
  static fromWorkspace(path: string): Promise<SpecLink>;  // 找 path/.speclink/link.yaml
  
  // Project lifecycle — MVP scope 嚴格規範
  //   init() / link() / unlink() 屬 [deferred]（跟 HttpProvider 一起做）
  //   MVP SDK 只支援 fromWorkspace / fromConfig / fromEnv 連到既有 LocalProvider
  //   調用 init() / link() / unlink() 會直接拋 provider.not_supported（MVP build）
  init(opts?: { force?: boolean; workspaceRoot?: string }): Promise<void>;  // [deferred]
  link(opts: { baseUrl: string; project: string }): Promise<void>;            // [deferred]
  unlink(): Promise<void>;                                                    // [deferred]
  
  // SDD operations（typed methods、對應 §21 catalogue）— 走 engine + provider
  changes: { create, list, get, delete, ... };
  artifacts: { write, read };
  tasks: { done, add };
  review: { approve, reject, history };
  archive: { run };
  discussions: { create, list, patch, conclude, ... };
  apply: { start, pause };
  config: { read, write };
  schema: { list, fork, validate };
  
  // Tool helpers — 跟 makeXxxToolDescriptors 等價、但內附 dispatcher
  //   Tier 1 應該用 makeXxxToolDescriptors 自寫 handler；
  //   Tier 2/3 用這些 method 拿到 「descriptor + 內建 handler 走 speclink.xxx.yyy」 的 bundle
  getCopilotSdkTools(opts?: ToolHelperOptions): Promise<CopilotSdkTool[]>;
  getCopilotKitActions(opts?: ToolHelperOptions): Promise<CopilotKitAction[]>;
  getOpenAITools(opts?: ToolHelperOptions): Promise<OpenAITool[]>;
  getLangChainTools(opts?: ToolHelperOptions): Promise<LangChainTool[]>;
}

// Provider trait — Tier 2 BYO 用，自寫 class 實作這些 method
export interface Provider {
  projectId(): string;
  capabilities(): Capabilities;
  // 純 CRUD（對應 §19.2 slim trait — 邏輯 method post-MVP 搬到 Engine、見 §19.1）
  readConfig(): Promise<Versioned<Config>>;
  writeConfig(req: WriteConfigRequest): Promise<Versioned<Config>>;
  createChange(req: CreateChangeRequest): Promise<Versioned<Change>>;
  // ... 對應 §19.2 全部
}
export interface Versioned<T> { value: T; etag: string; }
export type Etag = string;

// Tool helper option 型別
export interface ToolHelperOptions {
  filter?: OperationId[];                    // 只啟用部分 operation
  decorate?: Record<OperationId, {
    name?: string;                            // override tool name
    description?: string;                     // override description
  }>;
  hooks?: {
    afterAny?: (op: OperationId, result: unknown) => Promise<void>;
    beforeAny?: (op: OperationId, params: unknown) => Promise<void>;
  };
}
```

**Entry point 選擇規則**：

| 你的 Tier | Import from | 理由 |
|---|---|---|
| Tier 1 — Helpers-only | `@speclink/client/helpers` | 連 `SpecLink` class 都不載入 → bundle size 小、無 Provider 依賴 |
| Tier 2 — BYO Provider | `@speclink/client` | 需 `Provider` interface + `SpecLink` class（傳入自家 impl） |
| Tier 3 — Bundled | `@speclink/client` | 需 `SpecLink` class + provider config |

**Tier 1 user 也可 import from `@speclink/client`** — re-export 保證 superset 仍可用、只是多了用不到的 surface。Bundle 工具會 tree-shake 掉 `SpecLink` class，實務上差異不大；分兩 entry 是給 「不想看到 Provider 概念」的 user 一條乾淨路徑、不是強制隔離。

**Aliases 安全規則**：

- Alias 套用於 `catalogue.destructive: true` 的 operation 時（`change.delete` / `discuss.delete` / `schema.delete`），`installSkills()` 必須寫 audit event `skill.alias_applied`（§17.6）含 `canonical_op` / `alias` / `host` / `destructive_flag: true`
- Dispatcher 收到 tool call 永遠回傳 result 內含 `canonical_op` 欄位（即使 caller 用 alias 名稱），讓 AI 可 cross-check「我以為呼叫 `safe_remove_item`、實際 dispatch 到 `change.delete`」
- 對非 destructive op 套 alias 不需 audit（cost-free）
- `installSkills({ aliases: { 'change.delete': 'archive_change' } })` — 把 destructive `delete` 改名成看似安全的 `archive_change` 視為 social engineering vector；MVP 接受、audit event 是唯一防線

### 22.4 三層使用範例

#### Layer 1（一行）

```typescript
import { CopilotClient } from "@github/copilot-sdk";
import { SpecLink } from "@speclink/client";

const speclink = new SpecLink({ project: "billing", workspaceRoot: "/var/data/billing" });

const session = await client.createSession({
  model: "gpt-4.1",
  skillDirectories: ["./tmp/skills"],
  tools: await speclink.getCopilotSdkTools(),
});

await speclink.installSkills("./tmp/skills", { host: "copilot-sdk" });
```

#### Layer 2（filter + decorate）

```typescript
const tools = await speclink.getCopilotSdkTools({
  filter: ["change.create", "change.list", "change.get"],     // 只開 3 個
  decorate: {
    "change.create": {
      name: "create_spec",
      description: "Create a feature spec for the billing team",
    },
  },
  hooks: {
    afterAny: async (op, result) => {
      await audit.log(op, result);
    },
  },
});
```

#### Layer 3（完全自寫）

```typescript
import { defineTool } from "@github/copilot-sdk";
import { SpecLink, schemas, type ChangesCreateArgs } from "@speclink/client";

const speclink = new SpecLink({ project: "billing", workspaceRoot: "/var/data/billing" });

const proposeTool = defineTool("create_spec", {
  description: "Create a new SDD change for the billing team",
  parameters: schemas.changes.create,
  handler: async (args: ChangesCreateArgs) => {
    if (args.name.includes("billing")) {
      await audit.log("billing-spec-created", args);
    }
    const result = await speclink.changes.create(args);   // ← 走 core
    await slack.notify("New spec proposed", result);
    return result;
  },
});

// 用 aliases 確保 skill body 內的 change.create reference 改成 create_spec
await speclink.installSkills("./tmp/skills", {
  host: "copilot-sdk",
  aliases: { "change.create": "create_spec" },
});

const session = await client.createSession({
  skillDirectories: ["./tmp/skills"],
  tools: [proposeTool /* + 其他 */],
});
```

### 22.5 Workspace / Provider 設定

SDK **不**做 cwd-walk-up 自動探測 — webapp 的 cwd 通常不是 SDD project root。設定一定要 explicit。

**設定來源優先級**：

```
constructor 顯式參數
    ▼
fromConfig(path) / fromWorkspace(path) 指定的檔案
    ▼
SPECLINK_* 環境變數
    ▼
（無預設值 — SDK 一定要至少有一個來源；缺則 throw config.missing）
```

範例：

```typescript
// LocalProvider，顯式
const speclink = new SpecLink({
  project: "billing-system",
  provider: { type: "local", workspaceRoot: "/var/data/speclink/billing-system" },
});

// HttpProvider default，顯式
const speclink = new SpecLink({
  project: "billing-system",
  provider: {
    type: "http",
    baseUrl: "https://specs.team.internal",
    auth: { type: "bearer", token: process.env.SPECLINK_TOKEN! },
  },
});

// HttpProvider custom，顯式
const speclink = new SpecLink({
  project: "billing-system",
  provider: {
    type: "http",
    baseUrl: "https://my-webapp.example.com/api",
    customMapping: "/etc/speclink/mapping.yaml",
    auth: { type: "bearer", token: process.env.SPECLINK_TOKEN! },
  },
});

// 從 env var 全部
const speclink = SpecLink.fromEnv();

// 從外部 config 檔（YAML，schema 同 link.yaml）
const speclink = await SpecLink.fromConfig("/etc/speclink/billing.yaml");

// 從 working dir（讀 path/.speclink/link.yaml — 罕見，給「webapp 跑在 RD laptop dev mode」用）
const speclink = await SpecLink.fromWorkspace(process.cwd());
```

### 22.6 Multi-project webapp

SpecLink **不**做 multi-tenant routing；webapp 自管 instance cache。對應到 §19.2 — 每個 `SpecLink` instance 內部會走 `ProviderRegistry.open(project_id)` 拿綁定該 project 的 `Provider`、後續操作不需再帶 project_id。

```typescript
const speclinkCache = new Map<string, SpecLink>();

function getSpeclink(projectId: string): SpecLink {
  if (!speclinkCache.has(projectId)) {
    speclinkCache.set(projectId, new SpecLink({
      project: projectId,
      provider: { type: "local", workspaceRoot: `/var/data/speclink/${projectId}` },
    }));
  }
  return speclinkCache.get(projectId)!;
}

app.post("/api/projects/:id/propose", async (req, res) => {
  const speclink = getSpeclink(req.params.id);
  res.json(await speclink.changes.create(req.body));
});
```

**Cache eviction**：MVP 不規定策略；webapp 想做 LRU / TTL / size-bound 都可。HttpProvider 構造便宜（純塞 baseUrl + project_id + token 進 struct），cache miss penalty 很低。LocalProvider 構造會開 SQLite connection、open file lock，cache 比較有價值。

### 22.7 內部 transport

MVP 階段 client 內部一律 **spawn `speclink` CLI subprocess**。優點：

- 不依賴 native binding（純 TypeScript / Node.js）
- 可以跨任何 OS（macOS / Linux / Windows，只要 speclink CLI binary 存在）
- 跟 CLI 同 logic 路徑、沒 drift

CLI 透過 `--json` flag 出 JSON、env vars 傳設定、stdin 傳大型 payload。SDK 解析 JSON、map 回 typed result。

未來可能改 N-API binding（直連 speclink-core in-process）拿到 latency 改善；對外 API 不變。

### 22.8 安全 / Edge cases

- **Webapp restart**：SDK instance 重建後狀態仍同步（state 在 provider 端）
- **Concurrent operation**：兩個 request 同時操作同 change → 走 provider 端 lock（HttpProvider 是 server transaction、LocalProvider 是 file lock）
- **CLI binary 不存在**：constructor 不 throw（lazy 偵測），第一個 operation 失敗時清楚錯誤訊息 `cli.not_found`
- **CLI binary 版本不匹配**：SDK 在 init 時呼 `speclink --version` 對比 package version；不匹配 → warning（非 error）

## 23. 不在本文範圍

- 細部 CLI 指令的 JSON schema（待 `operations.md` 補完）
- 每個 state transition 的具體 SQL（待實作）
- Skill SKILL.md 的具體 prompt 內容（下一輪 discuss 後寫進 embedded templates、按 §4.3 workflow + bindings 結構）
- `speclink doctor` 具體檢查項目（後續討論）
- 完整 error code 表（後續討論）
- `@speclink/client` 完整 API reference 細節（待實作階段補；§22 是設計骨架）
- `doc/protocol/operations.md` 30+ operation 完整 schema（MVP 實作前要寫完）

實作階段會以小範圍 change 滾動式補充，每個 change 對應一份具體 spec。

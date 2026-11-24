## Context

`speclink instructions <產出物> --change <名稱> --json` 的回應由 speclink-core 的 `build_artifact` 產生（`crates/engine/speclink-core/src/workspace/instructions.rs`）。本機 CLI、Server（遠端連線的 CLI 向它要回應）與 Node SDK 的 `dispatch` 都呼叫這一個函式，所以回應的內容只有一個實作落點。

現況有三個問題，都在這個函式與它用的文字裡：

- 回應的 `locale` 對每個產出物都是專案的 `locale`。規格的語言由 `spec_locale` 決定，所以 `locale` 對 specs 產出物是錯的訊號（例如 `locale: tw`、`spec_locale: en`）。
- 規格的指示文字（`crates/engine/speclink-core/assets/schema/spec-driven/fork.schema.yaml` 的 specs instruction）先寫「English by default. If the project sets spec_locale in speclink/config.yaml…」。`spec_locale` 解析成非英文時，引擎才在最後補一段「This project sets spec_locale: <代碼> — write spec prose in …」。模型先讀到預設英文；遠端專案與內嵌宿主也沒有 `speclink/config.yaml` 這個檔。propose、ingest 技能再講一次「規格預設英文，除非 `workflow-config show --json` 的 `specLocale`…」，而 `workflow-config show` 不套用環境變數，和指示可能給出不同的語言。
- 任務的指示在 tdd、audit 開啟時寫「fetch the full discipline with speclink instructions --skill tdd」。這個命令沒有引擎命令，`dispatch` 以 `invalid_argv` 拒絕（node-sdk 的正式規格）。完整的紀律是實作時用的，apply 技能自己會取得。

限制：技能原稿一改，內嵌資產版號、golden 快照與 `assets.lock` 必須同批更新；技能全文不得提及 config.yaml（`crates/engine/speclink-core/tests/it/skill_verbization.rs` 守門）。

## Goals / Non-Goals

**Goals:**

- `instructions specs` 的回應自己說清楚規格的語言：`locale` 欄位與指示文字給同一個答案，不要求模型讀任何設定。
- propose、ingest 技能只看回應的 `locale`，不再自己查 `specLocale`。
- 任務的指示不指到 `dispatch` 拒絕的命令。
- 本機、遠端連線、內嵌宿主得到同一份回應，只有一個實作落點。

**Non-Goals:**

- 不讓 `dispatch` 接受帶 `--skill` 的 `instructions`，也不改它的拒絕訊息（node-sdk 的正式規格不動）。
- 不在回應新增欄位；不改 `workflow-config show` 的輸出（`specLocale` 仍是設定檔裡的原值）。
- 不改 baseline 技能：它沒有變更可以呼叫 `instructions`，繼續讀 `workflow-config show --json` 的 `specLocale`。
- 不改 `instructions apply` 的回應，也不改 apply 技能取得 tdd、audit 紀律的方式。
- 不處理 validate 錯誤結尾的「see: speclink drift」指向（討論 embedded-host-needs-8 的延後項目）。
- 不做 `skills.render` 的 `gitHistory` 選項（同一份討論的 cut 2）。

## Decisions

### locale 欄位對 specs 產出物回報規格的語言

`build_artifact` 組回應時，產出物是 `specs` 就把 `locale` 設成 `locale_display(policy.spec_locale)`，其他產出物維持 `policy.locale`。`policy.spec_locale` 是 `resolve_policy` 已經算好的值：`tw`、`ja` 原樣；`auto` 換成 `locale` 的有效值；沒設定、空字串、`en` 都是 `None`，`locale_display(None)` 是 `English`。環境變數 `SPECLINK_SPEC_LOCALE`、`SPECLINK_LOCALE` 由同一個解析入口處理，不新增解析邏輯。

判斷用產出物的識別字 `specs`，和現有補語言說明、補 tdd／audit 說明的寫法一致；自訂 schema 只要有 `specs` 產出物就適用。

替代方案：另加一個 `specLocale` 欄位、`locale` 不動。不採用，因為模型仍要在兩個欄位之間選，這次觀察到的錯誤會留著。

### 規格的語言說明只由引擎在指示最後寫出

從 `fork.schema.yaml` 的 specs instruction 移除整段「Spec file language: … so the analyzer and archive tooling can parse them.」。`build_artifact` 對 `specs` 產出物一律在指示最後加一段（中間空一行），不再只在非英文時加：

```
Write spec prose in <顯示名稱>. Structural markers (`## ADDED/MODIFIED/REMOVED/RENAMED Requirements`, `### Requirement:`, `#### Scenario:`, `- **WHEN**`/`- **THEN**`) and normative keywords (SHALL/MUST/SHALL NOT/MUST NOT) stay in English so the analyzer and archive tooling can parse them.
```

規格語言的代碼是 `tw` 或以 `zh` 開頭時，後面照舊接中文模糊用語的提醒（Vague Chinese wording … state requirements with SHALL/MUST.），文字不變。schema 的 specs 產出物沒有 instruction 時，這一段就是整個 instruction（和現在非英文時的行為相同）。

這一段不再印 `spec_locale: <代碼>`，所以現在 `auto` 時印出解析後代碼的不精確一併消失。

替代方案：保留 YAML 的那一段、只把引擎的補充改成一律寫出。不採用，因為「預設英文」與「去看 config.yaml」兩句仍在最前面，正是模型照做的那兩句。

已經用 `speclink schema fork` 複製出去的自訂 schema 仍帶舊的那一段；引擎照樣在最後寫出解析後的語言，兩段不矛盾（舊段講規則、新段講結果），所以不做遷移。

### propose 與 ingest 技能以回應的 locale 為規格語言

改兩支技能原稿裡說明 `locale` 的那一句（`crates/engine/speclink-core/assets/skills/propose.md` 的 instructions JSON 欄位清單、`crates/engine/speclink-core/assets/skills/ingest.md` 的更新步驟）：

- 寫明 `locale` 是「這一份產出物」的語言，引擎已經依產出物解析，specs 產出物拿到的就是規格的語言。
- 拿掉「Spec files default to English instead — unless speclink workflow-config show --json returns a specLocale…」整句。
- 保留「結構標記與 SHALL／MUST 維持英文」。

兩支技能全文不含 `specLocale`、不含 config.yaml。propose 技能其他步驟用到的 `speclink workflow-config show --json`（讀 `schema`、`worktree`）不動。新文字不加全大寫強調字（skill-authoring 的強調字上限只能降不能升）。

替代方案：技能繼續指向 `workflow-config show`。不採用：它不套用環境變數；討論 tdd-switch-apply-wiring 已排除「技能自己查政策」，理由是回應帶值更穩。

### 任務指示改寫 tdd 與 audit 紀律的指向

`build_artifact` 對 `tasks` 產出物加的兩段說明，括號改成一句話，其餘不變：

- tdd：`This project enables TDD — structure implementation tasks in Red-Green-Refactor order: each unit of work starts with a failing-test task, then the implementation task that turns it green. The apply step supplies the full TDD discipline; you do not need it to write the tasks.`
- audit：`This project enables the audit discipline — when tasks add APIs, configuration options, or parameter handling, include a step to apply the sharp-edges audit checklist. The apply step supplies the checklist; you do not need it to write the tasks.`

兩段都不含 `instructions --skill`。保留一句來源說明而不是直接刪掉括號，因為只留「audit checklist」卻不說它從哪裡來，模型可能自己去找。

替代方案：讓 `dispatch` 讀得到 `instructions --skill`。不採用：tdd、audit 的原文是實作時的紀律（audit 的原文還含獨立模式的步驟），寫任務用不到；而且要改 node-sdk 的正式規格，這條路也不經過宿主的 `beforeCommand`。

### 內嵌資產版號與 golden 同批更新

技能原稿與 schema 原稿都改了，`ASSET_VERSION`（`crates/engine/speclink-core/src/workspace/init.rs`）由 v1.59.0 升到 v1.60.0，`crates/engine/speclink-core/tests/golden/` 的十份技能快照與 `assets.lock` 用既有的更新方式重新產生。順序：先改原稿與版號、建置 CLI，再重新產生 golden 與這個 repo 自己安裝的技能檔（.claude/skills、.agents/skills）。

## Implementation Contract

**行為**

| 專案設定 | `instructions specs --json` 的 `locale` | 指示最後一段的開頭 |
| --- | --- | --- |
| `locale: tw`，沒設 `spec_locale` | `English` | `Write spec prose in English.` |
| `locale: tw`、`spec_locale: en` | `English` | `Write spec prose in English.` |
| `locale: tw`、`spec_locale: auto` | `Traditional Chinese (繁體中文)` | `Write spec prose in Traditional Chinese (繁體中文).`，後接中文模糊用語的提醒 |
| `locale: tw`、`spec_locale: ja` | `Japanese (日本語)` | `Write spec prose in Japanese (日本語).` |
| 沒設 `locale`、`spec_locale: auto` | `English` | `Write spec prose in English.` |
| `spec_locale: en`，環境變數 `SPECLINK_SPEC_LOCALE=ja` | `Japanese (日本語)` | `Write spec prose in Japanese (日本語).` |

上面每一列，`instructions proposal|design|tasks --json` 與 `instructions apply --json` 的 `locale` 都維持專案 `locale` 的顯示名稱。

- `instructions specs` 的 `instruction` 不含 `config.yaml`、不含 `English by default`、不含 `spec_locale`。
- `instructions tasks` 的 `instruction`：tdd 開啟時含 `Red-Green-Refactor` 與 `The apply step supplies the full TDD discipline`；audit 開啟時含 `sharp-edges audit checklist` 與 `The apply step supplies the checklist`；兩者都不含 `instructions --skill`。兩個開關都關閉時 `instruction` 和現在逐位元相同。
- 產生的 speclink-propose 與 speclink-ingest 技能（每一種工具、規格在不在 checkout 兩種情況）不含 `specLocale`、不含 `config.yaml`，並寫明 `locale` 是這份產出物的語言。

**介面與資料形狀**

回應的欄位名稱與型別不變（`locale` 仍是字串）。沒有新的命令、旗標、設定鍵或 serde 結構；設定檔的讀取不變，所以既有的 `speclink/config.yaml` 照常讀得進來。

**失敗模式**

不新增失敗路徑。`speclink/config.yaml` 無法解析時，`instructions` 照舊以非零 exit code 失敗（fail-closed）；`spec_locale` 的值不合法時照舊由設定的驗證拒絕。連到還沒升級的 Server 時，CLI 原樣輸出 Server 的回應（舊語意），不補、不改。

**驗收**

- `cargo test -p speclink-core`：`build_artifact` 的單元測試涵蓋上表每一列，以及 tdd／audit 兩段文字。
- `cargo test -p speclink-cli --test it`：`instructions_policy` 的整合測試經 CLI 斷言 `locale` 與 `instruction`（含環境變數那一列）。
- `npm --prefix crates/adapters/speclink-node test`：經 `host.dispatch` 得到與 CLI 相同的 `locale`，且任務的指示不含 `instructions --skill`。
- golden 快照與 `assets.lock` 更新後 `render_golden` 的測試通過；`skill_verbization` 的守門通過。

**範圍**

- 範圍內：`build_artifact` 的 `locale` 與兩處補充文字、specs 的 schema 原稿、propose 與 ingest 的技能原稿、資產版號與 golden、相關測試、`docs/configuration.md` 與 `docs/configuration.zh-TW.md` 的 `spec_locale` 說明。
- 範圍外：Non-Goals 列的每一項；speclink-host、speclink-server、speclink-remote、apps/desktop 的程式（它們只轉送回應）。

## Risks / Trade-offs

- [回歸對照] 沒設 `spec_locale` 的專案，`instructions specs` 的 `instruction` 與 `locale` 都會改變（以前這個情況的回應逐位元不變）。→ 這是刻意的變更，寫進規格；固定這段文字的測試與十份 golden 同批更新，更新後用 `git diff` 確認 golden 只有 propose、ingest 兩支技能的那一句與版號不同。
- [版本落差] repo 裡的技能檔已更新、但某位成員的 CLI 還是舊版時，新技能相信 `locale`，舊引擎回報的是專案的 `locale`；`locale` 不是英文而規格應該是英文的專案，規格會被寫成專案的語言。→ 舊引擎的指示仍寫著「English by default」，而且 CLI 在技能檔比自己新時會顯示技能版本提醒（workspace-tools「CLI 的技能版本提醒」）；升級 CLI 即可。
- [自訂 schema] 已複製出去的 schema 仍帶舊的語言段落，和引擎新加的一段並存。→ 兩段不矛盾，不做遷移；`speclink schema fork` 之後產生的新 schema 沒有舊段落。
- [遠端] 連到舊版 Server 時回應是舊語意。→ 遠端專案的技能也取自 Server，技能與回應的版本一致，行為和現在相同。
- [跨平台] 改動只有字串的組合，不碰路徑、檔案系統與子程序，Windows、macOS、Linux 行為相同。
- [儲存解耦] 語言的解析結果由引擎放進回應，技能不再假設有 `speclink/config.yaml` 這個檔，朝「規格不在 checkout 也能用」的方向前進。

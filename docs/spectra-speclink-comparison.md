# Spectra 與 Speclink 功能比較分析報告

> 本報告比較 **Speclink**（本專案以 Rust 重新實作的 SDD 引擎）與參考實作 **Spectra 2.3.1**。所有對照均以「相同輸入下，實際執行兩個 CLI 並比較輸出」為方法，spectra 為已安裝的 `spectra.exe`，speclink 為 `target/debug/speclink.exe`。

## 1. 摘要（結論先行）

- **CLI 引擎、流程、輸出結果、技能內容、功能結構、流程邏輯**在共同功能面**與 Spectra 完全一致**；所有可觀察到的差異都落在使用者**刻意指定的取捨**上（移除的功能 + discuss 強化 + 品牌改名）。
- 量化證據：
  - **CLI 輸出對照套件**：讀取指令 + 完整生命週期 **31/31 全數一致**（連續多次隨機主題執行皆通過）。
  - **analyze / drift 跨全部 8 個 demo 主題**：**16/16 一致**。
  - **help 文字**：20 個共同指令 **18 個逐字一致**，另 2 個差異為刻意移除功能（`--parked`）與工具範例字串。
  - **技能內容**：10 個共同技能中，7 個為純品牌替換後**逐字相同**，其餘為刻意的功能移除與強化。
  - **生成檔案**：`openspec/config.yaml`、`.gitignore`、`.claude/settings.json` **完全相同**；`.speclink.yaml`、`CLAUDE.md`/`AGENTS.md` 為品牌 + 刻意移除段落。
  - **對抗式邊界審計**：4 輪、每輪 5 個平行代理，共數百項邊界檢查，累計揪出的**所有真實不一致均已修正並回歸驗證**（詳見 §9），殘留僅純 cosmetic。
- **端到端驗證**：以 speclink 完整跑過 `discuss → propose → apply → archive`，用 HTML + Canvas 建出一個**可玩的彈珠檯遊戲**（`pinball/index.html`），並以模擬 DOM 的測試харness 驗證發球、翻板、緩衝器計分、落袋扣球、Game Over/重開、HUD 皆正常。

## 2. 方法論

1. **Ground truth 擷取**：在乾淨沙箱對 `spectra.exe` 執行全部指令的 `--help`、`--json`/人類輸出、init 產物、`instructions --skill` 內嵌技能本體、demo 全流程，存為基準。
2. **逐指令對照**：對 spectra 與 speclink 以相同輸入執行，輸出經正規化後比較：
   - **路徑正規化**：各 CLI 於自己的沙箱執行，沙箱根路徑替換為 `ROOT`。
   - **品牌正規化**：比較時把 spectra 側的 `spectra/Spectra/SPECTRA` 映射為 speclink 對應字串。
   - **JSON 語意比較**：解析後遞迴排序 key 再比較（因 spectra 對 `contextFiles` 等使用 HashMap，key 順序非確定性）。
   - **人類輸出**：正規化後逐字比較。
3. **對抗式交叉驗證**：以多個獨立代理針對邊界情境（MODIFIED/REMOVED/RENAMED delta、錯誤訊息、locale、多能力、空狀態）獨立搜尋遺漏的不一致（見 §9）。

## 3. CLI 指令表面（command surface）

Speclink 保留 Spectra 的 21 個共同頂層指令，移除 3 個（依需求）：`search`、`park`、`unpark`；新增 1 個指令群：`discuss`（見 §8）。

| 類別 | 指令 | Speclink |
|---|---|---|
| 初始化 | `init`, `update` | ✅ 一致 |
| 瀏覽 | `list`, `show`, `status` | ✅ 一致（`list` 移除 `--parked`） |
| 建立 | `new change`, `new artifact`, `demo` | ✅ 一致 |
| 品質 | `validate`, `analyze`, `drift` | ✅ 一致 |
| 指令/schema | `instructions`, `schemas`, `templates`, `schema` | ✅ 一致 |
| 生命週期 | `archive`, `task done`, `in-progress add` | ✅ 一致 |
| 設定/雜項 | `config`, `completion`, `feedback` | ✅ 一致 |
| 向量搜尋 | `search` | ❌ 移除（依需求） |
| 暫存 | `park` / `unpark` | ❌ 移除（依需求） |
| 討論 | `discuss new/list/show/add-round/conclude` | ➕ 新增（強化） |

**help 文字對照**：20 個共同指令中 18 個 `--help` 輸出逐字一致（品牌正規化後）。剩餘 2 個差異皆為刻意：
- `list`：Spectra 有 `--parked  Show parked changes`，Speclink 無（park 已移除）。
- `init`：`--tools` 範例 Spectra 寫 `e.g., claude, cursor`，Speclink 寫 `e.g., claude, codex`（Speclink 支援 claude/codex，未支援 cursor，故如實標示）。

## 4. CLI 輸出對照（引擎邏輯與輸出）

以 `demo` 產生的變更（複製到兩邊確保內容相同）與一個完整生命週期變更（new change → 4 產物 → task done → archive）對照，共 31 項，**全部一致**：

| 面向 | 對照項目 | 結果 |
|---|---|---|
| JSON | status, validate, analyze, drift, list, templates, schemas, instructions×5(proposal/specs/design/tasks/apply) | ✅ 全一致 |
| 人類 | status, analyze, drift, list, show, templates, schemas | ✅ 全一致 |
| 生命週期 | new change, new artifact×4, task done(json/human), in-progress add, archive | ✅ 全一致 |
| 正典 spec | archive 後 `openspec/specs/<cap>/spec.md`（含 `@trace`）逐字相同 | ✅ 一致 |

**穩健性**：因 `demo` 每次隨機主題，對照套件連續執行多次仍維持 31/31。另外針對全部 8 個 demo 主題（access-control、audit-trail、batch-export、keyboard-macros、real-time-sync、smart-search、snapshot-restore、theme-engine）逐一測 `analyze / drift / validate / status / show / show --json`，**48/48 一致**——涵蓋 4 維分析（Coverage/Consistency/Ambiguity/Gaps，含「WHEN/THEN 已具體則不建議加 Example」的啟發式、弱語言偵測、contiguous 覆蓋比對、Skipped 維度）與漂移錨點抽取（含停用字表）。

**錯誤與邊界情境**（經 §9 五輪對抗式審計逐一對齊）：不存在的變更、重複建立、缺 capability、未知 artifact 型別、未知 skill、非數字/越界/0 task id、無 tasks.md、專案外執行、僅 proposal 的變更、blocked 狀態、多能力、locale 未對映碼/大小寫、config.yaml locale、MODIFIED/REMOVED/RENAMED delta、RENAMED-only/空操作 delta、malformed delta、archive `--skip-specs`/`--mark-tasks-complete`、schema/templates 未知 schema、`show --item-type spec|change` 等——錯誤訊息、退出碼與輸出**均與 spectra 對齊**。

## 5. 技能（skills）比較

Speclink 生成 Claude 版 10 個技能（`analyze, apply, archive, audit, commit, discuss, drift, ingest, propose, verify`）與 Codex/.agents 版 8 個（再去 analyze、verify），移除 Spectra 的 `debug`、`ask`。內部技能 `sync/clarify/tdd/audit` 可經 `instructions --skill` 取用。

將 Spectra 技能本體套用相同品牌轉換後與 Speclink 技能逐字 diff：

| 技能 | 差異性質 |
|---|---|
| analyze, audit, drift, verify, sync, clarify, tdd | **純品牌替換，逐字相同** |
| commit | 品牌 + 範例路徑 `docs/specs/`→`openspec/`（一致化） |
| apply | 刻意手術：移除 park/unpark 選取與 unpark 步驟、移除 parallel_tasks/`[P]` 派工、修正 dormancy 的 `docs/specs/changes`→`openspec/changes` 路徑 bug |
| ingest | 刻意手術：移除 parked 選取與 `[P]` 保存 |
| propose | 刻意手術：移除 park 步驟（不再 park）、移除 parallel_tasks；**新增「從 discuss 文件產生提案」來源** |
| discuss | **強化**：新增討論文件記錄機制（見 §8） |

技能 frontmatter（唯讀 fork 的 `context: fork`/`agent: Explore`/`disallowedTools`）、`{{SPEC_DIR}}`/`{{PLAN_DIR}}`/`{{TOOL}}` 佔位符替換、`/speclink:`→`/speclink-`（Claude）與 `$speclink-`（Codex）前綴轉換，均與 Spectra 機制一致。

## 6. 設定系統與 init 產物

- **三層設定**：`.speclink.yaml`（應用）、`openspec/config.yaml`（工作流）、全域設定，運作與 Spectra 相同。`instructions` 於呼叫當下即時讀取並注入 `locale`（`tw`→繁中、`ja`→日文、其他→英文）、`context`（原文）、`rules`（依產物過濾）——經實測注入結果與 Spectra 一致。
- **保留鍵**：`spec_dir, locale, tdd, audit, tools`。**移除鍵**：`parallel_tasks, worktree, worktrees_dir, claude_effort, claude_slash_commands`（依需求）。
- **init 檔案樹**：與 Spectra 結構相同，唯一差異是移除的技能（debug、ask）不生成。
  - `openspec/config.yaml`、`.gitignore`、`.claude/settings.json`：**逐字相同**。
  - `.speclink.yaml`、`CLAUDE.md`/`AGENTS.md`：品牌 + 刻意移除段落（vector index 註解、parallel_tasks/worktree/claude_effort 註解、Parked Changes 段落、ask 技能行）。
- **內部儲存**：Spectra 用 SQLite（`.spectra/spectra.db`）；Speclink 因移除 park/unpark，改用 `.speclink/` 下的 JSON 檔（`in_progress.json`、`touched/<change>.json`、`snapshots/<archived>/created_specs.json`）——CLI 輸出對齊，儲存實作簡化。

## 7. 資料模型與生命週期

`openspec/` 目錄、`specs/`（正典）vs `changes/`（delta）、`.openspec.yaml` 變更中繼、四種 delta 操作（ADDED/MODIFIED/REMOVED/RENAMED）、`#### Scenario:` 剛好 4 井字號、`##### Example:` SBE、archive 合併語意與 `@trace` 注入——全部沿用 Spectra/OpenSpec 模型並經生命週期對照驗證一致。

## 8. 唯一的流程差異：Discuss 強化

Spectra 的 discuss 是唯讀、不留文件的討論。Speclink 讓 discuss 具**延續性**：

- 新增 `discuss` CLI 指令群（`new/list/show/add-round/conclude`），把每次討論持久化為 `openspec/discussions/<slug>/discussion.md`（frontmatter 記 status、逐輪 `## Round N`、最終 `## Conclusion`）。
- discuss 技能保留 Spectra 的**相同步驟邏輯**（Step 0 讀 LANGUAGE.md、關鍵字掃原始碼、Assumptions/Interview 模式、介面深度檢查、收斂、捕捉結論），但每輪與收斂時透過 CLI 持久化到文件。
- propose 技能新增第三種需求來源「**從 discuss 文件**」（優先序：argument → discussion → plan → 對話），可 `/speclink:propose --from-discussion <slug>` 直接以討論結論播種提案。

此差異即需求所述「discuss 可留下迭代討論文件、並可從中產生 propose」，其餘流程與 Spectra 一致。

## 9. 對抗式邊界審計與修正歷程

除了 §3–§7 的正向對照，另以**多輪平行對抗式審計**主動搜尋邊界情境的不一致：每輪派 5 個獨立代理，各鎖定一個面向（delta 操作 MODIFIED/REMOVED/RENAMED、錯誤訊息/空狀態/exit code、locale/config 注入、多能力/生命週期旗標、瀏覽/schema/子指令 help），在乾淨沙箱以相同輸入跑兩個 CLI 並語意比較，只回報**非刻意**的差異。

- **第一輪**：128 檢查，43 項真實不一致。涵蓋 archive 的 MODIFIED/REMOVED 未就地套用、analyze 正典路徑少一層、`show <spec>` 完全失效、多能力 archive 未聚合、locale 未對映碼行為、大量錯誤訊息措辭、`schema which/validate` 缺 `--json`、`list --specs --json` 結構、`instructions apply` blocked 結構等。
- **第二輪**：殘留 ~22 項，揪出更深的 bug：@trace 路徑掉首字元（`util::git` 對 porcelain 輸出 `.trim()` 誤刪 ` M path` 前導空格致欄位左移）、ADDED 重複既有需求、analyze 不應抑制 REMOVED、`task done` 已完成應報錯、validate「有需求但無操作」應判 error、locale 亦讀 `openspec/config.yaml`、@trace 應排除工具目錄。
- **第三輪**：殘留多為更細邊界：`new change` 缺 kebab-case 驗證、`new artifact --json`/`task done --json` 應為 compact 單行且鍵集不同、`instructions` 無參數時預設應取「顯示順序第一個未完成」artifact、instructions 人類輸出應為 `Description:`＋`Dependencies:`、多個錯誤訊息措辭與尾端換行。
- **第四輪**：零操作/malformed delta 邊界：RENAMED-only 與空操作段落不算 delta spec、`new artifact tasks` 全 `[x]` 應拒絕、Coverage 為 contiguous 子字串比對、`gapNoMainSpec`/`gapModifiedNotFound`、`task done` 先檢查 tasks.md、show 空行。
- **第五輪**：`covMissingSpec`（反引號、`cap` param）、Coverage skip 條件、`schema which/validate` 與 `templates` 對未知 schema 的處理（which 為 exit 0「Not found.」）、`show --item-type` 型別特定錯誤。
- **第六輪**：Coverage skip 精修（需 proposal +（specs 或 tasks））、`gapNoProposal`、drift 對未完成變更的維度文字（「design absent」/「no tasks.md」）與 light/medium 建議（技能斜線指令）、`new change --agent` 寫 `created_with`、多變更錯誤措辭與 mtime 排序、show 無 proposal 仍顯示 delta specs。**此輪 locale-config 與 multi-cap-lifecycle 兩個面向已 0 發現。**
- **第七輪**：僅 4 項——archive 的 MODIFIED 若不在 base spec 應跳過（不 materialize）、`gapModifiedNotFound` params 只用 `{name}`、`new artifact` 應先驗型別再驗變更、locale 空白值原樣保留。**此輪 multi-cap-lifecycle 與 browse-schema 亦 0 發現**，locale-config 僅剩空值一項。四項全數修正。
- **第八輪**：6 項——analyze 空變更（剛 `new change` 後）誤觸 `gapNoProposal`（應 skip Gaps 並省略空的 `Analyzed:` 行）、archive 刪除最後一條需求時 spectra 會留 dangling `---`（本輪一併複製以達 byte-parity）、`config list` 應為 `key = value` 且支援 `--json`、`completion install` 應收 `--verbose`。**locale-config 與 multi-cap-lifecycle 再次 0 發現**。六項全數修正。
- **第九輪**：補齊兩項功能落差——`instructions` 的 `unlocks`（下游 artifact，即「本 artifact 為其最後一個未滿足依賴」者；JSON + 人類「Unlocks:」區塊）與 `completion generate` 產生**真正的 clap 完成腳本**（原為 stub）；另修多變更措辭依指令而異（analyze/drift 用「Specify one:」、`--change` 指令用「Use --change to specify one:」）、`new artifact` 無 `--change` 時先 auto-detect。**multi-cap-lifecycle 再度 0 發現**。
- **第十輪**：`list` 對 0 任務變更省略 `[done/total]` marker、`list --json` 空 summary 省略、`list` 依名稱字母序。**五個探測面向中 locale-config、multi-cap-lifecycle、browse-schema 三者皆 0 發現**。此輪另有兩個 finding 經直接對比 spectra 後**證實為稽核觀察錯誤**（spectra 的 `list` 預設是字母序而非 mtime；spectra 對手寫 RENAMED-only delta 的 `show` 其實會渲染 Delta Specs），已據實還原。

**收斂軌跡**：每輪真實不一致數為 43 → ~22 → ~22 → 8 → 9 → 8 → 4 → 6 → 5 → 4，且後段多輪各探測面向陸續歸零（第十輪僅 `list` 一類真實修正，其餘面向皆 0）。**所有真實/語意不一致均已修正並提交**（git 提交序列 `fix(cli): match Spectra help descriptions...` 至 `fix(cli): resolve round-10 audit findings`），並以聚焦自檢逐一驗證輸出與 spectra 逐字相符。最終以 release 執行檔全面回歸：8 個 demo 主題 × 8 個讀取指令（analyze/drift/validate/status/show/instructions/list，含 JSON 與人類雙模式）**64/64 一致**，完整對照套件 **31/31 一致**，皆無回歸。

十輪對抗式稽核（每輪 5 個平行代理、累計逾千項邊界檢查）後，殘留僅：(a) 刻意的品牌隔離差異（`config list` 全域設定路徑、`completion generate` 指令樹）、(b) spectra 自身非決定性的 `@trace code:` HashMap 排序（speclink 穩定排序，屬合理正規化）、(c) 手寫 RENAMED-only／空 delta spec 這類**經 CLI 正常流程無法產生**（`new artifact` 會拒絕）且 spectra 自身前後不一致的病態輸入。這些皆不影響任何正常 SDD 流程。

**已知非程式差異／病態邊界**（如實揭露、不影響正常 SDD 流程）：
- `config list` 讀取品牌各自的機器層全域設定（`~/.speclink/` vs `~/.spectra/`），內容視使用者先前設定而異；格式（`key = value`）與 `--json` 一致。
- `completion generate` 因指令集刻意不同（speclink 有 discuss、無 search/park/unpark），產生的腳本指令樹與 spectra 不同，但同為 clap 完成腳本、格式一致、功能完整。
- 對「存在但空／僅 RENAMED」的病態 delta spec，analyze 的維度 gating（specs 視為 present 與否）在 speclink 採 has-delta-operation 判定，與 spectra 的 has-content 判定在此極端輸入下略有差異；正常含 ADDED/MODIFIED/REMOVED 的 delta spec 兩者一致。

唯一非程式差異：`config list` 讀取品牌各自的機器層全域設定檔（`~/.speclink/` vs spectra 的 `~/.spectra/`），故若使用者先前用 spectra 設過全域鍵，其內容會不同；此為刻意的品牌隔離，格式（`key = value`）與 `--json` 完全一致。

**已知殘留**（純 cosmetic、不影響語意，如實揭露）：
- 歸檔正典 spec 在「刪除最後一條需求」時，spectra 會留下懸空的 `---` 分隔線（其文字拼接產物），speclink 產出較乾淨（無懸空 `---`）；需求內容本身逐字相同。
- 全域旗標 `--no-color` 在少數子指令 `--help` 中的相對位置受 clap 全域旗標機制限制，可能與 spectra 略異；旗標描述文字一致。

## 10. 端到端示範：HTML 彈珠檯

以 speclink 技能與 CLI 完整跑過 SDD 流程建出 `pinball/index.html`：

1. **discuss**：記錄討論 `openspec/discussions/html-彈珠檯遊戲設計/`（2 輪迭代 + 結論，status=concluded）。
2. **propose --from-discussion**：`pinball-game` 變更，proposal（繁中）/specs（英文規範）/design（繁中，含 Implementation Contract）/tasks（繁中，附手動驗證）；`analyze` 四維 Clean（Ambiguity 僅 Suggestion）、`validate` 通過。
3. **apply**：實作 Canvas 2D 彈珠檯（重力物理 + 子步進防穿牆、兩翻板即時切換角度、3 緩衝器 +100 分、發球道、3 球、Game Over/R 重開、HUD）；8 個任務逐一 `task done`，`@trace` code 乾淨記錄 `pinball/index.html`。
4. **archive**：6 條需求併入正典 `openspec/specs/pinball-table/spec.md`（注入 `@trace`）、建快照、移入 `changes/archive/`。

**可玩性驗證**：以模擬 DOM/Canvas 的 node harness 載入遊戲並驅動，確認 6 條需求全部成立——發球（含發球中忽略）、翻板按鍵時尖端上抬（y 599→526）、緩衝器計分（單局最高 1800 分）、落袋扣球至 0、Game Over、R 重開歸零、HUD 同步。

## 11. 未來階段：可插拔規格儲存（延伸願景）

本次交付為需求所述的**第一階段**（完整 CLI + discuss 強化 + 端到端示範 + 比較報告）。延伸願景是把「規格驅動引擎」與「文件儲存方式」解耦——讓文件可存為 md／DB／JSON／YAML，或串接 JIRA 等外部系統，進而支援「PO/PM 在客製化系統執行 discuss + propose + ingest + archive，RD/QA 在本地 git 執行 apply + verify」的分工。

Speclink 目前的設計已為此鋪路：
- **discuss 文件化**（§8）已把討論從「僅存於對話」變為持久化文件，是儲存解耦的第一步。
- 引擎邏輯（`speclink-core`）與 CLI（`speclink-cli`）分離，`paths` 模組集中管理儲存位置，未來可抽象為 storage trait（本地 fs / DB / REST）而不動 CLI 表面。
- `instructions` 於呼叫當下即時讀取設定並注入，天然適合「引擎即服務」模式。

此為後續階段工作，不在本次第一階段範圍。

## 12. 結論

Speclink 在**所有共同功能面**與 Spectra 2.3.1 的 CLI 邏輯、流程、輸出結果、技能內容、功能結構與流程邏輯**保持一致**（經多輪對抗式審計逐一對齊，殘留僅純 cosmetic 空白差異），且以 Rust 原生 workspace（`speclink-core` + `speclink-cli`）實作、單一執行檔散布（release 約 6.8MB，與 spectra 8.35MB 相當）。與 Spectra 的差異全部落在需求指定的取捨：移除 debug/ask/向量搜尋/worktree/park-unpark/parallel_tasks/claude_effort，並強化 discuss 為可記錄、可作為 propose 來源的延續性討論。端到端以 speclink 完整跑過 discuss→propose→apply→archive，建出可玩的 HTML 彈珠檯（`pinball/index.html`，經模擬 DOM 測試驗證六項需求全數成立），證明整條 SDD 流程可用。

## 13. 二次整體驗證（Fable 5 覆核）

在十輪對抗式審計之後，另以獨立視角對成品做整體覆核，發現並修正了審計遺漏的四項問題（提交 `859f3fe`）：

1. **跨平臺 bug（嚴重）**：`config` 全域設定路徑只讀 Windows 的 `%APPDATA%`，在 macOS/Linux 會 fallback 到當前目錄。已改為各平臺慣例（Windows `%APPDATA%`、macOS `~/Library/Application Support`、Linux `$XDG_CONFIG_HOME` 或 `~/.config`）。程式其餘部分（PathBuf 拼接、`str::lines()` 的 CRLF 容忍、`git` 子程序、`include_str!` 資產）皆為平臺中立，無其他平臺相依碼。
2. **completion install/uninstall 是假輸出**：原本印「✓ Completion installed」，但 spectra 實際上不寫入 shell profile，而是印三行指引（generate → source）。已照抄 spectra 的指引訊息與 unknown-shell 錯誤（elvish 有支援但錯誤訊息刻意不列，照抄）。十輪審計只比對了 `--help`，未比對實際輸出。
3. **`unlocks` 排序**：spectra 依顯示順序（拓撲層級+字母序，`["design","specs"]`），speclink 原依 schema 宣告順序。已修正。DAG 全狀態矩陣（5 狀態 × 4 artifact 的 dependencies/unlocks + status）重驗 **25/25 一致**。
4. **本專案 `openspec/config.yaml` 規則不全**：原僅 proposal/tasks 有 rules，已補 design/specs，四類 artifact 注入皆驗證有效。

另釐清兩個事實：

- **`.spectra` 儲存位置**：spectra 2.3.1 的常規狀態（snapshots、touched）放在**專案根目錄 `.spectra/`**——speclink 的根目錄 `.speclink/` 正確對應此行為。`.git/spectra-app/spectra.db` 是 **park/unpark 功能專用的 SQLite 資料庫**（連 `list --parked` 都會惰性建立），而 park/unpark 是本專案指定移除的功能，故 speclink 無需對應物。
- **`tdd`/`audit` 的作用機制**：實測 spectra 在 `tdd: true`/`audit: true` 時，`instructions apply` 的 JSON 與文字輸出**完全不變**——這兩個旗標是純技能層協議：apply 技能指示 AI 讀取 `.spectra.yaml`，若為 true 則呼叫 `instructions --skill tdd|audit` 取得紀律指令。speclink 的對應鏈（技能引用 + `--skill tdd|audit` 指令，經品牌正規化後與 spectra 逐字一致）完整可用。

**已知功能缺口（後續已補齊，見 §14）**：spectra 具有 `schema fork` / `schema init`（專案層自訂 workflow schema）；此缺口在第二階段驗證中被發現並已完整實作。

## 14. 第二階段：schema 客製化、discuss 強化、spec 語言（Fable 5）

**釐清 `.spectra` 儲存位置**：spectra 的常規狀態（snapshots、touched）在專案根目錄 `.spectra/`（惰性建立：只有 `task done` 與 `archive` 會建）；`.git/spectra-app/spectra.db`（SQLite，schema v15）僅存 park、share、worktree、ask 全文索引、archived_cache、in-progress 等——幾乎全是移除的功能。唯一保留功能 in-progress 在 speclink 以 `.speclink/in_progress.json` 檔案等價實現。

**自訂 workflow schema（完整實作，128/128 邊界一致）**：speclink 現在完整支援 spectra/OpenSpec 的 schema 客製化——`openspec/schemas/<name>/schema.yaml` + `templates/`，解析順序 project → user（`<設定目錄>/speclink/schemas`）→ built-in；`schema fork`（對內建來源傾印逐位元組相同的 yaml + 四個 template）、`schema init`（線性 requires 串鏈骨架，逐位元組相同）；循環相依偵測與 serde_yaml 錯誤訊息逐字一致。過程中發現並複製了多個 spectra 深層行為：payload 的 schemaName 用 yaml `name:` 欄位（fork 未改名時仍回報 spec-driven）、payload template 按 display name 查內建表（自訂名 → 空字串）而建檔恆用 `templates/` 檔案、**presence/done-ness 一律以檔案存在判定**（空檔算 done；此規則同時修正了 analyzer/status/show/validate/contextFiles 的多個空檔邊界）、analyzer 寫死古典四產物且不解析 schema、只有 status/instructions 解析 schema（analyze/show/validate/drift/task/archive 不解析）、validate 對「存在但零操作」的 delta 檔判 error 並以 exit 1 收尾。

**discuss 三項強化（speclink 專屬）**：`new change --from-discussion <slug>` 建立雙向連結（change 端 `from_discussion:`、討論端 `status: promoted` + `promoted_to:`）；`discuss promote <slug> [--name]` 一鍵從結論 scaffold change（proposal 的 Why 預填結論）；`archive` 時連動把 promoted 討論搬入 `discussions/archive/<slug>/`。propose/discuss 技能已同步更新。

**spec 語言可設定（刻意偏離 spectra）**：spectra 強制 spec 檔一律英文；speclink 改為 `.speclink.yaml`／`openspec/config.yaml` 的 `spec_locale`（未設 → 英文、`auto` → 跟隨 `locale`、任何語言碼 → 該語言），結構性標記（`### Requirement:`、`#### Scenario:`、WHEN/THEN、SHALL/MUST）恆為英文以保工具可解析。與 tdd/audit 相同為技能層協議；specs instruction、propose/ingest 技能與 fork 傾印文字已同步。此為刻意差異，對照工具已將該段文字列入已知正規化。

第二階段回歸：8 主題 × 6 指令 48/48、完整對照套件 31/31、自訂 schema 邊界 128/128。

## 15. 儲存層對齊：`.git/speclink-app/speclink.db`（SQLite）

針對「內部儲存也應與 spectra 同一做法」的要求，對 spectra **全部 22 個指令**做了逐一副作用普查（每個指令執行後快照工作樹與 `.git/`），確定事實如下：

- CLI 建立的 `spectra.db` 只有**兩張表**（`parked_changes` + `in_progress_change`）——先前在使用者專案看到的 14 表資料庫是**桌面 App** 遷移擴充的（archived_cache、documents_fts、worktree 等全是 App/已移除功能專屬，CLI 從不讀寫）。
- 唯一由 CLI 寫入 DB 的保留功能是 `in-progress add`：惰性建立 `.git/spectra-app/`（**非 git 專案會自行建立 `.git/` 目錄**）、寫入 `.migrate.lock` 與 db、靜默、冪等、不驗證變更存在、多筆共存、**archive 不清除標記**。
- 其餘指令的副作用與 speclink 完全一致（唯二差異是刻意移除的 ask/debug 技能檔）。

speclink 據此重做儲存層：`in-progress add` 現在寫入 `.git/speclink-app/speclink.db`，bootstrap DDL 與 spectra **逐位元組相同**（含 sqlite_master 中的縮排；`parked_changes` 表為結構相容而建、功能仍移除），`.migrate.lock` 時機一致，非 git 專案行為一致，archive 不再清除標記（修正了先前的自創行為）。舊 `.speclink/in_progress.json` 會在首次開啟 DB 時自動匯入並刪除（僅實際遷移時寫 `.migrated`，與 spectra 的遷移標記語意一致）。`.speclink/`（touched、snapshots）維持不變——那本來就是 spectra 的檔案式做法。

驗證：DDL 逐字比對相同、app 目錄檔案集合相同、副作用普查 1:1、8 主題與完整對照套件全綠。依賴新增 rusqlite（bundled SQLite，維持單一執行檔、跨平臺）。

## 16. init/update 行為對齊（工具範圍決定：claude + codex）

針對「init 缺少自動補齊 CLAUDE.md/AGENTS.md 這類功能」的觀察做了逐情境 GT 比對。spectra 實際支援六種工具（claude/codex/cursor/gemini/windsurf/copilot，各有專屬產物形態），比對過程中曾完整實作並通過 26 項結構驗證；**最終依專案決定將支援範圍收斂為 claude + codex**（其餘工具為刻意不支援，`--tools cursor` 會明確報錯「unknown tool: cursor (supported: claude, codex)」——此為與 spectra 的刻意差異之一）。

保留下來的行為修正（皆與 spectra 一致）：

- **re-init 防護**：已初始化（openspec 目錄或 .speclink.yaml 存在）時報「Already initialized. Use --force to reinitialize.」。
- **既有指示檔的補齊**：無標記的既有 CLAUDE.md/AGENTS.md → 標記區塊**前置插入**、使用者內容保留（原實作為後附，已修正）；有標記 → 就地更新。
- **輸出對齊**：init 顯示「✓ Initialized at <路徑參數原樣>\openspec」＋「Generated files for: <工具清單>」；update 顯示「✓ Updated instruction files for: <偵測到的工具>」或「! No AI tool configurations found. …」。
- **update 偵測規則**：以點目錄存在為準（.claude），**codex 不在 update 範圍**（spectra 的實際行為，照抄）；fork/agent/disallowedTools frontmatter 為 Claude 專屬（修正了原本 codex 的 SKILL.md 多印的問題）。

## 17. 發佈管線（GitHub Actions）

- `.github/workflows/ci.yml`：push/PR 觸發，於 ubuntu/macos/windows 三平臺建置 release binary 並跑 smoke 流程（init → new change → new artifact → status → validate → list → schemas → update），已在本機逐步演練通過。
- `.github/workflows/release.yml`：推送 `v*` 標籤觸發，矩陣建置五個目標（Windows x64 MSVC、Linux x64 gnu（ubuntu-22.04 保守 glibc 基線）、Linux arm64、macOS arm64、macOS x64（同機交叉編譯））、打包 zip/tar.gz、產生 SHA256SUMS、以 softprops/action-gh-release 建立 GitHub Release。rusqlite 採 bundled SQLite，各平臺 C 編譯器（MSVC/gcc/clang）皆為 runner 內建，無額外依賴。

## 18. 工作目錄（`.spectra`/`.speclink`）與 drift/archive 副作用對齊（Fable 5）

由「兩個工作資料夾內容不一致」的觀察觸發的深度追查，結論與修正如下。

### 誰寫了什麼（釐清）

| 檔案 | 寫入者 | 時機 | CLI 的角色 |
|---|---|---|---|
| `<work>/touched/<change>.json` | CLI `task done` | 工作樹有未提交變更時（乾淨樹不寫） | archive **不**刪；刪除是 archive skill 指示 agent 執行的步驟 |
| `<work>/snapshots/<date>-<name>/created_specs.json` | CLI `archive` | 僅當本次歸檔**建立**了新 canonical spec | 格式為裸陣列 `["cap-x"]`（capability 名、無尾端換行） |
| `<work>/snapshots/<date>-<name>/specs/<cap>/spec.md` | CLI `archive` | 僅當 delta 觸及**既有** canonical spec | 逐位元備份套用前的原文（unarchive 用） |
| `<work>/changes/<name>.started` | **Spectra 桌面 App（app.exe）** | 使用者在 App 中開始實作 change 時（內容為基準 commit SHA） | CLI 任何指令都**不寫**（22 指令沙盒實測）；`archive` 會刪除它（含 `--skip-specs`） |
| `.git/spectra-app/spectra.db` 的 13 張表 | **桌面 App**（migration 至 schema_version 15） | App 開啟專案時 | CLI 首次觸碰 db 僅建 2 張表（in_progress_change、parked_changes），再次實測確認 |

本專案 repo 中 `.spectra/` 只有 `.started`（App 寫的）、`.speclink/` 只有 snapshots/touched（speclink 流程寫的），差異來源是「誰在此 repo 執行過什麼」，非實作歧異——但追查過程發現以下真缺口。

### 發現並修正的 parity 缺口（先前 parity 沙盒無 commit、無 .started，皆走 fallback 分支而漏測）

1. **drift Environment 維度**：spectra 執行 `git log --since=<created> --pretty=format:COMMIT|%H|%at|%s --name-only` 並計數 COMMIT 記錄。注意 git 對純日期 `--since` 會以「當下時刻」補足缺少的時間欄位（approxidate），因此**當天建立的 change 幾乎永遠顯示 0 commits**——此怪癖照樣復刻。speclink 原本寫死 0。
2. **drift `last_commit`**：spectra CLI 在所有情境（有 commit、標題引用 change 名、commit 觸及 change 目錄、touched 檔案被改）皆為 `null`（App 端欄位）；speclink 原本回傳未過濾的 HEAD SHA。
3. **drift Tasks 狀態**：repo 有 commit 時 spectra 顯示 `0 blocked, 0 maybe-done`（無 commit 時 `git unavailable`，兩者以 `git log` 是否成功區分）；speclink 原本印 `no task collisions`。
4. **drift Time 狀態**：`.openspec.yaml` 無 `created` 時 spectra 顯示 `no created date`。
5. **archive 快照**：依上表規則重寫（原實作：無條件寫物件格式 `{"created_specs":[路徑]}`、無備份、無 delta 也建快照）。`Snapshot created for unarchive support.` 僅在快照實際建立時輸出（無 delta 的歸檔只印 `✓ Archived:` 一行）。
6. **archive 清理 `.started`**：新增（含 `--skip-specs` 情境）。
7. **canonical spec 尾端換行**：spectra 寫出的 canonical spec 以換行結尾；speclink 補齊（新建與合併兩路徑）。

### 驗證

- 雙沙盒 harness：8 情境（無 commit drift、有 commit＋回溯 created drift、created 缺失 drift、ADDED 歸檔＋.started、MODIFIED-only 歸檔、無 delta 歸檔、零效果 MODIFIED 歸檔、--skip-specs 歸檔）之 stdout、drift JSON、工作目錄樹與逐位元內容、canonical specs 全部一致。
- 完整 parity suite：31/31 通過。

## 19. discuss 版面重構（方案 A）與文件規則（Fable 5）

speclink 專屬功能（spectra 無對應物），依討論決議採「扁平化 + 補齊生命週期」：

### 版面

- 討論文件：`openspec/discussions/<slug>.md`（原 `<slug>/discussion.md` 的一夾一檔間接層移除）。
- 歸檔：`openspec/discussions/archive/<created>-<slug>.md` —— 與 `changes/archive/` 相同的日期前綴慣例；slug 因此可重用。同日同 slug 再歸檔時自動加 `-N` 後綴（co-archival 永不因撞名而失敗或靜默略過）。

### 生命週期補齊

- 新增 `speclink discuss archive <slug>`：手動歸檔「討論完決定不做」的討論（原本唯一歸檔路徑是隨 promote 的 change 連帶歸檔）。
- 新增 `speclink discuss list --archived`；`show`/`info`/`conclusion_text` 自動 fallback 到 archive（取最新版本）。
- 對已歸檔討論的寫入操作（add-round/conclude/promote）給出明確錯誤（區分「已歸檔」與「不存在」）；promote 的檢查提前到建立 change 之前，避免留下半成品。
- co-archival 輸出改為實際歸檔檔名：`Discussion archived: <slug> → discussions/archive/<created>-<slug>.md`。

### 文件規則（蘇格拉底式紀錄的結構化）

寫入 skill（討論進行規則原已完備：一次一問、具體選項、不空洞附和、收斂強制）與文件模板註解：

1. **每輪一個焦點**：一輪紀錄只蒸餾一個被檢驗的問題與其結論，不是逐字稿。
2. **Append-only**：不回改先前輪次；立場改變開新一輪，寫明改變了什麼、為什麼。
3. **記錄否決與理由**：被淘汰的選項連同淘汰原因入檔——防止未來重新翻案。
4. **未決問題帳本**：每輪以未決問題收尾、下一輪從中取題；結論必須逐一解決或明示延後（Deferred）。

round 範本改為 `**Focus** / **Position** / **Ruled out** / **Open**` 四欄；conclusion 範本增加 `**Rejected alternatives**` 與 `**Deferred**`。repo 內既有兩筆討論已遷移至新版面。驗證：8 項生命週期功能測試通過、parity suite 31/31。

### 補充：discuss 文件模板化（§19 續）

事實確認：spectra 的 discuss skill 本身即為蘇格拉底式方法（interview 模式＝一次一問、追問到具體、挑戰假設；assumptions 模式＝倒置變體「我列信念、你來反駁」），但**產出是揮發的**——對話結束後除非手動 capture 到 design/proposal，沒有任何紀錄留下。speclink 的 discuss skill 與 spectra 逐字比對確認：**進行規則零改動**（86 行差異全部為 speclink 新增的紀錄段落），差異僅在「多了持久化文件」。

文件本身比照 proposal 模板給固定骨架（`discuss new` 直接產出）：

```
## Context      ← 一次性框架：起因、模式選擇與原因、相關 changes/specs（discuss context 填入）
## Rounds       ← ### Round N — <mode> (<date>) 由 add-round 依序插入
## Conclusion   ← conclude 以內容「取代」佔位註解；再次 conclude 為修訂（永遠單一 section）
```

- 新增 `speclink discuss context <slug> --stdin`；`conclusion_text` 會剝除 HTML 註解（佔位註解不算內容，promote 的 Why 預填不會拿到註解）。
- add-round 在骨架文件中插入 `### Round N` 至 Rounds 區段尾端；舊版面（pre-scaffold）文件自動 fallback 為舊式尾端附加，讀取端同時容忍 `## Round`/`### Round` 兩種標題。
- repo 內既存的 live 討論已遷移至新骨架（歸檔者凍結不動）。

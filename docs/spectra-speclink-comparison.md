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

**已知功能缺口（如實揭露）**：spectra 具有 `schema fork` / `schema init`（專案層自訂 workflow schema，存於 `openspec/schemas/<name>/schema.yaml` + `templates/`，`schemas` 列為 `(project)`、`new change --schema` 可用）；speclink 目前對這兩個子指令回報「custom schema management is not supported in speclink」。此功能不在移除清單上，屬審計未覆蓋的缺口，列為下一階段工作（設計建議見 §11 的延伸方向：以 serde_yaml 載入 schema.yaml，解析順序 project → user → built-in，`fork` 即傾印內建 schema 為 YAML）。

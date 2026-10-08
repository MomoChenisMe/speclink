## Context

speclink-core 以 `include_str!` 內嵌 `crates/engine/speclink-core/assets/skills/` 下的 23 份技能原稿。技能 registry 把其中 20 份組成 19 個對外技能（apply-with-worktree 在編譯期由 apply-worktree-pre、apply、apply-worktree-post 三份串成），另外 3 份（sync、clarify、tdd）是經 `speclink instructions --skill` 取用的內部技能。`render_skill_file_for` 為每個渲染目標加上 frontmatter、前言與替換字，Claude、Codex、自訂描述子，以及 add-copilot-tool 合併後的 Copilot，都是每個技能寫一份 SKILL.md。

現況量測（2026-10-08，engine v1.42.0）：

| 原稿 | 行數 | 全大寫強調字 |
| --- | --- | --- |
| discuss.md | 484 | 15 |
| propose.md | 466 | 38 |
| apply.md | 336 | 16 |
| archive.md | 328 | 7 |
| commit.md | 299 | 11 |
| ingest.md | 297 | 21 |
| verify.md | 273 | 33 |
| review.md | 183 | 36 |
| 全部 23 份 | 4598 | 約 290（NOT 92、STOP 50、Do NOT 39、MUST 30、NEVER 26、SHALL 22、CRITICAL 17、IMPORTANT 12、ALWAYS 2） |

既有約束：

- 17 份逐技能規格與 skill-routing、verb-contract 規定技能必須寫到的內容；`tests/it/render_golden.rs` 約有 40 條測試、138 處逐字比對，把其中一部分規定釘成英文片語。
- `tests/it/skill_verbization.rs` 守「技能不直接讀規格目錄」；`scripts/docs/vocabulary-guard.test.mjs` 掃原稿的避用詞。
- 改原稿必須同批升 ASSET_VERSION、重產 golden 快照與 assets.lock（`embedded_assets_are_locked_to_the_product_version`）。
- `claude plugin eval` 能實跑技能並打分，只能跑 Claude 模型；本機沒有 codex、copilot、gemini CLI。
- 討論 skill-prompt-optimization 已決定：目標是讓大部分模型更照做、行為不變；19 個技能在一個 change 內一次改完；以寫法規範為主要標準，Claude 實跑評測（含 Haiku）與 Codex／Copilot 抽查為輔。

## Goals / Non-Goals

**Goals:**

- 訂出跨模型寫法規範，19 個技能的 20 份原稿全部依它改寫，而且行為不變。
- 規範中能機械檢查的部分由測試擋住，原稿不會再被補丁撐長。
- 改寫前後各跑一次評測，證明新版每一題都不比舊版差。
- 修正 14 處寫死的技能引用，讓 Codex 與自訂描述子版的引用寫法和其他引用一致。

**Non-Goals:**

- 不改技能的 frontmatter 與 description（入口路由屬 skill-routing）。
- 不改三個內部技能 sync、clarify、tdd 的內文；它們只受守門測試檢查，強調字上限以現況登記。
- 不把原稿拆成多個附檔（三種渲染目標都是單一 SKILL.md；拆檔要改技能目錄格式、過期探測與孤兒清理，另案）。
- 不新增引擎檢查：stable ID 位置與 add-round 路徑格式這類可機械檢查的錯，交給另外的引擎變更，也不寫進技能文字。
- 不自建驅動多家 CLI 的評測工具；其他模型只做手動抽查。
- 不改任何 CLI 指令、旗標與輸出，不涉及 config.yaml 與 .speclink.yaml 欄位。
- 不處理原稿中寫死的 `openspec/` 路徑（apply、commit、discuss、ingest、propose 共 14 處，只在自訂規格目錄時有影響），另案。
- 不涉及規格儲存或流程抽象；本變更與「storage 解耦的規格驅動引擎」方向無關。

## Decisions

### 寫法規範十條與出處

寫法規範以 skill-authoring 規格的 requirement 承載。前九條取 OpenAI 與 Anthropic 官方指南的交集，第十條來自技能的渲染機制：

1. 每條規則在同一份技能裡只完整陳述一次，寫在模型做那件事的步驟裡；其他位置需要時用一句話指向它。（OpenAI GPT-5.6 prompt guidance：state each rule once）共用的退路句「Asking the user」在 7 份原稿逐字相同；組合技能 apply-with-worktree 渲染後因此有兩段（pre 與 apply 各一段）——守門以原稿為單位檢查退路句，比照第 8 條以原稿計。
2. 規則之間不矛盾；有例外時寫明哪一條優先。（OpenAI：衝突規則造成的不穩定比缺少細節更嚴重）
3. 強調字只用在鐵律：做錯會毀損資料、無法還原、必須停下等使用者，或引擎會靜默誤讀的固定格式（例如 `[M]` 標記位置）。限量使用而不是全刪，因為能力較弱的模型有時需要較強語氣。（OpenAI：ALWAYS／NEVER 只給真正的不變式；Anthropic：語氣過強讓新版 Claude 過度觸發）
4. 一個概念一個詞。（Anthropic Skill authoring best practices：consistent terminology）
5. 依工具而不同的能力都附退路，例如 AskUserQuestion 不可用時改用純文字詢問並等待回覆、不能開子代理時自己執行。
6. 多步驟流程用編號步驟；判斷點寫成「條件 → 動作」；固定格式的產出附樣板；結尾附產出自我檢查清單。（Anthropic：workflows、checklists；OpenAI：completion bar）有多種模式的技能，清單放在它所檢查的模式結尾：audit 的 Standalone check 接在獨立流程（Phase 1～3）之後，後面才是 apply 取用的 Discipline Mode 與參考資料，不移到檔尾。
7. 不寫歷史：design 決策編號、日期、某條規則由哪個 change 加入。範例中的時間戳格式不算歷史。（Anthropic：avoid time-sensitive information）
8. 單一原稿檔不超過 500 行。（Anthropic：SKILL.md body under 500 lines；使用者裁定以單一原稿檔計，組合技能 apply-with-worktree 渲染後可超過）
9. 判斷標準的原文不為縮短篇幅而濃縮。（討論 architecture-improve-flow 排除過濃縮 friction 清單）
10. 渲染記號：技能互相引用一律寫成 `/speclink:<名稱>`，`--agent` 的值一律寫成 `{{TOOL}}`，提到 plan mode 的內容獨立成一行。四種渲染目標共用同一份原稿，`render_skill_file_for` 依目標替換：`/speclink:ingest` 在 Claude 與 Copilot 成為 `/speclink-ingest`、在 Codex 成為 `$speclink-ingest`、在自訂描述子成為 `speclink ingest`；自訂描述子另會整行刪掉提到 plan mode 的行。寫死的 `/speclink-ingest` 會讓 Codex 版留著錯誤的叫用方式。（來源：crates/engine/speclink-core/src/workspace/skills.rs 的替換表；add-copilot-tool 讓 Copilot 也走同一份原稿）

替代方案：只依 Anthropic 指南——技能也會被 GPT 與 Copilot 可選模型讀取，單一廠商的建議可能互相衝突（例如 Anthropic 建議弱模型可用較強語氣、OpenAI 建議只給不變式），取交集才對多家都成立。

### 守門測試的五項機械檢查

新增 `crates/engine/speclink-core/tests/it/skill_authoring.rs`（掛進 `tests/it/main.rs`），以 `read_dir` 列出 assets/skills 下所有 `.md` 原稿（依檔名排序），逐份檢查：

1. **行數**：`lines().count()` ≤ 500（`lines()` 不分 LF／CRLF，Windows checkout 不誤判）。
2. **強調字上限**：以正規式 `\b(MUST|NEVER|SHALL|ALWAYS|IMPORTANT|CRITICAL|STOP|NOT)\b`（大小寫敏感）計次，次數 ≤ 測試內登記表的上限。
3. **不寫歷史**：先移除 fenced code block 與行內程式碼，再找前後不接數字的 `\d{4}-\d{2}-\d{2}`（`(?:^|[^0-9])(\d{4}-\d{2}-\d{2})(?:[^0-9]|$)`；不用 `\b`，因為 Unicode 字詞邊界下緊貼中文的「於2026-08-10加入」抓不到）；全文找 `design D\d+` 與 `design 決策`。
4. **工具退路**：原稿含 `AskUserQuestion` 時必須含 `plain text`（不分大小寫）；含 `sub-agent`、`subagent` 或 `Agent tool` 時必須含 `cannot spawn` 或 `can't spawn`。不收 `inline`：review 的「inline back until a real need shows」這類無關句子也含它，會讓刪掉真正的退路句後守門仍綠燈。
5. **渲染記號**：找前一個字元不是英數字、底線、`.`、`/` 或 `:` 的 `/speclink-` 加小寫字母（`.claude/skills/speclink-apply` 這類路徑不命中），以及 `$speclink-`（後接任何字元都算，含 `$speclink-<名稱>`）與 `--agent (claude|codex|copilot)\b`。

propose 期以同一套規則模擬現況：前四項 23 份原稿全部通過；第五項命中 drift（8 處）、archive（4 處）、apply（1 處）、commit（1 處）共 14 處，由任務 3.2 一併改成 `/speclink:<名稱>`，所以守門測試從第一天就是綠燈。

任一項不符時測試失敗，訊息指名原稿檔名、違反的項目，並寫明出處「skill-authoring 規格『技能原稿的跨模型寫法規範』」。替代方案：放進 render_golden.rs——該檔已有兩千多行、管渲染產物，寫法守門檢查的是原稿本身，獨立一檔較清楚；比照 skill_verbization.rs 的既有做法。

### 強調字上限的登記方式

測試內一張 `[(檔名, 上限)]` 常數表，涵蓋全部 23 份原稿。改寫完成後，把每份的實際次數登記為上限；三份內部技能登記現況次數。之後只能降低：多一個強調字就紅燈，要提高必須改這張表，讓提高成為看得到的刻意決定。assets/skills 出現表上沒有的原稿時也紅燈，要求登記。替代方案：全域固定比例（例如每百行 5 個）——各技能的鐵律密度天生不同（review 的停頓點多），單一比例不是太鬆就是太緊。

### 評測套件的位置與載入方式

評測套件放在 `integrations/claude-code/skill-evals/`，是一個只有 manifest 與 evals 的 Claude Code plugin：

- `.claude-plugin/plugin.json`：name 為 `speclink-skill-evals`。
- `evals/<題目名>/`：`prompt.md`（frontmatter 含 name、`tags: [skill:<技能短名>]`、max_turns、timeout_seconds、allowed_tools，本文為提示）、`case.yaml`（`context.scaffold_script` 指向同目錄的 `scaffold.sh`）、`graders/*.md`。
- 技能版本與 CLI 版本由指定的 `speclink` 執行檔一起決定。探測證實評測工作區不載入專案技能，所以採用「plugin 的 skills/」做法：執行腳本先在暫存目錄以該執行檔 `init --tools claude`、`workflow-config set worktree true`、`update`（worktree 政策開啟才會寫出 apply-with-worktree 與 worktree-merge，共 19 個技能），再把 repo 的 manifest 與 `evals/` 複製成 `target/skill-evals/.staging/<label>/plugin/`，並把 `.claude/skills/speclink-*` 放進它的 `skills/`；評測對這份副本執行。受測模型以 `/speclink-<短名>` 叫用時載入的是 `speclink-skill-evals:speclink-<短名>`。舊版與新版只差用哪一個執行檔渲染技能、PATH 上放哪一個執行檔。
- 每題的 scaffold.sh 在空白工作區執行 `speclink init . --tools claude`，再刪掉工作區的 `.claude/skills`，讓受測模型只看到 plugin 的技能。
- scaffold.sh 只依賴 PATH 上的 `speclink` 與 `git`；用到 git 時先在工作區內 `git init` 並設定本地 user.name／user.email（評測的 HOME 是臨時目錄）。

替代方案：在工作區 init 產生專案技能，由受測模型直接讀取——探測（1.1）顯示評測的 `claude -p` 不載入工作區的 `.claude/skills/`，技能清單沒有任何 speclink 技能，模型回覆「/speclink-discuss 沒有安裝」。把渲染好的技能直接寫進 repo 的 `integrations/claude-code/skill-evals/skills/` 也不採用：會在 repo 留下未追蹤的產物，舊版與新版的 run 也會互相覆蓋。

### 評測執行腳本與新舊比較

新增 `scripts/claude-code/skill-evals.mjs`，兩個子指令：

- `run --speclink <執行檔> --model <模型 ID> --label <名稱> --max-cost-usd <金額>`：把執行檔以 `speclink` 之名放進 `target/skill-evals/.staging/<label>/bin/`，把該目錄放到 PATH 最前面，先把 `speclink --version` 的輸出寫進 `target/skill-evals/<label>/speclink-version.txt`，以該執行檔建立 `target/skill-evals/.staging/<label>/plugin/`（見「評測套件的位置與載入方式」），再執行 `claude plugin eval target/skill-evals/.staging/<label>/plugin --scaffold --ablation none --runs 3 --model <模型 ID> --judge-model claude-sonnet-5-5 --allow-tools Bash Edit Write 'Edit(//private/tmp/**)' 'Write(//private/tmp/**)' 'Edit(//tmp/**)' 'Write(//tmp/**)' --trust-plugin --no-publish --threshold 0 --max-cost-usd <金額> --output-dir target/skill-evals/<label>`；`--case <題目>` 原樣轉交 claude，供退步時單題重跑；`--runs <次數>` 預設 3，只在驗題（每題 1 次）時改。exit code 沿用 claude（0 完成、2 花費上限或認證失敗）。claude 子行程的 stdin 設為 ignore、環境帶 `NO_COLOR=1`（規格要求兩個子指令不讀 stdin、輸出不帶顏色）。暫存目錄放在 `.staging/` 底下：label 不能以 `.` 開頭，所以重建暫存目錄時不會刪到任何 label 的結果目錄（例如 label `plugin-old` 的結果）。啟動前先檢查參數（sharp-edges 稽核，任務 1.4），任一項不符就在 stderr 說明並以 1 結束、不啟動評測：`--label` 只收英數字、`-` 與 `_` 且以英數字開頭（含 `/` 或 `..` 會寫出 `target/skill-evals/` 之外）；`target/skill-evals/<label>/` 已有 aggregate-result.json 時拒絕（不覆蓋付費跑出的結果）；`--max-cost-usd` 必須是正數；`--runs` 必須是 1 到 50 的整數；`--speclink` 必須指向存在的檔案。腳本以參數陣列啟動子行程、不經過 shell，所以執行檔路徑含空白不是問題。
- `compare <舊結果目錄> <新結果目錄>`：讀兩邊的 `aggregate-result.json`，逐題印出「題目、舊分數、新分數、差值」。任一題新分數低於舊分數時 exit 1；兩邊題目集合不同時 exit 2 並列出缺少的題目；否則 exit 0。無法比較時 exit 3 並在 stderr 指名該檔案：用法錯、結果檔不存在或不是 JSON，或結果因花費上限而不完整（頂層 `partial` 為 true，或某次執行的 `skippedPaidGraders` 為 true——那次略過了 llm 評分，分數不能和完整結果比）。錯誤不用 exit 1，自動化流程才分得出「有退步」與「沒有比較結果」。比較邏輯匯出為純函式 `compareResults(oldDoc, newDoc)`，回傳 `{ rows, regressions, missing }`。

新增 `scripts/claude-code/skill-evals.test.mjs`：

1. `compareResults` 的單元測試：退步、同分、進步、題目缺少四種情況；`compare` 子指令以暫存目錄裡的結果檔實際執行，逐一驗證 exit 0／1／2／3 與輸出（規格 Example「比較的判定」一列一個 test）。
2. 涵蓋度守門：repo 內 `.claude/skills/speclink-*` 的每個技能短名，都至少有一題的 tags 含 `skill:<短名>`。

結果寫在 `target/` 底下，已被 `.gitignore` 忽略，不進版本控制。執行腳本本身需要 claude CLI 與使用者額度，不在 CI 執行；只有兩項單元測試隨 `node --test scripts/*/*.test.mjs` 進 CI。

替代方案：在 shell script 裡用 jq 比較——CI 的 Windows runner 與開發機不保證有 jq，且無法單元測試。

### 19 題評測的題目表

每題只驗單輪（評測工具只支援 history_file，不支援多輪劇本）。提示一律用斜線指令形式；allowed_tools 為 `[Read, Glob, Grep, Bash, Edit, Write, Skill, Agent]`。題目同時併入由技能文字造成的已知失誤：propose 的 `[M]` 標記位置、apply 不代勾 `[M]` 任務、commit 的確認關卡。

| 題目 | 技能 | 前置（scaffold.sh） | 提示 | 打分規則 |
| --- | --- | --- | --- | --- |
| discuss-no-record-before-reply | discuss | init | `/speclink-discuss 看板要支援搜尋` | tool_used Bash input_match `discuss new` max 0；llm：回覆以向使用者提問或請使用者確認假設結尾 |
| improve-scan-without-edits | improve | init；`src/a.js` 與 `src/b.js` 含同一段重複函式 | `/speclink-improve` | tool_used Edit max 0、Write max 0；llm：列出改進候選並請使用者挑選 |
| propose-from-discussion | propose | init；以 discuss 動詞建立並結論 `search-bar` 討論，Decision 含一項需人工驗收的步驟 | `/speclink-propose --from-discussion search-bar（直接使用這份討論，change 名稱用 add-search-bar）` | regex（`openspec/changes/add-search-bar/tasks.md`）含 `- \[ \] \[M\] `；同檔不含 `- \[ \] \d+(\.\d+)* \[M\]`；tool_used Bash input_match `speclink validate` min 1；tool_used Skill input_match `speclink-apply` max 0 |
| apply-leaves-manual-task | apply | init；change `add-greeting` 含 proposal、spec 與兩個任務：1.1 建立內容為 hello 的 `greeting.txt`、`[M]` 1.2 打開檔案確認內容 | `/speclink-apply add-greeting` | file_exists `greeting.txt`；regex（tasks.md）含 `- \[x\] 1\.1`；同檔不含 `- \[x\] \[M\]` |
| apply-with-worktree-isolates | apply-with-worktree | git init＋首次 commit；init；`workflow-config set worktree true`；change `add-greeting` 並 commit | `/speclink-apply-with-worktree add-greeting` | tool_used Bash input_match `git worktree add` min 1；tool_used Bash input_match `git merge` max 0；llm：結尾把合回交給 worktree-merge |
| worktree-merge-lands-branch | worktree-merge | git init；init；依 apply-with-worktree 原稿的慣例（分支 `speclink/add-greeting`、路徑 `<repo-parent>/<repo-folder>.worktrees/add-greeting`）建立 worktree 並在其中 commit 完成的變更 | `/speclink-worktree-merge add-greeting` | tool_used Bash input_match `git merge` min 1；tool_used Bash input_match `git worktree remove` min 1 |
| archive-folds-delta | archive | init；change `add-greeting` 任務全勾、含 capability `greeting` 的 ADDED delta | `/speclink-archive add-greeting` | regex（`openspec/specs/greeting/spec.md`）含該 requirement 名稱；tool_used Bash input_match `speclink archive` min 1；tool_used Edit 與 Write input_match `openspec/specs` 各 max 0 |
| commit-gate-shows-plan | commit | git init＋首次 commit；init；change `add-greeting` 以 task done 記錄 `greeting.txt`；另改動無關的 `notes.txt` | `/speclink-commit add-greeting` | tool_used Bash input_match `git commit` max 0；llm：計畫列出 greeting.txt、不含 notes.txt，並請使用者確認 |
| analyze-reports-only | analyze | init；`speclink demo` | `/speclink-analyze <demo 建立的 change 名稱>` | tool_used Edit max 0、Write max 0；llm：回報涵蓋度或一致性發現 |
| audit-finds-dangerous-default | audit | git init；`src/http.js` 以 `rejectUnauthorized: false` 為預設值 | `/speclink-audit src/http.js` | tool_used Edit max 0、Write max 0；regex last_message 含 `rejectUnauthorized` |
| baseline-map-before-specs | baseline | init；`src/` 下兩個小模組 | `/speclink-baseline` | file_exists `openspec/specs/**/spec.md` exists false；llm：先呈現 capability 對照並請使用者確認 |
| config-asks-before-writing | config | init；`src/` 下一個小模組 | `/speclink-config` | tool_used Bash input_match `workflow-config (set\|context\|rules)` max 0；llm：一次只問一個政策欄位，或呈現待核准的 diff |
| drift-reports-only | drift | git init；init；change 的 design 提到 `src/old.js`，之後 git mv 成 `src/new.js` 並 commit | `/speclink-drift <change 名稱>` | tool_used Edit max 0、Write max 0；llm：指出 `src/old.js` 已不存在 |
| ingest-folds-new-requirement | ingest | init；change `add-greeting` 進行中 | `/speclink-ingest add-greeting 新需求：greeting.txt 第二行要加上中文問候「你好」` | regex（`openspec/changes/add-greeting/specs/greeting/spec.md`）含 `你好`；tool_used Bash input_match `speclink validate` min 1 |
| manual-generates-pages | manual | init；封存一個建立 `greeting` 正式規格的 change | `/speclink-manual 產生操作手冊` | file_exists `openspec/manual/*.md` |
| quality-pauses-after-round | quality | git init；init；change `add-greeting` 規定 `greet(name)` 回傳 `hello, <name>`、空字串回傳 `hello`；程式任務全勾、`src/greet.js` 已 commit，但條件式把空字串指派給 name（`if (name = '')`），違反規格也是程式錯誤 | `/speclink-quality add-greeting` | tool_used Bash input_match `(review\|verify) stamp` max 0；llm：兩站的發現一起回報後停下等使用者 |
| review-freezes-scope | review | 同 quality-pauses-after-round | `/speclink-review add-greeting` | tool_used Bash input_match `review scope` min 1；tool_used Bash input_match `review stamp` max 0；llm：呈現發現與選項後停下 |
| verify-freezes-scope | verify | 同 quality-pauses-after-round | `/speclink-verify add-greeting` | tool_used Bash input_match `verify scope` min 1；tool_used Bash input_match `verify stamp` max 0；llm：呈現發現與選項後停下 |
| trace-cites-archive | trace | init；封存一個建立 `greeting` 正式規格的 change `add-greeting` | `/speclink-trace greeting 這個能力是怎麼來的？`（技能要的是問題，只給能力名時模型會先請使用者改問） | tool_used Edit max 0、Write max 0；regex last_message 含 `add-greeting` |

驗題（任務 2.1、2.2）時定下的寫法：

- 各題的 scaffold.sh 以 `source` 載入 `evals/fixtures.sh` 共用片段（init、git 身分、add-greeting 的 artifact 與完成狀態），避免 19 份重複的 change 檔。`EVAL_TOOLS` 預設 claude；手動抽查時設成 codex 或 copilot，技能留在工作區給該工具讀（任務 7.1、7.2）。
- review、verify、quality 三題的程式要有真的錯誤：單站在第一輪沒有必修發現時會直接蓋章，用無錯的實作就測不到「停下等使用者」。
- 「max 0」的打分規則一律同時寫 `min: 0`（見 Open Questions）；`input_match` 是正規式，比對 `git -C <路徑> merge` 這類寫法時用 `git (-C \S+ )?merge(?!-)`。
- macOS 的 `/usr/bin/git` 是 xcrun 轉接程式，它要寫系統暫存目錄的快取而被評測沙箱擋下；真正的 git（`xcrun --find git`，位於 `/Library/Developer/CommandLineTools`）沙箱也讀不到，`DEVELOPER_DIR` 這類環境變數也傳不進受測模型的 shell。執行腳本因此把真正的 git 複製到 `target/skill-evals/.staging/<label>/bin/git`，與受測執行檔同一個 PATH 目錄；它只連結系統函式庫，status、add、commit、worktree、merge、rebase 等內建子指令不需要 git-core 目錄。
- 評審模型用 `claude-sonnet-5-5`：驗題時預設的 Haiku 評審對明顯符合打分規則的回覆（列出發現並請使用者選擇）連投三次 FAIL，同一題改用 Sonnet 評審即三票 PASS。
- `--allow-tools` 另加暫存根目錄的 Write／Edit 路徑規則：worktree 依慣例建在工作區旁邊（`<repo-parent>/<repo-folder>.worktrees/`），只給 Write／Edit 時受測模型在 worktree 內寫檔會被拒。

未收成題目的已知失誤與理由：add-round 路徑帶行號、stable ID 不在行尾——屬引擎可檢查的錯，另案；品質關卡子代理卡住——需要數萬字元的 patch 才重現，單輪評測做不到；蓋章前先讓 CI 跑綠——屬使用者的流程習慣，不是技能文字造成。

### 舊版基準與新版評測的跑法

1. add-copilot-tool 合併後、動任何原稿前：`cargo build -p speclink-cli`，確認 `target/debug/speclink --version` 的 engine 版號等於當下的 ASSET_VERSION，再複製成 `target/skill-evals/speclink-old`（之後的 cargo test 可能把 target/debug/speclink 重連到舊快取，所以馬上複製）。
2. 每題先用舊版在 claude-haiku-5-5 跑 `--runs 1` 驗題：分數為 0 時看原因，scaffold 或打分規則寫錯就修題目；技能本身做錯就保留（它正是改寫要改善的地方）。
3. 舊版基準：`run --label old-opus --model claude-opus-5-5` 與 `run --label old-haiku --model claude-haiku-5-5`。
4. 改寫與升版完成後：`cargo build -p speclink-cli`，確認 engine 版號為升版後的值，用 `target/debug/speclink` 跑 `new-opus` 與 `new-haiku`。
5. `compare` 兩組。某題退步時，用 `--case <題目>` 新舊兩版各重跑一次；仍退步才算真退步，修改該技能後重跑該題。
6. 每次 run 前向使用者確認 `--max-cost-usd` 的金額。四次完整 run 約為 19 題 × 3 次 × 4 ≈ 230 次執行，加上驗題與重跑約 250 次。

### 原稿改寫的順序與行為不變的檢查

20 份原稿分七批改寫：討論類（discuss、improve）、提案類（propose、ingest）、設定與文件類（baseline、config、manual）、實作類（apply、apply-worktree-pre、apply-worktree-post、worktree-merge）、品質關卡類（review、verify、quality）、檢查與查詢類（analyze、audit、drift、trace）、收尾類（archive、commit）。每一份的步驟：

1. 列出這份技能必須保留的內容：對應的逐技能規格每條 requirement 與 scenario、skill-routing 的交棒句、verb-contract 的讀取規則、render_golden.rs 中針對它的逐字比對。
2. 依十條規範改寫。
3. 跑 `cargo test -p speclink-core --test it`（render_golden 的逐字比對、skill_verbization、skill_authoring）與 `node --test scripts/docs/vocabulary-guard.test.mjs`。
4. 逐條核對步驟 1 的清單仍成立。

改寫期間 golden 快照與 assets.lock 會紅，這是預期；它們留到「版號、golden 與 assets.lock 一次重產」才處理，各批只看逐字比對與守門測試。

改寫時依第 2 條（規則不矛盾）處理的既有矛盾，各自只改到讓同一份技能前後一致：

- audit：frontmatter 的 description 寫「reports the sharp edges it finds」，Claude 版另有 `disallowedTools: [Edit, Write]`，本文 Phase 3 卻寫「If fixable: apply the fix directly」。改為獨立模式只回報並寫出修法、不改檔；apply 內的紀律模式照舊邊做邊修。Claude 版的實際行為本來就無法改檔，Codex、Copilot 與自訂描述子版從此與 description 一致。評測題 audit-finds-dangerous-default 量的就是這一點。
- commit：7a-iii 的說明寫封存新增的檔在 `{{SPEC_DIR}}archived/`，同一步的樣板卻是 `openspec/changes/archive/<name>/`（封存實際的位置）。說明改為 `{{SPEC_DIR}}changes/archive/`。
- quality：review 站以「stop without stamping」收尾時會提出自己的收尾問題，quality 的停頓卻在兩站都回報之後。改寫寫明 review 站的收尾不結束這一輪、同一回合直接接 verify，quality 的停頓優先。

### 規格釘字的處理原則

render_golden.rs 的逐字比對片語能保留就逐字保留。只有片語本身違反規範（重複陳述、歷史資訊、非鐵律的強調字）時才改：

- 片語只是措辭、對應的 scenario 沒有逐字要求 → 只改 render_golden.rs 的比對字串，測試名稱與所驗的行為不變。
- 對應的 scenario 逐字要求重複或該措辭 → 在本變更以 MODIFIED 改寫那條 requirement：整塊複製、scenario 名稱不變、只改字面要求，並同步改比對字串。

已知一條：discuss-skill「討論記錄的樹慣例與格式不變」。結論條列規則改為只在 conclude 步驟完整陳述一次，文件規則清單與 Capture decisions 摘要用一句話指向它；`discuss_skill_carries_the_conclusion_bullets_rule` 改為驗證：規則全文只出現一次、conclude 範例為條列形、Capture decisions 段指向 conclude 範例而不另抄樣板。

### 版號、golden 與 assets.lock 一次重產

七批改寫全部完成後才做，只做一次：

1. ASSET_VERSION 依當下版號升一個 minor（add-copilot-tool 合併後的值為基準）。
2. `UPDATE_GOLDEN=1 cargo test -p speclink-core --test it render_golden::` 重產全部 golden（含 Copilot 快照），逐份看過 diff 只有原稿內文改變。
3. `UPDATE_ASSETS_LOCK=1` 重產 assets.lock。
4. `cargo build -p speclink-cli` 後用 `target/debug/speclink update` 重產本 repo 的 `.claude/skills` 與 `.agents/skills` 技能檔；這些檔不會出現在 evidence，收尾 commit 時用 git status 盤點帶上。

### Codex 與 Copilot 手動抽查

由使用者在 Codex CLI（GPT 模型）與 Copilot 上，用新版技能各跑 propose、apply、discuss 三份；工作區沿用評測題 propose-from-discussion、apply-leaves-manual-task、discuss-no-record-before-reply 的 scaffold.sh 建立（init 時改用 `--tools codex` 或 `--tools copilot`）。判準與該題的打分規則相同，由使用者記錄每份「照做／未照做＋原因」。未照做的項目回頭修技能文字，修完重跑該題的 Claude 評測確認沒有退步。

## Implementation Contract

**行為**：

- 19 個技能在 claude、codex、copilot、自訂描述子四種渲染目標的內文依寫法規範改寫；frontmatter、description 與技能所描述的流程行為不變；CLI 的指令、旗標、人眼與 `--json` 輸出不變。
- Codex 與自訂描述子版中，原本寫死的 14 處技能引用改為和其他引用相同的寫法（`$speclink-<名稱>`、`speclink <名稱>`）。
- `cargo test -p speclink-core --test it` 新增 skill_authoring 測試；任一原稿違反五項機械檢查時，失敗訊息指名原稿檔名、違反項目與規範出處。
- `node scripts/claude-code/skill-evals.mjs compare` 對 old-opus／new-opus 與 old-haiku／new-haiku 兩組都 exit 0；某組列出退步時，退步題依「舊版基準與新版評測的跑法」第 5 點以 `--case` 新舊各重跑一次，重跑後不再退步也算達成。

**介面與資料形狀**：

- 執行腳本：`run --speclink <path> --model <id> --label <name> --max-cost-usd <usd> [--case <題目>] [--runs <次數>]`；`compare <oldDir> <newDir>`，exit 0（無退步）、1（有退步，列出題目與新舊分數）、2（題目集合不同，列出缺少的題目）、3（無法比較：用法錯、結果檔不存在、不是 JSON 或因花費上限而不完整）。
- 評測題目：`integrations/claude-code/skill-evals/evals/<題目名>/` 含 prompt.md（tags 含 `skill:<短名>`）、case.yaml、scaffold.sh、graders/*.md。
- 強調字登記表：skill_authoring.rs 內的 `(檔名, 上限)` 常數表，涵蓋 assets/skills 下全部原稿。

**失敗模式**：

- 原稿超過 500 行、強調字超過上限、出現日期或 design 決策編號、缺退路句、寫死技能引用或 `--agent` 值、出現未登記的原稿 → skill_authoring 測試失敗。
- 某技能缺評測題 → skill-evals.test.mjs 的涵蓋度守門失敗，列出缺題的技能短名。
- 評測花費達上限 → claude exit 2，執行腳本照樣以 2 結束，不寫比較結果。

**驗收**：

- `cargo test -p speclink-core --test it` 全綠（render_golden、skill_verbization、skill_authoring）。
- `node --test scripts/claude-code/skill-evals.test.mjs scripts/docs/vocabulary-guard.test.mjs` 全綠。
- `speclink validate optimize-skill-prompts` 與 `speclink analyze optimize-skill-prompts` 無 Critical／Warning。
- 兩組 compare 都 exit 0，或退步題依第 5 點重跑後不再退步；Codex 與 Copilot 抽查記錄完成。

**範圍**：

- 範圍內：20 份對外技能原稿的內文（含 14 處寫死引用的修正）、skill_authoring 測試、render_golden 的逐字比對與 golden、assets.lock、ASSET_VERSION、評測套件與執行腳本、discuss-skill 一條 requirement 的字面、改寫中找到的其他釘字 scenario。
- 範圍外：技能 description、內部技能內文、技能渲染程式（skills.rs）、任何 CLI 或 host 程式碼、引擎檢查、拆附檔、多家模型自動評測。

## Risks / Trade-offs

- [改寫順手改了行為，大 diff 中難以察覺] → 每份原稿先列必須保留的規格內容再改寫；render_golden 的逐字比對與評測雙重把關；verify 關卡逐條對照逐技能規格。
- [render_golden 大量比對字串被改，掩蓋行為漂移] → 只有片語違反規範才改；每處修改要對應「只是措辭」或「MODIFIED scenario」其中之一，審查時單獨看 render_golden.rs 的 diff。
- [Codex 與自訂描述子版的輸出改變] → 只改 14 處引用；5.2 重產 golden 時確認 codex 與 neutral 快照中這些位置分別變成 `$speclink-<名稱>` 與 `speclink <名稱>`，其餘差異只來自改寫。
- [golden 快照與 assets.lock 跟 add-copilot-tool 衝突] → 以 depends_on 排在 add-copilot-tool 之後；合併時一律重產衍生檔，不挑邊。
- [評測雜訊造成假退步] → 每題 3 次；退步時新舊各重跑一次確認。
- [評測只涵蓋 Claude] → Haiku 代表能力較弱的模型；Codex 與 Copilot 手動抽查；這是討論中接受的取捨。
- [評測花費] → 每次 run 前向使用者確認 `--max-cost-usd`。
- [跑到舊 binary] → 執行腳本把指定執行檔放在 PATH 最前面，並記錄 `speclink --version`；舊版執行檔在動原稿前就複製出來。
- [評測工具格式與預期不同（專案技能是否載入、scaffold_script 的寫法、結果檔的題目欄位）] → 第一個任務先做探測，依結果採用主要做法或替代做法。
- [跨平台] → skill_authoring.rs 用 `lines()` 計行（CRLF 安全）、`read_dir` 結果依檔名排序；node 測試用 `path.join` 並以正斜線比較邏輯路徑；執行腳本不在 CI 執行。
- [單輪評測測不到多輪行為] → 對話型技能只驗第一輪；多輪行為仍靠逐技能規格與品質關卡。

## Migration Plan

- 使用者端：ASSET_VERSION 升版後，既有工作區的技能檔被探測為過期，執行 `speclink update` 即重產；不需要其他遷移。
- 回滾：以新的版號重新發布舊內容，不倒退 ASSET_VERSION（`update_downgrade_guard` 會擋住較舊版號覆寫較新的技能檔）。

## 評測結果

舊版為 engine v1.43.0（`target/skill-evals/speclink-old`），新版為 engine v1.44.0（`target/skill-evals/speclink-new`）；每題 3 次、評審 claude-sonnet-5-5。花費為工具以 API 牌價換算的金額，實際扣的是登入帳號的方案用量。

**claude-haiku-5-5（old-haiku $0.67、new-haiku $0.71）**

`compare target/skill-evals/old-haiku target/skill-evals/new-haiku`：

```
題目                              舊分數  新分數  差值
analyze-reports-only              1.00    1.00  +0.00
apply-leaves-manual-task          1.00    1.00  +0.00
apply-with-worktree-isolates      0.78    1.00  +0.22
archive-folds-delta               1.00    1.00  +0.00
audit-finds-dangerous-default     0.67    1.00  +0.33
baseline-map-before-specs         1.00    1.00  +0.00
commit-gate-shows-plan            1.00    1.00  +0.00
config-asks-before-writing        1.00    1.00  +0.00
discuss-no-record-before-reply    1.00    1.00  +0.00
drift-reports-only                1.00    1.00  +0.00
improve-scan-without-edits        1.00    0.89  -0.11
ingest-folds-new-requirement      1.00    1.00  +0.00
manual-generates-pages            1.00    1.00  +0.00
propose-from-discussion           1.00    1.00  +0.00
quality-pauses-after-round        0.67    0.83  +0.17
review-freezes-scope              1.00    1.00  +0.00
trace-cites-archive               1.00    1.00  +0.00
verify-freezes-scope              1.00    1.00  +0.00
worktree-merge-lands-branch       1.00    1.00  +0.00
退步：improve-scan-without-edits（1.00 → 0.89）
```

improve-scan-without-edits 依「舊版基準與新版評測的跑法」第 5 點新舊各以 `--case` 重跑一次（每次 3 次）：舊版 0.89、新版 1.00，`compare` exit 0。原本新版的那一次失分是模型在提問後多加了一段給 review 的備註，回覆不是以提問結尾；重跑後新版不低於舊版，不算真退步。

**claude-opus-5-5（old-opus $15.08、new-opus $14.97）**

`compare target/skill-evals/old-opus target/skill-evals/new-opus`，exit 0：

```
題目                              舊分數  新分數  差值
analyze-reports-only              1.00    1.00  +0.00
apply-leaves-manual-task          1.00    1.00  +0.00
apply-with-worktree-isolates      1.00    1.00  +0.00
archive-folds-delta               1.00    1.00  +0.00
audit-finds-dangerous-default     0.67    1.00  +0.33
baseline-map-before-specs         1.00    1.00  +0.00
commit-gate-shows-plan            0.83    1.00  +0.17
config-asks-before-writing        1.00    1.00  +0.00
discuss-no-record-before-reply    1.00    1.00  +0.00
drift-reports-only                1.00    1.00  +0.00
improve-scan-without-edits        1.00    1.00  +0.00
ingest-folds-new-requirement      1.00    1.00  +0.00
manual-generates-pages            1.00    1.00  +0.00
propose-from-discussion           1.00    1.00  +0.00
quality-pauses-after-round        1.00    1.00  +0.00
review-freezes-scope              1.00    1.00  +0.00
trace-cites-archive               1.00    1.00  +0.00
verify-freezes-scope              1.00    1.00  +0.00
worktree-merge-lands-branch       1.00    1.00  +0.00
```

兩組都沒有真退步。改善集中在本變更處理過的矛盾與停頓點：audit 不再嘗試改檔（兩個模型都從 0.67 到 1.00）、quality 在 Haiku 上較常把兩站一起回報（0.67 → 0.83）、commit 與 apply-with-worktree 的雜訊下降。quality 在 Haiku 上仍有一次只跑完 review 就結束，留給後續觀察。

**品質關卡後修正的打分規則（未重跑）**

- propose-from-discussion 與 ingest-folds-new-requirement 的 runs-validate 原為 regex trace 含 `speclink validate`。trace 含注入對話的技能內文，兩份技能內文都寫了這個指令，所以這一項永遠通過。已改為 tool_used Bash input_match `speclink validate` min 1。
- archive-folds-delta 原本只檢查 Edit 有沒有改 `openspec/specs`；新增 no-hand-write，Write 同樣 max 0。
- 依使用者決定不重跑。上面兩張表是修正前的打分規則跑出的分數：這三項在新舊版的條件相同，比較結論不變。四份完整結果的 `partial` 都是 false，`compare` 拒絕不完整結果的新規則不影響它們。

**Codex 與 Copilot 抽查（任務 7.1、7.2）**

使用者請代理代跑一次。工作區由三題的 scaffold.sh 以 `EVAL_TOOLS=codex`／`copilot` 建在 repo 之外，PATH 最前面放新版執行檔（engine v1.44.0）與複製的 git；判準為各題的打分規則。

| 工具（模型） | discuss-no-record-before-reply | apply-leaves-manual-task | propose-from-discussion |
| --- | --- | --- | --- |
| Codex CLI 0.162.0-alpha.2（gpt-6.1-sol，reasoning medium） | 照做：未執行 `discuss new`、無記錄檔，以帶建議的單一提問結尾 | 照做：先跑 `plan`，`review prepare` 在 `in-progress add` 之前，`task done` 勾 1.1、`[M]` 1.2 未勾並點名 | 照做：`[M]` 緊貼 checkbox、跑了 `speclink validate`、未接著 apply、`--agent codex` |
| Copilot CLI 1.0.93（預設模型 mai-code-1.1-flash） | 照做：同上 | 照做：同上，並說明品質關卡可先跑、封存等手動任務 | 照做：同上、`--agent copilot`，先不帶 `--new` 被拒後才加 |

結論：兩個工具三份技能都照做，沒有「未照做」項目。打分規則以外的觀察：Codex 在 propose 第一次寫規格就帶 `--new`，沒有先不帶 `--new` 試一次；當時正式規格為空，結果相同，記錄供日後觀察。

環境注意事項：

- Codex 在 login shell（`zsh -lc`）執行指令，使用者的 profile 會把 `/opt/homebrew/bin` 排在前面，找到 Homebrew 舊版 `speclink`（0.5.0、engine v1.35.0，沒有 `plan`）。以 `-c allow_login_shell=false` 執行才會用到 PATH 最前面的新版執行檔；第一次未加這個設定的 apply 與 propose 結果作廢，以重跑為準。
- Copilot CLI 1.0.93 的 `--model` 對任何值都回報 not available（包括預設的 mai-code-1.1-flash），所以用預設模型。
- discuss-no-record-before-reply 的 scaffold.sh 原本寫死 `--tools claude`，沒有走 `fixtures.sh` 的 `eval_init`，Codex／Copilot 工作區因此沒有技能；已改為 `eval_init`，Claude 評測的行為不變。

## Open Questions

三項都由探測任務 1.1（claude 2.1.293、claude-haiku-5-5、舊版執行檔 engine v1.43.0）回答：

- 評測工作區是否載入 scaffold 產生的專案技能 → 不載入。trace 第一行的 `skills` 清單沒有任何 speclink 技能；改用「plugin 的 skills/」做法後，清單出現 `speclink-skill-evals:speclink-discuss` 等技能，提示 `/speclink-discuss` 會載入它，受測模型也執行到 PATH 上的 `speclink`。結論：採用「plugin 的 skills/」做法（見「評測套件的位置與載入方式」）。
- `scaffold_script` 的寫法 → 相對於題目目錄的檔案路徑（`scaffold_script: scaffold.sh`），評測以 `bash <路徑>` 在工作區執行，PATH 沿用呼叫者的 PATH、HOME 為臨時目錄、上限 120 秒。
- `aggregate-result.json` 中標示題目的欄位 → `cases[].name`；題目分數為 `cases[].aggregates.score`。`compareResults` 依這兩個欄位讀取。
- 另一項探測發現：`tool_used` 打分規則只寫 `max: 0` 時，`min` 預設為 1，結果永遠失敗；題目表中「max 0」的規則一律同時寫 `min: 0`。

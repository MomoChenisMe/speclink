---
topic: 內嵌宿主第九輪的兩項需求（討論代號 embedded-host-needs-9）。來源文件：~/Documents/GitHub/speclink-embedded-host-needs-9.md，接在已封存的討論 embedded-host-needs-8 之後；請把文件裡的每一項當成待檢驗的主張，對照程式碼確認。
slug: embedded-host-needs-9
status: promoted
created: 2026-11-24
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: spec-names-and-instructions-ids
---

# Discussion: 內嵌宿主第九輪的兩項需求（討論代號 embedded-host-needs-9）。來源文件：~/Documents/GitHub/speclink-embedded-host-needs-9.md，接在已封存的討論 embedded-host-needs-8 之後；請把文件裡的每一項當成待檢驗的主張，對照程式碼確認。

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

內嵌宿主把引擎升到 0.18.0（engine v1.60.0）之後，用真實模型照 ingest、trace 技能實跑，回報兩項：H1 規格語言的指示沒有說需求與情境的名稱要不要跟著翻；H2 ingest 技能讓 `artifact cat` 與 `instructions` 的 delta 規格代號看起來一樣。來源文件已經把每一項寫成待檢驗的主張，所以沒有 grill 階段，直接逐條對照程式碼與實測查證。相關的正式規格是 workflow-config（instructions 對 specs 產出物回報解析後的規格語言）、spec-validation（change 驗證納入合併守門）、ingest-skill、propose-skill、change-analysis；目前沒有進行中的變更。
Source doc: ~/Documents/GitHub/speclink-embedded-host-needs-9.md
Prior discussions: embedded-host-needs-8, skill-conversation-language, canonical-spec-zh-wording

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-11-24)

**Focus**: 來源文件 H1、H2 的每一條主張，對照程式碼與實測（暫時專案，`locale: tw`、`spec_locale: tw`，引擎 0.18.0）是否成立
**Position**: 兩項的現況大致屬實，但有三處和文件不同；使用者確認查證無誤。
- H1 屬實：`instructions specs` 的指示只講結構標記、沒講名稱（`workspace/instructions.rs:143`）；ingest 第 5 步（`assets/skills/ingest.md:130`）與 propose 第 8 步（`propose.md:297`）的 `locale` 段落同樣只講標記
- H1 屬實：本機模式是同一段文字，所有模式都經 `command/mod.rs:1725` 呼叫 `build_artifact`；workflow-config 正式規格已承諾宿主與本機 CLI 的 payload 相同；`spec_locale: tw` 時結尾另有中文模糊用語的提醒，來源文件引用時漏掉
- H1 不屬實：「名稱被翻掉要到封存才對不上」——`validate` 的合併守門（spec-validation「change 驗證納入合併守門」）在封存前就擋，與 `--strict` 無關；實測 MODIFIED 需求名翻掉報 `target requirement no longer exists in the canonical spec`，情境名翻掉報 `drops canonical scenario(s) … — carry them over, or declare the removal with <!-- REMOVED-SCENARIO: <name> -->`；ingest 寫回後本來就跑 `validate`
- H1 殘留風險：情境那條錯誤建議宣告 `REMOVED-SCENARIO`，模型照做會讓翻譯變成能通過檢查的改名
- H1 屬實但有副作用：ADDED 改名合法，但 `analyze` 的 Coverage 以完整需求名稱比對任務文字（`quality/analyzer.rs:323` 的 `req_covered`），引用舊名的任務會出 `has no matching task` 警告，已勾的任務 ingest 不能改
- H2 屬實且更廣：delta 規格有三種寫法——讀 `artifact cat specs/<capability>`、取指示 `instructions specs`、寫回 `new artifact spec <capability>`（共用段落 `artifact-write-back.md`，6 支技能都帶）；第二次實跑的 `instructions spec <能力>` 就是照抄寫回的寫法
- H2 部分不屬實：「錯誤訊息都夠清楚」——`artifact cat` 的錯誤列出可用代號，`instructions` 的 `Artifact '<id>' not found in schema`（`command/mod.rs:1729`）不列任何代號，所以第一次實跑試了 5 次
- H2 其他技能：propose 有反方向的坑——第 8 步用 `artifact cat <artifact-id>` 讀前置文件，tasks 的前置代號是 `specs`，實測 `artifact cat specs` 被拒；apply、verify、archive、commit 只用 `artifact cat specs/<capability>`，config 只用 `instructions <artifact>`，都沒有混用
- 決策樹：D1 名稱要不要跟著規格語言 → D2 規則寫在哪 → D3 H2 的修法 → D4 拆成幾個變更
**Open**: D1 名稱規則；D2 規則寫在引擎指示、技能段落或兩邊；D3 H2 修法（補技能文字／錯誤訊息列代號／instructions 多收一種寫法）；D4 一個變更還是兩個

### Round 2 — interview (2026-11-24)

**Focus**: D1 需求與情境的名稱要不要跟著規格語言
**Position**: D1 定案選 A：名稱算內文、跟著規格語言寫，但指向正式規格的名稱一字不改。
- 跟著規格語言的：ADDED 的需求名稱與情境名稱、RENAMED 的新名稱、MODIFIED 區塊裡新增的情境名稱
- 一字不改的：MODIFIED 與 REMOVED 的需求名稱、RENAMED 的舊名稱、MODIFIED 區塊裡沿用的正式規格情境名稱
- ingest 改到既有名稱時，回答裡列出「舊名稱 → 新名稱」，補上第一次實跑「回答沒提名稱也改了」的缺口
- 依據：這個 repo 是 `spec_locale: tw`，870 條正式規格的需求名稱全部含中文；來源文件的情境裡同一個能力的正式規格是中文
- 例：ADDED 的「… token endpoint」改成「… Token 端點」，回答寫明這個改名；同一條需求若放在 MODIFIED，名稱維持「… token endpoint」，只有內文改成中文
**Ruled out**: B 改既有 delta 規格時名稱一律不動（來源文件的提議）——照這次的情境，封存後中文正式規格會多出 3 條英文需求名稱與 9 個英文情境名稱；C 名稱一律英文、比照結構標記——推翻這個 repo 870 條名稱的慣例
**Open**: D2 規則寫在引擎指示、技能段落或兩邊；D3 H2 修法；D4 一個變更還是兩個

### Round 3 — interview (2026-11-24)

**Focus**: D2 名稱規則寫在哪
**Position**: D2 定案選 A：完整規則寫在引擎給 `instructions specs` 的指示，技能只加指向與摘要項目。
- 引擎：`workspace/instructions.rs:143` 的規格語言段落多一句名稱規則——新名稱用規格語言寫；MODIFIED 與 REMOVED 的需求名稱、RENAMED 的舊名稱、MODIFIED 區塊沿用的正式規格情境名稱照正式規格原文；確切英文提案時定
- 技能：ingest 第 5 步、propose 第 8 步講 `locale` 的段落各加一句，說 specs 的指示會寫明哪些名稱照正式規格原文
- ingest 第 11 步的摘要多列一項「改到的名稱：舊 → 新」
- 技能原稿有改，ASSET_VERSION、golden、assets.lock 一起更新
- 要改的正式規格：workflow-config「instructions 對 specs 產出物回報解析後的規格語言」、ingest-skill（摘要列出改名）、propose-skill 講規格語言的情境
**Ruled out**: B 只改引擎指示——「回答列出改名」是 ingest 對使用者的報告，引擎不知道是哪支技能在呼叫，而且技能段落只講標記會讓模型以為規則只有這些；C 引擎與技能各寫一份完整規則——兩份會漂移，違反第八輪「引擎說一次、說對」
**Open**: D3 H2 修法；D4 一個變更還是兩個

### Round 4 — interview (2026-11-24)

**Focus**: D3 H2 的修法
**Position**: D3 定案選 B：補技能文字，另外讓 `instructions` 找不到代號時列出可用的代號。
- ingest 第 5 步的 get instructions first 寫明三種寫法：取指示 `instructions specs`（每個能力都一樣、不帶能力名稱）、讀 `artifact cat specs/<capability>`、寫回 `new artifact spec <capability>`
- propose 第 8 步讀前置文件遇到 `specs` 時，照 `show <name> --json` 的 `deltaSpecs` 逐一以 `artifact cat specs/<capability>` 讀
- 引擎：`command/mod.rs:1729` 的 `Artifact '<id>' not found in schema` 接上可用代號，代號取自產出流程（自訂產出流程也正確）並含 `apply`；沒有測試或正式規格鎖住這句
- 例：`instructions specs/auth --change demo` 回 `Artifact 'specs/auth' not found in schema. Use proposal, design, specs, tasks, or apply`；確切文字提案時定
**Ruled out**: A 只補技能文字——模型沒照技能做時仍要試好幾次，舊版技能也不受惠；C `instructions` 接受 `specs/<capability>` 當成 `specs`——兩種寫法並存會讓人以為每個能力各有一份指示，而且 `instructions spec <能力>` 兩個參數的寫法照樣失敗
**Open**: D4 一個變更還是兩個

### Round 5 — interview (2026-11-24)

**Focus**: D4 一個變更還是兩個
**Position**: D4 定案選 A：H1、H2 合成一個變更，暫定名稱 `spec-names-and-instructions-ids`，提案時可改。
- 兩項都改 `ingest.md` 與 `propose.md`，合起來只升一次 ASSET_VERSION、只再生一次 golden
- 要改的正式規格幾乎重疊：H1 是 workflow-config、ingest-skill、propose-skill；H2 是 ingest-skill、propose-skill，引擎錯誤訊息的需求可能也放進 workflow-config
- 第八輪拆兩刀的理由（G3 改的是 node-sdk 選項）這一輪不存在：兩項都在講技能怎麼寫 specs 這份產出物
**Ruled out**: B 拆成兩個變更——改同樣的檔案只能依序做、要升兩次版號，前一個封存後後一個的 delta 規格常會過期
**Open**: none

## Conclusion

**Decision**: 兩項都做，合成一個變更 `spec-names-and-instructions-ids`（提案時可改名）：規格語言的指示寫明需求與情境名稱的規則，技能與 `instructions` 的錯誤訊息把 delta 規格的三種代號講清楚。
- H1 名稱規則：名稱算內文，跟著規格語言寫——ADDED 的需求與情境名稱、RENAMED 的新名稱、MODIFIED 區塊裡新增的情境名稱用規格語言；指向正式規格的名稱一字不改——MODIFIED 與 REMOVED 的需求名稱、RENAMED 的舊名稱、MODIFIED 區塊裡沿用的正式規格情境名稱
- H1 引擎：`workspace/instructions.rs:143` 的規格語言段落多一句名稱規則，所有模式都經 `build_artifact`，本機、遠端連線、內嵌宿主得到同一段文字；確切英文提案時定，草稿是「Requirement and scenario names are prose: write new names in <locale>. Names that point at the canonical spec stay exactly as it writes them: MODIFIED and REMOVED requirement names, the FROM name of a RENAMED entry, and the canonical scenario names a MODIFIED block carries over.」
- H1 技能：ingest 第 5 步、propose 第 8 步講 `locale` 的段落各加一句，說 specs 的指示會寫明哪些名稱照正式規格原文；完整規則只在引擎說一次
- H1 ingest 第 11 步的摘要多列一項「改到的名稱：舊 → 新」，沒改到就不列
- H1 不另加檢查：`validate` 的合併守門已在封存前擋下 MODIFIED、REMOVED、RENAMED 名稱對不上正式規格，以及 MODIFIED 區塊漏掉正式規格的情境，與 `--strict` 無關；來源文件「要到封存才對不上」不成立
- H2 技能：ingest 第 5 步的 get instructions first 寫明三種寫法——取指示 `instructions specs`（每個能力都一樣、不帶能力名稱）、讀 `artifact cat specs/<capability>`、寫回 `new artifact spec <capability>`
- H2 技能：propose 第 8 步讀前置文件遇到 `specs` 時，照 `show <name> --json` 的 `deltaSpecs` 逐一以 `artifact cat specs/<capability>` 讀（現在 `artifact cat specs` 會被拒）
- H2 引擎：`command/mod.rs:1729` 的 `Artifact '<id>' not found in schema` 接上可用代號，代號取自產出流程（自訂產出流程也正確）並含 `apply`；沒有測試或正式規格鎖住這句
- 其他技能已查過不用改：apply、verify、archive、commit 只用 `artifact cat specs/<capability>`，config 只用 `instructions <artifact>`
- 要改的正式規格：workflow-config「instructions 對 specs 產出物回報解析後的規格語言」（加名稱規則），以及錯誤訊息列出代號的需求（放 workflow-config 或 workflow-schemas，提案時定）；ingest-skill（三種代號、摘要列出改名）；propose-skill（講規格語言的情境、讀前置 delta 規格的寫法）
- 技能原稿有改，ASSET_VERSION、golden、assets.lock 一起更新；要跟著改的測試含 `instructions.rs` 的 `specs_locale_reports_the_resolved_spec_language` 與 CLI 的 `tests/it/instructions_policy.rs`
- 例（`spec_locale: tw`，正式規格是中文）：ingest 把英文的 ADDED delta 規格改成中文時，「… token endpoint」改成「… Token 端點」，摘要列出「… token endpoint → … Token 端點」；同一條需求若在 MODIFIED，名稱維持「… token endpoint」，只有內文改成中文
- 例：`instructions specs/auth --change demo` 回 `Artifact 'specs/auth' not found in schema. Use proposal, design, specs, tasks, or apply`；確切文字提案時定
- 來源文件「這一輪確認足夠的部分」不要求改動，沒有逐條再驗證
**Rationale**: 取捨的主軸是讓封存後的正式規格只有一種語言，同時不讓指向正式規格的名稱被默默改掉：這個 repo 在 `spec_locale: tw` 下 870 條需求名稱全是中文，名稱一律不動會在中文正式規格裡留下英文名稱，而會出事的那幾種名稱 `validate` 本來就擋，所以只需要把規則寫進引擎的指示、讓 ingest 把改名說出來。H2 的根源在技能把三種寫法混在一起，補技能文字是治本，錯誤訊息列出代號則讓沒照技能做的呼叫者也只多試一次，兩者都比讓 `instructions` 多收一種寫法小。
**Rejected alternatives**:
- 改既有 delta 規格時名稱一律不動（來源文件的提議） — 封存後中文正式規格會多出英文的需求與情境名稱
- 名稱一律英文、比照結構標記 — 推翻這個 repo 870 條名稱的慣例
- 名稱規則只寫在引擎指示 — 「回答列出改名」是 ingest 對使用者的報告，引擎不知道誰在呼叫；技能段落只講標記會讓模型以為規則只有這些
- 引擎與技能各寫一份完整名稱規則 — 兩份會漂移，違反第八輪「引擎說一次、說對」
- H2 只補技能文字 — 模型沒照技能做時仍要試好幾次，舊版技能不受惠
- `instructions` 接受 `specs/<capability>` 當成 `specs` — 兩種寫法並存會讓人以為每個能力各有一份指示，`instructions spec <能力>` 照樣失敗
- 拆成兩個變更 — 改同樣的檔案只能依序做、要升兩次版號，後一個的 delta 規格會在前一個封存後過期
**Deferred**:
- `validate` 在 MODIFIED 漏掉正式規格情境時建議宣告 `REMOVED-SCENARIO`，模型照做會讓翻譯變成能通過檢查的改名 — 名稱規則已要求沿用的情境名稱照原文，目前沒有觀察到這個錯
- ADDED 改名後引用舊名的任務會出 `analyze` 的 Coverage 警告，已勾的任務 ingest 不能改 — 摘要列出改名後使用者看得到，目前沒有觀察到問題
- validate 錯誤行尾的「(see: speclink drift <change>)」經 `dispatch` 會被拒絕（第八輪留下） — 仍沒有觀察到問題
**Capture to**: proposal | design | spec | tasks
**Next**: /speclink-propose --from-discussion embedded-host-needs-9

---
topic: 我想要優化每一個speclink組合出來的技能提示詞
slug: skill-prompt-optimization
status: promoted
created: 2026-10-08
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: optimize-skill-prompts
---

# Discussion: 我想要優化每一個speclink組合出來的技能提示詞

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者想優化 speclink 產生的每一份技能提示詞（crates/engine/speclink-core/assets/skills/ 的 23 份原稿，經 src/workspace/skills.rs 加檔頭與替換字後產生 19 個技能，Claude／Codex 各一套）。「優化」一開始不能檢驗，所以先跑一題釐清：目標是什麼。
現況盤點：原稿共約 318KB，最長的是 discuss（45KB／484 行、改過 28 次）、propose（31KB）、ingest、review、verify、apply（各約 21KB；apply-with-worktree 組合後約 33KB）；技能之間逐字重複的長句只有兩條（CLI 檢查 12 份、提問工具退路 9 份），重複主要在單一技能內部，是一次次補丁疊上去的；共用片段只有 {{NEXT_STEPS_LEAD}}；每個技能都是單檔，沒有需要時才讀的附檔。改原稿要連動 golden、assets.lock、ASSET_VERSION。
相關規格與變更：skill-routing、17 份逐技能規格（discuss-skill、propose-skill、review-skill 等）、verb-contract（skill_verbization 守門：技能不得直接讀規格目錄）、workspace-tools（渲染目標）；進行中變更 add-copilot-tool: workspace-tools, sync-specs-after-docs-cleanup: verb-contract。
Prior discussions: architecture-improve-flow, agent-tool-sdk-layering

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — interview (2026-10-08)

**Focus**: 這次優化的主要目標是什麼
**Position**: 目標定為「模型更照做」——少漏步驟、少踩已知的雷；縮短篇幅只是手段之一，判準是實跑表現。
- 決策樹：A 目標（本輪定案）→ B 範圍（19 份全做或先挑）→ C 手法（去內部重複／拆附檔／重排結構／抽共用片段）→ D 判準（篇幅門檻／實跑評測／人工抽查）→ E 切刀與規格、golden 連動
- 依據：記憶中累積多條同型的模型失誤（add-round 路徑帶行號、stable ID 不在行尾、[M] 標記位置），代價來自規則埋在長文裡被漏讀，不是 token 本身
- 舊討論 architecture-improve-flow 排除過「濃縮 friction 清單」（原文即判準，濃縮丟操作性）——本案的縮短須以不丟判準為前提
**Ruled out**: 以省 token 為主目標——單份技能載入約一萬 token 以內，對上下文不是瓶頸，且以篇幅為指標會鼓勵濃縮掉判準；以人好讀好改為主目標——對維護有利但不直接改善模型行為，留作附帶效益
**Open**: B 範圍（全做或先挑幾份）；C 手法；D 怎麼量「更照做」；E 與進行中 add-copilot-tool（新增 Copilot 渲染目標、會動 golden 與版號）的先後

### Round 2 — assumptions (2026-10-08)

**Focus**: 範圍、手法、判準與時機的初始假設（B／C／D／E）
**Position**: 使用者修正兩點——技能提示詞要讓大部分模型都能照做（不只 Claude）、19 份一起優化不做試點；其餘假設未被推翻。
- 修正一（跨模型）：技能經 Codex（GPT）、Copilot（可選多家模型）、自訂描述子（任意 harness 與模型）載入，所以寫法與判準都不能只對 Claude 成立；只用 `claude plugin eval` 量測只涵蓋 Claude，需重議 D
- 修正二（19 份一起）：推翻「先挑 propose 試點」
- 事實：工具名的跨模型問題已大致處理——AskUserQuestion 出現於 15 份原稿共 54 處，全部附「不可用就用純文字問」的退路；review／verify 的子代理寫成「例如 Agent tool，不能開就自己跑」；跨模型的風險主要在寫法本身，不在工具名
- 事實：本機沒有安裝 codex、copilot、gemini CLI，跨模型實跑需另行安裝與付費
- 維持：單檔內重整不拆附檔（C）；能用引擎擋的錯交給引擎（範圍守門）；原稿改動等 add-copilot-tool 合併後再進（E）
**Ruled out**: 先挑 propose 一份試點——使用者要 19 份一起優化；只以 Claude 實跑評測當唯一判準——技能會被多家模型讀，只量 Claude 不代表其他模型照做
**Open**: D 跨模型怎麼量（多家模型實跑、或跨模型寫法規範＋Claude 實跑＋抽查）；「19 份一起」是一個 change 一次改完，還是同一套規範分刀套用；跨模型寫法規範的內容

### Round 3 — interview (2026-10-08)

**Focus**: 兩項修正的意圖——跨模型怎麼判斷變好、「19 份一起」怎麼切
**Position**: 判準採「跨模型寫法規範為主＋Claude 實跑評測＋其他模型抽查」，切法採一個 change 一次改完 19 份。
- 判準：先訂一份跨模型寫法規範當主要標準，19 份都依它改；實跑評測用現成的 `claude plugin eval`；挑幾份重點技能在 Codex 或 Copilot 上手動抽查
- 切法：同一套規範一次套用到 19 份，ASSET_VERSION 只升一次，golden 與 assets.lock 只重產一次；代價是 diff 大（約 318KB 原稿），審查負擔重
**Ruled out**: 多家模型都實跑評測——本機沒有 codex／copilot／gemini CLI，需安裝、付費並自建能驅動多家 CLI 的測試工具，工具本身就是另一個專案；只靠寫法規範＋人工審稿——無法證明改寫沒有弄丟判斷標準；同一套規範分 3～4 刀——版號要升 3～4 次，使用者的技能檔連續過期多次
**Open**: 跨模型寫法規範的內容與存放位置；評測題目怎麼出、放在哪；抽查哪幾份、用哪家模型；行為是否零變化（逐技能規格要不要動）

### Round 4 — assumptions (2026-10-08)

**Focus**: 跨模型寫法規範的內容、存放與執行方式、評測設計、抽查、行為不變（五條假設）
**Position**: 五條假設全部成立——規範九條進新 capability 並以守門測試擋可機械檢查的部分，評測先建再改，Codex 抽查三份，逐技能規格不動。
- 規範九條（取 OpenAI 與 Anthropic 官方指南的交集）：每條規則只寫一次且寫在動作步驟裡；規則不矛盾、例外寫明優先序；強調字只給鐵律（會壞資料、不可逆、必須停下等人）——限量不全刪，因能力較弱的模型有時需要較強語氣；一概念一詞；工具相依能力附退路；步驟編號、判斷點寫成條件→動作、產出附樣板、結尾附自我檢查清單；不寫歷史（design D#、日期、哪個 change 加的）；SKILL.md 本文 ≤ 500 行；判斷標準原文不濃縮
- 量測現況：強調字約 290 個（NOT 92、STOP 50、Do NOT 39、MUST 30、NEVER 26、SHALL 22、CRITICAL 17、IMPORTANT 12），review 183 行有 36 個；discuss 484 行、propose 466 行逼近上限；discuss 的結論分條列規則寫了三次；工具退路已齊（54 處 AskUserQuestion 全有）
- 存放：新 capability（暫名 skill-authoring）承載規範；500 行上限、強調字數量、不寫歷史、工具名必附退路寫成守門測試，比照 skill_verbization.rs
- 評測：改寫前先建；19 份每份至少一題主流程＋記憶中由技能文字造成的失誤題；對話型技能（discuss、improve、quality）只驗第一輪；新舊兩版 × 預設模型與 Haiku（代表能力較弱的模型）各跑，約 80 多次執行，以 --max-cost-usd 設上限
- 抽查：安裝 Codex CLI 手動跑 propose、apply、discuss；Copilot 等 add-copilot-tool 合併後補
- 行為不變：17 份逐技能規格要求的內容（77 條 THEN 涉及技能內容）全數保留，逐技能規格不改，只新增寫法規範；改寫完以 analyze 與 verify 逐條對照
**Ruled out**: 規範只寫成文件不設守門——無機制擋回流，補丁會再把原稿撐長；只用預設模型評測——量不到能力較弱模型的照做度
**Open**: 強調字上限的具體數字；評測題目的存放位置；capability 最終命名（交命名守門）

## Conclusion

**Decision**: 開一個 change，依一份跨模型寫法規範一次改寫 19 個技能的原稿，目標是讓模型更照做、行為不變。
- 目標：大部分模型都照做——Claude、Codex 的 GPT、Copilot 可選的模型、自訂描述子接上的任意模型；縮短篇幅只是手段，不是指標
- 規範九條（取 OpenAI 與 Anthropic 官方指南的交集）：
  - 每條規則只寫一次，寫在模型實際做那件事的步驟裡
  - 規則之間不矛盾；有例外時寫明哪條優先
  - 強調字（MUST、NEVER、CRITICAL、IMPORTANT 等）只給鐵律——會壞資料、不可逆、必須停下等人；限量而非全刪，因能力較弱的模型有時需要較強語氣
  - 一個概念一個詞
  - 依工具而不同的能力都附退路（現況 54 處 AskUserQuestion 已齊，維持）
  - 步驟編號、判斷點寫成「條件 → 動作」、產出附樣板、結尾附自我檢查清單
  - 不寫歷史（design D#、日期、哪個 change 加的）
  - SKILL.md 本文 ≤ 500 行
  - 判斷標準的原文不濃縮
- 規範落點：新 capability（暫名 skill-authoring）；可機械檢查的四項寫成守門測試，比照 skill_verbization.rs——本文 ≤ 500 行；強調字上限，以改寫後各技能的實際數量為天花板，只能降不能升（要升須改測試，成為刻意決定）；不寫歷史；工具名必附退路
- 例：discuss 的「結論要分條列」現寫在 Document rules 第 8 條、conclude 指令範本、Capture decisions 摘要格式三處，改寫後只留在 conclude 那一步
- 評測先行（改原稿前建好並跑出舊版基準）：用 `claude plugin eval`；19 份每份至少一題主流程，加上記憶中由技能文字造成的失誤題；對話型技能（discuss、improve、quality）只驗第一輪；新舊兩版 × 預設模型與 Haiku（代表能力較弱的模型）各跑，約 80 多次執行，以 --max-cost-usd 設上限；通過條件為新版每題不比舊版差
- 例：題目 discuss-no-record-before-reply，輸入 `/speclink-discuss 看板要支援搜尋`，打分規則檢查第一輪沒有呼叫 `speclink discuss new` 且以問使用者的問題結尾
- 抽查：安裝 Codex CLI 手動跑 propose、apply、discuss；改寫排在 add-copilot-tool 之後，Copilot 屆時同樣抽查這三份
- 行為不變：17 份逐技能規格要求技能寫到的內容（77 條 THEN 涉及技能內容）全數保留，逐技能規格不改，規格只新增寫法規範；改寫後以 analyze 與 verify 逐條對照
- 順序：評測題目與舊版基準可先做；原稿改寫等 add-copilot-tool 合併後進行；ASSET_VERSION 只升一次，golden 與 assets.lock 一次重產
- 維持單一 SKILL.md，不拆附檔
- 能由引擎檢查的錯（stable ID 位置、add-round 路徑格式）不往技能文字加規則
**Rationale**: 技能會被多家模型讀，各家官方指南的交集——規則只寫一次、規則不矛盾、強調字只給鐵律——同時改善照做度與篇幅；只有 Claude 能便宜地實跑，所以以寫法規範為主要標準，評測（含 Haiku）防止改寫丟失判斷標準，其他模型以抽查補足。
**Rejected alternatives**:
- 以省 token 為主要目標 — 單份技能約一萬 token 內不是瓶頸，且以篇幅為指標會鼓勵濃縮掉判斷標準
- 以人好讀好改為主要目標 — 不直接改善模型行為，留作附帶效益
- 先挑 propose 一份試點 — 使用者要 19 份一起優化
- 同一套規範分 3～4 刀 — 版號要升 3～4 次，使用者的技能檔連續過期多次
- 多家模型都實跑評測 — 本機沒有 codex／copilot／gemini CLI，需安裝、付費並自建驅動多家 CLI 的測試工具，工具本身就是另一個專案
- 只靠寫法規範＋人工審稿 — 無法證明改寫沒有弄丟判斷標準
- 只以 Claude 預設模型評測 — 量不到能力較弱的模型
- 規範只寫成文件不設守門 — 沒有機制擋回流，補丁會再把原稿撐長
- 拆附檔漸進載入 — 三種產生目標都是每技能單一 SKILL.md，改檔案格式牽動過期探測、孤兒清理、golden，屬另一個引擎變更
**Deferred**:
- 拆附檔 — 等評測證明單檔整理後模型仍漏讀再議
- 由引擎檢查 stable ID 位置與 add-round 路徑格式 — 屬引擎行為變更，另案
- 評測題目的存放位置與 capability 最終命名 — propose 期決定，命名交命名守門
**Capture to**: proposal | design | spec | tasks
**Next**: /speclink-propose --from-discussion skill-prompt-optimization

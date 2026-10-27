---
topic: 討論偵察的規格段納入進行中變更的 delta 規格
slug: discuss-scout-in-flight-deltas
status: promoted
created: 2026-09-16
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: discuss-scout-in-flight-deltas
---

# Discussion: 討論偵察的規格段納入進行中變更的 delta 規格

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者釐清 discuss 技能的偵察順序（正式規格 → 舊討論 → 程式碼）是否正確。以 SDD 角度判定順序正確，但找到一個缺口：規格段只讀已封存的正式規格（speclink list --specs --json），進行中變更帶的 delta 規格（openspec/changes/<name>/specs/<capability>/spec.md）不在偵察範圍；技能檔只在「Speclink Awareness」段跑 speclink list --json 列出變更名稱，而該 JSON 不含 delta 規格路徑或 capability 清單。後果：假設清單可能建立在一條即將被進行中變更改掉的正式規格上，要到 propose 或 drift 才撞到。使用者認同此缺口，要求開討論記錄。
需求已清晰（可驗證目標：偵察規格段能看到進行中 delta 規格），不需 grill 階段，直接列假設。
相關正式規格：discuss-skill（「事實與決策分診及逐節點查證」需求、「正式規格先行的漏斗偵察」場景，@trace 來源 discuss-grounding-and-flow）；skill-routing（偵察慣例可能跨技能）。相關進行中變更：add-change-plan-desktop、add-change-plan-remote、add-plan-handoff-after-archive（三者的 delta 規格共 12 個 capability 目錄，皆不動 discuss-skill）。事實來源檔：crates/engine/speclink-core/assets/skills/discuss.md（第 175 行 Canon pass、第 188 行 related changes 句、第 395 行 Speclink Awareness）。
Prior discussions: canonical-spec-zh-wording, quality-skill-canonicalization

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-16)

**Focus**: 偵察的規格段要不要納入進行中變更的 delta 規格，以及納入後的決策空間長什麼樣
**Position**: 納入，順序不動（正式規格＋進行中 delta → 舊討論 → 程式碼），delta 規格算「規格段」的一部分而不是另開一段。
- 決策樹：N1 該不該納入（已定：納入，使用者認同）；N2 怎麼找到 delta（技能檔規定用檔案路徑比對 vs 引擎 list 加欄位）；N3 讀多深（時間盒）；N4 假設清單怎麼呈現（三分對照加第五類 vs 併入既有類別加標記）；N5 Context 段要不要多一行
- 假設 A（N2）：純技能檔改動，零引擎改動。canon pass 命中 capability 名後，順手比對 openspec/changes/*/specs/<capability>/spec.md 是否存在；存在即為「進行中 delta 命中」。Evidence：speclink list --json 每筆只有 name／status／summary／completedTasks／totalTasks，沒有 capability 清單；但 delta 目錄以 capability 命名，用正式規格命中的名字直接對路徑就找得到。錯了的後果：若引擎日後改 delta 目錄佈局，技能檔的路徑假設會靜默失效。
- 假設 B（N3）：時間盒沿用現行規則的精神——delta 命中只讀 Requirement 標題與 ADDED／MODIFIED／REMOVED 標記，至多 3 份，不讀全文。Evidence：discuss-skill 規格「開場 scout 維持淺掃」場景要求正式規格讀 Purpose ≤3、原始碼 ≤5 檔。錯了的後果：偵察變成調查，違反該場景。
- 假設 C（N4）：不加第五類。在既有「Covered by canon／Conflicts with canon」兩類上加一個標記「（進行中變更 <name> 將改動）」，因為 delta 是「即將成為正式規格的承諾」，性質仍屬規格對照。Evidence：discuss-skill 規格「開場舊討論查核與第四類對照」需求已把三分對照擴成四類，再加第五類讓表格變重。錯了的後果：使用者看不出某條假設其實踩在別人正在改的地方。
- 假設 D（N5）：Context 段不加新行。現行「related changes/specs」句已規定記進行中變更，只需把命中的 delta capability 一併寫進去。Evidence：技能檔第 188 行。錯了的後果：無，只是少一行機械標記；propose 目前沒有讀 related changes 行的機械行為。
- 假設 E（範圍）：只改 discuss 一支技能，不動 propose／improve 的偵察。Evidence：三支的偵察段各自獨立寫在各自的 asset；本題起因是 discuss。錯了的後果：propose 從討論轉出時同樣看不到進行中 delta，缺口只補一半。
**Ruled out**: 在偵察另開「第四段：進行中變更」——delta 規格與正式規格是同一種東西（承諾），拆段會讓漏斗順序變成四段而失去「規格先行」的單一入口；引擎 list --json 加 capability 欄位——為一支技能的偵察改引擎輸出與 JSON 契約，代價高於用路徑比對，且路徑比對已足夠
**Open**: N2 路徑比對 vs 引擎欄位由使用者裁定；N4 標記文字的確切寫法；假設 E 的範圍（要不要順手補 propose／improve）

### Round 2 — interview (2026-09-16)

**Focus**: 第一輪五條假設是否成立（N2 機制、N3 時間盒、N4 呈現、N5 Context、範圍）
**Position**: 使用者確認五條全對，樹上所有節點解決，進入收斂。
- N2：技能檔規定以路徑比對 openspec/changes/*/specs/<capability>/spec.md，零引擎改動
- N3：delta 命中只讀 Requirement 標題與 ADDED／MODIFIED／REMOVED 標記，至多 3 份
- N4：不加第五類，在「Covered by canon／Conflicts with canon」上加標記「（進行中變更 <name> 將改動）」
- N5：Context 不加新行，命中的 delta capability 寫進既有「related changes/specs」句
- 範圍：只改 discuss 技能
- 具體例：題目命中 client-protocol，且 openspec/changes/add-change-plan-remote/specs/client-protocol/spec.md 存在 → 假設清單該條標為「Covered by canon（進行中變更 add-change-plan-remote 將改動）」，Context 的相關變更句列出 add-change-plan-remote: client-protocol
**Open**: 無

## Conclusion

**Decision**: discuss 技能的偵察順序維持「正式規格 → 舊討論 → 程式碼」不變；規格段擴大為「正式規格＋進行中變更的 delta 規格」。canon pass 命中 capability 名後，技能檔規定順手比對 openspec/changes/*/specs/<capability>/spec.md 是否存在，存在即為進行中 delta 命中：只讀 Requirement 標題與 ADDED／MODIFIED／REMOVED 標記、至多 3 份；假設清單在既有「Covered by canon／Conflicts with canon」上加標記「（進行中變更 <name> 將改動）」；Context 的 related changes/specs 句一併列出命中的 delta capability。零引擎改動、零 JSON 契約改動。
**Rationale**: delta 規格是「即將成為正式規格的承諾」，與正式規格同性質，該在同一段被看到；否則假設清單可能建立在一條正被改掉的規格上，要到 propose 或 drift 才撞到。speclink list --json 不含 capability 清單，但 delta 目錄以 capability 命名，路徑比對已足夠，不值得為一支技能改引擎輸出。
**Rejected alternatives**: 偵察另開「第四段：進行中變更」——拆段破壞「規格先行」的單一入口；引擎 list --json 加 capability 欄位——改 JSON 契約代價高於路徑比對；對照表加第五類——已有四類，再加太重，標記已足以讓使用者看出踩在他人正在改的地方；一併改 propose／improve 偵察——本題起因是 discuss，範圍收斂到一支。
**Deferred**: propose／improve 的偵察段是否比照納入進行中 delta——本題未裁，待 discuss 這一刀落地後視實務有感再議；引擎日後若改 delta 目錄佈局，技能檔的路徑假設會靜默失效——屬既知風險，未設守門。
**Capture to**: spec（discuss-skill 的 delta：「正式規格先行的漏斗偵察」場景與「正式規格接地與三分對照」需求各加進行中 delta 的規定）；事實來源 crates/engine/speclink-core/assets/skills/discuss.md 同步改字（ASSET_VERSION／golden／assets.lock 三連動）
**Next**: /speclink-propose --from-discussion discuss-scout-in-flight-deltas

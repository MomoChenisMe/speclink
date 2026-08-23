---
topic: init 注入 CLAUDE.md/AGENTS.md 指示塊是否改走 OpenSpec 1.0 的無注入＋技能路由作法
slug: init-marker-openspec-alignment
status: promoted
promoted_to: remove-marker-injection
created: 2026-08-20
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: init 注入 CLAUDE.md/AGENTS.md 指示塊是否改走 OpenSpec 1.0 的無注入＋技能路由作法

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者想參考 OpenSpec 1.0 的作法，重新檢視 Speclink init 時注入 CLAUDE.md/AGENTS.md 的 marker 指示塊（SPECLINK:START..END 路由表）。模式：assumptions（掃到 init.rs、skills.rs、main.rs、21 份 skill assets）。相關程式碼：`crates/speclink-core/src/init.rs`（marker 注入與 MARKER_VERSION）、`crates/speclink-core/src/skills.rs`（技能生成）、`crates/speclink-cli/src/main.rs`（已有 status／instructions 動詞）。外部參照：Fission-AI/OpenSpec repo（src/core/templates/workflows/*.ts、CHANGELOG 1.0.0）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-20)

**Focus**: Speclink 現況與 OpenSpec 1.0 架構的落差在哪
**Position**: Speclink 的動態機制（status＝artifact DAG、instructions＝逐 artifact 指示）已存在且被技能消費，但 marker 注入仍在：
- CLI 已有 `speclink status`／`speclink instructions`（main.rs:116-119），propose 技能逐 artifact 走 DAG（propose.md:239-302），apply 消費 preflight＋contextFiles（apply.md）
- marker 塊是約 30 行路由表（情境 → 技能）＋workflow 箭頭行＋政策 bullet（init.rs:135-158），非 OpenSpec 舊版的完整手冊
- 初始四假設：(1) DAG 作法已完成 (2) marker 應瘦身而非加料 (3) 箭頭行是路由提示非階段枷鎖 (4) continue 式技能是獨立功能決策
**Ruled out**: 「為了對齊 OpenSpec 而重構動態指示機制」— status/instructions 機制早已同構，無需重做
**Open**: 使用者指正——OpenSpec 1.0 之後完全不注入 CLAUDE.md/AGENTS.md；題目轉為「Speclink 是否也該取消注入、路由改由技能承載」，需先研究 OpenSpec 技能如何指示下一步

### Round 2 — assumptions (2026-08-20)

**Focus**: OpenSpec 1.0 在無 CLAUDE.md 注入下如何做「下一步」路由（原始碼實查）
**Position**: OpenSpec 把路由拆成三層，全部活在技能與 CLI 裡，沒有任何集中式流程表：
- 入口路由＝skill description frontmatter：每個 SKILL.md 的 description 寫觸發情境（如 continue：「Use when the user wants to progress their change, create the next artifact, or continue their workflow」），host 工具自動載入技能清單即路由表（src/core/templates/workflows/*.ts）
- 狀態內路由＝CLI DAG 狀態：`openspec status --json` 回每個 artifact 的 ready/blocked/done，continue 挑第一個 ready 的建立、建完顯示「what's now unlocked」後 STOP（continue-change.ts）
- 出口交棒＝每個技能 Output 段的固定「recommended next command」：new→continue、continue 完成→apply→archive、ff/propose→apply、apply 全勾→archive、update 三分支且明文「guidance only - NEVER act on it」（各 workflow 模板結尾）
- CHANGELOG 1.0.0 明文：「Config files removed — Tool-specific instruction files (CLAUDE.md, .cursorrules, AGENTS.md, project.md) are no longer generated」；AGENTS.md 系工具改用 vendor-neutral `.agents/skills/` 目錄（1.9.x 起）
- Speclink 對照：出口交棒零星存在（apply.md「If all done: suggest archive」「quality stations can run now」），但 propose/drift 刻意不建議下一步；入口路由目前由 marker 塊集中承載
**Open**: Speclink 是否取消注入？三個未解節點：(1) Speclink 的 skill description 是否足以承載入口路由（現況描述偏短） (2) marker 裡的跨技能政策（drift-first、worktree 閘、quality 順序）搬去哪 (3) custom tool（Invocation::Cli/ToolCall）沒有技能機制，marker 是唯一載體，無法比照全刪

### Round 3 — assumptions (2026-08-20)

**Focus**: 使用者裁定「全面取消注入」後，逐一驗證三個未解節點是否有落點
**Position**: 三個節點都有現成或可補的家，全面取消技術上成立：
- 入口路由：現況 skill description 是動詞句（如 apply「Implement or resume tasks from a Speclink change」，skills.rs registry()），要改寫成 OpenSpec 式觸發情境句（「Use when...」）才能承載路由
- 跨技能政策各有新家：「恢復先跑 drift」引擎已在 apply 的 instructions preflight 回 drifted files＋staleness（apply.md 3b 步）；「archive 只能主 checkout」引擎本來就拒絕，worktree 技能出口交棒句可載明；「plan mode → ingest」搬進 ingest description 與 apply body；specs 路徑宣告已透過 {{SPEC_DIR}} 代換烙進技能本文，marker 的 store paragraph 是冗餘
- custom tool 不是阻礙：render_skill_file_custom 為每份 SKILL.md 內嵌 Invocation 前言（cli/tool-call 怎麼跑動詞），marker 的 invocation line 本來就重複；instructions_file 欄位現為必填（config.rs require_field），取消注入後轉 optional/棄用
- 波及面：init.rs（注入＋refuse_downgrade＋status probe 的 differing_files）、update、desktop core（project.rs marker 版號探測、settings.rs 同步測試斷言 marker 寫入/剝除）、render_golden、既有使用者 repo 的 marker 剝除遷移
**Ruled out**: 「custom tool 保留注入」的分層方案 — 技能已自帶 invocation 前言，無保留必要
**Open**: (1) 遷移方式：update 時自動剝除既有 marker，或提示確認後剝除 (2) 出口交棒句要不要有集中式流程總表的替代品，或完全去中心化 (3) 一個 change 全包或拆多個

### Round 4 — assumptions (2026-08-20)

**Focus**: 三個收斂旋鈕的裁定
**Position**: 使用者三題皆採建議選項：
- 遷移＝update 自動剝除既有 SPECLINK:START..END 區塊（引擎已有 remove_marker，只動區塊不動使用者內容）
- 流程總表＝完全去中心化：description 管入口、各技能結尾交棒句管出口，整張圖由交棒句拼出，不留集中總表
- 切法＝一個 change 全包（取消注入、description 改寫、交棒句補齊、遷移、desktop 調整互相依賴，拆開會有雙重路由來源的中間態）
**Ruled out**: 提示確認後剝除（多一步無實益，OpenSpec 前例即自動清理）；CLI 留流程總表／onboard 兼任（第二份要同步的真相）；拆兩個 change
**Open**: 無——實作層細節（description 逐句措辭、交棒句邊集、MARKER_VERSION 後續語意、desktop UI 替代呈現）留給 propose/design

### Round 5 — assumptions (2026-08-20)

**Focus**: 交棒句完整邊集（原 Deferred 項提前展開）與 onboard 類技能的去中心化處置
**Position**: 技能分四類，各自維護出邊，marker 的每條 bullet 都有對應邊、無孤兒：
- 流程鏈：propose→apply（preflight 管入場，drifted/stale→drift）；apply 全勾→品質站或 archive、剩 [M]→品質站可先跑；apply⇄ingest；review 落章→另一站或 archive；verify 落章→archive；quality 每輪停、兩站落章→archive（worktree 內→worktree-merge）；apply-with-worktree→品質站→worktree-merge→主 checkout archive；archive＝終點（linked discussion 隨行封存）
- 入口技能：onboard 出口兩條邊（需求清楚→propose、模糊→discuss），不重抄命令總表——總表職責由 host 技能清單（descriptions）承擔；discuss 依結論分岔（promote／link+ingest／archive／discard）；improve 記錄成討論後同 discuss
- 工具技能（commit、analyze、audit、config、drift）：隨叫隨用不佔流程位，出口回呼叫脈絡；analyze 缺口→建議 ingest；drift 依發現→ingest 或 apply
- 內部技能（tdd、clarify、sync）：經 instructions --skill 取用，不參與路由
**Ruled out**: onboard 結尾放命令總表（OpenSpec 做法）——與「不留集中總表」裁定衝突且冗餘
**Open**: 無——本輪即 Deferred 項「交棒句邊集」的落地草案，propose 直接取用

### Round 6 — assumptions (2026-08-20)

**Focus**: 立案前檢查——進行中規格改動的衝突盤點與文件更新範圍
**Position**: 三個進行中 change（皆 0 任務未開工）全數與本規劃重疊，施工順序＝兩個技能 change 先落地、本規劃殿後：
- capability-naming-guard：任務 4.1 改 propose.md＋ingest.md、4.2 bump MARKER_VERSION＋golden＋assets.lock——與本規劃同檔同版號行同 golden，直接對撞
- discuss-grounding-and-flow：整包重寫 discuss.md＋版號＋golden——直接對撞
- desktop-schema-panel：動 settings.rs（本規劃要改該處 marker 同步測試斷言）；其 delta 打在 desktop-config spec，本規劃也要改同 spec 的「tools 變更後技能同步」scenario——檔案級＋spec 級重疊、功能正交，可平行但合併留意順序
- 順序理由：兩個技能 change 改「內容」、本規劃改「載體與路由」，內容先定稿則 description／交棒句改寫以新版資產為底；反序則對方 delta 假設漂移、各需一輪 ingest
- workspace-tools spec 的「marker 技能指引跟隨 worktree 政策」需求將被移除或改寫，propose 時須明示宣告（引擎不抓未宣告的 scenario 刪除）
- 文件更新補進 scope：docs/getting-started(.zh-TW)、configuration(.zh-TW)、verb-contract.md（zh-TW 版 apply 時一併查）、platform-architecture.zh-TW、server-store-drivers.zh-TW、implementation-refactor-roadmap.zh-TW
**Open**: 無——結論不變，本輪為立案前的排程與範圍補充，propose 直接取用

### Round 7 — assumptions (2026-08-22)

**Focus**: 第二批進行中 change（tdd-switch-apply-wiring、feature-provenance-skill）的衝突複查
**Position**: 前一批三個 change 已完成；新兩個（皆 0 任務）同型對撞，結論不變、邊集與時機微調：
- tdd-switch-apply-wiring：改 apply.md／tdd.md／ingest.md／propose.md／onboard.md 字句＋版號三連動＋docs/configuration 兩語言版——與本規劃同檔不同區段，版號行／golden／assets.lock／docs 清單全撞
- feature-provenance-skill：新增技能 trace.md 並註冊 registry＋版號三連動——第 5 輪邊集由 17 技能擴為 18：trace 歸工具技能類（入口＝使用者問功能溯源；出口＝敘事答案、無固定下一步），description 改寫清單含之
- 立案時機維持殿後：兩 change 落地後再 propose，description／交棒句以定稿資產（apply.md 新步驟 5、trace.md）為底；若先 propose 則開工前須 drift → ingest 校正
**Open**: 無——結論不變，propose 前若 change 看板再變動，重跑一次本輪的衝突複查

## Conclusion

**Decision**: 全面取消 init/update 對 CLAUDE.md/AGENTS.md（含 custom tool instructions_file）的 marker 注入，路由全數改由技能承載——skill description 改寫為觸發情境句管入口、各技能結尾補「建議下一步（只建議、絕不代跑）」交棒句管出口、CLI 的 status/instructions/preflight 管狀態內路由；`speclink update` 自動剝除既有 marker 區塊；完全去中心化不留集中流程總表；一個 change 全包。
**Rationale**: Speclink 的動態指示機制（status＝artifact DAG、instructions＝逐 artifact 範本、preflight＝入場檢查）早已與 OpenSpec 1.0 同構，marker 是靜態重複層——每字改動觸發 MARKER_VERSION→golden→assets.lock 三連動並波及所有使用者 repo 重注入；OpenSpec 1.0 已實證無注入路線可行（CHANGELOG 1.0.0 明文移除 CLAUDE.md/.cursorrules/AGENTS.md 生成）。
**Rejected alternatives**: 瘦身保留極小 marker（specs 路徑已透過 {{SPEC_DIR}} 烙進技能本文，冗餘）；custom tool 分層保留注入（render_skill_file_custom 已為每份 SKILL.md 內嵌 Invocation 前言）；集中式流程總表（CLI 或 onboard 兼任都是第二份要同步的真相）；提示確認後才剝除 marker；拆兩個 change（中間態存在雙重路由來源）。
**Deferred**: description 逐 skill 的觸發情境句措辭、交棒句的完整邊集（哪個技能出口指向哪些技能）、MARKER_VERSION 在無 marker 後的命名與語意（仍須管技能版本）、desktop marker 狀態 UI 的替代呈現——皆留給 propose/design。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion init-marker-openspec-alignment

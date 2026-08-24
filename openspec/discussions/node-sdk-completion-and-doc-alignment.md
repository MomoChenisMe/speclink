---
topic: JS 端 Node SDK 做齊（npm 通路＋Node Host 面）＋規劃文件對齊程式碼現況
slug: node-sdk-completion-and-doc-alignment
status: promoted
promoted_to: engine-npm-publish, node-host-actor
created: 2026-08-23
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: JS 端 Node SDK 做齊（npm 通路＋Node Host 面）＋規劃文件對齊程式碼現況

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

SDK 完成度盤點對話（2026-08-23）後，使用者裁定：MCP/Copilot tools 不動、Rust 不發 crates.io（git 依賴即可），先把 JS 端 Node SDK 做齊——npm 通路與 Node Host 面（actor 注入）——並同步檢查規劃文件對齊程式碼現況。2026-08-12 的 release-first-and-distribution 討論已裁定「npm 發布留待 v0.2 與 engine 一起規劃」，本討論即該規劃。需求可驗證，無 grill 階段，直接假設清單＋逐條 codebase 比對。相關 specs：node-sdk（SDK 契約正典）、host-runtime（ExecutionContext 注入約束）、cli-distribution／desktop-release／server-release（發布通路鄰居）。相關基建：.github/workflows/node-sdk.yml（五平台 tarball 已在打）、release.yml npm-publish job（@speclink/server 先例）、scripts/npm-server-package.mjs（版號蓋章先例）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-23)

**Focus**: 「做齊 Node SDK」的決策空間展開——npm 通路（A）、Host 面（B）、文件對齊（C）三軸六假設
**Position**: 六條假設全數以現有基建與 canon 為據，可行方向成立：
- A1 npm 發布＝把 node-sdk.yml 已在打的五平台 tarball 接進 release.yml publish job，照 @speclink/server 先例（NPM_TOKEN 缺席跳過）
- A2 @speclink/engine 版號隨 release tag 蓋章，與 workspace 版本同步
- B1 actor 注入只能放 createEngine 建構期——host-runtime spec 明定 ExecutionContext 由 Host 解析一次、Command 輸入不得含 actor；多人場景＝每請求一個輕量 engine 實例
- B2 不做獨立 @speclink/host 套件，Host 選項收進 createEngine——藍圖三件套是過期構想，為單一 actor 選項開套件是為一次性使用建抽象層
- C1 文件對齊面＝sdk-node、product-status、roadmap 各中英六檔；platform-architecture 藍圖不動（已自我標註過期）
- C2 文件隨 change 收尾改，不另開文件專用 change（8/12 討論同款先例）
- Canon triage：node-sdk spec 對 npm 通路與 Host 面均沉默（新地盤＝delta）；B1 受 host-runtime 約束；發布面鄰居為 cli-distribution／server-release
**Open**: 使用者要求逐條比對 codebase 驗證後再定案；canon 落點（npm 通路進 node-sdk spec 或仿 server-release 開新 capability）待定

### Round 2 — assumptions (2026-08-23)

**Focus**: 六假設逐條 codebase 比對——哪些成立、哪些要修
**Position**: 4 條證實、2 條修正，另挖出一個版號漂移：
- A1 修正：release.yml 只在 v* tag 觸發、node-sdk.yml 只在 push main/PR 觸發，兩邊無 workflow_call/workflow_dispatch 橋，跨 workflow run 的 artifact 拿不到——接法改為把 node-sdk.yml 的 build/pack 改成 workflow_call 由 release.yml 呼叫（不複製五平台 matrix）
- A2 修正：crates/speclink-node/package.json:3 版號 0.1.0、workspace 0.1.3——已經漂移；改採 server 式蓋章（npm-server-package.mjs --version 先例），repo 內版號當佔位符，發布產物版號一律由 tag 決定；desktop 式驗證（release.yml:201-204）被漂移事實否定
- B1 證實且落點更順：actor 是 ExecutionContext 現成欄位，消費點在 newcmd.rs（created_by 章）與 station.rs:481-482（review/verify 章 _by）；newcmd.rs:201-204 已有「new change 收明確 actor」測試釘住注入語意；Node 側只需把 run_engine（lib.rs:83-96）寫死的 git_identity 換成「建構期 actor 優先、fs 模式回退 git identity」；per-request 實例成本可接受（engineFromFs 只存 PathBuf；engineFromStore 每次 validate＋建一條 ThreadsafeFunction）
- B2 證實：packages/ 只有 server-npm 與 ui，無 host 套件雛形
- C1 擴充：「尚未發布至 npm」宣稱實際散在 9 處，對齊面收為 8 檔——原六檔外補 README.md:35 與 README.en.md:41
- C2 維持（先例在 8/12 討論記錄，無 code 可驗）
**Ruled out**: release.yml 內複製五平台 build matrix（重複程式碼，workflow_call 可避免）；desktop 式版號一致性驗證（repo 內版號已證明會漂，蓋章物化才可靠）
**Open**: canon 落點（node-sdk delta vs 新 capability）；一個 change 或拆兩個——待定案裁決

## Conclusion

**Decision**: 做齊 JS 端 Node SDK，拆兩個可並行的 change：
1. npm 通路（暫名 engine-npm-publish）——node-sdk.yml 的 build/pack 改 workflow_call 由 release.yml 呼叫、新增 engine 的 npm-publish job（照 @speclink/server 先例：NPM_TOKEN 缺席跳過、npm publish --access public）、版號採 server 式 tag 蓋章（repo 內 package.json 版號為佔位符）；canon 落點＝新 capability node-sdk-release（與 desktop-release／server-release 命名對稱）
2. Node Host 面（暫名 node-host-actor）——createEngine 增建構期 actor 選項（"Name <email>" 字串），接進 ExecutionContext 現成 actor 欄位：有給值用給值、fs 模式未給回退 git identity、JS Store 模式未給維持無章（現行為不變）；canon 落點＝node-sdk spec delta
文件對齊 8 檔（sdk-node／product-status／roadmap／README 各中英）按歸屬隨兩個 change 收尾改：發布宣稱歸 change 1、createEngine 契約段歸 change 2；「尚未發布」的翻面文案以「管線已接、自下個 release tag 起上 npm」表述，實際可 npm install 以首個帶 engine 的 release 為準。
**Rationale**: 五平台 tarball 管線已在 CI 打好，缺的是 workflow 邊界的一刀（workflow_call）與 publish job——工程量是「接」不是「建」；actor 是引擎現成欄位且有測試釘住注入語意，Host 面缺口只是 Node 建構期沒把手，為它開獨立套件違反 YAGNI；兩刀觸及檔案幾乎不相交（CI/workflow vs crates/speclink-node），拆開可 worktree 並行。
**Rejected alternatives**: 獨立 @speclink/host 套件（藍圖三件套是過期構想，單一 actor 選項撐不起一個套件）；dispatch 參數帶 actor（違反 host-runtime「ExecutionContext 由 Host 解析一次、Command 不含 actor」canon，等於身分偽造旁路）；release.yml 複製五平台 build matrix（重複程式碼）；desktop 式版號一致性驗證（node package.json 已漂移至 0.1.0 vs workspace 0.1.3，證明 repo 內版號不可信，蓋章物化才可靠）；另開文件專用 change（與功能 change 搶同檔）。
**Deferred**: MCP／Copilot tools adapter（使用者裁定不動，觸發條件＝有非 Claude/Codex 的 agent 平台要接）；@speclink/host 拆套件時點（等 Host 面需求超過 actor 注入再議）；實際發版的 tag 時點與版號（發版時決定，8/12 討論的版號策略不變）；npm/ 平台子套件目錄的物化細節與 npm org 權限設定（propose 階段落地）。
**Capture to**: proposal（兩個 change）
**Next**: /speclink-propose --from-discussion node-sdk-completion-and-doc-alignment（跑兩次，各建一個 change）

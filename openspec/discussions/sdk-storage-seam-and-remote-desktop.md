---
topic: SDK／儲存縫優先、headless 為 demo、遠端模式運算位置與 desktop 遠端改動
slug: sdk-storage-seam-and-remote-desktop
status: promoted
promoted_to: speclink-sdk-and-store-seam, web-server-postgres, desktop-remote-mode
created: 2026-07-09
---

# Discussion: SDK／儲存縫優先、headless 為 demo、遠端模式運算位置與 desktop 遠端改動

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

web-server-postgres 剛 propose 完（design＋specs＋tasks 已驗證）。使用者提出三點挑戰其設計：(1) 雖說要「開箱即用 headless server」，實際希望先包出 SDK 功能、拆出儲存邏輯，headless 只是 SDK＋儲存邏輯之上的簡易方案兼使用範例；不是每個使用者都直接用 headless，可能整合自家系統＋SDK＋自家儲存；(2) 質疑 design D7——遠端模式下 validate/analyze 也該在遠端跑，不該 client 端；(3) 遠端設定後卡片 spec 資料、config.yaml 皆來自遠端，只有 .speclink.yaml 本地，故 desktop 有一定量改動。

模式：assumptions——甫完成本刀整套 propose 研究，對 @speclink/engine（SDK）、Store trait／JS bridge（儲存縫）、desktop SettingsView／WorkspaceAdapter、docs/verb-contract.md 正典皆有一手掌握。

相關正典：verb-contract（LIVE，薄；委派 docs/verb-contract.md）、remote-connection、remote-auth、node-sdk、store-abstraction、change-lifecycle。相關變更：web-server-postgres（本討論精修對象）、web-agent-channel（下游）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-09)

**Focus**: web-server-postgres 實際交付什麼？desktop 遠端改動是否 trivial？
**Position**: SDK＋儲存縫是既有資產、本刀補完並重新定位，headless 降為 demo；desktop 遠端非 trivial。
- SDK（@speclink/engine＝crates/speclink-node，napi 已發 npm）＋ Store 縫（speclink-core/src/store.rs trait＋store_bridge.rs JS bridge）皆已存在——work 是補完 dispatch 全動詞＋當公開整合面文件化，非新建。
- pg Store 定位為可複用參考實作；headless server 降為薄 demo（demo 級認證），正式整合者換掉整個 server 層、自帶認證與自家儲存。
- desktop 遠端模式：瀏覽路徑（看板/文件/spec/討論經 SpeclinkDataSource→IPC→desktop-core→RemoteClient）經後端替換對前端透明；但設定面分叉——config.yaml 來自遠端且唯讀（GET /config；契約 §5 PUT config 屬 host-admin），WorkspaceAdapter 的 writeWorkflowConfig/Context/Rules 在遠端無寫入標的；.speclink.yaml 本地＋新增遠端卡。
**Ruled out**: 「前端一位元不動」過強（僅瀏覽路徑成立）；把認證做成引擎/SDK 的縫（認證天生 server 層，整合者替換整個 server 層，不複用 demo 認證）。
**Open**: 遠端模式 validate/analyze 的運算位置（使用者質疑 client-side）；是否把本刀重切三刀（SDK＋縫／demo server／desktop 遠端）。

### Round 2 — assumptions (2026-07-09)

**Focus**: 遠端模式下 validate/analyze/drift 在哪運算？LLM 在遠端如何讀文件？
**Position**: 採使用者主張——遠端＝server 端運算；LLM 一律經 CLI 動詞讀文件（無本地檔）。
- 運算位置：遠端模式一律 server 端算（CLI 與 desktop 皆然）。最強理由是 team 一致性——client 端算會因各人 CLI/引擎版本歧異，讓全隊對同一 server 資料的 analyze/validate/drift 結果分裂；team server 應釘住分析語意。次要理由：模型一致（remote 一律 server 算）＋強化 SDK-first（server＝內嵌 SDK 的運算樞紐）。
- 代價：此修訂 verb-contract §6/§7（原把 analyze/validate/drift/show 判 client-side，理由為 CLI 內嵌引擎）——需新增 analyze/validate/drift 端點、CLI remote 路由改打端點、動到已封存 verb-contract 正典（delta 修訂），範圍上升。
- LLM 讀取：遠端模式無本地 openspec/ 檔；agent 一律以 CLI 動詞讀「文件」而非「檔案」——artifact cat / language show / discuss show / show <spec>，round-trip 至 server。remote init 的 CLAUDE.md/AGENTS.md marker 區塊＋remote 技能變體強制用動詞、禁讀路徑（remote-connection spec 既有兩需求）。真風險：agent 慣性 Read/grep openspec/（遠端不存在）→ remote 技能須強力改導。server 端運算讓 analyze/validate 變薄動詞，反而簡化 agent 故事。
- .speclink.yaml 為單一模式開關：CLI 與 desktop 同讀（Workspace::resolve_mode），設 remote 區段即同時翻兩者為遠端——確認為既定設計，非新問題。
**Ruled out**: client 端運算於遠端模式（全隊版本歧異、模型不一致）——惟對現有內嵌 Rust 引擎的 client（CLI、desktop-core）技術上可行，故此為刻意的正典修訂而非被迫。
**Open**: analyze/validate/drift 端點形狀（design 細節）；是否重切三刀（待使用者拍板）。

### Round 3 — assumptions (2026-07-09)

**Focus**: 本刀是否重切、怎麼切？
**Position**: 切三刀，依賴 ①→②→③、各自可獨立驗證，最貼合「SDK 先包出、server 為延伸範例、desktop 為消費者」。
- ① speclink-sdk-and-store-seam（可複用核心）：補完 @speclink/engine dispatch 全遠端託管動詞集；新增 server 端可算的 analyze/validate/drift 路徑；Store 縫文件化為公開整合面；修訂 verb-contract 正典（遠端＝server 運算，新增 analyze/validate/drift 端點、CLI remote 路由至之、agent 遠端一律經動詞讀文件的 marker/技能改導）。
- ② web-server-postgres（重定範疇＝範例）：pg 參考 Store（乾淨可複用）＋薄 demo server 包 createEngine＋pg Store＋demo 級認證（整合者換掉整個 server 層）＋SSE/LISTEN-NOTIFY＋docker-compose。
- ③ desktop-remote-mode（消費者、實質工程）：desktop-core 後端替換（resolve_mode→RemoteClient）；瀏覽路徑透明、設定面分叉（config.yaml 遠端唯讀、WorkspaceAdapter 寫 config 無標的、.speclink.yaml 本地＋遠端卡）；PAT 憑證；SSE client 即時刷新。
**Ruled out**: 兩刀（server 基座／desktop）與維持一刀——一刀已 ~45+ 任務且動正典；點 1「SDK 先包出」要求 SDK/縫成可獨立交付頭牌，兩刀把 SDK 埋在 server 裡不符該意圖。
**Open**: 各 change 的端點/artifact 細節（各自 propose 定）；② 既有全包 artifacts 的重寫路徑（re-propose 重寫 vs ingest 收窄）。

## Conclusion

**Decision**: 把 roadmap 原第 ③ 刀 web-server-postgres 重切為三刀，並確立「遠端＝server 端運算」與「SDK/儲存縫為頭牌、headless 為 demo」。
- ① speclink-sdk-and-store-seam（可複用核心）：@speclink/engine dispatch 補完全遠端託管動詞集；新增 analyze/validate/drift 的 server 端運算路徑；Store 縫文件化為公開整合介面；修訂 verb-contract 正典——遠端模式 analyze/validate/drift 改 server 端算（新增端點、CLI remote 路由至之），並確立 agent 遠端一律經 CLI 動詞（artifact cat／language show／discuss show／show）讀「文件」而非本地檔、由 remote marker 區塊＋remote 技能變體強制改導。
- ② web-server-postgres（重定範疇為範例）：pg 參考 Store（乾淨可複用）＋薄 demo headless server（包 createEngine＋pg Store）＋demo 級認證（明標「整合者換掉整個 server 層、自帶認證與自家儲存」）＋SSE/LISTEN-NOTIFY＋docker-compose。
- ③ desktop-remote-mode（消費者）：desktop-core 依 resolve_mode 後端替換複用 RemoteClient；瀏覽路徑對前端透明，設定面分叉（config.yaml 遠端唯讀、WorkspaceAdapter 寫 config 無標的、.speclink.yaml 本地＋遠端卡）＋PAT 憑證＋SSE client 即時刷新。
**Rationale**: (1) 遠端＝server 運算的關鍵理由是 team 一致性——client 端算會因各人引擎版本歧異使全隊 analyze/validate/drift 結果分裂；team server 應釘住分析語意。(2) SDK/儲存縫已是既有資產（@speclink/engine napi 套件＋Store trait/JS bridge），使用者要它們成可獨立交付、可整合自家系統的頭牌，headless 只是簡易方案兼範例——三刀讓 ① 可獨立交付、②③ 增量疊加。(3) desktop 遠端非 trivial（設定面分叉），且併入 server 端運算＋契約修訂後單刀已 ~45+ 任務並動正典，超出可維護規模。
**Rejected alternatives**: 維持一刀全包（規模爆炸、動正典、把 SDK 埋在 server 裡不符「SDK 先包出」）；兩刀 server 基座＋desktop（同把 SDK 埋在 server 內）；遠端模式 validate/analyze 仍 client 端算（team 版本歧異、模型不一致——惟對現有內嵌 Rust 引擎的 client 技術可行，故 server 端為刻意的正確性選擇）；認證做成引擎/SDK 的縫（認證天生 server 層、整合者替換整個 server 層）。
**Deferred**: 各 change 端點/artifact 細節（各自 propose 定）；verb-contract 修訂以 delta 動已封存 spec 的具體形式；② 既有全包 artifacts 的重寫路徑；board_rank 於遠端模式；pg 測試載具。
**Capture to**: proposal + design（跨三個 change：新建 ①③、重定範疇 ②）
**Next**: /speclink-propose --from-discussion sdk-storage-seam-and-remote-desktop --name speclink-sdk-and-store-seam

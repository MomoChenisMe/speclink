---
topic: 四情境預設 GUI 工具矩陣
slug: 四情境預設-gui-工具矩陣
status: promoted
promoted_to: desktop-shell-and-browser, desktop-acp-agent, web-server-postgres, web-agent-channel, web-role-views
created: 2026-07-05
---

# Discussion: 四情境預設 GUI 工具矩陣

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

承接 sdd-engine-as-sdk 討論定案的四種部署情境（1 PO 在系統/RD 本地、2 全在 Agent 系統、3 本地 CLI 連遠端文件、4 全本地）。使用者要求每個情境都有「使用者看得到、可實際使用的預設 GUI 工具」，而非僅測試劇本——這正式推翻該討論將 `speclink serve` 判為 YAGNI 遞延的結論。

模式：assumptions——相關原始碼超過 3 處（crates/speclink-node 的 index.d.ts 已公開 createEngine/dispatch/render API、crates/speclink-remote 的 client/auth、specs/verb-contract 正典、鄰近 D:/Git/wadpilot 生產碼已用 Copilot SDK defineTool）。

參照實作：反組譯使用者本機安裝的 Spectra 桌面版（spectra 2.3.1 x64，C:\Users\momoc\AppData\Local\Spectra\{app.exe,spectra.exe}）取得完整架構參照。

已定方向：情境 4 單機桌面版先做；儲存選 PostgreSQL；順序 4→3→2→1（依賴鏈：情境 3 的 web server 是 1/2 的共同底座）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-05)

**Focus**: 工具矩陣該跟「情境」還是跟「兩正交軸」做？以及 serve/MCP 的角色。
**Position**: 跟軸做，不跟情境做。四情境=兩軸（引擎執行位置 × 文件存放位置）組合，只需兩個元件即可組出全部：(a) 一個 web 應用（Node，經 createEngine 內嵌引擎）一身三職——對外暴露動詞契約 REST 端點（情境 1/3 的 CLI link 上來）＋GUI 看板/文件瀏覽＋Copilot agent 通道（情境 2）；(b) 現成 speclink CLI（fs 模式=情境 4，remote 模式對 web server=情境 3）。情境 1 是同一 web 應用的「角色切面」（PO 用 web、RD 本地 CLI），非獨立交付物。討論遞延的 speclink serve 被 web 應用吸收。
**Ruled out**: 每情境各做一工具（情境 1/3 共用同一 server 會複製實作；情境 4 的工具其實已存在=CLI；四工具=四份跟引擎版本走的維護負擔）。
**Open**: web/桌面技術棧；儲存與認證最小實作；分幾刀與順序；spectra.exe 的 GUI 具體形態。

### Round 2 — assumptions (2026-07-05)

**Focus**: 反組譯 Spectra 桌面版（spectra 2.3.1 x64）取得情境 4 的參照架構。
**Position**: 桌面版=Tauri 2.11.1 殼 ＋ SvelteKit 前端 ＋ spectra-core Rust「直嵌」（非 sidecar，app_lib::commands 為 Tauri invoke handler 直呼 core）。四項關鍵發現：(1) 內建 agent = GitHub Copilot CLI 的 ACP 模式（字串證據 `copilot --acp --model`、commands::acp_session、整套 ACP 訊息型別 SessionUpdate/AgentMessageChunk/ToolCall/RequestPermissionOutcome）——桌面 app 自身即 agent host，不依賴使用者另開 Claude Code；這與「情境 1/2 用 Copilot SDK」是同一 agent 機制、不同部署。(2) SQLite 是「檔案系統之上的快取/索引」非真相——markdown 檔（proposal.md/tasks.md，git 跟隨）仍是真相，DB（archived_cache_v14、parked_changes、worktree_artifact_ownership）給 GUI 快速列表/park/worktree 狀態，帶版本會 migration。(3) 支援 git worktree（WorktreePickerModal）。(4) spectra.exe(8.3MB) 為獨立 CLI 24 動詞，可 auto_install 到 PATH；app.exe(27MB) 與其分離。
**Ruled out**: 假設 4「桌面工具不內建 agent、純儀表板」被推翻——參照實作證明桌面版含 agent，使用者要求「像 spectra.exe」故情境 4 應含 agent。以 sidecar 呼叫 spectra.exe 的猜測被推翻——core 是直嵌。
**Open**: speclink 桌面版的 agent 選 Copilot ACP 還是 Claude Code ACP（或都支援）；PostgreSQL 在單機桌面版是否過重（spectra 用嵌入式 SQLite 正因零依賴）；技術棧沿用 Tauri+SvelteKit 還是自選。

### Round 3 — assumptions (2026-07-05)

**Focus**: 儲存、agent、技術棧三項最小實作拍板。
**Position**: (1) 儲存依情境分開——情境 4 桌面用嵌入式 SQLite 當快取/索引、本地 openspec/ markdown 為真相（git 跟隨，仿 spectra，零依賴離線可跑）；情境 1/2/3 web server 用 PostgreSQL（store 即真相，多人並發/中央治理）。(2) agent 雙支援——桌面與 web 皆可在 settings 切換 GitHub Copilot ACP 與 Claude Code ACP（v1 維護兩條 ACP 接線）。(3) 技術棧 Tauri 殼直嵌 speclink-core（同步 Rust，零成本內嵌）＋前端採 React/其他可跨桌面與 web 共用的框架——情境 4 桌面與情境 1/2/3 web 共用一套前端元件庫（看板/文件樹/spec 瀏覽/agent 對話面）。
**Ruled out**: 全情境統一用 PostgreSQL（單機桌面需常駐 DB server，違背 spectra 式零依賴雙擊即跑，且脫離情境 4「文件 git 跟隨」定義）；桌面用 SQLite 當真相（放棄 git 跟隨、與 spectra 架構分歧、情境 4→3 遷移需重做）；前端沿用 SvelteKit（選可與 web 共用元件的框架優先，Copilot/Node 生態偏 TS/React）。
**Open**: 分幾個 change、先後順序與各刀交付邊界。

### Round 4 — assumptions (2026-07-05)

**Focus**: 分幾刀、順序與交付邊界。
**Position**: 5 刀，序 4→3→2→1：① desktop-shell-and-browser（Tauri 殼直嵌 speclink-core ＋ SQLite 索引快取 ＋ React 共用元件庫看板/文件樹/spec 瀏覽 ＋ 動詞操作 list/show/status/validate/analyze/archive/park）→情境 4 儀表板；② desktop-acp-agent（桌面對話 agent 面板，雙 ACP Copilot＋Claude 切換）→情境 4 spectra-complete；③ web-server-postgres（Node createEngine 內嵌引擎 ＋ TypeScript PostgreSQL Store ＋ 動詞契約 REST 端點 ＋ 復用 React 元件當 web GUI）→情境 3；④ web-agent-channel（web 內嵌 Copilot SDK agent）→情境 2；⑤ web-role-views（角色切面 PO web/RD 本地交接）→情境 1。實作洞察：PostgreSQL 非新 Rust crate 而是 web 應用裡 TypeScript 實作的 Store——Node SDK store bridge 已支援 JS 實作 Store（index.d.ts 已定義），較另開 Rust adapter 簡單且合設計。依賴鏈：①的 React 元件庫→③復用；③的 Postgres store＋契約 server→④⑤增量疊加。
**Ruled out**: 全用 Rust speclink-postgres adapter（web 走 Node/Copilot 生態，JS 實作 Store 更直接）；④⑤ 合刀（角色切面與 agent 通道關注點不同，分刀可獨立驗證）。
**Open**: 無（技術棧細節留 design 階段）。

## Conclusion

**Decision**: 為四種部署情境各交付預設 GUI 工具，以兩元件（Tauri 桌面 app ＋ Node web 應用）加一套跨桌面/web 共用的 React 元件庫組出全部，切 5 個 change 依序 4→3→2→1 實作。① desktop-shell-and-browser：Tauri 殼直嵌 speclink-core ＋ SQLite 索引快取 ＋ React 共用元件庫（看板/文件樹/spec 瀏覽）＋ 動詞操作（list/show/status/validate/analyze/archive/park），交付情境 4 儀表板。② desktop-acp-agent：桌面對話 agent 面板，雙 ACP 接線（GitHub Copilot ＋ Claude Code，settings 切換），情境 4 達 spectra-complete。③ web-server-postgres：Node 應用經 createEngine 內嵌引擎 ＋ TypeScript PostgreSQL Store ＋ 動詞契約 REST 端點 ＋ 復用 React 元件為 web GUI，交付情境 3（CLI remote 有真 server 可連）。④ web-agent-channel：web 內嵌 Copilot SDK agent 通道，交付情境 2。⑤ web-role-views：web 角色切面（PO web 端 discuss/propose/archive、RD 本地 CLI apply/verify 交接），交付情境 1。儲存依情境分開：情境 4 桌面用嵌入式 SQLite 快取＋本地 openspec/ markdown 真相（git 跟隨）；情境 1/2/3 web 用 PostgreSQL（以 TypeScript 實作 Store，非 Rust crate）。
**Rationale**: 四情境=兩正交軸（引擎執行位置 × 文件存放位置）組合，跟軸做只需兩元件即覆蓋全部，跟情境做會複製 server 實作與維護負擔。依賴鏈使 4→3→2→1 最省：① 打底 React 共用元件庫供 ③ 復用，③ 打底 Postgres store＋契約 server 使 ④⑤ 成純增量，每刀各解鎖一個完整情境且無回頭重做。儲存分開因單機桌面用 PostgreSQL 是重依賴（需常駐 DB server），違背 spectra 式零依賴雙擊即跑並脫離情境 4「文件 git 跟隨」定義。反組譯 spectra 2.3.1 佐證整套參照架構（Tauri 2.11.1＋SvelteKit＋spectra-core 直嵌＋Copilot CLI ACP＋SQLite 快取），並證明桌面版內建 agent。
**Rejected alternatives**: 每情境各做一獨立工具（情境 1/3 共用同一 server 會複製實作、情境 4 工具其實已是 CLI、四工具=四份跟引擎版本走的維護負擔）；全情境統一 PostgreSQL（單機需常駐 DB、違零依賴、脫離 git 跟隨定義）；桌面用 SQLite 當真相（放棄 git 跟隨、與 spectra 分歧、情境 4→3 遷移需重做）；全用 Rust speclink-postgres adapter（web 走 Node/Copilot 生態，JS 實作 Store 更直接且 SDK 已支援）；前端沿用 SvelteKit（選可跨桌面/web 共用元件的框架優先）；④⑤ 合刀（角色切面與 agent 通道關注點不同，分刀可獨立驗證）。同時推翻既有討論：speclink serve 的 YAGNI 遞延（被 web 應用吸收，不再是獨立交付物）。
**Deferred**: 技術棧細節（React 具體選型、Tauri 版本、SQLite schema、ACP 接線細節、PostgreSQL schema）留各 change 的 design 階段定案。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion 四情境預設-gui-工具矩陣 --name desktop-shell-and-browser

---
topic: remote 模式 CLI 指令 parity 缺口盤點（起因：discuss --slug 不支援，產生全中文 slug 空記錄）
slug: remote-verb-parity
status: promoted
promoted_to: remote-cli-parity
created: 2026-07-30
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: remote 模式 CLI 指令 parity 缺口盤點（起因：discuss --slug 不支援，產生全中文 slug 空記錄）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在另一 repo（wad-old-web）以 remote 模式跑 /speclink-discuss 時，CLI 拒絕 `--slug`（remote_commands.rs:1056 明文 bail），agent fallback 改用無 --slug 的 `discuss new`，slugify 刻意保留 CJK（util.rs:151），產出全中文 slug 的 0 輪空記錄；remote 又無 `discuss discard`（:1117 bail），空殼清不掉。模式：assumptions（本 repo 即 speclink，相關原始碼 6+ 檔：remote_commands.rs、routes.rs、protocol/command.rs、remote/client.rs、core/command/mod.rs、core/util.rs）。關鍵前提：引擎層已完整支援 DiscussNew { slug: Option } 與 DiscussDiscard { slug, force }，缺口全在 remote 傳輸鏈（protocol → server 路由 → remote client → CLI）。使用者確認假設後，範圍擴大為「remote 模式所有指令的 parity 全面盤點」。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-30)

**Focus**: 事故成因與修法方向——remote discuss 的 --slug／discard 缺口怎麼補
**Position**: 修法是「接通管線」不是新功能，四項假設全數獲使用者確認：
- 引擎已支援：Command::DiscussNew 收 slug: Option<String>（core/command/mod.rs:289）、DiscussDiscard { slug, force }（:309）；缺口在 CreateDiscussionRequest 只有 topic（protocol/command.rs:155）、server 硬編 slug: None（routes.rs:1119）、CLI bail（remote_commands.rs:1056）
- --slug 直通：protocol 加 slug: Option<String>（serde default 向後相容）→ server 轉傳引擎 → CLI 移除 bail
- remote discard 複製 change 側既有模式（remote_discard 直通 DELETE、--force 為 query 參數、引擎 guard 在 server 端自動生效）
- slugify 保留 CJK 的行為不動：技能已規定 agent 必帶英文 --slug，管線修通後 fallback 幾乎不會踩到
- 接縫深度檢查通過：DELETE /discussions/{slug} 背後藏引擎 discard guard（0 輪可刪、有輪需 force），非純轉發
**Ruled out**: 改 slugify 強制 ASCII——會波及本機模式既有行為，且要另裁音譯或拒絕，是另一場討論
**Open**: discuss link/seal 在 remote 也 bail（:1121、:1126），是否納入同批修；其他 remote 指令是否有同類缺口（使用者要求全面盤點）

### Round 2 — assumptions (2026-07-30)

**Focus**: remote 模式全指令 parity 盤點——逐一標記 29 個 cmd_ 函式的 remote 覆蓋
**Position**: 缺口分四類，A 類（靜默走錯 store）比 B 類（明確拒絕）更危險：
- A 類·靜默本機 fallback：cmd_show 無 remote 分支（commands.rs:425 直接 open_project 讀本機空 store，remote 模式下回「not found」或過期資料）；cmd_in_progress add（:2009 靜默寫本機 marker，team system 看不見）；cmd_demo（:2020 靜默在本機造 demo change，邊緣案例）
- B 類·明確 bail 的「yet」缺口：discuss new --slug（remote_commands.rs:1056）、discuss discard（:1117）、discuss link（:1122）、discuss seal（:1127）
- C 類·明確 bail 且屬設計決定，不是缺口：status/instructions --schema（server workflow config 決定 schema，:200、:346）、claim 在 fs 模式反向拒絕（commands.rs:51，正確的 fail-loud）、bulk archive（remote_commands.rs:1006，有逐一 archive 替代路徑）
- D 類·本質本機操作，無需 remote：init、update、completion、feedback、config（全域使用者設定）、schemas/templates（內建資產）
- remote show 可純用既有 client API 組出（get_change、get_artifact、spec_document、list_specs 都在 client.rs），不用新 server 端點
**Ruled out**: 把 C／D 類當缺口修——claim 的 fs 拒絕與 --schema 的 server 決定權是刻意設計；bulk archive 有替代路徑，補它是錦上添花
**Open**: change 範圍怎麼切——B 類全修＋A 類 show/in-progress 是否同一個 change；in-progress add 在 remote 的正解是接 API 還是 loud bail 導向 claim；demo 要 loud bail 還是放著

### Round 3 — assumptions (2026-07-30)

**Focus**: claim 與 in-progress add 是否語意重疊——remote 版 in-progress 該接 API 還是導向 claim
**Position**: 不重疊，「bail 導向 claim」出局，remote 正解是接 server API：
- claim（server routes.rs:973）＝ team-mode 最小認領回應：確認 change 存在後回 claimed_by，不寫任何持久狀態；註解明言「durable ownership arrives with the auth/admin knife」——是暫時性所有權概念
- in-progress add（core/inprogress.rs）＝ 生命週期驛站標記（created → started → archived 的 started 站）：把 started_at/by/with 蓋進 change 自己的 metadata 文件；首蓋不改、冪等、未知名稱靜默成功（exit 0 無輸出不寫檔，pre-migration frozen baseline）
- 讀者是看板：packages/ui/src/stage.ts:40「meta 含 started_at 或任務完成數 > 0 → 進行中欄」；呼叫者是 apply 技能（SKILL.md:42，開工時蓋章）
- remote 模式實際傷害：cmd_in_progress 走本機 store → 本機無該 change → 引擎「未知名稱靜默成功」→ apply 完全無感；started_by/started_with 歸屬永久遺失，「開工但尚無任務完成」期間看板欄位錯（task done 有 remote 支援，完成數>0 後才靠 fallback 條件移欄）
- 修法：POST /changes/{name}/in-progress 路由直通 verb::run（引擎 Command::InProgressAdd 已存在）＋ protocol DTO ＋ client 方法 ＋ CLI 分支；「未知名稱靜默成功」語意由引擎自然保留
**Ruled out**: loud bail 導向 claim——claim 不寫持久狀態、語意是所有權而非生命週期，導過去等於把開工標記丟掉
**Open**: 無——範圍已獲使用者買單（B 類全修＋show＋in-progress 接 API，demo 一行 bail）

### Round 4 — assumptions (2026-07-30)

**Focus**: remote 模式目前的生命週期設計長什麼樣——in-progress 接上去要對齊什麼
**Position**: 三站兩制：created／archived 兩站完整，started 站三層全斷；wire 另有閒置的 lifecycle 保留槽：
- created 站完整：POST /changes 走 verb::run → 引擎 NewChange，created_* 由 server 認證身分蓋章（verb.rs:42 binding.execution_context()，auth.rs:58）落 server store 的 meta 文件
- archived 站完整：POST /changes/{name}/archive 直通引擎
- started 站三層斷：(1) CLI 不路由，本機靜默 no-op，server meta 永遠沒有 started_*；(2) wire 的 ChangeSummary 沒有 startedAt 欄（query.rs:190-210），server 有章看板也讀不到；(3) 消費端已明文將就——desktop remote.rs:828「wire 無 startedAt 欄，開工判定退為完成數 > 0，兩側一致」，與前端 changeStage 對 remote payload 同構退化
- 平行保留槽：ChangeSummary/ChangeStatus/CreateChangeResponse/ClaimResponse 都有 lifecycle: Option<String>（fixture 例值 drafting/applying）與 claimed_by，server 一律回 None——為未來 server 端 lifecycle 狀態機保留，屬 auth/admin knife 時代
- 修法定為方案 A（對齊 fs 模式）：路由 InProgressAdd 到 server（歸屬自動落認證 actor）＋ ChangeSummary 補 startedAt 選填欄（serde default 向後相容）＋ 兩個消費端把退化補回真標記；「完成數 > 0」fallback 保留（本來就涵蓋手改 tasks.md 等繞過路徑）
**Ruled out**: 方案 B 啟用 lifecycle 保留槽的 server 狀態機——新狀態機新語意、與 meta started_* 平行兩制，且 durable ownership 明言等 auth/admin knife，超出 parity 範圍；只路由不上 wire——蓋了章看板仍看不到，分欄照樣錯，只剩歸屬記錄價值
**Open**: 無——in-progress 修法從「路由直通」擴為「路由＋wire 欄＋消費端」，結論待更新

### Round 5 — assumptions (2026-07-30)

**Focus**: 要不要趁這次一併「完善 remote 生命週期」（啟用 lifecycle 狀態機／durable claim）
**Position**: 不併入——方案 A 落地後 remote 生命週期在本專案的設計語意下已經是完整的，剩下的是另一個功能而非缺口：
- 本專案的階段是「派生」不是「儲存」：changeStage 由 meta started_at ＋ 任務完成數即時推導（spec「看板欄位由生命週期標記驅動」），沒有可漂移的狀態。方案 A 修通後 created/started/archived 三站在 remote 端到端可用、與 fs 模式同構、單一事實來源仍是 meta 文件——這就是「完善」
- lifecycle 保留槽＋durable claimed_by 是「協調／所有權」問題，不是生命週期問題：需要先裁狀態集、轉移規則、搶佔與釋放語意（認領者離開誰能解？）、權限——server 註解明言 blocked on auth/admin knife，該基建不存在
- 儲存式狀態機會製造第二事實來源：lifecycle 說 drafting 但任務全勾時聽誰的？需要 reconciliation 設計，且三個 store 後端（fs/sqlite/postgres）都要長儲存＋conformance 測試
- 風險等級不同：parity 是低風險管線接通（引擎全現成），狀態機是高設計密度新功能——不同風險等級不該同車
**Ruled out**: server 派生 stage 回填 lifecycle 欄位以收斂兩份消費端推導——fs 模式無 server，前端推導必須存在，server 版只會是第三份實作不是收斂
**Open**: 無——結論的 Deferred 已載明 lifecycle/claimed_by 留給 auth/admin knife 時代，維持不變

## Conclusion

**Decision**: 一個 change 修齊 remote 模式的 parity 缺口，範圍七項：(1) discuss new --slug 直通（CreateDiscussionRequest 加 slug: Option<String>，serde default 向後相容；server 轉傳引擎；CLI 移除 bail）(2) discuss discard（DELETE /discussions/{slug} 路由，--force 為 query 參數，複製 change 側 remote_discard 模式）(3) discuss link 與 (4) discuss seal（路由＋client＋CLI）(5) show 的 remote 版（純 CLI 端以既有讀 API 組裝：get_change/get_artifact/spec_document/list_specs，不動 server）(6) in-progress add 的 remote 版採方案 A——POST /changes/{name}/in-progress 直通 verb::run（引擎 Command::InProgressAdd 已存在，started_by 自動落 server 認證 actor）＋ ChangeSummary 補 startedAt 選填欄 ＋ 兩個消費端（desktop remote.rs change_stage、前端 changeStage 的 remote payload 路徑）把「完成數 > 0」退化補回真標記（該 fallback 保留，涵蓋繞過路徑）(7) demo 在 remote 模式補一行 loud bail。
**Rationale**: 引擎層對所有動詞早已完整支援，缺口全在 remote 傳輸鏈（protocol → server 路由 → remote client → CLI）——同族缺口同一條管線，分批修等於付多次 protocol 變更成本。A 類靜默缺口（show 讀本機空 store 回錯資料、in-progress add 靜默 exit 0 丟失開工歸屬且看板欄位錯）比 B 類明確 bail 更危險，必須同批修。remote 生命週期現況是「created／archived 完整、started 三層斷」，方案 A 讓三站判定與 fs 模式同構、單一事實來源仍是 meta 文件。
**Rejected alternatives**: 只修 discuss 四動詞——A 類靜默缺口更危險；in-progress bail 導向 claim——claim 是不落地的所有權回應，語意不重疊；in-progress 只路由不上 wire——看板仍看不到，分欄照樣錯；啟用 wire 的 lifecycle 保留槽做 server 狀態機——新語意平行兩制，durable ownership 明言等 auth/admin knife，超出 parity 範圍；slugify 強制 ASCII——波及本機模式，另場討論；修 bulk archive——有逐一替代路徑。
**Deferred**: wad-old-web 上那筆全中文 slug 的 0 輪空記錄——等 remote discard 落地後用 CLI 清（或先從 team system 後台手動移）；slugify 對非 ASCII topic 的長期政策——本次不動；lifecycle/claimed_by 保留槽的 server 狀態機——auth/admin knife 時代再議。
**Capture to**: proposal（經 propose 正式化）
**Next**: /speclink-propose --from-discussion remote-verb-parity

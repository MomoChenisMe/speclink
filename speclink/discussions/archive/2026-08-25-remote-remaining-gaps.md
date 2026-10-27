---
topic: 遠端剩餘小縫的盤點與處置（capability 清單、change 詮釋資料、promotedTo、離線與衝突）
slug: remote-remaining-gaps
status: promoted
promoted_to: remote-read-parity, remote-claim-ownership, remote-docs-refresh
created: 2026-08-25
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 遠端剩餘小縫的盤點與處置（capability 清單、change 詮釋資料、promotedTo、離線與衝突）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

承接 2026-08-23 remote-partial-gaps-priority 結論的第三步（B 剩餘小縫盤點）：前兩步（remote-task-evidence、文件漂移修正）已於 2026-08-25 封存落地，本討論逐項決定剩餘縫的處置。目標可驗證（每縫一個處置決定），無需 grill，直接假設清單。相關 specs：remote-workspace-data（capability 宣告與停用語意）、remote-resilience（離線紅線：無佇列）、server-verb-api（單一 change 讀取的組合欄位承諾）、client-protocol／server-read-api（DiscussionInfo DTO 與讀取面）、remote-board-order（現有唯一完整 CAS 409 語意）。無進行中 change。同日曾照 docs/remote-getting-started.zh-TW.md 實跑最短路徑全通（server／CLI／Desktop 三端 0.1.3），實跑未觸及本討論各縫。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-25)

**Focus**: 三個縫的實況盤點與處置假設是否成立
**Position**: 三個假設全數獲使用者確認——縫全在 server 端沒給資料，client 端已按「缺什麼停用什麼、不偽造」紀律處理：
- capability 清單與 change 詮釋資料：立案補 server 端點——remoteDataSource.ts:87-92 明寫 unsupported，App.tsx:721-731 已有 capability 分流；掛 server-verb-api「單一 change 讀取攜帶 show 組合欄位」的既有承諾上
- promotedTo 空清單：立案補 wire 欄位——protocol query.rs DiscussionInfo 無 promoted_to，remoteDataSource.ts:41-43 以空清單補；additive（struct 已有 skip_serializing_if 慣例）
- 離線與衝突：不立大案——remote-resilience 六條 requirement 已覆蓋離線面且紅線明訂無佇列；「衝突有得選」的實體是線上 client 同時寫的 CAS 409 呈現面（現僅 remote-board-order 有完整語意），先盤 409 面再決定
- 使用者新增需求：四縫補完後開一個 change 做 docs 全面總整理，避免縫閉合而文件反而落後
**Ruled out**: 離線佇列方向——與 remote-resilience 紅線相抵，使用者未要求離線可操作
**Open**: TeamStore 討論文件面是否已存 promoted_to（決定假設 2 工程量是補 DTO 還是補儲存契約）；三個立案縫怎麼分刀與排序；docs 總整理 change 的範圍與觸發時機；前議第 4 項 desktop claim／開工歸屬是否納入本輪（使用者「四個縫」的指涉待確認）

### Round 2 — interview (2026-08-25)

**Focus**: promotedTo 的儲存事實查證與三縫分刀方式
**Position**: promoted_to 有存、縫只在列表管線，三縫併一刀成立：
- 事實：promoted_to 存於討論文件 frontmatter（crates/speclink-core/src/discuss.rs:339-346，逗號累加），TeamStore 走同一份文件——不缺儲存契約
- 缺口鏈：引擎列表結構 DiscussionInfo（discuss.rs:16-30）未帶出 → wire DTO（protocol query.rs:547-559）沒有 → client 以空清單補；三層皆 additive
- 分刀提案：縫 1–3 併一刀（暫名 remote-read-parity，同屬 server 讀取面補齊、delta 撞同幾份 spec，分開會平行版號對撞）；縫 4（衝突呈現）先盤 CAS 409 面再決定；docs 總整理獨立收尾刀
**Ruled out**: promotedTo 走「補儲存契約」的大工程路線——frontmatter 已存，查證後不成立
**Open**: 分刀方案待使用者確認；「四個縫」是否含前議的 desktop claim／開工歸屬待確認

### Round 3 — interview (2026-08-25)

**Focus**: desktop claim／開工歸屬是什麼、要不要排進本波
**Position**: 使用者確認分刀方案，並把 claim／開工歸屬排進 docs 刀之前加一小刀：
- claim＝唯一 RemoteOnly 動詞（speclink claim <change>）：多人共用 remote store 時認領 change 防撞工；本機模式明確拒絕（docs/verb-contract.zh-TW.md:24）
- 開工歸屬＝started_at／started_by 蓋章（verb-contract.zh-TW.md:183，取認證身分），看板以此推「進行中」欄
- 縫的實體：apps/desktop/src 全域 grep claim 零命中——桌面遠端看板不能認領、卡片也看不到誰在做；對 Day 12–13 的團隊協作情境是核心畫面缺口
- 定序確認：縫 1–3 併刀 → 衝突呈現先盤 → claim／開工歸屬小刀 → docs 總整理收尾刀
**Open**: 結論前使用者要求以 subagent 對結論草稿做一輪反證驗證

### Round 4 — interview (2026-08-25)

**Focus**: subagent 反證驗證結論草稿（使用者要求的結論前驗證輪）
**Position**: 四步結構存活，但三處內容被證據翻修：
- 縫 1a 刀口轉向：server 端點與 spec 承諾早已落地——GET /changes/{name} 已組 deltaCapabilities 與部分 meta（routes.rs:171-240、query.rs:254-274 ChangeStatus），桌面 remote_status 已把整包 ChangeStatus 丟到 TS 手上；真正的工作是桌面翻 capability＋映射既有 payload（便宜），與 meta 完整 parity 要補 wire 4 欄位並延伸到清單端點 ChangeSummary（缺 createdBy/created/fromDiscussions，看板建立者 chip 因此殘缺）；remote.rs:1005-1007 與 App.tsx:721 註解已過期
- 縫 1b 修法形狀改判：promoted_to 刻意不進引擎 DiscussionInfo（discuss.rs:691-693 design D2，保 CLI JSON 逐位元不變），本地是 desktop-core 邊緣組裝；正確形狀＝server route 邊緣組裝＋wire 欄位＋client 映射，引擎不動
- 縫 4 盤點在驗證中直接完成：桌面 409 呈現不只 board-order——config/policy 寫入有對照對話框（ProjectSettingsView.tsx:412,455）、退回提案有守門對話框、刪除有 toast；殘餘僅任務勾選／搬移／archive 撞 bridge commit CAS 時走一般錯誤路徑，面比預想小很多；另發現 remote-resilience spec.md:164「deleteChange 於 remote 維持停用」已被程式碼推翻（remote 刪除 2026-07-31 起已實作），spec 過期句待修
- claim 刀地基是空的：server POST claim 是不落盤 stub（routes.rs:1361-1385，claimed_by 只回聲、清單恆 None），桌面 Rust 橋 RemoteWorkspace::claim 已在（remote.rs:1456-1462）但無 Tauri command 與 UI；這刀主體是 server durable claim 語意，桌面是尾巴，工程量比「小刀」大
- 1c spec 撞擊名單修正：server-read-api 不涉入，換成 reference-server；docs 刀範圍實測 21 份檔案（platform-architecture、server-deployment、sdk-node、configuration、getting-started、development、workflow 等都提 remote）
**Ruled out**: 「server 補 capability 清單端點」的原修法——端點已存在，縫在桌面與 meta 欄位面；「promoted_to 進引擎 DiscussionInfo」——違反 design D2
**Open**: 無——進入結論

## Conclusion

**Decision**: 遠端剩餘縫依驗證後內容分四步處置：
1. 刀 A（暫名 remote-read-parity）：遠端讀取對等——桌面翻開 change_meta/change_capabilities capability 並映射 remote_status 已送達的 ChangeStatus payload；wire 與 server 補 meta 4 欄位（createdBy/createdWith/startedAt/startedBy）至單 change 讀取回應，清單端點 ChangeSummary 補 createdBy/created/fromDiscussions（看板建立者 chip 對等）；promoted_to 以 server route 邊緣組裝＋wire 欄位＋client 映射補齊（引擎 DiscussionInfo 不動，遵守 design D2）；併修過期註解（remote.rs:1005-1007、App.tsx:721）與 remote-resilience spec.md:164 過期句（deleteChange 停用已被 2026-07-31 archive-readiness-gating 推翻）
2. 衝突呈現不立案：驗證盤點確認桌面 409 呈現已覆蓋 config/policy（對照對話框）、退回提案（守門對話框）、刪除（toast）、board-order（CAS 重試收斂）；殘餘僅任務勾選／搬移／archive 撞 bridge commit CAS 走一般錯誤路徑——有明確報錯、非靜默失敗，重試成本低，不值一刀
3. 刀 B（暫名 remote-claim-ownership）：主體是 server 的 durable claim 語意（現為不落盤 stub，claimed_by 不持久），桌面補 Tauri command 曝露＋capability 位＋看板認領操作與歸屬呈現（含 remote 端 startedBy）；排在 docs 刀之前，工程量按「server 語意＋桌面面」估，不是純 UI 小刀
4. 刀 C（暫名 remote-docs-refresh）：全部遠端相關文件總整理，實測 21 份（roadmap/product-status/remote-getting-started/verb-contract/platform-architecture/server-deployment/sdk-node/configuration/getting-started/development/workflow 等雙語系），含實跑發現的「第一位管理員也要自己加 membership」教學縫；等 A、B 落地後開，避免縫閉合而文件落後
**Rationale**: 驗證揭露縫的實體大多不在原以為的位置——server 讀取端點早已落地、縫在桌面停用與 meta 欄位面；promoted_to 的儲存與查詢函式都在、缺的是 route 邊緣組裝；claim 的地基（durable 語意）是空的。分刀依「同一組 spec 的 delta 併刀、地基先於 UI、文件收尾一次對齊」排列，避免平行版號對撞與文件落後。
**Rejected alternatives**: 離線佇列方向（違反 remote-resilience 無佇列紅線，使用者未要求離線可操作）；「server 補 capability 清單端點」（端點已存在，驗證推翻）；promoted_to 進引擎 DiscussionInfo（違反 design D2 的 CLI JSON 逐位元不變保證）；衝突呈現立案（殘餘面過小）；三縫各自分刀（delta 撞同組 spec，平行版號對撞）
**Deferred**: 封存討論 slug 重用時 promoted_to 查詢的歧義（本地與 remote 同病，非本輪新縫）；桌面 in-progress 標記操作面（桌面現無任何開工標記入口，僅反向 revert——留待刀 B 設計時決定是否納入）
**Capture to**: proposal（刀 A、B、C 依序立案）
**Next**: /speclink-propose --from-discussion remote-remaining-gaps（先立刀 A）

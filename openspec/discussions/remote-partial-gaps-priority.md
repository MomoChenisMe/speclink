---
topic: Desktop 遠端工作區與遠端 task evidence 兩個 Partial 缺口的優先序
slug: remote-partial-gaps-priority
status: open
created: 2026-08-23
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: Desktop 遠端工作區與遠端 task evidence 兩個 Partial 缺口的優先序

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

Day 12 遠端能力盤點（docs/product-status.zh-TW.md、remote-getting-started.zh-TW.md、roadmap.zh-TW.md）後，使用者問兩個 Partial 缺口——Desktop Remote Workspace 與 remote task evidence——是否優先處理、先後怎麼排。目標可驗證（排序決策），無需 grill，直接假設清單。相關 specs：verify-evidence（evidence 語意正典）、workspace-session（remote locator 模型）、remote-workspace-data、teamstore-contract。相關已封存 changes：remote-data-source（2026-07-19，c4a8ba0）、offline-stale-reauth 與 remote-workspace-recovery-ux（2026-07-21）。待開 change 名 remote-task-evidence 已由 crates/speclink-server/tests/it/phase2_chain.rs 的 #[ignore] 紅色測試預留。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-23)

**Focus**: 兩個 Partial 缺口（Desktop Remote Workspace、remote task evidence）要不要做、先後怎麼排
**Position**: 兩個都做，先 remote-task-evidence 後 Desktop 遠端工作區：
- 「要不要做」已由路線圖定調（roadmap 遠端協作線明列這兩件為僅剩缺口），真問題只有順序
- evidence 是「進行中的資料遺失」：CLI 已在 wire 上送 touchedFiles，server 端 routes::task_done 以 Json(_req) 丟棄（phase2_chain.rs:619-622），每次遠端 task done 證據永久消失、補不回來
- evidence 刀小且萬事俱備：change 名已預留（remote-task-evidence）、#[ignore] 紅測試等轉綠、verify-evidence spec 已定格式，範圍＝server route 收 payload＋TeamStore 落庫（三 driver）＋查詢面
- Desktop 遠端工作區估為至少三刀（spec-only／remote+checkout／offline 衝突），依據 product-status「remote locator 沒有建構路徑」
- 兩者無硬依賴；evidence 消費端為 drift、commit 歸屬與封存 trace 提示，review／verify 站不直接吃
**Open**: 使用者質疑假設 4 的前提——情境 2（remote+checkout）似乎已存在，待驗證

### Round 2 — interview (2026-08-23)

**Focus**: Desktop 情境 2（remote+checkout）是否其實已經存在
**Position**: 已存在——且情境 1 也有、情境 3 部分有；文件與程式碼矛盾，程式碼＋綠測試為準：
- remote-data-source（c4a8ba0，2026-07-19，在 HEAD 祖先鏈上）即已「enable Desktop remote workspaces」
- 建構路徑完整可達：WorkspaceChooser → openRemoteWorkspace → remote_open（fail-closed handshake）→ createRemoteSession（main.tsx:30-37、session.ts:378）
- chooser 有兩種模式：skip＝免 checkout 直接開（＝情境 1 spec-only）；folder＝bindCheckout 綁本機資料夾後開（＝情境 2 remote+checkout，WorkspaceChooser.tsx:412-436）
- workspace-session spec 本文寫「remote 變體經 chooser 或 remote marker 探測的 handshake 成功路徑建構」——spec 與程式碼一致，錯的是狀態文件
- remoteDataSource（165 行）覆蓋面廣：列表／文件／搜尋／任務勾選與排序／validate／analyze／archive／討論全套／卡片排序；僅 changeCapabilities 與 changeMeta 明寫不支援
- 情境 3 部分存在：offline-stale-reauth 的連線狀態機與 stale 唯讀快照、remote-workspace-recovery-ux 的復原頁（2026-07-21 已封存）
- remoteOpen ＋ workspaceChooser 前端測試 42 綠（2026-08-23 實跑）
**Ruled out**: 「Desktop 遠端工作區＝至少三刀新功能」的估算——依據的 product-status（2026-08-13 查核）「remote locator 沒有建構路徑」與 remote-getting-started §6「登入後開不出看板」為陳舊記載，與 session.ts 頂部過期註解（本刀僅型別宣告）同源
**Open**: 缺口 B 真正剩什麼（changeCapabilities／changeMeta 不支援、討論 promotedTo 以空清單補、offline 衝突處理完成度、desktop 端 claim／開工歸屬）；文件漂移（product-status 三處）是否立 change 修正；優先序是否因此改判

## Conclusion

<!-- Written by `speclink discuss conclude`:
**Decision** / **Rationale** / **Rejected alternatives** / **Deferred** / **Capture to** / **Next** -->

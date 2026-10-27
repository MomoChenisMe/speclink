## Why

一份討論的結論拆成多刀依序立案時（`improve-lifecycle-layer` 拆四刀），每封存一刀，討論記錄就被隨行封存，下一刀要 `--from-discussion` 時記錄已不在途，只能手動從 `openspec/discussions/archive/` 搬回（本系列被連帶封存三次、手動搬回兩次）。病因是 2026-09-07 落地的 `--hold` 旗標只涵蓋「先轉出刀 A 再 conclude --hold」的順序：多刀系列是「先 conclude --hold 再轉出刀一」，轉出動詞對新名字一律清旗標，旗標在轉出刀一時就消失；要補救得每刀封存前重跑 conclude --hold（順帶對在途刀蓋 restale 章、再 seal 清掉），實務每次都忘。同時 Desktop 看板把「已結論、帶 hold、還欠一刀」的討論收進欄底「已轉出」收合列，與「做完等封存」的討論長得一樣，使用者以為討論結束了——根因是 `hold` 沒上到 server 的討論列表資料鏈。

目標使用者：透過 AI 代理跑 SDD、把一份討論分多刀立案的開發者；對應 `/speclink-discuss` 收尾（conclude）、`/speclink-propose --from-discussion` 與 `/speclink-archive` 三個階段，以及 Desktop 看板與系統匣的討論分區。

來源討論：multi-cut-discussion-hold（結論已寫，本變更即其 Next）。

## What Changes

- **引擎（speclink-core）hold 語意改成「轉出不清 hold」**：`DiscussionHead::promote` 對新名字累加 promoted_to 時不再移除 `hold:` 行；hold 只由「不帶 --hold 的 conclude」或手動 `speclink discuss archive` 解除。三個轉出路徑（`discuss promote`、`new change --from-discussion`、`discuss seal`）共用同一支，改一處即全涵蓋。`close_if_finished` 的三條件（無在途引用、已有結論、無 hold）不動。**BREAKING（行為）**：帶 hold 的討論在最後一個轉出變更封存時不再自動隨行封存，改由使用者跑一次 `speclink discuss archive <slug>` 收尾；忘記的後果從「記錄被吞進 archive、要 git mv 搬回」變成「記錄留在途、看板看得到」。
- **技能文字同步（discuss、improve 兩份 asset；claude 與 codex 兩工具）**：中途轉出段的分期轉出教學縮成「多刀系列 conclude --hold 一次；hold 直到不帶 --hold 的 conclude 或手動 archive 才解除；最後一刀封存後跑 `speclink discuss archive <slug>` 收尾」，刪掉「旗標由下一次轉出清除」「discard 不還原旗標、要重跑 conclude --hold」兩句；並補一句救援路徑「討論被誤封存時，把檔案從 openspec/discussions/archive/ 搬回 openspec/discussions/ 即可續用（引擎錯誤訊息即指向此路）」。ASSET_VERSION、render golden、assets.lock 三連動。
- **schema 的 tasks 指引註明 design 標題可只寫編號（順手併入）**：內建 spec-driven schema 的 tasks instruction 有一句「every `###` heading from design.md should be referenced」，但 2026-09-09 analyze-rule-tuning 已讓 analyzer 接受只寫編號（D1／決策一／Decision 1）即算引用，指引沒跟上，代理人仍照抄整串標題。改為明說「引用本文或編號（如 D1）任一即可」。schema 內文不渲染進工作區、不進 render golden 與 assets.lock，只需一行文字與 schema.rs 的內容斷言測試。
- **資料鏈把 hold 送到前端（speclink-protocol、speclink-server、desktop core）**：protocol `DiscussionInfo` 比照 `concluded` 增選填 `hold` 欄位（camelCase、serde default、缺席即未知、序列化缺席省略鍵）；server 的 GET /discussions 於 route 邊緣自引擎 `DiscussionInfo.head.hold` 恆填 true／false。本地專案不經 server：Tauri 的 `list_discussions` 指令由 desktop core 自組同形 JSON，同樣比照 `concluded` 多填一鍵，否則本地看板與系統匣看不到 hold。引擎 `DiscussionInfo` 結構與 `discuss list --json` 輸出逐位元不變（hold 在 `serde(skip)` 的 head 內，本來就不進 JSON）。
- **看板分區規則（packages/ui）改成「引擎會自動收走的討論才收合」**：欄底「已轉出」收合列只收 promoted 且 concluded 為 true 且 hold 非 true 的討論；promoted 且已結論且 hold 為 true 的討論留上區全尺寸卡，帶「已轉出・保留中」狀態標（與「已轉出・尚無結論」並列）、無任何動詞按鈕；hold 缺席（舊 server）視同不保留、沿既有分區。KanbanBoard 拖排落點清單共用同一判準，跟著變。
- **系統匣分區規則（apps/desktop）同步**：tray 快照導出的 `promoted` 布林加上 hold 非 true 的條件——帶 hold 的討論列於「討論」分區而非「已轉出」分區；tray 討論列與原生選單只顯示 slug 與 topic，不加新標籤。TrayPanel 與原生選單都吃同一個布林，改一處即全涵蓋。

相容性影響：`speclink discuss conclude`、`promote`、`seal`、`archive` 的人眼與 `--json` 輸出不變（promote 從不報告 hold）；`speclink archive <change>` 對帶 hold 的討論本來就不列入隨行封存清單，輸出不變；只有「最後一刀封存後記錄是否自動移入 archive/」這個檔案系統結果改變。既有 CLI 整合測試「分期轉出的生命週期先保留後釋放」的期望值要改。GET /discussions 回應多一個 `hold` 鍵，舊 client 忽略未知鍵、不受影響；舊 server 不送 hold 時新 client 視為未知、沿既有分區。無設定欄位變動。

## Capabilities

### New Capabilities

（無。步驟 3 掃描命中 discussion-docs、discuss-skill、client-protocol、server-verb-api、desktop-app、tray-status-menu 六份既有規格，全部以 MODIFIED 承接，沒有新能力。）

### Modified Capabilities

- `discussion-docs`：「conclude 以 --hold 保留討論在途」——轉出動詞不再清旗標；scenario「轉出清除旗標」改為「轉出保留旗標」；Example「分期兩刀的生命週期」改為刀 b 封存後記錄仍在途、手動 archive 收尾。
- `discuss-skill`：「中途轉出教學」——分期轉出教學改為「conclude --hold 一次、最後一刀封存後 discuss archive 收尾」，刪 discard 重跑一句，補救援路徑一句；scenario「分期轉出帶 --hold」同步。
- `client-protocol`：新增「討論資訊 payload 增選填 hold 欄位」需求（比照 concluded）。
- `server-verb-api`：「討論列表回應攜帶 concluded」擴為同時攜帶 hold。
- `desktop-app`：「討論於看板第 0 欄兩級呈現」——分區判準加 hold；帶 hold 的已結論 promoted 討論留上區、帶「已轉出・保留中」狀態標；hold 缺席退回既有分區。
- `tray-status-menu`：「討論列表」——帶 hold 的已結論 promoted 討論列於「討論」分區；hold 缺席退回既有分區。

## Impact

- Affected specs: discussion-docs、discuss-skill、client-protocol、server-verb-api、desktop-app、tray-status-menu（皆 MODIFIED）
- Affected code:
  - New: （無）
  - Modified:
    - crates/speclink-core/src/discuss.rs（`DiscussionHead::promote` 不清 hold；`mark_promoted`／`promoted_text` doc 註解；單元測試改期望）
    - crates/speclink-core/src/init.rs（ASSET_VERSION bump）
    - crates/speclink-core/assets/skills/discuss.md、crates/speclink-core/assets/skills/improve.md（分期轉出教學）
    - crates/speclink-core/assets/schema/spec-driven/fork.schema.yaml（tasks instruction 的 Cross-referencing 一句）、crates/speclink-core/src/schema.rs（內容斷言測試）
    - crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md、crates/speclink-core/tests/golden/claude-worktree.snapshot.md（UPDATE_GOLDEN 再生）與 assets.lock（UPDATE_ASSETS_LOCK 再生）
    - crates/speclink-cli/tests/it/discuss_conclude_auto_archive.rs（分期生命週期測試改期望；新增「轉出保留旗標」測試）
    - crates/speclink-protocol/src/query.rs（`DiscussionInfo` 增 `hold`）
    - crates/speclink-server/src/routes.rs（`discussion_dtos` 回填 hold）、crates/speclink-server/tests/it/discussion_routes.rs
    - packages/ui/src/adapter.ts（`DiscussionItem` 增 `hold`）、packages/ui/src/components/DiscussionColumn.tsx（`isCollapsedPromoted` 與狀態標）、packages/ui/src/i18n.tsx（「已轉出・保留中」中英文案）、packages/ui/src/__tests__/discussionColumn.test.tsx、packages/ui/src/__tests__/kanban.test.tsx
    - apps/desktop/core/src/discussions.rs（本地列表 JSON 增 `hold` 鍵與對應測試）
    - apps/desktop/src/adapter/remoteDataSource.ts（wire 型別增 `hold`）、apps/desktop/src/tray.ts（`promoted` 導出加 hold 條件）、apps/desktop/src/__tests__/tray.test.ts、apps/desktop/src/__tests__/trayPanel.test.tsx
    - .claude/skills/ 與 .agents/skills/ 下由 `speclink update` 再生的 SKILL.md（隨版號再生，不逐檔列）
  - Removed: （無）
- 刻意避開：在途變更 `lifecycle-archive-gate-owner` 正在改 crates/speclink-core/src/archive.rs、crates/speclink-core/src/command/mod.rs、crates/speclink-cli/src/verbs/lifecycle.rs，本變更不碰這三檔；`close_if_finished` 已在 discuss.rs 內且不需改。

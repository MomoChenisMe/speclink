## Context

桌面 app 的變更清單由 apps/desktop/core/src/query.rs 的 `list_changes_at` 組裝：讀引擎的 list 項、疊上 startedAt／whyExcerpt／審查驗證狀態等呈現層欄位，順序由 `board_sorted_changes`（修改時間回退 → rank 覆蓋）決定；看板（packages/ui 的 KanbanBoard）與系統匣面板（TrayPanel）都只按階段過濾、不再排序，所以 payload 的順序就是畫面的順序。拖排由 packages/ui 的 boardDnd 純函式解析落點，桌面 core 的 `reorder_change` 補章後呼叫引擎 `set_board_rank`。第一刀已在引擎提供 `plan::compute`（波次、blockedBy、next）、`plan::check_rank_move`、`plan::set_depends` 與 `model::stage`。卡片行內小章家族（審查章、驗證章、待手動章）與 stage.ts 的單一判定入口是既有樣式與結構慣例。

## Goals / Non-Goals

**Goals:**

- 畫面順序 = 引擎順序：看板三欄與系統匣都照 plan 配置順序，不另建第二套排序。
- 一眼看出誰先做、誰可並行、誰被擋：波次章與阻擋章。
- 拖排尊重宣告依賴：不合法落點在拖動時就灰化，引擎再守一次。
- 使用者能在詳情面板查看與編輯前置。

**Non-Goals:**

- remote 模式的 plan 順序、波次章與前置編輯（第三刀）。
- 討論卡的排序規則不變。
- 不改引擎演算法、不改 CLI 輸出。
- 不做「拖排自動寫 depends_on」——拖排只改 rank。

## Decisions

### D1 清單 payload 改以 plan 順序並疊四欄位

`list_changes_at` 改為：組好 store（含 worktree overlay）後呼叫 `plan::compute(store)`；成功時以 `plan.changes` 的順序排列清單項，每項疊 `wave`、`blockedBy`、`dependsOn`、`overlaps`，頂層 `planError` 為 null；`skipped` 的壞 meta 項照舊列出（無四欄位，meta_error 已在）。`PlanError::Cycle` 時：順序退回 `board_sorted_changes`、所有項不帶 `wave`／`blockedBy`／`overlaps`、`dependsOn` 仍給 meta 的宣告原文（與 plan 列同一來源，排程分頁靠它解環）、頂層 `planError` 為 `dependency cycle: a -> b -> a` 字串。`board_sorted_changes` 本身改為與 plan 同一組鍵的基底序（具 rank 在前依字典序、缺 rank 在後依 created 升冪、缺 created 殿後、名稱決斷），不再讀修改時間——規格的「plan 成環時退回此基底序」與這裡是同一件事。順序與 plan 結果由 query.rs 的 `board_order(store)` 一個入口給出：清單 payload 與拖排補章都走它，畫面順序與補章派發序不會分家。看板與系統匣只按階段過濾，順序自然跟著 payload。CLI 的 `changes_json` 不動（parity 紅線）。

替代案：另開 `plan` command 由前端合併——兩次 IO、兩份順序要對齊，被否決。

### D2 排序真相改為 plan 配置順序，rank 只是輸入

board-card-order 的 local 語意改為：欄內顯示序 = plan 配置順序在該欄的投影。plan 的基底順序已把「有 rank 者在前依字典序、缺 rank 者在後依 created 升冪、名稱決斷」定為全序，所以拖排寫 rank 仍能改變順序，只是會被宣告依賴修正。整欄補章依當前顯示序派發，而顯示序已滿足依賴，補章後再套移動；觸發條件比原本多一項——欄內 rank 序與顯示序不一致（排程分頁加了前置、或 CLI 宣告依賴後 rank 沒動而顯示序被依賴修正）時也整欄重派，否則反序的兩鄰居之間沒有可用的中點鍵，拖排會靜默不動（審查 Round 1）。全員具 rank 且序一致時仍只改被拖卡一檔。缺 rank 置後（原置欄頂）與回退 created 升冪（原修改時間）是刻意變更，理由是「新工作排隊尾、不插到使用者排好的卡前面」。remote 模式的順序真相與回退維持現況。

### D3 拖排前後兩層守門

- 前端層（packages/ui/src/boardDnd.ts 新增 `invalidDropTargets(column, deps, activeId)` 純函式）：拖動卡 X 時，同欄中「X 的 dependsOn 成員」及其上方所有位置、「dependsOn 含 X 的卡」及其下方所有位置為不合法落點；KanbanBoard 把不合法目標卡加 `data-drop-invalid` 與降透明度樣式，`onDragEnd` 命中不合法目標時不呼叫 `onReorder`。餵入純函式的欄是同欄全序（自全清單派生、不經搜尋過濾，與 `readyIds` 同款）——被隱藏的前置仍決定其上方位置不合法，否則過濾中拖排會漏灰化、放開後才被引擎退回。只用宣告依賴，不用 overlaps（夥伴可互換）。
- 引擎層（桌面 core `reorder_change`）：在補章之後、寫被拖卡的 rank 之前，以算好的新鍵呼叫 `plan::check_rank_move(store, id, &key)`；被拒即回 `Err(顯示文字)`，走既有的「寫回失敗單行錯誤＋刷新回磁碟現況」路徑。補章寫入已完成不回滾——補章只是把當前顯示序固化，本身不違反依賴，屬可接受的半套。

### D4 波次章進進度列，被擋整卡變淡

第一版把波次章與「等 N 項」灰章都放在標題列，實測（2026-09-16 安裝版）標題列被頭像、審查／驗證章、討論泡擠到名稱只剩十來個字，而「被擋」只靠一顆小灰章，看不出「哪些現在能做、哪些要等」。改為：ChangeCard 在進度列（進度條＋任務數那一列）最左渲染圓圈波次章（主色調），tooltip 無阻擋時「第 N 波」、有阻擋時「第 N 波 · 等待：a、b」；`blockedBy` 非空時整張 Card 加 `data-blocked="true"` 與 `opacity-60`——與 TrayPanel 被擋列同一種視覺語言（淡＝現在不能做），不再有「等 N 項」文字章，標題列不因排程資訊增加任何元素。判定入口收斂於 stage.ts 的 `planWave(c)` 與 `planBlockedBy(c)`：欄位缺席（remote 或 planError）一律回 null／空陣列，卡片只讀結果；文字組裝也收在 stage.ts 的 `planWaveLabel(wave, t)`／`planBlockedLabel(blockedBy, t)`，卡片、系統匣面板、排程分頁共用一處。i18n 詞條：`card.wave`「第 {n} 波」、`card.blockedTitle`「等待：{names}」、`common.listSeparator`（tw「、」／en「, 」），tooltip 以「 · 」串接兩段；`card.blocked`「等 {n} 項」不再需要、移除。拖曳期間的兩種變淡（被拖卡 0.35、不合法落點 0.4）掛在 sortable 外層，與被擋的 0.6 相乘即可，不另做互斥。

### D5 依賴成環的看板提示

`planError` 非 null 時，KanbanBoard 於既有的 `reorderUnavailableReason` 提示列同位置顯示一行 `t("board.planCycle")` 加訊息原文（tw「依賴成環：」前綴）；卡片無波次章；拖排照常（引擎的 check_rank_move 在成環時放行，因為無法判定順序）。宿主（App.tsx）把 payload 的 `planError` 傳給看板。

### D6 詳情抽屜的排程分頁

RichDetailDrawer 在「規格」分頁之後加「排程」分頁（`value="plan"`，標籤 i18n `drawer.tab.plan`），內容四段：波次（「第 N 波」＋同波其他 change 名稱清單，無則「本波只有這個變更」）、前置（`dependsOn` 每項一列附移除鈕；底部一個下拉選單列出其他作用中 change 供新增；選擇即寫入）、重疊（`overlaps` 每項顯示 change 名與 capability 清單，唯讀）、阻擋（`blockedBy` 清單，空則「可以開工」）。編輯控制項只在資料源 capability `setDepends` 為 true 時渲染；寫入經 store 的 `setDepends(change, on, remove)` → 資料源 → Tauri `set_change_depends` → 桌面 core `manage::set_depends_at` → 引擎 `plan::set_depends`；成功後刷新，失敗以單行錯誤呈現、畫面不變（不刷新）。`set_depends_at` 定根到該 change 的所在（有 worktree 映射寫其副本），但引擎的成環守門只看那份 store；分頁的候選來自看板同源視圖（overlay），分支後主 checkout 才宣告的反向邊副本看不到——所以新增後以 overlay 再跑一次 `plan::compute`，成環就撤銷剛加的邊、回引擎的成環訊息（審查 Round 1）。分頁在 `wave` 缺席時分兩種：remote 摘要（亦缺 `dependsOn`）只顯示單句說明；plan 成環（payload 仍帶 `dependsOn`）顯示成環說明並只渲染前置段（可移除、不長新增下拉），讓使用者解環。

### D7 系統匣的波次標示與順序同源

TrayPanel 的變更列在名稱之前加波次數字（與卡片同一 i18n 詞條的 tooltip），`blockedBy` 非空的列整列降透明度、tooltip 列出前置；hover 反白時數字隨列改前景色。原生選單（tray.ts 的 buildTrayModel）在列標籤前綴「N· 」（無 wave 時不加前綴）；原生選單無法變淡，不做阻擋標示。兩者的順序來自同一份 payload，不排序。

### D8 桌面 core 的階段判定改用引擎

manage.rs 的 `change_stage` 改為呼叫 `speclink_core::model::stage` 並映射到既有的 u8 欄序，刪除本地規則副本。

## Implementation Contract

**行為**

- 看板三欄的卡片順序等於 `plan.changes` 投影到各欄；拖排寫回後刷新仍照此順序。
- 卡片：有 `wave` 在進度列最左顯示圓圈數字章，tooltip「第 N 波」或「第 N 波 · 等待：a、b」；`blockedBy` 非空整卡 `data-blocked="true"` 並降透明度，標題列不增加任何章；remote 或 planError 時波次章缺席、不變淡且卡片其餘呈現與本變更前一致。
- 拖動時不合法落點卡片降透明度並帶 `data-drop-invalid="true"`；放下於不合法落點不觸發寫回；引擎拒絕時顯示單行錯誤並刷新。
- planError 非 null：看板頂部一行提示，卡片無波次章，拖排可用。
- 排程分頁：四段內容如 D6；新增或移除前置後 `.openspec.yaml` 的 `depends_on` 隨之變動並刷新看板順序；成環時只剩前置段（可移除）。
- 拖排：欄內 rank 序與顯示序不一致時先依顯示序整欄重派再套移動；拖動中的不合法落點以同欄全序判定（搜尋過濾不影響）。
- 系統匣面板：列首波次數字、被擋列變淡；原生選單標籤前綴「N· 」。

**介面／資料形狀**

- `list_changes` payload：`{ changes: [...], planError: string | null }`，每項新增 `wave?: number`、`blockedBy?: string[]`、`dependsOn?: string[]`、`overlaps?: { change: string; capabilities: string[] }[]`。
- `ChangeItem`（packages/ui adapter）加上述四個選填欄位；`SpeclinkDataSource` 加 `setDepends(change: string, on: string[], remove: boolean): Promise<void>`；capability 旗標 `setDepends`（local true、remote false）。
- 頂層 `planError` 的前端通道：`SpeclinkDataSource` 加選配 `listChangesWithPlan?(): Promise<ChangeListPayload>`（`{ changes, planError }`，與 `listChanges` 同一個 `list_changes` 請求）；store 的 refresh 有此方法就用它、沒有（remote 第三刀前）退回 `listChanges` 且 `planError` 為 null——`listChanges(): Promise<ChangeItem[]>` 的簽名不動，既有呼叫端零改。
- Tauri command `set_change_depends(root, change, on: Vec<String>, remove: bool) -> Result<(), String>`；桌面 core `manage::set_depends_at(root, change, on, remove) -> Result<(), String>`。
- boardDnd：`invalidDropTargets(column: ColumnCards, deps: Map<string, string[]>, activeId: string): Set<string>`。
- stage.ts：`planWave(c: ChangeItem): number | null`、`planBlockedBy(c: ChangeItem): string[]`。

**失敗模式**

- 引擎 plan 成環 → payload 帶 planError、順序退回、無 wave／blockedBy／overlaps（dependsOn 保留）；不是錯誤回應。
- rank 移動被引擎拒絕 → 單行錯誤（引擎 Display 文字）＋刷新；補章寫入保留。
- setDepends 守門失敗（自依賴、不存在、已封存、成環）→ 單行錯誤文字來自引擎，畫面不變。
- setDepends 寫入後 overlay 複檢成環（worktree 映射下的盲區）→ 撤銷剛加的邊、單行成環錯誤，看板不出 planError。

**驗收**

- 桌面 core 測試（`cargo test -p speclink-desktop-core`）：list payload 順序 = plan 順序且四欄位存在；成環時 planError、退回順序與 dependsOn 保留；CLI `changes_json` 同 store 下無四欄；reorder 違反依賴被拒且被拖卡 rank 不變；rank 序與顯示序不一致時整欄重派；set_depends_at 寫入與守門、worktree 映射下的 overlay 成環複檢；change_stage 與引擎 stage 一致。
- packages/ui 測試（`npm test -w packages/ui`）：波次章浮現於進度列與缺席、被擋整卡變淡與 tooltip 合併前置、invalidDropTargets 純函式、KanbanBoard 不合法目標的 data 屬性與不觸發 onReorder（含搜尋過濾下的同欄全序）、planError 提示、排程分頁四段與編輯控制項的 capability 開關、成環時的前置段。
- apps/desktop 測試（`npm test -w apps/desktop`）：TrayPanel 列首數字與變淡、tray.ts 標籤前綴、資料源 setDepends 委派與 remote 不可用。

**範圍**

- In：上列 apps/desktop 與 packages/ui 檔案。
- Out：crates/ 下所有 crate；remote 資料源的排序、章與編輯；討論卡。

## Risks / Trade-offs

- [看板顯示序刻意變更（缺 rank 置後、回退 created）] → 規格 MODIFIED 明載；kanban 既有測試中依賴舊回退序的案例同批改寫。
- [補章後才被拒，留下補章寫入] → D3 記載為可接受的半套：補章只固化當前顯示序，不違反依賴。
- [前端與引擎兩層規則漂移] → 前端只灰化宣告依賴、引擎同樣只擋宣告依賴；兩層測試各用同一組夾具（c depends a）。
- [Windows／Linux 原生選單無法變淡] → 只前綴數字，規格明載。
- [worktree overlay 下 plan 的階段判定] → list_changes_at 已用 overlay 後的 store 呼叫 compute，與 CLI plan 同一聚合面。

## Migration Plan

無資料遷移。本變更依賴 add-change-plan-engine 先落地；回滾即還原 payload 排序為 `board_sorted_changes` 並移除四欄位，前端因欄位缺席自動不顯示章。

## Open Questions

無。

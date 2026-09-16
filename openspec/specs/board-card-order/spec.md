# board-card-order Specification

## Purpose

看板卡片順序的持久化語意：順序以 change meta 的 board_rank 欄位為唯一真相、欄內拖排以中點 rank 只寫回單一檔案、欄內出現缺 rank 的卡時整欄補章，且跨欄拖曳只改順序不改變更階段。本 capability 保證 board_rank 不外洩到 CLI 輸出（既有輸出逐位元不變）、與其他 meta 寫入路徑互不破壞，壞 metadata 的變更不參與排序寫入。

## Requirements

### Requirement: 看板卡片順序以 board_rank 欄位為真相

本地 session 的看板欄內顯示順序 SHALL 為引擎 plan 配置順序（見 change-plan 能力）投影到該欄的結果；plan 的輸入之一是卡片自身 meta 的 `board_rank` 欄位：變更卡讀自 change 目錄的 .openspec.yaml、討論卡讀自 openspec/discussions/<slug>.md 的 frontmatter（討論卡不參與 plan，維持 rank 升冪、缺 rank 置頂依 slug 序）。remote session 的順序真相 SHALL 為 scope 的 board resource 文件，SHALL NOT 讀寫卡片 meta 的 `board_rank`；remote 的排序語意（含回退序與是否套用 plan）SHALL 依 remote-board-order 能力的規定，本需求不另行規定。rank 值 SHALL 為小寫英文字母組成的字串，以位元組字典序升冪排列。本地變更卡的同欄基底序 SHALL 為：具 rank 的卡在前依字典序升冪，缺 rank 的卡在後依 created 升冪、缺 created 者殿後；同值以變更名字典序決斷；此基底序再經宣告依賴修正（宣告前置一律在其依賴者之前）。repo 內所有變更卡皆缺 rank 時，欄內顯示 SHALL 為 created 升冪（刻意變更：原為修改時間序），plan 成環時退回此基底序。

#### Scenario: 依 rank 升冪且缺值置後

- **WHEN** 同一欄內存在具 rank 與缺 rank 的變更卡
- **THEN** 具值卡依 rank 字典序升冪排在前，其後接缺值卡依 created 升冪

##### Example: 四卡混排

- **GIVEN** 同欄四卡：W（rank: b）、X（rank: f）、Y（rank: n）、Z（無 rank）
- **WHEN** 看板渲染該欄
- **THEN** 顯示順序為 W、X、Y、Z

##### Example: 同值以名稱決斷

- **GIVEN** 同欄兩卡 beta 與 alpha 的 rank 皆為 n
- **WHEN** 看板渲染該欄
- **THEN** alpha 排在 beta 之前（名稱字典序），且兩台機器上的順序相同

##### Example: 宣告依賴修正基底序

- **GIVEN** 同欄兩卡 c（rank: b）與 a（rank: f），c 的 depends_on 含 a
- **WHEN** 看板渲染該欄
- **THEN** 顯示順序為 a、c（依賴修正蓋過 rank 序）

#### Scenario: 新建的卡落欄底

- **WHEN** 使用者新建一個變更（無 rank）且看板刷新
- **THEN** 該卡顯示於所屬欄缺 rank 群組的末端（同欄全員缺 rank 時即欄底）

#### Scenario: remote 拖排不寫卡片 meta

- **WHEN** editor 於 remote 分頁拖排一張變更卡
- **THEN** 順序變動記錄於 board resource，該變更的 .openspec.yaml 內容不變


<!-- @trace
source: add-change-plan-desktop
updated: 2026-09-16T14:57:15+08:00
-->

---
### Requirement: 欄內拖排以中點 rank 單檔寫回

使用者於看板同一欄內拖動卡片到新位置放開時，系統 SHALL 計算落點前後鄰居 `board_rank` 的字典序中點作為被拖卡的新 `board_rank` 並寫回其 meta 檔；落點為欄頂或欄底時 SHALL 以單側鄰居推導嚴格較小或較大的鍵。中點鍵 SHALL 嚴格介於兩鄰居之間；兩鄰居間無可用縫隙時 SHALL 延長鍵長產生新鍵，SHALL NOT 改寫鄰居的 `board_rank`。變更卡寫回前 SHALL 以新鍵執行引擎的 rank 移動依賴檢查（見 change-plan 能力）：新位置會排在任一宣告前置之前、或任一宣告依賴它的變更之後時，SHALL 拒絕寫回被拖卡的 rank、以單行錯誤訊息呈現並刷新回磁碟現況；僅因 delta 重疊而依序的夥伴 SHALL NOT 構成拒絕。目標欄內全員具 `board_rank` 且 rank 序與顯示序一致時，一次拖排 SHALL 只修改被拖卡的一個檔案，且該檔除 `board_rank` 一行外其餘內容 SHALL 逐位元組不變；欄內 rank 序與顯示序不一致時（宣告依賴修正過基底序，例如加了前置但 rank 未動），SHALL 先依顯示序整欄重派 rank 再套用移動——與缺 rank 時的整欄補章同一路徑，否則反序的兩鄰居之間沒有可用的中點鍵。寫回完成後看板 SHALL 刷新至磁碟現況；寫回失敗（如檔案不可寫）時 SHALL 以單行錯誤訊息呈現並刷新回磁碟現況，SHALL NOT 保留未落檔的假象順序。看板搜尋過濾中拖排 SHALL 沿同一語意：新鍵介於可見前後鄰居之間，被過濾隱藏的卡與其相對序不受本次寫回影響。

#### Scenario: 穩態拖排只改一檔

- **WHEN** 使用者於全員具 `board_rank` 且 rank 序與顯示序一致的欄內拖動一張卡到兩鄰居之間放開
- **THEN** 只有被拖卡的 meta 檔被修改（變更卡＝該 change 的 .openspec.yaml、討論卡＝該討論的 .md），diff 僅含 `board_rank` 一行的增改，重啟 app 後順序不變

##### Example: 中點與延長

| 前鄰居 rank | 後鄰居 rank | 新 rank 性質 | Notes |
| ----------- | ----------- | ------------ | ----- |
| b | f | 嚴格介於 b 與 f（如 d） | 有縫隙取中點 |
| ab | ac | 以 ab 為前綴延長（如 abn） | 無縫隙延長鍵長 |
| （欄頂） | b | 嚴格小於 b（如 an） | 單側推導 |
| n | （欄底） | 嚴格大於 n（如 t） | 單側推導 |

#### Scenario: 寫回失敗不留假象

- **WHEN** 拖排寫回因檔案不可寫而失敗
- **THEN** 錯誤以單行訊息呈現，看板刷新後顯示磁碟上的實際順序

#### Scenario: 鄰居於寫回前消失

- **WHEN** 拖排落點的鄰居卡在寫回前被封存或刪除
- **THEN** 系統以現存鄰居重新推導新鍵完成寫回或將卡置於欄頂／欄底，不損壞任何 meta 檔、不崩潰

#### Scenario: 依賴修正後的欄拖排

- **WHEN** c 的 depends_on 含 a、c 的 rank 小於 a 的 rank（顯示序 a、c），使用者把 x 拖到 a 與 c 之間放開
- **THEN** 整欄先依顯示序重派 rank，x 的新鍵介於 a 與 c 之間，刷新後顯示 a、x、c

#### Scenario: 跨越宣告前置被拒

- **WHEN** c 的 depends_on 含 a、兩者同欄，使用者把 c 拖到 a 之前放開
- **THEN** 被拖卡 c 的 meta 檔 `board_rank` 不變，畫面顯示單行錯誤並刷新回 a 在 c 前的順序


<!-- @trace
source: add-change-plan-desktop
updated: 2026-09-16T14:57:15+08:00
-->

---
### Requirement: 欄內存在缺 rank 卡時整欄補章

拖排落點所在欄內存在缺 `board_rank` 的卡時，該次寫回 SHALL 先依當前顯示序對該欄全部卡片派發 `board_rank`（鍵列嚴格遞增且兩兩留有可再分縫隙），再套用本次移動。補章 SHALL 只涵蓋該欄，SHALL NOT 波及其他欄的卡片。

#### Scenario: 首次拖排補章整欄

- **WHEN** 使用者首次於某欄拖排（該欄全員缺 `board_rank`）
- **THEN** 該欄每張卡的 meta 檔皆寫入 `board_rank`，欄序等於拖放後的視覺序，其他欄的檔案不變

##### Example: 三卡補章後移動

- **GIVEN** 某欄依顯示序有 A、B、C 三卡皆缺 `board_rank`
- **WHEN** 使用者把 C 拖到 A 之前放開
- **THEN** 三卡皆獲得 `board_rank` 且字典序滿足 C < A < B，任兩鍵之間仍可取中點插入新卡


<!-- @trace
source: desktop-card-reorder
updated: 2026-07-08
code:
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/rank.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/store.ts
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/inprogress.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/util.rs
  - packages/ui/src/__tests__/discussionColumn.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/boardDnd.ts
  - packages/ui/src/components/DiscussionColumn.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/i18n.tsx
  - packages/ui/src/index.ts
-->

---
### Requirement: 跨欄拖曳不改變變更階段

變更卡的所屬欄(提案中/進行中/已就緒)SHALL 維持由任務完成度推導;把變更卡拖到另一個階段欄放開時 SHALL 彈回原位,SHALL NOT 寫入任何檔案。封存落點 SHALL 僅於拖曳已就緒變更卡時提供(浮現條件由 desktop-app 規格「拖曳封存落點以浮層呈現」所定);已就緒變更卡拖至封存落點放開時 SHALL 走既有封存確認流程;非已就緒變更卡的拖曳 SHALL 僅得欄內排序,放開於任何欄外位置 SHALL 彈回且零寫入。討論卡 SHALL 僅於討論欄內可拖排。位移未達拖曳啟動門檻的按放 SHALL 視為點擊並開啟卡片詳情,SHALL NOT 觸發拖排。

#### Scenario: 跨欄放開彈回且零寫入

- **WHEN** 使用者把提案中欄的變更卡拖到已就緒欄內放開
- **THEN** 卡片回到提案中欄原位,git 工作樹無任何檔案變更

#### Scenario: 已就緒卡的封存落點行為保留

- **WHEN** 使用者拖曳已就緒變更卡到拖曳中浮現的封存落點放開
- **THEN** 既有封存確認流程啟動,與本變更前行為一致

#### Scenario: 非已就緒卡拖曳僅得排序

- **WHEN** 使用者拖曳進行中欄的變更卡(封存落點未浮現)並於原欄外放開
- **THEN** 卡片彈回原位,無封存確認、無任何檔案寫入;於原欄內其他卡片上放開則僅觸發欄內排序寫回

#### Scenario: 單擊仍開啟詳情

- **WHEN** 使用者在卡片上按下並於拖曳啟動門檻內放開
- **THEN** 卡片詳情開啟,無拖排發生、無檔案寫入


<!-- @trace
source: archive-readiness-gating
updated: 2026-07-31
code:
  - apps/desktop/core/src/manage.rs
  - apps/desktop/src-tauri/tests/remote_data.rs
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/helpers/remoteFixtures.ts
  - apps/desktop/src/__tests__/remoteDataSource.test.ts
  - apps/desktop/src/__tests__/remoteResilience.test.tsx
  - apps/desktop/src/adapter/remoteDataSource.ts
  - crates/speclink-cli/tests/archive_readiness_gate.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/__tests__/richDrawer.test.tsx
  - packages/ui/src/boardDnd.ts
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/i18n.tsx
-->

---
### Requirement: board_rank 不進 CLI 輸出且既有輸出逐位元不變

`board_rank` SHALL 為桌面看板專用欄位：speclink list --json、speclink discuss list --json 及對應人眼輸出 SHALL NOT 出現 rank 相關欄位，且對含 `board_rank` 的 repo，上述輸出 SHALL 與同一 repo 移除全部 `board_rank` 欄位後的輸出逐位元一致（項目順序與欄位皆不變）。本需求為輸出凍結敏感：既有輸出基線 SHALL 維持位元級一致。

#### Scenario: 含 rank 的 repo 之 CLI 輸出不變

- **WHEN** repo 內數個 change 的 .openspec.yaml 與討論 frontmatter 含 `board_rank`，執行 speclink list --json 與 speclink discuss list --json
- **THEN** 兩者輸出與移除全部 `board_rank` 後執行的輸出逐位元相同，payload 不含 rank 欄位


<!-- @trace
source: spectra-legacy-cleanup
updated: 2026-07-27
code:
  - README.en.md
  - README.md
  - apps/desktop/src/App.tsx
  - apps/desktop/src/components/ProjectTabs.tsx
  - apps/desktop/src/index.css
  - crates/speclink-cli/src/color.rs
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/tests/discuss_promote_snapshot.rs
  - crates/speclink-cli/tests/task_done_stamps.rs
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/src/analyzer.rs
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/demo.rs
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/drift.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/newcmd.rs
  - crates/speclink-core/src/preflight.rs
  - crates/speclink-core/src/schema.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/src/status.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/validate.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-host/src/context.rs
  - docs/platform-architecture.zh-TW.md
  - packages/ui/src/__tests__/delta.test.ts
  - packages/ui/src/__tests__/taskList.test.tsx
  - packages/ui/src/components/ChangeList.tsx
  - packages/ui/src/components/DeltaBadges.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/delta.ts
  - packages/ui/src/index.ts
  - packages/ui/src/theme.css
-->

---
### Requirement: meta 寫入路徑對 board_rank 互不破壞

拖排寫回 `board_rank` 時，該 meta 檔的既有欄位（schema、created_*、started_*、from_discussion 等）SHALL 逐位元組保留。反向亦然：既有 meta 寫入路徑（開工標記、轉為變更等）作用於含 `board_rank` 的檔案時 SHALL 原樣保留該欄位。speclink-core 讀取含 `board_rank` 的 meta 或 frontmatter SHALL NOT 失敗。

#### Scenario: 拖排保留既有欄位

- **WHEN** 對含 created_*、started_* 欄位的變更卡拖排寫回 `board_rank`
- **THEN** 該 .openspec.yaml 除 `board_rank` 外逐位元組不變

#### Scenario: 開工標記保留 rank

- **WHEN** 對 meta 已含 `board_rank` 的 change 執行開工標記
- **THEN** 寫回後 `board_rank` 值原樣保留，開工欄位如常寫入

<!-- @trace
source: desktop-card-reorder
updated: 2026-07-08
code:
  - apps/desktop/core/src/discussions.rs
  - apps/desktop/core/src/lib.rs
  - apps/desktop/core/src/manage.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/rank.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/App.test.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/store.ts
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/inprogress.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/util.rs
  - packages/ui/src/__tests__/discussionColumn.test.tsx
  - packages/ui/src/__tests__/kanban.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/boardDnd.ts
  - packages/ui/src/components/DiscussionColumn.tsx
  - packages/ui/src/components/KanbanBoard.tsx
  - packages/ui/src/i18n.tsx
  - packages/ui/src/index.ts
-->

---
### Requirement: 壞 metadata 不參與看板排序寫入

`.openspec.yaml` 存在但 YAML 解析失敗的 change：排序寫入（桌面拖排經引擎的 board_rank 寫入）SHALL 在文字手術前拒絕且 SHALL NOT 寫檔；欄內補章 SHALL NOT 將該 change 視為缺 rank 卡——補章 SHALL 僅對 metadata 有效的缺 rank 卡寫入，metadata 損壞的卡片 SHALL 照其階段顯示並帶 invalid 標記，且 SHALL NOT 因補章被寫入。單一損壞卡 SHALL NOT 使整欄補章或看板清單中止。

#### Scenario: 排序寫入對壞 metadata 拒絕

- **WHEN** 對壞 metadata 的 change 執行 board_rank 排序寫入
- **THEN** 回帶檔案位置與解析原因的錯誤；該 `.openspec.yaml` 逐位元不變

#### Scenario: 補章排除 invalid 卡且不中止

- **WHEN** 同一欄內同時存在缺 rank 的有效卡與 metadata 損壞卡，觸發整欄補章
- **THEN** 僅有效卡被寫入 board_rank；損壞卡的 `.openspec.yaml` 逐位元不變；其餘卡片補章照常完成

#### Scenario: 看板照常開啟並標記損壞卡

- **WHEN** 桌面看板載入含 metadata 損壞 change 的 workspace
- **THEN** 看板照常開啟且列出全部卡片；損壞卡帶 invalid 標記；對其發起的變更操作被引擎錯誤拒絕
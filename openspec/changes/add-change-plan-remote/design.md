## Context

remote 路徑的既有形狀：server 以 axum 提供動詞端點，寫入動詞（如 POST /changes/{name}/in-progress）經 `verb::run` 走 Command gateway 直通引擎命令，bridge 於 outcome 帶事件時 commit 並發布 invalidate；唯讀衍生查詢（validate、analyze）同樣經 gateway 但零寫入。protocol crate 的 Rust 型別是 wire 正典（camelCase serde），錯誤回應的 reason 屬封閉 registry（八值，細分情形放在 message），remote client 是型別化呼叫並把 registry 以外的 reason 翻成通用錯誤句；speclink-remote 的 convert 模組把回應轉回引擎型別讓 CLI 兩模式共用渲染。桌面 remote 由 src-tauri 的 remote.rs 組裝：list 先取 server 清單再以 board resource 疊 rank 順序；拖排以 `reorder_via` 讀快照、改全文、CAS PUT。第一刀提供 `plan::compute_with_ranks`、`plan::violations`、`plan::set_depends`、`plan::RankMoveBlocked` 與 `ChangeDependsChanged` 事件（host 的事件紀錄映射已含此事件）；第二刀提供前端的四欄位、章、灰化、排程分頁、`setDepends` 與 `listChangesWithPlan` 資料源介面，以及 local 拖排「rank 序與顯示序不一致時整欄重派」的規則。

## Goals / Non-Goals

**Goals:**

- remote 與 local 讀同一份計算：server 的 plan 端點呼叫同一個引擎函式。
- CLI 兩模式輸出同形；桌面 remote 分頁與 local 分頁看到同一種順序與章、拖排語意同構。
- 版本偏斜可容忍：舊 server 沒有 plan 端點時，新 client 退回既有行為、不報錯。
- 收掉第二刀接受的限制：local 排程分頁的新增候選與前置寫入看同一份名冊。

**Non-Goals:**

- 不改 board resource 的格式與 PUT 校驗（server 對 PUT 內容仍不校驗語意）。
- 不把 plan 結果塞進既有 list 端點或 ChangeSummary。
- 不擴大 error reason registry。
- 不做 server-web console 的呈現。

## Decisions

### D1 GET /plan 經 gateway 執行同一引擎計算，rank 來自 board resource

新增 scope 層 route `GET /plan`，handler 先讀 scope 的 board resource 文件，以 `BoardOrderDoc::from_content` 寬鬆解析（缺席、壞 JSON、非物件皆視為空圖）取出 `changes` 段的 rank 圖，再以 `Command::Plan { ranks: Some(..) }` 經 `verb::run` 執行 `compute_with_ranks`。`Command::Plan` 增 `ranks: Option<BTreeMap<String, String>>` 欄位：None 時引擎讀 meta 的 board_rank（local CLI 傳 None），Some 時以其為準（remote）。reader 與 editor 皆可呼叫；零寫入、零事件；回應 `PlanResponse` 附 scope ETag。board 讀取與 plan 計算是兩次快照：兩次之間若有人拖排，回應的順序最多落後一次 invalidate，訂閱端收到事件會再讀。

寬鬆解析的唯一實作：`BoardOrderDoc`（兩段 rank 圖，`#[serde(default)]`）與 `from_content` 自 apps/desktop/src-tauri/src/remote.rs 移入 `speclink_protocol::query`，桌面 overlay 與 server plan 端點共用同一個函式，損壞容錯的語意因此不會分叉。server 不依賴桌面 crate、桌面與 server 都已依賴 protocol，而 board resource 內容形狀本來就是各 client 之間經 server 交換的約定；protocol 的 `serde_json` 自 dev 相依升為一般相依（schemars 已把它帶進建置圖，不增新 crate）。

成環：引擎把 `PlanError::Cycle` 歸類為一般錯誤（`ErrorCode::Error`，local CLI 與其測試不變）。server 唯一的錯誤映射點 `From<CommandError> for ApiError` 自 `CommandError.source` 下鑽 `speclink_core::plan::PlanError`，命中即回 HTTP 409、reason `refused`、message 為引擎原文（`dependency cycle: a -> b -> a`）——依型別判定、不比對字串，與 in-progress 移除守門下鑽 `RevertBlocked` 同一手法。未命中者維持五碼映射（一般錯誤仍是 500）。

remote-board-order 的「server 不解析」收窄為「PUT／GET /board-order 不校驗語意」，plan 端點的讀取是唯讀且失敗即空圖。

替代案：client 把 rank 圖隨請求送上——server 算出的順序會依 client 手上的快照而異，兩個 client 看到不同順序，被否決。

### D2 POST /changes/{name}/depends 直通引擎命令，409 沿用 refused

body 為 `SetDependsRequest { on: Vec<String>, remove: bool }`；handler 先檢查 `binding.editor`（reader 回 403、不執行命令），再以 `Command::ChangeDepends` 經 gateway 執行。錯誤全由引擎分類、經 D1 同一映射點上 wire：`<name>` 不存在（含不合法名稱）為引擎 `NotFound` → 404；自依賴、目標不存在或已封存、加邊後成環為引擎 `Refused` → 409、reason `refused`，message 為引擎原文——分別是 `'<name>' cannot depend on itself`、`cannot depend on '<x>': no active change with that name`、`cannot depend on '<x>': it is already archived`、`dependency cycle: <name> -> … -> <name>`；`<name>` 的 meta 壞掉為引擎 `InvalidConfig` → 422。守門失敗零寫入。outcome `changed` 為 true 時 bridge commit、發布 `ChangeDependsChanged` 並 invalidate，revision 前進；冪等無改動時 HTTP 200、零寫入、零事件。回應 `SetDependsResponse { change, depends_on }` 附 scope ETag。bridge 的 command 標籤補 `change-depends`。

為何不新增 reason：client-protocol「標準 error reason registry」規定 reason 屬封閉八值，細分情形放在 message；typed client 對 registry 以外的 reason 回通用句「unexpected server response…」，新 reason 會讓 CLI 印不出引擎原文。`refused` 的 message 原樣轉發，CLI 與桌面印出的句子與 fs 模式相同。桌面要區分的只有「GET /plan 成環」：該端點唯一的 409 就是成環（見 D5）。

### D3 protocol DTO 與 convert

`PlanResponse { waves: Vec<PlanWave>, changes: Vec<PlanChangeEntry>, next: Option<String>, skipped: Vec<PlanSkipped> }`，`PlanWave { index, changes }`，`PlanChangeEntry { name, wave, stage, depends_on, overlaps: Vec<PlanOverlap>, blocked_by, ready }`，`PlanOverlap { change, capabilities }`，`PlanSkipped { change, reason }`，全部 camelCase、陣列欄位 `#[serde(default)]` 容缺；`stage` 為字串（proposed／in-progress／ready）。`SetDependsRequest`、`SetDependsResponse` 同規。speclink-remote 的 convert 加 `plan_report(PlanResponse) -> speclink_core::plan::PlanReport`：`stage` 字串以引擎新增的 `Stage::parse`（`as_str` 的反查，與它同在 lifecycle/model.rs，字串表只有一份）轉回；無法辨識的階段、或 wave 列出 changes 沒有的名稱，都是 server 違約——回錯誤（`unexpected server response — …`），不猜階段，也不讓渲染端逐波查無此項而 panic。CLI remote 臂轉回引擎型別後共用同一渲染函式（與 analyze 同一手法）。

### D4 CLI 兩動詞升為 Dual

main.rs 的宣告層把 `plan` 與 `change` 自 `fs_only` 改為 `dual`，刪除兩句 FsOnly 拒絕常數：remote 臂分別呼叫 `client.plan()`（轉回 PlanReport 後走既有人眼／`--json` 渲染；server 回 409 時 `RemoteError` 的 message 就是 `dependency cycle: …`，經同一條 anyhow 路徑印到 stderr、非零 exit、stdout 為空）與 `client.set_depends(name, on, remove)`（回應的 change 與 dependsOn 交給兩臂共用的成功行渲染函式；wire 回應沒有 changed 旗標，不組引擎 `DependsOutcome`）。mode_dispatch 測試中兩動詞的 FsOnly 拒絕案例移除；新增 remote_plan 整合測試斷言兩模式同形。

### D5 桌面 remote 清單合併 plan 並容忍舊 server

remote.rs 的 `list_changes` 改為：取 server 清單與 board doc 後再呼叫 `client.plan()`，回傳桌面側 DTO `RemoteChangeList { changes: Vec<RemoteChangeItem>, plan_error: Option<String> }`（camelCase；`RemoteChangeItem` 以 `#[serde(flatten)]` 攤平 ChangeSummary 與選填排程欄組 `RemoteSchedule`——wave／blockedBy／dependsOn／overlaps 四欄出自 plan 同一列、同有同無，欄組缺席時四鍵皆不輸出）。plan 成功時以 `PlanResponse.changes` 的順序排列變更、每筆疊四欄；清單內有 plan 沒列到的項（兩次請求之間新增，或壞 meta 被 plan 列入 skipped）落在末尾、維持 server 序、四欄缺席——排列規則與本地看板同一個函式（desktop-core `query::plan_placed`：本地 basis 為基底序、remote 為 server 回傳序）。plan 回 409（GET /plan 唯一的 409：成環）時順序退回 board overlay、四欄缺席、`planError` 為 message；其他錯誤（404 舊 server、連線失敗、非 JSON 回應）同樣退回且 `planError` 為 null（不打擾使用者）。討論仍以 board doc overlay。前端資料源介面的 `listChanges()` 改為回傳 `{ changes, planError }` 整包：第二刀為了不動約 60 處測試替身而加的選配 `listChangesWithPlan` 與 store 的退路（缺此面時 planError 為 null），在 local 與 remote 兩個資料源都供得出 planError 後只剩測試替身會走到，因此併入 `listChanges`——tauriDataSource 與 remoteDataSource 都原樣回傳 command payload，store 直接讀整包，測試替身以共用的 `changeList(items, planError)` 組 payload。看板與系統匣因欄位存在自動顯示章與順序（第二刀的 stage.ts 入口）。

### D6 remote 拖排與本地同構：顯示序補章與宣告依賴檢查

`reorder_via` 的每次讀取（含 409 重試）在清單與 board resource 之外再呼叫 `client.plan()`，`BoardSnapshot` 帶 `plan: Option<PlanResponse>`（任何錯誤皆為 None）。`reorder_full_text` 的變更卡分支改為：

1. 顯示序：有 plan 時為 plan 配置序（plan 沒列到的項接在末尾、維持 server 回傳序），無 plan 時為 board overlay 序——與 D5 同一條排列規則。
2. 欄成員：顯示序中與被拖卡同階段者，含 plan 沒配置到的卡（壞 meta，排在欄尾）。它照常參與整欄重派與落點：board resource 寫得進它的 rank，而前端拖到它之後送出的 prevId 就是它——排除它的話，`neighbor_midpoint` 查不到鄰居，新鍵會算成空欄首鍵、落不到欄底。local 因為寫不進壞 meta 而把它排出欄外，remote 沒有這個限制。
3. 欄內有缺 rank 卡、或 rank 序與顯示序不一致時，依顯示序整欄等距重派 rank（只寫入 board resource 圖）；全員具 rank 且序一致時沿用現值——與 local board-card-order「rank 序與顯示序不一致時整欄重派」同一規則，否則反序的兩鄰居之間沒有可用的中點鍵。判定是 desktop-core `rank::ranked_in_order`，本地拖排（manage）與 remote 拖排共用，與既有共用的 `rank::spread`、`rank::neighbor_midpoint` 同一模組。
4. 以落點鄰居中點鍵更新被拖卡條目。
5. 有 plan 時：以新 rank 圖把 plan 有配置的欄成員排序成拖放後序列（plan 略過的卡不列入：引擎視對它的依賴為已滿足，與 local `check_rank_move` 的判定範圍相同），連同 plan 的 dependsOn 圖呼叫 `speclink_core::plan::blocked_move`——自 `check_rank_move` 抽出、建立在 `violations` 上的共用判定，只取涉及被拖卡的配對（被拖卡在其前置之前＝must_follow、依賴被拖卡者在其之前＝must_precede），本機與 remote 因此只有一份配對規則；非空即回 `RemoteError`（message 為 `plan::RankMoveBlocked` 的 Display，與 local 同一句）、不發 PUT；無 plan（舊 server、成環、請求失敗）時跳過此步。

重派與新鍵只存在於待 PUT 的全文內，檢查失敗時 board resource 完全不變（local 會先落盤補章，remote 因全文一次寫入而不必）。UI 走既有單行錯誤路徑並刷新。

### D7 remote setDepends 對 editor 開放

新增 Tauri command `remote_set_change_depends(locator, change, on, remove)` 單行委派 remote.rs 的 `RemoteWorkspace::set_depends`（`client.set_depends`，寫入走 `run_write`：離線即拒）；remoteDataSource 的 `setDepends` 呼叫它。capability 的來源是 Rust `RemoteCapabilities::from_binding`：`set_depends` 與 `reorder_card` 同源（handshake 的 `policyWrite`，editor 為 true、reader 為 false），session.ts 的 `REMOTE_WRITE_CAPABILITIES` 已含 `setDepends`（離線遮罩同列），只更新其型別註解。server 409 的 message 原樣成為單行錯誤文字。

### D8 事件與即時性

`ChangeDependsChanged` 走 bridge commit 的既有 invalidate 通道：訂閱端收到後重讀清單，`list_changes` 再呼叫 plan，順序與章即時更新；PUT /board-order 成功的 invalidate 同樣使 plan 重算。不新增事件種類以外的推播。

### D9 local 排程分頁的候選來自該變更所在名冊

第二刀的 `set_depends_at` 把前置寫到該變更的所在（有 worktree 映射寫其副本），引擎守門只認那份 store 的名冊；排程分頁的候選卻來自看板清單（主名冊＋副本），分支後才在主 checkout 建立的變更會出現在候選裡、選了被拒。

- 桌面 core `manage::depends_candidates_at(root, change) -> Result<Vec<String>, String>`：與 `set_depends_at` 同一定根（抽出共用的所在 store 解析，兩者不會分叉），回傳該 store `list_changes` 的變更名、扣掉自己，依名稱排序。
- Tauri command `depends_candidates(root, change)` 單行委派。
- `SpeclinkDataSource` 增選配方法 `dependsCandidates?(change): Promise<string[]>`；tauriDataSource 實作；remoteDataSource 不實作（remote scope 只有一份名冊，清單就是名冊）。
- RichDetailDrawer 增選配 prop `loadDependsCandidates`，與 meta、capability 同一個載入流程（開啟、換 change、刷新世代時載入，latest-wins）；PlanTab 的候選在已載入時取該清單、扣掉已宣告的前置；查詢未完成時不長新增下拉；未提供、或首次載入（開啟、換 change）失敗時照舊自 `changes` 派生；刷新世代的重新載入失敗時沿用前一次的查詢結果——與抽屜其他資源「重載失敗維持前值」同一收斂函式，候選也比自清單派生更貼近寫入所在的名冊。App.tsx 於資料源提供方法時接線。

## Implementation Contract

**行為**

- `GET /plan`：HTTP 200、body 同 CLI `plan --json` 形狀、附 scope ETag；reader 可呼叫；成環時 HTTP 409、reason `refused`、message 為 `dependency cycle: a -> b -> a`。
- `POST /changes/{name}/depends`：editor 200 回 `{ change, dependsOn }`；reader 403；unknown name 404；四種守門（自依賴、目標不存在、目標已封存、成環）409、reason `refused`、message 為引擎原文；實際改動時 revision 前進並發布 invalidate，冪等時 revision 不前進。
- CLI remote 模式：`speclink plan`／`--json` 與 `speclink change depends` 的 stdout、stderr、exit code 與 fs 模式對同一內容一致。
- 桌面 remote 分頁：變更順序為 plan 配置順序、卡片有波次章與阻擋章、拖到宣告前置之前得到單行錯誤且 board resource 不變、拖到 rank 反序的兩鄰居之間落在兩者之間、plan 略過的卡（壞 meta）不擋依賴檢查、拖到它之後落在欄底；舊 server 時順序與本變更前一致、無章、無錯誤。
- editor 於 remote 分頁的排程分頁可新增與移除前置；reader 無編輯控制項。
- 桌面 local 排程分頁：有 worktree 映射的變更，新增候選只含其副本內的作用中變更；無映射時與本變更前相同。

**介面**

- 路由：`GET /{scope}/plan`、`POST /{scope}/changes/{name}/depends`。
- protocol：`PlanResponse`、`PlanWave`、`PlanChangeEntry`、`PlanOverlap`、`PlanSkipped`、`SetDependsRequest`、`SetDependsResponse`、`BoardOrderDoc::from_content`。
- remote client：`plan() -> Result<PlanResponse, RemoteError>`、`set_depends(name, on: &[String], remove: bool) -> Result<SetDependsResponse, RemoteError>`；convert：`plan_report(PlanResponse) -> Result<PlanReport, RemoteError>`。
- 引擎：`Command::Plan { ranks: Option<BTreeMap<String, String>> }`（local 呼叫端傳 None）；`Stage::parse(&str) -> Option<Stage>`（`as_str` 的反查）。
- 桌面 core：`query::plan_placed(basis, placed, name_of)`、`rank::ranked_in_order(ranks)`（本地與 remote 共用）。
- 桌面：Tauri `remote_set_change_depends`、`depends_candidates`；`remote_list_changes` 回 `{ changes: RemoteChangeItem[], planError: string | null }`；資料源 `listChanges(): Promise<{ changes, planError }>`（local 與 remote 皆然，併入第二刀的選配 `listChangesWithPlan`）、`dependsCandidates?`（local 實作）。

**失敗模式**

- plan 端點 404／連線失敗 → 桌面退回 board overlay、planError null；CLI remote 臂印既有的連線錯誤。
- plan 端點 409 → 桌面退回 board overlay、planError 為訊息；CLI remote 臂印訊息、非零 exit。
- plan 回應違約（未知階段、wave 成員不在 changes）→ CLI remote 臂印 `unexpected server response — …`、非零 exit，不 panic。
- 拖排違反依賴 → 不發 PUT、單行錯誤、刷新。
- depends 寫入 409 → 單行錯誤，畫面不變。
- 候選首次載入失敗 → 候選自清單派生；重新載入失敗 → 沿用前次名冊（寫入仍由引擎守門）。

**驗收**

- server 測試（`cargo test -p speclink-server --test it api::plan_api`）：GET /plan 形狀與 ETag、reader 可讀、rank 來自 board resource（有 rank 圖時順序改變）、壞 board 內容視為空圖、成環 409 refused；POST depends 的 200／403／404／四種守門 409 與 revision 行為。
- speclink-protocol／speclink-remote 測試：DTO camelCase 與缺欄位容忍、BoardOrderDoc 寬鬆解析、convert（含未知階段與 wave 成員不在 changes 皆回錯）、plan 與 set_depends 的路徑與 body、409 refused 原文轉發；speclink-core：`Stage::parse` 為 `as_str` 的反查。
- CLI 整合測試（`--test it remote_plan`、`mode_dispatch`、`remote_verb_parity`）：兩模式同形、remote 成環 exit code 與 stderr。
- 桌面 Tauri 測試（`cargo test -p speclink-desktop --test it remote_data` 與 remote.rs 單元測試）：list 合併順序與四欄位、舊 server 退回、成環退回、reorder 違反依賴不發 PUT、rank 反序時整欄重派、plan 略過的卡不擋依賴檢查、拖到它之後落在欄底；前端 `npm test -w apps/desktop`：remoteDataSource 的 setDepends 與 listChanges 帶 planError、capability 隨 role、tauriDataSource 的 dependsCandidates。
- 桌面 core 測試（`cargo test -p speclink-desktop-core`）：worktree 映射時候選來自副本名冊、`plan_placed` 與 `ranked_in_order` 的規則；`npm test -w packages/ui`：PlanTab 以載入的候選為準、重新載入失敗沿用前次名冊。

**範圍**

- In：上列 crates、apps/desktop 的 remote 路徑與排程分頁候選、packages/ui 的排程分頁、六份規格 delta。
- Out：local 的 plan 計算、清單排序與拖排的行為（本地 `board_order` 與拖排只改為呼叫與 remote 共用的 `plan_placed`、`ranked_in_order`，既有測試不變）；board resource 格式；error reason registry；server-web。

## Risks / Trade-offs

- [server 對 board resource「不解析」的承諾收窄] → 只有 plan 端點讀、寬鬆解析、失敗即空圖，PUT／GET 契約逐字不變；規格 MODIFIED 明載。
- [版本偏斜：新 client 對舊 server] → plan 404 視為無排程資訊，退回舊行為；新 server 對舊 client 沒有影響（新端點不被呼叫）。
- [兩次請求（list＋plan）的順序快照不一致] → 同一次 `list_changes` 內連續呼叫、以 plan 為排序真相，plan 未列出清單內的項時該項落在末尾並保持 server 序。
- [remote 成環時排程分頁無法解環（已接受的限制）] → plan 409 時四欄缺席，抽屜排程分頁顯示「此模式尚未提供排程資訊」、沒有可移除的前置；看板的成環提示列仍會出現。remote 成環只在兩人幾乎同時加反向前置（兩筆寫入改不同文件，CAS 攔不到），或匯入手改過的本機 meta 時發生；解環走 CLI `speclink change depends <name> --on <x> --remove`。
- [拖排每次多一個 plan 請求] → 只在拖排與清單讀取時發生，與既有 list＋board 兩請求同量級；plan 失敗不擋拖排。
- [remote_verb_parity 紅線] → 兩動詞的 remote 臂轉回引擎型別後共用渲染，測試斷言兩模式 stdout 逐位元一致。
- [回歸對照] → local CLI `plan`／`change depends` 的人眼與 `--json` 輸出不變（plan_verbs 測試）；`speclink list --json` 不含四欄（既有測試）；golden 不受影響（未動渲染）。
- [跨平台] → 新增路徑無檔案系統分隔符假設（名冊比對為 change 名逐字比對）；Windows／Linux 的桌面 Tauri 測試需 sidecar 與 server-web dist，沿 remote_data 既有夾具；worktree 候選測試沿 desktop-core 既有 worktree 夾具。

## Migration Plan

先落 add-change-plan-engine 與 add-change-plan-desktop。server 與 client 隨版本一起出貨；升級順序任意（見版本偏斜風險）。回滾即移除兩條 route、桌面合併邏輯與候選查詢，board resource 資料不受影響。

## Open Questions

無。

## Context

remote 路徑的既有形狀：server 以 axum 提供動詞端點，寫入動詞（如 POST /changes/{name}/in-progress）經 `verb::run` 走 Command gateway 直通引擎命令，bridge 於 outcome 帶事件時 commit 並發布 invalidate；唯讀衍生查詢（validate、analyze）同樣經 gateway 但零寫入。protocol crate 的 Rust 型別是 wire 正典（camelCase serde），remote client 是型別化呼叫，speclink-remote 的 convert 模組把回應轉回引擎型別讓 CLI 兩模式共用渲染。桌面 remote 由 src-tauri 的 remote.rs 組裝：list 先取 server 清單再以 board resource 疊 rank 順序；拖排以 `reorder_via` 讀快照、改全文、CAS PUT。第一刀提供 `plan::compute_with_ranks`、`plan::violations`、`plan::set_depends` 與 `ChangeDependsChanged` 事件；第二刀提供前端的四欄位、章、灰化、排程分頁與 `setDepends` 資料源介面。

## Goals / Non-Goals

**Goals:**

- remote 與 local 讀同一份計算：server 的 plan 端點呼叫同一個引擎函式。
- CLI 兩模式輸出同形；桌面 remote 分頁與 local 分頁看到同一種順序與章。
- 版本偏斜可容忍：舊 server 沒有 plan 端點時，新 client 退回既有行為、不報錯。

**Non-Goals:**

- 不改 board resource 的格式與 PUT 校驗（server 對 PUT 內容仍不校驗語意）。
- 不把 plan 結果塞進既有 list 端點或 ChangeSummary。
- 不做 server-web console 的呈現。

## Decisions

### D1 GET /plan 經 gateway 執行同一引擎計算，rank 來自 board resource

新增 scope 層 route `GET /plan`，handler 以 `Command::Plan` 經 `verb::run` 執行；gateway 側在執行前讀取 scope 的 board resource 文件並以桌面同一套寬鬆解析（缺席、壞 JSON、非物件皆視為空圖）取出 `changes` 段的 rank 圖，注入 `compute_with_ranks`。為此 `Command::Plan` 增一個 `ranks: Option<BTreeMap<String, String>>` 欄位：None 時引擎讀 meta 的 board_rank（local），Some 時以其為準（remote）。reader 與 editor 皆可呼叫；零寫入、零事件；回應 `PlanResponse` 附 scope ETag。remote-board-order 的「server 不解析」收窄為「PUT／GET 不校驗語意」，plan 端點的讀取是唯讀且失敗即空圖。

替代案：client 把 rank 圖隨請求送上——server 算出的順序會依 client 手上的快照而異，兩個 client 看到不同順序，被否決。

### D2 POST /changes/{name}/depends 直通引擎命令

body 為 `SetDependsRequest { on: Vec<String>, remove: bool }`；handler 以 `Command::ChangeDepends` 經 gateway 執行，editor 限定（reader 403）。錯誤映射：`<name>` 不存在 404；自依賴、目標不存在或已封存、成環 → 409，`reason` 分別為 `depends-self`、`depends-target-invalid`、`dependency-cycle`，訊息帶引擎原文。outcome `changed` 為 true 時 bridge commit、發布 `ChangeDependsChanged` 並 invalidate，revision 前進；冪等無改動時 HTTP 200、零寫入、零事件。回應 `SetDependsResponse { change, depends_on }`。

### D3 protocol DTO 與 convert

`PlanResponse { waves: Vec<PlanWave>, changes: Vec<PlanChangeEntry>, next: Option<String>, skipped: Vec<PlanSkipped> }`，`PlanChangeEntry { name, wave, stage, depends_on, overlaps: Vec<PlanOverlap>, blocked_by, ready }`，全部 camelCase、`#[serde(default)]` 容缺；`SetDependsRequest`、`SetDependsResponse` 同規。speclink-remote 的 convert 加 `plan_report(PlanResponse) -> speclink_core::plan::PlanReport`，CLI remote 臂轉回引擎型別後共用同一渲染函式（與 analyze 同一手法）。

### D4 CLI 兩動詞升為 Dual

main.rs 的宣告層把 `plan` 與 `change` 自 `fs_only` 改為 `dual`：remote 臂分別呼叫 `client.plan()`（轉回 PlanReport 後走既有人眼／`--json` 渲染，成環時 server 回 409 `dependency-cycle`，CLI 印同一句 `dependency cycle: …` 到 stderr、非零 exit）與 `client.set_depends(name, on, remove)`（成功印同一行 `✓ …`）。mode_dispatch 測試中兩動詞的 FsOnly 拒絕案例移除，remote_verb_parity 補兩動詞的同形斷言。

### D5 桌面 remote 清單合併 plan 並容忍舊 server

remote.rs 的 `list_changes` 改為：取 server 清單與 board doc 後再呼叫 `client.plan()`；成功時以 `PlanResponse.changes` 的順序排列變更（討論仍以 board doc overlay），每筆疊 wave／blockedBy／dependsOn／overlaps，回傳桌面側 DTO `RemoteChangeItem`（`#[serde(flatten)]` 包 ChangeSummary 加四個 `skip_serializing_if = Option::is_none` 欄位）與頂層 `planError`；plan 回 409 `dependency-cycle` 時順序退回 board overlay、四欄缺席、`planError` 為訊息；plan 回 404（舊 server）或其他錯誤時同樣退回且 `planError` 為 null（不打擾使用者）。前端 remoteDataSource 讀 `changes` 與 `planError`，看板與系統匣因欄位存在自動顯示章與順序（第二刀的 stage.ts 入口）。

### D6 remote 拖排前的宣告依賴檢查

`reorder_via` 在算出新全文前，先以快照的 plan 結果（D5 同一次 `client.plan()` 的 dependsOn 圖）與拖放後同階段的名稱序列呼叫 `speclink_core::plan::violations`；有違反即回 `RemoteError` 帶引擎同款訊息、不發 PUT；UI 走既有單行錯誤路徑並刷新。無 plan 結果（舊 server）時跳過檢查。前端的灰化（第二刀）只在有四欄位時生效，與此一致。

### D7 remote setDepends 對 editor 開放

新增 Tauri command `remote_set_change_depends(locator, change, on, remove)` 單行委派 remote.rs 的 `set_depends`（`client.set_depends`）；remoteDataSource 的 `setDepends` 呼叫它；session.ts 的 remote capability `setDepends` 與 `reorderCard` 同源（editor 為 true、reader 為 false、offline mask 同列）。server 409 的 reason 經既有錯誤翻譯成單行文字。

### D8 事件與即時性

`ChangeDependsChanged` 走 bridge commit 的既有 invalidate 通道：訂閱端收到後重讀清單，`list_changes` 再呼叫 plan，順序與章即時更新；PUT /board-order 成功的 invalidate 同樣使 plan 重算。不新增事件種類以外的推播。

## Implementation Contract

**行為**

- `GET /plan`：HTTP 200、body 同 CLI `plan --json` 形狀、附 scope ETag；reader 可呼叫；成環時 HTTP 409、reason `dependency-cycle`、訊息為 `dependency cycle: a -> b -> a`。
- `POST /changes/{name}/depends`：editor 200 回 `{ change, dependsOn }`；reader 403；unknown name 404；四種守門 409 附 reason；實際改動時 revision 前進並發布 invalidate，冪等時 revision 不前進。
- CLI remote 模式：`speclink plan`／`--json` 與 `speclink change depends` 的 stdout、stderr、exit code 與 fs 模式對同一內容一致。
- 桌面 remote 分頁：變更順序為 plan 配置順序、卡片有波次章與阻擋章、拖到宣告前置之前得到單行錯誤且 board resource 不變；舊 server 時順序與本變更前一致、無章、無錯誤。
- editor 於 remote 分頁的排程分頁可新增與移除前置；reader 無編輯控制項。

**介面**

- 路由：`GET /{scope}/plan`、`POST /{scope}/changes/{name}/depends`。
- protocol：`PlanResponse`、`PlanWave`、`PlanChangeEntry`、`PlanOverlap`、`PlanSkipped`、`SetDependsRequest`、`SetDependsResponse`。
- remote client：`plan() -> Result<PlanResponse, RemoteError>`、`set_depends(name, on: &[String], remove: bool) -> Result<SetDependsResponse, RemoteError>`。
- 引擎：`Command::Plan { ranks: Option<BTreeMap<String, String>> }`（第一刀的 `Command::Plan` 加欄位，local 呼叫端傳 None）。
- 桌面：Tauri `remote_set_change_depends`；`remote_list_changes` 回 `{ changes: RemoteChangeItem[], planError: string | null }`。

**失敗模式**

- plan 端點 404／連線失敗 → 桌面退回 board overlay、planError null；CLI remote 臂印既有的連線錯誤。
- 拖排違反依賴 → 不發 PUT、單行錯誤、刷新。
- depends 寫入 409 → 單行錯誤，畫面不變。

**驗收**

- server 測試（`cargo test -p speclink-server --test plan_api`）：GET /plan 形狀與 ETag、reader 可讀、rank 來自 board resource（有 rank 圖時順序改變）、壞 board 內容視為空圖；POST depends 的 200／403／404／409 四種與 revision 行為。
- speclink-remote／protocol 單元測試：DTO camelCase 與缺欄位容忍、convert 往返。
- CLI 整合測試（`--test it remote_plan`、`mode_dispatch`、`remote_verb_parity`）：兩模式同形、remote 成環 exit code 與 stderr。
- 桌面 Tauri 測試（`cargo test -p speclink-desktop --test remote_data`）：list 合併順序與四欄位、舊 server 退回、reorder 違反依賴不發 PUT；前端 `npm test -w apps/desktop`：remoteDataSource 的 setDepends 委派與 capability 隨 role。

**範圍**

- In：上列 crates 與 apps/desktop remote 路徑檔案、五份規格 delta。
- Out：local 模式行為；board resource 格式；server-web。

## Risks / Trade-offs

- [server 對 board resource「不解析」的承諾收窄] → 只有 plan 端點讀、寬鬆解析、失敗即空圖，PUT／GET 契約逐字不變；規格 MODIFIED 明載。
- [版本偏斜：新 client 對舊 server] → plan 404 視為無排程資訊，退回舊行為；新 server 對舊 client 沒有影響（新端點不被呼叫）。
- [兩次請求（list＋plan）的順序快照不一致] → 同一次 `list_changes` 內連續呼叫、以 plan 為排序真相，清單缺少 plan 內名稱時該項落在末尾並保持 server 序。
- [remote_verb_parity 紅線] → 兩動詞的 remote 臂轉回引擎型別後共用渲染，測試斷言兩模式 stdout 逐位元一致。
- [Windows／Linux 桌面 Tauri 測試需 sidecar 與 dist] → 沿 remote_data 既有夾具。

## Migration Plan

先落 add-change-plan-engine 與 add-change-plan-desktop。server 與 client 隨版本一起出貨；升級順序任意（見版本偏斜風險）。回滾即移除兩條 route 與桌面合併邏輯，board resource 資料不受影響。

## Open Questions

無。

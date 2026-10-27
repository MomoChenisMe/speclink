## Context

引擎已有兩道守門但入口涵蓋不全:批次封存在 CLI 層預過濾任務完成度(speclink-cli commands.rs 的 bulk 路徑),單筆封存(core archive 函式)不檢查;discard 動詞(speclink-core discard.rs)守開工痕跡,但 desktop 的本地刪除(apps/desktop/core manage.rs 的 delete_change_at)直接刪目錄、remote 刪除固定帶 force=true,雙雙繞過。UI 三表面(卡鈕/抽屜鈕/拖曳落點)的階段限制互不一致。desktop 的 archive 動詞經 verbs.rs 直呼 core 封存函式、不走 command runtime——守門位置的選擇必須涵蓋此直呼入口。先例:revert-in-progress-to-proposed 確立「UI 依派生階段決定可見性、引擎是唯一裁決點」分工;core 封存函式內的 metadata fail-closed 檢查即為「直呼入口也要守」的既有做法。

## Goals / Non-Goals

**Goals:**

- 單筆封存在 core 層 fail-closed:任務未全完成即拒絕,三入口(CLI/desktop/server)一體適用。
- desktop 本地與 remote 刪除皆走 discard 語意(force=false),守門、討論解鏈、touched 清理不再繞過。
- UI 三表面收斂:落點僅就緒卡浮現、抽屜鈕非法階段 disabled + 原因、卡鈕不變。

**Non-Goals:**

- 不動批次封存的預過濾與回報格式;不動 --mark-tasks-complete 旗標語意。
- 不將 drift(stale delta assumptions)檢查併入單筆守門。
- 不提供 desktop force 通道;不新增放棄型封存動詞;不處理 0 任務 change 邊界(維持與批次一致的總數>0 條件)。
- 不改卡片解剖學(不在卡片上加鈕)、不改封存確認對話框與 toast 機制。

## Decisions

### D1 守門放 core 封存函式本體,拒絕用 typed Refusal

任務完成度守門加在 speclink-core 的 archive 函式內、緊隨既有 metadata fail-closed 檢查之後:opts 未帶 mark_tasks_complete、且 tasks.md 解析後總數>0 且完成數<總數 → 拒絕,錯誤訊息(英文,與 CLI 錯誤慣例一致)列證據「N/M tasks complete」與兩條出路(完成任務、--mark-tasks-complete)。拒絕以 command 層的 typed Refusal 包裝(同 discard 先例),command runtime 分類為 refused;desktop 直呼路徑經既有 map_err 轉字串進 toast。任務讀取經 Store trait(read_artifact + tasks 解析),不觸儲存媒介——維持 storage 解耦。
替代:守門放 command runtime 的 run_archive——desktop verbs.rs 直呼會繞過,重蹈本案要修的覆轍;放各入口各自檢查——三份重複、日後再分歧。
注意:CLI 帶 --mark-tasks-complete 時 runtime 先把 tasks.md 全勾再進封存,守門此時自然通過;條件仍明列旗標,使未經 pre-write 的直呼入口語意一致(豁免=旗標,不是 pre-write 副作用)。

### D2 desktop 刪除改接 discard,remote 翻案 force=false

manage.rs 的 delete_change_at 改為委派 speclink-core 的 discard 函式(force=false),簽名與回傳(Result<(), String>)不變——解鏈明細不呈現,前端既有 refresh 自然反映討論狀態回復;Tauri 殼 command 維持單行委派。remoteDataSource 的 deleteChange 改帶 force=false,server 端 DELETE change 本就是 discard 全語意,拒絕經既有錯誤橋接進 deleteFailed toast——原「桌面 remote 刪除固定 force=true(與本地無 guard 直刪同模式)」決策隨本地補守門一併翻案。
替代:desktop 呈現解鏈明細——UI 無承載處且 refresh 已反映,過度設計;保留 remote force=true——與本地新語意矛盾,留下第二條繞道。

### D3 落點浮現條件以就緒名單傳入純函式

boardDnd.ts 的 archiveZoneVisible 擴充簽名:除 activeDndId 外收就緒變更名集合(ReadonlySet),僅當拖曳中的卡屬 change 種類且名列就緒集合時回 true。KanbanBoard 以既有 changeStage 派生就緒名單傳入;dragEnd 的 archived 落點分支同樣以就緒集合防禦(落點不可見時理論上不會 over,但守住快速手勢與測試路徑)。維持「純函式+薄 dragEnd 接線」模式,jsdom 照測純函式。
替代:把階段編進 dnd id 前綴——污染 id 契約,parseCardDndId 的消費端全要跟著改。

### D4 抽屜鈕守門原因與既有 UnavailableAction 合流

RichDetailDrawer 自身以 changeStage 派生階段(退回鈕已如此):封存鈕於非就緒、刪除鈕於非提案中時 disabled,原因經既有 UnavailableAction 呈現。原因優先序:remote 能力缺失(宿主傳入的 unavailable)優先於階段原因——「這條通道做不到」比「現在還不能」更硬。文案進 packages/ui 的 i18n(雙語):封存原因載明任務進度與出路、刪除原因指向退回提案中(CLI --force 僅在文案中提及,desktop 不提供通道)。
替代:原因在 App.tsx 組好傳入——抽屜已自派生階段(退回鈕先例),宿主組裝徒增接線;直接隱藏鈕——討論已裁定 disabled + 原因。

## Implementation Contract

- **可觀察行為**:
  - CLI:speclink archive <name> 對任務未完成(總數>0)的 change 非零 exit code、stderr 列「N/M」證據與兩出路;帶 --mark-tasks-complete 或任務全完成/0 任務時行為與現行逐位元一致;批次路徑輸出不變。
  - desktop 看板:拖曳非就緒變更卡全程不浮現封存落點、放開僅排序;拖曳就緒卡浮現落點且封存確認流程不變。
  - desktop 抽屜:非就緒 change 的封存鈕、非提案中 change 的刪除鈕 disabled 且 tooltip 呈現原因;就緒封存、提案中刪除照常。
  - desktop 刪除(本地與 remote):對從討論轉出的提案中 change 刪除後,來源討論的已轉出清單同步移除(清單空時狀態回復);有開工痕跡的 change 刪除被引擎/伺服器拒絕,以 toast 呈現。
- **介面/資料形狀**:ArchiveOptions 不變;archiveZoneVisible(activeDndId, readyIds) 簽名擴充;delete_change_at 簽名與回傳不變;remote DELETE change 的 force 參數改傳 false;--json 欄位無新增。
- **失敗模式**:守門拒絕=typed Refusal(runtime 分類 refused、HTTP 對應既有拒絕語意);desktop 併發情境(UI 可見期間階段已變)由引擎拒絕→既有 archiveFailed/deleteFailed toast;metadata 損壞維持既有 fail-closed(含 force)。
- **驗證目標**:speclink-core 單元測試(守門拒絕/豁免/0 任務放行、Refusal 分類)、speclink-cli 整合測試(單筆拒絕輸出、成功路徑逐位元)、desktop core 測試(delete_change_at 走 discard:守門拒絕與解鏈)、vitest(boardDnd 純函式、KanbanBoard 落點浮現、RichDetailDrawer disabled+原因、remoteDataSource force=false)。

## 風險與緩解

- **回歸對照**:單筆封存的成功路徑(全完成/0 任務/--mark-tasks-complete)須逐位元不變——既有測試保留為證,新增拒絕路徑測試釘住新輸出;批次路徑不動,既有 bulk 測試護欄。
- **跨平台**:守門唯讀 tasks.md 經 Store trait,無 git 或路徑分隔符依賴;discard 接線沿用已測的 core 函式,無新 I/O。
- **UI 迴歸**:拖曳語意沿「純函式+薄接線」模式,jsdom 測 archiveZoneVisible 與 resolveCardDrop 不互相干擾;抽屜 disabled 條件與 remote 能力旗標交疊處以測試釘優先序。

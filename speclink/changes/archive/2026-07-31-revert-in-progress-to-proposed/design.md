## Context

「進行中」是派生狀態:packages/ui/src/stage.ts 的 changeStage 由「meta 含 started_at」或「已勾任務數 > 0」推出。started_at 由 crates/speclink-core/src/inprogress.rs 的 add 蓋章(CLI in-progress add、任務首勾、remote 端點三路共用),但沒有反向動詞——戳記一旦蓋上就永久停留。touched 記錄(crates/speclink-core/src/tasks.rs 的 TouchedRecord,v1 touched 與 v2 entries 兩清單)只在 discard 時整檔刪除,沒有使用者可觸及的清除動詞;取消勾選是純狀態翻轉,不動 touched 與戳記。生命週期 gate(crates/speclink-host/src/gate.rs)是 forward-only 單一裁決點,尚未接線 enforcement。

看板拖曳有 spec 明文 pin「跨欄拖曳不改變變更階段」(packages/ui/src/boardDnd.ts);討論卡有封存按鈕但討論抽屜沒有(既有不對稱)。SpeclinkDataSource 介面(packages/ui/src/adapter.ts)只有 desktop 的兩個 adapter 實作(tauriDataSource 本地、remoteDataSource 遠端),server-web 不實作此介面。

## Goals / Non-Goals

**Goals:**

- 一個引擎反向動詞,三個入口(CLI、desktop 按鈕、remote 端點)共用同一守門裁決,行為一字不差。
- 守門語意精確可測:零工作痕跡才放行,拒絕時證據結構化回傳。
- desktop 的退回動作與守門對話框;討論抽屜補封存動詞。

**Non-Goals:**

- 不開放跨欄拖曳改狀態(spec pin 不動)。
- 不提供 --force 或機械清理 touched/已勾任務的路徑。
- 不動 gate 六站轉換表(退回是 gate 外修正動詞,與 discard 同類)。
- 不處理 remote 模式 touched/evidence 被污染時的修復路徑。
- server-web 不加此功能(不實作 SpeclinkDataSource,無看板)。

## Decisions

### D1 反向動詞落在 speclink-core 的 inprogress 模組,守門在引擎函式內

remove 函式與 add 同居 crates/speclink-core/src/inprogress.rs,簽名比照 add 收 store 與 change 名,另收 Workspace 以載入 TouchedRecord(與 tasks.rs 的 complete 同款來源)。守門裁決寫在引擎函式內,command runtime 只轉發——CLI、desktop core、server 三個入口自然共用。
替代方案:守門放 command runtime 或各入口——被否決,三入口重複裁決邏輯,漂移風險;守門是領域規則,歸 speclink-core。

### D2 守門與冪等語意(閉集,逐條可測)

- 已勾任務數:解析該 change 的 tasks.md 勾選數(與 stage 派生同源);tasks.md 不存在視為 0。
- touched 判空:TouchedRecord 的 touched(v1)與 entries(v2)兩清單皆空;記錄檔不存在視為空。
- 已勾任務 > 0 或 touched 非空 → 拒絕,錯誤帶結構化證據:已勾任務數與 touched 檔案清單(聯集、去重)。
- 未知 change → 錯誤(找不到變更)。與 add 對未知名稱的靜默成功刻意不對稱:add 的靜默是遷移前 parity 凍結,不是理想行為;remove 是新動詞、無 parity 包袱,修正動作打錯名字必須明確報錯。
- 未開工(meta 無任何 started_* 欄位)→ 冪等成功,零寫入,不發事件(與 add 重複蓋章的靜默語意對稱)。
- meta 損毀 → fail-closed 報錯,不動任何檔案(與 add 同)。
- 放行 → 以行過濾移除 started_at/started_by/started_with 三行,其餘內容逐字保留(read → 行過濾 → write,不重新序列化;與 add 的 append 手法互為鏡像)。
替代方案:未知 change 也靜默成功(與 add 全對稱)——被否決如上;守門只擋已勾任務、touched 另計——被否決,touched 非空代表檔案曾被動過,正是不可機械退回的證據。

### D3 Command 面:新 Command 變體、拒絕以錯誤回報、成功發事件

Command::InProgressRemove(收 change 名)與對應 outcome(記錄是否實際移除)。守門拒絕以 CommandError 回報(CLI exit 非 0、stderr 說明),證據隨錯誤結構走;成功且實際移除時發新 DomainEvent(ChangeInProgressRemoved),冪等未移除不發事件——SSE 端沿既有 invalidation hint 契約流動,server-event-stream 規格不需修改。
替代方案:拒絕做成 outcome 的一種(exit 0)——被否決,守門拒絕是失敗,呼叫端(對話框、腳本)需要以錯誤路徑分流;沿用既有事件型別——被否決,加標記與移除標記是不同事實,事件語意不可混用。

### D4 HTTP 形狀:DELETE /changes/{name}/in-progress

與 POST /changes/{name}/in-progress 同資源、反向方法,語意即「刪除 in-progress 標記」。成功(含冪等未開工)HTTP 200 Ack、零寫入時不 commit 不發事件;守門拒絕 HTTP 409,error payload 帶 camelCase 證據欄位:checkedTasks(數量)與 touchedFiles(檔案清單);未知 change HTTP 404。typed client(crates/speclink-remote/src/client.rs)補對應方法,證據形狀進 crates/speclink-protocol(serde camelCase rename),欄位為對外契約、只增不改,不影響既有 payload 的向後相容。
替代方案:POST /changes/{name}/in-progress/remove——被否決,REST 語意上標記是資源,DELETE 精確且與既有路由成對;409 改 422——被否決,拒絕源於資源當前狀態衝突(有工作痕跡),409 語意正確。

### D5 desktop:按鈕直呼引擎、確認後執行、被擋開證據對話框

- ChangeCard 與 RichDetailDrawer 在該變更為進行中(startedAt 非空或已勾任務 > 0 的派生)時呈現「退回提案中」動作,樣式沿討論卡封存按鈕(卡片動作列+抽屜動作區)。
- 點擊先出確認(與討論封存的確認同款流程),確認後直接呼叫 SpeclinkDataSource 新方法 revertChangeToProposed(change),UI 不預判守門——list payload 沒有 touched 狀態,預判會做出第二裁決點。
- 守門拒絕時,adapter 把引擎證據(已勾任務數、touched 檔案清單)以結構化錯誤拋出,App.tsx 接住開守門對話框:列出證據,說明出路(已勾任務可於任務分頁取消後重試;touched 需請 agent 判斷,GUI 不提供清理)。
- 本地模式:桌面邏輯歸 apps/desktop/core(不依賴 Tauri、可獨立 cargo test),Tauri 殼 command 單行委派;remote 模式:src-tauri 的 remote bridge 打 DELETE 端點,409 證據轉同一結構化錯誤——兩模式共用同一對話框元件與文案(packages/ui/src/i18n.tsx)。
- 退回成功後不手動搬卡:重載後 startedAt 為空、勾選為 0,changeStage 派生自然回「提案中」欄(看板是派生的鏡子)。
替代方案:UI 預判守門、無痕跡才顯示按鈕——被否決,需擴 list payload 且生第二裁決點;失敗只出 toast——被否決,證據清單(touched 檔案)需要可讀的載體,對話框是既有模式。

### D6 討論抽屜補封存動詞(順帶修復)

DiscussionDrawer 對 concluded 且未封存的討論呈現「封存」動作,接既有 onArchiveDiscussion 流程(App.tsx 的確認對話框與 adapter 呼叫原樣復用),與討論卡完全同語意。desktop-app 規格「討論抽屜檢視與轉出變更」需求同步修訂:封存動詞在卡與抽屜皆可及。
替代方案:抽屜內另做無確認的快速封存——被否決,同一動詞兩種確認行為違反單一語意。

### D7 詞彙與技能

- openspec/LANGUAGE.md 立「退回提案中」詞條:定義=把誤開工的變更自進行中退回提案中(移除開工戳記,僅零工作痕跡時可行),avoid=撤回開工、取消開工、unstart(中文散文),why=動詞直說結果、與看板欄名呼應。
- apply 技能正典(crates/speclink-core/assets/skills/apply.md)補「開錯工怎麼退」小節:指認 speclink in-progress remove、守門被擋時的兩條出路;生成物(.claude/skills/speclink-apply/SKILL.md)同步再生,claude 與 codex 兩工具皆含。

## Implementation Contract

**行為(使用者可觀察):**

- CLI:speclink in-progress remove 對零痕跡的進行中變更 exit 0 並印移除確認;對未開工變更 exit 0(冪等,提示已在提案中);對有痕跡變更 exit 非 0,stderr 列已勾任務數與 touched 檔案清單及出路;對未知 change exit 非 0。
- desktop:進行中卡與詳情抽屜有「退回提案中」;確認後成功則卡片回提案中欄;被擋則對話框列證據與出路。本地與 remote 模式行為一致。
- 討論抽屜:concluded 討論可自抽屜封存,流程與討論卡一致。
- speclink in-progress add 的輸出與行為位元級不變。

**介面/資料形狀:**

- 引擎:inprogress 模組新增 remove 函式(store、workspace、change 名),回傳是否實際移除;守門拒絕為結構化錯誤(已勾任務數、touched 檔案清單)。
- Command::InProgressRemove;成功且移除時發 DomainEvent::ChangeInProgressRemoved。
- HTTP:DELETE /changes/{name}/in-progress → 200 Ack | 409(payload 欄位 checkedTasks、touchedFiles,camelCase)| 404。
- SpeclinkDataSource 新方法 revertChangeToProposed(change),守門拒絕拋含證據的結構化錯誤。
- meta 寫回只做行移除,既有欄位逐字保留;不新增任何持久化欄位,touched 記錄唯讀。

**驗證目標(TDD,覆蓋 80% 起):**

- speclink-core 單元:remove 的放行/已勾任務擋/touched 擋(v1 與 v2 各自非空)/未知名/未開工冪等/meta 損毀 fail-closed/欄位逐字保留。
- command 層:outcome 與事件(移除發、冪等不發)、錯誤分類。
- CLI 整合(crates/speclink-cli/tests):四種 exit code 情境與 stderr 證據。
- server 路由:200/409(payload 形狀)/404,冪等不 commit 不發事件。
- desktop core(apps/desktop/core):本地 bridge 的成功與證據錯誤透傳。
- packages/ui 元件測試:按鈕出現條件(僅進行中)、確認流程、守門對話框渲染證據、討論抽屜封存動作出現條件(僅 concluded 未封存)。

**範圍邊界:**

- in scope:上述引擎/CLI/server/desktop/UI/技能/詞彙。
- out of scope:跨欄拖曳、--force、gate 轉換表、remote 污染修復、server-web。

## Risks / Trade-offs

- desktop 能推進(勾選蓋戳)卻只能在零痕跡時退回——刻意的不對稱,守門對話框誠實引導請 agent 處理,不給假乾淨的機械出路。
- add 與 remove 對未知名稱行為不對稱(靜默 vs 報錯)——add 受 parity 凍結,remove 選正確行為;於 CLI help 與技能文字明示。
- 新 DomainEvent 對既有 SSE 消費者是新 hint 型別——invalidation hint 契約本就允許增型別,消費端以重載回應,無相容性風險。

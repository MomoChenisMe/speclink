## Context

實測時間軸（stderr 儀器）定位：勾選的 set_task_done done=true 於 Rust 主執行緒耗 6.8 秒（git_identity 6.17s、tasks::complete 0.6s），取消勾選僅 5.5ms；Tauri 非 async command 佔用主執行緒使整窗凍結，後續動作排隊背黑鍋。相關規格：「勾選任務即時回饋」（樂觀更新，維持）、「GUI 勾任務與 CLI 完成語意一致」（檔案語意，逐字不變）。相關者：桌面 app 全體使用者；約束：此機器 GUI 進程 spawn git 首次 ~3 秒屬環境因素，修法必須讓熱路徑不依賴 spawn 速度。

## Goals / Non-Goals

**Goals**
- 任務寫回（單發、批次、拖放、看板排序）不阻塞主執行緒——寫回進行中視窗全程可操作。
- 熱路徑去 git spawn：身分每根快取＋預熱。
- 並發寫回序列化：不遺失更新。
- 樂觀狀態不被在途舊載入覆蓋。

**Non-Goals**
- 環境性 git spawn 慢的根治；讀取型 command async 化；archive／init／runVerb 動詞 async 化；勾選檔案語意變更。

## Decisions

### D1：identity 每根快取

desktop-core 新增 cached_git_identity(root)：以 static Mutex<HashMap<PathBuf, Option<String>>> 快取每專案根的 git 身分，首次呼叫 spawn git 取值後永存（身分變更需重啟 app 才生效——可接受，Spectra 同語意）。set_task_done_at 與 set_all_tasks_at 的完成路徑換用；src-tauri 於 setup 與 switch_root 以 std::thread 背景預熱（失敗靜默，後續首次勾選自行補抓）。
替代案：每次勾選照舊 spawn——正是 6 秒元凶，否決；把身分下放前端傳入——身分屬後端事實，前端不應持有，否決。

### D2：寫入 command 改 async＋spawn_blocking

set_task_done、set_all_tasks、move_task、reorder_card 四個寫入型 command 改 async fn，先於鎖外複製 root，再以 tauri::async_runtime::spawn_blocking 執行 desktop-core 委派——慢寫回佔用執行緒池工作緒，主執行緒照常處理輸入與其他 IPC。async command 借用 State 需回傳 Result（Tauri 約束）：reorder_card 若現為非 Result 簽名則補上。讀取型 command 維持同步（實測 <35ms）。
替代案：sync command 內開執行緒再阻塞等待——主執行緒仍被佔，無效，否決；全部 command async 化——讀取本來就快，動了徒增 diff，違反外科手術原則，否決。

### D3：寫入序列化（全域寫鎖）

desktop-core 新增 static WRITE_LOCK: Mutex<()>；set_task_done_at、set_all_tasks_at、move_task_at、reorder 寫回入口整段持鎖。D2 使寫回並發成為可能，鎖保證依提交順序落盤：慢勾選進行中使用者再取消勾選，取消排隊於其後，最終狀態＝最後一次操作。鎖在 desktop-core 層（而非 tauri 層）——單元測試可直接驗證並發正確性。
替代案：tauri 層 tokio::Mutex——邏輯與測試都該在 core（本專案薄殼慣例），否決；每 change 一把鎖——粒度收益低、複雜度高，否決。

### D4：前端在途載入作廢

RichDetailDrawer 的 handleToggle 於樂觀改寫 tasksMd 前遞增 requestSeq——所有在途 loadAll 回應因序號過期被丟棄，舊 tasks.md 內容不再覆蓋樂觀狀態。既有 pendingWrites 讓路（擋新載入）與世代補載（寫回後重讀磁碟）機制不變；reorder 與批次維持 taskBusy 全鎖語意，不套用樂觀作廢。

### D5：git 子進程不建主控台視窗

引擎的 git spawn 漏斗 crates/speclink-core/src/util.rs 的 git 與 git_raw 改經共用建構函式，Windows 下帶 CREATE_NO_WINDOW（0x08000000）——GUI 進程 spawn 主控台程式時系統不再另開 console 視窗（真實視窗驗證發現：勾選觸發 git status 記 touched 即黑窗閃現）。此處 git 呼叫皆為非互動短命令（config、status），無主控台照常執行；stdout/stderr 走 piped 不受影響，CLI 輸出零變化。
替代案：只在 desktop 層另寫帶旗標的 git 呼叫——與引擎漏斗重複、兩處漂移，否決；接受閃窗——每次勾選閃黑窗屬可見缺陷，否決。

## Implementation Contract

**可觀察行為**
1. 勾選任務後立即取消勾選：視窗全程可操作（無整窗凍結），兩次寫回依序落盤，最終 tasks.md 該任務為未勾選、畫面一致。
2. 同一專案第二次及以後的勾選不再 spawn git 取身分；開工章 started_by 內容與現行逐字一致；touched／meta 語意維持「GUI 勾任務與 CLI 完成語意一致」不變。
3. 並發觸發的兩筆任務寫回（不同任務）皆落盤，無遺失更新。
4. 樂觀勾選後，更早發起的文件載入回應到達時不改變畫面勾選狀態。
5. 勾選（含首次蓋章）過程無主控台黑窗閃現；CLI 下 git 呼叫行為與輸出不變。

**驗收目標**
- cargo test -p speclink-desktop-core --lib 全綠（含新增：快取每根正確、並發寫回序列化案例）。
- npm test -w packages/ui 全綠（含新增：在途載入不覆蓋樂觀狀態案例）；npm test -w apps/desktop 全綠。
- cargo check -p speclink-desktop 通過；真實視窗勾選→立即取消無凍結。

**範圍邊界**
- In scope：apps/desktop/core/src/manage.rs、apps/desktop/src-tauri/src/lib.rs、crates/speclink-core/src/util.rs（僅 git 子進程視窗旗標）、packages/ui/src/components/RichDetailDrawer.tsx 與對應測試。
- Out of scope：speclink-core 引擎的語意（tasks::complete 不動）、CLI 行為與輸出、讀取型 command、其他動詞 async 化、環境性 git 慢根治。

## Risks / Trade-offs

- [身分快取過期（使用者中途改 git config）] → 重啟 app 即更新；開工章身分本非高頻變動，可接受。
- [async 化後寫回與讀取交錯，讀到寫回中間態] → 寫回單檔單次寫（既有行為）＋D4 作廢在途回應＋世代補載收斂；讀取本身無鎖維持快路徑。
- [全域寫鎖使批次寫回期間單發寫回等待] → 等待即序列化的本意；批次僅秒級，且前端 taskBusy 已鎖工具列。
- [Tauri async command 借用 State 的簽名約束] → 鎖外先複製 root 再 move 進 spawn_blocking，State 不進閉包。

## Migration Plan

無資料遷移。純行為修正，隨一般建置發佈；回滾即還原 commit 重建。

## Open Questions

（無）

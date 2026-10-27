## Problem

桌面 app 勾選任務後數秒內整個視窗凍結、無法操作；使用者感知上凍結發生在「取消勾選」（該動作排隊在後、被前一勾選拖住），實際元凶是勾選的後端寫回。實測時間軸：set_task_done done=true 在 Rust 主執行緒耗時 6.8 秒（其中 git_identity 兩次 git spawn 佔 6.2 秒——此機器 GUI 進程 spawn git 首次約 3 秒），期間所有 IPC 與視窗輸入排隊，緊接的取消勾選（本身僅 5.5ms）連同整窗一起凍結。

## Root Cause

三個因素疊加：

1. 勾選（done=true）走引擎完成路徑，每次呼叫 speclink_core::util::git_identity 逐次 spawn 兩個 git 子進程（user.name、user.email）；此機器上 GUI 進程 spawn git 極慢（防毒掃描），單次勾選累計 6+ 秒。
2. Tauri 非 async 的 #[tauri::command] 在主執行緒同步執行——任何慢 command 都會凍結整個視窗（輸入、後續 IPC 全部排隊）。
3. 前端樂觀更新只覆蓋視覺；慢寫回進行中若有更早發起的文件載入回應到達，會以舊內容覆蓋樂觀狀態（潛在閃爍，現行 pendingWrites 只擋新載入、不擋在途回應）。

## Proposed Solution

1. **git 身分快取**（apps/desktop/core/src/manage.rs）：新增每專案根一次的 git 身分快取，set_task_done_at 與 set_all_tasks_at 的完成路徑換用；app 啟動與切換專案時於背景執行緒預熱——熱路徑不再逐次 spawn git。
2. **寫入型 command 非阻塞化**（apps/desktop/src-tauri/src/lib.rs）：set_task_done、set_all_tasks、move_task、reorder_card 改 async fn 並以 spawn_blocking 移至執行緒池——主執行緒與視窗互動不再被寫回堵塞。
3. **寫入序列化**（apps/desktop/core/src/manage.rs）：desktop-core 的任務寫回入口取全域寫入鎖——並發寫回依序落盤，杜絕慢寫回與後續寫回互相覆蓋的遺失更新。
4. **前端在途載入作廢**（packages/ui/src/components/RichDetailDrawer.tsx）：樂觀勾選時遞增載入序號，作廢所有在途載入回應——舊內容不再覆蓋樂觀狀態。
5. **git 子進程視窗抑制**（crates/speclink-core/src/util.rs）：引擎的 git spawn 漏斗（git、git_raw）於 Windows 帶 CREATE_NO_WINDOW 建構——GUI 進程 spawn 主控台程式時不再閃現黑色 console 視窗（真實視窗驗證時發現：勾選觸發 git status 記 touched 即閃窗）。CLI 情境下這些 git 呼叫皆為非互動短命令，無主控台照常執行。

## Non-Goals

- 不根治此機器 git spawn 慢的環境因素（防毒行為，非本專案可控）。
- 讀取型 command（清單、文件）維持同步——實測皆 <35ms，不動。
- archive、init、runVerb 等其他動詞的 async 化不在此刀（使用頻率低、另議）。
- 不改變勾選的檔案語意——touched 記錄、開工章行為維持規格「GUI 勾任務與 CLI 完成語意一致」逐字不變。
- 引擎 git 呼叫的內容與輸出不變——僅子進程視窗建構旗標；CLI 輸出零影響。

## Success Criteria

- 勾選後立即取消勾選：全程視窗可操作（可捲動、可點擊），無整窗凍結；最終 tasks.md 為未勾選（寫回依提交順序落盤）。
- 同一專案連續勾選多任務：git 身分僅首次（或預熱時）取得，後續勾選不再逐次 spawn git；started_by 內容與現行一致。
- 樂觀更新期間到達的舊載入回應不覆蓋畫面勾選狀態。
- 勾選過程無主控台黑窗閃現。
- npm test -w packages/ui、npm test -w apps/desktop、cargo test -p speclink-desktop-core --lib 全綠；真實視窗驗證無凍結。

## Impact

- Affected specs: desktop-app（新增需求：任務寫回非阻塞且序列化）
- Affected code:
  - Modified: apps/desktop/core/src/manage.rs、apps/desktop/src-tauri/src/lib.rs、crates/speclink-core/src/util.rs、packages/ui/src/components/RichDetailDrawer.tsx、packages/ui/src/__tests__/richDrawer.test.tsx
  - New: （無）
  - Removed: （無）

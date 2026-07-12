## 1. 引擎反向動詞（speclink-core）

- [x] 1.1 撰寫 uncomplete 的紅燈單元測試：於 crates/speclink-core/src/tasks.rs 的 #[cfg(test)] 新增案例，鎖定「任務取消勾選動詞」的引擎語意——只翻目標行 [x]→[ ] 且保留縮排、`- ` 與 `* ` 兩種 bullet 風格、檔尾換行；already（已是未勾選）回報旗標且零寫入；task id 超界回錯誤且零寫入；不寫 touched 記錄、不動 meta（對照 design「D1 引擎反向動詞：uncomplete 為獨立函式、零側效」）。驗證：cargo test -p speclink-core 此時紅燈（新案例失敗、既有案例綠燈）
- [x] 1.2 實作 tasks 模組的 uncomplete 函式（crates/speclink-core/src/tasks.rs）：輸入 store、change 名、1-based task id，經 store 讀寫 tasks.md 完成純狀態翻轉並回傳描述與 already 旗標。驗證：cargo test -p speclink-core 全綠
- [x] 1.3 重構：檢視 mark_done 與 uncomplete 的行掃描邏輯，若兩者出現整段重複的逐行狀態機則抽出共用 helper，否則維持現狀。驗證：cargo test -p speclink-core 維持全綠、無新增 public API

## 2. CLI 子指令 fs 模式（speclink-cli）

- [x] 2.1 撰寫 task undone 的紅燈整合測試：新增 crates/speclink-cli/tests/task_undone.rs，鎖定 fs 模式契約——成功時人眼輸出綠色 ✓ 與 Task <id> marked as not done: <desc>（--no-color 無 ANSI）、exit code 0；--json 輸出 compact 單行且鍵依序 change、status、task_desc、task_id、status 值為 undone；已是未勾選輸出 Task <id> is already not done 且 exit code 非 0；task id 非數字、id 超界、tasks.md 不存在的錯誤訊息與 task done 對應情境一致；全程 .speclink/ 無新增 touched 記錄且 meta 開工標記不變。驗證：cargo test -p speclink-cli --test task_undone 紅燈
- [x] 2.2 實作 CLI 子指令與分派使 2.1 綠燈：crates/speclink-cli/src/main.rs 新增 TaskCommands::Undone（位置參數 task_id、旗標 --change 與 --json），crates/speclink-cli/src/commands.rs 的 cmd_task 新增分派——前置檢查順序與錯誤文字重用 done 路徑（對照 design「D2 CLI 子指令 task undone 與 done 全面對稱」）。驗證：cargo test -p speclink-cli 全綠
- [x] 2.3 既有輸出回歸確認：task done 與其他既有指令的人眼與 --json 輸出零變化。驗證：cargo test --workspace 全綠（含 remote_write_path 的 task done 鍵序斷言、task_done_stamps 開工章案例），speclink task done --help 與 speclink task undone --help 均正確顯示

## 3. remote 攔截與 client（speclink-remote、speclink-cli）

- [x] 3.1 撰寫 remote 模式的紅燈整合測試：於 crates/speclink-cli/tests/remote_write_path.rs 新增 mock server 案例——POST /changes/demo/tasks/3/undone、request body 為空 JSON 物件（無 touchedFiles 欄位）、成功回應（camelCase 欄位含 alreadyUndone）翻譯為與 fs 模式位元一致的人眼與 --json 輸出、alreadyUndone 為 true 時以 already not done 錯誤結束；此為「動詞契約的涵蓋面與 payload 形狀」對 tasks 勾選／取消勾選的 remote 形狀一致要求（對照 design「D3 remote 攔截與 client endpoint 對稱新增」）。驗證：cargo test -p speclink-cli --test remote_write_path 紅燈
- [x] 3.2 實作 remote 路徑使 3.1 綠燈：crates/speclink-remote/src/client.rs 新增 task_undone 呼叫（POST /changes/{name}/tasks/{taskId}/undone），crates/speclink-cli/src/remote_commands.rs 新增對應分派與輸出翻譯，crates/speclink-cli/src/commands.rs 的 undone 分支接上 remote 攔截。驗證：cargo test -p speclink-cli --test remote_write_path 全綠

## 4. 桌面取消勾選收斂（speclink-desktop-core）

- [x] 4.1 擴充桌面行為鎖定測試：apps/desktop/core/src/manage.rs 既有「單發切換只動目標行」案例維持不變，另新增 `* ` bullet 任務的取消勾選案例（現行桌面 regex 只認 `- `，delegate 引擎後 `* ` 也正確翻轉——此案例於 delegate 前紅燈）。驗證：cargo test -p speclink-desktop-core 紅燈（僅新案例失敗）
- [x] 4.2 實作 delegate 使 4.1 綠燈：set_task_done_at 的 done=false 分支改建 core context 呼叫引擎 uncomplete，刪除 regex 行編輯；already 維持冪等成功（對照 design「D4 桌面取消勾選收斂 delegate 到引擎」）。驗證：cargo test -p speclink-desktop-core 全綠

## 5. apply 技能教學與 golden 同步

- [x] 5.1 apply 技能新增取消勾選指引：在勾選指引旁補「誤勾或實作回退時執行 speclink task undone --change <name> <task-id>，SHALL NOT 直接編輯 tasks.md」，三處同步——crates/speclink-core/assets/skills/apply.md、.claude/skills/speclink-apply/SKILL.md、.agents/skills/speclink-apply/SKILL.md（對照 design「D5 apply 技能教學與內嵌資產三處同步」）。驗證：三檔均含 task undone 指引（內容審視），cargo test -p speclink-core --test skill_verbization 綠燈（無直接讀檔指示）
- [ ] 5.2 乾淨樹再生 render golden：先以 git status 確認工作樹乾淨（僅含本變更已提交內容），執行 UPDATE_GOLDEN=1 cargo test -p speclink-core --test render_golden 再生快照，審視 diff 僅含 undone 指引相關變更。驗證：cargo test -p speclink-core --test render_golden 綠燈

## 6. 全量驗證

- [ ] 6.1 端到端冒煙與全量測試：cargo test --workspace 全綠；於臨時示範專案執行 speclink task done 1 --change <demo> 後接 speclink task undone 1 --change <demo>，git diff 顯示 tasks.md 回復原狀且 .speclink/ 的 touched 記錄與 meta 開工標記維持 done 當下狀態（取消勾選不回滾側效）。驗證：上述指令 exit code 依序 0、0，diff 內容符合預期

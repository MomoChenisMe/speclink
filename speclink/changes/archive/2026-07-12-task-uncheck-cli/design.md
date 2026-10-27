## Context

任務勾選目前只有單向動詞：引擎的完成函式做 `[ ]`→`[x]` 翻轉並帶側效（touched 記錄、首次完成蓋 started_at 開工章），CLI 曝露 `speclink task done`（fs 模式直呼引擎；remote 模式經 CLI 攔截層呼叫 server 的 done endpoint）。反向操作沒有動詞：agent 在 apply 中要取消勾選只能直接編輯 tasks.md，繞過儲存抽象；桌面 app 的單發取消勾選則在 `apps/desktop/core/src/manage.rs` 以 regex 行編輯自行實作，未進引擎。本設計把「取消勾選」補成引擎動詞，CLI、remote、桌面三個入口共用。

相關既有裁定：desktop-task-interactions design D1 已確立「取消勾選為純行編輯、無任何側效」。進行中的 engine-typed-core 變更將把動詞收進 typed runtime，但明言不擴增動詞覆蓋、remote 攔截路徑維持現狀——本變更以現行「CLI handler 直呼引擎」形式先落地。

## Goals / Non-Goals

**Goals:**

- 引擎（`speclink-core`）擁有取消勾選的唯一實作，CLI 與桌面共用。
- CLI 新增 `speclink task undone <task-id>`，fs 與 remote 兩模式輸出形狀一致，並與 `task done` 呈現對稱。
- speclink-apply 技能教會 agent 使用此動詞，取代直接編輯 tasks.md。

**Non-Goals:**

- 不回滾 touched 記錄與 started_at 開工章（trace 是歷史）。
- 不動桌面批次動詞（全部標完成／全部取消）的實作歸屬。
- 不改 `task done` 既有行為與輸出。
- 不做 typed runtime 收編（engine-typed-core 負責）。

## Decisions

### D1 引擎反向動詞：uncomplete 為獨立函式、零側效

`speclink-core` 的 tasks 模組新增 `uncomplete`：輸入 store、change 名、1-based task id；把目標任務 `[x]`→`[ ]` 後經 store 寫回，回傳任務描述與 already 旗標（已是未勾選時零檔案效果）。不接觸 touched 記錄、不接觸開工章，因此簽名不需要 workspace——與 `complete`（需 workspace 做 touched/git）刻意不同。行處理沿 `mark_done` 的既有模式：保留縮排、bullet 風格（`- `/`* `）與檔尾換行，跨平台不假設換行風格。

- 替代方案：在 `complete` 加方向參數單函式雙用——否決：兩個方向的側效完全不同（一有一無），合流會讓每條路徑都揹上對方的條件分支，違反禁止過度設計。
- 邊界：流程邏輯與檔案格式處理全在 core；ANSI 色彩與訊息呈現全在 cli。

### D2 CLI 子指令 task undone 與 done 全面對稱

`speclink-cli` 的 task 子指令新增 `undone`，位置參數 `<task-id>`、旗標 `--change <name>` 與 `--json`，無 stdin。前置檢查與錯誤訊息重用 done 的既有文字與順序（tasks.md 存在性檢查先於 id 驗證；Invalid task ID／Task ID must be >= 1／Task N not found (total: M)）。呈現：

- 成功（人眼）：綠色 ✓ 加 `Task <id> marked as not done: <desc>`，尊重 `--no-color`；exit code 0。
- 成功（--json）：compact 單行，keys 依字母序 `change`、`status`、`task_desc`、`task_id`，其中 `status` 為 `"undone"`——與 done 的 payload 同形狀僅 status 值不同（此 payload 為 Spectra parity 的 snake_case 特例，undone 沿用以保對稱）。
- 已是未勾選：exit code 非 0，stderr `Task <id> is already not done`（對稱 done 的 already done）。

替代方案：`task done --undo` 旗標——已於討論否決（對稱動詞對 agent 更直覺、--help 可發現性更好）。訊息用語 `marked as undone`／`already undone` 亦否決：英文語意含混（undone 可誤讀為「曾被取消過」），人眼訊息用 not done、機器欄位用 undone 動詞名。

### D3 remote 攔截與 client endpoint 對稱新增

`speclink-remote` client 新增 `task_undone`：`POST /changes/{name}/tasks/{taskId}/undone`，body 為空 JSON 物件（取消勾選不記 touched，故無 touchedFiles 欄位）；預期回應為 camelCase 欄位（change、taskId、taskDesc、status、alreadyUndone、tasksVersion），與 done endpoint 回應對稱。CLI 的 remote 攔截層新增對應分派，把回應翻譯成與 fs 模式完全相同的人眼與 `--json` 輸出；非 2xx 與 409 沿既有通用翻譯（version_conflict／change_busy 等訊息不另造）。server 端實作不在本 repo，以 mock server 整合測試固定契約形狀。

- 替代方案：remote 模式改走 artifact 讀寫（GET tasks.md → 本地翻轉 → PUT If-Match）——否決：done 已有專屬 endpoint，混用兩種粒度會讓 server 無法維持任務語意的單一入口，且 client 要重複 tasks.md 解析。

### D4 桌面取消勾選收斂 delegate 到引擎

`apps/desktop/core/src/manage.rs` 的單發取消勾選分支（done=false）刪除自製 regex 行編輯，改建 core context 後呼叫 D1 的 `uncomplete`；already（已是未勾選）維持冪等成功（引擎保證零檔案效果），與勾選分支對 already 的處理對稱。可觀察行為不變，既有桌面測試（單發切換只動目標行）繼續通過。

- 替代方案：桌面維持自製行編輯——否決：格式處理與引擎分岔（桌面 regex 只認 `- ` bullet，引擎認 `- `/`* ` 兩種），且「單一協作點」宣稱持續不成立。

### D5 apply 技能教學與內嵌資產三處同步

speclink-apply 技能在勾選指引旁新增取消勾選指引：誤勾或實作回退時執行 speclink task undone（動詞表述，不指示直接編輯 tasks.md——維持技能資產「不含直接讀檔指示」的既有掃描約束）。同步三處：內嵌資產（`crates/speclink-core/assets/skills/apply.md`）、repo 技能實例（`.claude/skills/speclink-apply/SKILL.md`、`.agents/skills/speclink-apply/SKILL.md`）；render golden 快照於乾淨樹以 UPDATE_GOLDEN 模式再生並審視 diff。

- 替代方案：只改 repo 技能實例不動內嵌資產——否決：speclink init/update 會用內嵌資產覆寫生成，三處不同步即回歸。

## Implementation Contract

**行為**：`speclink task undone <task-id> [--change <name>] [--json]`

| 情境 | stdout / stderr | exit code | 檔案效果 |
| ---- | --------------- | --------- | -------- |
| 目標任務已勾選 | stdout：綠色 ✓ `Task <id> marked as not done: <desc>`（--json 時輸出 compact 單行 payload） | 0 | tasks.md 目標行 `[x]`→`[ ]`，其餘位元不動；不寫 touched、不動 meta |
| 目標任務已是未勾選 | stderr：`Task <id> is already not done` | 非 0 | 無 |
| task id 非數字 | stderr：`Invalid task ID '<輸入>': must be a number` | 非 0 | 無 |
| task id < 1 | stderr：`Task ID must be >= 1` | 非 0 | 無 |
| task id 超界 | stderr：`Task <id> not found (total: <總數>)` | 非 0 | 無 |
| tasks.md 不存在 | stderr：`tasks.md not found for change '<name>'` | 非 0 | 無 |

省略 `--change` 時的變更解析規則與 done 相同（單一進行中變更自動解析）。

**`--json` payload**（compact 單行、鍵序固定）：`{"change":"<name>","status":"undone","task_desc":"<desc>","task_id":"<id>"}`

**remote 契約**：`POST /changes/{name}/tasks/{taskId}/undone`，request body `{}`；成功回應含 camelCase 欄位 change／taskId／taskDesc／status／alreadyUndone／tasksVersion；CLI 輸出（人眼與 --json）與 fs 模式位元一致；alreadyUndone 為 true 時 CLI 以 already not done 錯誤結束；非 2xx 沿既有語義化翻譯。

**桌面**：勾選框取消勾選走引擎 uncomplete；對已是未勾選維持冪等成功；bullet 風格與縮排保留。

**驗收**：

- core 單元測試：只翻目標行、保留 bullet 風格與檔尾換行、already 旗標零寫入、超界報錯零寫入。
- CLI 整合測試：fs 模式成功／already／錯誤輸入的 stdout、stderr 與 exit code；`--json` 鍵序與值。
- remote 整合測試（mock server）：endpoint 路徑與 body 形狀、fs parity 輸出、alreadyUndone 翻譯。
- 桌面測試：既有單發切換測試維持綠燈（取消勾選改走引擎後行為不變）。
- 技能資產：skill_verbization 掃描維持綠燈（無直接讀檔指示）；render golden 於乾淨樹再生後 `cargo test -p speclink-core --test render_golden` 綠燈。
- 回歸對照：`task done` 與其他既有指令輸出零變化。

**範圍邊界**：in scope＝上述動詞、三入口、技能資產與測試；out of scope＝touched／開工章回滾、批次動詞歸屬、typed runtime 收編、server 端實作。

## Risks / Trade-offs

- [render golden 於 dirty 樹再生把未提交狀態烙進快照，main 長期紅燈（曾發生）] → 任務明定「乾淨樹再生」為獨立步驟，再生後審視 diff 才提交。
- [undone 的 `--json` 誤用 camelCase 或鍵序不同，與 done 不對稱] → 測試直接斷言鍵序陣列（與 remote_write_path 既有 task done 測試同法）。
- [remote server（外部 repo）尚未實作 undone endpoint，remote 模式下 404] → CLI 沿既有非 2xx 翻譯輸出「資源不存在」語義化訊息，不崩潰；契約以 mock 測試先行固定，server 側依契約跟進。
- [engine-typed-core 同時改 commands.rs／tasks 模組造成合併衝突] → 本變更範圍小先落地；typed runtime 收編 undone 屬該變更的動詞清單擴充，屆時由其 drift／ingest 吸收。
- [跨平台：Windows CRLF 與縮排差異] → uncomplete 沿 mark_done 的逐行處理與檔尾換行保留邏輯，core 測試涵蓋 `* ` bullet 與縮排案例。

## Migration Plan

純新增動詞，無資料遷移；回退即 revert 對應 commit（不留殘置狀態）。

## Open Questions

（無）

## Context

「完成任務」的語意目前散落三處且不一致：CLI task done（crates/speclink-cli/src/commands.rs 的 cmd_task）經引擎 core::tasks::mark_done 勾章、寫回、記 touched-files，但不蓋開工章；桌面 GUI 勾任務（apps/desktop/core/src/manage.rs 的 set_task_done_at）以自有行編輯勾章，不記 touched、不蓋章；開工章（started_at/started_by/started_with）只由 apply 流程顯式執行 speclink in-progress add 時經 core::inprogress::add 蓋入 change meta。看板欄位派生（packages/ui/src/stage.ts 的 changeStage）以「meta 有 started_at」為進行中的唯一依據，於是任何未經 in-progress add 的進度（GUI 勾選、直接編輯 tasks.md、git pull 拉進他人變更）都讓卡片錯誤地停留「提案中」。

前置事實：inprogress::add 已具備冪等、首章保留、YAML 注入清洗與「不能歸屬即缺席」的身分規則（change-lifecycle spec 對其指令面有 parity 凍結需求，但引擎函式的呼叫端不受限）；CLI task done 的人眼與 --json 輸出對齊 Spectra 基線，是回歸保護對象。

## Goals / Non-Goals

**Goals:**

- 「完成任務」四合一語意（勾章、寫回、touched-files 記錄、首次完成蓋開工章）由引擎單點提供，CLI 與桌面 GUI 共用。
- CLI task done 的人眼輸出、--json、錯誤訊息與順序、exit code 位元級不變。
- 看板派生涵蓋所有寫入路徑：無章而有任務進度的 change 顯示於進行中欄。
- 開工歸屬維持誠實：詳情抽屜的開工列只在 meta 有 started_at 時顯示；缺席的歸屬不由任何機制事後補造。

**Non-Goals:**

- 不做事後補章：watcher 偵測補章、list 讀取路徑寫回、git hook 對帳皆否決（偽造歸屬與日期、讀操作寫檔）。
- 桌面的取消勾選與拖曳排序不整併進引擎（引擎無 uncheck/move 原語，屬 YAGNI）；此二操作不蓋章、不記 touched。
- in-progress add 指令面（輸出、exit code、冪等、對不存在 change 靜默成功）零變動。
- drift/verify 對「有進度無章」的診斷提示——deferred。
- web server／remote 端點不在本刀：server 內嵌引擎，經同一協作函式自動繼承語意。

## Decisions

### D1 引擎任務完成協作函式（core::tasks 單點四合一）

speclink-core 的 tasks 模組新增協作函式（暫名 core::tasks::complete），單點完成：mark_done 勾章 → 寫回 tasks.md → touched-files 記錄（git dirty 中未被先前任務認領者；無新檔不追加——沿現行 CLI 語意）→ 呼叫 inprogress::add 蓋首章（冪等、首章保留）。身分參數（identity/agent）由呼叫端供給，遵循「不能歸屬即缺席」。
替代方案：桌面層自行組合四步——CLI 路徑（agent 主路徑）的洞仍在，且兩處組合必然漂移，否決；GUI 以子行程呼叫 CLI exe——版本漂移、remote_ctx 意外路由、Windows 主控台閃現、錯誤處理退化為解析文字，否決。本決策把流程語意收進 storage 解耦的引擎層（經 Store trait 讀寫），與「規格驅動引擎」方向一致。

### D2 CLI task done 改薄呼叫端且輸出凍結

cmd_task 的 done 分支改呼叫 D1 函式，僅保留呈現層職責：錯誤訊息文字與檢查順序（tasks.md 缺失先於 id 驗證、already done 於勾章判定後）、人眼／--json 輸出、exit code 全部維持現狀。已 done 時維持 bail（錯誤結束），不寫任何檔案。
替代方案：CLI 路徑不動、只改 GUI——desktop-discussion-board 2/14 無章的實證正是 CLI/skill 路徑漏蓋，否決。

### D3 桌面勾選走協作函式且冪等寬容

set_task_done_at 的 done=true 路徑改呼叫 D1 函式：ordinal（1-based checkbox 行序）即引擎 task id（兩者皆以 checkbox 行計數）；identity 沿 CLI 同源 git 身分、agent 缺席。與 CLI 的差異僅一處：任務已完成時桌面視為冪等成功（no-op，不寫檔、不蓋章），因 GUI toggle 語意下重複 done=true 只可能來自競態，不應以錯誤打斷使用者。done=false（取消勾選）與 move_task_at 維持既有桌面行編輯，不觸發任何 D1 行為。
替代方案：桌面沿用自有行編輯再手動補蓋章——touched-files 記錄仍缺、與 CLI 語意漂移，否決。

### D4 看板派生加入任務進度（顯示與歸屬分離）

changeStage 優先序改為：任務全完成（總數>0）＝已就緒 ＞ meta 有 started_at 或完成數>0＝進行中 ＞ 其餘＝提案中。判定矩陣新增「無章、3/28 → 進行中」列。詳情抽屜開工歸屬列維持「meta 有 started_at 才顯示」——派生管顯示正確性（由構造涵蓋手改 tasks.md、agent 直改、git pull 等繞道），蓋章管事件歸屬（只由行動當下的工具誠實記錄）。
替代方案：watcher 偵測 tasks.md 變化自動補章——僅 app 執行中有效、讀寫回饋環、偵測日≠開工日且本機身分≠動手者（偽造歸屬），否決；引擎 list 讀取時自我修復寫回——讀操作產生檔案副作用，破壞唯讀期望與 CI/git status，否決。

### D5 歸屬參數縫維持開放

D1 函式的 identity/agent 參數：本刀 CLI 與桌面皆傳 git 身分＋agent 缺席（與現行 in-progress add 指令一致）。started_with 的供給縫留給後續 agent 通道（desktop-acp-agent、web-agent-channel）屆時填入，本刀不預設。
替代方案：桌面硬填 started_with（如 "desktop"）——started_with 語意是 agent 歸屬而非通道名，硬填製造假紀錄，否決。

## Implementation Contract

**行為（使用者可觀察）**

- agent 或人執行 speclink task done 完成某 change 的第一個任務後：tasks.md 該任務成 [x]、touched 記錄照舊、該 change 的 .openspec.yaml 新增 started_at（git 身分可得時含 started_by）；指令輸出與 exit code 與現行完全相同。
- 桌面看板勾下某 change 的第一個任務後：上述同一組檔案效果發生；卡片於同一次 refresh 移入進行中欄；詳情抽屜出現開工列。
- 手動編輯 tasks.md 勾掉任務（不經任何工具）後看板刷新：卡片顯示於進行中欄；meta 不變、抽屜無開工列。
- 取消勾選、拖曳排序：檔案效果僅 tasks.md，meta 永不變動。

**介面／資料形狀**

- 引擎：core::tasks 新增公開協作函式，輸入為 Store、Workspace、change 名、1-based 任務序號、identity Option、agent Option；輸出為完成結果（含任務描述與 already 旗標）或錯誤。不新增任何 serde 結構；meta 寫入沿 inprogress::add 的 append-only 文字路徑（讀既有檔案能力不變）。
- CLI：子指令 task done 的旗標（--change、--json）、stdin（不讀）、exit code（成功 0、already/缺件非 0）不變。
- 桌面：Tauri command set_task_done 的 IPC 形狀（change、ordinal、done）不變；前端 adapter 與 store 無介面變更。
- 前端：changeStage 輸入型別 ChangeItem 不變，僅判定邏輯與測試矩陣更新。

**失敗模式**

- 任務序號越界、tasks.md 缺失：維持現行各端錯誤訊息。
- 蓋章寫入失敗（meta 不可寫）：勾章已寫回、蓋章失敗——D1 函式回傳錯誤，CLI 照現行錯誤路徑呈現、桌面 reject 附訊息；tasks.md 的勾選不回滾（勾章與蓋章非原子，接受——重勾或下次完成會再嘗試首章）。
- git 身分不可得：started_by 缺席，started_at 照蓋（沿 inprogress::add 既有行為）。

**驗收準則**

- cargo test -p speclink-core：D1 函式的紅綠測試（首次完成蓋章＋touched；already 不寫檔；身分缺席規則；meta 既有欄位逐字元保留）。
- cargo test -p speclink-cli 與 parity／twin 對照：task done 輸出位元級不變；檔案樹差異僅 .openspec.yaml 新增 started_*（基線刻意更新並記錄）。
- cargo test -p speclink-desktop：set_task_done_at done=true 蓋章＋touched、done=false 與 move 不觸發、already 冪等成功。
- npm test -w packages/ui：changeStage 新矩陣（含「無章 3/28 → in-progress」「無章 0/28 → proposed」「章在 0 完成 → in-progress」）。
- 真實視窗驗證：GUI 勾首任務 → 卡片移進行中＋抽屜開工列；手改 tasks.md → 卡片移進行中＋抽屜無開工列。

**範圍邊界**

- In scope：crates/speclink-core/src/tasks.rs、crates/speclink-cli/src/commands.rs 的 task done 分支、apps/desktop/core/src/manage.rs 的 set_task_done_at、packages/ui/src/stage.ts 及其測試。
- Out of scope：in-progress add 指令、watcher、drift/verify、web server 端點、桌面 uncheck/move 的實作位置、stage 派生以外的任何 UI 變更。

## Risks / Trade-offs

- [ordinal 與引擎 task id 計數偏差（兩套 checkbox 行判定不一致）] → 以同一 tasks.md fixture 的對齊測試釘死：desktop ordinal N 與引擎 task id N 必指同一任務，含巢狀縮排與非 checkbox 行混排情形。**已知邊界（verify 實證確認）**：`* [ ]` 星號 bullet 與空描述 checkbox 引擎計入、前端與桌面行編輯不計——手改或匯入含這類行的 tasks.md 時 ordinal 會錯位（speclink 自產文件一律 dash＋有描述，不觸發）。收斂三方 checkbox 判定為單一規則屬後續 change。
- [雙沙盒檔案樹對照因 meta 蓋章出現差異] → 屬預期行為變更：更新自我基線並在變更記錄註明「task done 新增 meta 檔案效果」。
- [GUI 勾選當下 git dirty 集合與該任務無關（touched 錯誤歸屬）] → 與 CLI 現行語意一致（僅記未被先前任務認領的檔案）；使用者已裁定接受，不另設過濾。
- [勾章與蓋章非原子（蓋章失敗留下已勾未蓋狀態）] → D4 派生使顯示不受影響（有進度即進行中）；下次任一完成會再嘗試首章；不引入回滾或鎖（禁過度設計）。
- [併發寫 meta（GUI 與外部 CLI 同時首次完成）] → inprogress::add 首章冪等且兩端寫入內容等價（同日、同 git 身分），競態結果無語意差異；不加檔案鎖。
- [跨平台] → 無新平台面：git 互動沿既有 git_changed_files／git 身分工具，路徑經 Store trait。

## Context

remote-data-source 的三類覆蓋矩陣把 validate/analyze、刪除變更、任務拖排歸入 (c) 明確不支援——當時 server 沒有端點。引擎接縫其實已備齊：Command enum 已有 Validate／Analyze／Discard 變體與 typed outcome（crates/speclink-core/src/command/mod.rs）、core discard 是 store-trait 編排（guard→unlink→delete→report）、host BridgeStore 已實作 delete_change（逐文件 UoW 刪除）、server 的 verb::run 在 commit 後統一 notify 事件（crates/speclink-server/src/verb.rs）。桌面的任務搬移＋重編號行編輯邏輯位於 apps/desktop/core/src/manage.rs（move_task_at＋renumber_task_prefixes＋edit_tasks），是桌面專用 crate、server 用不到。CLI 側 cmd_validate／cmd_analyze 無 remote 分流（open_project 直接開本地）、remote_discard 明確 bail 拒絕。

範圍內：server 三組動詞端點、protocol DTOs、speclink-remote client 四方法、CLI remote 分流三動詞、桌面 remote 解鎖四操作、move 行編輯邏輯遷入 speclink-core。範圍外：看板拖排（獨立 CAS board resource 設計，屬 remote-board-order 刀）、桌面本地刪除語意改造（現況無 guard 直刪不動）、任務 stable ID 定址 wire、changeCapabilities／changeMeta 讀取面補洞。

## Goals / Non-Goals

- Goals：remote workspace 的 validate/analyze/刪除/任務拖排四操作與本地同語意可用；三個入口（desktop、CLI、任何 protocol client）共用同一組端點；capability 停用清單縮減至僅剩看板拖排。
- Non-Goals：不新增任何 client 端近似實作；不改本地路徑可觀察行為（move 邏輯遷 core 屬等價重構）；不動 UI 元件層（解鎖由 capability 管線自動發生）。

## Decisions

### 決策 1：三組端點全部經既有 Command gateway，不另寫流程

GET /changes/{name}/validate 與 GET /changes/{name}/analyze 沿 GET /changes/{name}/drift 的唯讀衍生查詢形狀：handler 呼叫 verb::run 帶 Command::Validate（item=該 change、strict=false）／Command::Analyze，typed outcome（ValidationResult 清單／AnalyzeReport）序列化為 DTO 回傳、附 scope ETag。DELETE /changes/{name} 走 Command::Discard、POST /changes/{name}/tasks/move 走新 Command::TaskMove——寫入動詞經既有 bridge UoW commit，verb::run 的 commit 後 notify 讓 SSE invalidate 自動發生，不加新事件機制。替代案：server 端 handler 手寫各流程——重寫 lifecycle 正是 roadmap「不建議的做法」明文（不讓各入口各自重寫 lifecycle），否決。

### 決策 2：validate 端點固定單 change；CLI 的聚合模式由 client 組合

端點只提供單 change 形狀（與桌面 runVerb 的消費一致、與 drift 端點對稱）。CLI remote 的 speclink validate 無參數／--all／--changes 聚合語意由 remote 分流以「先 list 再逐 change 打端點」組合達成，聚合輸出（人眼與 --json 的多筆 results 形狀）與 fs 模式一致。替代案：端點加 all 參數在 server 端聚合——多一個 wire 形狀只服務 CLI 一個消費者，且逐 change 端點本來就要有，否決（與 setAllTasks 逐筆組合的既有先例同理）。

### 決策 3：DELETE 語意＝Discard 全語意，force 為端點參數，兩入口各保既有行為

DELETE /changes/{name} 帶 force 布林（query 參數、預設 false）。server 端執行 core discard 全語意：fail-closed meta 檢查、started-work guard（force=false 時拒絕、錯誤 reason 機器可判為需要 force 的拒絕）、來源討論 unlink、刪除、touched 記錄清理。桌面 remote 刪除固定帶 force=true——桌面本地 delete 現況即無 guard 直刪（確認對話框在 UI 層），remote 若突然 guard 會是同一顆按鈕兩種模式行為分家；CLI discard 傳使用者的 --force 旗標——與本地 discard 的 guard 行為 parity。替代案 A：端點無 force 一律 guard——CLI --force 語意斷裂，否決。替代案 B：桌面也走 guard——與本地桌面行為分家，否決。附帶效益：remote 刪除比本地桌面刪除更完整（有討論 unlink），本地桌面直刪不 unlink 屬既有債、本刀不動（Non-Goals）。

### 決策 4：TaskMove 為新 Command 變體，行編輯語意遷入 speclink-core

Command::TaskMove { change, from, to, before }（from/to 為 1-based checkbox ordinal、before 為 Option 側別，鏡射 UI moveTask 簽名），outcome 回搬移後的任務描述。move＋重編號整段語意（方向推斷、明確側別、群組邊界規則、renumber 只改「數字.數字」前綴、保留檔尾換行）自 apps/desktop/core/src/manage.rs 遷入 speclink-core 的 tasks 模組成為 store-trait 函式；桌面 move_task_at 改薄呼叫同一 core 函式，可觀察行為零改動、桌面既有 move 測試不修改全綠（等價重構的驗收）。併發邊界：index 定址在他人同時編輯 tasks.md 時可能位移——與既有 ordinal 勾選路徑同等的競態暴露，server 端越界即拒絕、SSE invalidate 數秒內矯正視圖；不採 If-Match（scope ETag 隨任何文件寫入前進，對高頻任務操作誤衝突率過高）。替代案：stable ID 定址 wire——UI 層未暴露任務 stable ID（moveTask 簽名即 index），為此翻修 UI 任務管線超出補洞範圍，記為 deferred。

### 決策 5：寫入動詞 editor role 限定，唯讀動詞 reader 可用

DELETE 與 tasks/move 沿 server-policy-write 的 role 檢查先例：reader 收 403、reason 機器可判；validate/analyze 為唯讀衍生查詢，reader 可用、不發事件。覆蓋矩陣：capability 描述的 deleteChange/moveTask 對 reader 呈現停用（既有 policyWrite 同模式——handshake 已帶 role），validate/analyze 對全 role 翻真。

### 決策 6：CLI remote 分流沿 cmd_discard 既有模式，渲染共用本地路徑

cmd_validate／cmd_analyze 開頭加 remote_ctx() 分流（與 cmd_discard 現有形狀一致）；remote 分支把端點 DTO 反序列化回與本地相同的 ValidationResult／AnalyzeReport 型別後，走同一 render 函式——人眼與 --json 輸出形狀與 fs 模式逐位元一致（verb-contract 既有 SHALL 的自然延伸）。remote_discard 由 bail 改為呼叫 client 方法，guard 拒絕翻譯為與本地 discard 相同的「需要 --force」語義化訊息。

### 決策 7：capability 翻正即解鎖，UI 元件層零改動

RemoteCapabilities（apps/desktop/src-tauri/src/remote.rs）的 validate/analyze/deleteChange/moveTask 四欄由矩陣常量翻真（deleteChange/moveTask 依 role 條件翻真，見決策 5）；remoteDataSource.ts 對應四方法由 unsupported 拒絕改為 invoke 直達（runVerb 的 validate/analyze 分支、deleteChange、moveTask）。UI 元件不改：停用 affordance（按鈕 disabled、拖排把手不渲染）本來就由 capability 管線驅動，翻真即自動恢復——以桌面測試釘住「remote 分頁 capability 全真時四操作 affordance 與本地同形」。

## Risks / Trade-offs

- move 邏輯遷 core 是共檔重構（tasks.rs、command/mod.rs 為多刀熱點）：apply 前依平行 session 提交衛生確認無平行刀在改同檔。
- index 定址競態：接受與既有 ordinal 勾選同級的暴露（誤搬可再拖、無資料遺失），以 SSE invalidate 收斂；不為此引入高誤判率的 CAS。
- CLI validate --all 的逐 change 組合在大 workspace 下多次往返：可接受（validate 為顯式操作、非熱路徑）；輸出聚合語意以測試釘住。

## Migration Plan

純新增端點與解鎖，無資料遷移。單刀交付：core 遷移（等價重構）→ server 端點→ client／CLI → 桌面解鎖，每步既有測試全綠才進下一步。回滾＝revert（無 schema 變更、無持久化格式變更）。

## Open Questions

（無——stable ID 定址與本地刪除語意統一皆已明確記為範圍外。）

## Context

任務分頁的寫回鏈：TaskList 受控 checkbox → RichDetailDrawer 轉發 → SpeclinkDataSource.setTaskDone →Tauri 指令 → apps/desktop/core 的 set_task_done_at（含開工章／touched／冪等語意）→ 宿主 refresh 全量清單 → refreshGen 遞增 → 抽屜重載五份文件 → tasksMd 更新才反映勾選；期間 busy 旗標對整列上 pointer-events 鎖。本刀改動落在 apps/desktop/core、apps/desktop/src-tauri、packages/ui 三處；speclink-core／speclink-cli 不動。相關者：在桌面 app 勾核 apply 進度的開發者／PO／PM。

## Goals / Non-Goals

**Goals**
- 任務批次操作（全部已完成／重置）單次寫檔完成，開工章語意與逐一勾選一致。
- 「下一個未完成」一鍵（含 n 快捷鍵）定位第一個未完成任務。
- 勾選即時回饋：UI 先行、失敗回滾、清單不鎖。

**Non-Goals**
- CLI 端批次指令；拖放排序行為；markdown 渲染樣式（另刀）。

## Decisions

### D1：批次動詞單指令雙用

嵌入引擎新增批次函式（manage.rs），接受 done 布林——true＝全部標完成、false＝全部取消勾選；一次讀檔、一次寫回。語意沿用單發勾選：done=true 且變更未開工時蓋開工章一次、touched 記錄一次；done=false 不蓋章、不記 touched；目標狀態已達成時冪等成功、不寫檔。Tauri 指令層曝露為新 command，前端 SpeclinkDataSource 增 setAllTasks(change, done)。
替代案：前端迴圈 N 次單發——N 次寫檔＋N 次 watcher refresh，卡頓放大，否決；獨立 reset 與 complete-all 兩個指令——語意重複、參數即可分流，否決。

### D2：下一個未完成純前端

TaskList 由既有解析結果定位第一個未完成任務，捲動至該列並短暫高亮；n 快捷鍵在抽屜開啟且任務分頁作用中時等效。全部完成時按鈕與快捷鍵不作用。無後端參與。
替代案：後端查詢第一個未完成——狀態已在前端解析完成，多一趟 IPC 無實益，否決。

### D3：樂觀更新落在抽屜層 tasksMd

勾選時 RichDetailDrawer 立即以本地改寫 tasksMd 字串（翻轉該 ordinal 的 checkbox 標記）觸發重渲染，再發寫回；寫回失敗還原改寫前字串並顯示單行錯誤。TaskList 維持受控元件、不引入自有勾選狀態。ordinal 穩定性：單發勾選是行編輯、不重編號，樂觀值與磁碟不漂移；外部同時改檔由 workspace-changed 世代重載以磁碟現況收斂。
替代案：TaskList 內部 state override——與 tasksMd 雙真相易漂移，否決；縮小 refresh 範圍——動到單一資料流設計，收益重疊於樂觀更新，否決。

### D4：busy 指標鎖移除、批次操作例外

單發勾選期間不再對清單上 pointer-events 鎖（連續勾選不互鎖；並行單發寫回各自為行編輯、互不衝突）。批次操作（全部已完成／重置）屬大動作：執行期間工具列與清單短暫 disabled，完成後由世代重載收斂——避免批次寫回與單發樂觀值交錯。拖曳讓路機制（dragActive）保留不動。
替代案：全面不鎖含批次——批次寫回中的單發樂觀值會被整檔覆蓋回退，體驗更差，否決。

## Implementation Contract

**可觀察行為**
1. 任務分頁清單頂部有工具列三鍵；「全部已完成」後 tasks.md 中全部任務為 [x]、抽屜進度 100%、檔案僅單次寫入；「重置任務」後全部為 [ ]、進度 0%、不蓋開工章。
2. 未開工變更按「全部已完成」後，變更 meta 出現開工章（與逐一勾選首次完成的行為一致）；已達目標狀態時重按不改檔。
3. 「下一個未完成」（按鈕或 n 鍵）使第一個未完成任務捲入可視範圍並短暫高亮；全部完成時兩者不作用且按鈕呈不可用。
4. 勾選任一 checkbox 立即翻轉、其餘任務仍可立即勾選；寫回失敗時該勾選回滾並顯示單行錯誤。
5. 唯讀封存檢視無工具列；拖曳排序行為與現況相同。

**驗收目標**
- Rust：apps/desktop/core 的 #[cfg(test)] 測試涵蓋批次函式（全勾單次寫回、重置、冪等、開工章／touched 語意），cargo test -p speclink-desktop-core --lib 全綠（本機需 --lib，見開發備忘）。
- 前端：npm test -w packages/ui（工具列渲染與回呼、disabled 態、readOnly 隱藏、樂觀翻轉與回滾）、npm test -w apps/desktop（dataSource 轉發、App wiring）全綠。
- 真實視窗：勾選無可感知延遲、工具列三鍵操作正確、tasks.md 內容對應。

**範圍邊界**
- In scope：apps/desktop/core/src/manage.rs、apps/desktop/src-tauri/src/lib.rs、packages/ui 的 TaskList／RichDetailDrawer／adapter、apps/desktop 的 tauriDataSource／App。
- Out of scope：speclink-core／speclink-cli、看板與討論抽屜、規格頁、拖放排序邏輯。

## Risks / Trade-offs

- [批次寫回與在途單發寫回交錯] → 批次執行期間工具列與清單短暫 disabled（D4）；單發之間為行編輯互不衝突。
- [樂觀值與磁碟不一致（外部寫者同時改檔）] → workspace-changed 世代重載以磁碟現況覆蓋收斂；失敗路徑明確回滾。
- [誤觸「重置任務」清掉進度] → 與 Spectra 工具列對齊不加確認框；tasks.md 受版本控管，可由 git 還原。
- [回歸對照] → CLI 兩 crate 零接觸，parity／color 對照不受影響。
- [跨平台] → 批次寫回沿用既有行編輯與寫檔路徑，無平台特有行為。

## Migration Plan

無資料遷移。新 IPC 指令為純新增，舊前端不呼叫即不受影響；回滾即還原 commit 重建。

## Open Questions

（無——關鍵取捨已在討論 desktop-reading-and-tasks-ux 定案）

## 1. git 身分快取（desktop-core，TDD）

- [x] 1.1 快取每根身分（紅→綠，design D1：identity 每根快取）：apps/desktop/core/src/manage.rs 新增 cached_git_identity——先寫測試以兩個不同 git user.name 的暫存 repo 斷言各根取值正確、同根重複呼叫回傳一致（紅）；再實作 static Mutex<HashMap> 快取並讓 set_task_done_at 與 set_all_tasks_at 的完成路徑換用（綠），對應規格「任務寫回非阻塞且序列化」的 git 身分快取重用場景。驗證：cargo test -p speclink-desktop-core --lib 全綠，既有 set_task_done_true_stamps_meta_and_records_touched 案例的 started_by 斷言不變

## 2. 寫入序列化（desktop-core，TDD）

- [x] 2.1 全域寫鎖（紅→綠，design D3：寫入序列化（全域寫鎖））：先寫測試——兩執行緒並發對同一 change 的不同任務呼叫 set_task_done_at（done=false 行編輯路徑），斷言兩個翻轉皆落盤、tasks.md 無遺失更新（紅，現行無鎖下以重複多輪提高暴露機率）；再於 manage.rs 新增 static WRITE_LOCK 並讓 set_task_done_at、set_all_tasks_at、move_task_at 與看板排序寫回入口整段持鎖（綠）。驗證：cargo test -p speclink-desktop-core --lib 全綠

## 3. 寫入 command 非阻塞化（src-tauri 薄殼）

- [x] 3.1 async command＋預熱（design D2：寫入 command 改 async＋spawn_blocking）：apps/desktop/src-tauri/src/lib.rs 的 set_task_done、set_all_tasks、move_task、reorder_card 改 async fn——鎖外複製 root 後以 tauri::async_runtime::spawn_blocking 執行既有委派、簽名補 Result（Tauri async 借用 State 約束）；setup 與 switch_root 以 std::thread 背景預熱 cached_git_identity（失敗靜默）。驗證：cargo check -p speclink-desktop 通過、npm test -w apps/desktop 全綠（前端 invoke 介面不變）

## 4. 前端在途載入作廢（packages/ui，TDD）

- [x] 4.1 樂觀狀態不被舊回應覆蓋（紅→綠，design D4：前端在途載入作廢）：packages/ui/src/__tests__/richDrawer.test.tsx 先寫案例——外部世代重載的 tasks.md 載入尚未回應時樂觀勾選一任務，隨後讓該舊回應（未勾選內容）到達，斷言畫面 markdown 維持樂觀勾選狀態（紅）；再於 packages/ui/src/components/RichDetailDrawer.tsx 的 handleToggle 樂觀改寫前遞增 requestSeq 作廢在途回應（綠），對應規格「任務寫回非阻塞且序列化」的舊載入回應不覆蓋樂觀狀態場景。驗證：npm test -w packages/ui 全綠且既有樂觀更新、回滾、讓路案例無退化

## 5. git 子進程視窗抑制（speclink-core util）

- [x] 5.1 git spawn 不建主控台視窗（design D5：git 子進程不建主控台視窗）：crates/speclink-core/src/util.rs 的 git 與 git_raw 改經共用建構函式，Windows 下帶 CREATE_NO_WINDOW 旗標——對應規格「任務寫回非阻塞且序列化」的無主控台視窗閃爍場景；git 呼叫內容、piped 輸出與回傳行為不變。驗證：cargo test -p speclink-core --lib 全綠（身分與 changed_files 既有案例不變）、cargo check -p speclink-desktop 通過

## 6. 整合驗證（真實視窗）

- [x] 6.1 真實視窗驗證無凍結：關閉執行中的 exe 後 npm run build -w apps/desktop 與 cargo build --release -p speclink-desktop，啟動 app 對照規格「任務寫回非阻塞且序列化」場景——勾選任務後立即取消勾選（視窗全程可操作、最終未勾選）、連續勾選多任務（首次後不再逐次 spawn git、無秒級凍結）、勾選過程無主控台黑窗閃現、對照 .openspec.yaml 與 touched 記錄確認蓋章語意不變（操作前先確認使用者未在使用螢幕）。驗證：npm test -w packages/ui、npm test -w apps/desktop、cargo test -p speclink-desktop-core --lib 全綠，實際操作截圖與時間軸無凍結

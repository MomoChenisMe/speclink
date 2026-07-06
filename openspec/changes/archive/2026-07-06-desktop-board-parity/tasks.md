## 1. 引擎：標記真相遷移（design D1 標記真相遷入 change meta）

- [x] 1.1 紅：撰寫 Store trait active change meta 原文讀寫對的失敗測試（crates/speclink-fs/tests/store_fs.rs）——讀取 changes/<name>/.openspec.yaml 原文、寫回後未知欄位與既有欄位逐字元保留、change 不存在時讀取回 None。驗證：cargo test -p speclink-fs 出現預期紅燈。
- [x] 1.2 綠：於 crates/speclink-core/src/store.rs 新增讀寫對（與 read_archived_meta／write_archived_meta 對稱命名）、crates/speclink-fs/src/lib.rs 實作。驗證：1.1 測試全綠。
- [x] 1.3 紅：撰寫 spec 需求「in-progress 標記真相存於 change meta」的失敗測試（crates/speclink-core/src/inprogress.rs 的 #[cfg(test)]，以測試替身 store 或 tempdir＋FsStore）——add 後 meta 含 started_at／started_by／started_with 且既有欄位逐字元保留、重複 add 冪等（值不變）、不存在 change 回既有錯誤、執行後 .git/speclink-app/ 不存在。驗證：cargo test -p speclink-core 出現預期紅燈。
- [x] 1.4 綠：改寫 inprogress::add 為經儲存介面讀-改-寫 meta（簽名改收 store 與身分來源），刪除 SQLite bootstrap／.migrate.lock／legacy 遷移整段；crates/speclink-cli/src/commands.rs 呼叫點一行跟隨（stdout 不變）；change meta 解析結構（crates/speclink-core/src/model.rs）加 started_at／started_by／started_with 三個 Option 欄位。驗證：1.3 測試全綠、cargo build --workspace 通過。
- [x] 1.5 紅→綠：補 spec 需求「meta 新欄位向後相容」（舊 meta 無新欄位時 list --json 與 status 輸出與遷移前位元級一致、視為未開工）與 spec 需求「歸檔保留完整生命週期歸屬」（archive 後三站欄位並存、started_* 逐字元保留——crates/speclink-core/src/archive.rs 既有讀-改-寫路徑以測試釘住）的失敗測試後轉綠。驗證：cargo test -p speclink-core 全綠。
- [x] 1.6 引擎回歸總驗證：cargo test --workspace 全綠；scratchpad 的 parity_suite／color_suite／twin harness 對照照常通過；speclink in-progress add 的 stdout 與 exit code snapshot 比對遷移前一致（首次與重複執行兩情形）。驗證：上述全部通過方可進下一章。

## 2. 桌面清單疊加標記（design D2 桌面清單疊加標記欄位）

- [x] 2.1 紅：撰寫 apps/desktop/core/src/query.rs 的 tempdir 測試——list_changes_at 的每個 change 項疊加 startedAt／startedBy／startedWith（camelCase；未開工為 null），且既有欄位（name、status、totalTasks、completedTasks、summary）形狀不變；change_meta_at 帶出新欄位。驗證：cargo test -p speclink-desktop-core 出現預期紅燈。
- [x] 2.2 綠：實作疊加（資料取自 model::list_changes 已解析的 meta，不另讀檔）。驗證：2.1 測試全綠，並確認 speclink list --json 的 CLI 輸出未被觸及（parity 對照仍綠）。

## 3. 檔案監看（design D3 openspec 檔案監看）

- [x] 3.1 紅：新增 apps/desktop/src-tauri/src/watch.rs 與其 tempdir 整合測試——對監看中的 openspec/ 樹寫入檔案後，於去抖窗口後收到單一合併通知；openspec/ 外（如專案根 .speclink/）的寫入不觸發；監看目標不存在或無權限時建立失敗不 panic、回傳可記錄的錯誤。驗證：cargo test -p speclink-desktop 出現預期紅燈。
- [x] 3.2 綠：以 notify＋官方 debouncer 實作 watch.rs（回呼式核心邏輯與 Tauri 事件發送分離以利測試），apps/desktop/src-tauri/Cargo.toml 加依賴，src-tauri/src/lib.rs 於 setup 掛載並發送 workspace-changed 事件（無 payload）。驗證：3.1 測試全綠、cargo build -p speclink-desktop 通過。
- [x] 3.3 前端訂閱 wiring：App.tsx 掛載時經 @tauri-apps/api/event 訂閱 workspace-changed 呼叫既有 refresh（卸載時解除），涵蓋 spec 需求「外部變更即時反映」的前端半邊；apps/desktop 測試以模擬事件斷言 refresh 被觸發。驗證：npm test -w apps/desktop 全綠。

## 4. 封存讀取縫（design D4 Store trait 封存讀取擴充）

- [x] 4.1 紅：撰寫失敗測試——Store trait 新增封存 artifact 原文讀取與封存 delta capability 列舉（帶預設實作 None／空），FsStore 覆寫讀 changes/archive/<dated-name>/ 實體；desktop-core 對應查詢含路徑穿越拒絕（dated_name 或 artifact 含 ..／絕對路徑／磁碟前綴回 None）。驗證：cargo test -p speclink-fs 與 -p speclink-desktop-core 出現預期紅燈。
- [x] 4.2 綠：實作 trait 方法、FsStore 覆寫、apps/desktop/core/src/query.rs 的封存文件查詢，src-tauri/src/lib.rs 註冊兩支唯讀 command（封存文件讀取、封存 capability 列舉），apps/desktop/src/adapter/tauriDataSource.ts 對映。驗證：4.1 測試全綠。
- [x] 4.3 對本刀新增的 trait 方法、Tauri command 與參數處理執行 sharp-edges audit checklist（speclink instructions --skill audit 取得清單），逐項記錄結論，發現的尖銳邊以紅綠循環修正。驗證：audit 清單逐項有結論、cargo test 全綠。

## 5. 快取升版（design D5 封存快取升版帶任務計數）

- [x] 5.1 紅：撰寫 apps/desktop/core/src/cache.rs 測試——CACHE_VERSION 升為 2 後，v1 舊庫自動整表重建；收斂時經 store 讀封存 tasks.md 解析 tasks_total／tasks_done 入表並隨清單回傳；無 tasks.md 的封存項計數缺席；快取失敗退回目錄直讀（既有行為保留）。驗證：cargo test -p speclink-desktop-core 出現預期紅燈。
- [x] 5.2 綠：實作 schema v2 與計數收斂。驗證：5.1 測試全綠。

## 6. 封存展開 UI（design D6 封存列展開檢視）

- [x] 6.1 紅：撰寫 packages/ui 元件失敗測試，涵蓋 spec 需求「已封存變更可展開檢視」——ArchivedRow 點擊展開唯讀分頁（提案／設計／任務／規格）、內容懶載入、任務分頁核取方塊不可互動、任務數徽章顯示（無 tasks.md 不顯示）、缺件文件分頁顯示空狀態。驗證：npm test -w packages/ui 出現預期紅燈。
- [x] 6.2 綠：擴充 packages/ui/src/components/ArchivedList.tsx（展開列＋分頁，渲染復用既有 Markdown 與唯讀任務清單）、packages/ui/src/adapter.ts 的 SpeclinkDataSource 加封存文件讀取與封存 capability 列舉兩方法、ArchivedItem 加任務計數欄位；App.tsx 傳入新的載入 props。驗證：6.1 測試全綠、npm test -w apps/desktop 既有測試不破。

## 7. 看板欄位規則（design D7 看板 stage 標記驅動）

- [x] 7.1 紅：撰寫失敗測試，涵蓋 spec 需求「看板欄位由生命週期標記驅動」——packages/ui/src/stage.ts 四象限矩陣（無標記 0 任務＝提案中、無標記 0/28＝提案中、有標記 13/28＝進行中、28/28 無論標記＝已就緒）；RichDetailDrawer 於有 startedAt 時顯示開工者與開工日、未開工不顯示。驗證：npm test -w packages/ui 出現預期紅燈。
- [x] 7.2 綠：改寫 stage 派生規則、packages/ui/src/adapter.ts 的 ChangeItem 加 started 三欄位、RichDetailDrawer 標頭顯示；更新既有 stage／kanban 測試資料至新規則。驗證：7.1 測試全綠、npm test -w packages/ui 全綠。

## 8. 整合驗證

- [x] 8.1 全套自動化：cargo test --workspace、npm test -w packages/ui、npm test -w apps/desktop 全綠；parity_suite／color_suite／twin harness 照常通過；git diff 確認 crates/speclink-cli 僅 in-progress 呼叫點一行變更。驗證：全部通過。
- [x] 8.2 真實視窗驗證（cargo build --release -p speclink-desktop 前先關閉執行中 exe；操作前確認使用者沒在使用螢幕）：外部終端勾任務→看板秒級自動更新（外部變更即時反映）；speclink in-progress add→卡片自提案中移入進行中、抽屜顯示開工者；外部建立與歸檔 change→清單自動增減；封存列展開實際檢視 48/48 的 desktop-shell-and-browser 文件內容與徽章；既有看板拖曳互動回歸。驗證：每項有截圖或觀察記錄，行為與 specs 各 Scenario 一致。

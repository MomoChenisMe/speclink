## 1. 快照與 remote 修正（desktop core）

- [x] 1.1 先寫紅測（設定頁的產出流程節的資料來源）：快照組裝以引擎 list_all／resolve_with 組出清單——名稱、來源層級、artifact 圖與各 artifact 的 description／instruction／template 全文；解析失敗的 schema 逐項標記錯誤而不拖垮整份快照。驗證：cargo test -p speclink-desktop-core 出現預期紅燈 <!-- speclink-task:tsk_01M0EVYG8WE02D57XYPFHMNDEH -->
- [x] 1.2 實作 D1 快照組裝在 desktop core：settings.rs 新增快照組裝函式與型別（含 schemaKnown 欄位，預設 true）。驗證：1.1 轉綠 <!-- speclink-task:tsk_01M0EVYG8X34F5XJS2XK88S998 -->
- [x] 1.3 先寫紅測（remote 模式的內建限縮與誤解析修正）：workspace 缺席的 remote 入口不讀 user 層——內建名以內嵌定義解析；非內建時 schemaKnown 為 false 且 schemaArtifacts 為空；client 本機 user 層同名 schema 完全不參與（spec 解析結果表三列各一案例）。驗證：cargo test -p speclink-desktop-core 出現預期紅燈 <!-- speclink-task:tsk_01M0EVYG8XD3J3DC7Q5HSKYQWH -->
- [x] 1.4 實作 D3 remote 限縮與怪癖修正：read_workflow_settings_from_text 的 schema 解析改為僅內建；local 入口的三層解析不變。驗證：1.3 轉綠且既有 settings 測試全綠 <!-- speclink-task:tsk_01M0EVYG8X226M9C943NT4X5RH -->

## 2. IPC 與 adapter

- [x] 2.1 先寫紅測：adapter 新增 readSchemas、writeWorkflowSchema、forkSchema 三方法，各自呼叫對應 invoke 並回傳快照／結果型別——這是產出流程的切換寫入與產出流程的客製 fork 的前端通道。驗證：npm test -w apps/desktop 出現預期紅燈 <!-- speclink-task:tsk_01M0EVYG8X613N5NFASD7898HV -->
- [x] 2.2 實作 D2 IPC 面：一查一寫——tauri 端 read_schemas、write_workflow_schema、fork_schema 三指令（local 寫入走引擎 set_workflow_schema_text，壞檔拒寫錯誤上拋；fork 包引擎既有函式）；remote 切換沿既有 revision 守門的 config 寫入通道在 adapter 組合。驗證：2.1 轉綠且 cargo build -p speclink-desktop 綠 <!-- speclink-task:tsk_01M0EVYG8XQ2BZPMP1144KXEZ3 -->

## 3. UI 節與詞彙

- [x] 3.1 先寫紅測（設定頁的產出流程節）：view 測試涵蓋——節渲染清單與唯讀詳情；下拉切換觸發寫入且產出規則分節固定鍵更新（產出流程的切換寫入）；fork 僅 local 模式渲染、按下後清單反映新專案層項目（產出流程的客製 fork）；remote 非內建名稱顯示遠端自訂尚不支援且不猜固定鍵（remote 模式的內建限縮與誤解析修正）；寫入失敗錯誤浮出於表單。驗證：npm test -w apps/desktop 出現預期紅燈 <!-- speclink-task:tsk_01M0EVYG8XRDB698V8YRDB9ZM4 -->
- [x] 3.2 實作 D4 UI 節與詞彙：ProjectSettingsView 的 config.yaml 頁簽新增產出流程節（清單、收合式唯讀詳情、切換下拉、local 限定 fork 按鈕、remote 狀態文案）；openspec/LANGUAGE.md 新增詞條「產出流程」（avoid 列 schema 於使用者可見文案，記使用者裁定與 change 名）。驗證：3.1 轉綠且 ui-copy-vocabulary 守門測試綠 <!-- speclink-task:tsk_01M0EVYG8X8HPH70R7ADVXK5AX -->

## 4. 首輪收尾

- [x] 4.1 全量回歸：cargo test -p speclink-desktop-core 與 npm test -w apps/desktop 全綠；speclink validate desktop-schema-panel 通過。驗證：三項指令輸出全部通過 <!-- speclink-task:tsk_01M0EVYG8X6KXYY3QKQSZ4NFGR -->

## 5. 建立通道（desktop core＋IPC＋adapter；2026-08-21 驗收增補）

- [x] 5.1 先寫紅測（產出流程的建立）：desktop core 新增 init_schema_at 包裝引擎 init_schema——合法名稱在 openspec/schemas/<name>/ 產出預設骨架（schema.yaml＋templates/）且 read_schemas_at 清單反映新專案層項目；名稱不合法（如 My Flow）與目標已存在各浮出引擎錯誤且磁碟不變（spec 建立輸入與結果表三列各一案例）。驗證：cargo test -p speclink-desktop-core 出現預期紅燈 <!-- speclink-task:tsk_01M0J902H98SEQKA8YGBNHEGH5 -->
- [x] 5.2 實作 init_schema_at：discover 專案根後委派引擎 init_schema（artifacts 與 description 用引擎預設，不收佈局參數）。驗證：5.1 轉綠 <!-- speclink-task:tsk_01M0J911XE6CCPD1R4M0F19BNW -->
- [x] 5.3 先寫紅測（adapter 的建立通道）：WorkspaceSettingsProvider 新增 createSchema 方法——local 呼叫 init_schema 指令帶 root 與 name；remote 為 rejected Promise 且不發任何 invoke。驗證：npm test -w apps/desktop 出現預期紅燈 <!-- speclink-task:tsk_01M0J91XGD0EC66TYBNNSKQ6H0 -->
- [x] 5.4 實作 D5 的 IPC 與 adapter 面：tauri 新增 init_schema 指令（收 root、name，委派 init_schema_at）並註冊；workspace.ts 兩個 provider 各補 createSchema；既有測試 mock（session／view／remoteFixtures／App）同步補方法。驗證：5.3 轉綠且 cargo build -p speclink-desktop 綠 <!-- speclink-task:tsk_01M0J96C292C6TH7H0MVNB6RFD -->

## 6. 頁籤重構與建立 UI（D4 改版＋D5 表單）

- [x] 6.1 先寫紅測（設定頁的產出流程頁籤）：view 測試涵蓋——local 頁簽依序 config.yaml／產出流程／.speclink.yaml 且產出流程清單在自己頁籤內、config.yaml 簽內無 schema-card（產出流程自成頁籤）；remote 頁簽依序 Workflow／產出流程；建立表單僅 local 渲染，輸入名稱送出後 createSchema 被呼叫且清單反映新項（建立產出專案層骨架）；建立失敗錯誤浮出於表單（不合法名稱顯性失敗）；remote 無建立入口。驗證：npm test -w apps/desktop 出現預期紅燈 <!-- speclink-task:tsk_01M0J97FQXJQ2WR88SPRFWNJ0C -->
- [x] 6.2 實作 D4 UI 頁籤與詞彙的改版＋D5 建立走引擎骨架、不做編輯器的表單面：ProjectSettingsView 把產出流程整卡搬出 config.yaml 簽成獨立 TabsTrigger／TabsContent（local 排第二、remote 排第二）；籤內加建立表單（名稱輸入＋建立按鈕，僅 local）；i18n 新增頁籤與表單鍵（兩語言同步）。驗證：6.1 轉綠且既有 view 測試與 ui-copy-vocabulary 守門測試全綠 <!-- speclink-task:tsk_01M0J9BME5PV24NZV6P6RSQ4YB -->

## 7. 二輪收尾

- [x] 7.1 全量回歸（第二輪）：cargo test -p speclink-desktop-core 與 npm test -w apps/desktop 全綠；speclink validate desktop-schema-panel 通過。驗證：三項指令輸出全部通過

## 8. 編輯入口（D6 編輯入口＝檔案管理器跳板；2026-08-22 驗收增補）

- [x] 8.1 先寫紅測（產出流程的編輯入口的資料面）：SchemaEntry 新增 path 欄位——專案層與 user 層項目為該 schema 目錄的絕對路徑、內建為 None；serialize 為 camelCase path。驗證：cargo test -p speclink-desktop-core 出現預期紅燈 <!-- speclink-task:tsk_01M0KY73074HV5VXTH5Y58A197 -->
- [x] 8.2 實作 path 欄位：schemas_snapshot 組裝時帶入各層目錄路徑（內建 None）。驗證：8.1 轉綠 <!-- speclink-task:tsk_01M0KY8BCWFV5XXE3K5887JKWA -->
- [x] 8.3 先寫紅測（編輯入口的通道與 UI）：adapter 新增 revealSchema 方法——local 呼叫 reveal_in_folder 指令帶 path、remote 為 rejected Promise 且不發 invoke；view 測試——path 非空的項目渲染開啟所在資料夾按鈕且按下以該 path 呼叫 revealSchema（專案層項目開啟所在資料夾）、內建項無此按鈕（內建項無編輯入口）、remote 無此按鈕。驗證：npm test -w apps/desktop 出現預期紅燈 <!-- speclink-task:tsk_01M0KYA7XS7M4M9PPQQZHCREJB -->
- [x] 8.4 實作 D6 通道與 UI：tauri 新增 reveal_in_folder 指令（OpenerExt reveal_item_in_dir 薄委派）並註冊；workspace.ts 兩個 provider 各補 revealSchema；schema-item 對 path 非空且 local 渲染按鈕；i18n 新增按鈕鍵（兩語言同步）；既有測試 mock 補方法。驗證：8.3 轉綠且 cargo build -p speclink-desktop 綠 <!-- speclink-task:tsk_01M0KYE738F6YC60KAJEY92ZTJ -->
- [x] 8.5 全量回歸（第三輪）：cargo test -p speclink-desktop-core 與 npm test -w apps/desktop 全綠；speclink validate desktop-schema-panel 通過。驗證：三項指令輸出全部通過 <!-- speclink-task:tsk_01M0J9C0J4V1NPSG3K51ZNT33W -->
## 9. 刪除與頁籤標籤（D7 刪除僅限專案層、確認後執行＋D8 頁籤標籤直出 Schema；2026-08-22 驗收增補）

- [x] 9.1 先寫紅測（產出流程的刪除的 core 面）：desktop core 新增 delete_schema_at（收 root 與 name、目標固定解析為專案 openspec/schemas/<name>）——非使用中的專案層 schema 刪除成功且 read_schemas_at 不再列出；使用中（config schema 鍵指著）拒刪、錯誤浮出且目錄原封不動；目錄不存在拒刪。驗證：cargo test -p speclink-desktop-core 出現預期紅燈 <!-- speclink-task:tsk_01M0MHDY716P27VZHWBE5RVWG1 -->
- [x] 9.2 實作 delete_schema_at：discover 專案根、讀 config 現值擋使用中、remove_dir_all 專案層目錄。驗證：9.1 轉綠 <!-- speclink-task:tsk_01M0MHEWP0TRHDK1RG182CHM60 -->
- [x] 9.3 先寫紅測（刪除通道與 UI＋頁籤標籤）：adapter 新增 deleteSchema 方法——local 呼叫 delete_schema 指令帶 root 與 name、remote 為 rejected Promise 且不發 invoke；view 測試——local 頁簽序 config.yaml／Schema／.speclink.yaml 且 remote 頁簽序 Workflow／Schema（既有「產出流程」頁籤斷言全數改 Schema）；專案層項目渲染刪除按鈕、按下開確認對話框、取消零呼叫、確認後 deleteSchema 被呼叫且清單反映（刪除經確認後移除專案層目錄＋取消確認零變動）；刪除失敗錯誤浮出（使用中的 schema 拒刪）；內建項與 remote 無刪除按鈕。驗證：npm test -w apps/desktop 出現預期紅燈 <!-- speclink-task:tsk_01M0MHHZ02CSX9Z8AB029WS524 -->
- [x] 9.4 實作 D7 通道與 UI＋D8 標籤：tauri 新增 delete_schema 指令並註冊；workspace.ts 兩個 provider 各補 deleteSchema；schema-item 對 source 為專案層且 local 渲染刪除按鈕＋AlertDialog 確認（沿 app.deleteTitle 既有模式）；i18n 新增 settings.schemaTab（值 Schema，兩語言同值）與刪除確認鍵，頁籤標籤改用 schemaTab；messages.test 的 schema 守門對 settings.schemaTab 放行；LANGUAGE.md「產出流程」詞條補頁籤標籤明文例外；既有測試 mock 補方法。驗證：9.3 轉綠且 cargo build -p speclink-desktop 綠 <!-- speclink-task:tsk_01M0MHRSPVHBGSE99GW45JS86J -->
- [x] 9.5 全量回歸（第四輪）：cargo test -p speclink-desktop-core 與 npm test -w apps/desktop 全綠；speclink validate desktop-schema-panel 通過。驗證：三項指令輸出全部通過 <!-- speclink-task:tsk_01M0MHRSV60WAKPGTZ9AWMRYP0 -->
- [x] [M] 4.2 開啟 desktop app 目視驗收 Schema 頁籤：local 專案確認頁籤獨立（標籤 Schema）且 config.yaml 簽無此節、看清單與詳情、切換一次並確認 config.yaml、fork 一次並確認清單反映、建立一次（含一次不合法名稱看錯誤浮出）並確認 openspec/schemas/ 與清單、對建立的項目按開啟所在資料夾並確認檔案管理器顯示該目錄且內建項無此按鈕、刪除一個非使用中的專案層項目（先取消一次確認零變動、再確認刪除並看目錄與清單）、對使用中項目確認拒刪錯誤浮出；連上 remote 專案確認頁籤存在但僅內建、無 fork／建立／刪除入口、狀態文案正確 <!-- speclink-task:tsk_01M0EVYG8X855QNYXKCGEQMA04 -->

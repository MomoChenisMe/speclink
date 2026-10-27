## Context

引擎面已由 change schema-engine-openspec-parity 落地：內建 spec-driven 為單一正典（解析、fork、byte-preserving 的 schema 鍵 setter 都在 speclink-core）。desktop core 直接連結 speclink-core（apps/desktop/core/src/settings.rs 既有的 schema_artifact_ids 就在呼叫 resolve_with），所以檢視與切換所需的資料與寫入函式全部已在手上——本 change 是純消費層：desktop core 組快照、tauri IPC 露出、React 設定頁呈現。既有怪癖：remote 入口 read_workflow_settings_from_text 傳 workspace None 後，schema_artifact_ids 仍拿 client 本機 user 層目錄解析 server 專案的 schema 名稱。

## Goals / Non-Goals

**Goals:**

- 設定頁新增產出流程獨立頁籤：檢視（清單＋唯讀詳情）、切換（寫 schema 鍵）、fork（local 限定）、建立（local 限定，引擎 init_schema 骨架）
- remote 限縮內建並修掉 user 層誤解析
- 使用者可見文案以「產出流程」承載（LANGUAGE.md 新詞條）

**Non-Goals:**

- 引擎任何改動（消費既有函式）；server 端點或 protocol DTO 新增（remote 內建由本地解析）
- desktop 內建 schema 編輯器（討論已否決——fork／建立後交給編輯器）；建立表單不收 artifact 佈局（引擎預設骨架，要客製直接改 schema.yaml）
- store 納管 openspec/schemas/（remote 自訂 schema 的前置，維持 deferred）
- schemas 或 schema 詳情的搜尋、排序、分頁（數量級用不上）

## Decisions

### D1 快照組裝在 desktop core

apps/desktop/core/src/settings.rs 新增產出流程快照組裝：呼叫 speclink_core::schema 的 list_all 與 resolve_with，輸出清單（每項：名稱、來源層級、artifact 圖與各 artifact 的 description／instruction／template 全文）。一次全量回傳、無 lazy 載入——schema 數量級是個位數、內容是文字，快照小。壞 schema（解析失敗）沿設定頁既有的 parse-error 語意：該項標記錯誤而非整節失效。deletion test：刪掉此組裝，desktop 就沒有產出流程資料來源——非純轉發（含來源層級標註、remote 限縮邏輯、錯誤語意）。

### D2 IPC 面：一查一寫

apps/desktop/src-tauri/src/lib.rs 新增指令：read_schemas（收 root，回 D1 快照）與 write_workflow_schema（收 root 與名稱，local 走 set_workflow_schema_text 讀改寫檔）。fork 走 fork_schema（收 root 與來源名，包引擎 fork 函式；複本名固定為引擎預設 <source>-custom，不收自訂名；UI 對同名跨層只在解析命中層渲染 fork 入口——引擎以第一命中解析來源，被 shadow 的項按下去會複製到前層內容）；建立走 init_schema（收 root 與名稱，包引擎 init_schema——artifact 佈局用引擎預設，不收佈局參數）。remote 模式：read_schemas 由 desktop core 以內嵌內建組裝（不打 server）；切換走既有 remote config 寫入通道——讀 /config 原文與 revision、套 set_workflow_schema_text、以 revision 守門寫回，組合落在 Rust 側 remote.rs 的 RemoteWorkspace::write_workflow_schema＋第七個指令 remote_write_workflow_schema（沿 write_workflow_fields 的既有形狀；token 與 credential 管理都在 Rust 側，adapter 只帶 expectedRevision——不加 server 端點）；fork 與建立在 remote adapter 為拒絕（rejected Promise，UI 不渲染入口）。

**替代方案**：檢視走 server route＋protocol DTO（討論已否決——拿 desktop 自己就有的資料）；切換擴充 WorkflowPolicyFields 加 schema 欄位（會把 schema 混進政策欄位的 update 語意，setter 已存在且語意更窄，捨）。

### D3 remote 限縮與怪癖修正

read_workflow_settings_from_text（workspace None 的 remote 入口）改為：schema 名稱先對內建解析；非內建 → schemaArtifacts 給空，快照新增 schemaKnown 布林（false ＝ 遠端自訂尚不支援），UI 據此顯示狀態文案並停用切換以外的細節。user 層目錄在此入口完全不參與——local 入口（workspace Some）維持三層解析不變。

### D4 UI 頁籤與詞彙

「產出流程」為設定頁的獨立頁籤（2026-08-21 使用者驗收裁定，取代首刀的 config.yaml 簽內新節）：local 頁簽依序 config.yaml → 產出流程 → .speclink.yaml，remote 依序 Workflow → 產出流程。頁籤標籤「產出流程」是人話標籤（檔名直出例外僅限設定檔頁籤，此籤不適用也不需要）。籤內佈局沿首刀落地的節內容整卡搬移（清單、收合式唯讀詳情、切換下拉、fork 按鈕、remote 狀態文案），同檔搬移不抽新檔；清單→詳情用既有的展開／收合模式呈現，不引入新導航層。fork 與建立入口只在 local 模式渲染（沿 worktree 開關對模式條件渲染的既有作法）。openspec/LANGUAGE.md 新增詞條「產出流程」：definition 指向 config.yaml 的 schema 鍵概念，avoid 列 schema（使用者可見文案中），why 記「與產出規則、產出政策同族，字面可推出『這個 change 會產出哪些文件、什麼順序』；經使用者裁定用譯詞（desktop-schema-panel，2026-08-20）」。ui-copy-vocabulary 的守門測試自動接手 avoid 詞的文案面檢查。

### D5 建立走引擎骨架、不做編輯器

產出流程頁籤提供建立表單（僅 local）：單一名稱輸入＋建立按鈕。送出即呼叫引擎 init_schema（desktop core 包裝 init_schema_at：discover 專案根後委派），artifact 佈局用引擎預設骨架（plan → tasks，schema.yaml＋templates/*.md），成功後重拉 read_schemas 清單即時反映。名稱驗證不在前端重複——kebab-case 規則與已存在檢查都在引擎，錯誤原樣浮出於表單（與 fork 同語意）。表單不收 artifact 佈局與描述：建立的價值是「拿到一個可改的合法起點」，佈局客製直接編輯 schema.yaml（Non-Goal 的編輯器邊界不動）。remote 不渲染建立入口，adapter 的 createSchema 為拒絕（與 forkSchema 同形）。

**替代方案**：表單收 artifact id 清單（引擎 init_schema 支援逗號清單）——被捨：使用者要求是「能建立」，佈局輸入是未被要求的彈性；骨架建立後改檔案同樣快。

### D6 編輯入口＝檔案管理器跳板

建立／fork 之後內容要能改，但編輯器 Non-Goal 不動（2026-08-22 使用者驗收裁定：開啟入口就好）：有磁碟路徑的清單項（專案層與 user 層；內建在 binary 內無檔案）提供「開啟所在資料夾」按鈕，經 tauri-plugin-opener 的 reveal_item_in_dir 在檔案管理器顯示 schema 目錄（schema.yaml 與 templates/ 同在眼前）。資料面：SchemaEntry 新增 path 欄位（schema 目錄絕對路徑；內建 None）——user 層路徑由 global_config_dir 推導，前端拼不出來，必須由快照帶。通道面：tauri 新增 reveal_in_folder 指令（Rust 側 OpenerExt 薄委派，不引入前端 opener JS 套件）；adapter 新增 revealSchema 方法，remote 為拒絕（remote 清單只有內建，本就無路徑）。

**替代方案**：desktop 內建文字編輯（詳情面板 textarea 編輯 schema.yaml 與 templates）——被捨：推翻既有討論的編輯器否決，且外部編輯器對多檔文字編輯本來就更強。

### D7 刪除僅限專案層、確認後執行

專案層項目提供刪除（僅 local；2026-08-22 使用者驗收裁定）：desktop core 新增 delete_schema_at（收 root 與 name，不收任意路徑——目標固定解析為專案 openspec/schemas/<name>，這也是不提供 user 層刪除的原因之一：user 層跨專案共用、誤刪影響面大，內建則無檔案可刪）。防護兩道：config 的 schema 鍵正指著它（使用中）拒刪顯性失敗；目錄不存在拒刪。UI 沿 change 刪除的既有 AlertDialog 確認模式（app.deleteTitle 同款）：按刪除先開確認對話框，取消零呼叫，確認後執行並重拉清單。引擎零改動不變——刪除是純檔案系統操作，落在 desktop core。

### D8 頁籤標籤直出 Schema

頁籤標籤改「Schema」（2026-08-22 使用者裁定）：設定頁頁籤列全是技術 token（config.yaml、.speclink.yaml、Workflow），唯一的中文標籤在列上反而突兀——沿「開發者工具中原生詞即最直觀」的既有裁定線（檔名頁籤、討論 slug、worktree 同線）。籤內文案維持「產出流程」（卡標題、下拉標籤、建立表單），與 config.yaml 頁籤內是人話卡的既有模式對稱。LANGUAGE.md「產出流程」詞條補明文例外（僅限設定頁頁籤標籤）；ui-copy-vocabulary 守門對 settings.schemaTab 鍵放行。

## Implementation Contract

**可觀察行為**：

- local 模式設定頁：產出流程為獨立頁籤（標籤「Schema」；config.yaml 簽內不再有此節），列出內建＋專案層＋user 層可解析的 schema；點入任一項可讀四欄全文；下拉切換後 config.yaml 僅 schema 鍵行變動、產出規則分節固定鍵隨新 schema 更新；fork 後 openspec/schemas/ 出現複本且清單即時反映；建立表單送出合法名稱後 openspec/schemas/<name>/ 出現引擎預設骨架（schema.yaml＋templates/）且清單即時反映；有磁碟路徑的項目帶「開啟所在資料夾」按鈕，按下後檔案管理器顯示該 schema 目錄，內建項無此按鈕；專案層項目帶刪除按鈕——確認對話框後 openspec/schemas/<name>/ 移除且清單反映，取消則零變動，使用中的 schema 拒刪且目錄不變
- remote 模式設定頁：產出流程頁籤存在但清單只有內建、下拉可選目標只有內建（非內建現值以停用項顯示）、無 fork 與建立入口；config 的 schema 名稱非內建時顯示遠端自訂尚不支援、產出規則分節不猜固定鍵；client 本機 user 層目錄的內容對 remote 快照零影響
- 壞 config.yaml 上切換：拒寫、錯誤浮出、檔案不變

**介面**：desktop core 新增快照組裝函式與快照型別（含 schemaKnown；SchemaEntry 含 path——schema 目錄絕對路徑，內建 None）及 init_schema_at、delete_schema_at 包裝；tauri 新增 read_schemas、write_workflow_schema、fork_schema、init_schema、reveal_in_folder、delete_schema 六個 local 指令＋remote_write_workflow_schema（remote 切換通道，組合在 Rust 側）；adapter 對應六方法（readSchemas／writeWorkflowSchema／forkSchema／createSchema／revealSchema／deleteSchema）；SettingsSnapshot 的 workflow 面新增 schemaKnown 與 schemaName 欄位（schemaName＝config 的 schema 鍵現值、下拉現值來源，壞檔給空字串；既有欄位形狀不變）。

**失敗形**：切換寫入失敗（壞檔、revision 落後、讀取失敗）以表單錯誤浮出，不靜默；fork 目標已存在時浮出引擎的既有錯誤訊息；建立的名稱不合法或目標已存在時浮出引擎的既有錯誤訊息、磁碟不變；刪除遇使用中的 schema 或目錄不存在時以單行錯誤浮出、磁碟不變；壞 schema 在清單標記錯誤，不拖垮整節。

**驗收**：cargo test -p speclink-desktop-core 全綠（快照組裝、remote 限縮與建立骨架的單元測試）；npm test -w apps/desktop 全綠（view 測試涵蓋頁籤佈局、清單詳情、切換、fork 與建立的條件渲染、remote 狀態文案）；ui-copy-vocabulary 守門測試綠（schema 不出現在使用者可見文案）。

**範圍邊界**：in scope——apps/desktop/core/src/settings.rs、apps/desktop/src-tauri/src/lib.rs、apps/desktop/src-tauri/src/remote.rs（remote 切換通道）、apps/desktop/src/adapter/workspace.ts、apps/desktop/src/session.ts（provider 介面）、apps/desktop/src/i18n/messages.ts、apps/desktop/src/views/ProjectSettingsView.tsx 與其測試、openspec/LANGUAGE.md。out of scope——crates/ 全部、apps/server-web、speclink-protocol、store 三驅動、packages/ui 新元件。

## Risks / Trade-offs

- [instruction／template 全文較長，詳情區塊撐版面] → 唯讀詳情用收合預設、展開捲動；不做分頁
- [remote 快照形狀新增欄位影響既有消費端] → schemaKnown 為新增欄位、預設 true，舊讀者不受影響；view 測試釘形狀
- [「產出流程」與既有「產出規則」視覺相近致混淆] → 節內首行一句說明文案點明「此專案的變更會產出哪些文件」；review 站可再校
- [fork 後使用者直接編輯出壞 schema，清單解析失敗] → 沿 D1 的 per-item 錯誤語意，壞項標記錯誤並顯示引擎錯誤訊息

## Migration Plan

無資料遷移。回滾＝revert 單一 commit。LANGUAGE.md 詞條隨 change 進出。

## Open Questions

無。

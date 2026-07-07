## Context

config.yaml 的 context 與 rules 是指令注入的政策來源（rules 依 artifact id 注入產出指令、context 為專案自由文字說明），目前只能手改 YAML；WorkflowConfig 解析失敗會靜默退回預設，手寫反引號開頭的條目即炸整份檔。主刀 desktop-config-multiproject 已建立寫入基建：speclink-core config.rs 的 update_workflow_config_text（text→text 純函式、serde_yaml::Mapping 讀-改-寫、未觸及鍵保留、註解遺失為已接受取捨，其 D4）、apps/desktop/core/src/settings.rs 的讀取三態與寫入雙重驗證（其 D5）、SettingsView 設定頁。本變更把 rules/context 從主刀的「僅原樣保留」名單改為可編輯，全數決策源自已結論討論「config-context-與-rules-gui-編輯」。前置相依：主刀完成封存後方可 apply。

## Goals / Non-Goals

**Goals:**

- 設定頁可編輯「專案說明」（context）與「產出規則」（rules），寫入安全沿用主刀機制，「政策靜默失效」在 GUI 路徑上結構性不可能（固定鍵、自動引號、雙重驗證、解析失敗停用）。
- 清空即移除鍵，維持「未設定＝預設」語意。

**Non-Goals:**

- 自訂工具描述子與 remote 段編輯、遠端 store 寫入、規則語意 lint 與注入預覽、註解保留（含手寫，討論已確認不保護）、CLI 任何變更。

## Decisions

### D1 擴充主刀政策純函式的變更集（speclink-core 單一寫入真相）

crates/speclink-core/src/config.rs 的 update_workflow_config_text 變更集擴充兩個目標：context（三態——設值／移除／不動）與 rules（整份代換：以 artifact id 為鍵的字串清單映射；空清單移除該 artifact 鍵、整份為空移除 rules 鍵）。函式維持 text→text、不觸檔案系統。替代方案：(a) desktop-core 自行讀-改-寫——政策序列化知識外漏出引擎，主刀 D4 已否決同型方案；(b) 另立獨立函式——對同一檔兩次讀-改-寫、呼叫端串接兩輪驗證，徒增複雜度，否決。storage 解耦方向：純函式輸入輸出皆文字，遠端 store 情境（Deferred）屆時僅替換文字來源，不需改函式。

### D2 產出規則清單編輯器採 schema 固定鍵與上下移排序

「產出規則」區段以活躍 schema 的 artifact id 分節（spec-driven 即 proposal／design／specs／tasks），SHALL NOT 提供自由鍵輸入——打錯鍵靜默永不生效的風險結構性移除。每節條目為單行文字輸入，支援新增、編輯、刪除與上下移動按鈕排序（清單順序＝指令注入順序）。替代方案：dnd-kit 拖曳排序——質感較好，但 jsdom 測不出拖曳互動且需真實視窗驗證（專案備忘），條目數通常個位數，成本高於價值，否決；自由鍵值編輯器——重蹈靜默失效，討論已否決。

### D3 設定讀取 payload 擴充 context、rules 與 schemaArtifacts

apps/desktop/core/src/settings.rs 的讀取函式回傳擴充三欄：context（字串，缺席即未設定）、rules（依 artifact id 分組的字串清單）、schemaArtifacts（活躍 schema 的 artifact id 清單，依引擎既有顯示序）。橋接與前端欄位一律 camelCase。沿用既有單一設定讀取路徑，不另設 schema 查詢 command——替代方案：獨立 IPC 查詢——多一條通道只為一份靜態清單，否決。解析失敗時沿用主刀 parseError 語意，「專案說明」「產出規則」兩區段隨該檔表單一併停用。

### D4 寫入安全與鍵移除語意沿用主刀雙重驗證

寫入走主刀既有寫入函式同一流程：寫檔前解析原文（失敗中止）→ 產出新文字並以 WorkflowConfig 解析器驗證目標值 → 寫檔 → 回讀再驗；任一步失敗回單行錯誤（指明檔案與階段）、檔案與表單維持原狀。清空→移除鍵的語意在 D1 純函式內落實。含 YAML 保留起始字元（反引號等）的條目由 serde_yaml 序列化自動加引號——規格以反引號條目為 Example 釘住此契約。

### D5 GUI 文案採正典詞且新字串全數進 i18n 字典

區段名用 openspec/LANGUAGE.md 正典詞「專案說明」「產出規則」；本刀 apply 時主刀 i18n 已落地（前置相依），新字串一律進 apps/desktop/src/i18n/messages.ts 雙語字典（key 集合相等為主刀既有需求）。「產出規則」區段附一行說明文字點明儲存會重寫 config.yaml 且不保留檔內註解（管理註解遺失的意外感）。替代方案：硬編 zh-TW——違反主刀後的字典需求，否決。

## Implementation Contract

- **行為**：設定頁新增「專案說明」（多行文字區）與「產出規則」（schema 固定鍵分節清單編輯器）兩區段；儲存後 config.yaml 僅目標鍵變動、其餘鍵原樣保留；專案說明清空移除 context 鍵、規則某節清空移除該 artifact 鍵、全空移除 rules 鍵；寫出檔案永遠可被引擎解析且條目值逐字元還原；config.yaml 解析失敗時兩區段停用且不可儲存。
- **介面／資料形狀**：core——update_workflow_config_text 變更集新增 context 三態與 rules 整份代換；desktop-core——設定讀取 payload 新增 context、rules、schemaArtifacts（camelCase），寫入函式變更集對應擴充；前端——SettingsView 兩新區段，清單排序以上下移按鈕操作。
- **失敗模式**：寫入任一驗證步失敗→單行錯誤訊息指明檔案與階段、磁碟檔案逐字元不變、表單維持使用者輸入；載入 parseError→警告呈現＋表單停用（沿用主刀 D5 語意）。
- **驗收**：cargo test -p speclink-core 與 cargo test -p speclink-desktop-core 新增測試全綠；npm test -w apps/desktop 的 SettingsView 測試全綠；真實視窗於臨時工作區實測檔案效果（含反引號條目與清空移除鍵）。
- **範圍**：in——config.yaml 的 context 與 rules 兩鍵的 GUI 讀寫與表單；out——proposal Non-Goals 全數（描述子與 remote 段、遠端 store 寫入、lint 與預覽、註解保留、CLI 變更）。

## Risks / Trade-offs

- [主刀封存前 desktop-config delta 措辭調整，本刀 MODIFIED delta 失準] → apply 前執行 /speclink-drift，依封存後正典措辭校正本刀 delta。
- [serde_yaml 重寫多行 context 的標量樣式與原檔不同（block scalar 變引號式）] → 規格只釘「值逐字元還原＋可解析」，不釘標量樣式；測試斷言解析後的值。
- [使用者對「儲存後註解消失」意外] → 既定取捨（討論確認不保護）；D5 的區段說明文字先行告知。
- [規則條目含前導/尾隨空白造成比對困惑] → 條目存入前 trim；空字串條目不寫入（儲存時滌除），規格 Example 釘住。

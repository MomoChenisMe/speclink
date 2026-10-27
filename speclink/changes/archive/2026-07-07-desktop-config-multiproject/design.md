## Context

① desktop-shell-and-browser 交付的桌面 app 以 Tauri 殼直嵌 speclink-desktop-core（apps/desktop/core，不依賴 Tauri、可獨立測試），Tauri 層 AppState 僅一欄 root: PathBuf，啟動時取工作目錄、之後不可變；14 個既有 command 都以 root 逐次傳入 desktop-core 的 *_at(root, …) 函式，而 init_core_context(root) 每次呼叫重新以 Workspace::discover 向上探索並重建 ProjectContext——換 root 在 core 層天然可行，卡點只在 Tauri 層的不可變 state。前端為 App.tsx（頂欄「開啟專案」與側欄「設定」為無 handler 佔位）＋ Zustand store（createAppStore(dataSource)）＋ packages/ui 元件庫（經 SpeclinkDataSource 介面注入資料，不依賴 store）。

設定面：speclink-core 的 AppConfig（.speclink.yaml）與 WorkflowConfig（openspec/config.yaml）皆只有 Deserialize；唯一寫回先例是 init.rs 的 write_remote_section／remove_remote_section（對原始 serde_yaml::Mapping 讀-改-寫，註明不保留註解）。WorkflowConfig::from_text 對解析失敗靜默回傳預設——這是既知風險：GUI 寫壞 config.yaml 會使整份工作流政策無聲失效。

i18n 面：repo 無任何 i18n 基建；UI 字串硬編 zh-TW，集中在 packages/ui/src/components 之 13 個元件（約 92 行含中文）與 apps/desktop/src/App.tsx（約 24 行），另有測試檔以中文字串斷言。

Tauri 面：未使用任何 plugin（無 dialog）；capabilities/default.json 僅 core:default 權限。

## Goals / Non-Goals

**Goals:**

- 執行期切換專案 root（原生資料夾選擇器＋持久化專案分頁列），未初始化目錄經確認後自動 init。
- 圖形化設定頁：.speclink.yaml 的 tools 多選與 openspec/config.yaml 的 locale／spec_locale／tdd／audit，寫入具解析驗證、未觸及鍵原樣保留。
- UI 介面 i18n：zh-TW／en，預設跟隨系統、可手動切換，全硬編字串抽 key。

**Non-Goals:**

- 側欄「規格」「備忘」內容頁、GUI 自由文字編輯 artifacts、zh-TW／en 以外語言。
- CLI（speclink-cli）任何行為或輸出變更；speclink-core 既有函式簽名與行為變更（僅新增 pub API）。
- 經 GUI 編輯 rules／context／remote 段／自訂工具描述子／spec_dir（僅原樣保留）。
- 多視窗、同時多專案、專案間狀態隔離持久化（一次一個 root）。

## Decisions

### D1 執行期可變 root：AppState 持 Mutex<PathBuf>

AppState 的 root 欄位由 PathBuf 改為 Mutex<PathBuf>；既有 14 個 command 改為鎖內複製 root 後照舊傳入 desktop-core（呼叫模式不變，鎖粒度僅止於讀取路徑）。desktop-core 維持無狀態、逐呼叫收 root 的現行架構——不引入常駐 ProjectContext。
替代方案：(a) 換專案即重啟 app 行程——體驗差且丟前端狀態，否決；(b) desktop-core 持有可變全域 context——引入共享可變狀態與快取失效問題，違背現行「每呼叫重建、檔案系統為真相」的簡單模型，否決。歸檔快取（.speclink/desktop-cache.db）本就落在各專案根下，隨 root 切換自然分離，無須遷移。

### D2 資料夾選擇採 tauri-plugin-dialog

新增 tauri-plugin-dialog 依賴並於 capabilities/default.json 授予 dialog 開啟權限，前端經官方 JS API 開資料夾選擇器。
替代方案：(a) HTML file input（webkitdirectory）——WebView 安全模型拿不到絕對路徑，無法作為專案 root，否決；(b) 直接依賴 rfd crate 自製 command——繞過 Tauri 2 的 capabilities 權限模型，與生態慣例不符，否決。

### D3 開啟專案三態流程

後端以「所選目錄起向上探索」（與啟動語意一致，沿用 Workspace::discover）判定：(1) 命中專案 → 切換 root 並回專案資訊；(2) 未命中 → 回報 uninitialized、不切換，前端顯示確認對話框（含 AI 工具多選 claude／codex，預設勾 claude），確認後呼叫 init（消費 speclink_core::init::init，force=false、spec_dir 固定 openspec）再切換；(3) 使用者取消對話框 → 維持原專案。init 為既有邏輯的直接消費：openspec 骨架、.speclink.yaml、per-tool skills 與 CLAUDE.md／AGENTS.md marker 內容不變。
替代方案：選定即無確認靜默 init——在使用者選錯資料夾時未經同意寫入多個檔案，否決；GUI 提供 spec_dir 客製——超出需求、增加表單複雜度，否決（Non-Goal）。
確認對話框遵循「寫入型確認框按鈕原則」（討論「專案選擇對齊-spectra」提煉自 Spectra 升級對話框誤觸事故）：安全鈕（取消）靠左並持預設焦點，寫入鈕（建立工作區）靠右、與安全鈕拉開距離且視覺重量不同。

### D4 設定回寫純函式落在 speclink-core

speclink-core 新增兩個 pub 函式：更新工作流政策欄位（輸入原始 config.yaml 文字與欄位變更集，輸出新文字）與更新 .speclink.yaml 的 tools 清單（輸入原始文字與內建工具選集，輸出新文字）。兩者對 serde_yaml::Mapping 讀-改-寫：僅代換目標鍵，未觸及鍵（rules、context、remote、spec_dir、自訂描述子）原樣保留；政策欄位設回預設值時移除該鍵（維持「未設定＝預設」語意）而非寫入明值。檔案讀寫由呼叫端（speclink-desktop-core）負責——政策函式維持 text→text 純函式，與 WorkflowConfig::from_text 的儲存解耦方向一致（remote store 情境下 config 文字可能不來自本地 fs）；.speclink.yaml 屬 host-side 檔案，永遠本地。tools 寫入完成後呼叫 speclink_core::init::update 同步 skills 與清理殘留。
替代方案：(a) 錨定範本註解行的文字代換——保留註解但每欄位一條脆弱規則、範本演進即壞，否決；(b) 引入保留註解的 YAML 函式庫——新外部依賴只為註解，違反不過度設計，否決；(c) API 放 speclink-desktop-core——政策序列化知識外漏出引擎、未來 CLI 或 web 無法重用，否決。已知取捨：被寫入的檔案會失去範本教學註解（與 write_remote_section 先例一致；GUI 設定頁本身即註解教學的替代呈現）。

### D5 寫入前後雙重解析驗證

讀取：設定頁載入用的橋接函式必須區分「檔案缺席／欄位未設定」與「檔案存在但解析失敗」——後者於回傳 payload 帶 parse error 訊息，前端以警告呈現並停用該檔的表單（防止 GUI 把使用者手寫但暫時壞掉的檔覆蓋掉）。寫入：改寫前先解析原文（失敗即中止回報）、序列化後先以對應 config 解析器驗證新文字可解析且目標欄位值正確，再寫入檔案，寫後回讀再驗一次；任一步失敗回傳單行錯誤訊息，前端顯示且表單維持原值。
替代方案：僅寫前驗證——寫入中斷或編碼問題留下壞檔即觸發「政策靜默退預設」既知風險，雙重驗證成本極低，值得。

### D6 桌面專屬 command 不擴 SpeclinkDataSource

新增 Tauri command：open_project、init_project、current_project、project_stats、read_settings、write_app_tools、write_workflow_config（snake_case 與既有 14 個一致；payload 欄位 camelCase）。project_stats(path) 為唯讀、不切換 root——供分頁列還原時對各背景分頁取進行中變更數（委派 desktop-core 既有逐呼叫收 root 的查詢，路徑失效回 Err、前端據以轉錯誤態）。這組 workspace 管理操作不進 packages/ui 的 SpeclinkDataSource——該介面是「change／spec 瀏覽管理」抽象，未來 web adapter（web-server-postgres）的專案與設定語意完全不同（無本地資料夾可選），塞入只會逼 web 端決定「對應端點 vs 優雅缺席」。前端由 apps/desktop 直接 invoke（獨立的 workspace adapter 模組），SettingsView 與開啟專案 UI 因此屬 apps/desktop 而非 packages/ui。
替代方案：全數併入 SpeclinkDataSource——介面被單一宿主的能力污染，否決（延續 2026-07-05 web-server-postgres 骨架討論的結論）。

### D7 自製輕量 i18n（I18nProvider）

packages/ui 新增 i18n 模組：I18nProvider（props：locale、可選 messages 供 app 層附加字典）與 useI18n()（回傳 t(key) 與 locale）。packages/ui 元件字串內建 zh-TW／en 兩份字典（key 依元件命名空間），apps/desktop 的字串（App.tsx、SettingsView、store 的使用者可見訊息）由 app 層字典提供、經 provider 合併。兩語言 key 集合相等由單元測試保證；缺 key 時 t 回傳 key 本身（開發期可見的失敗，而非靜默錯語言）。既有以中文斷言的元件測試改為包 I18nProvider locale zh-TW 後照舊斷言。
替代方案：(a) react-i18next／react-intl——字串規模約 150 key 以下，用不到 ICU 複數與插值生態，新增執行期依賴與 bundle 不划算，否決；(b) 逐元件 props 傳字串——十餘個元件的 props 爆炸且每新增字串改兩層，否決。

### D8 語言偏好與最近清單存 localStorage

UI 語言偏好：localStorage 單鍵，null 表跟隨系統（navigator.language 以 zh 開頭 → zh-TW，否則 en）；設定頁提供「跟隨系統／zh-TW／en」三選，切換即時生效。專案分頁列（取代原「最近開啟清單」設計，見 D10）：localStorage 存分頁陣列（路徑＋顯示名＋順序）與最後活躍分頁，上限 10，成功開啟去重並移至既有分頁（設為 active，位置原地保留）、關閉分頁即自陣列移除——分頁列本身就是持久化的最近專案，不另設最近選單。兩者皆為 app 本機狀態——UI 語言與 config.yaml 的 locale（AI artifacts 產出語言）是兩件事，設定頁需在兩處欄位旁以說明文字區分。
替代方案：語言偏好寫入 .speclink.yaml——UI 語言是「這台機器上這個人」的偏好而非專案屬性，跨專案應一致，否決。

### D9 設定頁視圖與表單原語擴充

App.tsx 導航擴為三視圖（變更看板／已封存／設定），SettingsView.tsx 落在 apps/desktop（依賴 Tauri invoke，見 D6）。表單所需 shadcn 原語（select、switch、checkbox、dropdown-menu 等現缺者）新增至 packages/ui/src/components/ui 維持單一設計系統（Tailwind v4、teal 主色）。dnd-kit 既有備忘（PointerSensor activationConstraint、DragOverlay）不受本變更影響但真實視窗驗證時一併回歸。
替代方案：SettingsView 放 packages/ui 並經 props 注入所有讀寫——為單一宿主視圖引入十餘個 callback props 的抽象稅，否決。

### D10 專案分頁列（UI 形態對齊 Spectra）

頂欄以持久化分頁列取代「目前專案」名稱顯示（2026-07-06 討論「專案選擇對齊-spectra」實測 Spectra 桌面 app 後定案）：跨啟動還原上次分頁、active 分頁以 teal 粗框標示即目前專案、✕ 僅於 active 與 hover 顯示、「＋」掛分頁列尾端與右上「開啟專案」雙入口、Ctrl+Tab 循環與 Ctrl+1..9 直達。點分頁＝以該路徑走 D3 開啟流程（含 watcher 重掛與整批 refresh）；分頁指向已消失路徑時轉錯誤態（警示 icon＋灰字），點擊顯示錯誤與「自分頁移除」。零分頁（首次啟動）顯示「開啟專案」空狀態引導頁，取代現行空看板。ProjectTabs 元件落在 apps/desktop（依賴 workspace adapter，同 D6 歸屬邏輯）；徽章 tooltip 所需 shadcn tooltip 原語入 packages/ui/src/components/ui。
替代方案：多 root 同時活躍（per-tab watcher／cache／store 分 tab）——單視窗一次只呈現一個專案，範圍爆炸且違反 Non-Goal，否決；學 Spectra 的格式升級式 onboarding（切入分頁即彈寫檔對話框）——實測誤觸即改壞使用者 repo，維持 D3 確認後才寫入，否決；每專案儀表板首頁——看板欄頭已有計數，看板即儀表板，否決；分頁之外另設最近清單選單——同一概念重複表達，否決。

### D11 分頁徽章採背景快照制

分頁徽章＝該專案進行中變更數（hover tooltip「N 個進行中變更」）。活躍分頁由既有 watcher 整批 refresh 即時更新；背景分頁於 app 啟動還原分頁列時經 project_stats（D6）各輕掃一次，之後保留最後已知值（切走時以當下值快照）。
替代方案：背景分頁持續即時（per-tab watcher）——回到 D10 否決的多 root 活躍範圍，且徽章精度不值該成本，否決；不顯示徽章——Spectra 實測中徽章是掃視多專案狀態的主要線索，保留。

## Implementation Contract

**行為（使用者可觀察）：**

- 頂欄「開啟專案」（或分頁列尾端「＋」）開啟原生資料夾選擇器；選定 speclink 專案目錄（或其子目錄）後，看板、已封存、設定頁全數呈現新專案內容，無須重啟；分頁列新增（或去重上移）該專案分頁並標示為 active（root 目錄名＋進行中數徽章）。
- 分頁列跨啟動還原；點分頁或 Ctrl+Tab／Ctrl+1..9 切換專案；關閉分頁即自持久化清單移除；零分頁時顯示「開啟專案」空狀態引導頁；分頁路徑失效轉錯誤態、點擊可自分頁移除；hover 徽章顯示「N 個進行中變更」tooltip。
- 選定未初始化目錄時出現確認對話框（工具多選，預設 claude）；確認後該目錄產生 openspec/ 骨架、.speclink.yaml、所選工具的 skills 與 marker 檔，app 隨即切入該專案；取消則維持原專案、目標目錄無任何寫入。
- 側欄「設定」開啟設定頁：tools 多選、locale／spec_locale 下拉、tdd／audit 開關、UI 語言三選；儲存後重開 app 或重讀檔案可見值已持久化，未觸及鍵與自訂描述子原樣保留；對解析失敗的 config 檔顯示警告且拒絕寫入。
- 系統語言非中文時首啟 UI 為英文；設定頁切換 UI 語言即時全介面生效，且不影響 config.yaml 的 locale 值。

**介面／資料形狀：**

- 新 Tauri command 七支：open_project(path) 回專案資訊或 uninitialized 狀態（不切換）；init_project(path, tools) 執行 init 並切換；current_project() 回目前 root 與專案名；project_stats(path) 唯讀回該專案進行中變更數（不切換 root，路徑失效回 Err）；read_settings() 回兩檔的欄位值與各自可選的 parseError；write_app_tools(tools)；write_workflow_config(fields)。錯誤一律 Err(String)，payload 欄位 camelCase，與既有 14 command 慣例一致。
- speclink-core 新增 text→text 純函式兩支（工作流政策欄位更新、tools 清單更新），輸入原始 YAML 文字、輸出驗證過的新文字；未觸及鍵保留、設回預設即移除鍵。函式與變數 snake_case。
- packages/ui 新增並匯出 I18nProvider 與 useI18n；所有 packages/ui 元件顯示字串經 t(key) 取得，內建 zh-TW／en 字典。

**失敗模式：**

- open_project 對不存在或不可讀路徑回 Err 單行訊息；前端顯示錯誤並自最近清單移除該項。
- init_project 失敗（如目標不可寫）回 Err，root 不切換。
- 設定寫入任一驗證步失敗回 Err 單行訊息（指明哪個檔、哪一步），檔案維持原內容，表單維持原值——絕不留下不可解析的 config 檔。
- read_settings 對解析失敗的檔案回 parseError 而非靜默預設值（刻意浮出，對比引擎既有的靜默 fallback）。

**驗收條件：**

- cargo test -p speclink-core：新增純函式的單元測試（未觸及鍵保留、設回預設移除鍵、注入非法值被拒、壞輸入中止）。
- cargo test -p speclink-desktop-core：open／init／settings 橋接對 tempdir 專案的整合測試（三態流程、雙重驗證、parseError 區分）。
- npm test -w packages/ui 與 npm test -w apps/desktop：i18n key 集合相等測試、元件在 zh-TW／en 下渲染測試、設定頁與開啟專案流程的元件測試。
- 真實視窗驗證（依 CLAUDE.md 備忘）：release exe 實際切換專案、未初始化目錄 init、設定頁寫入後檔案內容檢視、語言切換——jsdom 測不出的互動一律實點驗證。
- CLI 回歸：speclink-cli 零改動；speclink-core 僅新增 pub API，既有 parity／color 對照套件照常通過。

**範圍邊界：**

- In scope：上述七 command、兩支 core 純函式、SettingsView、i18n 基建與全 UI 字串抽 key、專案分頁列（持久化、徽章、空狀態頁、快捷鍵、失效態）、dialog plugin 與權限。
- Out of scope：SpeclinkDataSource 介面變更、CLI 任何檔案、config.yaml 的 rules／context 編輯、遠端（remote store）專案的開啟與設定、封存快取遷移。

## Risks / Trade-offs

- [GUI 寫壞 config.yaml → 整份政策靜默退預設] → D5 雙重解析驗證＋寫後回讀；解析失敗的檔案拒寫並在 UI 呈現警告。
- [Mapping 讀-改-寫丟失範本教學註解] → 接受（與 write_remote_section 先例一致）；設定頁欄位說明文字承接教學角色；風險僅及被 GUI 寫過的檔。
- [i18n 抽 key 大面積觸碰 packages/ui 全元件，回歸面廣] → 既有 vitest 全數改包 I18nProvider 後照舊斷言（斷言字串不變即為回歸保護）；真實視窗驗證補 jsdom 盲區；key 集合相等測試防兩語言字典漂移。
- [root 切換瞬間 in-flight 查詢以舊 root 回應蓋畫面] → store 的切換 action 於 command 成功後才觸發整批 refresh。現況更新（2026-07-06）：drawer-live-reload 已為抽屜內容重載引入「刷新世代＋latest-wins」機制——切換 action 掛既有世代遞增即得殘影防護，無需自建 epoch。
- [跨平台路徑：Windows 磁碟前綴與分隔符] → root 全程以 PathBuf 傳遞不做字串解析；dialog 回傳的絕對路徑不經 is_safe_path_param（該防護僅針對 artifact 相對路徑參數）；desktop-core 測試以 tempdir 覆蓋 Windows 與 POSIX。
- [tools 寫入後 skills 未同步導致 .claude/.agents 殘留或缺漏] → 寫入成功即呼叫既有 update（同步＋清理殘留），失敗浮出錯誤而非靜默。
- [新增 tauri-plugin-dialog 依賴增加建置面] → 官方一方 plugin、Tauri 2 生態標準件；替代方案（自製檔案選擇）成本與風險更高。

## Context

情境 4（完全本地）目前僅有 CLI（speclink.exe）。使用者要求一個「像 spectra.exe 一樣雙擊即跑」的桌面工具：看得到 change 看板、文件樹與 spec，並能直接執行動詞。本 change 是四情境 GUI 工具矩陣的第 ① 刀（序 4→3→2→1），除交付情境 4 桌面儀表板外，另打底一套跨桌面/web 共用的前端元件庫供第 ③ 刀（web-server-postgres）復用。

現況與約束：本 repo 是 Cargo workspace（crates/speclink-core 引擎、speclink-cli 前端、speclink-fs/remote/node）。speclink-core 為同步 Rust、無 async runtime、無網路，且既有 `--json` payload（camelCase）已是穩定契約。尚無 JS 工作區（僅 crates/speclink-node 自帶 node_modules）。參照架構由反組譯 spectra 2.3.1 取得：Tauri 2.11 殼直嵌 core（非 sidecar）、SQLite 為檔案系統之上的衍生快取（archived_cache_v14 等）、markdown 檔為真相。

## Goals / Non-Goals

**Goals:**

- 交付單一可執行的桌面 app（Tauri 殼直嵌 speclink-core），本地零依賴雙擊即跑。
- 呈現：change 看板/清單、spec 清單、文件內容檢視（proposal/design/tasks/spec）。
- 動詞操作面：list、show、status、validate、analyze、archive，全部經內嵌 core 執行。（park/unpark 不列入——該功能已從 speclink 移除，見 inprogress.rs。）
- 打底一個 React + TypeScript 共用元件庫（看板、文件樹、文件檢視），資料源可抽換，供第 ③ 刀 web GUI 復用。
- markdown 檔（openspec/）維持真相與 git 跟隨；桌面 app 不改變 fs 模式的任何可觀察行為。

**Non-Goals:**

- 對話式 AI agent 面板——歸第 ② 刀（desktop-acp-agent）。
- 任何 remote 模式、web server、PostgreSQL——歸第 ③ 刀起。
- 內容編輯：v1 為唯讀瀏覽＋動詞操作，不提供在 GUI 內直接編輯 markdown。
- git worktree 選擇器（spectra 有 WorktreePickerModal）——v1 不做，留後續。
- 全量 SQLite 鏡像：active changes/specs 不進快取（見 Decision D4）。

## Decisions

### Tauri 殼直嵌 speclink-core，而非 sidecar 呼叫 CLI

桌面 app 以 Tauri command（Rust）直接呼叫 speclink-core 的既有函式，回傳其 `--json` payload 結構給前端。speclink-core 本就是 speclink-cli 的 lib 依賴，直嵌零額外成本、無跨進程序列化開銷、無需維護 CLI 版本對齊。
替代方案：(a) Electron + spawn speclink.exe sidecar——拖入整個 Node runtime、每次操作 spawn 進程、需解析 stdout；spectra 反組譯證實其為直嵌，不採 sidecar。(b) Tauri + sidecar speclink.exe——仍有 spawn 與 stdout 解析成本，且需處理 exe 路徑發現，無收益。

### 前端採 React + TypeScript，封裝為獨立共用元件庫

前端框架選 React + TypeScript，元件（change 看板、文件樹、文件檢視）封裝為獨立 package（packages/ui），資料源以 props/adapter 注入而非寫死 Tauri invoke——桌面注入 Tauri adapter、第 ③ 刀 web 注入 HTTP adapter，同一份元件兩處復用。
替代方案：(a) SvelteKit（spectra 所用）——可行，但 Copilot/Node 生態與第 ④ 刀 SDK 偏 TS/React，統一 React 使跨刀共用摩擦最小。(b) 前端寫死 Tauri invoke——第 ③ 刀 web 無法復用，違背本刀「打底共用元件庫」目標。

### 前端樣式與狀態：Tailwind + shadcn/ui + Zustand

樣式採 TailwindCSS（utility-first，取代手寫 styles.css），設計系統採 shadcn/ui——其元件為原始碼複製進 packages/ui/src/components/ui/ 的 React + Radix 原語（非 npm 依賴），作為跨桌面/web 共用的設計系統基底。領域元件（change 看板、文件樹、文件檢視）以 shadcn 原語與 Tailwind class 重構，維持 props 純呈現與 adapter 解耦不變。app 狀態（選取的 change/spec、載入的文件、動詞結果）採 Zustand 管理，store 落 apps/desktop——共用元件不直接依賴 store，狀態經 props 下傳，守住 packages/ui 的資料源解耦。設計系統放 packages/ui 使第 ③ 刀 web 直接復用同一套 UI，不重做。
替代方案：(a) 手寫 CSS（先前 styles.css）——無設計系統、跨刀不一致、維護散亂。(b) 設計系統放 apps/desktop 專屬——web（③）需自建一套，違背共用目標。(c) 狀態用 React Context/useState——app 狀態跨元件共享時 prop drilling 嚴重，Zustand 為輕量且不侵入共用元件的選擇。(d) shadcn 放 apps/desktop——同 (b)，web 無法復用設計系統。

### 看板式生命週期佈局：欄位分組 × 側滑抽屜 × 拖放歸檔

主畫面從靜態清單改為看板：欄位對應 SDD 生命週期階段（Proposed／In Progress／Ready／Archived），change 卡片依狀態自動歸欄。階段由既有清單資料客戶端派生——totalTasks 為 0＝Proposed（尚在建 artifact）、0<completed<total＝In Progress、completed==total 且 total>0＝Ready（可歸檔）；Archived 來自歸檔清單（cache）。點卡片開右側 shadcn Sheet 側滑抽屜，呈現該 change 的 artifact DAG（status 的 artifacts 與其 done/ready/blocked 狀態）、tasks 清單（解析 tasks.md 的 `- [ ]`／`- [x]` checkbox，唯讀）、與文件內容。拖放（@dnd-kit）：把卡片拖到 Archived 欄 SHALL 彈出確認對話框（shadcn AlertDialog），確認後執行 archive 動詞——僅 archive 有明確的階段↔動作對應，其他階段轉換非單一指令可達故不接受拖放。派生與解析為純函式落 packages/ui、可獨立測試；共用元件維持 props 純呈現與 adapter 解耦，DnD 與抽屜狀態由 apps/desktop 的 Zustand 管理。
替代方案：(a) 沿用靜態卡片清單——無生命週期推進感，使用者反饋不直覺。(b) 拖放觸發所有階段轉換——多數轉換（propose→apply、apply→verify）非單一指令可達，語意勉強且誤拖有風險，故僅 archive 接受拖放。(c) 詳情用獨立全頁——離開看板全局視野；抽屜可快速切換 change 且保留看板背景。(d) tasks 清單改由新增 core command 提供——tasks.md 的 checkbox 解析在客戶端即足夠，不擴張引擎。

### Spectra 風清單佈局取代看板：可展開清單 × 卡內分頁 × 富文本渲染

依使用者提供的 Spectra 2.3.1 實際截圖，主畫面從生命週期看板改為 Spectra 風的可展開變更清單：每張 change 卡片點開後在卡內以 shadcn Tabs 顯示提案／設計／任務／規格分頁，內容以 react-markdown＋remark-gfm 富文本渲染（標題、行內 code pill、GFM checkbox 任務清單、表格），取代先前的裸 `<pre>`。工具列含搜尋與「進行中／已封存」切換；殼為頂欄（專案名／開啟專案佔位）＋左側欄（變更／規格／備忘／設定）。分頁內容經注入的 loader 懶載入（proposal.md/design.md/tasks.md 與 specs/<cap>/spec.md），規格分頁的 capability 清單由新增的唯讀 command change_capabilities 提供。看板元件（KanbanBoard 等）保留於 packages/ui 供未來格狀檢視與 web 復用，但預設檢視為 Spectra 清單。
替代方案：(a) 維持生命週期看板——使用者看過 Spectra 實物後明確偏好清單＋卡內分頁＋富文本，看板的欄位流動非其所需。(b) 詳情用側滑抽屜（前一版）——Spectra 的卡內分頁就地展開更貼近參照且不遮擋清單。(c) 規格分頁改由前端解析 change 目錄——列 capability 需讀目錄，經 core 的 delta_capabilities 唯讀 command 最直接、不在前端重刻檔案系統邏輯。

### 看板為主視圖、Spectra 級詳情面板與細節功能補齊

使用者澄清：要的是看板的功能感（生命週期欄位），Spectra 清單降為次要切換；但詳情要達 Spectra 截圖的細節等級。定案：(1) 主視圖回到 KanbanBoard（Proposed／In Progress／Ready／Archived），工具列提供「看板／清單」切換（看板預設），清單視圖沿用 ChangeList。(2) 點卡片開加寬版詳情抽屜（RichDetailDrawer，shadcn Sheet 寬幅）：標頭含 change 名＋複製名稱鈕＋metadata 列（createdBy／createdWith／created 相對時間／任務數）＋進度條；動作列含 分析／驗證／封存／刪除；內容為 shadcn Tabs 分頁 提案／設計／任務（n/m 紅色計數徽章）／規格（+a ~m delta 計數），各分頁以 Markdown 富文本渲染。(3) 細節功能的資料來源：新增唯讀 command change_meta（回傳 .openspec.yaml 的 createdBy/createdWith/created，camelCase）；規格 delta 計數由前端純函式 specDeltaCounts 解析 delta spec 的 `## ADDED/MODIFIED/REMOVED/RENAMED Requirements` 區段內 `### Requirement:` 數。(4) 新增破壞性 command delete_change：刪除 active change 目錄（僅 active、路徑安全檢查、UI 以 AlertDialog 確認）——對齊 Spectra 的刪除鈕；此為 desktop 層操作，不動 speclink-core 的 Store trait。
替代方案：(a) 維持 Spectra 清單為主——使用者明確要看板功能感。(b) 編輯（Spectra 的編輯鈕）與匯入——v1 維持唯讀 Non-Goal，遞延。(c) 把 meta 塞進 list payload——破壞與 CLI `--json` 同形狀的契約，另立唯讀 command 較乾淨。(d) delete 加進 speclink-core Store trait——引擎無此動詞（CLI 亦無 delete 子指令），屬 GUI 管理操作，落 desktop 層即可。

### 詳情互動任務與版面精修：任務勾選/排序回寫 tasks.md、封存獨立頁、彩色 delta 與活化看板

使用者八點回饋定案：(1) 詳情抽屜寬度全程流動（max(720px,42vw)、上限 95vw），另設全螢幕切換鈕；(2) 移除看板/清單切換——看板即變更主視圖，封存改為獨立頁（列表式：搜尋＋日期＋名稱＋複製），由頂欄「已封存 N」進入、側欄「變更」返回看板；(3) 規格 delta 計數上色（+新增=綠、~修改=琥珀、-移除=紅）；(4)(5)(6) 任務分頁改為互動元件：checkbox 可勾選/取消、每列上下移動排序，兩者經新增的桌面 command set_task_done／move_task 直接改寫 tasks.md 的對應 checkbox 行（1-based 序數定位、僅動 checkbox 行、群組標題不動），寫後重載並刷新清單計數——GUI 與 tasks.md 保持單一真相；(7) 分頁加 icon（提案/設計/任務/規格）；(8) 看板活化：欄位頂部彩色飾條與 icon（提案中=紫、進行中=琥珀、已就緒=綠）、計數彩色徽章、卡片 hover 浮起、進度條隨階段配色。
替代方案：(a) 任務勾選經 speclink task done CLI 語意——只能單向標完成、無法取消勾選，直接改寫 checkbox 行才能雙向；(b) 拖曳排序——上下鈕更可靠且無 dnd 與點擊衝突的前科，先做按鈕排序；(c) 封存留在清單視圖分頁——使用者要求獨立一頁，入口更直接。自由文字編輯（Spectra 的編輯鈕）仍遞延——勾選與排序是結構化寫入，非開放編輯。

### 桌面透過內嵌 core 的既有 payload 供資料，不新增引擎邏輯

GUI 所需資料（change 列表、spec 列表、status DAG、文件內容、analyze/validate 結果）皆已存在於 core 的既有 payload builder（list/show/status/validate/analyze 的 `--json`）。桌面的 Tauri command 層僅是薄包裝：呼叫 core、回傳既有結構。動詞的可觀察行為（欄位、值、錯誤）SHALL 與 CLI 一致——本刀不改 core 呈現、不在 core 加 ANSI 或 GUI 專用邏輯（守 core/cli 邊界紅線）。
替代方案：為 GUI 新增專用 core API——多數需求既有 payload 已滿足，僅在缺口處（見 Open Questions）補唯讀 query，不預先擴張。

### SQLite 快取限縮於歸檔清單，active changes/specs 即時讀

active changes 與 specs 數量在單一專案內為個位到數十量級，每次開啟/刷新直接經 core 讀 markdown 即時完成，不需快取。SQLite 快取僅用於歸檔（archive）清單——歸檔量隨時間無上限成長，每次全量重解析浪費。此決定對齊 spectra（其 SQLite 亦以 archived_cache 為主）並守「禁止過度設計」紅線：不快取本已即時的東西、不背負全量鏡像的 schema migration 負擔（spectra 的 archived_cache 已迭代到 v14）。快取為衍生、可刪除重建，帶單一 schema 版本欄位；真相恆為 markdown 檔。
替代方案：(a) 完全不做快取——歸檔量大時列表開啟變慢，故保留歸檔快取。(b) 全量鏡像 active+archived 進 SQLite——為即時可得的 active 資料付出 migration 與同步成本，過度設計。(c) SQLite 當 Store 真相——脫離「markdown git 跟隨」的情境 4 定義，且第 ④ 刀 4→3 遷移需重做。

### 桌面 app 落於 apps/desktop/，新增 JS 工作區

新增 apps/desktop/（Tauri app：src-tauri/ 為新 Cargo workspace member，前端於 apps/desktop/src/）與 packages/ui/（共用 React 元件庫），並於 repo 根建立 JS 套件工作區（npm workspaces）納管 apps/desktop 與 packages/ui。既有 crates/ 佈局不動；speclink-core 由 src-tauri 以路徑依賴引用。
替代方案：桌面 app 塞進 crates/——crates/ 語意是 Rust library/binary，Tauri app 含前端資產，另置 apps/ 語意更清楚且不干擾既有 workspace members 列表。

## Implementation Contract

**可觀察行為**：使用者雙擊桌面 app 啟動後，在無需另裝任何服務的情況下，於本地 openspec/ 專案根看到：(1) change 看板/清單（含各 change 的 proposal/tasks 狀態）；(2) spec 清單；(3) 點選任一 change 或 spec 顯示其 markdown 文件內容；(4) 對選定 change 執行 validate/analyze/status 並看到結果、執行 archive 並看到清單更新。

**介面與資料形狀**：
- Tauri command 層（Rust，src-tauri）暴露唯讀查詢與動詞指令，每個回傳對應 core 既有 `--json` payload 的 camelCase 結構——例如列出 changes 回傳與 `speclink list --json` 同形狀、status 回傳與 `speclink status --change X --json` 同形狀（含 applyRequires/artifacts）。動詞欄位、值、失敗訊息 SHALL 與對應 CLI 指令一致。
- packages/ui 元件以注入的 data adapter 取得資料（介面：list changes、list specs、get document、run verb），桌面提供 Tauri adapter 實作；元件不直接依賴 Tauri 全域。

**失敗模式**：非 openspec/ 專案目錄啟動時，app 顯示明確的「非 speclink 專案」空狀態而非崩潰；動詞失敗（如 analyze 有 Critical、archive 前置未滿足）時，將 core 的錯誤訊息與非 0 語意呈現於 UI，不靜默吞掉。

**驗收標準**：
- Rust 側：src-tauri 的 Tauri command 有單元測試，斷言其回傳結構與對應 core payload 一致（至少涵蓋 list/status/show 三項與一個動詞 archive）。
- 前端側：packages/ui 元件以 vitest 測試（看板渲染給定 change 列表、文件檢視渲染給定 markdown、動詞按鈕觸發 adapter 呼叫）。
- 端到端：以 fs 模式既有回歸專案為資料，手動啟動 app 確認四項可觀察行為成立；CLI 的 fs 模式輸出回歸對照（parity_suite/color_suite/twin harness）維持通過——本刀不觸碰 core 呈現，應天然不變。
- 可打包為單一平台可執行檔（Windows 為先，Tauri bundle）。

**範圍邊界**：
- 在範圍：Tauri 殼、src-tauri command 薄包裝層、packages/ui 元件庫與 adapter 介面、桌面 Tauri adapter、上述 7 動詞的唯讀/操作面、歸檔清單的 SQLite 快取、Windows 打包。
- 不在範圍：AI agent（② 刀）、remote/web/Postgres（③ 刀起）、GUI 內編輯、worktree 選擇器、macOS/Linux 打包簽章流程（可後續補）、core 新增非唯讀的引擎邏輯。

## Risks / Trade-offs

- [引入 JS 工具鏈與 Tauri 使 build 複雜度上升] → 侷限於 apps/ 與 packages/，既有 cargo workspace 與 CI 的 Rust 測試不受影響；JS 測試獨立於 vitest。
- [直嵌 core 若誤將 GUI 專用邏輯滲入 core，破壞 core/cli 邊界] → Tauri command 僅薄包裝既有 payload，任何新查詢限唯讀且不含呈現；以 Rust 測試斷言回傳與 core payload 同形狀鎖住邊界。
- [SQLite 歸檔快取與檔案真相不同步（手動改動歸檔目錄）] → 快取可刪除重建、帶版本欄位；提供重建路徑，且真相恆為檔案，快取僅為列表加速。
- [跨平台差異（路徑、換行、Tauri WebView 依賴）] → v1 先鎖 Windows 交付並驗證，路徑經 core 既有跨平台處理；macOS/Linux 打包列後續。
- [React 元件庫的 adapter 介面若設計過窄，第 ③ 刀 web 無法復用] → adapter 介面以 core 領域語彙（change/spec/document/verb）定義而非 Tauri 專屬，第 ③ 刀僅換 HTTP 實作。

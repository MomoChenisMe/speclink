## Context

規格頁（SpecList）與已封存頁（ArchivedList）目前以行內收合／展開呈現內容：每列自帶 expanded 狀態、首次展開懶載入、規格列另有 refreshGen 世代快取。變更頁則以 Sheet 抽屜（RichDetailDrawer）呈現詳情，兩套閱讀心智模型並存。封存變更列展開後的四分頁（提案／設計／任務／規格）與 RichDetailDrawer 的分頁結構同構，內容元件（SectionedDoc、TaskList readOnly、DeltaSpecView、RoundsView/ConclusionView）皆已存在且可重用。

資料面：SpecItem 僅 id＋modifiedAt、ArchivedItem 僅日期／名稱／任務數；收合卡片為懶載入，前端在收合狀態下沒有內容可供計算，任何卡片新資訊都必須由 speclink-desktop-core 的清單 payload 帶出。封存清單經 SQLite 衍生快取加速（cache.rs，CACHE_VERSION=2，版本不符即重建）。專案分頁徽章＝changeStage 為 in-progress 的變更數（tabs.ts:inProgressCount），背景分頁取 project_stats_at 快照的 inProgressChanges。

本設計實作已結論討論 spec-archive-drawer-ux 的決議；同討論扇出的「看板卡片解剖學統一」屬後續變更，不在此處。

## Goals / Non-Goals

**Goals:**

- 規格頁與已封存頁的內容檢視統一為抽屜模式，行內展開移除。
- 三種收合卡片（規格／封存變更／封存討論）帶出足以判斷份量與狀態的資訊，資料由 Rust 端清單 payload 供給。
- 專案分頁徽章語意改為「待收尾數」，活躍與背景分頁的更新機制沿用現行架構。
- 新做的卡片採「標題文字後緊跟複製鈕」版面。

**Non-Goals:**

- 看板卡片（ChangeCard／DiscussionColumn）的改造——後續變更。
- desktop-ux-polish 在途任務的調整——另經 ingest。
- RichDetailDrawer 唯讀模式、行內展開與抽屜並存、前端預讀全文——討論中已否決。
- 新引擎動詞、CLI 子指令、設定欄位——一律不加；crates/speclink-core 與 crates/speclink-cli 不動。

## Decisions

### D1：唯讀抽屜成對新建，重用既有內容元件

SpecDrawer.tsx（規格：正典全文＋溯源 footer）與 ArchivedDrawer.tsx（封存：以 discriminated target 同檔承載「封存變更」四分頁與「封存討論」背景／討論過程／結論兩型檢視）各自成件，共用 Sheet 原語與 RichDetailDrawer 的寬度樣式（w-[max(720px,42vw)] max-w-[95vw]、全螢幕切換 w-[96vw]）。內容元件全數重用：SectionedDoc、TaskList readOnly、DeltaSpecView、splitDiscussionSections＋RoundsView／ConclusionView。

封存變更 target 的 header 比照 RichDetailDrawer 的同源連結：sourceDiscussions（{ slug, topic }[]，App 端自 store 解析）非空時顯示可點 chips，點擊經 onOpenDiscussion(slug) 回呼由宿主把 detailArchived 換成 { kind: "discussion", slug }——同一抽屜殼切換內容，記錄載入沿用 getDiscussionDocument 的 live 優先、封存後備語意（2026-07-11 真實視窗驗證回饋：封存變更抽屜原本無法連至討論）。封存討論 target 不顯示 chips。

替代方案：RichDetailDrawer 加 readOnly 旗標——否決，該元件綁滿 change 專屬互動（動詞列、任務勾選拖曳、樂觀更新、刪除），唯讀分支會滲透全件；封存討論獨立第三個抽屜檔——否決，與封存變更共享開閉狀態與寬度殼，同檔兩型較薄。

### D2：抽屜狀態比照 detailChange 接進 store

store 新增 detailSpec（capability id 或 null）與 detailArchived（{ kind: "change", datedName } | { kind: "discussion", slug } | null），開閉 action 與 detailChange 同款；App.tsx 掛載兩個抽屜並接線。列元件（SpecList／ArchivedList）降級為純資訊卡＋onOpen 回呼，自身不再持有 expanded／loaded／doc 狀態。

替代方案：App.tsx local state——可動，但與 detailChange 的接線風格分岔，且 store 已是抽屜狀態的既定家（drawerVerb 亦掛於此）。

### D3：懶載入與世代重載搬進抽屜

抽屜開啟（或換目標）時清空並全量載入；refreshGen 遞增時不清空、latest-wins 序號防交錯後單次替換——與 RichDetailDrawer 的 loadAll 模式同款。SpecRow／ArchivedRow 現有的行內懶載入與世代快取邏輯隨行內展開一併移除。

替代方案：保留列內快取、抽屜只讀列的已載內容——否決，卡片收合時本就不載入，快取無源；兩處快取兩套失效規則反而更複雜。

### D4：清單 payload 擴欄位，卡片資訊由 Rust 端計算

speclink-desktop-core 擴充（--json 欄位 camelCase，serde rename；欄位為新增、向後相容）：

- list_specs_at（query.rs）：每筆規格加 requirementCount（spec.md 內 `### Requirement:` 標題數）、purposeExcerpt（Purpose 區段首個非空行，原文帶出）、purposeTbd（bool，偵測 archive 產生的「TBD - created by archiving」佔位）、traceCount（全文 @trace 標記的 source 去重數）。
- archived_changes_at（cache.rs）：每筆封存變更加 specCount（specs/ 下 capability 目錄數）、createdBy（.openspec.yaml 的 created_by，缺席為 null）、fromDiscussions（來源討論 slug 陣列，缺席為空陣列）。
- 封存討論卡的衍生變更數自既有 promoted_to 欄位長度派生，討論清單 payload 不需擴欄位。

TBD 佔位偵測放 Rust 端（單一真相、可單元測試釘住），前端只消費 purposeTbd 旗標。

替代方案：前端展開時計算——否決，收合卡無內容可算；前端預讀全文——否決，封存清單量大（現況 39 筆×4 檔）啟動成本不可接受。與「storage 解耦」方向的關係：欄位計算走既有 list 函式的檔案讀取路徑，不新增儲存假設。

### D5：封存快取版本遞升與重建

archived_changes_at 的 SQLite 快取因新欄位入庫，CACHE_VERSION 2→3；版本不符即整表重建為既有語意，首次開啟自動重掃。不做 v2→v3 的就地遷移——重建成本（一次目錄掃描）遠低於遷移程式的維護成本。

替代方案：新欄位不入快取、每次清單另行計算——否決，快取存在的理由就是避免逐筆開檔，欄位不入庫等於快取失效。

### D6：頁籤徽章改「待收尾數」

派生規則：待收尾數＝changeStage 為 ready 的變更數＋status 為 concluded 的討論數（promoted 不計——已轉出即非等待）。tabs.ts 的 inProgressCount 改為 pendingWrapUpCount(changes, discussions)；活躍分頁隨看板刷新派生（資料流不變）。project_stats_at（project.rs）加 pendingWrapUp 欄位供背景分頁快照，既有 inProgressChanges 欄位移除（唯一消費者是 store.ts，同變更內一併改）。tooltip 文案改為待收尾語意；openspec/LANGUAGE.md 收新詞「待收尾」（定義：等使用者執行動詞的卡片＝已就緒變更＋已結論未轉出討論）。

替代方案：拿掉徽章——否決（多專案並用，背景分頁需要切換訊號）；數字改圓點——否決（「在飛」非行動訊號）；保留 inProgressChanges 欄位並存——否決（app 自有介面無外部消費者，留雙欄位是死碼）。

### D7：卡片版面與資訊欄位

三種卡片統一版面：標題＋複製鈕成一個 flex 群組（標題 min-w-0 truncate、複製鈕 shrink-0、hover 顯現＋copied 打勾回饋沿用），群組吃 flex-1，meta 資訊靠右。計數 meta 文法統一（2026-07-11 真實視窗驗證回饋：三卡樣式分歧）：需求數、溯源變更數、觸及規格數、衍生變更數一律「裸 icon＋數字」（inline-flex、無 pill 底色、無圓圈 Badge），tooltip 與 aria-label 保留在地化全文；任務數徽章維持 pill＋配色分級——它是狀態徽章（全完成靜默／未全完成琥珀警示），與計數語意不同。各卡欄位：

- 規格卡：標題（id）＋複製鈕｜meta＝需求數徽章、溯源變更數、相對修改時間；purposeTbd 時以琥珀色「Purpose 待補」提示取代摘要，否則顯示 purposeExcerpt 一行截斷。
- 封存變更卡：日期＋標題（name）＋複製鈕｜meta＝任務徽章（全完成維持靜默樣式、未全完成改琥珀警示——「沒做完就封存」才是需要被看見的異常）、觸及規格數、createdBy 頭像圓點（tooltip 全名，與 ChangeCard 同款）、來源討論 icon（MessageSquareText，tooltip 列 slug）。
- 封存討論卡：日期＋標題（topic）＋補上複製 slug 鈕｜meta＝「N 輪」、衍生變更數徽章。

i18n 新鍵一律 zh-TW 與 en 兩語系字典同步（鍵集合相等為既有規格約束）。

替代方案：卡片維持單列擠入全部資訊——否決，欄位變多後單列必然互相截斷；描述（Purpose 摘要）獨立成第二列與看板討論卡的 topic 描述列同型。

## Implementation Contract

**行為**：

- 規格頁點擊任一規格卡（整列可點）開啟規格抽屜：呈現正典 spec.md 全文與溯源 footer；卡片不再有 chevron 與行內展開。已封存頁點擊封存變更卡開啟四分頁唯讀抽屜（提案／設計／任務／規格；任務核取方塊 disabled、無工具列），點擊封存討論卡開啟背景／討論過程／結論唯讀檢視；文件缺席顯示既有空狀態文案。兩抽屜寬度與全螢幕切換行為與變更詳情抽屜一致。
- 帶來源討論的封存變更抽屜於標題下方顯示來源討論 chips（topic 文字、缺席不顯示），點擊後同一抽屜切換為該討論的唯讀檢視；封存討論檢視不顯示 chips。
- 三種卡片的計數 meta 統一「裸 icon＋數字」樣式（無 pill 底色、無圓圈數字）；任務數徽章維持 pill＋配色分級。
- 抽屜開啟後外部修改文件，refreshGen 遞增時抽屜內容單次替換且不重置分頁與捲動；互動語意與變更詳情抽屜相同。
- 收合卡片在不展開任何內容的前提下顯示 D7 所列欄位；Purpose 為佔位符的規格卡顯示「Purpose 待補」琥珀提示。
- 專案分頁徽章顯示待收尾數：看板上有 2 個已就緒變更與 1 份已結論未轉出討論時徽章顯示 3；全部收尾後徽章歸零。背景分頁於 app 啟動時查得快照值，之後保留最後已知值（既有語意）。

**介面／資料形狀**（camelCase）：

- 規格清單項：{ id, modifiedAt?, requirementCount, purposeExcerpt, purposeTbd, traceCount }。
- 封存清單項：{ datedName, date, name, tasksTotal?, tasksDone?, specCount, createdBy, fromDiscussions }。
- project_stats_at 回傳含 pendingWrapUp（number），不再含 inProgressChanges。
- 前端 adapter（packages/ui/src/adapter.ts）的 SpecItem／ArchivedItem 介面同步擴欄位；tabs.ts 匯出 pendingWrapUpCount(changes, discussions)。
- ArchivedDrawer 增 props：sourceDiscussions?: { slug, topic }[]（App 端自 store.archived 以 datedName 查 fromDiscussions、topic 自 discussions 兩節以 slug 解析、缺席退回 slug 原文）與 onOpenDiscussion?: (slug: string) => void（宿主以 openArchived({ kind: "discussion", slug }) 實作切換）。

**失敗模式**：

- 清單欄位計算遇不可讀／缺席檔案：計數欄位為 0、excerpt 為 null，清單照常回傳——不因單筆壞檔讓整頁失敗（與既有清單容錯一致）。
- 抽屜文件載入失敗或缺席：對應分頁顯示空狀態文案，不報錯。
- 快取版本不符：整表重建（既有路徑），使用者感知為首次開啟稍慢。

**驗收**：

- cargo test -p speclink-desktop-core：新欄位計算（requirementCount／purposeExcerpt／purposeTbd／traceCount／specCount／createdBy／fromDiscussions）、快取 v3 重建、pendingWrapUp 派生的單元測試。
- npm test -w packages/ui：規格卡／封存卡欄位渲染、Purpose 待補提示、點列開抽屜、抽屜分頁內容與空狀態、複製鈕位置（標題群組內）與剪貼簿寫入；封存變更抽屜來源討論 chips（顯示 topic、點擊以 slug 回呼、缺席不渲染）；卡片計數 meta 統一樣式（需求數與衍生變更數元素無 pill／Badge 類、含 icon）。
- npm test -w apps/desktop：store 抽屜狀態、pendingWrapUpCount 派生、stats 快照接線。
- 真實視窗手動驗證（CLAUDE.md GUI 備忘）：點卡開抽屜、全螢幕切換、外部改檔後抽屜內容更新、頁籤徽章隨封存動作歸零。

**範圍邊界**：in scope＝上述行為與資料形狀、i18n 兩語系新鍵、LANGUAGE.md「待收尾」條目；out of scope＝看板卡片改造、desktop-ux-polish 任務調整、引擎（crates/）任何改動、規格頁與已封存頁的搜尋行為（維持現狀）。

## Risks / Trade-offs

- [快取 v3 重建使首次開啟已封存頁變慢] → 重建為一次目錄掃描（現況 39 筆），既有「快取遺失時重建」路徑已涵蓋，可接受。
- [Purpose 佔位偵測綁定 archive 產生器的文案字串] → 偵測邏輯與佔位文案同在 Rust 側，以單元測試釘住兩者一致；文案若變動測試即紅。
- [與 desktop-ux-polish 在途變更的檔案交集（App.tsx／store.ts）] → 本變更不動看板搜尋與拖曳區塊，交集面窄；實作期間以 speclink drift 檢查，後落地者負責 rebase。
- [jsdom 測不出抽屜開閉與捲動互動] → 依 CLAUDE.md GUI 備忘保留真實視窗手動驗證步驟，列入 tasks。
- [移除 inProgressChanges 欄位屬 app 內部破壞性改動] → 唯一消費者 store.ts 同變更內改寫；app 自有 IPC 無外部相容性負擔（CLI --json 輸出不受影響，回歸對照不動）。

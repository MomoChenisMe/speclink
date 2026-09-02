## Context

變更 manual-skill 讓技能把正式規格轉成 `openspec/manual/*.md`，頁格式由 in-flight 的 `manual-pages` 契約定義：kebab-case 檔名、frontmatter 六欄（title／section／order／keywords／sources／generated）、GitHub Alert 內文、頁尾出處行。討論 `manual-generation-skill` 定案：呈現一致性由 desktop 的「手冊」頁承擔。

現況：desktop 側欄五項（變更、規格、已封存、專案設定；設定沉底），主畫面由 `apps/desktop/src/App.tsx` 切視圖；資料經 `packages/ui/src/adapter.ts` 的 `SpeclinkDataSource` 介面，本機由 `apps/desktop/src/adapter/tauriDataSource.ts` 呼叫 Tauri command，command 單行委派到 `apps/desktop/core/src/query.rs` 等純邏輯（`init_core_context` 取得 store）。watcher 監看整個 spec 目錄樹（`watch_targets_at` 的第一個目標即 `spec_dir()`），`openspec/manual/` 天生在範圍內。共用 Markdown 元件為 react-markdown ＋ remark-gfm ＋ remark-breaks，`skipHtml`，閱讀欄 96ch。desktop core 已依賴 serde_yaml 與 regex。規格頁的卡片支援 `onOpen` 展開單一規格。

## Goals / Non-Goals

**Goals:**

- 側欄新增「手冊」項，手冊頁以 frontmatter 機械推導側欄樹、搜尋、上一頁／下一頁；內頁沿用共用 Markdown 與閱讀欄。
- 過期頁與「生成後新增且未入冊」的規格在側欄可見。
- 出處 capability 可點跳規格頁並展開該卡。
- GitHub Alert 語法以提示框呈現，語意色分層。
- 外部寫入手冊目錄後秒級重載。

**Non-Goals:**

- 不做手冊寫入、編輯或生成——手冊頁唯讀；生成在 manual 技能。
- 不做 remote 模式的手冊讀取（PM 無 checkout）：remote 資料源回報尚不支援，頁面呈空狀態；投影列於討論 Deferred。
- 不做全文搜尋——搜尋只比對標題與 keywords。
- 不做 frontmatter 驗證或修復——缺欄位的頁寬容降級，不報錯、不改檔。
- 不動 CLI、speclink-core、system tray。

## Decisions

### 手冊查詢落在 desktop core 的 manual 模組

新增 `apps/desktop/core/src/manual.rs`，兩個 root-based 純函式：`list_manual_pages_at(root)` 回傳索引 JSON（`present`、`pages`、`uncoveredNew`、`reason`），`manual_page_at(root, slug)` 回傳去掉 frontmatter 的內文。Tauri 殼各一行委派；不進 speclink-core（手冊讀取是桌面呈現需求，非引擎語意），也不經 store 抽象（v1 只有本機路徑，remote 投影未定）。替代方案「前端直接讀檔」被排除：桌面規則是觸及檔案系統的 command 不佔主執行緒且邏輯在可獨立測試的 core。

### 索引 JSON 的形狀與推導規則

`pages` 每項：`slug`、`title`、`section`、`order`、`keywords`（陣列）、`sources`（陣列）、`generated`、`stale`（布林）。排序：依 `order` 升冪；分區順序為分區內最小 `order`；同 `order` 以 `slug` 決斷。缺 `title` 用 slug、缺 `section` 歸「其他」、缺或非整數 `order` 置該分區末（以 `i64::MAX` 排序、`order` 輸出 null）、缺 `generated` 視為未過期。frontmatter 無法解析（不以 `---` 開頭或 YAML 錯誤）的檔案列入 `pages` 時 `title` 為 slug、其餘缺席，並於索引的 `malformed` 陣列列出 slug——寬容降級，SHALL NOT 拋錯。`present` 為 `openspec/manual/` 目錄存在且至少一個 `.md`。

### 過期與未入冊的計算依 manual-pages 契約

`stale` ⇔ `sources` 中任一 capability 的正典 spec 內所有 `@trace` 區塊 `updated:` 的最大日期晚於該頁 `generated`；`sources` 為空或規格不存在時不判過期。`uncoveredNew` ⇔ 正典 capability 的最小 `@trace updated` 晚於全手冊最大 `generated`，且不在任何頁的 `sources`——這是「生成後新增」的機械近似，不做使用者面向分流（那是技能的判斷）。`@trace` 的 `updated` 以 regex 自 HTML 註解區塊擷取，沿用規格頁 footer 對 `@trace` 的讀法。

### 側欄樹、搜尋與上下頁在前端由索引推導

`packages/ui/src/components/ManualPage.tsx` 接索引與內文載入函式：側欄依分區分組、每頁一列（stale 者附「可能過期」標記）；搜尋列以大小寫不敏感子字串比對 `title` 與 `keywords`，命中頁的分區保留、其餘隱藏；上一頁／下一頁為排序序列的相鄰頁（首頁無上一頁、末頁無下一頁）。內文以共用 Markdown 渲染；頁尾出處行的 capability 名以既有規格卡的 `onOpen` 路徑跳規格頁展開該卡。索引底部在 `uncoveredNew` 非空時顯示「手冊生成後新增且未入冊的規格 N 份」提示。

### GitHub Alert 以共用 Markdown 元件的內建轉換呈現

在 `packages/ui/src/components/Markdown.tsx` 加一個小型 remark 轉換（不新增依賴）：blockquote 首段以 `[!NOTE]`／`[!TIP]`／`[!WARNING]`／`[!CAUTION]` 開頭者，轉為帶類型 class 的提示框並移除標記文字；四型配色取自介面狀態語意色（資訊、成功、警告、危險），不佔主色。對所有 Markdown 檢視恆開：既有內容無此語法者輸出不變。替代方案「引入第三方 remark 外掛」被排除：三十行內可完成，且不增加相依審核面。

### 手冊頁的外部變更重載沿用既有 watcher 事件

`openspec/manual/` 已在監看樹內；store 收到帶 root 的檔案變更事件後，若手冊視圖活躍則重取索引，並重載目前開啟頁的內文；重載回應交錯時以最新為準。不新增監看目標。

### 側欄第六項與零分頁行為與既有五項同型

「手冊」項插於「規格」之後、「已封存」之前；恆常渲染、單純切頁、零分頁時呈與變更頁相同的空狀態引導頁；無障礙標籤「手冊」。i18n 鍵加入 `packages/ui/src/i18n.tsx` 與 `apps/desktop/src/i18n/messages.ts` 的 zh-TW 與 en 字典，鍵集合維持相等。

## Implementation Contract

**可觀察行為**

- 側欄六項由上而下：變更、規格、手冊、已封存、專案設定；設定沉底。點「手冊」進入手冊頁，該項高亮。
- 有手冊時：左側分區樹（依 `order`）、頂部搜尋列、右側內文、頁尾上一頁／下一頁與出處行；點出處 capability 切至規格頁並展開該卡；過期頁列上有「可能過期」標記；索引底部在有未入冊新規格時顯示計數提示。
- 無 `openspec/manual/` 或其中無 `.md`：主內容顯示空狀態文案（說明尚無手冊、可用 manual 技能從規格生成）。remote 資料源：空狀態文案為「remote 模式尚不支援手冊」。
- 外部寫入手冊目錄後數秒內索引與開啟頁重載。
- 任何 Markdown 檢視中 `> [!NOTE]` 等四型 blockquote 呈現為提示框。

**介面與資料形狀**

- `SpeclinkDataSource.listManualPages(): Promise<ManualIndex>`，`ManualIndex = { present: boolean; reason?: "remote"; pages: ManualPageItem[]; uncoveredNew: string[]; malformed: string[] }`；`ManualPageItem = { slug; title; section; order: number | null; keywords: string[]; sources: string[]; generated: string | null; stale: boolean }`。
- `SpeclinkDataSource.getManualPage(slug: string): Promise<string | null>`（去掉 frontmatter 的內文；不存在回 null）。
- Tauri command：`list_manual_pages(root)`、`get_manual_page(root, slug)`，各單行委派至 `speclink_desktop_core::manual::{list_manual_pages_at, manual_page_at}`。

**失敗模式**

- 目錄不可讀：索引 `present: false`，畫面空狀態，錯誤只記日誌。
- 單頁 frontmatter 壞：列入 `malformed`、以 slug 為標題照常可開；不報錯。
- 內文載入失敗：內容區顯示載入失敗文案，側欄照常。

**驗收**

- `cargo test -p speclink-desktop-core manual`：frontmatter 解析、排序、缺欄降級、stale 與 uncoveredNew 判定、壞頁寬容。
- `npm test -w packages/ui`：ManualPage 的分區樹、搜尋、上下頁、stale 標記、出處點擊回呼、空狀態；Markdown 的四型提示框與無語法內容不變。
- `npm test -w apps/desktop`：側欄六項順序、切頁高亮、零分頁空狀態、i18n 鍵集合相等、外部事件觸發重取。
- 手動：於本 repo 以 manual 技能生成手冊後開 desktop 檢視；外部改一頁後數秒內更新。

**範圍邊界**：in scope＝上述七個決策所列。out of scope＝手冊生成與寫入、remote 讀取、全文搜尋、frontmatter 驗證、CLI 與引擎。

## Risks / Trade-offs

- [側欄六項改動 desktop 快照與 App 測試] → 同批更新 `apps/desktop/src/__tests__/App.test.tsx` 的側欄斷言；規格頁、已封存頁行為不變。
- [Markdown 提示框轉換誤觸既有文件] → 只在 blockquote 首段精確以四種標記開頭時轉換；vitest 以無標記 blockquote 斷言輸出不變。
- [跨平台：Windows 路徑與 CRLF] → frontmatter 解析以行為單位、剝除 `\r`；slug 取自檔名不含分隔符；core 測試以 tempdir 建目錄。
- [大型手冊每次事件全量重取索引] → 索引只讀 frontmatter 與規格的 `@trace` 行，不讀內文；v1 接受。
- [speclink-desktop（Tauri 殼）測試需 sidecar 與 server-web dist] → 本變更不動 protocol；殼層驗證以 `cargo check -p speclink-desktop` 為主，行為測試落在 core 與前端。

## Migration Plan

1. 合入後無資料遷移；手冊頁在無 `openspec/manual/` 時呈空狀態。
2. 回滾：移除側欄項與 manual 模組即可，無持久化狀態。

## Open Questions

- 無。手冊格式若日後變動，由 `manual-pages` 契約先改、本頁跟隨。

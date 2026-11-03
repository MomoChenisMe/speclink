## Context

討論 herdr-speclink-panel 定案由引擎解析討論結論的刀、各處讀同一份；本變更是第 4 刀（最後一刀），依賴的第 2 刀 plan-unpromoted-cuts 已封存。現況：

- 引擎 crates/engine/speclink-core/src/lifecycle/discuss/cuts.rs 的 `parse_cuts(conclusion)` 回傳 `Cut { id, title, names, names_in_bold }`，`Cut::unpromoted(promoted_to)` 判定尚未轉出的變更名；只有 `plan` 的 `unpromoted_cuts` 使用。`Cut` 沒有 `Serialize`，也不回傳首行其餘的說明文字與列點原文
- 結論文字由 discuss.rs 的私有函式 `conclusion_body` 取出（去掉 HTML 註解）；`live_conclusions` 讓結論與 `promoted_to` 取自同一次讀取
- 桌面的三種來源都以原始 markdown 取得記錄：local 經 Tauri `discussion_document` → speclink-desktop-core `discussions::discussion_document_at`（`show_discussion`，live 優先、已封存為後備）；remote 經 `remote_discussion_document` → `RemoteWorkspace::discussion_document` → GET /discussions/{slug}，只取 `content`；已封存討論頁走同一個 `getDiscussionDocument`
- server 的 `routes::show_discussion` 回 `ShowDiscussionResponse { info, content }`（crates/protocol/speclink-protocol/src/query.rs）；CLI `discuss show` 的 remote 模式只取 `info` 與 `content` 重建輸出，fs 與 remote 的 `--json` 鍵集合由 crates/adapters/speclink-cli/tests/it/remote_read_path.rs 釘住
- 前端 packages/ui/src/discussionModel.ts 的 `cutCard`、`parseCuts`、`cutPromotion` 自 Decision 的細節列抽刀（`CutCard { id, title, names, namesInBold, items }`），`items` 為首行其餘的說明加上列點巢狀內容拆成的細線列；消費者為 ConclusionDoc.tsx（`DecisionSections`、`CutCardView`、`cutPromotionOf`）、discussionDocSummary.tsx（`discussionDocData`、數字磚、`unpromotedCallout`）與 PromotedDoc.tsx（依刀分組）
- 兩邊規則已有差異：前端認 `*`、`+`、編號列點與列點外的粗體段落，引擎只認頂層 `- ` 列點；引擎先去 HTML 註解；同一份記錄有兩個 `**Decision**:` 標籤時前端只取最後一個欄位，引擎讀過兩個

## Goals / Non-Goals

**Goals:**

- 刀的判定（哪些列是刀、識別籤、範圍句、變更名籤、首行說明、尚未轉出的變更名）由引擎做一次，桌面詳情頁、`plan` 與路線圖讀同一份
- local、remote、已封存三種來源都帶出引擎解析的刀
- 刀卡、提示框、數字磚與衍生變更分組的畫面不變；移除前端的分刀解析

**Non-Goals:**

- CLI `discuss show`、`discuss list` 的輸出
- 分刀判定規則本身；刀卡巢狀子項的解析（照舊是一般 markdown 列表）
- 路線圖、Herdr 端

## Decisions

### 引擎多回傳首行說明與列點原文

`Cut` 增兩欄：

- `note`：首行其餘的說明文字。範圍句取自粗體時，為粗體之後的全部文字（去掉開頭冒號），外圍若恰為一整組 `（…）` 或 `(…)`（可巢狀）則拿掉；否則為第一個「——」之後的文字（去頭尾空白）；都沒有時為空字串。規則與目前前端 `cutCard` 的 `note` 相同，移植時以前端既有的測試列對照
- `line`：該刀列點的首行原文——去掉 `- ` 之後、去頭尾空白的文字（HTML 註解已去掉）

`parse_cuts` 的刀判定、`id`、`title`、`names`、`names_in_bold` 不變，`plan` 的 `unpromotedCuts` 不受影響。

替代方案：引擎連巢狀子項一起拆成細線列——要在 Rust 再寫一套 markdown 列表拆分，而子項只是呈現、不是判定；前端已有通用的列表拆分。

### 一份記錄的刀以單一入口計算

discuss.rs 新增純函式 `record_cuts(text: &str, promoted_to: &[String]) -> Vec<RecordCut>`：以 `conclusion_body` 取結論（沒有結論時回空），`parse_cuts` 後逐刀轉成 `RecordCut`（cuts.rs，`Serialize`、camelCase）：`cut`（識別籤原文）、`title`、`note`、`line`、`changes`（變更名籤）、`changesInBold`（布林）、`unpromoted`（`Cut::unpromoted(promoted_to)`）。呼叫端傳入的 `text` 與 `promoted_to` SHALL 來自同一次讀取的同一份記錄（`store.read_discussion` 的 `text` 與其 head 的 `promoted_to`），與 `live_conclusions` 同一原則。

欄名沿用 `plan` 的 `unpromotedCuts`（`cut`、`title`、`changes`、`unpromoted`），讓兩處讀的人看到同一組詞。替代方案：前端拿 `changes` 與 store 的 `promotedTo` 自己算尚未轉出——又是一份判定，且兩份 `promoted_to` 來自不同次讀取。

### protocol 與 server：詳情回應增列 cuts

`ShowDiscussionResponse` 增 `#[serde(default)] cuts: Vec<DiscussionCut>`；`DiscussionCut` 為 camelCase 的 protocol 型別（`cut`、`title`、`note`、`line` 字串，`changes`、`unpromoted` 字串陣列，`changesInBold` 布林），每欄 `#[serde(default)]`，讓新 client 讀舊 server 時缺鍵讀作空值；JSON Schema 匯出測試（`query_dtos_export_json_schema`）隨型別更新。`routes::show_discussion` 在既有 `Command::DiscussShow` 之後，以 `record_cuts(&content, &info.head.promoted_to)` 計算並逐欄搬進回應——內容與 `promoted_to` 都來自同一次 `DiscussShow`，server 不重讀檔、不另寫解析。live 與已封存記錄走同一條路由，兩者都有 `cuts`。

唯一實作落點是引擎的 `record_cuts`；server 與 speclink-desktop-core 都只呼叫它。

### CLI 輸出不變

crates/adapters/speclink-cli/src/verbs/discuss.rs 的 remote 模式照舊只自回應取 `info` 與 `content` 重建 `DiscussShowOutcome`，fs 模式不經 server；兩種模式的 `discuss show --json` 逐位元不變，由既有的 remote_read_path.rs 鍵集合測試守住。CLI 程式不改。

### 桌面讀記錄時一併取得刀

- speclink-desktop-core：`discussions::discussion_document_at(root, slug)` 改回 `Option<DiscussionDocument>`，`DiscussionDocument { content, cuts: Vec<RecordCut> }`（camelCase，JSON 為 `{ content, cuts }`）；以一次 `store.read_discussion` 取 `text` 與 head 的 `promoted_to` 呼叫 `record_cuts`。slug 安全檢查與 live 優先、已封存後備的解析不變
- Tauri 殼：`discussion_document` command 單行委派、回傳型別跟著改；`RemoteWorkspace::discussion_document` 自 `ShowDiscussionResponse` 取 `content` 與 `cuts` 組成同形的 JSON（404 照舊為 None）
- 前端：packages/ui/src/adapter.ts 新增 `DiscussionCut` 與 `DiscussionDocument { content: string; cuts: DiscussionCut[] }`，`getDiscussionDocument(slug)` 改回 `Promise<DiscussionDocument | null>`；tauriDataSource.ts、remoteDataSource.ts 照傳，`cuts` 缺席時補空陣列；`DiscussionDetailPage` 的 `loadDocument` 與 `ArchivedDetailPage` 的 `loadDiscussionDocument` 型別跟著改，App.tsx 的組裝不變

### 前端以列點原文把刀對回細節列

discussionModel.ts 移除 `CUT_ID_RE`、`tickNames`、`unwrapParens`、`cutCard`、`parseCuts`、`cutPromotion`，新增：

- `cutCards(details: ListItem[], cuts: DiscussionCut[]) → { cuts: CutCard[]; rest: ListItem[] }`：依 `cuts` 的順序，為每把刀找第一個尚未被認領、`text` 去頭尾空白後等於 `line` 的細節列；認領到的列自 `rest` 移除，卡的 `items` 為 `note`（非空時）接該列巢狀內容的細線列（沿用既有的 `blockRows(parseListItems(body))`）；找不到對應列的刀仍成卡，`items` 只有 `note`
- `CutCard` 改為 `{ id, title, names, namesInBold, unpromoted, items }`（`id` 取 `cut`、`names` 取 `changes`、`namesInBold` 取 `changesInBold`）
- `cutPromotion(cut) → "promoted" | "partial" | "none" | "unknown"` 只讀引擎欄位：`namesInBold` 為 false 時 unknown；`unpromoted` 為空時 promoted；`unpromoted` 與 `names` 等長時 none；其餘 partial

替代方案：引擎回傳刀在細節列中的序號——前端的細節列含 `*`、`+`、編號列點與段落，序號對不齊；以識別籤比對——又要在前端辨識粗體與識別籤，等於保留解析。

### 消費端改讀引擎的刀

- `ConclusionDoc` 增 `cuts: DiscussionCut[]` prop（預設空陣列），`DecisionSections` 以 `cutCards(splitVerdict(md).details, cuts)` 取刀卡與定案細節；`CutCardView` 的整刀未轉出與部分轉出改用 `cutPromotion(cut)`，部分轉出時「尚未轉出」籤接在 `unpromoted` 內的變更名之後；變更名籤的階段籤與點擊照舊來自 `stageOf`
- `discussionDocData` 的輸入 `text` 改為 `doc: DiscussionDocument | null | undefined`，`cuts` 以 `cutCards` 算出；`conclusionTiles` 的「定案細節」計數改用 `cutCards(...).rest.length`；`unpromotedCallout` 列出 `cutPromotion(cut) === "none"` 的刀，已封存討論照舊不出提示框
- `PromotedDoc` 的介面不變（吃 `CutCard[]`），只隨型別調整
- `DiscussionDetailPage`、`ArchivedDetailPage` 把文件的 `content` 交給既有的切分、把 `cuts` 交給 `ConclusionDoc` 與 `discussionDocData`

沿用的既有元件與工具：`splitVerdict`、`parseListItems`、`blockRows`、`CutCardView`、`StatusPill`、`Callout`、`CommandChip`、`PromotedDoc`；不新增元件。

## Implementation Contract

**行為**

- 討論詳情頁（live 與已封存、local 與 remote）的刀卡、「分刀」區段、「定案細節」計數、「分刀」數字磚、「N 刀尚未轉出」提示框與衍生變更分組，與本變更前對同一份以 `- ` 列點寫刀的記錄呈現相同
- 以 `*`、`+`、編號起頭的刀列或列點外的粗體段落，改留在定案細節，不成刀卡、不進提示框——與 `plan --json` 的 `unpromotedCuts` 一致
- remote 連到沒有 `cuts` 的舊 server：討論詳情頁沒有「分刀」區段與提示框，刀列留在定案細節
- `speclink discuss show <slug> --json`（fs 與 remote）與 `discuss list --json` 的輸出逐位元不變

**介面**

- 引擎：`discuss::record_cuts(text, promoted_to) -> Vec<RecordCut>`；`Cut` 增 `note`、`line`
- GET /discussions/{slug}：`{ info, content, cuts }`；`cuts` 每項 `{ cut, title, note, line, changes, changesInBold, unpromoted }`
- 桌面 local／remote 的討論記錄回應：`{ content, cuts }`（同上的每項形狀）；不存在時 null
- TS：`DiscussionDocument`、`DiscussionCut`；`getDiscussionDocument(slug): Promise<DiscussionDocument | null>`；`cutCards(details, cuts)`、`cutPromotion(cut)`

**失敗模式**

- 記錄沒有 `## Conclusion` 或沒有刀：`cuts` 為空陣列，畫面同現在的「沒有分刀」
- 舊 server 回應缺 `cuts` 鍵或刀的某欄：讀作空陣列／空字串／false，不報錯
- 引擎回傳的刀在細節列中找不到對應列（例：首行含 HTML 註解）：仍畫刀卡，卡內只有說明文字；該列同時留在定案細節
- slug 含路徑穿越：桌面照舊回 null；server 照舊 404

**驗收**

- cargo test -p speclink-core：`note` 對照前端既有測試列（「——」之後、粗體內有範圍詞時拿掉外圍括號、沒有說明時為空）；`line` 為去掉 `- ` 的首行；`record_cuts` 的 `unpromoted` 與 `changesInBold`
- cargo test -p speclink-protocol：`ShowDiscussionResponse` 缺 `cuts` 讀作空陣列、`DiscussionCut` 缺欄讀作空值、JSON Schema 匯出含 `cuts`
- cargo test -p speclink-server --test it：GET /discussions/{slug} 的 live 與已封存記錄都帶 `cuts` 且值與 `record_cuts` 相同
- cargo test -p speclink-cli --test it：既有 remote_read_path.rs 的 `discuss show --json` 鍵集合測試照舊通過
- cargo test -p speclink-desktop-core：`discussion_document_at` 回 `{ content, cuts }`、live 與已封存皆然、路徑穿越回 None
- cargo test -p speclink-desktop --lib：remote 的 `discussion_document` 帶出 `cuts`、舊 server 缺鍵為空
- vitest（packages/ui）：`cutCards` 的認領、`rest`、找不到對應列、`items` 組成；`cutPromotion` 四種判定；ConclusionDoc、DiscussionDetailPage、ArchivedDetailPage、PromotedDoc 既有的刀卡、計數、提示框與分組測試改以引擎形狀的 `cuts` 餵入後照舊通過；`*` 列點的刀列留在定案細節；`cuts` 為空時沒有分刀區段
- vitest（apps/desktop）：remoteDataSource 讀到 `{ content, cuts }`、App 的討論詳情頁照常載入

**範圍**

- 範圍內：speclink-core 的分刀解析與 `record_cuts`、speclink-protocol、speclink-server 的討論詳情路由、speclink-desktop-core、Tauri 殼的討論記錄 command 與 remote、@speclink/ui 的討論詳情相關元件與模型、apps/desktop 的資料源
- 範圍外：speclink-cli 程式、`plan` 輸出、路線圖、tray、server-web、Herdr

## Risks / Trade-offs

- [以 `*`、`+`、編號列點寫刀的舊記錄不再出刀卡] → 刻意對齊 `plan` 的判定；討論技能一律以 `- ` 寫刀，repo 內既有記錄在實作時以 grep 確認沒有這種寫法（有則記在 tasks 的排查結果）
- [前端以首行原文比對，HTML 註解或空白差異會對不上] → 引擎與前端都去頭尾空白；對不上時仍畫卡、不丟內容，只是巢狀子項留在定案細節
- [舊 server 沒有 `cuts`] → 讀作空陣列、刀列退回定案細節；提示框不出現，不會誤報
- [回歸對照] → CLI 的 `discuss show --json`、`discuss list --json`、`plan --json` 不變，golden 不動；remote_read_path.rs 的鍵集合測試守住 fs 與 remote 的 `discuss show` 一致；desktop-core 與 Tauri 殼的討論記錄測試改斷言新形狀
- [跨平台] → 純文字解析與 serde，Windows／macOS／Linux 相同；`[0-9]` 代替 `\d` 的既有處理不變
- [前端測試改餵入形狀] → 既有測試以原始文字餵入，改為文字加手寫的引擎形狀 `cuts`；以一個測試輔助函式產生，避免每個測試各寫一份

## Migration Plan

- 無資料遷移；server 與桌面可分開升級：新桌面連舊 server 時刀退回定案細節，舊桌面連新 server 時忽略 `cuts`
- 退版：還原前端的分刀解析即可，server 多出的 `cuts` 沒有其他讀者

## Open Questions

(none)

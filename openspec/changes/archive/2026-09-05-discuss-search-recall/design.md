## Context

討論記錄有固定骨架（Context／Rounds／Conclusion），每輪以 `### Round N — <mode> (<date>)` 起頭，輪內的 `**Ruled out**:` 行與 Conclusion 的 `**Decision**:`、`**Rejected alternatives**:`、`**Deferred**:` 行是技能模板規定的固定格式。引擎已有 `Store::list_live_discussions` 與 `Store::list_archived_discussions`，兩者回傳含全文的 `DiscussionDoc`；`discuss list` 與 `discuss show` 走命令層 `Command::DiscussList`／`DiscussShow`，server 的 `GET /discussions` 與 remote client 的 typed 方法共用同一 Command，CLI remote 模式以 `to_discussion_info` 把 wire 型別轉回引擎型別後共用同一個 renderer。

現況缺口：沒有任何動詞能依關鍵字找出記錄裡的決定行。既有 `GET /search` 是桌面全域搜尋，只搜在途記錄全文、每卡只回首個命中且語意綁桌面本地搜尋，不能拿來當定案查核。技能側 discuss 只查在途討論以接續，improve 則 list 之後逐筆 show。

限制：既有 `discuss list`／`show` 的人眼與 `--json` 輸出必須逐位元不變；技能 asset 改動連動 ASSET_VERSION、render golden 快照與 assets.lock；remote 模式與 sqlite／postgres 後端下不存在本機檔案，搜尋必須經引擎。

## Goals / Non-Goals

**Goals:**

- 一個引擎動詞 `speclink discuss search`，本機與 remote 同語意，找出在途與封存記錄中 topic、slug 與四種決定行的關鍵字命中。
- discuss 技能開場多一段「舊討論查核」並把結果放進假設清單的第四類對照；improve 技能的防重提檢查改用同一動詞。
- 新增全部為 additive：既有動詞輸出與回歸對照不動。

**Non-Goals:**

- 不改 `GET /search`（桌面全域搜尋）的語意或範圍。
- 不做索引、模糊比對、同義詞或語意搜尋；不做 AND 模式（`--all`）——留待命中率實測後再議。
- 不進 Node SDK dispatch（現不覆蓋任何 discuss 動詞）與 desktop UI。
- 不改討論記錄格式，既有記錄不需遷移。
- 關鍵字含空白的比對不支援（CLI 位置參數與 server 的 `q` 皆以空白切詞）。

## Decisions

### D1 搜尋演算法落在 speclink-core 的 discuss 模組，不加 Store 方法

搜尋函式放在 `crates/speclink-core/src/discuss.rs`，輸入 `&dyn Store` 與關鍵字清單，資料來源為既有 `list_live_discussions` 與 `list_archived_discussions`（皆含全文），不新增 Store trait 方法。理由：store-fs、sqlite、postgres 三種後端已經透過這兩個方法回傳全文，引擎端掃描即可覆蓋所有後端；加 Store 方法會逼三個 driver 各寫一次比對。替代方案「在 speclink-fs 用檔案 grep」被否決：remote 與資料庫後端無檔可讀，且違反「領域演算法歸 speclink-core」的邊界。

### D2 命中範圍限定 topic、slug 與四種決定行，以固定標記辨識

比對對象：frontmatter 的 topic 與 slug；輪內以 `**Ruled out**:` 起頭的行；Conclusion 區內以 `**Decision**:`、`**Rejected alternatives**:`、`**Deferred**:` 起頭的行；以及每個決定行之後緊接的條列行（`- `、`* `、`+ `、`N. ` 起頭，直到第一個非條列行為止）——實測封存記錄慣用「標記獨占一行、內容寫在下一行條列」（22 行 Ruled out／Rejected alternatives 如此），只比標記行會漏掉這些定案；每個命中的條列行各為一筆 match，kind 與 where 同其標記行。其他行（Focus、Position、Open、Evidence、散文）一律不比對。輪號取該行所在的最近一個 `### Round N` 標題；Conclusion 區的行位置記為 conclusion；topic 與 slug 記為 frontmatter。比對方式為雙方 `to_lowercase` 後的子字串包含；多關鍵字任一命中即算。理由：實測封存 121 筆，drawer 全文命中 27 筆但決定行只有 6 筆，其餘是 Evidence 提到檔名的雜訊；決定行是模板固定格式，引擎自己寫入所以知道形狀。替代方案「全文命中」與「只比 topic」分別因雜訊與漏報被否決。

### D3 命中結果重用 DiscussionInfo，排序 topic／slug 命中優先

單筆命中的形狀為既有 `DiscussionInfo` 加一個 `matches` 陣列；每個 match 帶 `kind`（topic／slug／ruled-out／decision／rejected／deferred）、`where`（frontmatter／round-N／conclusion）與 `text`（該行原文，去前後空白）。排序：有 topic 或 slug 命中的記錄排前，其餘其後；兩群內各依 created 由新到舊，同日依 slug 字典序；同一記錄內的 matches 依文件順序。理由：重用 DiscussionInfo 讓 CLI remote 模式沿用既有 `to_discussion_info` 轉換，人眼輸出也能沿用 list 的欄位；排序把「題目就是這個」的記錄放最上面。

### D4 命令層新增 DiscussSearch，server 端點以單一 q 參數空白切詞

`Command::DiscussSearch { terms: Vec<String> }` 與 `CommandOutcome::DiscussSearch(Vec<hit>)` 進命令層，列入覆蓋表查詢動詞；不發領域事件（唯讀）。server 新增 `GET /discussions/search?q=<以空白分隔的關鍵字>`，handler 以空白切詞後呼叫同一 Command；`q` 缺席或全空白回 400、reason 為 `invalid_argument`。protocol 新增 `SearchDiscussionsResponse { hits: Vec<DiscussionHit> }`，`DiscussionHit` 為 wire 版 `DiscussionInfo` 加 `matches`；remote client 新增 typed 方法 `search_discussions(terms)`，把關鍵字以空白接起放進 `q`。CLI remote 分支呼叫 typed 方法後以 `to_discussion_info` 轉回引擎型別，與本機共用同一個 renderer。理由：axum 的 `Query` 預設不解析重複鍵為 Vec，單一 `q` 不需新依賴，且與既有 `GET /search` 的 `q` 慣例一致；唯一實作落點在引擎，兩端只組裝輸入。

### D5 CLI 子指令形狀與輸出

`speclink discuss search <關鍵字>... [--json] [--no-color]`：位置參數至少一個（clap `required`，缺席時 clap 以非零 exit code 於 stderr 印用法）；不吃 stdin。人眼輸出：零命中印 `No discussions match "<關鍵字以空白接起>".`；有命中時第一行 `Discussions matching "<關鍵字>":`，每筆一行 `  • <slug> [<status>, archived|live] (<created>) — <topic>`，其下每個 match 一行 `      <where> <kind>: <text>`；`--no-color` 下無 ANSI。`--json` 輸出 `{ "hits": [ { ...DiscussionInfo 欄位, "matches": [ { "kind", "where", "text" } ] } ] }`，零命中為 `{ "hits": [] }`；exit code 皆為 0。引擎收到空關鍵字清單時以錯誤中止（防止 server 路徑繞過 clap）。

### D6 技能文字：discuss 偵察加舊討論查核，improve 防重提改用 search

`crates/speclink-core/assets/skills/discuss.md`：Step 2 漏斗改為「正典 → 舊討論查核 → 程式碼」，舊討論查核以使用者題目的關鍵字加正典轉譯出的英文詞執行 `speclink discuss search`；命中的決定行全數列出，整份 Conclusion 以 `speclink discuss show` 最多讀 3 份、topic 命中者優先。Presenting assumptions 的對照表加第四列「舊討論已定案」，細分曾否決（附當時理由；重開須說明該理由已失效）、曾延後（可接手）、已落地（正典會照出，不重列），並重申不得以此擋下方向。Context 段規定加一行 `Prior discussions: <slug 清單>`（零命中時寫 none）。`improve.md` Step 1 的三行指令改為 `speclink discuss search <範圍關鍵字>... --json` 加 `speclink list --json`，閱讀順序規定同範圍的舊 improve 記錄（kind 為 improve）排前。ASSET_VERSION 由 v1.27.0 升至 v1.29.0，五份 golden 快照與 assets.lock 同批再生。替代方案「技能文字自己用 Grep 工具掃 openspec/discussions/」被否決：remote 與資料庫後端無檔可掃，四種工具形狀的搜尋能力不一。

### D7 不重用桌面全域搜尋端點

`GET /search` 只搜在途記錄、每卡回首個命中、語意與桌面本地搜尋綁定；把它擴到封存並回多筆決定行會改變桌面契約。定案查核走獨立端點，兩者各守各的語意。

## Implementation Contract

**行為**

- 本機：`speclink discuss search drawer` 對含 `**Ruled out**: RichDetailDrawer 加 readOnly 旗標…` 行的封存記錄 spec-archive-drawer-ux 回一筆命中，match 的 kind 為 ruled-out、where 為 round-1（依實際輪號）、text 為該行原文。
- 多關鍵字：`speclink discuss search golden sse` 回所有 topic、slug 或決定行含 golden 或 sse（不分大小寫）之記錄。
- 條列定案：`**Ruled out**:` 獨占一行、其下 `- 把 drawer 拿掉` 的記錄，`speclink discuss search drawer` 回一筆 kind 為 ruled-out、text 為該條列行的 match；標記行之後的空行、散文或下一個 `**Field**:` 行結束該區塊。
- 非決定行不算：僅 Evidence 或 Position 含關鍵字的記錄不出現在結果。
- remote：同一 workspace 綁定 server 後執行同指令，stdout 的人眼與 `--json` 與本機同形（既有 remote 分歧只在 `path` 缺席與 promotedTo／concluded 多欄，沿用 list 的既定分歧）。
- 既有 `discuss list`、`discuss show` 的 stdout、stderr、exit code 逐位元不變。

**介面**

- 引擎：`discuss::search(store: &dyn Store, terms: &[String]) -> Result<Vec<DiscussionHit>>`；`DiscussionHit { info: DiscussionInfo, matches: Vec<DiscussionMatch> }`；`DiscussionMatch { kind: String, where_: String, text: String }`，serde 欄位名 `kind`、`where`、`text`。
- 命令層：`Command::DiscussSearch { terms: Vec<String> }`、`CommandOutcome::DiscussSearch(Vec<DiscussionHit>)`。
- server：`GET /discussions/search?q=<terms>`，回 `SearchDiscussionsResponse { hits }`，需綁定 scope 與既有讀取權限。
- protocol：`SearchDiscussionsResponse`、`DiscussionHit`（wire 版含 `promotedTo`、`concluded` 選填欄位，與 list 的 `DiscussionInfo` 對齊）、`DiscussionMatch`，皆 camelCase、JsonSchema 派生。
- remote client：`search_discussions(&self, terms: &[String]) -> Result<SearchDiscussionsResponse, RemoteError>`。
- `--json` payload：`{ "hits": [ { "slug", "topic", "status", "rounds", "created", "createdBy"?, "kind"?, "path", "archived", "matches": [ { "kind", "where", "text" } ] } ] }`。

**失敗模式**

- CLI 無關鍵字：clap 拒絕，非零 exit code，stderr 印用法。
- 引擎收到空或全空白關鍵字清單：以錯誤中止；server 映射為 400 `invalid_argument`。
- 記錄讀取失敗或格式不完整（無輪標題、無 Conclusion 區）：該記錄仍以 topic／slug 參與比對，缺的區段視為零決定行，不使整個查詢失敗。
- remote 認證失效與離線：沿用既有 typed client 錯誤分類（remote-connection 規格），不另立訊息。

**驗收**

- speclink-core 單元測試（discuss.rs 內 tests）：topic 命中、slug 命中、ruled-out 命中帶正確輪號、conclusion 三種行命中、不分大小寫、多關鍵字 OR、非決定行不命中、排序（topic 命中優先、created 由新到舊）、空關鍵字錯誤。
- CLI 整合測試 `crates/speclink-cli/tests/it/discuss_search.rs`：人眼與 `--json` 各一、零命中 exit 0、無關鍵字非零 exit、remote mock 下輸出與本機同形（沿 remote_verb_parity.rs 的 mock server 模式）。
- server 測試（discussion_routes.rs）：在途與封存各一筆命中、`q` 缺席回 400 invalid_argument。
- remote typed client 測試（typed_client.rs）：`search_discussions` 打 `GET /discussions/search?q=` 且回應反序列化為 typed 型別。
- render golden：五份快照與 assets.lock 更新後 `cargo test -p speclink-core --test it render_golden` 綠。
- 回歸：`crates/speclink-cli` 既有 discuss 相關整合測試全綠。

**範圍界線**

- In：引擎搜尋函式、命令層、CLI 子指令與 renderer、server 端點、protocol 型別、remote typed 方法、兩份技能 asset、版號與 golden。
- Out：Node SDK dispatch、desktop、`GET /search`、索引／模糊／語意／AND 模式、討論記錄格式。

## Risks / Trade-offs

- [render golden 五份快照與 assets.lock 一起變，diff 大] → 屬刻意變更；先跑 `speclink update` 再生後以 golden 測試對照，commit 時盤點 git status 確認 32 份再生 SKILL.md 一併帶上。
- [CLI 回歸對照：既有 list／show 輸出] → 新增子指令不動既有 renderer；既有 discuss 整合測試作為守門。
- [Windows 換行與路徑] → 比對前不依賴換行符（逐行以 `lines()` 切）；`path` 欄位沿用既有 `to_slash`；測試不做絕對路徑期望值。
- [Unicode 大小寫] → `to_lowercase` 對中日文無作用、對拉丁字母正確；不做 NFC 正規化（記錄與關鍵字皆來自同一輸入法慣例）。
- [關鍵字含空白不支援] → CLI 與 server 皆以空白切詞，文件與人眼用法說明中標明。
- [封存 121 筆逐檔掃描] → 每筆數 KB、一次查詢毫秒級；不做索引。
- [技能文字變長，偵察時間盒被撐開] → 命中的決定行本身就是一行摘要，整份 Conclusion 讀取維持 ≤3 份的上限。

## Migration Plan

新增動詞與端點為 additive，無資料遷移。部署：合併後 `speclink update` 讓各 workspace 取得新版技能檔（ASSET_VERSION v1.29.0）。回滾：移除子指令與端點即可，記錄格式未變。

## Open Questions

（無。AND 模式與同義詞比對依討論結論明列為延後事項，待命中率實測。）

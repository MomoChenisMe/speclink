## Context

討論 custom-review-rules 定案兩刀。第一刀 review-rules-config 已封存：config.yaml 的 `review` 區塊（`baseline` 與 critical／warning／suggestion 三清單）由 speclink-core 的 `ReviewConfig` 解析，審查技能開審前以 `speclink workflow-config show --json` 讀取，呈現開頭印一行套用規則；引擎以 `REVIEW_BASELINE_SMELLS` 單一提供 12 條 smell 名稱，desktop 設定頁經 settings 快照取得。

現況的缺口：套用規則只出現在執行審查的對話裡。工單 `review.md` 的輪次文法（speclink-core quality station 模組的 `parse_round_body`）只認 `**Scope**:`、`**Phase**:`、`**Patch**:` 與 findings 行，其餘行被略過；`Round::to_json` 是 CLI `--json`、desktop 本機工單分頁共用的唯一序列化，server 的 `ReviewRoundDto` 是同形的型別化鏡射，CLI remote 路徑以 `to_station_round` 把 DTO 轉回 `Round`。desktop 的工單分頁元件 `TicketView` 在 `@speclink/ui`，資料由 desktop-core 的 `ticket_json`（本機活變更、本機封存、遠端封存原文解析）與 Tauri 殼的 `remote_station_ticket`（遠端活變更）提供。

外部輸入：討論 config-skill-rule-routing 決定，沒有專案規則時「套用規則」列只寫「預設 12 條 code smell」，不顯示三個 0 計數——審查規則是選用的，大多數專案沒有專案規則。

## Goals / Non-Goals

**Goals:**

- 審查工單首輪記錄當時套用的 baseline 模式與專案規則原文，引擎能檢查格式並解析。
- `review show --json` 的每輪帶出 `rules`，local、remote CLI、typed client 與 server 同形。
- desktop 審查分頁在記錄了規則的那一輪顯示「套用規則」列，可展開查看 12 條名稱與專案規則原文。

**Non-Goals:**

- 驗證站不記錄套用規則：驗證站沒有專案規則；它共用同一份解析與序列化，所以 `verify show --json` 的 `rules` 恆為 `null`，驗證技能不改。
- 不在工單寫 12 條 smell 名稱，也不寫規則指紋（討論第 9 輪已否決）。
- 複驗輪不記錄規則：複驗不重新套用規則，只判前輪 finding。
- 不改「審查呈現標明套用的規則」：呈現開頭行照舊只出現在對話中、不寫入工單；本刀寫入工單的是結構化的 `**Rules**:`／`**Rule**:` 行，兩者是不同的東西。
- 不改蓋章、失效判定與 `--accept` 機制；規則記錄不影響章。
- 不改 config 技能與 `openspec/config.yaml` 的欄位。

## Decisions

### 規則行由審查技能寫入，引擎只檢查格式與解析

技能在 discovery 輪的 add-round stdin 中寫入規則行，內容取自開審前讀到的 `review` 物件。這份物件就是 Standards 軸簡報實際帶入的規則，所以記錄與判定一致。

替代方案：由 `review add-round` 在引擎端讀 config 自動補上。否決理由：add-round 執行時的 config 可能已和開審時不同，記錄會與實際套用的規則不一致；而且 remote 模式下 add-round 在 server 端執行，讀的是 server 的 config，增加耦合。討論結論也指定由技能寫入。

### 文法：一行模式、每條規則一行，只能出現在 discovery 輪

- `**Rules**: extend` 或 `**Rules**: replace`：每輪最多一行，值必須是這兩個 token 之一（區分大小寫）。
- `**Rule**: <SEVERITY> <規則原文>`：SEVERITY 為 CRITICAL、WARNING、SUGGESTION 之一，之後至少一個空白，再接非空的規則原文（去頭尾空白）。
- 有 `**Rule**:` 時同輪必須有 `**Rules**:`。
- 帶 `**Rules**:` 的輪必須是 `**Phase**: discovery`；validation 輪與 legacy 輪（沒有 Phase）帶此行一律拒絕。
- 規則行的順序：同一嚴重度內依出現順序保留，即清單順序。

每行都以 `**Rule` 開頭，不會被當成 findings 行（`- [`）或輪次分隔（`## `）。`**Rules**:` 與 `**Rule**:` 的前綴互不包含（前者在 `Rule` 後接 `s`），解析不會混淆。

替代方案：單行記條數（`**Rules**: extend; critical=2; …`）——討論第 9 輪已改為存全文，因為之後改規則會讓展開時讀到的目前設定被誤標為首輪設定。一行 YAML 或 JSON——手寫難讀，且工單其餘部分都是行式文法。

規則原文含換行時，技能以單一空白連接成一行。config 讀取時條目已去頭尾空白，只有手寫 YAML 區塊純量才會帶中間換行，這種情況少見；改寫只影響顯示，不影響判定（判定已在開審時完成）。

### 沿用 speclink-core 的 ReviewConfig 當作輪次的規則型別

`Round` 新增 `rules: Option<ReviewConfig>`；`ReviewBaseline` 新增 `parse`，與既有的 `as_str` 放在一起，成為 baseline token 的唯一來源。`ReviewConfig` 已 derive `Serialize`，欄位名都是單字，序列化即是 `{ baseline, critical, warning, suggestion }`，與 `--json` 的 camelCase 契約一致。`Round::to_json` 新增 `rules` 鍵，沒有規則時輸出明確的 `null`，比照 `phase`／`patchHash`。

替代方案：在 station 模組另定一個 `AppliedRules` 型別。否決理由：欄位與語意和 `ReviewConfig` 完全相同，另定一份會出現兩個 baseline 解析點。

### 唯一實作落點與各端接法

| 端 | 落點 | 做法 |
| --- | --- | --- |
| 文法與序列化 | speclink-core quality station 模組 | `parse_round_body` 解析與守門、`Round::to_json` 序列化；本機 CLI、server 解析、desktop 本機與封存工單都經這裡 |
| wire 形狀 | speclink-protocol `ReviewRoundDto` | 新增 `rules: Option<ReviewRulesDto>`（`#[serde(default)]`，舊 server 缺席即 `None`）；`ReviewRulesDto` 為 `{ baseline: String, critical, warning, suggestion: Vec<String> }` |
| server | speclink-server 的 `review_round_dto` | 由 `Round.rules` 對映，驗證站端點同一支函式 |
| CLI remote | speclink-cli 的 `to_station_round` | DTO 轉回 `Round`；baseline token 不認得時比照 phase／severity 回報「這支 CLI 比 server 舊」 |
| desktop | desktop-core `ticket_json` 與遠端活工單 | 見下一個決策 |

回歸對照：crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs 比對 local 與 remote 的 `review show --json` 同形。

### desktop 從工單 payload 取得 12 條名稱

desktop-core 的工單 payload 由 `{ rounds }` 改為 `{ rounds, reviewBaselineSmells }`，`reviewBaselineSmells` 取自 `speclink_core::skills::REVIEW_BASELINE_SMELLS`。組 payload 的邏輯收成 desktop-core 的一個函式：`ticket_json`（本機活變更、本機封存、遠端封存原文）直接用它，Tauri 殼的 `remote_station_ticket` 也改為把遠端 rounds 交給它組裝，殼層只做委派。兩站的 payload 都帶這個欄位，不依站別分支；`TicketView` 只在輪次有 `rules` 時才用到。

替代方案：
- 由 `TicketView` 的宿主傳入 settings 快照的名稱——詳情頁不會載入 settings 快照，要多一次讀取，遠端還要經 server。
- 新增一支 Tauri 指令與兩個資料源方法——為一個靜態常數多一條讀取路徑。
- 寫進工單——討論已否決。

這與 settings 快照帶 `reviewBaselineSmells` 是同一種做法：需要名稱的 payload 由 desktop-core 帶入引擎常數，前端只顯示、不另存。顯示的是 desktop 內建引擎的名稱，不是當時審查技能的版本；名稱只隨引擎版本改變，接受這個差異。

### 套用規則列放在 TicketView 的輪卡內，沿用既有元件

- 位置：輪卡展開後、範圍路徑之前，只在該輪 `rules` 非 `null` 時渲染。依文法只有 discovery 輪會有。
- 列頭是 `<button aria-expanded>`，用輪卡卡頭同一種 chevron 展開樣式（TicketView 內抽成一個局部元件，輪卡與規則列共用），預設收合。
- 嚴重度籤與條數籤沿用 `@speclink/ui` 的 `SeverityTag`，與輪卡卡頭的計數籤同一種寫法（為零的不出）。
- 12 條名稱清單：設定頁的審查規則頁（apps/desktop）已有同一份清單樣式。把它抽成 `@speclink/ui` 的 `BaselineSmellList` 元件，設定頁與工單分頁都改用它，不各寫一份。
- replace 且沒有專案規則時的琥珀底，用既有的 WARNING 語意色 token（與 `SeverityTag` 的 WARNING 同一組），不在呼叫端以 className 覆蓋元件變體。
- 文字放在 `@speclink/ui` 的 i18n（tw、en），計數以 `{count}` 代入 `reviewBaselineSmells.length`，不寫死 12。

### 技能版號升為 v1.47.0

`review.md` 第 8 步「Record the round」新增規則行的寫法，`ASSET_VERSION` 由 v1.46.0 升為 v1.47.0，golden 快照與 assets.lock 一併再生，本 repo 的技能檔以 `speclink update` 再生。

## Implementation Contract

**行為**

- 審查技能的 discovery 輪 stdin 依序為 `**Phase**:`、`**Patch**:`、`**Rules**:`、零或多行 `**Rule**:`（依 critical、warning、suggestion 順序，各清單內保持原順序）、`**Scope**:`、findings。未設定 review 區塊時只寫 `**Rules**: extend`。validation 輪不寫規則行。
- `speclink review add-round` 接受上述行；違反文法時 exit code 非零、stderr 點名問題、工單零寫入。錯誤類別與既有內容錯誤相同（CLI invalid content、server `invalid_argument` 400）。
- `speclink review show --json`：每輪的 `rules` 為 `{"baseline":"extend","critical":[…],"warning":[…],"suggestion":[…]}` 或 `null`；人眼路徑照印原文。
- desktop 審查分頁：有 `rules` 的輪顯示「套用規則」列，四種收合文字與展開內容如下表。

| 情況 | 收合時文字（tw） | 條數籤 | 底色 | 展開內容 |
| --- | --- | --- | --- | --- |
| extend、無專案規則 | 預設 12 條 code smell | 無 | 一般 | 12 條名稱、末尾註記 |
| extend、有專案規則 | 預設 12 條＋專案規則 | 非零的嚴重度各一個 | 一般 | 12 條名稱、各嚴重度規則原文、末尾註記 |
| replace、有專案規則 | 只用專案規則（沒有套用預設 12 條） | 非零的嚴重度各一個 | 一般 | 「沒有套用預設 12 條 code smell」、各嚴重度規則原文、末尾註記 |
| replace、無專案規則 | 整份換掉且沒有專案規則：只依 repo 文件審查 | 無 | 琥珀 | 「沒有套用預設 12 條 code smell」、末尾註記 |

末尾註記：「記錄首輪當時的設定；之後改了審查規則，這裡不會跟著變。」

**資料形狀**

- 工單文字：`**Rules**: <extend|replace>`、`**Rule**: <CRITICAL|WARNING|SUGGESTION> <text>`。
- `Round.rules: Option<ReviewConfig>`；JSON 鍵 `rules`。
- `ReviewRoundDto.rules: Option<ReviewRulesDto>`。
- desktop 工單 payload：`{ rounds: TicketRound[], reviewBaselineSmells: string[] }`；`TicketRound.rules: { baseline: "extend" | "replace"; critical: string[]; warning: string[]; suggestion: string[] } | null`。`@speclink/ui` 的 TypeScript 型別把 `rules` 與 `reviewBaselineSmells` 宣告為選填：引擎一定會帶，但既有測試的手寫工單字面值沒有這兩個欄位，宣告為必填會讓範圍外的檔案多出型別錯誤；缺席與 `null` 同義。

**失敗模式**

| 輸入 | 結果 |
| --- | --- |
| `**Rules**: Extend`（大小寫不符）或其他值 | 拒絕，stderr 列出允許值 extend、replace |
| 同輪兩行 `**Rules**:` | 拒絕 |
| `**Rule**:` 但同輪沒有 `**Rules**:` | 拒絕 |
| `**Rule**: NOTE 某規則` | 拒絕，stderr 列出 CRITICAL、WARNING、SUGGESTION |
| `**Rule**: CRITICAL`（沒有原文） | 拒絕 |
| validation 輪或 legacy 輪帶 `**Rules**:` | 拒絕，stderr 說明規則只記在 discovery 輪 |
| server 回傳不認得的 baseline | CLI remote 報錯，不靜默略過 |
| 舊 server 沒有 `rules` | 視為 `null`，desktop 不顯示該列 |

**驗收**

- speclink-core station 單元測試涵蓋上表每一列與成功解析（含 `to_json` 的 `rules` 與 `null`）。
- CLI 整合測試：review_verbs.rs 驗 add-round 拒絕與 show --json；remote_verb_parity.rs 驗 local 與 remote 同形。
- server 整合測試：review_api.rs 驗 `GET /changes/{name}/review` 帶出 `rules`。
- desktop-core 單元測試：payload 帶 `reviewBaselineSmells` 且與 `REVIEW_BASELINE_SMELLS` 相同。
- `@speclink/ui` 測試：ticketView.test.tsx 驗四種情況的文字、條數籤、展開內容與 `rules: null` 時列缺席；設定頁既有測試在換用 `BaselineSmellList` 後仍通過。
- golden：重新產生後 `cargo test -p speclink-core` 通過。

**範圍**

- 在範圍內：上述文法、序列化、wire 形狀、審查技能第 8 步、desktop 工單分頁與共用的 smell 名稱清單元件、docs/verb-contract 的輪次欄位與內容錯誤說明。
- 不在範圍內：驗證站技能、config 技能、蓋章與失效判定、看板卡片的審查標示、node SDK 型別。

## Risks / Trade-offs

- [golden 與 CLI 回歸對照] `Round::to_json` 多一個鍵，既有測試逐字比對欄位集合的地方會失敗（review_api.rs 與 verify_api.rs 都斷言「只有 index、phase、patchHash、scope、findings」，desktop 的 remote_data.rs 比對遠端工單形狀）→ 在同一任務內補上 `rules` 期望值；驗證站的期望值補 `rules: null`，因為兩站共用序列化。
- [版號對撞] config-skill-rule-routing 第二刀也要升 `ASSET_VERSION` → 兩者不同時實作；若仍並行，合併時重生 golden、assets.lock 與技能檔，不挑邊。
- [舊 server 收到新規則行] 舊版 server 的解析器略過不認得的行，會接受並存下規則行，但 `show` 不回傳 `rules` → desktop 對該工單不顯示列，不會出錯；server 升級後同一份工單即可解析出 `rules`。
- [舊技能寫的工單] v1.46.0 以前的審查技能不寫規則行 → 該工單 `rules` 為 `null`，desktop 不顯示列；這就是「舊工單沒有記錄時不顯示」。
- [名稱版本差] desktop 展開的 12 條名稱來自 desktop 內建引擎，不是審查當時的技能版本 → 名稱只隨引擎版本改變，接受。
- [跨平台] 工單文法只依行首前綴解析，CRLF 由既有 `lines()` 處理；規則原文含非 ASCII 字元（中文規則）照原樣保存，測試用中文規則覆蓋。

## Migration Plan

不需要遷移。既有工單照舊解析，`rules` 為 `null`。技能升版後，既有專案執行 `speclink update` 取得會寫規則行的審查技能。回退時移除規則行的解析即可，已寫入的規則行在舊解析器下會被略過。

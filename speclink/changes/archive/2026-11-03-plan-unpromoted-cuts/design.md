## Context

討論 herdr-speclink-panel 定案「規劃資料進引擎」，第 2 刀讓 `plan` 列出討論結論裡尚未轉出的刀。現況：

- 討論技能規定結論的分刀寫法為 Decision 欄位裡的列點 ``**cut N `change-name`**: 範圍句``；桌面討論詳情頁由 packages/ui/src/discussionModel.ts 的 `cutCard`／`parseCuts`／`cutPromotion` 解析並判定是否轉出（判定看刀的變更名是否在討論的 `promoted_to` 衍生列裡）
- 引擎已有 `discuss::list_discussions(store)`（每份討論帶 `head.promoted_to`）與 `discuss::conclusion_text(store, slug)`（去掉 HTML 註解的 Conclusion 區段）
- `plan::compute`、`compute_with_ranks`、`compute_strict` 都經 `compute_from` 產出 `Plan { waves, changes, next, skipped }`；CLI 本機與 server 的 GET /plan（經 Command gateway）都用它，remote CLI 再經 speclink-remote 的 `plan_report` 把 `PlanResponse` 轉回 `Plan` 後共用同一個渲染
- `plan --json` 的形狀由 change-plan、server-verb-api、client-protocol 三份規格釘住

## Goals / Non-Goals

**Goals:**

- 引擎有一份結論分刀的解析器，規則與桌面現行判定相同，之後第 3、4 刀共用
- `plan` 的 JSON 與人眼輸出列出尚未轉出的刀；本機與 remote 同形
- 舊 server、只讀既有鍵的工具都不受影響

**Non-Goals:**

- 改桌面討論詳情頁（第 4 刀）、桌面路線圖（第 3 刀）
- 推算刀與刀之間的依賴
- 已封存討論、粗體內沒有變更名的刀
- `discuss` 動詞的輸出

## Decisions

### D1 結論分刀解析器放在 speclink-core 的 discuss 模組

新增 crates/engine/speclink-core/src/lifecycle/discuss/cuts.rs，於 discuss.rs 以 `pub mod cuts;` 掛上。純函式 `parse_cuts(conclusion: &str) -> Vec<Cut>`，`Cut { id: String, title: String, names: Vec<String>, names_in_bold: bool }`。不做 I/O、不含 ANSI，CLI、server、之後的桌面都經引擎取用。規則逐條對照 discussionModel.ts 的 `cutCard`：

1. Decision 欄位：Conclusion 內以 `**Decision**:` 起頭的行到下一個欄位標籤行（`**Rationale**:`、`**Rejected alternatives**:`、`**Deferred**:`、`**Capture to**:`、`**Next**:`）之前
2. 刀：Decision 內行首為 `- ` 的頂層列點，首行以 `**` 開頭且有成對的收尾 `**`；粗體內去掉反引號片段、壓縮空白後，以 `cut` 加以數字起頭的編號（不分大小寫；編號延伸到空白或 `，,：:（(` 之前）、`第 <編號> 刀`、或 `刀` 加編號起頭
3. 刀編號 `id`：上述起頭片段的原文（例 `cut 2`、`第 2 刀`、`刀二`）
4. 變更名 `names`：粗體內不含空白的反引號片段，依出現順序去重；粗體內沒有時改取粗體之後的反引號片段，並把 `names_in_bold` 設為 false
5. 範圍句 `title`：粗體內編號之後還有文字（去掉前導空白與 `:：、—–-`）時取該文字；否則取粗體之後去掉前導冒號的文字，遇到第一個「——」就截斷
6. 圍欄：與 docModel.ts 的 `parseListItems` 同用 specSections.ts 的 `fenceFlags` 規則——圍欄行為 3 個以上 ` 或 ~（容忍前導空白）；收尾行要與開頭同字元、長度不短於開頭、其後沒有 info 字串；圍欄行與圍欄內的行都不是列點

替代方案：在 CLI 或 server 各自解析——兩份實作會漂移，且違反「同一對外契約唯一實作落點」。

### D2 plan 在 compute_from 結尾附上尚未轉出的刀

`compute_from` 組完既有欄位後呼叫 `unpromoted_cuts(store)`：對 `discuss::live_conclusions(store)`（未封存、依 `list_discussions` 的順序；每份記錄只讀一次，結論（與 `conclusion_text` 同樣剝掉 HTML 註解）與 `promoted_to` 出自同一份記錄）的結論以 `parse_cuts` 解析，以 `Cut::unpromoted` 判定；`names_in_bold` 為 true 且至少一個名稱不在該討論 `promoted_to` 的刀，產出 `UnpromotedCut { discussion, cut, title, changes, unpromoted }`。`Plan` 新增欄位 `unpromoted_cuts: Vec<UnpromotedCut>`，序列化為 `unpromotedCuts`（`UnpromotedCut` 以 camelCase）。三個 compute 入口都經 `compute_from`，所以 CLI 本機、`--strict-overlap` 與 server 都帶這個欄位。成環時整個 plan 照舊回錯誤，不輸出尚未轉出的刀。一份討論讀不到結論時跳過該討論，不讓 plan 失敗。只需要成環判定的呼叫端（桌面 `set_depends_at` 的 overlay 複檢）改走 `plan::check_cycle`：同一套配置與成環判定，不讀討論、不算重疊。

替代方案：做成獨立動詞或 `discuss` 的新旗標——使用者「挑下一刀」時本來就在看 plan，分開要多跑一支，Herdr 與路線圖也要多一次呼叫。

### D3 人眼輸出加一段 Unpromoted cuts

CLI 的 `render_plan` 在 Skipped 段之後、`next:` 行之前，`unpromoted_cuts` 非空時印粗體標題 `Unpromoted cuts`，每把刀一行 `  • <discussion> <cut>: <unpromoted 以 ", " 相接> — <title>`（`•` 用 cyan、` — <title>` 用 dim；title 為空時省略整段 ` — <title>`）。為空時不印，既有輸出位元級不變。所有樣式經 `color`，`--no-color` 下無 ANSI。

### D4 GET /plan 與 remote 轉換逐欄搬運

speclink-protocol 的 `PlanResponse` 新增 `#[serde(default)] unpromoted_cuts: Vec<PlanUnpromotedCut>`，`PlanUnpromotedCut` 五個欄位（`discussion`、`cut`、`title` 字串，`changes`、`unpromoted` 字串陣列且缺席讀作空）。server 的 `routes::plan` 自引擎報告逐欄搬入；speclink-remote 的 `plan_report` 逐欄搬回引擎的 `UnpromotedCut`，不做額外驗證（這些欄位沒有封閉值域）。舊 server 不帶這個鍵時，新 client 讀作空陣列，CLI 照常輸出、不出 `Unpromoted cuts` 段。JSON Schema 由 schemars 隨型別更新。

## Implementation Contract

**行為**

- `speclink plan --json` 的頂層多 `unpromotedCuts`：例如某討論結論有 ``**cut 1 `a`**`` 與 ``**cut 2 `b`**``、`promoted_to` 只有 `a`，則該鍵為 `[{"discussion":"<slug>","cut":"cut 2","title":"…","changes":["b"],"unpromoted":["b"]}]`
- 人眼輸出多一段：`Unpromoted cuts` 標題與 `  • <slug> cut 2: b — <範圍句>`，位置在 `next:` 行之前
- remote 模式的 `speclink plan` 與本機對同一內容輸出同形，含這一鍵與這一段

**介面**

- `speclink_core::discuss::cuts::parse_cuts(&str) -> Vec<Cut>`；`Cut { id, title, names, names_in_bold }`；`Cut::unpromoted(&[String]) -> Vec<String>`
- `speclink_core::discuss::live_conclusions(&dyn Store) -> Vec<(DiscussionInfo, String)>`
- `speclink_core::plan::check_cycle(&dyn Store) -> Result<(), PlanError>`
- `speclink_core::plan::UnpromotedCut { discussion, cut, title, changes, unpromoted }`，`Plan.unpromoted_cuts`
- `speclink_protocol::query::PlanUnpromotedCut`，`PlanResponse.unpromoted_cuts`
- CLI 旗標、stdin、exit code 不變

**失敗模式**

- 某份討論沒有 Conclusion、沒有 Decision 欄位或沒有刀：不產生項目，plan 照常
- 刀的粗體內沒有變更名：不列出（無從判定）
- 依賴成環：沿用既有錯誤（非零 exit、stderr 成環訊息、stdout 空），不輸出尚未轉出的刀
- 舊 server 缺鍵：client 讀作空陣列，不報錯

**驗收**

- `cargo test -p speclink-core`：`parse_cuts` 的單元測試涵蓋 spec「分刀解析」Example 的每一列；plan 測試涵蓋全部轉出、部分轉出、未轉出、粗體內無名稱、已封存討論、多份討論的順序
- `cargo test -p speclink-cli --test it`：plan_verbs 的 JSON 斷言改為五鍵並斷言 `unpromotedCuts` 的欄位與型別；人眼輸出斷言 `Unpromoted cuts` 段的確切行與 `--no-color` 無 ANSI；無尚未轉出的刀時人眼輸出與改動前相同；remote_plan 斷言 remote 模式同形
- `cargo test -p speclink-protocol`、`cargo test -p speclink-remote --test it`：`PlanResponse` 缺 `unpromotedCuts` 讀作空、帶值時逐欄搬回
- `cargo test -p speclink-server --test it`：plan_api 斷言 GET /plan 帶 `unpromotedCuts` 且與本機計算一致

**範圍**

- 範圍內：speclink-core（cuts.rs、discuss.rs、plan.rs）、speclink-cli（plan 渲染與測試）、speclink-protocol、speclink-remote、speclink-server（GET /plan）；apps/desktop 只有兩處：core/src/manage.rs 的成環複檢改用 `check_cycle`，src-tauri/src/remote.rs 的測試 helper 補 `PlanResponse` 新欄位
- 範圍外：apps/desktop 的其餘部分與 packages/ui（第 4 刀才改討論詳情頁）、桌面路線圖、Herdr、`discuss` 動詞輸出

## Risks / Trade-offs

- [回歸對照：既有測試斷言 plan JSON 恰為四鍵] → 同批更新 plan_verbs、remote_plan、plan_api 的斷言；人眼輸出在沒有尚未轉出的刀時位元級不變，既有人眼斷言不受影響
- [引擎與前端暫時各有一份分刀解析，第 4 刀前可能判定不同] → 規則逐條移植自 discussionModel.ts，單元測試採用同一組例子；第 4 刀移除前端解析
- [每次 plan 都讀全部未封存討論的結論，桌面看板刷新也會付這個成本] → 未封存討論通常是個位數，讀檔量與看板既有的討論清單同級；若日後成為瓶頸再加快取
- [跨平台] → 純字串解析，無路徑或 git 互動；CI 三平台照常
- [storage 解耦] → 解析只經 `Store` 介面讀討論（本機 fs 與 TeamStore 同一路），符合引擎與儲存解耦的方向

## Migration Plan

- 新增欄位，無資料遷移；舊 client 忽略不認得的鍵（serde 預設不拒絕未知欄位），新 client 讀舊 server 時讀作空
- 退版：移除 `unpromoted_cuts` 欄位與渲染段即可

## Open Questions

(none)

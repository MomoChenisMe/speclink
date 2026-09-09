## Context

`speclink analyze` 的引擎入口是 crates/speclink-core/src/analyzer.rs 的 `analyze(store, change, schema)`，回傳 `AnalyzeReport`（四個 `DimensionStatus` 加一串 `Finding`）。CLI 的 fs 模式與 remote 模式都只是把這份報告序列化輸出（人眼或 `--json`），desktop 的分析面板直接讀 `--json` 的欄位；規則本身只有這一個落點。

現行四條會動到的規則：

- `conDesignNotInTasks`：`design_headings` 取 design.md 全部 `###` 標題，整串轉小寫後用 `contains` 在 tasks.md 全文（也轉小寫）找。
- `ambNoScenario`：對 `parse_delta_spec` 解析出的每條需求，`scenarios` 為空即報，不看需求所屬的操作區塊。`Requirement` 結構已帶 `operation` 欄位（ADDED／MODIFIED／REMOVED）。
- `ambAbstractScenario`：scenario 內文任一行含 ASCII 數字、反引號或半形雙引號即視為「有具體值」；`##### Example:` 也算。
- `ambWeakLanguage`：英文樣式在整行轉小寫後以子字串比對；TBD／TODO／???／TKTK 另一組；CJK 樣式第三組，帶「不可能」豁免。每行至多一筆。

凍結權威：自 2026-07-27 起，analyze 的輸出以 speclink 自身契約為準（verb-contract 的「動詞 --json 輸出形狀凍結」），不再對齊 OpenSpec 原版；判斷規則允許分岔，輸出欄位不允許。analyzer.rs 目前零個 `#[test]`；CLI 端的 remote_verb_parity.rs 用 mock payload 對照序列化形狀。

## Goals / Non-Goals

**Goals:**

- 四條規則的誤報面消失：序號前綴、REMOVED 需求的 scenario、全形具體值、英文子字串誤命中。
- REMOVED 需求改由「Reason／Migration 齊備」守住。
- analyze 的規則有正典規格（新 capability `change-analysis`）與單元測試。
- `--json` 形狀、既有 finding 的訊息文字與 key 逐字不變。

**Non-Goals:**

- Coverage（`covMissingSpec`、`covMissingTask`）與 Gaps（`gapRestale`、`gapNoProposal`、`gapNoMainSpec`、`gapModifiedNotFound`）的規則不動。covMissingTask 對 REMOVED 需求仍要求 tasks 提到需求名——移除功能要有任務去拆。
- 不把 archive 的合併守門（`merge_violations`）接進 analyze；依討論 manual-marker-placement-lint 的分工，那歸 validate，另開 change。
- 不做模糊或 token 比對；不限縮 design 標題到 Decisions 區段（封存統計 743 個標題只有 4 個在區段外）；不豁免反引號內的英文弱語氣詞；不為 CJK 弱語氣詞建專案術語豁免表（延後）。
- 不改 CLI 指令、旗標、exit code、設定欄位、技能文字或文件。

## Decisions

### D1：design 標題比對改為「本文或編號任一命中」，前綴只認三種樣式

標題先經 `split_heading_label`（新函式，名稱可調）拆成 `(label, body)`：

- 前綴文法：`D<數字>`、`決策<數字或連續的中文數字一到十>`、`Decision <數字>`（數字只認 ASCII `[0-9]`，不用 Unicode `\d`，全形 `D１` 不當編號、與守衛的 ASCII 判準一致；`Decision` 與數字之間零或多個空白），後接零或多個空白、零或一個冒號（`：` 或 `:`）、零或多個空白。大小寫不敏感。以一條 anchored regex 表達，與 drift.rs 既有的 `Regex` 用法同一路線。
- 拆出後 `label` 是前綴去掉尾端冒號與空白、內部空白收成一個的部分（如 `D1`、`決策一`、`Decision 3`），`body` 是其餘文字 trim 後的結果。
- 命中判定（兩側皆轉小寫）：`tasks` 含 `body`，或 `tasks` 含 `label` 且 `label` 前後都不是 ASCII 字母或數字、後一字也不是中文數字（處理 `D1` 對 `D12`、task ULID 註解裡的 `d1`、`決策十` 對 `決策十二`）。`label` 比對容許 `D1` 與 `d1` 互通。邊界檢查與 D4 的英文字邊界共用同一個「找到子字串後看左右鄰字」的 helper，各自帶自己的邊界判準——不用 regex `\b`，因為 `\b` 把數字與底線算進單字、對 CJK 也沒有意義。
- `body` 為空（標題只有編號）時只比 `label`、守衛照常；標題無前綴時退回現行行為：整串標題小寫 `contains`。

為什麼不只去前綴：封存統計顯示 tasks 已有 13 處用「（design D1）」引用設計，只去前綴仍逼人抄本文。為什麼只認三種樣式：468 個帶前綴的封存標題全部落在這三種，`1.` 之類的數字列表樣式零筆，加了只增加誤判面。

Finding 的 `summary` 維持現行文字 `Design topic '<小寫整串標題>' not referenced in tasks`，`summary_msg.params.keyword` 也維持小寫整串標題——輸出不變，只有觸發條件變。

### D2：REMOVED 需求跳過 scenario 檢查，改查 Reason／Migration

`parse_delta_spec` 為每條需求多記一組「移除註記」（兩個布林收成一個小型別）：需求本文（`### Requirement:` 之後、第一個 `#### Scenario:` 之前）是否有一行 trim 後以 `**Reason**`／`**Reason:**`／`**Reason：**` 之一開頭、是否有一行以 `**Migration**` 的對應三種寫法開頭。

Ambiguity 的第一段迴圈改為：`operation == "REMOVED"` 的需求不進 `ambNoScenario`；改為缺 Reason 或缺 Migration（或兩者皆缺）時報一筆 Warning：

- `summary`：`REMOVED requirement '<name>' has no **Reason**/**Migration**`（缺哪個就列哪個；兩者皆缺列 `**Reason** and **Migration**`）。
- `recommendation`：`Add **Reason**: and **Migration**: lines under '<name>'`。
- `summary_msg.key`：`ambRemovedNoNotes.summary`，`params`：`req`＝需求名、`missing`＝`Reason`／`Migration`／`Reason and Migration`；`recommendation_msg.key`：`ambRemovedNoNotes.recommendation`，`params` 同。key 用 NoNotes 而非 NoReason，因為它同時涵蓋只缺 Migration 的情況；summary 裡的粗體字面直接由 `missing` 推導。
- 編號共用 Ambiguity 的 AMB-N 流水號，落在同一檔案的 no-scenario 段（凍結的「每檔依序：no-scenario → abstract → weak」順序不變，REMOVED 檢查併入第一段）。

為什麼不把 Reason／Migration 檢查放 validate：validate 只管結構與 archive 會不會拒收，archive 不需要這兩行；內容完整度歸 analyze（討論 manual-marker-placement-lint 的分工）。

### D3：具體值字元集加全形引號與全形數字

`has_concrete` 的判斷擴為：ASCII 數字、反引號、半形雙引號、`「」『』`、全形數字 `０`-`９`。不加中文數字（一二三…會命中「一起」「一律」等日常詞）、不加單引號（與現行「單引號不算」的既定判準一致）。`##### Example:` 的判定不變。

### D4：英文弱語氣詞改字邊界

英文五個樣式改為「命中的子字串前後都不是 ASCII 字母」，`n't` 縮寫視為字邊界（`shouldn't` 仍報 `should`）；代價是 `maybe`、`considered` 這類完整單字不再命中，測試與規格 Example 表把這個代價釘住；整行仍先轉小寫，因此 `Should`、`SHOULD` 仍命中（規格指引本來就要求用 SHALL 取代）。`TBD`／`TODO`／`???`／`TKTK` 與 CJK 樣式維持子字串比對。每行至多一筆與檢查順序不變。

為什麼 CJK 不改字邊界：中文沒有詞邊界字元，字邊界對它沒有意義；「不可能」豁免照舊。

### D5：規則落正典，新 capability 命名 change-analysis

與 spec-validation（validate）、drift-computation（drift）同一「對象加動作」命名線。規格以「使用者可觀察的 finding」敘述規則：觸發條件、dimension、severity、summary 與 key；不寫函式名。四個面向全部入正典，含本次不動的 Coverage 與 Gaps——這樣後續的規則調整才有 MODIFIED 可落。

### D6：測試放 analyzer.rs 的 `#[cfg(test)]` 模組，用 TestStore

比照 crates/speclink-core/src/validate.rs 的既有測試寫法：`TestStore::with_meta` 建 change、`put_artifact` 放 proposal／design／tasks／delta spec、`find_change` 取 `Change`、呼叫 `analyze(&store, &change, &spec_driven(), )`，斷言 findings 的 `summary_msg.key` 集合。不新增 golden：analyze 沒有既有 golden 檔，人眼輸出的渲染在 CLI 層未動。

## Implementation Contract

**行為（使用者可觀察）**

1. design.md 標題 `### 決策一：整個移除 listDepthLimit 擴充`，tasks.md 含「整個移除 listDepthLimit 擴充」或含「決策一」（後一字非數字）→ 不報 `conDesignNotInTasks`。tasks.md 兩者都沒有 → 報，`summary` 為 `Design topic '決策一：整個移除 listdepthlimit 擴充' not referenced in tasks`。
2. design.md 標題 `### D1 違規清單與聚合錯誤形狀`，tasks.md 只含 `D12` → 報；tasks.md 含 `(design D1)` → 不報。
3. design.md 標題 `### 索引 JSON 的形狀與推導規則`（無前綴），tasks.md 含整串 → 不報；含一半 → 報。與現行完全相同。
4. delta spec `## REMOVED Requirements` 下的需求，只有 `**Reason**:` 與 `**Migration**:` 兩行、無 scenario → 不報 `ambNoScenario`、不報 `ambRemovedNoNotes`。缺 `**Migration**` → 報 Warning，`summary` 為 `REMOVED requirement 'X' has no **Migration**`，`summary_msg.key` 為 `ambRemovedNoNotes.summary`、`params.missing` 為 `Migration`。ADDED／MODIFIED 需求無 scenario 仍報 `ambNoScenario`。
5. scenario 內文 `- **THEN** 顯示「已封存」` 且無 Example、無 ASCII 數字 → 不報 `ambAbstractScenario`。內文只有「顯示成功訊息」→ 報。
6. spec 行 `The shoulder strap SHALL lock` → 不報 `ambWeakLanguage`；`It should lock` → 報 `should`；`盡可能鎖定` → 報 `可能`（不變）。

**介面與資料形狀**：`analyze` 函式簽名、`AnalyzeReport`／`Finding`／`Msg` 的欄位與 serde 名稱不變。`--json` 新增的只有 finding 的 key 值 `ambRemovedNoNotes.summary`／`ambRemovedNoNotes.recommendation` 與 params 鍵 `req`、`missing`。

**失敗模式**：無新的錯誤路徑；規則函式對空字串、無標題的 design、無 REMOVED 區塊的 delta 皆回傳零 finding。

**驗收**：`cargo test -p speclink-core analyzer` 全綠，新測試至少覆蓋上述六條行為各一組正反例與「無前綴回歸」；`cargo test -p speclink-cli --test it remote_verb_parity` 仍綠（形狀未動）；`speclink validate analyze-rule-tuning` 通過、`speclink analyze analyze-rule-tuning` 無 Critical／Warning。

**範圍**：in scope＝analyzer.rs 的四條規則、其測試、新 capability 的 delta spec。out of scope＝Coverage／Gaps 規則、validate、archive 守門、CLI／desktop 呈現層、技能文字、文件。

## Risks / Trade-offs

- [既有 finding 編號前移]：同一份 artifacts 的 AMB-N／CON-N 流水號可能因誤報消失而變小。→ 編號本來就不是穩定識別（依 artifacts 內容而變），desktop 面板以整筆 finding 重繪；proposal 已載明相容性影響。
- [回歸對照]：analyze 無 golden；remote_verb_parity.rs 用 mock payload。→ 新增的單元測試就是這批規則的第一道回歸網；跑 `cargo test -p speclink-core` 與 `cargo test -p speclink-cli --test it` 確認未波及。
- [跨平台]：純字串處理，無路徑、無 git、無環境變數；CRLF 的 tasks.md 與 design.md 走 `lines()`，`\r` 會留在行尾但 `trim` 後不影響標題與 Reason／Migration 判斷。→ 測試加一組 CRLF 輸入。
- [前綴文法漏接]：未來出現第四種序號樣式（如 `決定一：`）仍會落回整串比對。→ 規格明列三種樣式；新增樣式走 MODIFIED delta。
- [D1 樣式的 label 太短]：`D1` 在 tasks 出現在無關脈絡（如 `D1` 是某個識別符）會被當引用。→ 後一字非數字的檢查擋掉 `D10`-`D19`；其餘誤放行只會讓一條 Warning 消失，不會多報。

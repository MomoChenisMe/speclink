## 1. D1：design 標題比對改為「本文或編號任一命中」，前綴只認三種樣式

- [x] 1.1 在 crates/speclink-core/src/analyzer.rs 新增標題拆解函式：輸入 design `###` 標題字串，輸出（編號、本文）；只認 `D<數字>`、`決策<數字或一到十的中文數字>`、`Decision <數字>` 三種前綴（大小寫不敏感，後接零或多個空白、零或一個全形或半形冒號、零或多個空白），無前綴回傳（無、整串）、只有編號回傳（編號、空）。驗證：`#[cfg(test)]` 測試逐列斷言 delta spec「Consistency 的 design 標題引用判定」Example 表的五組輸入輸出，`cargo test -p speclink-core analyzer` 綠。 <!-- speclink-task:tsk_01M22B3PGGXRTGSCAEFV981G9X -->
- [x] 1.2 在 crates/speclink-core/src/analyzer.rs 把 `conDesignNotInTasks` 的命中判定改為「tasks.md 全文（小寫）含本文，或含編號且編號後下一字元非 ASCII 數字」，無前綴或本文為空時退回整串子字串比對；finding 的 `summary` 與 `summary_msg.params.keyword` 維持小寫整串標題。驗證：測試涵蓋 delta spec「Consistency 的 design 標題引用判定」四個 Scenario（本文命中、編號命中 `(design D1)`、`D12` 不算、無編號整串比對），外加 wadpilot 實例（`### 決策一：整個移除 listDepthLimit 擴充` 對 `## 2. 實作：整個移除 listDepthLimit 擴充（design 決策一）`）零 finding；`cargo test -p speclink-core analyzer` 綠。 <!-- speclink-task:tsk_01M22B3PGGCFVW2KC216E5W44D -->

## 2. D2：REMOVED 需求跳過 scenario 檢查，改查 Reason／Migration

- [x] 2.1 在 crates/speclink-core/src/analyzer.rs 的 `parse_delta_spec` 為每條需求記錄本文是否各有一行 trim 後以 `**Reason**` 與 `**Migration**` 開頭；Ambiguity 第一段對 `operation == "REMOVED"` 的需求不報 `ambNoScenario`，改在缺 Reason／Migration 時報 Warning `ambRemovedNoReason`（`summary` 為 `REMOVED requirement '<name>' has no **Reason**`／`**Migration**`／`**Reason** and **Migration**`，`recommendation` 為 `Add **Reason**: and **Migration**: lines under '<name>'`，`summary_msg.key` 為 `ambRemovedNoReason.summary`、`params` 含 `req` 與 `missing`，`recommendation_msg.key` 為 `ambRemovedNoReason.recommendation`），共用 AMB 流水號並落在該檔第一段。驗證：測試涵蓋 delta spec「Ambiguity 的 scenario 缺席與 REMOVED 需求檢查」三個 Scenario（齊備零 finding、缺 Migration 報 `missing`＝`Migration`、ADDED 無 scenario 仍報 `ambNoScenario`），並斷言 `--json` 序列化後 `summary_msg.params` 含 `req` 與 `missing` 兩鍵；`cargo test -p speclink-core analyzer` 綠。 <!-- speclink-task:tsk_01M22B3PGG0RDYK8P6S8MNREK4 -->

## 3. D3：具體值字元集加全形引號與全形數字

- [x] 3.1 在 crates/speclink-core/src/analyzer.rs 的 scenario 具體值判斷加入 `「」『』` 與全形數字 `０`-`９`，ASCII 數字、反引號、半形雙引號維持，中文數字與單引號不算。驗證：測試逐列斷言 delta spec「Ambiguity 的具體值判定」Example 表六組輸入（`回傳 3 筆結果`、`顯示「已封存」`、`顯示『完成』`、`第１頁` 具體；`回傳三筆結果`、`顯示 '完成'` 抽象）的 `ambAbstractScenario` 有無；`cargo test -p speclink-core analyzer` 綠。 <!-- speclink-task:tsk_01M22B3PGGG7AW8VX8G7BEV9DK -->

## 4. D4：英文弱語氣詞改字邊界

- [x] 4.1 在 crates/speclink-core/src/analyzer.rs 的 `weak_pattern_in` 把五個英文樣式改為字邊界比對（整行轉小寫後，命中子字串前後字元都不是 ASCII 字母才算），`TBD`／`TODO`／`???`／`TKTK` 與 CJK 樣式維持子字串比對，每行至多一筆與檢查順序不變。驗證：測試逐列斷言 delta spec「Ambiguity 的弱語氣詞偵測——英文樣式 should、may、might、consider、possibly 以字邊界比對」Example 表六組輸入（`it should lock`、`Should lock` 報 `should`；`the shoulder strap`、`a considerable delay`、`the mayor` 不報；`outbdoor` 報 `TBD`）與 CJK Scenario（`系統盡可能鎖定` 報 `可能`、`這不可能發生` 不報）；`cargo test -p speclink-core analyzer` 綠。 <!-- speclink-task:tsk_01M22B3PGGHHA63Z9RKB5143CA -->

## 5. D5：規則落正典，新 capability 命名 change-analysis

- [x] 5.1 在 crates/speclink-core/src/analyzer.rs 的測試模組為本次不動的規則補回歸網，逐條對應 delta spec 的 Scenario：「面向的前置 artifact 與報告形狀」（缺 design 時 Consistency 為 `Skipped (insufficient artifacts)` 且 `artifacts_missing` 含 `design`；空 tasks.md 仍算存在）、「Coverage 規則」（`Implement CSV exporter` 命中 `CSV Export`；只在 `## 2. CSV Export` 群組標題出現時報 `covMissingTask`）、「Gaps 規則」（`R9` 不在正典時 `recommendation_msg.params.spec` 為 `auth`；正典缺席時 2 條 MODIFIED 只報 1 筆 `gapNoMainSpec`）。驗證：`cargo test -p speclink-core analyzer` 綠，且測試名稱可對應到 Scenario 名。 <!-- speclink-task:tsk_01M22B3PGG2FXGC729B2GE2QZJ -->

## 6. D6：測試放 analyzer.rs 的 `#[cfg(test)]` 模組，用 TestStore

- [x] 6.1 測試模組比照 crates/speclink-core/src/validate.rs 的寫法（`TestStore::with_meta`、`put_artifact`、`find_change`、`analyze(&store, &change, &spec_driven())`），並加一組 CRLF 換行的 design.md 與 tasks.md 輸入斷言 D1 判定不受 `\r` 影響。驗證：`cargo test -p speclink-core`（含 golden）綠、`cargo test -p speclink-cli --test it remote_verb_parity` 綠、`speclink validate analyze-rule-tuning` 通過、`speclink analyze analyze-rule-tuning` 無 Critical／Warning、`node --test scripts/*.test.mjs` 綠。 <!-- speclink-task:tsk_01M22B3PGGT14GHPN69P8VVTDD -->

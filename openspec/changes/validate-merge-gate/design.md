## Context

`speclink validate` 的引擎入口是 crates/speclink-core/src/validate.rs 的 `validate_change(store, change, schema, strict)`，回傳 `ValidationResult { change, errors, valid, warnings }`。它目前檢查：delta 至少一個操作區塊、新開 capability 的 Purpose、近似名 warning、同一 delta 內的重複需求名、tasks.md 的 `[M]` 標記位置、無 delta 的提示 warning。

同一個函式有五個呼叫者：core 的 Command 執行器（CLI fs 模式、server 的 `GET /changes/{name}/validate` 端點、node binding 都經它）、apps/desktop/core 的 `validate_at`、單筆 archive 的驗證前置（crates/speclink-core/src/archive.rs）、bulk archive 的驗證前置（crates/speclink-cli/src/verbs/lifecycle.rs）。

archive 的合併守門是 crates/speclink-core/src/archive.rs 的 `merge_violations(store, change) -> Vec<MergeViolation>`（欄位 `capability`／`operation`／`requirement`／`reason`），只讀 Store、不碰 git；drift 以 `spec_assumptions` 轉用、bulk archive 預檢直接呼叫。單筆 archive 的順序是「先 validate_change，再守門，再合併」，拒絕文案由 `merge_refusal` 聚合，是 archive-merge 規格凍結的輸出。

## Goals / Non-Goals

**Goals:**

- validate 的逐 change 驗證報出 archive 會拒收的每一類 delta 違規，使用者在 propose 收尾就看到。
- 守門判斷維持單一實作（`merge_violations`），validate 只是第四個消費者。
- archive（單筆與 bulk）的可觀察輸出逐位元不變——唯一例外是 delta 自寫 `## PURPOSE Requirements` 標頭這種既有守門的誤分類（D3）：`is_purpose_gate()` 兩欄化後，它從 Purpose 類歸回過期類，archive／drift／bulk 的分流呈現跟著變；真正的 Purpose 守門違規呈現不變。
- fs 與 remote 兩模式、CLI 與 desktop 同一份判斷。

**Non-Goals:**

- 不改守門的判斷內容與 reason 字串；`is_purpose_gate()` 的兩欄化只改分類判別，不改任何違規的產生條件。
- 不把守門搬進 analyze（討論 manual-marker-placement-lint 的分工：格式守門歸 validate）。
- 不改 validate 的 `--json` 欄位、既有 error 文字與順序、exit code 規則。
- 不新增 `--strict` 相依：守門違規不論 strict 一律 error。

## Decisions

### D1：驗證入口拆成兩層，archive 家族改呼叫結構層

`validate_change` 的既有本體改名為 `validate_change_structural`（公開），行為與輸出逐位元不變；`validate_change` 保留原簽名，實作為「先跑結構層，再把合併守門的違規追加為 error」。守門判斷沿用 `archive::capability_violations`（`merge_violations` 的逐 capability 本體，drift、bulk 預檢與單筆 archive 走的同一支）：結構層讀過的每份 delta 內文直接傳入，不再從 Store 重讀；走訪順序同 `merge_violations`（Store 的 delta capability 列舉順序）。Command 執行器與 desktop core 不改碼，自動取得守門；單筆 archive 與 bulk archive 改呼叫 `validate_change_structural`。

為什麼是 archive 改而不是 verb 路徑改：archive 的驗證前置後面緊接自己的守門與 `merge_refusal` 聚合文案（archive-merge「違規聚合一次回報」）。若 archive 的前置也含守門，違規會以「Validation failed:」的形狀先跳出來，`merge_refusal` 變成只有 `--no-validate` 才到得了，等於改了凍結輸出。讓 archive 明確選結構層，兩條凍結輸出都不動。

### D2：守門 error 的組字與位置

每筆 `MergeViolation` 化為一條 error：`specs/<capability>/spec.md: <operation> '<requirement>': <reason> (see: speclink drift <change>)`，組字放在型別旁邊（`MergeViolation::validation_error`，與 `merge_refusal` 的聚合呈現同檔，守門欄位一動兩種呈現同一 diff 可見）。路徑是邏輯路徑、一律正斜線（與既有 error 同慣例）；撞名類的 `<operation>` 是守門的逗號串（如 `ADDED, RENAMED`），照原樣列出；守門對同一區段寫兩次的同名需求會逐筆各產一筆相同違規，validate 同字只列一行。`<reason>` 逐字沿用守門的凍結字串（如 `target requirement no longer exists in the canonical spec`、`already exists in the canonical spec — archive would refuse it`、`drops canonical scenario(s) X — carry them over, or declare the removal with ...`）。守門 error 一律排在既有 error 之後（含 `[M]` 標記 error 之後），守門內部依 `merge_violations` 的回傳順序。`valid` 仍為 `errors.is_empty()`。

為什麼附 `see: speclink drift <change>`：archive 的拒絕文案給的補救動線是 drift → ingest；validate 的 error 是逐行字串，沒有聚合尾註的位置，每行自帶指向即可，`/speclink-ingest` 的步驟由 drift 的輸出接手。

### D3：與結構層的去重規則

守門的違規有兩類會與結構層重疊，去重規則：

- Purpose 類（`MergeViolation::is_purpose_gate()` 為 true）：一律略過。結構層的 Purpose error 已含範例骨架，比守門的 reason 更完整。`is_purpose_gate()` 同時比對 operation 與 requirement 兩欄——delta 自己寫 `## PURPOSE Requirements` 標頭時，底下需求的違規是 CANON_ABSENT 類，不得被當成 Purpose 守門吞掉。
- 同名撞區段類（reason 為守門的 section-collision 字串）：若該需求名已被結構層的「Duplicate requirement」或「appears in both」error 報過、且該撞名不含 RENAMED 端點，略過；含 RENAMED 端點的撞名（`involves_rename()`）一律保留——結構層的掃描只走 ADDED／MODIFIED／REMOVED 區段，看不到那一端。判別以 `MergeViolation` 新增的 `is_section_collision()` 與 `involves_rename()` 方法為準，與 `is_purpose_gate()` 同一模式，validate 不比對 reason 字串。

其餘類別（目標不存在、ADDED 撞正典、未宣告的 scenario 移除、malformed 註記、RENAMED 缺 TO 或撞名、新 capability 帶 MODIFIED／REMOVED／RENAMED）沒有結構層對應，全數列出。

### D4：測試落點

- crates/speclink-core/src/validate.rs 的既有 `#[cfg(test)]` 模組：用 `TestStore` 放正典與 delta，逐類斷言 error 文字與 `valid`；去重兩條各一組；`validate_change_structural` 對同一輸入零守門 error。
- crates/speclink-core/src/archive.rs 既有測試不動即為回歸網：archive 的拒絕文案未變。
- crates/speclink-cli/tests/it/validate_specs.rs 加一組端到端：正典有 R1、delta MODIFIED R9，`speclink validate <change>` 人眼輸出列該 error 且以 `Validation failed.` 非零收尾；`--json` 的 `errors` 含該字串、`valid` 為 false。
- remote 模式不另寫測試：server 端點經 Command 執行器，crates/speclink-cli/tests/it/remote_verb_parity.rs 已鎖形狀。

## Implementation Contract

**行為（使用者可觀察）**

1. 正典 `specs/auth/spec.md` 有需求 R1；change 的 delta 在 `## MODIFIED Requirements` 寫 R9 → `speclink validate <change>` 報 `✗ <change> — invalid`，error 行為 `specs/auth/spec.md: MODIFIED 'R9': target requirement no longer exists in the canonical spec (see: speclink drift <change>)`，exit code 非零；`--json` 的 `valid` 為 false、`errors` 含該字串。
2. 正典 R1 有 scenario `ok` 與 `fine`；delta MODIFIED R1 只寫 `ok`、沒有 `<!-- REMOVED-SCENARIO: fine -->` → error 行的 reason 為守門的 `drops canonical scenario(s) fine — carry them over, or declare the removal with ...` 字串。補上宣告後 → validate 通過。
3. delta 在 `## ADDED Requirements` 寫正典已有的 R1 → error 行的 reason 為 `already exists in the canonical spec — archive would refuse it`。
4. 新開 capability 的 delta 缺 Purpose → 只有既有的 Purpose error（含 `## Purpose` 範例骨架），沒有第二條守門 error。
5. 同一 delta 的 ADDED 區段寫兩次 `Fresh` → 只有既有的 `Duplicate requirement 'Fresh' in ADDED section`，沒有守門的 collision error；delta 的 RENAMED 把 `R1` 改成 `R2` 且 ADDED 也寫 `R2` → 守門的 collision error 列出（結構層看不到 RENAMED）。
6. 同一份會被守門拒收的 change 跑 `speclink archive <change>` → 輸出與本 change 之前逐位元相同（先 Validation failed 的結構項；結構全綠時為 `merge_refusal` 的聚合清單）。
7. remote 模式 `speclink validate <change>` 與 desktop 的「結構驗證」列，對同一 Store 內容顯示同一組 error。

**介面與資料形狀**：`validate_change(store, change, schema, strict) -> ValidationResult` 簽名不變；新增 `validate_change_structural`（同簽名）；`MergeViolation` 新增 `is_section_collision(&self) -> bool`、`involves_rename(&self) -> bool` 與 `validation_error(&self, change: &str) -> String`；`archive::capability_violations` 開放為 crate 內可見。`ValidationResult` 欄位與 serde 名稱不變。

**失敗模式**：守門只讀 Store；delta 讀不到時視為空字串（與 archive 相同），不新增錯誤路徑。

**驗收**：`cargo test -p speclink-core validate` 與 `cargo test -p speclink-core archive` 綠；`cargo test -p speclink-cli --test it validate_specs` 與 `--test it remote_verb_parity` 綠；對本 repo 現存的 change 跑 `speclink validate --all` 仍通過（現存 change 沒有守門違規）。

**範圍**：in scope＝validate.rs 的入口拆分與守門追加、archive.rs 與 lifecycle.rs 的呼叫點切換、`is_section_collision`／`involves_rename`／`validation_error` 與 `is_purpose_gate()` 的兩欄化、對應測試、兩份 delta spec。out of scope＝守門的違規產生條件（`is_purpose_gate()` 的分類判別除外）、analyze、drift、desktop／server 的呈現層、技能與文件。

## Risks / Trade-offs

- [archive 輸出回歸]：archive 家族若漏改呼叫點，拒絕文案會變形。→ archive.rs 既有測試（Validation failed 與 merge_refusal 的凍結字串）與 crates/speclink-cli/tests/it 的 archive 對照即回歸網；task 要求兩組都跑。
- [現存 change 突然 invalid]：接上守門後，repo 內既有的 change 若本來就會被 archive 拒收，validate 會開始報紅。→ 這正是目的；驗收步驟對現存 change 跑 `validate --all` 確認基準。
- [remote 一致性]：server 端點與 CLI fs 模式必須同一份判斷。→ 兩者都經 Command 執行器呼叫 `validate_change`，不平行實作；remote_verb_parity 鎖形狀。
- [跨平台]：error 內的路徑是邏輯路徑、手工以正斜線組字，不經 PathBuf；守門本身無路徑、無 git。→ 測試斷言字串含 `specs/auth/spec.md:`。
- [去重判別靠 reason 分類]：`is_section_collision` 以守門內部常數判別，常數改字時方法跟著改。→ 方法與常數同檔，同一 diff 可見。

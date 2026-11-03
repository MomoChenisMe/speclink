## 1. 結論分刀解析（D1）

- [x] 1.1 （Red）需求「plan 列出討論結論中尚未轉出的刀」的分刀解析：在 crates/engine/speclink-core/src/lifecycle/discuss/cuts.rs 的單元測試逐列寫入 spec「分刀解析」Example 的八列（斷言 cut、changes、title，以及不是刀、粗體內無名稱的判定），另加 Decision 欄位邊界三例：`**Rationale**:` 之後的粗體 cut 列點不算、巢狀列點不算、Conclusion 沒有 `**Decision**:` 時回空。驗證：`cargo test -p speclink-core discuss::cuts` 因 `parse_cuts` 不存在而編譯失敗 <!-- speclink-task:tsk_01M6G268J84685PP3AHJDK14J2 -->
- [x] 1.2 （Green）依設計 D1「結論分刀解析器放在 speclink-core 的 discuss 模組」實作 `parse_cuts(conclusion: &str) -> Vec<Cut>` 與 `Cut { id, title, names, names_in_bold }`，在 crates/engine/speclink-core/src/lifecycle/discuss.rs 以 `pub mod cuts;` 掛上；規則逐條對照 packages/ui/src/discussionModel.ts 的 `cutCard`。驗證：`cargo test -p speclink-core discuss::cuts` 全部通過 <!-- speclink-task:tsk_01M6G268J896N5VJB7RBTVKWN4 -->

## 2. plan 計算（D2）

- [x] 2.1 （Red）在 crates/engine/speclink-core/src/lifecycle/plan.rs 的測試以記憶體 store 建立討論記錄，涵蓋：只轉出 cut 1 時列出 cut 2、全部轉出時為空、一刀兩個變更只轉出一個時 `unpromoted` 只含未轉出者、粗體內無名稱的刀不列、已封存討論不列、沒有 Conclusion 的討論不列、兩份討論依討論清單順序排列；並斷言 `compute_with_ranks` 與 `compute_strict` 的結果同樣帶 `unpromoted_cuts`。驗證：`cargo test -p speclink-core plan` 因欄位不存在而失敗 <!-- speclink-task:tsk_01M6G268J8WE3FQHMQ99Q00B79 -->
- [x] 2.2 （Green）依設計 D2「plan 在 compute_from 結尾附上尚未轉出的刀」新增 `UnpromotedCut { discussion, cut, title, changes, unpromoted }`（camelCase 序列化）與 `Plan.unpromoted_cuts`（序列化為 `unpromotedCuts`），在 `compute_from` 結尾以 `discuss::live_conclusions`（每份記錄只讀一次）與 `parse_cuts` 算出；讀不到結論的討論略過。驗證：`cargo test -p speclink-core plan` 與 `cargo test -p speclink-core discuss` 全部通過 <!-- speclink-task:tsk_01M6G268J8RJB2CESDEBX4EBJE -->

## 3. CLI 輸出（D3）

- [x] 3.1 （Red）需求「plan 動詞輸出波次與阻擋清單」：在 crates/adapters/speclink-cli/tests/it/plan_verbs.rs 把「JSON 輸出形狀」的斷言改為 waves、changes、next、skipped、unpromotedCuts 五鍵，並斷言 unpromotedCuts 每項的 discussion、cut、title 為字串、changes 與 unpromoted 為字串陣列；新增人眼輸出測試——一個提案中變更加一把未轉出的刀時 `--no-color` 的 stdout 恰為 spec「人眼輸出的 Unpromoted cuts 段」的五行且無 ANSI，所有刀都已轉出時 stdout 不含 `Unpromoted cuts`。驗證：`cargo test -p speclink-cli --test it plan_verbs` 失敗 <!-- speclink-task:tsk_01M6G268J8M3AVCW7S8VEBDYF0 -->
- [x] 3.2 （Green）依設計 D3「人眼輸出加一段 Unpromoted cuts」在 crates/adapters/speclink-cli/src/verbs/plan.rs 的 `render_plan` 於 Skipped 段之後、`next:` 行之前加印該段，樣式全部經 `color`。驗證：`cargo test -p speclink-cli --test it plan_verbs` 全部通過 <!-- speclink-task:tsk_01M6G268J8EFB5PVERTY6GQHXW -->

## 4. GET /plan 與 remote（D4）

- [x] 4.1 （Red）需求「plan 回應 payload」：在 crates/protocol/speclink-protocol/src/query.rs 的測試斷言 `PlanResponse` 缺 `unpromotedCuts` 鍵時反序列化為空陣列；在 crates/protocol/speclink-remote/src/convert.rs 的測試斷言 spec「尚未轉出的刀逐欄搬回」與「舊 server 缺尚未轉出的刀」兩個情境。驗證：`cargo test -p speclink-protocol` 與 `cargo test -p speclink-remote --lib` 失敗 <!-- speclink-task:tsk_01M6G268J8R3ZR7CTBJ8GP4PA2 -->
- [x] 4.2 （Green）依設計 D4「GET /plan 與 remote 轉換逐欄搬運」新增 `PlanUnpromotedCut` 與 `PlanResponse.unpromoted_cuts`（`#[serde(default)]`），`plan_report` 逐欄搬回引擎的 `UnpromotedCut`。驗證：`cargo test -p speclink-protocol` 與 `cargo test -p speclink-remote --lib` 全部通過 <!-- speclink-task:tsk_01M6G268J86SM220C0RAS4JY54 -->
- [x] 4.3 （Red）需求「plan 唯讀衍生查詢端點」：在 crates/host/speclink-server/tests/it/api/plan_api.rs 新增情境「回應帶尚未轉出的刀」——scope 內討論 talk-a 的 promoted_to 為 add-a、結論有 cut 1 `add-a` 與 cut 2 `add-b`，GET /plan 回 200 且 unpromotedCuts 恰一項、revision 不前進。驗證：`cargo test -p speclink-server --test it plan_api` 失敗 <!-- speclink-task:tsk_01M6G268J8B3J8VSFEXKF2SYRZ -->
- [x] 4.4 （Green）在 crates/host/speclink-server/src/api/routes.rs 的 `plan` 端點把引擎報告的 `unpromoted_cuts` 逐欄搬進 DTO。驗證：`cargo test -p speclink-server --test it plan_api` 全部通過 <!-- speclink-task:tsk_01M6G268J89A3BRMR6RVJBXXCH -->
- [x] 4.5 （Red→Green）remote CLI 同形：在 crates/adapters/speclink-cli/tests/it/remote_plan.rs 斷言 server 回帶 unpromotedCuts 的 plan 時，remote 模式 `speclink plan --json` 的 unpromotedCuts 與回應相同、`--no-color` 人眼輸出含 `Unpromoted cuts` 段；server 回缺這個鍵的舊格式時 `--json` 的 unpromotedCuts 為 `[]`、人眼輸出不含該段；在 crates/protocol/speclink-remote/tests/it/typed_client.rs 的 plan 測試斷言型別化呼叫帶回 unpromoted_cuts。渲染與 fs 模式共用，不另寫第二份。驗證：`cargo test -p speclink-cli --test it remote_plan` 與 `cargo test -p speclink-remote --test it` 全部通過 <!-- speclink-task:tsk_01M6G268J8Z1AWVR2QWDYYR78J -->

## 5. 稽核與收尾

- [x] 5.1 套用 sharp-edges 稽核清單（`speclink instructions --skill audit`）檢查 `parse_cuts` 與 `unpromotedCuts` 欄位：手改的討論檔可能出現 CRLF 換行、未成對的粗體、粗體內未閉合的反引號、極長的列點、同一變更名在兩把刀重複；每個缺口先補成 cuts.rs 的失敗測試再修正，沒有缺口的項目在測試檔以一行註解記下判定。驗證：`cargo test -p speclink-core discuss::cuts` 全部通過 <!-- speclink-task:tsk_01M6G268J80PFT7S8FYR8N1VQK -->
- [x] 5.2 受影響面測試與文件守門全部通過。驗證：`cargo test -p speclink-core`、`cargo test -p speclink-cli --test it`、`cargo test -p speclink-protocol`、`cargo test -p speclink-remote`、`cargo test -p speclink-server --test it`、`cargo test -p speclink-desktop-core`（plan 報告多一個欄位，桌面核心讀 plan 的測試照舊通過）、`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 皆通過，且 `speclink validate plan-unpromoted-cuts` 通過 <!-- speclink-task:tsk_01M6G268J8GKV161PACZ5ARSET -->

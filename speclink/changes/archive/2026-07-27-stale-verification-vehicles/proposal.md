## Summary

把正典規格與路線圖文件裡「指向不存在的測試套件」的驗證載體，改指 repo 內實際存在且 CI 可跑的回歸保護——比照 spectra-legacy-cleanup 的 design 決策二。零行為變更。

## Motivation

spectra-legacy-cleanup 修掉了 store-abstraction 的 `parity_suite 31 項／color_suite 16 項／twin harness 8 情境`，但那次以 `Spectra` 字樣定位命中，同型的失真句只要不含該字樣就被漏掉。實測 repo 內既無 parity suite、也無 color suite，更沒有任何 twin harness 或以此命名的檔案：

1. 正典 4 處以不存在的載體當驗證條件——`client-protocol`（twin harness 全部情境、8 情境全綠）、`reference-server`（twin harness 全部情境的欄位形狀由 stub 對測凍結）、`drift-computation` 與 `host-runtime`（parity、color 與 twin 回歸對照全綠）。AI 代理在 verify 或 apply 階段照著執行時只能編造結果或卡住，這正是 spectra-legacy-cleanup 提案指認過的實害。
2. `docs/implementation-refactor-roadmap.zh-TW.md` 兩處宣稱 CLI 有 parity 保護（第 1 節結論的「parity、golden 與整合測試保護」、元件表 speclink-cli 列的「parity 護欄」），與 README 已修掉的失真句同型。

目標使用者：接手本 repo 的開發者，以及經 AI 代理跑 SDD 流程的維護者。使用情境：verify 階段核對「規格要求的驗證是否真的跑過」、apply 階段依規格挑測試載體、日常閱讀路線圖評估現況。

## Proposed Solution

兩類處置，措辭以「實存載體」取代虛構套件名：

1. **正典 4 份以 delta 改寫**：SHALL 句的凍結語意與情境結構全部不動，只置換驗證載體名——改指 `crates/speclink-cli/tests/` 的整合測試（其中 remote 讀路徑以 fs 與 remote 雙跑對照輸出，多支測試斷言 `--no-color` 人眼輸出）與 `crates/speclink-core/tests/render_golden.rs` 的 golden 快照。同時刪去「8 情境」這類會隨測試增減而過期的固定計數。
2. **路線圖文件兩處**：`parity、golden 與整合測試保護` 改為 `golden 與整合測試保護`；元件表的 `parity 護欄` 改為 `輸出凍結護欄`。

## Non-Goals

- 不動 `stub server` 措辭——`crates/speclink-cli/tests/remote_read_path.rs`、`remote_write_path.rs` 與 `remote_handshake_gate.rs` 各自起了 mock server，那是實存載體，僅命名口語化。
- 不動 `docs/verb-contract*.md` 與 `docs/sdk-node.md` 的 `parity`——該處語意是「入口間功能對等」，不是測試套件名。
- 不新增、不刪除任何測試——現存整合測試與 golden 就是自我基線。
- 不改變任何 CLI 行為、人眼輸出、exit code 或 `--json` 欄位。
- 不建立「禁止虛構載體名」的常駐守衛測試——收尾以一次性 grep 驗證即可。
- 不回改封存 artifacts 與 `@trace` 清單（歷史不回改）。

## Alternatives Considered

- 補建 parity/color/twin 三套 harness 讓規格成真：被否——speclink 已是自我基線，重建外部對照沒有服務對象，且會與現有整合測試重複覆蓋。
- 只修 verify 報告點名的兩處（client-protocol、reference-server）：被否——`drift-computation` 與 `host-runtime` 是同一句型，分批修等於保證下次 verify 再報一次。
- 在規格裡改寫成不指名載體的泛稱（如「既有回歸測試」）：被否——決策二的價值正是「指得出檔案」，泛稱會讓下一個維護者重蹈覆轍。

## 相容性影響

- CLI 人眼輸出與 `--json`：零變更，既有回歸測試須全數通過。
- 不涉及設定欄位（`openspec/config.yaml`／`.speclink.yaml`）。
- 不涉及技能或注入區塊：不動 `crates/speclink-core/assets/`，golden 快照不需再生。

## Impact

- Affected specs: client-protocol、reference-server、drift-computation、host-runtime（4 個能力全為 MODIFIED——措辭改寫、行為不變）
- Affected code:
  - Modified: docs/implementation-refactor-roadmap.zh-TW.md
  - New: （無）
  - Removed: （無）

影響的 crate 或 app：無——本變更只動正典規格與一份文件，不觸及任何 crate 或 app 的源碼。

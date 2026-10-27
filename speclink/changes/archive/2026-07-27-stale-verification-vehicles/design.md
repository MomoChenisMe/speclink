## Context

正典規格以「驗證載體」把需求綁到具體測試，這是 verify 階段核對「規格要求的驗證是否真跑過」的依據。目前有 4 條需求綁在 repo 內不存在的 parity suite、color suite 與 twin harness 上；`docs/implementation-refactor-roadmap.zh-TW.md` 另有兩處宣稱 CLI 受 parity 保護。spectra-legacy-cleanup 已用決策二修掉 store-abstraction 的同型問題，但該次以 `Spectra` 字樣定位，不含該字樣的句子被漏掉。

範圍內：4 份正典 delta 的載體名置換、路線圖兩處措辭。範圍外：任何 CLI 行為／輸出變更、測試增刪、內嵌技能資產與 golden、`stub server` 措辭、封存 artifacts 與 `@trace`。

## Goals / Non-Goals

- Goal: 每條被改寫的需求都指得出 repo 內實際存在的檔案路徑，AI 代理照著跑得起來。
- Goal: 改寫後 `cargo test --workspace` 全綠、`speclink validate` 通過，且不觸發 golden 再生。
- Non-Goal: 不重整需求的其他內容——只動驗證載體子句與固定計數，維持最小 diff。
- Non-Goal: 不把載體改成泛稱（如「既有回歸測試」）——指得出檔案才是這次要保住的價值。

## Decisions

### 決策一：載體對照表（4 條需求共用）

| 原措辭 | 改寫 |
| --- | --- |
| twin harness 的全部情境／twin harness 全部情境 | `crates/speclink-cli/tests/remote_read_path.rs` 的 fs 與 remote 雙跑對照情境 |
| 8 情境全綠 | （刪去固定計數，改為「全部對照情境全綠」） |
| parity、color 與 twin 回歸對照全綠 | `crates/speclink-cli/tests/` 的整合測試（含 `--no-color` 人眼輸出斷言與 fs／remote 對照）與 `crates/speclink-core/tests/render_golden.rs` 全綠 |
| （remote 對照情境的）於重構前後 | （刪去——新載體是單次執行內的 fs／remote 雙跑對照，不是重構前後的基線比對） |
| remote 與 fs 模式的 stdout、stderr 與 exit code 逐位元一致 | remote 與 fs 模式的 `--json` 欄位形狀（key 集合）一致 |
| 欄位形狀 parity 由 stub 對測凍結 | 欄位形狀由 stub 對測凍結 |

第三列的依據：`remote_read_path.rs:297` 的 `assert_same_keys` 只比對 `--json` 的 key 集合（第 365-367 行更明確剔除 `preflight` 後才比），repo 內沒有 fs／remote 的 stdout 位元比對測試。原措辭宣稱的驗證強度高於載體實際斷言的強度——那正是本變更要根除的失真型態，故連同載體名一併校正。

事實依據：`crates/speclink-cli/tests/remote_read_path.rs` 提供 `fs_twin` 建構相同輸入的 fs workspace，對 mock server 與 fs 模式雙跑同一動詞後以 `assert_same_keys` 比對 `--json` 的 key 集合；`remote_write_path.rs` 以 capturing mock server 驗寫路徑；多支 CLI 整合測試以 `--no-color` 斷言人眼輸出。替代方案：逐處自由改寫——被否，同一句型出現在四份規格，無表會改出不一致措辭。

### 決策二：固定計數一律刪除

「8 情境」這類數字會隨測試增減而過期，且無人維護。凡遇固定計數改為「全部」。替代方案：把數字更新為現值（14）——被否，等於把同一顆定時炸彈重新上鏈。

### 決策三：豁免清單（不動的措辭）

- `stub server`（`client-protocol` 56／61／102／148、`remote-connection` 322）與 `stub 對測`（`reference-server` 16 與 267）：`remote_read_path.rs`、`remote_write_path.rs`、`remote_handshake_gate.rs` 各自起了 mock server，是實存載體，僅命名口語化。
- `docs/verb-contract.md`、`docs/verb-contract.zh-TW.md`、`docs/sdk-node.md` 的 `parity`：語意為「入口間功能對等」，非測試套件名。
- 封存變更、討論記錄、正典內的 `@trace` 區塊：歷史不回改。

### 決策四：路線圖兩處的改寫

`docs/implementation-refactor-roadmap.zh-TW.md` 第 1 節結論列的「有 parity、golden 與整合測試保護」改為「有 golden 與整合測試保護」；元件表 `speclink-cli` 列「可保留部分」欄的「parity 護欄」改為「輸出凍結護欄」。該文件開頭已聲明不是持續更新的產品狀態表，因此只修正失真宣稱，不順手更新其他現況描述。

## Implementation Contract

- **可觀察行為**：無執行期行為改變。改寫後 `crates/speclink-cli/tests/`、`crates/speclink-core/tests/` 下所有測試維持現狀通過；`speclink` 任何指令的人眼輸出、`--json` payload 與 exit code 逐位元不變。
- **資料形狀**：4 份 delta 皆為 `## MODIFIED Requirements`，需求標題與正典逐字相同，且每條需求的 `#### Scenario:` 數量與正典一致（archive 時 MODIFIED 會整塊取代，缺一個情境就是永久遺失）。
- **失敗模式**：需求標題與正典不符時 archive 會靜默跳過該需求（analyze 的 gapModifiedNotFound 才會報）——task 內以標題逐字比對防守。
- **驗收條件**：`grep -rn "twin harness\|parity_suite\|color_suite" openspec/specs/` 命中數為 0；`grep -n "parity" docs/implementation-refactor-roadmap.zh-TW.md` 命中數為 0；`speclink validate stale-verification-vehicles` 通過；`cargo test --workspace` 全綠。
- **範圍邊界**：不動任何 `.rs`／`.ts`／`.css` 源碼、不動 `crates/speclink-core/assets/`、不再生 golden、不動 `stub server` 措辭。

## Risks / Trade-offs

- delta 漏抄情境導致正典內容遺失：緩解——task 明定以情境數對照為驗收，archive 前再跑一次比對。
- 改寫後載體路徑又過期（測試檔改名）：接受——指得出檔案的過期是可被 grep 抓到的，泛稱的過期抓不到。
- 與平行變更（`workflow-config-verb-and-skill`、`remote-login-ux-gaps`）的 delta 衝突：本變更只碰 client-protocol、reference-server、drift-computation、host-runtime 四份正典，與兩者的能力集合不重疊。

## Migration Plan

單一變更內完成，無資料遷移。commit 建議一批：4 份 delta 與路線圖同屬一次措辭校正。

## Open Questions

（無）

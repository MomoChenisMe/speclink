# CI 修正後品質檢查（2026-10-22）

## 範圍與限制

- HEAD：`46803b2b3af11719aadde6266a2370c2d7dff367`。
- 沿用使用者已確認的 trusted base：`faefda7dabde01b8d90974aea63f4f7a8bdd5c24`，不採本機建立較晚的 Apply baseline。
- Review／Verify 都是 discovery，固定範圍 229 paths，`patchHash=sha256:b7aa741a6239f224efea728cda7b1fed2392552f8fec6610522b3077da47fc2a`，`outOfScopeChanged=[]`。
- 範圍 payload 包含既有未追蹤的 `$null` 輸出紀錄；沒有閱讀其大型輸出內容、修改或提交它。生成檔按路徑／既有契約定位，不將整份生成差異當程式邏輯重新掃描。
- 任務 60 項：程式 59／59，另有未完成 `[M]` 12.3 公司 Proxy 打包後人工驗收。維持未勾選，不能用模擬或外網測試代簽。
- 已讀 17 份 delta specs，統計 46 Requirements、143 Scenarios、7 Examples。子檢查執行期間因額度限制中斷。主流程接手確認下列問題，並補讀標準根／KB、Node／Server／Protocol 與 UI 接線；**尚未完成所有 143 個情境、7 個範例的逐筆測試映射與全部非生成 hunk 的完整覆核，不宣稱全範圍 clean**。
- 規格格式驗證 `speclink validate desktop-workflow-and-offline-security-review` 與 `git diff --check` 通過。未修改程式，沒有為本輪重跑無變更的整套測試；沿用上一輪同一 HEAD 的 [三平台 CI](https://github.com/mosutw/speclink/actions/runs/39702058887) 與 [Node SDK](https://github.com/mosutw/speclink/actions/runs/39702058902) success。測試通過不代表已涵蓋下列並行窗口。

## Review

審查規則：`extend`（12 項 baseline smells），專案規則 CRITICAL 1、WARNING 4、SUGGESTION 0。規則取自當輪 `workflow-config show --json`。

### Standards

- [SUGGESTION] crates/host/speclink-host/src/security/deep.rs — Standards: possible Duplicated Code。acceptance_proof_for_target 與 acceptance_from_fresh 各有同形 hunk `for id in deep.iter().map(DeepReport::report_id).chain(builtin.iter().map(Report::report_id)) { decisions.extend(read_decisions(store, change, id)?); }`；security/remote.rs::proof 又有同形迴圈（順序相反）。可共用讀取成對報告決策的函式，避免決策載入邏輯分散。

此為選修建議：需要小幅整理三處呼叫，收益是避免決策載入規則分散；不阻擋蓋章。沒有確認的專案硬性規則違反，但未完成範圍不能當成無違反。

### Correctness

以下保留子檢查已確認的 finding 原文；主流程另讀直接 CLI caller、FS 鎖／提交時機與封存恢復範圍，確認沒有外層交易替本機路徑擋住問題。

- [CRITICAL] crates/engine/speclink-core/src/quality/station.rs — Correctness: 本機 verify 可以繞過最新安全報告。第 450 行先以 I0 通過 `require_acceptance`，接著計算來源指紋；另一程序此時提交不完整或含未接受 HIGH 的新報告 I1。第 701–712 行重新讀取 I1，並以 I1 作 CAS 比較，故仍成功蓋章、刪除 verify 工單，沒有重新裁決新報告。CLI caller 沒有外層 checks 鎖；FS 直到 `commit_checks` 才取鎖。應將 proof 的 index 指紋延續到提交比較。現有測試只涵蓋 gate 前已變更、或提交快照之後的 CAS 衝突。
- [CRITICAL] crates/engine/speclink-core/src/lifecycle/archive.rs — Correctness: 新增的封存 CAS 可能在正式規格已修改後拒絕。第 817 行建立 checks 計畫，第 870 行先寫 canonical，第 884 行才呼叫 FS 封存；`archive_checks.rs` 第 238–239 行取得鎖並檢查計畫基底。另一程序在計畫後更新 index／meta，會使封存回 `revision_conflict`，但正式規格已套用、活動提案仍存在，沒有回復；ADDED 項目重試時還會撞上已存在需求。應在涵蓋整段提交的鎖／交易內驗證，或完整回復前面的寫入。現有 bridge 測試只覆蓋遠端原子交易。

證據為直接控制流程及提交條件的不變量違反，尚未新增／執行這兩個時序的失敗回歸測試。本輪不修改程式或測試。

子檢查原始未完成說明：

> 未檢完：229 paths 尚未全覆蓋，尤其標準資料安裝／匯入／reader pin、完整 CLI／Desktop／Node／Server 串接、技能原稿與工具清理、部分 UI 與對應測試。已深入檢查 checks journal、封存與蓋章、acceptance、deep report／Remote context 的主要生產路徑，另讀排序與更新主要 hunks；沒有以 CI 通過推論其餘範圍無問題。

主流程後續補讀了其中部分路徑，但未將這段未完成狀態改寫成全數完成。

Standards：1 項，最高 SUGGESTION；Correctness：2 項，最高 CRITICAL。兩項 CRITICAL 均為 must-fix，沒有接受保留。

## Verify

| 維度 | 結果 |
| --- | --- |
| 完整性 | 程式任務 59／59；人工 12.3 延期；46 條需求、143 個情境、7 個範例已清點，完整逐筆映射未完成 |
| 正確性 | 確認下列 2 項 WARNING，與 Review 的兩項並行問題對應 |
| 一致性 | AcceptanceProof 指紋交棒與封存提交順序存在實作落差；修正可維持原規格／設計意圖 |

- [WARNING] crates/engine/speclink-core/src/quality/station.rs — Correctness: 「驗證蓋章守門與蓋章效果」及 design 的 AcceptanceProof 指紋契約要求以當前報告驗收；本機先驗 I0、提交時卻重讀並接受 I1，讓新不通過報告未經裁決仍可蓋章。應將已驗證 index 指紋綁入最終提交，並補另一程序在來源指紋計算期間更新報告的回歸測試。
- [WARNING] crates/engine/speclink-core/src/lifecycle/archive.rs — Correctness: 「封存時按類別保留完成紀錄」的提交失敗情境要求失敗不留下半套封存效果；本機在驗證 checks 計畫基底前已寫 canonical，並行 index／meta 更新造成 revision_conflict 時不回復正式規格。應調整本機鎖／提交邊界或完整復原，補衝突時 canonical 與活動提案都不變的測試。

兩項均為 must-fix；Verify 沒有另列選修建議。未將獨立安全掃描結果加入品質票據；本專案 securityVerification.enabled=false，本輪沒有執行額外深度安全審查。

## 本輪狀態

依序透過正式 CLI 記錄 Review、Verify discovery 輪次；不蓋章、不修復、不封存、不合併、不打包。兩個核心問題的修正不需修改 specs／design／tasks 意圖。完整覆核尚有缺口，必須明示，不能以本輪或歷史 clean 紀錄宣稱已完整通過。後續處置由使用者選擇。

## 使用者選擇全部修正後：修復與 Round 2

使用者明確選擇全部修正。以下是接續處置，不回寫上述 discovery 原始結果。

- 蓋章：兩個輸入入口都將 AcceptanceProof 交給 `write_stamp`，最終 checks 提交固定比較 proof 的 index digest；較晚讀到的 index 不再取代已驗證指紋。新增測試在安全守門讀 I0 後交錯寫入 I1，涵蓋自行計算與外部提供 scope 的兩條路徑，確認失敗時工單與 metadata 不變。舊實作已重現斷言失敗，修正後通過。
- 封存：正式規格內容放入既有 ArchiveChecksPlan，由 backend 在驗證計畫基底後寫入。FS 在同一 checks 鎖內先驗基底、完成 staging 驗證，再寫正式規格及發佈；Bridge 仍在既有 staged transaction 寫入。snapshot 備份仍先完成，錯誤保留原始型別與備份位置。Core 新測試在提交前更新 index，重現舊實作改寫既有規格且新增正式規格的錯誤；修正後兩者都不變。另補 FS 真實 backend 的成功／metadata 衝突案例。
- 重複程式碼：以 `report_decisions` 共用三處成對報告的決策載入。未新增另一套驗收判斷或改動規格／設計／任務。
- 同類核對：兩個 `write_stamp` caller 都傳 proof；Core 預設 Store、FS、Bridge、TestStore 的封存接點都消費計畫內的正式規格；三個原決策迴圈都改接共用函式，只有該函式保留 `decisions.extend(read_decisions(...))`。

### 修正後驗證

- `cargo build -p speclink-cli --locked` 通過；最後保留錯誤型別的調整後再建置一次，通過。
- `cargo test --locked -p speclink-core -p speclink-fs -p speclink-host -p speclink-cli -p speclink-desktop-core -p speclink-server`：2744 passed、3 ignored、0 failed；另有 3 項 Server PostgreSQL 測試因未設定資料庫跳過。涵蓋核心生成資產、FS 實際封存、Host 成對 Remote、安全接受、CLI 與 Server 完整流程。
- 最後調整後再跑 `cargo test -p speclink-core --lib --locked`：1075 passed、1 ignored、0 failed。
- scripts 全組：353 passed、33 skipped、0 failed；`git diff --check` 通過。
- 未重跑未改動的前端 UI、Node 原生 binding 與 Tauri 殼；三平台及這些介面由既有 CI 守門，但這批未提交修正**尚未在遠端 CI 執行**。沒有把先前 HEAD 的 CI success 當成本次修正的結果。

### 三項複驗（主流程備援，分開判斷）

因子程序額度限制，主流程依技能備援順序完成 Standards、Correctness、Verify，各自只判原 finding、凍結修正與直接回歸，未重開 discovery。

- Review validation patch：`sha256:b44b19d540af2862159a1daa78db893578759128c2f36dace8c7c309c5070546`。
- Verify validation patch：`sha256:f4460bdd66399f743b4d841286bd2332d0b9fb3581ff0794c717cdb8d90d8a13`。
- 兩站各 11 paths。`finding` 段為原問題檔案；其餘 `new` 段包含為本次修正接線的 Store／Bridge／FS、測試及共用函式接點，以及這些檔案在原變更已有的上下文。已逐檔對照用途：沒有混入其他工作的修改；既有上下文不作重新 discovery，也不當成本輪新增問題。
- **Standards**：原 possible Duplicated Code 已解決；三處都接同一函式。本輪未發現修正直接造成的新規則違反。未解 0，最高嚴重度無。
- **Correctness**：兩項原 CRITICAL 已解決，已確認 FS 的指紋比較在持鎖提交前執行、封存規格寫入在基底比較之後，Bridge 仍為 staged 寫入。未發現本次修正直接造成的新回歸。未解 0，最高嚴重度無。
- **Verify**：兩項原 WARNING 已解決，新增失敗回歸對應原指紋與封存衝突條件。程式任務仍 59／59；人工 12.3 仍延期；規格／設計意圖未變。未解 0。

依序以 CLI 記錄 Review Round 2、Verify Round 2，皆不蓋章；沒有保留接受項。本輪修復複驗通過，不代表前輪尚未逐項完成的完整 discovery 已補完。該限制仍保留供使用者決定後續檢查。修正未提交／推送，亦未合併、打包或封存。

## 使用者選擇補完覆核後

接續結果另存 [未完成覆核補查](quality-discovery-completion-2026-10-22.md)。保留本頁及兩站 Round 2 原始結果；補查確認的 Node 公開型別問題與規格落差不冒充修復造成的 validation 回歸，也不以既有 clean 輪次宣稱整體通過。

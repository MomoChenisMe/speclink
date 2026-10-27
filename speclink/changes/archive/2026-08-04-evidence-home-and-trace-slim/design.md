## Context

evidence 記錄（TouchedRecord：v1 touched 檔案清單＋v2 逐任務 entries）目前由 speclink-core 的 tasks 模組讀寫於 `.speclink/touched/<change>.json`；`.speclink/` 整層 gitignored，記錄不進版控、封存後由技能流程刪除。@trace 由 archive 注入正典，含 source、updated、code 三欄；code 清單在 evidence 缺席時退回掃描 git dirty worktree，並因此衍生 bulk archive 的整潔工作樹硬性要求。Host 的 `check_archive_evidence`（含 VerifyBundle 產生與 staleness 判定）已完整實作但全 repo 無生產呼叫端。消費端盤點：@trace 僅 source 欄位被 UI 讀取；code 與 updated 無任何程式讀者。討論 post-archive-spec-value 裁決本刀範圍：evidence 隨行、trace 瘦身、本地 gate 強制。gate 落地後，討論 evidence-gate-false-blocks 以沙盒探針推翻其觸發前提（純規格 change 零證據被擋、先 commit 再勾的 stale 死路、拆門後 basis digests 零讀者），裁決反轉為：gate 全套拆除、帳瘦身、零證據封存留一行提示——本設計的 gate 決策段隨之改寫為拆除決策，evidence 搬家與 trace 瘦身不受影響。

## Goals / Non-Goals

**Goals:**

- evidence 記錄隨 change 生命週期移動：committed、封存隨行、discard 隨滅
- @trace 收斂為 source＋updated 且一律注入，正典不再承載檔案清單
- evidence 帳收斂為「每一欄都有讀者的歷史事實」，封存不因 evidence 被擋
- 零證據封存以一行不擋人的提示提醒，AI 代理可自查是否漏走 apply 流程

**Non-Goals:**

- 不動 v1 檔案清單語意與 v2 其餘欄位（僅搬家＋移除 basisDigests）
- 不做桌面對零證據提示的任何 UI 呈現；不做測試結果（pass 數、指令）記入帳的欄位擴充
- 不清理既有正典的 374 個 @trace code 清單
- 不做 Store trait 統一讀 evidence、fingerprint／CAS、UI 變更

## Decisions

### evidence 檔的家：change 目錄 dot 檔與雙位置相容讀取

寫入位置改為 `openspec/changes/<change>/.evidence.json`，比照同目錄 `.openspec.yaml` 的機器寫入 dot 檔慣例（不是使用者 artifact，不進 artifact 清單）。serde 結構完全不變（version／change／touched／entries，camelCase 欄位既有），能讀既有檔案。讀取順序：change 目錄優先，缺席時回退舊路徑 `.speclink/touched/<change>.json`（唯讀）；寫入一律落新位置，成功後順帶刪除舊檔（內容已由回退讀取帶入）；封存亦比照 `.started` 標記清除舊檔、discard 既有刪除不變。不做整批遷移。原「孤兒不清理（無害）」的前提不成立——殘留舊檔會被日後同名 change 的回退讀取當成自己的記錄（review 第一輪修正）。落點仍在 speclink-core 的 tasks 模組（領域邏輯、不寫死儲存媒介之外的路徑組裝維持經 Workspace）。

否決替代案：留在 .speclink 並於封存時複製進封存目錄（討論已否決——不進 git 的根本問題不動）；非 dot 檔名 evidence.json（會被視為使用者 artifact 出現在清單與檢視）。

混用新舊版本 CLI 於同一 checkout 時，舊版寫舊路徑、新版已寫過新路徑後不再回看舊路徑——接受此邊界（同 checkout 混版使用不在支援範圍），記於 Risks。

### @trace 一律注入：source 與 updated 兩欄

trace 區塊縮為 source 與 updated，ADDED／MODIFIED 物化時一律注入，不再以檔案清單有無決定；archive 對 git dirty worktree 的 trace fallback 掃描整段移除（tasks 模組供任務證據歸屬使用的 dirty 檔掃描不受影響）。RENAMED 保留原 trace、REMOVED 隨區塊消失，維持現行。區塊前的間距規則（MODIFIED 與 ADDED 的空行差異）維持現行以縮小 golden 變動面。既有正典的 374 個含 code 區塊原樣留置：UI 僅解析 source、無功能影響，後續要清屬機械處理可另行提案。

### evidence gate 反轉拆除與帳瘦身（討論 evidence-gate-false-blocks）

gate 已於本 change 內實作完成，探針隨即推翻其前提，據裁決整套拆除——因 change 尚未封存，正典從頭到尾不會出現守門需求。拆除面：archive 流程無任何 evidence 判斷；`ArchiveOptions` 無 waive_evidence；CLI 旗標、protocol／server 的 waiveEvidence 查詢參數、remote client 參數、desktop／node 呼叫端預設值全數移除（wire 契約回到本刀之前，未曾發版故無相容性議題）。帳瘦身：`EvidenceEntry` 移除 `basis_digests` 欄位、complete() 不再寫入；core 的 evidence 判定模組（本刀先建）與 host 的 evidence.rs（VerifyBundle 產生、judge_staleness、check_archive_evidence——前刀所建、全 repo 無生產呼叫端）整檔刪除，對應正典需求以 REMOVED 撤掉。`current_basis_digests` 計算函式保留於 tasks 模組——drift 現場計算是它僅存且真實的讀者。既有含 basisDigests 的 v2 記錄：serde 忽略未知欄位照讀，不遷移不清理。

零證據提示：`ArchiveOutcome` 增 `evidence_recorded: bool`（TouchedRecord entries 非空即 true）；CLI 於 false 時在 stderr 印恰一行提示——`note: no task evidence recorded for change '<name>' — fine for spec-only changes; otherwise check that tasks went through apply`——不影響 exit code 與封存結果；有 entry 時一字不印。core 維持零列印——事實入 outcome、呈現歸 CLI；desktop 不呈現（Non-Goals）。

否決替代案：留 sha 不拆（零讀者資料即概念負債；「留給遠端 Phase 2」不成立——server 應自記自判，client 自報指紋不可信）；每勾必記修門（可解誤擋但補丁鏈持續延長）；桌面放行按鈕（門不存在則無放行對象）。

### bulk 整潔工作樹守門移除

bulk archive 拒絕 dirty worktree 的前置檢查整段移除——其存在理由（dirty 檔會混入每個 change 的 @trace code 清單）已隨清單移除而消失。bulk 其餘預檢（任務完成度、過期 delta）不動。

### 技能文字：刪除步驟消失與路徑更新

archive 技能移除「touched 記錄的刪除排在封存與提交之後」整段（記錄隨目錄移動後不存在刪除步驟）、@trace 來源敘述改為「source 與 updated、一律注入、無檔案清單」、bulk 段的整潔工作樹要求移除；不得含任何 evidence 守門或放行旗標敘述，另敘明零證據提示行的意義（見到提示即確認該 change 是否走過 apply 流程）。commit 技能的檔案歸屬來源改指 `openspec/changes/<change>/.evidence.json` 並敘明該檔本身歸入該 change 的提交。渲染產物由 render_golden 保護，golden 同批更新。

## Implementation Contract

- **evidence 寫入**：speclink task done 產生的證據寫入 `openspec/changes/<change>/.evidence.json`；對帶舊路徑記錄的 change，讀取（archive gate、drift、commit 歸屬）回退舊檔且行為不變；下一次寫入後新位置成為唯一來源，舊檔於寫入成功後移除、封存時亦清除。discard 刪 change 目錄即帶走證據。
- **trace**：封存後正典的 @trace 僅含 source 與 updated 兩行、ADDED／MODIFIED 一律注入；零證據的封存同樣注入。正典不再出現 code 清單（既有區塊除外）。
- **封存與提示**：speclink archive 不因 evidence 缺席或內容拒絕封存；change 無任何 v2 entry 時 stderr 恰一行提示、exit code 與封存結果不變；有 entry 時一字不印。bulk archive 於 dirty worktree 下不被整潔要求擋下（不變）。
- **介面**：CLI help 與 wire 無 waive-evidence／waiveEvidence 蹤跡；`ArchiveOutcome.evidence_recorded: bool`；`EvidenceEntry` 無 basis_digests；host 無 evidence 模組；含 basisDigests 的既有 v2 記錄照讀且 all_files 不變。
- **驗收**：speclink-core 單元測試涵蓋雙位置讀取、寫入落新位置、trace 兩欄一律注入、evidence_recorded 兩態（零 entry false／有 entry true 且皆封存成功）、含 basisDigests 舊記錄相容；render_golden 斷言渲染產出無守門與 waive-evidence 敘述殘留、零證據提示敘述到位；crates/speclink-cli/tests/ 整合測試含零證據提示的 --no-color 斷言（恰一行、exit 0）與 --help 無旗標斷言；desktop artifact 清單不出現 .evidence.json 的斷言不變。
- **範圍界線**：in scope——tasks 模組讀寫路徑與帳瘦身、archive trace 與零證據提示、gate 與旗標的全套拆除（core／command／CLI／remote／server／desktop 呼叫端）、host evidence 模組刪除、兩份技能文字、測試與 golden。out of scope——v1 檔案清單語意、Store trait 化、既有正典 trace 清理、桌面 UI 呈現、測試結果欄位擴充、遠端 server 端 evidence 記錄（Phase 2 自行設計）。

## Risks / Trade-offs

- [golden 與 CLI 測試大面積變動掩蓋回歸] → 刻意變更逐項列於提案相容性影響段；trace 瘦身與 gate 拒絕的 golden 分開更新審閱
- [零證據提示被當噪音忽略] → 僅零 entry 時出現、恰一行；archive 技能敘明其意義
- [拆除面廣、wire 殘跡遺漏] → 以 --help 斷言與全域 grep waive 收攏；remote／server 測試還原後全綠為準
- [.evidence.json 意外出現在桌面 artifact 清單或觸發 watcher 噪音] → 比照 .openspec.yaml 的 dot 檔過濾；以 desktop-core 測試釘住清單不含該檔
- [同 checkout 混用新舊版 CLI 導致證據分家] → 明示不支援；讀取回退僅為升級時的一次性銜接
- [Windows 路徑與換行] → 路徑一律經 Workspace／PathBuf 組裝；digest 比對沿既有 bundle 實作，不新增平台分支
- [host evidence 模組刪除誤傷 drift] → current_basis_digests 留在 tasks 模組，drift 既有測試釘住現場計算路徑

## Migration Plan

無強制遷移。in-flight change 的舊路徑記錄由回退讀取銜接，下次任務證據寫入自然落新位置；舊位置檔案於寫入成功後或封存時順帶刪除，不做整批遷移。既有 v2 記錄的 basisDigests 欄位：讀取端忽略未知欄位照讀，不遷移不清理。回滾即還原版本：新位置檔案對舊版不可見（舊版讀舊路徑），必要時手動搬回即可。與 archive-fail-closed-merge 同期在途：規格 delta 採 REMOVED＋ADDED 改寫，封存順序不受其新守門影響；惟兩刀同動 archive 流程，實作依序進行避免同檔互踩。

## Open Questions

（無——檔名、旗標名、判定落點與清理範圍已在本設計定案）

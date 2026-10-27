## Why

change 的 touched／evidence 記錄目前存於 `.speclink/touched/<change>.json`，而 `.speclink/` 整層被 gitignore——證據不進 git、跨機器不存在、commit 後即被技能流程刪除，@trace 的 code 清單因此成為檔案歸屬在 repo 內唯一被 commit 的紀錄。但盤點消費端後確認：@trace 的 code 與 updated 欄位沒有任何程式讀者（UI 只讀 source），code 清單卻是 dirty worktree 猜測、bulk archive 整潔工作樹硬性要求與平行 session 污染事故的共同根源。同時，Host 的 archive evidence gate 檢查函式全 repo 無生產呼叫端，封存紀律實際上無人強制。討論 post-archive-spec-value 裁決：evidence 隨 change 目錄移動、@trace 瘦身、本地 archive 強制 evidence gate。

實作完成後的討論 evidence-gate-false-blocks 以沙盒探針推翻了 gate 的觸發前提：evidence 僅在 task done 當下有新髒程式檔時寫入，導致純規格 change 走完全部任務仍零證據被擋、「先 commit 再勾任務」落入重跑也無解的 stale 死路——拒絕訊息的兩條補救指示都走不通；而拆門後帳內 basis digests 全 repo 零讀者（drift 現場重算、commit 只讀檔案清單），遠端 Phase 2 應由 server 自記自判、不以本機自報指紋為地基。裁決（反轉 post-archive-spec-value 的「本地強制 evidence gate」一項）：gate 全套拆除、evidence 帳瘦身為純歷史事實，零證據封存僅留一行不擋人的提示。

目標使用者是透過 AI 代理跑 SDD 的開發者，情境對應 apply（寫入證據）、archive（trace 與提示）、commit（檔案歸屬）三個工作流階段與對應技能。

## What Changes

- evidence 記錄搬家：寫入 `openspec/changes/<change>/.evidence.json`（比照 `.openspec.yaml` 的機器寫入 dot 檔），隨 change 被 commit、封存時隨目錄移入封存區、discard 隨目錄消失；讀取端先讀 change 目錄、缺席時回退舊路徑 `.speclink/touched/<change>.json`（唯讀相容），寫入一律落新位置；舊位置檔案不遷移——新位置寫入成功後與封存時順帶刪除（內容已由回退讀取帶入），避免同名新 change 經回退把已亡 change 的記錄讀成自己的帳
- @trace 瘦身：只含 source 與 updated 兩欄、**一律注入**（不再依檔案清單決定注入與否），code 清單與其 dirty worktree 產生邏輯整段移除；既有正典中 374 個含 code 清單的 @trace 區塊原樣留置
- 本地 archive 不設 evidence gate（討論反轉）：封存不因 evidence 缺席或內容被拒；change 無任何 v2 entry 時 CLI 於 stderr 印一行提示（無旗標、不影響 exit code 與封存結果），供 AI 代理自查是否漏走 apply 流程；有 entry 時不印
- evidence 帳瘦身：EvidenceEntry 移除 basisDigests，帳僅存 taskId／taskDesc／actor／repo／headCommit／touchedFiles／recordedAt——每一欄都是有讀者的歷史事實；staleness 判定與 VerifyBundle（core 判定模組、host `check_archive_evidence`）整套移除，對應正典需求以 REMOVED 撤掉；`current_basis_digests` 計算函式保留（drift 現場計算的讀者）；既有含 basisDigests 的 v2 記錄讀取端忽略未知欄位照讀
- bulk archive 的整潔工作樹硬性要求移除（其存在理由——防 dirty 檔污染 @trace code 清單——已隨清單移除而消失）
- 技能文字同步：archive 技能移除「封存後、提交後刪除 touched 記錄」步驟與 @trace 檔案清單來源敘述；commit 技能的檔案歸屬改讀新路徑

**相容性影響**：

- @trace 輸出格式改變（新封存起生效）：屬刻意變更，render_golden 與 CLI 整合測試同批更新；UI 的 source 解析（packages/ui/src/trace.ts）與溯源變更數統計不受影響
- 行為面：封存的通過與否不因 evidence 而變（與 gate 導入前一致）；唯零證據封存新增一行 stderr 提示
- `--json` 與 wire 契約：不變——waiveEvidence 欄位於本 change 內先加後拆，未曾發版
- `.evidence.json` 為新增受版控檔案：apply 期間的證據寫入會出現在 git status 與 commit 檔案集，commit 技能將其歸入該 change 的提交；v2 entry 不再含 basisDigests，既有含該欄位的記錄照讀
- host 對外符號：`check_archive_evidence` 與 VerifyBundle／staleness 型別移除——全 repo 無生產呼叫端（原 advisory 級），不構成破壞
- 規格面採 REMOVED＋ADDED 改寫（非 MODIFIED），對封存順序無額外前提；與 archive-fail-closed-merge 同期在途、封存順序不拘，惟兩刀皆改 archive 流程，實作 SHALL 依序進行避免同檔互踩

## Non-Goals

- 不動既有正典中的 374 個 @trace code 清單（歷史殘影無功能影響；後續要清可另行機械處理）
- 不做 remote 經 Store trait 統一讀 evidence（本刀僅搬檔案位置；Store 化屬後續議題）
- 不動 evidence 記錄的內容欄位與 v1／v2 格式語意（僅搬家）
- 不動 SpecDrawer 的 trace footer 呈現與 desktop UI
- 不做 requirement fingerprint／CAS 與穩定 requirement ID

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `verify-evidence`: evidence 記錄的家改為 change 目錄（含舊路徑回退讀取）且帳瘦身（「task done 寫入逐任務 evidence」MODIFIED——移除 basis digests）；「archive trace 由 evidence 建立」「VerifyBundle 固定驗證基準」「evidence 的 stale 判定」三需求 REMOVED；ADDED「archive trace 注入與零證據提示」（trace 兩欄一律注入＋零證據提示行）
- `archive-skill`: 「touched 記錄的刪除排在封存與提交之後」需求移除（刪除步驟不復存在）；「@trace 來源敘述與引擎行為一致」以 REMOVED＋ADDED 改寫為無檔案清單、無守門敘述的新敘述（含零證據提示行的意義）並移除整潔工作樹要求

## Impact

- Affected specs: verify-evidence（modified）、archive-skill（modified）
- Affected code:
  - Modified: crates/speclink-core/src/tasks.rs（TouchedRecord 讀寫路徑：change 目錄優先、舊路徑回退）
  - Modified: crates/speclink-core/src/archive.rs（trace_block 瘦身、一律注入、移除 git dirty fallback；無 evidence gate，ArchiveOutcome 增 evidence_recorded 事實）
  - Modified: crates/speclink-core/src/tasks.rs（TouchedRecord 讀寫路徑＋EvidenceEntry 移除 basis_digests；current_basis_digests 保留）
  - Modified: crates/speclink-cli/src/commands.rs（bulk 整潔工作樹守門移除、零證據 stderr 提示呈現）
  - Modified: crates/speclink-core/assets/skills/archive.md、crates/speclink-core/assets/skills/commit.md（路徑與敘述更新，無守門敘述）
  - Modified: crates/speclink-core/tests/render_golden.rs 與 crates/speclink-cli/tests/ 受影響整合測試（golden 同批更新）
  - New: (none)（core 的 evidence 判定模組於本 change 內先建後拆，對基線無淨變化）
  - Removed: crates/speclink-host/src/evidence.rs（VerifyBundle／staleness 判定／check_archive_evidence——正典需求同步 REMOVED）

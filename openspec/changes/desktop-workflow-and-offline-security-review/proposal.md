## Why

- 透過 AI 代理跑 SDD 的開發者、PO 與 PM，需要調整桌面專案順序、在公司 Proxy 內查看更新狀態，並理解預設審查規則。
- 審查／驗證蓋章後目前刪除工單，安全報告另存裝置，團隊無法在提案內回看，需保存完整紀錄供後續查詢。
- 需要保留內建基本掃描，並加入使用完整本機標準資料的深度審查 skill，追查架構、業務邏輯與跨模組資料流。
- 團隊需依專案選擇是否把安全結果納入驗收，並明確記錄風險接受與封存保留政策。
- 提案動機應與變更內容同樣容易逐項閱讀；所有專案預設條列起草與顯示，不依賴單一專案設定。

## What Changes

- 左側專案列支援拖曳及上移／下移，順序存於本裝置並於重開還原，不影響專案身分或目前選取。
- 更新頁將內附版本／更新日誌與網路檢查狀態分開；Proxy、TLS、407、解析及畫面失敗須有明確狀態與診斷，保留既有來源與簽章驗證。公司端根因尚未確認，以可重現測試及人工環境驗收完成修復。
- 「查看 12 條」每條增加問號說明，說明用途、例子與預設只列建議的效果，共用引擎的名稱清單。
- 標準下載／匯入即備齊原文、對照表及修復指南；新增本機知識庫準備入口，審查期間不得取得任何網路資源。附件資料清單僅作結構參考，不採浮動來源、虛構版本或未核對的檔名。
- 保留內建基本掃描，新增 speclink-security-review skill；沿用 security_verification 的主開關、範圍與標準版本，審查五種語言的架構、業務授權及跨模組資料流，依指定九項結構產生遮罩報告。
- 審查／驗證完成紀錄與安全報告放進提案的 checks 目錄；進行中工單仍用原路徑，蓋章前先保全紀錄。Desktop 的文件樹及已封存詳情可查歷次紀錄。
- 在產出流程新增 check_records.archive 的 review／verify／security 布林值，皆預設 true：控制封存時保留三類完整紀錄；蓋章後、封存前仍一律可查，調整設定不回溯刪除既有紀錄。
- 安全政策新增 required_for_acceptance=false；主開關下方新增「納入驗收條件」checkbox。勾選時，未處理的 CRITICAL／HIGH／MEDIUM 須修復或有明確接受理由，LOW 只列建議；未審查、缺必要資料或必要項未驗證不得通過。驗證蓋章與封存共用判定，既有 --accept 不豁免安全要求。
- 風險接受是可查詢的具體紀錄，綁報告、finding 與來源指紋；程式碼或政策變更後須重新審查，不能靠舊報告通過。參考模式不新增驗收阻擋。

- 所有專案的提案 Why 預設條列起草，共用提案畫面將 Why、What Changes、Proposed Solution 的散文區塊也逐項顯示；活動與已封存頁一致，保留原文及巢狀內容。

## Capabilities

### New Capabilities

無。掃描初查了 security-standard-data、security-review-report、desktop-config 等既有能力，本次擴充既有資料、審查及文件生命週期，不另建同義能力。

### Modified Capabilities

- `propose-skill`: 所有專案的提案 Why 預設以一事一項條列起草，沿共用模板與技能生成。
- `desktop-app`: 提案共用預設條列顯示、專案排序、更新頁失敗時仍可讀內附資訊、歷史文件樹。
- `desktop-config`: 十二條說明、封存紀錄設定與安全驗收 checkbox。
- `desktop-network-diagnostics`: 公司 Proxy 更新問題的可重現診斷及錯誤呈現。
- `workflow-config`: 紀錄封存及安全驗收政策、預設值與部分更新契約。
- `security-standard-data`: 版本固定的完整離線知識資料及安全的本機準備。
- `security-review`: 深度 skill、保留基本掃描、結果提交及驗收判定。
- `security-review-report`: 提案內報告、九項說明、風險接受與舊報告相容。
- `review-station`: 蓋章保存歷次工單與查詢，不影響既有章／未結工單語意。
- `verify-station`: 蓋章保存工單並檢查已啟用的安全驗收要求。
- `change-lifecycle`: 安全驗收與封存時紀錄保留，封存後仍可查詢。
- `client-protocol`: 歷史紀錄、安全報告與裁決的共用 wire 型別及錯誤。
- `review-skill`: 已完成審查紀錄的交棒與查詢指引。
- `verify-skill`: 安全驗收、紀錄查詢及缺資料的技能行為。
- `ui-copy-vocabulary`: 參考模式、納入驗收及紀錄保存的共用用詞。

## Impact

- 主要範圍：speclink-core、speclink-fs、speclink-host、speclink-protocol／remote、speclink-cli／server／node、speclink-desktop-core、apps/desktop、packages/ui。
- Planned new files（本次建立）: `crates/engine/speclink-core/assets/skills/security-review.md`、`crates/engine/speclink-core/assets/security/spec_registry.json`、`crates/engine/speclink-core/src/security/acceptance.rs`、`crates/engine/speclink-core/src/quality/history.rs`、`crates/engine/speclink-fs/src/security/knowledge.rs`；新檔負責判定、紀錄或安全資料準備，不增加純轉送層；公司環境證據新增於 openspec/changes/desktop-workflow-and-offline-security-review/evidence/company-proxy.md。
- Affected code — Modified: `apps/desktop/src/tabs.ts`、`apps/desktop/src/store.ts`、`apps/desktop/src/components/ProjectRail.tsx`、`apps/desktop/src/views/settings/UpdatesPage.tsx`、`apps/desktop/src/core/updater.ts`、`apps/desktop/src-tauri/src/updater_commands.rs`、`apps/desktop/src/views/project-settings/WorkflowPage.tsx`、`apps/desktop/src/views/project-settings/ReviewRulesPage.tsx`、`apps/desktop/src/views/project-settings/useProjectSettings.ts`、`packages/ui/src/components/BaselineSmellList.tsx`、`packages/ui/src/i18n.tsx`、`apps/desktop/src/i18n/messages.ts`、`packages/ui/src/components/ChangeDetailPage.tsx`、`packages/ui/src/components/ArchivedDetailPage.tsx`。
- Affected code — Modified: `crates/engine/speclink-core/src/security/policy.rs`、`crates/engine/speclink-core/src/security/report.rs`、`crates/engine/speclink-core/src/quality/station.rs`、`crates/engine/speclink-core/src/lifecycle/archive.rs`、`crates/engine/speclink-core/src/workspace/config.rs`、`crates/engine/speclink-core/src/workspace/skills.rs`、`crates/engine/speclink-fs/src/lib.rs`、`crates/engine/speclink-fs/src/security.rs`、`crates/engine/speclink-fs/src/security/reports.rs`、`crates/engine/speclink-fs/src/security_packages.rs`、`crates/host/speclink-host/src/security.rs`、`crates/host/speclink-host/src/security_data.rs`、`crates/host/speclink-host/src/bridge.rs`、`crates/host/speclink-host/src/network.rs`。
- Affected code — Modified: `crates/protocol/speclink-protocol/src/command.rs`、`crates/protocol/speclink-protocol/src/query.rs`、`crates/protocol/speclink-remote/src/client.rs`、`crates/adapters/speclink-cli/src/verbs/security.rs`、`apps/desktop/core/src/security.rs`、`crates/engine/speclink-core/assets/skills/review.md`、`crates/engine/speclink-core/assets/skills/verify.md`、`crates/engine/speclink-core/assets/skills/archive.md`、`crates/engine/speclink-core/assets/skills/config.md`、`crates/engine/speclink-core/tests/golden/assets.lock`、`openspec/LANGUAGE.md`、`docs/product-status.zh-TW.md`、`docs/product-status.md`。Server／Node 同步共用契約與測試，不平行重刻判定。
- CLI 新增 security kb prepare [--root PATH] [--json]（無 stdin）；security review add-round CHANGE --stdin [--json]、security review accept CHANGE --stdin [--json]（JSON stdin）；security review show CHANGE [--json]；review history／verify history CHANGE [--id ID] [--json]（無 stdin）；workflow-config check-records --stdin [--dry-run]（YAML stdin）。新增命令成功0、操作錯誤1、參數錯誤2；準備取消130，JSON／人眼成功結果至 stdout，進度／錯誤至 stderr，支援全域 --no-color。
- 相容性影響：既有基本掃描規則、無 --change 的 security run／report 及未啟用驗收的既有輸出維持回歸對照；指定變更的安全報告改存提案內，security report 增加 --change 查詢。review／verify stamp 新增歷史保存的檔案效果，命令人眼原摘要不變；新增政策及 wire 欄位有預設值。舊裝置報告唯讀可查，可明確匯入匹配變更，不能直接作新驗收依據。新資料包可匯出為擴充的 v1 包，舊包仍可供基本掃描，缺指南時深度審查明示缺件。
- 技能產物：新增 security-review，調整 review／verify／archive／config 的交棒；Claude、Codex、Copilot 及自訂工具共用同一原稿，透過既有 init／update 生成，更新資產版本與 golden，不手改單一工具副本。

- 追加顯示範圍：`packages/ui/src/components/ProposalDoc.tsx`、`packages/ui/src/__tests__/proposalDoc.test.tsx`、`crates/engine/speclink-core/assets/schema/spec-driven/proposal.template.md`、`crates/engine/speclink-core/assets/skills/propose.md`；共用入口涵蓋各專案與活動／封存提案。

# Verify discovery 逐項追溯表

原凍結HEAD：46803b2b；patchHash：sha256:b7aa741a6239f224efea728cda7b1fed2392552f8fec6610522b3077da47fc2a。
這是原 discovery 補查的逐項報告，由主流程保存進 evidence 供換機接續；不改票／章，不開 Round 3。

主流程逐檔重新計數：17 份規格、46 Requirements、133 Scenarios、7 Examples。前輪的 143 是加總錯誤，原歷史保留，本表予以更正。需求為 45 PASS、1 partial；情境為 125 PASS、3 partial、5 manual（其中 4 項為已核讀文件文字，1 項為延期的公司實機操作）。PASS 是證據映射，不表示全數端到端實機驗收。

PASS只表示條文已有實作與測試／靜態原稿／人工文字的具體對應，不表示本代理重新執行測試，也不是安全PASS。partial列精確缺口；manual分別說明已人工核讀的文字與尚未執行的實機項。#後為函式或測試名。先用manifest逐檔凍結patch定位；直接相鄰及既有未改測試在下文明標為patch外證據。
主流程已另確認原2項修復（verify proof/index競爭、本機封存canonical部分效果），本表不以原低階happy-path測試推論競爭已解；Round2判定沿原修復驗證。

## 重要缺口

- node-sdk「渲染API」條文仍要求instructions.render與options.store=fs|remote；src/render.rs的RenderOptions無store，也只有skills_list/skills_render。index.js具體出口搜尋只找到skills list/render。這是舊API退役後本delta全文未一致，需使用者選改artifact或實作。
- node-sdk Antigravity：主流程新確證package.json公開types=index.d.ts，而手工index.d.ts漏target antigravity。Rust/執行期測試接受，但最小TS引用由tsc回唯一TS2322、exit2。這是實際錯誤行為，主流程列WARNING並scope changed/failed保票，不另造Round3。原manifest誤分類該手工公開契約為generated，原path-only規則因此沒有足夠型別驗證。
- security-review-report「有dirty檔與無Git」缺真Gitcommit後dirty檔的HEAD參考＋當次bytes digest整組測試。已查具體report.rs、host/security.rs、host/security/deep.rs、fs/security/scan.rs與CLI security測試，只有null/digest長度、無Git來源變動fixture。實作存在，不推測一定錯。
- workspace-tools Example「--tools值的解析」只有vscode單值列缺相同輸入：目前單元與整合均測claude,vscode；不是unknown工具拒絕缺實作。此與dirty是測試追溯觀察，沒有錯誤行為證據，不藉此另開Round3。
- [M]12.3公司打包Proxy驗收依要求延期，不算缺實作、不列CRITICAL。真NAS/多主機UNC及Antigravity實機未宣稱測過。

## 路徑縮寫（每筆證據均由下列完整路徑與#測試名組成）

- `CLIsec` = `crates/adapters/speclink-cli/tests/it/security.rs`
- `CLIrev` = `crates/adapters/speclink-cli/tests/it/review_verbs.rs`
- `CLIver` = `crates/adapters/speclink-cli/tests/it/verify_verbs.rs`
- `CLIarc` = `crates/adapters/speclink-cli/tests/it/archive_readiness_gate.rs`
- `CLIinit` = `crates/adapters/speclink-cli/src/verbs/init.rs`
- `ITinit` = `crates/adapters/speclink-cli/tests/it/init_tools.rs`
- `Init` = `crates/engine/speclink-core/src/workspace/init/tests.rs`
- `Config` = `crates/engine/speclink-core/src/workspace/config/tests.rs`
- `Station` = `crates/engine/speclink-core/src/quality/station/tests.rs`
- `Accept` = `crates/engine/speclink-core/src/security/acceptance.rs`
- `Report` = `crates/engine/speclink-core/src/security/report.rs`
- `FSreport` = `crates/engine/speclink-fs/src/security/reports.rs`
- `Checks` = `crates/engine/speclink-fs/src/checks.rs`
- `FSarchive` = `crates/engine/speclink-fs/src/archive_checks.rs`
- `Archive` = `crates/engine/speclink-core/src/lifecycle/archive/tests.rs`
- `Hostdeep` = `crates/host/speclink-host/src/security/deep.rs`
- `Hostsec` = `crates/host/speclink-host/src/security.rs`
- `Remote` = `crates/host/speclink-host/tests/security_remote.rs`
- `Bridge` = `crates/host/speclink-host/src/bridge.rs`
- `Golden` = `crates/engine/speclink-core/tests/it/render_golden.rs`
- `Node` = `crates/adapters/speclink-node/__test__/render.spec.ts`
- `Paths` = `crates/host/speclink-host/src/context.rs`
- `FS` = `crates/engine/speclink-fs/src/security.rs`
- `KB` = `crates/engine/speclink-fs/src/security/knowledge.rs`
- `Package` = `crates/engine/speclink-fs/src/security_packages.rs`
- `Data` = `crates/engine/speclink-core/src/security/data.rs`
- `Proposal` = `packages/ui/src/__tests__/proposalDoc.test.tsx`
- `Design` = `packages/ui/src/__tests__/designDoc.test.tsx`
- `Change` = `packages/ui/src/__tests__/changeDetailPage.test.tsx`
- `Archived` = `packages/ui/src/__tests__/archivedDetailPage.test.tsx`
- `Help` = `packages/ui/src/__tests__/baselineSmellHelp.test.tsx`
- `Settings` = `apps/desktop/src/__tests__/projectSettingsView.test.tsx`
- `SecData` = `apps/desktop/src/__tests__/securityDataPage.test.tsx`
- `Updates` = `apps/desktop/src/__tests__/updatesPage.test.tsx`
- `Updater` = `apps/desktop/src/__tests__/updater.test.ts`
- `Workspace` = `apps/desktop/src/__tests__/workspace.test.ts`
- `Rail` = `apps/desktop/src/__tests__/projectRail.test.tsx`


外部證據：既有Station各站守門、Config原有版本／patch／descriptor、ITinit原有init、Proposal/Design/Change原有文件結構與Node原有render矩陣測試不在新增hunks，但按具體測試名讀取確認。文檔人工核讀結果由主流程提供：README雙語249-250、CONTRIBUTING58-60；LANGUAGE281-310、messages104-111/776-783與479/1163、i18n54/598；company-proxy延期、tasks12.3與development雙語main合併後打包順序。未把外部證據當成本次新增。

## change-lifecycle

`openspec/changes/desktop-workflow-and-offline-security-review/specs/change-lifecycle/spec.md`：2 Requirements／4 Scenarios／0 Examples。

| 編號 | Requirement | 狀態 | 實作定位 |
| --- | --- | --- | --- |
| R1 | 必要安全驗收的封存判定 | PASS | `crates/engine/speclink-core/src/lifecycle/archive.rs#archive；security/mod.rs#require_acceptance` |
| R2 | 封存時按類別保留完成紀錄 | PASS | `crates/engine/speclink-core/src/quality/history/archive.rs#prepare_archive_checks；crates/engine/speclink-fs/src/archive_checks.rs#archive；Bridge#archive_change_with_checks` |

| 編號 | Scenario | 狀態 | 測試／證據與限制 |
| --- | --- | --- | --- |
| S1 | 沒跑驗證也不能略過必要安全檢查 | PASS | CLIarc#required security archive guard；缺報告且無verify章仍拒絕 |
| S2 | 參考模式與明確接受 | PASS | Accept#acceptance_matrix_from_spec；reference/HIGH接受/LOW與既有archive守門 |
| S3 | 三類保留的八種組合 | PASS | Archive#archive_records_obey_all_eight_retention_combinations_and_keep_carried_tickets；8種選項/omitted/carry |
| S4 | staging或提交失敗 | PASS | FSarchive#staged_archive_keeps_active_records_on_every_precommit_failure；Bridge#archive_selection_commit_conflict_preserves_active_check_records；canonical已知修復由Round2另驗 |

## client-protocol

`openspec/changes/desktop-workflow-and-offline-security-review/specs/client-protocol/spec.md`：2 Requirements／4 Scenarios／0 Examples。

| 編號 | Requirement | 狀態 | 實作定位 |
| --- | --- | --- | --- |
| R1 | 檢查歷史與安全驗收的共用契約 | PASS | `crates/protocol/speclink-protocol/src/query.rs#CheckHistoryResponse；security.rs；crates/protocol/speclink-remote/src/client.rs` |
| R2 | 標準 error reason registry | PASS | `crates/protocol/speclink-protocol/src/error.rs#ErrorReason；remote/src/client.rs` |

| 編號 | Scenario | 狀態 | 測試／證據與限制 |
| --- | --- | --- | --- |
| S1 | Local與Remote型別一致 | PASS | Remote#paired_remote_publication_is_atomic_scoped_fresh_and_cas_guarded；host/history.query與protocol camelCase型別；Server reader_can_read_completed_review_history |
| S2 | 舊server與衝突 | PASS | crates/protocol/speclink-remote/tests/it/typed_client.rs#deep_security_commands_refuse_an_older_server_before_posting；Remote同revision第二次accept衝突 |
| S3 | reason 對映沿用現行訊息 | PASS | crates/protocol/speclink-remote/tests/it/typed_client.rs#every_registry_reason_maps_to_the_frozen_cli_message；既有8reason原訊息 |
| S4 | 未知 reason 不崩潰 | PASS | crates/protocol/speclink-remote/tests/it/client_errors.rs#err_unknown_reason_preserves_server_message_and_status；保message/status不panic |

## desktop-app

`openspec/changes/desktop-workflow-and-offline-security-review/specs/desktop-app/spec.md`：6 Requirements／20 Scenarios／2 Examples。

| 編號 | Requirement | 狀態 | 實作定位 |
| --- | --- | --- | --- |
| R1 | 本分支更新與原作者版本 | PASS | `apps/desktop/core/src/updater.rs#select_desktop_release；apps/desktop/src/views/settings/UpdatesPage.tsx；adapter/updater.ts` |
| R2 | 專案貢獻與原作署名 | PASS | `README.md；README.zh-TW.md；CONTRIBUTING.md` |
| R3 | 左側專案排序與裝置持久化 | PASS | `apps/desktop/src/tabs.ts#reorderTabs；store.ts#reorderProject；components/ProjectRail.tsx` |
| R4 | 更新頁內附資訊不依賴網路成功 | PASS | `apps/desktop/src/views/settings/UpdatesPage.tsx；core/updater.ts#updateCheckFailure` |
| R5 | 提案與已封存的檢查紀錄文件樹 | PASS | `packages/ui/src/components/CheckRecords.tsx#useCheckRecords；apps/desktop/src/adapter/checkRecords.ts；crates/host/speclink-host/src/history.rs` |
| R6 | 提案與設計章節以中文標籤呈現 | PASS | `packages/ui/src/components/ProposalDoc.tsx#ProposalSection；DesignDoc.tsx（既有未改）` |

| 編號 | Scenario | 狀態 | 測試／證據與限制 |
| --- | --- | --- | --- |
| S1 | 本分支有新版而原作者版號更高 | PASS | Updates#本分支與原作者版本獨立，沒有簽章描述檔仍可手動下載本分支；Updater#實際安裝build.3時只接受較新的本分支；desktop-core拒upstream包 |
| S2 | 只有未簽章安裝檔 | PASS | Updater#描述檔404才轉本分支手動更新，TLS失敗不被掩蓋；manual不進安裝流程 |
| S3 | 原作者查詢失敗且本分支正常 | PASS | Updates#原作者403／TLS失敗不覆蓋本分支，保留原成功時間；zh-TW/en、上次時間及版本/日誌 |
| S4 | 查看本分支貢獻說明 | manual | README雙語249-250與CONTRIBUTING58-60；主流程已人工核讀維護者/AI/原作、linked email/default branch/真共同作者；未改Git歷史 |
| S5 | 本機與遠端混排後重開 | PASS | apps/desktop/src/__tests__/tabs.test.ts#moves C before A and B and persists the active project；含remote locator；Workspace#保存後重開維持順序，且作用中專案與session不變 |
| S6 | 取消與保存失敗 | PASS | Rail#拖曳後%s正確提交或取消，不切換專案；escape/outside/drop；Workspace#裝置保存失敗或無效排列不變更狀態 |
| S7 | 更新元件缺席 | PASS | Updates#沒有updater仍有目前版本與內附日誌，只有網路動作停用；兩語與notesdialog |
| S8 | Proxy拒絕與背景失敗 | PASS | Updates#背景失敗保留最後結果、重試與診斷入口，不冒稱已最新；Updater#背景失敗仍記錄安全錯誤代碼及operationId，成功時清除舊錯誤 |
| S9 | 蓋章後與封存後查詢 | PASS | Change#顯示歷次審查、驗證與安全文件，內容唯讀並遮罩；Archived#已封存文件樹可查保存的歷次紀錄；CLI歷史與desktop-core章後查詢 |
| S10 | 切換專案與遠端失敗 | PASS | Change#切換專案時遲到的文件不覆蓋新的提案，讀取失敗不借用舊內容；同一提案遠端失敗保留唯讀紀錄；CheckRecords.errorLabel處理reauthentication_required |
| S11 | 提案模板章節成中文標籤 | PASS | Proposal#模板章節成中文段標題、依序進本頁目錄，英文標題不直出；patch外既有測試 |
| S12 | 章節標題為 16px 粗體 | PASS | Change#delta區段呈現色標標頭；既有text-base/font-semibold斷言，CSS對應16px；未做真瀏覽器computed-size量測 |
| S13 | 白名單外章節照排 | PASS | Proposal#白名單外的二級標題以原文成段，不整篇退回；Rollout |
| S14 | 無二級標題整篇退回 | PASS | Proposal#沒有二級標題時整篇照Markdown，無章節段 |
| S15 | 替代方案與影響段的逐項降級 | PASS | Proposal#替代方案含——的列拆成方案與落敗理由；影響段三組路徑/計數與散文降級 |
| S16 | 移除組的路徑籤沒有顯示鈕 | PASS | Proposal#移除組的路徑籤沒有顯示鈕，只有複製鈕；a/new.ts與c/gone.ts |
| S17 | 能力籤跳到規格與改寫提示框 | PASS | Change#變更文件的籤、數字磚、提示框與跳轉；desktop-app 3 modified/1 removed-scenario fixture與能力選取 |
| S18 | 提案的籤槽與數字磚 | PASS | Change#變更文件的籤、數字磚、提示框與跳轉；Refactor解法6/非目標5/能力1/檔案6 |
| S19 | 設計的決策卡與數字磚 | PASS | Design#決策段每個三級標題一張卡：預設收合、D籤與列點數；Change#設計文件數字磚依序為決策、目標／非目標、風險與取捨、開放問題；6/5-3/4/0 |
| S20 | 不同專案的散文提案預設列式 | PASS | Proposal#所有專案預設列式顯示散文與清單：%s；A/B三項連續編號、巢狀/連結/程式；空章節測試 |

## desktop-config

`openspec/changes/desktop-workflow-and-offline-security-review/specs/desktop-config/spec.md`：4 Requirements／10 Scenarios／0 Examples。

| 編號 | Requirement | 狀態 | 實作定位 |
| --- | --- | --- | --- |
| R1 | 安全標準資料存放路徑設定 | PASS | `crates/host/speclink-host/src/security_paths.rs#write_at/lease_at；apps/desktop/src/views/settings/SecurityDataPage.tsx` |
| R2 | 預設十二條的問號說明 | PASS | `packages/ui/src/components/BaselineSmellList.tsx；i18n.tsx` |
| R3 | 納入安全驗收與封存紀錄的設定 | PASS | `apps/desktop/src/views/project-settings/WorkflowPage.tsx；useProjectSettings.ts` |
| R4 | AI 工具名稱顯示技能目錄 | PASS | `apps/desktop/src/builtinTools.ts；views/project-settings/SpeclinkPage.tsx；components/WorkspaceChooser.tsx` |

| 編號 | Scenario | 狀態 | 測試／證據與限制 |
| --- | --- | --- | --- |
| S1 | 本機預設切換共用目錄 | PASS | Paths#standard_path_preferences_are_device_only_atomic_and_fail_closed；CLIsec#security_data_custom_root_preserves_outputs_and_disconnection_never_falls_back；UI UNC/reset |
| S2 | 共用位置唯讀或離線 | PASS | SecData#UNC路徑儲存失敗保留草稿，唯讀共用只准讀取／匯出，還原不搬檔；兩語offline unavailable；FS拒fallback |
| S3 | 儲存失敗或工作未完成 | PASS | Paths#standard_path_change_is_busy_while_a_reader_holds_its_snapshot；apps/desktop/core/src/security.rs#changing_paths_refuses_a_pending_job_before_touching_preferences；UI保draft |
| S4 | 十二條逐一查看 | PASS | Help#逐名涵蓋引擎的十二條用途與例子；直接解析REVIEW_BASELINE_SMELLS的12名，兩語逐個開關/Escape焦點 |
| S5 | 整份換掉仍能理解預設清單 | PASS | Help#replace模式清楚標示預設未套用；另Enter/Space與未知名稱測試 |
| S6 | 參考模式改驗收模式 | PASS | Settings#主開關關閉保留驗收選擇，儲存只送明確修改的欄位；預設/取消/reader；save不scan |
| S7 | 封存保留設定與遠端衝突 | PASS | Settings#CAS衝突保留驗收與保留草稿，確認後只套到最新快照；required/archive.verify draft與revision |
| S8 | 所有內建工具顯示目錄 | PASS | Settings#AI工具名稱顯示技能目錄，Antigravity儲存純工具名；四個無障礙label |
| S9 | 儲存新增與取消 Antigravity | PASS | Init#antigravity_coexists_with_codex_and_deselects_only_owned_skills；UI加選純名＋引擎取消只刪受管檔分層驗證 |
| S10 | 初始化與 checkout 選項一致 | PASS | apps/desktop/src/__tests__/workspaceChooser.test.tsx#WorkspaceChooser；四label checkout；App新增初始化label測試；預設claude沿pickerstate |

## desktop-network-diagnostics

`openspec/changes/desktop-workflow-and-offline-security-review/specs/desktop-network-diagnostics/spec.md`：2 Requirements／4 Scenarios／0 Examples。

| 編號 | Requirement | 狀態 | 實作定位 |
| --- | --- | --- | --- |
| R1 | 公司 Proxy 更新失敗的定位與呈現 | PASS | `apps/desktop/core/src/updater.rs#finish/check/download；network.rs；crates/host/speclink-host/src/network.rs` |
| R2 | 公司 Proxy 打包後驗收 | PASS | `openspec/changes/desktop-workflow-and-offline-security-review/tasks.md#12.3；evidence/company-proxy.md；docs/development.md；docs/development.zh-TW.md` |

| 編號 | Scenario | 狀態 | 測試／證據與限制 |
| --- | --- | --- | --- |
| S1 | 模擬 CONNECT 與407 | PASS | apps/desktop/core/src/network.rs#connect_tunnel_completes_verified_tls_and_returns_an_update_candidate；407由既有probe_manual_proxy及updater錯誤mapping測試；同Proxy無直連fallback |
| S2 | 無可得HTTP碼的TLS失敗 | PASS | apps/desktop/core/src/network.rs#connect_tunnel_success_does_not_disable_tls_or_hide_disconnects；壞TLS/斷線皆error且httpStatus=None；finish保operationId |
| S3 | 開發環境無公司安裝條件 | manual | company-proxy.md延期記錄/tasks12.3未勾/maincommit與installer待填；主流程人工核讀development雙語順序 |
| S4 | 打包後進入公司測試 | manual | 尚未打包後在公司實機驗收；明確延期，不列缺實作/CRITICAL，不宣稱通過 |

## node-sdk

`openspec/changes/desktop-workflow-and-offline-security-review/specs/node-sdk/spec.md`：1 Requirements／6 Scenarios／0 Examples。

| 編號 | Requirement | 狀態 | 實作定位 |
| --- | --- | --- | --- |
| R1 | 渲染 API | partial | `crates/adapters/speclink-node/src/render.rs#resolve_matrix/RenderOptions；index.js#skills（外部出口）；index.d.ts#RenderOptions（主流程TS失敗證據）` |

| 編號 | Scenario | 狀態 | 測試／證據與限制 |
| --- | --- | --- | --- |
| S1 | 中性 tool-call 渲染 | partial | Node#neutral × tool-call speaks tool-call wording without local prefixes/paths；測試沒有spec的store:remote，RenderOptions無store；instructions.render亦缺；patch外具體RenderOptions/resolve_matrix佐證 |
| S2 | 與 CLI 生成一致 | PASS | Node#claude × cli matches the SKILL.md the CLI init generates；apply逐字比較 |
| S3 | copilot target 與 CLI 生成一致 | PASS | Node#copilot × cli matches the SKILL.md the CLI init generates under .github/skills；/非$ |
| S4 | 未知 target 被拒 | PASS | Node#unknown target fails loud, listing every valid target；github-copilot與五合法值 |
| S5 | Copilot 原生渲染與 CLI 同源 | PASS | Node#copilot renders native syntax for every skill and matches CLI output；各registry/unknown skill與neutral語意 |
| S6 | Antigravity 原生渲染與 CLI 同源 | partial | Node#antigravity renders the same skills as init under its independent .agent directory；執行期測試通過但公開index.d.ts漏antigravity；主流程tsc最小引用TS2322 exit2，TS呼叫者不可用 |

## propose-skill

`openspec/changes/desktop-workflow-and-offline-security-review/specs/propose-skill/spec.md`：1 Requirements／1 Scenarios／0 Examples。

| 編號 | Requirement | 狀態 | 實作定位 |
| --- | --- | --- | --- |
| R1 | 提案內容預設以條列起草 | PASS | `crates/engine/speclink-core/assets/schema/spec-driven/proposal.template.md；assets/skills/propose.md；src/workspace/skills.rs#registry` |

| 編號 | Scenario | 狀態 | 測試／證據與限制 |
| --- | --- | --- | --- |
| S1 | 新專案採用相同條列預設 | PASS | Golden#proposal_template_defaults_to_bullets_for_every_project；static模板/registry/Tool.ALL一致；不宣稱實際AI起草品質 |

## review-skill

`openspec/changes/desktop-workflow-and-offline-security-review/specs/review-skill/spec.md`：1 Requirements／2 Scenarios／0 Examples。

| 編號 | Requirement | 狀態 | 實作定位 |
| --- | --- | --- | --- |
| R1 | 審查完成紀錄的技能交棒 | PASS | `crates/engine/speclink-core/assets/skills/review.md#Completed-review-history` |

| 編號 | Scenario | 狀態 | 測試／證據與限制 |
| --- | --- | --- | --- |
| S1 | 蓋章後提供可查詢紀錄 | PASS | Golden#stations_and_archive_preserve_history_and_defer_security_acceptance_to_host；static原稿review history/checks/review/recordId |
| S2 | 保存失敗 | PASS | Golden#stations_and_archive_preserve_history_and_defer_security_acceptance_to_host；static history_recovery_required；真保存失敗由Checks failpoint驗證 |

## review-station

`openspec/changes/desktop-workflow-and-offline-security-review/specs/review-station/spec.md`：3 Requirements／10 Scenarios／1 Examples。

| 編號 | Requirement | 狀態 | 實作定位 |
| --- | --- | --- | --- |
| R1 | 蓋章守門與蓋章效果 | PASS | `crates/engine/speclink-core/src/quality/station.rs#stamp_gate/write_stamp；history.rs；crates/engine/speclink-fs/src/checks.rs` |
| R2 | 檢查紀錄鎖與封存搬移 | PASS | `crates/engine/speclink-fs/src/checks.rs#lock/with_write；archive_checks.rs#archive` |
| R3 | 審查完成紀錄查詢 | PASS | `crates/host/speclink-host/src/history.rs#query；crates/adapters/speclink-cli/src/verbs/station.rs#print_history` |

| 編號 | Scenario | 狀態 | 測試／證據與限制 |
| --- | --- | --- | --- |
| S1 | 寫碼任務未全完成即拒絕 | PASS | Station#review::stamp_refuses_when_tasks_incomplete；4/5且meta/ticket不變 |
| S2 | 僅餘手動任務可蓋章 | PASS | Station#review::stamp_lands_when_only_manual_tasks_remain_and_anchors_the_full_total；4code+1M，Round2 empty，total5 |
| S3 | 末輪有未解 findings 且未帶 --accept | PASS | Station#review::stamp_refuses_unresolved_findings_without_accept；CRITICAL/WARNING拒 |
| S4 | 僅 SUGGESTION 的末輪乾淨蓋章 | PASS | Station#review::stamp_allows_a_suggestion_only_last_round；新immutable history test保SUGGESTION原文 |
| S5 | 帶保留蓋章 | PASS | Station#review::stamp_with_accept_overrides_findings_and_stamps；另accepted歷史不覆寫 |
| S6 | 乾淨蓋章 | PASS | Station#review::stamp_clean_round_writes_five_fields_and_deletes_ticket；5任務/五欄/刪工單 |
| S7 | 歷史寫入失敗保全工單 | PASS | Checks#every_check_publish_failure_restores_the_only_ticket_and_metadata；publish/delete/index/meta全failpoint；interrupted_recovery拒讀 |
| S8 | Windows 封存與競爭寫入 | PASS | Checks#archive_move_keeps_exclusion_without_recreating_the_active_directory；legacy writer/link lock拒寫；Windows由已有CI證據 |
| S9 | 查詢已蓋章紀錄的人眼與JSON | PASS | CLIrev#review_history_lists_completed_rounds_and_reads_active_or_archived_content；list/record/human/no-color/archive；server reader查歷史 |
| S10 | 缺歷史與非法ID | PASS | CLIrev#review_history_lists_completed_rounds_and_reads_active_or_archived_content；empty/unknown合法ID；穿越由共用validate_id及CLIver ../測試；reader401沿共用HTTP邊界 |

## security-review

`openspec/changes/desktop-workflow-and-offline-security-review/specs/security-review/spec.md`：5 Requirements／9 Scenarios／1 Examples。

| 編號 | Requirement | 狀態 | 實作定位 |
| --- | --- | --- | --- |
| R1 | 保留基本掃描並提供深度安全審查技能 | PASS | `crates/engine/speclink-core/assets/skills/security-review.md；src/workspace/skills.rs#registry` |
| R2 | 五種語言的架構與資料流深度審查 | PASS | `crates/engine/speclink-core/assets/skills/security-review.md#Trace-the-selected-families-and-languages` |
| R3 | 深度審查提交與查詢命令 | PASS | `crates/adapters/speclink-cli/src/verbs/security.rs#review_command/remote_review_command；Hostdeep#add_deep_round；crates/host/speclink-host/src/security/remote.rs#execute` |
| R4 | 安全驗收的等級完整性與新鮮度判定 | PASS | `Accept#evaluate；Hostdeep#acceptance_proof_for_target；host/security/remote.rs#proof` |
| R5 | 安全風險明確接受命令 | PASS | `Hostdeep#accept_current_risk；crates/engine/speclink-core/src/security/report/deep.rs#prepare_decision` |

| 編號 | Scenario | 狀態 | 測試／證據與限制 |
| --- | --- | --- | --- |
| S1 | 只選2025且保留基本掃描 | PASS | CLIsec#security_top10_2025_only_installed_then_offline_without_ai_or_archive_gate；HTTP trap零請求，只2025 KB；static原稿標準限制 |
| S2 | 停用或缺件 | PASS | Hostsec#security_run_disabled_reads_only_policy_and_basic_rules_remain_independent；disabled spy零scan；KB missing；skill stop |
| S3 | 跨帳戶ID的授權追查 | PASS | Report#deep_report_v2_masks_every_representation_and_has_all_nine_finding_groups；static skill accountId/service/repository資料流與擁有權；未呼叫實際AI模型 |
| S4 | 無法完成的路徑 | PASS | Golden#security_review_skill_preserves_basic_scan_and_selected_offline_scope；static外部gateway/環境保NOT_VERIFIED；Report incomplete_target_evidence |
| S5 | 提交及查詢人眼與JSON | PASS | CLIsec#security_top10_2025_only_installed_then_offline_without_ai_or_archive_gate；add/show/report人眼JSON/大小/型別/noANSI |
| S6 | 越界引用與Remote衝突 | PASS | Report#deep_report_rejects_missing_groups_unknown_ids_outside_evidence_and_401_chars；越界path/line/check；Host permitted_references；Remote CAS無部分效果 |
| S7 | 風險等級與必要資料矩陣 | PASS | Accept#acceptance_matrix_from_spec；required_incomplete_zero_findings_and_missing_data_still_blocks；resolves逐項與取消不可補齊 |
| S8 | 程式碼或政策變更 | PASS | Hostdeep#current_checkout_detects_added_deleted_and_changed_sources_and_knowledge；新增刪除改碼/KB與scope接受；digest忽略required/retention |
| S9 | 有理由的接受與空理由 | PASS | CLIsec#security_top10_2025_only_installed_then_offline_without_ai_or_archive_gate；身份/空或2001理由/16KiB/舊ID/真接受不改原報告；Remote同revision第二次拒 |

## security-review-report

`openspec/changes/desktop-workflow-and-offline-security-review/specs/security-review-report/spec.md`：5 Requirements／10 Scenarios／0 Examples。

| 編號 | Requirement | 狀態 | 實作定位 |
| --- | --- | --- | --- |
| R1 | 本機報告保存與查詢 | PASS | `crates/engine/speclink-core/src/security/report/deep.rs#save_change_report/read_change_report；FSreport；Hostsec#ProposalReports` |
| R2 | 版本化 JSON 報告模型 | PASS | `crates/engine/speclink-core/src/security/report/deep.rs#DeepWire/DeepReport；Report#Report` |
| R3 | SDD 提示與 AI 邊界 | PASS | `crates/engine/speclink-core/assets/skills/security-review.md；propose.md；verify.md` |
| R4 | 報告輸出命令與語言 | PASS | `crates/adapters/speclink-cli/src/verbs/security.rs；core/security/report/deep.rs#VersionedReport；Report#markdown` |
| R5 | 深度報告九項內容與風險接受紀錄 | PASS | `crates/engine/speclink-core/src/security/report/deep.rs#DeepFinding/sanitize/markdown/prepare_decision；Hostdeep#read_decisions/review_view` |

| 編號 | Scenario | 狀態 | 測試／證據與限制 |
| --- | --- | --- | --- |
| S1 | 保存中斷與錯誤 ID | PASS | Checks#every_check_publish_failure_restores_the_only_ticket_and_metadata；proposal同commit_checks；FSreport#security_report_atomic_pair_index_failure_quota_and_invalid_identity保舊latest；非法ULID |
| S2 | 封存後報告仍可查 | PASS | FSreport#proposal_security_reports_keep_v1_and_redacted_v2_in_one_index；archive後read_change_report；CLI datedarchive讀ID |
| S3 | 明確目標的提案內報告 | PASS | FSreport#proposal_security_reports_keep_v1_and_redacted_v2_in_one_index；v1/v2同checksindex；Remote explicit paired publication後查server |
| S4 | 有 dirty 檔與無 Git | partial | Report#security_report_serializes_one_redacted_model_and_preserves_status_counts只assert commitSha=null與64位digest；Hostdeep無Gitfixture測來源改動，缺真Gitcommit後dirty檔驗HEAD參考及當次bytes SHA案例 |
| S5 | 沒有 AI 的離線旅程 | PASS | CLIsec#security_top10_2025_only_installed_then_offline_without_ai_or_archive_gate；內建/HTTPtrap/無AI/reference不擋archive |
| S6 | 外部資料帶指令 | PASS | Golden#security_review_skill_preserves_basic_scan_and_selected_offline_scope；static原稿外部標準/程式/註解當data、禁scripts；不宣稱實際AI prompt-injection測試 |
| S7 | 兩種格式一致 | PASS | Report#deep_report_v2_masks_every_representation_and_has_all_nine_finding_groups；JSON→model→markdown；CLI兩格式無sentinel |
| S8 | 介面與 Agent 語言 | PASS | Report#security_report_localizes_owned_rule_explanations_and_markdown_sections；disabled_cancelled_and_language_are_explicit涵tw/ja/en；deep固定tw；Desktop兩語UI |
| S9 | 九項證據與機密遮罩 | PASS | Report#deep_report_v2_masks_every_representation_and_has_all_nine_finding_groups；九項/domain/evidence/400邊界/省略秘密；FS持久化sentinel |
| S10 | 接受理由可回查且不改原報告 | PASS | Report#security_risk_decision_validates_reason_identity_and_cas_without_rewriting_report；原JSON不改；CLI human/show每欄可查 |

## security-standard-data

`openspec/changes/desktop-workflow-and-offline-security-review/specs/security-standard-data/spec.md`：3 Requirements／9 Scenarios／0 Examples。

| 編號 | Requirement | 狀態 | 實作定位 |
| --- | --- | --- | --- |
| R1 | 應用程式資料目錄與離線版本清單 | PASS | `crates/host/speclink-host/src/security_paths.rs；FS#FsSecurityData/with_standard_root/pin/begin_install/remove；host/security_data.rs` |
| R2 | 深度審查完整離線資料包 | PASS | `crates/engine/speclink-core/assets/security/spec_registry.json；src/security/knowledge.rs；Data；Package` |
| R3 | 本機知識庫準備命令 | PASS | `KB#prepare/validate_existing；crates/adapters/speclink-cli/src/verbs/security.rs` |

| 編號 | Scenario | 狀態 | 測試／證據與限制 |
| --- | --- | --- | --- |
| S1 | Desktop 安裝後 CLI 離線可用 | PASS | CLIsec#security_top10_2025_only_installed_then_offline_without_ai_or_archive_gate；同Host資料根解析；security_data_list_is_mode_free_offline_json_and_no_color；HTTPtrap |
| S2 | 缺裝置路徑與缺資料 | PASS | Paths#standard_path_preferences_are_device_only_atomic_and_fail_closed；絕對路徑/壞型別拒；KB missing不下載；data_directory_unavailable沿context邊界 |
| S3 | 兩台電腦共用同一資料 | PASS | FS#shared_standard_root_keeps_private_data_local_and_refuses_shared_deletion；兩device root共用本機fixture root同SHA/private references留device；不是真NAS |
| S4 | 共用寫入競爭與失敗保舊 | PASS | FS#shared_standard_root_keeps_private_data_local_and_refuses_shared_deletion；pin阻writer；既有generation發布failpoint保舊；未實測兩host NAS鎖能力 |
| S5 | 共用位置不可用或唯讀 | PASS | SecData#離線路徑停用資料操作；CLI customroot斷線exit1不fallback；UI mock唯讀可匯出禁修改；FS customroot拒刪 |
| S6 | 下載後斷網與交換完整包 | PASS | Package#complete_guidance_round_trip_preserves_all_pinned_files_and_rejects_tampering；encode/decode同bytes/SHA；CLI2025only/零HTTP；imported沿既有 |
| S7 | 浮動來源與舊包缺件 | PASS | Data#deep_bundle_requires_pinned_guidance_and_licenses_while_legacy_is_missing；master/LTS/noLicense/noSHA/unknown/tamper；Package#old_bundle_keeps_basic_data_but_missing_guidance_cannot_be_deep_ready |
| S8 | 本機準備及人眼JSON | PASS | CLIsec#security_top10_2025_only_installed_then_offline_without_ai_or_archive_gate；prepare人眼JSON/型別/manifest只選2025；missing命令不建snapshot |
| S9 | 同名目錄或必要檔被改 | PASS | KB#knowledge_prepare_is_local_complete_and_preserves_foreign_or_missing_data；user/另一project/改SHA/取消保舊；knowledge_prepare_rejects_symlink_destination_and_gitignore |

## ui-copy-vocabulary

`openspec/changes/desktop-workflow-and-offline-security-review/specs/ui-copy-vocabulary/spec.md`：1 Requirements／2 Scenarios／0 Examples。

| 編號 | Requirement | 狀態 | 實作定位 |
| --- | --- | --- | --- |
| R1 | 安全驗收與檢查紀錄的用詞 | PASS | `openspec/LANGUAGE.md；packages/ui/src/i18n.tsx；apps/desktop/src/i18n/messages.ts` |

| 編號 | Scenario | 狀態 | 測試／證據與限制 |
| --- | --- | --- | --- |
| S1 | 參考與驗收模式 | manual | 主流程人工核讀LANGUAGE281-310/messages中英/WorkflowPage；i18n54/598 PASS不等沒有漏洞/認證 |
| S2 | 關閉封存完整紀錄 | manual | 主流程人工核讀messages479/1163與checkRecords.help：省略完整報告仍留最小摘要；WorkflowPage同詞 |

## verify-skill

`openspec/changes/desktop-workflow-and-offline-security-review/specs/verify-skill/spec.md`：2 Requirements／5 Scenarios／0 Examples。

| 編號 | Requirement | 狀態 | 實作定位 |
| --- | --- | --- | --- |
| R1 | Verify 報告中的獨立初步安全資訊 | PASS | `crates/engine/speclink-core/assets/skills/verify.md#Preliminary-security-review-boundary` |
| R2 | 驗證完成紀錄的技能交棒 | PASS | `crates/engine/speclink-core/assets/skills/verify.md#Completed-verification-history` |

| 編號 | Scenario | 狀態 | 測試／證據與限制 |
| --- | --- | --- | --- |
| S1 | 初步疑似風險不改驗證工單 | PASS | Golden#stations_and_archive_preserve_history_and_defer_security_acceptance_to_host；static安全資訊不進verify票/新章/reference不擋 |
| S2 | 缺資料或停用政策 | PASS | Golden#stations_and_archive_preserve_history_and_defer_security_acceptance_to_host；static缺件NOT_VERIFIED/disabled不掃碼；KB missing與disabled spy實作測試 |
| S3 | 既有守門仍有效 | PASS | CLIarc#既有open-ticket/stale章守門案例；static verify原稿明示保留原有守門；未讓新安全結果繞過 |
| S4 | 必要安全條件未滿 | PASS | CLIver#security_required_verify_cannot_be_bypassed_by_ordinary_accept；static required缺件/stale/HIGH不得自行接受；Accept真矩陣 |
| S5 | 成功與保存失敗的回報 | PASS | Golden#stations_and_archive_preserve_history_and_defer_security_acceptance_to_host；static checks/verify recordId/history_recovery_required；Checks保存故障 |

## verify-station

`openspec/changes/desktop-workflow-and-offline-security-review/specs/verify-station/spec.md`：2 Requirements／9 Scenarios／1 Examples。

| 編號 | Requirement | 狀態 | 實作定位 |
| --- | --- | --- | --- |
| R1 | 驗證蓋章守門與蓋章效果 | PASS | `crates/engine/speclink-core/src/quality/station.rs#stamp_with_acceptance/write_stamp；src/security/mod.rs#require_acceptance` |
| R2 | 驗證完成紀錄查詢 | PASS | `crates/host/speclink-host/src/history.rs#query；crates/adapters/speclink-cli/src/verbs/station.rs#print_history` |

| 編號 | Scenario | 狀態 | 測試／證據與限制 |
| --- | --- | --- | --- |
| S1 | 寫碼任務未完成即拒絕蓋章 | PASS | Station#verify::stamp_refuses_when_tasks_regress_after_the_ticket_was_opened；4/5拒/meta/ticket不變 |
| S2 | 僅餘手動任務可蓋章 | PASS | Station#verify::stamp_lands_when_only_manual_tasks_remain_and_anchors_the_full_total；另Example8total精確測試 |
| S3 | 末輪有未解 findings 且未帶 --accept | PASS | Station#verify::stamp_refuses_unresolved_findings_without_accept；CRITICAL/WARNING守門 |
| S4 | 僅 SUGGESTION 的末輪乾淨蓋章 | PASS | Station#verify::stamp_allows_a_suggestion_only_last_round；SUGGESTION不擋且完整history新共用測試 |
| S5 | 乾淨蓋章 | PASS | Station#verify::stamp_clean_round_writes_five_fields_and_deletes_the_ticket；metadata五欄/ticket刪/immutablehistory |
| S6 | 必修驗收不能用一般接受略過 | PASS | CLIver#security_required_verify_cannot_be_bypassed_by_ordinary_accept；Remote缺attestation拒絕；原proof/index TOCTOU由主流程Round2另驗 |
| S7 | 歷史寫入失敗保全工單 | PASS | Checks#every_check_publish_failure_restores_the_only_ticket_and_metadata；每個publish/index/meta/delete與recovery未完拒讀 |
| S8 | 查詢已蓋章紀錄的人眼與JSON | PASS | CLIver#verify_history_keeps_accepted_ticket_and_reads_archive；JSON/human/noANSI/accepted完整內容/datedarchive |
| S9 | 缺歷史與非法ID | PASS | CLIver#verify_history_keeps_accepted_ticket_and_reads_archive；missing/../拒；empty與review共用index/query |

## workflow-config

`openspec/changes/desktop-workflow-and-offline-security-review/specs/workflow-config/spec.md`：3 Requirements／8 Scenarios／0 Examples。

| 編號 | Requirement | 狀態 | 實作定位 |
| --- | --- | --- | --- |
| R1 | 安全政策區塊預設與獨立選項 | PASS | `crates/engine/speclink-core/src/security/policy.rs#SecurityPolicy；Config#WorkflowConfig` |
| R2 | 安全政策部分更新與保值 | PASS | `crates/engine/speclink-core/src/security/policy.rs#patch_workflow_security；crates/adapters/speclink-cli/src/verbs/config.rs` |
| R3 | 檢查紀錄封存政策 | PASS | `crates/engine/speclink-core/src/workspace/config.rs#CheckRecordsConfig/ArchiveRecords；security/policy.rs#patch_workflow_check_records` |

| 編號 | Scenario | 狀態 | 測試／證據與限制 |
| --- | --- | --- | --- |
| S1 | 各標準互不影響 | PASS | Config#security_policy_versions_and_all_four_standards_are_independent；patch_preserves_unknown_fields_comments_and_other_policies明驗ASVSL2保留/Top2025不變 |
| S2 | 非法安全政策 | PASS | Config#security_policy_rejects_invalid_known_fields_even_when_disabled；字串enabled/scopedesktop/L4/缺版本/enum拒 |
| S3 | 舊政策與停用後保值 | PASS | Config#security_acceptance_defaults_and_retains_disabled_selection；old requiredfalse/disabled保true；absent/null原預設 |
| S4 | 修改一個葉節點保留未知內容 | PASS | Config#security_policy_patch_preserves_unknown_fields_comments_and_other_policies；future_option/註解/tdd/audit/worktree原樣 |
| S5 | dry-run 與失敗原檔不變 | PASS | Config#security_policy_patch_rejects_invalid_unknown_and_oversized_input；65536/65537/unknown/alias/merge；CLI dryrun零寫 |
| S6 | Remote 並行修改 | PASS | Settings#CAS衝突保留驗收與保留草稿，確認後只套到最新快照；Remote expectedrevision沿workflow put；reader/offline/401共用adapter |
| S7 | 只修改安全紀錄的封存選項 | PASS | CLIsec#security_acceptance_and_record_policy_cli_contract；onlysecurityfalse/show其餘true/原文保留 |
| S8 | dry-run與非法值 | PASS | Config#check_records_defaults_patch_preserves_other_bytes_and_rejects_bad_input；CLI dryrun/no-color/stdout/stderr/unknown/字串false拒 |

## workspace-tools

`openspec/changes/desktop-workflow-and-offline-security-review/specs/workspace-tools/spec.md`：3 Requirements／20 Scenarios／2 Examples。

| 編號 | Requirement | 狀態 | 實作定位 |
| --- | --- | --- | --- |
| R1 | tools 自訂描述子的接受與驗證 | PASS | `crates/engine/speclink-core/src/workspace/config.rs#ToolDescriptor.validate；src/workspace/skills.rs#Tool.ALL` |
| R2 | init 內建 Agent 工具選擇 | PASS | `crates/adapters/speclink-cli/src/verbs/init.rs#resolve_init_tools；crates/engine/speclink-core/src/workspace/init.rs` |
| R3 | Antigravity 技能生成與獨立清理 | PASS | `crates/engine/speclink-core/src/workspace/skills.rs#Tool/registry；src/workspace/init.rs#reconcile_builtin_tools` |

| 編號 | Scenario | 狀態 | 測試／證據與限制 |
| --- | --- | --- | --- |
| S1 | 合法描述子生成對應工具檔 | PASS | Init#managed_skills_covers_the_registry_per_target；wad-harness/.wad/skills與render逐字；原update target整合無instructions |
| S2 | 名稱與內建工具衝突被拒 | PASS | Config#descriptor_validation_rejects_builtin_name_conflict；claude拒name/單行 |
| S3 | 路徑逸出專案根被拒 | PASS | Config#descriptor_validation_rejects_paths_escaping_project_root；../outside/skills拒 |
| S4 | 殘留 instructions_file 欄位得棄用提示 | PASS | Init#update_strips_a_descriptors_legacy_marker；instructions_file棄用；原workspace notes的具體消費端stderr非error；不影響生成 |
| S5 | 結尾分隔符於驗證時削去 | PASS | Init#probe_normalizes_a_descriptor_skills_dir_with_a_trailing_slash；config邊界削尾/probe無// |
| S6 | 削去後等同專案根的 skills_dir 被拒 | PASS | Config#descriptor_validation_rejects_a_skills_dir_that_normalizes_to_nothing；/、./、.、.wad/..；ITinit#update_rejects_a_descriptor_skills_dir_at_the_root_or_a_builtin_directory零寫 |
| S7 | 與內建工具 skills 目錄相撞被拒 | PASS | Config#descriptor_validation_rejects_a_builtin_skills_dir；claude等價拼法；CLIupdate零寫整合 |
| S8 | 名為 copilot 或指向 Copilot 目錄的既有描述子升級後被拒 | PASS | Config#descriptor_validation_reserves_the_copilot_name；rejects_the_copilot_skills_dir；copilot_descriptor_is_reserved_with_manual_migration_guidance；CLIupdateguard |
| S9 | filesystem init 顯式選擇 Codex | PASS | ITinit#fs_init_explicit_tools_generate_exactly_the_selection；codex純選集/skills/無AGENTS |
| S10 | Remote init 顯式選擇兩個工具 | PASS | ITinit#remote_init_explicit_both_tools_writes_section_without_a_spec_tree；claude,codex URL/repo/無openspec與instructions |
| S11 | 互動終端選擇 Claude 與 Codex | PASS | CLIinit#interactive_yes_yes_selects_both；四問順序與stderr |
| S12 | 互動終端不得提交空選集 | PASS | CLIinit#interactive_all_no_reasks_until_a_tool_is_picked；四題全否重問不進init |
| S13 | 非互動 init 缺少 tools 零寫入失敗 | PASS | ITinit#fs_init_without_tools_on_a_pipe_fails_with_zero_writes；remote_init_without_tools_on_a_pipe_fails_with_zero_writes；不讀pipe答案 |
| S14 | 空或未知的顯式 tools 被拒 | PASS | CLIinit#explicit_empty_selection_is_rejected_naming_the_flag_and_values；unknown claude,vscode與CLI零寫；單值vscode Example另partial |
| S15 | no-color 不改變工具選擇語意 | PASS | ITinit#no_color_init_emits_no_ansi_and_the_same_file_effects；互動prompt固定無ANSI文字；未真PTY E2E |
| S16 | filesystem init 顯式選擇 Copilot | PASS | ITinit#copilot_init_and_pipe_hint_preserve_cli_contracts；純copilot/.github/無其他instructions |
| S17 | 顯式選擇三個工具 | PASS | ITinit#fs_init_explicit_tools_generate_exactly_the_selection；claude,codex,copilot三組skills與純選集 |
| S18 | 同時使用 Codex 與 Antigravity | PASS | Init#antigravity_coexists_with_codex_and_deselects_only_owned_skills；兩目錄$與/、非Claude frontmatter、無AGENTS/GEMINI |
| S19 | 取消 Antigravity 保留使用者檔案 | PASS | Init#antigravity_coexists_with_codex_and_deselects_only_owned_skills；自有skill/rules/Codex保留；update_never_prunes_through_a_symlinked_antigravity_skills_dir |
| S20 | 舊自訂描述子與新內建工具衝突 | PASS | Init#antigravity_descriptor_conflicts_fail_without_overwriting_existing_skills；name/skills_dir兩支/Codex不變；config遷移hint |

## 七個 Example 的精確輸入追溯（主流程補存）

| 規格與 Example | 結果 | 具體測試與輸入 |
| --- | --- | --- |
| desktop-app：章節對照與畫法 | PASS | Proposal 的「章節對照與畫法」以 it.each 逐列使用 Summary、Why、What Changes、Proposed Solution、Non-Goals、Alternatives Considered、Capabilities、Impact、Context、Decisions、Implementation Contract、Risks / Trade-offs，驗中文標籤與畫法；既有 patch 外測試。 |
| desktop-app：影響段不符形狀時當一般列 | PASS | Proposal 的「影響段只有散文列點時不出『影響的檔案』格」使用規格原字串「影響看板與詳情頁」「無 CLI 變動」；另有同形渲染測試；既有 patch 外測試。 |
| review-station：蓋章寫入的任務錨 | PASS | Station 的 review `stamp_lands_when_only_manual_tasks_remain_and_anchors_the_full_total`：5 個任務、4 code、1 manual，錨為 5；既有 patch 外測試。 |
| verify-station：蓋章寫入的任務錨 | PASS | Station 的 verify `stamp_writes_the_task_anchor_from_the_ticket_moment`：8 個任務、7 code、1 manual，錨為 8；既有 patch 外測試。 |
| security-review：納入驗收的具體矩陣 | PASS | `security/acceptance.rs::acceptance_matrix_from_spec` 覆蓋 reference、LOW、MEDIUM、HIGH、已接受 CRITICAL；`required_incomplete_zero_findings_and_missing_data_still_blocks` 覆蓋零 finding 缺件；`acceptance_digests_missing_evidence_and_explicit_resolutions` 覆蓋 HIGH 接受的舊 scopeDigest。 |
| workspace-tools：skills_dir 的驗證結果 | PASS | Config 的 descriptor_validation 系列分別用 .wad/skills、.wad/skills/、../outside/skills、/、.wad/..、.claude/skills、./.claude/skills、.claude/skills-extra、.github/skills、./.github/skills、.github/skills-extra、.GitHub/skills 覆蓋全部 12 列；既有 patch 外測試。 |
| workspace-tools：--tools 值的解析 | partial | CLIinit 的 explicit_tools_accept_copilot_in_any_case_and_combination 使用 copilot、Copilot、claude,copilot、copilot,copilot；explicit_unknown_tool_lists_the_supported_set 使用 github-copilot。單值 vscode 未找到同輸入測試（目前是 claude,vscode），須補精確列；不代表解析實作已證實錯誤。 |

Examples：6 PASS、1 partial；沒有將缺少完全相同輸入的測試列計為已覆蓋。

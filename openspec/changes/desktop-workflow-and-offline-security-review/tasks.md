## 1. 安全與紀錄政策（先建立後續共用契約）

- [x] 1.1 RED：在 crates/engine/speclink-core/src/security/policy.rs 與 crates/engine/speclink-core/src/workspace/config/tests.rs 補「安全政策區塊預設與獨立選項」「安全政策部分更新與保值」「檢查紀錄封存政策」失敗案例：舊設定required=false、封存三項true、錯型別拒絕、未知鍵／註解保值、停用後保值；以 cargo test -p speclink-core --lib 確認新案例因缺功能失敗。 <!-- speclink-task:tsk_01M5CJM0GA79NNK3WQ4CAXJJFB -->
- [x] 1.2 GREEN：在 crates/engine/speclink-core/src/security/policy.rs 與 crates/engine/speclink-core/src/workspace/config.rs 加入required_for_acceptance及check_records.archive三布林值，採既有文字手術與yaml_scalar，只改指定葉節點；以 cargo test -p speclink-core --lib 通過1.1，並檢查空mapping、65,537bytes及字串布林不寫入。 <!-- speclink-task:tsk_01M5CJM0GACF0KJPQYTRPSKZBC -->
- [x] 1.3 在 crates/adapters/speclink-cli/src 與 crates/host/speclink-host/src/policy.rs 接workflow-config check-records --stdin／--dry-run及既有security的新葉節點；先在 crates/adapters/speclink-cli/tests/it/security.rs 加CLI失敗測試再實作，驗證exit0/1/2、stdout摘要／diff、stderr、--no-color及show JSON新增欄位型別；執行 cargo test -p speclink-cli --test it security。 <!-- speclink-task:tsk_01M5CJM0GA63QJXN39NRYX6KNS -->

## 2. 專案排序

- [x] 2.1 RED：在 apps/desktop/src/__tests__/tabs.test.ts 與 apps/desktop/src/__tests__/projectRail.test.tsx 補「左側專案排序與裝置持久化」案例：C/A/B重排、Remote離線、Escape／無效位置、首末項、儲存失敗、activeKey不變及重開還原；以 npm test -w @speclink/desktop -- tabs projectRail 確認新行為失敗。 <!-- speclink-task:tsk_01M5CJM0GAN3BNR3516DT04SGT -->
- [x] 2.2 GREEN／REFACTOR：在 apps/desktop/src/tabs.ts、apps/desktop/src/store.ts、apps/desktop/src/components/ProjectRail.tsx 接單一排序動作、既有dnd-kit與上移／下移選單，Tray及切換快捷鍵共用tabs；以2.1同命令轉綠，核對拖曳不誤觸切換、不修改專案文件且失敗還原。 <!-- speclink-task:tsk_01M5CJM0GAY70H2XTJQZN3S8KJ -->

## 3. 更新頁與 Proxy 診斷

- [x] 3.1 RED：在 apps/desktop/src/__tests__/updatesPage.test.tsx 與 apps/desktop/src/__tests__/updater.test.ts 補「更新頁內附資訊不依賴網路成功」：無updater仍有內附日誌、背景失敗可查、未就緒動作停用、zh-TW/en；以 npm test -w @speclink/desktop -- updatesPage updater 確認失敗原因符合目前空白／idle缺口。 <!-- speclink-task:tsk_01M5CJM0GAKTYBP5PN699SQTJX -->
- [x] 3.2 GREEN：在 apps/desktop/src/views/settings/UpdatesPage.tsx、apps/desktop/src/core/updater.ts、apps/desktop/src/store.ts 分開內附內容、updater動作與lastCheck，保留背景靜默及原狀態機，失敗顯示重試／診斷入口；以3.1同命令轉綠，更新提示與下載同意行為不回歸。 <!-- speclink-task:tsk_01M5CJM0GARVKQM3TBEGM77DNC -->
- [x] 3.3 RED→GREEN：在 crates/host/speclink-host/src/network.rs、apps/desktop/core/src、apps/desktop/src-tauri/src/updater_commands.rs 補可重現CONNECT成功／407／TLS／斷線／壞回應測試，再修「公司 Proxy 更新失敗的定位與呈現」實測缺口；共用Proxy快照且不放寬TLS／來源／簽章，無狀態碼留null；執行 cargo test -p speclink-host 及 cargo test -p speclink-desktop-core，Tauri殼只留委派。 <!-- speclink-task:tsk_01M5CJM0GAWAM2NZ2XN4JA9FAC -->
- [x] 3.4 排查同類位置：以rg搜尋if (!updater)、checkFailed及update_failed，涵蓋 apps/desktop/src/views/settings/UpdatesPage.tsx、apps/desktop/src/components/UpdateInstallRow.tsx、apps/desktop/src/views/AppSettingsView.tsx、apps/desktop/src/store.ts、apps/desktop/src-tauri/src/updater_commands.rs 及診斷測試入口；逐命中在變更證據中註明共修或不修理由，確認內附內容不再被網路失敗隱藏；以 npm test -w @speclink/desktop -- updatesPage updateInstallRow updater 驗證，未知公司根因不標已解。 <!-- speclink-task:tsk_01M5CJM0GADYNS2PGQ67WZJ4CA -->

## 4. 十二條問號說明

- [x] 4.1 RED：在 packages/ui/src/__tests__ 與 apps/desktop/src/__tests__ 加「預設十二條的問號說明」案例，逐名對照引擎十二項，驗證zh-TW/en、未知名稱、Enter／Space／Escape與焦點返回；以 npm test -w @speclink/ui 及 npm test -w @speclink/desktop -- review 確認新案例失敗。 <!-- speclink-task:tsk_01M5CJM0GAFVVH2TQ1JTVX8D76 -->
- [x] 4.2 GREEN：在 packages/ui/src/components/BaselineSmellList.tsx、packages/ui/src/i18n.tsx 與 apps/desktop/src/views/project-settings/ReviewRulesPage.tsx 共用Popover／Button呈現用途、例子及extend／replace效果，工單套用規則列沿同元件；以4.1同命令轉綠，不在apps複製名稱或覆蓋UI變體樣式。 <!-- speclink-task:tsk_01M5CJM0GACQC1MMJW4AR7WRMR -->

## 5. 離線知識資料

- [x] 5.1 RED：在 crates/engine/speclink-core/src/security 與 crates/engine/speclink-fs/src/security_packages.rs 補「深度審查完整離線資料包」案例：固定版本、缺指南、假LTS、未知檔、授權缺件、匯出往返及舊包仍可基本掃描；執行 cargo test -p speclink-core --lib 及 cargo test -p speclink-fs，確認深度ready條件尚未實作而失敗。 <!-- speclink-task:tsk_01M5CJM0GAN0F5S2Q6HDKYEE11 -->
- [x] 5.2 核對並建立 crates/engine/speclink-core/assets/security/spec_registry.json，引用既有sources.json，逐項核對官方真實檔名、固定commit／版本、sha256及授權；涵蓋認證、授權、機密、日誌、TLS、REST、注入、反序列化、XXE、SSRF、原型鏈、Webhook、DoS與實際語言指南，不假造Python/Rust檔名；以registry fixture驗證浮動來源或缺檔不能ready，記錄實際採用及未採用理由，不把下載紀錄寫成官方認證。 <!-- speclink-task:tsk_01M5CJM0GATGB05F0ZAQTAC1F9 -->
- [x] 5.3 GREEN：在 crates/host/speclink-host/src/security_data.rs、crates/engine/speclink-fs/src/security.rs、crates/engine/speclink-fs/src/security_packages.rs 接完整generation安裝、離線匯入／匯出與knowledgeStatus，維持ZIP v1及集合v2上限和失敗保舊；以5.1同命令及 cargo test -p speclink-host 驗證整包斷網交換，舊基本掃描回歸不變。 <!-- speclink-task:tsk_01M5CJM0GAEETWBC3W27KR1BNQ -->
- [x] 5.4 RED→GREEN：先在 crates/engine/speclink-fs/src/security/knowledge.rs 與 crates/adapters/speclink-cli/tests/it/security.rs 加「本機知識庫準備命令」缺件、symlink、外來目錄、取消、SHA及只選2025案例，再實作security kb prepare與原子快照；以 cargo test -p speclink-fs 及 cargo test -p speclink-cli --test it security 驗證人眼／JSON型別、exit0/1/2/130、--no-color、無網路請求及.gitignore，不覆蓋用戶知識庫。 <!-- speclink-task:tsk_01M5CJM0GACQH91FWC07GEBSRP -->

## 6. 完成紀錄與封存保留、多筆寫入與失敗恢復

- [x] 6.1 RED：在 crates/engine/speclink-core/src/quality/station/tests.rs 與 crates/engine/speclink-fs/src 補「蓋章守門與蓋章效果」「驗證蓋章守門與蓋章效果」的歷史保存、接受情形、工單刪除後可查及新輪不覆蓋案例；以 cargo test -p speclink-core --lib 及 cargo test -p speclink-fs 確認舊刪除行為使新斷言失敗。 <!-- speclink-task:tsk_01M5CJM0GAJEWRBQR75J3VTHE7 -->
- [x] 6.2 GREEN：在 crates/engine/speclink-core/src/quality/history.rs、crates/engine/speclink-core/src/quality/station.rs 定義checks/index.json、兩站完成快照與原活動工單身分，蓋章前保存原輪次／規則／指紋／身份／接受結果，discard不刪歷史；以6.1核心測試轉綠，原章欄位與成功摘要不變。 <!-- speclink-task:tsk_01M5CJM0GA0YZVWXKXG980K2QC -->
- [x] 6.3 在 crates/engine/speclink-fs/src/lib.rs 接專用checks journal，先加入每個發佈／刪工單／寫索引／寫metadata／恢復失敗點測試，再落實備份、sync、讀取拒半套與下次恢復；以 cargo test -p speclink-fs 驗證history_recovery_required、Windows鎖檔／路徑與唯一副本保全，不新增泛用交易框架。 <!-- speclink-task:tsk_01M5CJM0GARPMY8K00CGYFG91D -->
- [x] 6.4 在 crates/host/speclink-host/src/bridge.rs 與 crates/host/speclink-host/src 接兩站紀錄及index的同次staged transaction，先補CAS衝突與reader拒寫案例再實作；以 cargo test -p speclink-host 驗證歷史／章／活動工單同成同敗及正確scope，不傳送原始碼。 <!-- speclink-task:tsk_01M5CJM0GAFV1DGKXJ126XWAFW -->
- [x] 6.5 RED→GREEN：在 crates/adapters/speclink-cli/tests/it/review_verbs.rs、crates/adapters/speclink-cli/tests/it/verify_verbs.rs 驗證「審查完成紀錄查詢」「驗證完成紀錄查詢」，再於 crates/adapters/speclink-cli/src 接兩個history命令；執行 cargo test -p speclink-cli --test it review_verbs 及 cargo test -p speclink-cli --test it verify_verbs，斷言list／單筆JSON欄位與型別、原文人眼、active／archive、未知ID、--no-color及exit0/1/2，原show位元級輸出不變。 <!-- speclink-task:tsk_01M5CJM0GARN4MGMTJ366ADYPZ -->

## 7. 提案內報告與風險紀錄

- [x] 7.1 RED：在 crates/engine/speclink-core/src/security/report.rs、crates/engine/speclink-fs/src/security/reports.rs 加「版本化 JSON 報告模型」「深度報告九項內容與風險接受紀錄」的v1/v2、九欄、錯誤證據、400字元片段及唯一秘密sentinel案例；以 cargo test -p speclink-core --lib 及 cargo test -p speclink-fs 確認新增v2與保存檢查尚未滿足。 <!-- speclink-task:tsk_01M5CJM0GAM1DVKGHVGVBX5HQ8 -->
- [x] 7.2 GREEN：在 crates/engine/speclink-core/src/security/report.rs、crates/host/speclink-host/src/security.rs、crates/engine/speclink-fs/src/security/reports.rs 實作遮罩共用模型、提案checks/security與單一權威index，落實「本機報告保存與查詢」「報告輸出命令與語言」，v1及無change裝置報告維持；以7.1測試轉綠並執行 cargo test -p speclink-host 驗證日誌／事件／JSON／Markdown無秘密。 <!-- speclink-task:tsk_01M5CJM0GADABAEV0B8CZKKN6D -->
- [x] 7.3 先在 crates/adapters/speclink-cli/tests/it/security.rs 加「深度審查提交與查詢命令」及legacy匯入案例，再於 crates/adapters/speclink-cli/src/verbs/security.rs 接add-round／show及report --change；以 cargo test -p speclink-cli --test it security 驗證4MiB邊界、未知change／重複ID／越界證據／錯版拒寫、active／archive人眼與JSON型別及exit，legacy不作新驗收證據。 <!-- speclink-task:tsk_01M5CJM0GA5PFJV68E23BZCAD7 -->

## 8. 深度審查 skill

- [x] 8.1 RED：在 crates/engine/speclink-core/tests/it/render_golden.rs 與 crates/engine/speclink-core/tests/golden 建立「保留基本掃描並提供深度安全審查技能」「五種語言的架構與資料流深度審查」原稿與各工具契約斷言；確認缺新skill、指定標準優先、禁連網取資料、主開關、九欄及明確接受指引使測試失敗，不呼叫實際雲端模型。 <!-- speclink-task:tsk_01M5CJM0GAP6QNWN0F147QYD65 -->
- [x] 8.2 GREEN：在 crates/engine/speclink-core/assets/skills/security-review.md 與 crates/engine/speclink-core/src/workspace/skills.rs 註冊單一security-review，完整寫入五種語言入口／授權／機密／TLS／特定風險／資料流流程與scope限制，先讀KB、基本結果再深審提交；以 cargo test -p speclink-core --test it render_golden:: 驗證Claude／Codex／Copilot生成一致且基本掃描仍在，人工核讀生成原稿不新增外部標準或自動接受。 <!-- speclink-task:tsk_01M5CJM0GAVYGTGNQXA904MYJC -->
- [x] 8.3 在 crates/engine/speclink-core/assets/skills/review.md、crates/engine/speclink-core/assets/skills/verify.md、crates/engine/speclink-core/assets/skills/archive.md、crates/engine/speclink-core/assets/skills/config.md 更新「SDD 提示與 AI 邊界」「審查完成紀錄的技能交棒」「驗證完成紀錄的技能交棒」及「Verify 報告中的獨立初步安全資訊」；先加生成內容差異斷言，再同步資產版本與assets.lock，以8.2命令轉綠，不讓skills自行蓋過required判定或刪歷史。 <!-- speclink-task:tsk_01M5CJM0GAMAM2ANJW5H7K9AKS -->

## 9. 安全驗收判定

- [x] 9.1 RED：在 crates/engine/speclink-core/src/security/acceptance.rs 與 crates/host/speclink-host/src/security.rs 建立「安全驗收的等級完整性與新鮮度判定」矩陣：reference、LOW、HIGH／MEDIUM／CRITICAL、接受、缺資料／未跑／取消／未驗證、ASVS等級、resolvesCheckIds、停用項、政策／來源新增刪除／KB變更；以 cargo test -p speclink-core --lib 及 cargo test -p speclink-host 確認新判定缺席而失敗。 <!-- speclink-task:tsk_01M5CJM0GA2CNB5G1FNYQXRPRQ -->
- [x] 9.2 GREEN：在 crates/engine/speclink-core/src/security/acceptance.rs 依「深度提交的識別與指紋」實作單一純判定、由 crates/host/speclink-host/src/security.rs 供必要輸入與當次來源核對；保留basic與deep兩種報告、逐項補足未驗證，不把client-attested宣稱為server獨立驗源；以9.1同命令轉綠，只有相同digest接受可生效。 <!-- speclink-task:tsk_01M5CJM0GACY9Q43P4X1RTHRER -->
- [x] 9.3 RED→GREEN：在 crates/engine/speclink-core/src/security/report.rs、crates/host/speclink-host/src/security.rs 與 crates/adapters/speclink-cli/src/verbs/security.rs 實作「安全風險明確接受命令」與decisions.jsonl；先補空／2001字元理由、缺actor、舊reportId、未知finding、16KiB及CAS案例，再執行 cargo test -p speclink-host 及 cargo test -p speclink-cli --test it security，驗證新JSON型別、exit及原報告不改寫。 <!-- speclink-task:tsk_01M5CJM0GA4AQFT9P1Z1MSQN1H -->
- [x] 9.4 在 crates/engine/speclink-core/src/quality/station.rs、crates/engine/speclink-core/src/lifecycle/archive.rs 與 crates/host/speclink-host/src 接「必要安全驗收的封存判定」及verify stamp，先在 crates/adapters/speclink-cli/tests/it/verify_verbs.rs、crates/adapters/speclink-cli/tests/it/archive_readiness_gate.rs 加無章直封存、一般--accept與carry不能略過案例；以 cargo test -p speclink-cli --test it verify_verbs 及 cargo test -p speclink-cli --test it archive_readiness_gate 轉綠，reference既有輸出保持回歸對照。 <!-- speclink-task:tsk_01M5CJM0GAV4B07B5TCEBPF61F -->

## 10. Protocol、Remote 與 Desktop core

- [x] 10.1 RED：在 crates/protocol/speclink-protocol/src、crates/protocol/speclink-remote/tests/it 與 crates/host/speclink-server/tests/it 補「檢查歷史與安全驗收的共用契約」及「標準 error reason registry」的序列化、舊欄位預設、unsupported_version、403／401／CAS及跨scope案例；以 cargo test -p speclink-protocol 及 cargo test -p speclink-remote 觀察缺新型別或路由造成失敗。 <!-- speclink-task:tsk_01M5CJM0GA516A83Z7XS7AD77K -->
- [x] 10.2 GREEN：在 crates/protocol/speclink-protocol/src/command.rs、crates/protocol/speclink-protocol/src/query.rs、crates/protocol/speclink-remote/src/client.rs、crates/host/speclink-server/src 與 crates/adapters/speclink-node/src 接同一Host的history／report／accept契約；以10.1命令及 cargo test -p speclink-server 驗證型別與權限，Node先執行 npm --prefix crates/adapters/speclink-node run build 再 npm --prefix crates/adapters/speclink-node test，不複製驗收判定。 <!-- speclink-task:tsk_01M5CJM0GANR59CWB2499SZ7JX -->
- [x] 10.3 在 apps/desktop/core/src/security.rs 與 apps/desktop/src-tauri/src 接歷史、KB準備及結果查詢，先補projectKey／jobId隔離、取消、unknownID與Remote離線案例，再以 cargo test -p speclink-desktop-core 轉綠；Tauri命令維持單行委派，業務及事件結算不放殼層。 <!-- speclink-task:tsk_01M5CJM0GAWMEP28Q6GR1YCKN9 -->

## 11. 設定、文件樹與封存保留

- [x] 11.1 RED：在 apps/desktop/src/__tests__ 與 packages/ui/src/__tests__/changeDetailPage.test.tsx、packages/ui/src/__tests__/archivedDetailPage.test.tsx 補「納入安全驗收與封存紀錄的設定」「提案與已封存的檢查紀錄文件樹」：預設、取消／儲存、reader、CAS保草稿、歷次文件、A/B切換與已封存唯讀；以 npm test -w @speclink/desktop 及 npm test -w @speclink/ui 確認新行為失敗。 <!-- speclink-task:tsk_01M5CJM0GAH91EHDVMJM9BP5T2 -->
- [x] 11.2 GREEN：在 apps/desktop/src/views/project-settings/WorkflowPage.tsx、apps/desktop/src/views/project-settings/useProjectSettings.ts 及 packages/ui/src 實作checkbox與三類封存選項，主開關關閉保值、required未配置顯示阻擋說明，沿用SettingsCard／Row／Checkbox／CodeEditor；以11.1相關workspace測試轉綠，儲存不執行Agent或資料下載。 <!-- speclink-task:tsk_01M5CJM0GARKNWFXT8ZN6GZJFD -->
- [x] 11.3 GREEN：在 packages/ui/src/components/ChangeDetailPage.tsx、packages/ui/src/components/ArchivedDetailPage.tsx、apps/desktop/src/adapter 與 apps/desktop/src/store.ts 連同一index歷史查詢，顯示類別／時間／結果及唯讀內容，省略文件不留斷鏈，失敗不借用別專案內容；以 npm test -w @speclink/ui 及 npm test -w @speclink/desktop 轉綠並檢查鍵盤與Markdown遮罩。 <!-- speclink-task:tsk_01M5CJM0GA6S6ZBP5GBHWP82DM -->
- [x] 11.4 RED→GREEN：在 crates/engine/speclink-core/src/lifecycle/archive.rs、crates/engine/speclink-fs/src/lib.rs 與 crates/host/speclink-host/src/bridge.rs 補「封存時按類別保留完成紀錄」8種組合、required最小收據、carry工單、staging失敗及Remote衝突，再實作裁切與索引；以 cargo test -p speclink-core --lib、cargo test -p speclink-fs、cargo test -p speclink-host 驗證原活動提案不先刪、成功後已封存可查，驗收摘要保留接受理由。 <!-- speclink-task:tsk_01M5CJM0GABAXXT9363PB5R594 -->

## 12. 用詞、交付與實際環境驗收

12.3 的交付前置依使用者追加說明為：先完成實作與合併前必要檢查，將功能分支合併回本專案 main，再從 main 的明確 commit 打包產生安裝檔，之後才進公司驗收。該人工項在合併時仍保持未勾選，不阻擋上述安排；驗收證據須另記 main commit 與安裝檔名稱。此順序亦須由 15.2 同步到開發及換機說明，本輪不實際合併或打包。

- [x] 12.1 以先檢查再修正文案的方式，在 openspec/LANGUAGE.md、packages/ui/src/i18n.tsx、docs/product-status.zh-TW.md、docs/product-status.md 交付「安全驗收與檢查紀錄的用詞」，說明skill入口、基本掃描、資料離線、reference／required、歷史查詢、完整報告與驗收摘要的差別及版本需求；逐項核對九項原需求與追加checkbox均有手冊／操作說明，不把AI連線或驗收通過稱為離線認證。 <!-- speclink-task:tsk_01M5CJM0GA4F2CT0V5DE3FG1N8 -->
- [x] 12.2 對 crates/engine/speclink-core/src/security/acceptance.rs、crates/engine/speclink-fs/src/security/knowledge.rs、crates/host/speclink-host/src/security.rs 與 Protocol／CLI 新邊界套用audit的三個精簡觀點：設定降級／錯型別、來源與decisionId混用、舊server靜默略過、路徑連結、敏感證據及接受理由；每個實際缺陷先補失敗案例再修正，記錄無新增泛用抽象且失敗不當成功，執行對應受影響package測試。 <!-- speclink-task:tsk_01M5CJM0GAPTTDTJYAA6AMSRD6 -->
- [ ] [M] 12.3 公司 Proxy 打包後驗收（依 2026-10-21 使用者要求，開發期間跳過，待可安裝版本交付後執行）：在公司Proxy環境啟用診斷，以打包安裝的實際Desktop確認更新頁版本與日誌可見、手動檢查／下載或手動Release入口的正確路由與錯誤狀態；記錄app版本、OS、Proxy模式、遮罩operationId及結果到 openspec/changes/desktop-workflow-and-offline-security-review/evidence/company-proxy.md，不放憑證／URL機密；若仍失敗列具體錯誤並保持此任務未完成，不能以模擬測試或公司外網測試宣稱公司驗收通過。延期不阻擋繼續實作或產生測試安裝檔，不代表已通過正式品質驗收或封存。 <!-- speclink-task:tsk_01M5CJM0GADJK9R5AYBYKKR8VC -->
- [x] 12.4 RED：在 packages/ui/src/__tests__/proposalDoc.test.tsx 與 crates/engine/speclink-core/tests/it/render_golden.rs 建立「提案預設條列顯示」「提案內容預設以條列起草」案例，驗證兩份不同來源、Why散文加清單形成連續編號、保留巢狀／程式／連結、空章節與所有工具模板；執行 npm test -w @speclink/ui -- proposalDoc 及 cargo test -p speclink-core --test it render_golden::，確認新斷言因未條列而失敗。 <!-- speclink-task:tsk_01M5CTJ2NK1EC090BQS5VGVRH8 -->
- [x] 12.5 GREEN：依「提案與設計章節以中文標籤呈現」在 packages/ui/src/components/ProposalDoc.tsx 共用 NumberedSteps 呈現Why／What Changes／Proposed Solution的散文與清單，所有專案與活動／封存提案均預設套用，不改來源；更新 crates/engine/speclink-core/assets/schema/spec-driven/proposal.template.md、crates/engine/speclink-core/assets/skills/propose.md 及本提案的條列起草，資產版本／golden同步；以12.4測試轉綠及 git diff --check 驗證，排查同類入口並確認共用元件而無專案特例。 <!-- speclink-task:tsk_01M5CTJ2NKS57YK2EQVEPGSJYX -->
- [x] 12.6 完成交付回歸與文件對照：核對 crates/engine/speclink-core/tests/golden、crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs 的人眼／JSON契約，執行 cargo test -p speclink-core --test it render_golden:: 及 cargo test -p speclink-cli --test it remote_verb_parity，必要的刻意新欄位同批更新golden；檢查 docs/product-status.zh-TW.md 與 docs/product-status.md 對應、只重跑受影響面的失敗檢查，最後必跑 node --test scripts/*.test.mjs scripts/*/*.test.mjs，保留無法執行的原因，不將完整測試寫為本地任務驗證。 <!-- speclink-task:tsk_01M5CJM0GA7BNHKJSKW2F6R6PB -->

## 13. 本分支版本、更新來源與貢獻署名

- [x] 13.1 RED：針對「本分支更新與原作者版本」，在 apps/desktop/core/src/updater.rs 與 apps/desktop/src/__tests__/updatesPage.test.tsx 加失敗案例：兩個固定repo獨立結果、draft／prerelease／CLI-only過濾、最多100筆未取得、Proxy407／403／429、latest.json缺失但有桌面Release、原作者版號更高不成安裝候選。執行 cargo test -p speclink-desktop-core 及 npm test -w @speclink/desktop -- updatesPage，記錄因缺行為失敗。 <!-- speclink-task:tsk_01M5E8TT291QTBCZ8KMCQS4636 -->
- [x] 13.2 GREEN：在 apps/desktop/core/src/updater.rs、apps/desktop/src-tauri/src/updater_commands.rs 與 crates/host/speclink-host/src/network.rs 沿固定repo、既有HTTP限制與Proxy快照查桌面Release；有可信簽包才進原更新流程，無包回手動Release狀態，原作者只供資訊；以 cargo test -p speclink-desktop-core 及 cargo test -p speclink-host 驗13.1、錯誤分類與無默默直連，不更換簽章信任或新增任意URL。 <!-- speclink-task:tsk_01M5E9YEBPMTHRTG55VBMCS7JE -->
- [x] 13.3 GREEN：在 apps/desktop/src/views/settings/UpdatesPage.tsx、apps/desktop/src/views/settings/AboutPage.tsx、apps/desktop/src/adapter、apps/desktop/src/core/updater.ts 與 packages/ui/src/i18n.tsx 接獨立版本／時間／錯誤、原作者可點連結及本分支手動下載入口；補 apps/desktop/src-tauri/capabilities/default.json 精確放行與 tauri.conf.json 端點回歸。以 npm test -w @speclink/desktop -- updatesPage updater aboutPage 驗zh-TW／en、無updater仍有內附內容、失敗不冒充最新、候選只來自mosutw。 <!-- speclink-task:tsk_01M5E9YEXM3PM459ZBV9AKXRAB -->
- [x] 13.4 完成「專案貢獻與原作署名」：在 README.md、README.zh-TW.md 列mosutw維護、Codex AI協作與MomoChenisMe原作；CONTRIBUTING.md 記作者email／預設分支／真實共同作者條件及GitHub官方來源。人工核讀兩語一致、授權署名保留、未承諾側欄可手設；不改git config或歷史，執行 node --test scripts/*.test.mjs scripts/*/*.test.mjs。 <!-- speclink-task:tsk_01M5EA15GV1DYB7B2QCD8QFK9W -->

## 14. 安全標準共用資料路徑

- [x] 14.1 RED：在 crates/host/speclink-host/src/context.rs 補「應用程式資料目錄與離線版本清單」及「安全標準資料存放路徑設定」測試：缺檔／null舊預設、portable、自訂磁碟／Windows UNC、相對／不完整UNC／URL／NUL拒絕、設定壞版／錯型別／寫失敗保舊、共用根與私人根分離；執行 cargo test -p speclink-host 確認新增行為未實作。 <!-- speclink-task:tsk_01M5EA3FTR0513EXFHP0KFKG5E -->
- [x] 14.2 GREEN：在 crates/host/speclink-host/src/context.rs 及同crate負責裝置設定的模組建立單一security-data-preferences.json讀寫與根解析，原子保存formatVersion／standardsPath，缺值向後相容、錯值不fallback；接 apps/desktop/core/src/security.rs 與 apps/desktop/src-tauri/src/security_commands.rs 的讀寫IPC，回standardsPath／effectivePath／isDefault／access的camelCase型別，殼只委派。以 cargo test -p speclink-host 及 cargo test -p speclink-desktop-core 驗14.1、busy與路徑快照、唯讀候選可儲存及失敗保草稿。 <!-- speclink-task:tsk_01M5EB3DSK8XJFXJM1Z25M4WKD -->
- [x] 14.3 RED：在 crates/engine/speclink-fs/src/security.rs 及其測試補分離標準根的讀／裝／匯入／匯出案例：兩個裝置共用一根、私人reports／references／prepared不入share、唯讀來源可讀／匯出、斷線拒fallback、跨程序writer互斥、發佈失敗保舊、自訂根拒刪；執行 cargo test -p speclink-fs 確認新案例失敗。 <!-- speclink-task:tsk_01M5EA6H74322Y69F1P2QNHD04 -->
- [x] 14.4 GREEN：在 crates/engine/speclink-fs/src/security.rs 接標準根與application_data分離，沿不可變generation、跨程序鎖及同share staging發佈，唯讀者不建私人引用，自訂根禁止刪除；以 cargo test -p speclink-fs 通過14.3，核對不支援鎖／原子提交時拒寫而舊generation仍完整，不把本機fixture冒稱NAS實測。 <!-- speclink-task:tsk_01M5EB3ETN4FCHH3K7EFA4ACFG -->
- [x] 14.5 RED→GREEN：在 crates/host/speclink-host/src/security_data.rs、crates/adapters/speclink-cli/src/verbs/security.rs、apps/desktop/core/src/security.rs 及各Host建立FsSecurityData入口接同一解析；補CLI list人眼／JSON／--no-color、KB只讀已選版本、舊根私人報告可查、Server只讀服務主機設定案例。執行 cargo test -p speclink-host、cargo test -p speclink-cli --test it security 及 cargo test -p speclink-desktop-core，既有CLI shape與exit維持，Remote不接受客戶端任意UNC。 <!-- speclink-task:tsk_01M5EB5R4381M5H8KZ4CK5AR04 -->
- [x] 14.6 RED→GREEN：在 apps/desktop/src/views/settings/SecurityDataPage.tsx、apps/desktop/src/adapter 與 apps/desktop/src/__tests__ 補「安全標準資料存放路徑設定」互動並接IPC：手輸UNC／選目錄、儲存取消／還原、重開、唯讀變更動作停用但匯出可用、忙碌／斷線／設定失敗保舊。沿SettingsCard／SettingsRow與既有路徑輸入原語，以 npm test -w @speclink/desktop -- securityData 驗zh-TW／en與不觸發下載，不移動舊資料。 <!-- speclink-task:tsk_01M5EB3FHZ9MJSA7TM3D72S9R3 -->

## 15. 追加需求邊界核對與接續交付

- [x] 15.1 在 apps/desktop、crates/host/speclink-host、crates/engine/speclink-fs、crates/adapters/speclink-cli、.github/workflows/desktop-build.yml 與 scripts/release 以rg排查 MomoChenisMe／mosutw／latest.json／FsSecurityData::new／security_data_dir 同類入口，逐命中記共修或保留理由；對新設定與HTTP回應套audit，核對repo不混用、原作署名、UNC路徑／私人資料隔離、讀寫權限及跨主機鎖失敗不當成功。執行對應 cargo test -p speclink-host、cargo test -p speclink-fs、cargo test -p speclink-desktop-core 及 npm test -w @speclink/desktop，修實際缺陷後僅重跑受影響面。 <!-- speclink-task:tsk_01M5EBGWNJJH9AER40BV5FE4XM -->
- [x] 15.2 在 docs/development.md、docs/development.zh-TW.md、docs/product-status.md、docs/product-status.zh-TW.md 與 openspec/changes/desktop-workflow-and-offline-security-review/evidence 同步版本來源、手動更新、Contributors、UNC使用／唯讀維護、.security_kb仍占本機、公司打包後驗收與換機後新進度；明示本機模擬及未測NAS／公司限制。核讀與spec／design一致，最後執行 node --test scripts/*.test.mjs scripts/*/*.test.mjs 及 git diff --check，不代勾12.3、不封存。 <!-- speclink-task:tsk_01M5EBHDK7X3HHD44FANSFVE1A -->

## 16. Windows 封存前置修復（使用者同意優先處理）

- [x] 16.1 RED→GREEN：在 crates/engine/speclink-fs/src/checks.rs 與 apps/desktop/core/src/security.rs 保留並重現「檢查紀錄鎖與封存搬移」，將互斥鎖移至同spec store的.checks-locks並先鎖再建立活動目錄；補搬移期間競爭者busy且不重建活動目錄、舊鎖持有拒寫及鎖目錄拒連結測試。核對 archive_checks 的備份／發佈亦沿相同鎖、鎖不入Git且舊版寫入者必須先停止；執行 cargo test -p speclink-fs 及 cargo test -p speclink-desktop-core，最後執行 node --test scripts/*.test.mjs scripts/*/*.test.mjs 與 git diff --check，失敗不略過、不代勾人工驗收。 <!-- speclink-task:tsk_01M5E9VVQ5S88C8NHQFP33DVFD -->

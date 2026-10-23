# 重新審查與驗證：2026-10-21

本輪依使用者指示先備份兩份工單，透過 discard 清除活動工單，再由可信基準 faefda7d 重跑 discovery。完成後只記錄回報，未修正程式、未蓋章、未提交、未封存、未打包。

## 基準與備份

- 分支：feat/desktop-workflow-offline-security-review
- HEAD：f5f5179c805590e94fdf08083905874a6bb0507b
- Base：faefda7dabde01b8d90974aea63f4f7a8bdd5c24
- Phase：discovery；凍結範圍：203 個檔案；Review 與 Verify 使用相同 patch。
- Patch：sha256:3db821dc8347fc117a0896e8516a22105190993fdd233054c62fd535ebc159d2
- 原工單與 manifest：[備份目錄](quality-before-restart-2026-10-21/manifest.json)。兩份備份以 SHA256 核對，未從 Git 重建舊票據。
- review.md 備份 SHA256：1471f36270b78163f27786f9a3e183af0bda1a27afe4ad372b97e058bf1fcb1e
- verify.md 備份 SHA256：59ef55eb138fffa2892f4a6158998c8f4b961a67979f07122d80e610771b2058
- 新票據依序由 `review add-round`、`verify add-round` 寫入，兩站均為 Round 1；回讀確認 phase／patch／203 個 scope paths／findings／review rules 正確。

## 本輪結論與分流

共 13 項須修正：Review 6 項（1 CRITICAL、5 WARNING），Verify 7 項 WARNING。沒有 SUGGESTION，沒有接受保留項目。尚不可乾淨蓋章。

依目前規格修正這 13 項，不需修改 proposal／design／tasks／delta specs。V1 的 omitted 狀態與 V3 的必填欄位已有明文契約；若改成放寬契約，須由使用者先決定，不在本輪自行修改。V6 錨定規格檔是因範例位於該檔，建議修正落點仍為測試。

編號供後續選擇修正項目；工單內保留原文。

| 編號 | 等級 | 問題 |
|---|---|---|
| R1 | WARNING | CLI 重複實作已有共用的安全審查檢視組裝 |
| R2 | CRITICAL | 並行發佈安全報告可能遺失已完成審查的索引紀錄 |
| R3 | WARNING | 封存可能以舊 metadata 覆蓋較新修改 |
| R4 | WARNING | Node 缺少 security review 子命令時回傳 panic |
| R5 | WARNING | 手動更新候選未比較版本，最新版也會提示更新 |
| R6 | WARNING | macOS 系統路徑別名被當作不可信 symlink 拒絕 |
| V1 | WARNING | 省略封存紀錄時未在索引保留 omitted 狀態 |
| V2 | WARNING | CLI 人眼輸出缺少契約欄位 |
| V3 | WARNING | 深度報告接受缺少必填 endLine／function |
| V4 | WARNING | 遠端未知 reason 遺失 server message |
| V5 | WARNING | 缺少 CONNECT／TLS 完成並取得更新結果的成功測試 |
| V6 | WARNING | 缺少任務錨範例完整組合的同一測試 |
| V7 | WARNING | 缺少 required、零 finding、審查不完整且缺資料的阻擋測試 |

## Standards

審查規則：extend（12 項基本規則）＋專案規則 CRITICAL 1、WARNING 4、SUGGESTION 0。

- [WARNING] crates/adapters/speclink-cli/src/verbs/security.rs — Standards: 本機 `ReviewCommand::Show`（約577–615行）自行執行 `for entry in index.entries.iter().filter(...)`，讀取報告與風險決策，再組裝 reports／acceptance。Host 同次新增的 `review_view`（deep.rs 1159–1206行）已提供相同用途，Desktop、Node 與 Remote 也已沿用。違反「新增功能時，repo 已有相同用途的元件、編輯器或工具函式（例如 @speclink/ui 的元件、設定頁共用的程式碼編輯器）要直接沿用，不要另做一套平行實作」。建議 CLI 沿用 `review_view`，只保留 CLI 輸出排版，避免同一份資料組裝規則分頭維護。

其餘已查範圍未發現可成立的 Standards finding。backend／CLI／Node／Protocol 涵蓋 deep／remote／DeepReport、history／acceptance、FS checks／archive／knowledge、CLI config／security／station、Node bridge、Protocol DTO 與 typed client；另補查 Desktop core／Tauri／server 接線及相鄰新增測試。產生檔依 brief 僅作路徑背景，未逐行審查。

## Correctness

- [CRITICAL] crates/engine/speclink-core/src/security/report/deep.rs — Correctness: 第825行先解析 index，871行才讀 `old_index` 作比較。Local 可重現交錯：A 發佈安全報告，第一次 `read_artifact(INDEX)` 得 I0；B 蓋 review 章，提交 I1=I0+R、更新 metadata、刪除工單；A 第二次讀 INDEX 得 I1，卻提交 I0+A。FS 比較 I1 成功，R 從索引消失，metadata 仍顯示已 review，原工單也已刪除。FS 提交鎖未涵蓋前述讀取；Remote 的固定快照通常不受此交錯影響。

- [WARNING] crates/engine/speclink-core/src/quality/history/archive.rs — Correctness: 第214行先保存 `archived_meta`，`validate_base` 只比較 index。若另一個 Local 操作隨後透過 `set_board_rank`／`edit_meta` 成功修改 metadata，封存仍通過；archive_checks 第259–260行以計畫中的舊 metadata 覆蓋剛複製的最新檔案，成功保存的變更便遺失。原實作在移動完成後才讀 metadata，沒有這個覆蓋窗口。

- [WARNING] crates/adapters/speclink-node/src/lib.rs — Correctness: 第743–745行接受首項 `review` 後直接取 `&args[2..]`。呼叫 `dispatch(['security','review'])` 時 args 長度只有1，必定越界。外層會攔截並回傳 `code: "panic"`，而非正常的輸入錯誤；不會終止程序。

- [WARNING] apps/desktop/src/adapter/updater.ts — Correctness: 第40–48行遇到描述檔404／parse／unavailable，就將任何 fork Release 回傳為 ManualUpdate，未比較目前版本。已安裝最新版或較新版本時仍顯示「需手動更新」，也可能連到較舊安裝包。

- [WARNING] crates/host/speclink-host/src/security_paths.rs — Correctness: 第50–71行逐層拒絕 symlink，第180行 lease 又必經此檢查，但 Host 的 `global_config_dir`／`security_data_dir_for` 沒有先正規化路徑。實查此機 `/var → private/var`、`/tmp → private/tmp`；tempfile 回傳 `/var/folders/.../T`，同一目錄的實體路徑是 `/private/var/folders/.../T`。因此不只測試夾具：使用者選擇 `/tmp/standards`、`/var/...`，或 HOME／portable config 經過這些系統別名時，標準資料操作亦會被拒絕；一般 `/Users/...` HOME 不受此特定問題影響。FS 新增的 `with_standard_root` 第68–69行從 `/` 開始呼叫 `owned_directory`，也會在 `/var` 拒絕同一實體目錄。

原檢查限制原文：「已追讀交易、索引、驗收、Host 配對及部分 Desktop／Node 路徑；未逐項完成全部 CLI、Protocol、UI 顯示與 Windows／UNC 實機檢查，不宣稱全範圍通過。」後續補充了 Node、更新版本判斷與 macOS 路徑重現，未宣稱上述未查範圍全部完成。

並行索引與 metadata 問題為靜態交錯分析及不變條件違反，尚未建立可控制交錯的自動重現測試；不可把它們描述成已執行的回歸測試。Node 已用本輪重建的 addon 重現 `dispatch(["security","review"])` 回傳 code=panic，Rust 訊息為 range start index 2 out of range for slice of length 1。Updater 的獨立重現以安裝 0.10.0.4、404 fallback 分別提供同版 0.10.0.4 與舊版 0.10.0.3，兩者均得到 ManualUpdate。

Standards 共 1 項、最高 WARNING；Correctness 共 5 項、最高 CRITICAL。

## 驗證報告

| 面向 | 狀態 |
|---|---|
| 完整性 | 程式任務 55/55；全部任務 55/56；人工 Proxy 12.3 延後至打包後 |
| 規格對照 | 15 份規格、41 項需求、104 個情境、5 個範例均已核讀並靜態對照 |
| 情境測試檢閱 | 90 個情境有相關測試本體／斷言可對照，14 個為文件或 skill 指引；不代表 104 個情境的所有條件都受測 |
| 範例 | 3 個有同值組合測試，2 個缺完整組合（V6／V7） |
| 一致性 | 實作與規格有差異，7 項 WARNING 阻擋蓋章 |

- [WARNING] crates/engine/speclink-core/src/quality/history/archive.rs — Correctness: 封存省略紀錄後，索引沒有依規格標示 omitted。delta change-lifecycle/spec.md 第19行要求「完整報告被省略時索引須明示omitted」，第26行再次要求「省略的索引標omitted」。prepare_archive_checks 第209–213行直接以 index.entries.retain 移除整筆，CheckEntry 也沒有省略狀態。archive/tests.rs 的八種組合測試反而斷言省略類別不在索引；Remote 測試亦斷言索引 entries 為空。文件不會斷鏈，但封存後無法區別「未曾保存」與「依政策省略」。需保存可查的省略狀態並調整對應測試。

- [WARNING] crates/adapters/speclink-cli/src/verbs/security.rs — Correctness: 新命令的人眼輸出沒有包含規格要求與 JSON 相同的資訊。security-standard-data/spec.md 第55行列出 projectKey、policyDigest、standards 等欄位並要求「人眼列同資訊」，但第250行只印 Prepared knowledge at、path、files、bytes、knowledgeDigest，省略 projectKey／policyDigest／各標準的 standardId、version、dataSha256。Remote security review show 第865行也省略 required／blockingFindings／notVerified；本機第610行與 Remote 第865行的決策摘要沒有完整顯示補充契約要求的 action／policyDigest／scopeDigest，Remote 另少 reportId／acceptedAt。CLI 測試第574–575行對 KB 人眼輸出只檢查 Prepared knowledge at，未抓到欄位缺漏。需補齊人眼摘要及對應契約斷言。

- [WARNING] crates/engine/speclink-core/src/security/report/deep.rs — Correctness: 深度提交接受缺少規格必填欄位的 evidence。delta security-review/spec.md 第35行與 design 的 JSON 契約要求 endLine:number、function:string；DeepEvidence 第74–75行卻使用 Option，DeepReport::new 第203–204行以 line 代替缺少的 endLine，sanitize 第435行接受 None，第438行不要求 function 存在。FsSecurityData 的 proposal_security_reports_keep_v1_and_redacted_v2_in_one_index 測試也明確送入 endLine:null、function:null 並成功保存。需依契約拒絕缺欄／null 輸入並補提交邊界測試。

- [WARNING] crates/protocol/speclink-remote/src/lib.rs — Correctness: client-protocol/spec.md「未知 reason 不崩潰」第35行要求保留 server message 供顯示，但第132–134行的 Unknown(_) 分支直接丟棄 err.message，只回傳 unexpected server response 與 HTTP狀態。typed_client.rs 第786行附近的既有測試也斷言這個泛用字串，沒有保留 short and stout。這個分支是 patch 外既有行為，但它是本次 MODIFIED Requirement 的實際落點；擴充 registry 並沒有滿足 message 保留契約。需保留可顯示的 message 並調整未知 reason 測試。

- [WARNING] apps/desktop/core/src/network.rs — Correctness: 尚缺「模擬 CONNECT 與407」scenario 中首次成功取得更新結果的測試。connect_tunnel_success_does_not_disable_tls_or_hide_disconnects 第220–245行雖回 CONNECT 200，後續分別是壞 TLS、立即斷線及壞 Proxy 回應，三案都斷言 error_code.is_some()；既有 https_connect_407_retains_proxy_auth_status_and_diagnostics 覆蓋拒絕路徑，updater backend 測試則直接以 mock 回傳候選。這些測試沒有確認同一路由成功完成 TLS 並取得更新結果。需補成功路徑的可重現測試；此項與延期的公司人工12.3不同。

- [WARNING] openspec/changes/desktop-workflow-and-offline-security-review/specs/review-station/spec.md — Correctness: Example「蓋章寫入的任務錨」的完整輸入組合未被同一測試覆蓋。已定向核讀 station/tests.rs 的 review 蓋章測試、CLI review_verbs.rs 及 patch 外既有 manual_task_gates.rs；第555–564行使用5個任務、4寫碼全勾與1人工未勾，但只有Round 1；第622–637行使用Round 2空findings，但5個任務都是全勾寫碼任務。CLI manual案例則是2寫碼加1人工、Round 1。需增加或調整一個測試，同時使用5任務、4寫碼全勾、1人工未勾、Round 2空findings，並斷言 reviewed_tasks_total=5。

- [WARNING] crates/engine/speclink-core/src/security/acceptance.rs — Correctness: Example「納入驗收的具體矩陣」中 required=true、審查不完整、零finding且缺必要資料仍blocked的那列，未找到相同輸入的判定測試。acceptance_matrix_from_spec 第425–433行的不完整案例使用LOW finding；acceptance_digests_missing_evidence_and_explicit_resolutions 使用零finding但complete=true，測缺當次來源核對或增加必要check，沒有測缺必要資料且不完整的報告。另已核讀 Host freshness測試、Remote整合測試及CLI missing-data測試：前兩者帶HIGH finding，後者未啟用required或驗證安全驗收結果。需補零finding且缺必要資料／不完整時的required判定測試，明確斷言blocked及缺件原因。

沒有完整需求完全未實作的 CRITICAL；上述 7 項皆須修正，不能據此宣稱可封存。

### 靜態覆蓋清單

| 規格 | 需求 | 情境 | 範例 | 有相關測試檢閱的情境 | 文件／skill 情境 |
|---|---:|---:|---:|---:|---:|
| change-lifecycle | 2 | 4 | 0 | 4 | 0 |
| client-protocol | 2 | 4 | 0 | 4 | 0 |
| desktop-app | 6 | 20 | 2 | 19 | 1 |
| desktop-config | 3 | 7 | 0 | 7 | 0 |
| desktop-network-diagnostics | 2 | 4 | 0 | 2 | 2 |
| propose-skill | 1 | 1 | 0 | 1 | 0 |
| review-skill | 1 | 2 | 0 | 0 | 2 |
| review-station | 3 | 10 | 1 | 10 | 0 |
| security-review | 5 | 9 | 1 | 6 | 3 |
| security-review-report | 5 | 10 | 0 | 9 | 1 |
| security-standard-data | 3 | 9 | 0 | 9 | 0 |
| ui-copy-vocabulary | 1 | 2 | 0 | 2 | 0 |
| verify-skill | 2 | 5 | 0 | 0 | 5 |
| verify-station | 2 | 9 | 1 | 9 | 0 |
| workflow-config | 3 | 8 | 0 | 8 | 0 |
| 合計 | 41 | 104 | 5 | 90 | 14 |

## 本輪實際執行

| 指令／範圍 | 結果 |
|---|---|
| `cargo build -q -p speclink-cli` | 通過，使用目前原始碼建置 |
| `npx tsc --noEmit -p apps/desktop/tsconfig.json` | 通過 |
| `npm test -w @speclink/desktop` | 52 檔、1,306 項通過 |
| `cargo test -q -p speclink-fs -p speclink-host -p speclink-desktop-core` | Desktop core 357 通過；FS 50 通過、1 失敗、1 ignored，該次未繼續 Host |
| FS shared root 失敗案例單獨重跑 | 同樣失敗：`shared_standard_root_keeps_private_data_local_and_refuses_shared_deletion`，untrusted_link: data directory |
| `cargo test -q -p speclink-host -p speclink-desktop-core` | Desktop core 357 通過；Host 160 通過、2 失敗，均在標準路徑配置遇 linked／non-directory |
| `cargo test -q -p speclink-cli --test it security` | 9 通過、11 失敗；macOS 未正規化路徑守門造成直接拒絕及後續斷言失敗，未逐一單獨重跑 11 案 |
| `npm test -w @speclink/ui` | 1,340 通過、2 個 5 秒逾時；不是全組通過 |
| `npm test -w @speclink/ui -- changeDetailPage securityReviewPanel` | 原逾時兩檔以原 timeout 重跑，111 項通過 |
| `npm --prefix crates/adapters/speclink-node run build` | 通過，重建本輪原始碼 addon |
| `npm --prefix crates/adapters/speclink-node test` | 8 檔、55 項通過 |
| `node --test scripts/*.test.mjs scripts/*/*.test.mjs` | 新增本報告後再次執行，386 通過、0 失敗 |
| `git diff --check` | 通過 |

Host 失敗案例：`standard_path_change_is_busy_while_a_reader_holds_its_snapshot`、`standard_path_preferences_are_device_only_atomic_and_fail_closed`。實查 /var 與 /tmp 是 macOS 系統 symlink；測試路徑及實體路徑不同。

UI 首次逾時：changeDetailPage.test.tsx:2261（Remote 查詢失敗仍保留唯讀歷史）、securityReviewPanel.test.tsx:26（獨立安全資訊不產生 verify 工單）。兩檔重跑成功，未擅自增加 timeout。

## 未完成範圍與後續

- Windows／UNC／NAS 真實分享、權限及跨程序鎖未實機執行。
- 公司 Proxy 人工 12.3 依使用者指示待打包後驗收；不將此人工延期當作程式缺漏。V5 是獨立的可重現成功路徑測試缺口。
- 本輪未執行完整 `npm run test:all`；完整測試由 CI 守門，不宣稱全專案測試通過。
- 本專案目前安全審查主開關關閉；本次未啟動 OWASP 深度安全 skill，未下載或連網查安全資料。
- 待使用者選擇全部修正、指定項目或停止；不自動修正、蓋章或封存。

# OWASP / CWE 初步安全審查維護說明

本功能提供內建離線初步規則、Desktop／CLI 與獨立安全報告。人工斷網與視覺驗收仍待使用者完成；任務進度以 `openspec/changes/add-owasp-cwe-preliminary-review/tasks.md` 為準。

## 政策與資料邊界

政策唯一來源為 `openspec/config.yaml` 的 `security_verification`。缺席／null 預設停用。Rust 模型及局部文字改寫位於 `speclink-core::security::policy`；CLI 與 Desktop 共用，Remote 沿用 editor 權限及 revision 比對。前端只提交改過的葉節點，未知欄位及未改設定由原文保存。

`workflow-config security --stdin [--dry-run]` 接受至多 64 KiB 的 YAML mapping。四標準獨立，關閉仍保留版本；範圍 `off` 不代表主開關停用。沒有安裝資料不妨礙儲存政策，執行時另行列為資料缺失。

## 官方資料與授權

`crates/engine/speclink-core/assets/security/sources.json` 只放 SpecLink 維護的來源清單、版本、指紋及署名，不放官方標準全文。實際資料由使用者明確下載或匯入，與程式碼授權分開保存。解析表示是資料轉換，保留來源、授權、修改說明，不宣稱 MIT。

| 資料 | 固定版本來源 | 實際授權 |
| --- | --- | --- |
| Top 10 2021／2025 | OWASP/Top10 commit `3a31f35346c3f4e90f350395124382fa53af2afe`，每版十個分類與 LICENSE | 該 commit 的 CC BY-SA 4.0；不拿網站頁尾標示取代資產 LICENSE |
| API Top 10 2019／2023 | OWASP/API-Security commit `33cea37b2ffde3e2e62f6b5d79a029a22a846633` | CC BY-SA 4.0 |
| ASVS 4.0.3 | release `v4.0.3_release`；LICENSE.md 固定 commit `d31389d213b320cc2ef0cb1c30792cc80104d8ff` | CC BY-SA 3.0（不是新版的 4.0） |
| ASVS 5.0.0 | release `v5.0.0_release`；LICENSE.md 固定 commit `5cf9b032440be53ce345ab3c130fda46ba1ce7a2` | CC BY-SA 4.0 |
| CWE | MITRE 固定版 XML ZIP；版本取自 XML 根節點 | MITRE CWE Terms of Use，保留 copyright、條款及免責文字 |

官方證據：[Top 10 LICENSE](https://github.com/OWASP/Top10/blob/3a31f35346c3f4e90f350395124382fa53af2afe/LICENSE)、[API LICENSE](https://github.com/OWASP/API-Security/blob/33cea37b2ffde3e2e62f6b5d79a029a22a846633/LICENSE)、[ASVS 4.0.3 LICENSE](https://github.com/OWASP/ASVS/blob/d31389d213b320cc2ef0cb1c30792cc80104d8ff/LICENSE.md)、[ASVS 5.0.0 LICENSE](https://github.com/OWASP/ASVS/blob/5cf9b032440be53ce345ab3c130fda46ba1ce7a2/LICENSE.md)、[CWE 條款](https://cwe.mitre.org/about/termsofuse.html)。分享含官方改作的資料時須依適用版本保留署名與分享要求；不要把安裝資料打包成程式碼的 MIT 資產。

Top 10 只解析明列的 `List of Mapped CWEs`／`Related CWEs` 段落內 MITRE 定義連結，且標籤 ID 必須與連結 ID 相同。API 兩版的敘事／References 引用不視為正式分類對應。識別值永遠包含標準名稱、版本、項目 ID。

ASVS 4.0.3 逐級讀 L1／L2／L3.Required；5.0.0 按最低 L 展開。官方舊版有空架構章節，空章不產生要求，整份沒有要求仍拒絕。安裝資料與列出要求的預設狀態都是 NOT_VERIFIED，不代表專案滿足要求。ASVS release 沒有官方 digest 的資產使用 source-tls 加實際指紋；不冒稱官方 checksum。

## 一般程式庫審查

| 函式庫 | 授權與最低 Rust | 必要性及啟用範圍 |
| --- | --- | --- |
| quick-xml 0.41.0 | MIT；Rust 1.79；已在 Cargo.lock | CWE 串流解析；不開額外 feature、不解析外部實體；傳遞相依僅既有 memchr |
| ureq 2.12.1 | MIT OR Apache-2.0；Rust 1.71；已在 Cargo.lock | Host 明確下載操作使用 TLS；本層不開 gzip／json 等額外 feature，審查 run 不呼叫下載。既有 Remote client 的 feature 不變 |
| webpki-roots 0.26.11 | CDLA-Permissive-2.0；ureq 已使用同版 | 沿用同一份 TLS 信任根，建立可觀察取消的 TLS stream；保留資料隨附 license，既有 Desktop 第三方授權清單繼續納入本套件 |
| zip 4.6.1／flate2 1.1.9 | zip 為 MIT、Rust 1.82；flate2 為 MIT OR Apache-2.0、Rust 1.67 | 關閉 zip 預設 feature，只開 deflate-flate2，加 flate2 的 rust_backend；僅 store／deflate，不啟用 zopfli、zlib-rs、AES 或其他 codec。flate2／miniz_oxide／adler2 已在 lockfile，後兩者允許 MIT／Apache-2.0 等授權 |
| libc 0.2.186／windows-sys 0.61.2 | MIT OR Apache-2.0；均已在 Cargo.lock | 僅各自平台使用；安全資料檔拒絕符號連結／reparse，Windows 原子替換 current 指標，後續來源安全讀取沿用 |
| serde／serde_json／chrono／ulid | 均為既有專案依賴；MIT 或 MIT OR Apache-2.0 | Fs adapter 直接使用既有序列化、時間與不可猜測代號能力；不增加新的套件版本 |
| ctrlc 3.5.2 | MIT／Apache-2.0；Rust 1.69 | 僅 CLI 設定一次 Ctrl+C handler，回呼只設定取消旗標；不開 termination feature，不攔截其他訊號 |
| nix 0.31.3／cfg_aliases 0.2.2 | 兩者 MIT；nix Rust 1.69，cfg_aliases 未宣告最低版、已以本專案工具鏈建置 | ctrlc 的 Unix 訊號處理及建置輔助；nix 僅 signal／process，cfg_aliases 為 build dependency。實際 lockfile 只新增這三個 package；dispatch2、windows-sys、libc、bitflags、cfg-if 沿用既有版本 |

上述是一般格式函式庫，不是安全掃描框架。新增相依前核對本機 registry 的 Cargo.toml、LICENSE 與 Cargo.lock；新增其他相依時在此記錄實際 resolver 結果。

## 開發驗證

資料使用 `security/standards/<standard-id>/<version>/generations/<ULID>` 與原子 `current.json` 指標。版本鎖使用 Rust 標準庫的 `File::try_lock`（建置需要 Rust 1.89 以上），程序退出即釋放；讀取 pin 所選 generation，指標切換期間保留舊資料，競態最多重讀三次，不能一直重試。僅在版本鎖內清理逾24小時 staging；舊 generation 還須無讀取 pin 才清理。指標提交後的同步／清理錯誤回安裝成功附警告。Windows 使用 `ReplaceFileW`，沒有先刪舊指標的路徑；已核對 windows-sys 0.61.2 的宣告，尚待 Windows 實機驗收。

一般測試使用自有小型資料，不重製官方要求全文，也不連網。額外的 `security_standards_official_repository_snapshot` 測試預設忽略；必須由維護者明確取得官方固定版後，設定 `SPECLINK_SECURITY_OFFICIAL_FIXTURES` 再使用 `--ignored` 執行。目錄格式為 `<standard_id>_<version>/<官方檔名>` 與 `LICENSE`。測試不下載資料，核對已固定檔案指紋及真實格式。

Fs 的 `security_data_official_fixtures_install_and_read_offline` 使用同一目錄，另驗證七個版本在暫存資料根完成安裝及離線讀回；CWE 輸入放原始 `cwec_v4.20.xml.zip`，Engine 的格式測試另需已展開的 XML。均不操作使用者真正的應用程式資料。

官方 CWE 4.20 ZIP 的唯一 XML 帶 regular mode 100750，專用解碼器接受該固定名稱 XML 的資料內容並忽略執行位元，保存檔案不沿用該權限。這個特例僅在單一、版本名稱完全吻合的內層 CWE XML 封裝生效；同樣權限放在外層離線包仍拒絕。XML 不會執行。

離線包是一般 ZIP（store／deflate），包含 `manifest.json`、根 `LICENSE`、`raw/` 下對應版本的固定檔名；可帶空的 `raw/` 目錄記錄。manifest 欄位為 `formatVersion:1`、`standardId`、`version`、`sourceRevision`、`sourceUrl`、`sourceFormat`、`licenseName`、`licenseUrl`、`attribution`、`entries:[{name,sha256}]`。來源與署名欄位按 sources.json 的選定版本填入；每個實際檔案都必須列出 SHA-256。CWE 可放原始 XML ZIP 或展開 XML；同時提供時兩者須一致。拒絕 ZIP64／multi-disk／加密／其他 codec、重名或大小寫碰撞、特殊檔與可執行權限、任意多餘路徑。解碼前先檢查 end record 與最多128 entries，避免第三方 ZIP 函式庫依惡意 count 配置大量記憶體。匯入只證明此包內一致性，保留 imported 信任標記，不能宣稱官方簽章。

已於開發中驗證的政策測試包含核心設定、CLI Local／Remote、Desktop 文字改寫、Server 權限及 CAS、前端設定互動。完整交付與手動離線驗收仍須按任務清單完成。

## 資料管理操作（已實作）

Desktop 的應用程式設定新增「安全標準資料」，沒有開專案也能進入。進頁只列本機資料；點「更新版本清單」才查官方 GitHub contents／release JSON，CWE 固定候選用 HEAD 查可用性。遠端回應不能改寫維護者固定的下載網址、指紋、授權或規則。來源不符標為 unavailable，已安裝資料仍保留；新增格式或版本須先更新來源清單與解析器。點指定版本的「下載／重新下載」才取資料；「匯入離線資料包」由原生檔案選擇器選 ZIP，沒有網路操作。錯誤留在頁面，已安裝狀態不會因失敗變成成功。

CLI 已提供 `security data list --json`、`refresh --json`、`install owasp_top10 --version 2025 --json`、`import PATH --json`、`remove owasp_top10 --version 2025 --json`。這些是裝置層命令，不需專案，也不做 Remote handshake；只有 refresh／install 會明確連線。進度走 stderr，stdout 保持單一 JSON。Desktop 取消與 CLI Ctrl+C 均接共用取消旗標；Unix 實際 SIGINT 整合測試確認兩秒內結束、exit130 且保留部分報告，Windows 訊號操作仍待實機驗收。

下載使用正常 TLS 憑證驗證，不讀環境 proxy。DNS 的不可中斷系統查詢限制最多四個工作執行緒，等待上限一秒；取消或逾時後的結果不會發起請求。TLS 及 socket 每次讀寫檢查取消並限制一秒阻塞；單份資料工作限制120秒，實際下載總量64MiB。已測試重新導向拒絕、chunked 超量、虛擬期限、兩秒內取消、失敗保舊及不外洩傳輸錯誤原文。

本機已讀政策快照存於 `security/references/<專案識別的SHA256>.json`，與安裝／執行中的版本 pin 共同保護刪除。Desktop 設定讀取及安全政策更新會記錄引用，失敗顯示警告而不撤銷已寫入的專案政策；CLI 刪除前也檢查當前本機政策。後續 prepare／run 使用同一份紀錄。外部修改的政策需重新讀取才更新快照，不枚舉未知／未開啟的 Remote 專案。刪除不更動專案政策與舊報告，先移除 current 可見性再清理資料，清理失敗可重試。版本鎖檔保留原位，避免刪除重建鎖檔造成兩個行程各持不同鎖。

## 離線 CLI 與報告（已實作）

`security rules --json` 列19條自有規則；`security run [--root PATH] [--change NAME] --json` 只用已授權本機檔案及已安裝資料；`security report ID [--format json|markdown]` 不重掃描。沒有啟用政策會產 DISABLED 報告、來源讀取為零；全部標準關閉仍可明確執行自有規則，標準化部分顯示 unconfigured。完整執行即使有疑似風險仍 exit0，有未驗證／未完成 exit1，取消 exit130；語法錯誤維持2。報告不新增 verify／review 工單或章。

Remote 先明確執行 `security prepare`，透過既有認證與一致 Context Snapshot API 取得政策與指定規格；只傳送 change／flow 選擇，沒有原始碼。來源每份 digest 皆核對，政策64KiB、單規格1MiB、合計100MiB，HTTP JSON實際回應另限256MiB。失敗／401保留舊快照。`run` 不建立 remote client、不發 handshake、不改 Context Projection。快照綁 origin／project／repo／canonical checkout／change；未指定 repo 且本機有多份候選時拒絕猜測。

快照放 `security/prepared/<project-key>/<scope>/generations/<ULID>`。同代分開保存 policy.json 與 snapshot.json，current 指標原子切換，個別保存指紋；停用或不做規格審查時只讀政策，不讀規格內容。重新準備與刪除標準共用引用鎖；刪除直接檢查準備快照的政策。原始碼只活在短期記憶體，跨檢查共用同一份不可變文字，避免重複複製大型來源。

報告放 `security/reports/<project-key>/<report-id>`，JSON／Markdown／manifest 一起完整提交，latest 索引失敗不撤銷已完成報告。查詢核對兩格式指紋及模型一致性；上限1000份／200MiB，不自動清除舊報告。可先匯出需要的報告，再由使用者管理舊資料。Markdown 對 HTML、控制字元與格式符號做處理；疑似憑證整筆遮罩，其他證據亦遮罩字串、URL與 Token-like 內容。

OpenAPI 僅靜態讀3.0／3.1內部引用；不支援組合schema、自訂dialect／base URI、動態引用及 YAML anchor／alias 時列 NOT_VERIFIED，不猜測通過。參數覆寫及型別依 [OpenAPI 3.0.3](https://spec.openapis.org/oas/v3.0.3.html)、[3.1.0](https://spec.openapis.org/oas/v3.1.0.html) 查核。契約出現安全要求仍不代表執行時實作已驗證，ASVS無直接測試器的要求逐項保留未驗證。

## Desktop 操作與人工驗收

1. 以 `npm run dev:desktop` 啟動開發版，開啟已授權的測試專案。
2. 在專案設定「產出政策」的 audit 下方開啟「啟用 OWASP / CWE 安全審查」。選擇 Web 應用程式 + API、Top 10 **2025**；API Top 10、ASVS、CWE 保持關閉。三種審查可獨立選取；儲存後重新開啟確認還原。
3. 進入應用程式設定「安全標準資料」，明確點 Top 10 2025 的下載。確認版本、來源及授權；資料位於應用程式資料目錄，沒有複製進專案。
4. Remote 專案須先在變更詳情的「驗證」分頁點「準備離線快照」。這一步使用既有登入及 Context Snapshot API，需要連線，失敗不覆寫上次完整快照。之後也可明確重新準備，更新政策及規格。沒有 checkout 時，來源審查保留未驗證；不會去掃 server 或別的專案。
5. 關閉網路，進入變更詳情「驗證」，點「執行基本審查」。不需先建立 verify.md，也不需 AI Agent。過程可取消；部分完成的報告保留未驗證原因。
6. 檢查標準版本、風險、信心、來源位置、遮罩證據及逐項狀態。點來源只會在檔案管理員顯示已重新核根的本機檔案，不執行內容。
7. 匯出 JSON、Markdown，或複製遮罩摘要。匯出遇到同名檔或連結拒絕覆寫，請另選新檔名。確認沒有原始測試密碼或 Token。
8. 切換繁中／英文與淺色／深色，檢查可讀性、取消、重開設定，以及原 TDD／audit／worktree 與封存流程。這些實機步驟由使用者回報平台及結果，任務 7.4 保持未勾選；自動測試不能代替斷網與視覺驗收。

UI 入口由可選的 `SecurityReviewAdapter` 提供。server-web 沒有本機能力時不顯示可掃描入口；舊 server 沒宣告 securityReview 時不啟動安全工作，安全政策列顯示 unsupported_version，其他設定照常使用。背景事件同時核對 jobId／projectKey，換專案不會將舊工作的進度或報告顯示到新專案。

## 覆蓋、狀態與資源上限

| 面向 | 第一版行為 |
| --- | --- |
| 程式碼 | Rust、TypeScript／JavaScript 的六條基本規則；JSON／YAML 只支援明列不安全設定。其他語言 NOT_VERIFIED。每條規則有正向、負向、誤判與不支援案例 |
| 規格 | 選定變更的 proposal／design／tasks／specs；以中英日明列詞組找設計缺漏，有目標才提示，不將缺少文字視為已確認漏洞 |
| API | OpenAPI 3.0／3.1 JSON／YAML 的七項靜態邊界；只解析內部 `$ref`。外部引用、循環、無法處理的組合與自訂 schema 保留 NOT_VERIFIED，不呼叫 API |
| ASVS | 依版本和等級展開官方要求；目前沒有足以確認實作滿足完整控制項的直接測試器，要求保留 NOT_VERIFIED，不能做合規認證 |
| 來源上限 | 單檔1 MiB、合計100 MiB、10,000檔、深度32、60秒；額外目錄枚舉上限20,000項。超限、取消、解析／讀取失敗均記原因 |
| 規則上限 | 每檔最多100筆、合計10,000筆風險；超長程式行、過大模式視窗、OpenAPI節點／操作超限保留未驗證 |
| 資料安裝 | 下載64 MiB、解開總量256 MiB、最多128項；XML128 MiB、其餘單檔2 MiB、120秒。拒絕外部實體、惡意 ZIP 路徑、重複檔及超量展開 |
| 排除 | `.git`、`node_modules`、`target`、`dist`、`build`、`.speclink/context` 與明列產生檔標記；`exclude_dirs` 可補充相對路徑，不能跳出根 |
| 報告 | 本機1000份／200 MiB，不自動刪除；先匯出再由使用者管理。Unix 匯出完整暫存後不覆寫發布；Windows 使用拒絕覆寫且核對 handle 的寫入，實機行為仍待驗收 |

`PASS` 僅表示已執行的指定規則未命中；`FINDING` 是疑似風險或安全要求缺漏，`NOT_VERIFIED` 是未完成／不支援，`NOT_APPLICABLE` 必須有理由，`DISABLED` 是使用者停用。五者不能互換。沒有全域「沒有漏洞」或「OWASP 認證通過」。

SDD instructions 的 securityVerification 是安全政策摘要，包含 enabled、scope、標準版本、審查種類、固定需求提示與 executionMode。dataAvailability=device_local_check_required 代表 Engine／server 無法替這台裝置斷言安裝狀態，須查 `security data list`。不把下載原文放成 Agent 指令。Verify 的安全資訊與原工單分開，不產生安全章、不新增封存守門；AI 只由使用者自行把遮罩摘要交給既有 Agent，若 Agent 連線就不稱為離線審查。

## 維護位置與新增規則

| 層 | 主要修改位置與責任 |
| --- | --- |
| 純 Rust Engine | `crates/engine/speclink-core/src/security/`：政策局部更新、來源模型與解析、19條規則、狀態、準備快照與唯一遮罩報告；`assets/security/` 僅自有規則及來源清單 |
| 檔案系統 | `crates/engine/speclink-fs/src/security.rs` 與 `security/{scan,prepared,reports}.rs`：安全根、限制、交易、引用、保存與匯出 |
| Host | `crates/host/speclink-host/src/security.rs`／`security_data.rs`：共用離線流程、預算、明確下載／取消；不依賴 Fs adapter，呼叫端組裝 |
| CLI | `crates/adapters/speclink-cli/src/verbs/security.rs`、config／instructions／main：離線命令、單一 Ctrl+C handler、Remote prepare 既有認證 |
| Desktop | `apps/desktop/core/src/security.rs`、Tauri `security_commands.rs`／remote：背景工作、核根與薄 IPC；`src/adapter/security.ts`、WorkflowPage、SecurityDataPage、session／App 接線 |
| 共用 UI | `packages/ui/src/security.ts`、SecurityReviewPanel、ChangeDetailPage、i18n：可選能力、獨立驗證區與純文字呈現 |
| 協定／SDD | protocol binding／query、remote client、server routes／identity、instructions／policy、五份技能來源及產生的三種工具技能與 golden |

新增規則時，先在 `assets/security/rules.json` 加唯一 SL ID、語言、類型、嚴重程度、描述、修正方式與有來源的 CWE 參考。規則 selector 僅對應程式內已實作的檢查，不接受下載的 regex、Shell 或可執行插件。先寫正向、負向、誤判、不支援案例，再加受取消／上限控制的檢查。沒有可信對應就只保留 SL ID；OWASP 對應需使用所選版本官方列表的實際證據。規則變動需同步規則版本與報告測試。新增官方資料版本則先查核授權、固定來源與格式，更新 sources.json、解析器及離線 fixture 測試；版本清單更新不能遠端注入新解析程式。

常見錯誤碼：missing_data、unsupported_version／unsupported_language、invalid_package／invalid_format、version_mismatch、integrity_error、license_missing、busy、in_use、storage_limit、budget_exceeded、cancelled、missing_preparation、binding_mismatch、invalid_config、source_unavailable。Host/Fs 內部保留原因；Desktop 邊界以 code／message／stage／target 回報，未知錯誤縮為 review_error／data_error，不回原始回應本文或秘密。下載失敗舊版仍可讀；報告寫入失敗不能宣稱已完成保存。

目前未完成的是使用者實機驗收（任務7.4），以及其他平台的實機驗證。Windows 的條件編譯路徑、reparse、取消與輸出未在本次 macOS 環境執行；沒有宣稱跨平台實測通過。第一版也沒有依賴漏洞資料庫比對、DAST／實際 API 呼叫、完整語意分析、完整 ASVS 驗證、Security Gate 或自動 AI 審查；這些都不在本次實作範圍。

已補齊自有19條規則的繁中／日文說明與修正建議，以及報告章節標題；英文維持規則原始文字。官方控制項內容與結構鍵、ID 不翻譯，避免混入官方文件改作。

完整修改檔案索引見 [security-review-files.txt](security-review-files.txt)；下列最終測試紀錄在完成品質檢查後更新。

## 最終驗證紀錄（2026-10-17，macOS）

| 檢查 | 實際結果 |
| --- | --- |
| `cargo test -p speclink-core --offline` | 1055 單元＋88整合通過；預設忽略1個需官方資料的案例，已另行明確執行通過 |
| `cargo test -p speclink-cli --offline` | 25單元＋559整合通過；包含實際 SIGINT、Remote 401保舊、零 HTTP、Top10 2025獨立審查、遮罩及原封存守門 |
| `cargo test --no-fail-fast -p speclink-fs -p speclink-host -p speclink-protocol -p speclink-server -p speclink-desktop-core -p speclink-remote --offline` | Desktop core331、Fs41、Host149、protocol74、remote123、server381通過；合計1099。serverfs helper 預設 ignored，由重啟測試子程序執行；PostgreSQL3例因未設定 SPECLINK_TEST_POSTGRES_URL 而略過 |
| 官方資料明確驗證 | 設 `SPECLINK_SECURITY_OFFICIAL_FIXTURES=/private/tmp/speclink-security-research`，對 core／Fs 執行 `cargo test ... security_ --offline -- --ignored`，2項通過：七個固定版本真實資料格式、安裝及離線讀回。未下載進專案、未觸及使用者應用資料 |
| `npm test -w packages/ui -- --maxWorkers=2` | 80檔／1323項通過 |
| `npm test -w apps/desktop -- --maxWorkers=2` | 48檔／1234項通過 |
| `npm test -w apps/server-web -- --maxWorkers=2` | 21檔／244項通過；初輪高負載下2項等候失敗，降低平行度後整套通過 |
| `node --test "scripts/**/*.test.mjs"` | 364項通過，含雙語文件錨點、產物及腳本守門 |
| `npx tsc --noEmit -p apps/desktop/tsconfig.json`、`-p packages/ui/tsconfig.json` | 均通過 |
| `CARGO_NET_OFFLINE=true npm run build -w apps/desktop`、`npm run build -w apps/server-web` | 均成功；Vite 仍提示既有大型 chunk。Desktop 授權清單再生成功（npm227／Cargo574項） |
| `cargo check -p speclink-desktop --offline` | 通過；不是完整安裝包或跨平台實機測試 |
| 技能資產 | ASSET_VERSION=v1.49.1，明確 UPDATE_GOLDEN／UPDATE_ASSETS_LOCK 後，無旗標 core 全套及57項 render測試通過；`speclink update` 同步本專案已設定的 claude／codex／copilot |
| `cargo fmt --all -- --check` | **未通過**。專案既有大量格式差異仍在；已用 HEAD 原始的 cache.rs、lifecycle/newcmd.rs 複驗，兩者在本次修改前即不符 rustfmt。僅整理本次新程式與改動片段，沒有全面改寫原程式 |
| `speclink analyze add-owasp-cwe-preliminary-review --json` | 0 Critical、0 Warning、44 Suggestion（規格情境尚可補具體 Example）；Coverage／Consistency／Gaps均Clean |
| `speclink validate add-owasp-cwe-preliminary-review` | valid；有新能力名稱相近的提示，三個安全能力分別負責規則、資料與報告，不合併成品質關卡 |
| `git diff --check` | 通過 |

一般 Rust 共2826項通過，另2項官方資料案例明確執行通過；不把 PostgreSQL略過、Windows未執行、格式失敗或人工驗收算作成功。既有 CLI trace.rs 的 unused Path 警告保留，未刪不相關程式。

本分支 `feat/owasp-cwe-preliminary-review` 保留全部修改，未自動 commit／merge／archive。所有程式任務完成後只剩使用者的7.4人工驗收；可先進行獨立 review／verify 品質關卡，封存仍等人工結果。

## 實機回饋修正

- 點下載或重新下載後，只有當次標準及版本按鈕立即顯示「下載中」，直到工作結算；其他版本保持原文案並停用。失敗或取消後還原，沿用同一個下載進度及取消流程。
- 專案設定提供裝置層圖示底色色票，重開保留，主視窗與系統匣一致。自訂色票選取時深底白字，未選取時底色淡化50%並搭配深色字；不修改任何團隊政策。操作見 configuration 的「專案圖示底色」。
- 繁中與系統繁中採同一字典；LANGUAGE.md 以「預設」作為 default 的台灣用語，新增介面字典與系統語言測試。官方原文與歷史紀錄保留。

本輪回饋驗證：Desktop 49檔／1244項全部通過；腳本364項通過；Desktop TypeScript檢查、production build與git diff --check通過。色彩案例包含裝置儲存失敗不採用新值、Local／Remote隔離、重開保留、主視窗及面板；下載案例包含API2019／2023的當次版本與重下載失敗還原。規格分析0 Critical／0 Warning／46 Suggestion，validate通過。原人工7.4仍保留待驗收，不因本輪自動測試代勾。

圖示色彩第二輪回饋（後續由第三輪淡化調整取代）：取消自訂色票的選取深淺差別，六色在專案列、系統匣及設定預覽皆固定深底白字；紫色與粉紅色保留不同色系。既有儲存值直接生效，不需搬移資料。新增／調整案例先確認9項失敗，再完成修正；本輪針對性 Desktop 測試5檔／274項、腳本364項、Desktop型別、production build及git diff --check通過。建置仍有既有大型chunk提示；未代替使用者進行實機視覺驗收。

圖示色彩第三輪回饋：未選取時以原深色混入50%白色，避免最初100色階過淡；搭配同色系深色字以保留可讀性。選取時維持深底白字。共用於專案列、系統匣與設定預覽，既有設定不需搬移。本輪針對性Desktop測試5檔／274項、型別檢查與production build通過，確認六色淡化樣式皆進入建置產物；腳本364項與git diff --check通過，建置仍有既有大型chunk提示。實機視覺驗收仍待使用者確認。

品質審查第一輪修正：安全資料的來源／授權連結改用既有系統瀏覽器與失敗提示；Desktop opener只增列OWASP官方repository／原始文件及MITRE資料／條款網址，測試核對來源清單每個連結都有權限。標準關聯函式接入同一取消／期限回呼，已停止時不進行完整對應，迴圈中定期中止並保留NOT_VERIFIED原因。各審查結果合併時按finding.path共用每檔100筆與全域10,000筆額度，截斷保留finding_limit及不完整覆蓋，新增Host整合案例由200筆降為100筆，其他檔案不受影響。instructions摘要契約的規格文字修正為設計既定的standards／reviews陣列，完整政策的show物件不變；CLI逐artifact驗證欄位形狀。

本輪修正驗證：core／host／desktop-core共1625項、CLI instructions_policy 9項、Desktop 49檔1253項全部通過；core官方資料fixture 1項維持ignored。Desktop型別、production build及原生cargo check通過，既有大型chunk提示仍在。重現資料與程式只放暫存目錄，未加入專案；規格validate、腳本364項及git diff --check通過。未重跑其餘套件整套測試，由CI維持完整守門；未宣稱Windows實機或全部情境已驗證，品質章仍待本輪驗證及使用者決定。

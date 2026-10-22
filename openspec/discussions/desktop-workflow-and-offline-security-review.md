---
topic: 左側列的專案要能夠調整位置順序、設定中的更新在公司 proxy 內看不到資料、審查規則說明、離線 OWASP 安全審查 skill 與提案內紀錄保存
slug: desktop-workflow-and-offline-security-review
status: promoted
created: 2026-10-20
created_by: mosutw <luke1974@gmail.com>
promoted_to: desktop-workflow-and-offline-security-review
---

# Discussion: 左側列的專案要能夠調整位置順序、設定中的更新在公司 proxy 內看不到資料、審查規則說明、離線 OWASP 安全審查 skill 與提案內紀錄保存

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者提出九項需求：左側專案排序、公司 Proxy 環境更新頁無資料、預設十二條審查規則的問號說明、OWASP 審查只使用下載時備齊的離線文件、嚴守指定標準、安全報告存提案內且 Desktop 可查、產出流程可設定三類紀錄的封存保留、參考 spec_registry.json 設計資料包，以及以 skill 執行五種語言的深度安全審查。需求已有具體目標，直接提出假設，不需先追問泛化目標。使用者首輪回覆進一步指定審查與驗證蓋章後仍須在提案內留存可隨時查詢的紀錄，OWASP 安全審查採與 speclink-review、speclink-verify 一致的 skill 方式。
相關正式規格為 desktop-app、desktop-config、workspace-session、security-standard-data、security-review、security-review-report、review-station 與 verify-station；目前沒有進行中的變更。既有標準資料及審查規格已要求審查不連網、不擅用停用標準；安全報告現存裝置資料目錄，quality/station.rs 的 write_stamp 則在蓋章前刪除 review.md／verify.md，兩者都須調整才能符合此次保存要求。既有初步審查以非 AI 靜態規則為基礎，本次要求以 skill 執行深度審查，需明訂舊入口處置。
附件 /Users/mosu/Downloads/gemini-code-1792493480145.json 是資料清單參考，不是可直接執行的安裝指令；其中 cheatsheets 的 2024-LTS 指向浮動 master，CWE 4.16 也未以網址固定實際內容版本，且需補齊角色描述要求的對照表。不得修改附件或直接相信其 installed 狀態、版本及授權標籤已經過驗證。
Prior discussions: desktop-colors-network-and-offline-bundles, owasp-cwe-preliminary-security-review, config-skill-rule-routing, detail-quality-stamp-dedup, desktop-theme-and-project-directory, detail-content-redesign, copilot-project-tool, drawer-quality-ticket-tab, update-check-on-focus-and-cli, improve-layout-signal, improve-repo-layout, release-changelog-whats-new, remote-remaining-gaps, instruction-downgrade-guard, worktree-flow-gaps, worktree-parallel-apply, code-review-convergence-boundary, review-ticket-locale, code-review-stage, desktop-instruction-staleness-prompt, desktop-workspace-auto-init, remote-login-ux-gaps, remote-workspace-local-skill-bootstrap, web-service-navigation-redesign, tray-remote-project-switch, spec-archive-drawer-ux, desktop-導覽與專案首頁重構, 專案選擇對齊-spectra, skill-conversation-language, test-speed-and-wording, custom-review-rules, detail-locate-white-gap, manual-callout-unchecked-only, quality-gate-speed, doc-tiles-fill-and-round-order, server-web-desktop-alignment, tray-panel-content-priority, tray-panel-windows, spec-manual-page-alignment, desktop-ui-redesign, skill-prompt-optimization, docs-cleanup-findings, github-copilot-tool-support, plan-overlap-granularity, board-height-under-banners, worktree-apply-plan-preflight, change-execution-order, release-assets-trim, canonical-spec-zh-wording, analyze-rule-tuning, manual-stale-time-granularity, spec-drawer-trace-links, manual-generation-skill, rename-onboard-to-baseline, feature-provenance-skill, tdd-switch-apply-wiring, schema-engine-openspec-parity, desktop-cli-multi-workspace-concurrency, desktop-loading-skeleton-ux, archived-parity-and-spec-purpose, dev-sidecar-freshness, change-scoped-test-policy, config-skill-rule-alignment, quality-skill-pause-and-ui-polish, card-drawer-header-colors, change-drawer-header-redesign, worktree-toggle-and-guards, apply-provenance-scope, tray-station-badges, tray-panel-scroll-and-vibrancy, one-click-install-and-run, config-yaml-setup-skill, tray-badge-and-recovery-buttons, desktop-feedback-surface, tray-panel-card-design, system-tray-status, tray-rich-panel, collab-scenario-replan, desktop-reading-and-tasks-ux, desktop-window-and-settings-polish, config-context-與-rules-gui-編輯, sdd-engine-as-sdk-with-pluggable-document-storage-for-team-scenarios

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-20)

**Focus**: 三類檢查如何採共同的 skill 流程並在提案內保留可查詢的執行紀錄。
**Position**: 使用者已指定審查與驗證蓋章後仍保留提案內紀錄，且 OWASP 安全審查以 skill 執行；其餘原始需求全部保留，尚未將初始建議視為已獲確認。
- 決策樹分為桌面操作（專案排序、公司 Proxy 更新問題、十二條說明）、離線資料（完整下載、限定標準、附件資料清單）、檢查流程（skill、提案內紀錄、封存保留政策）。
- 紀錄保存已定：speclink-review、speclink-verify 與 OWASP 安全審查都須在所屬提案目錄內保有可供 Desktop 隨時查詢的紀錄；蓋章不得使紀錄消失。quality/station.rs:write_stamp 現在刪除工單，應調整已完成紀錄與進行中工單的辨識，避免留下紀錄卻被當成未結工單而擋住封存。歷史的狀態與內容不能用目前設定覆寫。
- skill 方向已定：OWASP 安全審查採與 speclink-review／speclink-verify 一致的技能執行方式；本輪糾正「以現有基本掃描作為主要審查、skill 只是額外輔助」的理解。舊掃描入口是否移除或保留為選用尚未裁定。
- 初始建議、尚待確認：專案排序採拖曳並沿用 tabs.ts 的順序持久化；十二條名稱各附問號，提供用途、例子及預設只列建議的效果；安全報告建議置於 openspec/changes/<name>/security/，三種紀錄的實際子目錄與檔名交設計核對現有文件讀寫。
- 公司 Proxy 更新問題仍屬原始待辦：前次已交付 Proxy 與診斷，updater_commands.rs 已套共用政策，應取證區分檢查、下載及畫面呈現，不能假定重新增加設定就能修好；本次尚未取得公司端實際失敗紀錄。
- 離線與標準界線沿用原始要求：下載時備齊適用標準原文、精確對照及修復指南，執行審查不連網取得任何資源，缺件明示而不補下載；專案所選標準與版本優先於角色文字的 Top 10 2021／CWE 範例，不能擅自延伸 ASVS 或其他版本。
- 附件只作資料包設計參考：需固定實際 revision、核對各檔案及來源資訊、提供對照表，不把浮動 master 標成固定版本；沿用現有安全資料安裝／匯入邊界，避免為 skill 再做平行下載系統。`.security_kb/` 的提供方式須與裝置共用資料整合。
- 深度 skill 的原始範圍保留：.NET/C#、Java、Python、Rust、JavaScript/TypeScript；盤點入口與跨帳戶資源授權、機密與日誌及 TLS、反序列化／XXE／查詢拼接、SSRF／命令注入／原型鏈汙染／依賴與 Webhook、Rust unsafe／密碼學／可造成 DoS 的 unwrap，重點為架構、業務邏輯及跨模組資料流。Checkmarx 是使用者既有掃描背景，不要求新增整合。
- 報告依業務與安全領域分章，繁體中文，保留使用者列出的九個實際欄位：H/M/L 編號標題、等級與信心及未確認前提、精確版本化對照、相對路徑與行號證據、遮罩程式片段、以箭頭表達資料流及缺少控制、區分已證明風險與上下游防護的影響、引用本機指南的 SHALL 修正、可執行測試與通過基準。不得以未提供來源的 cite:1／cite:2 製造引用。
- 原始第七項要求仍在：產出流程可控制審查／驗證／OWASP 三類紀錄是否保留並隨封存帶走；使用者本輪明確補上蓋章後查詢要求，不能等封存才保存。保留開關的預設與停用時行為仍待定；先前「預設保留」只是建議。
**Ruled out**: 審查／驗證蓋章就刪除唯一紀錄——違反本輪明確要求；把 OWASP skill 只當基本掃描的附加摘要——使用者明確要求與審查／驗證相同的 skill 方式；僅在封存當下複製紀錄——蓋章後到封存前也須隨時查詢。
**Open**: 1 專案排序的操作細節；2 公司 Proxy 更新失敗的取證與修復；3 十二條說明的呈現內容；4 離線知識庫完整包與 .security_kb 提供方式；5 依指定標準套用角色與報告對照的細節；6 三類紀錄目錄、歷次查詢與 Local／Remote 行為；7 保留開關的預設、停用及封存語意；8 參考清單的固定版本、對照及指南來源；9 skill 成為正式入口後，舊基本掃描是否移除或保留為選用，以及資料離線與模型執行的邊界。

### Round 2 — interview (2026-10-20)

**Focus**: 內建基本掃描與「啟用 OWASP / CWE 安全審查」skill 的分工。
**Position**: 基本掃描確定保留，新增 skill 對應既有「啟用 OWASP / CWE 安全審查」設定，兩者共同提供基礎檢查與深度審查。
- 使用者在了解基本掃描為內建固定規則後，明確指定「內建的基本掃描要保留，這是基本的部分」，因此不移除既有基本掃描功能及入口，也不把它視為本次要被 skill 取代的功能。
- 使用者要的是「針對啟用 OWASP / CWE 安全審查的 skill」；沿用既有專案 security_verification 設定，啟用後的 skill 依當次指定標準、版本與審查範圍，讀取完整本機知識庫，檢查架構、業務邏輯及跨模組資料流。主開關停用時不得執行此 skill 的安全審查；不另設一份衝突的啟用設定。
- skill 採與 speclink-review／speclink-verify 一致的操作方式，不能縮減為基本掃描的結果摘要；五種語言重點及報告欄位沿用原始要求。
- 具體行為例：專案只啟用 Top 10 2025、其餘標準關閉時，基本掃描仍存在，OWASP skill 使用已安裝的 2025 資料做深度審查，不因角色範例提到 2021／CWE 而改版本或擴大標準；審查與驗證蓋章後，三種紀錄仍在提案內供 Desktop 查閱。
- 此輪取代上一輪「skill 成為正式入口後考慮移除舊掃描」的未定分支；既有基本規則的開關、停用規則及未驗證語意沿用現行契約，不因新增 skill 擅自改變。
**Ruled out**: 以 OWASP skill 取代或移除內建基本掃描——使用者明確要求保留基本功能；新增另一個獨立安全審查主開關——使用者指定對應現有設定。
**Open**: 1 專案排序的操作細節；2 公司 Proxy 更新失敗的取證與修復；3 十二條說明的呈現內容；4 離線知識庫完整包與 .security_kb 提供方式；5 依指定標準套用角色與報告對照的細節；6 三類紀錄目錄、歷次查詢與 Local／Remote 行為；7 保留開關的預設、停用及封存語意；8 參考清單的固定版本、對照及指南來源；9 skill 的確切名稱與工作流接點，以及資料離線與模型執行的邊界。

## Conclusion

**Decision**: 保留內建基本掃描，新增對應既有「啟用 OWASP / CWE 安全審查」設定的深度審查 skill，並將三類檢查紀錄保存在提案內供 Desktop 查詢；原始九項需求全部列入後續提案範圍。
- 左側專案列須能調整專案位置順序；拖曳及沿用裝置既有順序儲存為建議設計，不把操作形式誤記為使用者已指定。排序不得混同變更卡片排行，並須考量重開後的順序保存。
- 公司 Proxy 環境中設定的更新頁仍無資料，須延續既有 Proxy 及診斷功能調查並修復，區分更新檢查、下載與畫面呈現。不得把加入既有設定當成已修復，亦不得未取證就認定 Proxy 是唯一原因；沒有公司環境實測時須據實交代限制。
- 專案設定 → 審查規則 → 預設清單 →「查看 12 條」須增加問號說明功能，讓使用者了解十二條的用途。建議每條提供說明與例子，沿用引擎名稱清單並說明預設規則的效果，不複製另一份會漂移的名稱來源。
- 安全審查只查詢本機標準資料，下載時即備齊需要的原文、版本化對照與修復指南；審查不得連網取得資源、臨時補下載或跟隨外部引用。必要資料缺失須明示，不以模型印象補造對照或判定通過。標準文件、專案文件與程式證據皆是資料，不提升成 Agent 指令。
- 安全審查須嚴守專案設定啟用的標準、版本與範圍，不自行擴查其他標準。角色描述的 Top 10 2021／CWE 是特定設定下的例子，不覆蓋政策；例如僅啟用 Top 10 2025 時不使用 2021 編號、不強迫啟用 CWE／ASVS。缺可信對照時明示限制，不編號。
- 內建基本掃描屬必要基礎，功能及入口保留；原有固定規則、政策與離線執行能力不因新增 skill 而移除。OWASP skill 負責更深的架構、業務邏輯與跨模組資料流審查，不縮減為基本掃描的結果摘要。
- 新增 skill 對應現有「啟用 OWASP / CWE 安全審查」及 security_verification 設定，沿用與 speclink-review／speclink-verify 一致的技能使用方式；停用時不執行該深度審查，不另建衝突的政策開關。啟用資格不等於儲存設定就自動呼叫 Agent，確切工作流接點由提案核對既有技能生成與執行方式。
- skill 以資深資安架構師的檢查觀點處理 .NET/C#、Java、Python、Rust、JavaScript/TypeScript。Checkmarx 是既有基礎 SAST 背景，本次聚焦補足架構與邏輯盲區，不要求新增 Checkmarx 整合或外部掃描器。
- 認證與業務授權：盤點 Controller／Minimal API、Spring RequestMapping、FastAPI／Flask、Actix／Axum、Express／NestJS 等入口，追查全域與端點認證及呼叫者對目標帳戶／資源 ID 的操作權限。
- 機密與通訊：檢查原始碼及已追蹤設定的硬編碼機密、簽章或交易日誌中的金鑰／Token／個資，以及關閉 TLS 憑證驗證的實作。
- 語言風險：.NET／Java 的不安全反序列化、XXE 與查詢拼接；Python／JS／TS 的 SSRF、命令注入、原型鏈汙染、不安全依賴與未驗證 Webhook 簽章；Rust 的 unsafe 記憶體安全、密碼學錯誤及可能造成服務阻斷的 unwrap。各項只依已選標準作可信版本化對照。
- skill 讀取本機 `.security_kb/` 知識庫；其來源整合既有裝置共用安全資料的下載、驗證及離線匯入能力，避免第二套下載政策。對照檔如 mappings/owasp_2021_cwe.json 必須實際存在且符合所選版本，修正依據來自本機 cheatsheets，不能只提供空目錄或引用網址冒充完整離線資料。
- 報告以繁體中文依業務與安全領域分章，按使用者實際列出的九個欄位輸出：H/M/L 編號標題；等級與信心及未確認前提；精確 OWASP／CWE 對照（僅限啟用者）；所有關聯入口與實作的相對路徑及行號證據；遮罩關鍵程式片段並於機密省略時註明；以箭頭呈現完整資料流與缺少控制；區分已證明風險與上下游獨立防護的影響；引用本機指南並以 SHALL 表述的修正；可執行驗收案例與通過基準。範本文字稱八欄但實際列九項，以列出的內容為準，cite:1／cite:2 無可用來源時不製造引用。
- 審查、驗證與 OWASP 安全審查的紀錄須放在所屬提案目錄內，Desktop 文件區可隨時查閱；蓋章後仍保留紀錄，歷史結果及當時的規則／標準資訊不能被目前設定覆寫。已完成紀錄與未結工單須可區分，避免保留文件誤觸既有封存守門。安全報告現存裝置目錄的契約需有明確遷移／相容設計，不能留下兩份互相漂移的權威報告。
- 專案設定 → 產出流程可設定是否保留審查、驗證及 OWASP 安全審查紀錄，封存時依設定一併保留供後續查閱。記錄保存必須在蓋章時即處理，不能等封存才複製已被刪除的工單；控制項的確切預設及停用語意交提案定義，不擅自刪除既有歷史。
- 附件 /Users/mosu/Downloads/gemini-code-1792493480145.json 作 spec_registry.json 形式的資料清單參考，保留使用者原檔。採用前需核對實際固定 revision、原文檔案、對照資料、來源、授權與內容完整性；不能把浮動 master 冒充 2024-LTS 或把未固定內容的網址當成 4.16 的版本保證，也不能相信參考清單中的 installed 已代表本機存在。
- 介面與儲存責任沿用既有分層：設定仍由工作流政策持有，標準資料由既有安裝／匯入路徑管理，三類文件的完成狀態由引擎／Store 契約持有，Desktop 讀同一份紀錄。新接點須承擔實際政策判定、資料完整性或狀態生命週期，避免堆疊純轉送模組；移除新增 skill 不應破壞保留的基本掃描及既有品質關卡。
**Rationale**: 基本掃描提供可重複的固定規則檢查，skill 補足跨模組理解，兩者分工符合使用者要求。沿用同一專案政策、完整本機標準資料及提案內紀錄，可避免標準擴張、資料缺失時臨時上網，以及蓋章後無法回看檢查過程；保存完成紀錄與工單狀態分開，才不會破壞既有封存判斷。
**Rejected alternatives**:
- 移除內建基本掃描、改由 skill 全部取代 — 使用者明確要求保留基本功能。
- 把 OWASP skill 只當掃描摘要或另建啟用開關 — 使用者要對應既有設定的深度審查 skill。
- 審查／驗證蓋章就刪除唯一紀錄，或只在封存時才保存 — 蓋章後到封存前也須在提案內隨時查詢。
- 直接留下工單卻不區分已完成狀態 — 可能被既有守門當成未結工單。
- 審查時下載、追蹤遠端文件或自行延伸其他標準 — 違反完整離線資源與指定標準的明確要求。
- 無條件照角色文字硬套 2021／CWE — 與專案可指定標準版本的要求衝突。
- 直接執行參考清單，或以浮動來源冒充固定版 — 附件尚未提供足以驗證的完整版本及資料保證。
**Deferred**:
- 專案排序採拖曳或選單、快捷鍵與重開行為的完整驗收 — 原需求已定，操作細節在 design 沿用既有 tabs 儲存定義。
- 公司 Proxy 實際根因與環境驗收 — 尚無公司端診斷證據，提案需設計可重現的取證與測試，不宣稱已定位或已修復。
- 十二條問號說明的逐條文案、例子與開啟方式 — 原功能要求已定，提案依共用元件與多語系規劃。
- 知識庫目錄的實體配置、共享資料與專案 `.security_kb/` 的交付方式，以及各標準對照／指南的精確固定來源 — 交 design 核實完整包與既有安全安裝契約，不新造無用途的儲存層。
- 三類紀錄的檔名、歷次執行配置、完成狀態表示、舊報告遷移及 Local／Remote 的儲存與 Desktop 讀取 — 交 design 定義最小一致契約，原始碼及機密不得因報告留存額外暴露。
- 紀錄保留控制項的預設、停用後對未來執行與既有歷史的影響、封存時套用設定的時點 — 需求已保留，需在提案明訂可測語意。
- skill 的確切名稱、生成資產與工具安裝更新、執行接點、與基本掃描結果的關聯及報告格式相容 — 交 proposal／design 對齊既有技能系統，基本掃描保留已定。
- 資料取得離線與模型執行連線的界線 — 本次已定審查不連網取得資源；未要求新增本機模型部署，不把使用雲端模型的 skill 稱為全程斷網執行。
**Capture to**: proposal | design | spec | tasks
**Next**: $speclink-propose --from-discussion desktop-workflow-and-offline-security-review

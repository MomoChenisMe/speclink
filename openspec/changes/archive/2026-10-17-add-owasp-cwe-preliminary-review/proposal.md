## Why

透過 AI 代理跑 SDD 的開發者、PO 與 PM，需要在設計、實作及驗證時看見可追查的初步安全風險；目前 audit 是 AI 的檢查指引，沒有無 AI 也能執行的離線規則或版本化標準資料。本變更重用現有政策、Remote 授權與設定介面，提供「OWASP / CWE 初步安全審查」（OWASP / CWE Preliminary Security Review），將疑似問題與未驗證範圍說清楚，不以誤判阻擋封存。

## What Changes

- 在產出流程頁的「產出政策」卡、audit 下方增加 security_verification，預設 enabled=false；含 off／web-api／api／web 單選範圍、三種獨立審查、排除目錄及個別規則停用。保留 locale、spec_locale、tdd、audit、worktree 的既有行為。
- 獨立選用 Top 10 2021／2025、API Top 10 2019／2023、ASVS 4.0.3／5.0.0 與 L1／L2／L3、動態已安裝 CWE 版本；允許其他標準全部關閉。所有標準關閉時標示未配置，自有基本規則結果獨立呈現。
- 應用程式設定新增「安全標準資料」：官方版本清單、明確觸發的更新／下載、進度／取消、重裝、刪除未使用版本、離線匯入與來源授權。資料放共用的應用程式資料目錄，限量驗證及原子安裝，失敗保留舊版。
- 自有 Rust 規則先支援 Rust、TypeScript、JavaScript，檢查疑似憑證、SQL／命令拼接、不安全 HTML、詳細錯誤輸出、不安全設定，並提供 SDD 安全要求與 API 靜態邊界審查。無 AI、無第三方掃描器也能執行；不執行專案程式、不自動下載、不上傳原始碼。
- 提供遮罩的 JSON／Markdown 報告與 Desktop 閱讀入口，保留版本、規則、來源、覆蓋與失敗原因；區分 PASS／FINDING／NOT_VERIFIED／NOT_APPLICABLE／DISABLED，明示 PASS 的有限意義。
- Propose／Design 帶入適用要求，Apply 保留 TDD／audit，Verify 顯示報告資訊但不將疑似發現轉為既有必修工單；Archive 的既有守門不變。claude、codex、copilot 與自訂工具的生成技能同步更新，AI 輔助只在使用者另行選用時提供遮罩摘要。
- Local 與 Remote 共用政策解析／文字改寫；Remote 維持角色與 revision 衝突處理。掃描只讀授權的本機 checkout；Remote 的政策與規格以明確準備操作建立離線快照，純 Remote 無 checkout 時回未驗證。

### 相容性影響

- 新增 security_verification 區塊，舊檔缺席視為 disabled，保留未知鍵、註解與既有政策。既有 workflow-config set 的五鍵不擴張；新增 workflow-config security --stdin [--dry-run]，讀取單一 YAML mapping 的部分更新，成功 0、非法輸入／壞設定／Remote 衝突 1，缺 --stdin 由 CLI 拒絕；錯誤至 stderr，diff／成功摘要至 stdout。
- workflow-config show --json 刻意新增 securityVerification 物件；人眼輸出於 review 行後增加 security_verification 摘要，其他行保持。instructions 的適用階段新增有效 securityVerification；停用時既有行為維持，必要 JSON／golden 同批更新。既有使用者無需遷移設定。
- 新增 security 動詞族：run [--change NAME] [--json]、prepare [--change NAME] [--json]、report ID [--format json|markdown]、rules [--json]；run 及 prepare 接受 --root PATH 明確指定本機目標，report／rules 不讀 stdin。run 退出碼 0 表示已完整產報告（可含疑似問題），1 表示執行失敗或覆蓋不完整（可仍有報告），取消為 130；不以疑似風險數作阻擋。
- security data 提供 list [--json]、refresh [--json]、install STANDARD --version VERSION [--reinstall] [--json]、import PATH [--json]、remove STANDARD --version VERSION [--json]。不讀 stdin，支援全域 --no-color；成功 0、非法／失敗 1、取消 130。只有 refresh／install 連官方來源，終端以 Ctrl+C 取消；進度至 stderr，JSON 單一 payload 至 stdout。各 JSON shape 在規格中列出。

## Capabilities

### New Capabilities

- `security-review`: 離線目標取得、可取消且有資源限制的自有規則、SDD／API 邊界檢查與執行命令。最近的 workflow-config、verify-skill 只管理政策或 AI 驗證流程，沒有確定性規則與授權檔案掃描契約。
- `security-standard-data`: 官方標準版本、來源授權、受限下載／匯入、原子安裝與離線使用。desktop-config 與 server-policy-write 管設定介面和政策文件，不承載裝置共用標準資料的生命週期。
- `security-review-report`: 遮罩報告模型、JSON／Markdown、Desktop 閱讀與提示性的工作流整合。verify-station 的工單及章具有封存守門語意，不能代替不阻擋的初步安全報告。

### Modified Capabilities

- `desktop-app`: 裝置層專案圖示色票及主視窗／系統匣一致呈現。
- `ui-copy-vocabulary`: 繁中與系統繁中統一台灣用語「預設」。

- `workflow-config`: 安全區塊預設／驗證／保值、show 輸出、security 部分更新動詞與有效政策 payload。
- `desktop-config`: 產出政策安全子設定與系統層資料管理入口，沿用既有元件及 Local／Remote 儲存。
- `server-policy-write`: 安全政策值域在 server 驗證並維持既有角色／CAS 契約。
- `verify-skill`: 在既有驗證報告加入獨立安全資訊，不寫成必修發現或新封存條件。

## Impact

- 規格掃描：相關正式規格為 workflow-config、desktop-config、server-policy-write、verify-skill、verify-station；新能力分界如上，verify-station 的工單與封存契約不修改。
- Affected code:
  - New：`crates/engine/speclink-core/src/security/mod.rs`、`crates/engine/speclink-core/src/security/policy.rs`、`crates/engine/speclink-core/src/security/standards.rs`、`crates/engine/speclink-core/src/security/rules.rs`、`crates/engine/speclink-core/src/security/report.rs`、`crates/engine/speclink-core/assets/security/rules.json`、`crates/engine/speclink-core/assets/security/sources.json`。
  - New：`crates/engine/speclink-fs/src/security.rs`、`crates/host/speclink-host/src/security.rs`、`crates/host/speclink-host/src/security_data.rs`、`crates/adapters/speclink-cli/src/verbs/security.rs`、`apps/desktop/core/src/security.rs`、`apps/desktop/src/adapter/security.ts`、`apps/desktop/src/views/settings/SecurityDataPage.tsx`、`packages/ui/src/components/SecurityReviewPanel.tsx`、`docs/security-review.zh-TW.md`、`docs/security-review.md`。
  - Modified：`crates/engine/speclink-core/src/lib.rs`、`crates/engine/speclink-core/src/workspace/config.rs`、`crates/engine/speclink-core/src/workspace/instructions.rs`、`crates/engine/speclink-core/src/workspace/skills.rs`、`crates/engine/speclink-fs/src/lib.rs`、`crates/host/speclink-host/src/lib.rs`、`crates/host/speclink-host/src/context.rs`、`crates/host/speclink-host/src/policy.rs`、`crates/protocol/speclink-protocol/src/query.rs`、`crates/host/speclink-server/src`、`crates/adapters/speclink-cli/src/main.rs`、`crates/adapters/speclink-cli/src/verbs/mod.rs`、`crates/adapters/speclink-cli/src/verbs/config.rs`、`crates/engine/speclink-core/src/workspace/init.rs`。
  - Modified：`apps/desktop/core/src/lib.rs`、`apps/desktop/core/src/settings.rs`、`apps/desktop/src-tauri/src/lib.rs`、`apps/desktop/src-tauri/src/remote.rs`、`apps/desktop/src/adapter/workspace.ts`、`apps/desktop/src/settingsCatalog.ts`、`apps/desktop/src/views/AppSettingsView.tsx`、`apps/desktop/src/views/project-settings/WorkflowPage.tsx`、`apps/desktop/src/views/project-settings/useProjectSettings.ts`、`packages/ui/src/adapter.ts`、`packages/ui/src/index.ts`、`packages/ui/src/components/ChangeDetailPage.tsx`、`packages/ui/src/i18n.tsx`、`apps/desktop/src/i18n/messages.ts`。
  - Modified：`Cargo.lock`、`crates/engine/speclink-core/Cargo.toml`、`crates/engine/speclink-fs/Cargo.toml`、`crates/host/speclink-host/Cargo.toml`、`crates/adapters/speclink-cli/Cargo.toml`、`crates/engine/speclink-core/assets/skills`、`crates/engine/speclink-core/tests/golden`、`crates/engine/speclink-core/tests/golden/assets.lock`、`.agents/skills`、`.claude/skills`、`.github/skills`、`openspec/LANGUAGE.md`、`docs/configuration.md`、`docs/configuration.zh-TW.md` 與各受影響 crate／app 的測試目錄。生成足跡只更新既有工具目標。
  - Removed：無。一般依賴沿用 lockfile 已有的 ureq 2、quick-xml、zip、libc 與 windows-sys；CLI 取消新增小型 ctrlc 3.5 系列，已核對官方 MIT／Apache-2.0 授權與最低 Rust 1.69；不安裝掃描器，不新增前端框架或外部 runtime。授權清單由既有 desktop third-party-licenses 流程產生。



## 實機回饋補充

依使用者下載畫面與專案列回饋，補上版本列「下載中」回饋、可持久保存的專案圖示色票，以及繁中台灣用語約定。專案圖示顏色是裝置層視覺偏好，Local／Remote 依既有 locator 分開保存，不改團隊政策。

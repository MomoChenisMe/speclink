# SpecLink Fork 0.9 升級報告

日期：2026-10-15 至 2026-10-16（Asia/Taipei）。

**程式整合與本機自動驗證完成；Windows 發行驗收及升級後的實機操作驗收尚待完成，不能宣稱跨平台發行全部通過。** 本次只建立本機升級分支的合併提交，不 push、不修改 main、不發布 Release。

## 版本與安全保護

| 項目 | 結果 |
|---|---|
| 官方正式版 | [v0.9.0](https://github.com/MomoChenisMe/speclink/releases/tag/v0.9.0)，`b2c96f548db08d9d16ea34acbc3dd848073224d8`；annotated tag 物件為 `f7608ce4fd04d04f1b3d4f2e2400fc895bafb574`，已以遠端 peeled ref 再確認；GitHub Release 非 draft／prerelease |
| Fork 升級前 | `f9d307138ab8638198d0c34a354360df34fad464` |
| 共同祖先 | `eb44f950e96e8023239bdfab145fa487062c99cf` |
| 備份分支 | `backup/pre-upstream-0.9-20261015-f9d30713` |
| 升級分支 | `upgrade/upstream-0.9-20261015` |
| Origin | `https://github.com/mosutw/speclink.git` |
| Upstream | `https://github.com/MomoChenisMe/speclink.git` |

起始工作區乾淨，沒有未提交或未追蹤內容。先建立備份，再 fetch upstream branches/tags，確認實際 tag 後合併。沒有 hard reset、force push、清除未追蹤檔或刪除分支。main 與備份維持升級前 SHA。`.speclink.yaml` 的三工具設定逐位元保留。遠端只有 fetch，沒有 push、PR、遠端 workflow 觸發或 Release 操作。

## 完整盤點與功能決策

以共同祖先盤點的 14 筆 Fork 專屬提交、164 個變動路徑及 18 類客製功能，見 [FORK_CUSTOMIZATION_INVENTORY.md](FORK_CUSTOMIZATION_INVENTORY.md)。每個檔案均有提交證據，不以 commit message 代替程式分析。逐項處理方式及必要行為見 [UPGRADE_FEATURE_MATRIX.md](UPGRADE_FEATURE_MATRIX.md)。

| ID | 決策 | 整合結果 |
|---|---|---|
| C01–C03 | REPLACE | 採官方 Copilot Tool、SyncPlan、SDK renderer；保留 init/update/adopt/checkout/清理/過期守門功能。Codex 使用 `.agents/skills` 與 `$speclink-`，Copilot 使用 `.github/skills` 與 `/speclink-`，不保留舊共享 renderer。 |
| C04 | MERGE | 保留主機 OS 語言預設、明示與環境優先順序、Remote 使用 Server 語言；合併官方 review rules。 |
| C05 | KEEP | 保留 workflow-config languages 只讀查詢與可選 Server metadata；舊 Server 缺資料會明示要求升級。 |
| C06 | MERGE | 官方 v1.48.0 新技能為底，只加入語言差異；產物版本 v1.48.1，golden、assets.lock、三工具技能同源重新生成。 |
| C07–C08 | MERGE | 官方 Light/Dark/System、設定分類、色票與首次繪製機制保留；讀取舊偏好鍵。依使用者確認，保存失敗不換色並提示重試。 |
| C09 | ADAPT | 保留官方 Windows/macOS 面板與原生 setTheme；接回先訂閱後 ready、首份快照前不呈現、卸載清理、workspace 不變仍推送偏好及尺寸量測。 |
| C10 | MERGE | 失敗 toast 與主視窗同步偏好；保留官方色票，補回原生控制項的 color-scheme；Server Web 仍跟隨系統。 |
| C11–C12 | ADAPT／KEEP | 新版五種專案設定分類共用唯讀目錄；支援 local/remote checkout/null，保留完整路徑及連續空白、Tab、換行。 |
| C13–C14 | ADAPT | Portable marker、data/WebView/登入隔離、CLI token、停用安裝器更新全部保留；新 Windows tray WebView 也指定 Portable data directory。 |
| C15 | ADAPT | 保留手動 Desktop Build 四平台、同源 CLI、ad-hoc 簽章；官方新平台 config 與正式 release workflow 一併納入。 |
| C16 | ADAPT + REVIEW | Portable staging 與 Actions ZIP 流程保留；依使用者確認不加入正式 Release assets。Windows 真正打包／啟動仍待 Runner。 |
| C17 | KEEP | Claude skill 分組檢查繼續以核心 registry 為正典，不依賴本機 Claude 安裝選集。 |
| C18 | MERGE | 三工具設定與客製封存歷史保留；正式規格採官方新需求加客製差異，文件修正為原生 Copilot 與系統語言語意。 |

實際遷移另使用升級前 `f9d30713` 的真正 Copilot 受管技能檔執行新版 update：舊 `.agents/skills/speclink-propose` 被清理，新 `.github/skills/speclink-propose` 正確生成，使用者 my-skill 與兩份設定原文均保留。不是只用新版 init 自我比對。

## 主要變更與語意衝突

- 官方 0.9 的新版 UI、分類式設定、文件閱讀介面、review rules、原生 Copilot、Windows tray panel、平台 Tauri config、發行流程均納入。
- 外觀整合集中於 `packages/ui/src/theme.ts`、`theme.css`、Desktop `index.html`、`App.tsx`、`GeneralPage.tsx`、`tray.ts`、`panel/main.tsx`。舊 Desktop theme 控制器與其測試已移轉到官方 API 的測試後移除，避免兩套偏好狀態。
- 目錄展示位於 `ProjectSettingsView.tsx`，由 active locator 傳入，不新增磁碟／Server 查詢或目錄寫入。
- 語言保留於 Core config/init、Host context、CLI workflow-config、Server config metadata、共享技能來源；與官方新政策欄位相容。
- Portable 的既有 Host/Remote/CLI/Tauri 實作、打包腳本及手動 workflow 保留。`src-tauri/src/panel.rs` 是必要的新架構適配：不能只處理初始 main WebView。
- 型別檢查在官方原版已重現錯誤。本次補齊測試 fixture 的新欄位、修正 JSX 對 never 的不合法展開為實際元件 props、錯誤的型別 import 與遺留 badge；安裝 `@types/node`、`@types/jest-axe`，UI/Server Web 宣告現有測試所用 ES2022 API。編譯 target 仍 ES2020，現有依賴版本沒有改動。Desktop 的 Array.at 改為等效索引，符合其 ES2020 lib。
- `.github/workflows/release.yml` 與官方內容一致，沒有把 Portable 加入正式 Release。手動 Desktop Build 的額外 config 只改 updater/signing，不覆蓋官方平台視窗設定。

89 個 Git 衝突路徑的逐檔解法附於本報告末尾。其他文字可自動合併的檔案也檢查了介面與行為；特別是新面板的 Portable WebView、語言提示、舊 Copilot 同文假設及新 review 欄位，均屬 Git 不會自動替人修正的問題。

## 實際驗證結果

主機為 macOS（Darwin），Node 22.14.0、Rust 1.99.0。以下都是本次整合後實際執行，未沿用舊版測試結果。

| 檢查 | 結果 | 本機紀錄（未提交的診斷檔） |
|---|---|---|
| Rust cargo check：Core/Host/Remote/CLI | 通過 | `.speclink/upgrade-0.9/rust-check.log` |
| Rust cargo test --workspace --offline | 2,966 通過、0 失敗、2 個既有輔助程序 ignored；36 組含 doc-tests | `workspace-verified.log` |
| UI Vitest | 1,320 通過 | `ui-final.log` |
| Desktop Vitest | 1,222 通過 | `desktop-final.log` |
| Server Web Vitest | 244 通過 | `server-web-retest.log` |
| Node scripts | 364 通過，含技能分組、文件、同源 sidecar、Portable staging、發行資產守門 | `script-final.log` |
| Node SDK | npm ci／原生 binding build 通過，52 項測試通過 | `node-ci.log`、`node-build.log`、`node-tests.log` |
| TypeScript | Desktop、UI、Server Web 三者 tsc --noEmit 均通過，包含測試檔 | `desktop-tsc-verified.log`、`ui-tsc-verified.log`、`server-web-tsc-check.log` |
| Vite production build | Desktop、Server Web 通過；大 chunk 警告保留 | `desktop-build-final.log`、`server-web-build.log` |
| Tauri 桌面建置 | 同源 debug sidecar、完整原生編譯及 macOS `.app` 打包成功，ad-hoc 簽章及 codesign --verify --deep --strict 通過；內含 CLI 回報 0.9.0／engine v1.48.1；無正式公證 | `sidecar-build.log`、`tauri-build-final.log` |
| Copilot 舊版遷移 | 真正舊技能檔 → 官方原生目錄；設定不變，自訂技能保留 | `copilot-migration-smoke.json` |
| Workflow | 五份 workflow YAML 解析通過，364 scripts 的發行守門通過；未安裝 actionlint，未執行遠端 Actions | `workflow-yaml.log`、`script-final.log` |
| Git | 無衝突標記；相對官方的 git diff v0.9.0 --check 通過；main／備份未移動 | 本機 Git 狀態 |

Rust 初跑時沙箱禁止測試連線埠，後改為經核准執行；暫存放在專案內又使「非專案目錄」案例找到父專案，最後經核准改用 `~/.cache/speclink-upgrade-0.9/tmp` 完整重跑並通過。為處理暫存路徑而暫改的兩個測試已恢復官方原文。沒有把環境造成的初次失敗當作成功，也沒有為過測試去停用測試。

UI 測試覆蓋主題矩陣、舊鍵遷移／新鍵優先、保存失敗與重試、面板延遲訂閱／卸載及首次快照、目錄各分類與特殊空白、toast 偏好與 Server Web 的既有行為。這些是自動測試，不能代替實機觀察玻璃材質、系統對話框或真正選取複製的結果。

本機可檢視的打包產物：`target/debug/bundle/macos/Speclink.app`。這是本機架構 debug app，不能當作 macOS universal／DMG 或正式簽章發行驗證。

相對 Fork 的 `git diff --cached --check` 另列出 6 個官方既有的檔尾空行（3 個技能來源、2 個官方封存規格、1 個 UI 測試）；已逐一比對官方 tag，保留原文，未混入本次新增的空白問題。

## Portable 與 Release 驗證界線

官方 v0.9.0 GitHub Release 的六項資產為 updater manifest、兩種 Linux AppImage、macOS universal DMG、universal app tarball、Windows x64 setup EXE，沒有 Portable ZIP。本次保留這個邊界。

| 項目 | 已驗證 | 尚待驗證 |
|---|---|---|
| staging 腳本 | distinct GUI/CLI 名稱、marker、資源對應、LICENSE/README/build-info、不帶個人 data、不混入 PDB、缺 sidecar 失敗、重建清掉舊產物且不動安裝檔 | 真正 Windows PE 執行檔與 WebView2 |
| 手動 workflow | YAML、四平台參數、同源 sidecar、靜態 CRT、artifact 路徑、現有 PowerShell 解壓與 CLI 寫入／搬移 smoke 保留 | Windows Runner 實際 NSIS＋ZIP 產出、Actions artifact 上傳 |
| Portable 執行 | Host/Remote/CLI 的純邏輯、設定位置與登入邊界經 Rust 測試；新 tray WebView 的路徑已適配 | Windows GUI 啟動、安裝版共存、WebView data 隔離、唯讀目錄、搬移、無 marker、重新啟動與更新入口 |
| 發行資產 | 官方已發布資產清單已讀取；正式 workflow 靜態守門通過 | 本 Fork 尚未執行 Actions，上傳 ZIP 與正式發行都未做 |

不能因 workflow 沒有文字衝突或 staging fixture 通過，就宣稱 Portable 能正常發行。

## 未解決風險與下一步

1. **Windows Runner 與實機驗收待辦**：使用者確認可 push 後，才推送升級分支並執行手動 Desktop Build。下載 NSIS 與 Portable artifact，依 `docs/development/windows-portable.md` 驗收上述 Windows 行為。未經確認不觸發遠端工作。
2. **升級後桌面實機驗收待辦**：用本機 `.app` 檢查 Light/Dark/System、重開記憶、目錄真正選取複製、macOS 材質／對話框。Windows tray 同樣須在 Windows 上操作。本次尚未實際手動完成，不沿用 0.8 的既有驗收。
3. **依賴掃描**：npm audit 實際回報 10 項（2 moderate、6 high、2 critical）；Node SDK 安裝另回報 6 項。根專案與官方鎖檔既有套件的版本逐項比對無變動，並非本次為修復而升級執行期依賴。critical 涉及 vitest/tinypool 等測試工具，high 包含 react-router 等；需要另行評估受影響用途及更新。沒有執行 audit fix --force。Vitest 建議修復跨主版本，應獨立處理相容測試後再採用，不能把 audit 數字當作可直接發行的安全保證。原始診斷見 `npm-audit.json`；例如 [Vitest advisory](https://github.com/advisories/GHSA-82fw-gwwq-j7x9)、[tinypool advisory](https://github.com/advisories/GHSA-5gmw-xhrv-c9v3)。
4. **發行範圍**：本次未跑 Linux 實際打包、macOS universal/DMG、正式簽章／公證；沒有更新遠端版本。大 chunk 與官方既有 Rust unused import 警告仍存在，沒有擴大清理無關程式碼。
5. 以上發行與人工驗收完成後，再由使用者決定建立 PR、合併 main 及正式發布。本機測試通過不代表已取得這些遠端操作授權。

## 89 個衝突路徑的處理紀錄

| 路徑 | 解法 |
|---|---|
| `.agents/skills/speclink-apply-with-worktree/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.agents/skills/speclink-apply/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.agents/skills/speclink-archive/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.agents/skills/speclink-audit/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.agents/skills/speclink-baseline/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.agents/skills/speclink-commit/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.agents/skills/speclink-config/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.agents/skills/speclink-discuss/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.agents/skills/speclink-drift/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.agents/skills/speclink-improve/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.agents/skills/speclink-ingest/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.agents/skills/speclink-manual/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.agents/skills/speclink-propose/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.agents/skills/speclink-quality/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.agents/skills/speclink-review/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.agents/skills/speclink-trace/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.agents/skills/speclink-verify/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.agents/skills/speclink-worktree-merge/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-analyze/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-apply-with-worktree/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-apply/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-archive/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-audit/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-baseline/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-commit/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-config/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-discuss/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-drift/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-improve/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-ingest/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-manual/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-propose/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-quality/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-review/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-trace/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-verify/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.claude/skills/speclink-worktree-merge/SKILL.md` | 官方 v0.9 renderer ＋ Fork 語言差異，整合引擎重新生成 v1.48.1 |
| `.speclink.yaml` | 保留升級前原文；三工具選集不變 |
| `apps/desktop/src-tauri/src/lib.rs` | 合併官方原生功能與 Portable 啟動／命令；未整份接受任何一方 |
| `apps/desktop/src/App.tsx` | 官方 0.9 UI／色票／分類架構；移轉唯一客製行為及其測試 |
| `apps/desktop/src/__tests__/App.test.tsx` | 官方 0.9 UI／色票／分類架構；移轉唯一客製行為及其測試 |
| `apps/desktop/src/__tests__/appSettingsView.test.tsx` | 官方 0.9 UI／色票／分類架構；移轉唯一客製行為及其測試 |
| `apps/desktop/src/__tests__/assetUpdatePrompt.test.tsx` | 舊元件由官方 AssetNotice 取代；Copilot 原生選擇與更新測試保留 |
| `apps/desktop/src/__tests__/messages.test.ts` | 官方 0.9 UI／色票／分類架構；移轉唯一客製行為及其測試 |
| `apps/desktop/src/__tests__/projectSettingsView.test.tsx` | 官方 0.9 UI／色票／分類架構；移轉唯一客製行為及其測試 |
| `apps/desktop/src/__tests__/workspaceChooser.test.tsx` | 官方 0.9 UI／色票／分類架構；移轉唯一客製行為及其測試 |
| `apps/desktop/src/components/WorkspaceChooser.tsx` | 官方 0.9 UI／色票／分類架構；移轉唯一客製行為及其測試 |
| `apps/desktop/src/i18n/messages.ts` | 官方 0.9 UI／色票／分類架構；移轉唯一客製行為及其測試 |
| `apps/desktop/src/index.css` | 官方 0.9 UI／色票／分類架構；移轉唯一客製行為及其測試 |
| `apps/desktop/src/tray.ts` | 官方 0.9 UI／色票／分類架構；移轉唯一客製行為及其測試 |
| `apps/desktop/src/views/AppSettingsView.tsx` | 官方 0.9 UI／色票／分類架構；移轉唯一客製行為及其測試 |
| `apps/desktop/src/views/ProjectSettingsView.tsx` | 官方 0.9 UI／色票／分類架構；移轉唯一客製行為及其測試 |
| `crates/adapters/speclink-cli/src/verbs/init.rs` | 官方 Copilot／政策架構，保留 Fork 主機語言與相容驗證 |
| `crates/adapters/speclink-node/src/render.rs` | 官方 Copilot／政策架構，保留 Fork 主機語言與相容驗證 |
| `crates/engine/speclink-core/assets/skills/config.md` | 官方 Copilot／政策架構，保留 Fork 主機語言與相容驗證 |
| `crates/engine/speclink-core/assets/skills/ingest.md` | 官方 Copilot／政策架構，保留 Fork 主機語言與相容驗證 |
| `crates/engine/speclink-core/assets/skills/propose.md` | 官方 Copilot／政策架構，保留 Fork 主機語言與相容驗證 |
| `crates/engine/speclink-core/src/workspace/config.rs` | 官方 Copilot／政策架構，保留 Fork 主機語言與相容驗證 |
| `crates/engine/speclink-core/src/workspace/config/tests.rs` | 官方 Copilot／政策架構，保留 Fork 主機語言與相容驗證 |
| `crates/engine/speclink-core/src/workspace/init.rs` | 官方 Copilot／政策架構，保留 Fork 主機語言與相容驗證 |
| `crates/engine/speclink-core/src/workspace/init/tests.rs` | 官方 Copilot／政策架構，保留 Fork 主機語言與相容驗證 |
| `crates/engine/speclink-core/src/workspace/skills.rs` | 官方 Copilot／政策架構，保留 Fork 主機語言與相容驗證 |
| `crates/engine/speclink-core/tests/golden/assets.lock` | 由整合後來源重新生成，另以無更新旗標的測試驗證 |
| `crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md` | 由整合後來源重新生成，另以無更新旗標的測試驗證 |
| `crates/engine/speclink-core/tests/golden/claude.snapshot.md` | 由整合後來源重新生成，另以無更新旗標的測試驗證 |
| `crates/engine/speclink-core/tests/golden/codex.snapshot.md` | 由整合後來源重新生成，另以無更新旗標的測試驗證 |
| `crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md` | 由整合後來源重新生成，另以無更新旗標的測試驗證 |
| `crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md` | 由整合後來源重新生成，另以無更新旗標的測試驗證 |
| `docs/configuration.md` | 官方新文件為底；還原系統語言及只讀查詢說明，雙語錨點檢查 |
| `docs/configuration.zh-TW.md` | 官方新文件為底；還原系統語言及只讀查詢說明，雙語錨點檢查 |
| `docs/getting-started.md` | 官方新文件為底；還原系統語言及只讀查詢說明，雙語錨點檢查 |
| `docs/getting-started.zh-TW.md` | 官方新文件為底；還原系統語言及只讀查詢說明，雙語錨點檢查 |
| `docs/sdk-node.md` | 官方新文件為底；還原系統語言及只讀查詢說明，雙語錨點檢查 |
| `docs/sdk-node.zh-TW.md` | 官方新文件為底；還原系統語言及只讀查詢說明，雙語錨點檢查 |
| `openspec/manual/about.md` | 官方新文件為底；還原系統語言及只讀查詢說明，雙語錨點檢查 |
| `openspec/manual/baseline.md` | 官方新文件為底；還原系統語言及只讀查詢說明，雙語錨點檢查 |
| `openspec/manual/desktop-projects.md` | 官方新文件為底；還原系統語言及只讀查詢說明，雙語錨點檢查 |
| `openspec/manual/desktop-update.md` | 官方新文件為底；還原系統語言及只讀查詢說明，雙語錨點檢查 |
| `openspec/manual/index.md` | 官方新文件為底；還原系統語言及只讀查詢說明，雙語錨點檢查 |
| `openspec/manual/init-workspace.md` | 官方新文件為底；還原系統語言及只讀查詢說明，雙語錨點檢查 |
| `openspec/manual/policy-config.md` | 官方新文件為底；還原系統語言及只讀查詢說明，雙語錨點檢查 |
| `openspec/manual/worktree.md` | 官方新文件為底；還原系統語言及只讀查詢說明，雙語錨點檢查 |
| `openspec/specs/desktop-app/spec.md` | 保留官方全部需求，整合語言／外觀失敗／目錄／面板差異 |
| `openspec/specs/desktop-config/spec.md` | 保留官方全部需求，整合語言／外觀失敗／目錄／面板差異 |
| `openspec/specs/node-sdk/spec.md` | 保留官方全部需求，整合語言／外觀失敗／目錄／面板差異 |
| `openspec/specs/tray-status-menu/spec.md` | 保留官方全部需求，整合語言／外觀失敗／目錄／面板差異 |
| `openspec/specs/workflow-config/spec.md` | 保留官方全部需求，整合語言／外觀失敗／目錄／面板差異 |
| `openspec/specs/workspace-tools/spec.md` | 保留官方全部需求，整合語言／外觀失敗／目錄／面板差異 |
| `packages/ui/src/theme.css` | 官方 0.9 UI／色票／分類架構；移轉唯一客製行為及其測試 |

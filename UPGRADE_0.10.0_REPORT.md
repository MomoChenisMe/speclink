# Fork 與 Upstream 0.10.0 整合報告

狀態：**部分完成**。已完成備份、三方盤點、保留歷史的程式整合與本機測試；沒有未解決的 Git 衝突。非 Portable 安裝版的更新來源仍須人工決定，Windows／Linux、正式簽章與發布驗收尚未完成，因此不宣稱已可正式發布。

## A. 版本與 Git 環境

| 項目 | 結果 |
|---|---|
| Fork | https://github.com/mosutw/speclink.git |
| Upstream | https://github.com/MomoChenisMe/speclink.git |
| Fork 關係 | GitHub API 確認 parent 與 source 均為 MomoChenisMe/speclink |
| 初始分支／工作區 | main；乾淨，無未提交及未追蹤檔；原有 ignored 建置與資料未清除 |
| 整合前版本 | App／Cargo 0.9.0-build.m1，技能資產 v1.49.1 |
| 整合前 Commit | `1b2ca6ee279c49ed99e64208eba5f86ac14ac78e` |
| 最新正式 Release | [v0.10.0](https://github.com/MomoChenisMe/speclink/releases/tag/v0.10.0)，非 draft、非 prerelease；GitHub API 完整列舉正式 Release 後核對 |
| Release 發布時間 | 2026-10-17 00:06:07（Asia/Taipei） |
| 目標 Tag 解參照 Commit | `05e5e65ac043d04e6e0b2a396c0bb0997e52ae70`；此次與 fetched upstream/main 一致，但整合對象始終是 Tag |
| 共同祖先 | `b2c96f548db08d9d16ea34acbc3dd848073224d8`（v0.9.0） |
| 備份分支 | `backup/pre-upstream-sync-0.10.0`，指向整合前 Commit |
| 整合分支 | `upgrade/upstream-0.10.0` |
| Merge Commit | `f8471fab17e843803f79e238187739a5a401f4b5` |
| 整合版本 | App／14 個 Cargo workspace packages／Cargo.lock：0.10.0-build.m1；技能資產 v1.50.1 |
| 版本保留原則 | 升級官方基版，保留 Fork 的 build.m1 預先發行尾碼；沒有建立任何 Tag |

執行 git status、remote -v、branch -avv、tag --list、worktree list、rev-parse --is-shallow-repository、fetch upstream --tags、fetch origin，以及 git fsck --connectivity-only。初始只有一個 worktree，歷史非 shallow，fsck 成功；已有 dangling 物件屬未被參照物件，沒有缺失／損毀證據，未執行清理。備份 SHA 與 tree 皆已讀取核對，可由該分支還原已提交版本；原始 ignored 資料未移動。

已核對本地與遠端分支及既有 Tag；Fork 既有 Release 為 desktop-v0.9.0-build.m1（prerelease）與 desktop-0.8.0-build.1（正式）。整合前尚不包含 v0.10.0；保留兩個 parent 的 merge 使它成為祖先，避免後續重複整合。main 與 origin/main 維持整合前 SHA。

## B. 雙方功能與變更差異

盤點方式：以共同祖先分別執行 git diff --name-status --find-renames、git log（含 merge 歷史與逐次路徑變動）、git cherry，並閱讀交集原稿、renderer、政策解析、Protocol／Host、Desktop 接線、測試、規格與 CI。過去的 0.9 整合報告僅用於追查，不拿舊驗證當作本次通過證據。

Fork 306 檔、Upstream 82 檔，交集 56 檔；完整聯集 332 檔與每檔決策見 [逐檔盤點](docs/development/upstream-0.10.0-inventory.md)。兩端相對祖先均無淨刪除或更名；歷史上的 10 筆封存移動也列入附錄。Fork 獨有 250 檔中，232 檔逐位元保留；18 檔 Copilot 技能隨 renderer 更新。這不代表未驗證平台已通過。

| 功能／變更 | Fork | Upstream | 整合方式 | 結果 |
|---|---|---|---|---|
| Copilot 與工具選集 | copilot／codex／claude 三工具；自訂內容保留與設定回歸 | 原生 Copilot 架構已在前次 0.9 合入 | 延用唯一 renderer，不再加入早期重複實作；保留三工具設定 | 通過 |
| 共用對話語言段落 | 各技能有 Fork 的安全指引 | 所有產品技能在本文前注入共用語言段落，raw instructions 不注入 | 收入新版 renderer；三個內建工具與自訂 descriptor 都共用；安全段落保留 | 通過 |
| discuss 記錄語言 | 未設定時採主機／Server 語言；正典 show 保留 null | 新增 locale 綁定、使用者原 topic、既有記錄語言保留 | 新規則改讀 languages 的解析後語言名稱，保留英文欄位／slug／既有記錄 | 通過 |
| improve 記錄與搜尋語言 | 系統語言預設與遠端解析 | 新增 topic／記錄語言及雙語關鍵字搜尋 | 保留新行為，改以解析後的語言查詢驅動 | 通過 |
| config 草稿語言 | 系統預設、政策原值、使用者核可、安全政策獨立 | 新增確認後 locale 重寫草稿規則，保留原有行及 review 原話 | 合併草稿規則與 Fork 預設；保留 dry-run 核可與安全葉節點寫入 | 通過 |
| audit 報告語言 | audit 與初步安全掃描為兩種獨立能力 | 新增獨立模式報告語言；Discipline Mode 不改 | 報告讀解析後語言；保留獨立開關與安全邊界，沒有兩套掃描器 | 通過 |
| 語言查詢與資料契約 | workflow-config languages --json；locale 是語言名稱、specLocale 是代碼；Remote 不猜 Client 語言 | 新規則原讀 show 的代碼／null，null 或錯誤回英文 | 使用 Fork 既有只讀 API，無新增 Protocol 欄位；查詢失敗明示停止；同步四份正典規格與手冊 | 通過 |
| baseline／propose／ingest 語系 | spec_locale、auto、OS 預設；Host 與 Server 解析；不回寫政策 | 本次僅共用對話段落，不重寫這些 Fork 行為 | 原有正文保留，對話段落由 renderer 注入 | 通過 |
| 三種桌面外觀與舊偏好遷移 | system／light／dark、舊 key 遷移、寫入失敗保留原色並提示 | 0.9 已採官方 shared theme，本次未改 | 保留前次移植結果，不恢復已淘汰的舊 theme controller | 通過 |
| Toast／配色與共用元件 | toast 使用有效配色、CSS 優先 | 本次無改動 | 完整保留 | 通過 |
| Tray 主題與訂閱競態 | listen-ready 握手、首次呈現及清理修正 | 0.9 跨平台 panel 與原生主題已合入 | 保留同一個事件架構與跨平台接線 | 本機測試通過；原生互動未驗證 |
| 完整專案目錄 | Local root／Remote checkoutRoot／無副本提示，唯讀 | 本次無改動 | 保留新分類設定頁上的客製區塊 | 通過 |
| 目錄空白與長路徑 | 選取複製保留空白、Tab、換行；pre-wrap 與長路徑換行 | 本次無改動 | 原碼及計算樣式回歸完整保留 | 自動測試通過；實機複製未驗證 |
| 專案圖示色票 | 裝置層持久化、Local／Remote locator 隔離；配色在 shared theme | 本次無改動 | 保留六色與 tray snapshot 同步 | 通過 |
| Windows Portable 資料隔離 | marker opt-in；EXE 旁 desktop／speclink／webview；唯讀不回退、搬移跟隨 | 無對應 Portable 功能 | 完整保留 Host／Tauri／WebView 啟動接線 | 主機可執行測試通過；Windows 實機未驗證 |
| Portable 認證與更新 | 不讀寫機器憑證；只在記憶體登入；CLI token；停用安裝器更新 | 無對應功能 | 完整保留，沒有恢復 OS Keychain 寫入或 updater | 主機測試通過；Windows 未驗證 |
| 安全政策與設定 | disabled 預設；off／web-api／api／web；Top 10／API／ASVS／CWE、規則排除；與 audit／TDD 獨立 | 本次語言規則非同類安全引擎 | 保留 core／Host 單一解析、原子 patch 與未知欄位；不自動啟用 | 通過 |
| 安全標準資料 | 明確下載／取消／匯入、版本來源與授權、原子安裝、離線讀取 | 本次無同類實作 | 完整保留；測試不下載官方資料、不改使用者安裝資料 | 合成資料測試通過；官方 fixture 兩項忽略 |
| 安全掃描與報告 | 離線 Rust 規則、SDD／API 邊界、遮罩 JSON／Markdown、取消與狀態分類 | AI audit 指引及其語言規則 | 保留兩者互補用途，不把疑似問題當漏洞證明，不以語言更新替換掃描功能 | 通過 |
| Remote 安全準備與授權 | 明確 prepare 才取 snapshot；角色／revision；舊 Server unsupported_version | 本次無改動 | 保留不可變準備資料、只掃授權本機 checkout、限量與取消 | Local／Remote 測試通過 |
| 安全桌面設定與報告入口 | 標準資料頁、專案政策頁、ChangeDetailPage 報告、繁中台灣用語 | 新增 AI audit 操作手冊 | 保留 Fork 畫面，新增 Upstream audit 手冊；兩種能力清楚區分 | 通過 |
| Windows 安全路徑修正 | 根目錄比對、更新期間舊資料可見性 | 本次無改動 | 原檔逐位元保留 | 本機測試通過；Windows 原生語意未驗證 |
| Desktop 手動／Tag 發布 | desktop-vX.Y.Z[-prerelease]；四個 target、ad-hoc macOS、Windows NSIS／Portable、Linux AppImage | 官方 vX.Y.Z 正式流程另存 | Fork desktop-build.yml 與版本 guard 原樣保留；新版本仍可走預先發行規則 | 靜態／腳本通過；未觸發 Actions |
| CI 測試範圍 | 純發布白名單可省應用測試，未知／產品路徑必跑完整 CI | 既有完整 CI 與各平台測試 | 完整保留 ci-scope；本次產品變更仍會跑完整 CI | 通過 |
| npm／SDK／依賴 | TypeScript fixture 與 ES2022 lib 相容修正、現有 types 依賴 | Cargo workspace 更新正式版號 | 保留 npm metadata／lockfile 和所有 Fork Cargo 依賴；Node 重新 build 後測試 | 通過 |
| 正式版發布／Updater | Fork 自用 workflow 不產生簽章更新包；非 Portable 既有端點仍指官方 | 官方簽章、npm、Release 與 latest.json 流程 | 全部保留；不變更發布目的地／公鑰／app ID，不觸發發布 | 更新來源待人工決策 |
| 文件、規格、封存歷史 | Copilot／外觀／安全規格、LANGUAGE、手冊、前次升級證據 | 語言規格／封存與 audit 手冊、0.10 日誌 | 合併新內容、保留舊證據；0.10 日誌的預設語言說明改為 Fork 真實行為 | 通過 |
| 資產版本與回歸快照 | v1.49.1 已高於官方 v1.49.0 | 新 renderer／測試／快照 | 整合資產升 v1.50.1、重產指紋與六組快照；沒有降版 | 通過 |

「通過」限本次已執行的自動驗證及保留檢查，不代表人工 UI、外部服務或所有作業系統已驗收。每個檔案的 A/M、交集及衝突均見附錄。

## C. Git 衝突與行為衝突

初次 git merge --no-ff --no-commit v0.10.0 產生 53 個 content conflicts，沒有 modify/delete 或 rename 衝突。未全域使用 ours／theirs。

| 衝突組 | 檔數 | 處理 |
|---|---:|---|
| .agents／.claude 生成技能 | 37 | 不手拼生成文字；先合併 source 與 renderer，再用本 checkout 新建 CLI update 重產。Fork 的 .github/skills 18 檔雖未衝突，也一併重產 |
| Cargo.toml／Cargo.lock／tauri.conf.json | 3 | 官方基版升 0.10.0，延續 build.m1；lock 中 14 個 workspace packages 同步，保留其他依賴 |
| workspace/init.rs | 1 | 保留系統語言 template；資產版升 v1.50.1，避免從 Fork v1.49.1 降到官方 v1.49.0 |
| assets.lock 與六組 golden | 7 | 由合併後 renderer 刻意重產，再於一般測試模式核對；不是選單側快照 |
| configuration 中英文件 | 2 | 採官方新增的文件／對話語言區別，加回 Fork 的系統語言預設與原有安全政策 |
| manual about／index／policy-config | 3 | 合併新 audit 導覽、對話語言與 Fork 安全入口／語言預設；保留 Fork 較新編纂日期並合併來源 |

沒有文字衝突的區域也經檢查：

- 四技能的 null→英文會與 Fork 語言解析衝突，改用既有 languages API；實際查詢確認 locale 是語言名稱，不能誤當 tw／ja 代碼。Remote 缺 metadata、離線、認證或壞設定仍回錯，不猜 Client OS。
- audit 的 AI 檢查與 Fork 安全掃描不是重複功能；安全尾段與獨立開關完整保留。
- 資產指紋會因整合內容改變，已同步版號／指紋；技能用詞守門不增加大寫強調詞的豁免。
- Fork desktop-v Tag 流程可接受新版完整 prerelease；官方 v Tag 的正式發版守門仍保留。
- Copilot 無文字衝突仍受 renderer 影響，已測與 CLI／Node 輸出一致。

所有初始衝突逐檔標記於附錄。最終 git diff --name-only --diff-filter=U 為空，已掃描追蹤檔中的衝突標記，git diff --check 通過。

## D. 測試與建置

本機：macOS Apple Silicon、aarch64-apple-darwin。所有命令在整合分支執行，沒有用正式發布來驗證。

| 驗證 | 執行命令 | 實際結果 |
|---|---|---|
| 先重現語言相容性問題 | cargo test --locked -p speclink-core --test it document_language_rules_preserve_fork_system_and_remote_defaults | 修正前失敗（discuss 缺解析語言查詢）；最終同名測試通過 |
| 技能渲染、語言、工具、指紋 | UPDATE_GOLDEN=1 UPDATE_ASSETS_LOCK=1 cargo test --locked -p speclink-core --test it render_golden:: | 刻意重產，60 通過；其後 workspace 在未開重產旗標下再核對 |
| Rust workspace／整合／API／回歸／doc tests | cargo test --workspace --locked | 3,054 通過、0 失敗、4 ignored；含 CLI 559 與 Core 單元 1,056 通過，render golden／Host／Server／Desktop／Store／Remote 等 |
| 腳本、發布／Portable／詞彙／文件守門 | node --test scripts/*.test.mjs scripts/*/*.test.mjs | 372 通過 |
| 共用 UI | npm test -w packages/ui | 80 檔、1,329 通過 |
| Desktop 前端 | npm test -w apps/desktop | 49 檔、1,253 通過 |
| Server Web | npm test -w apps/server-web -- --maxWorkers=2 | 21 檔、244 通過 |
| 0.10 日誌與彈窗 | npm test -w apps/desktop -- --maxWorkers=2 src/__tests__/whatsNew.test.ts src/__tests__/releaseNotesDialog.test.tsx src/__tests__/updatesPage.test.tsx | 修改日誌後再驗 37 通過 |
| UI 型別 | ./node_modules/.bin/tsc --noEmit -p packages/ui/tsconfig.json | 通過（含測試檔） |
| Desktop 型別 | ./node_modules/.bin/tsc --noEmit -p apps/desktop/tsconfig.json | 通過（含測試檔） |
| Server Web 型別 | ./node_modules/.bin/tsc --noEmit -p apps/server-web/tsconfig.json | 通過（含測試檔） |
| 靜態分析 | cargo clippy --locked --workspace --all-targets | 通過，仍有既有 warnings，未啟用 warnings-as-errors |
| Rust 格式 | cargo fmt --all --check | 未通過：269 檔有格式差異，其中 266 檔與 Fork 整合前完全未變；另外三檔的升級前原稿也會失敗。未為本次升級重排整個專案；新增回歸函式已經 rustfmt |
| Desktop 前端正式建置 | npm run build -w apps/desktop | 通過；有既有 chunk 大小提示 |
| Server Web 正式建置 | npm run build -w apps/server-web | 通過；有 chunk 大小提示 |
| 同版 CLI | cargo build --locked -p speclink-cli；target/debug/speclink --version | 0.10.0-build.m1、engine v1.50.1 |
| 三工具生成 | target/debug/speclink update | claude／codex／copilot 全部成功，政策／工具選集不變 |
| 同源 sidecar | node scripts/desktop/desktop-sidecar.mjs --profile debug | 成功佈署，版本與 CLI 同源 |
| macOS .app 打包 | npm run -w apps/desktop tauri -- build --debug --bundles app --ci --config '{"bundle":{"createUpdaterArtifacts":false,"macOS":{"signingIdentity":"-"}}}' | 通過：target/debug/bundle/macos/Speclink.app；codesign --verify --deep --strict 通過，Info.plist 與內附 CLI 均為 0.10.0-build.m1（engine v1.50.1）；僅 ad-hoc，未公證 |
| Node 原生 binding | npm --prefix crates/adapters/speclink-node run build | 通過，使用已安裝依賴、未改 npm lockfile |
| Node SDK | npm --prefix crates/adapters/speclink-node test -- --maxWorkers=2 | 8 檔、52 通過；含 Copilot 與 CLI 逐字輸出比對 |
| 差異與歷史完整性 | git diff --check；git fsck --connectivity-only；備份 tree／main SHA 核對 | 通過；未清理 dangling 物件 |

初次執行與修正紀錄：

1. 沙箱禁止綁定 localhost，首輪 Rust CLI 147 個 Remote 測試失敗，錯誤為 PermissionDenied／Operation not permitted。取得執行權限後重跑；最終整個 workspace 成功。
2. 三組前端同時跑時，Server Web 首輪 6 個非同步畫面斷言失敗；未改原碼，以 maxWorkers=2 重跑全組後 244 通過。保留初輪紀錄，沒有將它宣稱通過。
3. 開發中的語言文字調整曾使 assets.lock／大寫用詞守門失敗；已調整表述、資產版與指紋，最終 workspace 一般模式通過。沒有放寬用詞上限。
4. Node SDK 首輪 1 個 CLI 比對失敗，原因是 binding 已更新但測試 helper 重用較舊 CLI；兩者以最終原始碼重建後，52 項全部通過。

未驗證／跳過邊界：

- **PostgreSQL 真實 DB：未驗證**。未設 SPECLINK_TEST_POSTGRES_URL；既有測試的無 DB 路徑成功不代表實際 SQL driver 已測。
- **官方安全資料 fixture：未驗證**。security_standards_official_repository_snapshot 與 security_data_official_fixtures_install_and_read_offline 明確 ignored，需要使用者先下載的官方 fixture；本次未下載。其餘兩項 ignored 是由父測試啟動的 serverfs_verb_flow_child／lock_holder_child，不等於另外兩項業務功能缺測。
- **Windows／Linux／Intel Mac、NSIS／DMG／AppImage／Portable 實機：未驗證**。只有 Apple Silicon toolchain，未觸發遠端 Actions；Portable staging 與版本腳本測試不等於 Windows EXE 已執行。
- **視窗互動／Tray／Keychain／實機複製／安裝後升級與搬移：未驗證**。不將 jsdom 或 Rust 編譯當真人操作通過。
- **開發者簽章、公證、更新包、安裝包及正式 Release：未驗證**。本次不使用發行憑證或金鑰、不建立 Tag、不觸發發布。

原始本機命令紀錄保留在 ignored 的 .speclink/upgrade-0.10/；此報告與逐檔清單隨 Git 保存。舊 UPGRADE_REPORT.md 等 0.9 報告保留為歷史，不能當作本次報告。

## E. 發布相容性、風險與下一步

### 待人工決定：非 Portable 版更新來源

Fork 的 desktop-build.yml 仍發布到目前 repository，支援 desktop-v 預先發行 Tag，make_latest=false，且不產生 signed updater packages／latest.json。它與 CI／版本腳本相對整合前逐位元一致。

但既有 tauri.conf.json 的 updater endpoint 與公鑰仍指向原作者：

https://github.com/MomoChenisMe/speclink/releases/latest/download/latest.json

非 Portable 的 main.tsx 仍接官方 updater adapter；Portable 不注入 updater。這是升級前就存在的設定，本次未改目的地。若安裝版使用者接受官方更新，可能裝回不含 Fork 客製功能的原作者版本；0.10.0-build.m1 與官方穩定版的版本排序也須在決定更新策略後驗收。沒有實際下載或安裝更新。

建議在任何正式自用安裝版發布前，選定「Fork 自用包停用自動更新」或「建立 Fork 專用且有簽章的更新端點」。不能直接換 URL：還牽涉簽章、公鑰、權限及 artifact 配套。依任務要求，這個發布策略取捨保留給人工決策。

### 其他限制與建議

- 保留 app identifier app.speclink.desktop、資料格式、既有憑證處理及所有 Fork 依賴；本次沒有 schema 資料遷移或資料刪除。相同 app ID 代表與官方安裝版並存／覆蓋仍須人工驗收，沒有擅自改 ID 搬移資料。
- 0.10.0-build.m1 延續既有 Fork prerelease 行為；更新日誌的自動彈窗原本要求頂端日誌版本與 app 版本完全相同，因此該 build 可能不自動彈官方 0.10.0 日誌。手動瀏覽日誌保留；未更改既有彈窗守門來假裝正式版。
- 新技能的語言查詢失敗會明示停止，避免猜測遠端語言；不支援 metadata 的舊 Server 必須升級。對話語言仍跟使用者，既有討論記錄保留原語言，固定標記仍為英文。
- 大量 Rust 格式與 Clippy warnings 是既有維護問題；這次沒有連帶清理。fmt 未通過已明列，不能當成所有品質檢查全綠。
- 正式發布前補跑原有 CI 三平台矩陣、真 PostgreSQL、Windows Portable／WebView 隔離與搬移、簽章及安裝更新驗收。這些不是本機測試可替代的項目。
- main 未修改；沒有 Push、Force Push、PR、刪除分支、重寫歷史、Tag 或 Release。備份分支保留，最後停留在 upgrade/upstream-0.10.0，等待下一步確認。

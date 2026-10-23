# 換機接續開發

## 2026-10-22 家中接續：第 1、2 步完成（目前以本段為準）

已從 origin 快轉至 `c8b77ece`，本輪修改尚未提交／推送。使用者指定先做 CI 核對、SDK 規格方向與 Antigravity 型別修復；採現行技能 API，完成內容與測試見 [CI 與 SDK 修復紀錄](sdk-contract-fix-2026-10-22.md)。

- CI 核對：PR #12 已合併，head `46803b2b` 與 main `9774b7a2` 的 CI／Node SDK 都成功；最新功能提交 `c8b77ece` 沒有 Actions 或 check-runs，不得沿用前次 success。現有 push 觸發只匹配 main／master，本次沒有開 PR、提交或推送；後續須核對新提交 SHA 的 CI。
- SDK 規格方向已由本輪使用者要求採用：skills.list／skills.render，選項 target／invocation／specDir／toolName；不復活 instructions.render 或 store 渲染軸，delta 明確移除退役的指令檔 worktree 軸要求。原始九項需求與既有完成任務保留。
- 公開 index.d.ts 已補 antigravity；新增獨立 TypeScript 依賴與套件入口 strict／noEmit 編譯測試，先重現 TS2322 再轉綠；既有 Node SDK npm test 自動涵蓋。補預設值、specDir／toolName 及錯誤 invocation 回歸，並以突變確認測試能失敗。SDK 全組 61 passed、scripts 386 passed，analyze 無 Critical／Warning，validate 與 diff 檢查通過。
- 進度更新為程式 **61／61**、全部 **61／62**；公司 Proxy `[M]` 12.3 保持未完成。未執行完整品質重跑，舊 Round 2 與完成歷史不能當成本輪已通過。
- 下一步先處理補查剩餘的 vscode／真 Git dirty 組合覆蓋及 configuration 中英表格缺口，再按既有工單／快照規則重開完整 discovery。兩份活動工單與 checks 歷史均保留，本輪沒有 discard、蓋章、合併、打包、封存或產生 manual。品質 trusted base 仍是 `faefda7dabde01b8d90974aea63f4f7a8bdd5c24`。
- main 已含 PR #12 的歷史，故後續交付須以包含 c8b77ece 及本輪後續修復的新 main 提交為準，不能把舊 main 的 CI 或安裝包當成已包含本輪內容。

下方交接為較早狀態，其中「先決定 SDK 方向」已由本輪完成；歷史證據保持原文。

## 2026-10-22 前次換機交付

分支仍為 `feat/desktop-workflow-offline-security-review`，origin 為 `https://github.com/mosutw/speclink.git`。本次依使用者要求提交並推送以換機接續；本文件不硬填自身提交 SHA，請以對話提供的 SHA、`git log -1` 與 `origin` 分支一致性核對。

### 已完成與尚未完成

- 程式任務 **59／59**，全部 **59／60**；唯一人工任務 `[M]` 12.3 仍未完成，先 main 合併及打包後，再在公司安裝驗收。
- 前次 `46803b2b` 修正 PR #12 的 CI 回歸，該 commit 的三平台 CI 與 Node SDK 已成功；**本次新增提交的 CI 須重新確認**，不能沿用該次 success。
- 本次提交的 11 個程式檔修正：verify 最終提交固定使用已驗收 index 指紋，避免中途換報告仍蓋章；封存先在 backend 鎖內核對基底再寫正式規格，避免 CAS 衝突留下半套效果；共用三處安全決策載入。兩個原 CRITICAL 均有先失敗再通過的回歸測試。
- 受影響 Rust 測試：2744 passed、3 ignored；另外三項 PostgreSQL 測試未設 DB 而跳過。scripts：353 passed、33 skipped。最後微調後 core 再跑 1075 passed、1 ignored。補查另跑 UI 184、Desktop 720，共 904 passed；`git diff --check` 通過。詳見 [修復及 Round 2](quality-ci-followup-2026-10-22.md)。
- Review／Verify 活動工單各保留本次 Round 1、Round 2，Round 2 沒有未解原 finding。**後續完整補查另有新問題，整體不能稱已通過**；不將新問題偷塞進只驗原修復的 Round 3，不刪完成歷史。
- 補查已完成；正確總數為 **46 條需求、133 個情境、7 個範例**。先前 143 是加總錯誤。完整映射及實際 partial／manual 見 [逐項追溯](quality-spec-coverage-2026-10-22.md)。沒有自動蓋章、合併、打包、封存或更新 manual。

### 回到另一台電腦先做什麼

在專案目錄先執行 `git status --short`。若有未提交修改，先保存再整合，不可 reset、clean 或強制覆蓋。

```powershell
git fetch origin
git switch feat/desktop-workflow-offline-security-review
git pull --ff-only origin feat/desktop-workflow-offline-security-review
git log -1 --oneline
npm ci
cargo build -p speclink-cli --locked
```

若本機尚無分支，將 switch 那行改為：

```powershell
git switch --track origin/feat/desktop-workflow-offline-security-review
```

Windows 使用 **checkout 新建置的** `.\target\debug\speclink.exe`；PATH 上既有的 `D:\tools\Speclink\speclink.exe` 可能仍是 build.2，不能用舊 CLI 判斷新工單或功能。

### 下一步：先處理補查事項，不先收尾蓋章

詳見 [補查結果與規格修訂草案](quality-discovery-completion-2026-10-22.md)。優先順序：

1. **先決定 Node SDK 規格方向**。delta 還要求已不存在的 `instructions.render` 與 `store` 選項。建議依現行 SDK 更新 delta，只保留 skills API 與實際參數；這個方向尚未獲使用者選定，不能為了通過檢查直接刪規格。若使用者仍要舊 API，需先補設計與任務。方向確認後透過 `$speclink-ingest desktop-workflow-and-offline-security-review` 調整文件。
2. **修正 SDK 型別漏項**。`crates/adapters/speclink-node/index.d.ts` 是手寫公開型別，package 的 types 指向它；`RenderOptions.target` 缺 `antigravity`。最小 `skills.render('apply', {target:'antigravity'})` 用 tsc 已重現 TS2322；`copilot` 對照通過。新增公開型別的編譯測試，再修正值域，不能只依賴 Vitest 轉譯測試。
3. 補兩處精確測試追溯：單值 `--tools vscode`（目前只有 `claude,vscode`）、真 Git commit 後 dirty 內容的 HEAD 參考與實際 bytes digest。同時修 `docs/configuration.md` 及 zh-TW 的自訂工具表格斷開，以及規則表漏列 `.agent/skills`。
4. 本次補查以 **scope changed / failed** 結束，保留票據。修正及文件範圍確定後，再以正式處置開新 discovery；原可信基準仍為 **`faefda7dabde01b8d90974aea63f4f7a8bdd5c24`**，不可改用目前 HEAD。這次為換機而提交修正，不能假設換機後原 validation 快照仍能還原全部修復。
5. 新 discovery 通過後才由使用者明確選擇品質收尾。後續順序仍是 **CI → 正式版本號 → main 合併與打包 → 公司 Proxy 12.3 → 封存 → manual**；本次沒有授權自動合併或發版。

`.speclink/review-scopes/` 是本機忽略資料，**沒有推入 Git**，新電腦可能回 frozen snapshot missing。Git 中的 review.md／verify.md、兩份品質證據與逐項映射都有保存，程式修正不會因此遺失。不可偽造快照、自行改票或直接以目前 HEAD 建基準；需要丟棄活動票以重開 discovery 時，先將兩份票據原文另存 evidence，經使用者明確同意後用正式 review／verify discard，不刪 checks 完成歷史。下方較早的「直接 closing stamp」敘述不適用本次尚有新問題的狀態。

可貼給另一台 Codex：

> 請先讀 openspec/changes/desktop-workflow-and-offline-security-review/evidence/handoff.md 的 2026-10-22 段，以及 quality-discovery-completion-2026-10-22.md、quality-spec-coverage-2026-10-22.md。先核對分支最新提交與 CI。原兩項並行問題已修復，完整補查另發現 SDK 公開型別漏 antigravity，以及 delta 沿用已退役的 instructions.render/store；先讓我決定規格方向，再修復與補測試。保留兩站工單及 checks 歷史，不拿舊 Round 2 clean 當整體通過，不自動蓋章、合併、打包、封存或更新 manual。新 discovery 的 trusted base 是 faefda7dabde01b8d90974aea63f4f7a8bdd5c24，缺本機快照時先回報，不自動丟棄票據。

Git 不含依賴、Rust target、裝置設定、憑證、安全標準資料或本機 scope 快照；本次排除使用者既有 `$null` 輸出檔。私人裝置資料不隨專案推送。

## 較早交付紀錄（歷史，以下數字與收尾建議不代表最新狀態）

- 分支：`feat/desktop-workflow-offline-security-review`。
- 變更：`desktop-workflow-and-offline-security-review`，保留原始九項需求、安全驗收 checkbox 與所有專案提案預設條列顯示。
- 原提交 `945175c5` 的程式任務42／42、全部42／43。2026-10-21追加需求及Windows鎖定修復完成後，程式任務55／55、全部55／56；唯一未完成為公司Proxy手動驗收12.3。本文件隨附的換機交付提交包含追加功能、品質修正與兩站工單，不能只拉945175c5就取得本輪實作；交付SHA以本次對話及 `git log -1` 核對。
- 12.3 依使用者要求開發期間跳過，待打包後在公司安裝驗收；維持未勾選與原 ID，不阻擋其他實作或產生測試安裝檔，不代表驗收通過。
- 已執行speclink-quality兩輪：Round 1為review兩項、verify三項WARNING（合併重複後四個問題）；使用者選全部修正，Round 2兩站均無未解finding。兩票仍保留，未蓋章／封存，也未合併main或打包。verify複驗子程序未完成，最後由主流程依同一凍結修正範圍補完三項核對；不是把失敗的子程序當通過。
- 驗證結果與已知限制見同目錄 `delivery-regression.md`；公司實機紀錄填入 `company-proxy.md`。
- 新需求與規劃查核見 `ingest-2026-10-21.md`。Windows封存鎖已依使用者同意移出提案資料夾並補回歸；版本來源、署名、共用標準路徑及文件已完成。最新實測與未測NAS／公司限制詳見 `apply-2026-10-21.md`，不把歷史紀錄當本輪全套通過。

## 取得分支

後續交付順序已確認：先完成實作與合併前必要檢查，將功能分支合併回本專案 main，再從 main 的明確 commit 打包產生安裝檔，最後在公司執行 12.3；紀錄來源 commit 與安裝檔名稱。12.3 延期不阻擋此安排，合併本身不代表公司驗收或封存通過。

```bash
git fetch origin
git switch --track origin/feat/desktop-workflow-offline-security-review
```

若本機已有同名分支，改用 `git switch feat/desktop-workflow-offline-security-review`，再執行 `git pull --ff-only`。

回去後先確認工作目錄沒有未提交修改，再執行：

```powershell
git fetch origin
git switch feat/desktop-workflow-offline-security-review
git pull --ff-only origin feat/desktop-workflow-offline-security-review
git log -1 --oneline
npm ci
cargo build -p speclink-cli
npm run dev:desktop
```

分支尚不存在時以先前的 `git switch --track` 取代 `git switch`。不要用reset或強制覆蓋另一台尚未提交的修改。Windows先確認Git、Node/npm、stable Rust（x86_64-pc-windows-msvc）、C++ Build Tools及WebView2可用；安裝細節沿專案[開發說明](../../../../docs/development.zh-TW.md)。Rust安裝後需重開終端機讓PATH生效。既有CLI wrapper不會自動重建已存在的binary，因此先明確cargo build；也可用 `npm run cli -- ...` 呼叫checkout版本。

## 品質收尾與本機快照

已修復四項：封存後晚到寫入不重建提案、共享標準斷線不擋CLI／Server既有報告、確認封存前顯示三類保留／省略數與摘要提示、更新頁雙語原作者403／TLS錯誤及保留舊時間測試。

review.md／verify.md工單會隨Git交付；`.speclink/review-scopes/desktop-workflow-and-offline-security-review/` 的凍結快照不會。它包含本機工作資料，不推入Git。若要直接沿用原輪次，需另行將原機該完整目錄複製到新checkout相同相對位置，再以quality的closing stamp call收尾；蓋章仍需使用者明確選擇。

只有Git、未帶快照時，quality可能報frozen snapshot missing；這不是修正遺失。不可拿新HEAD當原始基準、偽造快照或自動丟棄工單。先保留現有票據，讓使用者選擇帶入快照，或明確同意把兩票備份到evidence後，透過review／verify discard並以原始可信基準重做discovery。重做檢查之後仍先報結果，不能直接蓋章。

可在新電腦貼給Codex：

> 請讀取 openspec/changes/desktop-workflow-and-offline-security-review/evidence/handoff.md，接續此變更的品質收尾。程式55／55、總55／56，12.3等main打包後公司驗收。先核對review／verify的Round 2。若缺本機凍結快照，我同意先將現有兩份工單原文備份到evidence，再用正式discard命令清除活動票據，以faefda7dabde01b8d90974aea63f4f7a8bdd5c24為可信基準重新執行兩站discovery；保留歷史，不把目前HEAD當原始基準。檢查完成先回報，不自動蓋章、封存、合併或打包。

本次為換機才提交修正。後續一般quality修正輪，盡量等兩站複驗收尾後再提交，避免改變凍結差異的判讀。真實公司Proxy、NAS／SMB與其他OS仍未驗收；104個scenarios未逐一核讀全部斷言，不把本輪針對四項的複驗說成所有場景已驗證。

## 本機準備與審查基準

Git 不包含 node_modules、target、安裝的安全標準資料、憑證、裝置設定及 `.speclink/` 本機快取。新電腦依專案既有安裝方式準備依賴；要做安全審查時，先離線匯入／安裝所選版本的完整資料，再準備 `.security_kb/`，不要在審查期間下載資源。專案目前未開啟安全審查政策，不因換機自行開啟。

這次Apply的原始可信基準是 `faefda7dabde01b8d90974aea63f4f7a8bdd5c24`；本輪使用者已明確選它作兩站discovery基準。host-local的baseline記錄建立較晚，不能用它的945175c5取代原始基準。重做discovery時核對該commit，明確使用 `speclink review scope CHANGE --base <SHA> --json` 與 `speclink verify scope CHANGE --base <SHA> --json`；有活動票據卻缺快照時須先走上節的明確處置，不能單加--base繞過。

不要提交 `.checks.lock` 或 `.checks-locks/` 內鎖檔；它們是執行時鎖，不是提案紀錄。升級前先停止舊版寫入者，不支援新舊協定混用。共用標準的裝置設定不跟Git搬移，換機需重新指定；私人報告與.security_kb仍留各裝置／checkout。

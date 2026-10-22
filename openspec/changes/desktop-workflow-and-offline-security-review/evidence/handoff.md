# 換機接續開發

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

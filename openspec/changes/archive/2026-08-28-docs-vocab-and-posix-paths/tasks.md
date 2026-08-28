## 1. 使用者文件避免詞修正

- [x] 1.1 把「抽屜」改為「詳情面板」共 5 處:docs/product-status.zh-TW.md(第 32、55 行附近各 1 處)、docs/roadmap.zh-TW.md(第 61、63 行附近各 1 處)、docs/verb-contract.zh-TW.md(第 224 行附近 1 處)。改詞後句子保持通順、語意不變,不重寫段落。驗證:對這 3 個檔案 grep「抽屜」零命中。 <!-- speclink-task:tsk_01M12Z7Z1X3HH3BM68290HCB0F -->
- [x] 1.2 把「追溯」改為「溯源」共 2 處:docs/workflow.zh-TW.md(第 70、76 行附近)。改詞後句子保持通順、語意不變。驗證:對 docs/workflow.zh-TW.md grep「追溯」零命中。 <!-- speclink-task:tsk_01M12Z7Z1XYG1M93KS34PM0GZX -->
- [x] 1.3 詞彙守門測試由紅轉綠:node --test scripts/vocabulary-guard.test.mjs 全數通過,「使用者可見文案面不出現 LANGUAGE.md 的 avoid 詞」測試回報的違規清單清空。 <!-- speclink-task:tsk_01M12Z7Z1X9ECTDBNYR6A65SHG -->

## 2. 截圖腳本邏輯路徑改用正斜線

- [x] 2.1 scripts/docs-screenshots.mjs 的純路徑推導函式 stateDirsFor、pathsFor、manifestPathIn、backupPlanFor 改用 path.posix.join 組路徑,推導結果在任何平台都是正斜線;不動真實搬移邏輯、demo workspace 建置與 CLI 呼叫。驗證:grep 確認這 4 個函式內不再有裸 path.join,且 node --test scripts/docs-screenshots.test.mjs 在本機(macOS)全綠。 <!-- speclink-task:tsk_01M12Z7Z1XT8VQB3X6ZDBBTS20 -->
- [x] 2.2 跨平台回歸確認:狀態目錄推導、工作路徑推導、備份計畫 3 個測試斷言的都是 posix 前綴(/fake/home、/fake/tmp),與改後實作一致;實際 Windows 驗證由 CI 的 build-and-smoke (windows-latest) job 承擔。驗證:重讀 scripts/docs-screenshots.test.mjs 相關斷言與實作對照,無其他被 path.join 影響的推導函式漏網。 <!-- speclink-task:tsk_01M12Z7Z1XHZBGDH86MX3M9F9V -->

## 3. 收尾

- [x] 3.1 scripts 測試面全綠且改動面乾淨:node --test scripts/ 全數通過;git status 只含本變更預期的 5 個檔案(docs 4 檔 + scripts/docs-screenshots.mjs)與 change 目錄自身。 <!-- speclink-task:tsk_01M12Z7Z1XZP09H9ZF3F9VNXY2 -->
- [x] [M] 3.2 推送後在 GitHub Actions 確認 CI 的 build-and-smoke 三平台(ubuntu、windows、macos)全綠;macOS 若再因 index.crates.io DNS 解析失敗而紅,屬 runner 基建飄移,重跑該 job 即可,不回頭改碼。 <!-- speclink-task:tsk_01M12Z7Z1XM7480RFC40D5RXVY -->

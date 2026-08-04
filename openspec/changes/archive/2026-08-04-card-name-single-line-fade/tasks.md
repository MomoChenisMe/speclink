## 1. 卡片名稱列單行截斷與淡出

- [x] 1.1 抽出看板卡片統一解剖學的名稱列共用元件 CardNameRow：名稱恆單行、溢出時尾端漸層淡出、未溢出時不套遮罩（末尾字元不被誤淡），複製鈕與名稱同列且不被壓縮。驗證：packages/ui/src/__tests__/cardNameRow.test.tsx 的「名稱不換行，複製鈕與名稱在同一列容器內」「名稱溢出時尾端套漸層淡出遮罩」「名稱未溢出時不套遮罩」三條通過 <!-- speclink-task:tsk_01KZ5RHDB0SNETH9FY1T2BGYRY -->
- [x] 1.2 變更卡改用名稱列共用元件：長名稱不再折行、複製鈕不落次行、meta icons 仍靠右不被擠出卡外。驗證：packages/ui/src/__tests__/kanban.test.tsx 的「標題等寬字型、單行不折行、複製鈕與標題同列尾隨」通過 <!-- speclink-task:tsk_01KZ5RHDB0HK1FKCW7JGA8PS85 -->
- [x] 1.3 討論卡全卡改用同一名稱列：長 slug 由強制斷字折行改為單行截斷淡出，複製 slug 鈕留在同列。驗證：cardNameRow.test.tsx 的 DiscussionColumn 兩條與 discussionColumn.test.tsx 的識別列斷言通過 <!-- speclink-task:tsk_01KZ5RHDB02ZWQKM7XA8QY5HXK -->
- [x] 1.4 兩張卡因改用共用元件而孤兒化的複製狀態與 icon imports 清除乾淨，無殘留死碼。驗證：以 tsc --noEmit 對 packages/ui 檢查本次檔案零錯誤，且 vitest run 於 packages/ui 全綠 <!-- speclink-task:tsk_01KZ5RHDB0KFAFSXCYQPRVSARD -->

## 2. 桌面版驗收

- [x] 2.1 本機桌面 app 重建安裝後，看板長名稱卡片呈單行截斷淡出且複製鈕維持在同列。驗證：先建前端 dist 再產 tauri app bundle（tauri.conf.json 無 beforeBuildCommand，dist 不會自動重建），覆蓋安裝 /Applications/Speclink.app 並啟動，肉眼確認提案中欄長名稱卡片的收尾與複製鈕位置 <!-- speclink-task:tsk_01KZ5RHDB0TNZJZ5RPSEHCRQBB -->

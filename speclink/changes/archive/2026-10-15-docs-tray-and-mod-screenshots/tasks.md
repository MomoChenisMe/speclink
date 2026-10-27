## 1. 截圖腳本（Red → Green）

- [x] 1.1 Red：改寫 `scripts/docs/docs-screenshots.test.mjs`，依「D1 檔名與引用位置」斷言 `SHOT_LIST` 為 15 個檔名（原 11 個加 `desktop-tray.png`、`claude-code-panel.png`、`claude-code-quality.png`、`claude-code-hash-menu.png`），並依「D2 示範專案加一張未結審查工單」斷言 `DEMO_REVIEW` 指向狀態為 ready 的示範變更、含 2 條 finding（一條 WARNING、一條 SUGGESTION），dry-run 報告列出這張工單。驗證：`node --test scripts/docs/docs-screenshots.test.mjs` 對上述斷言失敗。 <!-- speclink-task:tsk_01M4Z3M5WFAZKRS035JZXHH4XA -->
- [x] 1.2 Green：修改 `scripts/docs/docs-screenshots.mjs`：擴充 `SHOT_LIST`、新增 `DEMO_REVIEW`，`createDemoWorkspace` 依 D2 的步驟（`init` 後先 commit、標記進行中之前 `review prepare`、`task done` 之前寫示範檔、`review scope` 與 `review add-round`）開工單，`printHandoff` 加上系統匣與 Claude Code 的拍攝提示。驗證：`node --test scripts/docs/docs-screenshots.test.mjs` 全綠；在 scratchpad 實際建一次示範專案後 `speclink review show add-keyboard-shortcuts` 列出 2 條 finding。 <!-- speclink-task:tsk_01M4Z3M5WFHEMBYXSWKQCYABGH -->

## 2. 拍攝（D3、D4、D5）

- [x] 2.1 依「D5 先修再拍」確認 fix-english-status-labels-and-plurals 已在 HEAD，經使用者同意後結束 app、以 `scripts/desktop/desktop-install.mjs --install` 重裝；再依「D3 系統匣面板的拍法」在 `--setup` 的乾淨狀態下，中英兩輪重拍 `desktop-change-detail.png`、`desktop-discussion.png`、`desktop-spec.png` 並拍 `desktop-tray.png`，拍完還原使用者狀態並重開 app。驗證：兩個語言目錄各更新 3 張、新增 1 張；英文 `desktop-spec.png` 的狀態列不重疊且顯示「1 scenario」；腳本印出已還原、備份目錄不存在。 <!-- speclink-task:tsk_01M4Z3M5WFN9HGAYFV9RK0KP3H -->
- [x] 2.2 依「D4 Claude Code 截圖的拍法與隱私」，在 Terminal 新視窗、`/Users/Shared/acme-web` 示範專案、只開 speclink-skills 的 Claude Code session 中，外掛語言依輪次設為 `en` 與 `zh-TW`，各拍 `claude-code-panel.png`（看板分頁）、`claude-code-quality.png`（品質分頁）與 `claude-code-hash-menu.png`（打 `#` 的選單），全程不送出提示，拍完結束 session 並刪除 `/Users/Shared/acme-web` 與 scratchpad 的設定檔。驗證：兩個語言目錄各新增 3 張；session 記錄沒有任何送出的提示。 <!-- speclink-task:tsk_01M4Z3M5WFM87BTTB4J90C51C2 -->
- [x] 2.3 對 2.1 與 2.2 的 14 張圖執行隱私檢查：OCR 搜尋使用者帳號名稱、`/Users/`、`/var/folders/`、非示範信箱、用量百分比字樣與組織名稱，只能命中示範資料；再移除 `tEXt`／`iTXt`／`zTXt`／`eXIf`／`iDOT` 區塊並列出區塊清單（「使用者文件以截圖呈現實際介面」的隱私條款）。驗證：記錄 OCR 結果；區塊清單沒有上述區塊，`sips` 全部可解碼。 <!-- speclink-task:tsk_01M4Z3M5WFCCVHW0320W73S8DC -->

## 3. 文件引用（D1）

- [x] 3.1 依 D1 更新引用：`README.md` 與 `README.zh-TW.md` 的畫面一覽改為 3×2 並加入 `desktop-tray.png` 與 `claude-code-panel.png`，「Claude Code 外掛」段加品質分頁與 `#` 選單兩欄表格；`docs/workflow.md` 與 `docs/workflow.zh-TW.md` 在品質關卡章段落加系統匣截圖；`integrations/claude-code/speclink-skills/README.md`（繁中圖）與 `README.en.md`（英文圖）加三張外掛截圖；alt 文字依畫面撰寫。驗證：`node --test scripts/docs/docs-links.test.mjs scripts/docs/docs-parity.test.mjs` 的斷鏈、孤兒與中英對等檢查全部通過；外掛 README 的相對路徑以 `ls` 確認指得到檔案。 <!-- speclink-task:tsk_01M4Z3M5WFJ4H9GZN2X1W8KBPD -->

## 4. 收尾

- [x] 4.1 跑完整的文件守門：`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 以 exit code 0 結束；`git status` 列出的檔案都在本變更 proposal 的 Impact 範圍內。 <!-- speclink-task:tsk_01M4Z3M5WFS4V6VPQWBSE8XDA4 -->

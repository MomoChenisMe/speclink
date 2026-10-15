## Context

- 目前截圖有 11 個檔名，中英各一組，放在 `docs/assets/screenshots/en/` 與 `docs/assets/screenshots/zh-TW/`。`SHOT_LIST` 與 `SHOT_LOCALES` 定義在 `scripts/docs/docs-screenshots.mjs`，文件守門測試要求每張截圖都被同語言的 README 或 docs/ 文件引用。
- 系統匣面板是桌面 app 的獨立視窗，從 macOS 選單列的 Speclink 圖示打開。視窗層級可能高於一般視窗，取視窗 ID 時不能只找 layer 0。
- speclink-skills 是 Claude Code mod，放在 `integrations/claude-code/speclink-skills/`。它有技能列、側邊面板（看板、討論、品質三分頁）與 `#` 選單，介面語言由外掛設定 `language`（`auto`、`en`、`zh-TW`）決定。使用者本機另外裝了自己的 statusbar 與 tidy 等 mod，statusbar 會顯示用量百分比。
- 品質分頁只在有未結工單時有內容。示範專案目前沒有工單。
- fix-english-status-labels-and-plurals 會改變詳情頁狀態列的版面與英文計數，三個詳情頁截圖要在它之後重拍。
- 本變更不動任何程式碼。

## Goals / Non-Goals

**Goals:**

- 中英各補 4 張新截圖，並重拍 3 張詳情頁截圖。
- README、workflow 與外掛 README 引用新截圖，守門測試維持全綠。
- 截圖不含使用者的其他外掛輸出、帳號資訊、家目錄路徑與選單列。

**Non-Goals:**

- 不拍 Windows 或 Linux 的系統匣。
- 不改外掛 README 的語言主版（維持 `README.md` 繁中、`README.en.md` 英文）。
- 不改 Claude Code 外掛或桌面 app 的程式碼。
- 不在 Claude Code 截圖中送出任何提示給模型，所以技能列右側的「步驟 · 變更」不會出現。
- 不重拍其他 19 張未受影響的截圖。

## Decisions

### D1 檔名與引用位置

`SHOT_LIST` 新增 4 個檔名，變成 15 個：

| 檔名 | 畫面 | 引用處 |
| --- | --- | --- |
| `desktop-tray.png` | 系統匣面板 | README 畫面一覽、workflow 的品質關卡章段落 |
| `claude-code-panel.png` | 技能列＋側邊面板看板分頁 | README 畫面一覽、外掛 README |
| `claude-code-quality.png` | 側邊面板品質分頁 | README「Claude Code 外掛」段、外掛 README |
| `claude-code-hash-menu.png` | `#` 變更選單 | README「Claude Code 外掛」段、外掛 README |

README 的畫面一覽從 2×2 改為 3×2，依序為變更詳情、手冊、規格詳情、後台總覽、系統匣面板、Claude Code 外掛面板。「Claude Code 外掛」段在安裝指令後加一個兩欄表格，放品質分頁與 `#` 選單。外掛 README 從 `../../../docs/assets/screenshots/<語言>/` 引用三張外掛截圖。

### D2 示範專案加一張未結審查工單

`createDemoWorkspace` 在建立已就緒變更 `add-keyboard-shortcuts` 之後，用 CLI 為它開一張審查工單：

1. 在示範 repo 加一個與該變更任務對應的小檔案並 commit，讓審查範圍有內容。
2. 執行 `review prepare`、`review scope`，再以 `review add-round` 寫入第一輪。
3. 第一輪有 2 條 finding：1 條 WARNING、1 條 SUGGESTION，內容為虛構，路徑在凍結的範圍內。

工單不蓋章，所以桌面詳情頁的文件樹會多一個「審查」節點，外掛的品質分頁也會列出它。示範定義以 `DEMO_REVIEW` 常數匯出，讓單元測試檢查它的結構。如果引擎在空範圍時拒絕 add-round，就由第 1 步的 commit 提供範圍。

### D3 系統匣面板的拍法

在 `docs-screenshots.mjs --setup` 佈置的乾淨狀態下拍攝，面板只會看到示範專案：

1. 用 computer-use 點選單列上的 Speclink 圖示，打開面板。
2. 用 CGWindowList 找出 owner 為 Speclink、尺寸符合面板的視窗 ID，不限 layer 0。
3. 用 `screencapture -x -l <ID>` 只擷取面板視窗。

中英兩輪跟著桌面的語言設定切換。

### D4 Claude Code 截圖的拍法與隱私

- **終端機**：用 macOS 內建 Terminal 開一個新視窗，固定欄列數。只擷取這個視窗。Warp 會顯示使用者其他分頁的標題，所以不用 Warp。
- **示範專案**：用 `TMPDIR=/tmp` 執行建立函式，讓專案路徑是 `/tmp/acme-web`，不含使用者帳號名稱。
- **只開 speclink-skills**：用 `claude --settings <scratchpad 內的 JSON>` 覆寫：把使用者自己的其他外掛設為停用、清空 statusLine、開啟 fullscreen tui。外掛 language 依輪次設為 `en` 或 `zh-TW`。如果覆寫關不掉某個外掛，就先停下來問使用者，由使用者決定要不要暫時停用。
- **不送出提示**：只用 `/speclink-panel` 開面板、點分頁、在輸入框打 `#`，從不按 Enter 送出提示。截圖後用 Ctrl+C 結束 session。`/speclink-panel` 是外掛指令，不會呼叫模型。
- **隱私檢查**：沿用上一輪的 OCR 檢查與 PNG 文字區塊清理，另外搜尋 Claude Code 可能顯示的帳號信箱、組織名稱與用量字樣。

### D5 先修再拍

開拍前確認 fix-english-status-labels-and-plurals 已合進 HEAD。接著問使用者同意後，用 `scripts/desktop/desktop-install.mjs --install` 重裝桌面 app，再重拍三個詳情頁並拍系統匣。重裝前後都要先結束使用者正在用的 app，並問使用者。

## Implementation Contract

**可觀察的結果：**

- `docs/assets/screenshots/en/` 與 `zh-TW/` 各有 15 張同名 PNG。
- README 兩版的畫面一覽有 6 張截圖，「Claude Code 外掛」段有 2 張。
- workflow 兩版在品質關卡章段落有系統匣截圖。
- 外掛 README 兩版各有 3 張外掛截圖。
- 三張重拍的詳情頁截圖中，英文狀態列不重疊，計數是單數形（例如「1 scenario」）。
- `node scripts/docs/docs-screenshots.mjs --dry-run` 列出 15 個檔名與示範審查工單。

**驗收：**

- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 以 exit code 0 結束（斷鏈、孤兒截圖、中英對等）。
- 新拍與重拍的 14 張圖 OCR 只命中示範資料。PNG 區塊清單沒有 `tEXt`、`iTXt`、`zTXt`、`eXIf`、`iDOT`。
- Claude Code 截圖不出現使用者其他外掛的輸出（例如用量百分比）。

**失敗模式：**

- 截圖腳本在 app 執行中被呼叫時，維持既有行為：拒絕開始，不搬任何目錄。
- 拍攝中斷時，維持既有的還原行為。
- Claude Code 設定覆寫無效時停下來問使用者，不繞過。

**範圍：**

- 範圍內：截圖腳本與其測試、上述文件、上述 14 張圖。
- 範圍外：程式碼、其他截圖、外掛 README 的語言主版。

## Risks / Trade-offs

- [`--settings` 覆寫不一定能停用已安裝的外掛] → 先在 scratchpad 試；不行就請使用者在 `/plugin` 暫時停用，拍完再開回來。
- [Claude Code 版本更新讓畫面變動] → 截圖反映拍攝當下的版本；外掛 README 已註明測試版本。
- [審查工單讓已就緒變更多一個「審查」節點，改變其他截圖] → 已就緒變更只出現在看板卡片上，看板截圖不重拍；卡片若因工單多出圖示，接受這個差異，或在拍看板之前不開工單（擇一，在 apply 時依實際畫面決定並記錄）。
- [回歸對照] → 不動 CLI 與引擎，只有文件守門測試與 `docs-screenshots.test.mjs` 是回歸面。
- [跨平台] → 截圖腳本只支援 macOS，屬既有限制；守門測試在 Windows CI 照常跑。

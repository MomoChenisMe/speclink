## Summary

補上 macOS 系統匣面板與 Claude Code 外掛 speclink-skills 的文件截圖（中英各一組），並在單複數與狀態列修好後重拍三個詳情頁截圖。

## Motivation

目標讀者是在 GitHub 上評估 Speclink 的開發者、PO、PM，以及已在用 Claude Code 的人。目前文件有 22 張截圖，但缺兩個常用的介面：

- **系統匣面板**：macOS 選單列上的 Speclink 面板，不開主視窗就能看到各專案的變更與品質關卡章。workflow 文件提到它，卻沒有畫面。
- **speclink-skills（Claude Code 外掛）**：輸入框上方的技能列、側邊面板（看板與品質分頁）與 `#` 變更選單。README 的「Claude Code 外掛」段只有一行安裝指令，讀者看不出它長什麼樣。

另外，`en/desktop-spec.png` 拍到了英文狀態列標籤重疊與「1 scenarios」。fix-english-status-labels-and-plurals 修好之後，三個詳情頁截圖要重拍。

規格掃描：`user-documentation` 的「使用者文件以截圖呈現實際介面」已規範截圖的存放、分語言、腳本與隱私；本變更補上畫面一覽要涵蓋系統匣與 Claude Code 外掛，以及終端機截圖的隱私條件。其他規格不受影響。

## Proposed Solution

- **新截圖**：每種語言 4 張，共 8 張。
  - `desktop-tray.png`：系統匣面板，只擷取面板視窗，不含選單列。
  - `claude-code-panel.png`：技能列加上側邊面板的看板分頁。
  - `claude-code-quality.png`：側邊面板的品質分頁，列出一張未結的審查工單。
  - `claude-code-hash-menu.png`：在輸入框打 `#` 跳出的變更選單。
- **重拍**：中英兩組的 `desktop-change-detail.png`、`desktop-discussion.png`、`desktop-spec.png`。重拍前先重裝含 fix-english-status-labels-and-plurals 的桌面 app。
- **示範專案**：`scripts/docs/docs-screenshots.mjs` 的示範專案為已就緒的變更建立一張未結的審查工單，讓桌面與外掛的品質分頁都有內容。
- **文件引用**：
  - README 的畫面一覽改為 3×2：變更詳情、手冊、規格詳情、後台總覽、系統匣面板、Claude Code 外掛面板。
  - README 的「Claude Code 外掛」段加兩張小圖：品質分頁與 `#` 選單。
  - workflow 文件提到系統匣的段落加系統匣截圖。
  - 外掛 README（中英）放三張外掛截圖。
- **隱私**：Claude Code 截圖只開 speclink-skills，用設定覆寫關掉使用者自己的其他外掛與狀態列（例如用量百分比）；畫面不得出現帳號名稱、信箱與家目錄路徑；全程不送出任何提示給模型。

## Non-Goals

（design.md 會建立，範圍排除寫在 design 的 Goals / Non-Goals。）

## Alternatives Considered

- **系統匣截圖連選單列一起拍**：比較好理解面板從哪裡來，但會拍到使用者選單列上的其他 app 圖示與時鐘。改成只擷取面板視窗，在文字說明它從選單列打開。
- **外掛截圖只放外掛 README**：README 的守門測試只檢查 README 與 docs/ 的引用，外掛 README 不在範圍內；放進 README 的「Claude Code 外掛」段，讀者在首頁就看得到，也符合「沒有孤兒截圖」。

## Impact

- Affected specs: `user-documentation`（「使用者文件以截圖呈現實際介面」：畫面一覽涵蓋系統匣與 Claude Code 外掛、終端機截圖的隱私條件）
- Affected apps／crates：不改任何程式碼；只改文件、截圖與 `scripts/docs/` 的截圖場景腳本。CLI、`--json` 與設定欄位都不變。
- Depends on: fix-english-status-labels-and-plurals（重拍詳情頁前要有它的修正）
- Affected code:
  - New:
    - 英文介面截圖目錄與繁中介面截圖目錄各新增 4 張 PNG（`desktop-tray.png`、`claude-code-panel.png`、`claude-code-quality.png`、`claude-code-hash-menu.png`）
  - Modified:
    - `scripts/docs/docs-screenshots.mjs`
    - `scripts/docs/docs-screenshots.test.mjs`
    - `README.md`
    - `README.zh-TW.md`
    - `docs/workflow.md`
    - `docs/workflow.zh-TW.md`
    - `integrations/claude-code/speclink-skills/README.md`
    - `integrations/claude-code/speclink-skills/README.en.md`
    - `docs/assets/screenshots/en/desktop-change-detail.png`
    - `docs/assets/screenshots/en/desktop-discussion.png`
    - `docs/assets/screenshots/en/desktop-spec.png`
    - `docs/assets/screenshots/zh-TW/desktop-change-detail.png`
    - `docs/assets/screenshots/zh-TW/desktop-discussion.png`
    - `docs/assets/screenshots/zh-TW/desktop-spec.png`

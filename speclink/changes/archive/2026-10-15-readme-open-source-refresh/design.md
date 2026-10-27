## Context

- README 目前是 `README.md`（繁中）＋`README.en.md`（英文），docs 底下則是「英文無後綴、繁中 `.zh-TW`」。兩套命名相反。
- `user-documentation` 規格把 README 首段寫死：必須含 Spectra App 2.3.1 起源，且「不得把首段改成只剩導覽連結」。中英兩版必須共用同一組截圖。
- 截圖在 `docs/assets/screenshots/` 平放 9 張，2026-10-08 拍攝。之後 `apps/desktop`、`packages/ui`、`apps/server-web` 有 50 多個改版 commit（視窗外框、三欄詳情頁、設定模式、關於頁、後台外殼、列表頁與詳情頁）。
- `scripts/docs/docs-screenshots.mjs` 已經會：備份並還原桌面 app 的兩個狀態目錄、app 執行中拒絕開始、以 checkout CLI 建示範 workspace、在示範 repo 設定本地 git 身分（`Speclink Demo <demo@speclink.invalid>`）。它不按快門。
- 文件守門測試在 `scripts/docs/`：`docs-parity`（成對 H2 錨點、截圖引用）、`docs-links`（斷鏈、截圖孤兒）、`remote-docs`（Remote 教學導流）、`vocabulary-guard`（繁中避免詞）、`docs-surface`（CLI 指令與技能名存在）。這幾支以檔名寫死 README 的語言。
- 這次變更不動任何 crate 與 app 程式碼；crate 邊界相關的設計規則不適用。

## Goals / Non-Goals

**Goals:**

- GitHub 首頁（`README.md`）是英文，結構符合開源專案慣例，三步內看得到怎麼開始。
- 中英文件各自顯示對應語言介面的最新截圖，畫面不含任何個人或機敏資訊。
- 外部貢獻者找得到貢獻流程、漏洞私下回報管道與 issue／PR 範本。
- 10/8 之後改版造成的過時介面描述全部修正。
- 文件守門測試全部通過。

**Non-Goals:**

- 不改品牌資產 README（`docs/assets/brand/`）與 Claude Code 外掛 README（`integrations/claude-code/speclink-skills/`）的語言主版。它們維持「`README.md` 繁中、`README.en.md` 英文」；新 README 各自連到語言相符的那一份。
- 不新增 CODE_OF_CONDUCT（使用者選擇不加）。
- 不把後台截圖的場景佈置寫進版本庫腳本。後台用暫存目錄的一次性 server，步驟記在本設計。
- 不拍系統匣面板：它貼著 macOS 選單列，會拍到選單列上其他 app 的圖示與時鐘。
- 不改示範內容的語言：中英兩組截圖用同一份英文示範內容，只切換介面語言。
- 不調整 docs 的章節結構，只修正過時事實、用語與截圖引用。
- 不改 CHANGELOG 與已封存的變更、討論。

## Decisions

### README 語言主版對調

英文版改名為 `README.md`，繁中版改名為 `README.zh-TW.md`，用 git 改名保留歷史。兩版頂部的語言切換互指。docs 內所有指向 README 的連結改成語言相符的檔名：英文 docs 指 `README.md`，繁中 docs 指 `README.zh-TW.md`。

替代方案：維持現狀。被否決，因為國際讀者第一眼看到中文，也和 docs 的命名相反。

### README 開源首頁結構

README 第一個 H2 之前放：Logo、標語、徽章、語言切換、一段白話介紹（Rust SDD 引擎、PM／PO／RD／AI Agent 共用語意、Local Repo／Remote Store 兩條路），以及一張桌面看板主截圖。

H2 錨點序列（中英相同）：

| 錨點 | 英文標題 | 繁中標題 | 內容 |
| --- | --- | --- | --- |
| `why` | Why Speclink | 為什麼用 Speclink | 3–4 點：規格跟著 repo 走、AI Agent 每站有技能、品質關卡、個人到團隊同一套語意 |
| `quick-start` | Quick start | 快速開始 | 三步：裝 CLI、`speclink init --tools …`、請 Agent 執行 propose |
| `screenshots` | Screenshots | 畫面一覽 | 2×2 表格：變更詳情、手冊、規格詳情、後台總覽（看板已是開頭的主截圖，不重複） |
| `features` | Features | 特色 | 保留現有特色，包含 OpenSpec 相容說明 |
| `workflow` | How it works | 工作流程 | 保留現有流程圖與入口表 |
| `install` | Install | 安裝 | 見下一個決定 |
| `self-hosting` | Self-hosting a server | 自架 server | 三種跑法表、參考實作與兩份公開契約的說明 |
| `docs` | Documentation | 文件 | 保留現有文件地圖 |
| `status` | Project status | 目前狀態 | 現有狀態摘要，連到 product-status 與 roadmap |
| `contributing` | Contributing | 參與貢獻 | 連到 CONTRIBUTING、SECURITY、開發環境文件與從原始碼建置 |
| `acknowledgments` | Acknowledgments | 致謝 | Spectra App 2.3.1 行為參考起源、OpenSpec 目錄結構 |
| `license` | License | 授權 | MIT |

`install` 錨點保留原名，外部既有的 `README…#install` 連結不會斷。Claude Code 外掛維持為安裝段底下的 H3。

替代方案：保留現有 H2 只補段落。被否決，因為「怎麼開始」仍會落在安裝細節之後。

### 安裝細節下沉到 getting-started

README 的安裝段只留下這些內容：

- 桌面三平台表格
- Windows SmartScreen 提示
- CLI 三條一行指令
- 「Windows 沒有安裝腳本」與「無圖形介面的 Linux 改用 CLI」兩句

下面三塊搬到 getting-started 的安裝節（中英），README 用一句話加連結導過去：

- 「桌面 app 覆蓋 CLI」的逐平台表與保留自有 CLI 的做法
- 安裝腳本的三個環境變數表
- `.deb` 遷移說明

getting-started 原本指向「README 的安裝說明」的那句改成指向本頁的小節。

### 截圖依語言分目錄、檔名相同

截圖放在 `docs/assets/screenshots/en/` 與 `docs/assets/screenshots/zh-TW/`，兩個目錄的檔名逐一相同。英文文件只引用 `en/`，繁中文件只引用 `zh-TW/`。舊的平放截圖全部移除。

`SHOT_LIST`（`docs-screenshots.mjs`）列出 11 個檔名，每個檔名在兩個語言目錄各一張：

| 檔名 | 畫面 | 主要引用處 |
| --- | --- | --- |
| `desktop-board.png` | 看板四欄都有卡片 | README 主截圖、workflow |
| `desktop-change-detail.png` | 變更詳情頁（三欄：文件樹、內容、資訊） | README 畫面一覽、workflow |
| `desktop-discussion.png` | 討論詳情頁 | workflow |
| `desktop-spec.png` | 規格詳情頁 | README 畫面一覽、getting-started |
| `desktop-archived.png` | 已封存列表 | workflow |
| `desktop-manual.png` | 手冊頁 | README 畫面一覽、workflow 的手冊說明 |
| `desktop-settings.png` | 設定模式 | configuration |
| `server-setup.png` | 初始設定第二步（建立專案與儲存庫） | remote-getting-started |
| `server-overview.png` | 後台總覽（含「開始使用」區塊） | README 畫面一覽、server-deployment |
| `server-members.png` | 使用者詳情頁的成員資格 | server-deployment |
| `server-project.png` | 專案詳情頁（儲存庫與成員） | remote-getting-started |

`desktop-change-drawer.png` 改名為 `desktop-change-detail.png`，因為抽屜介面已經不存在。

替代方案：用檔名後綴區分語言（`desktop-board.zh-TW.png`）。被否決，因為 22 張平放不好分組，守門測試也要解析後綴。

### 示範專案補齊四欄、手冊與中性名稱

`docs-screenshots.mjs` 的示範 workspace 改成這樣：

- **目錄名**：改成 `acme-web`。分頁標題與視窗標題只會顯示這個名稱。目錄仍放在系統暫存目錄下。
- **看板第四欄**：新增一個任務全部勾完、但還沒封存的變更，讓「已就緒」欄有卡片。
- **規格頁**：新增第二個已封存的變更，讓規格頁至少有兩份正式規格。
- **手冊頁**：寫入三頁固定內容的示範手冊（`openspec/manual/`）。frontmatter 使用 `title`、`section`、`order`、`keywords`、`sources`、`generated`，`sources` 只引用示範規格。
- **git 身分**：沿用現有的 `Speclink Demo <demo@speclink.invalid>`。
- **語言**：示範內容維持英文。

### 截圖拍攝流程與隱私檢查

- **桌面**：
  - 拍攝前確認安裝版涵蓋目前的 `apps/desktop` 與 `packages/ui`：比對安裝時間與最後一個相關 commit。不涵蓋就先用 `scripts/desktop/desktop-install.mjs --install` 重裝。
  - 經使用者同意後結束 app，再執行 `docs-screenshots.mjs --setup`。stdin 接 FIFO，讓還原步驟一定會跑。
  - 開 app、加入示範專案，在設定裡切換介面語言，兩種語言各拍一輪。
  - 用 `screencapture -x -l <視窗 ID>` 只擷取 Speclink 視窗。這個做法不含游標、選單列、Dock、桌布、通知與其他 app。
  - 拍完先結束 app，再觸發還原，最後重開 app 給使用者。
- **後台**：
  - 在 scratchpad 寫一份獨立的 server 設定：store 與 identity 路徑都在 scratchpad，`public_url` 為 `http://localhost:8080`。
  - 先建置 `apps/server-web`，再直接跑 `target/debug/speclink-server`。不跑 `npm run dev:server`，因為它會改寫使用者的 `.dev/config.yaml`。
  - 用 headless Chromium 透過 CDP 操作：viewport 1410×820、deviceScaleFactor 2、`Emulation.setLocaleOverride` 切換 `en`／`zh-TW`、以 localStorage 關掉首次導覽疊層。
  - 帳號只用 `admin@example.com` 與 `dev@example.com`，專案代號 `demo`，儲存庫代號 `backend`。
  - 8080 被占用時，先問使用者再停掉占用的程式。
- **隱私檢查清單**：每張圖逐張目視檢查，畫面不得出現以下內容：
  - 使用者帳號名稱，以及含 `/Users/` 或 `/var/folders/` 的路徑
  - 真實信箱（只允許 `example.com` 與 `speclink.invalid`）
  - 主機名稱
  - 使用者真實的專案或連線名稱
  - 存取金鑰明文或前綴
  - setup token
  - 資料目錄路徑

  不拍憑證頁與系統頁，因為這兩頁會顯示金鑰前綴與資料路徑。檢查完後，用 scratchpad 的小腳本移除 PNG 的 `tEXt`／`iTXt`／`zTXt`／`eXIf` 區塊，再列出區塊清單確認只剩影像資料與色彩描述。

### 開源社群檔案只出英文版

`CONTRIBUTING.md`、`SECURITY.md`、Issue 範本與 PR 範本只有英文版。GitHub 只會在固定路徑顯示這些檔案，沒有語言切換。CONTRIBUTING 會註明「issue 與 PR 可用英文或繁體中文」。

- **SECURITY**：指向 GitHub 私下漏洞回報（Security 分頁的 Report a vulnerability），不公開任何信箱。受支援版本為最新發布的版本。repo 設定裡的 Private vulnerability reporting 要由使用者手動開啟。
- **Issue 範本**：用 YAML issue forms。
  - `bug_report.yml` 的必填欄位：版本（`speclink --version`）、作業系統、模式（Local／Remote）、重現步驟、預期與實際結果。表單提醒不要貼存取金鑰、setup token 或內部網址。
  - `feature_request.yml` 的欄位：要解決的問題、建議做法、替代方案。
  - `config.yml` 保留空白 issue，並加一個連結指向 SECURITY 的私下回報。
- **PR 範本**：包含摘要、對應的 Speclink 變更名稱（若有）、跑過的測試，以及文件是否需要同步。

### 文件過時描述的對照範圍

逐份讀完 13 組成對文件，對照現行程式碼與 i18n 訊息檔（`apps/desktop/src/i18n/messages.ts`、`apps/server-web/src/i18n/messages.ts`、`packages/ui/src/i18n.tsx`），修正介面描述。已知要查的點：

- configuration：「設定」頁兩個分頁的描述，改成設定模式的左欄分類與關於頁。
- workflow：討論與變更詳情頁「各占一個分頁」的描述，改成三欄的文件樹版面；已封存頁的分頁描述。
- remote-getting-started：在使用者頁授予成員資格的操作步驟（詳情頁與置中對話框）。
- server-deployment：「使用者頁與使用者詳情」的描述與截圖說明。
- product-status：「Local 桌面 app」列的入口清單（設定模式、關於頁、系統匣面板），以及查核日期。
- 所有截圖的 alt 文字依新畫面重寫。

以介面上實際的字（i18n 訊息）為準。英文文件用英文介面的字，繁中文件用繁中介面的字。

### 守門測試跟著檔名與截圖目錄改

- `docs-parity.test.mjs`：成對清單與繁中清單改用 `README.zh-TW.md`／`README.md`。截圖比對改成「兩版引用的檔名序列相同，且繁中版全在 `zh-TW/`、英文版全在 `en/`」。截圖的比對樣式要能匹配語言子目錄。
- `docs-links.mjs`：README 清單改成 `README.md` 與 `README.zh-TW.md`。
- `docs-links.test.mjs`：孤兒檢查改成 `SHOT_LIST` 每個檔名在兩個語言目錄都要被至少一份文件引用。測試資料裡的 README 檔名跟著改。
- `remote-docs.test.mjs`：README 導流斷言與連結檢查清單改用新檔名。
- `vocabulary-guard.test.mjs`：守門面的 README 檔名改成 `README.md` 與 `README.zh-TW.md`。
- `docs-screenshots.mjs`：`SHOT_LIST` 換成 11 個新檔名。新增 `SHOT_LOCALES = ['en', 'zh-TW']`，讓守門測試算出 22 個路徑。`printHandoff` 列出兩種語言的待拍清單。示範 workspace 依前述決定擴充。
- `docs-screenshots.test.mjs`：對應改動的單元測試（工作區目錄名、示範變更涵蓋四種狀態、手冊頁的 frontmatter 欄位）。

## Implementation Contract

**可觀察的結果：**

- 在 GitHub 開啟 repo 首頁，看到的是英文 README。頂部可以切到 `README.zh-TW.md`，兩版的 H2 錨點序列逐項相同（依上表）。
- 兩版 README 的開頭都有桌面看板截圖：英文版是英文介面，繁中版是繁中介面。「畫面一覽」段有 4 張截圖。
- README 安裝段只有桌面表格、SmartScreen 提示與 CLI 三條指令。覆蓋說明、環境變數表與 `.deb` 遷移說明在 getting-started 的安裝節，README 有連結導過去。
- 「致謝」段有 Spectra App 2.3.1 起源與 OpenSpec。README 開頭不再出現 Spectra App。
- 版本庫根目錄有 `CONTRIBUTING.md`、`SECURITY.md`。`.github/ISSUE_TEMPLATE/` 有 `bug_report.yml`、`feature_request.yml`、`config.yml`，`.github/` 有 `PULL_REQUEST_TEMPLATE.md`。兩版 README 的參與貢獻段都連到前兩者。
- `docs/assets/screenshots/en/` 與 `docs/assets/screenshots/zh-TW/` 各有 11 張同名 PNG，`docs/assets/screenshots/` 底下沒有平放的 PNG。
- `node scripts/docs/docs-screenshots.mjs --dry-run` 印出示範 workspace 路徑，結尾是 `acme-web`。

**驗收方式：**

- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 以 exit code 0 結束。
- 對 22 張截圖逐張完成隱私檢查清單，PNG 區塊清單不含文字區塊。
- 逐份讀過 13 組成對文件，介面描述與 i18n 訊息相符。

**失敗模式：**

- 截圖腳本在 app 執行中被呼叫時，維持既有行為：拒絕開始、非零結束、不搬任何目錄。
- 拍攝中斷時，維持既有的還原行為。

**範圍：**

- 範圍內：README 兩版、getting-started 等 13 組 docs、社群檔案、截圖、`scripts/docs/` 的腳本與測試。
- 範圍外：crate 與 app 程式碼、品牌與外掛的 README、CHANGELOG、`openspec/manual/`。

## Risks / Trade-offs

- [外部連結指向 `README.en.md` 會失效] → GitHub 不會自動轉址被改名的檔案。repo 內的引用全部改掉。外部引用只會落到 404 頁，影響小，接受。
- [截圖讓 repo 變大：22 張 2x PNG 約 8–12 MB] → 移除舊的 9 張。擷取範圍只有視窗，不拍全螢幕。接受這個成本，換取高解析度的畫面。
- [安裝版與 HEAD 不同步，拍到舊畫面] → 拍攝前比對安裝時間與相關 commit，必要時重裝。
- [拍攝期間使用者的桌面狀態被移開] → 腳本已有備份與還原，加上中斷還原與 app 執行中拒絕開始。開始前取得使用者同意，拍完先結束 app 再還原。
- [畫面意外帶出個人資訊] → 用隔離的示範 workspace、只擷取視窗、目視檢查清單，再清除 PNG 中繼資料。
- [跨平台：守門測試在 Windows CI 執行] → 新的路徑比對一律用正斜線的邏輯路徑。截圖腳本仍只支援 macOS，這是既有限制，不在本次範圍。
- [回歸對照] → 不動 CLI 與引擎，golden 與 CLI 測試不受影響。文件守門測試是唯一的回歸面。

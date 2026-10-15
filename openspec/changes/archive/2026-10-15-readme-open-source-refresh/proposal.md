## Summary

把 README 改寫成開源專案首頁的寫法（英文為主版），補上開源社群檔案，修正桌面與後台改版後過時的文件描述，並以中英兩種介面重拍全部截圖。

## Motivation

目標讀者是第一次在 GitHub 上看到 Speclink 的人：想透過 AI 代理跑 SDD 的開發者、PO、PM，以及想參與貢獻的外部開發者。他們從 README 判斷「這是什麼、值不值得試、怎麼開始」，這一步發生在任何 workflow 階段之前。

現況有三個問題：

- README.md 是繁中版，國際讀者第一眼看不懂；227 行裡安裝段占一半（CLI 被桌面 app 覆蓋的細節、安裝腳本環境變數表），「怎麼開始」被推到很後面。首段塞了 Spectra App 起源，搶走「Speclink 是什麼」的位置。
- 9 張截圖拍於 2026-10-08。之後桌面 app 換了視窗外框、詳情頁改成三欄整頁、設定改成設定模式，server 後台換成與桌面一致的外殼、列表頁與詳情頁，截圖已不是現在的樣子；中英文件共用一組中文介面截圖，英文讀者看到的是中文畫面。
- 缺少開源專案常見的 CONTRIBUTING、SECURITY 與 Issue／PR 範本，外部貢獻者不知道怎麼回報問題、怎麼送 PR、漏洞該私下回報到哪裡。

規格掃描結果：`user-documentation` 規範 README 首段內容、中英成對檔名與「兩版共用同一組截圖」，`ui-copy-vocabulary` 以檔名列出 README 的詞彙守門面；兩者都要跟著改。`desktop-release`（SmartScreen 放行說明由 README 承載）與 `delivery-baseline`（README.md 含安裝腳本 raw 網址）的條文在新版 README 仍成立，不需修改。

## Proposed Solution

- **README 語言主版對調**：英文版改為 `README.md`，繁中版改名 `README.zh-TW.md`，語言切換、全部交叉連結與文件守門測試跟著改。docs 底下本來就是「英文無後綴、繁中 `.zh-TW`」的命名，對調後 README 與 docs 一致。
- **README 改成開源首頁結構**：Logo／標語／徽章／語言切換 → 白話介紹與主截圖 → 為什麼用 Speclink → 三步快速開始 → 截圖展示 → 特色 → 工作流程 → 精簡安裝 → 自架 server → 文件地圖 → 專案狀態 → 參與貢獻與安全回報 → 致謝 → 授權。
- **安裝細節下沉**：README 的安裝段只留桌面三平台表格、CLI 三條一行指令、Windows 與無圖形介面 Linux 的指引與 SmartScreen 提示；「桌面 app 覆蓋 CLI」的逐平台說明與安裝腳本環境變數表搬到 getting-started 的安裝節，README 以連結導過去。
- **起源移到致謝**：Spectra App 2.3.1 的行為參考起源從首段移到「致謝」段，同段致謝 OpenSpec（Local 模式沿用其目錄結構）。
- **開源社群檔案**：新增英文的 `CONTRIBUTING.md`（開發環境、測試、commit 規範、以 Speclink 變更流程送 PR）、`SECURITY.md`（以 GitHub 私下回報漏洞，不公開信箱）、Issue 範本（bug 回報、功能建議）與 PR 範本。不新增 CODE_OF_CONDUCT。
- **docs 過時描述修正**：逐份對照現行 `apps/desktop`、`apps/server-web` 與 `packages/ui`，修正 10/8 之後改版造成的介面描述（設定模式、三欄詳情頁、手冊頁、後台詳情頁與對話框等）與截圖說明文字。
- **截圖中英各一組**：同名截圖分放 `docs/assets/screenshots/en/` 與 `docs/assets/screenshots/zh-TW/`，英文文件引用英文介面、繁中文件引用中文介面。桌面 7 張（看板、變更詳情、討論詳情、規格詳情、已封存、手冊、設定模式）、後台 4 張（初始設定、總覽、使用者詳情與成員資格、專案詳情）。舊的 9 張平放截圖移除。
- **截圖隱私防護**：只擷取 app 視窗；桌面狀態由 `scripts/docs/docs-screenshots.mjs` 移開再還原；示範專案改用中性名稱並補齊四欄看板與手冊頁；後台用暫存目錄的一次性 server 與 example.com 帳號；畫面不得出現使用者名稱、家目錄路徑、真實信箱、主機名稱、真實專案、存取金鑰明文與 setup token；每張圖逐張檢查畫面並清掉 PNG 文字中繼資料。

## Non-Goals

（design.md 會建立，範圍排除寫在 design 的 Goals / Non-Goals。）

## Alternatives Considered

- **維持 README.md 為繁中版**：國際讀者第一眼仍是中文，與開源慣例相反；使用者已選英文為主。
- **中英文件共用一組英文介面截圖**：張數減半，但繁中讀者看到英文畫面；使用者已選中英各一組。
- **以檔名後綴區分語言（`desktop-board.zh-TW.png`）**：與 docs 命名一致，但 22 張平放在同一目錄難以一眼分組，守門測試也要解析後綴；改用語言子目錄、檔名相同。

## Impact

- Affected specs: `user-documentation`（README 結構與首段、中英成對檔名、截圖分語言與隱私、安裝衝突說明的落點、開發者文件導流、新增開源社群檔案需求）、`ui-copy-vocabulary`（詞彙守門面的 README 檔名）
- Affected apps／crates：不改任何 crate 與 app 程式碼；只改文件、文件資產與 `scripts/docs/` 的文件守門與截圖場景腳本。CLI 指令、人眼輸出與 `--json` 皆不變，沒有相容性影響。生成的技能檔不變。
- Affected code:
  - New:
    - `README.zh-TW.md`
    - `CONTRIBUTING.md`
    - `SECURITY.md`
    - `.github/ISSUE_TEMPLATE/bug_report.yml`
    - `.github/ISSUE_TEMPLATE/feature_request.yml`
    - `.github/ISSUE_TEMPLATE/config.yml`
    - `.github/PULL_REQUEST_TEMPLATE.md`
    - 英文介面截圖目錄與繁中介面截圖目錄底下各 11 張 PNG（docs 截圖目錄的 en 與 zh-TW 子目錄）
  - Modified:
    - `README.md`
    - `docs/getting-started.md`
    - `docs/getting-started.zh-TW.md`
    - `docs/workflow.md`
    - `docs/workflow.zh-TW.md`
    - `docs/configuration.md`
    - `docs/configuration.zh-TW.md`
    - `docs/remote-getting-started.md`
    - `docs/remote-getting-started.zh-TW.md`
    - `docs/server-deployment.md`
    - `docs/server-deployment.zh-TW.md`
    - `docs/product-status.md`
    - `docs/product-status.zh-TW.md`
    - `docs/roadmap.md`
    - `docs/roadmap.zh-TW.md`
    - `docs/development.md`
    - `docs/development.zh-TW.md`
    - `docs/server-backup.md`
    - `docs/server-backup.zh-TW.md`
    - `docs/server-store-drivers.md`
    - `docs/server-store-drivers.zh-TW.md`
    - `docs/sdk-node.md`
    - `docs/sdk-node.zh-TW.md`
    - `docs/verb-contract.md`
    - `docs/verb-contract.zh-TW.md`
    - `scripts/docs/docs-screenshots.mjs`
    - `scripts/docs/docs-screenshots.test.mjs`
    - `scripts/docs/docs-parity.test.mjs`
    - `scripts/docs/docs-links.mjs`
    - `scripts/docs/docs-links.test.mjs`
    - `scripts/docs/remote-docs.test.mjs`
    - `scripts/docs/vocabulary-guard.test.mjs`
  - Removed:
    - `README.en.md`
    - `docs/assets/screenshots/desktop-board.png`
    - `docs/assets/screenshots/desktop-change-drawer.png`
    - `docs/assets/screenshots/desktop-spec.png`
    - `docs/assets/screenshots/desktop-discussion.png`
    - `docs/assets/screenshots/desktop-archived.png`
    - `docs/assets/screenshots/desktop-settings.png`
    - `docs/assets/screenshots/server-setup.png`
    - `docs/assets/screenshots/server-overview.png`
    - `docs/assets/screenshots/server-members.png`

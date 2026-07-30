---
topic: GitHub 一鍵安裝 CLI+desktop、clone 後一鍵執行各元件、與對應說明文件
slug: one-click-install-and-run
status: promoted
promoted_to: desktop-installer-and-updater, dev-quickstart-and-docs
created: 2026-07-30
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: GitHub 一鍵安裝 CLI+desktop、clone 後一鍵執行各元件、與對應說明文件

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者提出三個相關題目：(1) GitHub 上供人下載的 CLI+desktop 一鍵安裝檔；(2) clone 專案後除 npm run dev 外，能單獨一鍵執行後端 server、desktop 或 CLI；(3) 所有一鍵執行方式都要有說明文件。模式選 assumptions：release.yml（已發 CLI+server 五平台+Docker，desktop 明文排除）、scripts/dev.mjs（server+tauri dev 同起編排器）、scripts/cli.mjs（checkout debug CLI wrapper）、docs/ 雙語慣例，程式碼脈絡充足。相關前情：keychain-always-allow-reprompt（已封存）的 Deferred 明訂「正式發行簽章策略留待發行規劃」——本討論即該規劃。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-30)

**Focus**: 三個題目的實際缺口盤點與方案方向
**Position**: 五項假設經使用者全數確認：
- 缺口比表面小——release.yml 已發 CLI+server（五平台、checksums、Docker），npm run dev 已是一鍵編排，雙語文件慣例已存在；真正缺的是 desktop 不是 release 產物、dev harness 無單獨模式、docs 無 clone 開發者路徑
- (1) 安裝檔：Tauri bundler 掛進現有 release.yml，desktop 升格 release 產物（.dmg/.msi/.AppImage）；「CLI 進 PATH」用 desktop 內建安裝動作（VS Code 模式）抹平 .dmg 拖放裝不了 PATH 的平台差異
- (2) 一鍵執行：擴充 npm scripts——dev:server（只起後端、開箱 sqlite）、dev:desktop（只起 tauri dev，desktop 本地模式不需 server）、npm run cli 缺 binary 時自動 cargo build（現為報錯提示，cli.mjs:44-46）
- (3) 文件：新開 docs/development.md + development.zh-TW.md 雙語對，README 加節連結；getting-started 是產品使用文件，受眾不同不混用
**Ruled out**: justfile/Makefile（多一套工具鏈，違反一切走 npm scripts 的慣例）；開發路徑塞 README 或 getting-started（過長／受眾混淆）；CLI 與 desktop 分開下載（「一鍵」只剩一半）
**Open**: 簽章投資的必要性——開源專案是否值得 Developer ID + notarization 的成本？

### Round 2 — assumptions (2026-07-30)

**Focus**: 開源專案是否值得投資簽章（Developer ID + notarization）？
**Position**: 現階段不投資，但 CI 把簽章做成可插鑰匙的開關：
- 受眾是開發者，對「未簽章開源軟體手動放行」有預期也有能力（Sequoia：系統設定 > 隱私權與安全性；Windows：SmartScreen 仍要執行）
- 下載用戶的鑰匙圈痛感遠低於開發機：二進位在升級之間不變，體感是「每次升級後重授權幾次」而非每次呼叫都跳；純本地看板用戶完全無感
- 專案 0.1.0、尚無下載用戶，先付 $99/年＋notarize 管線是為不存在的需求付錢（YAGNI）
- 保險：release workflow 簽章步驟 env-gated（secrets 在才簽），未來升級是「填鑰匙」不是「改管線」；文件寫明放行步驟
- 觸發重估條件：出現非開發者用戶、或 remote 模式成為主打體驗；Windows 可接 SignPath.io（開源免費）或 Azure Trusted Signing（約 $9.99/月），macOS 無免費路
**Ruled out**: 現在購買 Developer ID——成本先於需求
**Open**: desktop+CLI 自動更新是否可行、需不需要伺服器？

### Round 3 — assumptions (2026-07-30)

**Focus**: desktop+CLI 自動更新的可行性與形態
**Position**: 可行，GitHub Releases 即靜態更新端點，無需自架伺服器：
- tauri-plugin-updater 查 releases/latest/download/latest.json（CI 產安裝檔時一併附上）
- 更新完整性用 Tauri 自產 minisign 金鑰對（免費）：私鑰進 GH secrets 簽更新包、公鑰嵌 app——與 OS 程式碼簽章正交，不買 Developer ID 也有安全更新
- CLI 更新搭便車：「安裝 CLI 到 PATH」做成 symlink 指進 app bundle（VS Code 的 code 指令模式），desktop 更新即 CLI 更新；Windows NSIS 更新重跑安裝器，CLI 一起換
- 既知代價：未簽章＋自動更新 → remote 用戶每次更新後鑰匙圈重授權幾次（前輪已接受的取捨，寫入文件）
**Ruled out**: 自架 update server——只有分階段灰度、更新統計、私有 repo 才需要，均不在情境內；CLI 獨立 self-update 指令——symlink 模式下是多餘機制
**Open**: 無

## Conclusion

**Decision**: desktop 以 Tauri bundler 進 release.yml 產三平台安裝檔（未簽章起步，簽章為 env-gated 開關）；內建自動更新（tauri-plugin-updater + GitHub Releases latest.json + 自產 minisign 金鑰）；CLI 進 PATH 以 symlink 指向 app bundle、隨 desktop 更新（Windows 隨 NSIS 重裝）；npm scripts 新增 dev:server／dev:desktop、npm run cli 缺 binary 時自動建置；新開 docs/development.md + development.zh-TW.md 雙語對，涵蓋各一鍵入口與未簽章放行教學，README 加節連結。
**Rationale**: 受眾是開發者且專案尚無下載用戶，簽章成本先於需求（YAGNI）；GitHub Releases 靜態端點已滿足發行與自動更新，不自架伺服器；更新完整性由免費的 minisign 簽名保障，與 OS 簽章正交。
**Rejected alternatives**: 現購 Developer ID（成本先於需求）；justfile/Makefile（多一套工具鏈）；CLI 與 desktop 分開下載（一鍵只剩一半）；自架 update server（灰度／統計／私有 repo 才需要）；CLI 獨立 self-update（symlink 模式下多餘）；開發路徑塞 README 或 getting-started（受眾混淆）。
**Deferred**: 簽章投資的觸發時機（非開發者用戶出現、或 remote 模式成主打）；屆時 Windows 優先接 SignPath.io（開源免費）／Azure Trusted Signing，macOS 走 Developer ID + notarization。
**Capture to**: proposal（扇出兩個 change：① desktop 安裝檔＋自動更新管線；② 一鍵執行 scripts＋開發文件）
**Next**: /speclink-propose --from-discussion one-click-install-and-run（或 speclink discuss promote one-click-install-and-run --name <change-name>，promote 兩次扇出兩個 change）

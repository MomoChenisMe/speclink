## Why

維護者在本機用 node scripts/desktop/desktop-install.mjs --install 重裝 Speclink 之後，macOS 會一再跳出鑰匙圈詢問框：「speclink」想使用鑰匙圈「speclink-desktop」裡的機密資訊。原因是這支腳本建出來的 app 與內含的 CLI 只有 ad-hoc 簽章，簽章身分帶著隨程式內容變動的雜湊（例如 `speclink-9973c961f6bb5a30`），每次重建都是 macOS 沒見過的程式。CLI 與 Speclink Desktop 共用的遠端登入憑證（`remote-auth` 與 `desktop-connections` 規定存在 OS 鑰匙圈的 `speclink-desktop` 服務底下）因此每個項目都要重新允許；按「允許」只放行一次讀取，而每個 CLI 指令都是新的程序，定期呼叫 CLI 的工具會讓詢問框不斷出現。

正式發版的 app 由 Release 工作流以 Developer ID 簽章並公證，沒有這個問題；只有本機安裝這條路是 ad-hoc。維護者的鑰匙圈裡通常已經有同一張「Developer ID Application」憑證，本機安裝可以直接用它，讓每次重建的簽章身分都一樣。

目標使用者是在本機建置並安裝 Speclink 的維護者：發版技能 `/release` 收尾的「重裝本機 app」就是執行這支腳本。連帶受益的是在同一台電腦上用 CLI 或 Speclink Desktop 連遠端專案的人。

## What Changes

- 本機安裝腳本在 macOS 上建置之前先決定簽章身分：環境變數 `APPLE_SIGNING_IDENTITY` 已設定時用它；沒有設定時，從 `security find-identity -v -p codesigning` 的輸出找第一張「Developer ID Application」憑證，找到就交給 `tauri build`。Tauri 用它簽 app 與 bundle 內的 sidecar CLI。
- 找不到憑證時照現在的 ad-hoc 簽章建置，並印一行說明（重裝後鑰匙圈會重新詢問）；建置不因此失敗。
- 用了簽章身分時，bundle 的版號斷言之後多一道簽章斷言：bundle 內的 CLI 必須帶 Team ID、不是 ad-hoc；不符時印出實際的簽章、以非零結束，不進行安裝。
- 只簽章、不公證：本機建置不經下載，不會被 Gatekeeper 的隔離檢查攔下，所以不需要 Apple ID 與公證。
- `desktop-release` 規格的「本機安裝的新鮮度斷言」加入上述簽章步驟與情境。

相容性影響：腳本的指令、旗標與 exit code 不變，`/release` 收尾的重裝指令不用改。改完後第一次重裝，鑰匙圈還會對每個項目再問一次（簽章身分從 ad-hoc 換成 Developer ID），按「永遠允許」之後，之後的重裝不再詢問。第一次用這張憑證簽章時，macOS 也可能先問一次 codesign 能不能使用它的私鑰。

三種形態：這是 repo 自用的建置腳本，不是引擎命令、動詞或技能步驟。本機 repo、連到遠端的 checkout、沒有本機規格檔的內嵌宿主三種形態的產品行為都不變。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `desktop-release`: 「本機安裝的新鮮度斷言」加入建置前的簽章身分選擇、找不到憑證時的說明，以及簽章結果的斷言。

## Impact

- Affected specs: `desktop-release`
- Affected code:
  - Modified:
    - scripts/desktop/desktop-install.mjs
    - scripts/desktop/desktop-install.test.mjs
  - New: （無）
  - Removed: （無）
- 影響的 app：apps/desktop 只有本機建置的簽章方式改變，原始碼與 Tauri 設定檔都不動。官方 Release 工作流（.github/workflows/release.yml）、CLI 與引擎都不受影響。

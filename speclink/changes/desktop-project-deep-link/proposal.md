## Why

Herdr 面板（開發者在 Herdr 終端機裡看專案進度的側欄）左上角的 Logo 現在只叫出 Speclink 桌面 app，app 會停在原來的專案。開發者從某個 space 按 Logo，想看的是這個 space 的專案，卻得再到 app 的分頁列切一次；面板上會切專案的路徑只有「路線圖」鈕和點變更或討論列，這些都會多開一個頁面。原因是桌面 app 的 speclink 網址只有 change、discussion、roadmap 三種，沒有「只切專案」的目標。

目標使用者是用 AI agent 開發、同時開著 Herdr 與桌面 app 的開發者。使用情境不綁定 workflow 階段或技能：任何階段想從 Herdr 回到桌面 app 看板時都會用到。

## What Changes

- 桌面 app 新增第四種網址 `speclink://project?project=<專案資料夾絕對路徑>`：切到該專案並顯示看板的卡片檢視，關掉開著的詳情頁與可取消的浮層，不開詳情頁也不開路線圖。喚起主視窗、開專案的探測流程、未初始化或帶 remote 標記時走既有對話框、不可取消浮層開著時不切換、只開啟與檢視不寫入，都與現有三種網址相同。
- Herdr 面板的 Logo 改成依狀況二選一：
  - 讀得到目前 space 的專案、而且專案在這台機器上：送 `speclink://project`，切到該專案。
  - 專案沒初始化、還讀不到專案，或 space 在別台機器上：維持只叫出 app。
- 舊版 app 的處理分兩種：
  - 沒有註冊 speclink 網址的版本（v0.11.0 以前的發佈版都是）：網址送不出去（open 指令結束碼 1），Logo 自動退回只叫出 app，行為與現在相同。
  - 註冊了網址但不認得 `project` 的版本（2026-11-03 之後、本變更之前自行建置的版本）：app 一樣會被叫到前景，專案不切換，並跳出「無法開啟這個 speclink 網址」的提示。外部程式分辨不出這種版本，所以寫進 README，更新 app 後就不會再出現。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `desktop-app`：「以 speclink 網址開啟詳情頁」新增 `project` 種類，定義它的格式、切換後的畫面與解析範例。

## Impact

- Affected specs: `desktop-app`（修改）
- 規格掃描結果：相關的正式規格是 desktop-app（網址的格式與派送）與 desktop-roadmap（看板頁的兩種檢視）。desktop-roadmap 的「維持上次選的檢視」講的是使用者切分頁或切頁，網址明確指定檢視屬於另一條路徑，現有的路線圖網址也是如此，所以這份規格不用改。Herdr 面板沒有對應的正式規格，Logo 的行為由 README 與測試記錄。
- 影響的 app：apps/desktop、integrations/herdr。不動 crate、CLI 動詞、設定欄位與生成的技能。
- 相容性影響：
  - Logo 在已初始化的本機專案上，會多做切專案、切到卡片檢視、關掉詳情頁這幾件事。其他情況維持只叫出 app。
  - 現有三種網址的格式與行為不變。`speclink://project` 在舊版 app 上的結果如 What Changes 所列；使用者要更新桌面 app 才能切專案，不用做其他遷移。
- Affected code:
  - Modified:
    - `apps/desktop/src/deepLink.ts`
    - `apps/desktop/src/store.ts`
    - `apps/desktop/src/__tests__/deepLink.test.ts`
    - `apps/desktop/src/__tests__/store.test.ts`
    - `integrations/herdr/mods/speclink/main.js`
    - `integrations/herdr/mods/speclink/model.js`
    - `integrations/herdr/README.md`
    - `integrations/herdr/README.en.md`
    - `scripts/herdr/panel-model.test.mjs`
    - `docs/product-status.md`
    - `docs/product-status.zh-TW.md`
  - New: （無）
  - Removed: （無）

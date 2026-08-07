## Why

在本機迭代 Desktop 前端的開發者（apps/desktop 的 React 介面），執行 npm run dev 後於 dev 視窗看到的並不是當前 checkout 的前端，而是「上一次 Rust 重編那一刻」的舊快照。這讓最基本的開發迴圈失效：改了畫面、存檔、重啟 dev，畫面仍然沒變。

根因已於 2026-08-07 在本機實測確認，分為兩段：

1. **前端在編譯期被嵌入 Rust binary**。`apps/desktop/src-tauri/tauri.conf.json` 的 build 區塊只有 `frontendDist`，沒有 devUrl，因此 dev 模式與 release 模式走同一條路——Tauri 的 context 產生巨集在編譯 `apps/desktop/src-tauri` 時把 `apps/desktop/dist` 整包嵌進 `target/debug/speclink-desktop`。對該 binary 取字串可撈到 dist 的 hash 化資產檔名，即為證據。
2. **Cargo 的重編判定看不到 dist**。`target/debug/build/speclink-desktop-*/output` 中由 build script 宣告的重編觸發清單，只涵蓋 `apps/desktop/src-tauri/tauri.conf.json`、capabilities 目錄、sidecar binary 與 localizations 三類，並不包含前端產物目錄。

兩者相乘的結果是：純改前端時 Cargo 判定無需重編，直接執行既有 binary，視窗載入的是舊快照。`scripts/dev.mjs` 的前置步驟每次都確實重建了 dist，但那份 dist 進不了視窗，等於白建。

此缺陷具有間歇性外觀，因而長期未被定位：sidecar binary 在重編觸發清單內，所以每當開發者跑過 sidecar 佈署腳本，就會連帶觸發重編並把新前端一併嵌入，看起來像是「有時候會好」。

## What Changes

- `apps/desktop/src-tauri/tauri.conf.json` 的 build 區塊新增 devUrl 與 beforeDevCommand 兩個欄位，讓 dev 模式改由 Vite dev server 供應前端，脫離編譯期嵌入路徑。frontendDist 保留不動，release 與 bundle 仍走原本的靜態產物。
- `apps/desktop/vite.config.ts` 新增 dev server 設定，固定連接埠並開啟嚴格模式。目前該檔完全未設定 server 區塊，Vite 會採用預設埠且在被占用時自動遞增，與 tauri.conf.json 寫死的 devUrl 對不上，故此項為 devUrl 生效的必要前提。
- `scripts/dev.mjs` 的前置步驟中，移除 desktop 前端建置。該步驟在新路徑下由 beforeDevCommand 取代，保留會造成每次啟動多一次無用的完整建置。
- `scripts/dev.test.mjs` 同步更新既有的前置步驟斷言（現有兩處斷言前端建置指令）。

**相容性影響**：npm run dev 與 npm run dev:desktop 的終端輸出會變——原本啟動前的一次性前端建置訊息消失，改為 Vite dev server 的常駐輸出。兩者的 exit code 語意、Ctrl+C 同時收束兩個 child process 的行為、以及 CLI 建置失敗即拒絕啟動的守門，皆維持不變。不涉及任何 speclink CLI 子指令、旗標或 --json 輸出，故無 golden 對照影響。開發者無需遷移動作。

## Non-Goals

- **不處理 dev 與安裝版的資料隔離**。兩者共用 Tauri identifier 因而共用同一份 app 設定目錄（connections.json 與 tabs.json 互相覆寫），且 `~/.local/bin/speclink` 是指向安裝版 app 內部的符號連結——使用者已明確表示本次不納入。
- **不改動 release 與 bundle 路徑**。不新增 beforeBuildCommand：`scripts/desktop-install.mjs` 現行是先手動建置前端再呼叫 bundle，加上該欄位會造成重複建置。release 路徑維持零改動。
- **不觸碰 apps/server-web**。該 app 的前端產物有各自獨立的過期問題（server 於 debug 模式動態讀取磁碟上的產物目錄），與本次的編譯期嵌入根因無關。
- **不改由 build script 宣告 dist 為重編觸發來源**。見下方替代方案。

## Alternatives Considered

- **在 build script 中把前端產物目錄加入重編觸發清單**：能讓改前端觸發重編，但每次改一行 CSS 都要重編整顆 Rust binary（debug 產物逾 80 MB），迴圈時間以分鐘計。治標且代價高。
- **由 scripts/dev.mjs 在建置前端後主動觸碰一支 Rust 原始檔以強制重編**：同樣要付整趟重編成本，且以偽造檔案時間戳操縱建置系統，屬於難以維護的取巧手法。
- **維持現狀，靠文件告知開發者手動強制重編**：把已知缺陷轉嫁為每位開發者的記憶負擔，且無法防止再次誤判為「程式沒生效」。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `dev-harness`: 「一鍵啟動 remote 開發環境」需求中，npm run dev 於 CLI 建置後「建置 Desktop 前端」的編排改為由 tauri dev 啟動 Vite dev server 供應前端；並新增需求，要求 dev 模式下前端變更不需重編 Rust 即可生效。

## Impact

- Affected specs: `dev-harness`
- Affected code:
  - Modified:
    - `apps/desktop/src-tauri/tauri.conf.json`
    - `apps/desktop/vite.config.ts`
    - `scripts/dev.mjs`
    - `scripts/dev.test.mjs`
  - New: (none)
  - Removed: (none)
- Affected app：僅 `apps/desktop`（Tauri 殼設定與前端建置設定）與 repo root 的開發編排腳本。不影響任何 Rust crate 的原始碼，不影響 `apps/server-web`。
- 設定欄位：本次變更的是 Tauri 與 Vite 的建置設定檔，不涉及 `openspec/config.yaml`、`.speclink.yaml` 或 SPECLINK_ 環境變數三層設定中的任何欄位。
- 技能與注入區塊：無影響，不改動任何 speclink 技能或 CLAUDE.md / AGENTS.md 注入內容。

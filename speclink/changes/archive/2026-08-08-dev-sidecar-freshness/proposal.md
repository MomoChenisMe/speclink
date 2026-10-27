## Problem

本機迭代的開發者（dev-harness 工作流的使用者）面對兩個現象：

1. **全新 clone 開不了 dev 視窗**。在沒有 gitignored 產物的全新 checkout 執行 npm run dev（或 npm run dev:desktop、直接於 apps/desktop 跑 tauri dev），speclink-desktop 的編譯以「resource path binaries/speclink-<triple> doesn't exist」硬失敗——dev-harness 正典場景「全新 checkout 且未安裝 CLI 仍可啟動」實測不成立。此落差在 desktop-dev-frontend-hmr 的驗證輪帶保留接受，本 change 即其根治。
2. **過期 sidecar 主動污染新 CLI**。修改 crates/ 後 sidecar 不會更新；更糟的是 tauri-build 的行為讓過期不只是被動的：speclink-desktop 的 build script 每次執行都把 apps/desktop/src-tauri/binaries/speclink-<triple> 剝掉平台後綴、先刪後複製成 target/debug/speclink——正是 npm run cli 所用、dev 編排剛建好的那顆。過期 sidecar 會蓋掉新 CLI（2026-08-08 實讀 tauri-build 2.6.3 的 copy_binaries 原始碼確認），足以造成「明明重建了行為卻還是舊的」的誤判。dev 視窗的「安裝 CLI」功能也會佈出舊版。

## Root Cause

sidecar（apps/desktop/src-tauri/binaries/speclink-<triple>）是 gitignored 產物，但全 repo 只有本機安裝流程（scripts/desktop-install.mjs 呼叫 scripts/desktop-sidecar.mjs）會佈署它——dev 路徑沒有任何一環維持它的存在與新鮮。而 speclink-desktop 的編譯硬性要求該檔存在（Tauri externalBin 的資源檢查），其 build script 又會把該檔複製到 target/debug/ 供 dev 期取用，於是「沒人佈」在全新 checkout 成為硬失敗、在既有 checkout 成為靜默污染源。

## Proposed Solution

依討論 dev-sidecar-freshness 的結論：

- **擴充 scripts/desktop-sidecar.mjs**：新增 --profile 參數（debug｜release，無參數維持 release 與現行為完全一致），並加入「內容相同即跳過複製」的防抖——binaries/ 在 cargo 的重編觸發清單內，無條件覆蓋會讓每次 dev 啟動都多一輪 speclink-desktop 重編。
- **apps/desktop/package.json 新增 npm predev hook** 呼叫該腳本佈 debug sidecar。tauri dev 的 beforeDevCommand 是 npm run dev（於 apps/desktop 執行），npm 對 dev 會自動先跑 predev——因此所有 dev 視窗入口（repo root 的 npm run dev、npm run dev:desktop、直接跑 tauri dev）都在 vite 與 Rust 編譯開始前佈好當前 checkout 的 sidecar，scripts/dev.mjs 的編排零改動，剛封存的 dev-harness 規格與 dev.test.mjs 測試不必動。
- dev 佈 debug 版與 npm run cli 驗證所用同一顆，內容一致；本機安裝與 CI 的 release／交叉編譯路徑零改動。

## Success Criteria

- 全新 checkout（無 binaries/）執行 npm run dev:desktop：sidecar 於編譯前自動建置並佈署，dev 視窗正常開啟，不因缺檔失敗。
- 修改 CLI 相關原始碼後啟動任一 dev 入口：佈署後的 sidecar 與 target/debug/speclink 內容一致，皆為當前 checkout 的建置結果。
- CLI 未變動時連續啟動：第二次啟動不改寫 sidecar 檔案，speclink-desktop 不因 sidecar 觸發重編。
- 對 scripts/desktop-sidecar.mjs 傳入未知的 --profile 值：以非零狀態結束並輸出點名該值的錯誤訊息；predev 失敗時 dev 啟動中止、視窗不開。
- release 未回歸：scripts/desktop-install.mjs 內容不變，無參數執行 scripts/desktop-sidecar.mjs 仍為 release 建置與佈署。
- 驗證入口：node --test scripts/desktop-sidecar.test.mjs 與 node --test scripts/dev.test.mjs 全數通過。

## Impact

- Affected specs: `dev-harness`（ADDED 一條需求：dev 啟動自動佈署當前 checkout 的 sidecar；既有場景「全新 checkout 且未安裝 CLI 仍可啟動」因此成真，其文字不需修改）
- Affected code:
  - Modified: `scripts/desktop-sidecar.mjs`（加 --profile 與防抖，重構為可測形狀）、`apps/desktop/package.json`（加 predev script）、`scripts/dev.test.mjs`（加 predev 設定守門）
  - New: `scripts/desktop-sidecar.test.mjs`
  - Removed: (none)
- Affected app：僅開發編排腳本與 `apps/desktop` 的 npm scripts。零 Rust 改動，不涉任何 crate。
- 相容性影響：speclink CLI 的子指令、旗標、人眼輸出與 --json 皆無涉，無 golden 對照影響。npm run dev 的終端輸出多一段 predev 的 sidecar 佈署訊息；desktop-install.mjs 無參數呼叫 desktop-sidecar.mjs 的行為必須與現行完全一致（release 建置＋佈署），此為回歸保護對象。
- 設定欄位：不涉 openspec/config.yaml、.speclink.yaml 或 SPECLINK_ 環境變數。
- 技能與注入區塊：無影響。

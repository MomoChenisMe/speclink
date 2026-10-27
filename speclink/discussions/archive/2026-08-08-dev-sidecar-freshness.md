---
topic: dev 的 sidecar 新鮮度——過期 sidecar 蓋掉新 CLI、全新 clone 編譯失敗
slug: dev-sidecar-freshness
status: promoted
promoted_to: dev-sidecar-freshness
created: 2026-08-08
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: dev 的 sidecar 新鮮度——過期 sidecar 蓋掉新 CLI、全新 clone 編譯失敗

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：desktop-dev-frontend-hmr 的 verify Round 1 帶保留蓋章（全文在 commit 5a76ad9）留下兩個未根治的既有落差——(1) 全新 clone 跑 npm run dev 因缺 gitignored 的 apps/desktop/src-tauri/binaries/speclink-<triple> 而編譯硬失敗，spec 場景「全新 checkout 且未安裝 CLI 仍可啟動」實測不成立；(2) 改 crates/ 後 sidecar 不會更新，全 repo 只有 scripts/desktop-install.mjs 會呼叫 scripts/desktop-sidecar.mjs 佈署。使用者要求討論解決方式。

模式：assumptions——相關程式碼充足（scripts/dev.mjs、scripts/desktop-sidecar.mjs、scripts/desktop-install.mjs、apps/desktop/src-tauri/src/cli_install.rs、tauri.conf.json），且本輪 scout 直接讀了 tauri-build 2.6.3 原始碼驗證關鍵行為。

Scout 關鍵發現（改變問題定性）：tauri-build 的 copy_binaries()（~/.cargo/registry .../tauri-build-2.6.3/src/lib.rs）在 speclink-desktop build script 每次執行時，把 binaries/speclink-<triple> 剝掉平台後綴、先刪後複製成 target/debug/speclink——正是 npm run cli 與 dev.mjs 建置目標的那顆。過期 sidecar 因此不只被動過期，會主動蓋掉剛建好的新 CLI。另確認：dev 視窗引擎為直嵌（desktop-app spec），tauri dev watch 涵蓋全部 crates，dev 視窗內的動詞邏輯不受 sidecar 過期影響；cli_install.rs 的 bundled_cli_path() 取主執行檔同目錄的 speclink，dev 模式下即 target/debug/speclink。

相關 change：desktop-dev-frontend-hmr（已封存 2026-08-08，本題是它的 verify 保留事項）；cli-render-unification（進行中，觸碰面為 crates 與 CLI 規格，與本題無檔案交集）。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-08)

**Focus**: sidecar 過期的實際傷害面在哪、修法落點選哪裡
**Position**: 五項假設一次提出，使用者全數確認——擴充 desktop-sidecar.mjs 走 npm predev hook 佈 debug sidecar：
- 傷害面重新定界：dev 視窗引擎直嵌＋tauri dev watch crates/，視窗內邏輯不會舊；真正的傷害是 (a) 全新 clone 編譯硬失敗 (b) dev 視窗「安裝 CLI」功能佈出舊版 (c) tauri-build copy_binaries 會把過期 sidecar 蓋到 target/debug/speclink，主動污染 npm run cli 用的那顆（tauri-build-2.6.3 lib.rs 實讀驗證，先 remove 再 copy）
- 單一實作落點：擴充 scripts/desktop-sidecar.mjs 加 debug profile 支援，dev 與 install 共用同一支（規則：同一契約不得平行實作兩套）
- 掛載點：apps/desktop package.json 的 npm predev hook（beforeDevCommand 維持 npm run dev 不變）——覆蓋所有 tauri dev 入口（npm run dev、dev:desktop、直接 tauri dev），scripts/dev.mjs 零改動，剛封存的 dev-harness 規格與 dev.test.mjs 測試不必動
- 防抖：內容相同即跳過複製——binaries/ 在 cargo rerun-if-changed 清單內，無條件覆蓋會讓每次 dev 啟動多一輪 speclink-desktop 重編（~3s）＋一次 clobber 複製；dev 佈 debug 版與 npm run cli 同顆內容，一致性剛好
- 全新 clone 順帶根治：sidecar 在 vite 起來前就位，tauri dev 編譯不再缺檔；desktop-install.mjs 與 CI --target 交叉編譯路徑零改動
**Ruled out**: 掛 dev.mjs prerequisite——覆蓋不到直接跑 tauri dev 的入口，且「單獨啟動 desktop」規格要再 MODIFIED 一次；build script 造 placeholder 假檔——違反專案 fail-closed 哲學，dev 視窗的安裝功能會佈出垃圾 binary；dev 佈 release 版 sidecar——重複付一次 release 建置成本，且與 npm run cli 用的 debug 顆不一致
**Open**: 無——五項假設全數確認，收斂至結論

## Conclusion

**Decision**: 擴充 scripts/desktop-sidecar.mjs 支援 debug profile 與「內容相同即跳過」防抖；apps/desktop package.json 加 npm predev hook 呼叫它——所有 tauri dev 入口（npm run dev、dev:desktop、直接 tauri dev）在啟動前自動佈署當前 checkout 的 debug sidecar。release、desktop-install.mjs 與 CI 交叉編譯路徑零改動。
**Rationale**: 三個傷害面（全新 clone 編譯硬失敗、dev 安裝功能佈舊 CLI、過期 sidecar 經 tauri-build copy_binaries 蓋掉 target/debug/speclink 污染 npm run cli）共用同一個根因——binaries/ 的 sidecar 沒人維持新鮮。predev hook 落點讓覆蓋面最大且編排零改動：剛封存的 dev-harness 規格與 dev.test.mjs 都不必動；內容比對防抖避免每次啟動觸發 speclink-desktop 重編。單一實作落點守住「同一契約不得兩套實作」。
**Rejected alternatives**: 掛 dev.mjs prerequisite（覆蓋不到直接 tauri dev，規格要再改）；build script 造 placeholder（違反 fail-closed，會佈出垃圾 CLI）；dev 佈 release sidecar（多付一次 release 建置、與 npm run cli 的 debug 顆不一致）。
**Deferred**: spec delta 的形狀——「全新 checkout 且未安裝 CLI 仍可啟動」場景因此成真即可，或另 ADDED 一條 sidecar 新鮮度需求，由 propose 階段決定；Windows 上 predev hook 的行為屬 npm 標準、預期無事，apply 時實測即可。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion dev-sidecar-freshness

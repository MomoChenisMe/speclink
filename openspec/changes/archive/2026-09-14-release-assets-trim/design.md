## Context

發版管線（`.github/workflows/release.yml`，由 `/release` 技能壓 tag 觸發）目前把三類檔案都附在同一個 GitHub Release：給人下載的安裝檔（7 個）、桌面 app 自動更新用的檔案（latest.json、.app.tar.gz×2、.sig×5）、CLI 安裝通路用的檔案（五 target 壓縮檔＋SHA256SUMS.txt）。GitHub Release 頁依檔名排序且沒有隱藏 asset 的功能，機器讀的檔案把安裝檔擠到第 12 位。

既有可沿用的基礎：`@speclink/server` 的 npm 物化流程（`packages/server-npm`＋`scripts/npm/npm-server-package.mjs`：主套件＋五個平台子套件 optionalDependencies、發布時從 build artifacts 物化、NPM_TOKEN 缺席跳過、同版已在 registry 即跳過）；`@speclink/engine` 發布 job 的「先 npm pack 再 npm publish tgz、等 registry 可見」寫法；Tauri CLI 的 `--target universal-apple-darwin`（需同時安裝 aarch64 與 x86_64 兩個 rustup target，build job 已在 arm runner 交叉編譯 x86_64）；tauri-bundler 對 universal 建置要求 sidecar 為 universal binary 且檔名帶 `-universal-apple-darwin` 後綴。

邊界：本變更落在發版管線、`scripts/` 的發版與安裝腳本、`packages/cli-npm`（新）、`apps/desktop` 的 CLI 佈署分流（core `cliInstall.ts` 純邏輯＋Tauri 殼 `cli_install.rs` 事實收集），以及使用者文件。Rust workspace 的引擎 crate 與 CLI 的輸出（golden、`--json`）完全不動。

## Goals / Non-Goals

**Goals:**

- Release 頁只剩六個 assets：universal dmg、universal .app.tar.gz、x64 setup.exe、AppImage×2、latest.json；第一次來的人在頁面上直接看到自己那個安裝檔。
- 桌面 app 的自動更新在三平台維持可用（macOS 兩種晶片、Windows、Linux AppImage）。
- CLI 有三條通路、binary 只有一個來源：npm（`@speclink/cli`）、Homebrew（formula 指 npm tgz）、install.sh（抓 npm tgz，免 Node，服務無圖形介面的 Linux 與 CI）。
- 每台機器恰有一條路：macOS dmg 或 brew／npm；Windows setup.exe 或 npm；Linux 有圖形 AppImage；Linux 無圖形 install.sh 或 npm。

**Non-Goals:**

- 不出 Linux 桌面版以外的 Linux 格式（deb、rpm）；不做 brew cask 裝桌面 app；不做 CI 專用的 setup action。
- 不放棄自動更新、不把更新包搬到 GitHub Release 以外的主機、不改更新端點網址。
- 不改 CLI 子指令、旗標、人眼輸出與 `--json`；不動 `speclink-core`／`speclink-cli` 等引擎 crate。
- 不重生 `openspec/manual/` 手冊頁（由 `/speclink-manual` 依規格另行處理）。
- 不把 CLI 壓縮檔改名或保留給任何通路——它們整批離開 Release 頁。

## Decisions

### D1 Release 頁的 asset 封閉集合與收集規則

Release 頁的 assets 是封閉集合，只允許六個檔名樣式：`Speclink_<版本>_universal.dmg`、`Speclink_<版本>_universal.app.tar.gz`、`Speclink_<版本>_x64-setup.exe`、`Speclink_<版本>_amd64.AppImage`、`Speclink_<版本>_aarch64.AppImage`、`latest.json`。

收集規則：release job 只把 `desktop-*` artifact（安裝檔）合併進 dist；`updater-*` artifact 只以保留目錄結構的方式下載到 updater/ 供組裝 latest.json；組裝完成後，把 updater/ 底下**副檔名不是 .sig 的檔案**複製進 dist（Windows 與 Linux 的更新包就是安裝檔本身、同名同內容覆蓋無害；macOS 多出 .app.tar.gz）。不再產生 SHA256SUMS.txt，不再下載 `cli-*`。

為什麼不留 .sig：Tauri updater 讀的是 latest.json 內嵌的 signature 內容（官方文件：「路徑或 URL 都不行」），發布後沒有讀者。為什麼不留 SHA256SUMS.txt：它唯一的機器讀者是 install.sh 與 formula 產生器，兩者都改讀 npm；安裝檔本身有 OS 簽章。替代方案「更新檔搬到固定 updater 預發布 tag」與「更新檔放 gh-pages／R2」都在討論中排除（搬 tag 重建 release 把雜亂搬去索引頁；靜態主機每版 36MB 進分支或多一套基建）。

### D2 macOS universal 建置與 universal sidecar

desktop job 的 macOS 由兩列 matrix（aarch64、x86_64）合為一列：`target: universal-apple-darwin`、`platform: darwin-universal`、`bundles: app,dmg`，toolchain 步驟為這一列安裝 aarch64-apple-darwin 與 x86_64-apple-darwin 兩個 rustup target（matrix 新增 `rust_targets` 欄位，其他列等於 `target`）。Tauri 把產物放在 `target/universal-apple-darwin/release/bundle/`，收集步驟的路徑樣式不變；dmg 檔名由 Tauri 產出為 `Speclink_<版本>_universal.dmg`，.app.tar.gz 由收集步驟改名為 `Speclink_<版本>_universal.app.tar.gz` 放進 `updater/darwin-universal/`。

sidecar：`scripts/desktop/desktop-sidecar.mjs` 收到 `--target universal-apple-darwin` 時，對 aarch64-apple-darwin 與 x86_64-apple-darwin 各跑一次 cargo build，佈**三份**檔到 `apps/desktop/src-tauri/binaries/`：兩份 per-triple 複本 `speclink-aarch64-apple-darwin`、`speclink-x86_64-apple-darwin`，加一份以 lipo -create 合成的 `speclink-universal-apple-darwin`。三份都要在：`tauri build --target universal-apple-darwin` 是對兩個 triple 各跑一次 cargo build，`speclink-desktop` 的 build script（tauri-build）在每次 cargo build 期間以 cargo 的 TARGET 找 `speclink-<triple>`，缺即 ResourcePathNotFound 失敗；universal 那份是 bundler 最後階段放進 app bundle 的（它要求 universal 建置的 external binary 也是 universal、命名帶 -universal-apple-darwin）。每份輸出各走「內容相同即跳過」的防抖（合成檔比對的是合成後的結果）。universal 以外的 target 行為不變（host 建置與既有交叉編譯路徑零改動）。

代價與取捨：mac 使用者下載量約 1.7–2 倍（現況約 18MB）；mac desktop 由兩個平行 job 變一個（總 CI 分鐘數不變，等待時間可能多一個 mac 建置的長度）。換到的是 dmg 與 .app.tar.gz 各少一個、訪客少一個「哪種晶片」的決定。替代方案「兩個 target 各建再手動 lipo 兩個 .app」被排除：bundle 內每個 Mach-O 都要合成，Tauri 的 universal target 已正確處理。

### D3 latest.json：darwin 兩鍵共用 universal 更新包

`scripts/release/release-latest-json.mjs` 的 --dir 契約由「每個平台鍵一個子目錄」改為「每個**更新包目錄**對應一或多個平台鍵」：必要目錄 `darwin-universal`（對應 darwin-aarch64 與 darwin-x86_64 兩鍵）、`windows-x86_64`、`linux-x86_64`；可選目錄 `linux-aarch64`。每個目錄仍須恰有一個更新包與同名 .sig，缺任一必要目錄、缺更新包或缺 .sig 一律非零結束；可選目錄只有「目錄不存在」才放過，目錄在但缺更新包或缺 .sig 同樣非零。輸出的 platforms 物件仍至少含 darwin-aarch64、darwin-x86_64、windows-x86_64、linux-x86_64 四鍵，darwin 兩鍵的 url 與 signature 相同。

為什麼不把鍵名改成 darwin-universal：Tauri updater 依執行機器查 darwin-aarch64 或 darwin-x86_64，鍵名是 updater 的契約，不是我們的。

### D4 @speclink/cli 的套件形狀與 shim 置換

套件形狀比照 server：主套件 `@speclink/cli`（bin `speclink` → `bin/speclink`，一個帶 shebang、無副檔名的 JS shim）加五個平台子套件 `@speclink/cli-darwin-arm64`、`cli-darwin-x64`、`cli-linux-x64`、`cli-linux-arm64`、`cli-win32-x64`（各只含 `speclink`／`speclink.exe`，以 os／cpu 欄位圈定；Linux 兩個另宣告 `libc: ["glibc"]`——binary 是 glibc 動態連結，musl 上讓 npm 跳過子套件、由 shim 報找不到 binary；五個都宣告 `preferUnplugged: true`，Yarn PnP 才會把 binary 解到磁碟）。repo 內只維護 `packages/cli-npm/`（`private: true` 擋誤發布）；`scripts/npm/npm-cli-package.mjs` 在發版時從 `cli-<target>` build artifacts 物化去 private、蓋版本、補五組同版 optionalDependencies 的主套件與五個子套件，缺任一平台 binary 即非零結束。物化規則與 server 的同一份（`scripts/npm/npm-platform-package.mjs`：套件家族名、binary 基底名、主套件來源目錄為參數；主套件要帶的檔就是來源 `package.json` 的 `files` 清單，不另抄一份），兩支入口腳本只解析參數；CLI 的入口不提供 `--scope`（shim 的子套件解析寫死 `@speclink`，可換 scope 是假的彈性）。`engines.node` 為 `^18.19.0 || ^20.10.0 || >=21`：無副檔名的 ESM 入口要這些版本起才能當程式入口執行；npm 對不合的 Node 版本在安裝時印 EBADENGINE 警告（設了 engine-strict 才會擋下安裝），使用者在裝的當下就看得到原因，而不是執行期才炸。

shim 行為（`packages/cli-npm/bin/speclink`）：以 createRequire 解析本機對應的平台子套件路徑，找不到（不支援的平台或 optionalDependencies 被略過）時 stderr 說明並以 exit 1 結束；找到即 spawn 該 binary、stdio 繼承、把子程序的 exit code 與訊號原樣帶回。這條路是 Windows 的正式路徑（npm 在 Windows 產生的 .cmd 殼會以 node 執行 bin，置換成原生檔會壞掉）與 --ignore-scripts 安裝的退路。

postinstall（`packages/cli-npm/postinstall.mjs`）：只在 macOS／Linux 且套件管理器不是 Yarn 時執行（Yarn Berry 一律以 node 執行套件的 bin，換成原生檔會壞；依 `npm_config_user_agent` 是否以 `yarn/` 開頭判斷，esbuild 同款守門）——找到平台 binary 後，把 `bin/speclink` 這個檔**原地**換成原生 binary（先寫到同目錄暫存名再 rename，保 0755），npm 建立的全域 symlink 因此直接指到原生執行檔，之後每次呼叫零 Node 啟動成本。找不到平台 binary 時 postinstall 以 exit 0 結束並保留 shim（不讓安裝失敗，錯誤留到執行時由 shim 報）。這是 esbuild 的做法。

為什麼 bin 名維持 `speclink`：所有技能檔與文件都呼叫 `speclink ...`；PATH 上多一個 speclink 位置的問題 brew 早已存在（`/opt/homebrew/bin` 與 `~/.local/bin`），不是新問題。替代方案「比照 Spectra 另取 bin 名」被排除：兩套指令名的成本遠超本變更。

### D5 Homebrew formula 改指 npm registry

`scripts/release/homebrew-formula.mjs` 的輸入由 SHA256SUMS.txt 改為 `cli-npm-sums.txt`（每行「sha256<兩空白>tgz 檔名」，由 npm 發布 job 在發布並等 registry 可見之後，以 `npm pack <name>@<version>` 自 registry 取回每個 tgz 計算——拿到的是 registry 原 bytes；不能算在本次 pack 的 tgz 上：重跑時已上架的同版會被跳過，本次重建的 bytes 不一定相同，而 formula 指的是 registry 那份）；輸出的四組 url 改為 `https://registry.npmjs.org/@speclink/cli-<os>-<cpu>/-/cli-<os>-<cpu>-<版本>.tgz`（darwin-arm64、darwin-x64、linux-arm64、linux-x64），sha256 取自 sums 對應行；缺任一組即非零結束並點名平台；`bin.install "speclink"` 與 `test do` 區塊不變。npm tgz 的內容在單一頂層目錄 `package/` 下，Homebrew 解壓時會自動進入唯一的頂層目錄，所以 `bin.install "speclink"` 照常成立（第一次發版後以真實 brew install 驗證，見 Migration Plan）。

這是 homebrew-core 對 Node CLI 的既有寫法（url 指 registry.npmjs.org 的 tgz），不是取巧。替代方案「為 brew 保留四個 tar.gz 在 Release 頁」被排除：那正是要清掉的檔。

### D6 install.sh 改抓 npm registry，install.ps1 退役

`scripts/install.sh` 維持同一個 raw 網址、同樣的 --dry-run／--help、同樣的 SPECLINK_INSTALL_DIR 與 SPECLINK_INSTALL_VERSION，改動如下：

- 平台對映改為 npm 的 os／cpu 名（Darwin arm64→darwin-arm64、Darwin x86_64→darwin-x64、Linux x86_64→linux-x64、Linux aarch64→linux-arm64）；Windows 的 uname 值以非零結束並導向 npm 與 setup.exe；Linux 上 `ldd --version` 顯示 musl（Alpine 等）時以非零結束並說明只支援 glibc（沒有 ldd 就不擋）。
- 版本解析：未釘選時 GET `<registry>/@speclink/cli/latest` 取 version；釘選值接受 `0.5.0` 或 `v0.5.0`（去 v 前綴）。
- 下載：`<registry>/@speclink/cli-<os>-<cpu>/-/cli-<os>-<cpu>-<版本>.tgz`。
- 驗證：GET `<registry>/@speclink/cli-<os>-<cpu>/<版本>` 取 dist.integrity（`sha512-<base64>`），以 openssl dgst -sha512 -binary 加 openssl base64 -A 算出實際值比對；不符即非零結束且安裝目錄不落任何檔；找不到 openssl 即非零結束。
- 安裝：解出 tgz 內的 `package/speclink`（tar 的 --strip-components=1）到暫存目錄，驗證通過後才複製到安裝目錄並 chmod 755；PATH 提示不變。
- 環境變數：`SPECLINK_INSTALL_REGISTRY`（預設 https://registry.npmjs.org）取代 `SPECLINK_INSTALL_REPO`；dry-run 只印平台、版本、tgz 網址、安裝目錄，不連網。

`scripts/install.ps1` 刪除，`scripts/install.test.mjs` 的 PowerShell 測試組一併刪除。為什麼留 sh 不留 ps1：無圖形介面的 Linux（伺服器、WSL、CI 容器）需要一條免 Node、免 brew 的路；Windows 幾乎都有圖形介面，setup.exe 內含同版 CLI 並由安裝器管 PATH，有 Node 的走 npm。替代方案「兩支都退役」在討論第 4 輪一度採納、第 5 輪因無圖形 Linux 推翻；「只靠 brew 服務無圖形 Linux」被排除（Linuxbrew 重，伺服器與 CI 不裝）。

### D7 .deb 退場與桌面 Linux 分流

desktop job 的 Linux 兩列 `bundles` 由 `appimage,deb` 改為 `appimage`，收集步驟刪除 deb 的複製。桌面 app 的 Linux 分流由「有 APPIMAGE 環境變數→linux-appimage，否則→linux-deb（探測 /usr/bin/speclink、佈署歸套件管理器）」改為「有 APPIMAGE→linux-appimage，否則→linux-unpackaged（開發建置或手動解開的 app：不佈署、只以 ~/.local/bin/speclink 回報狀態）」；`CliPlatform` 型別移除 `linux-deb`、新增 `linux-unpackaged`，佈署計畫回 `{ action: "none", reason: "unpackaged" }`，啟動自動佈署判定對它恆為 false。Tauri 殼的 `cli_install.rs` 只改平台鍵字串與探測路徑的對映，判斷仍歸 core。

為什麼砍而不修：deb 只服務 Debian 家族桌面機，對無圖形 Linux 沒有貢獻（裝的是桌面 app）；它的自動更新現況是壞的（updater 2.10.1 依建置包型態查 linux-x86_64-deb 鍵、退回 linux-x86_64 後下載 AppImage 而報 InvalidUpdaterFormat），修好要加兩個鍵並收集 .deb.sig，是為兩個檔加碼。deb 唯一的 CLI 貢獻（/usr/bin/speclink）由 npm／install.sh 取代。

### D8 發版 workflow 的 job 拓樸

- build job：移除 Package 步驟（不再打 tar.gz／zip），`cli-<target>` artifact 改為 raw binary（`target/<triple>/release/speclink[.exe]`），與 `server-<target>` 同形。
- desktop job：macOS 一列 universal（D2）、Linux 只建 appimage（D7）、收集步驟把 .sig 只放進 updater/（D1）。
- release job：`needs: [build, desktop, docker-manifest]` 不變；下載 `desktop-*` 進 dist、`updater-*` 進 updater/；組裝 latest.json；複製非 .sig 更新包進 dist；不產 SHA256SUMS；下載指南前置。
- 三條 npm 通路共用一份 `workflow_call` 的 `.github/workflows/npm-publish.yml`（NPM_TOKEN 閘門、下載 artifact、物化＋npm pack 或沿用已打包的 tarball、逐份斷言 tarball 版號等於 tag 版、npm view 冪等、`npm publish "./<tgz>" --access public` 子套件先發主套件最後、輪詢等主套件在 registry 可見、再以 `npm pack <name>@<version>` 自 registry 取回每個 tgz 算 sha256 清單、輸出 `published`）；呼叫端只給差異：`artifacts`（下載 pattern）、`materialize`（物化腳本路徑，留空＝不重新打包）、`main-package`、`sums-artifact`（有值才上傳校驗清單）。
- 新 job `cli-npm-publish`：`needs: [release]`，`uses` 上述 workflow，`artifacts: cli-*`、`materialize: scripts/npm/npm-cli-package.mjs`、`main-package: @speclink/cli`、`sums-artifact: cli-npm-sums`（清單檔名 `cli-npm-sums.txt`）。NPM_TOKEN 缺席時 job 綠、`published=false`。
- tap-publish：`needs: [release, cli-npm-publish]`；閘門為 TAP_PUSH_TOKEN 存在**且** `cli-npm-publish` 輸出 `published=true`；下載 `cli-npm-sums` artifact、以產生器輸出 formula、推送 tap（推送步驟不變）。兩個條件任一不成立即跳過且 job 綠。
- server 的 `npm-publish`（`artifacts: server-*`、`materialize: scripts/npm/npm-server-package.mjs`）與 engine 的 `engine-npm-publish`（`artifacts: npm-tarballs`、不傳 `materialize`——發布單位是上游打包好的 tarball，不重新打包）改為呼叫同一份 workflow，語意（閘門、冪等、順序、fail-closed）不變；engine 的另兩個 job、docker 兩個 job 不動。

順序理由：通路推送（npm、brew）排在 Release 之後，不是發布的前置條件，失敗可單獨重跑；brew 現在依賴 npm 是「binary 單一來源」的直接結果。

### D9 下載指南與文件

`scripts/release/release-notes.mjs` 的對照表改為四行：macOS（Apple Silicon 與 Intel 同一檔）→ universal dmg；Windows（x64）→ setup.exe；Linux 桌面機（x64／arm64）→ 對應 AppImage；Linux 伺服器或無圖形介面 → 「不用下載，用下方 CLI 一行安裝」。CLI 節列三條：npm i -g @speclink/cli、curl … install.sh | sh、brew install MomoChenisMe/tap/speclink。server 節不變。尾註改為「.app.tar.gz 與 latest.json 是桌面 App 自動更新機制用的，毋須手動下載」。

文件同步：README 中英的安裝區塊（桌面表移除 deb 與晶片分列、CLI 三條指令、覆蓋行為表移除 deb 行、新增 deb 使用者遷移一句）；getting-started 中英的安裝節；development 中英的安裝檔清單與 Linux 放行段（移除 deb、移除 SHA256SUMS 句）；product-status 中英的安裝通路列。

## Implementation Contract

**Release assets（D1、D2、D7）**：push tag vX.Y.Z 且 workflow 全綠後，該 Release 的 assets 恰為 `Speclink_X.Y.Z_universal.dmg`、`Speclink_X.Y.Z_universal.app.tar.gz`、`Speclink_X.Y.Z_x64-setup.exe`、`Speclink_X.Y.Z_amd64.AppImage`、`Speclink_X.Y.Z_aarch64.AppImage`、`latest.json` 六個；沒有任何 .sig、.deb、.tar.gz（.app.tar.gz 除外）、.zip、SHA256SUMS.txt。驗證：`scripts/release/delivery-gate.test.mjs` 對 release.yml 的靜態斷言（release job 不下載 cli-*、dist 收集不含 .sig、Linux bundles 不含 deb、macOS 只有 universal 一列、build job 無 Package 步驟）；發版後 gh release view 的 assets 清單人工比對一次（[M]）。

**latest.json（D3）**：version 為去 v 前綴的 tag；platforms 含 darwin-aarch64、darwin-x86_64、windows-x86_64、linux-x86_64（linux-aarch64 有則收）；darwin 兩鍵的 url 都指向 `.../releases/download/vX.Y.Z/Speclink_X.Y.Z_universal.app.tar.gz` 且 signature 相同。失敗模式：缺 darwin-universal、windows-x86_64、linux-x86_64 任一目錄、缺更新包或缺 .sig → 非零結束、不寫檔。驗證：`scripts/release/release-latest-json.test.mjs`。

**universal sidecar（D2）**：desktop-sidecar.mjs 帶 `--target universal-apple-darwin` 時在 `apps/desktop/src-tauri/binaries/` 產出三份：`speclink-aarch64-apple-darwin`、`speclink-x86_64-apple-darwin`（各為該 triple 的建置產物）與 `speclink-universal-apple-darwin`（lipo -info 顯示 arm64 與 x86_64 兩個架構）；其他 target 的產出路徑與檔名不變。驗證：`scripts/desktop/desktop-sidecar.test.mjs` 對參數解析與 universal 的建置計畫（兩個 cargo target、三個輸出的來源與目的檔）的單元測試；CI 的 desktop job 建置成功即為整合驗證。

**@speclink/cli（D4）**：npm i -g @speclink/cli 後，macOS／Linux 的全域 `speclink` 是原生 binary（`file` 顯示 Mach-O／ELF）、`speclink --version` 輸出與 tag 同版；Windows 的 `speclink` 經 shim spawn `speclink.exe`，exit code 原樣帶回；`npm i -g --ignore-scripts @speclink/cli` 後 speclink 仍可執行（走 shim）；以 Yarn 安裝時 postinstall 保留 shim；不支援平台執行 shim 時 stderr 說明並 exit 1。物化腳本輸入 `--version X.Y.Z --binaries <dir> --out <dir>`（缺任一 `cli-<target>/speclink[.exe]` 即非零結束並點名 target；版本不符 X.Y.Z 即非零結束）。驗證：`scripts/npm/npm-cli-package.test.mjs`（物化欄位、fail-closed）、`scripts/npm/npm-cli-launcher.test.mjs`（shim 的平台解析與 exit code 轉發、postinstall 的置換與找不到 binary 時保留 shim）。

**formula（D5）**：`node scripts/release/homebrew-formula.mjs --tag vX.Y.Z --sums cli-npm-sums.txt` 輸出的 formula 四組 url 為上述 registry.npmjs.org 樣式、sha256 等於 sums 對應行（sums 算在 registry 取回的 tgz 上，重跑也不會與 registry 脫節）；sums 缺任一組（darwin-arm64、darwin-x64、linux-arm64、linux-x64）即非零結束並點名；不引用 win32。驗證：`scripts/release/homebrew-formula.test.mjs`（含 ruby -c 合法性）。

**install.sh（D6）**：見 D6 的逐項行為；dry-run 不連網不寫檔且印出 tgz 網址；integrity 不符 → 非零結束、安裝目錄無任何新增檔；成功 → `<安裝目錄>/speclink` 存在、可執行、`speclink --version` 為該版。驗證：`scripts/install.test.mjs` 以假 uname／curl 與 fixture tgz（含 package/speclink）、fixture 版本 JSON（含 sha512 integrity）驅動；PowerShell 測試組刪除。

**桌面 Linux 分流（D7）**：Linux 且無 APPIMAGE 環境變數時，安裝 CLI 介面顯示狀態（以 ~/.local/bin/speclink 探測）但不提供佈署動作、啟動不自動佈署；`CliPlatform` 無 `linux-deb` 值。驗證：`apps/desktop` 的 vitest（cliInstall 的 plan 與 needsRedeploy 對 linux-unpackaged）、`cargo test -p speclink-desktop` 既有測試不變綠。

**下載指南（D9）**：產生器輸出含 universal dmg、setup.exe、兩個 AppImage 的檔名（含該版號）、三條 CLI 指令、server 兩條指令、「.app.tar.gz 與 latest.json 自動更新用」註記；不含 .sig、.deb、SHA256SUMS、tar.gz／zip 字樣。驗證：`scripts/release/release-notes.test.mjs`。

**範圍內／外**：範圍內＝上列九項與其測試、四份 delta spec、八份文件。範圍外＝手冊頁重生、brew cask、setup action、CLI 輸出、引擎 crate、Windows 安裝器與簽章流程、docker 與 engine 的 job。

## Risks / Trade-offs

- [回歸對照] CLI 人眼輸出與 `--json` 零改動，golden 與 `crates/adapters/speclink-cli/tests/` 不受影響 → tasks 仍以 `cargo test -p speclink-desktop` 與 scripts 全組測試守門；`scripts/release/delivery-gate.test.mjs` 是本變更的主要回歸面，先改測試（紅）再改 workflow（綠）。
- [跨平台] install.ps1 退役後 `scripts/install.test.mjs` 在 Windows runner 上整組跳過（sh 測試本來就跳過）→ Windows 的 CLI 通路由 npm 的 shim 路徑承擔，`npm-cli-launcher.test.mjs` 的 spawn、參數透傳與 exit code 轉發在三平台都跑（POSIX 以 sh 腳本、Windows 以 node.exe 複本當假 binary）；訊號轉發只在 POSIX 驗，Windows 沒有訊號。
- [universal sidecar 命名] tauri-bundler 文件明寫 universal 建置的 external binary 須為 universal 且命名 `<name>-universal-apple-darwin`，而 tauri-build 在兩次 per-triple 的 cargo build 期間又各要一份 `<name>-<triple>` → 三份都佈；desktop job 建置失敗即為紅燈，不會發出缺 sidecar 的 Release。
- [mac job 時間] 兩個平行 mac job 變一個，等待時間可能多一個 mac 建置的長度 → 接受；CI 分鐘數不變（本來就編兩次）。
- [Homebrew 解壓假設] 假設 Homebrew 對單一頂層目錄的 tgz 會自動進入該目錄 → 第一次發版後由使用者實機 brew install 驗證（[M]）；若不成立，formula 改為 `bin.install "package/speclink"` 一行即可修。
- [postinstall 被略過] 使用者以 --ignore-scripts 安裝或 npm 未跑 postinstall → shim 路徑保底，功能不缺、只多 Node 啟動成本。
- [registry 可見延遲] 首次上架的套件 npm publish 後 packument 可能延遲可見 → cli-npm-publish 沿用 engine job 的十分鐘輪詢；tap-publish 的 formula 指向 tgz 網址，tgz 在 publish 成功即可下載。
- [brew 依賴 npm] NPM_TOKEN 缺席時 tap 也跳過 → 明寫於 spec 與 job 註解；兩者都是 Release 之後的通路推送，不影響 Release。
- [既有 mac 使用者升級] 0.4.0 的 aarch64／x64 專屬 app 讀到 universal 更新包 → Tauri 更新流程以新 .app 整包替換，架構不同不影響。
- [deb 使用者] 沒有新版 deb，且既有 deb 版 app 的更新按鈕本來就失敗 → README 中英註明遷移（移除 deb 後改裝 AppImage）。
- [釘選舊版] install.sh 的 SPECLINK_INSTALL_VERSION 釘 v0.4.0 以前的版本會失敗（那些版本不在 npm）→ 文件註明自本版起才可釘。

## Migration Plan

1. 本變更合併到 main 後，下一次 `/release`（feat 加 BREAKING，0.x 階段封頂 minor → v0.5.0）觸發新管線；同一次發版同時上架 @speclink/cli 首版與更新後的 formula。
2. 發版後的人工驗證（tasks 的 [M] 項）：Release 頁六個 assets；macOS 既有 0.4.0 app 自動更新到 universal；npm i -g @speclink/cli 在 macOS／Windows 各驗一次 speclink --version；brew upgrade 後 speclink --version；install.sh 在一台無 Node 的 Linux 上跑一次。
3. 回退：管線改動全在 release.yml 與 scripts/，revert 該 commit 並重新發版即回到舊 asset 集合；已上架的 npm 套件與 tap formula 不需撤回（舊 formula 指向的 Release 壓縮檔在舊版 Release 頁仍在）。

## Open Questions

（none）——sidecar 命名已由 tauri-bundler 文件確認；Homebrew 解壓假設的驗證放在首次發版後的 [M] 任務，若不成立有一行修法。

---
topic: release 發布後 CI 產出太多檔案，能否只留安裝檔（比照 Spectra）
slug: release-assets-trim
status: promoted
created: 2026-09-14
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: release-assets-trim
---

# Discussion: release 發布後 CI 產出太多檔案，能否只留安裝檔（比照 Spectra）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者看到 v0.4.0 Release 頁有 21 個 assets，問能否只留安裝檔、比照 Spectra（3 個檔）。需求已夠明確（可驗證：檔案數、第一次來的人能否選對檔），不需 grill，直接以假設清單開場。
Scout：.github/workflows/release.yml 的 build／desktop／release／tap-publish／npm-publish job；scripts/release/{release-latest-json,release-notes,homebrew-formula}.mjs；scripts/install.{sh,ps1}；apps/desktop/src-tauri/tauri.conf.json 的 updater 端點；packages/server-npm 與 scripts/npm/npm-server-package.mjs（npm 平台子套件模式）。相關 specs：desktop-release、cli-distribution、server-release。
Spectra 對照：spectra-app 是 releases-only repo；無 app 自動更新（CHANGELOG 無 updater 字樣，靠 brew cask 升級）、CLI 走 npm（@kaochenlong/spxa，Node ≥22.14）、無 Linux 版——三個取捨造就 3 個檔。
Prior discussions: one-click-install-and-run, release-first-and-distribution, release-changelog-whats-new, node-sdk-completion-and-doc-alignment

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-14)

**Focus**: v0.4.0 的 21 個 assets 各自的讀者是誰、哪些砍了沒人受影響
**Position**: 可以砍，但保留自動更新與現有 CLI 通路時砍不到 Spectra 的 3 個；地板是 14–16 個。
- 分類：7 個給人下載的安裝檔（dmg×2、setup.exe、AppImage×2、deb×2）；8 個自動更新用（latest.json、.app.tar.gz×2、.sig×5）；6 個 CLI 通路用（五 target 壓縮檔＋SHA256SUMS.txt）
- .sig×5 發布後零讀者：scripts/release/release-latest-json.mjs:50-51 已把簽章內容內嵌進 latest.json，Tauri updater 只讀 latest.json；全 repo 掃過無其他讀者 → 可直接砍（21→16），連動 release.yml 收集步驤、下載指南一句、desktop-release spec「Release 說明含下載指南」一條、docs/development.md 一句
- latest.json＋.app.tar.gz×2 必留：tauri.conf.json:29-31 端點釘在 releases/latest/download/latest.json；macOS 更新包只能是 .app.tar.gz（dmg 不能拿來更新）；GitHub Release 沒有隱藏 asset 功能
- CLI 壓縮檔×5＋SHA256SUMS.txt：install.sh:112-131、install.ps1:92、homebrew-formula.mjs:69 都從 Release 頁抓 → 不換通路就砍不掉
- .deb×2 是使用者決策（AppImage 已是 Linux 更新包，deb 服務套件管理器使用者）；CLI 壓縮檔改名加 cli 是選配
- Spectra 乾淨的原因：無 app 自動更新、CLI 走 npm、無 Linux
**Ruled out**: 放棄自動更新換 5 個檔——與 one-click-install-and-run（2026-07-30）決議相反、產品倒退；latest.json 搬到別的 Release／tag——releases/latest 會指到錯的頁，README 連結與更新端點同時受害
**Open**: .deb 去留；CLI 壓縮檔是否改名；CLI 是否換通路以清掉 6 個檔（使用者接著問「CLI 能否如 Spectra 走 npm 或 brew」）

### Round 2 — assumptions (2026-09-14)

**Focus**: CLI 走 npm／brew 能不能把 6 個 CLI 檔從 Release 頁清掉
**Position**: brew 已經有了但不解題；npm 可做且 repo 已有 @speclink/server 的現成模式——只有「npm 成為 CLI binary 唯一來源、brew 與安裝腳本改抓 npm」才會讓頁面變乾淨。
- brew 現況：MomoChenisMe/homebrew-tap 的 Formula/speclink.rb 由 tap-publish job 每版自動推；formula 的 url 指向 Release 頁的 tar.gz → brew 是 Release 檔的消費者，不是替代品
- Spectra 的 brew 是 cask（裝桌面 app）；它的 CLI 只有 npm（@kaochenlong/spxa）
- npm 模式現成：packages/server-npm＋scripts/npm/npm-server-package.mjs＝主套件＋五個平台子套件（optionalDependencies），發版時從 build artifacts 物化；cli-<target> artifacts 已在同一 build job 產出，複製這條線即 @speclink/cli
- 前置條件已滿足：release-first-and-distribution 把 npm 通路延後到 engine 上 npm 之後；@speclink/engine 已上 npm（node-sdk-release）
- 提案：@speclink/cli 成為 CLI binary 唯一來源；brew formula 的 url 改指 registry.npmjs.org 的平台子套件 tgz（sha256 在 CI 由 tgz 算）；install.sh／install.ps1 退役；SHA256SUMS.txt 一起砍；Release 頁剩安裝檔＋自動更新三件（10 個；再砍 deb 為 8 個）
- 代價：npm 裝的 speclink 每次呼叫多一次 Node 啟動（約 50ms）；PATH 多一個 speclink 位置（brew 已造成同類問題）；specs 動兩份（cli-distribution 三條、desktop-release 兩條）
**Ruled out**: npm 只當「多一個通路」而保留 Release 頁的 CLI 壓縮檔——一個檔都不會少，達不到原目標
**Open**: 安裝腳本退役 vs 改抓 npm registry（保住免 Node 一行安裝）；bin 名維持 speclink 或比照 Spectra 另取名；SHA256SUMS.txt 去留；.deb 去留；.sig 砍除與 CLI 換通路是否同一個 change

### Round 3 — assumptions (2026-09-14)

**Focus**: 第二輪提案有沒有更完善的版本（使用者問「有沒有更完善的作法」）
**Position**: 有，三處補強：macOS 改出 universal、npm 套件用 esbuild 式 shim 置換消掉 Node 啟動成本、brew 指 npm registry 是 homebrew-core 慣例；地板從 10 降到 8（砍 deb 為 6）。
- 新事實（curl 抓 Release 頁的 expanded_assets 片段）：GitHub 依檔名排序、不分大小寫、`-` 排在 `_` 前——v0.4.0 頁前 7 個是 latest.json、SHA256SUMS.txt 與 5 個 CLI 壓縮檔，第一個 dmg 排第 12；混淆來自安裝檔被機器檔擠到後面，不只是數量
- Tauri 官方 CLI 文件：`tauri build --target universal-apple-darwin` 為合法值，需同時安裝 aarch64 與 x86_64 兩個 rustup target（build job 已在 arm runner 交叉編譯 x86_64，條件現成）→ dmg 2→1、.app.tar.gz 2→1，訪客少一個「哪種晶片」的決定；latest.json 的 darwin-aarch64 與 darwin-x86_64 兩鍵指同一檔同一簽章
- universal 代價：mac 下載量約 1.7–2 倍；mac desktop 兩個平行 job 變一個（發版等待時間可能多一個 mac 建置的長度）；sidecar 要 lipo 合成並命名 speclink-universal-apple-darwin（官方文件未明寫此命名，change 內先驗）；release-latest-json.mjs「一平台一子目錄、恰一 .sig」契約要改
- Tauri updater 文件明寫：signature 欄位放 .sig 的內容、「路徑或 URL 都不行」→ .sig 檔執行期無用，第一輪結論成立；macOS 更新包只有 .app.tar.gz，dmg 不能更新
- npm：postinstall 把 JS shim 換成原生 binary（esbuild 做法），--ignore-scripts 時 shim 仍以 spawn 退回 → 第二輪列的每次呼叫 50ms 成本消失
- brew：homebrew-core 大量 Node CLI formula 的 url 就是 registry.npmjs.org 的 tgz，指 npm 不是取巧
- 結果頁面 8 個：universal dmg、universal .app.tar.gz、setup.exe、AppImage×2、deb×2、latest.json；砍 deb 為 6
**Ruled out**: 固定 `updater` 預發布 tag 承載 latest.json 與更新包——要每版搬 tag 並重建 release，把雜亂搬到 releases 索引頁而非消除；靜態主機（gh-pages／R2）放更新包——每版 36MB 進分支或多一套基建，為了少 3 個檔不值
**Open**: 是否採 universal macOS（下載量與 job 時間代價）；安裝腳本退役 vs 改抓 npm registry（CI／免 Node 一行安裝）；.deb 去留；分幾刀；brew cask 裝桌面 app（Spectra 有，另案）

### Round 4 — interview (2026-09-14)

**Focus**: 使用者裁定三項（universal macOS 採、安裝腳本退役、一刀做完），並問 .deb 的用途
**Position**: 三項落定；.deb 是 Debian／Ubuntu 家族的套件管理器安裝包，而且在現況下它的自動更新是壞的。
- 使用者裁定：採 universal macOS；install.sh／install.ps1 退役（cli-distribution「安裝腳本一行安裝」整條 REMOVED，兩支腳本與測試刪除，下載指南 CLI 節改 npm i -g ＋ brew）；.sig 砍除、universal、CLI 上 npm 合為一個 change
- .deb 用途：交給 apt／dpkg 安裝，app 進系統選單、可 apt remove 乾淨移除、CLI 由套件放到 /usr/bin/speclink（README.md:91、apps/desktop/src-tauri/src/cli_install.rs:48）；只有 Debian 家族能用。AppImage 是免安裝單檔、任何發行版可用、CLI 由 app 複製到 ~/.local/bin、自動更新走此格式
- 現況問題（tauri-plugin-updater 2.10.1 原始碼）：安裝器依建置時的包型態選定，deb 版先查 linux-x86_64-deb 鍵、找不到退回 linux-x86_64；我們的 latest.json 只有 linux-x86_64 且指向 AppImage → deb 版下載 AppImage 後 install_deb 驗出非 deb，回 InvalidUpdaterFormat。deb 使用者每版看到「有新版」，按下去失敗
- 兩條路：A 砍 deb（頁面 6 個檔，Linux 一律 AppImage、更新故事一致；動 desktop-release／desktop-app／user-documentation 三份 spec 的 deb 字句、README／getting-started／development 六份文件、下載指南表，清掉 cli_install.rs 與 cliInstall.ts 的 linux-deb 分支）；B 留 deb 並修好（latest.json 加 linux-x86_64-deb／linux-aarch64-deb 兩鍵指向 .deb、收集 .deb.sig——bundler 是否簽 deb 要驗；頁面 8 個檔）
- 建議 A
**Open**: .deb 砍（A）或留並修（B）

### Round 5 — assumptions (2026-09-14)

**Focus**: Linux 有／無圖形介面兩種使用者，對 .deb 與安裝腳本兩個決定的影響（使用者要求再想）
**Position**: 圖形介面的問題不改變「砍 .deb」的結論，但推翻「安裝腳本全退役」——無圖形的 Linux 需要一條免 Node 的 CLI 路。
- 兩種 Linux 使用者：有圖形（桌面）要桌面 app → AppImage 即可、自動更新能用；無圖形（伺服器、WSL、CI）只要 CLI，桌面 app 根本跑不起來（Tauri 需要 WebKitGTK 與顯示器）
- .deb 不是無圖形使用者的答案：它裝的是桌面 app，會拉整套 GUI 函式庫；它對 CLI 的唯一貢獻（/usr/bin/speclink）由 CLI 通路取代 → 砍 .deb（A）的建議不變且理由更強
- 決定 2 修正：留 install.sh、改抓 npm registry 的平台 tgz（curl＋tar 免 Node、版本解析改 registry.npmjs.org/@speclink/cli/latest、checksum 改讀 npm 的 sha512 integrity）；install.ps1 退役（Windows 幾乎都有圖形介面，NSIS 安裝器內含 CLI 並管 PATH，其餘走 npm）
- 檔案數不變（6）：install.sh 抓的 tgz 在 npm，不在 Release 頁
- 每台機器一條路：macOS dmg 或 brew／npm；Windows setup.exe 或 npm；Linux 有圖形 AppImage；Linux 無圖形 curl|sh 或 npm
- 另一條可選路 C：像 Spectra 不出 Linux 桌面版（頁面剩 4 個檔）——不建議：AppImage 已能建、自動更新能用，砍掉是產品倒退不是簡化
**Ruled out**: .deb 當無圖形 Linux 的 CLI 通路——裝桌面 app 拉 GUI 函式庫，且其自動更新現況是壞的；只靠 brew 服務無圖形 Linux——Linuxbrew 重、伺服器與 CI 不會裝
**Open**: 決定 2 修正版是否接受（留 sh 退役 ps1，或兩支都留並改抓 npm）；.deb 砍或留；是否走 C 不出 Linux 桌面版

## Conclusion

**Decision**: 一個 change 把 Release 頁從 21 個 assets 收成 6 個——universal dmg、universal .app.tar.gz、x64 setup.exe、AppImage×2（x86_64／aarch64）、latest.json——並把 CLI 通路改以 npm 為唯一 binary 來源：
1. 砍 .sig×5：簽章內容已內嵌 latest.json，發布後零讀者；release.yml 收集步驟不再放進 dist。
2. macOS 改 `tauri build --target universal-apple-darwin`：dmg 與 .app.tar.gz 各剩一個，latest.json 的 darwin-aarch64／darwin-x86_64 兩鍵指同一檔同一簽章；sidecar 以 lipo 合成並命名 speclink-universal-apple-darwin（命名未見官方文件，change 內先驗）；release-latest-json.mjs 的「一平台一子目錄」契約改為兩鍵共用。
3. 砍 .deb×2：deb 只服務 Debian 家族桌面機，不服務無圖形 Linux（它裝的是桌面 app），且其自動更新現況是壞的（updater 2.10.1 依包型態選安裝器，deb 版退回 linux-x86_64 鍵下載 AppImage 後 InvalidUpdaterFormat）；清掉 cli_install.rs 與 cliInstall.ts 的 linux-deb 分支。
4. CLI 上 npm：@speclink/cli 主套件＋五平台子套件（optionalDependencies），比照 packages/server-npm＋scripts/npm/npm-server-package.mjs 物化、NPM_TOKEN 缺席跳過；postinstall 以原生 binary 置換 JS shim（esbuild 做法），--ignore-scripts 時 shim 退回 spawn。五 target CLI 壓縮檔與 SHA256SUMS.txt 離開 Release 頁。
5. brew 留：formula 的 url 改指 registry.npmjs.org 的平台 tgz（homebrew-core 慣例），sha256 由 CI 打包的 tgz 計算；tap-publish 改依賴 npm publish 輸出。
6. install.sh 留、改抓 npm registry 的同一個 tgz（curl＋tar 免 Node、版本問 @speclink/cli/latest、checksum 改讀 sha512 integrity）——服務無圖形 Linux；install.ps1 退役（Windows 幾乎都有圖形介面，setup.exe 內含 CLI 並管 PATH，其餘走 npm）。
7. 下載指南改為：macOS 一個檔、Windows 一個檔、Linux 桌面機 AppImage、Linux 伺服器 CLI 一行（curl|sh 或 npm）；.sig 註記移除，.app.tar.gz 與 latest.json 的「自動更新用」註記保留。
canon deltas：desktop-release（「Release 產出三平台桌面安裝檔」「更新描述檔隨 release 發布」「Release 說明含下載指南」三條 MODIFIED，安裝檔集改為上列六個且 CLI 壓縮檔 SHALL NOT 出現）；cli-distribution（「安裝腳本一行安裝」改 sh 單支＋npm 來源、「Homebrew formula 產生器」「Formula 隨發版自動推送 tap」改 npm 來源，ADDED「CLI 以 npm 套件發布」）；desktop-app:2232 移 deb 分句；user-documentation:451、605 通路與覆蓋行為表同步。
**Rationale**: 混淆的根源是機器讀的檔案把安裝檔擠到第 12 位（GitHub 依檔名排序，`-` 排在 `_` 前），不只是數量；GitHub 沒有隱藏 asset 功能，唯一解法是讓機器檔離開頁面——.sig 零讀者可直接砍，CLI 檔要換通路才能走，自動更新的兩個檔（latest.json、.app.tar.gz）是保留自動更新的最低代價。Spectra 的 3 個檔靠「無自動更新、CLI 走 npm、無 Linux」三個取捨換來；我們保留自動更新與 Linux 桌面，所以地板是 6。Linux 的複雜度來自 deb 與安裝腳本，不來自 AppImage——拆成「有圖形→AppImage、無圖形→CLI 一行」後每台機器恰一條路。
**Rejected alternatives**: 放棄自動更新換 5 個檔（與 one-click-install-and-run 決議相反、產品倒退）；latest.json 與更新包搬到固定 updater 預發布 tag（每版搬 tag 重建 release，雜亂搬到索引頁）；更新包放 gh-pages／R2（每版 36MB 進分支或多一套基建）；npm 只當多一個通路（Release 頁一個檔都不少）；留 deb 並補 -deb 鍵修更新（為兩個檔加碼，且不服務無圖形使用者）；不出 Linux 桌面版比照 Spectra（AppImage 已能建、更新能用、spec 已承諾，砍掉是倒退）；安裝腳本全退役（第 4 輪一度採納，第 5 輪因無圖形 Linux 需免 Node 路徑推翻）；只靠 brew 服務無圖形 Linux（Linuxbrew 重，伺服器與 CI 不裝）；三刀分開（使用者選一刀）；CLI 壓縮檔改名讓排序後移（檔案離開頁面後不需要）。
**Deferred**: brew cask 裝桌面 app（Spectra 有；universal dmg 會讓 cask 更簡單，另案）；SPECLINK_INSTALL_VERSION 釘選值語意（v 前綴 tag vs npm 版號）於 propose 定；CI 專用 setup-speclink action（未討論）；universal 對 scripts/desktop/desktop-install.mjs 本機安裝流程是否有影響（本機仍為 host 架構，propose 期確認）。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion release-assets-trim

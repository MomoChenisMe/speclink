## Context

sidecar（apps/desktop/src-tauri/binaries/speclink-<triple>）是 Tauri externalBin 要求的 gitignored 產物：speclink-desktop 編譯期硬性要求它存在，build script 並會把它剝掉平台後綴複製到 target/debug/speclink 供 dev 期取用（tauri-build 2.6.3 copy_binaries，先刪後複製）。目前全 repo 只有 scripts/desktop-install.mjs（經 scripts/desktop-sidecar.mjs）佈署它，dev 路徑無人維護——全新 checkout 硬失敗，既有 checkout 則被過期 sidecar 蓋掉 npm run cli 用的新 CLI。

scripts/desktop-sidecar.mjs 現況：純執行腳本、無匯出，固定 release 建置（cargo build --release -p speclink-cli，支援 --target 交叉編譯），無條件 copyFileSync。呼叫者有二：scripts/desktop-install.mjs 步驟 (2)（無參數）與 CI release 管線（--target <triple>）。

本次變更零 Rust 改動，僅動開發編排腳本與 apps/desktop 的 npm scripts；不涉引擎 crate 邊界（speclink-core／fs／host／store 皆不碰），無 local／remote 雙路徑議題，無 YAML 改寫，無 git 互動。

## Goals / Non-Goals

**Goals:**

- 所有 dev 視窗啟動入口（repo root 的 npm run dev、npm run dev:desktop、直接於 apps/desktop 跑 tauri dev）在 vite 與 Rust 編譯開始前，自動佈署當前 checkout 建置的 debug sidecar。
- 全新 checkout 不再因缺 sidecar 編譯失敗；過期 sidecar 不再蓋掉 target/debug/speclink 的新 CLI。
- sidecar 內容未變時不觸碰檔案，避免每次啟動觸發 speclink-desktop 重編。
- 本機安裝（desktop-install.mjs）與 CI 交叉編譯（--target）的 release 路徑行為完全不變。

**Non-Goals:**

- 不動 scripts/dev.mjs 的編排（devPrerequisites、startDevEnvironment 維持 desktop-dev-frontend-hmr 封存後的形狀），不改剛併入正典的 dev-harness 既有需求文字。
- 不處理 dev 與安裝版共用 Tauri identifier 的資料目錄隔離、也不處理指向安裝版的 CLI 符號連結（前次已明確排除，本次仍不納入）。
- 不改 tauri-build 的 copy_binaries 行為（上游套件）；防抖是在我方複製端做，不是阻止 Tauri 的複製。
- 不為 release 佈署加防抖以外的新行為——desktop-install.mjs 的「sidecar 永遠重建」語意（每次都 cargo build --release）維持不變，防抖只作用於複製這一步且內容相異時照常覆蓋。

## Decisions

### 決策一：掛載點為 apps/desktop 的 npm predev hook

在 apps/desktop/package.json 加 predev script 呼叫 node ../../scripts/desktop-sidecar.mjs --profile debug。npm 對 run dev 會自動先執行 predev（npm 標準 pre-hook 行為），而 tauri dev 的 beforeDevCommand 正是於 apps/desktop 執行的 npm run dev——因此三個 dev 入口全數被涵蓋，且 scripts/dev.mjs 零改動、剛封存的 dev-harness 規格與 dev.test.mjs 既有測試不必動。

替代方案與否決理由（承討論記錄）：掛 scripts/dev.mjs 的前置步驟——覆蓋不到直接跑 tauri dev 的入口，且「單獨啟動 desktop」規格才剛改為零前置、要再 MODIFIED 一次；在 Tauri build script 造 placeholder 假檔——違反專案 fail-closed 哲學，dev 視窗的安裝功能會佈出垃圾 binary。

時序保證：tauri dev 先跑完 beforeDevCommand（含 predev）等 devUrl 就緒，才啟動 Rust 編譯——sidecar 必定在編譯前就位。副作用：單獨跑 npm run dev -w apps/desktop（純 vite、不經 tauri）也會觸發 predev 多建一次 CLI，增量情況為秒級 no-op，可接受。

### 決策二：dev 佈 debug profile，--profile 參數白名單驗證

scripts/desktop-sidecar.mjs 新增 --profile 參數：合法值僅 debug 與 release，無參數預設 release（與現行為位元級一致，保護 desktop-install.mjs 與 CI 兩個既有呼叫者）。未知值以非零狀態結束並輸出點名該值與合法值清單的錯誤訊息——設定選項不靜默吞錯（audit 紀律：失敗要大聲）。

dev 佈 debug 的理由：與 npm run cli 驗證所用同為 target/debug/speclink，內容一致，tauri-build 後續把它複製回 target/debug/speclink 時即為無害的同內容覆蓋；且省去 dev 啟動付 release 建置的時間。

### 決策三：內容相同即跳過複製（防抖）

複製前比對來源與目的檔內容（讀檔比對位元組），相同即跳過、不觸碰目的檔。理由：binaries/speclink-<triple> 在 cargo 的 rerun-if-changed 清單內，copyFileSync 無條件覆蓋會更新 mtime，使每次 dev 啟動都觸發一輪 speclink-desktop 重編（實測約 3 秒）加一次 copy_binaries 複製。目的檔不存在時直接複製。此防抖對 release 路徑同樣生效但不改變語意——cargo build --release 照跑（「sidecar 永遠重建」維持），僅內容未變時省略複製這一步，佈署結果位元級相同。

### 決策四：desktop-sidecar.mjs 重構為可測形狀

現況是無匯出的純執行腳本，無法單元測試。重構為：純函式匯出（參數解析、建置產物來源路徑推導、是否跳過複製的判定），main 薄委派並沿用既有的 import.meta.url 執行判斷模式（同 scripts/desktop-install.mjs 的做法，node --test 匯入時不執行佈署）。測試落點 scripts/desktop-sidecar.test.mjs，用 node:test，與 scripts/ 其餘測試同慣例。

## Implementation Contract

**行為**：開發者在任一 dev 視窗啟動入口（npm run dev、npm run dev:desktop、apps/desktop 內直接 tauri dev）按下啟動後，終端先出現 sidecar 佈署輸出（cargo 建置＋佈署完成或跳過訊息），再進入 vite 與 Rust 編譯；全新 checkout 不因缺 sidecar 失敗；CLI 原始碼未變時第二次啟動不改寫 sidecar 檔案。

**介面與資料形狀**：

- scripts/desktop-sidecar.mjs 的 CLI 介面：新增 --profile <debug|release>，預設 release；與既有 --target <triple> 正交（--target 時建置輸出路徑為 target/<triple>/<profile>/speclink[.exe]，無 --target 時為 target/<profile>/speclink[.exe]）。匯出純函式供測試：參數解析（回傳 profile 與 target）、來源路徑推導、跳過判定（目的檔存在且內容相同 → 跳過）。
- apps/desktop/package.json 的 scripts 新增一行 predev，內容為以 node 呼叫 ../../scripts/desktop-sidecar.mjs 帶 --profile debug；既有 dev、build、test、tauri scripts 不變。
- 佈署目的位置不變：apps/desktop/src-tauri/binaries/speclink-<triple>[.exe]。

**失敗模式**：

- --profile 收到白名單外的值：stderr 輸出點名該值與合法值的錯誤，exit code 1，不執行任何建置或複製。
- cargo build 失敗：沿用現行 checkSpawn 語意以非零結束；predev 非零使 npm run dev 中止，tauri dev 不進入編譯、dev 視窗不開——不以缺檔或過期檔繼續。
- 內容比對本身失敗（來源檔缺失）：明確報錯非零，不靜默跳過。

**驗收標準**：

- scripts/desktop-sidecar.test.mjs：--profile 解析（預設 release、debug 合法、未知值報錯點名）、來源路徑推導（profile × --target × Windows .exe 的組合）、跳過判定（目的不存在 → 複製；內容相同 → 跳過；內容相異 → 複製）。
- scripts/dev.test.mjs 新增設定守門：apps/desktop/package.json 存在 predev script、其內容呼叫 desktop-sidecar.mjs 且帶 --profile debug——防止 hook 被移除或改成 release 而靜默退化。
- 手動驗收：移走 binaries/ 模擬全新 checkout 後 npm run dev:desktop 正常開啟視窗；連續兩次啟動第二次 sidecar 檔案未被改寫且終端無 speclink-desktop 重編輸出；node scripts/desktop-sidecar.mjs --profile bogus 以 exit 1 結束。
- release 未回歸：scripts/desktop-install.mjs 檔案內容零改動；無參數執行 desktop-sidecar.mjs 仍為 release 建置與佈署。

**範圍邊界**：in scope＝scripts/desktop-sidecar.mjs、scripts/desktop-sidecar.test.mjs、apps/desktop/package.json 的 scripts 區、scripts/dev.test.mjs 的設定守門段。out of scope＝任何 Rust 原始碼、scripts/dev.mjs、scripts/desktop-install.mjs、tauri.conf.json、CI workflow 檔、dev-harness 既有需求文字。實作中發現邊界外缺陷：記錄，不順手修。

## Risks / Trade-offs

- [回歸對照：golden 與 CLI 測試] → 零 Rust 改動、不涉 speclink CLI 的任何輸出面，golden 與 CLI 測試預期零影響；收尾仍以 node --test "scripts/**/*.test.mjs" 全量確認 scripts 面，並跑一次 npm test -w apps/desktop 確認 package.json 變動未擾動測試 harness。
- [跨平台：Windows] → predev 由 npm 執行、內容為 node 呼叫，無 shell 語法依賴；路徑推導含 .exe 與 --target 組合由單元測試覆蓋；rustc -vV 的 host triple 偵測沿用現行程式。macOS 本機可實測，Windows 由 CI 與下次該機開發時驗證。
- [跨平台：CI 交叉編譯] → CI 以 --target 呼叫且無 --profile，預設 release 保證行為不變；--target 與 --profile 的路徑組合有測試釘住。
- [第一次啟動變慢] → 全新 checkout 的 predev 要付一次 debug CLI 建置（冷建置分鐘級）——但這本來就是缺的必要條件，現行是直接失敗；增量情況 cargo 為秒級 no-op，防抖確保不觸發後續重編。
- [npm pre-hook 的隱性觸發] → predev 對不知情者是隱性行為，佈署輸出訊息即為線索；設定守門測試防止被誤刪。

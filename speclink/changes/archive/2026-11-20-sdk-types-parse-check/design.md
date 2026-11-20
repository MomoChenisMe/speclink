## Context

`@speclink/engine` 0.13.0 的型別定義檔有一段 JSDoc 提早結束（範例裡的 `'*/*'`），匯入這個套件的 TypeScript 專案全部編譯不過。型別定義檔是手寫的（crates/adapters/speclink-node/index.d.ts），不是從 Rust 產生的那一份 binding.d.ts；發佈的主套件只帶 index.js、index.d.ts 與 binding.js 三個檔。

現有的保護有兩層，都照不到這個錯：

- crates/adapters/speclink-node 的測試用 vitest。vitest 只轉譯測試檔，測試匯入的是 index.js，沒有任何一步解析 index.d.ts；這個套件的開發相依也沒有 typescript。
- .github/workflows/node-sdk.yml 的 package 工作在打包後檢查 tarball 的檔案清單（主套件有平台載入器、子套件有平台二進位），不檢查內容能不能用。

另一個缺口：測試在 build 工作裡對建置產物執行；package 工作組出來的 tarball 沒有被當成使用端載入過。

限制：

- package 工作在 ubuntu-latest（x64）執行，預設工作目錄是 crates/adapters/speclink-node，只對這個套件執行過 npm ci，repo 根的相依沒有安裝。
- release 觸發時，蓋章步驟把主套件的 optionalDependencies 寫成 tag 版號的五個平台子套件，那個版號在檢查當下還沒有發佈。
- scripts/ 的測試在 .github/workflows/ci.yml 的三個平台（ubuntu、macOS、Windows）都跑，前面先對 repo 根執行過 npm ci。

## Goals / Non-Goals

**Goals:**

- 發佈出去的型別定義檔能被 TypeScript 編譯。
- 同類的錯（型別定義檔語法錯誤、執行期多了匯出而型別定義檔沒跟上、打包好的套件載入不了）在產出可發佈的 tarball 之前被擋下。
- 檢查邏輯能在本機、不經 CI 執行與測試。

**Non-Goals:**

- 不改任何型別宣告或執行期行為；這次只動一行註解。
- 不在 crates/adapters/speclink-node 的 vitest 測試裡另外加一個編譯 index.d.ts 的測試。檢查只做一份，對象是打包好的 tarball，那才是宿主拿到的東西。
- 不檢查型別定義檔的內容是否和 Rust 端的實際行為一致（例如回傳值的形狀）；那是 SDK 測試的事。
- 不為 Linux arm64、macOS x64 加上執行測試的機器（討論 embedded-host-needs-4 延後）。
- 不對五個平台各檢查一次：型別定義檔與 index.js 在每個平台都是同一份，一個平台足夠。
- 發 0.13.1 不是這個變更的任務；封存後用既有的發版流程處理。
- 命令名稱清單、原因代碼與其他觀察的修正在討論的 cut 2，不在這裡。

## Decisions

### 決策一：檢查寫成一支腳本，放在 scripts/npm

檢查邏輯放進新檔 scripts/npm/npm-engine-consumer-check.mjs，workflow 只負責呼叫它。這和同資料夾既有的 npm-engine-package.mjs（版號蓋章）是同一個做法：邏輯在腳本裡，旁邊有一支 node:test 測試，workflow 的 YAML 保持一行。

腳本落在 repo 的建置工具層，不屬於任何 crate；它不碰引擎、宿主套件或儲存介面，speclink-core、speclink-host 與各 store crate 都不受影響。

沒有選的做法：把步驟直接寫在 workflow 的 run 區塊。那樣在本機跑不了，四種結果（通過、語法錯誤、缺宣告、載入失敗）也沒辦法寫測試。

### 決策二：用解開 tarball 的方式組出使用端的 node_modules

腳本在系統暫存資料夾建立一個使用端資料夾，把每個 tarball 複製進去，以工作目錄設在那個資料夾、只帶相對檔名的方式呼叫 tar 解開，再把解出來的 package 資料夾搬到 node_modules 底下、位置由該套件 package.json 的 name 決定（主套件是 node_modules/@speclink/engine）。結束時（不論成功或失敗）刪掉暫存資料夾。

沒有選的做法：在暫存資料夾對 tarball 執行 npm install。release 觸發時主套件的 optionalDependencies 指向還沒發佈的版號，npm 會去 registry 解析它們；這讓檢查的結果取決於網路與 npm 對「選用相依解析失敗」的處理方式，而發版當下是最不該有不確定性的時候。這個套件除了平台子套件沒有其他相依，解開 tarball 得到的檔案和安裝的結果相同。

tar 一律收相對檔名、靠工作目錄定位：Windows 上 Git 附的 GNU tar 會把含磁碟機代號（如 C:）的路徑當成遠端主機，測試在 Windows 的 CI 上也要跑。

### 決策三：匯入的名稱取自執行期的實際匯出

腳本先在使用端資料夾起一個 Node 子程序載入主套件，把匯出物件的鍵寫進使用端資料夾裡的一個檔案（不經 stdout：套件載入時自己印的字不影響結果）。再用這份名稱產生使用端的 TypeScript 檔：一行具名匯入全部名稱，一行把它們放進一個匯出的陣列（避免未使用的匯入被忽略）。

匯入的那一行把每個名稱寫成字串形式、各配一個腳本自己編的別名（`import { "createEngine" as e0, "agentTool" as e1 } from …`）。這樣名稱不必是合法的識別字：`default` 或含連字號的名稱照樣比對，缺宣告時編譯器回報的是「沒有這個匯出的成員」並點名那個名稱，不是語法錯誤。字串形式的匯入名稱從 TypeScript 5.6 開始支援，決策四的版本範圍（^5.7.0）涵蓋得到。

載入成功但匯出的名稱是零個時，腳本以載入失敗結束、不執行編譯：沒有名稱可以比對時，具名匯入那一行是空的，編譯一定通過，等於什麼都沒檢查。

這樣不需要手寫一份匯出清單：往後 index.js 新增匯出（例如 cut 2 的 commandNames）而 index.d.ts 沒有宣告，編譯就會失敗並點名那個名稱；反過來，型別定義檔宣告了執行期沒有的值不在這個檢查的範圍內。

載入這一步同時補上「打包好的套件沒有被載入過」的缺口。主套件載入時需要平台二進位，所以 workflow 呼叫時要給平台子套件的 tarball；腳本的平台參數是選填，因為測試用的假套件不需要原生模組。

沒有選的做法：在腳本裡寫死七個名稱。清單會和 index.js 脫節，而且脫節時檢查照樣通過。

沒有選的做法：遇到不是合法識別字的名稱就以專屬的訊息失敗。那會多一種失敗，而且擋下的是腳本自己處理不了的寫法，不是套件的錯。

### 決策四：TypeScript 編譯器從執行腳本的資料夾解析

腳本以執行當下的工作目錄為起點解析 typescript 套件，用目前的 Node 執行它的 tsc 入口（不經過 .cmd 轉接檔，三個平台同一種呼叫方式）。編譯的設定寫在使用端資料夾的 tsconfig：module 與 moduleResolution 都是 nodenext、strict、noEmit、skipLibCheck 為 false、types 為空陣列。

- 在 package 工作裡，工作目錄是 crates/adapters/speclink-node，所以這個套件的開發相依要加上 typescript，版本範圍和三個前端 workspace 相同（^5.7.0），package-lock.json 一併更新。typescript 只是開發相依，不在 files 清單裡，不進發佈的套件。
- 在 scripts/ 的測試裡，工作目錄是 repo 根，用的是根目錄已經安裝的 typescript。

解析不到 typescript 時，腳本以非零結束，訊息寫明是從哪個資料夾找不到，並提示在有安裝 typescript 的資料夾執行。

沒有選的做法：檢查當下用 npx 下載 typescript。那會讓檢查依賴網路，版本也不固定。

### 決策五：註解改寫成跳脫的斜線

index.d.ts 裡 Express 範例的 `'*/*'` 改成 `'*\/*'`。在 JavaScript 的字串裡，反斜線加斜線就是斜線，所以範例的意思不變，讀者照樣可以複製。docs/sdk-node.md 與 docs/sdk-node.zh-TW.md 裡同一段範例在 Markdown 的程式碼區塊內，沒有這個問題，不改。

沒有選的做法：把範例的型別條件換成別的寫法（例如回傳 true 的函式）——那會讓型別說明和 SDK 文件的範例不一致；或把範例從型別說明拿掉——編輯器的提示會少掉最有用的一段。

### 決策六：workflow 的步驟位置與守門測試

node-sdk.yml 的 package 工作新增一個步驟，放在打包步驟（Pack the main package and every platform sub-package）之後、上傳步驟（Upload npm tarballs）之前，沒有執行條件，所以 push、PR 與 release 都會跑。它以 package 工作的預設工作目錄執行腳本，主套件參數給工作目錄下唯一的 tgz，平台參數給 npm/linux-x64-gnu 底下的 tgz。步驟失敗時工作就停在這裡，後面的上傳不執行，release 管線也就拿不到 tarball。

scripts/release/delivery-gate.test.mjs 既有的 node-sdk.yml 測試用步驟名稱與字串位置守住步驟順序；這次比照加上三個斷言：package 工作裡有呼叫 npm-engine-consumer-check.mjs 的步驟、它排在打包步驟之後、排在上傳步驟之前。

## Implementation Contract

**行為**

- 腳本的呼叫形式：node scripts/npm/npm-engine-consumer-check.mjs --main <主套件 tgz> [--platform <平台子套件 tgz>]。
- 三項都成立時：exit code 0，stdout 印一行結論，內容含主套件的名稱與檢查到的匯出名稱個數；不改動傳入的 tarball。
- 載入失敗：exit code 非零，stderr 第一行以 load failed 開頭，後面是 Node 子程序的錯誤輸出；不執行編譯。
- 載入成功但沒有任何匯出的名稱：exit code 非零，stderr 第一行以 load failed 開頭，並說明主套件沒有任何執行期匯出的名稱；不執行編譯。
- 匯出的名稱不是合法識別字（`default`、含連字號）：照樣比對。有宣告時通過；沒有宣告時是編譯失敗，tsc 的訊息點名那個名稱。
- 編譯失敗（語法錯誤、型別錯誤、匯出缺宣告）：exit code 非零，stderr 第一行以 type check failed 開頭，後面是 tsc 的完整輸出。
- 用法錯誤（沒有 --main、tarball 不存在、tarball 裡沒有 package/package.json、解析不到 typescript）：exit code 非零，stderr 說明哪個參數或哪個檔有問題，不留下暫存資料夾。
- 不論結果，暫存資料夾都被刪除。
- 收到 SIGINT 或 SIGTERM：刪掉暫存資料夾之後以 128 加訊號編號結束（130、143），stderr 有一行以 interrupted 開頭；不以 0 結束。
- 每個子程序（tar、載入、tsc）有 120 秒的上限；超過時該步驟算失敗，暫存資料夾照樣刪除。
- 腳本不連線到 registry，也不執行被檢查套件的安裝腳本。

**驗收**

- scripts/npm/npm-engine-consumer-check.test.mjs 以假套件涵蓋四種結果：完好的套件通過；型別定義檔的註解含 `'*/*'` 時失敗且 stderr 含 TS1131 與 index.d.ts；執行期匯出 commandNames 而型別定義檔沒有宣告時失敗且 stderr 含 commandNames；index.js 載入時拋錯時失敗、stderr 以 load failed 開頭、沒有 tsc 的輸出。另有兩個用法錯誤的案例（缺 --main、tarball 沒有 package/package.json）。同一個檔裡的案例以並行方式執行。
- 同一個測試檔另外涵蓋：匯出是空物件與匯出是沒有屬性的函式時以 load failed 失敗、沒有 tsc 的輸出；執行期匯出 `a-b` 與 `default` 而型別定義檔宣告了 `a-b` 時通過，匯出個數包含這兩個名稱；執行期匯出 `x-y` 而沒有宣告時以 type check failed 失敗，stderr 含 TS2305 與 `x-y`、不含 consumer.mts 的語法錯誤。
- scripts/release/delivery-gate.test.mjs 的 node-sdk.yml 測試守住新步驟的存在與位置。
- 本機預演：建置 napi binding，打包主套件與本機平台的子套件，對兩個 tarball 執行腳本。修正 index.d.ts 之前，結果是失敗且第一個錯誤是 index.d.ts 的 TS1131；修正之後 exit code 是 0。
- crates/adapters/speclink-node 的 npm ci 在加上 typescript 之後成功，既有的 vitest 測試不受影響。

**範圍**

- 範圍內：決策一到六列出的腳本、測試、workflow 步驟、開發相依與那一行註解。
- 範圍外：index.d.ts 的其他內容、index.js、Rust 程式碼、release.yml、npm-publish.yml、其他 npm 套件（CLI 與 server 的套件只帶可執行檔的啟動器，沒有型別定義檔）。

## Risks / Trade-offs

- [scripts/ 的測試多了三次 tsc 執行，整組不再是一秒內跑完] → 同一個測試檔裡的案例並行執行，假套件的型別定義檔只有幾行、types 設成空陣列；預期多出一到兩秒。這組測試先於其他測試面執行的理由（先擋下設定壞掉）不變。
- [Windows 的 CI 上 tar 與暫存路徑的行為不同] → 決策二的相對檔名做法；測試在三個平台的 CI 都跑。本機只能驗 macOS，Windows 與 Linux 要靠 CI。
- [package 工作的機器換成別的平台或架構，寫死的 linux-x64-gnu 子套件就載入不了] → 那時載入會以 load failed 明確失敗，不會默默通過；改步驟的平台參數即可。
- [執行期的匯出名稱不是合法的識別字] → 匯入名稱一律寫成字串形式（決策三），不是合法識別字的名稱照樣比對；這個寫法需要 TypeScript 5.6 以上，版本範圍 ^5.7.0 與根目錄已安裝的版本都符合。
- [檢查誤判讓發版卡住] → 檢查不連網、輸入只有 tarball，同一份 tarball 的結果固定；push 與 PR 每次都跑同一步，問題會在發版前出現。
- [既有輸出的回歸對照] → 這個變更不動任何 CLI 或 SDK 的輸出，golden 與 CLI 測試不需要更新。

## Migration Plan

沒有資料或設定要遷移。合併後下一次 push 的 Node SDK workflow 就會執行新步驟；要撤回時，還原 workflow 的那一個步驟即可，腳本與開發相依留著不影響發佈的套件。已經用修補檔繞過 0.13.0 的宿主，升到 0.13.1 後拿掉修補檔。

## Open Questions

沒有。

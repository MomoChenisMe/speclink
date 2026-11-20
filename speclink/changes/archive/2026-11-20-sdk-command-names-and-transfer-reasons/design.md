## Context

內嵌宿主第四輪回報的五項，查證後的現況：

- **命令名稱**：`Command::name()` 是一個沒有萬用分支的 match，54 個名稱散在分支裡，沒有一份可以列舉的清單。宿主套件另有 `scope-export`、`scope-import` 兩個常數，HTTP 入口寫看板排序（`PUT /board-order`）時還會以 `put-board-order` 問掛鉤並寫 commit，這個字串在 HTTP 路由與看板文件模組各打一次。`@speclink/engine` 的七個執行期匯出裡沒有清單。
- **匯出與匯入的失敗**：匯入的失敗有型別（`TransferError`），但「內容超過上限」「文件不合規則」「格式版本不支援」三種都收在同一個帶字串的變體裡。本機匯出清單的失敗是沒有型別的字串。Node 這一層的信封只有 `code` 與 `message`，`index.js` 的 `unwrap` 只把 `code` 放到 Error 上。
- **`new artifact` 的 `path`**：經儲存接口執行時，回報的路徑由儲存接口的 artifact 寫入函式組出，開頭帶規格資料夾的預設名稱。遠端 CLI 不讀這個值：它印的路徑是自己組的，Server 的寫入回應也不帶路徑。所以這個值只出現在內嵌宿主的 `host.dispatch` 結果裡。
- **snake_case 欄位**：實跑全部讀取動詞加上已知的寫入動詞，帶底線的欄位只出現在 `task done`／`task undone`、`discuss archive`、`analyze`、`drift` 四組。verb-contract 與 change-analysis 的規格已經釘住其中兩組。
- **空字串的 `locale`**：行為是規格明寫的；`set --help` 的 `<VALUE>` 說明只提到布林欄位。

限制：speclink-core 不含儲存媒介與網路；HTTP 入口的狀態碼與 body 是遠端 CLI 與 Speclink Desktop 依賴的契約，這個變更不動它。

## Goals / Non-Goals

**Goals:**

- 宿主用一行匯入拿到全部命令名稱，並能寫一個測試確認自己的對照表涵蓋每個名稱。
- 宿主依 `error.reason` 分辨匯出與匯入的八種失敗，不比對訊息。
- 內嵌時 `new artifact` 的 `path` 與 SDK 文件、`agentTool.description` 的說法一致。
- 文件對 `--json` 欄位命名的說法符合現況；`set --help` 寫出空值的作用。

**Non-Goals:**

- `commandNames` 不帶「會不會寫入」。不寫入不等於可以放行：`scope-export`、`workflow-config show`、`discuss show` 都不寫入但都帶出全文，這個旗標會引導宿主寫出「讀取一律放行」的預設。
- 一般動詞的失敗不加 `reason`。失敗點有上百個，每一個都給穩定代碼等於多凍結一份契約。
- 不把 snake_case 的欄位改名成 camelCase：它們已經是凍結的契約，技能與 Speclink Desktop 都在讀。
- 不拒絕空字串的 `locale`：它是把欄位設回未設定的唯一做法。
- 不改 HTTP 入口（`GET /export`、`POST /import`）的回應，不改 Rust 這一層失敗的 `code` 與訊息。
- 不改儲存接口回報的變更資料夾（它被許多訊息共用，包含「Artifact already exists」），只改 artifact 寫入與建立變更這兩個回傳值。
- 這次不給代碼的失敗：匯入包的格式版本不支援、傳入的不是匯入包、變更指定的產出流程找不到、匯出時範圍一直在變。它們之後可以加上代碼而不破壞相容，因為「沒有 `reason`」本來就是合法的結果。

## Decisions

### 命令名稱清單是 speclink-core 的一份常數

在 `Command::name()` 旁邊加一份常數陣列，列出 54 個名稱，順序照 match 的分支。既有的逐命令對照測試已經保證「每一種命令恰有一個樣本」，在同一個測試檔加一條斷言：樣本的名稱集合等於常數的集合，而且常數沒有重複。新增命令時，`name()` 的 match 先不通過編譯；補上分支後，這條斷言接著要求把名稱加進常數。

替代做法：用巨集從一份清單同時產生 match 與常數。它讓兩者不可能不同步，但要改寫整個 match 與它的說明；一條測試斷言已經足夠，而且失敗訊息能直接點名缺的名稱。

### 宿主套件在引擎的清單後面加上自己的三個名稱

speclink-host 提供一個函式回傳完整的清單：引擎的常數加上 `scope-export`、`scope-import` 與 `put-board-order`。前兩個沿用匯出匯入模組既有的常數；`put-board-order` 改成看板文件模組的一個公開常數，HTTP 路由與 commit 都讀它，不再各打一次字串。清單的唯一實作落點在這裡；Node 綁定只轉交。品質關卡第 1 輪之後，這個函式由匯出匯入模組搬到宿主套件模組（`kit.rs`）：清單是宿主套件的，也用到看板文件模組的常數。任務 1.1、1.2 的文字仍寫原本的位置。

`put-board-order` 在 apply 期才找到：`host.handleRequest` 的 `PUT /board-order` 會以這個名稱問 beforeCommand 並寫 commit，漏掉它就違反「掛鉤收到的名稱都在清單裡」。

### SDK 在載入時取一次清單並凍結

speclink-node 加一個原生函式回傳字串陣列。`index.js` 在模組載入時呼叫一次，把結果以 `Object.freeze` 凍結後匯出成 `commandNames`。`index.d.ts` 宣告 `export const commandNames: readonly string[]`，說明寫明：含宿主套件的三個名稱、順序不是契約、不帶讀寫資訊。0.13.1 加進打包流程的使用端檢查會比對執行期匯出與型別定義檔，所以漏掉宣告時 CI 會失敗。

### 原因代碼是失敗型別上的一個方法

在 speclink-host 的匯出匯入模組加一個原因的列舉，八個成員各對應一個固定的字串（`no_spec_folder` 等）。原因由失敗的種類決定，不解析訊息：

- 匯入的失敗型別把「帶字串的不合法匯入包」拆開：內容超過上限、文件不合規則各成一個變體，其餘（格式版本不支援、第一版文件沒有對應的鍵）留在原變體。三個變體的訊息與 `code`（`invalid_argv`）都不變。「目標不是空的」維持原樣。型別提供一個方法回傳選填的原因。
- 本機匯出清單的函式，回傳的錯誤由字串改成一個小型別：訊息加上選填的原因。它的顯示文字就是原本的字串，所以 Speclink Desktop 的呼叫端只要把錯誤轉成字串，行為不變。五個有代碼的失敗點各自帶上原因；其餘失敗點（產出流程找不到、重複的文件、序列化）不帶。
- 「沒有規格資料夾」的檢查在 speclink-node 的本機匯出函式裡，原因在那裡帶上。

替代做法：在 Node 這一層用訊息的開頭對應代碼。這正是宿主現在的做法，改措辭就失效，所以不採用。

### 原因代碼經信封的選填欄位送到 JS

Node 綁定的失敗結構多一個選填的原因；信封只在有原因時多一個 `reason` 鍵。`index.js` 的 `unwrap` 在信封有 `reason` 時把它設到 Error 上，沒有時不設，所以 `'reason' in error` 在其他失敗是 `false`。宿主套件的匯入失敗轉成命令層的失敗時，把原本的失敗掛在命令失敗既有的來源欄位上；Node 綁定從來源取回它並問原因。`host.handleRequest` 走另一條路徑組回應，不經這個信封，HTTP 回應因此不變。

`index.d.ts` 在 `exportScope`（兩處）與 `importScope` 的說明列出代碼，並寫明沒有 `reason` 的失敗有哪些。型別上不新增 Error 的子類別：`reason` 和 `code` 一樣是掛在一般 Error 上的屬性，文件用說明與範例表達。

### new artifact 與 discuss promote 的 path 只改儲存接口的兩個回傳值

儲存接口的 artifact 寫入函式，回傳值改成它寫入的那個鍵（`changes/<變更>/<artifact>`），和它呼叫儲存時用的鍵是同一個來源，不再另外拼字串。儲存接口建立變更的函式，回傳值改成新變更的鍵的共同開頭（`changes/<變更>`）。儲存接口回報的變更資料夾（變更清單裡每個變更的資料夾）不動。

這兩個回傳值的讀者是 `new artifact` 與 `discuss promote` 的結果；`new change` 沒有 `--json`，內嵌宿主的人眼輸出本來就不印路徑。Server 也用同一個儲存接口，但它的寫入回應不帶路徑，遠端 CLI 印的路徑是自己組的，所以連到遠端的 CLI 輸出不變。

`discuss promote` 是 apply 期的排查（任務 5.3）找到的：它的 `path` 由建立變更的回傳值組出，內嵌時也帶 `speclink/` 前綴。

已知的不一致，這次不動：

- artifact 已存在時的拒絕訊息仍然寫 `speclink/changes/…`，因為它由變更資料夾組出，而那個值被許多訊息共用。訊息是給人看的文字，不是鍵；寫進規格當成刻意維持。
- 內嵌時 `instructions --json` 的 `changeDir` 與 `contextFiles` 也由變更資料夾組出，仍以 `speclink/changes/` 開頭，雖然內嵌宿主沒有這些檔案。改它要動變更資料夾本身或 instructions 的組法，範圍較大，留給之後的變更。

### snake_case 的欄位清單寫進規格與文件

verb-contract 新增一條需求列出四組動詞的欄位，`docs/verb-contract`（英文與繁中）加一張同樣內容的表。`docs/sdk-node`（英文與繁中）說明 dispatch 結果的那一句改寫，並連到那張表。不新增執行期的機制。實作時用一個指令掃過全部動詞的 `--json` 輸出再確認一次清單；找到表裡沒有的欄位時，先更新 delta spec 與文件再繼續。

### set 的說明改在動詞定義的一處

`<VALUE>` 的說明文字在 speclink-verbs 的流程設定動詞定義裡，CLI 的 `--help` 與動詞層的用法錯誤訊息都讀它。只改這一處。`docs/sdk-node`（英文與繁中）說明流程政策動詞的段落補同一句。

## Implementation Contract

**行為**

- `require('@speclink/engine').commandNames` 是凍結的陣列，57 個不重複的字串，含引擎全部命令名稱與 `scope-export`、`scope-import`、`put-board-order`。
- `engine.exportScope`、`host.importScope` 的失敗在八種原因下帶 `error.reason`，值與原因的對應見 node-sdk 的 delta spec；其他失敗沒有 `reason` 屬性。`code` 與 `message` 逐字不變。
- `host.dispatch(['new','artifact',…,'--json'])` 的 `path` 是 `changes/<變更>/<artifact 的路徑>`；`host.dispatch(['discuss','promote',…,'--json'])` 的 `path` 是 `changes/<變更>`。
- `speclink workflow-config set --help` 的 `<VALUE>` 說明含 `an empty value unsets locale or spec_locale`。

**介面與資料形狀**

- speclink-core：一份公開的常數陣列（54 個 `&'static str`）。
- speclink-host：一個回傳完整清單的公開函式；一個原因的列舉，能轉成固定字串；匯入失敗型別與本機匯出失敗型別各有一個回傳選填原因的方法。本機匯出清單的函式簽名改變（錯誤型別），兩個呼叫端（speclink-node 與 Speclink Desktop 的搬移）一起改。
- speclink-node：一個回傳字串陣列的原生函式；信封的 `reason` 鍵（只在有原因時出現）。
- 八個原因字串：`no_spec_folder`、`spec_folder_outside_project`、`symlink_in_spec_folder`、`spec_folder_unreadable`、`invalid_change_metadata`、`scope_not_empty`、`bundle_too_large`、`invalid_bundle_document`。

**失敗模式**

- 沒有新的失敗。既有失敗的 `code`、訊息、HTTP 狀態碼都不變。
- 不在清單裡的失敗刻意不帶 `reason`；呼叫端要把「沒有 `reason`」當成合法的結果。

**驗收**

- speclink-core 的命令對照測試多一條「常數等於全部命令名稱」的斷言並通過。
- speclink-host 的單元測試：每一個帶原因的失敗點回傳對應的原因；不帶原因的失敗點回傳「沒有」；儲存接口寫入 artifact 回傳鍵。
- speclink-node 的 vitest：`commandNames` 的內容與凍結狀態；八種原因各一個案例加上四個沒有原因的案例；內嵌 `new artifact --json` 的 `path`；createEngine 的 `path` 不變。
- 打包後的使用端檢查通過，匯出個數由 7 變成 8。
- `node --test` 的文件守門與 `cargo test` 的既有回歸對照（golden、CLI 的本機與 remote 對照）都不需要更新期望值；有一個需要更新就表示改到了範圍外的輸出。

**範圍**

- 範圍內：上列四個 crate 的程式與測試（含 speclink-host 的看板文件模組與 HTTP 路由改讀同一個常數）、`index.js`、`index.d.ts`、四份文件、Speclink Desktop 呼叫本機匯出清單的那一處。
- 範圍外：HTTP 入口的回應、speclink-cli 與 speclink-server 的行為、技能資產（不 bump ASSET_VERSION）、其他動詞的失敗、既有 `--json` 欄位的名稱。

## Risks / Trade-offs

- [改到凍結的輸出] 儲存接口也供 Server 使用，回傳值的改動有機會漏到別的輸出 → 以 CLI 的本機與 remote 對照測試、Server 的端對端測試當回歸對照；它們的期望值都不該變。
- [本機匯出清單的錯誤型別改了，Desktop 的搬移行為跟著變] → 新型別的顯示文字與原字串逐字相同；Desktop 的既有測試（搬移的中止訊息）當對照。改完要跑 speclink-desktop 這個 crate 的測試。
- [`spec_folder_unreadable` 的測試跨平台] Windows 沒有 000 這種權限，root 身分下權限也擋不住 → 這個案例在 Windows 與 root 身分下略過，Rust 的單元測試直接對回傳原因的函式斷言，不依賴檔案權限。
- [符號連結的測試在 Windows] 建立符號連結需要權限 → 沿用既有匯出測試的做法（建立失敗就略過該案例）。
- [清單漏列 snake_case 欄位] 實跑只涵蓋當下有資料的輸出 → 實作時再掃一次，包含有發現的 `analyze` 與每個寫入動詞；多出來的欄位先進 delta spec。
- [宿主把 `commandNames` 的順序當成契約] → 規格與型別說明都寫明順序不是契約。
- [之後要給其他失敗加代碼] → 規格寫明新代碼只加在還沒有 `reason` 的失敗上，既有代碼的意思不變。

## Migration Plan

沒有資料或設定要遷移。這是 SDK 的新增加上一處內嵌輸出的修正，隨下一個版本發佈。依賴 `speclink/` 前綴的宿主在升級時拿掉那個假設；更新日誌要寫明這一點。要撤回時還原這個變更的 commit 即可，沒有寫入任何持久的資料。

## Open Questions

沒有。

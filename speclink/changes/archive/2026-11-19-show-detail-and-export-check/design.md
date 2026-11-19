## Context

討論 embedded-host-needs-3 對照 main（engine-export-scope 封存後）確認了三件事：

- 蓋章時的檔案指紋（`reviewed_scope`、`verified_scope`）引擎早就讀進關卡狀態裡，`show --json` 輸出時刻意不帶。本機路徑由 speclink-core 的關卡狀態判定讀出完整的章；遠端路徑由 speclink-remote 把 `GET /changes/{name}` 回應的 `review`、`verify`（含 `scope`）轉成同一個關卡狀態。兩條路最後都交給 speclink-verbs 的 `show` 輸出函式，由它把關卡狀態序列化成 JSON。`list --json --detail` 也用同一個序列化（不含 `scope`）。
- Node 的 `createEngine({ store: { type: 'fs', root } }).exportScope()` 由 speclink-node 呼叫 speclink-host 的共用組包函式（Speclink Desktop 的「遷移到 Server」用同一支）。組包函式先檢查規格資料夾名稱（不能是絕對路徑、不能有 `..` 或 `.` 段）與符號連結，規格資料夾不存在時回傳空的清單。Desktop 新增本機資料夾時不檢查裡面有沒有規格資料夾，所以「把空資料夾遷移到 Server」是 Desktop 的合理用法。
- SDK 文件已有封存條件的掛鉤範例；釋放認領的範例只在 docs/verb-contract。

## Goals / Non-Goals

**Goals:**

- 讀取動詞（含掛鉤的 `read`）能拿到蓋章時的檔案指紋，形狀和線上格式相同。
- 宿主在後端匯出放錯位置的資料夾時得到明確的錯誤，而不是零份文件。
- 宿主的整合者只看 SDK 文件就能完成「在伺服端匯入」與「限定誰能釋放認領」。

**Non-Goals:**

- 不改 `show --json` 的預設輸出：讀它的技能用不到雜湊。
- 不讓 `list --json --detail` 帶 `scope`：與線上清單一致。
- 不改 `review show`、`verify show` 的意思：它們讀工單，工單在蓋章後就刪掉了。
- 不改共用的組包函式，Speclink Desktop 的「遷移到 Server」照舊接受空資料夾。
- 不做規格資料夾的持續同步，也不讓匯入覆寫非空的範圍（workspace-migration 只接受 CreateNew）。
- 引擎不限定誰能釋放認領，宿主傳入的身分不加角色。
- 不加 CLI 的匯入匯出動詞（已知缺口維持）。
- 線上格式、speclink-host、speclink-protocol、speclink-remote、apps/desktop 的程式碼不改（speclink-host 只在 transfer.rs 補一個測試，釘住沒有規格資料夾的資料夾得到空清單）。

## Decisions

### `--detail` 只影響 show 的輸出，不進引擎命令

`ShowArgs` 新增 `--detail`，以 clap 的 `requires = "json"` 綁住 `--json`，和 `list --detail` 宣告的方式相同；用法錯誤因此在解析階段就發生，CLI、`host.dispatch` 與掛鉤的 `read` 都走同一份解析（宿主套件把解析錯誤轉成 `invalid_argv`）。引擎的 `Show` 命令不新增欄位：`show` 的結果本來就帶著完整的關卡狀態（含 `scope`），引擎不需要為了 `--detail` 多讀任何東西。

- 替代：把 `detail` 放進引擎的 `Show` 命令（`list --detail` 的做法）。沒有採用：`list --detail` 要引擎多讀每個變更的 metadata，所以放進命令；`show` 已經讀好了。放進命令還會改動命令描述的 golden、掛鉤收到的 `params` 與宿主套件裡建立 `Show` 命令的每一處，卻沒有任何行為需要它。
- 結果：`show` 的掛鉤 `params` 不變；命令描述的 golden 不變。

### 關卡狀態帶 scope 的 JSON 形狀只定義一次

speclink-core 的關卡狀態新增一個序列化方法，在既有的 `show --json` 形狀上，於 `state` 為 `stamped` 時加上 `scope`：陣列，每項 `{ "path": 字串, "hash": 字串 }`，順序照 metadata 記錄的順序。既有的序列化（`show --json` 預設與 `list --json --detail` 共用）不變。`scope` 的鍵名與每項的鍵名和線上格式 `StationState.scope` 相同（protocol 的審查指紋項目），所以三種執行方式輸出同形。

- 已蓋章但沒有記下任何指紋：`scope` 是空陣列，鍵照樣出現，讓讀的人分得出「有帶 `--detail`」與「沒帶」。引擎與 Server 自己判定的章一定有指紋（沒有指紋的 metadata 讀作沒有蓋章），所以這只會出現在回應的關卡是 `stamped` 卻沒有帶 `scope` 時。
- `inProgress` 或沒有記錄的關卡：與預設輸出相同，不加 `scope`。
- 修正 `ShowChange` 上「without the scope」的註解：它帶著 scope，只是預設輸出不印。

### show 的輸出函式收旗標，三種執行方式共用一支

speclink-verbs 的 `show` 輸出函式多收一個「是否帶詳細」的參數，傳給 `show --json` 的組裝。呼叫點有三處，都傳同一個值：

- CLI 本機路徑（`cmd_show`）與遠端路徑（`remote_show`）傳 `ShowArgs` 的 `detail`。遠端路徑的資料來自既有的 `GET /changes/{name}` 回應，不多發請求；連到不回報關卡的舊版 Server 時，`review`、`verify` 本來就不出現，指令不失敗。
- 宿主套件與 Node `engine.dispatch` 共用的輸出分派（`render_with`）從已解析的動詞讀出 `show` 的 `detail`。

人眼輸出不受影響：`--detail` 不能脫離 `--json`。`--no-color` 對 JSON 沒有作用，輸出不隨 `locale` 改變。

### exportScope 在既有檢查之後確認規格資料夾

speclink-node 的本機匯出在共用組包函式成功回傳之後，以引擎既有的「是不是規格資料夾」判斷（資料夾裡有 `changes/`、`specs/`、`discussions/` 或 `config.yaml`）檢查 `root` 加上規格資料夾名稱的位置；不是規格資料夾時，整個匯出以 Error 拒絕，`code` 是 `error`，不回傳任何包。

- 順序的理由：組包函式先檢查名稱與符號連結，所以壞的名稱（例如 `../other/speclink`）仍然得到「spec folder outside the project」的錯誤，檢查規格資料夾時不會碰到專案外的路徑；規格資料夾不存在時，組包函式什麼都讀不到、回傳空的清單，接著被這一步拒絕。
- message：`no spec folder: "<名稱>" under the root holds no changes/, specs/, discussions/ or config.yaml — point root at the folder above the spec folder, or set specDir`。以 `no spec folder:` 開頭，與既有兩種拒絕（`symbolic link in the spec folder:`、`spec folder outside the project:`）同一種寫法；只含資料夾名稱，不含任何檔案內容或絕對路徑。
- 替代一：把檢查放進共用組包函式。沒有採用：會擋掉 Desktop 把空資料夾遷移到 Server 的路。
- 替代二：改用「是不是專案根目錄」判斷（`.speclink.yaml`、規格資料夾、`openspec/` 或遠端設定檔其中之一）。沒有採用：只有 `.speclink.yaml`、沒有規格資料夾的 `root` 會通過，仍然得到零份文件。
- 這推翻 engine-export-scope 設計的「規格資料夾不存在：不算錯」。理由：匯出是一次性的搬遷，零份的結果看起來像成功；`dispatch` 讀錯位置時呼叫端馬上看得到，匯出不會。

### SDK 文件的三段放在既有段落旁邊

- 「在伺服端匯入」接在既有的「匯出本機專案」一節（錨點 `export-local`）之後，新錨點 `server-side-import`。內容四點：`root` 與 `store.specDir`（只下載規格資料夾時，`root` 指上一層，`specDir` 給資料夾名稱；不給時讀 `root/.speclink.yaml` 的 `spec_dir`）；`limits.importBytes` 也管程序內的 `importScope`（預設 32 MiB），匯出沒有上限，大專案要調高；匯入是一次性的搬遷不是同步；要重新匯入，先清空自己儲存裡這個範圍的全部文件再匯入，這會刪掉匯入後在宿主裡做的全部改動（變更、討論、蓋章）。另寫明放錯位置時 `exportScope` 以 `no spec folder:` 拒絕。
- 釋放認領的範例搬到掛鉤段落、封存範例之後：沿用 docs/verb-contract 現有範例的比對邏輯（讀 `show --json` 的 `claimed`，有 `byId` 比對識別，沒有時比對名稱），加一行 `if (await isAdmin(identity)) return`，並註明 `isAdmin` 是宿主自己的函式。讀取失敗的處理和原本的範例不同：只讓 `not_found` 通過（由釋放本身回報找不到變更），`read` 的其他錯誤往外拋，掛鉤失敗、這次呼叫不放行。理由：這是權限掛鉤，判斷不了時放行，會讓不是認領的人在儲存暫時出錯時釋放成功（品質關卡第 1 輪的審查發現）。範例寫成函式宣告 `async function beforeCommand(...)`，可以單獨貼上，以 `hooks: { beforeCommand }` 接上。「Speclink 不內建…規則」那句補上「也不限定誰可以釋放認領」。另寫明 `reason` 給誰看：CLI 原文印到 stderr；Speclink Desktop 沒有釋放認領的操作，認領被拒時失敗提示帶出原因。
- 蓋章指紋：在掛鉤段落加一段「封存前比對蓋章時的檔案」，用 `read(['show', name, '--json', '--detail'])` 讀 `review.scope` 與 `verify.scope`，逐項比對宿主自己算出的指紋。算法寫精確：能以 UTF-8 解讀的檔，把 CRLF 換成 LF 後取 SHA-256（小寫十六進位）；不能以 UTF-8 解讀的檔，取原始位元組的 SHA-256；開頭的 BOM 保留、不剝掉。範例以 Node 的 `TextDecoder('utf-8', { fatal: true, ignoreBOM: true })` 判斷能否解讀。寫明蓋章時已不存在的檔不在 `scope` 裡，宿主要確認刪除就要自己另外比對。既有蓋章段落裡「文字檔先把 CRLF 換成 LF」那句改成同樣精確的寫法。
- docs/verb-contract：`show --json` 欄位表之後加 `--detail` 的說明與範例；釋放認領一節刪掉範例，只留「命令名稱是 `claim-release`，只擋 `claim` 的掛鉤擋不到釋放」並連到 SDK 文件的新範例。
- docs/product-status：Node SDK 那一列補上 `show --json --detail` 帶出蓋章時的檔案指紋，日期更新。

## Implementation Contract

**行為**

- `speclink show demo --json --detail`（demo 的審查已蓋章，章記下 `src/a.rs` 的指紋）：stdout 是 JSON，`review` 物件含既有的 `state`、`stampedAt`、`rounds`、`tasksTotal`（以及有值的 `stampedBy`、`stampedWith`、`commit`），另含 `scope: [{ "path": "src/a.rs", "hash": "<64 個小寫十六進位>" }]`；exit code 0。
- `speclink show demo --json`：與這個變更之前逐位元相同，沒有 `scope`。
- `speclink show demo --detail`：exit code 非零，stderr 是 clap 的用法錯誤並指出 `--json`，stdout 沒有輸出。經 `host.dispatch` 或掛鉤的 `read` 執行同樣的 argv：以 `invalid_argv` 拒絕。
- `speclink show auth --json --detail`（auth 是正式規格）：輸出與不帶 `--detail` 相同。
- `speclink list --json --detail`：不變，任何一項都沒有 `scope`。
- 本機專案、連到遠端的 CLI、宿主套件對相同資料執行 `show demo --json --detail`，`review` 與 `verify` 深度相等。
- `createEngine({ store: { type: 'fs', root } }).exportScope()`，`root` 底下沒有 `speclink/`（也沒有 `.speclink.yaml` 指定別的名稱）：Promise 以 Error 拒絕，`code` 是 `error`，message 以 `no spec folder: "speclink"` 開頭；沒有回傳包，`root` 的檔案不變。
- 同上但 `root/speclink/` 只有一個空的 `specs/`：成功，`documents` 是空陣列。
- `spec_dir: ../other/speclink`：仍然是既有的 `spec folder outside the project:` 錯誤，不是新的錯誤。
- Speclink Desktop 對沒有規格資料夾的本機資料夾執行「遷移到 Server」：行為與這個變更之前相同。
- 以 Node SDK 文件的釋放範例當 `beforeCommand`：認領的人與管理者能釋放；其他人以 `refused` 拒絕；變更不存在時以 `not_found` 拒絕；範例裡的 `read` 因儲存出錯而失敗時，這次呼叫失敗、不釋放，儲存沒有收到寫入。

**介面 / 資料形狀**

- CLI 旗標：`show` 的 `--detail`（kebab-case，無值，需要 `--json`）。沒有 stdin。
- JSON：`review.scope`、`verify.scope`，camelCase 鍵名 `scope`、`path`、`hash`，只在 `state` 為 `stamped` 且帶 `--detail` 時出現。
- Node：`Engine.exportScope()` 簽名不變，只多一種拒絕；`index.d.ts` 的說明把「a `root` that is no project gives none」改成新的拒絕規則。

**驗收**

- speclink-verbs 的解析測試：`show --detail` 不帶 `--json` 被拒；帶 `--json` 時解析成功。
- speclink-verbs 的輸出測試：已蓋章、進行中、沒有記錄的關卡在帶與不帶 `--detail` 下的 JSON。
- CLI 整合測試（新檔 `show_detail.rs`）：本機專案蓋章後 `show --json --detail` 帶 `scope`，`show --json` 與 `list --json --detail` 不帶；用法錯誤的 exit code、stderr、stdout。
- `remote_verb_parity.rs`：本機與遠端的 `show --json --detail` 輸出相同。
- Node `host.spec.ts`：`host.dispatch` 與掛鉤 `read` 的 `show --json --detail` 帶 `scope`，不帶 `--json` 時 `invalid_argv`。
- Node `export-local.spec.ts`：沒有規格資料夾時拒絕（message 開頭、`code`）；空的規格資料夾成功；壞名稱仍是既有錯誤。
- Node `host.spec.ts`：從中英兩版 SDK 文件抽出釋放範例的函式宣告當掛鉤執行，涵蓋上面釋放範例的五種情況。
- 文件：中英兩版都有三段新內容；docs/verb-contract 的釋放範例只剩連結；文件裡的範例程式用到的 `@speclink/engine` 名稱都在型別宣告裡（既有守門）。

**範圍**

- 在範圍內：上列旗標、JSON 鍵、`exportScope` 的拒絕、文件。
- 不在範圍內：線上格式與 Server、Desktop、`review show`／`verify show`、`list --detail`、組包函式、匯入規則、認領的引擎規則、CLI 匯入匯出動詞。

## Risks / Trade-offs

- [回歸對照] `show --json` 預設輸出被意外改動 → 輸出測試以「不帶 `--detail` 時與既有 JSON 逐位元相同」釘住；既有的 golden 與 CLI 測試不改期望值，若需要改就是回歸。
- [回歸對照] `list --json --detail` 被新序列化波及 → 新方法另立，不改既有序列化；`list --detail` 的既有測試照跑。
- [三端不一致] 遠端路徑的 `scope` 與本機排序不同 → 兩端都照 metadata 記錄的順序（Server 讀同一份 metadata）；parity 測試以深度相等比對。
- [舊版 Server] 回應沒有關卡欄位 → 對應的鍵不出現、指令不失敗，與 `list --detail` 相同。
- [跨平台] `scope` 的 `path` 在蓋章時已正規化成正斜線，輸出照原樣，不在輸出時再轉換；`exportScope` 的 message 只用資料夾名稱，不印絕對路徑，Windows 上不會出現反斜線差異。檢查規格資料夾只看資料夾與檔案是否存在，不讀內容。
- [指紋文件與實作不一致] 文件的 JS 範例若剝掉 BOM 會算出不同的值 → 範例明寫 `ignoreBOM: true`，並以測試資料（含 BOM 的 UTF-8 檔、CRLF 檔、非 UTF-8 檔）對照引擎的指紋確認範例正確。
- [行為變更] 依賴「沒有規格資料夾回傳零份」的宿主會改成收到錯誤 → `exportScope` 尚未發版，沒有已發布的使用者；文件寫明要匯出空資料夾時先建立規格資料夾。

## Migration Plan

不需要資料遷移：沒有新的檔案格式或設定欄位。發版時隨下一版的 CLI 與 `@speclink/engine` 一起發。回退：還原這個變更的 commit 即可，`show --json` 預設輸出與 metadata 都沒有變。

## Open Questions

(none)

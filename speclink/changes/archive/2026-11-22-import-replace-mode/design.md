## Context

匯入的實作在 `speclink-host` 的 `transfer` 模組，只用儲存接口的三個操作（列出、讀取、提交）。流程是：先檢查匯入包（格式版本、每份文件的鍵與摘要、重複的鍵、看板順序的大小），再以空前綴列出目標範圍；範圍有任何文件就以「範圍不是空的」拒絕；通過後用一次提交寫入全部文件，每一筆預期「必須不存在」，並帶一筆 `scope-imported` 事件。

呼叫這段實作的入口有四個：

| 入口 | 位置 | 這次的處置 |
| --- | --- | --- |
| 宿主套件的 `HostKit::import_scope` | `speclink-host` 的 `kit` | 多一個選項參數 |
| Node SDK 的 `host.importScope` | `speclink-node` | 多 `mode` 與 `dryRun`，回報多三個清單 |
| HTTP 入口的 `POST /import` | `speclink-host` 的 `http/routes` | 維持建立模式 |
| 示範用 Server 的備份還原 | `speclink-server` 的 `admin/backup`，直接呼叫 `transfer::import` | 不改 |

前兩輪討論的決定：`embedded-host-needs` 定了「匯入維持只能進空的範圍」；`embedded-host-needs-3` 否決覆寫非空的範圍，理由是（一）違反 workspace-migration 的「只接受 CreateNew」，（二）「四支儲存實作都要改」。這次推翻的理由：

- 理由（二）已經不成立：匯入只用儲存接口的三個操作，實作只有 `transfer` 一處，任何實作儲存接口的儲存都不用改。
- 理由（一）管的是 HTTP 的匯入端點。取代模式只在程序內的呼叫提供，端點不動，所以不違反。
- 新的證據：管理者實際需要再匯入一次；替代做法（換代號，或宿主自己清空儲存）成本高，而且清空與匯入之間有空窗。

限制：流程政策（`locale`、`tdd`、`audit`、`worktree`…）與專案說明、產出規則、審查規則都在同一份文件 `config.yaml`。儲存接口的提交只能對個別的鍵下預期，沒有「整個範圍沒有變」的預期。

## Goals / Non-Goals

**Goals:**

- 宿主的程式能用一次呼叫、一次提交，讓一個已有文件的範圍等於一份匯入包。
- 宿主能先試跑，把會新增、更新、刪除的鍵給管理者看，再決定要不要執行。
- 既有的呼叫（不帶新欄位）與既有的三個入口行為不變。

**Non-Goals:**

- HTTP 入口的取代模式：workspace-migration 規定線路上不暴露覆寫，原版用戶端沒有這個需求。
- 合併：保留匯入包沒有的變更與討論。repo 已封存的變更在範圍裡會同時留著進行中的那一份，repo 的刪除也永遠不會生效。
- 保留範圍裡的 `config.yaml`，或逐鍵合併流程設定：前者讓 repo 裡改過的專案說明與產出規則永遠進不來，後者要引擎定義哪些鍵算政策。
- 持續同步 repo 的規格資料夾：取代模式是宿主主動再匯入一次，不是同步。
- 清空範圍的功能：空的匯入包加取代模式會被拒絕。
- CLI 的匯入與匯出動詞、示範用 Server 管理頁的取代操作：維持列在已知缺口。
- 匯入的進度回呼：cut 1 的實測決定不做。

## Decisions

### D1 取代模式是既有匯入函式的一個選項，不是新的入口

`transfer` 模組加一個選項型別：模式（建立或取代，預設建立）與是否試跑（預設否）。內部的匯入函式收這個選項；`HostKit::import_scope` 多一個選項參數並原樣傳下去。

對外的 `transfer::import` 維持現在的簽名，等於「建立模式、不試跑」：示範用 Server 的備份還原直接呼叫它，不用改。

唯一的實作落點是 `speclink-host` 的 `transfer`：這是應用層的裁決（要不要寫、寫哪些鍵），不是領域演算法，也不碰儲存媒介。Node 綁定與 HTTP 入口只組裝輸入與呈現結果。

**替代方案：新增一個 `replaceScope` 方法與命令名稱 `scope-replace`。** 不採用。檢查匯入包、大小上限、掛鉤、事件、回報的形狀全部相同，分成兩個入口就要維持兩套；`commandNames` 也會多一個名稱，宿主的權限表要跟著改。討論的結論是命令名稱不變、用參數區分。

**替代方案：宿主自己清空再匯入，SDK 只提供清空的函式。** 不採用。兩步之間有空窗，而且「清空」本身是比取代更危險的單獨操作。

### D2 取代的比對與單次提交

取代模式的步驟，依序：

1. 檢查匯入包：和建立模式完全相同的檢查，在呼叫儲存之前完成。
2. 匯入包沒有任何文件時拒絕（見 D3），儲存沒有被呼叫。
3. 以空前綴列出範圍，得到每個鍵目前的版本。
4. 對兩邊都有的鍵，分批讀出範圍裡的內容（沿用匯出的批次大小），與匯入包的內容比對。
5. 分類並組出這次提交的操作：
   - 只在匯入包：寫入，預期「必須不存在」→ `added`。
   - 兩邊都有、內容不同：寫入，預期「必須是列出時的版本」→ `changed`。
   - 兩邊都有、內容相同：沒有操作。
   - 只在範圍：刪除，預期「必須是列出時的版本」→ `removed`。
6. 沒有任何操作時不提交：回報三個空的清單與目前的範圍版本，沒有事件，不通知提交後掛鉤。
7. 試跑時到這裡為止：回報三個清單與目前的範圍版本。
8. 用正好一次提交送出全部操作，帶一筆 `scope-imported` 事件，payload 是 `documents`（寫入的份數）與 `removed`（刪除的份數）。

寫入與刪除在同一次提交，所以任一步失敗後範圍都是原樣：儲存契約保證提交全部成功或全部不做。沒有半套狀態。

內容相同的文件不重寫，是為了讓它們的版本與修改時間不變：連線中的用戶端不用重新讀取，宿主用修改時間做的統計也不會被洗掉。

**並行寫入的邊界。** 儲存接口的提交只能對個別的鍵下預期：

- 另一方在列出之後改了這次會寫入或刪除的鍵：提交時預期不成立，整筆拒絕（見 D3）。
- 另一方在第 4 步之前刪掉了一個列出時存在、兩邊都有的鍵：讀取得到「不存在」，同樣以「範圍變了」拒絕，不重試。
- 另一方改了一個內容相同、所以沒有操作的鍵：這次匯入不會察覺，那個鍵留著另一方的新內容。
- 另一方新增了一個列出時不存在、匯入包也沒有的鍵：這次匯入不會察覺，那份文件原樣保留。這和建立模式既有的邊界相同。

後兩種情況下，取代之後範圍不完全等於匯入包；再試跑一次就會看到差異。文件要寫明：要保證完全相等，宿主在取代期間要自己擋住其他寫入（例如在 `beforeCommand` 拒絕）。

**範圍裡不屬於引擎命名規則的鍵**（宿主自己放進同一個範圍的文件）也會被刪除：規則是「範圍的內容等於匯入包」，試跑的 `removed` 會列出它們。

**建立模式不變**：範圍有任何文件就拒絕，不讀內容；提交時的衝突仍然回報「範圍不是空的」。

**替代方案：內容相同的鍵也重寫。** 不採用。每次取代都讓全部文件的版本前進，用戶端全部重新讀取，一份 1,600 份文件的範圍每次都是整包寫入。

**替代方案：比對摘要而不讀內容。** 做不到：儲存接口的列出只給鍵、版本與修改時間，沒有摘要。

### D3 兩個新的拒絕各有原因代碼

| 情況 | `code` | `reason` | 訊息 |
| --- | --- | --- | --- |
| 取代模式，匯入包沒有任何文件 | `invalid_argv` | `empty_bundle` | `import (replace): the bundle holds no documents — a replace with it would remove every document of the scope` |
| 取代模式，列出之後另一方動了這次會寫入或刪除的鍵 | `refused` | `scope_changed` | `import (replace): the target scope changed during the import — run it again` |

兩者都是新的失敗，所以符合「新的代碼只加在還沒有 `reason` 的失敗上」。`transfer` 的錯誤型別各多一個成員；`TransferReason` 多兩個成員。

空的匯入包在建立模式維持現在的行為：不寫入、回報零份，不是錯誤。

`scope_changed` 用 `refused` 而不是儲存層的 `revision_conflict`：建立模式已經把同一種提交衝突回報成 `refused` 加 `scope_not_empty`；原因代碼只掛在命令層的失敗上，儲存層的失敗沒有 `reason`。宿主只認 `reason`，兩種模式用同一種形狀。

**替代方案：空的匯入包加取代模式就清空範圍。** 不採用。`engine.exportScope()` 對只有空子資料夾的規格資料夾會成功並回傳零份文件；下載放錯位置加上一次取代，就會刪掉整個範圍。

**替代方案：衝突時自動重試。** 不採用。重試要重新比對，結果會和試跑給管理者看的不同；由宿主決定要不要再跑。

### D4 回報多三個鍵的清單

匯入的回報多 `added`、`changed`、`removed` 三個鍵的陣列，各自依鍵的位元組順序排列。`documents` 的意思不變：寫入的份數，等於 `added` 加 `changed` 的長度。`scopeVersion` 在有提交時是提交後的版本；沒有提交（試跑、沒有任何操作、建立模式的空匯入包）時是列出時的版本。

建立模式也給三個清單：`added` 是匯入包全部的鍵，另外兩個是空的。這樣宿主的型別只有一種形狀。

給鍵而不只給數字：管理者看不到 repo，要知道會刪掉哪幾個變更與討論才能決定。鍵的對照表是契約，宿主用前綴就能分類。清單只有鍵，沒有內容；一份 1,600 份文件的範圍約 100 KB。

**替代方案：只給三個數字。** 不採用。宿主要顯示名稱時得自己再匯出一次來比對。

### D5 掛鉤收到模式與試跑，試跑也先問

`scope-import` 的參數從 `documents`、`bytes` 變成 `documents`、`bytes`、`mode`（`create` 或 `replace`）、`dryRun`（布林）。四個鍵每次都給，經 HTTP 入口的匯入也給（`create`、`false`）。參數仍然沒有文件內容。

順序不變：大小上限 → 執行前掛鉤 → 匯入（檢查匯入包、列出、比對、提交）。試跑會讀範圍的內容並回報鍵，所以和真的執行一樣先問掛鉤；宿主可以用 `params.dryRun` 放寬試跑的權限。提交後掛鉤只在真的有提交時被通知。

**替代方案：試跑不問掛鉤。** 不採用。試跑的回報會透露範圍裡有哪些鍵（變更與討論的名稱）。

### D6 Node 綁定嚴格檢查新欄位

`host.importScope` 的呼叫多兩個選填欄位。JS 端在跨到原生之前檢查，不合格時 Promise 以 `code` 為 `invalid_argv` 的 Error 拒絕，儲存與掛鉤都沒有被呼叫：

- `mode` 不給時是 `'create'`；給了但不是 `'create'` 或 `'replace'` 時拒絕，訊息指出收到的是什麼：字串顯示原文（過長時截斷），其他型別的值只顯示型別。錯誤訊息不隨呼叫端給的值變長，也不帶出物件的內容。
- `dryRun` 不給時是 `false`；給了但不是布林時拒絕。字串 `'false'` 不會被當成真的或假的。呼叫的物件有 `dryRun` 鍵而值是 `undefined` 也算給了：`dryRun: body.dryRun` 而對方送的是 `dry_run` 時，不能變成真的取代。
- `mode` 是 `'replace'` 時，呼叫的物件只能有 `identity`、`scope`、`bundle`、`mode`、`dryRun` 五個鍵；有其他的鍵時拒絕並指出它的名稱。理由：把 `dryRun` 拼成 `dryrun` 或 `dry_run` 會讓一次本來要試跑的呼叫真的刪除文件。建立模式不加這條檢查，既有的呼叫不受影響。

回報在 JS 端是 `{ scopeVersion, documents, added, changed, removed }`。

### D7 HTTP 入口、示範用 Server 與 Speclink Desktop 維持建立模式

`http/routes` 的匯入路由以預設選項呼叫宿主套件（建立模式、不試跑），請求與回應的形狀都不變。請求本文裡多出來的 `mode` 或 `dryRun` 照舊在解析時以 422 拒絕：包的線路型別不收未知的欄位（`speclink-protocol`，不在這次的範圍），這時儲存與掛鉤都沒有被呼叫。送錯入口的用戶端會得到明確的錯誤，不會靜默走建立模式。示範用 Server 的備份還原、Speclink Desktop 的「遷移到 Server」都不改程式。

### D8 SDK 文件與狀態文件

中英兩版同步改，結構對等：

- **SDK 文件「匯出與匯入」**：範例加上取代與試跑；說明兩種模式、三個清單、不重寫內容相同的文件、D2 的並行邊界、D6 的檢查；匯入的原因代碼表從三列變五列。
- **SDK 文件「在伺服端匯入」**：「要重新匯入」那一點改寫成取代模式，寫明它會刪掉匯入之後在宿主裡做的全部改動（變更、討論、蓋章），以及流程設定會變成 repo 的版本；附一段範例：取代前用 `workflow-config show --json` 讀出目前的政策，先試跑，把 `removed` 給管理者確認，再取代，最後用 `workflow-config set` 把宿主管理的政策鍵寫回去。「先調政策、再匯入」用同一段範例。這段範例的呼叫順序要有對應的自動測試。
- **SDK 文件的掛鉤段落**：`scope-import` 的參數多 `mode` 與 `dryRun`，並給一個「只讓管理者取代」的判斷例子。
- **動詞參考文件**的匯入一節：寫明取代模式只在程序內的呼叫提供，`POST /import` 只收空的範圍。
- **產品狀態文件**：Node SDK 那一列提到取代模式；已知缺口不變。
- **型別說明**（`index.d.ts`）：`HostImportCall`、`HostImportReport` 與 `importScope` 的說明跟著改。

## Implementation Contract

**行為：**

- `host.importScope({ identity, scope, bundle })`（不帶新欄位）：寫入的結果與失敗的 `code`、`reason`、`message` 與現在逐字相同；回報多 `added`（全部的鍵）、`changed`（空）、`removed`（空）。
- `host.importScope({ …, mode: 'replace' })`：一次提交之後，範圍以空前綴列出的鍵等於匯入包的鍵，每個鍵的內容等於匯入包的內容（D2 寫明的兩種察覺不到的並行寫入除外）；內容相同的文件的版本與修改時間不變。
- `host.importScope({ …, dryRun: true })`：儲存的提交函式沒有被呼叫，提交後掛鉤沒有被呼叫；回報與緊接著真的執行所得到的三個清單相同。
- 執行前掛鉤對每一次呼叫正好被問一次，參數是 `{ documents, bytes, mode, dryRun }`；拒絕時儲存沒有收到任何呼叫。
- HTTP 入口的 `POST /import` 一律是建立模式：對非空的範圍照舊拒絕；請求本文帶 `mode` 或 `dryRun` 時照舊以 422 拒絕，儲存與掛鉤都沒有被呼叫。

**資料的形狀：**

```
HostImportCall {
  identity, scope, bundle        // 不變
  mode?: 'create' | 'replace'    // 預設 'create'
  dryRun?: boolean               // 預設 false
}
HostImportReport {
  scopeVersion: number           // 有提交時是提交後的版本，否則是列出時的版本
  documents: number              // 寫入的份數 = added.length + changed.length
  added: string[]                // 鍵，依位元組順序
  changed: string[]
  removed: string[]
}
beforeCommand 的 params: { documents: number, bytes: number, mode: 'create' | 'replace', dryRun: boolean }
scope-imported 的 payload: 建立模式 { documents }；取代模式 { documents, removed }
```

**失敗：**

| 情況 | `code` | `reason` |
| --- | --- | --- |
| 取代模式，匯入包沒有文件 | `invalid_argv` | `empty_bundle` |
| 取代模式，範圍在匯入途中被改動 | `refused` | `scope_changed` |
| `mode` 不是兩個值之一、`dryRun` 不是布林、取代模式的呼叫有未知的鍵 | `invalid_argv` | 沒有 |
| 建立模式，範圍不是空的 | `refused` | `scope_not_empty`（不變） |
| 匯入包不合格、超過上限、掛鉤拒絕、儲存失敗 | 不變 | 不變 |

每一種失敗之後範圍都是原樣。

**例子：** 範圍有 `config.yaml`（含 `tdd: true`）、`specs/auth/spec.md`（舊內容）、`changes/add-x/proposal.md`、`discussions/d1.md`、`LANGUAGE.md`（與匯入包相同）；匯入包有 `config.yaml`（沒有 `tdd`）、`specs/auth/spec.md`（新內容）、`specs/billing/spec.md`、`LANGUAGE.md`。

- 試跑的回報：`added` 是 `['specs/billing/spec.md']`，`changed` 是 `['config.yaml', 'specs/auth/spec.md']`，`removed` 是 `['changes/add-x/proposal.md', 'discussions/d1.md']`，`documents` 是 3；範圍不變。
- 真的執行：儲存收到一次提交，含 3 筆寫入與 2 筆刪除；之後範圍正好是匯入包的四份文件，`LANGUAGE.md` 的版本與修改時間不變；`scope-imported` 的 payload 是 `{ documents: 3, removed: 2 }`。
- 緊接著再執行一次同樣的取代：沒有提交，三個清單都是空的，`scopeVersion` 不變。

**驗收：**

- `speclink-host` 的整合測試（`tests/transfer.rs`）涵蓋上面的例子、空的匯入包、兩種會被察覺的並行寫入、兩種察覺不到的並行寫入、試跑不提交、掛鉤的參數與拒絕、建立模式的回報與既有行為不變。
- `speclink-host` 的 HTTP 入口測試（`tests/http_door.rs`）確認 `POST /import` 帶 `mode: "replace"` 或 `dryRun` 時照舊以 422 拒絕、儲存與掛鉤都沒有被呼叫、既有的文件不變；不帶這些欄位的匯入，掛鉤收到 `mode` 是 `create`、`dryRun` 是 `false`。
- `crates/adapters/speclink-node/__test__/transfer.spec.ts` 從 JS 端涵蓋取代、試跑、三個清單、兩個新的原因代碼、D6 的三種檢查，以及文件裡「重新套用政策」範例的呼叫順序。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過。

**範圍：**

- 在範圍內：`transfer` 的匯入、`HostKit::import_scope`、HTTP 路由的呼叫點（只改成傳預設選項）、Node 綁定、型別說明、四份文件。
- 不在範圍內：線路的型別（`speclink-protocol`）、`speclink-remote` 的用戶端、`speclink-server`、`apps/desktop`、儲存契約與它的合格測試、CLI。

## Risks / Trade-offs

- [取代會刪掉匯入後在宿主裡做的全部改動] → 預設是建立模式；取代要明確傳 `mode: 'replace'`；空的匯入包被拒絕；D6 擋住拼錯的欄位；試跑先列出會刪除的鍵；`beforeCommand` 看得到模式；文件用粗體寫明後果。
- [既有測試釘住掛鉤參數的整個物件] → `speclink-host` 的 `tests/transfer.rs`、`tests/http_door.rs` 與 Node 的 `transfer.spec.ts` 各有一處以 `{ documents, bytes }` 做相等比對，同批更新成四個鍵。
- [察覺不到的並行寫入讓範圍不完全等於匯入包] → 這是儲存接口的限制（只能對個別的鍵下預期），不改契約；文件寫明邊界與宿主的做法，再試跑一次可以看到差異。
- [一次提交同時有上千筆寫入與刪除] → 建立模式已經是一次提交寫入全部文件（實測 1,636 份）；取代只會更少，因為內容相同的不寫。讀取分批進行，批次大小沿用匯出的。
- [取代模式比對時要把範圍裡兩邊都有的文件內容讀進記憶體] → 和匯出同一個量級；`limits.importBytes` 仍然限制匯入包的大小。
- [跨平台] → 這次沒有檔案路徑或換行的處理：鍵是儲存的位址，比對是位元組相等。Node 的測試在 Windows CI 也跑。
- [回歸對照] → 沒有 CLI 動詞或 `--json` 輸出改變，golden 不受影響；`crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs` 不涉及匯入。HTTP 入口的匯入由 `tests/http_door.rs` 與 `speclink-server` 的匯入測試守住既有行為。

## Migration Plan

沒有資料要搬。宿主升級 `@speclink/engine` 之後：

1. 既有的呼叫照舊可用。
2. 以整個物件比對 `scope-import` 參數或匯入回報的程式，改成讀個別的欄位。
3. 要提供「重新匯入」時，照 SDK 文件的範例：先試跑、給管理者確認、取代、重新套用政策。

回退：不傳 `mode` 就是原本的行為。

## Open Questions

沒有。

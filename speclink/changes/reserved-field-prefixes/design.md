## Context

自由欄位由引擎核心存放（`speclink-core` 的 `fields` 模組），規則是 free-fields 規格：鍵與值的格式、每份文件最多 32 個鍵、引擎不解讀任何鍵。從請求寫欄位的命令有五種：三個建立命令（`NewChange`、`DiscussNew`、`DiscussPromote`）的 `fields`，以及 `ChangeFieldsEdit`、`DiscussFieldsEdit` 的 `set` 與 `unset`。

宿主附上的欄位在宿主套件處理：`speclink-host` 的 `kit.rs` 裡，`HostKit::run_with` 先呼叫 `attach_fields` 把宿主附上的欄位併入建立命令（同名時宿主的值蓋過），再問 `beforeCommand`。`host.dispatch` 與 `host.handleRequest` 都經過 `run_with`，這是兩個門唯一的合併點。合併之後，掛鉤看到的 `params.fields` 分不出哪些鍵來自宿主、哪些來自請求。

對照程式碼確認的事實：宿主今天能在 `beforeCommand` 用 `params.set`、`params.unset` 擋掉欄位編輯，也能檢查建立時合併後的值；擋不住的是「請求把來源標成另一個呼叫者也有權限的值」，而且宿主要自己記得每一個會寫欄位的命令。

## Goals / Non-Goals

**Goals:**

- 宿主宣告的前綴底下的鍵，只有宿主自己的程式寫得了；Agent、CLI、Speclink Desktop 三個入口一次涵蓋。
- 以後新增會寫欄位的命令時，自動納入檢查。
- 沒有設定這個選項時行為完全不變。

**Non-Goals:**

- 讓宿主在建立之後改保留的鍵的值：宿主附上的欄位只在建立時寫入，目前沒有改值的需求。
- 匯入時檢查保留的鍵：`host.importScope` 與 `POST /import` 寫的是整份文件，只有管理者能做。
- 在引擎核心加入「保留鍵」的概念：free-fields 規定引擎不解讀任何鍵。
- 隱藏保留的鍵：讀取照常輸出。
- 讓 `beforeCommand` 知道哪些欄位是宿主附上的：討論否決了這個做法。

## Decisions

### 檢查放在宿主套件合併欄位的那一步

`HostKitOptions` 多一個欄位 `reserved_field_prefixes: Vec<String>`。`HostKit::run_with` 在呼叫 `attach_fields` 之前，先檢查命令「自己帶的」欄位鍵（合併之前，所以只有來自請求的鍵）：有任何一個鍵以保留的前綴開頭，就以 `ErrorCode::Refused` 結束。因為在合併之前檢查，宿主附上的保留鍵不受影響。這是唯一的實作落點，兩個門、示範用 Server 與任何宿主都經過它。

順序：這個檢查在問 `beforeCommand` 之前、在儲存收到任何呼叫之前。被拒絕時掛鉤不被問，`afterCommit` 不被呼叫。它和「欄位不合規則時以 invalid_argv 失敗」是同一個位置的兩個檢查；兩者同時成立時先回報保留鍵的拒絕。

替代方案：放在引擎核心的 `fields` 模組。否決，因為引擎不應該認得任何鍵。替代方案：放在 `beforeCommand`，由宿主自己寫。否決理由見 Non-Goals 與討論。

### 命令帶了哪些請求欄位鍵，由 speclink-core 的一個方法回答

在 `speclink-core` 的 `command/fields.rs`（已經有 `creation_fields` 與 `creation_fields_mut`）加一個方法，回傳這個命令會寫入或刪除的全部自由欄位鍵：三個建立命令的 `fields` 的鍵、兩個編輯命令的 `set` 的鍵與 `unset` 的鍵，其他命令是空的。宿主套件只呼叫這個方法，不自己列命令。以後新增會寫欄位的命令時，改的是 core 這一處，而這個檔的註解已經寫明「帶欄位的命令只在這裡列出」。

這個方法只回答「有哪些鍵」，不判斷鍵是不是保留的，所以引擎核心仍然不解讀任何鍵。

### 拒絕的錯誤與訊息

錯誤碼是既有的 `refused`（經入口時是 409、reason 為 `refused`），不新增代碼。訊息是固定的英文句式並指出鍵：`field '<key>' is reserved by the host`。同一次呼叫有多個保留的鍵時，指出依位元組順序的第一個。宿主的 `beforeCommand` 拒絕與這個拒絕用同一個代碼；呼叫端用訊息分辨。

### 前綴的規則與比對

前綴 SHALL 符合自由欄位鍵的規則 `^[a-z][a-z0-9_.-]{0,63}$`：不能是空字串，不能含大寫或其他字元。不合規則的前綴建立不出宿主套件（Rust 回錯誤；Node 的 `createHost` 同步拋出）。比對是單純的字串開頭比對：前綴 `host.` 保留 `host.project`；前綴 `host` 會同時保留 `host`、`hostname`、`host.project`，所以文件建議前綴以 `.`、`-` 或 `_` 結尾。重複的前綴視為一個；空陣列等於沒有設定。

### Node 的 createHost 選項

`crates/adapters/speclink-node/index.js` 的 `HOST_OPTIONS` 加 `reservedFieldPrefixes`；JS 這一側檢查它是字串陣列（否則 `TypeError`），再放進交給 native 的入口設定；`src/host.rs` 把它帶進 `HostKitOptions`，由 Rust 檢查每個前綴的格式。`index.d.ts` 的 `CreateHostOptions`、`HostDispatchCall.fields`、`HostRequestExtras.fields` 的說明跟著更新。

## Implementation Contract

**行為**（以 `reservedFieldPrefixes: ['host.']` 為例）

- 請求寫保留的鍵被拒絕：argv `['new', 'change', 'spoof', '--field', 'host.project=p2']`、`['discuss', 'new', '主題', '--field', 'host.project=p2']`、`['discuss', 'promote', 'talk', '--field', 'host.project=p2']`、`['change', 'field', 'set', 'ok', 'host.project=p2']`、`['discuss', 'field', 'set', 'talk', 'host.project=p2']`。
- 請求刪保留的鍵被拒絕：`['change', 'field', 'unset', 'ok', 'host.project']`、`['discuss', 'field', 'unset', 'talk', 'host.project']`。
- 入口同樣拒絕：建立變更、建立討論、轉出變更的請求本文的 `fields` 含保留的鍵；`PUT /changes/{name}/fields` 與 `PUT /discussions/{slug}/fields` 的本文含保留的鍵。回應是 409、reason 為 `refused`。
- 宿主附上的保留鍵照常寫入：argv `['new', 'change', 'ok']` 加上 `fields: { 'host.project': 'p1' }` 成功，變更有 `host.project=p1`。
- 請求與宿主都帶同一個保留的鍵：拒絕，沒有寫入。
- 非保留的鍵不受影響：`--field team=web` 成功；請求帶 `team=cli`、宿主附上 `team=web` 時結果是 `web`。
- 被拒絕時：儲存沒有收到任何呼叫，`beforeCommand` 沒有被問，`afterCommit` 沒有被呼叫，變更與討論不變。
- 讀取照常輸出保留的鍵與它的值。

**介面與資料形狀**

- Rust：`HostKitOptions.reserved_field_prefixes`；core 的 `Command` 多一個回傳請求欄位鍵的方法。
- Node：`CreateHostOptions.reservedFieldPrefixes?: string[]`。
- 錯誤：`code` 是 `refused`，`message` 是 `field '<key>' is reserved by the host`。

**失敗模式**

- 前綴不是字串陣列、或某個前綴不合規則：`createHost` 同步拋出，不建立宿主套件。
- 沒有設定或空陣列：沒有任何檢查，行為與現在相同。

**驗收條件**

- `crates/host/speclink-host/tests/free_fields.rs`：上面每一種行為各有測試，並斷言被拒絕時儲存的呼叫次數是 0、掛鉤的呼叫次數是 0。
- `crates/host/speclink-host/tests/http_door.rs`：入口的五條寫入路徑在保留鍵上回 409。
- `crates/adapters/speclink-node/__test__/free-fields.spec.ts`：`createHost` 的選項驗證、`dispatch` 與 `handleRequest` 的拒絕與放行。
- 沒有設定選項時，`speclink-host` 與 `speclink-server` 既有的測試全部不改就通過。

**範圍界線**

- 在範圍內：core 的那一個方法、宿主套件的選項與檢查、Node 的選項與型別、`docs/sdk-node` 的中英文說明。
- 不在範圍內：`speclink-core` 的 `fields` 模組的規則、CLI 的指令與輸出、線上格式、Speclink Desktop、匯入匯出、示範用 Server 的設定。

## Risks / Trade-offs

- [前綴太寬，連帶保留了沒有預期的鍵（`host` 保留 `hostname`）] → 文件建議以分隔字元結尾；比對規則寫進規格與型別說明。
- [宿主之後想改保留鍵的值卻沒有辦法] → 提案列為已知缺口；出現需求時另開變更（例如讓宿主附上的欄位也能在編輯時生效）。
- [匯入可以帶進保留的鍵] → 提案列為已知缺口；匯入只有管理者能做，寫的是整個範圍。
- [新增會寫欄位的命令時忘了納入檢查] → 命令帶哪些欄位鍵集中在 core 的一個方法；任務裡加一個測試，走訪全部命令種類，確認每一個帶 `fields`、`set` 或 `unset` 參數的命令都被這個方法涵蓋。
- [`refused` 同時代表掛鉤的拒絕與保留鍵的拒絕] → 訊息是固定句式，文件寫明；不新增錯誤碼，避免擴大錯誤碼的清單。
- [回歸對照] → 沒有設定選項時不進任何新分支；既有的 golden、CLI 測試與示範用 Server 的整合測試不改就要通過。
- [跨平台] → 只有字串比對，沒有路徑、檔案系統或 git 的呼叫；Windows、macOS、Linux 的行為相同。

## Context

討論 embedded-host-needs 定案：規格由一台 Server 持有時，儲存媒介、連線、遷移與歷史保存都由宿主負責；speclink 只提供一份小的儲存契約與合格測試。這是十六個變更的第一個，只新增契約本身，不接線。

現況有兩套儲存介面，都不適合由宿主實作：

- **引擎內層的儲存介面**（`speclink-core` 的 `Store` trait）：約四十個方法，每個方法讀寫一份文件。沒有版本比對，也沒有交易邊界。
- **TeamStore 契約**（`speclink-store`）：十二個方法。它做對三件事——一個動詞一次寫入、版本比對、事件與寫入同一次提交。但它假設 speclink 自己持有連線與資料表（`migrate`、`health`、outbox 的讀取與確認），快照只能逐份讀取、不能列出，文件種類是封閉的九種（`DocumentId`），實作只能用 Rust 寫。

引擎在示範用的 server 上已經是「先讀、跑流程、交出寫入清單與事件、一次提交」（`speclink-host` 的橋接層）。所以儲存真正需要的動作只有三個：列出、讀取、提交。

限制：

- 引擎核心維持同步，不引入 async runtime。
- 之後的變更會讓 JavaScript 寫的儲存經 N-API 實作同一份契約，所以契約裡的每個值都要能用 JavaScript 的值無損表示。
- 這個變更完成後，示範用的 server 與既有測試都要照常運作。

## Goals / Non-Goals

**Goals:**

- 一份由宿主實作的儲存契約，只有列出、讀取、提交三個操作。
- 契約對儲存媒介中立：宿主能在自己的資料庫交易裡完成提交。
- 一套合格測試，任何實作（Rust 或之後的 JS）都跑同一組檢查，並能抓出常見的錯誤實作。
- 一份記憶體版的參考實作，供合格測試自我驗證，也供之後的變更寫測試。
- 朝「storage 解耦的規格驅動引擎」靠攏：文件改用字串鍵，新增文件種類時儲存的實作不用改。

**Non-Goals:**

- 不讓引擎經這份契約讀寫。那是變更 engine-on-store-port 的範圍。
- 不提供 Node 的綁定，也不把合格測試放進 npm。那是變更 host-kit-agent-door 的範圍。
- 不改 `speclink-core` 的儲存介面、`speclink-store`、三個官方 driver、`speclink-host` 的橋接層、線上協定。
- 不做檔案系統版的實作。本機專案繼續用 `speclink-fs`。
- 不提供讀回事件的操作。推送事件預設關閉，用戶端靠範圍版本號輪詢。
- 不做分頁、不做歷史查詢、不做匯入匯出。匯入匯出由變更 import-export-on-port 建在這三個操作上。
- 不為契約的型別加上序列化（serde）。JSON 的欄位名稱在 Node 綁定的變更裡定。
- 不提供「不檢查版本」的寫入。每一筆寫入都要說明預期。

## Decisions

### D1: 契約放在新的 crate speclink-store-port

新增 `crates/store/speclink-store-port`，套件名稱 `speclink-store-port`。它屬於 store crates，依賴只有 `serde_json`（事件的內容）。它不依賴 `speclink-core`，也不依賴 `speclink-store`。

- **為什麼不放進 `speclink-store`**：`speclink-store` 會在最後的收尾變更整個移除。新契約放在獨立的 crate，移除時不用拆檔。
- **為什麼不放進 `speclink-core`**：之後宿主的 JS 儲存橋接、示範用的範例實作都只需要契約，不需要引擎。契約不依賴引擎，依賴方向才不會倒過來。

### D2: StorePort 是同步、object-safe、可跨執行緒共用的 trait

`StorePort` 要求 `Send + Sync`，三個方法都收 `&self`：

- `list(scope, prefix)` 回傳 `Listing`
- `read(scope, keys)` 回傳與 `keys` 等長、順序相同的清單，每一項是文件或「沒有」
- `commit(scope, commit)` 回傳新的範圍版本號

同步的理由和 TeamStore 相同：引擎不帶 async runtime，非同步的宿主在自己的邊界轉接。要求 `Send + Sync` 的理由：一個儲存物件由全部請求共用（定案的每次呼叫才給身分），合格測試的併發檢查也要從多個執行緒呼叫同一個物件。

- **否決：每個請求建立一個儲存物件** — 連線會跟著物件走，宿主的連線池用不上。

### D3: 範圍用不透明的字串識別

`ScopeId` 是 1 到 256 個位元組、不含控制字元的字串。契約不解讀它的內容。線上的專案與 repo 怎麼對到一個範圍，由宿主套件的變更決定。

沒有「建立範圍」的操作。對一個從沒寫過的範圍：列出回傳版本號 0 與空清單，讀取每一項都是「沒有」，第一次提交直接成功。範圍的建立、刪除與權限都是宿主的事。

- **否決：沿用 TeamStore 的 Project 加 Repo 兩段識別** — 那是示範用 server 的帳號模型，宿主的「專案」概念不一定是兩段。

### D4: 文件鍵 DocKey 的格式在建構時檢查

`DocKey` 只能經建構函式取得。規則：

- 1 到 512 個位元組的 UTF-8。
- 不含控制字元（U+0000 到 U+001F、U+007F），不含反斜線。
- 用 `/` 分段。不以 `/` 開頭或結尾，沒有空的段，沒有內容正好是 `.` 或 `..` 的段。

`KeyPrefix` 是 0 到 512 個位元組、不含控制字元與反斜線的字串。前綴比對是位元組層級的「開頭相符」，不認得分段：前綴 `changes/add` 同時符合 `changes/add-x/proposal.md` 與 `changes/addendum/tasks.md`。要限定一層的呼叫端自己在前綴結尾加 `/`。空的前綴符合全部的鍵。

格式在型別的建構函式檢查，儲存的實作收到的鍵一定合法，宿主不用自己驗證。分段規則讓用檔案或物件儲存的實作能把鍵直接對到路徑。

- **否決：沿用封閉的文件種類清單** — 每新增一種文件，全部宿主的儲存都要跟著改。

### D5: 版本號是 0 到 2^53−1 的整數

`Version` 包一個 `u64`，建構時檢查不超過 9007199254740991（JavaScript 能精確表示的最大整數）。範圍版本號與文件版本用同一個型別。時間 `Timestamp` 是 Unix epoch 起算的毫秒數，同樣的上限。

- **範圍版本號**：從沒寫過的範圍是 0。每次成功的提交之後，新的版本號嚴格大於之前的值。不要求每次加 1，宿主能用資料庫的序列。失敗的提交不改變它。
- **文件版本**：等於最後一次寫入這份文件的那次提交所回傳的範圍版本號。

文件版本直接用範圍版本號有三個好處：宿主只要存一個整數欄位；刪除後重建的文件一定拿到更大的版本，舊的預期版本不會誤中；合格測試能精確斷言。

- **否決：文件版本用不透明的字串（例如內容雜湊）** — 線上協定的版本欄位是整數，宿主套件要另做對照；內容雜湊在「改成 B 又改回 A」時會讓過期的寫入通過。

### D6: 提交 Commit 一次帶全部的變動

`Commit` 只能經建構函式取得，欄位：

- `actor`：操作者的識別（不會變的 id）與顯示名稱。
- `command`：引擎命令的名稱，非空字串。
- `at`：這次提交的時間，由呼叫端給。
- `ops`：`Put { key, content, expected }` 或 `Delete { key, expected }` 的清單。`Put` 的預期是 `Expected::Absent`（必須不存在）或 `Expected::At(version)`；`Delete` 的預期一定是某個版本。
- `events`：`Event { name, payload }` 的清單，`payload` 是 JSON 值。

建構函式拒絕兩種內容：同一個鍵出現在兩筆 `ops`；`ops` 與 `events` 都是空的。只有事件、沒有寫入的提交是合法的，它一樣讓範圍版本號前進。

儲存的實作 MUST 做到：

1. 先確認每一筆 `ops` 的預期都相符，再套用全部的變動。任何一筆不符就回傳 `PortError::Conflict`，不改任何文件、不改範圍版本號、不保存任何事件。
2. 被寫入的文件，版本設為這次提交的新範圍版本號，最後修改時間設為 `at`。沒被碰到的文件，版本與最後修改時間不變。
3. 讀取與列出只看得到完整提交之後的狀態，看不到提交到一半的內容。

時間由呼叫端給，不由儲存自己取：合格測試才能精確斷言最後修改時間，宿主也不用處理資料庫與應用程式的時鐘差。`actor` 與 `command` 交給儲存，是讓宿主自己決定要不要記下「誰用哪個命令改的」。契約不要求保存它們。

- **否決：提供不檢查版本的寫入** — 目前沒有需求。引擎改經契約讀寫時若證明需要，由那個變更加上。

### D7: 儲存回傳的錯誤 PortError 只有三類

- `Conflict { key, expected, actual }`：預期不符。`actual` 是文件目前的版本，文件不存在時是「沒有」。多筆不符時指出其中任一筆。
- `Unavailable { message }`：暫時性的失敗，例如連不上資料庫或逾時。呼叫端能重試。
- `Backend { message }`：其他失敗。

值的格式錯誤（鍵太長、版本超出範圍、提交裡有重複的鍵）是另一個型別 `InvalidValue`，由建構函式回傳，儲存的實作不會遇到。權限不在這一層：宿主在入口與掛鉤就擋掉了。

### D8: 事件只交給儲存，契約不讀回

事件在提交裡交給儲存。儲存要不要保存由宿主決定。有保存的實作 MUST 把事件和文件的變動放在同一次交易：提交成功才有事件，提交失敗沒有事件，順序和提交裡的順序相同。

契約沒有讀回事件的操作，所以合格測試經測試用的轉接（D11 的 `PortHarness`）觀察事件。不保存事件的實作在轉接裡回報「不保存」，事件的檢查在報告裡標為略過。

- **否決：在契約裡加上讀取與確認事件的操作** — 那是 TeamStore 的 outbox，宿主已經有自己的事件表與傳遞機制。

### D9: 鍵的命名規則 keys 模組

`keys` 模組提供建立鍵的純函式與前綴常數。它不做 I/O。

| 文件 | 鍵 |
|---|---|
| 進行中變更的 metadata | `changes/<name>/.meta.yaml` |
| 進行中變更的 artifact | `changes/<name>/<artifact>`（例：`changes/add-x/specs/auth/spec.md`） |
| 進行中變更的 evidence | `changes/<name>/.evidence.json` |
| 已封存變更的文件 | `archive/changes/<dated-name>/<doc>` |
| 正式規格 | `specs/<capability>/spec.md` |
| 討論 | `discussions/<slug>.md` |
| 已封存的討論 | `archive/discussions/<created>-<slug>.md` |
| 流程設定 | `config.yaml` |
| 詞彙表 | `LANGUAGE.md` |
| 看板順序 | `board/order` |

鍵和本機專案 `speclink/` 底下的相對路徑相同，只有兩處例外：看板順序在本機沒有對應的檔案；封存的位置不同。本機放在 `changes/archive/` 與 `discussions/archive/` 底下，鍵改放在最上層的 `archive/`。理由是列出 `changes/` 時不能連已封存的文件一起回傳——參考量是 165 個已封存變更佔資料的 84%。

每個建立函式檢查名稱的段：非空、不含 `/`（artifact 與封存文件的路徑例外，它們能有多段）。不合法時回傳 `InvalidValue`。

另外三條檢查，都比規格的「非空、不含 `/`」更嚴，不改變規格的要求：

- 變更名稱不能是 `archive`。本機專案把已封存的變更放在 `changes/archive/`，變更叫這個名字，接成本機路徑時會撞到封存資料夾。
- 已封存變更的名稱必須是 `YYYY-MM-DD-<變更名稱>`，已封存討論的建立日必須是 `YYYY-MM-DD`。這兩個函式的兩個參數都是字串，對調時編譯器擋不到，日期格式讓對調的呼叫被拒絕，而不是靜默產生錯的鍵。只檢查數字與連字號的位置，不檢查月、日的範圍。本 repo 現有的 301 個封存變更與 171 份封存討論全部符合這個格式。

這個變更只提供建立鍵的函式與前綴常數，不提供把鍵解析回文件種類的函式。那個方向等引擎改經契約讀寫時再加。

### D10: 記憶體版參考實作 MemoryStorePort

`MemoryStorePort` 用一把互斥鎖保護「範圍 → 狀態」的對照。每個範圍的狀態是版本號、依鍵排序的文件、已保存的事件。提交時在同一把鎖裡先檢查全部的預期，再套用變動、把版本號加 1、附加事件。

它另外提供 `recorded_events(scope)`，回傳該範圍已保存的事件（名稱、內容、所屬的範圍版本號）。這個方法不屬於契約，只供合格測試的轉接與之後的變更寫測試用。

### D11: 合格測試 conformance 的入口與檢查清單

入口是 `conformance::run(harness)`，回傳 `ConformanceReport`。`PortHarness` 是測試用的轉接，由實作儲存的人提供：

- `store()`：回傳受測的儲存。
- `new_scope()`：回傳一個空的、能直接使用的範圍。由轉接提供，是因為宿主的範圍通常要先在自己的資料表建立一筆資料。
- `recorded_events(scope)`：回傳該範圍已保存的事件；不保存事件的實作回傳「不保存」。

報告列出每個檢查的名稱與結果（通過、失敗並附說明、略過並附理由）。任何一個失敗，整份報告就是不通過。檢查的名稱是穩定的識別：

| 檢查 | 斷言 |
|---|---|
| `empty-scope` | 新的範圍：列出回傳版本號 0 與空清單，讀取回傳「沒有」 |
| `put-then-read` | 提交兩份新文件後，讀取與列出都回傳內容、等於範圍版本號的文件版本、等於 `at` 的最後修改時間 |
| `list-prefix-order` | 前綴是位元組層級的開頭相符；結果依鍵的位元組順序遞增；空的前綴列出全部 |
| `read-alignment` | 讀取的結果與要求的鍵等長、順序相同；不存在的鍵與重複的鍵都照位置回傳；空的要求回傳空清單 |
| `update-advances-version` | 以正確的預期版本改寫後，範圍版本號變大，該文件拿到新版本；沒被碰到的文件版本與時間不變 |
| `delete-then-recreate` | 刪除後讀取與列出都看不到；重建的文件版本大於刪除前的版本 |
| `conflict-stale-version` | 用過期的版本改寫或刪除，回傳衝突並指出鍵、預期、實際 |
| `conflict-absent-but-exists` | 對已存在的文件用「必須不存在」，回傳衝突 |
| `conflict-missing-document` | 對不存在的文件用某個版本改寫或刪除，回傳衝突，實際是「沒有」 |
| `conflict-rejects-whole-commit` | 三筆變動裡最後一筆衝突：前兩筆沒有寫入，範圍版本號不變，事件沒有保存 |
| `events-with-commit` | 成功的提交保存它的全部事件，順序相同，並標上這次的範圍版本號（不保存事件時略過） |
| `events-only-commit` | 只有事件的提交成功，範圍版本號變大，文件不變 |
| `scope-isolation` | 一個範圍的寫入在另一個範圍看不到；兩個範圍的版本號各自前進 |
| `concurrent-single-winner` | 八個執行緒用同一個預期版本改寫同一份文件：正好一筆成功，其餘都是衝突 |
| `batch-read-atomic` | 寫入方連續 200 次在同一次提交改寫兩份文件；讀取方反覆批次讀取這兩份直到寫入方結束，結束後再讀一次：每次讀到的兩份都屬於同一次提交 |
| `content-roundtrip` | 空字串、多位元組文字、CRLF 換行、1 MiB 的內容，讀回來逐位元組相同 |

合格測試要夠嚴格才有意義。所以這個 crate 的測試另外準備四個故意寫錯的儲存，並斷言合格測試抓得到：邊檢查邊寫入的（`conflict-rejects-whole-commit` 失敗）、不檢查預期版本的（`conflict-stale-version`、`conflict-absent-but-exists`、`conflict-missing-document` 失敗）、提交失敗仍保存事件的（`conflict-rejects-whole-commit` 失敗）、批次讀取時第二份文件回傳上一次提交內容的（`batch-read-atomic` 失敗）。第四個不靠執行緒的排程：讀取方在寫入方結束後的最後一次讀取一定讀到不一致的兩份。

- **否決：沿用 TeamStore 合格測試的故障注入（在提交的各階段模擬當機）** — 它要求實作提供注入點，JS 寫的儲存做不到。改用「最後一筆衝突」來檢查整筆不寫，任何實作都能跑。

## Implementation Contract

**行為**

- workspace 多一個套件 `speclink-store-port`。沒有其他套件依賴它，既有的指令、輸出、線上協定都不變。
- 實作 `StorePort` 的人，用 `conformance::run` 就能知道自己的實作是否符合契約，以及是哪一個檢查沒過。

**介面**

- 公開的型別：`StorePort`、`ScopeId`、`DocKey`、`KeyPrefix`、`Version`、`Timestamp`、`Actor`、`Document`、`ListEntry`、`Listing`、`Expected`、`Op`、`Event`、`Commit`、`PortError`、`InvalidValue`、`MemoryStorePort`、`RecordedEvent`。
- 公開的常數：`MAX_VERSION`（9007199254740991）、`MAX_KEY_BYTES`（512）、`MAX_SCOPE_BYTES`（256）。
- 公開的模組：`keys`（D9 的建立函式與前綴常數）、`conformance`（`PortHarness`、`run`、`ConformanceReport`、`CheckResult`、`CheckOutcome`）。
- 每個值型別都經建構函式取得，建構函式回傳 `Result<_, InvalidValue>`。

**失敗的樣子**

- 值不合法：建構函式回傳 `InvalidValue`，訊息指出是哪個值、違反哪一條規則。
- 預期不符：`PortError::Conflict`，帶鍵、預期、實際。
- 暫時性失敗與其他失敗：`PortError::Unavailable`、`PortError::Backend`。記憶體版實作不會回傳這兩類，唯一的例外是：範圍版本號已經是 `MAX_VERSION` 時，下一次提交回傳 `PortError::Backend`，而且不改任何東西。
- 合格測試不 panic：受測儲存回傳非預期的結果時，該檢查記為失敗並附說明，其餘檢查照跑。

**驗收**

- `cargo test -p speclink-store-port` 全部通過，包含：值型別的邊界測試、`keys` 的對照表測試、`MemoryStorePort` 通過 `conformance::run` 的全部檢查（事件的檢查不略過）、四個故意寫錯的儲存各自讓指定的檢查失敗。
- `cargo tree -p speclink-store-port` 的輸出不含 `speclink-core` 與 `speclink-store`。
- `cargo build --workspace` 通過，沒有新的警告。

**範圍**

- 範圍內：新 crate 的全部程式與測試、workspace 的成員清單。
- 範圍外：任何既有 crate 的程式、Node 綁定、文件（README 與 docs 留給最後的文件變更）、`speclink/config.yaml` 的專案說明。

## Risks / Trade-offs

- [回歸對照] 這個變更不碰既有的輸出，golden 與 CLI 測試不受影響 → 驗收只跑新 crate 的測試與 workspace 建置，並在最後跑 scripts 的守門。
- [跨平台] 併發的兩個檢查用作業系統的執行緒，在 Windows、macOS、Linux 的排程不同 → `concurrent-single-winner` 的斷言與排程無關（不論誰先，正好一筆成功）；`batch-read-atomic` 的寫入次數固定是 200 次，讀取方在寫入方結束後一定再讀一次，結果不依賴時間。鍵一律用 `/`，排序用位元組順序，不受平台影響。
- [三個操作是否足夠] 還沒有引擎的程式經這份契約讀寫，「三個操作足以供應引擎全部的讀取」尚未驗證 → 由變更 engine-on-store-port 驗證。契約在同一次發版之前都能調整。
- [鍵的命名規則先於使用者] D9 的對照表還沒有呼叫端 → 同上，由下一個變更在接線時確認，必要時在那個變更修改。
- [合格測試抓不到當機] 提交到一半當機後的狀態沒有檢查，因為不能要求每個實作提供注入點 → 契約寫明提交 MUST 是一次交易；文件變更會在宿主要做的事裡寫明這一點。
- [併發檢查對慢的儲存] 在遠端資料庫上跑 200 次提交會慢 → 次數是常數，先用固定值；宿主回報太慢時再調整。
- [沒有不檢查版本的寫入] 引擎若有「沒讀過就覆寫」的路徑，接線時要先讀一次 → 由下一個變更量測後決定要不要加。

## Open Questions

- `batch-read-atomic` 的 200 次提交對宿主的資料庫是否太多，要等第一個真實的實作回報。
- 看板順序的鍵 `board/order` 的內容格式沿用今天 TeamStore 裡的那份文件，接線時確認。

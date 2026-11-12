## Context

- 變更的 metadata（`.meta.yaml`）是一份扁平的 YAML。引擎用 serde 解析成固定欄位的結構，沒有設定「拒絕未知欄位」，所以未知的鍵在讀取時被忽略。改寫用的是逐行的文字手術（`KeyLines`）：只動頂層的那幾行，其餘位元組原樣保留，文件不會被重新序列化。它有寫入整個區塊的方法（頂層的鍵加上縮排的續行）。值的跳脫是呼叫端的責任，用引擎的純量跳脫函式。
- 討論記錄開頭有一個 frontmatter 區塊，解析是對九個固定的鍵各取一次值；改寫同樣只動有變的鍵，未知的鍵原樣保留。
- 建立變更時 metadata 是用字串格式化組出來的；建立討論時 frontmatter 也是。
- 沒有通用的「設一個鍵」的動詞。寫單一鍵的動詞（宣告依賴、改 schema、排序、開工標記）各有自己的命令。
- 宿主套件每次呼叫收身分與範圍；Node 的 `host.dispatch` 收 `{ identity, scope, argv, stdin }`，`host.handleRequest` 收請求與身分。沒有附帶資料的位置。執行前掛鉤收到命令的全部參數。
- 命令的名稱與參數有逐命令的 golden；線上歸屬表在新增命令時會逼出決定。
- 已標示即將移除的 31 方法 JS 儲存，把 metadata 轉成十個固定的欄位再交給 JS，未知的鍵過不去。

限制：

- 討論的裁定：引擎只存不解讀；宿主附上的資料不經過模型。
- 文字手術寫 YAML 的純量必須經跳脫函式。
- `--json` 的欄位是契約；沒有用到新功能的專案，輸出不能變。

## Goals / Non-Goals

**Goals:**

- 宿主能在變更與討論上存自己的鍵值，並從每個讀取的出口讀回來。
- 建立時附帶的欄位和建立是同一次寫入。
- 原版的 CLI 也能寫與讀。
- 引擎的任何行為都不因欄位的內容而改變。

**Non-Goals:**

- 非字串的值、已封存文件的修改、查詢與篩選、Desktop 的畫面、其他種類的文件。

## Decisions

### D1: 規則

| 項目 | 規則 |
|---|---|
| 鍵 | `^[a-z][a-z0-9_.-]{0,63}$` |
| 值 | UTF-8 字串，0 到 1024 個位元組，不含換行與其他控制字元 |
| 數量 | 每份文件最多 32 個鍵 |

- 空字串是合法的值（「有這個鍵、沒有內容」）。要移除用 unset。
- 引擎不保留任何鍵或前綴。欄位在自己的區塊裡，不會和 speclink 的鍵相撞。
- 規則只在寫入時檢查。讀取寬鬆：區塊裡既有的項目，值是純量就讀成字串（數字與布林照字面），不是純量的略過；鍵不合規則的照樣讀出。這是為了不讓人手改過的檔案變成整份 metadata 讀不了。

規則放在 `speclink-core` 新增的 `lifecycle/fields.rs`，是唯一的實作；CLI、線上與 Node 都經引擎命令，所以都用到它。

### D2: 存放的格式

變更的 metadata：

```yaml
fields:
  source: "portal"
  ticket: "PROJ-123"
```

- 用 `KeyLines` 寫入整個區塊的方法寫 `fields`。區塊裡每個鍵一行，兩格縮排，依鍵的位元組順序排列。
- 值**一律**加雙引號，內部的 `\` 與 `"` 跳脫。不用「需要時才加引號」的跳脫函式，因為 `123`、`true`、`null` 這些值不加引號會被 YAML 讀成別的型別。新增一個小函式做這件事，放在引擎的 util 旁，和既有的跳脫函式並列並在註解寫明差別。
- 最後一個鍵被移除時，整個 `fields` 區塊移除，不留空的鍵。
- `ChangeMeta` 新增 `fields`（預設是空的），用寬鬆的反序列化（D1 的讀取規則）。

討論記錄：同樣的 `fields:` 區塊，放在 frontmatter 裡。frontmatter 今天沒有區塊形式的鍵；`KeyLines` 的 frontmatter 模式本來就支援寫區塊。討論的解析新增讀這個區塊（同一個讀取函式）。

封存：metadata 與討論記錄整份搬移，欄位跟著走。封存流程對 metadata 的既有改寫不動 `fields` 區塊。

- **否決：每個欄位一個頂層的鍵加前綴（例如 `x_ticket`）** — 會和未來 speclink 自己的鍵混在一起，讀取時要掃描全部的鍵。
- **否決：另存一份文件（例如 `.fields.json`）** — 建立時要兩筆寫入；封存、刪除都要多處理一份。

### D3: 命令

`speclink-core` 新增 `command/fields.rs`：

- `Command::ChangeFieldsEdit { name, set: BTreeMap<String, String>, unset: Vec<String> }`，名稱 `change-fields-edit`。
- `Command::DiscussFieldsEdit { slug, set, unset }`，名稱 `discuss-fields-edit`。
- 步驟：讀文件 → 取出既有的欄位 → 先套用 unset 再套用 set → 檢查 D1 的規則（每個新寫入的鍵與值、合計的數量）→ 和原本相同就不寫 → 否則一次文字手術寫回。
- 失敗後的狀態：任何檢查不過都在寫入之前，零寫入。寫入只有一筆。
- 同一個鍵同時在 set 與 unset 裡 → `invalid_argv`。set 與 unset 都是空的 → `invalid_argv`。
- 討論只能改進行中的記錄；已封存的回 `not_found`（和其他討論的寫入動詞一致）。
- 結果帶寫入後的全部欄位與這次實際變動的鍵。有變動時產生事件 `change-fields-changed { change, keys }` 或 `discussion-fields-changed { slug, keys }`；沒有變動時不產生事件、不提交。
- 值不出現在事件裡，只有鍵：事件會被推送給訂閱者，值可能是宿主內部的資料。

建立時附帶：`Command::NewChange`、`Command::DiscussNew`、`Command::DiscussPromote` 各新增 `fields: BTreeMap<String, String>`（預設空）。建立的函式在組出 metadata 或 frontmatter 的文字時，把欄位區塊一起寫進去，所以是同一份文件的同一次寫入。規則不過時整個建立失敗，零寫入。建立時的事件不變（不另外產生欄位的事件）。

### D4: 動詞

`speclink-verbs` 新增家族檔 `families/fields.rs`，掛在既有的 `change` 與 `discuss` 兩個家族底下：

```
speclink change  field set   <name> <key>=<value>...
speclink change  field unset <name> <key>...
speclink discuss field set   <slug> <key>=<value>...
speclink discuss field unset <slug> <key>...
```

- `<key>=<value>` 在第一個 `=` 分開；沒有 `=` 是用法錯誤。值可以含 `=`。
- 同一次呼叫重複給同一個鍵是用法錯誤。
- `new change` 與 `discuss new` 新增可重複的 `--field <key>=<value>`，同樣的解析。
- 輸出：`✓ <name>: set ticket, source` 或 `✓ <name>: unset ticket`；沒有任何變動時 `✓ <name>: no change`。`--json`：`{ "change": "<name>", "fields": { … } }`（討論是 `"discussion"`）。
- 值會出現在 shell 的歷史與程序清單裡；文件要提醒不要放機密。這次不提供從 stdin 讀值的形式。

### D5: 讀取的輸出

引擎的結果型別新增選填的 `fields`，序列化時「空的就不輸出」：

- 清單的變更項目（`list --json`）。
- 單一變更的 show（`show <change> --json`）。
- 討論資訊（`discuss list --json`、`discuss show --json`、`discuss search --json` 的命中項目——它們共用同一個型別）。

沒有欄位的文件，輸出逐位元不變；既有的 golden 不用更新，並用這一點當回歸的檢查。

線上：`ChangeSummary`、`ChangeStatus`、`ArchivedItem`、線上的討論資訊各新增選填的 `fields`（空的不序列化）。清單與單筆由 Host 補欄位時讀的同一份 metadata 填入；討論由引擎的討論資訊帶出。CLI 的遠端分支把它們轉回引擎的型別，兩種模式同形。

成本：清單本來就讀每個變更的 metadata，不多發呼叫。

### D6: 宿主在呼叫時附上

Rust：`HostKit` 新增 `run_with_fields(caller, cmd, fields)` 與 HTTP 入口的 `handle_with(request, identity, &CallExtras { fields })`。既有的 `run` 與 `handle` 等於附上空的欄位。

套用的規則（一個函式，兩個門共用）：

- 命令是 `NewChange`、`DiscussNew`、`DiscussPromote` → 把宿主的欄位合併進命令的 `fields`，同名時宿主的值蓋過命令自己的。
- 其他命令 → 忽略宿主的欄位。
- 合併發生在執行前掛鉤之前，所以掛鉤看到的參數是最後要寫入的欄位。
- 合併後不合 D1 的規則 → 這次呼叫以 `invalid_argv` 失敗（是宿主的程式錯誤，要讓它看得到），零寫入。

Node：

```ts
host.dispatch({ identity, scope, argv, stdin, fields?: Record<string, string> })
host.handleRequest(request, identity, extras?: { fields?: Record<string, string> })
```

`fields` 的值不是字串時，呼叫以 `invalid_argv` 失敗。

為什麼「其他命令忽略」而不是報錯：宿主的工具處理函式不知道模型這次選了哪個動詞；它最自然的寫法是每次都帶同一組欄位（例如來源）。

- **否決：宿主改寫命令的參數（一般化的改寫掛鉤）** — 討論已否決能改內容的掛鉤。這裡宿主只能加自己的欄位，不能動命令的其他部分。

### D7: 線上

- `CreateChangeRequest`、`CreateDiscussionRequest`、轉出變更的請求：新增選填的 `fields`（物件，字串對字串）。
- `PUT /changes/{name}/fields` 與 `PUT /discussions/{slug}/fields`：請求 `{ "set": { … }, "unset": [ … ] }`（兩者都選填，至少一個不是空的），回應 `{ "fields": { 寫入後的全部欄位 } }` 並帶新的範圍版本。它們是寫入，受寫入能力限制。
- 錯誤：規則不過 → 400 `invalid_argument`；找不到 → 404。
- 線上歸屬表新增兩個命令。
- `speclink-remote`：`set_change_fields`、`set_discussion_fields`；建立的三個呼叫多一個參數。

新版 CLI 對舊版 Server 用 `--field`：舊版 Server 會忽略請求裡不認得的欄位並成功建立。CLI 在建立成功後比對回應：送出了欄位而握手的版本資訊顯示 Server 不支援時，在 stderr 印一行警告「這台 Server 沒有保存欄位」。判斷的依據用握手 `capabilities` 新增的布林 `fields`（宿主套件固定宣告為真）。

### D8: 舊的 JS 儲存

`createEngine` 的 JS 儲存形式：四個欄位動詞與 `--field` 以 `refused` 在動手前拒絕，訊息指向 `createHost`。理由是那個橋接不帶未知的鍵，寫進去也讀不回來；它已標示即將移除。fs 形式照常可用。

## Implementation Contract

**行為**

- `speclink change field set demo ticket=PROJ-1 source=portal` 之後：metadata 有 `fields` 區塊，兩個值都加雙引號；`speclink show demo --json` 的 `fields` 是這兩組；`speclink list --json` 裡 demo 的項目有同樣的 `fields`；其他變更的項目沒有 `fields` 鍵。
- 值 `123`、`true`、`a: b # c`、含 `"` 與 `\` 的字串、中文，寫入後讀回來逐字相同，metadata 仍可解析。
- `speclink new change demo --field ticket=PROJ-1`：變更建立時 metadata 就有欄位；經儲存接口是正好一次提交。
- 對有欄位的變更執行既有的寫入動詞（勾選任務、宣告依賴、蓋章、封存）之後，欄位原樣還在。
- `host.dispatch({ …, argv: ['new', 'change', 'demo'], fields: { source: 'portal' } })`：建立的變更有 `source`；`beforeCommand` 收到的參數含它。同樣的 `fields` 配上 `['list', '--json']`：照常成功，沒有任何寫入。
- 沒有任何欄位的專案：`list --json`、`show --json`、`discuss list --json` 的輸出與這個變更之前逐位元相同。

**介面與資料形狀**

- 引擎：`Command::ChangeFieldsEdit`、`Command::DiscussFieldsEdit`；三個建立命令的 `fields`；兩種事件；`ChangeMeta.fields`；討論資訊的 `fields`。
- CLI：四個動詞、兩個 `--field` 旗標。
- 線上：三個請求的 `fields`、兩條路由、四個回應型別的 `fields`、`capabilities.fields`。
- Rust：`HostKit::run_with_fields`、`HostKit::handle_with`、`CallExtras`。
- Node：`HostDispatchCall.fields`、`handleRequest` 的第三個參數。

**失敗的樣子**

| 情況 | 結果 |
|---|---|
| 鍵不合規則、值太長或含換行、超過 32 個鍵 | `invalid_argv`（線上 400），零寫入 |
| 同一個鍵同時 set 與 unset；兩者都空 | `invalid_argv` |
| 變更或討論不存在；討論已封存 | `not_found` |
| unset 不存在的鍵 | 成功，沒有變動 |
| 宿主附上的欄位不合規則 | 這次呼叫 `invalid_argv`，零寫入 |
| 新版 CLI 的 `--field` 對舊版 Server | 建立成功，stderr 一行警告 |
| 舊的 JS 儲存 | `refused`，動手前拒絕 |
| 掛鉤拒絕 | `refused`，零寫入 |

**驗收**

- `cargo test -p speclink-core`：規則、寫入的格式、往返、建立時附帶、既有動詞不弄丟欄位、封存後還在；命令的 golden 同批更新；`render_golden` 不變。
- `cargo test -p speclink-verbs`：四個動詞與 `--field` 的解析與輸出。
- `cargo test -p speclink-host --test free_fields`：兩條路由、三個請求、回應的欄位、宿主附上的合併與忽略、掛鉤看到的參數、一次提交。
- `cargo test -p speclink-cli --test it free_fields::`：兩種模式同形、舊版 Server 的警告。
- Node：`free-fields.spec.ts`。
- `cargo test -p speclink-server`、`cargo test -p speclink-remote`、`cargo test -p speclink-desktop` 不改就通過。

**範圍**

- 在範圍內：上面列的介面。
- 不在範圍內：桌面 app、技能、已封存文件的修改、查詢、使用者文件。

## Risks / Trade-offs

- [回歸對照] 四個 `--json` 輸出多一個選填的鍵。→ 「空的不輸出」讓既有的 golden 不變；另加有欄位時的快照。
- [YAML 注入] 值由使用者或宿主提供，寫進 metadata。→ 值一律加雙引號並跳脫，不含換行；鍵限制在安全的字元集；往返測試涵蓋會被 YAML 誤讀的值與含 `:`、`#`、引號的值。
- [既有動詞弄丟欄位] 改寫 metadata 的地方很多。→ 文字手術不動未知的行；測試對每一個寫 metadata 的動詞（含封存、變更 schema、蓋章）跑一次「之後欄位還在」。
- [跨平台] metadata 可能是 CRLF。→ 區塊的寫入沿用 `KeyLines` 對行尾的處理；測試放一份 CRLF 的 metadata。
- [被當成機密的存放處] 值會出現在清單回應、事件以外的各處。→ 事件只帶鍵；文件寫明不要放機密。
- [儲存解耦] 沒有新的文件種類；宿主的儲存不用改。

## Migration Plan

沒有資料搬移。要退回時還原 commit；既有文件裡的 `fields` 區塊會被舊版忽略並保留。

## Open Questions

- 已封存清單項要不要帶 `fields`。目前定為帶（宿主常要在封存後對回工單）；它用的是已經讀過的封存 metadata，不多發呼叫——實作時用計數的儲存確認，會多發時改成不帶並寫進已知缺口。
- 轉出變更（`discuss promote`）的 CLI 要不要也加 `--field`。目前只在命令與線上加，CLI 的旗標先不加；實作時看這個動詞的旗標有沒有既有的對等做法再定。

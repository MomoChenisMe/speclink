## ADDED Requirements

### Requirement: 清單的詳細輸出

`speclink list --json` SHALL 接受旗標 `--detail`。帶上它時，`changes` 的每一項 SHALL 在既有欄位之外增列下列選填的鍵，每個鍵 SHALL 只在該變更有對應的記錄時出現：

- `review`、`verify`：物件。形狀 SHALL 與 `speclink show <change> --json` 的同名鍵相同（`state` 是 `inProgress` 或 `stamped`，蓋章後另有蓋章的時間、輪數、任務數等既有欄位）。該關卡沒有工單也沒有章時 SHALL 不輸出這個鍵。
- `claimed`：物件，`by`（字串）、有記錄才出現的 `byId`（字串）與 `at`（字串）。形狀 SHALL 與 `show --json` 的 `claimed` 相同。沒有被認領時 SHALL 不輸出。
- `branch`：字串，開工時記下的分支。沒有記錄時 SHALL 不輸出。
- `landed`：物件，只有 `commit`（字串）。沒有落地記錄時 SHALL 不輸出。

鍵名一律 camelCase。清單的既有欄位、項目的順序與 `specs` 區段 SHALL 不受 `--detail` 影響；metadata 損壞的變更 SHALL 照既有行為列出（帶 `metaError`），且 SHALL NOT 帶上述任何一個鍵。

不帶 `--detail` 的 `speclink list --json` SHALL 維持既有的位元級輸出。不帶 `--json` 的 `--detail`、以及 `--detail` 與 `--specs` 並用，SHALL 以用法錯誤拒絕：CLI 以非零 exit code 結束、訊息走 stderr、stdout 沒有輸出；經宿主套件執行時以 `invalid_argv` 拒絕。`--detail` SHALL NOT 改變人眼輸出，因為它不能脫離 `--json` 使用。輸出 SHALL 不隨 `locale`（`tw`、`ja`、`en` 或未設定）改變。

三種執行方式對相同的資料 SHALL 輸出相同的鍵與值：本機專案、連到遠端的 CLI、宿主套件。連到遠端的 CLI SHALL 以既有的變更清單回應組出這些鍵，SHALL NOT 對每個變更另外發一次請求。連到不提供這些欄位的舊版 Server 時，對應的鍵 SHALL 不出現，指令 SHALL NOT 失敗。離線或認證失效時 SHALL 與不帶 `--detail` 的 `list` 相同地失敗。本機專案沒有認領，所以 `claimed` 在本機 SHALL 不出現（除非 metadata 本來就帶著認領欄位）。

蓋章之後檔案是否又被改過，SHALL NOT 由清單判斷：`review` 與 `verify` 只反映記錄下來的狀態，與 `show --json` 相同。

#### Scenario: 有記錄的變更帶出四類狀態

- **WHEN** 變更 demo 的審查已蓋章、驗證有未結的工單、由 Alice（識別 u_1）認領、開工時記下分支 `speclink/demo`、落地的 commit 是 `0123456789abcdef0123456789abcdef01234567`，執行 `speclink list --json --detail`
- **THEN** demo 這一項含 `review`（`state` 是 `stamped`）、`verify`（`state` 是 `inProgress`）、`claimed`（`by` 是 `Alice`、`byId` 是 `u_1`、有 `at`）、`branch`（`speclink/demo`）、`landed`（`{ "commit": "0123456789abcdef0123456789abcdef01234567" }`），既有的欄位與不帶 `--detail` 時相同

##### Example: 各種記錄對應的鍵

| 變更的記錄 | 多出的鍵 |
| --- | --- |
| 什麼記錄都沒有 | 沒有任何新鍵；這一項與不帶 `--detail` 時相同 |
| 只有審查章 | `review` |
| 審查工單未結、沒有章 | `review`（`state` 是 `inProgress`） |
| 被認領、沒有識別 | `claimed`（只有 `by` 與 `at`） |
| 記下分支、還沒落地 | `branch` |
| 已落地 | `branch`（有記錄時）、`landed` |

#### Scenario: 不帶 --detail 時輸出不變

- **WHEN** 對同一個專案執行 `speclink list --json`，專案裡有已蓋章、被認領、已落地的變更
- **THEN** 輸出與引入 `--detail` 之前逐位元相同，任何一項都沒有 `review`、`verify`、`claimed`、`branch`、`landed`

#### Scenario: --detail 不帶 --json 被拒絕

- **WHEN** 執行 `speclink list --detail`
- **THEN** 指令以非零 exit code 結束，stderr 指出 `--detail` 要和 `--json` 一起用，stdout 沒有清單

#### Scenario: --detail 與 --specs 並用被拒絕

- **WHEN** 執行 `speclink list --specs --json --detail`
- **THEN** 指令以非零 exit code 結束，stderr 指出兩個旗標不能並用

#### Scenario: 三種執行方式輸出相同

- **WHEN** 本機專案、一台 Server 與一個宿主套件的儲存含相同的變更與相同的記錄（不含認領），分別執行 `speclink list --json --detail`、連到該 Server 執行同一個指令、以及經宿主套件執行同一個 argv
- **THEN** 三個結果的 `changes` 深度相等（本機專案才有的 `worktree` 鍵除外）

#### Scenario: 連到舊版 Server 時不失敗

- **WHEN** CLI 連到一台變更清單回應沒有關卡、分支與落地欄位的 Server，執行 `speclink list --json --detail`
- **THEN** 指令成功，每一項都沒有新鍵

#### Scenario: metadata 損壞的變更不帶新鍵

- **WHEN** 專案含一筆 metadata 損壞的變更，執行 `speclink list --json --detail`
- **THEN** 該項照既有行為帶 `metaError`，沒有 `review`、`verify`、`claimed`、`branch`、`landed`；指令成功

### Requirement: 內嵌宿主的人眼輸出不印路徑行

經宿主套件執行動詞而得到的人眼輸出，SHALL NOT 含下列路徑行，因為文件存在宿主的儲存裡，沒有對應的本機路徑：

1. `new change` 的 `Path:` 行。
2. `discuss promote` 的 `Path:` 行與緊接其後的 propose 提示行（兩行一起去留，與 remote 模式的裁定相同）。
3. `discuss new` 的 `Path:` 行。

這三個動詞的其餘輸出行 SHALL 與本機模式逐位元相同。它們的 `--json` 輸出 SHALL 不變，`path` 鍵照舊出現。其他動詞經宿主套件的人眼輸出 SHALL 不受本需求影響。CLI 在本機與 remote 兩模式的輸出、以及 Node SDK 的 createEngine（本機資料夾）的輸出 SHALL 維持既有的位元級輸出；「動詞人眼輸出的兩模式同形」的分歧清單 SHALL 不變。

#### Scenario: 經宿主套件建立變更不印路徑

- **WHEN** 經宿主套件執行 `new change demo`（人眼形式）
- **THEN** 輸出的第一行是 `✓ Created change: demo`，有 `Schema:` 行，沒有以 `Path:` 開頭的行

##### Example: 三個動詞在兩種執行方式的輸出

| 動詞 | 本機 CLI 的輸出行 | 經宿主套件的輸出行 |
| --- | --- | --- |
| `new change demo` | `✓ Created change: demo`、`Path: …`、`Schema: spec-driven` | `✓ Created change: demo`、`Schema: spec-driven` |
| `discuss new "Topic" --slug topic` | `✓ Created discussion: topic`、`Topic: Topic`、`Path: …` | `✓ Created discussion: topic`、`Topic: Topic` |
| `discuss promote topic` | `✓ Promoted discussion 'topic' → change 'topic'`、`Path: …`、propose 提示行 | `✓ Promoted discussion 'topic' → change 'topic'` |

#### Scenario: JSON 的 path 鍵保留

- **WHEN** 經宿主套件執行 `discuss new "Topic" --slug topic --json`
- **THEN** 回傳的物件含 `path` 鍵，與引入本需求之前相同

#### Scenario: 本機資料夾的引擎照常印路徑

- **WHEN** 以 Node SDK 的 createEngine 對一個本機專案資料夾執行 `new change demo`
- **THEN** 輸出含 `Path:` 行，與引入本需求之前逐位元相同

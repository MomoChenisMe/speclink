## ADDED Requirements

### Requirement: 討論的 JSON 輸出帶出轉出的變更

`speclink discuss list --json` 的每一項、`speclink discuss show <slug> --json` 的 `info` 與 `speclink discuss search --json` 的每一筆命中 SHALL 增列 `promotedTo`：字串陣列，這份討論轉出或併入的變更名稱，順序 SHALL 與討論記錄 frontmatter 的 `promoted_to` 累加順序相同。清單是空的時候 SHALL 不輸出這個鍵。輸出的鍵照字母順序排列，所以 `promotedTo` SHALL 排在 `path` 與 `rounds` 之間；既有鍵的名稱、相對順序與值 SHALL 不變。`discuss list --archived --json` 的項目 SHALL 適用同一條規則。搜尋的命中項是清單的項目加上 `matches`，所以在途與已封存討論的命中項 SHALL 帶出與清單相同的 `promotedTo`。

這是刻意的輸出變更：在這條需求之前，這三個動詞的 `--json` 輸出沒有 `promotedTo`。沒有轉出也沒有併入任何變更的討論，輸出 SHALL 維持既有的位元級輸出。三個動詞的人眼輸出 SHALL 維持既有的位元級輸出，`--no-color` 的行為不變。輸出 SHALL 不隨 `locale`（`tw`、`ja`、`en` 或未設定）改變。這條需求 SHALL NOT 改變任何寫入：討論記錄與變更的 metadata SHALL 不變。成功時 exit code SHALL 是 0，輸出走 stdout；找不到討論時 `discuss show` SHALL 與既有行為相同地失敗（非零 exit code、訊息走 stderr）。

三種執行方式對相同的資料 SHALL 輸出相同的 `promotedTo`：本機專案、連到遠端的 CLI、宿主套件。連到遠端的 CLI SHALL 以討論資訊回應與搜尋回應既有的 `promotedTo` 組出這個鍵，SHALL NOT 另外發請求。連到不送這個欄位的舊版 Server 時，這個鍵 SHALL 不出現，指令 SHALL NOT 失敗。離線或認證失效時 SHALL 與這條需求之前相同地失敗。這三個動詞只讀取，沒有 revision 衝突的情況。

#### Scenario: 已轉出的討論帶出變更名稱

- **WHEN** 討論 two 先轉出變更 add-two、再併入變更 add-three，執行 `speclink discuss list --json`
- **THEN** exit code 是 0；two 這一項含 `"promotedTo": ["add-two", "add-three"]`，位置在 `path` 與 `rounds` 之間

#### Scenario: discuss show 的 info 帶出同一份清單

- **WHEN** 對同一份討論執行 `speclink discuss show two --json`
- **THEN** 輸出的 `info` 含 `"promotedTo": ["add-two", "add-three"]`，`content` 與這條需求之前相同

#### Scenario: 搜尋的命中項帶出同一份清單

- **WHEN** 討論 two 已轉出 add-two、再併入 add-three，討論 idea 沒有轉出也沒有併入任何變更，執行 `speclink discuss search two idea --json`
- **THEN** two 的命中項含 `"promotedTo": ["add-two", "add-three"]`；idea 的命中項沒有 `promotedTo` 鍵，與這條需求之前逐位元相同；兩筆命中的 `matches` 不變

#### Scenario: 沒有轉出的討論輸出不變

- **WHEN** 討論 idea 沒有轉出也沒有併入任何變更，執行 `speclink discuss list --json` 與 `speclink discuss show idea --json`
- **THEN** idea 的輸出沒有 `promotedTo` 鍵，與這條需求之前逐位元相同

#### Scenario: 已封存的討論也帶出

- **WHEN** 討論 old 轉出變更 first-cut 後已封存，執行 `speclink discuss list --archived --json`
- **THEN** old 這一項含 `"promotedTo": ["first-cut"]`

#### Scenario: 人眼輸出不變

- **WHEN** 對含已轉出討論的專案執行 `speclink discuss list`、`speclink discuss show two` 與 `speclink discuss search two`
- **THEN** 三個輸出與這條需求之前逐位元相同

#### Scenario: 三種執行方式輸出相同

- **WHEN** 本機專案、一台 Server 與一個宿主套件的儲存含相同的討論 two（已轉出 add-two），分別執行 `speclink discuss list --json`、連到該 Server 執行同一個指令、以及經 `host.dispatch` 執行同一個 argv
- **THEN** 三個結果裡 two 的 `promotedTo` 都是 `["add-two"]`

#### Scenario: 連到舊版 Server 時不失敗

- **WHEN** CLI 連到一台討論資訊回應沒有 `promotedTo` 的 Server，執行 `speclink discuss list --json`
- **THEN** 指令成功，每一項都沒有 `promotedTo`

#### Scenario: 找不到討論

- **WHEN** 沒有 slug 為 ghost 的討論，執行 `speclink discuss show ghost --json`
- **THEN** exit code 非零，stderr 是既有的找不到訊息，stdout 沒有輸出

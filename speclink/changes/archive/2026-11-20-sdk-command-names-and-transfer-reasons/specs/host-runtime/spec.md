## ADDED Requirements

### Requirement: 經儲存接口寫入時回報儲存的鍵

引擎經儲存接口執行 `new artifact` 時，結果的 `path` SHALL 是那份文件在儲存裡的鍵：`changes/<變更名稱>/<artifact 的路徑>`，以 `/` 分隔，開頭沒有規格資料夾的名稱。這個值 SHALL 與儲存的 commit 函式在同一次呼叫收到的那一筆寫入的鍵逐字相同，也 SHALL 與同一個宿主裡 `discuss new` 回報的 `path` 同一種形式（鍵）。這是刻意的變更：變更前這個值是 `speclink/changes/<變更名稱>/<artifact 的路徑>`。

引擎經儲存接口執行 `discuss promote --json` 時，結果的 `path` SHALL 是新變更的鍵的共同開頭 `changes/<變更名稱>`，以 `/` 分隔，開頭沒有規格資料夾的名稱。這也是刻意的變更：變更前這個值是 `speclink/changes/<變更名稱>`。

本需求 SHALL NOT 改變下列輸出，它們維持既有的位元級輸出：CLI 在本機模式的 `new artifact`（`path` 是本機檔案的路徑）、CLI 在 remote 模式的 `new artifact`、Node SDK 的 createEngine（本機資料夾）的 `new artifact`、經儲存接口執行時「Artifact already exists: …」這則拒絕訊息的文字、內嵌宿主的人眼輸出（依「內嵌宿主的人眼輸出不印路徑行」不印路徑），以及經儲存接口執行時 `instructions --json` 的 `changeDir` 與 `contextFiles`（它們由每個變更的資料夾組出，仍以 `speclink/changes/` 開頭）。

#### Scenario: 內嵌宿主的 new artifact 回報鍵

- **WHEN** 宿主經 host.dispatch 執行 `['new', 'artifact', 'proposal', '--change', 'demo', '--stdin', '--json']`，stdin 是一份合法的 proposal
- **THEN** 回傳物件的 `path` 是 `changes/demo/proposal.md`；儲存的 commit 函式收到一筆寫入，它的鍵也是 `changes/demo/proposal.md`

#### Scenario: delta spec 的路徑也是鍵

- **WHEN** 宿主經 host.dispatch 執行 `['new', 'artifact', 'spec', 'user-auth', '--change', 'demo', '--stdin', '--json']`，`user-auth` 是正式規格已有的 capability
- **THEN** 回傳物件的 `path` 是 `changes/demo/specs/user-auth/spec.md`

#### Scenario: 內嵌宿主的 discuss promote 回報鍵的開頭

- **WHEN** 討論 topic-one 已有結論，宿主經 host.dispatch 執行 `['discuss', 'promote', 'topic-one', '--name', 'next-one', '--json']`
- **THEN** 回傳物件的 `path` 是 `changes/next-one`；儲存的 commit 函式收到的寫入裡有鍵 `changes/next-one/.meta.yaml`

#### Scenario: 本機資料夾的輸出不變

- **WHEN** 以 createEngine 對本機資料夾執行同一個 `new artifact … --json`
- **THEN** 回傳物件的 `path` 與引入本需求之前相同，是本機檔案的路徑，結尾是 `speclink/changes/demo/proposal.md`

#### Scenario: 已存在時的拒絕訊息不變

- **WHEN** 宿主經 host.dispatch 對已經有 proposal 的變更 demo 再執行一次 `new artifact proposal --change demo --stdin --json`，沒有 `--force`
- **THEN** 呼叫 reject，Error 的 `message` 與引入本需求之前逐字相同，仍以 `Artifact already exists:` 開頭；儲存的 commit 函式沒有被呼叫

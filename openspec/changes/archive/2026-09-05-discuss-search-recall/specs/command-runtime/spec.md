## MODIFIED Requirements

### Requirement: 動詞覆蓋與跨入口一致性

引擎 SHALL 提供唯一的命令執行層，覆蓋讀寫規格儲存的領域動詞——查詢：list、show、status、instructions、validate、analyze、drift、artifact cat、language show、discuss list、discuss show 與 discuss search；變更：new change、new artifact、task done、task undone、claim、in-progress add、archive、discard、discuss new／context／add-round／conclude／promote／link／seal／archive／discard。CLI 與 Node SDK dispatch SHALL 經此層執行覆蓋表動詞；對相同 workspace 狀態執行同一動詞，各入口 SHALL 得到相同的語意結果與錯誤分類，且既有人眼輸出與 --json 形狀 SHALL 維持位元級一致（既有輸出基線不變）。discuss search 為唯讀查詢動詞，SHALL NOT 發出領域事件。workspace bootstrap 與周邊工具動詞（init、update、config、schema、completion、templates、feedback、demo）及 remote 連線管理動詞（link、unlink、auth）SHALL NOT 進入命令層。

#### Scenario: CLI 與 dispatch 的成功結果語意一致

- **WHEN** 同一 workspace 內分別執行 speclink list --json 與 engine.dispatch(['list'])
- **THEN** 兩者回傳的 changes 清單語意相同（同名稱集合、同排序），dispatch 結果為與 CLI --json 對齊的結構化物件

#### Scenario: CLI 與 dispatch 的錯誤分類一致

- **WHEN** 對不存在的 change 分別執行 speclink status --change ghost 與 engine.dispatch(['status', '--change', 'ghost'])
- **THEN** CLI 以非零 exit code 結束且 stderr 為現行訊息；dispatch 以 Error 拒絕、code 為 not_found、message 與 CLI 訊息文字相同

#### Scenario: 覆蓋動詞輸出凍結

- **WHEN** 對同一 workspace 於命令層導入前後執行覆蓋表內任一動詞（人眼與 --json 兩形式）
- **THEN** stdout 與 stderr 逐位元一致、exit code 相同（壞設定檔情境除外，該情境見 workflow-config 與 remote-connection 規格）

#### Scenario: discuss search 本機與 server 同語意

- **WHEN** 對同一組討論記錄分別以本機 speclink discuss search drawer --json 與 server 的 GET /discussions/search?q=drawer 查詢
- **THEN** 兩者回傳的 hits 順序、每筆的 slug 與 matches 陣列相同；本機執行 SHALL NOT 產生任何領域事件

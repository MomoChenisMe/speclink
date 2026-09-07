## MODIFIED Requirements

### Requirement: archive trace 注入與零證據提示

archive 於 ADDED／MODIFIED 物化正典需求時 SHALL 一律注入 trace 區塊，內容 SHALL 僅含 source（change 名）與 updated（封存時戳）兩欄；updated SHALL 為帶時區偏移量的 RFC 3339 時戳、秒級（例 `2026-09-05T23:17:28+08:00`），且其日曆日 SHALL 與同一次封存的目錄名日期前綴相同（兩者取自同一個當下）。既有正典中純日期的 updated 行 SHALL NOT 被 archive 回改。trace 區塊 SHALL NOT 含檔案清單，SHALL NOT 掃描工作樹髒檔產生任何 trace 內容。archive SHALL NOT 因 evidence 缺席或內容拒絕封存。本機 fs 模式下，change 無任何 v2 evidence entry 時 CLI SHALL 於 stderr 印恰一行提示（點名 change 名、說明無任務證據記錄），exit code 與封存結果 SHALL 不受影響；有任一 entry 時 SHALL NOT 印出提示。引擎 SHALL 以結構化事實（ArchiveOutcome 的 evidence_recorded）回報有無證據，呈現歸呼叫端。遠端 store 模式不在本需求範圍：證據的記錄與提示應由 server 端自記自判（Phase 2 另行設計），wire 契約 SHALL NOT 為此攜帶欄位。

#### Scenario: trace 兩欄一律注入

- **WHEN** 對含 v2 evidence 的 change 執行 speclink archive
- **THEN** 封存後正典的 trace 區塊僅含 source 與 updated 兩行，updated 為 RFC 3339 時戳且前十字元等於封存目錄名的日期前綴，無 code 清單；evidence 記錄本身不受注入影響、隨目錄移入封存區

#### Scenario: 既有純日期 trace 不回改

- **WHEN** 封存一個 MODIFIED 既有需求的 change，該正典內其他需求的 trace 區塊 updated 為純日期
- **THEN** 只有被物化的需求得到新的 RFC 3339 時戳，其他需求的純日期 updated 行逐位元不變

#### Scenario: 零證據照常封存並提示

- **WHEN** 對無任何 v2 entry 的 change 執行 speclink archive
- **THEN** 封存成功、exit code 0，stderr 恰一行提示點名該 change 無任務證據記錄；正典 trace 照樣注入 source 與 updated 兩欄

#### Scenario: 有證據時一字不印

- **WHEN** 對含任一 v2 entry 的 change 執行 speclink archive
- **THEN** 封存成功且 stderr 無任何 evidence 相關提示

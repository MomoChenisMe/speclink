## MODIFIED Requirements

### Requirement: 討論以 link 動詞併入既有變更

speclink discuss link SHALL 接受兩個位置參數（討論 slug 與既有變更名），鑄造變更側連結：變更 meta 檔（openspec/changes/<change>/.openspec.yaml）的 from_discussion 欄位 SHALL 為逗號分隔清單——尚無該欄位時增寫為單值；已指向其他討論時 SHALL 於既有值尾端以逗號累加本 slug、既有值保留不覆蓋；清單已含本 slug 時 SHALL 為冪等成功不改檔。討論記錄（openspec/discussions/<slug>.md）SHALL 逐位元不變——link SHALL NOT 標記 status: promoted、SHALL NOT 寫 promoted_to；「已轉出」的標記職責移交 speclink discuss seal。open 與 concluded 狀態的討論皆 SHALL 可併入；已封存討論 SHALL NOT 可併入。指令不吃 stdin，旗標僅 --json。成功時 exit code 0，stdout 輸出單行成功訊息（含討論 slug 與變更名；--no-color 下無 ANSI 色彩）；帶 --json 時輸出含 slug 與 change 欄位（camelCase）的 payload。守衛失敗（討論不存在、討論已封存、變更不存在）時 SHALL 以非零 exit code 結束、stderr 說明原因，且兩側檔案 SHALL 逐位元不變。同一組合重跑 SHALL 為冪等成功：exit code 0 且兩側檔案內容不變。變更封存時，其 from_discussion 清單中的每份討論 SHALL 各自檢查：無其他在途變更的 from_discussion 清單引用該討論時，既有自動封存機制 SHALL 將該記錄移入 openspec/discussions/archive/；僅單一來源討論之變更，其封存的人眼輸出 SHALL 與變更前逐位元一致。本指令為 Speclink 自有延伸，不在 Spectra 對照範圍；既有指令的人眼與 --json 輸出 SHALL 逐位元不變。

#### Scenario: 成功併入既有變更

- **WHEN** 執行 speclink discuss link 並給定一份未封存討論的 slug 與一個尚無 from_discussion 的既有變更名
- **THEN** exit code 0；stdout 單行成功訊息含該 slug 與變更名；openspec/changes/<change>/.openspec.yaml 增寫 from_discussion: <slug>；討論記錄逐位元不變（status 與 promoted_to 皆不變）；帶 --json 時 payload 的 slug 與 change 欄位分別為兩參數值

#### Scenario: link 不改討論記錄

- **WHEN** 對任一狀態（open、concluded、或已由 seal 標記 promoted）的討論執行 speclink discuss link 指向既有變更
- **THEN** exit code 0；變更 meta 的 from_discussion 累加該 slug；討論記錄 frontmatter 與內文逐位元不變

#### Scenario: 出身自討論的變更再併入新討論

- **WHEN** 對 meta 已有 from_discussion 指向其他討論的變更執行 speclink discuss link 給定另一份討論的 slug
- **THEN** exit code 0；變更 meta 的 from_discussion 於既有值尾端累加本 slug、既有值保留；本討論與先前連結的討論記錄皆逐位元不變

##### Example: 累加後的 meta 欄位

- **GIVEN** 變更 cut-a 的 meta 含 from_discussion: alpha-search
- **WHEN** 執行 speclink discuss link beta-cache cut-a
- **THEN** cut-a 的 meta 欄位為 from_discussion: alpha-search, beta-cache；alpha-search 的記錄逐位元不變

#### Scenario: 同一組合重跑為冪等

- **WHEN** 對已互相連結的同一組討論與變更再次執行 speclink discuss link
- **THEN** exit code 0；變更 meta 檔與討論記錄內容逐位元不變

#### Scenario: 守衛拒絕且不落檔

- **WHEN** 執行 speclink discuss link 且命中任一守衛：討論不存在、討論已封存、變更不存在
- **THEN** 指令以非零 exit code 結束，stderr 說明原因，openspec/changes/ 與 openspec/discussions/ 下任何檔案逐位元不變

#### Scenario: 併入後隨變更自動封存

- **WHEN** 已 link 的變更執行封存，且無其他在途變更引用同一討論
- **THEN** 討論記錄自動移入 openspec/discussions/archive/，與 promote 型討論的既有封存行為一致

#### Scenario: 多來源討論的變更封存逐一共行

- **WHEN** 封存 from_discussion 清單含兩份討論的變更
- **THEN** 清單中每份討論各自檢查存活引用：無其他在途變更引用者移入 openspec/discussions/archive/、人眼輸出逐討論各一行共行訊息；仍被引用者維持在途不動

### Requirement: 技能指示引導 ingest 型結論先鑄鏈

speclink 生成的 discuss 技能內容 SHALL 指示 agent：討論結論的 Capture to 指向既有變更時，先執行 speclink discuss link 鑄鏈、再導向 /speclink-ingest 更新該變更的 artifacts。生成的 ingest 技能內容 SHALL 指示 agent：目標變更 meta 帶 from_discussion 時，經 speclink discuss show 讀取該討論結論作為一等來源、併入既有對話脈絡或 plan（不取代），並於 artifacts 更新完成時執行 speclink discuss seal 標記已轉出。內嵌技能資產、repo 技能實例（claude 與 codex 兩工具）與 render golden 基準 SHALL 同步反映此指示。

#### Scenario: 生成的 discuss 技能含 link 指引

- **WHEN** 執行 speclink init 或 speclink update 生成 claude 與 codex 的 discuss 技能
- **THEN** 生成的技能檔內容包含「Capture to 指向既有變更時先執行 speclink discuss link 再走 /speclink-ingest」的指示，render golden 測試以更新後基準通過

#### Scenario: 生成的 ingest 技能含讀討論與封印指引

- **WHEN** 執行 speclink init 或 speclink update 生成 claude 與 codex 的 ingest 技能
- **THEN** 生成的技能檔內容包含「目標變更帶 from_discussion 時經 speclink discuss show 讀結論併入來源，並於完成時執行 speclink discuss seal」的指示，render golden 測試以更新後基準通過

## ADDED Requirements

### Requirement: 內容落地以 seal 動詞標記已轉出

speclink discuss seal SHALL 接受兩個位置參數（討論 slug 與變更名）。前置守衛 SHALL 全數通過方可寫入：討論 SHALL 存在且未封存、變更 SHALL 存在、且變更 meta 的 from_discussion 清單 SHALL 含該 slug（鏈須先由 link／promote／new change 鑄妥）——任一不滿足時 SHALL 以非零 exit code 結束、stderr 說明原因，且兩側檔案逐位元不變。守衛通過時：討論記錄 frontmatter 的 status SHALL 標記 promoted（由 open 或 concluded 轉入），promoted_to SHALL 以逗號累加該變更名、既有值保留不覆蓋。指令不吃 stdin，旗標僅 --json 與 --no-color。成功時 exit code 0、stdout 單行成功訊息（--no-color 下無 ANSI 色彩）；帶 --json 時輸出含 slug 與 change 欄位（camelCase）的 payload。同一組合重跑 SHALL 為冪等成功：promoted_to 已含該變更名時不改檔、exit code 0。本指令為 Speclink 自有延伸，不在 Spectra 對照範圍。

#### Scenario: 成功封印標記已轉出

- **WHEN** 對一份 concluded 討論執行 speclink discuss seal，且目標變更 meta 的 from_discussion 已含該 slug
- **THEN** exit code 0；討論 frontmatter 變為 status: promoted 且 promoted_to 含該變更名；stdout 單行成功訊息；帶 --json 時 payload 的 slug 與 change 欄位分別為兩參數值

#### Scenario: 鏈未鑄妥守衛拒絕

- **WHEN** 執行 speclink discuss seal 但目標變更 meta 的 from_discussion 不含該 slug
- **THEN** 指令以非零 exit code 結束、stderr 說明鏈未存在；討論記錄與變更 meta 皆逐位元不變

#### Scenario: 重跑封印為冪等

- **WHEN** 對 promoted_to 已含該變更名的討論再次執行 speclink discuss seal
- **THEN** exit code 0；討論記錄逐位元不變

##### Example: seal 守衛一覽

| 情境 | 結果 |
| ---- | ---- |
| slug 無對應討論記錄 | 拒絕：討論不存在 |
| slug 僅存在於 discussions/archive/ | 拒絕：討論已封存 |
| 變更名無對應目錄 | 拒絕：變更不存在 |
| 變更 meta 的 from_discussion 未含該 slug | 拒絕：鏈未鑄妥 |
| promoted_to 已含該變更名 | 冪等成功，不改檔 |

### Requirement: from_discussion 鏈可經 show --json 觀察

speclink show <change> --json 的 payload SHALL 含 fromDiscussions 欄位（camelCase），值為變更 meta from_discussion 逗號清單解析後去除空白的有序字串陣列；變更無 from_discussion 時 SHALL 為空陣列。既有 payload 欄位 SHALL 逐位元不變。

#### Scenario: 有連結時列出討論 slug

- **WHEN** 對 meta 含 from_discussion: alpha-search, beta-cache 的變更執行 speclink show <change> --json
- **THEN** payload 的 fromDiscussions 為 ["alpha-search", "beta-cache"]（順序與 meta 一致）

#### Scenario: 無連結時為空陣列

- **WHEN** 對 meta 無 from_discussion 的變更執行 speclink show <change> --json
- **THEN** payload 的 fromDiscussions 為空陣列 []；其餘欄位不變

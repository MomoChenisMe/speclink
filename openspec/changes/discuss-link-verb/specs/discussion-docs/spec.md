## ADDED Requirements

### Requirement: 討論以 link 動詞併入既有變更

speclink discuss link SHALL 接受兩個位置參數（討論 slug 與既有變更名），將兩側連結：變更 meta 檔（openspec/changes/<change>/.openspec.yaml）SHALL 增寫 from_discussion 欄位指向該討論；討論記錄（openspec/discussions/<slug>.md）的 frontmatter SHALL 標記 status: promoted，且 promoted_to SHALL 以逗號累加該變更名、既有值保留不覆蓋。open 與 concluded 狀態的討論皆 SHALL 可併入；已封存討論 SHALL NOT 可併入。指令不吃 stdin，旗標僅 --json。成功時 exit code 0，stdout 輸出單行成功訊息（含討論 slug 與變更名；--no-color 下無 ANSI 色彩）；帶 --json 時輸出含 slug 與 change 欄位（camelCase）的 payload。守衛失敗（討論不存在、討論已封存、變更不存在、變更已連結其他討論）時 SHALL 以非零 exit code 結束、stderr 說明原因，且兩側檔案 SHALL 逐位元不變。同一組合重跑 SHALL 為冪等成功：exit code 0 且兩側檔案內容不變。併入後，該變更封存且無其他在途變更引用同一討論時，既有自動封存機制 SHALL 將討論記錄移入 openspec/discussions/archive/。本指令為 Speclink 自有延伸，不在 Spectra 對照範圍；既有指令的人眼與 --json 輸出 SHALL 逐位元不變。

#### Scenario: 成功併入既有變更

- **WHEN** 執行 speclink discuss link 並給定一份未封存討論的 slug 與一個尚無 from_discussion 的既有變更名
- **THEN** exit code 0；stdout 單行成功訊息含該 slug 與變更名；openspec/changes/<change>/.openspec.yaml 增寫 from_discussion: <slug>；討論 frontmatter 變為 status: promoted 且 promoted_to 含該變更名；帶 --json 時 payload 的 slug 與 change 欄位分別為兩參數值

#### Scenario: 已轉出其他變更的討論再併入

- **WHEN** 對 promoted_to 已含其他變更名的討論執行 speclink discuss link 指向另一個既有變更
- **THEN** promoted_to 以逗號累加新變更名且既有值保留；exit code 0

#### Scenario: 同一組合重跑為冪等

- **WHEN** 對已互相連結的同一組討論與變更再次執行 speclink discuss link
- **THEN** exit code 0；變更 meta 檔與討論記錄內容逐位元不變

#### Scenario: 守衛拒絕且不落檔

- **WHEN** 執行 speclink discuss link 且命中任一守衛：討論不存在、討論已封存、變更不存在、或變更已連結其他討論
- **THEN** 指令以非零 exit code 結束，stderr 說明原因，openspec/changes/ 與 openspec/discussions/ 下任何檔案逐位元不變

##### Example: 守衛一覽

| 情境 | 結果 |
| ---- | ---- |
| slug 無對應討論記錄 | 拒絕：討論不存在 |
| slug 僅存在於 discussions/archive/ | 拒絕：討論已封存 |
| 變更名無對應目錄 | 拒絕：變更不存在 |
| 變更 meta 已有 from_discussion 指向其他討論 | 拒絕：已連結其他討論 |
| 變更 meta 的 from_discussion 即為本 slug | 冪等成功，不改檔 |

#### Scenario: 併入後隨變更自動封存

- **WHEN** 已 link 的變更執行封存，且無其他在途變更引用同一討論
- **THEN** 討論記錄自動移入 openspec/discussions/archive/，與 promote 型討論的既有封存行為一致

### Requirement: 技能指示引導 ingest 型結論先鑄鏈

speclink 生成的 discuss 技能內容 SHALL 指示 agent：討論結論的 Capture to 指向既有變更時，先執行 speclink discuss link 鑄鏈、再導向 /speclink-ingest 更新該變更的 artifacts。生成的 ingest 技能內容 SHALL 包含來源討論確認提示：更新內容源自某份討論結論時，確認該討論已與目標變更連結。內嵌技能資產、repo 技能實例（claude 與 codex 兩工具）與 render golden 基準 SHALL 同步反映此指示。

#### Scenario: 生成的 discuss 技能含 link 指引

- **WHEN** 執行 speclink init 或 speclink update 生成 claude 與 codex 的 discuss 技能
- **THEN** 生成的技能檔內容包含「Capture to 指向既有變更時先執行 speclink discuss link 再走 /speclink-ingest」的指示，render golden 測試以更新後基準通過

#### Scenario: 生成的 ingest 技能含來源討論提示

- **WHEN** 執行 speclink init 或 speclink update 生成 claude 與 codex 的 ingest 技能
- **THEN** 生成的技能檔內容包含「更新源自討論結論時確認已執行 speclink discuss link」的提示，render golden 測試以更新後基準通過

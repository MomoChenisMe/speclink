## MODIFIED Requirements

### Requirement: 討論以 link 動詞併入既有變更

<!-- BEFORE: 變更已連結其他討論時守衛拒絕（from_discussion 單值）；自動封存僅檢查單一來源討論 -->

speclink discuss link SHALL 接受兩個位置參數（討論 slug 與既有變更名），將兩側連結：變更 meta 檔（openspec/changes/<change>/.openspec.yaml）的 from_discussion 欄位 SHALL 為逗號分隔清單——尚無該欄位時增寫為單值；已指向其他討論時 SHALL 於既有值尾端以逗號累加本 slug、既有值保留不覆蓋；清單已含本 slug 時 SHALL 為冪等成功不改檔。討論記錄（openspec/discussions/<slug>.md）的 frontmatter SHALL 標記 status: promoted，且 promoted_to SHALL 以逗號累加該變更名、既有值保留不覆蓋。open 與 concluded 狀態的討論皆 SHALL 可併入；已封存討論 SHALL NOT 可併入。指令不吃 stdin，旗標僅 --json。成功時 exit code 0，stdout 輸出單行成功訊息（含討論 slug 與變更名；--no-color 下無 ANSI 色彩）；帶 --json 時輸出含 slug 與 change 欄位（camelCase）的 payload。守衛失敗（討論不存在、討論已封存、變更不存在）時 SHALL 以非零 exit code 結束、stderr 說明原因，且兩側檔案 SHALL 逐位元不變。同一組合重跑 SHALL 為冪等成功：exit code 0 且兩側檔案內容不變。變更封存時，其 from_discussion 清單中的每份討論 SHALL 各自檢查：無其他在途變更的 from_discussion 清單引用該討論時，既有自動封存機制 SHALL 將該記錄移入 openspec/discussions/archive/；僅單一來源討論之變更，其封存的人眼輸出 SHALL 與變更前逐位元一致。本指令為 Speclink 自有延伸，不在 Spectra 對照範圍；既有指令的人眼與 --json 輸出 SHALL 逐位元不變。

#### Scenario: 成功併入既有變更

- **WHEN** 執行 speclink discuss link 並給定一份未封存討論的 slug 與一個尚無 from_discussion 的既有變更名
- **THEN** exit code 0；stdout 單行成功訊息含該 slug 與變更名；openspec/changes/<change>/.openspec.yaml 增寫 from_discussion: <slug>；討論 frontmatter 變為 status: promoted 且 promoted_to 含該變更名；帶 --json 時 payload 的 slug 與 change 欄位分別為兩參數值

#### Scenario: 已轉出其他變更的討論再併入

- **WHEN** 對 promoted_to 已含其他變更名的討論執行 speclink discuss link 指向另一個既有變更
- **THEN** promoted_to 以逗號累加新變更名且既有值保留；exit code 0

#### Scenario: 出身自討論的變更再併入新討論

- **WHEN** 對 meta 已有 from_discussion 指向其他討論的變更執行 speclink discuss link 給定另一份討論的 slug
- **THEN** exit code 0；變更 meta 的 from_discussion 於既有值尾端累加本 slug、既有值保留；本討論 frontmatter 變為 status: promoted 且 promoted_to 累加該變更名；先前連結的討論記錄逐位元不變

##### Example: 累加後的 meta 欄位

- **GIVEN** 變更 cut-a 的 meta 含 from_discussion: alpha-search
- **WHEN** 執行 speclink discuss link beta-cache cut-a
- **THEN** cut-a 的 meta 欄位為 from_discussion: alpha-search, beta-cache；alpha-search 的記錄逐位元不變

#### Scenario: 同一組合重跑為冪等

- **WHEN** 對已互相連結的同一組討論與變更再次執行 speclink discuss link（含該討論僅為 from_discussion 清單其中一員的情形）
- **THEN** exit code 0；變更 meta 檔與討論記錄內容逐位元不變

#### Scenario: 守衛拒絕且不落檔

- **WHEN** 執行 speclink discuss link 且命中任一守衛：討論不存在、討論已封存、變更不存在
- **THEN** 指令以非零 exit code 結束，stderr 說明原因，openspec/changes/ 與 openspec/discussions/ 下任何檔案逐位元不變

##### Example: 守衛一覽

| 情境 | 結果 |
| ---- | ---- |
| slug 無對應討論記錄 | 拒絕：討論不存在 |
| slug 僅存在於 discussions/archive/ | 拒絕：討論已封存 |
| 變更名無對應目錄 | 拒絕：變更不存在 |
| 變更 meta 已有 from_discussion 指向其他討論 | 累加：既有值尾端追加本 slug |
| 變更 meta 的 from_discussion 清單已含本 slug | 冪等成功，不改檔 |

#### Scenario: 併入後隨變更自動封存

- **WHEN** 已 link 的變更執行封存，且無其他在途變更引用同一討論
- **THEN** 討論記錄自動移入 openspec/discussions/archive/，與 promote 型討論的既有封存行為一致

#### Scenario: 多來源討論的變更封存逐一共行

- **WHEN** 封存 from_discussion 清單含兩份討論的變更
- **THEN** 清單中每份討論各自檢查存活引用：無其他在途變更引用者移入 openspec/discussions/archive/、人眼輸出逐討論各一行共行訊息；仍被引用者維持在途不動

##### Example: 一份隨行一份留下

- **GIVEN** 變更 cut-a 的 meta 含 from_discussion: alpha-search, beta-cache；另一在途變更 cut-b 的 meta 含 from_discussion: beta-cache
- **WHEN** 執行 speclink archive cut-a
- **THEN** alpha-search 移入 openspec/discussions/archive/、輸出含其共行訊息一行；beta-cache 仍為在途記錄不動

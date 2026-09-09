## MODIFIED Requirements

### Requirement: 討論以 link 動詞併入既有變更
<!-- BEFORE: 變更封存時的隨行封存三條件中，「無其他在途變更引用」只看能解析的 meta；`.openspec.yaml` 解析失敗的在途變更被當作不引用，討論會被掃進封存區 -->

speclink discuss link SHALL 接受兩個位置參數（討論 slug 與既有變更名），鑄造變更側連結：變更 meta 檔（openspec/changes/<change>/.openspec.yaml）的 from_discussion 欄位 SHALL 為逗號分隔清單——尚無該欄位時增寫為單值；已指向其他討論時 SHALL 於既有值尾端以逗號累加本 slug、既有值保留不覆蓋；清單已含本 slug 時 SHALL 為冪等成功不改檔。討論記錄（openspec/discussions/<slug>.md）SHALL 逐位元不變——link SHALL NOT 標記 status: promoted、SHALL NOT 寫 promoted_to；「已轉出」的標記職責移交 speclink discuss seal。open 與 concluded 狀態的討論皆 SHALL 可併入；已封存討論 SHALL NOT 可併入。指令不吃 stdin，旗標僅 --json。成功時 exit code 0，stdout 輸出單行成功訊息（含討論 slug 與變更名；--no-color 下無 ANSI 色彩）；帶 --json 時輸出含 slug 與 change 欄位（camelCase）的 payload。守衛失敗（討論不存在、討論已封存、變更不存在）時 SHALL 以非零 exit code 結束、stderr 說明原因，且兩側檔案 SHALL 逐位元不變。同一組合重跑 SHALL 為冪等成功：exit code 0 且兩側檔案內容不變。變更封存時，其 from_discussion 清單中的每份討論 SHALL 各自檢查三個條件皆成立才隨行封存：無其他在途變更的 from_discussion 清單引用該討論、該討論的 Conclusion 段已寫入內文（scaffold 佔位註解不算內文；判準 SHALL NOT 依 frontmatter status——promoted 討論寫入結論後 status 仍為 promoted）、且該討論的 frontmatter 不帶 `hold: true` 行；三條件皆成立時既有自動封存機制 SHALL 將該記錄移入 openspec/discussions/archive/。Conclusion 未寫入或帶 hold 的討論 SHALL 維持在途、SHALL NOT 隨行封存、SHALL NOT 出現於封存輸出的隨行封存清單，其後 discuss add-round、discuss conclude 與轉出動詞 SHALL 照常可用；討論記錄讀取失敗時 SHALL 視同未寫入結論（留在途，不吞進封存區）。「無其他在途變更引用」的判定 SHALL 對 meta 損壞的在途變更 fail-closed：任一在途變更的 .openspec.yaml 存在但解析失敗時，SHALL 視同該變更仍引用此討論（無論其 from_discussion 實際內容為何），該討論 SHALL 維持在途、SHALL NOT 隨行封存、SHALL NOT 出現於封存輸出的隨行封存清單；變更本身照常封存。此判定與 speclink discuss conclude 閉環步的「壞 meta 視為仍引用」為同一條規則。僅單一來源討論且該討論已有結論、無 hold 之變更，其封存的人眼輸出 SHALL 與變更前逐位元一致。本指令為 Speclink 自有延伸；既有指令的人眼與 --json 輸出 SHALL 逐位元不變。

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

- **WHEN** 已 link 且 Conclusion 段已寫入內文、無 hold 行的討論，其變更執行封存，且無其他在途變更引用同一討論
- **THEN** 討論記錄自動移入 openspec/discussions/archive/，與 promote 型討論的既有封存行為一致

#### Scenario: 未有結論的討論不隨變更封存

- **WHEN** 討論中途轉出（或併入）的變更執行封存，該討論的 Conclusion 段仍為 scaffold 佔位註解、無其他在途變更引用它
- **THEN** 變更照常封存（exit code 0），該討論維持於 openspec/discussions/、SHALL NOT 移入 openspec/discussions/archive/，封存輸出的隨行封存清單不含該討論；其後對該討論執行 discuss add-round 與 discuss conclude 皆照常成功

#### Scenario: 帶 hold 的討論不隨變更封存

- **WHEN** 討論轉出的唯一變更執行封存，該討論 Conclusion 已寫入、frontmatter 帶 `hold: true`、無其他在途變更引用它
- **THEN** 變更照常封存（exit code 0），該討論維持於 openspec/discussions/、SHALL NOT 移入 openspec/discussions/archive/，封存輸出的隨行封存清單不含該討論；其後對該討論執行 speclink discuss promote 照常成功

#### Scenario: 多來源討論的變更封存逐一共行

- **WHEN** 封存 from_discussion 清單含兩份討論的變更
- **THEN** 清單中每份討論各自檢查存活引用、結論與 hold：無其他在途變更引用、已有結論且無 hold 者移入 openspec/discussions/archive/、人眼輸出逐討論各一行共行訊息；仍被引用、未有結論或帶 hold 者維持在途不動

#### Scenario: 在途變更 meta 損壞時討論不隨變更封存

- **WHEN** 封存一個 from_discussion 指向某討論的變更，該討論 Conclusion 已寫入、無 hold 行，且另有一個在途變更的 openspec/changes/<other>/.openspec.yaml 存在但 YAML 解析失敗（其 from_discussion 內容不論）
- **THEN** 本變更照常封存（exit code 0）；該討論維持於 openspec/discussions/、SHALL NOT 移入 openspec/discussions/archive/，封存輸出的隨行封存清單不含該討論；人眼輸出與 --json payload 的其餘部分與變更前逐位元一致；修復該 meta 後再封存任一引用此討論的變更，三條件成立時討論照常隨行封存

##### Example: 壞 meta 的在途變更擋住隨行封存

- **GIVEN** 討論 d1 已有結論、無 hold；變更 cut 的 meta 含 from_discussion: d1，tasks 全數完成；另一在途變更 broken 的 .openspec.yaml 內容為 `schema: [unclosed`（解析失敗）
- **WHEN** 執行 speclink archive cut
- **THEN** exit code 0，cut 移入 openspec/changes/archive/；d1 仍在 openspec/discussions/d1.md；封存輸出不含 d1 的隨行封存行

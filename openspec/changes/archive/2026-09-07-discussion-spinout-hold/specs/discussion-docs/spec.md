## ADDED Requirements

### Requirement: conclude 以 --hold 保留討論在途

speclink discuss conclude SHALL 接受布林旗標 --hold，表示「本討論還欠一個尚未建立的變更」。帶 --hold 時 SHALL 於同一次寫入將結論內文與 frontmatter 的 `hold: true` 行一併落盤（hold 行為 frontmatter 的獨立一行；記錄已有該行時 SHALL NOT 重複），status 的轉換規則（open 轉 concluded、promoted 保持）與待重新反映蓋章 SHALL 與不帶旗標時相同。不帶 --hold 的 conclude SHALL 移除記錄既有的任何 `hold:` 行（含手改的 `hold: false` 等變體）；帶 --hold 而記錄已有 `hold:` 行時 SHALL 原位改寫為 `hold: true`、不留重複鍵。記錄無 frontmatter 而帶 --hold 時 SHALL 以非零 exit code 拒絕、stderr 說明原因、記錄逐位元不變（不帶 --hold 時沿既有路徑照常結論）。帶 --hold 的 conclude SHALL NOT 觸發閉環封存。人眼輸出 SHALL 於既有各行之後多一行告知記錄保留在途（--no-color 下無 ANSI 色彩）；帶 --json 時 payload SHALL 增 held 欄位（camelCase 布林），且僅於本次寫入後記錄帶 hold 時出現；不帶 --hold 且記錄無 hold 行時，人眼與 --json 輸出 SHALL 與本變更前逐位元一致。轉出動詞（speclink discuss promote、speclink new change --from-discussion、speclink discuss seal）累加 promoted_to 時 SHALL 一併移除任何 `hold:` 行（僅於實際累加新變更名時；promoted_to 已含該名的重跑 SHALL 保留旗標）；speclink discuss link SHALL 維持討論記錄逐位元不變（不清旗標）。speclink discuss archive SHALL 無視 hold 照常封存。remote 模式下 --hold SHALL 可用且可觀察行為與本機一致。

#### Scenario: 帶 --hold 的 conclude 寫入旗標且不閉環

- **WHEN** 對 promoted_to 含一個已封存變更名、無在途變更引用的討論執行 speclink discuss conclude --hold 並自 stdin 給定結論
- **THEN** exit code 0；結論寫入且 frontmatter 含 `hold: true` 行、status 保持 promoted；記錄維持於 openspec/discussions/；stdout 於既有各行後多一行告知保留在途；帶 --json 時 payload 含 held: true 且無 autoArchived 鍵

#### Scenario: 不帶 --hold 的 conclude 輸出不變並清除旗標

- **WHEN** 對一份 frontmatter 帶 `hold: true` 的討論執行不帶 --hold 的 speclink discuss conclude；再對一份無 hold 行的 open 討論執行同指令
- **THEN** 前者 exit code 0、hold 行消失、結論改寫；後者人眼與 --json 輸出與本變更前逐位元一致（無 held 鍵）

#### Scenario: 轉出清除旗標

- **WHEN** 對 frontmatter 帶 `hold: true` 的討論執行 speclink discuss promote；對另一份帶 hold 的討論先 link 至既有變更再執行 speclink discuss seal
- **THEN** 兩份記錄的 promoted_to 皆累加對應變更名且 `hold: true` 行消失；link 當下記錄逐位元不變

##### Example: 分期兩刀的生命週期

- **GIVEN** 討論 alpha 的 promoted_to 為 cut-a，結論以 --hold 寫入
- **WHEN** 封存 cut-a；再對 alpha 執行 speclink discuss promote --name cut-b；再封存 cut-b
- **THEN** 封存 cut-a 後 alpha 留在 openspec/discussions/ 且封存輸出不列它；promote 後 alpha 的 promoted_to 為 cut-a, cut-b 且無 hold 行；封存 cut-b 後 alpha 移入 openspec/discussions/archive/

#### Scenario: 無 frontmatter 的記錄拒絕 --hold

- **WHEN** 對一份沒有 frontmatter 的討論記錄執行 speclink discuss conclude --hold；再對同一記錄執行不帶 --hold 的 speclink discuss conclude
- **THEN** 前者以非零 exit code 結束、stderr 說明原因、記錄逐位元不變；後者 exit code 0、結論寫入（沿 pre-scaffold 既有路徑）

#### Scenario: 手動封存無視旗標

- **WHEN** 對 frontmatter 帶 `hold: true` 的討論執行 speclink discuss archive
- **THEN** exit code 0；記錄移入 openspec/discussions/archive/，人眼與 --json 輸出與既有 archive 動詞相同

## MODIFIED Requirements

### Requirement: 討論以 link 動詞併入既有變更

speclink discuss link SHALL 接受兩個位置參數（討論 slug 與既有變更名），鑄造變更側連結：變更 meta 檔（openspec/changes/<change>/.openspec.yaml）的 from_discussion 欄位 SHALL 為逗號分隔清單——尚無該欄位時增寫為單值；已指向其他討論時 SHALL 於既有值尾端以逗號累加本 slug、既有值保留不覆蓋；清單已含本 slug 時 SHALL 為冪等成功不改檔。討論記錄（openspec/discussions/<slug>.md）SHALL 逐位元不變——link SHALL NOT 標記 status: promoted、SHALL NOT 寫 promoted_to；「已轉出」的標記職責移交 speclink discuss seal。open 與 concluded 狀態的討論皆 SHALL 可併入；已封存討論 SHALL NOT 可併入。指令不吃 stdin，旗標僅 --json。成功時 exit code 0，stdout 輸出單行成功訊息（含討論 slug 與變更名；--no-color 下無 ANSI 色彩）；帶 --json 時輸出含 slug 與 change 欄位（camelCase）的 payload。守衛失敗（討論不存在、討論已封存、變更不存在）時 SHALL 以非零 exit code 結束、stderr 說明原因，且兩側檔案 SHALL 逐位元不變。同一組合重跑 SHALL 為冪等成功：exit code 0 且兩側檔案內容不變。變更封存時，其 from_discussion 清單中的每份討論 SHALL 各自檢查三個條件皆成立才隨行封存：無其他在途變更的 from_discussion 清單引用該討論、該討論的 Conclusion 段已寫入內文（scaffold 佔位註解不算內文；判準 SHALL NOT 依 frontmatter status——promoted 討論寫入結論後 status 仍為 promoted）、且該討論的 frontmatter 不帶 `hold: true` 行；三條件皆成立時既有自動封存機制 SHALL 將該記錄移入 openspec/discussions/archive/。Conclusion 未寫入或帶 hold 的討論 SHALL 維持在途、SHALL NOT 隨行封存、SHALL NOT 出現於封存輸出的隨行封存清單，其後 discuss add-round、discuss conclude 與轉出動詞 SHALL 照常可用；討論記錄讀取失敗時 SHALL 視同未寫入結論（留在途，不吞進封存區）。僅單一來源討論且該討論已有結論、無 hold 之變更，其封存的人眼輸出 SHALL 與變更前逐位元一致。本指令為 Speclink 自有延伸；既有指令的人眼與 --json 輸出 SHALL 逐位元不變。

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

### Requirement: conclude 於全數轉出變更已封存時順手封存討論

speclink discuss conclude 寫入結論後 SHALL 檢查閉環條件：討論 frontmatter 的 promoted_to 清單非空、無任何在途變更的 from_discussion 清單引用本討論，且本次寫入後記錄不帶 `hold: true` 行。三條件皆成立時 SHALL 於結論寫入後將討論記錄移入 openspec/discussions/archive/（沿用既有討論封存的檔名與同日撞名解法），stdout 於既有輸出之後 SHALL 多一行告知已順手封存（--no-color 下無 ANSI 色彩），帶 --json 時 payload SHALL 增 autoArchived 欄位（camelCase 布林）且僅於觸發時出現。任一條件不成立時 SHALL NOT 封存，不帶 --hold 時人眼與 --json 輸出 SHALL 與變更前逐位元一致（不出現 autoArchived 鍵）。寫入順序為兩步：先寫結論（含 hold 行的寫入或移除）、再嘗試封存；封存步失敗時結論寫入 SHALL NOT 回滾——可觀察狀態為「已結論、記錄仍在 openspec/discussions/」，指令以非零 exit code 結束、stderr 說明封存步失敗原因，其後執行 speclink discuss archive SHALL 可收尾。此閉環與連帶封存守門互補：conclude 時仍有轉出變更在途則交由最後一個變更封存時隨行封存；帶 hold 的記錄則等待下一次轉出清掉旗標後，由該變更封存時隨行封存。

#### Scenario: 全數轉出變更已封存時 conclude 順手封存

- **WHEN** 對 promoted_to 含一個變更名、該變更已封存、無在途變更引用的討論執行不帶 --hold 的 speclink discuss conclude 寫入結論
- **THEN** exit code 0；結論寫入記錄且 status 保持 promoted；記錄移入 openspec/discussions/archive/；stdout 多一行告知順手封存；帶 --json 時 payload 含 autoArchived: true

#### Scenario: 仍有轉出變更在途時 conclude 不封存

- **WHEN** 對 promoted_to 非空、但仍有一個在途變更的 from_discussion 引用本討論的討論執行 speclink discuss conclude
- **THEN** exit code 0；結論寫入，記錄維持於 openspec/discussions/；人眼與 --json 輸出與本變更前的 conclude 行為逐位元一致，無 autoArchived 鍵

#### Scenario: 帶 --hold 時 conclude 不封存

- **WHEN** 對 promoted_to 含一個已封存變更名、無在途變更引用的討論執行 speclink discuss conclude --hold
- **THEN** exit code 0；結論寫入、frontmatter 含 `hold: true`；記錄維持於 openspec/discussions/；stdout 無順手封存行、有保留在途行；帶 --json 時 payload 含 held: true 且無 autoArchived 鍵

#### Scenario: 未曾轉出的討論 conclude 行為不變

- **WHEN** 對 promoted_to 缺席的 open 討論執行 speclink discuss conclude
- **THEN** exit code 0；status 轉為 concluded，記錄維持在途；人眼與 --json 輸出與本變更前逐位元一致

#### Scenario: 閉環封存步失敗保留結論

- **WHEN** conclude 的閉環條件成立、結論寫入成功、但封存步因儲存層錯誤失敗
- **THEN** 指令以非零 exit code 結束，stderr 說明封存步失敗原因；結論已寫入且不回滾，記錄仍於 openspec/discussions/；其後執行 speclink discuss archive 該 slug 成功收尾

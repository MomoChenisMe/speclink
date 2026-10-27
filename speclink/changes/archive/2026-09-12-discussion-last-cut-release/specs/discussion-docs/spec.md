## MODIFIED Requirements

### Requirement: conclude 以 --hold 保留討論在途
<!-- BEFORE: hold 只由不帶 --hold 的 conclude 或 discuss archive 解除；轉出動詞一律保留旗標；帶 hold 的討論在所有轉出變更封存後維持在途，要手動 archive 收尾 -->

speclink discuss conclude SHALL 接受布林旗標 --hold，表示「本討論還欠至少一個尚未建立的變更」。帶 --hold 時 SHALL 於同一次寫入將結論內文與 frontmatter 的 `hold: true` 行一併落盤（hold 行為 frontmatter 的獨立一行；記錄已有該行時 SHALL NOT 重複），status 的轉換規則（open 轉 concluded、promoted 保持）與待重新反映蓋章 SHALL 與不帶旗標時相同。不帶 --hold 的 conclude SHALL 移除記錄既有的任何 `hold:` 行（含手改的 `hold: false` 等變體）；帶 --hold 而記錄已有 `hold:` 行時 SHALL 原位改寫為 `hold: true`、不留重複鍵。記錄無 frontmatter 而帶 --hold 時 SHALL 以非零 exit code 拒絕、stderr 說明原因、記錄逐位元不變（不帶 --hold 時沿既有路徑照常結論）。帶 --hold 的 conclude SHALL NOT 觸發閉環封存。人眼輸出 SHALL 於既有各行之後多一行告知記錄保留在途（--no-color 下無 ANSI 色彩）；帶 --json 時 payload SHALL 增 held 欄位（camelCase 布林），且僅於本次寫入後記錄帶 hold 時出現；不帶 --hold 且記錄無 hold 行時，人眼與 --json 輸出 SHALL 與本變更前逐位元一致。轉出動詞（speclink discuss promote、speclink new change --from-discussion、speclink discuss seal）SHALL 各接受布林旗標 --last，表示「本次轉出的是結論規劃的最後一刀」；new change 的 --last SHALL 與 --from-discussion 綁定，缺 --from-discussion 時 SHALL 以 exit code 2 拒絕、stderr 說明相依、不建立任何檔案。不帶 --last 的轉出累加 promoted_to 時 SHALL NOT 改動 `hold:` 行——旗標逐字保留，不論累加的是新變更名或已在清單的名字。帶 --last 的轉出 SHALL 於累加 promoted_to 的同一次寫入移除記錄的 `hold:` 行；名字已在清單（重複轉出、re-ingest 的 seal）時 SHALL 同樣移除；記錄無 hold 行時 SHALL 為無害的無操作、記錄逐位元不變。hold SHALL 只由不帶 --hold 的 conclude、speclink discuss archive，或帶 --last 的轉出解除。轉出動詞帶或不帶 --last 的人眼與 --json 輸出 SHALL 與本變更前逐位元一致（轉出不報告 hold）。speclink discuss link SHALL 維持討論記錄逐位元不變、SHALL NOT 接受 --last 以外的旗標變動。speclink discuss archive SHALL 無視 hold 照常封存。帶 hold 而未以 --last 轉出的討論在其所有轉出變更封存後 SHALL 維持在途（隨行封存三條件之「無 hold」不成立），封存輸出的隨行封存清單 SHALL NOT 列它；以 --last 轉出過的討論 SHALL 在其最後一個轉出變更封存時（不論封存順序）由既有隨行封存機制移入 openspec/discussions/archive/，封存輸出列它。remote 模式下 --hold、--last 與轉出保留或解除旗標的可觀察行為 SHALL 與本機一致。

#### Scenario: 帶 --hold 的 conclude 寫入旗標且不閉環

- **WHEN** 對 promoted_to 含一個已封存變更名、無在途變更引用的討論執行 speclink discuss conclude --hold 並自 stdin 給定結論
- **THEN** exit code 0；結論寫入且 frontmatter 含 `hold: true` 行、status 保持 promoted；記錄維持於 openspec/discussions/；stdout 於既有各行後多一行告知保留在途；帶 --json 時 payload 含 held: true 且無 autoArchived 鍵

#### Scenario: 不帶 --hold 的 conclude 輸出不變並清除旗標

- **WHEN** 對一份 frontmatter 帶 `hold: true` 的討論執行不帶 --hold 的 speclink discuss conclude；再對一份無 hold 行的 open 討論執行同指令
- **THEN** 前者 exit code 0、hold 行消失、結論改寫；後者人眼與 --json 輸出與本變更前逐位元一致（無 held 鍵）

#### Scenario: 轉出保留旗標

- **WHEN** 對 frontmatter 帶 `hold: true` 的討論執行不帶 --last 的 speclink discuss promote；對第二份帶 hold 的討論執行不帶 --last 的 speclink new change --from-discussion；對第三份帶 hold 的討論先 link 至既有變更再執行不帶 --last 的 speclink discuss seal
- **THEN** 三份記錄的 promoted_to 皆累加對應變更名、status 為 promoted，且 `hold: true` 行逐字保留；三個動詞的人眼與 --json 輸出與本變更前逐位元一致；link 當下記錄逐位元不變

#### Scenario: 帶 --last 的轉出解除旗標

- **WHEN** 對 frontmatter 帶 `hold: true` 的討論執行 speclink discuss promote --last；對第二份帶 hold 的討論執行 speclink new change --from-discussion --last；對第三份帶 hold 的討論先 link 至既有變更再執行 speclink discuss seal --last；再對一份無 hold 行的討論執行 speclink discuss promote --last
- **THEN** 前三份記錄的 promoted_to 皆累加對應變更名、status 為 promoted，且 `hold:` 行消失、其餘位元組不變；第四份記錄除 promoted_to 累加外逐位元不變；四次動詞的人眼與 --json 輸出與不帶 --last 時逐位元一致

##### Example: 分期三刀的生命週期

- **GIVEN** 討論 alpha 尚未轉出任何變更，結論以 --hold 寫入
- **WHEN** 對 alpha 執行 speclink discuss promote --name cut-a；封存 cut-a；再 promote --name cut-b；封存 cut-b；再 promote --name cut-c --last；封存 cut-c
- **THEN** 三次 promote 後 alpha 的 promoted_to 依序累加 cut-a、cut-b、cut-c；前兩次 promote 後 `hold: true` 行保留，第三次後消失；封存 cut-a 與 cut-b 後 alpha 皆留在 openspec/discussions/ 且封存輸出不列它；封存 cut-c 後 alpha 移入 openspec/discussions/archive/ 且封存輸出列它；全程未執行 speclink discuss archive

#### Scenario: 已在清單的名字帶 --last 仍解除旗標

- **WHEN** 對 promoted_to 已含 cut-c、frontmatter 帶 `hold: true`、cut-c 仍在途的討論執行 speclink discuss seal <slug> cut-c --last
- **THEN** exit code 0；promoted_to 不變、`hold:` 行消失；人眼與 --json 輸出與不帶 --last 的 seal 逐位元一致；其後封存 cut-c 時記錄隨行封存

#### Scenario: new change 的 --last 缺 --from-discussion 被拒

- **WHEN** 執行 speclink new change beta --last 而不帶 --from-discussion
- **THEN** exit code 2；stderr 說明 --last 需要 --from-discussion；openspec/changes/beta 不存在、任何討論記錄逐位元不變

#### Scenario: 無 frontmatter 的記錄拒絕 --hold

- **WHEN** 對一份沒有 frontmatter 的討論記錄執行 speclink discuss conclude --hold；再對同一記錄執行不帶 --hold 的 speclink discuss conclude
- **THEN** 前者以非零 exit code 結束、stderr 說明原因、記錄逐位元不變；後者 exit code 0、結論寫入（沿 pre-scaffold 既有路徑）

#### Scenario: 手動封存無視旗標

- **WHEN** 對 frontmatter 帶 `hold: true` 的討論執行 speclink discuss archive
- **THEN** exit code 0；記錄移入 openspec/discussions/archive/，人眼與 --json 輸出與既有 archive 動詞相同

### Requirement: conclude 於全數轉出變更已封存時順手封存討論
<!-- BEFORE: 結尾敘述寫「帶 hold 的記錄則等待下一次轉出清掉旗標後，由該變更封存時隨行封存」，與 2026-09-10 後轉出不清旗標的行為不符 -->

speclink discuss conclude 寫入結論後 SHALL 檢查閉環條件：討論 frontmatter 的 promoted_to 清單非空、無任何在途變更的 from_discussion 清單引用本討論，且本次寫入後記錄不帶 `hold: true` 行。三條件皆成立時 SHALL 於結論寫入後將討論記錄移入 openspec/discussions/archive/（沿用既有討論封存的檔名與同日撞名解法），stdout 於既有輸出之後 SHALL 多一行告知已順手封存（--no-color 下無 ANSI 色彩），帶 --json 時 payload SHALL 增 autoArchived 欄位（camelCase 布林）且僅於觸發時出現。任一條件不成立時 SHALL NOT 封存，不帶 --hold 時人眼與 --json 輸出 SHALL 與變更前逐位元一致（不出現 autoArchived 鍵）。寫入順序為兩步：先寫結論（含 hold 行的寫入或移除）、再嘗試封存；封存步失敗時結論寫入 SHALL NOT 回滾——可觀察狀態為「已結論、記錄仍在 openspec/discussions/」，指令以非零 exit code 結束、stderr 說明封存步失敗原因，其後執行 speclink discuss archive SHALL 可收尾。此閉環與連帶封存守門互補：conclude 時仍有轉出變更在途則交由最後一個變更封存時隨行封存；帶 hold 的記錄則等待帶 --last 的轉出清掉旗標後，由最後一個轉出變更封存時隨行封存，或由使用者以 speclink discuss archive 收尾。

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

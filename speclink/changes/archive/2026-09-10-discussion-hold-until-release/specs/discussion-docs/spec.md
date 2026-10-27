## MODIFIED Requirements

### Requirement: conclude 以 --hold 保留討論在途

speclink discuss conclude SHALL 接受布林旗標 --hold，表示「本討論還欠至少一個尚未建立的變更」。帶 --hold 時 SHALL 於同一次寫入將結論內文與 frontmatter 的 `hold: true` 行一併落盤（hold 行為 frontmatter 的獨立一行；記錄已有該行時 SHALL NOT 重複），status 的轉換規則（open 轉 concluded、promoted 保持）與待重新反映蓋章 SHALL 與不帶旗標時相同。不帶 --hold 的 conclude SHALL 移除記錄既有的任何 `hold:` 行（含手改的 `hold: false` 等變體）；帶 --hold 而記錄已有 `hold:` 行時 SHALL 原位改寫為 `hold: true`、不留重複鍵。記錄無 frontmatter 而帶 --hold 時 SHALL 以非零 exit code 拒絕、stderr 說明原因、記錄逐位元不變（不帶 --hold 時沿既有路徑照常結論）。帶 --hold 的 conclude SHALL NOT 觸發閉環封存。人眼輸出 SHALL 於既有各行之後多一行告知記錄保留在途（--no-color 下無 ANSI 色彩）；帶 --json 時 payload SHALL 增 held 欄位（camelCase 布林），且僅於本次寫入後記錄帶 hold 時出現；不帶 --hold 且記錄無 hold 行時，人眼與 --json 輸出 SHALL 與本變更前逐位元一致。轉出動詞（speclink discuss promote、speclink new change --from-discussion、speclink discuss seal）累加 promoted_to 時 SHALL NOT 改動 `hold:` 行——旗標逐字保留，不論累加的是新變更名或已在清單的名字；hold SHALL 只由不帶 --hold 的 conclude 或 speclink discuss archive 解除（此為刻意變更：本變更前轉出新名字會清旗標，多刀系列在轉出第一刀時旗標即失效）。轉出動詞的人眼與 --json 輸出 SHALL 與本變更前逐位元一致。speclink discuss link SHALL 維持討論記錄逐位元不變。speclink discuss archive SHALL 無視 hold 照常封存。帶 hold 的討論在其所有轉出變更封存後 SHALL 維持在途（隨行封存三條件之「無 hold」不成立），封存輸出的隨行封存清單 SHALL NOT 列它。remote 模式下 --hold 與轉出保留旗標的可觀察行為 SHALL 與本機一致。

#### Scenario: 帶 --hold 的 conclude 寫入旗標且不閉環

- **WHEN** 對 promoted_to 含一個已封存變更名、無在途變更引用的討論執行 speclink discuss conclude --hold 並自 stdin 給定結論
- **THEN** exit code 0；結論寫入且 frontmatter 含 `hold: true` 行、status 保持 promoted；記錄維持於 openspec/discussions/；stdout 於既有各行後多一行告知保留在途；帶 --json 時 payload 含 held: true 且無 autoArchived 鍵

#### Scenario: 不帶 --hold 的 conclude 輸出不變並清除旗標

- **WHEN** 對一份 frontmatter 帶 `hold: true` 的討論執行不帶 --hold 的 speclink discuss conclude；再對一份無 hold 行的 open 討論執行同指令
- **THEN** 前者 exit code 0、hold 行消失、結論改寫；後者人眼與 --json 輸出與本變更前逐位元一致（無 held 鍵）

<!-- REMOVED-SCENARIO: 轉出清除旗標 -->

#### Scenario: 轉出保留旗標

- **WHEN** 對 frontmatter 帶 `hold: true` 的討論執行 speclink discuss promote；對第二份帶 hold 的討論執行 speclink new change --from-discussion；對第三份帶 hold 的討論先 link 至既有變更再執行 speclink discuss seal
- **THEN** 三份記錄的 promoted_to 皆累加對應變更名、status 為 promoted，且 `hold: true` 行逐字保留；三個動詞的人眼與 --json 輸出與本變更前逐位元一致；link 當下記錄逐位元不變

##### Example: 分期三刀的生命週期

- **GIVEN** 討論 alpha 尚未轉出任何變更，結論以 --hold 寫入
- **WHEN** 對 alpha 執行 speclink discuss promote --name cut-a；封存 cut-a；再 promote --name cut-b；封存 cut-b；再 promote --name cut-c；封存 cut-c；最後執行 speclink discuss archive alpha
- **THEN** 三次 promote 後 alpha 的 promoted_to 依序累加 cut-a、cut-b、cut-c 且 `hold: true` 行全程保留；三次封存後 alpha 皆留在 openspec/discussions/ 且封存輸出不列它；discuss archive 後 alpha 移入 openspec/discussions/archive/

#### Scenario: 無 frontmatter 的記錄拒絕 --hold

- **WHEN** 對一份沒有 frontmatter 的討論記錄執行 speclink discuss conclude --hold；再對同一記錄執行不帶 --hold 的 speclink discuss conclude
- **THEN** 前者以非零 exit code 結束、stderr 說明原因、記錄逐位元不變；後者 exit code 0、結論寫入（沿 pre-scaffold 既有路徑）

#### Scenario: 手動封存無視旗標

- **WHEN** 對 frontmatter 帶 `hold: true` 的討論執行 speclink discuss archive
- **THEN** exit code 0；記錄移入 openspec/discussions/archive/，人眼與 --json 輸出與既有 archive 動詞相同

## ADDED Requirements

### Requirement: 打包好的主套件通過使用端檢查

engine 的 build/pack workflow SHALL 在打包之後、上傳 tarball 之前，把打包好的主套件當成使用端的相依來檢查；這個檢查 SHALL 對 push、PR 與 release 三種觸發一律生效。檢查的對象 SHALL 是打包好的 tarball——主套件，加上執行檢查的機器所屬平台的子套件——SHALL NOT 以 repo 內的原始檔代替。

檢查 SHALL 涵蓋三件事：

1. 主套件能被 Node 載入，而且載入後至少有一個執行期匯出的名稱。
2. 主套件的型別定義檔能被 TypeScript 編譯器編譯：語法錯誤與型別錯誤都算失敗，且 SHALL NOT 以略過函式庫檢查的選項放寬。
3. 主套件執行期匯出的每一個名稱，都能從型別定義檔具名匯入。名稱不是合法的識別字時（例如 `default` 或含連字號的名稱）SHALL 照樣比對；檢查 SHALL NOT 因為名稱的寫法本身而回報失敗。

任一項不成立時，檢查 SHALL 以非零結束，並於 stderr 說明是哪一項與原因（編譯失敗時含編譯器的錯誤訊息）；workflow SHALL NOT 產出可發佈的 tarball artifact。三項都成立時檢查 SHALL 以 0 結束、不改動任何 tarball，後續的上傳照常進行。

主套件載入後沒有任何執行期匯出的名稱時，第三項沒有東西可以比對，檢查 SHALL 視為第一項不成立，SHALL NOT 以 0 結束。

檢查 SHALL NOT 從 registry 下載被檢查的主套件或它的平台子套件：release 觸發時 tarball 已經蓋上 tag 的版號，而那個版號當下還沒有發佈。

#### Scenario: 型別定義檔的註解提早結束時打包失敗

- **WHEN** 主套件的型別定義檔有一段區塊註解，註解裡的範例含 `'*/*'`，使註解在那裡結束
- **THEN** 檢查以非零結束，stderr 含編譯器回報的語法錯誤（錯誤代碼如 TS1131）與型別定義檔的檔名，workflow 不上傳 npm-tarballs artifact

#### Scenario: 執行期匯出沒有型別宣告時失敗

- **WHEN** 主套件執行期匯出 `commandNames`，而型別定義檔沒有宣告它
- **THEN** 檢查以非零結束，stderr 的編譯器訊息點名 `commandNames`

#### Scenario: 套件載入不了時失敗

- **WHEN** 打包好的主套件在載入時拋出錯誤（例如它要載入的檔案不在 tarball 裡）
- **THEN** 檢查以非零結束，stderr 說明是載入失敗並帶出 Node 的錯誤訊息；編譯那一步不執行

#### Scenario: 載入成功但沒有任何匯出時失敗

- **WHEN** 打包好的主套件能被載入，但執行期沒有任何匯出的名稱（例如 index.js 匯出的是空物件，或匯出的是一個沒有屬性的函式）
- **THEN** 檢查以非零結束，stderr 說明主套件沒有任何執行期匯出的名稱可以比對；編譯那一步不執行

#### Scenario: 匯出的名稱不是合法識別字時照樣比對

- **WHEN** 主套件執行期匯出 `a-b` 與 `default`，型別定義檔宣告了 `a-b`，其餘的匯出也都有宣告
- **THEN** 檢查以 0 結束，結論的匯出名稱個數包含 `a-b` 與 `default`

#### Scenario: 不是合法識別字的匯出沒有型別宣告時失敗

- **WHEN** 主套件執行期匯出 `x-y`，而型別定義檔沒有宣告它
- **THEN** 檢查以非零結束，stderr 的編譯器訊息點名 `x-y`，而且回報的是「沒有這個匯出的成員」，不是語法錯誤

#### Scenario: 完好的套件通過

- **WHEN** 主套件能載入、型別定義檔沒有錯誤、每個執行期匯出都有型別宣告
- **THEN** 檢查以 0 結束，主套件與平台子套件的 tarball 內容逐位元不變，npm-tarballs artifact 照常上傳

#### Scenario: release 觸發時不需要 registry 上有該版號

- **WHEN** release 管線以 version 輸入 0.13.1 呼叫 build/pack workflow，蓋章後主套件的 optionalDependencies 釘住 0.13.1 的五個平台子套件，而 registry 上還沒有 0.13.1
- **THEN** 檢查照常執行，結論與同一份內容未蓋章時相同

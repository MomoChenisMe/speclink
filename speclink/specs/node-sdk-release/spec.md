# node-sdk-release Specification

## Purpose

`@speclink/engine` 的 npm 發布通路：五平台 build/pack workflow 的可重用邊界、release tag 版號蓋章與平台子套件物化，以及 npm 發布的閘門與順序。本 capability 保證發布產物版本一律與 release tag 對齊——repo 內版號只是佔位符，不會外流成發布版本。

## Requirements

### Requirement: engine npm 套件家族與版號蓋章

專案 SHALL 提供 `@speclink/engine` 的 npm 通路：主套件（index.js、index.d.ts、binding.js）與五個平台子套件（Windows x64、macOS x64／arm64、Linux glibc x64／arm64），主套件以 optionalDependencies 指向各平台子套件，安裝時只下載符合平台者。發布產物的版本 SHALL 與 release tag 對齊，由發布管線於打包前蓋章寫入主套件與全部平台子套件——repo 內 `crates/adapters/speclink-node/package.json` 的版號 SHALL 視為佔位符，SHALL NOT 作為發布版號來源。蓋章 SHALL 同時把主套件 optionalDependencies 物化為各平台子套件的同版釘定，且平台子套件清單 SHALL 自平台子套件目錄實際內容列舉而非硬編碼。

#### Scenario: 蓋章物化主套件與平台子套件

- **WHEN** 對含五個平台子套件目錄的套件樹執行版號蓋章（如 0.2.0）
- **THEN** 主套件與每個平台子套件的 version 皆為 0.2.0，且主套件 optionalDependencies 恰為五個平台子套件名各釘 0.2.0

#### Scenario: 佔位版號不外流

- **WHEN** 以 0.2.0 蓋章打包後檢視主套件 tarball 內的 package.json
- **THEN** version 為 0.2.0，而非 repo 內佔位的 0.1.0


<!-- @trace
source: engine-npm-publish
updated: 2026-08-25
-->

---
### Requirement: build/pack workflow 可被 release 管線重用

engine 的五平台建置與打包 workflow SHALL 同時支援 push／PR 觸發與 `workflow_call` 呼叫：`workflow_call` SHALL 接受選填的版號輸入，有值時於打包前執行版號蓋章、無值時維持佔位版號打包（push／PR 情境的**版號行為** SHALL 不變）。打包後的產物完整性檢查 SHALL 對所有觸發一律生效——主套件 tarball SHALL 含平台載入器、每個平台子套件 tarball SHALL 含其平台二進位，任一缺漏 SHALL 以非零結束；這些檔案不在 repo 內、由建置產生，缺漏時打包仍會成功，因此檢查是唯一的攔截點。release 管線 SHALL 經 `workflow_call` 於同一 workflow run 內重用此建置，SHALL NOT 另行複製五平台 build matrix。

#### Scenario: PR 觸發不受版號蓋章影響

- **WHEN** pull request 觸發 build/pack workflow（無 version 輸入）
- **THEN** 五平台 tarball 以 repo 內佔位版號（如 0.1.0）打包上傳，版號行為與導入 workflow_call 前一致

#### Scenario: 產物缺漏時打包失敗

- **WHEN** 打包完成，但主套件 tarball 內沒有平台載入器，或某個平台子套件 tarball 內沒有 `.node` 二進位
- **THEN** 該步驟以非零結束並點名缺漏的 tarball，不產出可發布的 artifact（push／PR 與 release 觸發皆同）

#### Scenario: release 帶版號呼叫得到蓋章 tarball

- **WHEN** release 管線因 tag v0.2.0 觸發，以 version 輸入 0.2.0 呼叫 build/pack workflow
- **THEN** 產出的主套件與五個平台子套件 tarball 版本皆為 0.2.0，並以 npm-tarballs artifact 供同 run 的後續 job 下載


<!-- @trace
source: engine-npm-publish
updated: 2026-08-25
-->

---
### Requirement: npm 發布閘門與發布順序

engine 的 npm 發布 SHALL 由 release 管線在 NPM_TOKEN 存在時執行（`npm publish --access public`），缺席時 SHALL 跳過發布步驟且 job 結果為成功、SHALL NOT 影響 Release 其餘產物；token 存在而發布失敗時 SHALL 以紅燈呈現且可單獨重跑。發布順序 SHALL 為平台子套件先發、主套件最後發，使主套件的 optionalDependencies 於發布完成時皆可解析到同版。發布單位 SHALL 為 build/pack workflow 產出的 tarball 檔，發布 job SHALL NOT 重新打包。

版號閘門 SHALL 在建置前先行：tag 版號不符 `X.Y.Z` 時 SHALL 略過 engine 的建置與發布，且 SHALL NOT 影響 Release 其餘產物——蓋章只收 `X.Y.Z`，留到打包才擋等於白跑一輪五平台建置。發布前 SHALL 逐份斷言 tarball 內的版號等於 tag 版號，不符即以非零結束：npm 發布不可撤回，蓋章一旦沒跑就會把佔位版號永久上架，而重跑會因該版號已存在而全部略過。

#### Scenario: 無 token 時 Release 不受影響

- **WHEN** 在未設定 NPM_TOKEN 的 repo 推送 release tag（如 v0.2.0）
- **THEN** engine 的 npm 發布各步跳過、job 結果為成功，GitHub Release 與 Docker 等其餘產物照常發布

#### Scenario: 主套件最後發布

- **WHEN** NPM_TOKEN 存在且五個平台子套件（如 @speclink/engine-darwin-arm64）的 tarball 皆發布成功
- **THEN** 主套件 @speclink/engine 的 tarball 最後發布，其 optionalDependencies 指向的每個平台子套件皆已存在於 registry 且與主套件同版（如 0.2.0）

#### Scenario: prerelease tag 不進入 engine 建置

- **WHEN** 推送不符 `X.Y.Z` 的 release tag（如 v1.0.0-rc.1）
- **THEN** engine 的建置與發布整條略過（不是紅燈），五平台 matrix 不啟動，GitHub Release、Docker 與桌面安裝檔照常產出

#### Scenario: 版號未蓋章時拒絕發布

- **WHEN** NPM_TOKEN 存在，但取到的 tarball 內版號為 repo 佔位版（如 0.1.0）而 tag 版為 0.2.0
- **THEN** 發布 job 以非零結束並點名不符的套件與兩個版號，registry 上不新增任何 engine 套件版本

<!-- @trace
source: engine-npm-publish
updated: 2026-08-25
-->

---
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

<!-- @trace
source: sdk-types-parse-check
updated: 2026-11-20T11:38:52+08:00
-->
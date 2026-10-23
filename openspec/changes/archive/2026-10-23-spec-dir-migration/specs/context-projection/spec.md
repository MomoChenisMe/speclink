## MODIFIED Requirements

### Requirement: 投影佈局與 manifest

<!-- BEFORE: 鏡像佈局為 openspec/，CLI 寫死以 openspec 為鏡像根 -->

remote 模式的 Context Projection SHALL 位於 workspace 的 speclink 工作目錄下 context 子目錄，含 manifest.json（snapshot ID、policy revision（有值時）、逐文件 digest 與 revision，欄位 camelCase——即 protocol ContextSnapshot 既有欄位）、INDEX.md 與 speclink 鏡像佈局（config、LANGUAGE、canonical specs、change 文件）。server 產生的快照文件路徑 SHALL 以 `speclink/` 開頭。client 指向投影的路徑（例如 instructions 的 contextFiles）SHALL 以快照文件路徑的第一段決定鏡像根目錄，SHALL NOT 寫死資料夾名，因此快照路徑以 `openspec/` 開頭的舊 server 也照常可讀。投影 SHALL 可隨時整目錄刪除並重建；本地 fs 模式 SHALL NOT 建立投影。

#### Scenario: materialize 產出完整佈局

- **WHEN** 以測試 snapshot provider 對含一個 change 與兩個 canonical specs 的 scope 執行 materialize
- **THEN** 投影含 manifest.json（digest 逐文件齊備）、INDEX.md 與對應鏡像文件；刪除整目錄後重新 materialize 得到等價內容

#### Scenario: 新 server 的鏡像根為 speclink

- **WHEN** 遠端模式對 server 執行 `speclink instructions apply --change demo --json`，server 的快照路徑以 `speclink/` 開頭
- **THEN** 投影目錄下有 `speclink/changes/demo/` 的文件，payload 的 `contextFiles` 指向這些檔案

#### Scenario: 舊 server 的投影照常可讀

- **WHEN** 遠端模式的 server 為舊版、快照路徑以 `openspec/` 開頭，執行 `speclink instructions apply --change demo --json`
- **THEN** exit code 為 0，`contextFiles` 指向投影目錄下 `openspec/changes/demo/` 中實際存在的檔案

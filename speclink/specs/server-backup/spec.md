# server-backup Specification

## Purpose

Speclink server 的備份與還原：備份檔自描述且逐項可驗證、verify-backup 攔截竄改與未知格式，以及 restore 只進空目標並把驗證當成還原的一部分。本 capability 保證還原不會覆蓋既有資料、竄改過的備份不會被吃下去，且 admin 的資料操作一律入 audit。

## Requirements

### Requirement: 備份檔自描述且逐項可驗證

backup 子命令 SHALL 產生單一備份檔，內含：manifest（備份格式版本、UTC 建立時間、engine/API 版本、儲存的種類、identity schema version、scope 清單）、每個 registry scope 的 export bundle（經宿主套件的範圍匯出產生，以鍵標示每份文件，SHALL NOT 直接拷貝儲存的資料庫檔或資料夾）、identity 資料庫時點一致快照，與覆蓋每個成員的 digest。這個版本產生的備份 SHALL 宣告備份格式版本 2。備份 SHALL NOT 含任何憑證明文。備份 SHALL 於無寫入的條件下執行（server 未運行或部署層維護窗口），SHALL NOT 宣稱寫入中快照的一致性。

#### Scenario: 備份內容完備

- **WHEN** 對含兩個 scope、若干使用者與 audit 記錄的 server 執行 backup 子命令
- **THEN** 備份檔含 manifest、兩個 scope 的 bundle 與 identity 快照，逐成員 digest 齊全；manifest 的備份格式版本是 2 並記有儲存的種類；bundle 裡每份文件以鍵標示；檔內不存在任何 PAT/密碼/token 明文


<!-- @trace
source: server-on-port-examples
updated: 2026-11-16T23:24:33+08:00
-->

---
### Requirement: verify-backup 攔截竄改與未知格式

verify-backup 子命令 SHALL 只讀備份檔完成驗證：manifest 與逐成員 digest 比對、bundle 結構可解析、備份格式版本已知——全數通過回 0；任一成員 digest 不符、結構不可解析或格式版本未知 SHALL 回非零 exit code 並指出不符成員。已知的備份格式版本 SHALL 是 1（0.12.0 以前的版本產生，bundle 以文件種類標示文件）與 2（以鍵標示文件）。restore SHALL 以同一驗證為第一步。

#### Scenario: 竄改一位元即拒絕

- **WHEN** 修改備份檔內任一成員的一個位元後執行 verify-backup 與 restore
- **THEN** 兩者皆非零 exit code 且指出 digest 不符的成員；restore 未寫入目標任何內容

#### Scenario: 未知格式版本拒絕

- **WHEN** 對 manifest 宣告未知備份格式版本的備份檔執行 restore
- **THEN** 非零 exit code 且原因指出版本不相容；目標不被寫入

#### Scenario: 舊版的備份檔通過檢查

- **WHEN** 對 0.12.0 產生的備份檔（備份格式版本 1）執行 verify-backup
- **THEN** exit code 為 0；報告的備份格式版本是 1


<!-- @trace
source: server-on-port-examples
updated: 2026-11-16T23:24:33+08:00
-->

---
### Requirement: restore 只進空目標且驗證即還原的一部分

restore 子命令 SHALL 要求目標 store 與 identity 皆空，非空 SHALL 拒絕並輸出既有內容摘要，SHALL NOT 提供覆蓋既有資料的旗標。還原 SHALL 依序：備份檔完整性驗證、identity 快照落位、逐 scope import、restore validation——逐 scope 重讀比對 bundle digest 與文件數、identity 的使用者/registry/audit 計數與 schema version 對 manifest 比對。驗證 SHALL 輸出逐項報告；任一不符 SHALL 回非零 exit code、明示差異項並明示目標不可投產。

逐 scope import SHALL 經宿主套件的範圍匯入完成：每個 scope 一次提交，只進空的範圍。備份格式版本 1 的 bundle SHALL 先把每份文件的種類換成鍵再匯入；這個轉換 SHALL 在 identity 快照落位之前對全部 scope 完成，任何一份文件換不成鍵時 SHALL 以非零 exit code 結束並指出該 scope 與文件，目標 SHALL NOT 被寫入。

#### Scenario: 災難演練閉環

- **WHEN** 對播種完整的 server（changes、討論、成員、PAT、audit）備份後，於全新空目標 restore 並啟動 server
- **THEN** restore validation 全綠；成員以備份前的 PAT 照常通行；CLI 查詢輸出與備份前一致；audit 歷史完整在列

#### Scenario: 非空目標拒絕

- **WHEN** 對已含任一 scope 文件或任一使用者的目標執行 restore
- **THEN** 非零 exit code；輸出既有內容摘要；目標內容位元不變

#### Scenario: 舊版的備份還原到這個版本

- **WHEN** 把 0.12.0 產生的備份檔（備份格式版本 1，含進行中的變更、正式規格、討論、已封存的變更、流程設定、evidence 與看板順序）restore 到這個版本的全新空目標並啟動 server
- **THEN** restore validation 全綠；成員以備份前的 PAT 照常通行；CLI 的 list、show 與 plan 輸出的變更、規格與順序與備份前一致

#### Scenario: 舊版的備份裡有換不成鍵的文件

- **WHEN** 備份格式版本 1 的某個 bundle 含一份種類不認得的文件，執行 restore
- **THEN** 非零 exit code，訊息指出該 scope 與文件；identity 與 store 都沒有被寫入


<!-- @trace
source: server-on-port-examples
updated: 2026-11-16T23:24:33+08:00
-->

---
### Requirement: admin 資料操作入 audit

admin 面 SHALL 提供：scope export 下載（經宿主套件的範圍匯出即時產生以鍵標示文件的 bundle）與備份資訊檢視（最近備份與驗證的 manifest 摘要與結果）。兩者 SHALL 沿用 admin 門禁並各記 audit（動作種類：scope-exported、backup-recorded）；未知 scope 的 export 下載 SHALL 回 404。admin 面 SHALL NOT 提供資料結構遷移：管理 browser API 的 data/migrate 路由 SHALL 回 404，server SHALL NOT 再產生 store-migrated 的 audit 記錄；既有的 store-migrated 記錄 SHALL 照常列在稽核頁。

#### Scenario: export 下載可還原驗證

- **WHEN** admin 下載某 scope 的 export bundle 並對其執行結構與 digest 驗證
- **THEN** bundle 可解析、每份文件以鍵標示且 digest 與 store 內容一致；audit 含一筆 scope-exported 記錄

#### Scenario: 遷移的路由不存在

- **WHEN** admin 對管理 browser API 的 data/migrate 路由送出 POST
- **THEN** 回 404；audit 沒有新增 store-migrated 記錄


<!-- @trace
source: server-on-port-examples
updated: 2026-11-16T23:24:33+08:00
-->
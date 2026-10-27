## Why

roadmap §5 的 Phase 2 gate 明列「backup/export/restore validation 經端到端演練」——這是宣稱 server 可供團隊正式使用前的最後一個非 driver gate（藍圖 §13.4 的最低配備清單也含 backup/restore）。地基早已就緒但沒有營運介面：TeamStore 契約的 export/import（Bundle 往返、digest、ImportMode/ImportReport）從 teamstore-contract-v2 就定案並被 conformance 覆蓋，SQLite driver 也已通過——但運維者今天要備份只能手動抄資料庫檔，沒有一致性保證（identity 庫與 store 庫兩個檔）、沒有還原驗證、災難演練無從談起。本刀把契約方法接成營運能力：一個命令產生完整備份、一個命令還原並驗證、admin 面可下載 scope export 與檢視備份資訊。

目標使用者：server 運維者（例行備份與災難還原）與 Admin（scope 級 export 下載、遷移或歸檔單一 project 資料）。

## What Changes

- 新增 server CLI backup 子命令：對未運行的資料（server 停止或部署層維護窗口）產生單一備份檔——內含全部 registry scopes 的 export bundles（走 TeamStore export 契約）、identity 資料庫快照、manifest 資訊（schema versions、driver、建立時間）與逐項 digest。備份不含任何憑證明文（identity 庫本就只存 hash）。
- 新增 restore 子命令：只還原到空目標（store 與 identity 皆空，非空即拒絕）；還原後自動執行 restore validation——逐 scope 重讀比對 bundle digest、文件數與 revision、identity schema version 與使用者/registry 計數，輸出驗證報告，任何不符即非零 exit code 並明示差異。另有獨立 verify-backup 子命令：不還原、只驗證備份檔完整性（digest 與結構）。
- admin 面補齊 §13.2 清單的資料操作項：scope export 下載（bundle 檔）、備份資訊檢視（最近備份的 manifest 與驗證結果，若有）、store migration 觸發（TeamStore migrate 契約的營運入口，含前置 health 檢查與 audit 記錄）；此三者皆記 audit。
- 端到端災難演練測試：播種（changes、討論、使用者、PAT、audit）→ backup → 全新環境 restore → validation 全綠 → 真實 CLI 對還原後 server 執行動詞且查詢輸出與備份前一致、既有 PAT 照常通行；另含竄改測試——備份檔任一位元被改，verify-backup 與 restore 皆拒絕。

## Capabilities

### New Capabilities

- `server-backup`: 備份、還原與驗證——備份檔內容與 digest、空目標還原、restore validation 報告、verify-backup、admin 面的 export 下載與 migration 觸發。

### Modified Capabilities

(none)

## Impact

- 相容性影響：純新增子命令與 admin 功能；既有路由、認證、事件與 CLI client 零變更；parity 31 項、color 16 項、twin 8 情境凍結不動。備份檔格式自帶版本標記，屬本刀新定義的產物格式（fail closed：未知版本拒絕還原）。前置依賴：server-setup-registry（registry 在庫）與 server-admin-audit（admin 門禁與 audit 基座）。
- Affected specs: `server-backup`（新增）
- Affected code:
  - New: crates/speclink-server/src/backup.rs、crates/speclink-server/tests/backup_restore.rs
  - Modified: crates/speclink-server/src/main.rs、crates/speclink-server/src/admin.rs、crates/speclink-server/src/identity.rs、crates/speclink-server/src/identity_sqlite.rs、crates/speclink-server/src/web.rs
  - Removed: 無

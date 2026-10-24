## MODIFIED Requirements

### Requirement: 遷移成功後才轉換本地且一律備份

<!-- BEFORE: 本地 openspec/ 改名為帶日期的備份目錄 -->

遷移流程 SHALL 於 import 成功後才觸碰本地：本機規格資料夾 SHALL 改名為帶日期的備份目錄 `<資料夾名>.migrated-<日期>`（例如 `speclink.migrated-2026-11-02`；舊佈局為 `openspec.migrated-2026-11-02`；同名既存則加序號）——SHALL NOT 存在刪除本地內容的路徑；隨後寫入 remote marker 並原地將分頁轉為 remote（checkoutRoot 為該資料夾）。import 失敗（scope 非空、網路、驗證）時本地 SHALL 分毫不動且 UI 原樣呈現錯誤。遷移確認對話 SHALL 指出目標 Project/Repo 與本地備份行為。

#### Scenario: 失敗零副作用

- **WHEN** 遷移於 import 步驟因 scope 非空失敗
- **THEN** 本地規格資料夾原樣存在、無 marker 寫入、分頁維持 local，錯誤原樣呈現

#### Scenario: 成功後資料夾轉為 checkout

- **WHEN** 遷移成功完成
- **THEN** 資料夾含帶日期的備份目錄與 remote marker、原分頁原地轉為 remote 分頁且 checkoutRoot 為該資料夾，CLI 於該資料夾以 remote 模式運作

#### Scenario: 備份目錄依實際資料夾命名

- **WHEN** 規格資料夾為 speclink/ 的專案在 2026-11-02 遷移成功，資料夾內已有 speclink.migrated-2026-11-02
- **THEN** 本機規格資料夾改名為 speclink.migrated-2026-11-02-2

## ADDED Requirements

### Requirement: 新舊預設資料夾都可遷移到遠端

本機遷移到遠端（含採用伺服器內容）SHALL 接受規格資料夾為預設 `speclink/` 或舊佈局 `openspec/` 的專案；只有 `.speclink.yaml` 明寫的自訂 `spec_dir`，以及 `speclink/` 與 `openspec/` 並存的專案（只備份其一會留下另一個）SHALL 拒絕並說明原因。組裝上傳內容時，已封存變更的資料檔（`.meta.yaml`，或舊檔名 `.openspec.yaml`）SHALL 一律以 `.meta.yaml` 為鍵上傳；同一個已封存變更兩者並存時 SHALL 只上傳 `.meta.yaml` 的內容。

#### Scenario: 舊佈局專案可遷移

- **WHEN** 仍是 openspec/、沒有明寫 spec_dir 的專案執行本機遷移到遠端且 import 成功
- **THEN** 遷移完成，備份目錄為 `openspec.migrated-<日期>`，遠端的已封存變更資料檔鍵皆為 `.meta.yaml`

#### Scenario: 明寫自訂資料夾仍拒絕

- **WHEN** `.speclink.yaml` 含 `spec_dir: docs/specs` 的專案執行本機遷移到遠端
- **THEN** 遷移被拒絕，訊息說明只支援預設資料夾，本地不變、遠端沒有任何寫入

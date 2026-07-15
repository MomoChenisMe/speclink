## Context

TeamStore 契約提供 export(scope) 到 Bundle（逐文件內容與 digest）與 import(bundle, mode) 回 ImportReport；digest 是契約固定的 SHA-256（speclink-store 的 content_digest），conformance 已覆蓋 bundle 往返。server 有兩份持久資料：store 資料庫（TeamStore driver）與 identity 資料庫（users/memberships/invitations/PATs/sessions/device/registry/audit，schema version 守門）。admin 門禁、audit 單點寫入與 /admin 頁面組在 server-admin-audit 刀就緒；registry 在庫（server-setup-registry 刀），全部 scopes 可自 registry 列舉。TeamStore 契約另有 migrate(target_version) 與 health，尚無營運觸發入口。

## Goals / Non-Goals

**Goals:**

- 一個命令得到可驗證的完整備份、一個命令還原並自動驗證——災難演練是測試套件的一部分，不是文件上的承諾。
- 備份自帶完整性證據（逐項 digest＋格式版本），驗證不需要原環境在場。
- admin 面補齊 §13.2 資料操作項（export 下載、備份資訊、migration 觸發），全部入 audit。

**Non-Goals:**

- 不做排程備份與保留輪替（cron/systemd timer 屬部署層；文件註記範例即可）。
- 不做增量備份與串流備份——資料規模是小型團隊規格庫，全量夠用且驗證簡單。
- 不做跨版本還原轉換：備份格式版本不符即拒絕，不嘗試就地升級備份檔（還原到新版 server 後由既有 migrate 機制升級資料庫）。
- 不做遠端備份目標（S3 等）：輸出是本機檔案，搬運屬部署層。
- 不做執行中 server 的線上備份（維護模式擋寫）：本刀的一致性前提是備份期間無寫入，由運維者停機或維護窗口保證。
- 不做 audit log 的保留政策（audit 隨 identity 庫整體備份還原，裁剪議題後續需求出現再議）。
- 不動 TeamStore 契約與任何 driver 實作——本刀是契約方法的消費者。

## Decisions

### 決策 1：備份檔是自描述的單一 tar 檔，逐項 digest

backup 子命令輸出單一 tar 檔：manifest 檔（備份格式版本、建立時間 UTC、server engine/API 版本、store manifest、identity schema version、scope 清單與逐項 digest）、每 scope 一個 export bundle（JSON，內容即 TeamStore export 的 Bundle）、identity 資料庫快照。digest 覆蓋每個成員檔；manifest 自身的 digest 附於側檔。tar 選擇是為單檔搬運與逐成員驗證兼得，不壓縮（規格文本量小，可讀性與簡單優先）。

### 決策 2：identity 快照走 SQLite backup API，store 走契約 export

兩份資料的一致性策略不同：identity 資料庫以 SQLite 的線上備份機制產生時點一致快照（連 WAL 一起收斂）；store 資料經 TeamStore export 契約逐 scope 匯出——不直接抄 store 資料庫檔，備份因此與 driver 無關（未來 ServerFS/PostgreSQL driver 的備份不需要新格式）。兩者間的宏觀一致性以「備份期間無寫入」保證：backup 子命令對未運行的資料操作（server 停止，或部署層確保維護窗口無寫入），不做寫入中快照的樂觀一致性；執行中 server 的線上維護模式備份屬後續能力，本刀不交付。

### 決策 3：restore 只進空目標，驗證是還原的一部分

restore 子命令要求目標 store 與 identity 皆空——非空即拒絕，無 force 旗標（誤指目標的代價是覆蓋正典，寧可要求運維者自行清空以示意圖明確）。還原順序：驗證備份檔完整性（等同 verify-backup）→ identity 快照落位 → 逐 scope import（ImportMode 全新建立）→ restore validation：逐 scope 重讀比對 bundle digest 與文件數、revision 檢查、identity 的使用者/registry/audit 計數與 schema version 比對 manifest。驗證輸出逐項報告，任一不符非零 exit code 且明示差異項；驗證失敗的還原目標視為不可用（報告明示勿投產）。

### 決策 4：verify-backup 獨立且不接觸目標環境

verify-backup 只讀備份檔：manifest digest、逐成員 digest、bundle 結構可解析、備份格式版本已知——全部通過才回 0。這讓例行備份的健康檢查不需要空環境；竄改偵測（任一位元變動）在此層攔截，restore 內建同一驗證作為第一步。

### 決策 5：admin 面三項資料操作皆走既有門禁與 audit

admin API/頁面新增：scope export 下載（呼叫 TeamStore export 即時產 bundle 檔下載，audit 記 scope）、備份資訊檢視（最近一次 backup/verify 的 manifest 與結果——由 backup 子命令寫入 identity 庫的備份記錄表，schema 演進一版）、migration 觸發（前置 health 檢查通過才執行 TeamStore migrate，結果入 audit；失敗回明確錯誤不重試）。三者沿用 admin 門禁；audit 動作種類增列 scope-exported、store-migrated、backup-recorded。

## Implementation Contract

- Behavior：運維者執行 backup 子命令得到單一備份檔；verify-backup 回報完整性；在全新主機 restore 後啟動 server，成員以既有 PAT 連線、全部規格資料與 audit 歷史完整；admin 可下載單一 scope 的 export bundle 並觸發 store migration。
- Interface / data shape：backup 子命令（--config 定位資料、--output 檔案路徑；對未運行的資料執行）；restore 子命令（--config 指向空目標、--input 備份檔）；verify-backup 子命令（--input）；備份 tar 內 manifest/bundles/identity 快照與 digest 側檔；admin API 的 export 下載、備份資訊、migration 觸發路由；audit 新動作種類三個。
- Failure modes：備份檔任一 digest 不符或格式版本未知 → verify-backup 與 restore 皆非零 exit code 並指出成員；restore 目標非空 → 拒絕並列出既有內容摘要；restore validation 任一項不符 → 非零 exit code、逐項差異報告、明示目標勿投產；migration 前置 health 失敗 → 不執行並回報；export 下載對未知 scope → 404。
- Acceptance criteria：cargo test -p speclink-server 全綠（備份往返、竄改拒絕、空目標守門、validation 報告、admin 三項與 audit）；npm run test:all 全綠且 parity/color/twin 凍結零 diff。

## Risks / Trade-offs

- 備份要求無寫入窗口（停機或維護窗口）→ 小型團隊規格庫的全量 export 是秒級，窗口可忽略；換取免做快照一致性協定。
- 不壓縮的 tar 較大 → 規格文本量級下可接受；壓縮可後續加於格式版本 2，驗證邏輯不變。
- identity 快照是 SQLite 檔位元拷貝，還原目標的 SQLite 版本差異理論上存在 → SQLite 檔案格式向後相容性極強，且 restore validation 的計數/版本比對會攔截異常。
- restore 無 force 旗標增加演練摩擦 → 誤覆蓋正典的風險遠大於多一步清空的成本。

## Migration Plan

前置依賴 server-setup-registry 與 server-admin-audit 已歸檔。identity schema 演進一版（備份記錄表，只加表，migrate 自動升級）。本刀落地後 Phase 2 的 backup gate 由 e2e 演練測試常駐把關；部署文件補 backup/restore/verify-backup 操作與排程範例。回退即回捨 change，已產生的備份檔自帶版本標記不受影響。

## Open Questions

（無）

## Why

藍圖 §13.1 把 Server FS 列為官方 server 首次 setup 可選的 single-node 持久層（NAS/共享目錄部署情境），§14 Phase 2 第 2 項與 roadmap §5 gate 要求三個 driver 全部通過共同 conformance suite。SQLite reference implementation 已全綠，roadmap §4.3 的複製條件成立：以同一套 conformance（含 arm_crash 四故障點）實作檔案系統 driver。§15.3 另列 FS 特有的失敗模型義務——NAS 暫時失聯、檔案鎖失效、mtime 精度不足——必須文件化並受測，不能只是「在我機器上能跑」。

目標使用者：偏好純檔案持久層的 server 部署者（NAS、備份工具直接可見的目錄、無資料庫維運）；本刀同時是「conformance 範本可複製」的第一次實證。

## What Changes

- 新增 `speclink-store-fs` crate（藍圖 §13.4 的正典交付名）：以單一資料目錄實作 TeamStore 契約全數方法——manifest（single-node 等級、snapshot/cas/transaction/history/outbox/migration/backup 能力）、health、migrate、snapshot、UoW commit/rollback、history、outbox read/ack、export/import。
- 原子性策略：每 scope 一份索引檔為唯一事實指標——commit 將新內容寫入 staging（revision 內容檔、history/outbox 記錄檔，全數未被引用前皆惰性），最後以索引檔的原子替換為單一發布點；任一故障點崩潰後重開，舊索引完好、孤兒檔案清理，commit 從未發生（與 conformance 的 partial-commit gate 語意一致）。排序與 revision 全部出自索引序號，SHALL NOT 依賴 mtime。
- 單寫者：資料目錄內鎖檔（advisory file lock）序列化寫入；鎖不可得回 unavailable；殘留鎖（持有者已死）的偵測與接管策略明確。
- §15.3 失敗模型受測：I/O 錯誤（模擬 NAS 失聯）回 backend/unavailable 且不損毀既有狀態、重開即恢復；mtime 不參與任何語意（測試以人為竄改 mtime 驗證行為不變）；版本守門沿用 sqlite 範本（未知較新版本拒開、非本 driver 目錄拒用）。
- server 組態 driver 清單新增 serverfs 選項（build_store 單點接線，路徑宣告），sqlite 維持預設。

## Capabilities

### New Capabilities

- `serverfs-team-store`: 檔案系統 TeamStore driver——conformance 全綠、索引原子發布、檔案鎖單寫者、mtime 無涉、NAS 失敗模型與版本守門的行為保證。

### Modified Capabilities

(none)

## Impact

- 相容性影響：純新增 crate 與組態選項；既有 sqlite/memory driver、server 路由與 CLI 行為零變更；parity 31 項、color 16 項、twin 8 情境凍結不動。備份格式與 driver 無關（bundle export），本 driver 自動被既有 backup/restore 涵蓋。與 postgres-team-store 刀無共享檔案（除 workspace Cargo 清單與 server 組態 enum，兩處為可並行的小接點），可平行實作。
- Affected specs: `serverfs-team-store`（新增）
- Affected code:
  - New: crates/speclink-store-fs/Cargo.toml、crates/speclink-store-fs/src/lib.rs、crates/speclink-store-fs/src/layout.rs、crates/speclink-store-fs/tests/conformance.rs
  - Modified: Cargo.toml、Cargo.lock、crates/speclink-server/src/config.rs、crates/speclink-server/src/lib.rs、crates/speclink-server/Cargo.toml
  - Removed: 無

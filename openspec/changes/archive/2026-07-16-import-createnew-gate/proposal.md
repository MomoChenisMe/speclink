## Problem

ImportMode 的 CreateNew 在四支 TeamStore 實作間語意分歧：memory 與 sqlite 只檢查「bundle 內的文件是否已存在於目標 scope」，fs 與 postgres 檢查「目標 scope 是否持有任何文件」。目標 scope 持有文件 X、bundle 只含 Y 時——memory/sqlite 錯誤地成功匯入，把兩個來源的歷史交錯在同一個 revision 計數器下；fs/postgres 正確拒絕。四支的錯誤類別與訊息文字已一致（backend、同一句來源描述），分歧純在檢查條件。serverfs-team-store 的 verify 發現此 parity 分歧，當時裁定「維持 FS 嚴格解讀、記錄後續處理」——本刀即該筆收債。

## Root Cause

契約 doc（ImportMode::CreateNew 的註解）寫明「The target scope must hold no documents; import creates everything」，但最早的 memory reference 與照抄它的 sqlite 把前置檢查實作成逐 bundle 文件存在性檢查；conformance suite 只往空 store 匯入，從未測「CreateNew 進非空 scope」這條邊界，偏差因此存活兩刀，直到 fs driver 按 doc 字面實作才暴露。

## Proposed Solution

裁定契約 doc 字面為正典（fs/postgres 現行為即正確實作）：

- 修 memory 與 sqlite 的 CreateNew 前置檢查——由「bundle 內文件已存在」改為「目標 scope 持有任何文件」，錯誤類別與訊息維持現值（backend、既有來源描述）。
- conformance suite 新增「CreateNew 進非空 scope 必須拒絕」gate：目標 scope 先持有一份 bundle 外的文件，import CreateNew 必須整筆拒絕、不部分套用、scope 狀態位元不變；Overwrite 模式不受影響照常通過。
- 四支實作（memory、sqlite、fs、postgres）對更新後的 suite 重跑全綠。

## Non-Goals

- 不動 ImportMode 的值域與 Overwrite 語意；不新增 error 類別（backend 分類維持——封閉六類無 precondition 專屬類別，擴值域成本大於效益且四支現值已一致）。
- 不動 server restore 流程（其 target-not-empty 前置與本檢查疊加，行為不變）。
- 不動 export、bundle 格式與 digest 驗證。

## Success Criteria

- 對四支實作各執行：目標 scope 持有文件 X 後 import 只含 Y 的 bundle（CreateNew）→ 四支一致拒絕（backend），scope 仍只有 X 且 revision 未動。
- conformance suite 含該 gate；cargo test 對 speclink-store（memory）、speclink-store-sqlite、speclink-store-fs、speclink-store-postgres（含環境變數指向的真實例）全綠。
- npm run test:all 全綠；server 既有 backup/restore e2e 不受影響。

## Impact

- Affected specs: `teamstore-contract`（修改：export/import 需求明文 CreateNew 的空 scope 前置與拒絕情境）
- Affected code:
  - Modified: crates/speclink-store/src/memory.rs、crates/speclink-store-sqlite/src/lib.rs、crates/speclink-store/src/conformance/mod.rs
  - New: 無
  - Removed: 無

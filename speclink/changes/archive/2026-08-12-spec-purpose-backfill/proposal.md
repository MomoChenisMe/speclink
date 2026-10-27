## Summary

一次補寫 67 份正典規格的 Purpose 佔位——每份以一至三句寫清該 capability 管什麼、邊界到哪，直接編輯 openspec/specs/<capability>/spec.md（不經 delta），以 spec-purpose-gates 產出的 validate --specs 全綠為驗收。

## Motivation

現行 67 份正典規格中 66 份的 Purpose 仍是 archive 佔位句（討論 archived-parity-and-spec-purpose 議題 4 實測）；本 change 開跑前先行封存的 manual-task-marker-gates 會新建 capability manual-task-marker，其 delta 未帶 Purpose，封存後成為第 67 份佔位（隨後封存的 task-marker-ui-and-parallel-removal 全為既有 capability 的 delta，不再新增佔位）。Purpose 的唯一機器消費者是 propose 的 capability 歸屬判斷——全數佔位使歸屬只能靠 capability 名稱猜，是 capability 邊界漂移的直接來源；規格頁的「Purpose 待補」警示也持續佔據視覺。守門（spec-purpose-gates）只擋守門上線後新開的 capability，對這 67 份存量零作用——存量必須補寫收拾。使用者裁定一次全補（非用到才補），品質控管採抽審＋validate 全綠。

## Proposed Solution

- 逐份補寫：每份 Purpose 自該規格的 Requirements 內容與封存歷史（@trace 的 source change、封存目錄的 proposal Why）提煉，一至三句、50 字元以上，寫清「管什麼」與「保證什麼」；直接編輯正典檔，符合規則正典「改既有 Purpose（含殘留佔位）直接編輯正典檔、不經 delta」的指引。
- 行文以 archive-merge 既有的真 Purpose 與 spec-validation 的新 Purpose 為範本（一句定義轄域＋一句保證面）。
- 品質控管（使用者裁定）：抽審——完成後自 67 份中抽 10 份（涵蓋引擎／CLI／desktop／server／品質站五個領域各至少 1 份）供使用者過目；其餘以 validate --specs 全綠（零 error、零佔位 warning）為驗收，日後用到哪份覺得不準隨手修。
- 分批推進以控品質：按領域分六批（引擎與生命週期／CLI 與 store 基建／desktop 與 workspace／server／remote／技能與品質站），每批補完即跑一次 validate --specs 確認該批零佔位殘留。

## Non-Goals

- 不動任何 Requirements 內容——只補 Purpose 區段，逐份改動嚴格限縮於 `## Purpose` 與 `## Requirements` 之間。
- 不經 delta、不觸發封存合併——本 change 零程式碼、零 delta（正典直編是規則正典明文指引的路徑）。
- 不重寫 archive-merge 與 spec-validation 既有的真 Purpose（維持原樣）。
- 不處理守門上線後新增 capability 的 Purpose（由 spec-purpose-gates 的守門保證）。

## Impact

- Affected specs: 67 份 Purpose 為佔位的正典規格（openspec/specs/ 下除 archive-merge 與 spec-validation 外全部，含 manual-task-marker）——僅 Purpose 區段
- Affected code:
  - Modified: （無——純規格文件補寫）
  - New: （無）
  - Removed: （無）

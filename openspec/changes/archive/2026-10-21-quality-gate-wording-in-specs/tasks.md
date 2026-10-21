## 1. 正式規格用詞

- [x] 1.1 直接修改五份正式規格的 Purpose 段，把「品質站」改為「品質關卡」，其餘文字逐字不變：openspec/specs/change-diff-scope/spec.md、openspec/specs/client-protocol/spec.md、openspec/specs/desktop-app/spec.md、openspec/specs/server-verb-api/spec.md、openspec/specs/tray-status-menu/spec.md。驗證：這五份檔案在 `## Requirements` 之前的段落沒有「品質站」；`git diff openspec/specs` 只有這五處 Purpose 的用詞差異 <!-- speclink-task:tsk_01M5EQMX9JHFREPEPZ1T9PVC8V -->
- [x] 1.2 核對 delta 涵蓋所有帶舊詞的需求，且每條 MODIFIED 都帶齊正式規格的情境：MODIFIED 為「變更清單的寫碼進度欄位」「技能規定固定輸入來源與四條內容判準」「介面狀態語意色分層」「看板卡片的驗證標示」「兩站時序的編排行為」「審查技能的生成與正典化」「出口交棒由技能結尾承載」；REMOVED＋ADDED 為「品質站蓋章配色與主色分離」→「品質關卡蓋章配色與主色分離」、「面板變更列的品質站章」→「面板變更列的品質關卡章」、「品質站蓋章效果與非成員錯誤碼的文件揭露」→「品質關卡蓋章效果與非成員錯誤碼的文件揭露」；兩個改名的情境有 REMOVED-SCENARIO 註解。驗證：`grep -rn "品質站" openspec/changes/quality-gate-wording-in-specs/specs` 只命中 REMOVED 需求標題、Reason／Migration 與 BEFORE／REMOVED-SCENARIO 註解；逐條比對每個 MODIFIED 區塊的情境名稱與正式規格相同（改名的兩個除外）；`speclink validate quality-gate-wording-in-specs` 通過 <!-- speclink-task:tsk_01M5EQMX9JA2TV12R014XS70HQ -->

## 2. 收尾

- [x] 2.1 文件守門與變更檢查通過。驗證：`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過，且 `speclink validate quality-gate-wording-in-specs` 通過 <!-- speclink-task:tsk_01M5EQMX9JJD9DSBFV6TRY63EN -->

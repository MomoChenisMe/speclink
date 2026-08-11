> 本 change 零程式碼、零 delta（proposal Non-Goals 明列）：僅直編 67 份正典規格的 `## Purpose` 區段。前置：manual-task-marker-gates 與 task-marker-ui-and-parallel-removal 已封存（前者新建 capability manual-task-marker，納入本次補寫範圍）、spec-purpose-gates 已落地（validate --specs 可執行）。每份 Purpose 一至三句、50 字元以上，自該規格 Requirements 與 @trace 溯源提煉「管什麼＋保證什麼」，行文以 archive-merge 與 spec-validation 的真 Purpose 為範本；嚴禁動 `## Requirements` 起的任何內容。

## 1. 前置確認

- [ ] 1.1 確認 spec-purpose-gates 已落地：./target/debug/speclink validate --specs 可執行且對現存佔位規格輸出佔位 warning（工具就位才有驗收面） <!-- speclink-task:tsk_01KZNFGDHHYXX9SQKE93C9BFT4 -->
- [ ] 1.2 確認補寫範圍與清單一致：openspec/specs/ 下 capability 目錄數為 68（含 manual-task-marker），其中 Purpose 為佔位者 67 份（archive-merge 與 spec-validation 已有真 Purpose）；數量對不上時以實際佔位清單為準並修正下方批次 <!-- speclink-task:tsk_01KZNFGDHH4GCCJ3Y0C480DC90 -->

## 2. 批次補寫（每批完成即跑 ./target/debug/speclink validate --specs 確認該批零佔位殘留）

- [ ] 2.1 引擎與生命週期 13 份：change-lifecycle、change-diff-scope、drift-computation、task-identity、manual-task-marker、discussion-docs、context-projection、command-runtime、host-runtime、delivery-baseline、workflow-config、phase2-acceptance、phase3-acceptance <!-- speclink-task:tsk_01KZNFGDHHDCJ5VFPXCT42NVBR -->
- [ ] 2.2 CLI、SDK 與 store 基建 8 份：verb-contract、node-sdk、store-abstraction、teamstore-contract、serverfs-team-store、sqlite-team-store、postgres-team-store、worktree-overlay <!-- speclink-task:tsk_01KZNFGDHHA61TFJDKEX6DN1PV -->
- [ ] 2.3 desktop 與 workspace 11 份：desktop-app、desktop-config、desktop-connections、desktop-release、client-protocol、tray-status-menu、workspace-chooser、workspace-migration、workspace-session、workspace-tools、board-card-order <!-- speclink-task:tsk_01KZNFGDHHV1WMF2T06K8ZSCCC -->
- [ ] 2.4 server 13 份：server-admin、server-backup、server-context-api、server-device-auth、server-drift-api、server-event-stream、server-identity、server-policy-write、server-read-api、server-release、server-setup、server-verb-api、server-web-console <!-- speclink-task:tsk_01KZNFGDHHPGREQD64TY52CCGK -->
- [ ] 2.5 remote 與 reference server 6 份：reference-server、remote-auth、remote-board-order、remote-connection、remote-resilience、remote-workspace-data <!-- speclink-task:tsk_01KZNFGDHHVH9CAT6G2B0Z46PA -->
- [ ] 2.6 技能與品質站、文件 16 份：review-skill、review-station、verify-skill、verify-station、verify-evidence、quality-skill、improve-skill、discuss-skill、propose-skill、archive-skill、commit-skill、config-skill、worktree-apply-skill、worktree-merge-skill、user-documentation、dev-harness <!-- speclink-task:tsk_01KZNFGDHHG0VJE8GMDXZ9JH6P -->

## 3. 驗收

- [ ] 3.1 全量驗收：./target/debug/speclink validate --specs --strict 全綠——零 error、零佔位 warning、零過短 warning（67 份全數合格，archive-merge 與 spec-validation 原樣未動） <!-- speclink-task:tsk_01KZNFGDHH1SX8SZ6H6M0N4EWB -->
- [ ] 3.2 抽樣清單：自六批各抽至少 1 份、共 10 份（含 manual-task-marker、desktop-app、verb-contract、change-lifecycle、server-verb-api、review-station 六份必抽＋隨機 4 份），列出清單與各份 Purpose 全文供過目 <!-- speclink-task:tsk_01KZNFGDHHPFE3FK6WHBW00YE4 -->
- [ ] 3.3 [M] 使用者抽審：過目 3.2 的 10 份 Purpose，不準處提出後逐份修正；抽審通過即驗收完成（其餘 57 份以 validate 全綠承保，日後用到覺得不準隨手修） <!-- speclink-task:tsk_01KZNFGDHH29F5BVBW9N9H8QV0 -->

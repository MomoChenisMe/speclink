## 1. 三個守門測試（皆於 crates/speclink-cli/tests/it/archive_readiness_gate.rs 的 GitProject 情境）

- [ ] 1.1 主 checkout 放行測試：GitProject 建好後於 repo 本體執行 git checkout -b speclink/demo（主 checkout 的 .git 仍為目錄），以 PATH 清空的環境執行 speclink archive demo → 斷言 exit 0、change 目錄移入 archive；PATH 清空同時佐證主 checkout 路徑完全不依賴 git（fs 短路先於分支判定的可觀察等價斷言）。驗證：cargo test -p speclink-cli --test it archive_readiness_gate 新測試綠、既有測試不破 <!-- speclink-task:tsk_01KZAYGZE7TBKAG3ZK1TYG08Z6 -->
- [ ] 1.2 前置寫入零效果測試：把 GitProject::new 的 tasks.md 內容參數化（或加一個帶自訂 tasks 的建構變體，沿用既有 fixture 慣例、不另立新 fixture），建含未勾任務的 change；於 speclink/demo 分支的 worktree 內帶 --mark-tasks-complete 執行封存 → 斷言 exit 非零、worktree 內該 change 的 tasks.md 與寫入時逐位元相同（前置全勾寫入未發生）。驗證：同 1.1 測試指令，對照 delta scenario「拒絕時 --mark-tasks-complete 前置寫入零效果」 <!-- speclink-task:tsk_01KZAYGZE713A50X2CKKNYW0EY -->
- [ ] 1.3 bulk 同受守門測試：GitProject 建第二個 change（demo2，全勾、有效 meta），於 speclink/demo 分支的 worktree 內執行 speclink archive demo demo2（多 change 名即 bulk 路徑）→ 斷言 exit 非零、合併後輸出（stdout＋stderr）含 worktree 事實與 worktree-merge 指路、兩個 change 目錄皆原地不動且無 archive 目錄。驗證：同 1.1 測試指令，對照 delta scenario「bulk 封存同受守門」 <!-- speclink-task:tsk_01KZAYGZE713JM8X6HFR2GKVKJ -->

## 2. 收尾驗證

- [ ] 2.1 全量回歸與條文對照：cargo test -p speclink-cli --test it 全綠、cargo test -p speclink-core 全綠（守門實作零變動，純測試新增）；逐一確認三個新測試與 change-lifecycle 需求「封存的 linked worktree 環境守門」delta 的三個新增／強化 scenario 一一對應（1.1↔主 checkout 零額外開銷含 speclink/ 分支名情形、1.2↔前置寫入零效果、1.3↔bulk 同受守門）。驗證：測試輸出與人工對照 <!-- speclink-task:tsk_01KZAYGZE7H7Z61AKDJ880QME8 -->

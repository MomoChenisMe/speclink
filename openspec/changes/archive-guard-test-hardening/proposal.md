## Summary

補齊封存 linked worktree 守門的三個測試盲區——守門行為已落地且經手動驗收，但三條路徑缺自動化紅燈，守門被誤移或誤改時測試不會咬人。

## Motivation

change worktree-flow-guards-and-guidance（2026-08-06 封存）落地封存守門後，review 與 verify 兩站各留下測試覆蓋缺口：

1. **主 checkout 反向案例缺席**（verify WARNING）：三個守門測試全在 `.git` 為檔案的 worktree 內跑，沒有任何測試覆蓋「`.git` 為目錄、分支恰為 speclink/*」。若守門的 fs 短路與分支判定被調換順序，既有測試仍全綠，主 checkout 上切到 speclink/* 分支的封存卻會被誤拒。
2. **`--mark-tasks-complete` 前置寫入無紅燈**（review 保留項）：設計約束「拒絕時前置寫入不得發生」只靠手動驗收蓋過，runtime 守門若被移到前置寫入之後，tasks.md 會在被拒的封存中被靜默全勾且不回滾——無測試釘住。
3. **bulk 封存無專屬測試**（verify SUGGESTION）：change-lifecycle 需求明寫「單筆與 bulk」，實作靠共用 run_archive 天然涵蓋，但無測試釘死 bulk 路徑，重構時可能繞過。

## Proposed Solution

在既有的 crates/speclink-cli/tests/it/archive_readiness_gate.rs 的 GitProject fixture 上補三個整合測試（不新增 fixture、不動任何產品程式碼）：

1. **主 checkout 放行**：GitProject 建好後於 repo 本體以 git 切出 speclink/demo 分支，執行封存 → 成功；同時清空 PATH 佐證主 checkout 完全不依賴 git（等價於「不 spawn git」的可觀察斷言）。
2. **前置寫入零效果**：worktree 內、tasks.md 含未勾任務、帶 --mark-tasks-complete 執行封存 → 非零 exit，且 tasks.md 逐位元不變（前置全勾寫入未發生）。
3. **bulk 同受守門**：worktree 內執行 speclink archive --all（或多 change 名）→ 非零 exit、stderr 指路 worktree-merge，所有 change 目錄原地不動。

## Non-Goals

- 不動守門實作與任何產品程式碼——三條測試都應在現行實作上直接轉綠，紅燈只在守門被破壞時出現
- 不處理 submodule／--separate-git-dir 佈局的 `.git` 檔案誤判（review 保留項，觸發組合牽強、可自行繞開）
- 不收斂 speclink/ 前綴常數的 core／host 雙持有（review 保留項，成本大於效益）
- 不抽共用測試 fixture（repo 慣例為每檔自帶；GitProject 沿用即可）
- 不補 change-lifecycle 條文的「remote store 封存不受此守門」敘述（規格文字變更，與測試補強分開處理）

## Impact

- Affected specs: change-lifecycle（MODIFIED——需求句補「前置全勾寫入不得發生」與「主 checkout 短路先於分支判定」兩處明文，scenario 新增「拒絕時 --mark-tasks-complete 前置寫入零效果」與「bulk 封存同受守門」、主 checkout scenario 補 speclink/ 分支名情形；行為不變，僅把既有實作的可觀察斷言錨進條文）
- Affected code:
  - Modified: crates/speclink-cli/tests/it/archive_readiness_gate.rs（於既有 GitProject 情境補三個測試與必要的 fixture 小幅參數化）
  - New: （無）
  - Removed: （無）

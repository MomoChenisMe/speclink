# Verify — tray-panel-restyle

## Round 1

**Phase**: discovery
**Patch**: sha256:3a0a7293738dd8167b711c824eeca1408f43ef47a19974e1ee9cffba76c254ea
**Scope**: apps/desktop/src/__tests__/trayPanel.test.tsx, apps/desktop/src/components/ProjectRail.tsx, apps/desktop/src/components/projectTile.ts, apps/desktop/src/panel/TrayPanel.tsx, apps/desktop/src/panel/promotedExpanded.ts, packages/ui/src/__tests__/copyButton.test.tsx, packages/ui/src/components/CopyButton.tsx
- [SUGGESTION] apps/desktop/src/panel/TrayPanel.tsx — Coherence: delta spec「面板樣式（macOS）」第一段還留著舊句子「討論比照原生選單分『討論』與『已轉出』兩分區」。這和同一變更「討論列表」寫的「macOS 面板 SHALL NOT 另立『已轉出』分區」字面衝突。程式照後者做是對的（TrayPanel.tsx:620–656 的討論卡加收合列）。建議把第一段改成「討論——分流判準與原生選單同源，已轉出收在討論卡的收合列」，免得封存進正式規格後兩句互相矛盾。
- [SUGGESTION] apps/desktop/src/__tests__/trayPanel.test.tsx — Correctness: 需求寫明「作用中的 restoring／error 專案都要和 ready 一樣用實心主色方塊標示選取」，但測試只測了 restoring（trayPanel.test.tsx:642），error 沒測。舊版程式正是在這裡依狀態分支畫不同樣式，算是容易退化的地方。建議把這題改成 `it.each(["restoring","error"])`。程式本身已不再依狀態分支（TrayPanel.tsx:500），現在沒有錯。
- [SUGGESTION] apps/desktop/src/__tests__/trayPanel.test.tsx — Correctness: Scenario「進度條依階段深淺」的測試只檢查填色 class 各是 `STAGE_BAR` 的哪一階，沒有斷言「提案中比進行中淺」這個順序。只要 `STAGE_BAR` 表本身沒排錯就不會出事。可以補一條斷言，確認三階的 class 互不相同且順序固定。
- [SUGGESTION] apps/desktop/src/panel/TrayPanel.tsx — Coherence: 復原卡的圖示方塊（`TrayPanel.tsx:325` 的 `flex h-8 w-8 shrink-0 items-center justify-center rounded-lg`）和 `RAIL_SQUARE` 幾乎同字串，導致任務 1.1 的 grep 驗證寫錯了範圍。這不是程式錯誤，但 tasks.md 留下一個沒有成立的驗證條件。建議在 tasks.md 1.1 的驗證欄註明「排除復原卡圖示方塊」，或改成檢查 `ProjectRail.tsx` 不含這串字。
- [SUGGESTION] apps/desktop/src/panel/TrayPanel.tsx — Coherence: 有兩處和 design 字面不同，都已經向使用者報告過，判定為合理的偏離。第一，「加入專案」容器用 `min-w-16`（design D3 寫「容器同 tab 的 hover」，hover 確實一樣，只有寬度規則不同）。第二，切換中的 spinner 用 h-4，和圖示列一致。建議在 design D3 補一句說明，讓 artifact 和實作對得上。

## Round 2

**Phase**: validation
**Patch**: sha256:fd2f3f0cb0e13588ac94b395aaece3ef581ad33978c31f946f242f905083fdd9
**Scope**: apps/desktop/src/__tests__/trayPanel.test.tsx, apps/desktop/src/__tests__/updateRailButton.test.tsx, apps/desktop/src/components/ProjectRail.tsx, apps/desktop/src/components/UpdateRailButton.tsx, apps/desktop/src/panel/TrayPanel.tsx

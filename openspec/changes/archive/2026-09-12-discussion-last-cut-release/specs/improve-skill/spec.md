## MODIFIED Requirements

### Requirement: candidates 以討論記錄承載
<!-- BEFORE: 扇出段規定分期立案時最後一刀封存後教執行 speclink discuss archive <slug> 收尾 -->

渲染產出的 speclink-improve 技能檔 SHALL 規定:掃描完成後以 speclink discuss new 帶 --kind improve 與 --slug(慣例 improve-<範圍>)建立討論記錄,candidates 以 Round 1(mode 標籤 scan)記錄,每個 candidate SHALL 含 Files、Problem、Solution、Wins、建議強度(強烈建議/值得探索/尚屬臆測三級)五欄位,結尾 SHALL 含首選建議並詢問使用者深入哪一個。grilling SHALL 沿用一次一題、提案帶證據的紀律,interface depth check SHALL 對每個被挑中的 candidate 無條件執行;對第六訊號(分組只靠檔名猜)的 candidate,四問 SHALL 改為分組邊界的客觀來源、既有路徑靠什麼維持不變、搬移前盤點的路徑風險(內嵌路徑的公開網址、tag 觸發或依路徑篩選的 CI workflow、以相對路徑寫的建置期檔案引入、跨套件 path 相依)、以及回到平鋪讀者失去什麼,並 SHALL 附帶兩條做法:測試鏡射原始碼分組搬移(內嵌測試逾行數門檻時搬至同名子檔)、切刀依路徑相依排序(先動目錄本身的刀、再動目錄內分組的刀,SHALL NOT 以 worktree 平行)。收斂 SHALL 走 conclude,經 promote 或 link 扇出變更;結論規劃分期立案(先立一刀、封存後再回同一記錄轉出下一刀)時 conclude SHALL 帶 --hold 一次,hold 不因不帶 --last 的轉出而清除、只由不帶 --hold 的 conclude、speclink discuss archive 或帶 --last 的轉出解除,最後一刀 SHALL 由 propose 於轉出時帶 --last、其後最後一個轉出變更封存時記錄自動隨行封存,SHALL NOT 教執行 speclink discuss archive 作為常態收尾;忘了帶 --last 時 SHALL 註明執行 speclink discuss archive <slug> 一次收尾;未帶旗標的記錄在最後一個轉出變更封存時隨行封存、之後的刀 SHALL 走新討論;使用者全數否決時 SHALL 仍以 conclude(記明不做與理由)加 archive 收尾,SHALL NOT discard。

#### Scenario: candidates 記錄形式

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的建記錄段落
- **THEN** 內容 SHALL 規定 discuss new 帶 --kind improve、Round 1 以 mode 標籤 scan 記錄、candidate 五欄位與三級建議強度、結尾首選建議

#### Scenario: 全數否決仍留記錄

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的收斂段落
- **THEN** 內容 SHALL 規定全數否決時走 conclude 加 archive,並 SHALL 含禁止 discard 的敘述

#### Scenario: 分期立案帶 --hold

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的扇出段落
- **THEN** 內容 SHALL 規定結論規劃分期立案時 conclude 帶 --hold 一次、hold 只由不帶 --hold 的 conclude、discuss archive 或帶 --last 的轉出解除、最後一刀由 propose 帶 --last 轉出且最後一個封存自動收尾、忘帶 --last 時執行 discuss archive 一次收尾、未帶旗標時後續刀走新討論;SHALL NOT 含「旗標由下一次轉出清除」與「最後一刀封存後手動 discuss archive 收尾」的常態敘述;claude 與 codex 兩工具的技能實例與 render golden SHALL 同步反映

#### Scenario: 第六訊號的深度四問對應版

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔 grilling 段落的 interface depth check 條目
- **THEN** 內容 SHALL 於既有四問之後規定第六訊號的對應四問(分組來源、路徑維持、路徑風險盤點、讀者預測測試)取代既有四問,並含測試鏡射搬移與切刀依路徑相依排序、不以 worktree 平行的敘述;Guardrails 的 deletion test 一條 SHALL 註明第六訊號改答讀者預測測試

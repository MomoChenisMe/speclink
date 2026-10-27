## MODIFIED Requirements

### Requirement: candidates 以討論記錄承載

渲染產出的 speclink-improve 技能檔 SHALL 規定:掃描完成後以 speclink discuss new 帶 --kind improve 與 --slug(慣例 improve-<範圍>)建立討論記錄,candidates 以 Round 1(mode 標籤 scan)記錄,每個 candidate SHALL 含 Files、Problem、Solution、Wins、建議強度(強烈建議/值得探索/尚屬臆測三級)五欄位,結尾 SHALL 含首選建議並詢問使用者深入哪一個。grilling SHALL 沿用一次一題、提案帶證據的紀律,interface depth check SHALL 對每個被挑中的 candidate 無條件執行。收斂 SHALL 走 conclude,經 promote 或 link 扇出變更;結論規劃分期立案(先立一刀、封存後再回同一記錄轉出下一刀)時 conclude SHALL 帶 --hold,記錄留在途直到下一次轉出清掉旗標,未帶旗標的記錄在最後一個轉出變更封存時隨行封存、之後的刀 SHALL 走新討論;使用者全數否決時 SHALL 仍以 conclude(記明不做與理由)加 archive 收尾,SHALL NOT discard。

#### Scenario: candidates 記錄形式

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的建記錄段落
- **THEN** 內容 SHALL 規定 discuss new 帶 --kind improve、Round 1 以 mode 標籤 scan 記錄、candidate 五欄位與三級建議強度、結尾首選建議

#### Scenario: 全數否決仍留記錄

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的收斂段落
- **THEN** 內容 SHALL 規定全數否決時走 conclude 加 archive,並 SHALL 含禁止 discard 的敘述

#### Scenario: 分期立案帶 --hold

- **WHEN** 檢視渲染產出的 speclink-improve 技能檔的扇出段落
- **THEN** 內容 SHALL 規定結論規劃分期立案時 conclude 帶 --hold、旗標由下一次轉出清除、未帶旗標時後續刀走新討論;claude 與 codex 兩工具的技能實例與 render golden SHALL 同步反映

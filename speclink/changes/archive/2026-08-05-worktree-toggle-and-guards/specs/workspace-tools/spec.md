## ADDED Requirements

### Requirement: worktree 技能的政策條件式生成

技能足跡生成（speclink init 與 speclink update）SHALL 依 openspec/config.yaml 的 worktree 檔值過濾兩顆 worktree 技能（speclink-apply-with-worktree 與 speclink-worktree-merge）：值為 true 時生成，false 或未設時不生成，且既有生成物依既有清理生命週期移除（技能目錄移除，因而變空的目錄一併移除）。生成判定 SHALL NOT 納入 SPECLINK_WORKTREE 環境變數（環境層僅影響執行期政策，不影響足跡）。其餘技能的生成集合不受 worktree 政策影響。此過濾對 claude、codex 與自訂描述子工具一視同仁。

#### Scenario: 政策關閉時生成集合不含 worktree 技能

- **WHEN** openspec/config.yaml 無 worktree 鍵（或值為 false）的專案執行 speclink update
- **THEN** 各工具 skills 目錄不含 speclink-apply-with-worktree 與 speclink-worktree-merge 目錄，其餘技能照常生成

#### Scenario: 政策開啟時注入兩顆技能

- **WHEN** openspec/config.yaml 含 worktree: true 的專案執行 speclink update
- **THEN** 各工具 skills 目錄含兩顆 worktree 技能，內容與 golden 對照一致

#### Scenario: 政策由開改關後再生即清理

- **WHEN** 先於 worktree: true 下執行 speclink update，再將該鍵改為 false 並重新執行 speclink update
- **THEN** 兩顆 worktree 技能目錄被移除，其餘技能與 marker 區塊保留

#### Scenario: 環境變數不影響生成

- **WHEN** openspec/config.yaml 無 worktree 鍵，但於 SPECLINK_WORKTREE=true 環境下執行 speclink update
- **THEN** 兩顆 worktree 技能不生成——環境層僅於技能執行期由 P1 政策檢查讀取

#### Scenario: 過期探測不把被政策排除的技能報成過期

- **WHEN** worktree 政策關閉的專案於技能檔已同步的狀態下執行指令檔過期探測
- **THEN** 探測結果不列出兩顆 worktree 技能——被政策排除者不屬於預期生成集合

### Requirement: marker 技能指引跟隨 worktree 政策

內建工具指令檔（CLAUDE.md／AGENTS.md）的 SPECLINK marker 區塊，其兩行 worktree 技能指引（apply-with-worktree 與 worktree-merge）SHALL 與技能檔生成套用同一個 worktree 政策閘：政策為 true 時輸出，false 或未設時不輸出。marker 的其餘內容（其他技能指引、Workflow 段）SHALL NOT 受此政策影響。自訂描述子的中性 marker 本就不列 worktree 技能，不受此需求影響。

#### Scenario: 政策關閉時 marker 不提 worktree 技能

- **WHEN** worktree 政策關閉的專案執行 speclink update
- **THEN** CLAUDE.md 的 marker 區塊不含 apply-with-worktree 與 worktree-merge 兩行，其餘技能指引與 Workflow 段照舊

#### Scenario: 政策開啟時 marker 含 worktree 技能

- **WHEN** worktree 政策開啟的專案執行 speclink update
- **THEN** CLAUDE.md 的 marker 區塊含 apply-with-worktree 與 worktree-merge 兩行，且與政策關閉版本僅差這兩行

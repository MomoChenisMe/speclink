## ADDED Requirements

### Requirement: 品質關卡技能的生成與正典化

`speclink update` SHALL 生成 `/speclink-quality` 技能檔至已啟用工具（claude／codex）的技能目錄，內容以引擎內的正典模板為準（golden 對照涵蓋）。同次更新 SHALL 使生成之 CLAUDE.md／AGENTS.md 的 workflow 行含 quality 入口，並於技能使用清單加入 quality 條目，敘明觸發時機：事前已知 review 與 verify 兩站都要跑時使用；只跑一站直接呼叫該站技能。

#### Scenario: 技能檔生成

- **WHEN** 於已啟用 speclink 的專案執行 `speclink update`
- **THEN** 已啟用工具的技能目錄出現 speclink-quality 技能檔，且內容與 golden 對照一致

#### Scenario: workflow 行與技能清單含 quality 條目

- **WHEN** `speclink update` 完成後讀取生成的 CLAUDE.md
- **THEN** workflow 行含 quality 入口，技能使用清單含 quality 條目與其觸發時機（兩站都跑時使用；單站直接呼叫該站技能）

### Requirement: 兩站時序的編排行為

技能 SHALL 只承載兩站時序，SHALL NOT 重述兩站的檢查內容、工單與蓋章語意（由 review／verify 技能正典各自承載）。前提為 change 任務全數完成；未完成時 SHALL 依兩站既有前提行為呈現（拒絕或中途盤點，依該站正典），技能不另設守門。時序 SHALL 為：review 檢查以「先不蓋章」離場 → verify 檢查以「先不蓋章」離場 → 兩站 findings 合併 triage 並統一修正（一律回主線、依專案 TDD 慣例）→ 重跑 review 複驗至必修淨空、仍以「先不蓋章」離場 → 重跑 verify 複驗至必修淨空、同樣先不蓋章 → 確認無任何後續修正後，review 章與 verify 章接連補蓋 → 建議封存。任一站複驗引出新的修正時 SHALL 於修正後重跑兩站複驗，迴圈至一輪完整複驗零新修正才補蓋兩章。收尾補蓋時，技能 SHALL 向兩站明示本次為收尾補蓋呼叫——兩站據此關閉其禁蓋例外；未宣告即補蓋不合本規格。兩章 SHALL 接連落、中間零編輯，使兩章至封存皆不出現「其後有變動」狀態。

#### Scenario: 完整兩站時序

- **WHEN** 使用者對任務全數完成的 change 要求兩站都跑（或執行 /speclink-quality）
- **THEN** 依序完成 review 檢查先不蓋章、verify 檢查先不蓋章、兩站 findings 統一修正、review 複驗淨空不蓋章、verify 複驗淨空不蓋章，收尾以明示的收尾補蓋呼叫讓兩章接連補蓋，且到封存皆維持已完成狀態、無「其後有變動」

#### Scenario: 複驗引出新修正時延後補蓋

- **WHEN** 統一修正後某站複驗仍揪出必修並再次修正
- **THEN** 此時兩章皆尚未蓋下，技能於修正後重跑兩站複驗，直到一輪完整複驗零新修正才接連補蓋兩章，全程不出現已蓋章又被後續修正打黃的狀態

#### Scenario: 事後變卦加跑第二站

- **WHEN** 一站已蓋章後使用者才要求加跑另一站
- **THEN** 技能 SHALL NOT 重做已蓋章的站，照跑新站並接受前章暫態轉「其後有變動」，封存後定格回已完成

#### Scenario: 單站請求不經編排

- **WHEN** 使用者只要求跑單一品質站
- **THEN** 直接呼叫該站技能，quality 不啟動，該站維持修完即蓋章的預設行為

#### Scenario: 前提未完成時依站內前提行為呈現

- **WHEN** change 尚有未完成任務即啟動兩站時序
- **THEN** 依兩站既有前提行為呈現（review 站拒絕並停止；verify 站以中途盤點報告收場），quality 不另設守門、不吞錯

# quality-skill Specification

## Purpose

TBD - created by archiving change 'quality-skill-canonicalization'. Update Purpose after archive.

## Requirements

### Requirement: 品質關卡技能的生成與正典化

`speclink update` SHALL 生成 `/speclink-quality` 技能檔至已啟用工具（claude／codex）的技能目錄，內容以引擎內的正典模板為準（golden 對照涵蓋）。同次更新 SHALL 使生成之 CLAUDE.md／AGENTS.md 的 workflow 行含 quality 入口，並於技能使用清單加入 quality 條目，敘明觸發時機：事前已知 review 與 verify 兩站都要跑時使用；只跑一站直接呼叫該站技能。

#### Scenario: 技能檔生成

- **WHEN** 於已啟用 speclink 的專案執行 `speclink update`
- **THEN** 已啟用工具的技能目錄出現 speclink-quality 技能檔，且內容與 golden 對照一致

#### Scenario: workflow 行與技能清單含 quality 條目

- **WHEN** `speclink update` 完成後讀取生成的 CLAUDE.md
- **THEN** workflow 行含 quality 入口，技能使用清單含 quality 條目與其觸發時機（兩站都跑時使用；單站直接呼叫該站技能）


<!-- @trace
source: quality-skill-canonicalization
updated: 2026-08-07
-->

---
### Requirement: 兩站時序的編排行為

技能 SHALL 只承載兩站時序，SHALL NOT 重述兩站的檢查內容、工單與蓋章語意（由 review／verify 技能正典各自承載）。前提為 change 任務全數完成；未完成時 SHALL 依兩站既有前提行為呈現（拒絕或中途盤點，依該站正典），技能不另設守門。

時序 SHALL 為每輪暫停制：

- 檢查輪：review 檢查以「先不蓋章」離場 → verify 檢查以「先不蓋章」離場 → 彙整兩站 findings 向使用者報告並停下詢問下一步；SHALL NOT 未經使用者裁示即開始修正。
- 停下時提供的選項 SHALL 至少涵蓋：全修、挑選部分修正、不修就停（兩站工單與凍結快照留存、不蓋章離場）。必修尚未淨空時 SHALL NOT 提供補蓋選項——站內「必修淨空才蓋章」的正典不被繞過。
- 修正輪：使用者裁示的修正統一落地（一律回主線、依專案 TDD 慣例）後重跑 review 複驗與 verify 複驗；每輪兩站複驗完成後 SHALL 再次停下詢問，直到使用者裁示收尾或停止。
- 乾淨輪：兩站皆零 findings，或必修淨空且該輪零新修正時，SHALL 同樣停下報告兩站皆綠，由使用者決定是否進入收尾補蓋；SHALL NOT 自動補蓋。
- 收尾：使用者裁示補蓋後，review 章與 verify 章接連補蓋；補蓋時技能 SHALL 向兩站明示本次為收尾補蓋呼叫——兩站據此關閉其禁蓋例外；未宣告即補蓋不合本規格。票內仍留有使用者裁示不修的 findings 時，該站之章為帶保留章——技能 SHALL 於補蓋選項中預先載明，並以停下時的使用者裁示為明示授權；SHALL NOT 未經明示逕自蓋保留章。兩章 SHALL 接連落、中間零編輯，使兩章至封存皆不出現「其後有變動」狀態。封存 SHALL 以建議形式提出、由使用者執行。

#### Scenario: 完整兩站時序

- **WHEN** 使用者對任務全數完成的 change 要求兩站都跑（或執行 /speclink-quality）
- **THEN** 依序完成 review 檢查先不蓋章、verify 檢查先不蓋章，彙整兩站 findings 停下待裁示；裁示的修正統一落地後兩站複驗、每輪複驗完成再停；必修淨空且零新修正的輪停下報告兩站皆綠，使用者裁示後以明示的收尾補蓋呼叫讓兩章接連補蓋，且到封存皆維持已完成狀態、無「其後有變動」

#### Scenario: 檢查輪後暫停等待裁示

- **WHEN** 兩站檢查完成且合計揪出至少一筆 finding
- **THEN** 技能彙整兩站 findings 停下詢問（全修／挑選部分修正／不修就停），使用者裁示前不落任何修正編輯

#### Scenario: 複驗引出新修正時延後補蓋

- **WHEN** 統一修正後某站複驗仍揪出必修
- **THEN** 此時兩章皆尚未蓋下；技能停下回報並待使用者裁示，裁示的修正落地後重跑兩站複驗，迴圈至一輪完整複驗零新修正且使用者裁示補蓋，才接連補蓋兩章，全程不出現已蓋章又被後續修正打黃的狀態

#### Scenario: 乾淨輪停下由使用者決定補蓋

- **WHEN** 某輪兩站皆零 findings，或必修淨空且該輪零新修正
- **THEN** 技能報告兩站皆綠並停下；使用者裁示後才以明示的收尾補蓋呼叫接連補蓋兩章、中間零編輯，使用者未裁示前不蓋任何章

#### Scenario: 使用者選擇不修就停

- **WHEN** 檢查輪或複驗輪停下時使用者選擇不修就停
- **THEN** 技能以兩站「先不蓋章」出口離場收尾，工單與凍結快照留存，不蓋任何章、不自行封存

#### Scenario: 事後變卦加跑第二站

- **WHEN** 一站已蓋章後使用者才要求加跑另一站
- **THEN** 技能 SHALL NOT 重做已蓋章的站，照跑新站並接受前章暫態轉「其後有變動」，封存後定格回已完成

#### Scenario: 單站請求不經編排

- **WHEN** 使用者只要求跑單一品質站
- **THEN** 直接呼叫該站技能，quality 不啟動，該站維持修完即蓋章的預設行為

#### Scenario: 前提未完成時依站內前提行為呈現

- **WHEN** change 尚有未完成任務即啟動兩站時序
- **THEN** 依兩站既有前提行為呈現（review 站拒絕並停止；verify 站以中途盤點報告收場），quality 不另設守門、不吞錯


<!-- @trace
source: quality-skill-round-pause
updated: 2026-08-07
-->
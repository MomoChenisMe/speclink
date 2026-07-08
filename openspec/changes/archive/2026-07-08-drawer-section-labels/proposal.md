## Why

drawer-document-readability 刀完成後，討論側（輪卡片、結論欄位、規格色標）已有一致的結構化視覺語言——中文標籤區塊、卡片邊界、色標——但變更抽屜的提案／設計分頁仍是生 markdown：Why、What Changes、Context 等英文模板章節名直接以 h2 印給讀者，任務分頁群組標題另用粗體 h4，三種分頁形同三套視覺語言。使用者明示：discuss 與 changes 的分頁內容設計以一致為準。英文模板詞直出也與專案 UI 文字規範（繁體中文、工程詞不進使用者可見文案）抵觸。目標使用者是以桌面 app 檢視 SDD 文件的開發者／PO／PM，情境是變更抽屜與已封存檢視的提案／設計／任務分頁日常閱讀。

## What Changes

- 提案／設計分頁章節標籤化：已知模板章節名（提案側 Why／What Changes／Non-Goals／Capabilities／New Capabilities／Modified Capabilities／Impact／Problem／Root Cause／Proposed Solution／Success Criteria／Summary／Motivation／Alternatives Considered；設計側 Context／Goals / Non-Goals／Decisions／Implementation Contract／Risks / Trade-offs／Migration Plan／Open Questions）映射中文標籤（為什麼／變更內容／非目標／能力／影響／背景／決策／實作契約／風險與取捨／未解問題等），以討論側結論欄位同款標籤區塊呈現，英文模板標題不再直出；白名單以外的章節標題（如設計的 D1 決策標題、手寫自訂章節）照 prose 排。
- 變更抽屜與已封存檢視一體生效：RichDetailDrawer 提案／設計分頁與 ArchivedList 提案／設計分頁重用同一章節檢視元件。
- 任務分頁群組標題調成同款標籤樣式：TaskList 群組標題由粗體 h4 改為與章節標籤同款式；群組標題文字是使用者內容，不翻譯、不改寫。
- 標籤款式為粗體大標題（實作驗收後使用者比對裁定，取代首版的小字大寫 muted 標籤）：計算字級大於內文，套用同一款式家族——提案／設計章節、討論輪欄位（焦點／立場等）、討論結論欄位（決定／理由等）、規格分頁色標區段標頭（保留各色）、已封存討論檢視的區段標題——五處引用同一主標題常數。
- 任務群組標題採款式家族的次級款（第二次比對裁定）：粗體、字級與內文基準一致（16px）——即 Spectra 任務清單的原尺寸；與 Capabilities 次級標籤共用同一常數。
- 相容性影響：純前端呈現層變更（packages/ui），speclink-core／speclink-cli 不動，CLI 人眼與 --json 輸出逐位元不變；來源 markdown 檔案不回寫。

## Non-Goals

- 章節卡片化（每章一張卡）——提案／設計是連續論述，卡片切碎閱讀流，已否決。
- 只調樣式不映射中文——介面仍混英文工程詞，與 UI 文字規範抵觸，已否決。
- 規格分頁——delta 色標已於 drawer-document-readability 完成，requirement 原文照排是已裁決的 A 案，不動。
- 討論抽屜各分頁的結構（輪卡片切分、結論欄位切分、背景 prose）不動——本刀僅統一其標籤「款式」為大標題。
- 模板章節的中文對譯不回寫任何 markdown 來源檔，僅為渲染層映射。

## Capabilities

### New Capabilities

（無）

### Modified Capabilities

- `desktop-app`: 新增章節呈現需求——變更抽屜與已封存檢視的提案／設計分頁將已知模板章節以中文標籤區塊呈現（英文模板標題不直出、未知章節照排），任務分頁群組標題與章節標籤同款式；標籤款式為粗體大標題（計算字級大於內文），與討論側欄位標籤同款。

## Impact

- Affected specs: desktop-app
- Affected crates: 無（純前端，speclink-core／speclink-cli 不動）
- Affected code:
  - New: packages/ui/src/components/SectionedDoc.tsx
  - Modified: packages/ui/src/components/RichDetailDrawer.tsx、packages/ui/src/components/ArchivedList.tsx、packages/ui/src/components/TaskList.tsx、packages/ui/src/components/DiscussionDrawer.tsx、packages/ui/src/components/DeltaBadges.tsx、packages/ui/src/i18n.tsx、packages/ui/src/__tests__/richDrawer.test.tsx、packages/ui/src/__tests__/archivedList.test.tsx、packages/ui/src/__tests__/taskList.test.tsx、packages/ui/src/__tests__/components.test.tsx、packages/ui/src/__tests__/discussionDrawer.test.tsx
  - Removed: （無）
- 新增依賴：（無）

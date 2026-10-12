## Summary

變更詳情頁（含已封存變更）六份文件換上各自的渲染器：提案與設計依章節白名單逐段畫（引言段、細線列、編號步驟、方案／落敗理由、決策卡、合約鍵值列、風險提示框）、任務分群成卡且每列拆標題／內文／驗證三層、規格 delta 沿用正式規格的需求段並加修改色條與移除 scenario 提示框、排程與工單加上數字磚與推導提示框——全部由現有 Markdown 與工單 JSON 的結構推出，原始格式與 schema 一字不動。

## Motivation

- 討論 detail-content-redesign 的結論（cut 2）：cut 1 `desktop-detail-doc-tree` 已封存，三欄骨架、閱讀卡標題區、數字磚與提示框的容器都就位，但六份變更文件仍以 cut 1 之前的畫法落在新殼裡——提案與設計只有章節標籤加散文、任務是一條平面清單、規格 delta 只有色標標頭、排程與工單的閱讀卡標題下是空的。使用者在討論中裁定的驗收標準是「每份文件第一屏看得出層次、項目型內容用介面元件、散文只佔該佔的位置」，這一刀把變更側六份文件推到這條線。
- 討論同時裁定三條落地規則，全部落在這一刀：章節白名單留在前端（三種提案形狀與設計模板的聯集），任何二級標題都切成一段、進本頁目錄、算進數字磚，白名單命中才有翻譯與專屬畫法，整份沒有二級標題才退回單一 Markdown 檢視；退回規則改逐項降級（替代方案切不到「——」當一般列、影響段不符 Affected code 形狀當一般列、數字磚缺資料少一格）；提示框只放推得出的事實（能力有幾條需求被改寫、移除了哪些 scenario、有幾個手動任務、末輪只剩 SUGGESTION 可蓋章、排程的可以開工／等 N 項兩態）。
- 目標使用者：在桌面 app 讀變更內容的開發者與 PM（apply 前看提案與設計、apply 中看任務、封存前看規格 delta、排程與兩站工單）。落點為 apps/desktop 的詳情頁所用的 packages/ui 元件，宿主只多接兩條回呼（開正式規格、在 Finder 顯示路徑）。不涉及 CLI 指令、`--json` 輸出、設定欄位、技能檔與 server。
- 互動原型：https://claude.ai/artifact/1QhDWmQi6muFFQnHTqqHa3 最底三列的提案、設計、任務、規格、排程、審查六張畫板（資料取自 tray-panel-restyle 與其蓋章前的審查工單快照），各畫板右側 Tweaks 有深色模式。

## Proposed Solution

- 章節切分改為「任何二級標題都成段」：白名單命中的章節顯示中文標籤與專屬畫法，白名單外的章節以原文為標題照 prose 排，整份沒有二級標題才整篇退回 Markdown。章節標題統一為 16px 粗體（取代 20px 大標籤），閱讀卡內文 15px、行高 1.75。
- 提案渲染器：摘要成引言段（灰底）、動機與成功準則成細線列、變更內容與解法成編號步驟、非目標帶禁止圖示、替代方案以「——」拆成「方案／落敗理由」兩段、能力段的能力名成可點籤（有 delta 規格時點擊跳到規格文件並選定該能力）、影響段依規格與新增／修改／移除分組列等寬路徑籤（hover 出複製鈕，宿主有本機目錄時另出 Finder 鈕）。數字磚依章節取前四格（解法步驟、非目標、成功準則、能力、影響的檔案、替代方案依序取存在的）。能力段之下的提示框：修改的能力有幾條需求被改寫、幾個 scenario 以 REMOVED-SCENARIO 宣告。閱讀卡籤槽標出模板形狀（Feature／Bug Fix／Refactor）。
- 設計渲染器：背景成細線列、目標與非目標兩欄、決策段的每個三級標題一張可收合卡（D 編號籤、列點數；任務的 D 籤跳到對應卡並展開）、實作合約成鍵值列、風險與取捨逐條 warning 提示框、遷移計畫與開放問題照 prose。數字磚：決策、目標／非目標、風險與取捨、開放問題。
- 任務渲染器：每個群組一張卡（序號、標題、小進度 n/m），任務列拆「標題／內文／驗證」三層，預設只顯示標題，列尾箭頭展開內文與驗證（以「驗證：」切分，切不到時整段當標題）；標題內的 design D 籤可點；手動任務列改為整列暖黃淡底＋列尾「手動」徽章（不再獨佔描述上方一行），暖黃為新增的手動語意色，看板「待手動」章與手動提示框同色；整列可勾、拖曳把手、批次工具列與 n 快捷鍵行為不變。數字磚：任務、已完成、手動、群組；手動任務存在時暖黃提示框列出編號；籤槽「全部完成」或 n/m。
- 規格 delta 渲染器：每個能力一列能力頭（等寬名稱、delta 計數、「看正式規格」鈕開規格詳情頁），各 delta 區段的色標標頭之下每條需求沿用規格詳情頁的需求段（Requirement 小標、標題、需求文、左線縮排的 Scenario）並加該操作色的左色條；REMOVED-SCENARIO 宣告以 caution 提示框列出刪除線的 scenario 名；SHALL 等關鍵字等寬灰底；Scenario 的 WHEN／THEN 成兩欄；需求文超過三段收合標「展開其餘 N 段」、scenario 超過三個收合標「還有 N 個」；第一條需求展開、其餘收合為一列。數字磚：觸及的能力、修改的需求、移除的 scenario、新增／更名。
- 排程文件：四卡維持 data-plan-section 與全部互動（前置增減、引擎拒絕的單行錯誤、唯讀），卡頭改 16px 章節標題帶計數籤；加四格數字磚（波次、宣告的前置、同名需求重疊、封存前要等）與兩態提示框（tip「可以開工，也可直接封存」／warning「等 N 項前置，封存前要等 M 個變更」）；籤槽帶「可以開工」或「等 N 項」。缺 wave 的兩種退回不放磚與提示框。
- 工單文件：閱讀卡籤槽為「第 N 輪」與階段詞，數字磚為輪數與末輪三級計數，末輪無 CRITICAL 與 WARNING 時 tip 提示框「可以蓋章」；每輪一張卡（輪數、階段詞、範圍檔數、條數、嚴重度計數籤，展開後列範圍檔與每條發現）；發現列自描述前綴取出軸向（Correctness／Standards／Coherence）成籤，嚴重度色章、等寬路徑、原文與「已接受」籤不變；末輪展開、其餘收合。
- 已封存變更詳情頁共用同一批渲染器（唯讀：任務勾選框停用、排程文件不在已封存頁）。
- 相容性影響：只改桌面 app 的呈現層；無 CLI、`--json`、設定欄位、技能檔與 server 變動；既有 vitest 題目依新結構改錨點；無 golden 回歸對照變動。

## Non-Goals

- 不改任何 artifact 的 Markdown 格式、schema 模板與 discuss scaffold；不讀 schema 模板取章節名。
- 討論側四份文件（結論、討論過程、背景、衍生變更）與其數字磚、提示框屬 cut 3；閱讀卡的出處行維持 cut 1 的定義（檔案路徑或「由 speclink plan 推算」），不加段。
- 規格詳情頁與手冊頁的畫法不動（規格詳情頁只把需求段抽成共用元件，外觀不變）；server-web 不受影響。
- 不加底部「上一份／下一份」；左欄文件樹不列段落；不改文件樹、狀態列、標題列動作與「開啟 ▾」。
- 任務列「驗證：」切分的英文對應不在本刀（討論已擱置）。

## Alternatives Considered

- 保留 SectionedDoc 的「白名單外章節併入 prose」——討論裁定任何二級標題都成段、進本頁目錄；手寫自訂章節才看得到目錄。
- 整篇退回只看「有無白名單章節」——討論裁定改為「整份沒有二級標題才退回」，自訂章節的提案不再退成一團散文。
- 任務列三層改以「：」之外的規則（如第一句）切標題——tasks.md 既有寫法是「標題：內文；驗證：方式」，以「：」切標題、「驗證：」切驗證命中率最高，切不到就整段當標題，不猜。
- 工單的段標題列（站名、第 N 輪、三級計數）保留在內文——閱讀卡標題已是站名、籤槽放輪數與階段詞、數字磚放計數，再留一條就是同一資訊畫三次。
- 一刀連討論側一起做——cut 2 與 cut 3 平行、spec delta 互不重疊是討論裁定的切法。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 修改「提案與設計章節以中文標籤呈現」（任何二級標題成段、白名單專屬畫法、16px 章節標題、數字磚與提示框、能力籤與路徑籤）、「任務群組標題與章節標籤同款式」（群組卡、三層任務列、D 籤、數字磚與手動提示框）、「規格分頁 delta 區段以色標呈現」（能力頭、需求段與色條、REMOVED-SCENARIO 提示框、WHEN／THEN 兩欄、收合、數字磚）、「詳情頁的排程分頁」（數字磚、兩態提示框、籤槽、卡頭款式）、「詳情頁的工單分頁」（籤槽、數字磚、可蓋章提示框、輪卡、軸向籤）、「markdown 內容保留文件結構呈現」（閱讀卡內文 15px 行高 1.75）、「markdown 文件內容行寬有上限」（閱讀卡內以卡欄 768px 為上限）、「任務列的手動任務徽章」（整列淡底＋列尾徽章，取代上方獨佔一行）、「介面狀態語意色分層」（新增待手動暖黃系）。

## Impact

- Affected specs: desktop-app（9 條 MODIFIED）
- Affected code:
  - New: packages/ui/src/docModel.ts、packages/ui/src/components/DocParts.tsx、packages/ui/src/components/ProposalDoc.tsx、packages/ui/src/components/DesignDoc.tsx、packages/ui/src/components/RequirementBlock.tsx、packages/ui/src/components/useDocNav.ts、packages/ui/src/components/PlanDoc.tsx、packages/ui/src/components/changeDocSummary.tsx、packages/ui/src/__tests__/docModel.test.ts、packages/ui/src/__tests__/proposalDoc.test.tsx、packages/ui/src/__tests__/designDoc.test.tsx、packages/ui/src/__tests__/deltaSpecView.test.tsx
  - Modified: packages/ui/src/components/DeltaBadges.tsx、packages/ui/src/components/TaskList.tsx、packages/ui/src/components/TicketView.tsx、packages/ui/src/components/SpecDetailPage.tsx、packages/ui/src/components/ChangeDetailPage.tsx、packages/ui/src/components/ArchivedDetailPage.tsx、packages/ui/src/components/DetailShell.tsx、packages/ui/src/components/Markdown.tsx、packages/ui/src/components/DiscussionDetailPage.tsx、packages/ui/src/specSections.ts、packages/ui/src/i18n.tsx、packages/ui/src/index.ts、packages/ui/src/theme.css、packages/ui/src/tone.ts、packages/ui/src/components/ui/callout.tsx、packages/ui/src/components/ChangeCard.tsx、packages/ui/src/__tests__/awaitingManualBadge.test.tsx、packages/ui/src/__tests__/stage.test.ts、packages/ui/src/__tests__/detailShell.test.tsx、packages/ui/src/__tests__/taskList.test.tsx、packages/ui/src/__tests__/planTab.test.tsx、packages/ui/src/__tests__/ticketView.test.tsx、packages/ui/src/__tests__/changeDetailPage.test.tsx、packages/ui/src/__tests__/archivedDetailPage.test.tsx、packages/ui/src/__tests__/components.test.tsx、packages/ui/src/__tests__/discussionDetailPage.test.tsx、apps/desktop/src/App.tsx、apps/desktop/src/store.ts、apps/desktop/src/__tests__/App.test.tsx、apps/desktop/src/__tests__/store.test.ts
  - Removed: packages/ui/src/components/SectionedDoc.tsx

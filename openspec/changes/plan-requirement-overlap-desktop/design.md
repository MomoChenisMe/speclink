## Context

第一刀之後，speclink-core 的 PlanChange 帶 `requirement_overlap` 與 `archive_after`，而且兩個欄位已經走上 wire：speclink-protocol 的 PlanChangeEntry 為九欄（兩個新欄位皆 serde default），speclink-remote 的 plan_report 與 server 的 plan handler 逐欄搬運（原訂在本刀的 wire 群組於第一刀的品質站階段提前完成）。桌面 local 清單由 apps/desktop/core 的 query 模組把 plan 列的四欄寫進每個變更項的 JSON；remote 清單由 src-tauri 的 remote 模組把 GET /plan 的列合併進 ChangeSummary。packages/ui 的 ChangeItem 型別帶四欄，stage 模組的 planWave／planBlockedBy 是唯一讀取入口，RichDetailDrawer 的 PlanTab 依此渲染四段（波次、前置、重疊、阻擋），版面是小標題加純文字；審查／驗證分頁的 TicketView 用 `rounded-md border border-border/60 bg-muted/20` 的邊框卡片，標題列與內容以 `border-t` 分格。

## Goals / Non-Goals

**Goals:**

- 桌面 local 與 remote 清單六欄同進同出，值全部來自引擎同一入口（remote 經已帶兩欄的 GET /plan）。
- 排程分頁改為卡片式四段，讓「為什麼被擋」「封存排誰後面」「哪個 requirement 撞名」一眼可見。
- 桌面讀舊 server：缺席的兩個欄位讀作空陣列（沿 wire 型別的 default）。

**Non-Goals:**

- 引擎、CLI 與 wire（protocol、server、remote convert；皆在第一刀）、卡片波次章與系統匣視覺、remote 臂的 change rank、桌面的 strict 開關。

## Decisions

### 桌面清單六欄同進同出

apps/desktop/core 的 query 模組在 plan 列存在時多寫 `requirementOverlap` 與 `archiveAfter` 兩鍵（成環或壞 meta 時與其他四欄一起缺席，dependsOn 的成環例外維持）；src-tauri 的 remote 模組的排程欄組加同兩欄。packages/ui 的 ChangeItem 加 `requirementOverlap?: ChangeRequirementOverlap[]` 與 `archiveAfter?: string[]`；stage 模組新增 planArchiveAfter(c) 與 planRequirementOverlap(c)，與 planBlockedBy 同一規則（wave 缺席即空陣列），PlanTab 只讀結果。

### 排程分頁卡片化，阻擋併入前置卡的狀態徽章

PlanTab 的每一段改為一個 `<section data-plan-section="…">`，外觀沿 TicketView 的邊框卡片：`rounded-md border border-border/60 bg-muted/20`，標題列 `flex items-center justify-between px-2 py-1 text-xs font-semibold uppercase tracking-wider text-muted-foreground`，內容區 `border-t border-border/60 px-2 py-1.5 text-sm`。四段與 data-plan-section 值：

| 段 | data-plan-section | 標題列 | 內容 |
| --- | --- | --- | --- |
| 波次 | wave | 「波次」 | 「第 N 波」＋同波夥伴名（PlanName 晶片），無夥伴時「本波只有這個變更」 |
| 前置 | depends | 「前置」＋右側狀態徽章 | dependsOn 每項一列附移除鈕（capability 為真時）、底部新增下拉；空時「尚無前置」 |
| 重疊 | overlaps | 「重疊」 | requirementOverlap 每項一列：對方 PlanName、`capability › requirement` 文字、雙方操作各一枚小標籤（`rounded border border-border/60 px-1 py-0.5 text-[10px]`）、conflict 時加一枚「同名衝突」destructive 色標籤；空時「無重疊」 |
| 封存順序 | archive | 「封存順序」 | archiveAfter 每項 PlanName，下方一句「先封存它們，再對照正典重寫同名 requirement 後封存本變更」；空時「可直接封存」 |

前置卡標題列右側的狀態徽章沿既有 Badge 元件：blockedBy 空時 variant secondary 文字「可以開工」；非空時 variant outline 文字「等 N 項」、title 為 planBlockedLabel 的前置名。原本的 `data-plan-section="blocked"` 段移除。wave 缺席的兩種說明句（remote 未取得 plan、成環）維持原樣，成環時只渲染前置卡（無徽章、無新增下拉）。捨棄「維持段落只換標題徽章」（使用者選卡片化）與「重疊卡繼續顯示目錄級 overlaps」（第一刀後它不再影響任何順序，顯示只會誤導）。

### i18n 詞條

新增 tw／en：`plan.archiveAfter`「封存順序」／"Archive order"、`plan.archiveHint`「先封存它們，再對照正典重寫同名 requirement 後封存本變更」／"Archive them first, then rewrite the same-named requirement against the canon before archiving this change"、`plan.noArchiveAfter`「可直接封存」／"Nothing to wait for"、`plan.conflict`「同名衝突」／"Name conflict"、`plan.blockedCount`「等 {n} 項」／"Waiting on {n}"。改寫：`plan.overlaps` 維持「重疊」／"Overlaps"、`plan.noOverlaps` 維持。移除：`plan.blocked`（阻擋段標題不再使用）。`plan.canStart` 改用於徽章。

## Implementation Contract

**Behavior**

- remote 專案的排程分頁與 local 對同一 scope 內容顯示相同的四段內容；舊 server（無新欄位）時重疊卡「無重疊」、封存順序卡「可直接封存」，其餘不變。
- 開啟 wave=1、dependsOn=[]、blockedBy=[]、requirementOverlap=[{change:"add-b",capability:"desktop-app",requirement:"看板與任務",ownOperation:"MODIFIED",otherOperation:"MODIFIED",conflict:false}]、archiveAfter=["add-b"] 的變更：波次卡「第 1 波」；前置卡徽章「可以開工」、內容「尚無前置」與新增下拉；重疊卡一列 add-b、`desktop-app › 看板與任務`、兩枚 MODIFIED 標籤；封存順序卡列 add-b 與提示句。
- 開啟 blockedBy=["add-a","add-c"] 的變更：前置卡徽章「等 2 項」、title 含 add-a、add-c；頁面不存在 data-plan-section="blocked"。
- conflict 為 true 的重疊列多一枚「同名衝突」標籤。

**Interface / data shape**

- GET /plan 回應沿第一刀：changes 每項九鍵、requirementOverlap 每項六鍵 camelCase、缺席讀作空陣列（本刀只讀，不改 wire）。
- 桌面 local 與 remote 清單項：wave、blockedBy、dependsOn、overlaps、requirementOverlap、archiveAfter 六欄同進同出；頂層 planError 不變。
- packages/ui：ChangeItem 新兩欄可選；stage 模組匯出 planArchiveAfter、planRequirementOverlap。

**Failure modes**

- plan 缺席（舊 server、成環、請求失敗）：六欄一起缺席，分頁行為與第一刀前對缺席的處置相同。
- 新欄位缺席但 wave 存在（舊 server）：兩卡顯示空態文案，不報錯。

**Acceptance criteria**

- speclink-desktop-core query 測試：清單項六欄同進同出。
- src-tauri remote_data 測試：merge_plan 帶六欄。
- packages/ui planTab 測試：四張卡片的 data-plan-section 與內容、前置卡徽章兩態、conflict 標籤、舊 server 空態；planBadge 測試不變。
- apps/desktop App 與 remoteCapabilities 測試：remote 分頁排程分頁唯讀且顯示新卡片。

**Scope boundaries**

- In：桌面 local／remote 清單注入、ui 型別與 PlanTab、i18n、對應測試。
- Out：引擎、CLI、wire（protocol、server plan handler、remote convert，第一刀已完成）、卡片章、系統匣、change rank remote 臂。

## Risks / Trade-offs

- [排程分頁 DOM 改變讓既有 planTab 測試整批紅] → 同批改寫測試，以 data-plan-section 四值為錨。
- [舊 server 與新桌面：兩卡永遠空態] → 接受：空態文案明確，且第一刀 CLI 對舊 server 同樣退化。
- [跨平台] → 純型別與前端改動，無路徑或 git 假設；vitest 與 cargo 測試在三平台同行為。
- [main 上 desktop lib 測試耗時] → 只跑受影響 target（apps/desktop 的 vitest、speclink-desktop-core 的 query 測試、src-tauri 的 remote_data）。

## Migration Plan

1. 桌面清單 → ui，逐層落地並各自跑受影響測試。
2. 依賴第一刀落地（depends_on: plan-requirement-overlap，wire 的兩個欄位在那裡）；回退為 revert 本 change 的 commit。

## Open Questions

（無。）

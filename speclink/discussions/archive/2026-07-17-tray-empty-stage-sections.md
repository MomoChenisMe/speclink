---
topic: desktop tray 空資料時常駐顯示生命週期分區
slug: tray-empty-stage-sections
status: promoted
promoted_to: tray-empty-stage-sections
created: 2026-07-17
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: desktop tray 空資料時常駐顯示生命週期分區

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者提供 tray 面板截圖：全無變更時三個生命週期分區（提案中／進行中／已就緒）不顯示，只見「尚無進行中變更」佔位卡，希望空資料時分區也顯示出來。模式選 assumptions——掃得 TrayPanel.tsx、tray.ts、KanbanBoard.tsx、DiscussionColumn.tsx 等 3+ 相關原始碼。相關規格：tray-status-menu——「生命週期分區與變更進度」明文「每個非空階段 SHALL 有一個分區標題」「全無變更時 SHALL 顯示明確空狀態文字」，現狀是規格行為而非 bug。對照組：討論分區已是常駐＋計數 0（design D8「空狀態與非空同構」）。純 UI 空狀態呈現、無新架構縫，跳過介面深度檢查。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-17)

**Focus**: tray 面板全無資料時，生命週期分區（提案中／進行中／已就緒）是否應常駐顯示
**Position**: 三個生命週期分區改為常駐（標題＋計數 0），比照討論分區的 design D8 空狀態同構：
- 現狀成因：TrayPanel.tsx:179-182 把空階段 filter 掉，全空時收合成「尚無進行中變更」佔位卡——tray-status-menu 規格明文行為，非 bug
- 討論分區已有現成模式（TrayPanel.tsx:298-309）：標題＋計數 0、min-h-12 垂直置中
- 「尚無進行中變更」佔位卡移除，由三張計數 0 分區卡取代
- 部分有資料時空階段也常駐——分區位置固定、面板不隨資料增減跳動
- 全空時面板高度增約 100px；Rust 側高度自適應，無技術障礙
- ASCII 三態對照（現狀／提議全空／提議部分有資料）呈現後，使用者未異議並續問已轉出分區
**Ruled out**: 保留佔位卡與空分區卡並存——同一訊息重複表達；僅處理「全空」特例——部分有資料時空分區仍消失，版面照樣跳動
**Open**: 已轉出分區（tray 與看板兩側）是否也常駐；Windows/Linux 原生選單是否跟進空分區常駐

### Round 2 — assumptions (2026-07-17)

**Focus**: 「已轉出」分區是否也升格常駐（使用者提議統一顯示）
**Position**: 維持有料才現——統一線劃在「工作站常駐、衍生群組有料才現」，經使用者裁定：
- 前提修正：看板（DiscussionColumn.tsx:353 收合列）與 tray 現狀彼此一致，皆有料才現——不存在兩面分歧，題目實為「已轉出要不要升格工作站」
- 已轉出非工作站，是轉為變更的副產品；生命週期設計就是自動歸零（最後一個衍生變更封存時討論自動封存）
- 空工作站是行動邀請（「可以開提案」）；空已轉出無可行動，常駐「已轉出 0」反而暗示是該填滿的階段
- 看板設計意圖本來就壓低其存在感（討論欄底收合列、預設收合、不持久化）
- tray 補齊四工作站常駐後，兩面心智模型對齊，看板零改動
- 其餘假設（原生選單不動、佔位卡移除、部分有資料時空分區亦常駐）使用者一併確認
**Ruled out**: 已轉出升格常駐（tray＋看板）——空狀態無訊息量、與收合設計意圖相反，使用者裁定放棄
**Open**: none

## Conclusion

**Decision**: tray 面板（macOS）三個生命週期分區（提案中／進行中／已就緒）改為常駐顯示——標題＋計數 0，比照討論分區的 design D8 空狀態同構；「尚無進行中變更」佔位卡移除；部分有資料時空階段亦常駐、分區位置固定。「已轉出」分區維持有料才現（tray 與看板皆不動）；Windows/Linux 原生選單維持現狀。
**Rationale**: 統一線劃在「工作站常駐、衍生群組有料才現」——空工作站是行動邀請且版面不隨資料增減跳動；已轉出是轉為變更的副產品、設計上自動歸零，常駐計數 0 反而誤導。全空時面板高度增約 100px，Rust 側高度自適應無技術障礙。
**Rejected alternatives**: 保留佔位卡與空分區卡並存（同一訊息重複表達）；僅處理全空特例（部分有資料時版面照樣跳動）；已轉出升格常駐（空狀態無訊息量、與看板收合設計意圖相反）；原生選單跟進空分區標題（原生下拉選單中空灰標題是噪音、不符平台慣例）。
**Deferred**: none
**Capture to**: proposal（tray-status-menu 規格「生命週期分區與變更進度」需修訂）
**Next**: /speclink-propose --from-discussion tray-empty-stage-sections

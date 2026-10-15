---
topic: 詳情頁的審查／驗證章籤在身分區與狀態區重複顯示
slug: detail-quality-stamp-dedup
status: promoted
created: 2026-10-13
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: detail-quality-stamp-dedup
---

# Discussion: 詳情頁的審查／驗證章籤在身分區與狀態區重複顯示

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在變更詳情頁看到「審查中」「驗證中」章籤同時出現在左欄身分區（名稱下方、階段籤旁）與「狀態」區的「品質關卡」列，同一欄裡重複兩次。需求一開始就清楚（去掉重複），不需要 grill 階段；要決定的是保留哪一處。重複是正式規格寫明的：desktop-app 的「變更詳情頁的標頭與狀態卡」要求兩處都顯示，已封存詳情頁的需求同樣在身分區放結局籤、在「狀態」區放品質關卡結局。在途變更 doc-tiles-fill-and-round-order: desktop-app, discussion-stage-and-unpromoted-cuts: desktop-app, task-row-expand-by-done: desktop-app 都在改 desktop-app，但沒有碰這兩條需求。
Prior discussions: detail-content-redesign, desktop-ui-redesign, drawer-quality-ticket-tab

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-13)

**Focus**: 章籤重複時保留哪一處，以及連帶要搬什麼
**Position**: 使用者確認五條假設全對——保留「狀態」區的「品質關卡」列，拿掉身分區的審查與驗證章籤，變更詳情頁與已封存詳情頁一起改，另開小變更。
- 理由與既有規則同一條線：規格已寫「身分區 SHALL NOT 顯示任務計數——任務進度由『狀態』列承載」；身分區回答「這是什麼」（名稱、階段、出身），「狀態」區回答「做到哪了」，品質關卡屬於後者
- 「品質關卡」列資訊較多：直出蓋章日期，兩站皆 none 時有「尚未經過品質關卡」空態；身分區在 none 時什麼都不畫
- 身分區章籤的主題化提示（蓋章日期＋含 email 的蓋章者完整識別）搬到「品質關卡」列的章籤上，資訊不流失（ChangeDetailPage.tsx 的 TitleStamp 與 StatusRow mark="quality"）
- 已封存詳情頁同樣有重複（ArchivedDetailPage.tsx 的 outcomes 同時餵身分區 badges 與品質關卡列），一起改
- 不動：身分區的階段籤（與看板欄位對應的分類）、左欄「審查」「驗證」文件節點（工單入口）、看板卡片章籤
- 載體：另開小變更，不併入未開工的 task-row-expand-by-done——兩件事改不同元件與不同規格需求，分開好審查
- 重開 detail-content-redesign 的「身分區：階段籤與兩站章」：那次沒談過兩處要不要合併；兩處是從抽屜時代（標題旁章籤＋遠處狀態卡）沿用過來，搬進同一欄後才變成肉眼可見的重複
**Ruled out**: 保留身分區章籤、拿掉品質關卡列——none 時看不到任何品質資訊，日期要停留才看得到；只改變更詳情頁——兩種詳情頁身分區長得不一樣；併入 task-row-expand-by-done——混兩件不相關的事

## Conclusion

**Decision**: 變更詳情頁與已封存詳情頁的審查／驗證章籤只留在「狀態」區的「品質關卡」列，身分區拿掉章籤、只留階段籤；只改 desktop 顯示。
- 變更詳情頁：身分區只剩階段籤（已就緒、進行中等），不再畫審查與驗證章籤；「品質關卡」列照舊呈現各站章籤與蓋章日期（inReview／inVerify 只有狀態詞，兩站皆 none 時灰字「尚未經過品質關卡」）
- 蓋章者提示搬家：原本掛在身分區章籤上的主題化提示（蓋章日期＋含 email 的蓋章者完整識別）改掛在「品質關卡」列的章籤上；進行中、無戳記時提示缺席；可視文字仍不直出 email
- 已封存詳情頁：身分區只剩「已封存」籤，審查／驗證結局籤只留在「品質關卡」列；兩站皆無時照舊灰字「未經品質關卡」
- 不動：身分區階段籤、左欄「審查」「驗證」文件節點（含「第 N 輪」徽章）、看板卡片與已封存清單的章、品質站配色、提示的 300ms 延遲
- 例：reviewed＋verified、蓋章者含 email 的變更 → 身分區只有「已就緒」；「品質關卡」列為「已審查 2026-10-12」「已驗證 2026-10-13」，停留在「已審查」上顯示蓋章日期與含 email 的完整識別。reviewStatus 為 inReview 的變更 → 身分區只有「進行中」；「品質關卡」列為「審查中」、無日期、無提示
- 規格（desktop-app，4 條 MODIFIED）：「變更詳情頁的標頭與狀態卡」（身分區改為只有階段籤、提示改掛品質關卡列，改寫情境「章籤與品質關卡列」「品質關卡列的空態與進行中」）、「已封存項目以詳情頁檢視」（身分區去掉結局籤與對應情境字面）、「詳情頁的文件樹、閱讀卡與本頁目錄」（身分區描述與情境字面去掉章籤）、「詳情頁的工單分頁」（蓋章後「身分區章籤改為『已審查』」改為「品質關卡列的章籤改為『已審查』」）；「主題化提示統一延遲」的「變更詳情頁的章籤」仍成立，不改
- 與在途變更的重疊：doc-tiles-fill-and-round-order 也 MODIFIED「詳情頁的文件樹、閱讀卡與本頁目錄」；它已就緒、會先封存，本變更的 delta 要以它封存後的正典字面為底，否則先封存者的修改會被整塊覆蓋
- 程式落點：packages/ui 的 ChangeDetailPage（TitleStamp 退場、提示併入品質關卡列章籤）與 ArchivedDetailPage（outcomes 只餵品質關卡列）；既有 data 標記 data-review-row／data-verify-row／data-review-tone／data-verify-tone 跟著章籤搬到品質關卡列，reviewBadge／verifyBadge／changeDetailPage／archivedDetailPage 測試的取點跟著改
**Rationale**: 身分區回答「這是什麼」（名稱、階段、出身），「狀態」區回答「做到哪了」；規格對任務計數已經做過同樣的切分（身分區不顯示任務計數，由「狀態」列承載），品質關卡是同一類資訊。「品質關卡」列直出日期、有空態，資訊比身分區章籤完整，所以留它；兩處並存是從抽屜時代（標題旁章籤＋遠處狀態卡）沿用下來，搬進同一欄後才變成肉眼可見的重複。
**Rejected alternatives**:
- 保留身分區章籤、拿掉「品質關卡」列 — 兩站皆 none 時看不到任何品質資訊，日期要停留才看得到
- 只改變更詳情頁 — 兩種詳情頁身分區長得不一樣
- 併入未開工的 task-row-expand-by-done — 兩件事改不同元件與不同規格需求，混在一起難審查
**Deferred**: none
**Capture to**: spec | tasks
**Next**: /speclink-propose --from-discussion detail-quality-stamp-dedup（等 doc-tiles-fill-and-round-order 封存後再寫 delta）

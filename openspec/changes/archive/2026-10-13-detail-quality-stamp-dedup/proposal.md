## Summary

變更詳情頁與已封存詳情頁的審查／驗證章籤只留在左欄「狀態」區的「品質關卡」列，身分區拿掉章籤、只留階段籤（或「已封存」籤）。

## Motivation

目標使用者是在 desktop 上看變更進度的開發者、PO 與 PM，使用情境是 `/speclink-review`、`/speclink-verify` 兩站跑完後，回到詳情頁確認一個變更「做到哪了」。

詳情頁改成三欄骨架之後，左欄同一欄裡出現兩次相同的章籤：身分區（名稱下方、階段籤旁）一次，「狀態」區的「品質關卡」列又一次。這個重複是正式規格寫明的——desktop-app 的「變更詳情頁的標頭與狀態卡」要求兩處都顯示；已封存詳情頁的「已封存項目以詳情頁檢視」同樣在身分區放結局籤、在「狀態」區放品質關卡結局。兩處並存是從抽屜時代（標題旁章籤＋遠處狀態卡）沿用下來，搬進同一欄後才變成肉眼可見的重複。

身分區回答「這是什麼」（名稱、階段、出身），「狀態」區回答「做到哪了」。規格對任務計數已經做過同樣的切分（身分區 SHALL NOT 顯示任務計數，由「狀態」列承載），品質關卡是同一類資訊。「品質關卡」列直出蓋章日期、兩站皆 none 時有空態，資訊比身分區章籤完整，所以留它。

規格掃描結果：相關的正式規格只有 desktop-app，本變更改它的四條需求；沒有其他能力涉及詳情頁章籤。

## Proposed Solution

- **變更詳情頁**（packages/ui 的 ChangeDetailPage）：
  - 身分區只剩階段籤（已就緒、進行中等），不再畫審查與驗證章籤。
  - 「品質關卡」列照舊呈現各站章籤與蓋章日期；inReview／inVerify 只有狀態詞、無日期；兩站皆 none 時灰字「尚未經過品質關卡」。
  - 原本掛在身分區章籤上的主題化提示（蓋章日期＋含 email 的蓋章者完整識別）改掛在「品質關卡」列的章籤上；進行中、無戳記時提示缺席；可視文字仍不直出 email。
  - 身分區用的 TitleStamp 退場，「品質關卡」列改用同一個元件渲染章籤（章籤＋日期＋提示），兩條渲染路徑合成一條。
- **已封存詳情頁**（packages/ui 的 ArchivedDetailPage）：身分區只剩「已封存」籤；審查／驗證結局籤只在「品質關卡」列渲染一次；兩站皆無時照舊灰字「未經品質關卡」。
- **既有 data 標記跟著章籤搬家**：data-review-row／data-verify-row／data-review-tone／data-verify-tone 改落在「品質關卡」列的章籤上；已封存側的 data-review-outcome／data-verify-outcome 只剩「品質關卡」列一份。
- **規格**：desktop-app 四條需求 MODIFIED，見 Impact。

**相容性影響**：

- 人眼：身分區少了章籤；「品質關卡」列外觀不變，多了停留提示。
- `--json`：無。本變更不碰 CLI、protocol 或任何資料形狀，ChangeItem 的 reviewStatus／verifyStatus／reviewedAt／reviewedBy 等欄位不變。
- 回歸對照：DOM 上的 data 標記位置改變，packages/ui 的 reviewBadge、verifyBadge、changeDetailPage、archivedDetailPage 四份測試的取點跟著改；沒有外部使用者依賴這些標記。

## Non-Goals

design.md 會建立，範圍外的項目與否決的做法記在 design 的 Goals/Non-Goals。

## Alternatives Considered

- 保留身分區章籤、拿掉「品質關卡」列——兩站皆 none 時看不到任何品質資訊，日期要停留才看得到。
- 只改變更詳情頁——兩種詳情頁的身分區會長得不一樣。
- 併入未開工的 task-row-expand-by-done——兩件事改不同元件與不同規格需求，混在一起難審查（該變更已另行封存）。

## Impact

- Affected specs: desktop-app（MODIFIED 四條：「變更詳情頁的標頭與狀態卡」「已封存項目以詳情頁檢視」「詳情頁的文件樹、閱讀卡與本頁目錄」「詳情頁的工單分頁」）
- Affected apps: packages/ui（詳情頁元件與測試）；apps/desktop 只透過 packages/ui 受影響，自身程式碼不改。
- Affected code:
  - Modified:
    - `packages/ui/src/components/ChangeDetailPage.tsx`
    - `packages/ui/src/components/ArchivedDetailPage.tsx`
    - `packages/ui/src/__tests__/changeDetailPage.test.tsx`
    - `packages/ui/src/__tests__/archivedDetailPage.test.tsx`
    - `packages/ui/src/__tests__/reviewBadge.test.tsx`
    - `packages/ui/src/__tests__/verifyBadge.test.tsx`
  - New: (none)
  - Removed: (none)

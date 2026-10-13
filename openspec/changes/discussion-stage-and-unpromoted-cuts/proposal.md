## Summary

討論詳情頁的階段步驟列改成與身分區狀態籤同色同義；結論文件以橄欖色（手動任務色）提示框與刀卡標示還沒轉出的刀。

## Motivation

目標使用者是在 desktop 透過 AI 代理跑 SDD 流程的開發者（PO／PM 亦同），情境是 discuss 收斂、以 `/speclink-propose --from-discussion` 分刀轉出的期間，回到討論詳情頁確認「這份討論走到哪、下一刀是什麼」。

- 身分區狀態籤在保留中與尚無結論時是琥珀色，左欄「階段」步驟列的現站卻固定主色綠：同一個狀態出現兩種顏色，讀者以為是兩件事。
- 已轉出但尚無結論的討論，狀態籤寫「已轉出・尚無結論」，步驟列卻把「已結論」畫成打勾的已過站，兩者直接矛盾。
- 結論「分刀」區段裡還沒轉出的刀沒有任何標示：讀者要自己拿刀列的變更名去比 `promoted_to` 才知道哪刀還沒做，轉出下一刀的指令也只藏在「下一步」欄位裡。

兩項由討論 discussion-stage-and-unpromoted-cuts 定案。正式規格現在寫「結論文件 SHALL NOT 有提示框」，本變更有意識地偏離：先前討論 detail-content-redesign 的判準是「提示框只放推得出的事實」，未轉出的刀由結論分刀列比對 `promoted_to` 直接推出，與衍生變更文件的「只差最後一刀」同類；當時的「七處」是數量，不是上限。

## Proposed Solution

- 步驟列現站的色調共用身分區狀態籤的判準：已轉出・保留中與已轉出・尚無結論為琥珀（warning），其餘維持主色；現站文字不變（「保留在途」列照舊解釋保留中）。
- 已轉出但尚無結論的討論，步驟列的「已結論」站畫成未到站的灰圈，不打勾。
- 刀是否轉出只比對刀列的變更名是否在 `promoted_to`；刀列沒寫變更名時不標示；不推算還欠幾刀。
- 整刀未轉出（刀列至少一個變更名、且全都不在 `promoted_to`）的刀卡改為橄欖色虛線外框＋淡橄欖底，刀編號旁加「尚未轉出」籤；同刀部分轉出時外框照舊，只在未轉出的變更名旁加「尚未轉出」籤。
- live 討論的結論文件，有整刀未轉出的刀時，數字磚之下加橄欖色提示框「N 刀尚未轉出」：每刀一列（刀編號籤、範圍句、「前往 →」捲到該刀卡），末行說明這幾刀還沒有變更，並附可複製的轉出指令 `/speclink-propose --from-discussion <slug>`；版型比照任務文件的手動任務提示框。已封存討論頁不出提示框，刀卡標示照常。
  - 例：wad-plugins-merge-or-split 結論有 cut 1、cut 2，`promoted_to` 只有 cut 1 的 add-team-plugin-library → 提示框標題「1 刀尚未轉出」、列出 cut 2 與「前往 →」；cut 2 卡為橄欖色虛線框、帶「尚未轉出」籤；cut 1 卡照舊實線框、變更名旁為「進行中」階段籤。
- 「尚未轉出」籤以 StatusPill 新增的 manual 色調呈現，不以 className 覆蓋既有色調。
- 規格：desktop-app「討論詳情頁檢視與轉出變更」補步驟列現站色調與尚無結論時的「已結論」站；「討論結論以欄位標籤呈現」移除「結論文件 SHALL NOT 有提示框」、加入未轉出提示框與刀卡標示，並改寫「分刀卡與變更籤」與「結論文件的數字磚與出處行」兩個 scenario。

相容性影響：只改 desktop 與共用 UI 套件的呈現。CLI 人眼輸出與 `--json`、引擎、討論記錄格式、server API 全部不變；i18n 新增 tw／en 文案鍵；既有使用者不需遷移。

## Non-Goals

- 衍生變更文件不列「預定中、還沒轉出」的刀（現行只列 `promoted_to` 的變更，維持不變）。
- 不推算「還欠幾刀」或依子變更狀態推論系列進度（先前討論 multi-cut-discussion-hold 已否決此類推論）。
- 身分區狀態籤的判準與色調不變；看板討論卡與系統匣不改。
- 討論記錄的 markdown、引擎的 hold／promoted_to 語意不變。

## Alternatives Considered

- 狀態籤改回主色綠以配合步驟列 — 看板討論卡用同一套琥珀判準（規格明文），改了與看板不一致。
- 未轉出刀只用灰色籤加虛線外框 — 使用者認為不夠明顯。
- 未轉出刀用琥珀色 — 琥珀在本 app 代表需要注意，保留中的系列有刀未轉出是正常狀態。
- 未轉出刀與提示框用藍色（note 色） — 使用者選橄欖色，且手動任務色「要你親自動手」的語意更貼切：轉出下一刀是使用者發起的動作。
- 已封存討論頁也出提示框 — 封存記錄不能再轉出，指令按了無效。
- 併進 doc-tiles-fill-and-round-order — 檔案不重疊、變更名不準確。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 「討論詳情頁檢視與轉出變更」的階段步驟列現站改與狀態籤同色、尚無結論時「已結論」站不打勾；「討論結論以欄位標籤呈現」加入未轉出刀的提示框與刀卡標示，移除「結論文件 SHALL NOT 有提示框」。

## Impact

- Affected specs: desktop-app（2 條 MODIFIED）
- Affected app／package: `packages/ui`（共用 UI 元件，由 apps/desktop 渲染）；speclink-core、speclink-cli、speclink-host、server 皆不受影響
- Affected code:
  - Modified: `packages/ui/src/discussionModel.ts`、`packages/ui/src/components/DiscussionDetailPage.tsx`、`packages/ui/src/components/ConclusionDoc.tsx`、`packages/ui/src/components/discussionDocSummary.tsx`、`packages/ui/src/components/ui/status-pill.tsx`、`packages/ui/src/i18n.tsx`、`packages/ui/src/__tests__/discussionModel.test.ts`、`packages/ui/src/__tests__/statusPill.test.tsx`、`packages/ui/src/__tests__/conclusionDoc.test.tsx`、`packages/ui/src/__tests__/discussionDetailPage.test.tsx`、`packages/ui/src/__tests__/archivedDetailPage.test.tsx`
  - New: (none)
  - Removed: (none)

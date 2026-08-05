## Why

審查蓋章的「已審查」狀態直接引用主題主色(REVIEW_TONE.reviewed = text-primary),而主色 teal 同時承載進度條、分頁、連結籤、生命週期階梯等全 app 品牌視覺——蓋章淹沒在滿版 teal 裡,品質站標記的辨識度失效(討論 card-drawer-header-colors 的起點截圖即此問題)。討論定案三層色彩角色規則:teal=連結/互動/進度、語意色=狀態、中性=靜態 metadata;蓋章屬狀態,須讓出 teal,改用全 app 零佔用的紫色系,並確立「紫=品質站蓋章專屬」的分工。

目標使用者是透過 AI 代理跑 SDD 的開發者、PO 與 PM;使用情境為在看板卡片、變更詳情抽屜與已封存頁一眼辨識變更的審查狀態——對應 review 品質站與 archive 收尾階段。

## What Changes

- packages/ui/src/components/reviewStyle.tsx 的 REVIEW_TONE.reviewed 由 text-primary 改為 text-violet-600 dark:text-violet-400(與既有 tone 家族的 -600/dark:-400 階梯模式一致);其餘三態(inReview sky、reviewedStale amber、reviewedNotPassed rose)不動。
- 同檔注解更新:記載「紫=品質站蓋章專屬」分工(一般成功/新增語意由 emerald 承載,屬後續全域色彩系統變更),與後續驗證站(verify-station-parity)共用同一張 tone 表的約定。
- 消費端零改動:卡片、詳情抽屜、已封存清單、已封存抽屜、封存對話框五個表面引用同一份 REVIEW_TONE,自動生效。
- packages/ui/src/__tests__/reviewBadge.test.tsx 四處 text-primary 斷言(卡片章、抽屜資訊列、已封存清單、已封存抽屜)改為新色契約——四個表面共用同一份 REVIEW_TONE,換色會一起變動。

## Non-Goals

- 不動其餘三態的色值與圖示;不動 REVIEW_LABEL_KEY 詞條。
- 不涉及驗證站的 tone 表(verify-station-parity 以 ingest 承接「tone 共用+盾牌系圖示」的討論定案)。
- 不涉及全域色彩系統收斂(teal 越界退中性、錯誤態琥珀改紅、語意常數表、theme.test 守門收緊、死碼清理等)——屬同一討論扇出的第二個變更。
- 討論已排除方案不再考慮:emerald/green 作蓋章色(青綠鄰域辨識弱、emerald 已被 delta 新增徽章佔用)、各站各配一色(8 色稀釋狀態語意)、換色塞進 verify-station-parity(被 19 個未開工任務扣住)。

## Capabilities

### New Capabilities

(無)

### Modified Capabilities

- `desktop-app`: 新增「品質站蓋章配色與主色分離」約束——「已審查」蓋章以紫色系呈現且不得引用主題主色,紫色系為品質站蓋章專屬;既有審查標示的狀態、詞條、圖示行為不變。

## Impact

- Affected specs: `desktop-app`
- Affected code:
  - New: (無)
  - Modified: packages/ui/src/components/reviewStyle.tsx、packages/ui/src/__tests__/reviewBadge.test.tsx
  - Removed: (無)
- 影響的 app/套件:packages/ui(改動本體);apps/desktop 隨套件更新,無自身程式碼改動。
- 相容性影響:純視覺;CLI 人眼輸出與 --json 零變化,golden 與 CLI 測試不受影響。

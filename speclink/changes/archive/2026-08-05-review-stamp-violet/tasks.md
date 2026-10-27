<!-- 依專案 TDD 慣例:先改測試錨定新契約(紅燈),再改實作轉綠。 -->

## 1. 蓋章換紫

- [x] 1.1 [測試先行] packages/ui/src/__tests__/reviewBadge.test.tsx 的四處 reviewed 斷言(卡片章、抽屜審查資訊列、已封存清單、已封存抽屜)由 text-primary 改為 text-violet-600 契約,並增列深色變體斷言(dark:text-violet-400)——契約:規格「品質站蓋章配色與主色分離」的可區辨場景有測試錨點,新斷言紅燈;驗證:npm test -w @speclink/ui 顯示該案例失敗、其餘案例不受影響。 <!-- speclink-task:tsk_01KZ5MDNZXZBNN3SKY84P4EK76 -->
- [x] 1.2 packages/ui/src/components/reviewStyle.tsx 的 REVIEW_TONE.reviewed 改為 text-violet-600 dark:text-violet-400,注解同步改寫:記載「紫=品質站蓋章專屬(規格同名約束);一般成功/新增語意由 emerald 承載,屬全域色彩系統變更;驗證站與本表共用同一張 tone 表」——契約:任務 1.1 轉綠,inReview/reviewedStale/reviewedNotPassed 三態斷言不變;驗證:npm test -w @speclink/ui 全綠。 <!-- speclink-task:tsk_01KZ5MDNZX3FYTFBKZH4BDMDFV -->
- [x] 1.3 全套驗證與手動確認:npm run build -w apps/desktop 與 npm run build -w apps/server-web 通過(@speclink/ui 為純原始碼套件、無自身 build script,實際編譯它的是這兩個消費端);建置桌面 app 後開啟 reviewStatus 為 reviewed 的變更,確認卡片章、詳情抽屜審查資訊列、已封存側三處蓋章呈紫且與進度條/分頁的 teal 可區辨,深淺主題皆然——驗證:逐條核對規格三個場景。 <!-- speclink-task:tsk_01KZ5MDNZX2WMQQB27GPZEB2XV -->

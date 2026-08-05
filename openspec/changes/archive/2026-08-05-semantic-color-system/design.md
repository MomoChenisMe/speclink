## Context

三路審計(討論 card-drawer-header-colors Round 3)給出全域違規清單:packages/ui 有 teal 靜態越界與三紅並存;apps/desktop 的 sky/emerald/rose 零使用,錯誤大量塗琥珀,4 處原生色階繞過 token;apps/server-web 的 token 治理乾淨但語意詞彙萎縮,狀態徽章全灰。主題單一真相源(packages/ui/src/theme.css,雙 app 同 import、系統匣面板同管道、Rust 端零色值)已確認,收斂只需補齊詞彙、集中常數、收緊守門。全案純呈現層:Rust crates、CLI、server API 零改動。

前置關係:review-stamp-violet(蓋章換紫)獨立在途,本變更不觸碰 reviewStyle.tsx;change-drawer-header-redesign 已落地共用來源籤(同源籤 teal 已達成);verify-station-parity 的驗證章 tone 由 ingest 另行釘定。worktree-toggle-and-guards 已落地的卡片 worktree 標示(text-primary/60)晚於三路審計,由討論 worktree-color-semantics 補位裁定歸「進行中」語意。

## Goals / Non-Goals

**Goals:**

- 三層色彩角色規則全域落地:主色只做連結/互動/進度,狀態一律語意色,靜態一律中性。
- 語意色單一來源常數表,守門測試阻止未來漂移。
- 後台狀態徽章可辨識;錯誤不再穿琥珀;進行中/成功有自己的顏色。
- 清除四個無消費端的舊元件。

**Non-Goals:**

- 不動 reviewStyle.tsx(避免與 review-stamp-violet 同檔互踩)、不動 stage.ts 階梯值、不動 theme.css token。
- 不重排任何版面;不抽新的共用 React 元件(揭示橫幅兩處同樣式即可,不為兩處建抽象)。

## Decisions

**D1 語意色常數表(packages/ui/src/tone.ts)。** 輸出 `SEMANTIC_TONE` 四鍵:inProgress=text-sky-600 dark:text-sky-400、success=text-emerald-600 dark:text-emerald-400、warning=text-amber-600 dark:text-amber-500、danger=text-destructive;另輸出面色版 `SEMANTIC_SURFACE`(同語意的 border+bg 淡色組,供橫幅/卡片用,如 warning=border-amber-500/40 bg-amber-500/10)。表頭註記三紅分工:錯誤訊息與危險動作=destructive token、品質站未通過章=rose(reviewStyle)、delta 刪除=red(DeltaBadges)——三表各司其職不合併,rose/red 保留各自 -600/dark:-400 階梯模式。自 packages/ui/src/index.ts 匯出供兩 app 使用。取捨:CSS token(--success 等)可讓 hover 變體更自由,但現行慣例是 TS 常數表(reviewStyle/stage/DeltaBadges),加 token 層等於兩套機制並存——討論已裁定常數表。

**D2 守門收緊(packages/ui/src/__tests__/theme.test.ts)。** 現行 PALETTE 白名單放行一切原生色階,改為:掃描 packages/ui/src、apps/desktop/src、apps/server-web/src 的 .ts/.tsx 原始碼(排除 __tests__ 與 dist),原生語意色階字面(text|bg|border|ring|from|to)-(sky|amber|emerald|rose|red|teal|green|violet|purple|orange|yellow|fuchsia)-<階> 僅允許出現於集中常數檔白名單:packages/ui/src/tone.ts、packages/ui/src/components/reviewStyle.tsx、packages/ui/src/components/DeltaBadges.tsx、packages/ui/src/stage.ts(stage.ts 現況只用 primary,列入為防未來);違規時測試失敗並列出檔名+class。此測試先行收緊即為本變更的紅燈:現存違規(TrayPanel/App/ProjectTabs/RemoteWorkspaceRecovery/RemoteConflictDialog/InstructionUpdatePrompt/ProjectSettingsView/AppSettingsView 的 amber 字面、ServersPanel 的 teal-700 與 red-600、connectionLogin 的 red-600、ChangeCard 的 amber-500、HighlightText 修正後的 amber 若硬編)全數被列出,後續任務逐批修掉轉綠。

**D3 各表面處置以審計清單為準。** proposal What Changes 逐面列妥;實作原則:(1)語意色一律 import SEMANTIC_TONE/SEMANTIC_SURFACE,不再寫色階字面;(2)「選取」與「警示」分離——系統匣作用中分頁以 border-primary/60 表達選取,狀態另由列內語意色圖示承載;(3)needs-reauth/stale 屬警示維持 amber,error/access-denied/not-found 屬錯誤改 destructive;(4)已撤銷徽章=outline+中性文字,「有效」側改綠承擔區辨,不用刪除線;(5)看板卡片 worktree 標示(GitBranch 圖示)改 import SEMANTIC_TONE.inProgress——worktree 掛著=工作正於副本進行中,屬狀態非靜態 metadata,不為 worktree 在常數表加鍵(單點使用直接引用 inProgress);抽屜的分支+路徑維持 meta 列中性(掃視層搶眼、閱讀層安靜)。取捨:中性化被使用者否決(一眼辨識功能優先);orange 雖有 git 品牌心智,但與同列 restale 的 amber 警示在 14px 圖示下難辨且誤讀為警示——出自討論 worktree-color-semantics。

**D4 頭像中性化五處。** bg-primary text-primary-foreground 改 bg-muted text-muted-foreground(ChangeCard、RichDetailDrawer、DiscussionColumn、ArchivedList、DiscussionDrawer);首字母與圓形不變。

**D5 tooltip 反色。** ui/tooltip.tsx 氣泡由 bg-primary text-primary-foreground 改 bg-foreground text-background(shadcn 傳統深色氣泡),全域 tooltip 一次生效;SourceDiscussionChip 提示內的 slug/topic 行不需調整(繼承前景色)。

**D6 死碼清理範圍。** 刪 packages/ui/src/components/ 的 ChangeBoard.tsx、ChangeList.tsx、ChangeListItem.tsx、DetailDrawer.tsx;index.ts 移除 ChangeBoard/ChangeBoardProps、DetailDrawer/DetailDrawerProps、ChangeListItem/ChangeListItemProps、ChangeList/ChangeListProps/ListView 匯出;測試面刪 changeListItem.test.tsx 整檔、components.test.tsx 的 ChangeBoard 區塊、kanban.test.tsx 的 DetailDrawer 區塊。刪除前 grep 確認 ListView 等型別無其餘消費端;若有,保留該型別並於任務中記錄。

## Implementation Contract

**觀察行為:**

- 桌面 app:分頁 error/更新失敗/設定檔解析錯誤呈紅(destructive);還原中/遷移中/更新下載中/看板卡片 worktree 標示呈藍;遷移成功呈綠;stale/needs-reauth/政策衝突維持琥珀;系統匣「討論/已轉出」分區標題與計數為中性,生命週期三分區維持主色階梯;「重新登入」按鈕為中性 outline;搜尋高亮為琥珀 mark;tooltip 氣泡深底白字;頭像灰底。
- Web 後台:儲存健康徽章 online 綠/offline 紅;成員 active 綠/停權琥珀;PAT 與裝置憑證有效綠/已撤銷中性;PAT 揭示與邀請連結橫幅綠系。
- 主題守門:任何非白名單檔出現原生語意色階字面,npm test -w @speclink/ui 失敗並指名檔案。

**介面/資料形:** 新匯出 SEMANTIC_TONE/SEMANTIC_SURFACE(@speclink/ui);移除四個死碼元件匯出。無其他 API 變化。

**失敗模式:** 純樣式,無新失敗路徑;守門測試失敗即建置紅燈,不影響執行期。

**驗收條件:** npm test -w @speclink/ui、npm test -w apps/desktop、npm test -w apps/server-web 全綠;npm run build -w apps/desktop 與 npm run build -w apps/server-web 通過;手動走查深淺主題與系統匣 vibrancy 底上的對比。

**範圍邊界:** in scope=上列三套件的換色、常數表、守門、死碼;out of scope=reviewStyle.tsx、stage.ts 階梯值、theme.css、任何版面結構、Rust/CLI/server。

## Risks / Trade-offs

- [兩 app 既有測試斷言舊色而失效] → 各批次採測試先行:先改斷言錨定新契約(紅),再實作(綠);不放寬斷言。
- [系統匣 HudWindow 半透明底上中性/語意色對比不足] → 手動走查列入 6.x 驗收;必要時就地微調透明度,不回退語意。
- [守門誤殺(regex 撞非色彩字串)] → 白名單以檔案為單位+regex 錨定 Tailwind class 型式;誤殺案例加測試內註解排除規則。
- [回歸對照] → CLI 與 golden 零影響;跨平台為純 web 呈現。
- [死碼移除影響外部消費者] → @speclink/ui 為 monorepo 內部套件,兩 app grep 零引用;無外部發佈通道。

## Migration Plan

無資料遷移。@speclink/ui 匯出面縮減僅影響 monorepo 內部,消費端零引用已確認。

## Open Questions

(無)

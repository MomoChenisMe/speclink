---
topic: 卡片與抽屜 header 的色彩系統:品質站蓋章換色與其它項目的色系搭配
slug: card-drawer-header-colors
status: promoted
promoted_to: review-stamp-violet, semantic-color-system, verify-station-parity
created: 2026-08-04
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 卡片與抽屜 header 的色彩系統:品質站蓋章換色與其它項目的色系搭配

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者以卡片與抽屜截圖發起:審查蓋章「已審查」直接引用 text-primary(reviewStyle.tsx:10),與 app 主色 teal 完全同色,辨識度失效;要求換色,且並行品質站 verify-station-parity(0/19 未開工)的驗證章配色一併裁定。隨後範圍擴大:卡片與抽屜 header 其它項目的顏色色系搭配一併檢討。

模式:assumptions——reviewStyle.tsx、ChangeCard.tsx、RichDetailDrawer.tsx、ArchivedList/ArchivedDrawer、theme.css、reviewBadge.test.tsx 脈絡充足。

相關 change/spec:正典 desktop-app 的審查標示條文未釘色值(配色屬實作層);verify-station-parity design D5「同構呈現」未釘 verify 章色值;change-drawer-header-redesign(提案中)即將重排抽屜 header 四層結構,色系結論與其相關。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-04)

**Focus**: 品質站蓋章(審查/驗證)的換色裁決
**Position**: 四項假設全數成立(使用者確認):
- 修法落點:reviewStyle.tsx 的 REVIEW_TONE.reviewed 由 text-primary 換色,卡片/詳情抽屜/已封存清單/已封存抽屜/封存對話框五個消費面自動生效
- 色值:violet-600(深色 violet-400)——grep 證實 violet/purple/fuchsia 全 app 零使用;與既有 tone 的 -600/dark:-400 模式一致;「官印」語意
- 兩站共用同一張 tone 表:顏色承載狀態(sky=進行中/violet=通過/amber=其後有變動/rose=未通過)、圖示形狀承載站別(審查=Badge 徽章系、驗證=盾牌系);卡片兩章並排時「兩顆紫」=雙站通過
- 落地拆兩段:已出貨的審查章換色開獨立小 change(reviewStyle.tsx 一行+reviewBadge.test.tsx:185,200 兩處 text-primary 斷言);verify-station-parity(0/19 未開工)以 ingest 在 design D5 釘「tone 值共用、圖示採盾牌系」
- 佐證:正典 desktop-app 審查標示條文未釘色值→換色零 spec delta;DeltaBadges 的 added 已佔 emerald-600,綠家族雙重佔用,更排除 emerald
**Ruled out**:
- emerald/green 作蓋章色——仍在青綠鄰域(主色 teal oklch 192),小圖示尺寸辨識弱,且 emerald 已被 delta「新增」徽章佔用
- 各站各配一色——8 色 palette,「琥珀=其後有變動」的狀態語意被站色稀釋
- 換色塞進 verify-station-parity——已出貨表面的修正被 19 個未開工任務扣住
**Open**:
- 卡片與抽屜 header 其它項目的色系搭配(使用者本輪擴大範圍)

### Round 2 — assumptions (2026-08-04)

**Focus**: 色彩角色規則明文化,與卡片/抽屜 header 僅剩兩個違規者的裁決
**Position**: 三層色彩角色規則成立,兩個違規者依建議修正(使用者確認):
- 規則一 teal 主色=連結與進度:可點導航(來源討論籤、分頁、動作鈕)、進度(進度條、任務徽章)、連結語意指示(卡片討論徽章 text-primary/60)
- 規則二 語意色=狀態:品質站章(sky/violet/amber/rose)、delta 徽章(emerald/amber/red/sky)、restale amber、metaError destructive;狀態色不挪作他用
- 規則三 中性灰=靜態 metadata:名字、工具、時間、開工(現況已符)
- 裁決(a):建立者頭像(卡片+抽屜)bg-primary 改 bg-muted 中性化——非互動非連結非進度,不再搶主色
- 裁決(b):同源籤 bg-muted 改 teal(bg-primary/10)與來源討論籤一致——「可點連結籤」單一長相,討論/變更之別由前綴字「來自/同源」承載,不以色彩重複編碼
**Ruled out**:
- 頭像保留 teal 作品牌點綴——teal 密度降不下來,與「讓蓋章讓出 teal」的方向矛盾
- 同源籤保留灰以色分連結對象——前綴字已承載該區分,色彩雙重編碼是浪費
**Open**:
- 使用者擴大範圍:整個 desktop 介面+web 後台(server-web console)依同一套三層規則全面判定一次

### Round 3 — assumptions (2026-08-04)

**Focus**: 全域(共用 UI 套件/desktop 殼層/web 後台)三路色彩審計的結果與判讀
**Position**: 病灶不是亂用色,是「語意色詞彙萎縮」——三層規則裡的狀態色只實作了一半,teal 與 amber 承包過載:
- desktop 殼層(apps/desktop/src):sky/emerald/rose 零使用;「進行中」全用 teal spinner、「成功」全用 teal、「錯誤」大量溢到 amber(ProjectTabs error、RemoteWorkspaceRecovery 失敗卡、AppSettings 更新錯誤、ParseErrorBanner 等);另有 4 處繞過 token 直用原生 teal-*/red-*;theme.test.ts 的 PALETTE 白名單放行原生色階,是漂移能長期全綠的原因
- web 後台(apps/server-web):token 治理最乾淨(零硬編色),但語意詞彙同樣萎縮——sky/amber/emerald 全站零出現,健康/啟用停權/有效撤銷等狀態徽章全壓成兩種灰;PAT/邀請揭示橫幅借 primary 當成功色;OverviewPage 同頁「儲存離線」上橫幅 destructive、下徽章中性灰自相矛盾
- 共用套件(packages/ui):AnalyzePanel 的 Warning 徽章=bg-primary/15(注解明寫「Warning 主色」,同檔 72 行同語意已用 amber);teal 靜態越界一批(ChangeList 計數實心 teal、DiscussionDrawer 輪次籤、tooltip 氣泡實心 teal、DiscussionColumn 硬編平行 teal 階未走 stage.ts);頭像 bg-primary 漏網三處(DiscussionColumn:133、ArchivedList:104、DiscussionDrawer:379);三種紅並存(destructive token/rose 章/red delta);amber 階梯三種寫法(ChangeCard:116 無 dark 變體)
- 對掃描的兩處糾正:stage.ts 生命週期 teal 階梯(看板欄+系統匣共用單一來源)判定為「進度」角色的合法用法,不動;複製勾號 text-primary(全域 7 處)判定為互動即時回饋,合規——TrayPanel 的討論/已轉出分區「借用」STAGE_BADGE.proposed 才是違規(非生命週期卻穿生命週期的衣服)
- 主題同源性:desktop 與 server-web 共用唯一 theme.css(oklch 192),tray 面板也走同一 CSS 管道,Rust 端零色值——單一真相源基礎良好,收斂可行
**Open**:
- 六項裁決:成功語意色=emerald?;落地機制=packages/ui 集中 TS 語意常數表+收緊 theme.test 守門?;搜尋高亮改琥珀?;tooltip 氣泡改反色中性?;三紅分工(訊息/按鈕=destructive、章=rose、delta=red)?;落地拆分與執行順序

### Round 4 — assumptions (2026-08-04)

**Focus**: 六項裁決與死碼處置的定案
**Position**: 六項全數依建議通過,死碼由「提一句」升級為「清理」(使用者裁定):
- 成功語意色=emerald(紫=品質站蓋章專屬、emerald=一般成功/新增,delta added 前例)
- 落地機制=packages/ui 集中 TS 語意常數表(比照 reviewStyle/stage.ts 慣例,不加 CSS token),theme.test.ts 的 PALETTE 白名單守門收緊為 token+集中常數檔
- 搜尋高亮 bg-primary/20 改琥珀 mark
- tooltip 氣泡改反色中性(深底白字),解除與 STAGE_BADGE.ready 撞色
- 三紅分工:錯誤訊息與危險按鈕=destructive token、品質站未通過章=rose、delta 刪除=red,各常數表加註分工
- 執行順序:蓋章換紫小 change 先出 → 在途 change-drawer-header-redesign 先 apply(結構) → 全域色彩系統 change 最後(著色+機械修正批+後台語意徽章+死碼清理),避免同檔互踩;verify-station-parity 以 ingest 釘 tone 共用+盾牌系圖示
- 死碼清理:ChangeBoard/ChangeList/ChangeListItem/DetailDrawer 四檔(grep 證實 apps 零引用;ChangeListItem 僅被死掉的 ChangeList 引用),連同 index.ts 匯出與專屬測試,併入全域色彩 change 一次清除——清掉後色彩守門的審計基準面才乾淨
**Open**: 無——進入結論

## Conclusion

**Decision**: 確立三層色彩角色規則並全域收斂。(1) 規則:teal 主色=連結/互動/進度(含 stage.ts 生命週期深淺階梯與複製勾號互動回饋);語意色=狀態(sky=進行中、emerald=成功/新增、amber=警示/其後有變動、rose/red/destructive=未通過/刪除/錯誤與危險動作、violet=品質站蓋章專屬);中性=靜態 metadata。(2) 品質站:REVIEW_TONE.reviewed 由 text-primary 改 violet-600/dark:violet-400;審查與驗證兩站共用同一張 tone 表,色承載狀態、圖示形狀承載站別(審查=Badge 徽章系、驗證=盾牌系)。(3) 卡片/抽屜:建立者頭像全域中性化(bg-muted,含卡片、變更抽屜、討論卡、封存卡、討論抽屜五處);同源籤改 teal 與來源討論籤一致。(4) 機械修正批約 40 處:teal 靜態越界退中性、錯誤態琥珀改 destructive、進行中改 sky、原生色階改 token、後台狀態徽章補語意色、琥珀階梯統一 amber-600/dark:amber-500。(5) 搜尋高亮改琥珀 mark;tooltip 氣泡改反色中性。(6) 落地機制:packages/ui 集中 TS 語意色常數表,theme.test.ts 白名單守門收緊。(7) 死碼清理:ChangeBoard/ChangeList/ChangeListItem/DetailDrawer 四檔連同匯出與專屬測試移除。
**Rationale**: 審計證實病灶是「語意色詞彙萎縮」——sky/emerald/rose 在 desktop 殼層與 web 後台零使用,teal 與 amber 承包一切,蓋章淹沒在主色裡只是最顯眼的症狀;地基(單一 theme.css、tray 同管道、Rust 零色值)乾淨,收斂只需補齊詞彙+集中常數+收緊守門。紫作蓋章色:violet 全 app 零佔用、與 -600/dark:-400 家族模式一致、官印語意;emerald 不作蓋章色但作一般成功色,與 delta added 前例一致。
**Rejected alternatives**: emerald/green 作蓋章色(青綠鄰域辨識弱且已被 delta 佔用);各站各配色(8 色稀釋狀態語意);換色塞進 verify-station-parity(被 19 個未開工任務扣住);CSS token 落地(現行慣例是 TS 常數表,加 token 層多一套機制);三紅強行合併(破壞章家族深淺階梯);頭像保留 teal 品牌點綴(teal 密度降不下來);同源籤保留灰(前綴字已承載區分);tray 生命週期階梯改語意色(它是進度角色的合法用法,且為看板/系統匣單一來源)。
**Deferred**: 死碼四檔的專屬測試與匯出清單細節、機械修正批的逐處最終清單——留給 propose 階段盤點;emerald 具體暗階(dark:400 vs 500)與後台徽章樣式細節留給 design。
**Capture to**: proposal(扇出兩個新變更:蓋章換紫小變更、全域色彩系統收斂變更)+既有變更 verify-station-parity(以 ingest 釘 tone 共用與盾牌系圖示)
**Next**: /speclink-propose --from-discussion card-drawer-header-colors(先蓋章小變更,再全域收斂變更);speclink discuss link card-drawer-header-colors verify-station-parity 後 /speclink-ingest verify-station-parity

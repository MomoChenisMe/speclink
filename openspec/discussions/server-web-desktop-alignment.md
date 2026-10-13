---
topic: server 網頁後台（server-web）的 UI／UX 調整成與已全面改版的 desktop 一致的設計風格
slug: server-web-desktop-alignment
status: open
created: 2026-10-13
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: server 網頁後台（server-web）的 UI／UX 調整成與已全面改版的 desktop 一致的設計風格

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

desktop 已依 desktop-ui-redesign 全面改版（左側標題列、詳情頁取代抽屜、PageHeader、設定零件、列式清單與換頁），server 網頁後台（apps/server-web）的殼、版面與互動仍停在 admin-console-redesign 的樣子，使用者要求兩邊的設計風格與 UI／UX 一致。未經 grill 階段：需求有具體參考對象（desktop 現況與 desktop-ui-redesign 的二十張畫板原型），可直接列假設。相關規格為 server-web-console（主體）與 desktop-app（參考）；進行中變更 clickable-cursor-hover-rule 的 delta 觸及 clickable-cursor-hover-rule: server-web-console（ADDED「網頁後台可點元素顯示手指游標」）。
Prior discussions: admin-console-redesign, desktop-ui-redesign, web-service-navigation-redesign

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-13)

**Focus**: server-web 對齊 desktop 的範圍與五個方向（範圍、殼、頁面版面與元件、抽屜、密度）
**Position**: 五條假設全數確認：server-web 全面對齊 desktop 的殼、版面、元件與互動，顏色與資訊架構不動。
- 決策地圖：1 範圍 → 2 殼 → 3 頁面版面與元件 → 4 抽屜改詳情頁 → 5 密度 → 6 分刀（待定）
- 範圍：管理殼、六個管理頁、帳號頁、登入／setup／邀請／啟用四個流程頁；六個目的地與網址不變；顏色字型不動——兩邊 index.css 已 import 同一份 packages/ui/src/theme.css（admin-console-redesign 否決換色票的理由仍成立）
- 殼：頂部橫貫 header（ConsoleLayout.tsx）改成 desktop 形狀——側欄頂端 48px 放品牌、主區頂列放麵包屑（後台 / 使用者）、email／語言／登出收進側欄底部帳號選單（dropdown-menu 原語）；不做 56px 圖示列（後台無多專案切換）；一般成員也有只含品牌＋帳號選單的窄側欄，殼只有一種形狀；HeaderAccount.tsx「不引入 dropdown 原語」的理由已因 packages/ui 已有 dropdown-menu 而失效
- 頁面版面：頁標題改 PageHeader（24px 一般字重＋灰字說明＋右側主要動作）；使用者、專案與儲存庫、憑證三個列表頁以 ListCard＋ListRow＋ListPager＋SearchField 取代 server-web 自有的 DataList＋ListToolbar（全寬、卡內捲動、底部換頁列）；稽核紀錄欄多保留表格但放進同一種 ListCard、同列高與細線；帳號、系統、總覽為 768px 置中＋SettingsCard／SettingsRow；四個流程頁對齊 desktop 零專案空狀態與新增專案對話框的卡；證據為 server-web 目前零使用 PageHeader、Settings*、ListRow／ListCard／ListPager、SearchField、DropdownMenu、StatusPill
- 互動：右滑抽屜（DetailSheet）改為詳情頁——主區換成該筆資料的頁、有自己的網址、麵包屑「後台 / 使用者 / <email>」、內容用設定型卡與列；邀請、建立專案、建立存取金鑰改置中對話框（同 desktop 新增專案）；破壞性動作確認框不變；重開 admin-console-redesign 的抽屜裁定，因其理由「與 desktop 同語彙」已反轉——desktop-ui-redesign 讓 Sheet 自桌面退場，server-web 成為唯一抽屜使用者
- 密度：字級與控件高度與 desktop 相同（外殼 11–13px、閱讀內文 15px、按鈕 28–36px）；改寫 server-web-console「正文 ≥16px、互動目標 ≥44×44px」——無任何測試檢查、共用 Button 本為 h-9／h-7、全站只有 header email 連結有 min-h-11；重開 web-service-navigation-redesign「44px 觸控目標為驗收條件」，理由是承諾從未被共用元件遵守且一致性需要同一套控件；鍵盤操作、focus ring、4.5:1 對比、skip link 保留
- 受影響規格（server-web-console）：「全部 browser route 由單一 SPA 提供可發現導覽」（帳號入口、一般成員側欄）、「共用設計系統維持高密度可存取體驗」（16px／44px）、「管理列表以抽屜承載建立與編輯」（整條改寫）
**Ruled out**: 保留抽屜只換外觀——兩個產品「點一列」反應不同，正是要消掉的差異；列表保留表格——清單外觀仍兩套；做 56px 圖示列——後台無多專案，圖示列會是空欄；保留 44px／16px——server-web 控件要比 desktop 大一號，一致只能做一半；一般成員維持無側欄——殼會有兩種形狀
**Open**: 窄螢幕（<1024px）導覽是否保留 Sheet；詳情頁的網址形狀與規格路由清單；首次導覽（Tour）步驟是否受殼改動影響；是否先做互動原型當視覺基準；分刀

### Round 2 — interview (2026-10-13)

**Focus**: propose 前是否先做互動原型當各刀的視覺基準；順帶以程式碼查清三個事實節點
**Position**: 先做互動原型再分刀與 propose，約 7 張畫板，追加在 desktop 原型同一份 artifact。
- 畫板：殼＋總覽（管理員）、使用者列表、使用者詳情頁、邀請對話框、帳號頁（一般成員窄側欄）、登入頁、375px 窄螢幕
- 理由：desktop-ui-redesign 與 detail-content-redesign 皆先以原型逐面確認、各刀 design 以原型為準；這次有三樣 desktop 沒有現成樣子——無圖示列的殼、側欄底部帳號選單、以設定卡排的使用者詳情頁
- 事實（查程式碼定案，不需使用者裁決）：窄螢幕（<1024px）導覽保留 Sheet——desktop 無窄螢幕模式可對齊，導覽抽屜與要拿掉的資料詳情抽屜是兩回事，web-service-navigation-redesign 亦定行動版 Sheet；Tour 每步指向側欄六個目的地的 data-tour，帳號移位不影響；詳情頁網址 /admin/users/<id>、/admin/registry/<代號> 落在規格既有的 /admin/* 內，路由清單不改
- 抽屜盤點：DetailSheet 共 6 處用途——使用者細節、專案細節改詳情頁；邀請使用者、建立專案、建立存取金鑰改對話框；憑證頁只有確認框不動
- 測試影響：users.test.tsx（49 處）、registry.test.tsx（23 處）、account.test.tsx（13 處）提及對話框或抽屜，需隨之改寫
**Ruled out**: 不做原型直接 propose——畫面要到 apply 後才看得到，改動成本較高
**Open**: 原型確認後的修正；分刀

## Conclusion

<!-- Written by `speclink discuss conclude`:
**Decision** / **Rationale** / **Rejected alternatives** / **Deferred** / **Capture to** / **Next** -->

---
topic: desktop 導覽與專案首頁重構
slug: desktop-導覽與專案首頁重構
status: promoted
promoted_to: desktop-nav-reorder, desktop-config-card
created: 2026-07-07
---

# Discussion: desktop 導覽與專案首頁重構

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者連提兩個桌面殼需求:(1) 移除備忘導覽項、已封存鈕移入側欄規格下方、設定同列,順序:變更>規格>已封存>設定;(2) config 設定區重設計為 Spectra「一進專案」的專案設定＋產出規則頁。模式:假設(殼層程式碼)＋實測(computer use 開 Spectra 2.3.1 exe 逐頁截圖)。相關程式碼:apps/desktop/src/App.tsx(側欄 NavItem、頂欄已封存鈕)、messages.ts、App.test.tsx。相關變更:desktop-config-rules-context(已封存,其 SettingsView 專案說明/產出規則區段與 desktop-core 橋接是本題重用基礎)。正典 desktop-app 與 desktop-config spec 均未釘導覽位置。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-07)

**Focus**: 導覽重排——備忘移除、已封存與設定入側欄的具體形態
**Position**: 使用者確認五項假設:(1) 備忘是死佔位鈕(App.tsx 無 onClick、無視圖、無資料),移除=刪鈕+i18n 鍵;(2) 已封存改側欄 NavItem、保留計數徽章(NavItem 擴充 trailing);(3) 點擊語意由 toggle 改單純切頁;(4) 順序:變更>規格>已封存>設定,全頂群組,規格佔位鈕保留,並 ADDED 一條側欄導覽結構需求進 desktop-app spec;(5) 獨立小刀,範圍 App.tsx、messages.ts、App.test.tsx。
**Ruled out**: 保留 toggle 語意(與其他導覽項行為不一致);拿掉已封存計數(資訊降級);設定留底部的桌面慣例(使用者明確指定順序)。
**Open**: 與第二輪的專案首頁重構是一刀還是兩刀。

### Round 2 — assumptions (2026-07-07)

**Focus**: Spectra「一進專案」的專案設定＋產出規則頁實際長怎樣(computer use 實測 v2.3.1)
**Position**: 實測結果——Spectra 進專案落在「專案首頁」(不屬任何側欄項,點 logo 返回):上方三張統計卡(進行中/規格/已封存,大數字,檔案變動即時更新);下方「專案設定 config.yaml」卡:專案說明/產出規則兩分頁、每分頁有一行灰色說明、唯讀優先(專案說明以 markdown 渲染＋顯示更多收合;產出規則以 artifact 鍵為紅色小節標題＋bullet 清單)、右上編輯就地切換為取消/儲存(專案說明=raw markdown 等寬 textarea;產出規則=每條一輸入框＋X 刪除、整節垃圾桶,無排序控制)。Spectra 設定頁為純 app 層(主題、字型、介面語言、文件產出語言、檢查更新)——專案層與 app 層分離。
**Ruled out**: 無(本輪為證據蒐集)。
**Open**: (1) Speclink 是否新增專案首頁並改落地行為(切專案預設進首頁、logo 返回),或僅重設計設定頁內容;(2) 統計卡是否可點跳轉;(3) 產出規則編輯保留上下移排序(rules-context 刀已做,Spectra 無)或完全對齊 Spectra;(4) 設定頁瘦身為 app 層(僅 UI 語言等)是否成立;(5) 與第一輪導覽重排一刀或兩刀。

### Round 3 — assumptions (2026-07-07)

**Focus**: 改造方向定調——保持 Speclink 設計語言或照抄 Spectra
**Position**: 使用者定調:保持 Speclink 既有設計(teal 主色、頂欄分頁列+側欄骨架、shadcn 元件)吸收 Spectra 概念,以 ASCII UI 稿確認形態。掃描修正一項假設:SettingsView 除專案說明/產出規則外還有 locale、spec_locale、tdd、audit 等專案政策欄位——僅搬「專案說明＋產出規則」到專案首頁卡,政策欄位與 app 層(UI 語言、自訂工具)留在設定頁。packages/ui 已有 Markdown.tsx,唯讀渲染重用之。
**Ruled out**: 照抄 Spectra 視覺(紅主色、獨立設定頁結構)——使用者要保持自家設計;設定頁全部欄位搬入首頁卡(政策開關與說明內容性質不同,混在卡裡失焦)。
**Open**: ASCII 稿的形態確認——首頁入口(字標點擊+切專案預設落地)、統計卡樣式、編輯互動(上下移保留)、空鍵呈現。

### Round 4 — assumptions (2026-07-07)

**Focus**: 首頁形態與產出規則編輯粒度
**Position**: 使用者拍板:(1) 首頁就是看板——不新增專案首頁視圖、不改落地行為;數量以既有通道標題計數徽章呈現即可(已封存計數落在導覽重排刀的側欄徽章),不做統計卡。(2) Spectra 式專案設定卡(專案說明/產出規則分頁、唯讀優先、就地編輯)改放設定頁頂部。(3) 產出規則要整份編輯,否決逐項輸入框＋新增/刪除/上下移的編輯器——傾向每鍵一個 textarea、一行一條規則(行序=注入序、刪行=刪規則、清空=移除鍵,與既有 core 變更集完美對應),免去自由鍵名輸入錯誤。
**Ruled out**: 獨立專案首頁視圖與統計卡(看板已是首頁、計數已在通道標題);逐項編輯器(上一刀的 UI 形態,使用者體感繁瑣);點字標返回首頁的入口(不再需要)。
**Open**: 每鍵一個 textarea 或全部規則一個大 textarea;設定頁頂部卡片形態最終確認。

## Conclusion

**Decision**: 扇出兩把刀。刀一(導覽重排):移除備忘佔位鈕、已封存自頂欄改為側欄 NavItem(保留計數徽章、toggle 改單純切頁)、設定上移,側欄順序:變更>規格>已封存>設定,並 ADDED 一條側欄導覽結構需求進 desktop-app spec。刀二(設定頁專案設定卡):設定頁頂部新增專案設定卡——專案說明/產出規則兩分頁、唯讀優先(專案說明 markdown 渲染＋顯示更多收合;產出規則以鍵分節＋bullet)、右上編輯就地切換取消/儲存;產出規則整份編輯(每 schema 鍵一個多行文字區、一行一條規則、清空=儲存時移除鍵);專案說明編輯為 raw markdown 文字區。首頁維持看板,不做統計卡與獨立首頁視圖。
**Rationale**: 保持 Speclink 設計語言,只吸收 Spectra 的唯讀優先與就地編輯概念;看板已是落地頁、通道標題已有計數徽章,獨立首頁與統計卡是重複;整份編輯低摩擦,行序=注入序直接對應 speclink-core 既有 rules 整份代換變更集,零引擎改動。
**Rejected alternatives**: 獨立專案首頁視圖＋統計卡(與看板功能重複);照抄 Spectra 視覺與 app/專案分離的設定頁結構(使用者要保持自家設計);逐項輸入框編輯器(繁瑣、被使用者否決);單一大文字區編輯全部規則(需手寫鍵名分節、易格式錯);已封存鈕保留頂欄 toggle 語意;備忘保留。
**Deferred**: 規格頁本體(佔位鈕保留,另刀);統計卡點擊跳轉(不做)。
**Capture to**: proposal、design、spec(desktop-app ADDED 導覽結構;desktop-config MODIFIED 設定頁需求)、tasks
**Next**: /speclink-propose --from-discussion desktop-導覽與專案首頁重構(扇出兩個變更)

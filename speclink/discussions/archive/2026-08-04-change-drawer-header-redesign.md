---
topic: 變更抽屜 header 的 UIUX 重設計:來源討論圓籤排版不一致與資訊分層
slug: change-drawer-header-redesign
status: promoted
promoted_to: change-drawer-header-redesign
created: 2026-08-04
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 變更抽屜 header 的 UIUX 重設計:來源討論圓籤排版不一致與資訊分層

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者以兩組截圖發起:(1) 變更抽屜的「來自討論」圓籤在多筆討論時排版不一致——topic 短時與標籤同行、topic 長時各自佔滿一行,左緣不對齊;(2) 抽屜 header 資訊持續增生(建立者、✳ 工具、相對時間、任務數、開工資訊、審查狀態、來源討論、同源、進度條、動作列),想做一次資訊分層的重設計。

模式:assumptions——掃到 RichDetailDrawer.tsx(header 本體)、SourceDiscussionChip.tsx(共用圓籤)、ArchivedDrawer.tsx(同一顆圓籤)、i18n.tsx,程式碼脈絡充足。

相關 change/spec:無直接在途 change;LANGUAGE.md 的「slug 直出」明文例外(desktop-card-identity, 2026-07-09)是本題的既有裁定基礎。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-04)

**Focus**: header 亂象的根因為何,以及重設計的大方向
**Position**: 四個假設全數成立,出身資訊定調「精簡一條單行列、永遠可見、不收合」(使用者裁定):
- 排版不一致的根因:圓籤顯示討論 topic 全文(整句話),flex-wrap 下短句與標籤同行、長句各自佔滿一行,同一元件兩種長相(SourceDiscussionChip.tsx:27、RichDetailDrawer.tsx:350-378)
- 圓籤改顯示 slug,topic 降為 tooltip——延伸 LANGUAGE.md「slug 直出」既有裁定(desktop-card-identity, 2026-07-09);適用範圍需明文擴充至變更抽屜/已封存抽屜的來源討論籤
- 排版快修與 header 重設計併為同一場:來源討論列本身就是 header 堆疊的一層,資訊分層才是病根
- 重設計骨架:身份(標題)/狀態(進度、審查)/出身(誰建、誰開工、來自討論、同源)/動作 四層
- 一致性範圍涵蓋 ArchivedDrawer(共用 SourceDiscussionChip);純呈現層改動,資料側 sourceDiscussions 已同時帶 slug+topic,不動引擎與 adapter
**Ruled out**:
- 維持 topic 全文、另想截斷策略——排版能救但識別性仍差,多句長文疊放無法分辨;且與 LANGUAGE.md「topic 一律降為描述/副標」裁定相悖
- 出身資訊收合區塊(點開才展開)——使用者裁定不收合
**Open**:
- 目標藍圖的具體構成:任務數、審查章、建立者 email 各歸哪一層/如何精簡
- 單行出身列塞不下時的溢出策略

### Round 2 — assumptions (2026-08-04)

**Focus**: 出身資訊單行列在多筆來源討論時是否成立(溢出策略)
**Position**: 單行是常態設計目標,不是硬保證——以全專案真實分佈與寬度估算為據:
- 131 個 change 中 73 個有來源討論;分佈:1 筆=67 個(92%)、2 筆=5 個、3 筆=1 個、4 筆=1 個(verify-station-parity,即截圖 2)
- 抽屜內容寬約 670px;出身列前綴(頭像+名字+✳工具+日期+開工+「來自」標籤)約 300px;slug 籤約 110-215px → 1 筆必定單行、2 筆看 slug 長度、3-4 筆必折行
- 前案 drawer-source-chip-overflow(2026-07-25)已做過一次截斷止痛,明文 Non-goal 不動籤內容;topic 當籤的病根未除,本次為第二次發作——證明再修截斷是死路
- 提案:流式折行——slug 籤等高等形,折行後是整齊的短籤流,不重演「標籤孤行+長句清單」的排版分歧;最壞情境(4 筆)出身資訊佔 2 行,對比現況 5 行
**Ruled out**:
- 固定拆兩行(人與時間一行、來自+同源一行)——92% 情境多耗一行,違背「精簡」定調
- 硬單行+「+N 摘要籤」或水平捲動——藏資訊等於變相收合,使用者前輪已裁定排除
**Open**:
- 使用者對「常態單行+溢出流式折行」的裁決
- 藍圖三取捨待確認:(a)任務數自 header 移除 (b)審查章升狀態列 (c)email 收進 tooltip

### Round 3 — assumptions (2026-08-04)

**Focus**: +N 數字籤展開後的歸宿與元件來源
**Position**: 使用者裁定歸宿一——出身列恆定單行,溢出收「+N」數字籤,點擊以浮層列出其餘可點籤;元件一律以 shadcn/ui 實作:
- 推翻前輪「流式折行」推薦:使用者取捨為 header 恆定高度優先於零互動可見
- 浮層原語(Popover)以 shadcn 增補,與既有 ui/ 原語(button、sheet、tooltip 皆 shadcn 系)同源;前案 drawer-source-chip-overflow「不引入新原語」原則由使用者明文解除
- tooltip 不可承載可點籤(hover 純文字),浮層是溢出討論保住「點籤跳回討論」的唯一路徑;看板卡片的圖示+tooltip 呈現維持不變,抽屜仍是全 app 唯一可點跳入口(ChangeCard.tsx:101-112)
- +N 切點採固定顆數上限,非量測式「塞得下幾顆放幾顆」——同一 change 在任何視窗寬度長相一致,呼應「排版一致」的討論初衷;確切顆數與籤寬上限值留給 design 階段
- 同源籤比照同一規則納入單行預算
**Ruled out**:
- 流式折行——高度隨資料浮動,使用者否決
- +N 原地展開——展開後即是折行,恆定高度在點擊瞬間失效,只多了一道門
- hover tooltip 列出溢出清單——看得到點不到,斷了跳轉路徑
**Open**: 無——進入結論

## Conclusion

**Decision**: 變更抽屜(含已封存抽屜)header 重設計為四層固定結構——標題列(名稱+複製)/狀態列(進度條+百分比+審查章)/出身列(單行:頭像+名字+✳工具+建立時間+開工資訊+「來自」slug 籤+「同源」籤)/動作列。來源討論籤改顯示 slug、topic 降為 tooltip;「N/N 任務」自 header 移除(任務分頁徽章已有);email 收進 tooltip 僅顯示名字;出身列恆定單行,溢出收「+N」數字籤,點擊以 shadcn Popover 彈浮層列出其餘可點籤,同源籤比照;+N 切點採固定顆數上限。元件一律以 shadcn/ui 實作(Popover 以 shadcn 增補)。
**Rationale**: 排版不一致的病根是「整句 topic 當籤」——長短不定使同一元件呈現兩種長相,前案 drawer-source-chip-overflow 的截斷止痛已證明不動籤內容是死路;slug 籤短而穩定,並延伸 LANGUAGE.md「slug 直出、topic 降副標」既有裁定。恆定單行使 header 高度可預期(使用者取捨:固定高度優先於零互動可見);固定顆數切點讓同一 change 在任何視窗寬度長相一致。
**Rejected alternatives**: topic 全文+更強截斷(識別性差,違 LANGUAGE.md 裁定);出身資訊收合區塊(使用者否決不收合);固定拆兩行(92% 單討論情境浪費一行);流式折行(高度隨資料浮動,使用者否決);+N 原地展開(展開即折行,恆定高度失效);hover tooltip 列溢出清單(不可點,斷跳轉路徑)。
**Deferred**: +N 前顯示的確切顆數與籤寬上限值、浮層內籤的排版細節(是否附 topic 副標)——留給 design 階段;LANGUAGE.md「slug 直出」明文例外的適用範圍擴充(至變更/已封存抽屜的來源討論籤與其浮層)隨本變更落地時記錄。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion change-drawer-header-redesign

---
topic: 系統匣是否比照看板補上品質站標示(code-review-stage/verify-station-parity)
slug: tray-station-badges
status: promoted
promoted_to: verify-station-parity
created: 2026-08-01
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 系統匣是否比照看板補上品質站標示(code-review-stage/verify-station-parity)

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者看到系統匣面板的變更列沒有任何審查標示,問:根據 code-review-stage 規格,tray 缺的符號要不要補上?verify-station-parity 是否比照?

模式:假設(assumptions)——偵察命中大量相關碼:apps/desktop/src/tray.ts(選單模型)、apps/desktop/src/panel/TrayPanel.tsx(macOS 面板)、packages/ui/src/components/reviewStyle.tsx(章樣式三處共用)、packages/ui/src/adapter.ts(ChangeItem.reviewStatus 已在協定)。

相關 changes:code-review-stage(done 18/18、已就緒待封存,其 desktop-app delta 僅涵蓋看板卡片/詳情抽屜/已封存頁/封存入口,無 tray)、verify-station-parity(0/17 提案中未開工,desktop-app delta 定「兩章並排」)。相關正典 specs:tray-status-menu(變更列=名稱+進度條+n/m;第 83 行已有原生選單 vs macOS 面板的平台分支先例)、desktop-app。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-01)

**Focus**: tray 沒有審查章是規格缺口還是實作漏洞?要不要納入、載體放哪?
**Position**: 是規格缺口(非實作 bug),方向為「值得加」:
- code-review-stage 的 desktop-app delta 僅列看板卡片/詳情抽屜/已封存頁/封存入口;proposal.md 與 design.md 全文無 tray/系統匣——當初未討論,非明文排除
- 資料源已通:ChangeItem.reviewStatus 已在協定(packages/ui/src/adapter.ts),TraySnapshot.changes 即 ChangeItem[],TrayPanel 拿得到但沒渲染
- 最有價值的位置是「已就緒」區:章是收尾決策訊號——inReview 有未結工單(封存會彈三選項)、reviewed 可安心封存、reviewedStale 要留意
- 範圍限 macOS 面板(復用 reviewStyle.tsx 的 icon/色/tooltip);原生選單為純文字 label,四態擠單一 unicode 字元不可辨識,不加(正典 tray-status-menu 第 83 行已有平台分支寫法先例)
- verify 章比照並排(審查前、驗證後,與卡片順序一致);載體=ingest 進 verify-station-parity(0/17 未開工、正要做兩章並排),delta 掛 tray-status-menu capability
**Ruled out**: 重開 code-review-stage(18/18 已就緒,重開延遲封存);原生選單同步加章(無 tooltip 無色彩,辨識度不可接受);另開新 change(verify 章依賴 parity 的 verifyStatus 欄位,會有先後依賴且同一列 UI 碰兩次)
**Open**: 使用者未逐條裁定假設,轉問下一節點:看板卡名列的行內符號(頭像/審查章/來源討論泡)是否全數比照進 tray?

### Round 2 — assumptions (2026-08-01)

**Focus**: 看板卡名列的行內符號(建立者頭像/審查章/來源討論泡,另有 restale 與 metaError 標記)是否全數比照進 tray?
**Position**: 只納入品質站章;判準=「行動訊號」進 tray、「閱讀脈絡」留在看板:
- 審查章+驗證章=行動訊號(直接影響收尾動作:封存前要不要先過站、有無未結工單)→ 納入,兩章並排順序與卡片一致(ChangeCard.tsx:87)
- 建立者頭像=協作閱讀脈絡,不改變任何 tray 動作;單人本地專案恆為自己首字母、零鑑別度,只添密度(ChangeCard.tsx:72)→ 不納入
- 來源討論泡=出身脈絡;tray 面板已有「已轉出」討論分區,一瞥層級再標出身是重複(ChangeCard.tsx:101)→ 不納入
- metaError=罕見診斷標記,修復動作在看板/引擎側 → 不納入
- restale 重整標記=嚴格說也是行動訊號(待 re-ingest),但本次維持 tray 列「進度+站章」極簡,不納入 → 列 Deferred,若日後 re-ingest 常被漏掉再議
**Open**: 使用者對「只納站章、其餘不納」的裁定

### Round 3 — assumptions (2026-08-01)

**Focus**: 驗證章在 tray 的完整樣態——與審查章並排的全狀態矩陣、版面位置
**Position**: 與看板卡片同構,全狀態如下:
- 驗證章四態沿用 parity design D5(none/inVerify/verified/verifiedStale),tray 列一律依協定狀態渲染、不分生命週期區(與卡片同構,實務上章多出現在已就緒區)
- 並排順序固定:審查章前、驗證章後;版面位置在名稱與 n/m 之間、靠右貼齊
- 色調沿用 reviewStyle:進行中=藍、已蓋章=青綠、其後有變動=琥珀;各章有 tooltip 正典詞
- 紅色「曾審查未通過/曾驗證未通過」只存在於已封存側;tray 僅列 active change,永不出現紅章
- 原生選單(非 macOS)維持現狀無章,依正典 tray-status-menu 第 83 行的平台分支寫法明文排除
**Open**: 使用者對符號集(只納站章、頭像與討論泡不納)與版面位置的最終裁定

### Round 4 — assumptions (2026-08-01)

**Focus**: 載體——使用者提議 ingest 進 code-review-stage 與 verify-station-parity 兩個 change,還是只進後者?
**Position**: 只 ingest 進 verify-station-parity:
- tray 章是「一條 Requirement」(變更列的品質站章,同時規範審查章與驗證章的並排):拆進兩個 change 代表兩個 active change 對 tray-status-menu 同一 Requirement 出 delta,直接衝突
- code-review-stage 18/18 已就緒且審查中(review.md 工單未蓋章):ingest 塞新任務會讓任務清單回到未全完成,蓋章守門(任務全完成)被擋,收尾與封存被拖延;工單既有 findings 也不覆蓋新增實作
- 施工面:TrayPanel ChangeRow 是同一個渲染區塊一次畫兩章,單一載體一次施工
**Ruled out**: 雙載體 ingest——delta 衝突+code-review-stage 收尾被拖延
**Open**: 無——決策樹已走完

## Conclusion

**Decision**: 系統匣 macOS 面板的變更列比照看板卡片加入品質站章——審查章與驗證章並排(審前驗後,位於名稱與 n/m 之間靠右),active 四態沿用 reviewStyle 與 parity design D5 的圖示、色調與 tooltip 正典詞;建立者頭像、來源討論泡、restale、metaError 皆不納入;原生選單(非 macOS)維持現狀無章。載體=單一 ingest 進 verify-station-parity,delta 掛 tray-status-menu capability。
**Rationale**: tray 收「行動訊號」、看板收「閱讀脈絡」——站章直接影響收尾動作(未結工單會擋封存),其餘符號是閱讀資訊。tray 章是同一條 Requirement 規範兩章,單一載體避免兩個 active change 對同一 Requirement 出 delta 的衝突;且 verify-station-parity(0/17 未開工)正要做兩章並排,一次施工。
**Rejected alternatives**: ingest 進 code-review-stage(18/18 已就緒+審查中,新任務破蓋章守門、拖延封存,工單 findings 不覆蓋新碼);全符號比照(頭像=協作脈絡且單人專案零鑑別度、討論泡=出身脈絡且 tray 已有已轉出分區);原生選單加章(單一 unicode 字元承載四態、無 tooltip 無色彩,不可辨識);另開新 change(verifyStatus 協定依賴+同一列 UI 前後碰兩次)。
**Deferred**: restale 標記是否進 tray(同屬行動訊號,若日後 re-ingest 常被漏掉再議);原生選單章的降級方案。
**Capture to**: verify-station-parity 的 proposal/design/specs(tray-status-menu delta)/tasks——經 /speclink-ingest 併入
**Next**: /speclink-ingest verify-station-parity(discuss link 已先行)

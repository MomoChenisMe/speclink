---
topic: 系統匣面板：討論很多時，重要的提案中、進行中與已就緒要往下捲很久——有什麼 UI／UX 改善方式
slug: tray-panel-content-priority
status: promoted
created: 2026-10-13
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: tray-panel-content-priority
---

# Discussion: 系統匣面板：討論很多時，重要的提案中、進行中與已就緒要往下捲很久——有什麼 UI／UX 改善方式

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者附 wadpilot 專案的 macOS 系統匣面板截圖：8 則討論時，討論卡排在最上面且每列兩行（slug＋說明），佔掉可捲中段約八成（約 330／410px），打開面板只看得到「提案中」的標題，提案中、進行中、已就緒要往下捲。另一個壓縮來源是固定頁尾四列動作區（約 130px）。需求本身可驗證（打開面板即看得到三個生命週期分區），不需要 grill，直接以假設模式進行；使用者另要求以設計畫布畫出現況與方案（畫布「Speclink 桌面介面新設計」最下排 TrayCrowdedNow／TrayCrowdedProposal 兩張，資料取自 wadpilot 真實狀態）。相關正式規格：tray-status-menu（「面板樣式（macOS）」的區塊順序與動作區塊卡四列、「分區溢出摺疊」的前 5 筆門檻、「討論列表」的已轉出收合列）；區塊順序「討論在前」源自已封存變更 tray-empty-stage-sections（2026-07-17 實測後追加、未記理由），原生選單本來就是生命週期在前；相關程式：apps/desktop/src/panel/TrayPanel.tsx、apps/desktop/src/panel/main.tsx（PANEL_MAX_HEIGHT 640、ResizeObserver 自適應高度）、apps/desktop/src/tray.ts（OVERFLOW_LIMIT 5）、apps/desktop/src/panel/promotedExpanded.ts（已轉出收合列的本機記憶）。在途變更無觸及 tray-status-menu。
Prior discussions: tray-panel-scroll-and-vibrancy, tray-empty-stage-sections, tray-panel-card-design, desktop-ui-redesign

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-13)

**Focus**: 面板中段被討論卡佔滿時，要拉哪些槓桿讓三個生命週期分區打開就看得到
**Position**: 使用者看過畫布後接受「生命週期在前＋頁尾一列」，高度上限與溢出門檻不動；但推翻「不做收合」，要求每張內容卡都能收合。
- 決策空間：A 順序（誰排最上）／B 空間（頁尾四列、上限 640）／C 密度（討論列兩行、每區前 5 筆）／D 互動（分段切換、收合）
- A 定案：面板順序改為提案中→進行中→已就緒→討論；三階段維持看板左到右順序，不改成已就緒最上。與正式規格「面板樣式（macOS）」的「討論卡在前」衝突，該順序源自 tray-empty-stage-sections 實測後追加、未記理由；原生選單本來就是生命週期在前，兩者因此對齊
- B 定案：頁尾四列改一列——左「開啟 Speclink」文字鈕，右為專案設定、設定、結束三顆圖示鈕（滑過顯示名稱、帶 aria-label），結束與前兩顆拉開間距防誤按；中段多出約 90px。與規格 scenario「動作區塊卡四列」衝突，tray-panel-scroll-and-vibrancy 只定了四個項目、沒定排法
- B／C 不動：上限 640（main.tsx:37，tray-panel-scroll-and-vibrancy 已定）、每區前 5 筆門檻（tray.ts:89）、討論列兩行（說明行是唯一可讀內容）
- 畫布實測：方案下三個生命週期卡約 460px，落在約 488px 的中段內，討論卡標題露在底部提示下方還有內容
- D 修正：使用者要求每張卡都可收合（本輪假設 3 原本主張不做收合），細節進下一輪
**Ruled out**: 分段切換（中段一次只顯示一個清單）——面板的用途是一眼看全部狀態，切換後四個清單只剩一個；提高上限 640——tray-panel-scroll-and-vibrancy 已定不變，選單列彈出的面板太高會像一個整窗；討論列拿掉說明行——只剩英文 slug 難辨認；已就緒排最上——與看板左到右順序不一致
**Open**: 收合範圍（哪幾張卡、頁首頁尾是否在內）；收合外觀與觸發點；預設展開或收合；收合狀態是否記憶、記憶範圍（全域或每專案）；原生選單是否受影響

### Round 2 — interview (2026-10-13)

**Focus**: 每張內容卡可收合的範圍、外觀與收合狀態的記憶方式
**Position**: 四張內容卡都可收合，收合狀態存 app 本機、所有專案共用一份。
- 範圍：討論、提案中、進行中、已就緒四張內容卡；固定頁首（專案 tab 條）與固定頁尾（動作區）不收合；只改 macOS 面板，原生選單本來就是子選單層級、不動
- 外觀：整條分區標題可點，計數徽章右側加箭頭（展開朝下、收合朝右）；收合後只剩一行「圖示＋名稱＋計數」，計數照常更新，所以「三個階段分區常駐」的既有規則仍成立
- 預設全部展開；收合不影響「還有 N 個…」溢出列與「已轉出」收合列的既有行為
- 記憶：沿用 apps/desktop/src/panel/promotedExpanded.ts 的模式（面板視窗的 localStorage、讀寫失敗靜默），所有專案共用一份——收合表達的是個人閱讀習慣，與專案無關；「已轉出」收合列本來也是全域一份
- 成本：面板高度已由 ResizeObserver 依內容自適應（main.tsx），收合後面板自動變矮，不需另寫高度邏輯
- 畫布 TrayCrowdedProposal 已加上可點的收合標題
**Ruled out**: 每個專案各記各的收合狀態——要以分頁 key 分存並在專案關閉時清理，複雜度上升，而收合是個人習慣、不隨專案變；頁首頁尾也可收合——兩者本來就固定不捲動，收起來省不到中段空間
**Open**: 收合中的卡有新項目時是否自動展開

### Round 3 — interview (2026-10-13)

**Focus**: 收合中的卡有新項目時是否自動展開
**Position**: 不自動展開，只更新標題計數；使用者以範例確認整份結論草稿。
- 範例：在 wadpilot 收起「討論」→ 關面板再開或切到 speclink，討論卡仍只剩「討論 8 ›」一行；「已就緒」收著時有變更進入已就緒，標題由「已就緒 0」變「已就緒 1」，卡不自己展開
- 理由：使用者刻意收起的卡自己彈開會打亂畫面，計數已在標題上可見
**Ruled out**: 新項目進入時自動展開——推翻使用者剛做的收合，且計數已足以提示

## Conclusion

**Decision**: 系統匣面板（macOS）一刀改三件事，讓生命週期分區打開就看得到。
- **cut 1 `tray-panel-content-priority`**: 面板中段重排、頁尾收成一列、內容卡可收合
  - 順序：可捲中段改為提案中→進行中→已就緒→討論（討論卡含「還有 N 個…」與「已轉出」收合列整張移到最後）；三階段維持看板左到右順序；原生選單本來就是生命週期在前，不動
  - 頁尾一列：動作區塊卡改為單列——左「開啟 Speclink」文字鈕，右為專案設定、設定、結束三顆圖示鈕（滑過顯示名稱、帶 aria-label），結束與前兩顆拉開間距防誤按；四個動作的行為不變；中段約多出 90px
  - 收合範圍：討論、提案中、進行中、已就緒四張內容卡；固定頁首（專案 tab 條）與固定頁尾（動作區）不收合
  - 收合外觀：整條分區標題可點切換，計數徽章右側加箭頭（展開朝下、收合朝右）；收合後只剩「圖示＋名稱＋計數」一行，計數照常更新，「三個階段分區常駐」規則仍成立
  - 收合預設：全部展開；收合不影響「還有 N 個…」溢出列與「已轉出」收合列的既有行為
  - 收合記憶：面板視窗的 localStorage、讀寫失敗靜默（沿用 promotedExpanded.ts 模式），所有專案共用一份
  - 收合中的卡有新項目時不自動展開，只更新標題計數
  - 面板高度沿用既有 ResizeObserver 自適應（main.tsx），收合後自動變矮；高度上限 640、每區前 5 筆門檻、討論列兩行（slug＋說明）皆不變
  - 只改 macOS 面板；非 macOS 原生選單不受影響
  - 範例：在 wadpilot 收起「討論」→ 關面板再開或切到 speclink，討論卡仍只剩「討論 8 ›」一行；「已就緒」收著時有變更進入已就緒，標題由「已就緒 0」變「已就緒 1」，卡不自己展開
**Rationale**: 擠壓有兩個來源——討論卡排最上且每列兩行（8 則討論時佔中段約八成），以及固定頁尾四列（約 130px）；重排與頁尾一列在不犧牲「一眼看全部」的前提下讓三個生命週期分區落在第一屏（畫布實測三卡約 460px、中段約 488px），收合再把「不想看哪一區」的控制權交給使用者，且既有的高度自適應讓收合幾乎零額外成本。
**Rejected alternatives**:
- 分段切換（中段一次只顯示一個清單）— 面板的用途是一眼看全部狀態，切換後只剩一個清單
- 提高高度上限 640 — tray-panel-scroll-and-vibrancy 已定不變，選單列彈出的面板過高會像整個視窗
- 討論列拿掉說明行 — 只剩英文 slug 難以辨認
- 已就緒排最上 — 與看板左到右順序不一致
- 每個專案各記收合狀態 — 要依分頁 key 分存並在關閉專案時清理，而收合是個人閱讀習慣、與專案無關
- 頁首頁尾也可收合 — 兩者本來就固定不捲動，收起來省不到中段空間
- 新項目進入時自動展開收合中的卡 — 推翻使用者的收合，計數已足以提示
**Deferred**: none
**Capture to**: spec（tray-status-menu「面板樣式（macOS）」：區塊順序、動作區塊卡一列、內容卡收合規則；相關 scenario 同步）、design、tasks
**Next**: /speclink-propose --from-discussion tray-panel-content-priority

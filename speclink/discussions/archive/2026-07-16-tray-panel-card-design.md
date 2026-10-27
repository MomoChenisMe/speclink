---
topic: macOS 系統匣面板改為 CodexBar 式卡片設計、強化主色色彩，並修復開啟時自動 focus 複製鈕
slug: tray-panel-card-design
status: promoted
promoted_to: tray-panel-card-design
created: 2026-07-16
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: macOS 系統匣面板改為 CodexBar 式卡片設計、強化主色色彩，並修復開啟時自動 focus 複製鈕

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者比對 CodexBar 的系統匣面板後，希望 speclink 的 macOS 面板（tray-copy-and-panel-mode 產出、tray-macos-panel-only 裁決為 macOS 唯一樣式）從「原生選單式」升級為卡片設計感、增加主色色彩，並修復面板開啟時焦點自動落在第一顆複製鈕的問題。模式：assumptions（scout 找到 TrayPanel.tsx、panel/main.tsx、panel.rs、tray.ts、stage.ts、KanbanBoard.tsx 等充分脈絡）。相關 change：archive/2026-07-16-tray-copy-and-panel-mode、archive/2026-07-16-tray-macos-panel-only；相關 spec：tray-status-menu。色彩原則的既有約束：packages/ui/src/stage.ts 與 KanbanBoard.tsx 明文「單一 teal 色相以深淺表達生命週期推進」。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-16)

**Focus**: 卡片化的結構解讀、色彩方向、複製鈕自動 focus 的成因定位
**Position**: 五項假設全數成立，色彩方向確認為 teal 主色加量（呼應桌面看板視覺），並新增一條需求：
- 「卡片感」＝分區（提案中／進行中／討論／已轉出）改為半透明圓角卡片容器疊在 vibrancy 上，分區標題放大、間距放寬，取代 hr 分隔線——非每列一卡
- 色彩守 teal 單一色相（stage.ts／KanbanBoard.tsx 既有原則），以深淺與面積加量：進度條沿用 STAGE_STYLE 深淺階梯、分區圖示上色、可疊極淡 teal 漸層 wash
- 複製鈕自動 focus 成因：panel.rs:51 show_and_make_key() 使面板成 key window 後，WebKit 把焦點給第一個可 tab 元素——面板中列本體皆 div，唯一 button 是第一顆複製鈕；修法在前端（tabIndex=-1，必要時顯示時補 blur）
- 範圍限呈現層（TrayPanel.tsx）；唯一可能碰 Rust 的點是 PANEL_WIDTH（panel.rs 與 panel/main.tsx 兩處需同步）
- 不做 CodexBar 式資訊擴充（provider 分頁、用量統計），資料照舊 TraySnapshot 薄渲染
- 使用者新增：作用中專案列除打勾外需有 active 配色以利區別
**Ruled out**: 多色相配色——違反單一 teal 原則且與看板視覺分歧，使用者確認要的是「多一點主色」而非多色；每列一卡——320px 寬度下臃腫、資訊密度掉太多；Rust 端修 focus——成因在 WebKit 焦點行為，前端可解
**Open**: 作用中專案列 active 態的具體樣式；面板寬度是否從 320 放寬

### Round 2 — assumptions (2026-07-16)

**Focus**: 作用中專案列 active 態的具體樣式
**Position**: 採「淡 teal 底 pill」：
- 打勾與專案名文字用 primary（teal），列底鋪 primary/10 圓角淡底
- 與看板欄位計數徽章（STAGE_BADGE）同語彙；hover 實心 teal 反白維持不變，兩態層次分明
**Ruled out**: 實心 teal 填滿（與 hover 態同色——滑過其他列時會同時出現兩條實心 teal，混淆）；只上色不鋪底（「多一點主色」效果最弱）
**Open**: 面板寬度最終值——預設維持 320，卡片內距實測若擠再放寬（panel.rs 與 panel/main.tsx 兩處常數同步）

### Round 3 — assumptions (2026-07-16)

**Focus**: 專案選擇改為 CodexBar 式橫向 tab 條（可左右捲動＋圖示）
**Position**: 採橫向 tab 條取代垂直專案列，active 樣式隨之更新：
- 切換語意零改動：面板點專案本就只原地切換（tray.ts open-project → openProjectAt，不喚主視窗），store 訂閱重推快照、下方內容原地更新——與 CodexBar tab 行為天然一致
- 每個 tab＝專案名首字母的圓角方塊 avatar＋專案名，橫向 overflow 捲動（隱藏捲軸；320px 約容 3 個 tab）
- active tab＝實心 teal 圓角卡＋白字（呼應桌面側欄選中態與 CodexBar 的藍色選中卡）；非 active hover＝淡 teal 底——chip 形態下實心 teal 不再與整列 hover 相撞，第 2 輪的顧慮解除
- 不做 CodexBar 的 per-tab 狀態小條——需要跨專案資料，違反本討論「不擴充資訊」的範圍決定
**Ruled out**: 第 2 輪選定的「淡 teal 底 pill」active 專案列——垂直列前提已被 tab 條取代；資料夾圖示（各 tab 視覺相同、喪失辨識度，首字母 avatar 勝出）；per-tab 進度小條（資訊擴充、快照無跨專案資料）

## Conclusion

**Decision**: macOS 系統匣面板做四件事：(1) 專案選擇改為 CodexBar 式橫向 tab 條——首字母圓角方塊 avatar＋專案名、可左右捲動（隱藏捲軸）；active tab 實心 teal 圓角卡＋白字、非 active hover 淡 teal 底；切換沿用既有 open-project 原地切換語意（不喚主視窗）。(2) 分區卡片化——提案中／進行中／討論／已轉出各分區改為半透明圓角卡片容器疊在 vibrancy 上，分區標題放大、間距放寬，取代 hr 分隔線（非每列一卡）。(3) teal 主色加量——進度條沿用看板 STAGE_STYLE 深淺階梯、分區圖示上色、可疊極淡 teal 漸層 wash。(4) 修復開啟時自動 focus 複製鈕——複製鈕設 tabIndex=-1（必要時面板顯示時補 blur）。範圍限呈現層 TrayPanel.tsx；資料照舊 TraySnapshot 薄渲染、不擴充資訊。
**Rationale**: 「多一點主色」而非多色——守 stage.ts／KanbanBoard 的單一 teal 色相原則，面板與看板視覺同語彙（本就同源同 store）；tab 條可行的關鍵是面板點專案本就只原地切換內容（tray.ts open-project → openProjectAt），呈現層改造即可；focus 成因是 show_and_make_key 後 WebKit 把焦點給面板中唯一可 tab 的第一顆複製鈕，前端可解、不動 Rust。
**Rejected alternatives**: 多色相配色（違反單一 teal 原則、與看板分歧）；每列一卡（320px 下臃腫）；active 專案垂直列淡 teal pill（被 tab 條取代——chip 形態下實心 teal 不與 hover 相撞）；資料夾圖示（各 tab 視覺相同、喪失辨識度）；per-tab 狀態小條（資訊擴充、快照無跨專案資料）；Rust 端修 focus（不必要）。
**Deferred**: 面板寬度最終值——預設維持 320，卡片內距與 tab 條於真實視窗實測後若擠再放寬（panel.rs 與 panel/main.tsx 兩處常數同步改）。
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion tray-panel-card-design

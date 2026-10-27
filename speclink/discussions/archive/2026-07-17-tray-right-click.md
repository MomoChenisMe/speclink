---
topic: tray 只能左鍵開啟面板，右鍵沒有反應——右鍵應有相同行為？
slug: tray-right-click
status: promoted
promoted_to: tray-right-click
created: 2026-07-17
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: tray 只能左鍵開啟面板，右鍵沒有反應——右鍵應有相同行為？

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

macOS 面板樣式下右鍵點擊系統匣圖示無任何反應（左鍵正常開閉面板），使用者質疑右鍵應與左鍵同行為。採 assumptions 模式（tray.ts、panel.rs、lib.rs、TrayPanel.tsx 四檔脈絡充足）。根因已定位：apps/desktop/src/tray.ts:352 的 action handler 只放行 button==="Left"。相關 spec：tray-status-menu（「面板樣式（macOS）」requirement 只寫「點擊」未分鍵）。連帶發現：面板樣式下 tray 完全沒有「結束」入口（面板動作區僅「開啟 Speclink」），翻查三個封存 panel 變更（tray-macos-panel-only、tray-panel-card-design、tray-copy-and-panel-mode）的 design 均未討論過「結束」去向——屬無意遺落。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-17)

**Focus**: 右鍵點擊 tray 圖示應做什麼——同左鍵開閉面板（A），還是開含「結束」的原生小選單（B）？
**Position**: 採 A——右鍵與左鍵完全等價（開閉面板），並把面板動作區補齊：加入原生選單既有的「結束」與新增的「設定」。
- 根因：apps/desktop/src/tray.ts:352 action handler 只放行 button==="Left"——右鍵事件有進 handler，是被前端刻意過濾，非 Tauri 層缺事件
- macOS 系統狀態列（Wi-Fi、電池、輸入法）左右鍵同行為；spec tray-status-menu「面板樣式（macOS）」本就只寫「點擊」未分鍵，右鍵無反應屬實作缺口
- 面板樣式下 tray 原無任何「結束」入口（TrayPanel.tsx 動作區僅「開啟 Speclink」）；三個封存 panel 變更的 design 均未討論——確認為無意遺落，非刻意決策
- 使用者裁決：A，且面板要補「結束」，並詢問可否加「設定」——已確認可行（store 既有 setBoardView("settings")，面板動作經既有 tray-panel-action 事件通道回流主視窗執行）
**Ruled out**: B（右鍵開原生小選單）——引入面板／原生選單雙路徑、複雜度高一階；A＋面板動作區補齊即可解 quit 缺口。plugin-process 也排除——整顆外掛只為一個 exit 呼叫，改以單行 Rust command（app.exit(0)）跟隨 lib.rs 既有薄包裝模式。
**Open**: 非 macOS 原生選單動作區是否同步補「設定」（跨平台對稱）；「設定」跳轉語意確認（開主視窗＋切設定頁，與「開啟此變更」同 openIn 語意）；Ctrl+左鍵在 macOS 是否被 tray-icon 回報為右鍵（apply 時實測即可，非阻塞）。

### Round 2 — assumptions (2026-07-17)

**Focus**: 非 macOS 原生選單動作區是否同步補「設定」？
**Position**: 同步補上——tray 動作區跨平台定義為「開啟 Speclink、設定、結束」。
- spec 動作區定義一處修改全平台生效；實作面 buildTrayModel 多一個項目種類、面板動作區多兩列
- 「設定」跳轉語意兩側一致：喚起主視窗＋切換至設定頁（setBoardView("settings")），與「開啟此變更」同 openIn 語意
**Ruled out**: 只動 macOS 面板（最小範圍）——兩平台動作區從此不對稱，之後要補得再開一刀。
**Open**: 無（Ctrl+左鍵回報行為留 apply 實測——deferred，非設計阻塞）。

### Round 3 — assumptions (2026-07-17)

**Focus**: deferred 項回填——Ctrl+左鍵在 macOS 是否被 tray-icon 回報為右鍵？
**Position**: 是——Ctrl+左鍵點系統匣圖示會開閉面板，行為與右鍵一致。
- apply 階段 macOS 真實視窗實測（release .app 安裝至 /Applications 後操作系統匣）：Ctrl+左鍵開閉面板成立
- 結論：macOS 將 Ctrl+左鍵回報為 button "Right"，本變更新增的 Right 分支即涵蓋此輸入，無需額外處理
- 同批實測一併確認：右鍵開閉、左右鍵交叉混用等價、面板「設定」喚起主視窗並跳設定頁、面板「結束」結束行程、左鍵開閉未回歸
**Open**: 無——Round 1 留下的 deferred 項至此結清。

## Conclusion

**Decision**: 右鍵與左鍵完全等價——點擊系統匣圖示不分左右鍵皆開閉面板（macOS）；tray 動作區跨平台補齊為「開啟 Speclink、設定、結束」——macOS 面板與非 macOS 原生選單皆同。「設定」＝喚起主視窗並切至設定頁；面板的「結束」經新增單行 Rust command（app.exit(0)）達成。
**Rationale**: macOS 狀態列慣例左右鍵同行為，spec 本就只寫「點擊」未分鍵——右鍵無反應是實作過濾造成的缺口（tray.ts 只放行 Left）；「結束」在面板樣式下無任何入口屬無意遺落，補齊動作區順勢解掉，比讓右鍵承擔第二條選單路徑（B 案）簡單一階。
**Rejected alternatives**: B 右鍵開原生小選單——面板／原生選單雙路徑，複雜度高一階且 A 案已解 quit 缺口；plugin-process——整顆外掛只為一個 exit 呼叫，改跟隨 lib.rs 既有薄包裝模式（介面深度檢查：能力橋接、單一 adapter、刪除即缺口重現，合格）；只動面板不動原生選單——跨平台動作區不對稱。
**Deferred**: 無——原 deferred 項（Ctrl+左鍵是否被回報為右鍵）已於 Round 3 由 apply 階段 macOS 真實視窗實測結清：會開閉面板，即 macOS 回報為 Right，由本變更的 Right 分支涵蓋。
**Capture to**: proposal（spec delta 落在 tray-status-menu：「面板樣式（macOS）」requirement 補右鍵點擊 scenario、動作區定義加入「設定」、面板動作區與結束行為）
**Next**: /speclink-propose --from-discussion tray-right-click

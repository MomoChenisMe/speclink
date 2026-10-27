---
topic: desktop 從背景回前景後 cursor 偵測、hover、tooltip 失效但點擊仍正常
slug: desktop-hover-dead-after-foreground
status: promoted
created: 2026-09-16
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: fix-desktop-hover-after-tray-panel
---

# Discussion: desktop 從背景回前景後 cursor 偵測、hover、tooltip 失效但點擊仍正常

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者回報：桌面 app 有時從背景回到前景後，游標樣式、CSS hover、Radix tooltip 全部失效，但點擊仍可操作；再切換幾次或放背景一陣子回來就恢復。症狀已夠具體，未經 grill 直接進假設。
偵察：主視窗用預設標題列（tauri.conf.json），macOS 預設系統匣為 NSPanel 面板樣式（store.ts trayStyle、src-tauri/src/panel.rs：can_become_key_window、nonactivating、resign key 即 orderOut）；App.tsx onFocusChanged 只觸發更新重檢；tooltip 為 packages/ui 的 Radix Tooltip。相關規格：desktop-app、tray-status-menu。
Prior discussions: update-check-on-focus-and-cli, quality-skill-pause-and-ui-polish

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-16)

**Focus**: 根因在原生層（主視窗不是 key window）還是網頁層（Radix／React 狀態卡住）？
**Position**: 押原生層，頭號嫌犯是系統匣 NSPanel；但使用者以「開面板→Cmd-Tab 來回」手動重現一次未觸發，簡單路徑不是穩定觸發條件。
- 症狀組合（游標樣式＋CSS hover＋tooltip 同死、點擊活）只有「視窗收不到滑鼠移動事件」能同時解釋；macOS 只把 mouseMoved 送給 key window，mouseDown 任何視窗都收
- 決策樹：A 原生層（A1 NSPanel 搶走或殘留 key 狀態；A2 tao 0.35／wry 0.55 上游 bug）／B 網頁層
- 排除更新重檢：App.tsx onFocusChanged 只呼叫 recheckOnFocus，純網路請求
- 螢幕控制重現被另一個 Claude session 佔住未執行；改由使用者手動重現，一次未觸發
**Open**: 觸發條件是什麼（回前景的方式、背景多久、是否開過面板、是否切過 Space／睡醒）？發作當下三顆燈是否變灰（判 key window）？CSS hover 與 tooltip 是否同時死（判 A／B 層）？

### Round 2 — interview (2026-09-16)

**Focus**: 使用者抓到的穩定路徑（背景 5～10 秒→開面板→點變更→主視窗自動前景→點抽屜灰底關閉→hover 死）指向哪個節點？
**Position**: 指向 A1「從 nonactivating 面板把主視窗叫到前景時 key window 交接失敗」；抽屜灰底那一下多半只是揭露、不是觸發。
- 路徑：面板 emit tray-panel-action → tray.ts openIn → openMainWindow（unminimize→show→setFocus）與 openDetail 同步發出
- tao 0.35.3 util/async.rs set_focus 順序＝先 makeKeyAndOrderFront、再 activateIgnoringOtherApps；此時 app 尚未啟用、面板是 key window，主視窗被要求成 key 觸發面板 resign key→orderOut，接著 app 才啟用——三件事在同一輪主執行緒上互相踩
- Cmd-Tab 路徑之所以重現不到：app 啟用由系統驅動、順序正常，面板早已收合
- WKWebView 只在 key window 內處理滑鼠移動（Safari 背景視窗同樣無 hover），故主視窗「前景但非 key」就長成此症狀
- 螢幕控制仍被另一 session 佔用，未能實機驗證三顆燈
**Ruled out**: 網頁層 Radix／React 卡住——路徑差異全在原生啟用順序，抽屜開關在 Cmd-Tab 路徑同樣會做卻不觸發
**Open**: 發作時三顆燈是否變灰、點標題列是否立即恢復（坐實 A1）？抽屜未關前 hover 是否已死（判灰底點擊是揭露還是觸發）？修法選「面板先收再 setFocus」還是 Rust 端新命令改啟用順序？

### Round 3 — interview (2026-09-16)

**Focus**: 三顆燈彩色、抽屜開著時 hover 已死——「主視窗不是 key window」的假設是否被推翻？
**Position**: 沒被推翻，但三顆燈不是判準；下一個判準改用鍵盤（發作時按 Esc 抽屜關不關）。
- 三顆燈反映的是 main window 狀態；面板設 can_become_main_window: false，面板當 key 時主視窗仍是 main、燈仍彩色（AppKit 對 NSPanel 的設計）
- 抽屜開著就已死＝觸發點在面板→主視窗的交接（openIn→setFocus），灰底點擊只是揭露；round 2 判斷成立
- tao set_focus 的順序（先 makeKeyAndOrderFront、再 activateIgnoringOtherApps）在 app 未啟用時 makeKey 會被 AppKit 延後到啟用那一拍，與面板 resign key→orderOut 同拍互踩
- 上游查核：tauri#7884（tray 選單開視窗要多點一下才有焦點）、tauri#11386（非 active 視窗收不到滑鼠移動）、wry#175（游標不更新，resize 可暫時修復）皆同一族，無人給出根因
- 螢幕控制整輪被另一 session 佔用，未能實機驗證
**Ruled out**: 用三顆燈顏色判 key window——對 NSPanel 情境無鑑別力
**Open**: 發作時 Esc 能否關抽屜／搜尋框能否打字（判主視窗是否 key）？修法選 A「面板先收再 setFocus」或 B「Rust 端先 activate 再 makeKey」？

### Round 4 — interview (2026-09-16)

**Focus**: 發作時 Esc 關得掉抽屜，主視窗是 key window——那 hover 為何仍死？
**Position**: 「主視窗不是 key window」整條線出局；病灶是 WebKit／AppKit 的滑鼠追蹤區在「從 nonactivating 面板交接到主視窗」這條啟用路徑後沒有重新啟動。
- Esc 由主視窗的 WKWebView 收到並關閉 Radix Sheet＝鍵盤事件正常送達＝主視窗確為 key
- 點擊正常、滑鼠移動全無＝AppKit 追蹤區（NSTrackingArea）對主視窗的「游標已進入」狀態沒被重算；app 再切換幾次或 resize 才重算，與 wry#175「resize 可暫修」吻合
- 面板貼齊 tray 圖示下方、與主視窗上緣重疊：點面板時游標其實位於主視窗框內，面板 orderOut 後游標「無移動地」落在主視窗上，AppKit 不會補送 mouseEntered——此為最合理的觸發機制，但未實機證實
- 靜態閱讀到此為止，再往下要靠實機試修；修法改成「按順序試、第一個有效即收」的 spike
**Ruled out**: 修法只做 A「面板先收」——它只處理兩視窗搶 key，對追蹤區未重算沒有必然效果；改為 A→B→C 依序驗證
**Open**: 結論的 capture 與變更切法（單一變更、任務內含重現路徑的手動驗證）

## Conclusion

**Decision**: 立一個 desktop 變更，修「系統匣面板點變更後主視窗 hover／游標樣式／tooltip 失效」。修法依序試、第一個讓重現路徑不再發作的就收：A. tray.ts 的 open-* 動作先收面板再 openMainWindow（幾行 JS）；B. Rust 端新命令取代 setFocus——先 activateIgnoringOtherApps、再 makeKeyAndOrderFront、再把 WKWebView 設回 first responder（對齊 wry 建視窗的做法）；C. 主視窗成為前景後對其呼叫一次 AppKit 追蹤區重算或等價的無感 nudge。驗收標準寫死為使用者的重現路徑：Speclink 在背景 5～10 秒→開面板→點變更→主視窗前景後抽屜開著時 hover 要活→關抽屜後卡片 hover 要活。
**Rationale**: 四次觀察（點擊活、Cmd-Tab 路徑不觸發、三顆燈彩色、Esc 關得掉抽屜）把病灶鎖定在原生層的滑鼠追蹤區：主視窗確為 key window、鍵盤正常，但 AppKit 未重算「游標已進入」狀態；最合理機制是面板與主視窗上緣重疊、面板 orderOut 後游標無移動地落在主視窗上。靜態讀碼到此無法再縮小，繼續猜的成本高於實機試修。
**Rejected alternatives**: 「主視窗不是 key window」修法——Esc 測試推翻；只做 A——只解決兩視窗搶 key，對追蹤區未重算無必然效果；前端層修 Radix／React——CSS hover 與游標樣式同死，前端管不到；macOS 預設改回原生選單——等於放棄面板功能；用三顆燈判 key window——面板不能當 main，燈色無鑑別力。
**Deferred**: AppKit 的確切機制；若 C 才是有效解，修好後回頭補一個上游 issue（同族：tauri#7884、tauri#11386、wry#175）。
**Capture to**: proposal（含驗收路徑）＋ design（A→B→C 的順序與各自證據）
**Next**: /speclink-propose --from-discussion desktop-hover-dead-after-foreground

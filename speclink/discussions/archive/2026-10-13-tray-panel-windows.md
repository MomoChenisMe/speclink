---
topic: Windows 也要有跟 macOS 系統匣面板同樣的功能（不限於 tray）
slug: tray-panel-windows
status: promoted
created: 2026-10-13
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: tray-panel-windows
---

# Discussion: Windows 也要有跟 macOS 系統匣面板同樣的功能（不限於 tray）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在 tray-panel-content-priority 定案後追問：系統匣面板只有 macOS 有，Windows 若也想要類似功能（不限於 tray）可以怎麼做。需求可驗證（Windows 使用者能從系統匣一眼看到與 macOS 面板相同的內容），不需要 grill。現況查核：面板前端（apps/desktop/src/panel/TrayPanel.tsx、apps/desktop/src/panel/main.tsx）沒有任何平台分支；只有 Rust 端 apps/desktop/src-tauri/src/panel.rs 用了 macOS 專屬的 tauri-nspanel（不搶焦點的 NSPanel）與 window-vibrancy HudWindow，toggle_tray_panel／hide_tray_panel 兩個 command 在非 macOS 回 Err／無事（apps/desktop/src-tauri/src/lib.rs）；系統匣由 apps/desktop/src/tray.ts 的 TrayIcon.new 建立，原生選單樣式掛 menu 且 showMenuOnLeftClick，面板樣式不掛 menu、由 action 處理左右鍵；定位用 tauri-plugin-positioner 的 TrayBottomCenter，其 TrayCenter 在 Windows 會把視窗放在圖示上方（放不下時改放下方）。面板限 macOS 源自已封存變更 tray-macos-panel-only（2026-07-16），列為 Non-Goal、理由是 NSPanel／vibrancy 專屬，沒有記錄技術上做不到。使用者手邊有 Windows 可以安裝，但無法立即手動測試。相關正式規格：tray-status-menu（「系統匣圖示與原生選單」的平台分流、「面板樣式（macOS）」、「面板內容卡收合」由在途變更新增）；在途 delta：tray-panel-content-priority: tray-status-menu（ADDED 面板內容卡收合；MODIFIED 面板樣式（macOS））、clickable-cursor-hover-rule: tray-status-menu（MODIFIED 面板交接主視窗後滑鼠互動完整）。
Prior discussions: tray-rich-panel, system-tray-status, tray-panel-content-priority, tray-panel-scroll-and-vibrancy

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-13)

**Focus**: Windows 要用什麼形式提供與 macOS 面板相同的一眼狀態
**Position**: 選 A——把同一個面板搬到 Windows 系統匣；全域快捷鍵（B）延後。
- 決策空間：A Windows 系統匣面板／B 全域快捷鍵叫出面板（三平台）／C 工作列圖示徽章與跳躍清單／D 系統通知／E Windows 11 小工具板
- A 的成本最低：面板前端零平台分支可直接共用，只需 Rust 端為 Windows 補開視窗分支；Windows 使用者本來就會到系統匣找狀態
- tray-rich-panel 最初就決定 macOS／Windows 都用面板，後來 tray-macos-panel-only 只把 Windows 列為 Non-Goal、沒有否決理由，本次重新納入不違背先前裁定
- B 留作之後解 Windows 11 圖示預設藏在「^」溢位區的可發現性問題
**Ruled out**: C 工作列徽章與跳躍清單——資訊量太小、跳躍清單只能放啟動連結，不能顯示進度或複製；D 系統通知——只能推播、不能一眼看全部狀態，只適合當補充；E Windows 11 小工具板——要 MSIX 打包與另寫系統元件，與 Tauri 架構差距太大
**Open**: Windows 面板能否與新的 macOS 面板同一套設計、有哪些必然差異；右鍵行為

### Round 2 — interview (2026-10-13)

**Focus**: Windows 面板能否沿用新的 macOS 面板設計，以及右鍵要做什麼
**Position**: 畫面完全共用（含 tray-panel-content-priority 的重排、頁尾單列、卡片收合與共用收合記憶），差異只在開視窗那一層；右鍵顯示現有原生選單。
- 共用：專案 tab 條、四張內容卡、頁尾單列、複製鈕、收合、資料來源、深淺色、高度自適應與上限 640
- 位置：圖示上方置中（positioner 的 TrayCenter；工作列在頂端時套件自動改放下方）
- 高度改變後必須重新定位：macOS 頂端貼選單列、變高變矮頂端不動；Windows 底部貼工作列，但改視窗高度時系統固定左上角，不重新定位就會離開工作列或蓋到工作列
- 背景：Windows 用不透明底色，不用 Mica／Acrylic——macOS 補光層本來就厚，視覺差很小，也不必處理 Windows 10 沒有 Mica
- 焦點：Windows 沒有不搶焦點的視窗，打開會搶焦點（與系統音量、網路面板相同），失焦自動收起
- 靠近螢幕右緣時面板（寬 320）要限制在螢幕範圍內
- 右鍵：顯示現有原生選單（符合 Windows「右鍵找結束」的習慣；原生選單程式碼本來就要保留作面板建立失敗的備用）；左鍵開閉面板
- Windows 11 圖示預設藏在「^」：設計解決不了，只能在手冊提示把圖示拖到工作列
- 使用者可以安裝但無法立即手動測試：位置、重新定位、焦點與收合只能在真機確認，會有 [M] 任務且封存要等它；面板建立失敗時退回原生選單的既有保險仍在
- 順序：排在 tray-panel-content-priority 之後——兩者都改「面板樣式（macOS）」，且這次要把它改成不限 macOS
**Ruled out**: 右鍵也開面板（與 macOS 一致）——違反 Windows 右鍵出選單的慣例，使用者找不到「結束」；Windows 用 Mica／Acrylic 毛玻璃——Windows 10 沒有 Mica、Acrylic 拖動卡頓，且被厚補光層蓋掉後幾乎看不出差別
**Open**: 規格怎麼從「面板樣式（macOS）」改成跨平台；Windows 視窗外框（圓角、陰影）由誰畫；點圖示收起面板時「失焦先收、點擊又打開」的競態；自動與手動驗證的分工

### Round 3 — assumptions (2026-10-13)

**Focus**: 規格結構、Windows 視窗外框、開閉競態、程式落點與驗證分工
**Position**: 使用者確認五項假設全部成立。
- 規格：「面板樣式（macOS）」改名為「面板樣式」，內文分「共用」與「平台差異」；同步改「系統匣圖示與原生選單」（Windows 左鍵面板、右鍵原生選單；Linux 原生選單）與「面板內容卡收合」（拿掉 macOS 限定）；須在 tray-panel-content-priority 封存後才封存
- 視窗外框：Windows 面板為不透明、無系統外框、開陰影（Win11 由系統畫圓角與陰影，Win10 直角細邊），比照 desktop-ui-redesign 對主視窗的 decorations false＋shadow true 裁定；面板根層在 Windows 不畫圓角、不鋪半透明補光，改不透明側欄底色；平台判斷沿用 tray.ts 的 detectPlatform()
- 開閉競態：面板因失焦收起後約 300ms 內收到的開啟要求一律忽略，避免點圖示關面板時閃一下又打開（tray.ts 於放開滑鼠時切換；macOS 靠 NSPanel 失焦通知，Windows 沒有）
- 程式落點：panel.rs 加 Windows 分支（建立、TrayCenter 定位並限制在螢幕內、視窗大小改變即重新定位、失焦收起、競態保護）；lib.rs 的 toggle_tray_panel／hide_tray_panel 在 Windows 委派 panel 模組；tray.ts 在 Windows 掛原生選單但 showMenuOnLeftClick 為 false、左鍵 action 開閉面板；不新增 IPC command，Linux 不動
- 介面深度：panel 模組是唯一的平台差異落點，承載建立、定位、收起與競態保護；刪掉它 Windows 退回只有原生選單
- 驗證分工：CI 的 windows-latest 已跑 cargo test --workspace 與 npm test -w apps/desktop（抓 Windows 編譯錯誤與前端平台分支）；真機逐項確認放 [M] 任務（工作列底部／頂端、圖示靠右緣、收合後重新貼齊、點圖示可關且不閃開、右鍵選單、深淺色），封存要等 [M] 完成
**Ruled out**: 另增「面板樣式（Windows）」需求、保留 macOS 那條——兩條需求描述同一版面，日後改版要改兩處；透明視窗＋CSS 圓角——Windows 不替透明視窗畫陰影，邊緣問題多
**Open**: none

## Conclusion

**Decision**: Windows 系統匣改為「左鍵開閉面板、右鍵原生選單」，面板畫面與 macOS 共用，一刀落地並排在 tray-panel-content-priority 之後。
- **cut 1 `tray-panel-windows`**: Windows 系統匣面板
  - 畫面：與 macOS 共用專案 tab 條、四張內容卡（提案中→進行中→已就緒→討論）、頁尾單列、常駐複製鈕、內容卡收合與共用收合記憶、高度自適應與上限 640
  - 外觀差異：Windows 面板根層用不透明側欄底色、不鋪半透明補光、不畫自己的圓角；視窗不透明、無系統外框、開陰影——Win11 由系統畫圓角與陰影，Win10 為直角細邊；平台判斷沿用 apps/desktop/src/tray.ts 的 detectPlatform()
  - 位置：圖示上方置中（tauri-plugin-positioner 的 TrayCenter，工作列在頂端放不下時套件自動改放下方），水平方向限制在螢幕範圍內；視窗大小每次改變就重新定位，讓面板底部一直貼著工作列
  - 焦點：打開時取得焦點（Windows 沒有不搶焦點的視窗），失焦自動收起；收起後約 300ms 內收到的開啟要求一律忽略，避免點圖示關面板時閃一下又打開
  - 點擊：左鍵開閉面板；右鍵顯示現有原生選單（tray.ts 在 Windows 掛 menu 且 showMenuOnLeftClick 為 false）；面板建立失敗時左鍵也退回原生選單；Linux 維持原生選單不動
  - 程式落點：apps/desktop/src-tauri/src/panel.rs 加 Windows 分支（建立、定位、重新定位、失焦收起、競態保護）；apps/desktop/src-tauri/src/lib.rs 的 toggle_tray_panel／hide_tray_panel 在 Windows 委派 panel 模組；不新增 IPC command
  - 規格：tray-status-menu 的「面板樣式（macOS）」改名為「面板樣式」，內文分「共用」與「平台差異」；同步修改「系統匣圖示與原生選單」（平台分流改為 macOS 面板、Windows 面板＋右鍵原生選單、Linux 原生選單）與「面板內容卡收合」（拿掉 macOS 限定）
  - 驗證：CI 的 windows-latest 既有 cargo test --workspace 與 npm test -w apps/desktop 負責編譯與前端平台分支；真機逐項確認（工作列在底部與頂端、圖示靠螢幕右緣、收合後重新貼齊工作列、點圖示可關且不閃開、右鍵選單、深淺色）放 [M] 任務，封存要等它完成
  - 順序：依賴 tray-panel-content-priority（同改「面板樣式（macOS）」且建立在其版面之上），封存須在其後
  - 範例：Windows 11、工作列在底部、wadpilot 專案——左鍵點圖示，面板出現在圖示正上方、底部貼工作列，內容與 macOS 相同、底色不透明、四角為系統圓角；收起「討論」後面板變矮、底部仍貼工作列；再點圖示面板關掉、不閃開；右鍵點圖示出現原生選單、最下面是「結束」
**Rationale**: 面板前端本來就零平台分支，Windows 缺的只是開視窗那一層；沿用同一份畫面、只在 panel 模組補 Windows 分支，成本最低且兩平台不會分岔。Windows 與 macOS 的必然差異（位置在工作列上方、會搶焦點、沒有毛玻璃、右鍵慣例）都收在視窗層處理，畫面規格維持一份。使用者暫時無法真機測試，所以把能自動驗的交給 CI 的 Windows 環境，真機行為以 [M] 任務把關，並保留「面板建立失敗退回原生選單」的既有保險。
**Rejected alternatives**:
- 工作列圖示徽章與跳躍清單 — 資訊量太小，跳躍清單只能放啟動連結，不能顯示進度或複製
- 系統通知 — 只能推播，不能一眼看全部狀態
- Windows 11 小工具板 — 要 MSIX 打包與另寫系統元件，與 Tauri 架構差距太大
- 右鍵也開面板（與 macOS 一致） — 違反 Windows 右鍵出選單的慣例，使用者找不到「結束」
- Windows 用 Mica／Acrylic 毛玻璃 — Windows 10 沒有 Mica、Acrylic 拖動卡頓，且被厚底色蓋掉後幾乎看不出差別
- 另增「面板樣式（Windows）」需求、保留 macOS 那條 — 兩條需求描述同一版面，日後改版要改兩處
- 透明視窗＋CSS 圓角 — Windows 不替透明視窗畫陰影，邊緣問題多
**Deferred**:
- 全域快捷鍵叫出面板（三平台） — 先看 Windows 系統匣面板實際使用情形，再決定是否需要解圖示藏在「^」溢位區的可發現性
- 手冊提示「把圖示從 ^ 拖到工作列」 — 等面板在真機確認後再寫
**Capture to**: spec（tray-status-menu：面板樣式改名與平台差異、系統匣圖示與原生選單、面板內容卡收合）、design、tasks
**Next**: /speclink-propose --from-discussion tray-panel-windows

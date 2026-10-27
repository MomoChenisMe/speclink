## Context

系統匣面板目前只在 macOS 出現。面板前端（apps/desktop/src/panel/TrayPanel.tsx、apps/desktop/src/panel/main.tsx）沒有任何平台分支；平台限制全在 Tauri 殼與接線層：

- apps/desktop/src-tauri/src/lib.rs 的 `mod panel` 與 `toggle_tray_panel`／`hide_tray_panel` 兩個 command 以 `cfg(target_os = "macos")` 限定，其他平台回 Err／無事；positioner 與 nspanel 兩個外掛也只在 macOS 註冊。
- apps/desktop/src-tauri/Cargo.toml 把 tauri-nspanel、tauri-plugin-positioner、window-vibrancy 放在 macOS 專屬的相依區段。
- apps/desktop/src-tauri/capabilities/macos.json 只對 macOS 宣告面板視窗需要的能力（positioner、剪貼簿、set-size、set-theme）。
- apps/desktop/src/store.ts 的 `trayStyle` 初值為「macOS → panel，其餘 → native-menu」；apps/desktop/src/tray.ts 在 panel 樣式下不掛選單、左右鍵都開閉面板。

來源討論 `tray-panel-windows` 定案：Windows 左鍵開閉面板、右鍵顯示原生選單，畫面與 macOS 共用，差異收在視窗層。本變更建立在 `tray-panel-content-priority` 的面板版面（生命週期在前、頁尾單列、卡片收合、補光層約六成）之上。

邊界：沒有新的領域邏輯，speclink-core／speclink-host 不動；兩個可單元測試的純計算（面板位置、失焦後的重開抑制）依專案慣例放 speclink-desktop-core，Tauri 殼只做視窗操作與委派。

## Goals / Non-Goals

**Goals:**

- Windows 使用者左鍵點系統匣圖示，得到與 macOS 相同內容與版面的面板；右鍵仍是原本的原生選單。
- 面板在 Windows 底部貼著工作列，高度改變（收合、資料增減）後仍貼齊，且不超出螢幕。
- 點圖示可以關掉面板，不會閃一下又打開。
- 面板建立失敗時退回原生選單（既有保險延伸到 Windows）。

**Non-Goals:**

- 不改 Linux（維持原生選單）。
- 不改 macOS 的面板行為與外觀。
- 不在 Windows 用 Mica／Acrylic 毛玻璃，不讓面板自繪圓角。
- 不做全域快捷鍵叫出面板（延後）。
- 不寫手冊的「把圖示從 ^ 拖到工作列」提示（延後）。
- 不新增 IPC command，不改 TraySnapshot 資料形狀。
- 不改「面板交接主視窗後滑鼠互動完整」需求（它處理的是 macOS NSPanel 的問題，仍限 macOS；在途變更 clickable-cursor-hover-rule 也在改它）。

## Decisions

### D1 Windows 面板視窗：不透明、無外框、系統陰影

Windows 分支以一般 WebviewWindow 建立面板：無標題列與外框（decorations false）、不透明（transparent false）、開陰影（shadow true）、不進工作列、置頂、不可調整大小、先隱藏。Windows 11 對「無外框＋陰影」的視窗由系統畫圓角與陰影；Windows 10 為直角加細邊。這與 desktop-ui-redesign 對主視窗在 Windows 的裁定（decorations false＋shadow true 保留 Win11 圓角）一致。

- 替代方案「透明視窗＋CSS 圓角」：Windows 不替透明視窗畫陰影，邊緣問題多，被否決。
- 替代方案「Mica／Acrylic」：Windows 10 沒有 Mica、Acrylic 拖動卡頓，且被厚底色蓋掉後幾乎看不出差別，被否決。

### D2 面板前端的平台外觀分支

`apps/desktop/src/panel/main.tsx` 以既有的 `detectPlatform()`（apps/desktop/src/platform.ts）判斷平台並傳給 TrayPanel。panel-root 在 macOS 維持 13px 圓角與約六成的補光層；在 Windows 改為不透明側欄底色（`bg-sidebar`）、不畫圓角。其他所有呈現（卡片、列、收合、頁尾）不分平台。

### D3 定位：依圖示位置在其螢幕可用範圍內自算，大小改變就重新定位

Windows 的開啟流程：讀取系統匣圖示的實際位置（Tauri 的 tray rect，實體像素）並記下，找出圖示中心所在的螢幕，以該螢幕的可用範圍（work area）算出面板位置後移動視窗。面板視窗收到大小改變事件時，以記下的圖示位置與目前視窗大小重做同一套計算，所以貼齊工作列的那一邊維持貼齊。記下開啟當下的位置、不在事後重查，是因為圖示在溢出區（^）時，溢出視窗收起後就查不到原本的位置。

- 位置計算是純函式 `panel_position`，放 `apps/desktop/core/src/tray_panel.rs`：輸入圖示矩形、面板寬高與 work area，輸出面板左上角。水平以圖示為中心，再限制在 work area 內（超出右緣往左貼齊、超出左緣往右貼齊，面板比範圍寬時貼齊左緣）；垂直看圖示在 work area 垂直中線的哪一側——中線以下（工作列在底部）時面板底部貼齊 work area 下緣，否則（工作列在頂端）面板頂部貼齊 work area 上緣。只看圖示所在的螢幕，其他螢幕的位置不影響結果。
- 圖示位置或圖示所在螢幕取不到時不移動視窗，面板照樣顯示（見 Failure modes）。
- positioner 只留在 macOS：Cargo.toml 的 positioner 維持在 macOS 相依區段，lib.rs 只在 macOS 註冊，tray.ts 只在 macOS 把點擊事件餵給 positioner，positioner 權限放在只給 macOS 的 `capabilities/tray-panel-macos.json`。面板視窗操作的能力宣告 `capabilities/macos.json` 改名為 `capabilities/tray-panel.json`、platforms 加入 windows，apps/desktop/src/__tests__/themeBoot.test.ts 的讀檔路徑跟著改。
- 替代方案「positioner 的 `TrayCenter` 加上水平限制」（原定案，品質關卡後改掉）：TrayCenter 以整個虛擬桌面的 y < 0 判斷上方放不下，副螢幕位於主螢幕上方時會把面板翻到另一個螢幕；它也以圖示上緣而非工作列上緣對齊，面板會壓到工作列幾 px。
- 替代方案「固定放在螢幕右下角」：工作列可能在頂端或圖示不在右側，被否決。

### D4 焦點與開閉競態保護

Windows 沒有不搶焦點的視窗：面板顯示後取得焦點，失焦（WindowEvent::Focused(false)）即收起並記下收起時間。之後的開啟要求若距收起不到 300ms 就忽略——在 Windows 上點系統匣圖示會先讓面板失焦收起、接著點擊事件才送到，沒有這層保護，點圖示關面板會閃一下又打開。

- 判斷是否忽略是純函式，放 `apps/desktop/core/src/tray_panel.rs`：輸入上次收起時間（可為無）、現在時間與門檻 300ms，輸出是否忽略。
- 上次收起時間與開啟當下的圖示位置（D3）存在面板狀態裡，隨面板建立以 `app.manage` 註冊，與殼層其他共用狀態同一做法。
- macOS 維持 NSPanel 的失焦通知與不搶焦點，不套這層保護。

### D5 點擊分流：Windows 左鍵面板、右鍵原生選單

`trayStyle` 初值改為「macOS 與 Windows → panel，Linux → native-menu」。tray.ts 在 panel 樣式下依平台分流：

- macOS：不掛選單、左右鍵放開都開閉面板（現狀）。
- Windows：掛同一份原生選單、`showMenuOnLeftClick` 為 false（右鍵由系統顯示選單），只有左鍵放開會開閉面板；選單內容隨快照更新，與原生選單樣式同一份建構。
- 面板建立失敗（toggle_tray_panel 回錯）時沿用 `panelFallback`：trayStyle 轉 native-menu，左鍵也開原生選單，設定頁浮出單行錯誤。

### D6 Tauri 殼的模組與 command 委派

panel 模組在所有平台編譯，是唯一的平台差異落點：模組內以平台區塊分開三套實作——macOS 為既有 NSPanel 實作，Windows 為 D1／D3／D4，Linux 為守門（toggle 回 Err、hide 無事）——對外只露出同樣的 `toggle` 與 `hide`，lib.rs 的兩個 command 因此不必列舉平台。兩平台共用的視窗設定（標籤、尺寸、無外框、不可調整大小、不進工作列、置頂、先隱藏）收在同一個 builder 前段，各平台再疊上自己的外觀設定。不新增 IPC command。

- `toggle_tray_panel` 在 Windows 為 async command：首次開啟會建立面板視窗，而在同步 command 內建立視窗，Windows 會因 WebView2 死結（Tauri `WebviewWindowBuilder::new` 的 Known issues）。macOS 維持同步 command，因為 NSPanel 的操作必須在主執行緒。
- 主視窗關閉時一併關閉 Windows 面板：Tauri 在所有視窗都關閉後才結束 app，常駐隱藏的面板會讓關主視窗後行程不結束；一併關閉才維持加入面板前「關主視窗即結束 app」的行為。
- 介面深度：panel 模組承載建立、定位、重新定位、失焦收起與競態保護；刪掉它 Windows 退回只有原生選單。

### D7 規格：以「面板樣式」取代「面板樣式（macOS）」

引擎不允許同一個 delta 同時 RENAMED 又 MODIFIED 同一條需求（archive 合併閘會判為撞名），所以「改名＋改內容」以 REMOVED「面板樣式（macOS）」＋ADDED「面板樣式」表達：新需求承載原有全部內容（含 tray-panel-content-priority 帶進來的版面），分為共用段與平台差異段，並保留原有 scenario、加上 Windows scenario。同時 MODIFIED「系統匣圖示與原生選單」（平台分流）與「生命週期分區與變更進度」（原生選單適用範圍改為 Linux、Windows 右鍵選單與失敗後備，面板引用改指「面板樣式」）。「面板內容卡收合」由 tray-panel-content-priority 新增，在它封存前不在正式規格裡，`speclink validate` 不允許 MODIFIED 一條不存在的需求；因此改由 tray-panel-content-priority 把該需求寫成不綁平台、不點名舊需求的措辭（「面板的四張內容卡」「依面板的高度自適應規定」「系統匣原生選單不受影響」），本變更不再修改它，Windows 的收合行為以「面板樣式」的總則與一條 Windows scenario 承載。本規格其他寫作「macOS 面板」的需求（品質站章、波次標示、開啟視窗與結束動作、討論列表的面板 scenario）不逐條改寫，由「面板樣式」的總則宣告 Windows 面板一體適用。

- 封存順序：tray-panel-content-priority 修改「面板樣式（macOS）」，本變更移除它，必須在它之後封存（plan 的 archiveAfter 會依需求重疊排出）。
- 替代方案「另增面板樣式（Windows）、保留 macOS 那條」：兩條需求描述同一版面，日後改版要改兩處，被否決。

## Implementation Contract

**Behavior**

- Windows：左鍵放開系統匣圖示 → 面板在圖示上方水平置中出現、底部貼工作列（工作列在頂端時改在圖示下方）、完整落在螢幕可用範圍內、取得焦點；內容與 macOS 面板相同。
- Windows：面板高度改變（收合、展開、資料增減）→ 面板重新定位，貼齊工作列的那一邊維持貼齊。
- Windows：面板開著時點圖示 → 面板收起且保持收起；點面板外任何地方 → 收起。
- Windows：右鍵放開系統匣圖示 → 顯示原生選單（專案區、生命週期分區、討論區、動作區），不開啟面板（面板開著時，點圖示會讓它失焦而收起）。
- Windows：面板外觀為不透明側欄底色、無自繪圓角；Windows 11 由系統呈現圓角與陰影。
- Windows：面板建立失敗 → 左鍵也開原生選單，設定頁浮出單行錯誤。
- Windows：多螢幕時面板留在圖示所在的螢幕，上下邊依該螢幕的工作列位置決定。
- Windows：開過面板後關閉主視窗 → app 照常結束。
- macOS 與 Linux 的系統匣行為不變。

**Interface / data shape**

- 無新 IPC command；`toggle_tray_panel`、`hide_tray_panel` 在 Windows 生效。
- `TrayStyle` 型別不變（panel／native-menu），只改初值的平台判斷。
- speclink-desktop-core 新增模組 `tray_panel`，匯出實體像素矩形型別 `PxRect` 與兩個純函式：面板位置（圖示矩形、面板寬、面板高、work area → 面板左上角）與重開抑制（上次收起時間、現在時間 → 是否忽略，門檻 300ms 為模組內常數）。
- 能力宣告檔改名為 `capabilities/tray-panel.json`，platforms 為 macOS 與 windows；positioner 權限另置於只給 macOS 的 `capabilities/tray-panel-macos.json`。

**Failure modes**

- 面板視窗建立失敗或 toggle 回錯 → 走既有 `panelFallback`（native-menu＋設定頁單行錯誤），不崩潰。
- Windows 只有面板視窗建立或顯示失敗時 toggle 回錯；收起失敗與定位失敗不回錯，暫時性問題不會把整個執行期降為原生選單。
- Windows 取不到圖示位置或圖示所在螢幕 → 不移動視窗，面板照樣顯示在原位。
- macOS 維持原樣：positioner 定位失敗回錯，走 `panelFallback`。

**Acceptance criteria**

- `cargo test -p speclink-desktop-core`：水平限制（範圍內不變、超出右緣往左移、超出左緣往右移、視窗比範圍寬時貼左緣）、面板位置（spec Example「水平位置限制」三列與「垂直位置」三列：工作列在底部、在頂端、副螢幕位於主螢幕上方）與重開抑制（無收起紀錄不忽略、299ms 忽略、300ms 不忽略）的單元測試全綠。
- `npm test -w apps/desktop`：
  - store：Windows UA 的 trayStyle 初值為 panel、Linux UA 為 native-menu、macOS 為 panel。
  - tray：Windows 的 panel 樣式掛原生選單且 showMenuOnLeftClick 為 false，只有左鍵放開觸發 onPanelToggle、右鍵不觸發；macOS 左右鍵都觸發且不掛選單；面板建立失敗後轉 native-menu；點擊事件只在 macOS 餵給 positioner。
  - trayPanel：Windows 平台的 panel-root 不含圓角與半透明補光、含不透明側欄底色；macOS 維持圓角與 `bg-sidebar/60`。
  - themeBoot：改讀 `capabilities/tray-panel.json`，斷言不變。
- CI 的 windows-latest 跑 `cargo test --workspace`，Windows 分支可編譯。
- [M] 真機確認：工作列在底部與頂端、圖示靠螢幕右緣、收合後重新貼齊、點圖示可關且不閃開、右鍵選單、深淺色。

**Scope boundaries**

- In scope：上列 Affected code 的檔案；tray-status-menu 的 delta（REMOVED 面板樣式（macOS）、ADDED 面板樣式、MODIFIED 系統匣圖示與原生選單、MODIFIED 生命週期分區與變更進度）。
- Out of scope：Linux 系統匣、macOS 面板行為、面板畫面內容、全域快捷鍵、手冊、「面板交接主視窗後滑鼠互動完整」需求。

## Risks / Trade-offs

- [使用者暫時無法在真機測試，Windows 行為只能靠 CI 編譯與單元測試把關] → 面板建立失敗退回原生選單的保險仍在；真機項目列為 [M] 任務，封存前必須完成。
- [圖示位置與螢幕可用範圍在 Windows 多螢幕、高 DPI 下的座標是否一致只有真機看得到] → 列入 [M] 檢查；兩者都取實體像素，位置計算只看圖示所在的螢幕。
- [Tauri 殼以固定 id `speclink-tray` 找系統匣圖示，必須與 tray.ts 建立圖示的 id 一致] → 兩處以註解互相指向；id 不一致時取不到圖示位置，面板照樣顯示但不定位。
- [不透明視窗在深色模式下，WebView2 首次繪製前可能先閃一下白底] → 面板 lazy 建立後常駐隱藏，只在第一次開啟時可能發生；真機若明顯，再於建立視窗時指定與主題相符的背景色。
- [300ms 的抑制門檻在很慢的機器上可能不夠，或擋到使用者刻意快速再開] → 門檻是常數，真機若有問題可調；快速再開只需再點一次。
- [REMOVED＋ADDED 讓「面板樣式」的溯源歷史從本變更重新開始] → 舊需求的歷史仍留在已封存變更與 git 中；這是引擎不允許同一 delta 改名又改內容的必然結果。
- 回歸對照：沒有 CLI 輸出、golden 檔受影響；回歸面是 desktop 的 vitest、speclink-desktop-core 的 cargo test 與 CI 三平台的 workspace 編譯。
- 跨平台：Linux 走原生選單、panel 模組只編譯守門區塊；macOS 實作不動，只是被包進平台區塊並改用共用的 builder 前段；Windows 為新路徑。

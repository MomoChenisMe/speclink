## 1. 剪貼簿基建（D3 剪貼簿經 clipboard-manager）

- [x] 1.1 引入 tauri-plugin-clipboard-manager：apps/desktop/src-tauri/Cargo.toml 加相依、apps/desktop/src-tauri/src/lib.rs 註冊外掛、apps/desktop/src-tauri/capabilities/default.json 授權 clipboard-manager:allow-write-text、apps/desktop/package.json 加 @tauri-apps/plugin-clipboard-manager。契約：前端可呼叫 writeText 且不受視窗焦點限制。驗證：cargo build --release -p speclink-desktop 成功、npm run build -w apps/desktop 成功。 <!-- speclink-task:tsk_01KXMQG5XT1M71T9F7JN7QE7W3 -->

## 2. 原生選單複製與討論 slug 化

- [x] 2.1 【紅】撰寫 apps/desktop/src/__tests__/tray.test.ts 新測試：涵蓋規格「變更子選單動作」（buildTrayModel 產出的變更動作依序為開啟此變更、複製名稱，複製內容等於 name 不含進度條字元）與「討論列表」（討論項為子選單：父標籤等於 slug、首行為 disabled 的 topic 描述行、續為開啟此討論與複製 slug；無討論時顯示「討論 0」不變）。驗證：npm test -w apps/desktop 新測試紅、既有測試綠。 <!-- speclink-task:tsk_01KXMQG5XTCNQ2Q5ZK76WHH5TC -->
- [x] 2.2 【綠】實作 apps/desktop/src/tray.ts：D1 原生選單複製動作（TrayChangeAction 擴充 copy-name）與 D2 討論項 slug 化與子選單（TrayMenuItem 討論項改子選單模型），接線層以 clipboard 外掛 writeText 掛複製動作（寫入失敗靜默、不彈窗）；apps/desktop/src/i18n/messages.ts 新增複製名稱／開啟此討論／複製 slug 等鍵且 zh-TW 與 en 字典 key 集合相等。驗證：npm test -w apps/desktop 全綠（含 messages key 相等測試）。 <!-- speclink-task:tsk_01KXMQG5XTFGRHVVV4KRZD19KN -->
- [x] 2.3 【重構】整理 tray.ts 模型與接線層重複（動作聯集共用建構），並擴充 openspec/LANGUAGE.md 討論識別錨點例外的枚舉面（納入系統匣討論項與其複製動作，比照 desktop-ux-polish 範圍擴充記法）。驗證：npm test -w apps/desktop 仍綠；LANGUAGE.md 內容審視——例外枚舉含系統匣討論項。 <!-- speclink-task:tsk_01KXMQG5XTJ9EAJZA3KGDAE92Z -->

## 3. 系統匣樣式偏好（D4 系統匣樣式偏好與設定頁切換）

- [x] 3.1 【紅】撰寫 apps/desktop/src/__tests__/trayStyle.test.ts 紅測試：app 本機偏好模組 apps/desktop/src/trayStyle.ts（比照 i18n/locale.ts 的 localStorage 單鍵模式）——readTrayStylePreference 於缺鍵或非法值一律回 native-menu（規格「系統匣樣式偏好」的向後相容場景）、writeTrayStylePreference 寫入後可讀回 panel。驗證：npm test -w apps/desktop 新測試紅。 <!-- speclink-task:tsk_01KXMQG5XTY9TC9V0PEMZKNQ8S -->
- [x] 3.2 【綠】實作 trayStyle 偏好讀寫與前端接線：apps/desktop/src/trayStyle.ts 落實 localStorage 單鍵讀寫、apps/desktop/src/store.ts 暴露偏好狀態與切換動作、apps/desktop/src/views/SettingsView.tsx 於 macOS 以 shared ui RadioGroup 呈現「系統匣樣式」二選（原生選單／面板），非 macOS 不顯示此偏好；切換即時持久化且不觸碰 .speclink.yaml 與 openspec/config.yaml。驗證：npm test -w apps/desktop 綠（trayStyle 模組、SettingsView 顯示邏輯與平台分流測試）。 <!-- speclink-task:tsk_01KXMQG5XT2YCM3EYC5GGHW49Y -->
- [x] 3.3 系統匣樣式分流接線：apps/desktop/src/tray.ts 依偏好切換互動樣式——native-menu 掛原生選單、panel 卸選單改掛圖示點擊事件（Tauri 不支援動態切換 showMenuOnLeftClick 時以重建 TrayIcon 實例等效，D4 載明），滿足修訂後規格「系統匣圖示與原生選單」的樣式分流敘述、切換無需重啟。驗證：npm test -w apps/desktop 綠（分流模型測試）；手動切換偏好觀察 tray 行為即時改變。 <!-- speclink-task:tsk_01KXMQG5XT635R09XA907GBKXQ -->

## 4. 面板視窗（D5 面板視窗架構（snapshot 推送）／D6 面板僅限 macOS）

- [x] 4.1 引入 macOS 條件相依：apps/desktop/src-tauri/Cargo.toml 以 macOS target 條件加入 tauri-nspanel、window-vibrancy、tauri-plugin-positioner 並於 apps/desktop/src-tauri/src/lib.rs 註冊（D6 面板僅限 macOS——非 macOS 建置不攜入）。驗證：macOS 上 cargo build --release -p speclink-desktop 成功；cargo metadata 確認三相依位於 macOS 條件區。 <!-- speclink-task:tsk_01KXMQG5XTK85VPCZ61WAQASH2 -->
- [x] 4.2 【紅】撰寫 apps/desktop/src/__tests__/trayPanel.test.tsx 紅測試：TrayPanel 元件以 TraySnapshot 渲染專案／生命週期分區變更（含進度）／討論清單（討論列以 slug 為題）、變更與討論列 hover 顯示複製鈕、點擊列本體發出開啟事件（攜帶 change name 或 discussion slug）。驗證：npm test -w apps/desktop 新測試紅。 <!-- speclink-task:tsk_01KXMQG5XTY00K0YZ99ACCEX3A -->
- [x] 4.3 【綠】實作面板呈現層：apps/desktop/panel.html＋apps/desktop/src/panel/main.tsx＋apps/desktop/src/panel/TrayPanel.tsx，apps/desktop/vite.config.ts 加多頁入口；主視窗於 store 去抖訂閱時將 TraySnapshot 經 Tauri event 推送至面板（不建第二條資料查詢路徑），面板動作事件回流主視窗執行既有 openDetail／openDiscussion，複製鈕走 clipboard 外掛。驗證：npm test -w apps/desktop 全綠；npm run build -w apps/desktop 產出面板入口。 <!-- speclink-task:tsk_01KXMQG5XT7JT32YMXCKR9N0YV -->
- [x] 4.4 面板原生化接線與失敗退回：lazy 建立面板視窗（無標題列、透明、skipTaskbar、alwaysOnTop）、tauri-nspanel 轉 nonactivating NSPanel 後顯式 window-vibrancy 套 popover 材質（類別交換後套用；panel 另設 set_opaque(false)＋set_transparent(true)）、positioner 貼齊 tray、show_and_make_key 顯示使 resign key 驅動失焦收合、點擊圖示 toggle、面板高度自適應內容（ResizeObserver → setSize，上限 640 後內部捲動）、複製鈕常駐列尾；面板建立失敗時退回原生選單並於設定頁浮出單行錯誤——滿足規格「面板樣式（macOS）」全部場景。驗證：cargo build --release -p speclink-desktop 後於 macOS 真實視窗手動驗證清單（面板貼齊圖示、毛玻璃於彩色背景下可辨、開啟不奪前景 app 焦點、失焦收合、高度貼合內容無多餘空白、主視窗隱藏時點常駐複製鈕成功）——依專案備忘 GUI 改動須真實視窗驗證，操作前確認使用者未在使用螢幕。 <!-- speclink-task:tsk_01KXMQG5XTMJP9EMBFPK9K4HP4 -->

## 5. 實測裁決（D7 實測裁決收尾）

- [x] 5.1 使用者於 macOS 實測裁決：以設定頁切換兩樣式 A/B 對照（操作手感、原生質感、資訊密度），明確裁決保留面板、保留原生選單、或兩者皆留（含「系統匣樣式偏好」設定項去留）。此任務無自動驗收——完成條件為使用者給出明確裁決；裁決若改變規格假設（拆除一側），後續以 /speclink-ingest 收斂，不在本變更內執行拆除。 <!-- speclink-task:tsk_01KXMQG5XTPMYM4650VPQ22P4A -->

## 6. 討論分流（D8 討論分流（討論中／已轉出）；ingest 2026-07-16）

- [x] 6.1 【紅】撰寫討論分流紅測試：apps/desktop/src/__tests__/tray.test.ts——TraySnapshot 討論項攜帶 promoted 布林（自 store 的 DiscussionItem.promotedTo 非空派生），buildTrayModel 產出「討論」分區（僅討論中）與其後「已轉出」分區（僅已轉出、子選單結構相同），無已轉出不出現該分區、無討論中顯示「討論 0」（規格「討論列表」分流場景）；apps/desktop/src/__tests__/trayPanel.test.tsx——TrayPanel 同分流呈現兩分區 header。驗證：npm test -w apps/desktop 新測試紅、其餘綠。 <!-- speclink-task:tsk_01KXMXCR0Z2BN3W0VKDY72T5HR -->
- [x] 6.2 【綠】實作分流：apps/desktop/src/tray.ts 的 toSnapshot 攜出 promoted、buildTrayModel 分兩分區（已轉出 header 於討論分區後、空則略）、apps/desktop/src/panel/TrayPanel.tsx 面板同構渲染（已轉出分區 header 帶 ArrowUpRight 圖示，與看板已轉出群組一致）、apps/desktop/src/i18n/messages.ts 新增「已轉出」鍵且 zh-TW／en key 集合相等。驗證：npm test -w apps/desktop 全綠；npm run build -w apps/desktop 成功。 <!-- speclink-task:tsk_01KXMXFBN7Z01GBK7Q6KMZE4SC -->

## 7. 分區溢出摺疊（D9 分區溢出摺疊（每分區 5 筆）；ingest 2026-07-16）

- [x] 7.1 【紅】撰寫分區溢出紅測試：apps/desktop/src/__tests__/tray.test.ts——buildTrayModel 於任一分區逾 5 筆時直列前 5、尾端產出「還有 N 個…」溢出節點（內嵌其餘項目的同構模型；5 筆以下無溢出節點——規格「分區溢出摺疊」門檻邊界 Example 5/6/20）；apps/desktop/src/__tests__/trayPanel.test.tsx——面板溢出列點擊展開其餘列、再點收合。驗證：npm test -w apps/desktop 新測試紅、其餘綠。 <!-- speclink-task:tsk_01KXMXGTRDDZZZ3ER1XQMM0ASH -->
- [x] 7.2 【綠】實作溢出摺疊：apps/desktop/src/tray.ts 的 buildTrayModel 分區切片＋overflow 節點（門檻 5 單點常數）、接線層遞迴轉原生子選單；apps/desktop/src/panel/TrayPanel.tsx 可展開溢出列（展開後高度自適應照常）；apps/desktop/src/i18n/messages.ts 新增「還有 {n} 個…」鍵且 zh-TW／en key 集合相等。驗證：npm test -w apps/desktop 全綠。 <!-- speclink-task:tsk_01KXMXM9DBJEX7DM4TK7M5Q1Q1 -->

## 8. 複製回饋（D5 面板視窗架構（snapshot 推送）的複製鈕回饋；ingest 2026-07-16）

- [x] 8.1 【紅】撰寫複製回饋紅測試：apps/desktop/src/__tests__/trayPanel.test.tsx——點擊面板複製鈕後圖示短暫轉為勾號（Check）、約 1.2 秒後復原為 Copy（規格「面板樣式（macOS）」常駐複製鈕回饋場景；沿用看板 ChangeList 的 copied 模式）。驗證：npm test -w apps/desktop 新測試紅。 <!-- speclink-task:tsk_01KXMYEGQBPXXYP2NM35SQDW0B -->
- [x] 8.2 【綠】實作複製回饋：apps/desktop/src/panel/TrayPanel.tsx 的 CopyButton 加 copied 狀態（點擊後 Check 圖示、1200ms 復原，Check 帶 primary 色與 group-hover 反白）。驗證：npm test -w apps/desktop 全綠。 <!-- speclink-task:tsk_01KXMYF6CXT0A5DYZS2DMHEM28 -->

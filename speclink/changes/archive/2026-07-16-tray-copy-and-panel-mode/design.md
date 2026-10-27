## Context

系統匣現況（apps/desktop/src/tray.ts）分兩層：buildTrayModel 純函式（store 快照 → 選單模型，jsdom 直測）與接線層（Tauri JS tray/menu API、去抖重建）。變更項已是子選單（唯一動作「開啟此變更」），討論項是單層項（點擊即開啟、標籤為 topic）。詳情抽屜已有複製先例：變更複製 name、討論複製 slug（LANGUAGE.md 識別錨點例外）。討論 tray-copy-title-and-native-status 裁定：複製入選單、討論項 slug 化、新增 webview 面板樣式與原生選單並存、設定頁切換、使用者實測後裁決去留。

本變更全落在 apps/desktop（Tauri 殼 src-tauri＋desktop core crate＋前端 src），speclink-core／speclink-cli 完全不動——無 CLI 輸出相容性議題、無回歸對照影響。

## Goals / Non-Goals

**Goals:**

- 原生選單內可複製變更 name 與討論 slug，且在主視窗隱藏／無焦點時仍可靠
- 討論項標籤與看板卡一致（slug 為題），topic 降為描述
- 提供可實測對照的第二種系統匣樣式（面板），質感盡可能貼近原生（不搶焦點、毛玻璃、貼齊圖示、失焦收合）
- 兩樣式以 app 本機偏好即時切換，原生選單程式零拆除

**Non-Goals:**

- 不 fork muda 補原生選單自訂 view；不動 crates/（speclink-core、speclink-cli）
- 不改 .speclink.yaml 與 openspec/config.yaml；不擴充 macOS 徽章文字資訊
- 不預寫裁決結果進規格；裁決後的拆除（若有）走 ingest 另收斂
- 面板不做 Windows／Linux 支援（見 D6）

## Decisions

### D1 原生選單複製動作

變更子選單在「開啟此變更」之後新增「複製名稱」項，選取即把該變更的 name 寫入系統剪貼簿（不含進度條字元、不帶完成數）。模型層 TrayChangeAction 由單一 kind 擴充為 open-change | copy-name 的動作聯集，buildTrayModel 保持純函式可測。替代方案：複製標籤全文——夾帶 Unicode 進度條雜訊，已於討論排除。

### D2 討論項 slug 化與子選單

討論項由單層項改為子選單，父項標籤改為 slug 直出；子選單依序含：topic 灰字描述行（disabled，僅供辨識）、「開啟此討論」、「複製 slug」。此結構與看板討論卡「slug 為題、topic 為副標」對齊，並解掉「原生選單帶子選單的父項不觸發點擊」的限制。openspec/LANGUAGE.md 識別錨點例外的枚舉面同步擴充（納入系統匣討論項與其複製動作），比照 desktop-ux-polish 的範圍擴充記法。替代方案：保留單層項點擊即開——無處放複製動作，已於討論排除；topic 捨棄不呈現——喪失辨識資訊，灰字行零互動成本，故保留。

### D3 剪貼簿經 clipboard-manager

複製一律走 tauri-plugin-clipboard-manager（Rust 端寫剪貼簿）：src-tauri 註冊外掛、capabilities/default.json 授權 clipboard-manager:allow-write-text、前端以 @tauri-apps/plugin-clipboard-manager 的 writeText 呼叫。面板內的 hover 複製鈕同走此通道（面板視窗常態無焦點，navigator.clipboard 必拒寫）。替代方案：navigator.clipboard——tray 點擊時主視窗可能隱藏／無焦點會拒寫，已於討論排除。packages/ui 詳情抽屜既有複製鈕不在本變更範圍（視窗必有焦點、行為正確），不順手改。

### D4 系統匣樣式偏好與設定頁切換

app 本機偏好比照既有 UI 語言偏好（apps/desktop/src/i18n/locale.ts）與分頁列（apps/desktop/src/tabs.ts）的模式：新增前端單鍵 localStorage 模組 apps/desktop/src/trayStyle.ts，值 native-menu | panel，缺鍵或非法值（手改 localStorage）一律視為 native-menu——舊安裝無此鍵時行為不變（向後相容）。SettingsView 於 macOS 顯示「系統匣樣式」二選（原生選單／面板），沿用本機設定簽既有的介面語言 pill 按鈕樣式（@speclink/ui 無 RadioGroup；實作時修正）；切換即時生效無需重啟：切至 panel 時 tray 卸下選單並改掛點擊事件（開閉面板），切回時重掛選單。若 Tauri TrayIcon 不支援動態切換 showMenuOnLeftClick，以重建 TrayIcon 實例達成等效。i18n：messages.ts 新增鍵（系統匣樣式、原生選單、面板、複製名稱、開啟此討論、複製 slug 等），zh-TW／en 字典 key 集合維持相等。替代方案：存 openspec/config.yaml——UI 偏好非專案設定，且該檔解析失敗會使政策靜默退回預設，風險不對稱，排除；存 apps/desktop/core/src/settings.rs——該層職責是專案兩檔（.speclink.yaml／config.yaml）的雙重驗證橋接，UI 偏好落此違反職責邊界，且既有 app 本機偏好（語言、分頁）均走前端 localStorage，排除（實作時修正）。

### D5 面板視窗架構（snapshot 推送）

面板為第二個 webview 視窗：獨立 HTML 入口 apps/desktop/panel.html（vite 多頁入口）＋ apps/desktop/src/panel/main.tsx 掛 React root、TrayPanel.tsx 渲染。視窗屬性：無標題列、透明背景、skipTaskbar、alwaysOnTop；經 tauri-nspanel 轉為 nonactivating NSPanel（開啟不搶走目前 app 焦點）；vibrancy 走 tauri 內建 window effects（NSVisualEffectView popover 材質——直加 window-vibrancy 會與 tauri 內建版本重複連結 objc 符號，實作時修正；需 macOSPrivateApi 讓 webview 背景透明）；tauri-plugin-positioner 以 tray 相對位置（TrayCenter／TrayBottomCenter）定位；面板 blur 即自動隱藏（面板以 show_and_make_key 顯示——nonactivating panel 成為 key 才收得到 resign key，app 仍不 activate）；點擊系統匣圖示 toggle 顯隱；首次切至面板樣式才建立視窗（lazy），之後隱藏重用。面板高度自適應內容：入口層以 ResizeObserver 量測內容實高、經視窗 setSize 即時貼合（寬 320 固定、高上限 640，超過由面板內部捲動；rubber-band 以 overscroll-behavior 關閉）。複製鈕常駐列尾（不採 hover 才顯示——使用者實測回饋修正），點擊後短暫轉勾號回饋（Copy→Check、1.2 秒復原——沿用看板 ChangeList 複製鈕既有模式；使用者實測回饋補充）；分區圖示沿用看板同款 lucide（提案中 Lightbulb、進行中 Hammer、已就緒 CircleCheckBig、討論 MessageSquareText）。

資料流維持「與看板同一 store、不建第二條查詢路徑」的既有規格約束：主視窗（tray 擁有者）沿用既有 TraySnapshot 模型，於 store 去抖訂閱時把快照經 Tauri event 推送給面板視窗；面板是薄渲染層，不自建 store、不直呼資料查詢指令。互動回流：開啟變更／討論 → 面板 emit 事件回主視窗執行既有 openDetail／openDiscussion 並顯示主視窗；複製 → 面板直呼 clipboard 外掛（D3）。介面縫檢核：契約落在既有 TraySnapshot 型別（tray.ts 已有、面板複用同型別作 payload），單一 adapter、無疊套薄轉發；刪除面板模組不影響原生選單樣式（deletion test 通過）。替代方案：面板自帶 store 直呼 Tauri commands——形成第二條資料查詢路徑，違反 tray-status-menu 既有同源需求，排除。

### D6 面板僅限 macOS

tauri-nspanel 與 popover 材質為 macOS 專屬，且「statusbar 原生設計」訴求本就是 macOS 情境：面板樣式僅在 macOS 提供；非 macOS 平台設定頁不顯示「系統匣樣式」偏好，系統匣固定原生選單（含 D1／D2 的複製與 slug 化，跨平台皆得）。相依外掛以 macOS target 條件引入，避免 Windows／Linux 建置攜入無用相依。替代方案：Windows 以一般 always-on-top 視窗模擬面板——質感差且非本次訴求，不做。

### D7 實測裁決收尾

最後一項任務為使用者實測裁決：在 macOS 真實環境切換兩樣式對照（操作手感、原生質感、資訊密度），決定保留面板、保留原生選單、或兩者皆留（偏好項隨之去留）。此任務無自動驗收——產物是使用者的明確裁決記錄；裁決若改變規格假設（例如拆除一側），以 /speclink-ingest 收斂規格與後續拆除工作，不在本變更內預做。

**裁決記錄（2026-07-16）**：使用者實測後裁決**保留 webview 面板、移除原生選單樣式**。拆除依本變更範圍界線（out of scope＝裁決後拆除）另立變更收斂；拆除範圍須顧及跨平台——面板僅 macOS，非 macOS 平台的系統匣仍以原生選單運作。

### D8 討論分流（討論中／已轉出）

系統匣的討論區（原生選單與面板共用同一模型）分兩個分區：「討論」列討論中（promotedTo 為空）的討論、其後「已轉出」列已轉出變更的討論，無已轉出時不顯示該分區——與看板討論欄「討論中為主、已轉出收合」的語意對齊（使用者實測後裁決，ingest 2026-07-16）。分流把手為既有 DiscussionItem.promotedTo 清單（看板同一判準），TraySnapshot 討論項增帶 promoted 布林、buildTrayModel 據此分區，兩分區子選單結構相同。替代方案：只列討論中（已轉出入口交給其變更）——資訊面較窄，使用者裁決保留直達已轉出討論的入口；混列加標記——雜訊仍在，排除。

### D9 分區溢出摺疊（每分區 5 筆）

各分區（生命週期三階段、討論、已轉出）直列前 5 筆，第 6 筆起收進尾端「還有 N 個…」節點——避免項目多時 tray 淪為長捲軸、喪失一瞥性（使用者實測後裁決，ingest 2026-07-16）。模型層落實於 buildTrayModel：溢出節點攜帶其餘項目的同構模型（change／discussion 項原樣內嵌），接線層遞迴轉原生子選單（macOS 選單超高原生捲動）；面板以可展開列對應（webview 無子選單概念，點擊展開／收合，展開後高度自適應與上限捲動照常）。門檻 5 為字面常數（模型層單點定義）。替代方案：無上限全列——數十項時一瞥性差，使用者裁決收摺；門檻做成設定——臆測性彈性，排除。

## Implementation Contract

- **行為（原生選單）**：變更子選單含「開啟此變更」「複製名稱」兩項；選「複製名稱」後系統剪貼簿內容等於該變更 name（純文字、無進度條字元）。討論區分「討論」（討論中）與「已轉出」（promoted）兩分區，無已轉出時不顯示後者；每則討論為子選單：父項標籤＝slug；子選單含 topic 灰字行（不可選取）、「開啟此討論」（顯示主視窗＋開啟該討論）、「複製 slug」（剪貼簿內容等於 slug）。主視窗隱藏或無焦點時複製仍成功。
- **行為（面板樣式，僅 macOS）**：設定頁切至「面板」後，點擊系統匣圖示不再出現原生選單，改為在圖示下方彈出面板：呈現與原生選單同源的專案／變更（含進度）／討論清單（討論同樣分「討論」「已轉出」兩分區），變更與討論列列尾常駐複製鈕（行為同上）、點列本體開啟主視窗對應詳情；面板高度自適應內容（上限 640 後內部捲動）；面板開啟不奪取目前前景 app 焦點（不 activate app）；點面板外任意處（resign key）面板收合。切回「原生選單」後行為完全回復。
- **介面／資料形**：app 本機偏好新增 trayStyle 單鍵（localStorage，值 native-menu | panel，缺鍵或非法值一律解為 native-menu——舊安裝向後相容）；主視窗 → 面板的推送 payload 為既有 TraySnapshot 型別（tabs、activeRoot、changes、discussions）經 Tauri event 傳遞；面板 → 主視窗的動作事件攜帶 change name 或 discussion slug。無新增 speclink CLI 介面。
- **失敗模式**：剪貼簿寫入失敗（外掛未授權等）不彈窗、不中斷選單／面板——靜默不寫（與詳情抽屜複製鈕現行 void 語意一致）；面板視窗建立失敗時系統匣退回原生選單樣式且設定頁浮出單行錯誤。
- **驗收**：npm test -w apps/desktop 全綠——tray.test.ts 覆蓋模型層新結構（變更動作聯集、討論子選單、slug 標籤、topic 灰字行、樣式偏好對模型的分流），面板元件測試覆蓋渲染與動作 emit；cargo build --release -p speclink-desktop 成功；macOS 真視窗手動驗證面板三性質（不搶焦點、貼齊圖示、失焦收合）與無焦點複製——依專案備忘 GUI 改動須真實視窗驗證。
- **範圍界線**：in scope＝apps/desktop 全部前述改動＋openspec/LANGUAGE.md 枚舉擴充；out of scope＝crates/ 一切、packages/ui 既有複製鈕、Windows／Linux 面板、徽章文字擴充、裁決後拆除。

## Risks / Trade-offs

- [tauri-nspanel／window-vibrancy 與現用 Tauri 版本不相容或行為缺陷] → 外掛僅 macOS target 引入、面板 lazy 建立；不相容時面板任務可獨立暫停，原生選單樣式（含複製）不受牽連——這正是兩樣式並存的保險。
- [面板實測後質感仍不夠原生] → 設計上已預留裁決任務（D7）；最壞情況拆面板走 ingest，複製與 slug 化成果保留。
- [動態切換 showMenuOnLeftClick 在 Tauri JS API 不可行] → 以重建 TrayIcon 實例等效替代（D4 已載明），tray.ts 接線層 dispose 已具備清理路徑。
- [第二 webview 記憶體成本] → lazy 建立＋隱藏重用；未切至面板樣式的使用者零成本。
- [跨平台回歸：clipboard 外掛註冊與 capabilities 改動影響 Windows 建置] → clipboard-manager 三平台皆支援、正常引入；僅 nspanel／vibrancy／positioner 的 macOS 條件化要在 Windows CI／本機建置驗證通過。
- [jsdom 測不出面板互動失效（專案既知地雷）] → 驗收明定 macOS 真視窗手動驗證清單（Implementation Contract），不以單元測試綠燈宣稱完成。

## Migration Plan

無資料遷移：trayStyle 缺鍵即現行為（native-menu），舊安裝的 app 本機偏好不受影響。部署即一般桌面版建置；回滾＝設定切回原生選單或還原版本，無持久狀態需清理。

## Open Questions

無——面板版型細節以看板同源清單為準（D5），最終樣式去留由 D7 裁決任務產出。

## MODIFIED Requirements

### Requirement: 全部 browser route 由單一 SPA 提供可發現導覽

Server SHALL 以 Vite、React 與 TypeScript SPA 提供 `/`、`/setup`、`/invite/:token`、`/login`、`/activate`、`/account`、`/admin` 與 `/admin/*`；每個頁面 SHALL 可由應用內導覽或當前流程中的明確動作到達，且 SHALL 支援直接開啟與重新整理。未登入流程 SHALL 使用專注流程殼；已登入的管理員與一般成員 SHALL 共用同一個依角色裁切的主控台殼：左側側欄（頂端品牌字標、中段導覽、底部帳號選單）加主欄（頂列麵包屑、下方主內容），殼 SHALL NOT 含橫貫頂部的 header。管理員的側欄導覽 SHALL 提供總覽、使用者、專案與儲存庫、憑證、系統、稽核紀錄六個目的地，且 SHALL NOT 提供獨立的資料操作目的地；一般成員的側欄導覽 SHALL 只有帳號一項，SHALL NOT 呈現任何管理目的地。帳號 SHALL NOT 列為管理員側欄的目的地，SHALL 由側欄底部帳號選單的「帳號」項進入。帳號選單 SHALL 在觸發鈕上顯示當前使用者的顯示名稱與電子郵件，展開後 SHALL 顯示顯示名稱與「電子郵件 · 管理員」或「電子郵件 · 成員」，並提供帳號、介面語言與登出；登出請求失敗時 SPA SHALL 仍導向登入頁。管理員開啟 `/account` 時側欄 SHALL 完整呈現且無任何項目高亮。主區頂列的麵包屑 SHALL 標示目前位置：管理面為「後台 / <目的地>」且「後台」可點回總覽，帳號頁為「帳號」。SPA SHALL NOT 提供 changes、specs 或 discussions 的檢視與編輯。

#### Scenario: 管理員不需手打 URL 走訪管理功能

- **WHEN** 已登入管理員從 `/` 進入服務並依可見導覽走訪全部管理目的地
- **THEN** 管理員可經側欄到達六個管理目的地、經側欄底部帳號選單到達帳號頁與登出動作，且不需修改瀏覽器網址

#### Scenario: 管理員在帳號頁保有全站導覽

- **WHEN** 已登入管理員由帳號選單的「帳號」項進入 `/account`
- **THEN** 側欄六個目的地維持可見且無項目高亮，麵包屑為「帳號」，管理員可直接前往任一管理目的地而不使用瀏覽器上一頁

#### Scenario: 一般成員不顯示管理導覽

- **WHEN** 已登入但無 admin 旗標的成員開啟 `/account`
- **THEN** 頁面顯示帳號、存取金鑰、登入工作階段與裝置資訊；側欄只有作用中的「帳號」一項與帳號選單，不呈現任何管理目的地

#### Scenario: 資料操作目的地已併入系統

- **WHEN** 已登入管理員檢視側欄並開啟系統目的地
- **THEN** 側欄不含資料操作項目，系統頁單頁呈現執行環境、儲存狀態、匯出與資料結構遷移

#### Scenario: 深連結可直接重新整理

- **WHEN** 管理員直接開啟或重新整理 `/admin/audit`
- **THEN** Server 回傳 SPA shell，SPA 完成 session 檢查後呈現稽核紀錄頁

#### Scenario: 帳號選單提供帳號、語言與登出

- **WHEN** 已登入管理員點側欄底部顯示自己名稱與電子郵件的帳號選單
- **THEN** 選單頂端顯示顯示名稱與「電子郵件 · 管理員」，下方依序為帳號、介面語言（中文、英文、跟隨系統三選一且標示目前選擇）與登出；選登出後 session 撤銷並呈現登入頁

#### Scenario: 登出失敗仍回到登入頁

- **WHEN** 已登入使用者在帳號選單選登出，但登出請求回傳錯誤或網路中斷
- **THEN** SPA 導向登入頁，不停留在仍顯示帳號資訊的主控台殼

#### Scenario: 麵包屑標示位置並可回總覽

- **WHEN** 管理員開啟 `/admin/users` 後點麵包屑的「後台」
- **THEN** 點擊前麵包屑為「後台 / 使用者」，點擊後呈現總覽且麵包屑為「後台 / 總覽」

### Requirement: 共用設計系統維持高密度可存取體驗

Server SPA SHALL 復用 `packages/ui` 的 shadcn/ui 原語、共用 semantic theme、Noto Sans TC 與青綠 focus token，SHALL NOT 建立第二套 theme 或 import Desktop 的 Tauri、Zustand store 與 SDD board 元件。介面上的下拉選單 SHALL 使用 `packages/ui` 的 shadcn/ui Select 原語，SHALL NOT 使用未套用 theme 的原生 `select`；同一組工具列內的文字輸入、下拉與日期輸入 SHALL 具有相同高度、內距、圓角與陰影。下拉展開後的選單 SHALL 具有不透明底色，且在抽屜或對話框內開啟時 SHALL 可正常開啟與選取。主控台殼的側欄頂列與主區頂列高度、側欄寬度與主內容內距 SHALL 與 Desktop 應用程式殼採用相同數值；側欄頂列 SHALL 顯示共用元件庫的橫式字標，主區頂列的麵包屑 SHALL 使用與 Desktop 同一個共用麵包屑元件。主控台殼 SHALL NOT 整頁捲動——側欄與主區頂列 SHALL 恆常留在可視範圍內，只有主內容區捲動。寬度至少 1024px 時主控台殼 SHALL 顯示 icon 加 label 的固定側欄；更窄時 SHALL 隱藏固定側欄，以主區頂列上可見的 trigger 開啟從左側滑出的 Sheet，Sheet 內容 SHALL 與側欄相同（字標、該角色的導覽目的地與帳號選單），管理員與一般成員皆同。每頁標題 SHALL 使用與 Desktop 同一個共用頁標題元件（24px 一般字重標題、灰字說明、右側頁級動作），且頁標題 SHALL 為該頁唯一的第一層標題。專注流程殼 SHALL 於畫面正中呈現橫式字標，其下為一張與 Desktop 對話框同款的卡片（12px 圓角、15px 標題加灰字說明、頁尾細線上方右側為主要動作）；多步驟流程 SHALL 於卡片頂端標示「步驟 n / 總數」與步驟條；連結無效或無權限時卡片 SHALL 只呈現警示色圖示、標題與說明並以 `role=alert` 宣告。每頁 SHALL 只有一個視覺 primary action，手機版資料 SHALL 轉為可換行 row 或 card 且 SHALL NOT 造成整頁水平捲動。

字級與控件高度 SHALL 與 Desktop 相同：側欄、麵包屑與選單文字 11–13px，閱讀內文 15px，按鈕高度 28–36px。所有互動 SHALL 可由鍵盤完成，並包含 skip link、至少 2px focus ring、連續 heading、icon-only control 的 aria-label、輸入 label／helper text／autocomplete、鄰近欄位且以 `role=alert` 宣告的錯誤與至少 4.5:1 正常文字對比。動畫 SHALL 限於 150–250ms opacity 或 transform；`prefers-reduced-motion` 啟用時 SHALL 停用動畫。

#### Scenario: 375px 完成主要流程

- **WHEN** 使用者以 375px viewport、200% zoom 與鍵盤操作 login、setup 或 invite 流程
- **THEN** 主要內容與操作保持可見、focus 順序合理、沒有整頁水平捲動，且可完成提交

#### Scenario: 窄螢幕管理導覽使用 Sheet

- **WHEN** 管理員以 768px viewport 開啟 `/admin` 並按主區頂列的開啟導覽鈕
- **THEN** 固定側欄不顯示，Sheet 從左側滑出並含字標、六個目的地與帳號選單，關閉後 focus 回到開啟導覽鈕

#### Scenario: 窄螢幕一般成員同樣使用 Sheet

- **WHEN** 一般成員以 375px viewport 開啟 `/account` 並按主區頂列的開啟導覽鈕
- **THEN** 固定側欄不顯示，Sheet 從左側滑出並含字標、「帳號」一項與帳號選單

#### Scenario: 管理殼版面數值與 Desktop 一致

- **WHEN** 管理員以 1440px viewport 開啟任一管理目的地
- **THEN** 側欄頂列與主區頂列高度、側欄寬度與主內容內距與 Desktop 應用程式殼相同，且頁面沒有橫貫頂部的 header

#### Scenario: 只有主內容區捲動

- **WHEN** 管理員開啟內容長度超過視窗高度的管理目的地並向下捲動
- **THEN** 側欄與主區頂列維持在原位，只有主內容區捲動

#### Scenario: 工具列控件高度一致

- **WHEN** 管理員檢視同時具有關鍵字搜尋與下拉篩選的列表工具列
- **THEN** 搜尋輸入、下拉篩選與日期輸入的高度相同，且下拉展開後的選單套用與其他控件相同的 theme

#### Scenario: 抽屜內的下拉可正常開啟

- **WHEN** 管理員在抽屜內開啟下拉選單並選取一個選項
- **THEN** 選單以不透明底色呈現於抽屜內、選取生效，且畫面不失去回應

#### Scenario: reduced motion 停用轉場

- **WHEN** 作業系統設定 `prefers-reduced-motion: reduce`
- **THEN** SPA 不執行 route、Sheet、toast 或狀態切換動畫，且功能狀態仍完整可辨識

#### Scenario: 頁標題與 Desktop 同款

- **WHEN** 管理員開啟使用者頁
- **THEN** 頁頂為該頁唯一的第一層標題「使用者」（24px 一般字重），其下為灰字說明，右側為「邀請使用者」動作

#### Scenario: 控件密度與 Desktop 相同

- **WHEN** 管理員以鍵盤在任一管理目的地的側欄、麵包屑與頁級按鈕間移動焦點
- **THEN** 導覽項、麵包屑與選單文字為 11–13px，按鈕高度落在 28–36px，且每個取得焦點的控件顯示至少 2px focus ring

#### Scenario: 專注流程以置中字標與對話框式卡片呈現

- **WHEN** 未登入使用者開啟 `/login` 並以錯誤的密碼提交
- **THEN** 畫面正中為橫式字標，其下卡片含 15px 標題「登入」、灰字說明、電子郵件與密碼欄位，頁尾細線上方右側為「登入」；提交後錯誤以 `role=alert` 緊貼欄位下方，已輸入的電子郵件保留

#### Scenario: 初始設定標示步驟

- **WHEN** 持有效 bootstrap token 的使用者開啟 `/setup` 並完成建立管理員
- **THEN** 卡片頂端先顯示「步驟 1 / 2」且步驟條第一段為主色；建立成功後改為「步驟 2 / 2」且兩段皆為主色

#### Scenario: 連結無效以提示卡呈現

- **WHEN** 使用者以已失效的初始設定或邀請連結開啟頁面
- **THEN** 卡片內只有警示色圖示、標題與說明，以 `role=alert` 宣告，不呈現任何表單欄位

### Requirement: 介面語言支援中文與英文

Server SPA SHALL 以中文（zh-TW）與英文（en）提供全部使用者可見文案，兩種語言的訊息集合 SHALL 完全對應。使用者未明示偏好時 SHALL 依瀏覽器語言決定——以 `zh` 開頭者為中文，其餘為英文。已登入時介面 SHALL 於側欄底部帳號選單提供語言切換，可選中文、英文或跟隨系統，且 SHALL 標示目前選擇；專注流程殼 SHALL 於卡片下方提供中文與英文兩個切換鈕並標示目前語言。切換 SHALL 立即生效並在重新載入後維持，兩處切換 SHALL 寫入同一個偏好。語言選擇 SHALL NOT 影響 artifacts 產出語言、CLI 輸出、稽核事件的動作名稱或伺服器回傳的錯誤訊息。

#### Scenario: 未設定偏好時跟隨瀏覽器語言

- **WHEN** 尚未選過語言的使用者以英文瀏覽器開啟管理面
- **THEN** 介面文案為英文

#### Scenario: 切換語言即時生效並持續

- **WHEN** 管理員在帳號選單將語言切換為英文並重新整理頁面
- **THEN** 管理面文案維持英文，不因重新載入而還原

#### Scenario: 兩種語言的訊息集合對應

- **WHEN** 檢視任一管理目的地的中文與英文版本
- **THEN** 兩者呈現相同的欄位、動作與說明，沒有任何一方缺漏或退回顯示訊息代碼

#### Scenario: 跟隨系統清除偏好

- **WHEN** 已選過英文的使用者在帳號選單選「跟隨系統」，且瀏覽器語言為 zh-TW
- **THEN** 偏好被清除，介面文案立即回到中文，重新整理後仍為中文

#### Scenario: 登入前可切換語言

- **WHEN** 尚未選過語言、瀏覽器為 zh-TW 的使用者在登入頁點卡片下方的「English」後登入
- **THEN** 登入頁立即改為英文且「English」標示為目前語言，登入後的管理面文案也是英文

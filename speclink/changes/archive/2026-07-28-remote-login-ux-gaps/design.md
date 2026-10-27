## Context

三個斷點的資料全數已在系統內：memberships 由 server 為管理頁計算、device 授權資訊由初始化回應帶回、授權結果由輪詢取得。本 design 決定呈現層怎麼接，關鍵是 desktop 的 device login 編排——現行為單一阻塞呼叫，UI 拿不到中途狀態，必須分段。

範圍內：server account summary 擴充、server-web 帳號頁與核准頁、desktop 連線編排分段與伺服器頁籤 UI。範圍外：授權狀態機、credential 處理、workspace 開啟流程本體。

## Goals / Non-Goals

- Goal: 三個斷點補齊後，登入鏈從按下登入到開工作區全程有可見狀態與下一步。
- Goal: desktop 分段後每段可獨立測試（Rust 單元＋前端 store 測試）。
- Non-Goal: 不做授權流程的自動化銜接（自動開 chooser、自動輪詢背景連線）。

## Decisions

### 決策一：memberships 進 account summary，UI 共用一個元件

server 端把既有的隸屬查詢（管理頁已在用）套用到 account summary 的當前使用者，payload 增 memberships 陣列（每項 projectKey、projectName、role，camelCase）；隸屬與角色查詢逐專案進行，與管理頁同一資料來源故無語意分岔。web 端 AccountPage 新增我的專案區塊，admin 與一般成員走同一元件——兩種殼都經 header 帳號入口到達 /account，無需第二進入點。替代方案：成員版另開 /projects 頁——被否：單一區塊資訊量小，獨立頁徒增導覽面；管理殼的側欄目的地是凍結清單（server-web-console 規格），加頁要動殼。

### 決策二：desktop device login 由單一阻塞呼叫改為分段編排

現行編排把靜默 refresh、初始化、開瀏覽器、輪詢迴圈、身分寫回全包在一個同步呼叫內，UI 只能顯示忙碌。改為兩段 IPC：
- 第一段（啟動）：靜默 refresh 快路徑照舊——成功直接回已登入；否則初始化＋開瀏覽器，回傳等待授權資訊（裝置碼、驗證網址、有效期限、輪詢間隔）。
- 第二段（單次觀測）：對授權請求做一次輪詢，回傳目前狀態（pending／slow_down／approved／denied／expired）；approved 時完成 credential 存入與身分寫回後回已登入。
輪詢節奏歸前端 store：依啟動段回傳的間隔排程單次觀測，收到 slow_down 加大間隔，倒數歸零或收到終態即停。取消＝store 停止排程並清除等待狀態——無須通知 server（授權請求自然逾期），Rust 側無長駐迴圈可中斷、無取消旗標。替代方案：保留 Rust 迴圈＋事件推播＋取消 command——被否：同一流程狀態分裂在 Rust 迴圈與前端 store 兩處，取消需跨執行緒旗標，測試面大；分段後 Rust 每段皆為純請求-回應，login_orchestration 測試直接覆蓋。取捨：輪詢排程移到前端後，app 進入背景時排程可能被節流——等待授權面本就要求使用者在場，可接受。

### 決策三：等待授權面是連線互動狀態的新變體

連線互動狀態（既有 idle／busy／error／notice／patInput）新增等待授權變體，承載裝置碼、驗證網址、截止時刻與剩餘秒數；ServersPanel 據此渲染碼（等寬字體）、兩個複製鈕（沿用系統匣複製 slug 的既有複製語彙與 toast 回饋）、倒數與取消鈕。倒數以截止時刻減現在時間計算、每秒更新，不依賴輪詢節奏。替代方案：彈出獨立 dialog——被否：連線列就地展開與 PAT 輸入的既有模式一致，dialog 多一層焦點管理。

登入回饋跟著發起登入的介面走：工作區選擇器的「新增並登入」走同一個 store 流程（connectionPhases），選擇器的 server 步驟據該連線的 phase 就地渲染等待授權面、PAT 輸入或錯誤——不得只留靜態「已新增」提示、要求使用者切到設定頁才看得到狀態。等待授權面與 PAT 輸入抽成共用元件，兩處無重複實作；store 的 addConnection 回傳正規化 origin，供選擇器追蹤剛新增連線的互動狀態。

### 決策四：登入成功的行動呼籲沿用 focus 管理既有模式

登入成功後該連線列的開啟工作區鈕取得鍵盤焦點並以主色顯眼呈現（既有 reauth 聚焦模式的同款 ref 機制），不自動開 chooser。web 核准結果頁於核准與拒絕兩種結果補收尾指引文字（i18n 中英兩版）。替代方案：成功後自動開 chooser——被否（討論已裁定：打斷 recoverRemoteSessions 與多 server 登入流）。

### 決策五：向後相容與序列化

account summary 為 browser API：新增 memberships 欄位、既有欄位不動，舊 SPA 讀新 payload 不受影響（多欄位忽略）。desktop 的 IPC 形狀變更屬 app 內部（前後端同版出貨），無跨版相容需求。無設定檔變更。

## Risks / Trade-offs

- 前端排程輪詢在系統睡眠後恢復：以截止時刻判定逾時（非累計計數），醒來即正確顯示逾時或續輪。
- 倒數與 server 的實際逾時存在時鐘偏差：倒數僅為引導性顯示，真實終態以輪詢回應為準。
- GUI 互動（複製鈕、焦點、倒數）jsdom 測不出真實體感：緩解——依開發備忘於真實視窗驗證（macOS 以安裝版 bundle 掛新 binary）。
- 兩段 IPC 之間 app 重啟：等待狀態不持久化，重啟後重新登入即可（授權請求自然逾期，無殘留）。

## Migration Plan

純新增與內部編排改動，同版前後端一起出貨，無遷移步驟。

## Open Questions

（無）

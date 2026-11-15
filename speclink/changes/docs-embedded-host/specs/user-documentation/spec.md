## REMOVED Requirements

### Requirement: 官方 server 定位為參考實作而非唯一路徑

**Reason**: 官方 server 的定位從「參考實作」改成「示範與試用」；遠端專案的正式路徑是把引擎內嵌進自己的系統。需求名稱裡的「參考實作」不再成立

**Migration**: 由「官方 server 定位為示範與試用」承接：定位句、出現的位置，以及能力對照表標明量測對象的情境

## ADDED Requirements

### Requirement: 官方 server 定位為示範與試用

使用者文件描述遠端專案時 SHALL 說明 `speclink-server` 是示範與試用用的，並 SHALL 把正式使用導向把引擎內嵌進自己的系統。定位句 SHALL 逐字出現：繁體中文版是「官方 server 是示範與試用用的。它讓你不寫程式就能試遠端專案。正式使用請把引擎內嵌進自己的系統。」，英文版是「The official server is for demonstration and trial. It lets you try remote projects without writing code. For production use, embed the engine in your own system.」。定位句 SHALL 出現於 README 中英兩版的「試用官方 server」一節的開頭、Remote 入門教學中英兩版、Server 部署文件中英兩版的開頭，以及 Server 備份與還原文件中英兩版的開頭。文件 SHALL 說明官方 server 與內嵌的宿主用同一套宿主套件與同一份儲存契約組成，CLI 與 Speclink Desktop 對兩者同樣可用。文件 SHALL NOT 把官方 server 稱為參考實作，SHALL NOT 把它寫成正式環境的建議做法。

#### Scenario: 讀者判斷得出正式使用的路徑

- **WHEN** 讀者在 README 或 Remote 入門教學（中英任一版）讀到官方 server
- **THEN** 可見定位句，以及通往「把 Speclink 內嵌進你的系統」的連結

#### Scenario: 營運文件開頭標明定位

- **WHEN** 讀者開啟 Server 部署或 Server 備份與還原文件（中英任一版）
- **THEN** 第一個 H2 之前可見定位句

#### Scenario: 能力對照表標明量測對象

- **WHEN** 讀者查閱本地與遠端能力對照表
- **THEN** 表格載明遠端一欄以官方的示範用 server 為量測對象，並說明內嵌的宿主行為相同、差異列在已知缺口

### Requirement: 使用者文件採用本機專案與遠端專案的用詞

README 與 docs SHALL 用下列用詞描述兩條路徑與整合的概念：使用者看到的是本機專案、遠端專案、Server（英文版 local project、remote project、Server）；整合者看到的是宿主、內嵌、SDK、宿主套件、儲存接口、入口、合格測試（英文版 host、embed、SDK、host kit、store port、entry point、conformance tests）。SDK SHALL 只指 npm 套件 `@speclink/engine` 這個工具箱，SHALL NOT 用來統稱整塊遠端功能。README 與 docs 的中英兩版 SHALL NOT 出現「Remote Store」「Self-hosting」「自架 server」「團隊模式」「主機套件」。`speclink/LANGUAGE.md` SHALL 有本機專案、遠端專案、宿主、內嵌、SDK、宿主套件、儲存接口、入口、合格測試的詞條；遠端專案的 avoid SHALL 含「團隊模式」，宿主套件的 avoid SHALL 含「主機套件」；「儲存後端」詞條的定義 SHALL 指示範用 server 的儲存（三個儲存接口範例之一）與它的狀態，SHALL NOT 提到 TeamStore。

#### Scenario: 舊的說法不再出現

- **WHEN** 在 README 與 docs 的中英兩版搜尋 Remote Store、Self-hosting、自架 server、團隊模式、主機套件
- **THEN** 沒有命中

#### Scenario: 詞彙表有新的詞條

- **WHEN** 檢視 `speclink/LANGUAGE.md`
- **THEN** 可見九個新詞條各自的定義、avoid 與理由；儲存後端的定義不含 TeamStore

#### Scenario: 詞彙守門涵蓋新的 avoid 詞

- **WHEN** 在任一份繁體中文的使用者文件、介面文案或技能原稿寫入「團隊模式」後執行詞彙守門
- **THEN** 守門失敗並指出該檔與該詞

### Requirement: 內嵌宿主的文件涵蓋宿主套件的對外介面

Node SDK 文件（中英兩版）SHALL 以宿主套件為主線，並 SHALL 涵蓋：`createHost` 與它的輸入；三個儲存函式（列出、讀取、提交）的輸入輸出與三項保證；用套件裡的合格測試檢查自己的實作；兩個掛鉤（執行前可拒絕、提交後只能旁觀且不保證送達）；`host.dispatch` 與 Agent 工具定義的匯出；`host.handleRequest`（宿主只掛一條路由，驗不出身分時傳空值）；登入方式的宣告；技能清單的設定；呼叫時附上自由欄位；匯出與匯入的方法與大小上限（含宿主框架的本文上限要不小於它、壓縮由路由層做）。文件 SHALL 說明 `createEngine` 只接受本機資料夾，SHALL NOT 含 Store 物件與它的方法表。文件 SHALL 內嵌 SDK 的組成圖與一個動詞的四個步驟圖。文件裡每一段範例程式用到的匯出名稱 SHALL 存在於套件的型別宣告。

#### Scenario: 整合者照文件接上自己的儲存

- **WHEN** 整合者照 Node SDK 文件實作三個儲存函式、執行合格測試、建立宿主套件並掛一條路由
- **THEN** 文件提供每一步的範例程式與預期結果，不需要翻正式規格

#### Scenario: 文件不含已移除的介面

- **WHEN** 在 Node SDK 文件的中英兩版搜尋 Store 物件的方法名稱（例如 listChanges、writeArtifact）與「31」個方法的說法
- **THEN** 沒有命中

#### Scenario: 範例用到的名稱存在

- **WHEN** 取出 Node SDK 文件範例程式裡從 `@speclink/engine` 取用的名稱，對照套件的型別宣告
- **THEN** 每個名稱都在型別宣告裡

### Requirement: 儲存接口的範例有專文說明

Server 儲存後端文件（中英兩版，檔名不變）SHALL 改以儲存接口的範例為主題，並 SHALL 說明：三個範例（SQLite、PostgreSQL、檔案系統）的位置與它們也是示範用 server 的儲存；每個範例的資料表或檔案配置；一次提交怎麼做到全部成功或全部不做、版本比對，以及事件與寫入同一次交易；事件記錄與它的序號；範例不做的事（連線池、TLS、配置版本之間的遷移、多個實例之間的事件傳遞）。文件 SHALL NOT 含契約版本、等級、能力清單與資料結構遷移的說明。

#### Scenario: 整合者找得到同類儲存的範例

- **WHEN** 用 PostgreSQL 的整合者開啟這份文件
- **THEN** 可見 PostgreSQL 範例的位置、資料表、一次提交的步驟與它的限制

#### Scenario: 管理者選擇示範用 server 的儲存

- **WHEN** 示範用 server 的管理者要在三種儲存之間選擇
- **THEN** 文件說明各自適合的情況與設定方式，並連到 Server 部署文件

### Requirement: 遠端專案的登入與技能來源有說明

Remote 入門教學（中英兩版）SHALL 說明：三種登入方式（瀏覽器登入、裝置登入、存取金鑰）與用戶端選擇的順序；Server 沒有宣告登入方式時的行為；存取金鑰存放在哪裡；連上遠端專案之後執行 `speclink update` 取得技能；遠端專案的技能由 Server 提供，版本跟著 Server；連到一台 Server 等於信任它提供的技能。設定說明（中英兩版）SHALL 說明 CLI 的技能版本提醒（什麼時候出現、同一台 Server 一天最多一次、不擋操作）與它的狀態檔位置。「動詞在遠端的差異」一節 SHALL 與 CLI 的實際行為一致。

#### Scenario: 讀者知道技能從哪裡來

- **WHEN** 讀者在 Remote 入門教學讀到連上遠端專案之後的步驟
- **THEN** 可見執行 `speclink update` 的指示、技能來自 Server 的說明，以及信任的說明

#### Scenario: 讀者知道會用哪一種登入方式

- **WHEN** 讀者要連到一台宣告了瀏覽器登入的 Server
- **THEN** 文件說明用戶端會先用瀏覽器登入，以及怎麼改用另外兩種

### Requirement: 動詞參考文件涵蓋遠端可用的動詞與新增的路由

動詞與旗標的參考文件（中英兩版）SHALL 列出本系列新增的動詞與旗標：`evidence show`、`change landed`、`change field set`、`change field unset`、`discuss field set`、`discuss field unset`，以及 `new change` 與 `discuss new` 的 `--field`。文件 SHALL 列出 `list --json` 新增的 `created` 與 `modifiedAt`，與 `show --json` 新增的審查、驗證、分支與落地欄位。端點一節 SHALL 列出新增的路由：溯源、嚴格重疊的 plan、技能、匯出、自由欄位的寫入，以及不用登入就能查的登入方式；並說明審查與驗證的事件歸到變更。文件列出的每個動詞與旗標 SHALL 存在於 CLI 的說明輸出。

#### Scenario: 讀者查得到新的動詞

- **WHEN** 讀者在動詞參考文件查 `change landed`
- **THEN** 可見它的用法、旗標與它只記錄不檢查的說明

#### Scenario: 文件與 CLI 一致

- **WHEN** 取出動詞參考文件列出的動詞與旗標，對照已建置的 CLI 的說明輸出
- **THEN** 每一個都存在

### Requirement: 已知缺口逐項列出

專案能力狀態（中英兩版）SHALL 有「已知缺口」一節，逐項列出這個系列刻意延後或不做的項目，每項 SHALL 說明影響誰與目前的替代做法。清單 SHALL 至少含：沒有本機規格檔的內嵌宿主不能用需要 checkout 的動詞；`task done` 不替宿主收集改了哪些檔；用 Node 內嵌的宿主沒有推送事件；沒有把封存的討論記錄搬回來的動詞；沒有寫入詞彙表的動詞；baseline 與 manual 兩支技能在遠端專案被拒絕；連到沒有宣告技能的舊版 Server 時用本機的範本；已封存的文件不能改自由欄位，也沒有依欄位查詢的動詞；CLI 沒有匯入匯出的動詞；沒有 JavaScript 寫的儲存接口範例；示範用 server 只支援單一實例；規格放在 checkout 之外的本機位置（`spec_dir` 不檢查絕對路徑與 `../`，行為未定義）；唯讀引用另一套規格；遠端專案的自訂產出流程；嚴格一致的讀取快照；多個宿主實例之間的推送事件。README 的「目前狀態」SHALL 用一行連到這一節，SHALL NOT 複製清單。路線圖 SHALL 為其中屬於之後方向的項目提供落點。

#### Scenario: 整合者在採用前看得到缺口

- **WHEN** 整合者開啟專案能力狀態的已知缺口一節
- **THEN** 可見每一項缺口、它影響誰，以及目前的替代做法

#### Scenario: 缺口與提案一致

- **WHEN** 把前二十個變更封存後的提案裡寫的「已知缺口」與討論結論的延後項目，逐項對照這一節
- **THEN** 每一項都在這一節，或在文件裡註明已經由後面的變更解決

### Requirement: 圖解以自帶樣式的 SVG 呈現

使用者文件 SHALL 內嵌五張圖解：整體形態（README 的「把 Speclink 內嵌進你的系統」）、SDK 的組成與一個動詞的四個步驟（Node SDK 文件）、你的系統與官方 server 共用宿主套件（Server 部署文件）、技能用路徑讀與用動詞讀（工作流程文件）。圖檔 SHALL 是 SVG，放在 `docs/assets/figures/zh-TW/` 與 `docs/assets/figures/en/`，兩組檔名 SHALL 逐一相同，圖裡的文字各用該語言。每個 SVG SHALL 自己帶樣式與不透明的背景，SHALL NOT 依賴頁面的樣式表、外部字型、外部圖片或 script。每張圖 SHALL 至少被一份同語言的使用者文件引用，引用處 SHALL 有說明圖意的替代文字，並在圖的前後以文字說明同一件事。圖的內容 SHALL 與實作一致。

#### Scenario: 圖在深色與淺色頁面都看得清楚

- **WHEN** 在 GitHub 的淺色與深色主題開啟含圖的文件
- **THEN** 每張圖都有自己的背景，文字與線條清楚可讀

#### Scenario: 兩組圖檔對等

- **WHEN** 比對兩個語言資料夾的圖檔
- **THEN** 檔名逐一相同；中英成對的文件引用相同的圖檔名序列，各自指向自己語言的資料夾

#### Scenario: 圖檔不帶外部資源

- **WHEN** 檢查任一張圖的原始內容
- **THEN** 沒有 script、沒有外部的樣式表、字型與圖片連結

### Requirement: 示範用 server 的升級步驟有說明

Server 部署文件（中英兩版）的升級一節 SHALL 說明從 0.12.0 以前的版本升級的步驟：以舊版執行備份、把設定指向空的儲存位置與空的 identity 資料庫、以新版執行還原並確認驗證報告；並 SHALL 說明新版遇到舊資料時會拒絕啟動且不改動它，以及回退的方式（換回舊版並指回原本的位置）。Server 備份與還原文件 SHALL 說明備份格式第 2 版、新版能檢查與還原第 1 版的備份，以及舊版讀不了第 2 版的備份。

#### Scenario: 管理者照文件完成升級

- **WHEN** 0.12.0 的示範用 server 的管理者照 Server 部署文件的升級一節操作
- **THEN** 文件給出三個步驟各自的指令與預期的輸出，以及失敗時怎麼回退

#### Scenario: 管理者直接換版本啟動

- **WHEN** 管理者沒有先備份就用新版啟動
- **THEN** 文件說明 server 會拒絕啟動、資料沒有被改動，以及接下來要做的事

## MODIFIED Requirements

### Requirement: 使用者文件採漸進揭露與單一責任
<!-- BEFORE: 雙路徑段用 Local Repo／Remote Store；README 的 H2 清單含「自架 server」；沒有規定內嵌與試用兩節的內容 -->

Speclink 使用者文件 SHALL 以 README、getting-started、workflow、product-status 與 roadmap 形成漸進揭露入口。README 第一個 H2 之前 SHALL 呈現品牌圖片、標語（繁中版「讓 AI 寫的每個變更，都對得上規格、查得到證據」，英文版「Every AI-built change, checked against its spec」）、徽章、語言切換、定位段、雙路徑段，以及一張 desktop 看板截圖。定位段 SHALL 說明 Speclink 是以 Rust 實作的 SDD 引擎，並以可驗收為主軸：每個變更先寫一小份規格、交給 AI agent 實作，封存前由審查與驗證兩道品質關卡檢查並留下證據，規格是 repo 裡的純文字；定位段 SHALL NOT 出現 PM 與 PO。雙路徑段 SHALL 說明本機專案與遠端專案兩條路徑（英文版 local project 與 remote project）：一個人用本機專案即足夠，團隊要共用規格時用遠端專案——規格由一台 Server 持有；PM／PO／RD／AI Agent 共用 change／artifact／task／verify／archive 語意，以及 PM、PO 在團隊這一層參與討論與提案的說明，SHALL 放在雙路徑段。README SHALL 以 H2 提供：為什麼用 Speclink、快速開始、畫面一覽、特色、最短流程心智模型（工作流程）、安裝、把 Speclink 內嵌進你的系統、試用官方 server、文件地圖、由 product-status 校正的目前狀態摘要、參與貢獻、致謝與授權；快速開始 SHALL 位於安裝之前，並以安裝 CLI、`speclink init` 與請 Agent 執行 propose 三步呈現本機專案的開始入口。「把 Speclink 內嵌進你的系統」SHALL 位於「試用官方 server」之前，SHALL 列出宿主要做的四件事——實作三個儲存函式並通過合格測試、掛一條路由把請求交給入口、驗不出身分時傳空值、要支援瀏覽器登入時讓自己的登入服務提供標準的說明文件並接受本機回呼——並 SHALL 內嵌整體形態的圖、連到 Node SDK 文件與儲存接口範例的文件。「試用官方 server」SHALL 以官方 server 的定位句開頭，並保留三種跑法。設計之初以 Spectra App 2.3.1 CLI 為行為參考的歷史起源 SHALL 寫在致謝段，同段 SHALL 致謝 OpenSpec 的目錄結構；該起源 SHALL NOT 寫在第一個 H2 之前。getting-started SHALL 只承載可直接完成的本機專案第一輪，workflow SHALL 作為完整使用流程正典，product-status SHALL 作為目前能力狀態正典，roadmap SHALL 作為尚未完成的方向的唯一落點。各文件 SHALL 以連結導向下一層細節，SHALL NOT 在 README 或 getting-started 複製完整架構與狀態矩陣。

#### Scenario: README 保留專案定位與起源

- **WHEN** 使用者開啟繁體中文或英文 README 判斷 Speclink 是什麼及為何存在
- **THEN** 第一個 H2 之前可見品牌圖片、可驗收標語、語言切換、以 Rust 實作的 SDD 引擎與可驗收定位、本機專案與遠端專案的說明（含 PM、PO 參與的團隊情境）及 desktop 看板截圖；致謝段可見 Spectra App 2.3.1 行為參考起源與 OpenSpec；目前狀態段清楚區分已可運作與規劃中內容，並連到專案路線圖
- **AND** 文件整理只校正過時事實、術語與連結，不得將第一個 H2 之前的內容改成僅含導覽連結的入口

#### Scenario: 定位段以可驗收開場、團隊角色放在雙路徑段

- **WHEN** 讀者只讀繁體中文或英文 README 的標語與第一段
- **THEN** 標語為「讓 AI 寫的每個變更，都對得上規格、查得到證據」（英文版「Every AI-built change, checked against its spec」），第一段依序可見「先寫一小份規格」「交給 AI agent 實作」「封存前由審查與驗證兩道品質關卡檢查並留下證據」
- **AND** 第一段不出現 PM 與 PO；PM、PO 與共用語意的說明出現在緊接其後的雙路徑段，且與遠端專案的團隊情境寫在一起

#### Scenario: 首次使用者由 README 到完成第一輪

- **WHEN** 首次使用者從繁體中文或英文 README 尋找安裝與本機專案的第一輪操作
- **THEN** README 的快速開始段以三步呈現安裝 CLI、`speclink init` 與請 Agent 執行 propose，並提供可見的 getting-started 入口；getting-started 以目前存在的 CLI／skill 完成 init、提案、實作檢查與封存，遇到選用分支時可連到 workflow

#### Scenario: 進階使用者查詢目前能力與目標

- **WHEN** 使用者要判斷遠端專案或 Server 某項能力目前是否可用及最終目標
- **THEN** README 導向 product-status 取得目前狀態、證據與限制，並由該列導向專案路線圖取得之後的方向，兩者 SHALL NOT 混成同一狀態描述

#### Scenario: 整合者從 README 找到內嵌的四件事

- **WHEN** 整合者在繁體中文或英文 README 尋找怎麼把 Speclink 接進自己的系統
- **THEN** 「把 Speclink 內嵌進你的系統」一節排在「試用官方 server」之前，可見宿主要做的四件事、整體形態的圖，以及通往 Node SDK 文件與儲存接口範例文件的連結

#### Scenario: 試用的讀者先看到定位

- **WHEN** 讀者開啟 README 的「試用官方 server」一節
- **THEN** 第一句是官方 server 的定位句，之後才是三種跑法

### Requirement: 產品狀態以證據分類目前與規劃能力
<!-- BEFORE: 能力清單的列是 Local CLI／skills、Local Desktop、Node SDK、Server／Store drivers、Desktop Remote Workspace、Agent／Protocol 整合與營運能力 -->

中英文 product-status SHALL 對 Local CLI／skills、Local Desktop、Node SDK 與宿主套件、儲存接口的範例、示範用 server、Speclink Desktop 的遠端專案、Agent／Protocol 整合與營運能力逐列標記 Available、Partial、Planned 或 Deprecated；繁體中文對應為可用、部分可用、規劃中、已棄用。每列 SHALL 包含使用者入口、至少兩項互相獨立的現行證據或一項可執行端到端證據、目前限制／下一步，以及最後查核日期。crate、canonical spec、已封存 change 或目標架構文字單獨存在 SHALL NOT 足以標記 Available。

#### Scenario: 已有底層實作但產品路徑未閉合

- **WHEN** 某能力已有 crate 與 canonical spec，但缺少完整使用者入口或端到端流程
- **THEN** product-status 將其標為 Partial 或 Planned，列出可用子集與缺少入口，SHALL NOT 僅因程式目錄存在寫成已完整交付

#### Scenario: 已棄用路徑仍存在

- **WHEN** legacy remote REST 路徑仍可在程式或歷史文件找到但已明確不是目標架構
- **THEN** product-status 將其標為 Deprecated，說明替代方向並連到目前目標架構，SHALL NOT 與新的 Remote Platform 混寫為同一可用能力

#### Scenario: 狀態查核可重做

- **WHEN** 維護者在之後的 checkout 重新更新 product-status
- **THEN** 文件提供查核日期與證據位置，使維護者可用當下 CLI help、skills、workspace members、tests／操作文件與正式規格重做判斷，而非沿用舊日期結論

### Requirement: 中英文文件保持結構與事實對等
<!-- BEFORE: 兩版 README 對等的概念寫 Local Repo／Remote Store；對等的項目不含圖解 -->

`README.zh-TW.md`／`README.md`（繁中／英文），以及 `docs/` 下每一組 `<名稱>.zh-TW.md`／`<名稱>.md`（getting-started、workflow、product-status、roadmap、remote-getting-started、development、configuration、verb-contract、sdk-node、server-deployment、server-store-drivers、server-backup）SHALL 分別保持相同的 H2 章節集合與順序、狀態矩陣列集合、命令語意、截圖檔名序列、圖解檔名序列及交叉連結。英文版 SHALL 為無語言後綴的檔名，繁中版 SHALL 帶 `.zh-TW` 後綴。每個 H2 SHALL 寫成 `## <a id="<錨點>"></a><標題>`，兩版的錨點序列 SHALL 逐項相同；標題文字 SHALL 各用該版的語言，SHALL NOT 在單一語言版本使用雙語標題。兩版 README SHALL 共用品牌圖片；截圖 SHALL 以相同檔名、各自引用該語言介面的那一組；圖解 SHALL 以相同檔名、各自引用該語言的那一組。兩版 README SHALL 保持標語、產品定位、共同流程語意、本機專案與遠端專案、Spectra App 2.3.1 行為參考起源及目前狀態摘要的概念對等。繁體中文散文 SHALL 使用 `speclink/LANGUAGE.md` 的正典詞彙；引擎動詞、CLI 命令、欄位名與程式識別符 SHALL 保留於 code span，不以避免詞取代使用者文案。

#### Scenario: 語言切換不遺失流程資訊

- **WHEN** 使用者在任一成對文件切換繁體中文與英文版本
- **THEN** 兩版呈現相同流程階段、決策分支、能力列與限制，只改變自然語言，不新增或遺漏事實

#### Scenario: 繁體中文採正典詞彙

- **WHEN** 繁體中文文件描述 discuss promote、promoted discussion 與 archive
- **THEN** 散文分別使用「轉為變更」「已轉出變更」「封存」，`promote`、`promoted`、`archive` 只在 CLI／欄位／code span 或必要的引擎動詞對照中出現

#### Scenario: 截圖引用兩版一致

- **WHEN** 並列比對任一組中英成對文件的截圖引用
- **THEN** 兩版引用相同的檔名序列與相同的引用數量，繁中版的每個引用都在 `docs/assets/screenshots/zh-TW/`、英文版的每個引用都在 `docs/assets/screenshots/en/`

#### Scenario: README 語言切換互指

- **WHEN** 使用者在 GitHub 開啟版本庫首頁
- **THEN** 顯示的 `README.md` 為英文版，頂部語言切換連到 `README.zh-TW.md`；`README.zh-TW.md` 的語言切換連回 `README.md`；README 與 docs 沒有任何連結指向根目錄的 `README.en.md`（品牌資產與外掛目錄各自的 `README.en.md` 不在此限）

#### Scenario: 圖解引用兩版一致

- **WHEN** 並列比對任一組中英成對文件的圖解引用
- **THEN** 兩版引用相同的檔名序列與相同的引用數量，繁中版的每個引用都在 `docs/assets/figures/zh-TW/`、英文版的每個引用都在 `docs/assets/figures/en/`

### Requirement: 品質關卡蓋章效果與非成員錯誤碼的文件揭露
<!-- BEFORE: 動詞參考文件的本質本機動詞列 SHALL 含 demo 與 trace 兩者 -->

中英文 workflow 文件 SHALL 說明品質關卡蓋章的工單消耗效果：蓋章於同一原子寫入內寫入章欄位並刪除工單（review.md 與 verify.md）、封存的已蓋章 change 不含工單檔、僅未結工單經 carry 旗標隨封存移動、fs 模式的工單文字僅存於 git 歷史、remote 模式蓋章後工單文字不可回讀。中英文 remote-getting-started SHALL 以 403（permission_denied）描述已登入非成員讀取專案資源的結果，SHALL NOT 寫成 404。中英文 verb-contract 參考文件的本質本機動詞列 SHALL 與 CLI 的實際宣告一致：SHALL 含 demo，SHALL NOT 含 trace、change rank 與 plan --strict-overlap（三者在遠端專案可用）。

#### Scenario: 讀者可自 workflow 查得蓋章後工單去向

- **WHEN** 讀者於 workflow 文件的品質關卡段落查詢蓋章效果
- **THEN** 文件說明蓋章刪除工單、封存不含工單檔、remote 模式工單文字不可回讀，讀者不需翻正式規格即可得知 show 於蓋章後回報無工單為預期行為

#### Scenario: 非成員錯誤碼敘述與正式規格一致

- **WHEN** 讀者依 remote-getting-started 的 membership 段落驗證非成員讀取行為
- **THEN** 文件敘述的狀態碼為 403，與 server-identity 正式規格及實際 server 行為一致

#### Scenario: 本質本機動詞列表完整

- **WHEN** 讀者於 verb-contract 參考文件查閱動詞模式分岔表
- **THEN** FsOnly 列含 demo、不含 trace，且與 CLI dispatch 的實際宣告一致

### Requirement: 使用者文件載明本地產物的純文字可攜性
<!-- BEFORE: 不適用遠端的理由寫「遠端的正式規格在 Store，本機只有唯讀投影」 -->

使用者文件 SHALL 說明 **Local 模式**的產物位置——規格資料夾（預設 `speclink/`）下的 `specs/<capability>/spec.md`、`changes/<名稱>/`、`changes/archive/` 與 `config.yaml`——並說明內容為純 Markdown 與 YAML、可不經 Speclink 讀寫、每次變動皆呈現於 Git diff；說明 SHALL 明示此可攜性不適用遠端模式（遠端專案的正式規格由 Server 持有，checkout 裡沒有規格檔）。說明 SHALL 一併載明 Speclink 在這些產物上的兩項擴充：`discussions/` 與各變更目錄的 `.meta.yaml`。此說明 SHALL 出現於 README 中英兩版與入門教學中英兩版。這些說明 SHALL NOT 宣稱與 OpenSpec 相容或沿用 OpenSpec 的目錄結構；README 特色段與入門教學 SHALL 寫明可從 OpenSpec 專案匯入、且相容為單向：在 OpenSpec 專案執行 `speclink init` 會把 `openspec/` 搬到 `speclink/`，之後 OpenSpec CLI 讀不到這個專案。README 致謝段對 OpenSpec 目錄結構的致謝不受此限。

#### Scenario: 讀者判斷得出資料可攜

- **WHEN** 讀者在 README 或入門教學（中英任一版）讀到本地產物的說明
- **THEN** 可見產物位於 `speclink/` 下的目錄位置、純文字格式、不裝 Speclink 也能讀寫、Git diff 可見的事實，以及 Speclink 額外新增的兩樣東西（`discussions/` 與 `.meta.yaml`），並可見此說明只適用 Local 模式

#### Scenario: 本地產物說明寫明單向匯入

- **WHEN** 讀者在 README 特色段或入門教學中，尋找 speclink 與 OpenSpec 的關係
- **THEN** 說明寫出在 OpenSpec 專案執行 `speclink init` 即可匯入、資料夾會搬到 `speclink/`、之後 OpenSpec CLI 讀不到這個專案；沒有「相容 OpenSpec」或「沿用 OpenSpec 的目錄結構」的宣稱（英文版同樣沒有 OpenSpec-compatible 或 OpenSpec folder layout 的宣稱）

### Requirement: 使用者面路線圖與內部交付順序分列
<!-- BEFORE: 第二條方向是「以引擎自建客戶端與 server 端（使用者以 SDK 引擎自行開發桌面、其他前端或自家 server）」 -->

版本庫 SHALL 提供面向使用者的路線圖文件 `docs/roadmap.zh-TW.md` 與 `docs/roadmap.md`，涵蓋 SDK 發布、以引擎自建用戶端與內嵌宿主（使用者以引擎自行開發桌面、其他前端，或把引擎內嵌進自己的系統）、遠端協作完整化、agent 工具整合與系統整合五條方向，每條載明要解決的問題、目前進度與可觀察的下一步。該文件 SHALL NOT 出現版本號或日期承諾。尚未完成的方向 SHALL 集中寫在路線圖；product-status 的規劃中列 SHALL 只連到路線圖，SHALL NOT 另行敘述規劃內容。

#### Scenario: 對外方向不含時程承諾

- **WHEN** 讀者開啟任一語言的使用者面路線圖
- **THEN** 五條方向各自可見問題、進度與下一步，且全文不含版本號或日期形式的交付承諾

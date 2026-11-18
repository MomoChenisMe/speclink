## Context

這是系列的最後一個變更。前二十個變更都沒有動使用者文件，各自在提案的 Impact 留了「文件影響」與「已知缺口」。

文件的現況：

- README 中英兩版各 274 行，H2 的順序與錨點相同。「自架 server」一節的錨點是 `self-hosting`。
- docs 有十二組中英成對的文件。英文版沒有語言後綴，繁中版帶 `.zh-TW`。
- 截圖 17 張，兩個語言各一套，放在 docs/assets/screenshots 底下的兩個語言資料夾。docs 底下還沒有任何內嵌在文件裡的 SVG。
- `speclink/LANGUAGE.md` 沒有本機專案、遠端專案、宿主、內嵌、SDK、宿主套件、儲存接口、入口的詞條；「儲存後端」的定義寫的是 TeamStore。
- scripts/docs 有六組守門：連結能不能解析、中英兩版的 H2 錨點與截圖序列是否相同、專案能力狀態的表格、詞彙守門（從 LANGUAGE.md 的 avoid 取中文的詞，掃 README、docs、三份介面文案與技能原稿）、截圖清單、Remote 入門的必要字串、入門與工作流程文件裡的指令是否存在於 CLI。守門紅的時候，CI 會跳過其他四組測試。
- 正式規格 user-documentation 把文件的結構與幾句固定的說法寫成需求；守門依它檢查。

限制：

- 文件要描述實作完成之後的狀態，而前四個變更在這份設計寫下時還沒有實作。所以這份設計給結構、固定的句子與判斷的規則；逐句的內容在實作時對照程式與封存後的提案。
- 討論的結論規定：延後的項目必須在文件裡明確寫成缺口；圖用 SVG、自己帶樣式與背景色、中英各一套、放 docs/assets 底下；圖的內容以當時的實作為準。

## Goals / Non-Goals

**Goals:**

- 讀 README 的人在第一個畫面知道有本機專案與遠端專案兩條路；要整合的人找得到「內嵌」一節與四件事。
- Node SDK 文件足夠讓整合者不看正式規格就接上自己的儲存。
- 文件裡沒有已經不對的句子：只限本機的動詞、已刪除的儲存層、已移除的介面。
- 延後的項目逐項列在一個地方。
- 文件的守門在這個變更之後涵蓋新的結構（圖、兩節、禁用的說法）。

**Non-Goals:**

- 不新增截圖、不新增成對的文件。
- 不改程式的行為。
- 不全面替換「遠端模式」「本地」「Local Repo」。
- 不寫更新日誌。

範圍內：README、docs、docs/assets 的圖與一組截圖、`speclink/LANGUAGE.md`、scripts/docs 的守門、正式規格 user-documentation（經 delta）。詞彙守門照出存量時，連帶修改介面文案與技能原稿的用詞。範圍外：其餘全部。

## Decisions

### D1: 先盤點，再動筆

第一個任務只盤點，不改文件：

- 讀前二十個變更**封存後**的提案，取出每份的「文件影響」「已知缺口」「相容性影響」。封存後的版本以實作期間的修正為準，和這份設計寫下時的版本有出入時，以封存後的為準。
- 讀討論結論的 Deferred 清單。
- 對每一條「文件影響」決定落在 D3 到 D7 的哪一份文件、哪一節；對每一條「已知缺口」決定它仍然成立，或已經由後面的變更解決（例如「連到示範用 server 時沒有最後修改時間」已由 server-on-port-examples 解決）。
- 把結果寫成一張表，放進這份 design 的「盤點結果」一節。之後的任務照這張表做，最後一個任務用它核對沒有遺漏。

### D2: 用詞與 LANGUAGE.md

LANGUAGE.md 沒有寫入的動詞，直接編輯檔案。新增九個詞條，格式與既有詞條相同（definition、avoid、why）：

| 詞條 | definition 的要點 | avoid |
| --- | --- | --- |
| 本機專案 | 規格放在 repo 的規格資料夾、用 Git 協作的專案 | （不列中文的 avoid） |
| 遠端專案 | 規格由一台 Server 持有的專案；checkout 裡沒有規格檔 | 團隊模式 |
| 宿主 | 把引擎內嵌進自己後端程序的系統；它自己負責帳號、登入、權限與儲存 | （無） |
| 內嵌 | 在宿主的程序裡執行引擎，相對於連到另一台 server | （無） |
| SDK | npm 套件 `@speclink/engine` 這個工具箱；引擎是裡面的一件 | （無；說明不用它統稱遠端功能） |
| 宿主套件 | speclink 提供給宿主的那一層：每次呼叫帶身分與範圍、兩個掛鉤、HTTP 入口 | 主機套件 |
| 儲存接口 | 宿主實作的三個函式：列出、讀取、提交 | （無） |
| 入口 | 宿主套件接收請求的地方：給 CLI 與 Speclink Desktop 的 HTTP 入口，與給宿主 Agent 工具的呼叫入口 | （無） |
| 合格測試 | 檢查一個儲存接口的實作是否符合契約的那一組檢查 | （無） |

「Remote Store」「Self-hosting」是純英文，詞彙守門不收純英文的詞（既有設計），所以它們不放進 avoid 的機械守門；改由 D9 在文件的中英對照檢查裡直接禁止。「自架 server」是中英混寫，同樣放在 D9。

「儲存後端」詞條：definition 改成「示範用 server 用來存規格文件的儲存——三個儲存接口範例之一（sqlite、postgres、serverfs）與它是否可用，統一以『儲存後端』為主詞，呈現於系統頁」；avoid 與 why 不變，why 補一句這次修改的原因與日期。

新增 avoid 詞（團隊模式、主機套件）之後立刻跑詞彙守門，把掃描面照出的存量在同一個任務清掉。技能原稿（`speclink-core` 的 assets）被改到時，技能的資產版號、技能的快照與資產的鎖定檔三樣要一起更新，並重新產生 repo 裡的技能檔；沒有改到就不動它們。

### D3: README 的兩節

- 雙路徑段（第一個 H2 之前）：兩個項目改成「本機專案」與「遠端專案」。遠端專案那一項說明規格由一台 Server 持有，Server 可以是內嵌了引擎的宿主，也可以是官方的示範用 server；PM、PO 與共用語意的說明留在這一項。
- 畫面一覽裡 server 後台截圖的說明文字：「自架 server 的後台」改成「示範用 server 的後台」。
- `## <a id="self-hosting"></a>自架 server` 換成兩節：
  1. `## <a id="embedding"></a>把 Speclink 內嵌進你的系統`（英文版 Embed Speclink in your system）。內容依序：一段說明什麼時候選這條路；整體形態的圖；宿主要做的四件事（編號清單，每件一到兩句，並連到 Node SDK 文件對應的章節）；一句說明合格測試與三個範例在哪裡；一句說明 speclink 不做的事（帳號、登入、權限、資料庫都是宿主的）。
  2. `## <a id="official-server"></a>試用官方 server`（英文版 Try the official server）。第一句是定位句；接著既有的三種跑法的表與第一次啟動的說明；最後連到 Server 部署與 Remote 入門。原本「遠端模式不綁這一份 server…」那一段刪除，它的內容由上一節取代。
- 「文件」一節：團隊與整合的分組改成「遠端專案」與「把 Speclink 內嵌進你的系統」兩組；「Store 介面」的說法移除。
- 「目前狀態」：三個項目照盤點結果改寫，不再提「三種 TeamStore」；加一行連到專案能力狀態的已知缺口。

定位句兩個語言的文字固定在規格「官方 server 定位為示範與試用」，逐字使用。

### D4: Node SDK 文件

兩版用相同的 H2 錨點序列。章節：

1. 這個套件有什麼（SDK 的組成圖；SDK 是工具箱的名字）
2. 安裝（保留既有內容與可安裝的起始版本）
3. 在本機專案上執行動詞（`createEngine` 的本機資料夾形式、`dispatch`、錯誤碼）
4. 把引擎內嵌進你的後端（`createHost` 的輸入：儲存、掛鉤、列出範圍的函式、登入方式、技能清單、上限）
5. 實作三個儲存函式（輸入輸出、三項保證、一個用 Map 寫的最小範例、連到三個 Rust 範例的文件）
6. 用合格測試檢查你的實作
7. 一個動詞的四個步驟（圖；執行前掛鉤、提交後掛鉤；需要至少送達一次時在提交的同一次交易裡寫自己的事件表）
8. 給 Agent 工具用（`host.dispatch`、身分與範圍由宿主的程式給、`agentTool`、自由欄位）
9. 服務 CLI 與 Speclink Desktop（`host.handleRequest`、一條路由、驗不出身分時傳空值、登入方式的宣告、瀏覽器登入對宿主登入服務的三個要求）
10. 匯出與匯入（方法、大小上限、路由層的本文上限、壓縮）
11. 產生技能檔（保留既有內容，補上「規格在不在 checkout」的選項）
12. 完整範例
13. 目前的限制（連到專案能力狀態的已知缺口，不複製清單）

範例程式用到的每個名稱在寫的時候對照套件的型別宣告。Store 物件、31 個方法的表、`claim` 選配一節全部移除。

### D5: 其餘文件逐份的改動

| 文件 | 改動 |
| --- | --- |
| Server 儲存後端（檔名不變） | 標題改成儲存接口的範例。章節：三個範例與位置；怎麼選（給示範用 server 的管理者）；各自的資料表或檔案配置；一次提交的步驟（用 PostgreSQL 的 SQL 當例子）；事件記錄；範例不做的事；換一種儲存要走備份與還原 |
| Server 部署 | 第一個 H2 之前加定位句；「官方 server 是參考實作」一節改寫；放共用宿主套件的圖；升級一節加從 0.12.0 以前升級的三個步驟、拒絕啟動時的訊息與回退 |
| Server 備份與還原 | 第一個 H2 之前加定位句；備份格式第 2 版；新版能檢查與還原第 1 版；範圍匯出下載的內容 |
| Remote 入門 | 定位句；CLI 一節改寫成三種登入方式與順序、存取金鑰存放的位置；新增「取得技能」一段（`speclink update`、技能來自 Server、信任的說明）；「動詞在遠端的差異」照 CLI 的實際宣告改寫 |
| 動詞與旗標的參考文件 | 模式分岔表的只限本機一列；新動詞與旗標；`--json` 的新欄位；端點一節的新路由與事件分類；匯入包的兩個版本 |
| 工作流程 | `trace` 與 plan 一節移除「只限本機」的標示；commit 與 worktree 合併之後回報落地；遠端專案的 commit 只含程式碼的檔案；放技能讀法的圖 |
| 設定說明 | 遠端一節的用詞；CLI 的技能版本提醒與狀態檔；自由欄位的鍵與值的規則；`spec_dir` 不檢查絕對路徑與 `../` |
| 入門教學 | 只校正用詞與連到 README 新錨點的連結 |
| 專案路線圖 | 「以引擎自建客戶端與 server」一節改成已經可用的部分與之後的方向；延後的項目各放進對應的方向 |
| 開發指南 | repo 結構表裡 crates/store 與 crates/host 的說明 |

每一份的中英兩版在同一個任務裡一起改，H2 錨點序列保持相同。

### D6: 專案能力狀態與已知缺口

- 能力清單：Node SDK 一列改成「Node SDK 與宿主套件」；三列 TeamStore 併成「儲存接口的範例」一列（三個範例各一個證據連結）；server 的列改稱示範用 server；Speclink Desktop 的遠端一列改稱遠端專案。每列的證據連結指向存在的測試，查核日期改成實作當天，兩個語言相同。
- 本地與遠端對照：照 CLI 的實際宣告重做；表頭說明遠端一欄以示範用 server 為量測對象，內嵌的宿主行為相同、差異見已知缺口。
- 新增一節 `## <a id="known-gaps"></a>已知缺口`（英文版 Known gaps），放在能力清單之後。每一項三欄：缺口、影響誰、目前的替代做法。項目以 D1 的盤點結果為準，至少含規格「已知缺口逐項列出」列的十六項。
- 既有的「已知文件缺口」一節保留它原本的用途（指還沒寫的文件），和新的一節分開。

### D7: 五張圖

檔名與內容（文字以繁中版為例；英文版同樣的版面）：

1. `embedded-shape.svg` 整體形態。左側兩個來源：「工程師的 CLI 與 Speclink Desktop」經 HTTP 到「宿主的一條路由」，再到「入口」；「宿主的 Agent 工具」直接到「入口」。中間是「宿主套件」與「引擎」。右側引擎經「列出、讀取、提交」到「宿主實作的三個函式」，再到「宿主的資料庫」。用兩種底色分出「speclink 提供的」（宿主套件、引擎）與「宿主自己的」（路由、驗身分、三個函式、資料庫、通知），附圖例。
2. `sdk-contents.svg` SDK 的組成。一個外框標 `@speclink/engine`（SDK），裡面四格：引擎、宿主套件、儲存接口的契約、合格測試。
3. `verb-four-steps.svg` 一個動詞的四個步驟。由左到右四格並編號：執行前掛鉤（可以拒絕並給原因）、列出與讀取、一次提交（全部寫入與事件）、提交後掛鉤（只能旁觀）。下方標出哪幾步呼叫宿主的函式。
4. `shared-host-kit.svg` 你的系統與官方 server 共用宿主套件。兩欄三層：最上層各自不同（你的路由與登入；官方的路由、帳號與管理頁），中間一層橫跨兩欄（宿主套件與引擎，相同），最下層各自不同（你的三個函式與資料庫；三個儲存接口的範例）。
5. `skill-path-vs-verb.svg` 技能用路徑讀與用動詞讀。一個技能步驟要讀正式規格：上面一條路用路徑讀，在遠端專案的 checkout 找不到檔案；下面一條路用動詞讀，CLI 經 HTTP 向 Server 取得內容。

SVG 的規格：

- 用 `viewBox`，自然寬度 760；引用處不指定固定的像素高度。
- 第一個元素是鋪滿的白色矩形。樣式寫在 SVG 裡（屬性或 SVG 內的 style 元素），不引用外部的樣式表、字型、圖片，不含 script 與 foreignObject。
- 字型用系統字型的清單（含繁體中文的常見字型）並以 sans-serif 結尾；最小字級 12。圖裡的檔案路徑與指令用系統的等寬字型清單，以 monospace 結尾。
- 顏色用固定的一組，文字與底色的對比在白底上清楚；箭頭用 marker。
- 每個 SVG 有 `role="img"` 與 `aria-label`（一句說明圖意）。

位置：docs/assets/figures 底下的 zh-TW 與 en 兩個資料夾，檔名相同。README 用 repo 根目錄起算的路徑引用，docs 用相對於 docs 的路徑引用，兩者都用 Markdown 的圖片語法並寫替代文字。圖的前後有一段文字說同一件事，讀者不看圖也讀得懂。

畫之前對照實作：每個方塊與箭頭對得上程式裡的一個東西（例如四個步驟對得上宿主套件執行一個命令的順序）。畫完用無頭瀏覽器把每張圖轉成 PNG 檢查文字沒有超出方塊、沒有重疊。PNG 只用來檢查，不進版控。

### D8: 截圖

只重拍 `server-setup.png`（兩個語言）。用既有的截圖腳本與它的程序；重拍後照既有規則移除 PNG 的文字區塊與拍攝資訊，並確認畫面裡沒有真實的帳號或金鑰。截圖的清單與檔名不變。

### D9: 守門

- `scripts/docs/docs-parity.test.mjs`：比對圖解的檔名序列（規則與截圖相同，資料夾是 figures）；十二組文件加 README 的兩個語言都不得出現 `Remote Store`、`Self-hosting`、`自架 server`；README 兩版都有 `embedding` 與 `official-server` 兩個錨點且順序是前者在前，沒有 `self-hosting`。
- 新增 `scripts/docs/docs-figures.test.mjs`：兩個語言資料夾的 SVG 檔名相同；每個 SVG 不含 script、foreignObject、外部的 href 或 url；第一個繪圖元素是鋪滿的矩形；有 `aria-label`；每張圖至少被一份同語言的 README 或 docs 引用。
- `scripts/docs/remote-docs.test.mjs`：Remote 入門、Server 部署與 Server 備份與還原的兩個語言含各自語言的定位句；既有的必要字串照舊。
- `scripts/docs/docs-links.test.mjs`：連結檢查本來就涵蓋圖片；只在它寫死文件數下限或清單時才需要調整。品質關卡第 1 輪之後追加：「文件的語言看檔名」與「分語言放的圖有沒有被同語言的文件引用」收成 `scripts/docs/docs-links.mjs` 的共用函式，截圖的檢查（這個檔）、圖解的檢查與定位句的檢查都用它，不各寫一份。
- `scripts/docs/docs-surface.test.mjs`（品質關卡第 1 輪之後追加）：「文件裡的指令存在於 CLI」的檢查多掃動詞與旗標的參考文件的兩個語言，讓規格情境「文件與 CLI 一致」有常駐的守門。
- 守門先改（測試先失敗），文件後改。

### D10: 發版說明的素材

整個系列完成後才發版。把前二十個變更的「相容性影響」裡使用者要採取行動的項目整理成一份清單，寫進這份 design 的「發版說明的素材」一節，供發版時使用。至少含：Node SDK 的 `createEngine` 不再接受 Store 物件與遷移方式；示範用 server 從 0.12.0 以前升級要走備份與還原；備份格式第 2 版；管理頁移除資料結構遷移；README 的錨點變動。這一節只是素材，不是更新日誌。

## Implementation Contract

**行為**（讀者看得到的結果）

- README 兩版：雙路徑段用本機專案與遠端專案；有「把 Speclink 內嵌進你的系統」與「試用官方 server」兩節，前者在前；後者第一句是定位句。
- Node SDK 文件以宿主套件為主線，沒有 Store 物件的內容。
- 專案能力狀態有「已知缺口」一節。
- 五張圖在兩個語言各一套，被對應的文件引用。
- 文件裡沒有 Remote Store、Self-hosting、自架 server、團隊模式、主機套件。

**介面**

- README 的錨點：`embedding`、`official-server`（取代 `self-hosting`）。專案能力狀態的新錨點：`known-gaps`。
- 圖檔：docs/assets/figures 底下兩個語言資料夾各五個 SVG，檔名如 D7。
- LANGUAGE.md：九個新詞條與修改後的「儲存後端」。

**失敗的樣子**

- 任何一份文件的中英兩版 H2 錨點不同、圖或截圖的序列不同、連結失效、出現禁用的說法、文件裡的指令不存在於 CLI：對應的守門失敗並指出檔案。

**驗收**

- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全部通過。
- 先 `cargo build -p speclink-cli`，讓「文件裡的指令存在於 CLI」的檢查不被略過。
- Node SDK 文件的範例：先 `npm --prefix crates/adapters/speclink-node run build`，再把「完整範例」存成暫存檔執行一次，確認不丟錯。
- D1 的盤點表每一列都標了落點且已完成。
- 正式規格 user-documentation 這次新增與修改的每個情境，都能在文件裡指出對應的段落。
- 動到技能原稿時：`cargo test -p speclink-core` 的技能快照與原稿守門通過，repo 裡的技能檔已重新產生。沒有動到時，`git status` 顯示技能相關的檔案沒有變動。

**範圍邊界**

- 範圍內：D1 到 D10。
- 範圍外：程式行為、新截圖、新的成對文件、更新日誌、手冊。

## Risks / Trade-offs

- [文件寫的和實作不一樣] → D1 以封存後的提案為準；D4 的範例實際執行一次；動詞與旗標由既有的守門對照 CLI；圖在畫之前對照程式。
- [回歸對照] → 這個變更不改程式，引擎的 golden 與 CLI 的整合測試不受影響。唯一的例外是技能原稿被詞彙守門要求改用詞時：技能的快照是刻意在同一批更新，並照 D2 一起更新版號與鎖定檔。
- [跨平台] → 守門的腳本在三個平台的 CI 都跑。新的守門讀檔用 Node 的路徑函式組路徑、以 UTF-8 讀取，比對檔名時不依賴作業系統的排序。SVG 是純文字，沒有平台差異。
- [新增 avoid 詞照出大量存量] → 只新增兩個中文的 avoid 詞（團隊模式、主機套件），盤點時先搜尋存量；「遠端模式」刻意不列。
- [守門紅的時候 CI 跳過其他測試] → 每個任務的驗證都跑全部的 scripts 測試；最後一個任務再跑一次。
- [GitHub 對 SVG 的限制] → 不用 script、foreignObject 與外部資源；樣式寫在 SVG 裡。由新的守門檢查，並在 GitHub 上實際看一次（手動任務）。
- [README 的舊錨點失效] → 外部連到 `#self-hosting` 的連結會落在頁首而不是 404。接受。
- [一個變更改 26 份文件，審查範圍大] → 任務依文件分組，每組中英一起改；審查時可以一組一組看。

## Migration Plan

沒有資料要遷移。文件隨整個系列一起發版。回退方式是還原這個變更的 commit。

## Open Questions

（無）

## 盤點結果

任務 1.1 的結果（2026-11-18）。來源是前二十個變更封存後的提案（`speclink/changes/archive/`）與討論 embedded-host-needs 結論的 Deferred 清單。之後的任務照這幾張表做；任務 5.2 用它核對。

落點的代號：README＝D3；SDK＝Node SDK 文件（D4）；範例＝Server 儲存後端文件（D5）；部署、備份、Remote、動詞、流程、設定、路線圖、開發＝D5 對應的文件；狀態＝專案能力狀態（D6）；缺口＝專案能力狀態的 `known-gaps` 一節；素材＝這份 design 的「發版說明的素材」（D10）。

### 文件影響

| # | 變更 | 文件影響 | 落點 |
| --- | --- | --- | --- |
| 1 | store-port-contract | Node SDK 文件新增儲存接口與合格測試；Server 儲存後端改寫成範例；專案能力狀態列出已知缺口 | SDK 第 5、6 節；範例整份；缺口 |
| 2 | engine-on-store-port | 專案能力狀態的 Server 效能說明；Server 儲存後端「每個動詞載入整個範圍」的段落 | 狀態「示範用 server」一列的限制；範例「一次提交的步驟」（改成用到才讀） |
| 3 | shared-verb-parsing | Node SDK 文件補上「不認得的旗標會被拒絕」 | SDK 第 3 節 |
| 4 | shared-verb-output | Node SDK 文件「只支援 9 個動詞」的幾句（已在該變更改寫）；其餘留給這個變更 | SDK 第 3、8 節（全部有引擎命令的動詞、需要 checkout 的動詞被拒絕） |
| 5 | workflow-config-commands | Node SDK 文件補上流程設定的動詞與選填的儲存方法 | SDK 第 8 節（流程設定的動詞可用）；「選填的儲存方法」已隨 Store 物件移除，不寫 |
| 6 | host-kit-agent-door | Node SDK 文件新增宿主套件、兩個掛鉤、JS 版儲存接口、合格測試、Agent 工具定義；README 的內嵌一節引用 `createHost`；專案能力狀態列出這個門的缺口 | SDK 第 4 到 8 節；README `embedding`；缺口 |
| 7 | server-routes-on-store-port | 專案能力狀態記一句「遠端封存不留本機備份」 | 缺口 |
| 8 | host-kit-client-door | 內嵌一節的第二、三件事與範圍函式、能力開關、登入方式宣告；Node SDK 文件的 `host.handleRequest` 與 `createHost` 新輸入；專案能力狀態的推送事件與匯入；Server 部署的定位句 | README `embedding`；SDK 第 4、9 節；缺口；部署開頭 |
| 9 | client-browser-login | Remote 入門與動詞參考文件的 `auth login` 新旗標、三種登入方式與順序、存取金鑰存放位置；內嵌一節對登入服務的三個要求；Speclink Desktop 的登入方式卡（說明與截圖） | Remote「CLI」一節；動詞；README `embedding` 第四件事與 SDK 第 9 節；登入方式卡只用文字（Non-Goals：不新增截圖） |
| 10 | skills-verbs-first-wave | 動詞參考文件：`change rank` 在遠端專案可用；專案能力狀態列兩個缺口 | 動詞的模式分岔表；流程；缺口 |
| 11 | remote-read-verbs | 動詞參考文件：`evidence show`、trace 與 `plan --strict-overlap` 移出只限本機、`list --json` 新欄位、兩條新路由；正式規格的那一句 | 動詞；流程；delta 已改 |
| 12 | skills-verbs-second-wave | Node SDK 文件的 `skills.render` 的 `specInCheckout`；工作流程：遠端專案的 commit 只含程式碼；專案能力狀態：baseline 與 manual 的缺口 | SDK 第 11 節；流程；缺口 |
| 13 | remote-skill-delivery | Remote 入門：`speclink update`、技能來自 Server、信任；內嵌一節與 Node SDK 文件的 `skills`；設定說明：CLI 的提醒與狀態檔；Speclink Desktop 的技能檔提示 | Remote「取得技能」；SDK 第 4 節；設定；Desktop 的提示只用文字（Remote 入門的 Speclink Desktop 一節） |
| 14 | station-status-and-landing | 動詞參考文件：`change landed`、新欄位與路由、事件的分類；內嵌一節：用執行前掛鉤加「沒有落地就不能封存」；工作流程：回報落地 | 動詞；SDK 第 7 節（掛鉤的例子）；流程 |
| 15 | host-free-fields | 動詞參考文件：四個動詞、`--field`、新欄位與路由；內嵌一節與 Node SDK 文件：呼叫入口時附上欄位、用掛鉤檢查；鍵與值的規則 | 動詞；SDK 第 8 節；設定 |
| 16 | import-export-on-port | 內嵌一節與 Node SDK 文件：匯出與匯入、大小上限、本文上限、壓縮；動詞參考文件：`GET /export`、第二版的包；專案能力狀態：CLI 沒有動詞 | SDK 第 10 節；動詞；缺口 |
| 17 | node-retire-legacy-store | Node SDK 文件移除 Store 物件的全部內容、「完整範例」改用 `createHost`；README「Store 介面」的說法；路線圖提到 Store bridge 的兩處；發版說明 | SDK 整份；README 文件一節；路線圖；素材 |
| 18 | store-port-examples | Server 儲存後端改寫；內嵌一節的第一件事連到三個範例；開發指南的 repo 結構表；專案能力狀態：沒有 JavaScript 範例 | 範例；README `embedding`；開發；缺口 |
| 19 | server-on-port-examples | Server 部署的升級三步驟與回退；Server 備份的格式第 2 版；Server 儲存後端移除契約版本等說明；專案能力狀態移除「沒有最後修改時間」並改三列儲存；後台截圖；發版說明 | 部署；備份；範例；狀態；截圖只重拍 `server-setup.png`（D8；清單裡沒有系統頁的截圖，`server-overview.png` 拍的是總覽）；素材 |
| 20 | retire-teamstore-and-drivers | 專案能力狀態的三列 TeamStore 與「Command Runtime、Host 與 Protocol」一列；開發指南的 crates/store；README「三種 TeamStore」；LANGUAGE.md 的「儲存後端」；手冊重新產生 | 狀態；開發；README 目前狀態；LANGUAGE.md（D2）；手冊在 Non-Goals，不做 |

沒有文件影響的變更：沒有。二十個變更各自都列了至少一項（server-routes-on-store-port 只有一句）。

### 已知缺口

「缺口」一欄的編號對應規格「已知缺口逐項列出」列的十六項（G1 到 G16）；G17 以後是盤點多出來的。

| # | 來源 | 缺口 | 狀態 | 落點 |
| --- | --- | --- | --- | --- |
| G1 | host-kit-agent-door、shared-verb-output | 沒有本機規格檔的內嵌宿主不能用需要 checkout 的動詞（drift、review prepare、review scope、verify scope、帶 `--skill` 的 instructions；實際清單在 3.7 對照程式） | 仍然成立 | 缺口 |
| G2 | host-kit-agent-door、station-status-and-landing | `task done` 不替宿主收集改了哪些檔；蓋章與開工標記也不替宿主讀 commit 與分支（宿主套件不讀 git） | 仍然成立 | 缺口 |
| G3 | host-kit-client-door | 用 Node 內嵌的宿主沒有推送事件，用戶端只靠輪詢 | 仍然成立 | 缺口 |
| G4 | skills-verbs-first-wave、skills-verbs-second-wave | 沒有把封存的討論記錄搬回來的動詞 | 仍然成立 | 缺口 |
| G5 | skills-verbs-first-wave、skills-verbs-second-wave | 沒有寫入詞彙表的動詞 | 仍然成立 | 缺口 |
| G6 | skills-verbs-second-wave；Deferred「遠端專案的手冊產生」 | baseline 與 manual 兩支技能在遠端專案被拒絕（manual 的導覽模式仍可用） | 仍然成立 | 缺口；路線圖 |
| G7 | remote-skill-delivery | 連到沒有宣告技能的舊版 Server 時用本機的範本，worktree 的兩支不會產生 | 仍然成立 | 缺口 |
| G8 | host-free-fields | 已封存的文件不能改自由欄位，也沒有依欄位查詢的動詞 | 仍然成立 | 缺口 |
| G9 | import-export-on-port | CLI 沒有匯入匯出的動詞 | 仍然成立 | 缺口 |
| G10 | store-port-examples | 沒有 JavaScript 寫的儲存接口範例 | 仍然成立 | 缺口 |
| G11 | server-on-port-examples（Non-Goals） | 示範用 server 只支援單一實例 | 仍然成立 | 缺口 |
| G12 | Deferred | 規格放在 checkout 之外的本機位置（`spec_dir` 不檢查絕對路徑與 `../`，行為未定義） | 仍然成立 | 缺口；設定；路線圖 |
| G13 | Deferred | 唯讀引用另一套規格 | 仍然成立 | 缺口；路線圖 |
| G14 | Deferred | 遠端專案的自訂產出流程 | 仍然成立 | 缺口；路線圖 |
| G15 | Deferred；engine-on-store-port、server-routes-on-store-port | 嚴格一致的讀取快照（純讀取可能短暫看到新舊混合的內容） | 仍然成立 | 缺口 |
| G16 | Deferred | 多個宿主實例之間的推送事件 | 仍然成立 | 缺口；路線圖 |
| G17 | Deferred | 依範圍版本號的快取（進行中的變更很多時，列出變更對儲存的呼叫數會成長） | 仍然成立 | 缺口 |
| G18 | server-routes-on-store-port | 遠端專案的封存不留本機備份（`snapshotCreated` 一律 false） | 仍然成立 | 缺口 |
| G19 | skills-verbs-first-wave | 「讀使用者給的文件路徑」這種技能輸入只在 agent 有檔案系統時可用 | 仍然成立 | 缺口（併入 G1 那一列的說明） |
| G20 | host-kit-client-door（Non-Goals） | 入口固定掛在 `/api/speclink/v1/` 底下；不經引擎的讀取不逐一問執行前掛鉤 | 仍然成立（設計上的限制） | SDK 第 9 節 |
| — | store-port-contract | 引擎還不會經儲存接口讀寫、Node 還拿不到它 | 已由 engine-on-store-port 與 host-kit-agent-door 解決 | 不寫 |
| — | engine-on-store-port | Node 的宿主還不能交出自己的儲存接口；沒有每次呼叫帶身分的入口 | 已由 host-kit-agent-door 解決 | 不寫 |
| — | shared-verb-parsing | SDK 只有 9 個動詞 | 已由 shared-verb-output 解決 | 不寫 |
| — | shared-verb-output | 流程設定的動詞不能用；沒有每次呼叫帶身分 | 已由 workflow-config-commands 與 host-kit-agent-door 解決 | 不寫 |
| — | workflow-config-commands | 沒有執行前檢查 | 已由 host-kit-agent-door 解決 | 不寫 |
| — | host-kit-client-door | 匯入不可用；遠端的技能安裝仍來自本機的 CLI | 已由 import-export-on-port 與 remote-skill-delivery 解決 | 不寫 |
| — | client-browser-login | 沒有任何 Server 回答登入方式 | 已由 host-kit-client-door 解決 | 不寫 |
| — | remote-read-verbs、skills-verbs-second-wave | 連到示範用 server 時清單沒有最後修改時間 | 已由 server-on-port-examples 解決 | 不寫 |
| — | skills-verbs-second-wave | worktree 的兩支技能在遠端專案不會被產生；`speclink link` 之後技能檔不會自動重新產生 | 前者已由 remote-skill-delivery 解決；後者改成 Remote 入門的一個步驟（連上之後執行 `speclink update`） | Remote「取得技能」 |

### 相容性影響（使用者要採取行動的項目，供 D10）

| 來源 | 項目 |
| --- | --- |
| node-retire-legacy-store | `createEngine` 不再接受 Store 物件；改寫成三個函式並改用 `createHost` |
| shared-verb-parsing | Node SDK 的 `dispatch` 以 `invalid_argv` 拒絕不認得的旗標；帶 `--stdin` 才讀 stdin 的動詞要一併帶 `--stdin` |
| shared-verb-output | 原本一定被拒絕的動詞現在會執行；`new change` 與空清單的 `status` 的 `output` 尾端多一個換行 |
| server-on-port-examples | 示範用 server 從 0.12.0 以前升級要走備份與還原；備份格式第 2 版（舊版讀不了）；管理頁移除資料結構遷移與 `POST /admin/data/migrate`；範圍匯出下載的內容改變 |
| client-browser-login | 存取金鑰在有金鑰圈的機器上改存金鑰圈（`credentialSource` 變成 `keychain_pat`）；`auth login --pat` 不再要求是專案成員；Server 位址含大寫主機名稱的登入要重新登入一次 |
| skills-verbs-first-wave、skills-verbs-second-wave、station-status-and-landing | 技能檔內容改變，要執行 `speclink update`；自訂工具的交棒寫法從 `speclink <名稱>` 變成 `speclink-<名稱>` |
| remote-read-verbs | 變更名稱必須是單一路徑段；`artifact cat --change archive/<封存名稱>` 改用變更名稱或封存名稱本身；evidence 的路由對不存在的名稱回 404 |
| server-routes-on-store-port | 遠端封存的回應 `snapshotCreated` 一律 false，CLI 少印一行 |
| engine-on-store-port | 版本衝突（409）的 `message` 文字改成字串鍵的表示法 |
| 這個變更 | README 的錨點 `self-hosting` 移除，改成 `embedding` 與 `official-server` |

### 用詞的存量（2026-11-18）

掃描面是 README 兩版、docs、三份介面文案（`apps/desktop/src/i18n/messages.ts`、`apps/server-web/src/i18n/messages.ts`、`packages/ui/src/i18n.tsx`）與技能原稿（`crates/engine/speclink-core/assets/skills`）。

| 詞 | 命中 |
| --- | --- |
| 團隊模式 | 沒有 |
| 主機套件 | 沒有 |
| Remote Store | README 兩版各 2；docs/product-status 兩版各 2；docs/verb-contract.md 1；技能原稿 7 份各 1（技能原稿不在 D9 的禁用範圍，不改） |
| Self-hosting（含錨點 `self-hosting`） | README 兩版各 1 |
| 自架 server | README.zh-TW.md 2；docs/development.zh-TW.md 1 |
| TeamStore | README 兩版各 1；docs/product-status 兩版各 3 |
| 參考實作／reference implementation | README 兩版各 1；docs/remote-getting-started、docs/sdk-node、docs/verb-contract 兩版各 1；docs/server-deployment 兩版各 2 |

結論：新增的兩個 avoid 詞沒有存量，任務 2.2 不會動到介面文案與技能原稿。

## 稽核結論

任務 5.1 的結果（2026-11-18）。對象是 README 兩版與 docs 的十二組文件。

### 檢查過的項目

| 檢查 | 做法 | 數量 | 結果 |
| --- | --- | --- | --- |
| 文件裡的 `speclink` 與 `speclink-server` 指令存在 | 取出行內程式碼與 shell 區塊裡的指令，逐層對照 `target/debug/speclink` 與 `target/debug/speclink-server` 的 `--help`（子指令與長旗標）。先用三個故意寫錯的指令確認檢查會抓到錯 | 26 個檔、192 個不重複的指令 | 全部存在 |
| Node SDK 文件範例用到的名稱在型別宣告裡 | 取出兩版每一段範例從 `@speclink/engine` 取用的名稱，對照 `crates/adapters/speclink-node/index.d.ts` 的匯出 | 6 個名稱（`createEngine`、`createHost`、`agentTool`、`skills`、`runStoreConformance`、`StoreConflictError`） | 全部存在 |
| Node SDK 文件的「完整範例」能執行 | 先 `npm --prefix crates/adapters/speclink-node run build`，把兩版的範例各存成暫存檔以 node 執行，再對它的路由送請求 | 2 份範例 | 合格測試通過、建立與列出變更成功、`auth-methods` 回 `{"methods":["token"]}`、沒有身分時回 401、帶金鑰時回變更清單、非入口的路徑回 404 |
| Node SDK 文件不含已移除的介面 | 在兩版搜尋 `listChanges`、`writeArtifact`、`31` | 2 個檔 | 沒有命中 |
| 動詞參考文件列的路由在路由表裡 | 取出兩版寫的「方法＋路徑」，對照宿主套件的路由表（`crates/host/speclink-host/src/http/router.rs` 與 `mod.rs`）與示範用 server 自己的路由 | 每版 54 條 | 全部找得到（兩條是 `/changes/{name}/artifacts/{*artifact}` 的實例） |
| 專案能力狀態的證據連結指向存在的檔案 | 連結守門（`scripts/docs/docs-links.mjs`） | 28 份文件的全部相對連結 | 全部指得到 |
| 文件之間帶錨點的連結指得到錨點 | 取出 README 與 docs 之間帶 `#` 的連結，對照目標檔的 `<a id>`（連結守門不檢查錨點） | 200 條 | 全部存在 |
| 停用的說法 | 在 README 與 docs 搜尋 Remote Store、Self-hosting、自架 server、團隊模式、主機套件、TeamStore、參考實作、reference implementation、Store 介面、Store interface、Store driver、Store bridge | 26 個檔 | 沒有命中 |
| Remote 入門的步驟 | 以記憶體設定（`store.driver: memory`、`identity.driver: memory`）啟動 `target/debug/speclink-server`，用無頭瀏覽器走初次設定的兩步、加成員資格、建立存取金鑰，再用 `target/debug/speclink` 在暫存資料夾執行 `link`、`auth status`、`update`、`new change`、`list`、`plan --strict-overlap`、`discard` | 第 1、2、3、4、6 節 | 每一步的輸出與文件相同：`link` 的兩行、`auth status` 的 `Credential from:` 與 `Repo 'backend' is registered in this project`、沒有 `tools` 時的 `No AI tool configurations found`、加上 `tools` 之後的 `✓ Updated skill files for: claude` 與 `Skills source: server (v1.58.0)`、`list` 的 `• smoke-test`；直接 GET 建立金鑰的路徑回 405。非成員得到 403 的那一段沒有實際觸發（只有一個帳號），以既有的 server 測試為準 |
| 十張圖 | 用無頭瀏覽器把每張圖轉成 PNG 逐一檢視；圖解的守門（`scripts/docs/docs-figures.test.mjs`） | 10 張 | 文字都在方塊內、沒有重疊；沒有 script、foreignObject 與外部資源 |
| 守門 | `node --test scripts/*.test.mjs scripts/*/*.test.mjs` | 575 條 | 全部通過，沒有被略過的 |

### 發現與修正

| 發現 | 修正 |
| --- | --- |
| 工作流程文件「怎麼呼叫」的表有一列「Host」，寫它「把引擎、Store、認證…組在一起」。這是舊架構的說法，而且和「宿主」的新意思衝突 | 兩版都改成「引擎」一列：本機專案由 CLI 裡的引擎執行，遠端專案由 Server 上的引擎執行 |
| 自由欄位的鍵有 64 個字元的上限（`crates/engine/speclink-core/src/lifecycle/fields.rs`），Node SDK 文件只寫了字元規則 | 兩版補上「最長 64 個字元」 |
| Remote 入門原本寫非成員執行 `speclink auth login` 會得到 `access denied`。實際上登入只確認身分，不看成員資格 | 改成 `auth status` 與讀寫專案的動詞才回 `access denied` |
| 只用 `speclink link` 接上的資料夾沒有 `tools` 清單，`speclink update` 會印 `No AI tool configurations found`，不會取得技能 | Remote 入門的「取得技能」加上先寫 `tools` 的步驟，故障排除加一列 |
| Server 儲存後端的舊文寫「連線 URL 裡的密碼優先於 `SPECLINK_POSTGRES_PASSWORD`」、PostgreSQL 的資料表是 `documents`、`history`、`outbox`、`meta`、檔案系統的舊內容檔「下次啟動才清」 | 照程式改寫：環境變數的密碼優先；四張表是 `meta`、`scopes`、`documents`、`events`；被取代的內容檔在同一次提交的最後刪除 |
| 路線圖寫「儲存保留每份文件不可變的歷史」與「Agent 工具沒有入口」 | 前者改成每次提交把操作者與命令交給儲存、沒有查詢的 API；後者改成已經有 `agentTool` 與 `host.dispatch`，剩下 MCP |
| 專案能力狀態的 Node SDK 一列寫「還沒有 Agent 工具層」 | 改寫；`agentTool` 已隨 SDK 提供 |
| 初次設定頁多了「儲存狀態」卡，舊的截圖高度不夠 | 重拍的兩張截圖改成 2820×1720（原本 2820×1640） |

### 沒有實際執行的部分

- `speclink auth login` 把憑證寫進系統鑰匙圈的那一步：稽核用環境變數 `SPECLINK_TOKEN` 帶金鑰，不寫入這台機器的鑰匙圈。裝置登入與瀏覽器登入的流程以程式與既有測試為準。
- Remote 入門第 5 節（Speclink Desktop）、第 7 到 9 節（重啟、離線、重置）。
- Server 部署文件的升級三步驟：指令與訊息逐字對照 `crates/host/speclink-server/src/main.rs` 與 `storage.rs`，舊版的輸出對照 `v0.12.0` 的原始碼；沒有拿一份 0.12.0 的資料實際升級。Docker Compose 與 npx 的升級路徑只有說明，沒有走過。
- 十張圖在 GitHub 淺色與深色主題的顯示：留給手動任務 4.3。

### 規格情境對應的段落

| 需求 | 情境 | 段落 |
| --- | --- | --- |
| 官方 server 定位為示範與試用 | 讀者判斷得出正式使用的路徑 | README `#official-server` 第一段；Remote 入門開頭 |
| | 營運文件開頭標明定位 | Server 部署、Server 備份與還原第一個 H2 之前 |
| | 能力對照表標明量測對象 | 專案能力狀態 `#local-and-remote` 表格前的說明 |
| 使用者文件採用本機專案與遠端專案的用詞 | 舊的說法不再出現 | 全部文件；守門「停用的說法」與詞彙守門 |
| | 詞彙表有新的詞條 | `speclink/LANGUAGE.md` 最後九個詞條與「儲存後端」 |
| | 詞彙守門涵蓋新的 avoid 詞 | 任務 2.2 的植入檢查：寫入「團隊模式」「主機套件」後守門失敗並指出檔案與詞 |
| 內嵌宿主的文件涵蓋宿主套件的對外介面 | 整合者照文件接上自己的儲存 | Node SDK `#create-host`、`#store-port`、`#conformance`、`#http-entry`、`#example` |
| | 文件不含已移除的介面 | Node SDK 兩版（搜尋沒有命中） |
| | 範例用到的名稱存在 | Node SDK 兩版的範例（六個名稱） |
| 儲存接口的範例有專文說明 | 整合者找得到同類儲存的範例 | Server 儲存後端 `#examples`、`#layout`（`#postgres`）、`#commit`、`#limits` |
| | 管理者選擇示範用 server 的儲存 | Server 儲存後端 `#choose` |
| 遠端專案的登入與技能來源有說明 | 讀者知道技能從哪裡來 | Remote 入門 `#cli` 的「取得技能」 |
| | 讀者知道會用哪一種登入方式 | Remote 入門 `#cli` 的「三種登入方式與選擇的順序」；設定說明 `#remote` 的「CLI 的技能版本提醒」 |
| 動詞參考文件涵蓋遠端可用的動詞與新增的路由 | 讀者查得到新的動詞 | 動詞參考文件 `#verb-reference` |
| | 文件與 CLI 一致 | 上表第一列（192 個指令） |
| 已知缺口逐項列出 | 整合者在採用前看得到缺口 | 專案能力狀態 `#known-gaps`（18 列） |
| | 缺口與提案一致 | 同一節，表格下方列出已經解決的缺口；對照「盤點結果」的 G1 到 G19 |
| 圖解以自帶樣式的 SVG 呈現 | 圖在深色與淺色頁面都看得清楚 | 手動任務 4.3 |
| | 兩組圖檔對等、圖檔不帶外部資源 | `docs/assets/figures/` 兩個資料夾；圖解的守門 |
| 示範用 server 的升級步驟有說明 | 管理者照文件完成升級 | Server 部署 `#upgrade-from-0-12` |
| | 管理者直接換版本啟動 | Server 部署 `#refused-start` |
| 使用者文件採漸進揭露與單一責任 | README 保留專案定位與起源；定位段以可驗收開場 | README 第一個 H2 之前與 `#acknowledgments` |
| | 首次使用者由 README 到完成第一輪；進階使用者查詢目前能力與目標 | README `#quick-start`、`#status` |
| | 整合者從 README 找到內嵌的四件事 | README `#embedding` |
| | 試用的讀者先看到定位 | README `#official-server` |
| 產品狀態以證據分類目前與規劃能力 | 三個情境 | 專案能力狀態 `#capabilities`、`#recheck` |
| 中英文文件保持結構與事實對等 | 圖解引用兩版一致 | 中英對照的守門（每一組文件一條） |
| 品質關卡蓋章效果與非成員錯誤碼的文件揭露 | 本質本機動詞列表完整 | 動詞參考文件 `#mode-assignment`（只限本機：`demo`） |
| 使用者文件載明本地產物的純文字可攜性 | 讀者判斷得出資料可攜 | README `#features` 的「純文字」；入門教學 `#init` |
| 使用者面路線圖與內部交付順序分列 | 對外方向不含時程承諾 | 專案路線圖 `#own-client` 與其餘四節 |

### 盤點表的核對

「盤點結果」的二十列文件影響都已落在對應的文件。唯一沒有做的是第 20 列的「手冊重新產生」，它在 Non-Goals。仍然成立的缺口 G1 到 G19 都在專案能力狀態的 `known-gaps`（G19 併入 G1 那一列）；G20 寫在 Node SDK 文件的 `#http-entry`。

## 發版說明的素材

任務 5.2 的結果。這一節只是素材，不是更新日誌：發版流程從這裡取要列進發版說明的項目。來源是前二十個變更封存後的提案的「相容性影響」，只收使用者要採取行動、或升級後會看到差別的項目。

### 破壞性變更（要採取行動）

| 對象 | 變了什麼 | 要做什麼 | 來源 |
| --- | --- | --- | --- |
| 用 Node SDK 的宿主 | `createEngine` 只接受本機資料夾。傳入 Store 物件會在建立引擎時拋錯 | 把儲存改寫成三個函式（`list`、`read`、`commit`），改用 `createHost`，每次呼叫給身分與範圍。用 `runStoreConformance` 檢查三個函式。見 docs/sdk-node.zh-TW.md | node-retire-legacy-store |
| 用 Node SDK 的宿主 | `dispatch` 以 `invalid_argv` 拒絕不認得的旗標與多出來的位置參數（原本靜默忽略）；帶 `--stdin` 才讀 stdin 的動詞（`new artifact`、`review stamp`、`verify stamp`），給了 stdin 就要一併帶 `--stdin` | 拿掉 CLI 沒有的旗標；給 stdin 時在 argv 加上 `--stdin` | shared-verb-parsing |
| 用 Node SDK 的宿主 | 原本一定回 `invalid_argv` 的動詞（`show`、`validate`、`archive`、`discuss` 全系列、`workflow-config` 等）現在會執行；`new change` 與沒有變更時的 `status`，`output` 尾端多一個換行 | 依賴「這些動詞一定被拒絕」或比對 `output` 全文的程式要調整 | shared-verb-output、workflow-config-commands |
| 示範用 server 的管理者 | 0.12.0 與更早版本建立的資料，新版不能直接開啟：server 會拒絕啟動，不改動資料 | 用舊版執行 `backup`，把組態指向空的儲存位置與空的 identity 資料庫，再用新版執行 `restore`。見 docs/server-deployment.zh-TW.md 的升級一節 | server-on-port-examples |
| 示範用 server 的管理者 | 備份格式改成第 2 版。新版能檢查與還原第 1 版；舊版讀不了第 2 版 | 升級之後做的備份不能拿回舊版還原。回退要用升級前的備份與原本的資料 | server-on-port-examples |
| 示範用 server 的管理者 | 管理頁移除「遷移資料結構」；`POST /admin/data/migrate` 回 404；系統頁與初次設定頁不再顯示契約版本、等級與能力；範圍匯出下載的檔案內容改用鍵 | 讀這些欄位或下載檔的自動化腳本要調整 | server-on-port-examples |
| 連到 README 舊錨點的外部連結 | `#self-hosting` 移除，改成 `#embedding` 與 `#official-server` | 更新連結；舊連結會落在頁首 | docs-embedded-host |

### 升級後會看到的差別（多半不用動作）

| 對象 | 變了什麼 | 來源 |
| --- | --- | --- |
| 全部使用者 | 技能檔的內容改變（讀寫規格一律經過動詞；commit 與 worktree-merge 多一步回報落地）。升級後執行一次 `speclink update` | skills-verbs-first-wave、skills-verbs-second-wave、station-status-and-landing |
| 用自訂工具描述子的人 | 技能裡交棒給其他技能的寫法，從 `speclink <名稱>` 變成 `speclink-<名稱>` | skills-verbs-second-wave |
| 遠端專案的使用者 | `trace`、`change rank`、`plan --strict-overlap` 在遠端專案可用；技能改由 Server 提供，連上之後執行 `speclink update` | skills-verbs-first-wave、remote-read-verbs、remote-skill-delivery |
| 用存取金鑰登入 CLI 的人 | 在有鑰匙圈的機器上，金鑰改存鑰匙圈，`speclink auth status --json` 的 `credentialSource` 從 `credentials_file` 變成 `keychain_pat`；既有的憑證檔照常可用 | client-browser-login |
| 用存取金鑰登入 CLI 的人 | `auth login --pat` 與 `--token-stdin` 不再要求你是該專案的成員：金鑰有效就登入成功，不是成員時執行動詞才收到權限錯誤 | client-browser-login |
| Server 位址含大寫主機名稱的人 | 位址正規化之後鑰匙圈的鍵不同，要重新登入一次 | client-browser-login |
| 讀 `--json` 的腳本 | `speclink list --json` 每個變更多了 `created` 與 `modifiedAt`；`speclink show <change> --json` 在有記錄時多了審查、驗證、分支與落地的欄位；有自由欄位時多一個 `fields` 鍵 | remote-read-verbs、station-status-and-landing、host-free-fields |
| 用 `artifact cat` 讀封存文件的人 | 變更名稱必須是單一路徑段。`artifact cat --change archive/<封存名稱>` 改用變更名稱或封存名稱本身 | remote-read-verbs |
| 遠端專案的使用者 | 遠端封存不留本機備份：回應的 `snapshotCreated` 一律 false，CLI 少印 `Snapshot created for unarchive support.` 一行 | server-routes-on-store-port |
| 讀錯誤訊息文字的程式 | 版本衝突（HTTP 409，`revision_conflict`）的 `message` 改用字串鍵表示文件；錯誤碼與狀態碼不變 | engine-on-store-port、server-routes-on-store-port |
| 接 evidence 路由的程式 | 變更不存在時回 404 `not_found`（原本回空集合）；變更存在但沒有記錄時仍回空集合 | remote-read-verbs |
| 用 shell 補全的人 | `speclink completion generate` 產出的腳本裡子指令的排列順序改變，功能不變 | shared-verb-parsing、workflow-config-commands |
| 用 `workflow-config` 的人 | 值沒有變的寫入不再有後續動作（不重新同步技能檔、不印過期提示、不建空的 config.yaml） | workflow-config-commands |
| 從原始碼引用舊儲存層的程式 | `speclink-store` 與三個舊 driver 的 crate 已移除（它們沒有發布到 crates.io） | retire-teamstore-and-drivers |

### 新功能（供發版說明取材）

- 內嵌：`createHost`、三個儲存函式、合格測試、兩個掛鉤、給 Agent 工具的入口與 `agentTool`、給 CLI 與 Speclink Desktop 的 HTTP 入口、匯出與匯入、技能清單的設定。
- 三個儲存接口的範例（SQLite、PostgreSQL、檔案系統）。
- 用戶端的瀏覽器登入，與三種登入方式的自動選擇。
- 遠端專案的技能由 Server 提供；CLI 的技能版本提醒。
- 新動詞：`evidence show`、`change landed`、`change field set`／`unset`、`discuss field set`／`unset`；`new change` 與 `discuss new` 的 `--field`。
- 關卡狀態、分支與落地出現在線上的清單與 `show --json`；Speclink Desktop 的遠端專案顯示關卡的標示。
- 文件：README 的「把 Speclink 內嵌進你的系統」與「試用官方 server」兩節、改寫的 Node SDK 文件、儲存接口的範例、已知缺口、五張圖解。

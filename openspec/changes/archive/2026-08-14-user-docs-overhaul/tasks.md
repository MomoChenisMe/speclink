## 1. 截圖場景基建（測試先行）

- [x] 1.1 先寫 scripts/docs-screenshots.test.mjs 再實作 scripts/docs-screenshots.mjs，落實設計「D1：截圖以「腳本備份還原＋人工擷取」分工」與 spec「使用者文件以截圖呈現實際介面」的腳本條款：以 --dry-run 驗證備份路徑推導、示範 workspace 的產出清單、以及兩條分支判斷（狀態目錄原本不存在時還原階段應刪除而非搬回；偵測到 app 執行中應以非零結束且不搬移任何目錄）；測試不得實際搬移使用者目錄，一律以注入的假路徑與假偵測結果驗證。驗證：node --test scripts/docs-screenshots.test.mjs 全綠 <!-- speclink-task:tsk_01KZTJHVY56Y5VKTE6C2Q9AJKT -->
- [x] 1.2 實作 --setup 的示範內容產生：在系統暫存區建出 git 初始化並經 speclink init 的示範 workspace，造出分別位於提案中、進行中、已封存的示範變更各至少一個與一則已結論的討論，使看板三欄與討論頁皆非空；示範內容全部由腳本產生，不引用使用者任何真實資料。驗證：實跑 --setup 後以 speclink list 與 speclink discuss list 確認三種狀態的變更與討論皆存在 <!-- speclink-task:tsk_01KZTJHVY5P2DAMBF6C1Q2FP66 -->
- [x] 1.3 實作 --restore 與中斷保護：註冊 SIGINT 與 SIGTERM 處理器，使中斷時仍執行還原；還原順序為先刪除拍攝期間產生的狀態目錄再搬回備份，備份不存在時改為刪除；還原失敗時印出備份路徑供人工搬回；示範 workspace 一併清除。驗證：以假的狀態目錄實跑 setup 後送出 SIGINT，確認目錄內容與執行前逐位元組相同 <!-- speclink-task:tsk_01KZTJHVY55CYE7DDTBRYF00CE -->

## 2. 截圖擷取（人工）

- [x] [M] 2.1 擷取六張 desktop 截圖，落實設計「D2：截圖清單固定為九張，中英共用」的 desktop 六張：先關閉 desktop app，執行截圖腳本的 setup，依提示把示範 workspace 加入 app，逐張擷取並存入 docs/assets/screenshots/——desktop-board.png（看板三欄與卡片）、desktop-change-drawer.png（變更詳情抽屜含任務與 artifacts）、desktop-spec.png（規格檢視）、desktop-discussion.png（討論記錄檢視）、desktop-archived.png（已封存檢視）、desktop-settings.png（設定含 Server 連線）；擷取時避開載入骨架與動畫中間態。完成後執行腳本的 restore。完成判準：六個檔案存在，且畫面內不含使用者任何真實 workspace 名稱、變更名稱或連線位址 <!-- speclink-task:tsk_01KZTJHVY5FVFXB35P81CTE8TC -->
- [x] [M] 2.2 擷取三張 server 後台截圖，補齊設計 D2 清單的 server 三張：依開發環境入口文件啟動 server，逐張擷取並存入 docs/assets/screenshots/——server-setup.png（首次 setup 畫面）、server-overview.png（後台總覽）、server-members.png（成員與 PAT 管理）；擷取時避開含真實 token 值的畫面（token 以遮蔽或示範值呈現）。完成判準：三個檔案存在，且畫面內不含可用的真實憑證 <!-- speclink-task:tsk_01KZTJHVY51MN5X2888KMBRWW6 -->
- [x] [M] 2.3 確認還原無誤：執行截圖流程後開啟 desktop app，確認 workspace 分頁與 Server 連線清單與拍攝前一致。完成判準：分頁與連線逐項比對相同 <!-- speclink-task:tsk_01KZTJHVY5RF2AE5J2GDYBBAQ0 -->

## 3. 入口與上手文件

- [x] 3.1 重寫 README 中英兩份，落實設計「D3：文件分層與各自的單一責任」的入口層與 spec「使用者文件以截圖呈現實際介面」。**保留**（正典必留）：品牌圖片、標語、語言切換、Rust SDD 引擎與工具平台定位、共用 change／artifact／task／verify／archive 語意、Local Repo 與 Remote Store 雙路徑、Spectra App 2.3.1 起源、單一份目前狀態摘要、最短流程心智模型、快速開始入口、安裝章節（桌面三平台下載入口與 CLI 一行安裝——受 `release-signing-and-channels` 的 spec「安裝通路文件與發布狀態誠實化」約束，SHALL NOT 刪除或降級為連結）、文件地圖（含新增的路線圖）；並於定位段落後內嵌至少一張 desktop 截圖。**新增**：安裝章節補上與桌面對等的 server 一行啟動表（npx／Docker／Compose 三種形態各一行指令，細節連往 Server 部署），使讀者在 README 就看得出三條路都存在。**移出**（違反正典「各文件以連結導向下一層細節」而被 README 複製的下層細節，改為一行摘要＋連結，內容併入工作流正典）：`improve` 小節整段的操作細節（掃描前收斂範圍、開場讀已封存討論的 Ruled out 等）、品質站小節的六列比較表與四段蓋章時序／內容指紋凍結／複驗必修集合遞減規則、開發章節的逐條測試指令（併入開發環境入口文件）。**消重**：目前狀態的引言區塊與「目前能力」章節兩處重疊敘述合併為一份摘要，逐項證據一律連往產品能力狀態文件。**校正**：文件地圖結尾「進階 verb-contract 使用者指南尚未建立」的陳述已不成立（該文件存在），改為指向它。**修正既有漂移**：中英兩版目前行數不等（189 對 202），改寫後兩版 H2 章節集合與順序、截圖引用集合須一致。驗證：並列比對兩版的 H2 標題序列與截圖引用逐項相同；全文搜尋確認移出的三類細節不再出現於 README，且工作流正典中可找到對應內容 <!-- speclink-task:tsk_01KZTJHVY5RC44DHJGECSMPC44 -->
- [x] 3.2 重寫 getting-started 中英兩份：只承載本地第一輪可完成的流程（init、提案、實作、檢查、封存），每步標明預期輸出，遇選用分支連到工作流文件；不重複 README 的定位內容。驗證：照文件逐步實跑一輪，每步的預期輸出與實際相符 <!-- speclink-task:tsk_01KZTJHVY5VBWSCM201X8ZSQ25 -->
- [x] 3.3 重寫 remote-getting-started 中英兩份：從 server setup、membership、登入到 Desktop 與 CLI 連線與失聯恢復，內嵌 server-setup.png；只寫已驗證可走的入口。驗證：照文件逐步實跑，每步入口確實存在 <!-- speclink-task:tsk_01KZTJHVY5HB4405BQ9VK1CZCE -->

## 4. 工作流正典

- [x] 4.1 重寫 workflow 中英兩份，落實 spec「工作流正典逐站列出技能與完成判準」：以單一結構列出 onboard、discuss、improve、propose、apply、ingest、quality、review、verify、archive 與 worktree 全部站別，每站含用途、對應 /speclink-* 技能名稱、完成判準與下一站；保留既有的討論結論分流、恢復路徑與呼叫層級章節；內嵌 desktop-board.png 與 desktop-change-drawer.png 說明看板與抽屜對應的流程位置。驗證：逐站檢查四項資訊皆在本文件內可找到，且技能名稱與 .claude/skills/ 下實際存在的目錄逐一對得上 <!-- speclink-task:tsk_01KZTJHVY50VKES9ANNA4G57KE -->

## 5. 參考文件

- [x] 5.1 重寫 product-status 中英兩份，落實 spec「本地與遠端能力對照集中呈現」：加入本地與遠端逐項能力對照表、刷新查核日期、移除已不成立的 verb-contract 文件缺口記載、依實際交付狀態更新各能力列。驗證：對照表每一列的狀態主張都能指到版本庫中的證據（測試、入口或規格），且全文搜尋無「docs/verb-contract.md 不存在」類的陳述 <!-- speclink-task:tsk_01KZTJHVY5Z800SMR0JVG25MRY -->
- [x] 5.2 重寫 configuration 與 verb-contract 中英四份：configuration 只回答設定欄位歸屬與意義（Local 與 Remote 分列），verb-contract 只回答動詞與旗標契約；兩者皆移除與其他文件重複的流程敘述，改以連結導向。驗證：逐份確認無與 workflow 或 getting-started 重複的流程段落 <!-- speclink-task:tsk_01KZTJHVY5HNVFA2D77J55QYQX -->
- [x] 5.3 重寫 sdk-node 與 development 中英四份：sdk-node 只回答 Node SDK 介面並維持發布狀態的誠實陳述，development 只回答開發環境與本機建置入口。驗證：sdk-node 全文與 docs/product-status 的 Node SDK 列陳述一致；development 所列指令實跑可用 <!-- speclink-task:tsk_01KZTJHVY5BR80FWXBGXCFS446 -->
- [x] 5.4 重寫 server-deployment、server-store-drivers、server-backup 三份（維持僅中文）：分別只回答部署與升級、driver 選型前提、備份還原操作；內嵌 server-overview.png 與 server-members.png 於部署文件的驗收段落。驗證：三份文件的職責無互相重疊，所列指令與 speclink-server --help 的實際入口相符 <!-- speclink-task:tsk_01KZTJHVY52J9TBBBVMG4Z18C9 -->

## 6. 使用者面路線圖

- [x] 6.1 新增 docs/roadmap.zh-TW.md 與 docs/roadmap.md，落實 spec「使用者面路線圖與內部交付順序分列」與設計「D4：使用者面路線圖只寫方向、不寫日期」：涵蓋 SDK 發布、以引擎自建客戶端（使用者以 SDK 引擎自行開發桌面或其他前端）、遠端協作完整化、agent 工具整合（Copilot、MCP 等）、系統整合能力五條線，每條寫要解決什麼、目前到哪、可觀察的下一步；與實作重構路線圖互相連結並說明受眾差異；全文不得出現版本號或日期承諾。驗證：全文搜尋無版本號與日期形式的承諾字樣；兩版章節結構一致 <!-- speclink-task:tsk_01KZTJHVY536P8CTXAX48A3ZJH -->

## 7. 一致性驗收

- [x] 7.1 落實 spec「文件內部連結全部可解析」：新增一道可重複執行的檢查，掃出全部使用者文件的 markdown 相對連結與圖片路徑並逐一確認檔案存在，任一斷鏈即以非零結束；納入 scripts 測試面。驗證：檢查對現況全綠；刻意改壞一個連結後確實以非零結束並指出該路徑 <!-- speclink-task:tsk_01KZTJHVY55RCCDXSV8VV2BR78 -->
- [x] 7.2 中英對等驗收，落實 spec「中英文文件保持結構與事實對等」與設計「D5：中英對等以「同結構、同事實」定義」：逐組成對文件（README、getting-started、workflow、product-status、roadmap）比對 H2 章節標題序列與截圖引用集合，確認數量與順序一致、事實主張無互相矛盾；並確認繁體中文散文使用正典詞彙（轉為變更、已轉出變更、封存），引擎動詞只出現在 code span。驗證：逐組列出比對結果全部一致；以 speclink language show 的避免詞清單掃過繁中文件，命中者皆位於 code span 內 <!-- speclink-task:tsk_01KZTJHVY5NNE5HBM6QYA7T8G2 -->

## 8. 補充輪（使用者回饋）

- [x] 8.1 落實 spec「使用者文件載明本地產物的 OpenSpec 結構相容性」：README 中英兩份與入門教學中英兩份補上本地產物沿用 OpenSpec 目錄結構的說明，載明純 Markdown／YAML、可不經 Speclink 讀寫、Git diff 看得見，並列出 Speclink 的兩項擴充。驗證：四份文件皆可搜到目錄結構說明，且入門教學含目錄樹範例 <!-- speclink-task:tsk_01KZYXC18E11QCYMBW161KVYVV -->
- [x] 8.2 落實 spec「安裝章節載明桌面 app 與 CLI 的佈署衝突」：README 中英兩份的安裝章節補上覆蓋行為的逐平台差異、對釘選版本的影響與保留自有 CLI 的做法；入門教學安裝段落補一句指回 README。驗證：說明的行為與 apps/desktop/src/core/cliInstall.ts 的 cliDeployPlan／needsRedeploy 逐平台分流一致 <!-- speclink-task:tsk_01KZYXC1ABXC9HX3RHRNZ58DTG -->
- [x] 8.3 校正 Codex 技能呼叫語法的敘述：`$speclink-*` 為明確呼叫寫法，`/skills` 清單亦可選取同一技能；入門教學、工作流呼叫層級與能力狀態三處同步。驗證：敘述與 crates/speclink-core/src/skills.rs 的 slash_replacement 一致，且不宣稱 `/` 完全不可用 <!-- speclink-task:tsk_01KZYXC1BQ67S2D65C3HMTSC6S -->
- [x] 8.4 說明 Check 階段的自動執行時機：入門教學中英兩份載明 propose／apply／ingest 三個技能已自動跑 analyze（propose 與 ingest 收尾另跑 validate），並說明自己下指令的時機。驗證：敘述與三份 SKILL.md 的實際步驟一致 <!-- speclink-task:tsk_01KZYXC1D4P2CE694EWE9JW82J -->
- [x] 8.5 繁體中文用詞調整：「選跑」改為「可選」或「你選擇要跑的」。驗證：全文搜尋無「選跑」 <!-- speclink-task:tsk_01KZYXC1EF0RFJ9169HV3TBFHN -->
- [x] 8.6 落實 spec「使用者文件採簡化技術英文的寫作紀律」：逐份通讀全部使用者文件（中英兩版與 zh-TW 專屬的 server 三份），依 STE 紀律調整散文——拆長句、改主動語態、固定動詞、把並列項改成清單、段落先講結果。不以腳本代行。驗證：逐份記錄通讀後的調整重點，並確認程式碼區塊、路徑與識別符未被改動 <!-- speclink-task:tsk_01KZYXC1FTJVGN50HYWBANVQFG -->
- [x] 8.7 落實 spec「官方 server 定位為參考實作而非唯一路徑」與設計「D6」：README 中英兩份的部署路徑與安裝章節、Remote 入門中英兩份的開場、Node SDK 中英兩份的用途段、Server 部署開場、以及本地與遠端對照表的量測對象註記，各補上「官方 server 是參考實作、遠端模式由 host-runtime 與 client-protocol 兩份契約定義、可自建 server 端」。驗證：陳述與 openspec/specs/reference-server 的 Purpose 一致（該 spec 自述為「參考 server 實作」與「wire contract 的活基準」），且全文無「遠端模式必須使用官方 server」類的陳述 <!-- speclink-task:tsk_01KZZ2MTXDXYQR2KB2VHF0J4GT -->
- [x] 8.8 移除使用者文件對兩份維護者自用架構文件的全部引用（README 中英的文件地圖背景區塊、workflow／product-status／roadmap／sdk-node 的相關文件清單、server-deployment 的延伸閱讀）；product-status 的 Deprecated 一列改以實際程式碼與現行正典為證據。驗證：全 docs 與兩份 README 搜尋 `platform-architecture` 與 `implementation-refactor-roadmap` 零命中（兩份檔案自身除外），且連結檢查全綠 <!-- speclink-task:tsk_01KZZ2MTZ565GRBJCBBZE9JE6S -->

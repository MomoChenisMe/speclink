## Context

- 來源為 archived-quality-history-access 討論結論；此變更服務封存後查閱品質關卡的開發者、PO 與 PM。
- 歷次紀錄及索引已隨封存保存。ArchivedDetailPage 只有靜態品質狀態籤，App 雖傳入 checkRecords，封存元件沒有對應 prop 與使用路徑。
- history::index 同時被查詢、蓋章、深度報告與封存計畫使用，會檢查全部紀錄；history::record 先呼叫 index。直接放寬 index 會影響寫入及安全驗收，因此必須區分查閱與嚴格檢查。
- FsStore 的既有檢查文件讀取將 Result 轉成 Option，無法區分缺檔和權限、連結或 IO 錯誤。Host history::query 是 CLI、Node、Desktop、本機與 Remote 的共同落點。

## Goals / Non-Goals

**Goals:**
- 封存前後從相同品質關卡操作查閱歷次紀錄，完整保留仍為預設，沿用既有各類保留設定。
- 單筆缺失或內容異動可見，其他完整紀錄可讀，整體來源錯誤不偽裝成空清單。
- 健康回應相容、缺損情境明確改變；寫入、驗收、復原與路徑界線不放寬。

**Non-Goals:**
- 不新增清除、刪除、重建索引、自動修復、只留最後一次或安全章功能。
- 不重寫既有封存文件、原章、政策、歷史結果或技能資產，不新增安全標準或模型執行。
- 不新增獨立文件樹歷史節點，不將歷史視為活動工單；不新增安全報告查閱頁。

## Decisions

### D1: 可信檢查文件讀取

在 core Store 增加窄介面 read_check_document(change, relative_path, archived) → Result<Option<String>>；僅服務既有 checks 允許路徑，None 代表真正不存在，Err 保留讀取失敗。FsStore 委派既有 checks::read_document，不再於此介面吞錯；BridgeStore 在已完成權限檢查及快照載入後讀對應的活動／封存 DocumentId。純記憶體測試 Store 可使用預設實作委派既有 Option 讀取，但具外部 IO 的實作必須覆寫，不能將 IO 錯誤當 None。

活動／封存定位由查閱入口解析一次，checked read 仍受 ensure_checks_readable、有效相對路徑、根目錄界線、檔案大小與拒絕符號連結限制；封存尚未提交／journal 未復原時不可讀半套資料。此介面讓 core 不直接讀磁碟，也不讓 UI 判斷文件真相。其他一般 artifact 的 Option 介面不改，避免一次性重構整個 Store。

### D2: 查閱與嚴格完整性檢查分流

保留 history::index、既有嚴格 record 及其寫入／驗收呼叫語意。於同一 history 模組加入 browse_index 與 browse_record，沿用共同索引結構驗證、路徑、摘要及遮罩判斷，不複製整套驗證演算法。

browse_index 先確認變更及已提交讀取狀態，透過 D1 讀取及驗證索引結構，再只檢視請求類別的每筆文件，輸出原 CheckEntry 加選填 read_error。omitted 不讀已按政策省略的檔案；不存在為 missing_check_record；digest 或審查／驗證文件頭與索引 metadata 不符為 integrity_error；單筆 IO、連結、編碼或大小限制錯誤為 check_record_read_error。單筆異常只留下安全代碼，不回傳該筆原文。不存在索引沿用舊變更無歷史的空清單；存在但不可讀、格式錯誤、重複 ID 或非法路徑則整體失敗，不重建、不略過壞項目。

browse_record 依已驗證索引的 ID 與 kind 選定一筆，再重新讀取該筆及驗證 digest、metadata、遮罩；不檢查其他紀錄內容。缺檔、omitted 或完整性錯誤回失敗，避免清單載入後被修改的內容仍可顯示。此讀取不寫文件、索引、metadata 或交易資料；strict index 在後續蓋章、報告提交、風險決策與封存仍檢查全部必要紀錄，缺損仍拒絕。

### D3: 共用通訊型別與呼叫端相容

CheckHistoryEntry 增加 readError?: missing_check_record | integrity_error | check_record_read_error，Rust 使用帶 default 且 None 不序列化的選填列舉，JSON Schema 同步由既有型別匯出。readError 不放進持久化 CheckEntry，不覆蓋 outcome；omit 使用既有 outcome=omitted。

Host history::query 的清單及 review／verify 單筆改用 D2 的查閱入口，本機、Remote、Server、Node 及 CLI 皆共用；安全報告提交、接受風險與驗收維持嚴格入口，既有安全摘要與安全單筆查閱不擴張。typed client 沿用現有 station_history，路由與 Tauri 命令不新增第二套邏輯。

健康 JSON 完全省略 readError，健康人眼行與單筆全文維持原值。部分缺損時 review history／verify history 清單成功 exit0，異常列人眼輸出在原五欄後追加 readError=代碼，JSON 在該列加入 readError；這是刻意取代原整組失敗的行為。指定異常 ID 仍 stderr＋exit1，找不到變更／非法 ID／壞索引／整體 IO 仍失敗，參數錯誤2，無 stdin，--no-color 不產生 ANSI；健康及異常均覆蓋 Local／Remote 回歸對照。

新 reader 接舊 server 時缺 readError 視為舊形狀，單筆開啟仍由來源驗證；舊 server 回整體錯誤就顯示錯誤及重試，不聲稱具備單筆隔離或自行重算。新 server 的未知附加欄位不影響既有健康 reader。不增加設定、端點、binding capability 或 SDK 動詞。

### D4: 共用紀錄元件處理逐筆狀態

擴充 packages/ui 的 CheckRecordEntry 與 QualityCheckRecordList、CheckRecordBody、useCheckRecords。readError 的三種狀態由共用 i18n 文案呈現，保留原結果與時間，不把歷史 clean 當成文件現在完整。異常項仍可選取以看到狀態及重試，缺失與異動內容不進 Markdown；重試會重新載清單及所選單筆。選取後來源再失敗時顯示錯誤與重試，不殘留前一筆內容，也不只顯示永久骨架。

CheckRecordsAdapter.list 增加選填 kinds 參數，未指定時保持既有呼叫行為；品質關卡明確只請求 review、verify，Desktop adapter 僅對這兩類呼叫既有查詢，避免 Promise.all 中安全報告或 acceptance receipt 的失敗拖累品質紀錄。安全資料 UI 的既有預設路徑不變。入口被舊遠端 capability 停用時顯示不支援說明，而不是讓可點章無反應。

清單時間解析與排序共用一處：時間前綴為 UTC；無前綴的舊文件用 createdAt 解析為時間，倒序，同秒以 recordId 升序穩定排列。時間缺失或錯誤不得顯示假日期，列出原時間並以0排序。scope identity 包含 adapter、change、refresh generation、選取 ID，切換專案、活動／封存項目時清除舊清單、正文及展開狀態；晚到回應不能回填，失敗不能當空清單。

### D5: 封存品質關卡接上相同查閱入口

ArchivedDetailPageProps 明確接收既有 checkRecords adapter；App 的已封存頁組裝沿用同一 workspace adapter，以完整 datedName 定址，不剝日期。沿用活動頁既有的 QualityCheckRecordList、CheckRecordBody、TabsContent、DocHeader、Markdown、DocSkeleton、StatusPill 原語及文件動作，將品質章的展開按鈕與 ARIA 接法收在共用紀錄元件，活動／封存頁各接自己的狀態。兩站各自展開，Enter／Space 切換，aria-expanded／aria-controls 與 section 對應。

只有品質關卡下出現歷史清單，選取後切換閱讀區；歷史 docTarget 必須帶 archived datedName 與索引中的 relativePath，既有文件開啟／複製路徑操作不得組成 undefined。閱讀卡標題顯示類別與時間，使用既有 Markdown 與本頁目錄骨架；不新增獨立文件樹節點。沿用既有 carried review／verify 未通過工單的條件式節點與唯讀內容，不因新增歷史入口而移除。

### D6: 回歸、完整保留與驗收界線

建立活動與封存的隔離測試資料，含六筆 review 與六筆 verify、不同時區、舊 ULID 檔名、同秒不同 ID、omitted。預設封存前後索引所指文件與內容 digest 均保留；false 保持 omitted，不觸發讀缺件，歷史數量不只剩一筆。手動刪除／修改只作用於測試 fixture，不動真實封存證據。

驗收包含：一筆 review 缺失、一筆 verify 異動、完整單筆仍能取得；安全類別缺件不拖累 review／verify；索引錯誤或來源401／403／離線不能變成空清單；清單載入後再刪檔可重試且無錯誤正文；活動與封存、Local 與 Remote 具相同隔離語意。

同一份缺損 fixture 再執行蓋章或封存，須失敗且保留工單、章、索引及所有既有文件，原子性及安全驗收不放寬。測試分佈 core／fs／Host／Protocol／CLI／Server／UI／Desktop；只跑受影響 Cargo package 與前端 workspace，Node binding 共用 Host 的結果以既有 SDK 測試確認。最後執行專案 scripts 守門，不在任務要求完整測試，完整測試仍由 CI 執行。

## Implementation Contract

| 範圍 | 可觀察結果 | 失敗界線 | 驗收 |
| --- | --- | --- | --- |
| 歷史清單 | 原六欄＋僅異常項有 readError，outcome 不變 | 壞索引與來源失敗整體回錯 | 部分缺損仍列完整項目，健康輸出回歸 |
| review／verify 單筆 | 驗證所選 ID 後回既有遮罩內容及 metadata | 異常目標不回內容，不因別筆異常失敗 | 清單載入後刪檔及跨類缺檔測試 |
| Store | 真正缺檔、存在文件與讀取失敗可區分 | 拒連結／越界／未提交交易，保留來源錯誤 | fs 與 Remote fixture 驗證 |
| Desktop | 活動與封存品質章獨立展開，最新在前，全文唯讀 | 單筆異常可見、來源失敗可重試、舊來源不假裝支援 | 兩站各六筆、鍵盤、docTarget、換專案晚回應 |
| 保存與寫入 | 所有歷次預設保留，false 按舊政策略過 | 查閱不寫入，完整性不足不通過蓋章或封存 | 比對前後完整文件與 metadata |

實作範圍僅 proposal 的五項能力與上述讀取／呈現接點；不擴成歷史編輯器、一般 Store 重構、刪除工具或資安掃描功能。

## Risks / Trade-offs

- [健康輸出或 typed schema 回歸] → readError 只有異常時序列化；CLI Local／Remote、Protocol serialization 與既有 golden 逐一對照。
- [Windows／macOS／Linux 錯誤差異] → 使用原生 path 與既有受控讀取；權限失敗以注入錯誤測試，不依賴 root 權限下的 chmod 結果；時間前綴按 UTC，舊紀錄解析 offset。
- [查閱可容忍被誤用為驗收可容忍] → 嚴格 index 及寫入呼叫點保留，使用同一缺損 fixture 驗證查閱可用而蓋章／封存失敗。
- [大型索引檢查成本] → 維持既有100000筆上限及64 MiB單檔上限，按請求類別檢視，不新增全目錄掃描、分頁服務或背景工作；不把原文保存在清單回應。
- [Remote 版本不一致] → 沿用缺欄位向後相容與明示整體錯誤，不用 UI 補寫索引模擬新 server 行為。

## Migration Plan

- 不變更磁碟上的 CheckIndex 版本、檔名、政策與完成紀錄；先落 Store／core，再落 Protocol／Host，最後接 UI 及 Desktop。
- 新 server 與 client 可逐步更新；舊 server 在缺損時仍會報整體錯誤，需更新 server 才能取得逐筆診斷。回退程式後保留原檔可由舊版讀取，舊版仍有原本的缺損阻擋行為。
- 此變更不寫 YAML 純量、不新增 Git 動作、不需要資料遷移或遠端寫入，因此沒有多文件寫入順序；任何歷史查閱都須保持原文件不變。

## Open Questions

無產品方向待決；實作時依本契約完成測試，不另擴充政策或端點。

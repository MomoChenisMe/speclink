## 1. 儲存邊界先測試再實作

- [x] 1.1 RED：為「檢查紀錄的缺席與讀取失敗可區分」建立活動及封存fixture，分別重現真正缺檔、索引IO／編碼錯誤、單筆IO、連結、越界與未提交交易被吞成None的問題（D1、D6；crates/engine/speclink-fs/src/checks.rs、crates/engine/speclink-fs/src/lib.rs、crates/host/speclink-host/tests/security_remote.rs）。驗證：cargo test -p speclink-fs及cargo test -p speclink-host，記錄新行為的失敗斷言；既有拒越界測試須維持通過，不刪真實封存文件。 <!-- speclink-task:tsk_01M5M8N2SH5SKF68K91EED63KP -->
- [x] 1.2 GREEN／REFACTOR：於crates/engine/speclink-core/src/store.rs加入可信檢查文件窄介面，由crates/engine/speclink-fs/src/lib.rs委派既有受控讀取，crates/host/speclink-host/src/bridge.rs使用已授權快照；真正不存在與Err分開，保留活動／封存路徑、限制及交易狀態（D1）。驗證：1.1新增測試轉綠，cargo test -p speclink-fs及cargo test -p speclink-host通過，讀取前後檔案與revision無變化。 <!-- speclink-task:tsk_01M5M8N2SHXQ18K0PP1C8SY99V -->

## 2. 查閱與嚴格完整性檢查分流

- [x] 2.1 RED：在crates/engine/speclink-core/src/quality/history.rs與crates/engine/speclink-core/src/quality/station/tests.rs建立六筆review及六筆verify、omitted、壞索引、單筆metadata異動及清單後刪檔的行為測試，重現「審查完成紀錄查詢」與「驗證完成紀錄查詢」的完整單筆被別筆缺損拖累；另鎖定同一fixture的嚴格index、蓋章與封存仍拒絕、預設封存保留全部及false仍omitted（D2、D6）。驗證：cargo test -p speclink-core，新查閱隔離斷言為紅，原嚴格守門與保留政策斷言為綠。 <!-- speclink-task:tsk_01M5M8N2SHWC2QRAVVD96B40W8 -->
- [x] 2.2 GREEN／REFACTOR：於crates/engine/speclink-core/src/quality/history.rs實作browse_index及browse_record，索引結構先完整驗證、清單逐筆診斷、單筆只重驗所選內容，沿用摘要、metadata與遮罩邏輯，保留嚴格index／record及寫入／安全驗收呼叫（D2）。驗證：2.1轉綠，cargo test -p speclink-core及cargo test -p speclink-fs通過，斷言問題單筆無原文、完整單筆可讀、壞索引整體失敗及無文件效果。 <!-- speclink-task:tsk_01M5M8N2SH43Z138ESV307QXN9 -->

## 3. 共用型別、Host與各端相容

- [x] 3.1 RED：在crates/protocol/speclink-protocol/src/query.rs、crates/host/speclink-host/tests/security_remote.rs與crates/host/speclink-server/tests/it/api/review_api.rs，為「檢查歷史與安全驗收的共用契約」加入健康payload無readError、舊payload可讀、異常列有三種readError且outcome不變、Local／Remote同形、跨類缺檔、401／403與唯讀revision測試（D3、D6）。驗證：cargo test -p speclink-protocol、cargo test -p speclink-host及cargo test -p speclink-server，新增契約測試先紅且無模擬安全驗收通過。 <!-- speclink-task:tsk_01M5M8N2SHEKCKEQQV24Q3NGS8 -->
- [x] 3.2 GREEN／REFACTOR：在crates/protocol/speclink-protocol/src/query.rs新增只於異常序列化的選填readError列舉，於crates/host/speclink-host/src/history.rs統一接入browse查閱，沿用Server、typed client、Node與Desktop既有路徑；安全寫入及驗收仍用嚴格入口（D3）。驗證：3.1轉綠，cargo test -p speclink-protocol、cargo test -p speclink-host及cargo test -p speclink-server通過，檢查由型別匯出的JSON Schema和回歸對照健康欄位不變。 <!-- speclink-task:tsk_01M5M8N2SHPRJGJR3KC7AZZ0A5 -->
- [x] 3.3 RED：在crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs及crates/adapters/speclink-node既有測試加入review／verify history的健康人眼、JSON、--no-color、異常清單exit0與指定異常ID失敗、非法ID、參數exit2、舊server失敗及SDK選填欄位測試（D3、D6）。驗證：cargo test -p speclink-cli及Node建置後測試，先確認新增人眼異常標記斷言失敗，健康golden保持原值。 <!-- speclink-task:tsk_01M5M8N2SH5HPVS93CES8WQC3R -->
- [x] 3.4 GREEN／REFACTOR：於crates/adapters/speclink-cli/src/verbs/station.rs的人眼異常列追加readError=代碼，健康五欄維持原輸出；確認Node沿用Host取得相同新增選填欄位，不增動詞或旗標（D3）。驗證：cargo test -p speclink-cli及cargo test -p speclink-core --test it render_golden::通過；依序執行npm --prefix crates/adapters/speclink-node run build與npm --prefix crates/adapters/speclink-node test，健康JSON與單筆全文不變，只有既定缺損輸出差異。 <!-- speclink-task:tsk_01M5M8N2SHZ8BY32YDCFJPJRKK -->

## 4. 共用歷史元件先測試再呈現

- [x] 4.1 RED：於packages/ui/src/__tests__/changeDetailPage.test.tsx透過活動頁測試共用元件，為「品質關卡展開審查與驗證紀錄」加入三種readError、omitted、來源錯誤、選取後缺檔、重試、清除舊正文、時間offset與舊檔名排序測試；以07:00Z、06:00Z、05:00Z斷言倒序及同秒ID穩定序（D4、D6）。驗證：npm test -w packages/ui，新增逐筆錯誤與時間排序測試先紅，其他既有品質章測試維持綠。 <!-- speclink-task:tsk_01M5M8N2SH9G79Y0JV87KX1H3Q -->
- [x] 4.2 GREEN／REFACTOR：在packages/ui/src/checkRecords.ts、packages/ui/src/components/CheckRecords.tsx與packages/ui/src/i18n.tsx提供選填readError、kinds、共用展開控制項、逐筆狀態與正文重試；統一時間解析，body錯誤不殘留內容或永久骨架，adapter／change／generation／選取ID隔離晚回應；接回packages/ui/src/components/ChangeDetailPage.tsx（D4、D5）。驗證：4.1既有活動頁及共用元件相關測試轉綠，npm test -w packages/ui確認繁中／英文詞條、原結果不被錯誤狀態取代及無跨項目內容。 <!-- speclink-task:tsk_01M5M8N2SHACGCMDVZ2J4PNYQ3 -->

## 5. 封存品質關卡與Desktop組裝

- [x] 5.1 RED：在packages/ui/src/__tests__/archivedDetailPage.test.tsx、apps/desktop/src/__tests__/checkRecords.test.ts與apps/desktop/src/__tests__/App.test.tsx重現「文件區只顯示進行中的審查與驗證」及封存品質章無歷史入口；建立六筆各類紀錄，斷言兩站獨立展開、鍵盤、完整datedName與docTarget、carried工單保留、切專案晚回應隔離、adapter只查review／verify且安全摘要失敗不拖累（D4、D5、D6）。驗證：npm test -w packages/ui及npm test -w apps/desktop，封存入口與指定類別請求斷言先紅。 <!-- speclink-task:tsk_01M5M8N2SHP7DPNV559VGRSK6Z -->
- [x] 5.2 GREEN／REFACTOR：於packages/ui/src/components/ArchivedDetailPage.tsx宣告checkRecords並接上共用展開清單、唯讀正文、標題及文件動作，沿用StatusPill／TabsContent／Markdown／DocSkeleton與本頁目錄；apps/desktop/src/adapter/checkRecords.ts實作kinds選擇，apps/desktop/src/App.tsx依workspace adapter傳入，不讀無關安全摘要；保留無adapter的說明及carried工單條件節點（D4、D5）。驗證：5.1轉綠，npm test -w packages/ui及npm test -w apps/desktop通過；完整datedName與relativePath正確且歷史不在文件樹重複出現，無重跑、蓋章或寫入呼叫。 <!-- speclink-task:tsk_01M5M8N2SH7SV6TE6TX3CFB9EQ -->

## 6. 邊界複核與交付驗證

- [x] 6.1 對D1至D6套用audit精簡三角色清單並排查同類位置：搜尋history::index、history::record、read_document與.ok().flatten()、Promise.all的歷史／安全收據請求；核對crates/engine/speclink-core/src/quality/history.rs、crates/engine/speclink-fs/src/lib.rs、crates/host/speclink-host/src/history.rs、apps/desktop/src/adapter/checkRecords.ts、packages/ui/src/components/CheckRecords.tsx及兩個詳情頁，逐一記錄一併修正或維持嚴格的理由。驗證：無新增任意path讀取、無吞索引IO、無單筆失敗偽裝可信內容、無寫入／驗收改用browse；以既有測試驗證比對，內容記於此變更的證據紀錄，不啟動額外安全標準掃描。 <!-- speclink-task:tsk_01M5M8N2SHAHPMPBAAK9YFHNKR -->
- [x] 6.2 完成交付回歸與文件一致性（D6；crates/engine/speclink-core、crates/engine/speclink-fs、crates/protocol/speclink-protocol、crates/host/speclink-host、crates/host/speclink-server、crates/adapters/speclink-cli、crates/adapters/speclink-node、packages/ui、apps/desktop、scripts/及openspec/changes/archived-quality-history-access）：確認各層受影響測試的通過紀錄、健康golden與Local／Remote輸出對照、六筆各類紀錄完整保留及omitted不變、讀取零文件效果、嚴格守門拒絕缺損；若本輪修正產生新失敗則重跑對應Cargo package／workspace，避免無理由重複完整測試。最後驗證必執行node --test scripts/*.test.mjs scripts/*/*.test.mjs、speclink analyze archived-quality-history-access --json與speclink validate archived-quality-history-access，修正本變更造成的錯誤；不在本機要求npm run test:all，不執行提交、蓋章或封存。 <!-- speclink-task:tsk_01M5M8N2SHSM37CGEDAS31B5B4 -->


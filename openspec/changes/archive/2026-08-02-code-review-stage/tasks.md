## 1. 引擎品質站機制（speclink-core）

- [x] 1.1 撰寫工單生命週期測試（規格「審查工單的建立與追加」「審查工單的讀取」「放棄審查」）：`review add-round` 首輪建檔／追加輪次 append-only（Round 1 位元級不變）／change 不存在拒絕／缺 `**Scope**:` 拒絕；`review show` 解析 rounds 與 lastRound；`review discard` 刪工單、無工單非零。以 `#[cfg(test)]` 寫於 crates/speclink-core/src/review.rs，經 `&dyn Store` 測試替身驗證本地與 store 路徑同行為。驗證：`cargo test -p speclink-core review` 全紅（尚未實作） <!-- speclink-task:tsk_01KYWDRYHJX2ZBJV4SSDFWAF4C -->
- [x] 1.2 依 design「D1 品質站機制落點」（常數參數化、不建 trait）與「D2 工單文件與儲存」（動詞驅動 markdown 文件）實作工單動詞核心：review.md 固定骨架（`## Round N`＋`**Scope**:`＋分級 findings 行）的產生、驗證與解析；站別差異（檔名、欄位前綴、狀態詞）集中為常數組。檔案 crates/speclink-core/src/review.rs、crates/speclink-core/src/lib.rs（module 註冊）。驗證：1.1 測試全綠 <!-- speclink-task:tsk_01KYWDRYHJQ3PZSCEHQDZQX8BK -->
- [x] 1.3 撰寫蓋章與雙錨測試（規格「蓋章守門與蓋章效果」「內容指紋錨與失效判定」）：任務未全完成拒絕／末輪有 findings 拒絕與 `--accept` 放行／成功時五欄位寫入且工單刪除為同一原子寫入（無「章在工單也在」中間態）；指紋計算（路徑 `\`→`/` 正規化、內容 CRLF→LF 後 SHA-256）；失效純函式：全符 fresh、任一檔內容變或缺檔 stale、行尾差異不觸發。檔案 crates/speclink-core/src/review.rs、crates/speclink-core/src/model.rs。驗證：`cargo test -p speclink-core` 新測試全紅 <!-- speclink-task:tsk_01KYWDRYHJ3WH34H2R0NTWMPP3 -->
- [x] 1.4 依 design「D3 章與雙錨」（stamp 單一 unit_of_work、失效判定為讀取端純函式）實作 stamp／discard 與失效純函式：ChangeMeta 增列 `reviewed_at`／`reviewed_by`／`reviewed_with`／`reviewed_tasks_total`／`reviewed_scope`（全部 serde default，缺席讀作未審查，既有 .openspec.yaml 可讀）；以既有 sha2 依賴計算指紋。檔案 crates/speclink-core/src/model.rs、crates/speclink-core/src/review.rs。驗證：1.3 測試全綠 <!-- speclink-task:tsk_01KYWDRYHJKJP93FNBY1N0A2H8 -->
- [x] 1.5 依 design「D5 archive 未結工單守門」撰寫並實作規格「封存的未結工單守門」：工單存在時預設拒絕且 stderr 同時含 stamp／discard／`--carry-review` 三處置；`--carry-review` 時工單隨目錄搬入封存區；無工單時輸出位元級不變（回歸斷言）。檔案 crates/speclink-core/src/archive.rs。驗證：`cargo test -p speclink-core archive` 全綠 <!-- speclink-task:tsk_01KYWDRYHJ4ED5ZKWFS0GZRJ0E -->
- [x] 1.6 依規格「CLI 清單輸出的相容性釘住」延伸 parity pin 測試：meta 帶全套 reviewed 欄位的 change 與不帶者，`list --json` 項目欄位集合同形。檔案 crates/speclink-core/src/listing.rs。驗證：`cargo test -p speclink-core listing` 全綠 <!-- speclink-task:tsk_01KYWDRYHJCA1S8NSAZ17CM0GK -->

## 2. CLI 子命令（speclink-cli）

- [x] 2.1 撰寫 CLI 整合測試（design「D4 CLI 子命令面」）：`speclink review add-round --stdin`／`show`／`show --json`（斷言 payload 含 `change`、`rounds[].index`、`rounds[].scope`、`rounds[].findings[].severity`、`lastRound`——camelCase 與型別）／`stamp [--accept]`／`discard` 的 exit code 與 stdout/stderr 去向；`archive` 三處置訊息與 `--carry-review`。檔案 crates/speclink-cli/tests/（比照既有整合測試檔命名）。驗證：`cargo test -p speclink-cli` 全紅 <!-- speclink-task:tsk_01KYWDRYHJ26R3A1FWV3SV0TE7 -->
- [x] 2.2 註冊 `review` 子命令家族與 `archive --carry-review` 旗標，委派 speclink-core 動詞；`--no-color` 下無 ANSI。檔案 crates/speclink-cli/src/（子命令入口比照 discuss 子命令的分檔慣例）。驗證：2.1 測試全綠；`speclink review show <demo> --json` 手動確認 payload <!-- speclink-task:tsk_01KYWDRYHJADK5PPDZY14BAH4V -->

## 3. desktop 協定與 desktop-core

- [x] 3.1 撰寫 query 增列測試（規格「變更清單的審查狀態欄位」「已封存清單的審查結局欄位」）：fixture 覆蓋 active 四態（none／inReview／reviewed／reviewedStale——後者以修改 scope 檔內容觸發）與 archived 三態（none／reviewed／reviewedNotPassed）；斷言 `reviewStatus`、`reviewedAt`、`reviewedBy` 欄位與 camelCase；斷言既有 CLI 形狀欄位不動。檔案 apps/desktop/core/src/query.rs。驗證：`cargo test -p speclink-desktop-core` 新測試全紅 <!-- speclink-task:tsk_01KYWDRYHJ5WDG51HNWQGDQTWG -->
- [x] 3.2 依 design「D6 desktop 資料流與 UI」實作 desktop-core 增列：呼叫 speclink-core 失效純函式重算凍結度（不依賴 Tauri）；已封存側讀化石工單判定結局、不重算凍結度；Tauri command 維持單行委派。檔案 apps/desktop/core/src/query.rs、apps/desktop/src-tauri/src/（委派處）。驗證：3.1 全綠；`npm test -w apps/desktop` 通過 <!-- speclink-task:tsk_01KYWDRYHJ40E5YPRRRNMG9JGP -->

## 4. desktop UI（packages/ui）

- [x] 4.1 撰寫 UI 測試（規格「看板卡片的審查標示」「詳情抽屜的審查資訊列」「已封存側的審查標示」「封存入口的未結工單三選項」）：ChangeCard 依 reviewStatus 四態渲染行內小章與 tooltip 詞（none 無元素）；RichDetailDrawer 資訊列（reviewed 含時間與審查者、inReview 僅狀態詞、none 不渲染）；ArchivedList／ArchivedDrawer 的 reviewedNotPassed 標示；封存入口 inReview 時彈三選項且未選擇不封存。檔案 packages/ui/src/__tests__/。驗證：`npm test -w packages/ui` 新測試全紅 <!-- speclink-task:tsk_01KYWDRYHJHPAWQ252WW2ZC3ZK -->
- [x] 4.2 實作規格「看板卡片的審查標示」與「詳情抽屜的審查資訊列」：adapter 型別增列 reviewStatus 等欄位；行內小章沿用 lucide 既有 icon 家族＋Tooltip，維持極簡卡片不加文字列。檔案 packages/ui/src/adapter.ts、packages/ui/src/components/ChangeCard.tsx、packages/ui/src/components/RichDetailDrawer.tsx。驗證：對應測試轉綠 <!-- speclink-task:tsk_01KYWDRYHJYA71YP24SJQG15VP -->
- [x] 4.3 實作規格「已封存側的審查標示」與「封存入口的未結工單三選項」：三選項對話框（前往蓋章導引／放棄審查／照樣帶走並警示永久標示）。檔案 packages/ui/src/components/ArchivedList.tsx、packages/ui/src/components/ArchivedDrawer.tsx、封存入口所在元件（KanbanBoard 或 DetailDrawer，依現行封存動作落點）。驗證：4.1 測試全綠 <!-- speclink-task:tsk_01KYWDRYHJGPHQJZHXSHRRHNHR -->
- [x] 4.4 增列 i18n 詞條（卡片與抽屜標示的 tw／en 文案）：tw 用正典詞（審查中／已審查／已審查·其後有變動／曾審查未通過）、en 對應詞條。檔案 packages/ui/src/i18n.tsx、apps/desktop/src/i18n/messages.ts。驗證：`npm test -w packages/ui` 與 `npm test -w apps/desktop` 全綠 <!-- speclink-task:tsk_01KYWDRYHJP2DAZGBBV4J2QYRK -->

## 5. skill 與生成文件

- [x] 5.1 依 design「D7 skill 與生成文件」實作規格「審查技能的生成與正典化」「審查流程的技能行為」「Standards 軸的 smell baseline」「審查後的迴圈與收尾」：skills.rs 新增 /speclink-review 正典模板（claude 與 codex 雙工具）——主線 orchestrator 全流程（守門自檢→定界範圍→artifacts 脈絡→平行兩軸 read-only sub-agent 各 400 字內→並列呈現→add-round→三選項迴圈→空輪 stamp；codex 變體以純文字詢問）；Standards sub-agent 指示內嵌 design「D7a Smell baseline 正典文字」全段逐字（引言＋兩條約束規則＋12 smells 逐項＋出處註記，不改寫不翻譯）；instructions.rs 更新 workflow 行為 `(review? ∥ verify?)` 與技能清單。檔案 crates/speclink-core/src/skills.rs、crates/speclink-core/src/instructions.rs。驗證：`cargo test -p speclink-core` 之 render／golden 測試指出待再生；模板字串含 12 條 smells 專有名詞與兩條約束規則 <!-- speclink-task:tsk_01KYWDRYHJDY0CMWJFB5JTW4W5 -->
- [x] 5.2 乾淨樹再生 golden 並確認：`crates/speclink-core/tests/golden` 更新、`cargo test --workspace` 全綠；於本 repo 執行 `speclink update` 落地 .claude/skills/speclink-review/ 與更新後的 CLAUDE.md 注入區塊。驗證：golden 測試通過、生成檔內容與模板一致，claude 與 codex 兩份生成檔皆含 D7a 全段（規格「Standards 軸的 smell baseline」Example：12 專有名詞逐一命中且各帶「→」修法句） <!-- speclink-task:tsk_01KYWDRYHJCEYP6MDQGHMHXF02 -->
- [x] 5.3 更新 README.md 與 README.en.md：workflow 圖加入並行品質站、審查站一節（何時用、與 verify 的分工表）。驗證：內容審閱——分工表與討論 code-review-stage 結論一致 <!-- speclink-task:tsk_01KYWDRYHJ2J921M2VK40SXMF5 -->

## 6. 端到端驗證

- [x] 6.1 E2E 走查（含規格「remote 模式下的動詞行為」）：以 demo change（`speclink demo`）完成任務後跑完整審查迴圈——add-round（帶 findings）→ show --json 取工單 → stamp 被拒（有 findings）→ `--accept` 蓋章 → 卡片顯示已審查 → 修改 scope 檔 → 卡片轉「已審查·其後有變動」；另走 archive 三處置各一遍；remote workspace 以 dev harness 驗證 review 動詞經 store 文件管道寫入與離線非零 exit。驗證：`npm run test:all` 全綠、`speclink validate code-review-stage` 通過 <!-- speclink-task:tsk_01KYWDRYHJY8Y8X6YEJRFM1J10 -->

## 7. 審查裁量層與收斂（skill 模板）

- [x] 7.1 依 design「D7b 裁量層與收斂機制」實作規格「審查結果的裁量分類」「修復迴圈的驗證門」「已接受事項的續輪前饋」：更新 skills.rs 的 /speclink-review 模板（claude 與 codex 雙工具）——兩軸呈現後逐筆標必修／可裁並附一行裁量理由、三選項詢問帶推薦選項（有必修推薦修正並列必修清單、僅剩可裁推薦 `--accept`）、修正完成後下一輪派出前執行專案完整建置與測試並要求全綠、已接受事項由主線原樣帶入續輪記錄且續輪 sub-agent 指示附不重報清單。檔案 crates/speclink-core/src/skills.rs。驗證：`cargo test -p speclink-core` 之 render／golden 測試指出待再生；模板字串含必修／可裁分類判準、驗證門與不重報指示的關鍵詞 <!-- speclink-task:tsk_01KYYV29KSEV94788SBJE0Q5W8 -->
- [x] 7.2 乾淨樹再生 golden 並落地（8.1 完成後執行，一次涵蓋 7.1 與 8.1 兩批模板變更）：`crates/speclink-core/tests/golden` 更新、`cargo test --workspace` 全綠；執行 `speclink update` 更新 claude 與 codex 兩工具的 speclink-review 生成檔。驗證：golden 測試通過；兩份生成檔皆含裁量分類、驗證門與續輪不重報三段內容（design 可驗證行為 10 的關鍵詞檢核）、locale 綁定三處內容（design 可驗證行為 11 的關鍵詞檢核），且 D7a smell baseline 全段仍逐字在場 <!-- speclink-task:tsk_01KYYXAN9Z4TPSTP9GAZFRHR27 -->

## 8. 審查產出語言綁定（skill 模板）

- [x] 8.1 依 design「D7c 產出語言與 locale 綁定」實作規格「審查產出的語言綁定」：先於 crates/speclink-core 既有 render／模板測試補 locale 綁定關鍵詞斷言（紅），再更新 skills.rs 的 /speclink-review 模板（claude 與 codex 雙工具）三處——(1) 守門自檢保留 payload 時聲明 locale 適用整條產出鏈（報告、呈現、工單記錄）；(2) 兩軸 sub-agent 指示攜帶解析後 locale 並要求 finding 描述以該語言撰寫，severity 標籤、`Standards:`／`Correctness:` 前綴與檔案路徑留英文；(3) 並列呈現與 add-round 記錄與 sub-agent 產出同語言、主線不翻譯、locale 未設定則全英文（同步移除「render verbatim」與「write in the resolved locale」的矛盾措辭），綁定句式仿 verify 模板既有寫法。檔案 crates/speclink-core/src/skills.rs。驗證：新增關鍵詞斷言轉綠；`cargo test -p speclink-core` 之 golden 測試指出待再生（於 7.2 收攏） <!-- speclink-task:tsk_01KYYW5JEHKCYQ4NCRTRY1JE87 -->

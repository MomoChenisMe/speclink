## Context

task done 的 evidence 鏈今天只在本地模式成立：crates/speclink-core/src/tasks.rs 的 complete() 透過 host workspace 探 git 狀態取得候選髒檔、過濾歸屬後直接以檔案 I/O 寫入 change 目錄的 .evidence.json——這段寫入繞過 Engine 的 Store seam。server 模式下 host workspace 缺席，整段被跳過；同時 crates/speclink-server/src/routes.rs 的 task_done 以 Json(_req) 丟棄 wire 上已送達的 TaskDoneRequest.touchedFiles，Command::TaskDone 也沒有欄位可帶。結果是 server 端沒有任何 evidence 事實可查（phase2_chain.rs #[ignore] 紅測試釘著）。host 的 drift 衍生（crates/speclink-host/src/drift.rs）已預留 evidence_summary 欄位，remote 模式下恆空。

相關正典：verify-evidence（evidence 語意與 .evidence.json 格式 v2）、teamstore-contract（文件定址與 UoW 原子 commit）、server-verb-api（動詞端點面）、host-runtime-binding-policy（Host 解析事實、Engine 只消費）。

## Goals / Non-Goals

**Goals:**

- 遠端 task done 的 touchedFiles 從 wire 一路落到 store，成為可查事實（文件面＋事件面）。
- evidence 寫入收進 Store seam，本地與 store 兩個 supplier 共用同一套 Engine 邏輯（歸屬過濾不分岔）。
- 本地模式可觀察行為零變化：.evidence.json 位置、格式（v2）、寫入時機皆不動。
- 併刀修正三份使用者文件與一行過期註解的事實漂移。

**Non-Goals:**

- Desktop 遠端勾任務的本機 git 探測（checkout 綁定下的 touched 收集）——本刀 desktop 遠端勾任務不送 touchedFiles，沿「無新髒檔不新增記錄」語意。
- evidence 回填工具與離線佇列。
- legacy remote REST v1 旁路的 evidence 支援。
- Desktop 遠端工作區剩餘小縫的盤點（討論結論第三步）。

## Decisions

**決策一：touched 候選由 Host 注入 Command，本地探測留在原位。** Command::TaskDone 增 `touched_files: Option<Vec<String>>`（引擎內部型別，非 argv 面）。`None` ＝「Host 未注入，沿本地 host workspace 的 git 探測」（本地路徑零改動）；`Some(files)` ＝「Host 已在邊界解析好候選」（server 路由把 TaskDoneRequest.touched_files 原樣填入）。歸屬過濾（僅未被先前任務認領的新髒檔）仍在 crates/speclink-core/src/tasks.rs 單點實作，兩模式同一套——符合「同一對外契約單一實作落點」與 host-runtime-binding-policy。
替代方案：把本地 git 探測也上提到 CLI／Host 層（全面 Host 注入）——更純粹但churn本地路徑且無行為收益，落選；在 bridge 偽造一個「wire 餵入的 workspace」——把網路資料假裝成檔案系統，語意欺騙，落選。

**決策二：evidence 走 Store seam，TeamStore 側新增專屬文件變體。** crates/speclink-core/src/store.rs 的 Store trait 增 evidence 讀寫面（read_evidence／write_evidence，內容為 .evidence.json 的既有 v2 序列化文字）；本地 fs supplier 映射到 change 目錄 .evidence.json（含既有 legacy 路徑回退讀取，行為不變）；TeamStore bridge（crates/speclink-host/src/bridge.rs）映射到新變體 `DocumentId::ChangeEvidence { change }`（crates/speclink-store/src/types.rs），staged 進同一 Unit of Work。封存側不加新變體：run_archive 經 seam 讀 evidence 後以既有 `ArchivedChange { doc: ".evidence.json" }` 落檔，與本地「evidence 隨 change 目錄封存」對稱；discard 隨 change 文件一併刪除。
替代方案：借用 `ChangeArtifact { artifact: ".evidence.json" }`——省一個變體，但 artifact 名是 schema 定義的對外語彙，evidence 不是 artifact，會污染 artifact 列舉面（UI 的 artifact 清單、instructions 依賴解析），落選；三 driver 各開專屬資料表——文件模型已足夠承載，過度工程，落選。

**決策三：查詢面＝事件面＋文件面兩條，不動 change read 組合欄位。** (1) 事件面：DomainEvent::TaskCompleted 增 touchedFiles，outbox 的 task-completed payload 隨之攜帶（additive）。(2) 文件面：server 增唯讀端點 GET /changes/{name}/evidence 回傳 evidence 記錄（--json 同形之 camelCase，viewer 以上可讀；記錄缺席回空集合而非 404——缺席是正常狀態）。host drift 的 evidence 事實改經 seam 讀取：fs 模式的 evidence_summary 與 remote 模式的 Environment touched 事實同一來源——server 把 store 保存的 evidence 記錄隨 drift 回應下行，remote 的工作區面計算據此不再恆空。phase2_chain.rs 紅測試去 #[ignore]，斷言指向 outbox payload 與新端點。
替代方案：把 evidence 塞進單一 change 讀取回應的組合欄位——server-verb-api 的組合欄位是 show 所需，evidence 非 show 語彙且會膨脹高頻讀取，落選；只留 outbox 事件面——事件是流不是狀態，無法回答「這個 change 目前的 evidence」，落選。

**決策四：寫入順序與失敗語意。** store 模式：tasks.md 勾選、evidence 文件、task-completed 事件三者在同一 UoW 內原子 commit——任一步失敗整筆回退，無半套狀態。本地模式：沿既有順序（tasks.md 先、.evidence.json 後），兩步之間失敗的可觀察狀態為「任務已勾、無 evidence entry」——此半套為既有語意（等同無新髒檔），本刀明載並接受，不改。

**決策五：文件漂移修正的落點。** docs/product-status 兩列改判（Desktop Remote Workspace 依實況重寫證據與限制；remote task evidence 落地後改 Available）、remote-getting-started 第 6 節改為描述已可用的遠端開啟流程（skip 與 folder 兩模式）、roadmap 遠端協作線改寫「目前到哪」與「可觀察的下一步」、apps/desktop/src/session.ts 頂部註解改為指向現行建構路徑。中英兩語每處同步改，查核日期更新。

## Implementation Contract

**行為（可觀察）：**

- 遠端模式 CLI 於髒 git checkout 執行 speclink task done：server 端該 change 的 evidence 記錄含該任務 entry（taskId、actor、repo、headCommit 由 wire 攜入或 host 解析、touchedFiles、recordedAt），GET /changes/{name}/evidence 可讀回；outbox 的 task-completed payload 含 touchedFiles。
- 同 change 於 store 模式封存後，封存文件集含 .evidence.json，內容與封存前一致。
- 遠端無新髒檔（或 desktop 遠端勾任務）：不新增 evidence entry，端點回空集合，與本地語意一致。
- 本地模式全流程（task done／undone、drift、commit 歸屬、archive）行為與檔案佈局與本刀之前逐位元一致。
- task done 的 argv、人眼輸出、--json 欄位皆不變。

**驗證錨點：**

- crates/speclink-server/tests/it/phase2_chain.rs：task_done_with_touched_files_leaves_queryable_evidence_on_the_server 去 #[ignore] 轉綠（outbox 面＋端點面斷言）。
- crates/speclink-store 的 conformance suite 增 ChangeEvidence 文件的建立、讀回、UoW 原子性與隨封存搬移案例；三 driver（sqlite、serverfs、postgres）同套通過。
- crates/speclink-core 既有 task／evidence 單元測試全綠（本地行為凍結的回歸對照）。
- crates/speclink-cli/tests/it/remote_verb_parity.rs 全綠（模式分岔宣告未動）。

## Risks

- **回歸對照**：本地 .evidence.json 為 golden 級行為（drift、commit 歸屬、archive trace 皆消費）——decision 一的 None 分支必須逐位元保留現行寫入；CLI 整合測試與 core 單元測試是防線。
- **跨平台**：touchedFiles 為邏輯路徑，依既有正典一律正斜線；Windows 端 git 探測輸出的反斜線在 Host 邊界正規化（本地路徑既有行為，不新增轉換點）。serverfs driver 的檔案鎖語意沿 teamstore-contract 既有要求，不因新文件變體改變。
- **driver 波及面**：DocumentId 新變體需三 driver 的 locator 映射與快照列舉同步；漏一處由 conformance suite 擋下（契約由測試釘死，不靠文件）。
- **文件雙語同步**：product-status／remote-getting-started／roadmap 中英各一份，漏改一側即製造新漂移；收尾以 git status 盤點六檔皆動過。

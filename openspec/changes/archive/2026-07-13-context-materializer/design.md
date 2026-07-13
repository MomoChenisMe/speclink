## Context

remote 模式的 instructions 目前輸出 contextFiles（BTreeMap，crates/speclink-core/src/instructions.rs）指向 workspace 內的 openspec 路徑，但遠端正典不在本機——apply／verify skill 讀不到文件（路線圖 §3.8 的斷鏈）。workspace 的 .speclink/ 工作目錄已存在（touched 記錄等）。protocol-typed-client 刀已定 Context snapshot 的 DTO 形狀（ContextSnapshot／ContextDocument：snapshotId、policyRevision、逐文件 digest 與 revision）與 handshake capabilities。藍圖 §7.2 給定投影佈局（manifest.json、INDEX.md、openspec 鏡像）、八條 Context 規則與「不放 .git/」的理由；§7.3 給依流程縮小表。技能資產有三處同步鐵律（assets、repo 實例、render golden——golden 必須乾淨樹再生，dirty 樹再生曾使 main 長紅）。

## Goals / Non-Goals

**Goals:**

- speclink-host 的 materializer：staging → atomic switch、manifest 與逐文件 digest、唯讀屬性、gitignore 保證、stale 標記、依流程縮小。
- command 前 manifest digest 驗證：被修改的投影 fail closed。
- remote instructions 的 contextFiles 指向投影；apply／verify skill 明確讀投影、禁止寫回。
- snapshot 來源介面化：本刀以測試替身驗證，Phase 2 接真實 Context API。
- 本地 fs 模式零變更。

**Non-Goals:**

- Server 端點、HTTP 傳輸、事件驅動 stale、gitdir 位置選項、無 checkout 投影形式、增量更新、Desktop 呈現。

## Decisions

### 決策一：materializer 落 speclink-host，snapshot 來源以 trait 注入

投影是本機檔案系統操作與應用層編排（屬 host/client 職責，沿 WorkspaceFacts 蒐集器先例），落 crates/speclink-host/src/projection.rs。資料來源定義為 snapshot provider trait（給定 scope 與流程參數——即 protocol 的 ContextSnapshotRequest，回 protocol 的 ContextSnapshot：manifest 資訊與文件內容集合）；本刀提供測試替身（以本地 Store 建快照）與 stub 形狀對測，HTTP 實作屬 Phase 2。替代方案：直接在 CLI remote 層寫檔——Desktop 與 Node host 未來各自重寫投影規則，違反單一實作，被拒。

### 決策二：staging 目錄加原子 rename 切換

materialize 流程：在投影旁的 staging 目錄產生完整 snapshot（manifest、INDEX、鏡像文件、digest 全算好）→ 驗證完整性 → 原子 rename 切換（舊投影先改名再刪，或 rename 交換），任何失敗時現有投影不受影響。跨平台注意 Windows 對開啟中檔案的 rename 限制：切換失敗回明確錯誤並保留 staging 供重試，不做半套覆寫。替代方案：逐檔覆寫——Agent 閱讀中文件被偷換，藍圖明文禁止，被拒。

### 決策三：完整性驗證 fail closed、stale 為顯式標記

manifest 記逐文件 digest 與 snapshot ID／revisions；host 提供 verify_projection：任一文件 digest 不符或 manifest 缺失即回「投影已被修改或不完整」的拒絕（要求 refresh），不把修改解讀為遠端寫入意圖。stale 標記為投影根下的 marker 檔（不動文件內容）；讀取端見 marker 即提示 refresh。唯讀屬性盡力而為（平台允許時設定），完整性以 digest 為準、不依賴唯讀成功。替代方案：以檔案 mtime 偵測——編輯器 touch 與時鐘偏移誤報漏報皆有，digest 才可靠，被拒。

### 決策四：依流程縮小 context 為挑選規則、預設全量

materialize 接受流程參數，對齊藍圖 §7.3：discuss（config、LANGUAGE、canonical specs 索引）、propose（discussion、相關 canonical specs、schema/template）、apply（proposal、design、tasks、delta specs、base specs）、verify（apply 集合加最新 tasks 與驗證規則）、archive（delta specs、canonical base、tasks、revision）；未給流程參數時全量。挑選規則落 materializer（單一實作），不由 skill 各自定義。替代方案：一律全量——大型專案 materialize 延遲與模型 context 浪費（藍圖 §15.2 P1 已點名），被拒。

### 決策五：remote instructions 與 skill 的接線為最小文案變更

instructions 的 remote 變體把 contextFiles 值改指投影下的對應路徑（key 與集合邏輯不變）；apply 與 verify skill 增補 remote 段落：讀投影路徑、投影唯讀、任何規格修改必須經 speclink 動詞（不得直接編輯投影）。技能變更遵守三處同步：assets 修改後同步 repo 技能實例，render golden 於乾淨樹 UPDATE_GOLDEN=1 再生並逐 diff 審視。本地模式 instructions 零變更（twin 對照中 remote instructions 的 contextFiles 值為刻意變更、同步更新期望值）。替代方案：skill 大改寫成 remote 專用變體——雙份 skill 語意漂移風險，藍圖 §16 明定不為各工具維護多套流程語意，被拒。

### 決策六：gitignore 保證沿 init／update 既有管理

投影位於 .speclink/ 工作目錄下（該目錄的 gitignore 管理已存在於 init／update 流程）；materializer 於寫入前驗證 gitignore 涵蓋，未涵蓋時補寫並警告（不靜默）。替代方案：假設一定已 gitignore——舊 workspace 或手動刪過 .gitignore 的情境會把投影提交進 repo，成為第二正典，被拒。

## Implementation Contract

- **行為**：remote 模式下執行 materialize（由 remote 動詞流程或顯式 refresh 觸發）後，workspace 的 .speclink/context/ 含 manifest.json、INDEX.md 與 openspec 鏡像，文件盡力唯讀且被 gitignore；Agent 修改投影文件後任何 command 前驗證即 fail closed 要求 refresh；stale 標記存在時讀取端提示 refresh；依流程參數 materialize 只含該流程的預設集合；本地 fs 模式無任何投影且行為不變。
- **介面／資料形狀**：snapshot provider trait（protocol 的 ContextSnapshotRequest → protocol 的 ContextSnapshot）；manifest.json 含 snapshotId、policyRevision（有值時）、逐文件 digest 與 revision（camelCase，對齊 ContextSnapshot 既有欄位）；stale marker 為固定檔名的 marker 檔；materialize／verify_projection／mark_stale 為 host 公開 API。
- **失敗模式**：staging 失敗不影響現有投影；切換失敗保留 staging 並回明確錯誤；digest 不符 fail closed 要求 refresh；gitignore 未涵蓋時補寫並警告。
- **驗收**：cargo test -p speclink-host 全綠（staging／switch／digest／stale／流程縮小／gitignore 情境）；cargo test -p speclink-core 與 render golden 乾淨樹再生後全綠；twin 對照中 remote instructions 的 contextFiles 新期望值通過、其餘逐位元不變；npm run test:all 全綠。
- **範圍邊界**：in scope——projection.rs、provider trait 與測試替身、instructions remote 指向、apply／verify skill remote 段落、三處同步；out of scope——HTTP 來源、事件 stale、gitdir 選項、無 checkout 形式、增量更新、本地模式任何變更。

## Risks / Trade-offs

- [golden 於 dirty 樹再生把未提交狀態烙進快照] → 鐵律寫進 tasks：先提交技能與 instructions 變更、乾淨樹上 UPDATE_GOLDEN=1 再生、逐 diff 審視後才提交 golden。
- [Windows rename 語意與唯讀屬性差異] → 切換失敗保留 staging 與明確錯誤；唯讀盡力而為、完整性以 digest 為準；CI 三平台跑 host 測試。
- [投影路徑滲入非 remote 輸出] → twin 與 parity 對照全綠為 gate；本地 instructions 測試斷言零投影路徑。
- [provider trait 形狀與 Phase 2 真實 Context API 不合] → trait 輸入輸出即 protocol 刀的 Context DTO，wire 形狀已定；不合處屬 server 實作責任。
- [流程縮小集合挑錯文件造成 skill 斷檔] → 每流程集合各有測試斷言含必要文件；未給流程參數的全量路徑作 fallback。

## Migration Plan

新增檔面（投影目錄）與 remote instructions 值變更：本地模式零遷移；remote 使用者首次執行動詞時自動 materialize，無手動步驟。回滾還原 commit 後，殘留投影目錄為 gitignored 可刪檔案、不影響任何行為。

## Open Questions

（無）

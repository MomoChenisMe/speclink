## Context

引擎端分解已定案（drift-client-server-split 刀）：compute_spec_drift(store, change) 純函式回 SpecDriftReport；collect_workspace_facts（host，讀本機 git/檔案為 WorkspaceFacts 三值語意）＋ compute_workspace_drift 回 WorkspaceDriftReport；merge_drift_reports 是唯一合併器回 CombinedDriftReport；DriftBundle（host）攜固定 basis。fs 模式的 drift 動詞經此路徑輸出，人眼與 --json 形狀是凍結對照的權威。remote 攔截層（crates/speclink-cli/src/remote_commands.rs）對其他動詞的模式是 argv → typed client → 與 fs 相同的渲染；drift 動詞目前完全未被攔截，remote 模式不可用。server 已有 change-scoped 路由骨架、單一 store snapshot 讀取（context-api 刀）與 bearer/binding 前置。

## Goals / Non-Goals

**Goals:**

- remote drift 動詞可用且輸出與 fs 模式逐位元同形：server 供規格面、client 供工作區面、merge_drift_reports 單點合併——不出現第二個合併實作。
- 無 checkout 時誠實：工作區面標示不可得（unavailable≠clean），規格面照常回報。
- drift 維持診斷性質，不新增任何 gate。

**Non-Goals:**

- 不動引擎的 drift 計算與合併邏輯（純接線刀）；不動 fs 模式輸出。
- 不做 drift 結果的持久化、歷史比較或訂閱。
- 不做 VerifyBundle/stale evidence 的 wire 化，也不補 server 端的 evidence 記錄（remote task done 目前丟棄 touched_files——既有缺口，另刀處理）。
- 不動 twin harness 既有 8 情境（drift 的端到端覆蓋屬 phase2-e2e-chain 刀的劇本）。

## Decisions

### 決策 1：wire 只載 server 由 snapshot 可知者，工作區面永不上行

protocol 的 drift DTO 只定義 server 由一個 store snapshot 可知的部分：`SpecDriftReport` 的 wire 形狀（規格面維度與規格假設）、basis digests，以及 client 工作區面計算所需的 **store 面輸入**（created metadata 與 design/tasks 內容）。WorkspaceFacts 與 WorkspaceDriftReport 不進 wire——工作區事實屬本機，server 不 shell-out git 的鐵律（roadmap §6）由型別層面直接封死：server 想撒謊也沒有欄位可填。

broken anchors 依此鐵律落在工作區面，不在 wire 上：它是引擎 `WorkspaceDriftReport` 的欄位，由 `compute_workspace_drift` 從 `WorkspaceFacts` 的 `git grep HEAD`／`tracked_docs`／path stat 算出。server 要算它就得跑 git，正是本決策封死的事。

store 面輸入下行不牴觸此鐵律：created 與 design/tasks 是 **store 事實**，server 從 snapshot 直接讀得，方向是 server→client。引擎的 `collect_workspace_facts` 與 `compute_workspace_drift` 都吃 `&dyn Store`，實際只讀這個 change 的 design.md／tasks.md（內容與存在性）與 `meta.created`；remote 模式本機沒有 `openspec/`，這些只能由 server 供給。設計上區別「缺席」與「空內容」——`artifact_exists` 驅動 Structure 維度的「no design」分支，兩者混同會讓報告說謊。

`evidenceSummary` **不上 wire**：server 從未存過它。remote 的 `task_done` 路由把 client 送上的 touched_files 直接丟棄（`speclink-server/src/routes.rs` 的 `Json(_req)`），server 端沒有 evidence 記錄可給。client 讀自己本機的 `TouchedRecord`——與 fs 模式同一來源，也是 remote 流程下唯一誠實的答案。影響有限：Environment 維度是 display-only（score 0、不計入總分），其 relevant 集合另一半（task 路徑引用）從 tasks.md 就取得。（design 原 Non-Goals 稱「evidence 上行已在 task done 路徑」與程式碼不符——那是既有的 server 缺口，本刀不處理。）

### 決策 2：host 開專用查詢入口，端點只是它的傳輸殼

drift 的規格面計算不從 host 外洩橋接視圖，而是由 host 的 drift 模組開一個專用查詢入口：

```rust
pub struct SpecDriftView {
    pub spec: SpecDriftReport,
    pub basis: BasisDigests,
    pub created: Option<String>,   // Time 維度輸入
    pub design: Option<String>,    // 錨點來源；None = 缺席（≠ 空字串）
    pub tasks: Option<String>,     // 任務路徑引用來源；None = 缺席
}

pub fn spec_drift(store: &dyn TeamStore, scope: &Scope, change: &str)
    -> Result<SpecDriftView, BridgeError>
```

入口內部取一次 store snapshot（與 context-api 同模式）materialize 私有橋接視圖、跑 `compute_spec_drift`、由**同一個 snapshot** 算 basis digests 與讀出 store 面輸入，一起回傳。server 端點只做 binding 裁決 → 呼叫入口 → 映射成 wire DTO。未知 change 回 404 not_found；store 失聯 503。計算是唯讀查詢，不取寫鎖、不產生事件。

不開泛用的 `bridge::read_view(store, &scope) -> impl Store`，理由有三：

1. **裸視圖外洩是慢性架構債**——adapter 拿到它就能繞過 host 跑任意引擎函式，之後每個新查詢都會想走這條近路，「host 是唯一組合點」被逐步稀釋。正是 roadmap §6「不讓各入口自己重組 lifecycle」要防的形狀。
2. **寫入面是 footgun**——core 的 `Store` trait 含寫入方法，橋接視圖的寫入是捕捉進 UoW 的；視圖外洩後 adapter 誤呼寫入方法會變成靜默被丟棄的寫入，比 panic 還難查。
3. **basis 綁死在組合點**——專用入口讓「規格面報告與 basis digests 出自同一 snapshot」由 host 保證，server 忘不了帶 basis，也不可能對不同 snapshot 各算一半。

### 決策 3：合併發生在 client，渲染共用 fs 路徑

remote drift 攔截：typed client 取回規格面報告、basis 與 store 面輸入 → 以 store 面輸入包成 host 的唯讀最小 `Store` adapter（只服務這個 change 的 design/tasks 與 created；引擎其餘 store 表面一律 `unreachable!`，沿用 `core::teststore` 既有哲學——把 drift 實際觸及的儲存面明白釘住，而非默默放寬）→ 本機 git 可用時（`util::git_available` — remote 模式下 workspace 必然存在，故 checkout 的有無就是判準）collect_workspace_facts＋compute_workspace_drift，否則傳 `facts: None` 讓工作區面走三值語意的不可得 → merge_drift_reports 合併 → 走與 fs 模式同一個渲染函式輸出。

無 checkout 時**必須**走 `None` 而非「git 不可用的 facts」：後者仍會對 path 錨點做 fs stat，在沒有程式碼的目錄裡全數 Missing，報告會宣稱程式碼不見了——不可得被講成壞掉，比不講更糟。

adapter 而非改引擎簽名：`collect_workspace_facts`／`compute_workspace_drift` 吃 `&dyn Store` 是既有形狀，動它屬本刀 Non-Goal。

凍結驗證：同一 change 內容在 fs 模式與 remote 模式（有 checkout）下的 drift 輸出逐位元一致；無 checkout 時輸出如實含工作區面不可得的既有標示（fs 模式在 git 不可用時的同一形狀）。

basis 在合併器裡只有一個用途：比對 `expected` 與 `current` 判定 stale。remote 模式的規格面與 basis 出自同一 snapshot（決策 2），兩者恆等，client 以 `DriftBasis { expected: b, current: b }` 餵入 → `stale` 為 `None`。這與 fs 模式一致（fs 的 bundle 與 current 也是背對背從同一 store 算出，恆等），輸出凍結因此成立。wire 上的 basis 是「規格面基準為何」的誠實記錄，本刀不引入任何跨 snapshot 的 stale 判定。

### 決策 4：DTO 與引擎型別的映射單點雙向，落在 host

wire DTO 與 core 的 `SpecDriftReport` 之間的雙向轉換單點實作於 `speclink-host::drift`——它是唯一同時依賴 core 與 protocol、且 server 與 client（speclink-remote）都已依賴的組合點。protocol crate 依契約不得依賴 speclink-core（`protocol/src/lib.rs`：「must never depend on speclink-core」），映射放不進去；分散到 server（core→wire）與 client（wire→core）各一半則既非單點、也沒有任何 crate 能寫出往返測試。

引擎核心型別不加 serde 標註，演進自由不被 wire 需求綁架。轉換測試以往返斷言（core → wire → core 結構相等）固定於 `cargo test -p speclink-host`。

## Implementation Contract

- Behavior：remote 模式（有 checkout）執行 drift 得到與 fs 模式同形的完整報告；無 checkout 得到規格面報告＋工作區面不可得標示；fs 模式輸出零變更。
- Interface / data shape：GET 形式的 change-scoped drift 端點回 wire DTO（specDrift＋basis＋change 的 store 面輸入，camelCase；specDrift 只含規格面維度與規格假設，無 broken anchors 等工作區/git 欄位；design/tasks 缺席與空內容可區別）；host 的 `spec_drift` 專用入口回 `SpecDriftView`，橋接 `Store` 視圖不對外公開；host 的唯讀最小 `Store` adapter 供 client 以 store 面輸入餵引擎；typed client drift 方法；CLI drift 動詞在 remote 模式的人眼與 --json 輸出與 fs 模式形狀權威一致。
- Failure modes：未知 change → 404 not_found；store 失聯 → 503 unavailable（CLI 翻譯為既有 remote 錯誤訊息）；本機 git 不可用 → 工作區面三值不可得，動詞仍成功；server 錯誤 → 動詞以既有 remote 錯誤路徑失敗，不輸出偽報告。
- Scope boundaries：不動引擎的 drift 計算與合併；不動 fs 模式輸出；不開泛用的橋接視圖存取；不引入跨 snapshot 的 stale 判定。
- Acceptance criteria：cargo test -p speclink-protocol（DTO serde 與 schema）、-p speclink-host（wire↔引擎往返、spec_drift 入口）、-p speclink-server（端點）、-p speclink-cli（remote drift 對 stub 與輸出對齊）全綠；npm run test:all 全綠且既有凍結零 diff。

## Risks / Trade-offs

- wire DTO 與 core 型別平行存在 → 單點映射＋往返測試守住；避免在 core 加 serde 的耦合代價更高。
- host 的公開表面多一個 drift 專用入口，未來每個新的 server 查詢可能各要一個 → 這正是想要的形狀：每個入口都是被審視過的組合點，勝過一個讓 adapter 自由取用的裸視圖（決策 2）。
- 規格面與工作區面各自基於不同時點（server snapshot vs 本機當下）→ 既有合併器本就以 basis 標示規格面基準，報告如實呈現，diagnostic 性質下可接受。
- 無 checkout 的輸出依賴 fs 模式「git 不可用」的既有形狀 → 該形狀在 drift-client-server-split 已凍結有測試，直接複用。

## Migration Plan

純新增接線；前置（server 路由骨架、snapshot 讀取、引擎分解）全部就緒。phase2-e2e-chain 刀依賴本刀先歸檔。回退即回捨 change，remote drift 回到不可用，fs 模式不受影響。

## Open Questions

（無）

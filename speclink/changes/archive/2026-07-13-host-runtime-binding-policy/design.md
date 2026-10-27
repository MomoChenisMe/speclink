## Context

engine-typed-core 之後，CLI 與 Node dispatch 都經 crates/speclink-core/src/command/mod.rs 的 execute(store, ws, cmd) 執行；但 execute 仍攜 Workspace 供 host 側查找，且 core 內有五處直接讀本機事實：util 的 git_identity（git config user.name/email）被 newcmd、archive、demo、inprogress、discuss 呼叫；config 的 EnvOverrides 便利建構直接讀 process env（SPECLINK_TDD 等）；workspace 的模式解析直接讀 SPECLINK_STORE_URL 與 current_dir。政策解析核心已是純函式（resolve_policy(env, app, wf)），fail-closed 已由前兩刀交付。teamstore-contract-v2 已固定 TeamStore 契約與 in-memory reference，但尚無任何消費者。平台架構 §4.2 定義 Host 職責、§4.6–4.8 定義 binding、ExecutionContext 與 policy 歸屬；路線圖 §3.3／§3.4／§3.7 是本刀的缺口清單。

## Goals / Non-Goals

**Goals:**

- 新 `speclink-host` crate 成為 canonical 應用服務層：ExecutionContext、binding fail-closed 驗證、policy injection、lifecycle gate 裁決、對 TeamStore 的 UoW／event commit 組合。
- Engine 規格面去 env／git：core 非測試碼不再讀 process env 與 git identity，全部由 Host 邊界解析注入。
- CLI 與 Node dispatch 改經 host 組裝，現行輸出逐位元凍結（baseline exe 對照＋parity／color／twin 全綠）。

**Non-Goals:**

- 不做 approval 綁 revision 與 stale evidence（順位 5）；不拆 drift 的 git 讀取（順位 6）；不做 Protocol／HTTP binding handshake（順位 7／Phase 2）。
- 不實作真實 authorization（hook 介面位，本地恆允許）；不把 policy digest 加進任何現有輸出。
- 不動 speclink-fs、Store seam、skills 與 Context Projection。

## Decisions

### 決策一：speclink-host 為獨立 crate，依賴方向 host → core 與 host → store

Host 組合 Engine（speclink-core）與 TeamStore 契約（speclink-store），core 不反依賴 host。CLI 與 Node 的組裝點從「直接呼叫 core」改為「經 host 建 ExecutionContext 再呼叫 core」。替代方案：host 作 core 的模組——core 將被迫依賴 speclink-store 且應用層與領域層互相滲透，違反藍圖「Host 是 Engine 對外唯一應用層邊界」的分層，被拒。

### 決策二：ExecutionContext 由 Host 邊界一次解析，command 參數不可覆寫 identity

SpeclinkExecutionContext 含 actor（顯示身分字串與來源）、project/repo binding、mode（fs／shared-store）、resolved EffectiveWorkflowPolicy（含政策內容 digest）。Host 在進入點解析一次：本地模式 actor 來自 git config（沿用現行 git_identity 語意，無 git 時為匿名、行為同今日）、政策來自既有 fail-closed 四層解析。execute 簽名改收 ExecutionContext；Command enum 不含任何 actor／policy 欄位，模型或呼叫端無從覆寫（藍圖 §4.7）。替代方案：per-command 傳 actor 參數——回到「Tool 可繞過租戶邊界」的 P0 缺口，被拒。

### 決策三：git identity 搬遷至 host，core 流程改收明確 actor

util 的 git_identity 函式移入 speclink-host（identity 解析屬 host 職責）；newcmd、archive、demo、inprogress、discuss 五處呼叫點改由 ExecutionContext 取 actor。「無 git 或未設 user.name 時不蓋章」的現行行為由 actor 的 Option 語意保留，輸出逐位元不變。替代方案：core 留函式、僅約定不呼叫——約定無法被編譯器與測試強制，違反「單一實作」原則，被拒。

### 決策四：policy 與模式的 env 層由 host 注入，core 保留純函式

EnvOverrides 的「直接讀 process env」便利建構從 core 非測試碼移除；host 邊界以 std::env lookup 建 EnvOverrides 後傳入既有 resolve_policy。workspace 模式解析的 SPECLINK_STORE_URL 讀取同樣上移（core 保留 resolve_mode_with 注入形）。EffectiveWorkflowPolicy 型別落在 host：包 core 的 ResolvedPolicy 加政策文件 digest（本地以 config.yaml 內容 digest 充當 policyRevision 前身；不進任何現有輸出）。替代方案：core 同時保留讀 env 的捷徑——兩條路徑必然漂移，且 server 模式誤用捷徑即是 §3.4 的 client 決定 policy 缺口，被拒。

### 決策五：binding 型別與 fail-closed 驗證，本地模式映射 default binding

ProjectId／RepoId 沿用 speclink-store 的身分型別；binding 解析規則：本地 fs 模式以 workspace root 映射固定 default project/repo（行為不變、無新設定）；binding 缺失、無權限或多個候選時回拒絕錯誤、不得自動選第一個（藍圖 §4.7）。遠端 binding 的網路驗證屬 Phase 2，本刀交付驗證邏輯與錯誤形狀。替代方案：本地也要求顯式 binding 設定——破壞零設定的本地體驗且無對應需求，被拒。

### 決策六：lifecycle gate 狀態機落在 host，本地為唯讀映射

LifecycleStation 封閉 enum（drafting、review、ready、applying、verified、archived）與 transition 裁決函式（來源站→目標站→允許或拒絕原因）構成單一裁決點；本地模式提供唯讀推導（未開工＝drafting、started_at 存在＝applying、封存＝archived；review／ready／verified 需 approval 與 evidence 語意，本地推導不產生）。本刀不讓任何現行動詞經 gate 裁決（避免行為變更）；順位 5 起 evidence 與 approval 接入後才啟用強制。替代方案：立即讓本地 archive 等動詞走 gate——本地無 approval 來源，強制即破壞現行流程與輸出凍結，被拒。

### 決策七：Host 對 TeamStore 的 commit 骨架以整合測試交付

host 的 commit 模組實作「以 ExecutionContext 開 unit of work → 領域事件映射為 event records（單向：core typed event → store record）→ commit」的組合路徑；以 speclink-store 的 in-memory reference 做整合測試證明 event records 原子落 outbox、CAS 衝突傳遞 host 錯誤。不接線任何現行 CLI 流程（本地仍走既有 Store seam）。替代方案：等 Phase 2 driver 再寫 commit 路徑——host 的 UoW／event commit 職責（路線圖 §3.7）將在 server 刀首次實作、失去 canonical 先行驗證的意義，被拒。

### 決策八：輸出凍結以 baseline exe 雙沙盒對照驗證

CLI 與 Node 組裝點改動屬行為保持重構：遷移前保存 baseline exe（cargo build --release），對樣本 workspace 逐動詞比對 stdout／stderr／exit code 逐位元一致；parity 31 項／color 16 項／twin 8 情境全綠。身分與 env 效果不變的驗證含：設定 SPECLINK_TDD 與 git identity 的情境在遷移前後結果相同。

## Implementation Contract

- **行為**：交付後工作區多一個 speclink-host crate；CLI 與 Node dispatch 的一切現行輸出（人眼、--json、exit code、錯誤訊息）逐位元不變；git identity 與政策環境變數的效果不變、讀取點在 host。cargo test -p speclink-host 涵蓋 context 解析、binding fail-closed、gate 裁決與 TeamStore commit 整合測試。
- **介面／資料形狀**：SpeclinkExecutionContext { actor, project, repo, mode, policy }（Rust PascalCase 型別、snake_case 欄位）；EffectiveWorkflowPolicy 包 ResolvedPolicy 與 digest；LifecycleStation 六站封閉 enum 與 transition 裁決函式回允許或帶原因拒絕；binding 錯誤形狀區分缺失、無權限、多義。execute 簽名攜 ExecutionContext（取代 host 查找用途的 Workspace 參數；純查詢所需的 workspace 資訊由 context 或既有欄位供給）。
- **失敗模式**：binding 缺失／多義／無權限回拒絕錯誤（不自動選擇）；非法 gate transition 回帶原因拒絕；host commit 遇 TeamStore revision_conflict 原樣傳遞；本地模式一切現行錯誤訊息不變。
- **驗收**：baseline exe 對照逐位元一致；parity／color／twin 全綠；cargo test --workspace 與 npm run test:all 全綠；git grep 確認 core 非測試碼無 process env 讀取與 git identity 呼叫。
- **範圍邊界**：in scope——host crate、core 去 env/git、execute 簽名、CLI/Node 組裝點、gate 型別與裁決、TeamStore commit 骨架整合測試；out of scope——approval/evidence、drift 拆分、Protocol/HTTP、真實 authorization、任何輸出變更。

## Risks / Trade-offs

- [execute 簽名變更波及面大（CLI 全部 handler 與 Node dispatch）] → 型別驅動：簽名一改殘留呼叫全編譯失敗；行為保持由 baseline 對照與 parity／color／twin 兜底。
- [identity 搬遷後某流程漏接 actor、蓋章消失] → 每個蓋章流程（created_by、started_by、discuss 建立者章、archive 歸屬）都有既有測試斷言章內容；遷移前後對照樣本 workspace 的 metadata 逐位元比對。
- [gate 型別先行、無消費者而漂移] → gate 裁決函式以 spec 場景鎖住 transition 表；順位 5 接入 evidence 時只增來源不改表。
- [雙沙盒 baseline 對照的 scratchpad 基建會消失] → baseline exe 與樣本存非 scratchpad 位置（沿 engine-typed-core 慣例），對照步驟記錄在 tasks。
- [Windows 與 macOS 的 git config 行為差異] → identity 解析沿用現行 util 實作原樣搬移不改邏輯；既有跨平台測試護欄不動。

## Migration Plan

單刀交付：內部邊界搬遷對使用者零可見變更，無資料與設定遷移。回滾即還原 commit。後續採用：順位 5 的 evidence 與 gate 強制、順位 7 的 Protocol、Phase 2 的 Server 全部落在 speclink-host 之上。

## Open Questions

（無）

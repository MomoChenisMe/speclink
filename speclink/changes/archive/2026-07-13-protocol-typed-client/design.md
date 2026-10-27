## Context

遠端路徑現況：crates/speclink-remote（約 576 行）提供 PAT auth、project-scoped URL、repo key 與逐 verb 的 raw serde_json::Value 收發；crates/speclink-cli/src/remote_commands.rs（約 784 行）攔截 remote 模式動詞、自行把 raw payload 重組成與 fs 模式一致的輸出。verb-contract 正典已要求「remote 模式輸出形狀與 fs 一致」，twin harness 8 情境以 stub server 對照 remote/fs 輸出。藍圖 §4.5 定義 Protocol 的五個組成（Command/Query/Context/Event discovery/version+error+ETag+scope）、§4.7 定義 binding handshake payload、§9.2 定義 event capability discovery 宣告形狀。host 刀已交付 binding fail-closed 驗證邏輯與 ExecutionContext；teamstore 刀已定 revision 語意。本刀是純 client 側：Server 端點屬 Phase 2。

## Goals / Non-Goals

**Goals:**

- speclink-protocol crate 成為 wire contract 的唯一 Rust 定義，Rust 型別為正典、JSON Schema 可匯出。
- speclink-remote 全面 typed 化：raw serde_json::Value 退場、error translation 正式化為 error reason registry、加 binding handshake。
- CLI remote 攔截層改消費 typed client，remote/fs 輸出逐位元凍結。
- stub server 對測涵蓋請求形狀、ETag/If-Match、error 對映、handshake fail-closed。

**Non-Goals:**

- Server 端點、server-side conformance、SSE/WS transport 實作、Context Materializer、Desktop RemoteDataSource、npm/OpenAPI 發布。

## Decisions

### 決策一：protocol 為獨立 crate，依賴方向 remote → protocol、未來 server → protocol

speclink-protocol 只含型別、常數與 registry（serde、schemars 級輕依賴），不依賴 speclink-core／host／store——wire contract 是 client 與 server 的共同下游，任何一側的實作細節都不得滲入。speclink-remote 依賴 protocol；Phase 2 的 server 同樣依賴 protocol 實作端點。替代方案：DTO 放 speclink-remote——server 將被迫依賴 client crate，或複製型別重演語意分叉，被拒；放 speclink-host——protocol 是網路契約非應用服務，host 的 ExecutionContext 等內部型別不應與 wire 型別混包，被拒。

### 決策二：Rust 型別為正典、JSON Schema 為匯出物

DTO 以 serde（camelCase rename）定義，配 schemars derive 匯出 JSON Schema（測試中產出並斷言可序列化），供非 Rust 實作與文件消費；不引入 OpenAPI 產生器與文件站。API version 為 protocol crate 的常數，請求標頭與 handshake 回應皆攜帶。替代方案：先寫 OpenAPI YAML 再 codegen Rust——生成碼品質與 serde 慣例衝突，且本 workspace 的正典一貫是 Rust 型別（store／host 先例），被拒。

### 決策三：error reason registry 正式化既有 mapping 經驗

protocol 的 error 回應形狀固定：HTTP status 加 machine-readable reason 字串（封閉 registry：not_found、permission_denied、revision_conflict、invalid_argument、invalid_config、refused、unavailable、internal——對齊 store 六類與命令層五碼的聯集語意，去除重複）加人類訊息。typed client 把 reason 對映到 CLI 既有錯誤訊息（現行 remote error translation 的文字保留，訊息逐位元不變）。替代方案：沿用 HTTP status 單軌分類——409 同時承載 CAS 衝突與 refused 類前置條件拒絕，語意不可分，正是現行旁路的缺陷，被拒。

對映表（wire reason → CLI 訊息，引擎類 pass-through）：wire 形狀維持 `{ status, reason, message }` 三元組，無細粒 code、無 details 判別欄位。

- **client 表僅擁有連線層文字**（沿現行 translation 逐位元）：`revision_conflict` →「content changed since you read it — re-read it and re-apply your edit」；`unavailable`（含所有 5xx）→「server unavailable — check the connection url (`remote.url` in .speclink.yaml or SPECLINK_STORE_URL)」；`internal` →「internal speclink error — update speclink or report a bug」；`permission_denied` 以 status 判別——401 →「authentication failed — run `speclink auth login`」、403 →「access denied — your account has no access to this project; ask a project admin」。
- **引擎類 pass-through**：`not_found`、`invalid_argument`、`invalid_config`、`refused` 顯示 server `message` 原文——對齊 fs 模式「引擎訊息直接印」的語意；Phase 2 server 包引擎後 remote/fs 錯誤輸出自然一致。細粒子情境（ownership_lost、gate_pending、tasks_incomplete、discussion_archived、repo 鏈、token 三態等）的措辭改由 server（測試中為 stub）以 message 承載，隨實驗性旁路退場。
- **語意邊界**：`revision_conflict` 嚴格限於「請求所指資源的 ETag/If-Match CAS 前置失敗」（teamstore 刀的 revision 語意）；archive 時 canonical spec 前進屬 server 端 gate → `refused` pass-through。
- **fallback**：transport 失敗 →「server unreachable — check the connection url (`remote.url` in .speclink.yaml or SPECLINK_STORE_URL)」；未知 reason 或 envelope 缺失／不可解析 → 一般錯誤「unexpected server response — update speclink or report a bug (HTTP {status})」，message 保留供顯示、不 panic。

### 決策四：binding handshake 為 client 連線前置，fail closed

typed client 提供 handshake 呼叫（GET /binding 形狀：actor、project、repo、apiVersion、engineVersion、capabilities 含 events 宣告）；API version 不相容、binding 缺失／無權限／多義時回拒絕錯誤——對映 host 刀的 binding fail-closed 語意，不自動選擇。client 端唯一自判條件是 apiVersion 相容性（與 protocol 常數比對），拒絕文字沿現行「server does not support this CLI's API version — upgrade the CLI or the server」；binding 缺失／無權限／多義由 server 以 registry 錯誤拒絕（多義時候選清單由 server 在 message 列出），client 依決策三對映或 pass-through。現行 remote 動詞流程在 handshake 失敗時停止；stub 對測涵蓋不相容 version 與多義候選情境。Event discovery 宣告（transports／polling／resume）隨 handshake 回應解析為型別，本刀只解析保存、不建立連線。替代方案：每動詞自行探測——回到逐 verb 旁路的重複探測與不一致，被拒。

### 決策五：remote 攔截層收編為薄轉譯層、輸出凍結

remote_commands.rs 改為：解析 argv 後呼叫 typed client、以 typed response 走與 fs 模式相同的渲染路徑；自行重組 raw payload 的碼全數移除。remote/fs 輸出一致性由 twin harness 8 情境守住；本刀不改攔截層的觸發條件與動詞覆蓋（路線圖 §3.2 的 CommandGateway 全面收斂屬後續與 Phase 2 接線）。替代方案：本刀順手把 remote 攔截收進命令層 CommandGateway——牽動 fs 模式執行路徑且與 Materializer 刀的 skill 語意互相依賴，超出單刀範圍，被拒。

### 決策六：client 對測沿 twin harness 的 stub server 基建

以 stub server 斷言：請求路徑與 body 符合 protocol DTO、If-Match 攜帶與 412/409 處理、error reason 對映到既有訊息、handshake fail-closed 三情境；twin 8 情境全綠保證輸出凍結。server-side conformance suite（讓自訂服務自測）以本刀的 stub 斷言為雛形，正式交付屬 Phase 2。

## Implementation Contract

- **行為**：remote 模式的全部現行動詞輸出（人眼、--json、exit code、錯誤訊息）與本刀前逐位元一致；fs 模式零變更。程式介面新增：speclink-protocol 的 DTO／registry／handshake 型別與 JSON Schema 匯出；speclink-remote 的 typed 呼叫面與 handshake。speclink-remote 與 remote_commands.rs 內不再存在 serde_json::Value 的 wire payload 處理。
- **介面／資料形狀**：DTO serde camelCase；error 回應 { status, reason, message }，reason 屬封閉 registry；handshake 回應含 actor／project／repo／apiVersion／engineVersion／capabilities.events（transports 陣列與 polling 宣告）；API version 常數隨請求攜帶。
- **失敗模式**：API version 不相容、binding 缺失／無權限／多義、digest 或 If-Match 衝突各回對應 reason 的 typed 錯誤；CLI 對映後的訊息文字沿用現行；handshake 失敗不進入動詞流程。
- **驗收**：cargo test -p speclink-protocol（含 JSON Schema 匯出測試）與 -p speclink-remote（stub 對測）全綠；git grep 確認兩處無殘留 wire 層 serde_json::Value；twin 8 情境、parity 31、color 16 全綠；npm run test:all 全綠。
- **範圍邊界**：in scope——protocol crate、remote typed 化、handshake、攔截層薄化、stub 對測；out of scope——server 端點、SSE/WS 實作、Materializer、CommandGateway 全面收斂、Desktop、發布物。

## Risks / Trade-offs

- [protocol 形狀先於 server 實作定案，Phase 2 發現不合] → stub 對測即最小 server 消費者；不合處以 API version 演進，registry 與 DTO 為封閉集合可控擴充。
- [remote 輸出重組碼移除時遺漏隱性格式化行為] → twin harness 逐位元對照是硬 gate；移除前先為每動詞留存 stub 回應樣本。
- [error reason registry 與命令層五碼、store 六類三套語彙混淆] → design 明定對映表方向（wire reason → CLI 訊息；host 負責內部分類 → wire reason 屬 Phase 2 server 側），三套各守一層、不互相取代。
- [schemars 依賴引入] → 僅 protocol crate 依賴、feature 可關；不滲入 core／host。
- [twin stub 與新 handshake 的相容] → stub server 加 handshake 端點；舊情境不帶 handshake 的路徑在 client 端以「未 handshake 則先行」策略統一，於對測中固定。

## Migration Plan

工作區內部重構加新 crate：無使用者可見變更、無設定遷移；回滾即還原 commit。後續採用：context-materializer 刀消費 Context API DTO 與 handshake capabilities；Phase 2 server 實作 protocol 端點並建立 server-side conformance；Phase 3 Desktop 以 typed client 建 RemoteDataSource。

## Open Questions

（無）

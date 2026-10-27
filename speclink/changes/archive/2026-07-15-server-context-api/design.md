## Context

host 的投影機制（crates/speclink-host/src/projection.rs）已完整：SnapshotProvider trait（輸入輸出即 protocol Context DTO）、materialize/staging/原子切換、verify_projection 的 digest fail-closed、mark_stale/is_stale/refresh、依流程縮小。remote 動詞流程（crates/speclink-cli/src/remote_commands.rs）以 VerbContextProvider 實作該 trait——註解明標 Phase 1 過渡：逐 artifact GET 拼裝 proposal/design/tasks，delta 與正典 specs「隨 Phase 2 Context API 到來」，snapshot id 為內容衍生假識別。server 側有 TeamStore snapshot（單次取得即一致讀取面）、scope 狀態記號（/sync-state 與查詢 ETag 同源，任何 commit 後必變）與既有 bearer/binding 前置。protocol 的 ContextSnapshotRequest（change/flow 欄位）與 ContextSnapshot（snapshotId、policyRevision、digest、documents）自 protocol-typed-client 刀就緒。

## Goals / Non-Goals

**Goals:**

- Context 三柱落地：一個請求得到一致快照，remote 投影含正典 specs、delta specs、config 與 LANGUAGE。
- snapshot id 是真識別（scope 狀態同源），refresh 可依 id 判免重寫；304 讓輪詢式刷新便宜。
- 過渡供應者退場，供應者 seam 不外洩到動詞流程（既有函式簽名與行為邊界不動）。

**Non-Goals:**

- 不動 apply/verify 技能內容（既有指示已是「自投影讀取、唯讀、動詞寫入」）——無三處同步。
- 不做 flow 縮小的 server 端實作（§7.3 縮小維持 materializer 職責；server 只依 change 縮資料量）。
- 不做投影的推播刷新（SSE 收 invalidation 後刷新屬 client 訂閱者的事，CLI 短命程序維持每動詞刷新）。
- 不做 Desktop 消費（Phase 3）；不動本地 fs 模式（不建投影的既有語意不變）。
- 不做 context 端點的分頁或增量 diff——規格文本量級下全量快照夠用，有量測證據再議。

## Decisions

### 決策 1：端點語意——POST 取快照，If-None-Match 走 header

context snapshot 端點掛在 project-scoped 路由樹下，以 POST 接受 ContextSnapshotRequest body（change/flow 為選填縮小欄位）。回應標頭帶 scope 狀態記號（與 /sync-state 同源）作 ETag；請求帶 If-None-Match 且記號未變時回 304 無 body。POST 而非 GET 是因請求有結構化 body；304 語意仍以標頭記號判定，快取正確性與查詢路由一致。端點上線後 handshake 的 `contextSnapshots` capability 由 false 翻為 true（誠實宣告；client 不以此旗標把關，故無行為相依，twin stub 的 binding body 不含此欄不受影響）。

### 決策 2：一致性出自單一 TeamStore snapshot

handler 取一次 store snapshot，全部文件（正典 specs、指定 change 的文件與 delta specs、config、LANGUAGE、未指定 change 時的全部 changes）自該 snapshot 讀出——不逐檔對 live store 讀。snapshot id 即取 snapshot 當下的 scope 狀態記號；文件 digest 用契約的 content_digest（與 materializer 驗證同源）；policy revision 為 workflow config 文件在該 snapshot 的 revision（文件不存在則缺席）。併發寫入發生在取 snapshot 之後不影響本次回應——client 下次刷新自然拿到新 id。

### 決策 3：server 依 change 縮資料量，flow 透傳

請求指定 change 時，documents 含：該 change 的全部 artifacts、該 change 的 delta specs、全部正典 specs、config 與 LANGUAGE——這是 apply/verify 流程需要的完整讀取面；未指定 change 時回全量投影內容（§7.2 佈局的全部鏡像文件）。flow 欄位原樣透傳不影響 server 文件集——materializer 既有的依流程縮小（§7.3）繼續負責 INDEX 與 contextFiles 的選擇。切這一刀的理由：資料量縮減是傳輸問題（server 職責），檔案選擇是呈現問題（materializer 職責），兩者混在 server 會讓縮小規則長在離消費者最遠的地方。

### 決策 4：typed client 方法與 304 的型別表達

typed client 新增 context snapshot 方法：輸入 ContextSnapshotRequest 與既知 snapshot id（選填），輸出為二值——未變（304）或新快照。走既有請求骨架（三標頭、錯誤翻譯、handshake 前置）；303/未知 reason 等異常沿用既有 translate 路徑。CLI 的 Context API 供應者以「manifest 現值的 snapshot id」帶入 If-None-Match：未變時 refresh 免重寫（materialize 跳過），變了才走 staging 原子切換。

### 決策 5：過渡供應者原地汰換，失敗語意不變

VerbContextProvider 的逐 artifact 拼裝換成單一 context snapshot 呼叫，struct 與 trait seam 保留（供應者仍是動詞流程的注入點）。失敗語意維持現行：Context API 錯誤是響亮警告、動詞照常完成、既有投影標 stale——投影是加速 Agent 的 side effect，不是動詞正確性的一部分。投影內容從「三個 artifact」升級為完整佈局後，context-projection 既有的 materialize/verify/stale 測試面全部直接適用。

「未變免重寫」與「失敗標 stale」是本刀新增、既有 CLI 流程尚未有的行為：CLI 端 `point_context_files_at_projection` 讀現行 manifest 的 snapshotId 帶入 If-None-Match，client 方法回二值——未變（304）則整個 materialize 跳過（投影檔案不動）、新快照則以「預載該快照的 provider」呼叫 materialize；client 失敗時響亮警告、標既有投影 stale、動詞照常。免重寫與 stale 決策留在 CLI 編排層（materialize 之前），materializer 與 SnapshotProvider trait 一律不動，host projection 既有測試面因此零改動。

### 決策 6：LANGUAGE 文件種類補進 store 契約（apply 期範圍調整）

原提案假設 server 能提供 LANGUAGE 文件，但 TeamStore 的 `DocumentId` 封閉集只有六種、無 shared-vocabulary 種類，host bridge 的 `read_language` 因而寫死回 None——server 模式下 LANGUAGE 永不可得，規格「documents SHALL 涵蓋 … config 與 LANGUAGE」無法成立。取捨後決定補齊 store 契約：`DocumentId` 新增 `Language` 變體（openspec 路徑 `<spec_dir>/LANGUAGE.md`），sqlite 以 `lg` 標籤編解碼，host bridge 的 `read_language` 改讀 `DocumentId::Language`。不新增 write 命令——LANGUAGE 在 fs 模式由 onboarding 直接寫檔，server 模式的種子由測試以 UnitOfWork 直寫（與其他文件相同）。fs 模式的 `read_language`（讀 LANGUAGE.md）不變；封閉集的窮盡匹配（closed-set 測試、sqlite encode/decode）同步補上 `Language` 臂。這是純加性擴充：既有六種文件語意不動。

## Implementation Contract

- Behavior：remote 模式執行 apply 階段動詞後，投影含正典 specs、該 change 的 delta specs、artifacts、config、LANGUAGE 與 INDEX，manifest 的 snapshot id 為 server 真識別；重複執行同動詞且 server 無新 commit 時投影不重寫；server 有新 commit 後下次動詞刷新得到新快照。
- Interface / data shape：POST context snapshot 端點（project-scoped；body 為 ContextSnapshotRequest；回 ContextSnapshot；ETag/If-None-Match/304）；typed client 方法回「未變或新快照」二值；ContextSnapshot 欄位不變（DTO 已定案）——snapshotId 為 scope 狀態記號、policyRevision 為 config 文件 revision（存在時）、documents 逐項 digest。store 契約：`DocumentId::Language` 為新增變體（唯一新增；openspec 路徑 `<spec_dir>/LANGUAGE.md`），sqlite 標籤 `lg`；host bridge `read_language` 讀該文件。
- Failure modes：未認證/非成員 → 既有 401/403 三元組；未知 change 縮小 → 404 not_found；store 失聯 → 503 unavailable；client 側 API 失敗 → stderr 響亮警告、動詞 exit code 不受影響、既有投影標 stale；投影 digest 驗證不符 → 既有 fail-closed 行為（context-projection 能力）不變。
- Acceptance criteria：cargo test -p speclink-server 全綠（一致性、縮小、304、前置、e2e）；cargo test -p speclink-cli 與 -p speclink-host 全綠（供應者汰換後投影測試、免重寫）；npm run test:all 全綠且 parity/color/twin 凍結零 diff。

## Risks / Trade-offs

- 全量快照的傳輸量隨規格庫成長 → 規格是文本、change 縮小已涵蓋主要流程；分頁/增量留待量測證據。
- snapshot id 綁 scope 狀態記號，任何 commit（含無關文件）都使投影重刷 → 正確性優先與 ETag 粒度一致；粗粒度的代價是多一次全量拉取，可接受。
- POST 端點回 304 較不常見 → 語意以標頭記號為準且在規格情境中固定，client 是我們的 typed client，不依賴中介快取行為。
- 供應者升級後投影變大，staging 原子切換的檔案數上升 → 既有機制本就為完整佈局設計（Phase 1E 測試即以正典 specs 入投影），無新機制。

## Migration Plan

前置依賴全部就緒（server 路由骨架、TeamStore snapshot、scope 狀態記號、投影機制）。實作順序：server 端點 → typed client 方法 → 供應者汰換與免重寫 → e2e。回退即回捨 change：供應者退回逐 artifact 拼裝（過渡程式碼在版本控制中），投影機制與技能不受影響。

## Open Questions

（無）

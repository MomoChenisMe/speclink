## Context

verb-contract-and-remote-client 交付的 remote 模式以獨立連接檔 .speclink.remote.yaml 承載連接設定（url、repo），檔案存在即模式訊號——此為討論 Round 11 的兩檔佈局。維護者重審後決定回歸單檔：政策鍵遷居 store 側後，.speclink.yaml 已瘦身為純 workspace 設定（tools），再為兩個欄位維持一個獨立檔案與獨立的發現路徑不符比例原則。本 change 在 verb-contract-and-remote-client 歸檔後動工，把連接設定改為 .speclink.yaml 的 remote 區段。

約束：fs 模式的人眼與 --json 輸出是回歸保護對象，完全不可變；core 不得含 ANSI 與寫死的儲存假設；跨平台（Windows/macOS/Linux）。

## Goals / Non-Goals

**Goals**
- remote 連接設定（url、repo）以 .speclink.yaml 的 remote 區段承載，模式訊號改為區段存在與否
- init --store remote、link、unlink 改寫該區段且保留檔內既有欄位
- 殘留 .speclink.remote.yaml 有明確的一行遷移警告

**Non-Goals**
- 不改動 verb-contract 與 remote-auth 的任何需求（僅換連接設定載體）
- 不提供自動遷移指令（兩個欄位手動搬移，警告給指引）
- 不改變 fs 模式任何行為與輸出
- 不把政策鍵搬回 .speclink.yaml
- 敏感 url 不另設機制（沿用 SPECLINK_STORE_URL）

## Decisions

### Decision 1: remote 區段存在即模式訊號（不設 type 欄位）

.speclink.yaml 含 remote 區段即 remote 模式，無區段即 fs 模式。不引入 store.type 之類的判別欄位——區段存在已是充分訊號，額外欄位是冗餘狀態（可能與區段內容矛盾）。

- 替代方案 A：保留獨立連接檔（Round 11 現狀）——被本 change 的動機否決：兩檔兩路徑的維護成本高於「存在即訊號」的簡潔收益。
- 替代方案 B：remote 區段加 enabled 或 type 欄位——多一個可與事實矛盾的開關，YAGNI。

朝「storage 解耦引擎」的靠攏說明：本決定只動 bootstrap 載體，Store 縫線與動詞契約不變，屬中性調整。

### Decision 2: url 可缺省，由 SPECLINK_STORE_URL 供給；兩處皆缺為明確失敗

Round 11 分檔的理由之一是敏感 url 團隊可單獨 gitignore 連接檔。合併後的等價機制：committed 的 remote 區段可省略 url（僅留區段與 repo），執行時由 SPECLINK_STORE_URL 供給；兩處皆無 url 時 CLI 明確失敗並提示設定方式——不得靜默 fallback 到 fs 模式（與討論既定的「連不上不製造假真相」紅線一致）。

- 替代方案 A：整檔 gitignore .speclink.yaml——犧牲 tools 清單的團隊共享與 update 修剪確定性，代價過大。
- 替代方案 B：url 欄位支援加密或 secret 引用——認證資訊本來就不在此檔（PAT 存使用者層級），url 敏感度屬邊緣需求，環境變數已覆蓋，過度設計。

### Decision 3: 殘留 .speclink.remote.yaml 不解析、僅單行警告

CLI 偵測到專案根有 .speclink.remote.yaml 時，於 stderr 輸出一行遷移警告（沿用既有 deprecation 警告慣例：speclink: warning: 前綴、每次執行至多一行），內容指引把 url/repo 搬入 .speclink.yaml 的 remote 區段並刪除舊檔；舊檔內容不參與模式判定與連線。

- 替代方案 A：雙軌相容（繼續解析舊檔、標 deprecated）——兩個真相來源需要衝突裁決規則，pre-release 階段不值得背這個複雜度；政策鍵遷移採軟著陸是因為值有既有使用者語意，連接檔是剛發佈的新面。
- 替代方案 B：自動遷移（CLI 代寫 .speclink.yaml 並刪舊檔）——對使用者檔案的隱式寫入副作用，兩欄位的手動成本趨近零，不值得。

### Decision 4: serde 結構與向後相容

AppConfig（crates/speclink-core/src/config.rs）新增選填的 remote 巢狀區段（url 選填字串、repo 選填字串）。向後相容：無 remote 鍵的既有 .speclink.yaml 照常解析（serde default）；區段內未知鍵的容忍度沿用 AppConfig 既有行為。寫入路徑（init/link/unlink）採讀取—修改—寫回：保留檔內既有欄位與註解以外的內容（YAML 重序列化不保證註解存活——警告訊息與文件註明此限制）。

- 替代方案：獨立解析 remote 區段為另一個檔案級結構——與 AppConfig 分離會複製發現與讀檔邏輯，無收益。

### Decision 5: core / cli 邊界

模式解析（發現 remote 區段、殘留舊檔偵測的事實判定）歸 speclink-core 的 workspace 發現；警告訊息的文字組裝與 stderr 輸出、link/unlink 子指令的參數解析歸 speclink-cli。core 回傳結構化的偵測結果，不含任何呈現字串。

- 替代方案：core 直接輸出警告文字——違反 core 無終端呈現的既有邊界，否決。

### 命名慣例

remote 區段鍵名沿用連接檔欄位名（url、repo），無縮寫；警告前綴沿用 speclink: warning:；模組與函式命名遵循 snake_case、旗標 kebab-case（均與既有慣例一致）。

## Risks / Trade-offs

- **風險：前置 change 的實作形狀偏移**——本 design 假設 verb-contract-and-remote-client 依其規格交付連接檔解析與 link/unlink。緩解：動工前執行 speclink drift 檢查 stale delta assumptions；本 change 的 delta 以正典需求文字為基準，不錨定程式碼行號。
- **風險：讀取—修改—寫回破壞 .speclink.yaml 的使用者註解**——serde_yaml 重序列化不保留註解。緩解：文件註明；tools 等欄位值完整保留；此檔內容極少，影響有限。
- **風險：回歸對照**——fs 模式輸出零變更（不觸及既有基線）；remote 模式為 verb-contract 建立的新行為面，本 change 修改其測試而非 parity 基線。緩解：任務含 parity/color suite 通過驗證。
- **跨平台**——僅專案根單層檔案讀寫，無路徑分隔、換行、git 行為的平台分歧。

## Migration Plan

使用者操作（一次性）：把 .speclink.remote.yaml 的 url 與 repo 兩欄位搬入 .speclink.yaml 的 remote 區段，刪除舊檔。未遷移時每次執行 CLI 得到一行警告，模式判定以 .speclink.yaml 為準（殘留檔不生效——注意這表示未遷移的專案會退回 fs 模式並在無 openspec/ 目錄時得到明確錯誤，警告訊息須把這個後果講清楚）。

## Open Questions

（無——時序前置條件記錄於 proposal；動工時以 drift 驗證。）

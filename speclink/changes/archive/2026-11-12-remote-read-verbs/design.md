## Context

查證過的現況：

- **evidence**：引擎的儲存介面能讀進行中變更的 evidence（`read_evidence`），已封存的 evidence 是封存文件集裡的一份（`read_archived_artifact` 讀 `.evidence.json`）。沒有引擎命令，也沒有動詞。宿主套件的 HTTP 入口有 `GET /changes/{name}/evidence`，它不經引擎，由 `documents::change_evidence` 直接讀進行中的那一份；記錄不存在時回空集合，連不存在的變更也回空集合。`speclink-remote` 有對應的呼叫，CLI 只在遠端蓋章時用它。
- **trace**：`Command::Trace` 由 `lifecycle/trace.rs` 執行，讀的全部是儲存：正式規格與它的 `@trace` 註記、已封存變更的清單與各自的 delta capability、已封存的 metadata、已封存的 `.evidence.json`、討論記錄。沒有 git。CLI 在 dispatch 的宣告把它列為只限本機；線上歸屬表寫的不上線理由是「溯源要讀本機的程式碼與 git 歷史」，和程式不符。
- **嚴格重疊**：`Command::PlanStrict` 是「列出變更、問儲存要順序鍵、以嚴格模式計算」。不需要本機事實。線上歸屬表註明「隨第二波技能上線」。線上的 `GET /plan` 先讀看板順序文件，再把順序表交給 `Command::Plan`。
- **時間**：`list --json` 的項目沒有建立時間與最後修改時間。引擎的儲存介面有 `updated_at_secs`（排序用），檔案實作回傳檔案的修改時間，儲存視圖回傳該變更底下各文件最後修改時間的最大值。線上的清單項已經有 `created`（由 Host 從 metadata 補上），沒有最後修改時間。過渡用的 TeamStore 轉接回報的最後修改時間一律是 0。
- 命令的名稱與參數有逐命令的 golden（`command-describe.json`）；線上歸屬表用不含萬用分支的 `match`，新增命令時這兩處都會逼出決定。

限制：

- `--json` 的欄位是契約；動到它要同批更新 golden 並寫明。
- 兩種模式的人眼輸出要逐位元相同（除了明文列出的分歧）。
- 線上既有路由的行為要維持，除了這裡明寫的一項。

## Goals / Non-Goals

**Goals:**

- 任何形態下都能用一個動詞讀到變更的 evidence，進行中與已封存都可以。
- `trace` 與 `plan --strict-overlap` 在遠端的輸出和本機相同。
- 技能可以從變更的清單算出「建立多久、多久沒動」，不需要 git。

**Non-Goals:**

- 技能文字、溯源內容的擴充、evidence 的寫入、需要工作樹的動詞。

## Decisions

### D1: 引擎命令 EvidenceShow

`speclink-core` 新增 `command/evidence.rs`；名稱的查找放在 `lifecycle/model.rs`（`locate_change`、`archived_change_name`），`artifact cat` 與宿主套件共用：

- `Command::EvidenceShow { change }`，名稱 `evidence-show`。
- 結果 `EvidenceShowOutcome { change, archived_as: Option<String>, entries: Vec<EvidenceEntry> }`。`EvidenceEntry` 是引擎既有的型別。
- 解析名稱的順序：
  1. 有同名的進行中變更 → 讀它的 evidence。沒有記錄時 `entries` 是空的。
  2. 否則在已封存變更的清單裡找：名稱完全等於某個封存名稱，或去掉開頭的日期後等於它。有多筆時取最新的一筆。讀它的 `.evidence.json`，`archived_as` 是封存名稱。「最新」的判定：給的是帶日期的封存名稱而且封存區有這個名稱時，就是它；否則取 `<日期>-<名稱>` 裡日期最大的；都沒有時才是名稱不帶日期的舊封存（它沒有日期，視為最舊）。
  - 名稱不是單一路徑段（含 `/`、`\`，或是 `.`、`..`、空字串、`archive`）時不查任何儲存，直接是找不到。這道守門放在所有動詞共用的 `model::find_change`，所以每個以名稱找變更的動詞都一樣。
  3. 都沒有 → `not_found`，訊息沿用既有「找不到變更」的句式。
- evidence 的內容無法解析：回報錯誤（分類 `error`），不回傳空集合。這和線上路由今天「內容損壞是內部錯誤」一致。
- 純讀取，不產生事件，不寫入。

「取最新的一筆」的理由：同一個名稱被封存兩次很少見，而呼叫端（技能）通常剛封存完，要的就是最新的。

- **否決：動詞收一個 `--archived` 旗標** — 技能在封存前後都會讀，多一個旗標就多一個分支；名稱本身已經能分辨。

### D2: 動詞與輸出

`speclink-verbs` 新增家族 `evidence`：`evidence show <change> [--json] [--no-color]`，轉成 `EvidenceShow`，不讀 stdin。

人眼輸出：

```
Evidence: <change>  (<n> task(s))
  [archived as <dated-name>]          ← 只有讀到已封存的才有這一行

  <taskId>  <taskDesc>
    recorded <recordedAt>
    <file>
    <file>
```

沒有記錄時：`No evidence recorded for '<change>'.`，exit code 0。顏色只用在標題行與任務行；`--no-color` 只省略色碼。

`--json`：`{ "change", "archivedAs"?, "entries": [{ "taskId", "taskDesc", "actor"?, "repo"?, "headCommit"?, "touchedFiles", "recordedAt" }], "files": [...] }`。`files` 是全部 `touchedFiles` 的聯集，去重複並依位元組順序排序。它由輸出函式算，不存在引擎的結果裡。

### D3: 線上——evidence 的路由改成執行命令

`GET /changes/{name}/evidence` 的處理函式改成執行 `EvidenceShow`，回應是既有的 `ChangeEvidenceResponse`，新增兩個選填欄位：`archivedAs`，以及 `change`（這份記錄所屬變更自己的名稱，新的路由一律會帶）。`documents::change_evidence` 移除（它的解析邏輯已經在引擎裡）。

行為的差別只有一項：進行中與已封存都沒有這個名稱時，從 200 空集合變成 404 `not_found`。「變更存在但沒有記錄」仍然是 200 空集合，既有規格的那個情境不變。

線上歸屬表：`EvidenceShow` 對到這條路由。

CLI 的遠端分支呼叫 `speclink-remote` 既有的 `change_evidence`，把回應轉回引擎的結果，交給共用的輸出函式。轉換時以回應的 `change` 當變更名稱；回應沒有 `change` 就是舊版 Server（它只讀進行中的變更，對已封存或不存在的名稱都回空集合），轉換回報錯誤，CLI 不把那個答案印成「沒有記錄」。遠端蓋章直接用 `change_evidence` 的回應，不經這個轉換，不受影響。

### D4: 線上——溯源與嚴格重疊

- `GET /trace/{capability}`：執行 `Command::Trace`，回應是 `TraceResponse`。它的欄位和 `speclink trace --json` 的輸出一對一（camelCase），定義在 `speclink-protocol`。找不到 capability 時是 404，訊息含引擎的近似建議（和本機相同的句子）。
- `GET /plan?strictOverlap=true`：執行 `Command::PlanStrict`，回應是既有的 plan 回應型別，另帶 `strictOverlap: true`（一般的 plan 回應沒有這個鍵）。沒有這個查詢參數或值是 `false` 時是既有的行為。值不是 `true` 或 `false` 時是 400 `invalid_argument`。
- `strictOverlap: true` 是線上的標記，不進 `plan --json`。它的用途只有一個：舊版 Server 不認得這個查詢參數，會忽略它並回一般的計畫；`Client::plan_strict()` 沒收到標記時回報錯誤，不把一般的計畫當成嚴格重疊的結果（實作時稽核發現，見「稽核結論」第 5 項）。
- 兩條都是唯讀，不受寫入能力限制；線上歸屬表把 `Trace` 與 `PlanStrict` 從「不上線」改成對到路由。
- `speclink-remote`：`Client::trace(capability)` 與 `Client::plan_strict()`（兩個新呼叫與 `change_evidence` 把名稱編碼成單一路徑段，名稱裡的 `/`、`?`、空白不會變成別的路徑或查詢）；`convert` 新增把 `TraceResponse` 轉回引擎溯源報告的函式，做法和既有的 plan 報告相同。
- CLI：dispatch 的宣告把 `trace` 與 `plan --strict-overlap` 改成兩種模式都有。遠端分支取得報告後用共用的輸出函式印出。兩個固定的拒絕訊息與它們的常數移除。

嚴格重疊在儲存接口上用的順序鍵來自看板順序文件（儲存視圖的 `board_ranks`），和一般的 plan 在遠端用的來源相同。

### D5: 清單項的時間

引擎的清單項（`ListChangeJson`）新增兩個選填欄位：

- `created`：metadata 的建立日期，格式 `YYYY-MM-DD`（引擎讀 metadata 時本來就把時間收成日期）。metadata 沒有這個欄位或壞掉時不輸出。
- `modifiedAt`：`updated_at_secs` 轉成 RFC 3339 的 UTC 時間（秒）。回傳 0 時不輸出。

列出時每個變更呼叫一次 `updated_at_secs`。成本：檔案實作是讀檔案的修改時間；儲存視圖用的是已經列出過的 `changes/` 前綴，不多發任何呼叫（用成本測試確認列出變更的呼叫數不變）。

線上：`ChangeSummary` 新增選填的 `modifiedAt`，由引擎的清單項填入；`created` 維持由 Host 從 metadata 補上的既有做法（和引擎的值相同，測試釘住兩者一致）。CLI 的遠端分支把兩個欄位轉回清單項，所以兩種模式的 `list --json` 同形。

人眼輸出不用這兩個欄位。

本機的 `modifiedAt` 是檔案的修改時間，git checkout 或複製會改變它；它的用途只是「大概多久沒動」，技能把它當提示，不當判定的依據。這一點寫進下一個變更的技能文字。

- **否決：把閒置的判定做進引擎，輸出一個布林** — 門檻（幾天）是技能的建議，不是引擎的規則；引擎只給事實。
- **否決：只在 `instructions apply` 的結果裡加** — drift 技能也要用，而它不呼叫 `instructions apply`。

### D6: artifact cat 的已封存退路

`Command::ArtifactCat` 的名稱解析改成和 D1 相同的兩步：先找進行中的變更；沒有時找已封存的變更（封存名稱或原本的名稱，取最新的一筆），經儲存介面既有的「讀已封存文件」讀出。artifact 的識別沿用既有的詞彙（`proposal`、`design`、`tasks`、`specs/<capability>`）。兩步共用 D1 的解析函式，不寫第二份。

線上不用新路由：`GET /changes/{name}/artifacts/{*artifact}` 本來就執行這個命令，退路在引擎裡，所以遠端自動得到。回應裡的文件版本在讀到已封存的文件時是該文件的版本：命令的結果（`ArtifactCatOutcome`）帶出讀到的封存名稱，路由用它直接讀那份文件的版本，不再自己推一次「進行中或已封存」。

對進行中的變更，輸出與錯誤訊息逐位元不變。名稱在兩邊都沒有時，錯誤訊息維持既有「找不到變更」的那一句。

- **否決：新增 `archive cat` 之類的新動詞** — 技能只需要「讀這個名稱的提案」，不需要知道它封存了沒有。

## Implementation Contract

**行為**

- `speclink evidence show <change>`：本機、遠端、`dispatch`、`host.dispatch` 對同樣的資料輸出相同的人眼文字與相同的 `--json`。
- 對一個剛封存的變更執行 `evidence show <原本的名稱>`：得到封存前記錄的同一批項目，`archivedAs` 是封存名稱。
- `speclink trace <capability>` 與 `speclink plan --strict-overlap` 在遠端的 stdout（人眼與 `--json`）和本機對同樣的資料逐位元相同。
- `speclink list --json` 的每個變更有 `created`；儲存答得出時有 `modifiedAt`。勾選一個任務後，該變更的 `modifiedAt` 不早於勾選前的值。
- `speclink artifact cat proposal --change <封存名稱或原本的名稱>` 輸出已封存變更的提案；對進行中的變更輸出不變。
- 線上歸屬表對 `Trace`、`PlanStrict`、`EvidenceShow` 都回答路由，綁定路由表的兩個測試通過。

**介面與資料形狀**

- 引擎：`Command::EvidenceShow`、`EvidenceShowOutcome`；`ArtifactCatOutcome`（內容與讀到的封存名稱）；`model::locate_change`、`model::archived_change_name`；`ListChangeJson` 的 `created`、`modified_at`（序列化為 `modifiedAt`）。
- 線上：`GET /trace/{capability}` → `TraceResponse`；`GET /plan?strictOverlap=true`（回應帶 `PlanResponse.strictOverlap`）；`ChangeEvidenceResponse.archivedAs` 與 `ChangeEvidenceResponse.change`；`ChangeSummary.modifiedAt`。
- CLI：`speclink evidence show <change> [--json] [--no-color]`。

**失敗的樣子**

| 情況 | 結果 |
|---|---|
| `evidence show` 的名稱在進行中與已封存都沒有 | `not_found`；CLI 非 0，線上 404 |
| evidence 的內容無法解析 | 本機 `error`；線上 500（既有） |
| `trace` 的 capability 不存在 | `not_found` 與近似建議；線上 404 |
| `strictOverlap` 的值不合法 | 400 `invalid_argument` |
| 依賴成環時的嚴格計畫 | 和一般的 plan 相同：409 `refused`，訊息是引擎的 `dependency cycle: …` |
| 新版 CLI 連到沒有溯源路由的舊版 Server 執行 `trace` | 404；CLI 顯示既有的找不到訊息，非 0 |
| 新版 CLI 連到不認得 `strictOverlap` 的舊版 Server 執行 `plan --strict-overlap` | Server 回一般的計畫（沒有 `strictOverlap` 標記）；CLI 非 0，stderr 說明這個 Server 不支援 `plan --strict-overlap`，stdout 空 |
| 新版 CLI 連到舊版 Server 執行 `evidence show` | 舊路由的回應沒有 `change`；CLI 非 0，stderr 說明這個 Server 不支援 `evidence show`，stdout 空 |
| 變更名稱不是單一路徑段（含 `/`、`\`，或是 `.`、`..`、空字串、`archive`） | `not_found`，不查任何儲存；每個以名稱找變更的動詞都一樣 |

**驗收**

- `cargo test -p speclink-core`：`EvidenceShow` 的名稱解析與各種失敗；清單項的兩個欄位；命令名稱與參數的 golden、`render_golden` 同批更新。
- `cargo test -p speclink-verbs`：`evidence show` 的解析、人眼輸出與 `--json`（含沒有記錄、已封存）。
- `cargo test -p speclink-host --test http_door`：三條路由與線上歸屬表。
- `cargo test -p speclink-cli --test it remote_read_verbs::`：三個動詞在遠端的輸出等於本機對同樣資料的輸出；既有測試裡「trace 於遠端拒絕」「嚴格重疊於遠端拒絕」改寫成新行為。
- `cargo test -p speclink-server`、`cargo test -p speclink-remote` 通過；Node 的測試新增 `evidence show` 經 `dispatch` 與 `host.dispatch` 各一。

**範圍**

- 在範圍內：上面列的介面。
- 不在範圍內：技能的原稿、桌面 app、溯源的內容、使用者文件。

## Risks / Trade-offs

- [回歸對照] `list --json` 多兩個欄位會改到 golden 與 CLI 的快照測試。→ 刻意的變更，同批更新；人眼輸出的 golden 不能變，用 diff 確認。
- [回歸對照] 桌面 app 與 `speclink-remote` 用 `ChangeSummary` 與清單項。→ 新欄位都是選填、有預設值；桌面的遠端整條鏈測試不改就要通過。
- [evidence 路由的 404] 對不存在的名稱從空集合變成 404。→ 用搜尋確認呼叫端：CLI 的遠端蓋章只對存在的變更呼叫；Speclink Desktop 不呼叫這條路由。更新釘住舊行為的測試並在任務裡列出。
- [跨平台] `modifiedAt` 在 Windows 與 macOS、Linux 都取自檔案系統的修改時間，精度不同但都轉成秒。→ 測試只斷言「存在、格式正確、不倒退」，不斷言精確值。
- [成本] 列出變更時多一個時間的查詢。→ 儲存接口上不多發呼叫，用既有的成本測試釘住；本機是每個變更幾次 stat。
- [儲存解耦] 三個原本「只限本機」的讀取其實都只需要儲存；這個變更把它們的限制拿掉，剩下只限本機的只有真正需要工作樹的動詞。

## Migration Plan

沒有資料搬移。要退回時還原這個變更的 commit。

## Open Questions

- （已回答）溯源的 evidence 存在性偵測在儲存接口上讀已封存的 `.evidence.json` 用的鍵，是否對名稱不帶日期的舊封存也成立。**成立。** 儲存視圖讀已封存文件的鍵是「封存區前綴＋封存名稱＋文件路徑」，不檢查名稱有沒有日期（只有新寫入的封存才要求帶日期）。測試 `an_archived_change_without_a_date_in_its_name_is_traced_and_read_like_any_other`（`speclink-host` 的 `http_door`）釘住：名稱是 `legacy` 的舊封存進得了溯源鏈、帶得出 evidence，`evidence show legacy` 也讀得到，`archivedAs` 就是 `legacy`。

## 稽核結論

依 sharp-edges 清單（三個角色：存心濫用的人、趕時間照抄的人、搞錯用法的人）檢查這個變更新增的介面。找到的問題都在這個變更修掉並補了測試；留下的已知限制列在最後。

**檢查過、沒有問題的項目**

1. **三條路由的可見範圍與寫入。** 入口在進到任何路由之前先檢查身分、專案與 repo（既有的 `audit_without_an_identity_every_route_is_401_and_nothing_is_read` 逐一走過路由表，新的 `/trace/{capability}` 自動在內）。三條都是 GET，不受寫入能力限制，測試用沒有寫入能力的身分讀取，並斷言儲存沒有收到任何 commit。
2. **evidence 的內容不進錯誤訊息。** 記錄無法解析時，訊息只帶解析器回報的錯誤類別與行列位置，不帶內容。引擎、HTTP 入口與 CLI 各有一個測試，用含 `SECRET-CONTENT` 的壞記錄確認訊息裡沒有它。
3. **`strictOverlap` 以外的查詢參數不影響計畫。** `strict=true`、`foo=bar` 得到和一般 plan 相同的回應；`strictOverlap` 的值只接受 `true` 與 `false`，`maybe`、空字串、`1` 都是 400，不會被當成「有給就算開」。
4. **`modifiedAt` 遇到異常大的值不會 panic。** 儲存回報的秒數超出 RFC 3339 能表示的範圍（西元 9999 年之後，含 `u64::MAX`）時不輸出這個欄位；0 也不輸出。測試 `a_time_no_rfc3339_text_can_state_gives_no_modified_at_and_no_panic`。

**找到並修掉的問題**

5. **舊版 Server 對 `plan --strict-overlap` 會靜默回錯的計畫。** 原設計以為舊版 Server 會回 404，實際上 `GET /plan` 本來就存在，舊版只是忽略不認得的查詢參數，回一般的計畫，CLI 會把它當成嚴格重疊印出來。修法：嚴格重疊的回應帶 `strictOverlap: true`，`Client::plan_strict()` 沒收到就回報錯誤（D4、失敗表與兩份 delta spec 已同步）。這一項問過使用者後採用。
6. **用名稱讀到不屬於該變更的記錄。** 本機的檔案儲存是用「資料夾存不存在」找變更，所以 `evidence show ..` 會讀到規格資料夾底下的 `.evidence.json`，`evidence show archive/<封存名稱>` 會把已封存的變更當成進行中的讀出來，`evidence show archive` 會把封存資料夾本身當成變更。修法：所有動詞共用的 `model::find_change` 在查儲存之前先擋掉不可能是變更名稱的名稱（不是單一路徑段：含 `/`、`\`，或是 `.`、`..`、空字串；以及 `archive`），一律 `not_found`。守門原本只放在 `evidence show` 與 `artifact cat`，品質關卡指出其他動詞有同樣的問題後搬到共用的那一處。儲存接口那一側本來就由鍵的建構規則擋住，HTTP 入口另有測試 `audit_a_crafted_name_reaches_no_other_document_through_the_reads` 確認。
7. **遠端呼叫的名稱會變成別的路徑。** `speclink trace a/b` 在遠端原本會送出 `/trace/a/b`（對不到路由），帶 `?` 的名稱會變成查詢參數。修法：`Client::trace` 與 `Client::change_evidence` 把名稱編碼成單一路徑段，Server 收到的就是使用者打的名稱，兩種模式得到同一句找不到的訊息。`speclink-remote` 其餘約 30 個呼叫沒有一起改，理由寫在 tasks.md 的 5.1 底下。
8. **舊版 Server 對 `evidence show` 會回錯的答案。** 舊路由只讀進行中的變更，對已封存或不存在的名稱都回空集合，CLI 會印「沒有記錄」、exit code 0。修法和第 5 項同一個做法：新路由的回應一律帶 `change`，CLI 把回應轉回結果時沒看到它就回報錯誤（D3、失敗表與兩份 delta spec 已同步）。品質關卡指出後問過使用者採用。

**留下的已知限制**

- **`.`、`..` 與空字串在遠端**：這三個名稱沒辦法當成路徑段送出（點名稱不管怎麼編碼都會被當成往上一層，空字串則沒有路徑段），遠端得到的是 Server 的 404 而不是「找不到變更」那一句。兩種模式都是非 0 結束、不印任何記錄。
- **第一版格式的記錄讀成沒有項目**：較舊的封存裡，evidence 只有逐任務的檔案清單，沒有記錄時間與執行者。`evidence show` 的每個項目都要有記錄時間，所以不替它們捏造，讀成「沒有記錄」——這是沿用被取代的舊路由的既有決定。溯源（`trace`）仍然讀得到這些檔案。下一個變更的技能若需要舊封存的檔案清單，再在那裡決定要不要改規格。
- **`created` 的值照 metadata 的原樣輸出**：metadata 裡的 `created` 不是日期（例如手改成 `someday`）時，清單照樣輸出那個字串，和 `speclink show` 既有的行為一致；讀取端要把它當提示，解析失敗就略過。
- **本機檔案儲存的大小寫**：在不分大小寫的檔案系統上 `evidence show DEMO` 會讀到 `demo`。這是檔案儲存「用資料夾探查找變更」的既有行為，所有動詞都一樣，不在這個變更處理。
- **使用者文件還沒跟上**：`docs/verb-contract.md` 仍把 `trace` 與 `plan --strict-overlap` 列為只限本機。正式規格 user-documentation 有一條需求規定那一列要含 `trace`，兩者要一起改，留給變更 docs-embedded-host。
- **宿主的 before hook 現在也會被問到讀 evidence**：evidence 的路由改成執行引擎命令後，和其他讀取一樣會先問 hook（命令名稱 `evidence-show`）。只允許特定命令名稱的宿主要把它加進允許清單。測試 `audit_the_reads_ask_the_before_hook_and_a_refusal_reads_nothing`。
- **找不到名稱時會列出整個封存區**：名稱不是進行中的變更時，要列出已封存變更的清單才找得到「去掉日期後同名」的那一筆。進行中的變更不會走到這一步，成本只落在讀已封存變更與打錯名稱的時候。

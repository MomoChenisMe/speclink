## Context

任務清單的每一行是一個勾選框加一段文字。讀取端把這段文字切成三段來畫：標題、內文、驗證。現況有三個事實：

- 寫入端沒有規則。兩個內建產出流程（spec-driven 與 no-spec-change）的 tasks 指示只要求每個任務寫出行為與驗證方式，共用的任務範本的佔位文字是 `<!-- Behavior/contract delivered + verification target -->`。指示與範本都沒有提短標題。
- 讀取端的規則只有一份實作，在 packages/ui 的 `splitTaskText`：先剝開頭的序號，以首個「驗證：」切出驗證，再以首個全形「：」切出標題與內文，標題尾的 design 括號變成 D 籤。apps/desktop 與 apps/server-web 都經 packages/ui 使用它。
- 這條規則只認中文的標記，而且沒有寫進任何使用者文件。自己畫任務頁的內嵌宿主只能猜。

指示的文字只存在於內建產出流程的正典 YAML（workflow-schemas「instruction 單一來源」），`schema fork` 逐位元組輸出同一份檔案。技能檔不含這段文字，所以這個變更不動技能的版號（`ASSET_VERSION` 只跟著 assets/skills 的 render 內容走）。

限制：任務的解析、勾選、stable ID（行尾註解）與 analyze 的需求名稱覆蓋檢查，都以勾選框那一整行為單位。任何寫法都必須留在同一行。

## Goals / Non-Goals

**Goals:**

- Agent 照指示寫出來的新任務，在 Desktop 與宿主的任務頁都切得出短標題，可以收合。
- 繁體中文、日文、英文各有一組固定的標記，寫入端與讀取端用同一組。
- 切分規則寫進使用者文件，宿主照文件解析，不用猜。
- 原本切得開的任務，切法逐字不變。

**Non-Goals:**

- 不由引擎輸出結構化的任務清單（群組、編號、標題、內文、驗證）。討論 embedded-host-needs-10 把它列為延後。
- `instructions apply --json` 的任務不加 `group` 或其他鍵。
- `validate` 與 `analyze` 不檢查任務有沒有照這個形狀寫。這是寫法的建議，不是格式的守門。
- 不遷移、不改寫已經寫好的任務清單。
- 不改專案層或使用者層的自訂產出流程；它們用自己那一份指示。
- 不改任何技能檔。
- 讀取端不依 locale 選規則。切分只看文字裡出現哪一個標記。
- 標題不獨立成一行。任務的解析以勾選框那一行為單位。

## Decisions

### D1：寫法放在 tasks 指示，三種語言各一組固定標記

兩個內建產出流程的 tasks 指示，在 Guidelines 清單裡加一條「Short title first」，放在「Behavior + verification」那一條之後。它說明每個任務的形狀是「短標題、分隔符號、行為、最後一段是驗證」，並列出三種語言的寫法：

| locale | 形狀 |
| --- | --- |
| 繁體中文 | `<短標題>：<行為>。驗證：<驗證方式>` |
| 日文 | `<短いタイトル>：<振る舞い>。検証：<検証方法>` |
| 英文與其他語言 | `<Short title>: <behavior>. Verify: <how it is verified>` |

這一條另外寫明三件事：照 instructions JSON 的 `locale` 選一組；標題是幾個字的短語、不是句子，裡面不放冒號；標記本身不翻譯、不改寫（和既有的「machine-readable syntax」那一條同一個精神）。指示裡的 Example 區塊改成符合這個形狀的英文任務。no-spec-change 的指示加同一條，它的 Example 同樣改寫；「最後一個任務確認行為未變」那一條不動。

用語是「寫成這個形狀」的直述句，不加 REQUIRED：沒有守門會檢查它，寫成 REQUIRED 會和實際行為不符。

替代方案：
- 引擎依 locale 在指示後面接一句解析過的文字（像 specs 指示的規格語言那一段）。否決：三組標記是固定的清單，靜態文字就夠；動態段落要改 `workspace/instructions.rs`，而且把指示的內容分成兩個來源，違反「instruction 單一來源」。
- 每個範圍自己在 `rules.tasks` 加規則。否決：每個範圍都要加，討論已排除。

### D2：讀取端先找驗證標記，再找標題的分隔符號

`splitTaskText` 的順序不變，只把兩個「找標記」的步驟各自擴充：

1. 剝開頭的序號（不變）。
2. 驗證標記有三個：「驗證：」「検証：」「Verify: 」（大寫 V、半形冒號、一個空白）。依「驗證：」→「検証：」→「Verify: 」的順序，取第一個有出現在文字裡的標記，在它首次出現的位置切：之前是前段，之後是驗證。三個都沒有時沒有驗證層。
3. 前段裡有全形「：」時，以首個全形「：」切標題與內文（不變）。沒有全形「：」時，才以首個「半形冒號加一個空白」切。兩種都沒有時整段是標題。
4. 標題尾的 design 括號照舊處理（不變）。

「Verify: 」與「半形冒號加空白」落在行內程式碼（成對的反引號之間）時不算標記。全形的三個標記維持現況，不看行內程式碼：這樣原本切得開的任務，結果逐字相同。

全形優先於半形是刻意的：一個有全形冒號的中文任務，就算前面出現「status: ok」這種半形寫法，仍然在全形冒號切，和今天一樣。驗證標記的順序也是刻意的：舊版只認「驗證：」，所以只要文字裡有「驗證：」就在那裡切；中文任務的一般文字裡提到「検証：」或「Verify: 」（例如說明這些標記的任務），切法和今天一樣。

替代方案：
- 依 locale 選一組標記。否決：讀取端（已封存的檢視、宿主的畫面）不一定知道任務是用哪個 locale 寫的；看文字比較可靠，也讓混用的清單逐行各自成立。
- 半形冒號不需要後接空白。否決：會切到網址（`http://`）、時間與 Windows 磁碟代號。
- 三個驗證標記一律取最先出現的一個，或「驗證：」與「検証：」取最先出現的一個。否決：中文任務在一般文字裡提到「検証：」或「Verify: 」，又排在「驗證：」前面時，驗證會從那裡切開，原本切得開的任務切法因此改變。

### D3：範本的佔位文字改成同一個形狀

共用的任務範本（spec-driven 的 tasks 範本，no-spec-change 也用它）有兩處要改：開頭那段說明加一句「每個任務的形狀是短標題、分隔符號、行為、最後一段是驗證；各語言的標記見 tasks 指示」；四個任務的佔位文字由 `<!-- Behavior/contract delivered + verification target -->` 改成 `<!-- Short title: behavior/contract delivered. Verify: verification target -->`。範本維持英文、不分語言；各語言的標記只在指示說一次。

### D4：切分規則寫進動詞參考文件

docs/verb-contract 的中英兩份各加一節（錨點 `task-line-shape`），放在講 `show --json` 欄位的段落之後。內容是讀取端的完整規則：行首的勾選框與選用的 `[M]`、序號、D2 的四個步驟、行尾的 stable ID 註解要先拿掉、切不到時的結果，以及一張三種語言的例子表。這一節也說明寫入端的建議在 tasks 指示裡，舊任務不保證符合。

選 verb-contract 而不是 workflow 文件，是因為需要這條規則的讀者是「讀動詞輸出的程式」的作者；workflow 文件講的是人怎麼走流程。

## Implementation Contract

**行為**

- 對 schema 為 spec-driven 或 no-spec-change 的變更取 tasks 指示時，`instruction` 欄位含 D1 的那一條與三種語言的形狀，Example 區塊的任務符合英文的形狀；`template` 欄位的四個任務佔位文字是 D3 的新文字。
- `speclink schema fork spec-driven` 與 `schema fork no-spec-change` 的輸出仍然與正典 YAML 逐位元組相同（同一份檔案，不需要另外處理）。
- `splitTaskText` 的結果（輸入是去掉勾選框的任務文字）：

| 輸入 | number | title | body | verify |
| --- | --- | --- | --- | --- |
| `1.1 匯出 CSV：export --csv 寫出含標題列的檔案。驗證：cargo test -p demo export_csv` | `1.1` | `匯出 CSV` | `export --csv 寫出含標題列的檔案。` | `cargo test -p demo export_csv` |
| `1.2 CSV エクスポート：ヘッダー行つきのファイルを書き出す。検証：cargo test -p demo export_csv` | `1.2` | `CSV エクスポート` | `ヘッダー行つきのファイルを書き出す。` | `cargo test -p demo export_csv` |
| `1.3 Export CSV: the command writes a file with a header row. Verify: cargo test -p demo export_csv` | `1.3` | `Export CSV` | `the command writes a file with a header row.` | `cargo test -p demo export_csv` |
| `2.1 確認輸出為 status: ok（design D2）：補上欄位` | `2.1` | `確認輸出為 status: ok` | `補上欄位` | （無） |
| 2.2 Handle &#96;a: b&#96; keys | `2.2` | Handle &#96;a: b&#96; keys | （無） | （無） |
| `2.3 Run the whole suite` | `2.3` | `Run the whole suite` | （無） | （無） |

  第四列的 D 籤是 `D2`。第五列的冒號在行內程式碼裡，所以不切。

- 既有測試「三層切分」的四個中文例子，結果逐字不變。
- 驗證標記依「驗證：」→「検証：」→「Verify: 」的順序取：`說明「Verify: 」標記：補測試。驗證：npm test` 的標題是 `說明「Verify: 」標記`、內文是 `補測試。`、驗證是 `npm test`；`補日文標記：處理「検証：」。驗證：npm test` 的標題是 `補日文標記`、內文是 `處理「検証：」。`、驗證是 `npm test`。

**介面與資料形狀**

- `splitTaskText(raw: string): TaskTextParts` 的簽名與 `TaskTextParts` 的欄位不變。呼叫端（任務列、任務文件的摘要）不用改。
- 不新增也不變更 CLI 指令、旗標、`--json` 欄位、HTTP 路由與事件。

**失敗模式**

- 沒有失敗路徑：任何文字都切得出結果，最差是整段當標題（今天的行為）。
- Agent 沒照建議寫時，`new artifact tasks`、`validate`、`analyze` 都照常通過。

**驗收**

- speclink-core 的 schema 單元測試：兩份正典 YAML 的 tasks 指示都含三組標記的形狀；任務範本含新的佔位文字、不含舊的佔位文字。
- packages/ui 的 `docModel` 測試：上表六列，加上既有四個例子不變。
- docs/verb-contract 的中英兩份都有 `task-line-shape` 這一節，scripts 的中英對照與連結檢查通過。

**範圍**

- 範圍內：兩份正典 YAML 的 tasks 指示、共用的任務範本、`splitTaskText` 與它的測試、docs/verb-contract 的中英兩份。
- 範圍外：Non-Goals 列的每一項；任務列的版面與展開行為；宿主那一側「沒有冒號時在逗號切」的啟發式做法。

## Risks / Trade-offs

- [回歸對照：指示與範本的文字是既有輸出] → 釘住這兩段文字的只有 speclink-core 的 schema 單元測試，同批更新；render 的 golden 只涵蓋技能檔，不含產出流程的指示，預期不變，若 golden 變紅代表改到了不該改的檔案。CLI 測試裡取 tasks 指示的案例只斷言欄位存在，預期不受影響，仍在任務裡跑一次確認。
- [舊任務的顯示改變] → 沒有全形冒號、但文字裡有「半形冒號加空白」的舊任務，標題會在那個冒號變短。只影響顯示，不改檔案；全形優先的規則讓有全形冒號的任務完全不受影響。這個 repo 的任務都用全形冒號。
- [模型不照建議寫] → 這是建議，沒有守門。指示把形狀放在 Example 與範本兩處，模型看到的每個例子都是同一個形狀；仍然不照寫時，行為退回今天的樣子。
- [跨平台] → 改的是文字資產與 TypeScript 的字串處理，沒有路徑、換行或 git 的行為差異。正典 YAML 以 `include_str!` 內嵌，三個平台讀到同一份位元組。
- [自訂產出流程拿不到建議] → 已在提案的相容性影響寫明；使用者可以自己把那一條加進自己的 schema。
- [三組標記之外的語言] → locale 目前只有 tw、ja、en（以及未設定）；其他語言落在英文那一組，指示寫明。

## Migration Plan

不需要資料遷移。舊的任務清單照舊可讀。回滾時還原這個變更即可；用新形狀寫的任務在舊版的 Desktop 上，中文的照樣切得開，日文與英文的退回整句當標題。

## Open Questions

(none)

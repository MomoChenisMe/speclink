## Context

變更 shared-verb-parsing 完成後，動詞文字到引擎命令只剩一份（`speclink-verbs`）。輸出還是兩份：

- **CLI**：`src/verbs/` 底下約 30 個 `render_*` 函式，加上散在 `cmd_*` 裡的直接印出，共約 300 處 `println!`、十幾處 `eprintln!`。`--json` 的內容多半在 render 函式裡現場組出，再經 `common::print_json`（縮排的 JSON 加換行）印出。顏色經 `color.rs` 的函式，它們讀一個程序層級的開關（`color::init` 依 `--no-color`、`NO_COLOR`、`CLICOLOR`、stdout 是不是終端機決定）。
- **Node SDK**：`verb_list`、`verb_status`、`verb_new`、`verb_claim`、`verb_station` 各自把引擎的結果組成 JSON。

CLI 的本機分支與遠端分支今天已經共用 render 函式：兩邊各自組出同一種結果型別，再交給同一個函式印出。`remote_verb_parity` 的 51 個測試比對兩邊的 stdout 與 stderr 逐位元相同。引擎自己的 golden（`render_golden`）守的是引擎產生的文字，不是 CLI 的 render。

限制：

- CLI 的人眼輸出與 `--json` 都是契約，一個位元組都不能變。
- 引擎不帶 ANSI，這條不變；顏色屬於 adapter 層。
- 這個變更不新增動詞，不改引擎的結果型別。

## Goals / Non-Goals

**Goals:**

- 每個動詞的輸出只有一份定義，CLI 與 SDK 共用。
- 輸出函式是純函式，能在任何程序裡呼叫。
- SDK 的 dispatch 涵蓋全部有引擎命令的動詞，回傳值和 CLI 的 `--json` 同源。

**Non-Goals:**

- 不搬 CLI 自己留下的家族（init、update、link、unlink、auth、schemas、templates、feedback、schema、config、workflow-config、completion、demo）的輸出。
- 不改任何輸出的內容或格式。
- 不把領域事件交給宿主（SDK 今天丟掉事件，這次不變）。
- 不讓只能在有 checkout 的一端執行的動詞在 SDK 可用。
- 不改 CLI 遠端分支組結果的程式（`convert.rs`）。

## Decisions

### D1: render 模組放在 speclink-verbs，一個家族一個檔

`speclink-verbs` 新增 `src/render/`，檔案和 `src/families/` 一一對應。每個動詞提供：

- `<verb>_json(&結果) -> serde_json::Value`：只有在 CLI 有 `--json` 形式的動詞才有。
- `<verb>_human(&結果, &Paint) -> Text`：人眼輸出。`Text` 帶 stdout 與 stderr 兩段文字。

函式的輸入沿用今天 render 函式收的型別（引擎的結果型別，或它的欄位）。CLI 的本機與遠端分支照舊各自組出同一種型別再呼叫，兩模式同形的保證不變。

唯一的實作落點是 `speclink-verbs`。CLI 不留任何一份這十一個家族的輸出。

### D2: 輸出的回傳型別 Rendered

單一出口 `render::render(verb, outcome, options)` 回傳 `Rendered`：

- `stdout`：CLI 要印到 stdout 的完整文字，含結尾的換行。`options.json` 為真時是縮排的 JSON 加一個換行，和今天的 `print_json` 相同。
- `stderr`：CLI 要印到 stderr 的完整文字，多數動詞是空的。
- `json`：這個動詞有 `--json` 形式時是那個值，不論 `options.json` 的真假；沒有 `--json` 形式時是「沒有」。

`options` 是 `{ json, color }`。`render` 裡對引擎結果的 `match` 不帶萬用分支：引擎新增結果型別而沒有輸出時編譯不過。

exit code 不在 `Rendered` 裡。今天「有輸出又要以非零結束」的動詞（例如 validate 有錯誤時），由 `cmd_*` 依結果的內容決定；這段判斷留在 CLI，搬移時一併確認它們不依賴輸出函式的副作用。

### D3: 顏色用 Paint 傳入

`render/paint.rs` 定義 `Paint { enabled }`，提供 bold、dim、red、green、yellow、cyan 與四個粗體顏色的方法。ANSI 色碼只在這裡定義。

CLI 的 `color.rs` 保留 `init` 與開關的判斷，上色的函式改成呼叫 `Paint`，留給沒有搬的家族用。CLI 呼叫輸出函式時把開關的結果放進 `options.color`。SDK 一律傳 `color: false`。

- **否決：輸出函式繼續讀程序層級的開關** — 在宿主的程序裡，那個開關不屬於任何一次呼叫。

### D4: 搬移時逐行改寫，不重排

每個 `println!(…)` 改成往 `stdout` 的字串寫入同樣的內容與一個換行；`print!` 不加換行；`eprintln!` 寫進 `stderr`。格式字串、分支順序、空行都照原樣。`print_json(v)` 改成「`json` 設為 v，`options.json` 為真時 `stdout` 是它的縮排字串加換行」。

散在 `cmd_*` 裡、不在 render 函式裡的印出，凡是屬於這十一個家族而且內容來自引擎的結果，一併搬進對應的輸出函式。來自 CLI 自己的事實的印出留在 CLI，清單在實作時盤點並寫進任務的完成說明，預期至少有：技能檔過期的提醒（`warn_if_station_skills_stale`）、規格資料夾位置的提醒（`warn_spec_layout`、`warn_leftover_remote_file`）、沒有任何變更時的提示（`info_if_no_changes`）、封存的批次迴圈的彙總。

### D5: 守門測試 no_direct_print

`crates/adapters/speclink-verbs/tests/no_direct_print.rs` 掃描 `speclink-verbs` 的 `src/` 全部檔案（遞迴，不只頂層），出現 `println!`、`print!`、`eprintln!`、`eprint!`、`dbg!` 就失敗。測試程式碼區塊（`#[cfg(test)]`）也不例外。

### D6: 輸出逐位元相同的對照

既有的對照：`cargo test -p speclink-cli` 的整合測試，尤其 `remote_verb_parity`。

另外在動手前建立一份輸出基準（放在暫存位置，不進 repo）：在一個固定內容的測試 workspace 上，用搬移前的 CLI 對十一個家族的每個末端動詞，以人眼、`--json`、`--no-color` 三種形式各執行一次，存下 stdout、stderr 與 exit code。workspace 的內容要讓每個動詞都有非空的輸出，並含失敗的情境（不存在的變更、驗證有錯誤）。每搬完一批，用新的 CLI 重跑並比對，必須逐位元相同。強制上色（`CLICOLOR_FORCE`）的形式另外跑一次，確認色碼的位置不變。

會改動儲存的動詞在 workspace 的複本上執行，每次從同一份初始內容開始。輸出裡的日期與時間用固定的環境或在比對前遮掉，做法寫在腳本裡。

### D7: Node SDK 的 dispatch 涵蓋全部有引擎命令的動詞

`run_dispatch` 變成一條路徑：`parse` → `into_command` → 引擎的 `execute` → `render::render`（`color: false`）→ 回傳值。

- 回傳值：`Rendered.json` 有值時回傳它；沒有時回傳 `{ output: stdout }`。
- 移除 9 個動詞的允許清單，以及 `verb_list`、`verb_status`、`verb_new`、`verb_station` 裡組結果的程式。
- `LocalOnly` 的動詞仍回傳 `invalid_argv`。
- 三段 Node 專屬的邏輯保留，並各有測試釘住：`claim` 經宿主儲存的方法執行（不經引擎命令）；蓋章前檢查宿主儲存有沒有三個選填方法，以及蓋章的互斥鎖；`status` 在沒有任何變更時的輸出。
- 原本 9 個動詞的回傳值要和變更前相同。實作時逐一比對：共用的 `_json` 函式得到的值和 Node 原本組出的值不同時，以 CLI 的 `--json` 為準，並把差異列為相容性影響回報；預期沒有差異，因為正式規格要求兩者對齊。

`index.d.ts` 的 `dispatch` 簽名不變，文件註解改成「全部有引擎命令的動詞」。

### D8: 分三批搬移

和 shared-verb-parsing 相同的三批：第一批 query、checks、documents、trace、instructions；第二批 new、progress、lifecycle、plan；第三批 discuss、station。每一批：搬輸出函式、CLI 改成呼叫並印出、跑 D6 的比對與 CLI 測試。三批都完成後才改 Node。

## Implementation Contract

**行為**

- CLI：十一個家族的每個動詞，在人眼、`--json`、`--no-color`、強制上色四種形式下，stdout、stderr 與 exit code 和變更前逐位元相同。
- SDK：`dispatch` 接受全部有引擎命令的動詞。有 `--json` 形式的動詞回傳和 CLI `--json` 相同內容的物件；沒有的回傳 `{ output }`，`output` 和 CLI 在 `--no-color` 下的 stdout 相同。
- 之後新增動詞時，在 `speclink-verbs` 加輸出函式，CLI 與 SDK 同時得到輸出。

**介面**

- `speclink_verbs::render::{render, Rendered, RenderOptions, Paint, Text}`，以及各家族公開的 `_json`、`_human` 函式。
- `speclink-node` 的 JS 介面不變。

**失敗的樣子**

- 輸出函式不回傳錯誤、不 panic：它們只把已經成功的結果轉成文字。JSON 序列化失敗的情況在型別上不存在時用不到錯誤回傳；存在時回傳引擎的錯誤型別，分類是一般錯誤。
- 引擎回報的錯誤不經輸出函式：CLI 照舊印出訊息並以非零結束，SDK 照舊以帶 `code` 的 Error 拒絕。

**驗收**

- `cargo test -p speclink-verbs` 通過，含 `no_direct_print` 與每個輸出函式至少一個斷言 stdout 全文的測試。
- `cargo test -p speclink-cli` 全部通過。
- D6 的比對：全部動詞、四種形式逐位元相同。
- `npm --prefix crates/adapters/speclink-node run build` 後 `npm --prefix crates/adapters/speclink-node test` 全部通過，含：原本 9 個動詞的回傳值不變；新可用的動詞（至少 show、validate、archive、discuss 的 new／add-round／conclude／list／show、task done、review show、review discard）回傳值和 CLI 的 `--json` 相同；`drift` 仍被拒絕。
- 在 `speclink-cli` 的十一個家族檔案裡搜尋不到 `render_` 開頭的函式定義。

**範圍**

- 範圍內：`speclink-verbs` 的 render 模組；CLI 十一個家族改成呼叫並印出；SDK 的 dispatch。
- 範圍外：引擎的結果型別；CLI 留下的十三個家族的輸出；流程設定；事件；server；使用者文件。

## Risks / Trade-offs

- [回歸對照] 約 300 處印出逐行改寫，漏一個換行就改變輸出 → D6 的四種形式逐位元比對，加上 `remote_verb_parity` 與 CLI 的整合測試。引擎的 golden 不受影響。
- [跨平台] 輸出函式只組字串，不碰路徑分隔與終端機；既有程式裡若有依平台不同的換行或路徑呈現，原樣搬移，不統一。D6 的比對在同一台機器前後對照；Windows 的 CI 跑同一批 CLI 測試。
- [stdout 與 stderr 的交錯順序] 分成兩段字串後，CLI 先印哪一段會影響終端機上看到的順序 → 兩者是不同的串流，測試與使用者的管線都分開讀；CLI 維持「先 stdout 再 stderr」或原本的順序，搬移時逐一確認並在比對腳本裡分開存。
- [守門的掃描面] `no_raw_wire_json` 的逐行允許清單會因搬移而對不上 → 同批更新；新的 `no_direct_print` 遞迴掃描，不靠檔案在哪一層。
- [SDK 行為放寬] 原本被拒絕的動詞現在會寫入 → 列為相容性影響；宿主要用自己的權限檢查決定開放哪些動詞（執行前的掛鉤在後面的變更提供）。
- [Node 原本組出的 JSON 與 CLI 不同] 若有差異，SDK 的回傳值會變 → D7 規定逐一比對並回報，不靜默改變。
- [大量搬移與其他進行中的變更衝突] 這十一個檔案被別的變更同時修改時合併困難 → 這個變更排在 shared-verb-parsing 之後立刻做，並在 worktree 裡進行；開工前先看有沒有別的變更碰這些檔案。

## Open Questions

- 散在 `cmd_*` 裡的印出有哪些屬於「CLI 自己的事實」，D4 列了預期的四類，完整清單要等盤點。
- `status` 在沒有任何變更時，CLI 與 SDK 的輸出是否相同；不同時這次維持各自的行為，並在完成說明裡記下。

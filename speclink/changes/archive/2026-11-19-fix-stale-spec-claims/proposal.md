## Why

正式規格有四條需求沒有跟上後來的變更。它們和較晚的規格互相矛盾，也和程式的實際行為不符。

正式規格是技能與手冊的唯一依據，所以錯的字句會被照抄出去。2026-11-19 重生操作手冊時就發生了一次：手冊照 change-plan 的字句，寫出「遠端專案不能用 plan --strict-overlap」這句錯的話，逐條對照之後才改正。同一次對照另外找到三條同類的字句。

受影響的人是透過 AI agent 開發的開發者，以及連到遠端專案的工程師。情境是任何以正式規格為準的站：propose 的規格掃描、manual 的手冊生成、trace 的溯源，以及人直接讀規格的時候。

掃過的相關正式規格：

- 要改的四份：change-lifecycle、change-plan、discuss-skill、user-documentation。
- 較晚、說法正確、這次不改的三份：verb-contract（動詞的模式歸屬與兩模式同形）、discussion-docs（討論以 restore 動詞搬回進行中）、server-verb-api（discard 語意的刪除端點）。

四條過期字句與證據：

| 規格與需求 | 現在寫什麼 | 較晚的規格與程式怎麼說 |
| --- | --- | --- |
| change-lifecycle「變更以 discard 動詞廢棄」 | remote 模式下 discard 回報不支援 | verb-contract 把 discard 列為兩種模式都能用，並有情境「remote discard 的 guard 語意與本地一致」；CLI 的整合測試 remote_discard_without_force_translates_the_guard_message 與 remote_discard_with_force_succeeds_with_fs_parity_json 守著這個行為 |
| change-plan「plan 動詞輸出波次與阻擋清單」 | --strict-overlap 只限本機模式，remote 模式以固定訊息拒絕 | verb-contract「模式分岔的單點宣告」（2026-11-12）規定這一族動詞沒有只限本機的子指令或旗標；CLI 的整合測試 remote_plan_strict_overlap_asks_the_server 守著這個行為 |
| discuss-skill「中途轉出教學」 | 討論被誤封存時，把記錄檔從封存資料夾手動搬回來 | 同規格「正式規格與變更文件一律經動詞讀取」與 discussion-docs（都是 2026-11-18）規定用 speclink discuss restore，技能不得指示搬檔；技能原稿已經照新的寫法 |
| user-documentation「已知缺口逐項列出」 | 已知缺口至少要列「沒有把封存的討論記錄搬回來的動詞」「沒有寫入詞彙表的動詞」 | 這兩個動詞已經由變更 language-replace-and-discuss-restore 做完；專案能力狀態的中英兩版已經把這兩項移到「已經解決」的清單 |

## What Changes

這個變更只改正式規格的字句，讓四條需求說出程式今天的行為。不改程式、測試、技能原稿與使用者文件。

- **change-lifecycle「變更以 discard 動詞廢棄」**：拿掉「remote store 模式下回報不支援」。改成：remote 模式下 discard 經 Server 執行，動工痕跡的守門、帶 --force 的放行、人眼輸出與 --json 輸出都和本機相同。情境「remote store 模式不支援」換成兩條：沒帶 --force 被守門拒絕，以及帶 --force 成功。
- **change-plan「plan 動詞輸出波次與阻擋清單」**：最後一句從「--strict-overlap 只限本機模式」改成「--strict-overlap 在 remote 模式同樣可用，CLI 把旗標送到 server，輸出與本機同形」。新增一條情境。
- **discuss-skill「中途轉出教學」**：救援路徑從「把記錄檔搬回來」改成「執行 speclink discuss restore」。情境「分期轉出帶 --hold」裡的同一句跟著改。
- **user-documentation「已知缺口逐項列出」**：從「至少要列」的清單拿掉兩項已經解決的缺口。補一句規則：已經解決的缺口移到這一節表格下方的清單並寫明解法，不直接刪掉。新增一條情境。

相容性影響：

- 沒有新增或變更任何 CLI 子指令、旗標、stdin 或 exit code。人眼輸出與 --json 輸出都不變，回歸對照不受影響。
- 不涉及設定欄位。
- 不涉及生成的技能：discuss 技能的原稿早已用 speclink discuss restore，這次不改原稿，內嵌資產的版本號不前進，claude 與 codex 的技能檔都不會變。
- 這個變更不新增也不改任何引擎命令、動詞、文件種類或技能步驟，所以沒有三種形態的新結果要寫。規格改成的兩個行為（discard、plan --strict-overlap）在本機 repo 與連到遠端的 checkout 早已可用，由上表的測試守著；沒有本機規格檔的內嵌宿主不在這四條需求的字句範圍內，這次不增不減。

封存之後，操作手冊有幾頁會被標為可能過期（執行順序、實作、認識資料、討論，以及來源頁記的三條矛盾）。那是預期的結果：再執行一次 /speclink-manual 就會對齊。

## Non-Goals

- 不改程式、測試、技能原稿與 docs。四條需求的正確行為都已經有測試或既有文件。
- 不改任何規格的 Purpose 段落。change-plan、remote-auth、workflow-config、board-card-order、workflow-schemas、ingest-skill 的 Purpose 也有沒跟上的句子，但它們不是需求，也不是這個系列留下的。
- 不處理操作手冊來源頁列的其他舊矛盾（例如 remote-connection 仍寫 Claude／Codex 兩個工具、desktop-app 幾處舊的按鈕字面）。那些要逐條先確認程式的行為，另開變更。
- 不在這個變更裡重生手冊。手冊只取材已封存的正式規格，要等封存之後才讀得到新的字句。
- 不寫 design：沒有設計上的取捨，四處都是把字句改成已經存在的行為。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `change-lifecycle`: 「變更以 discard 動詞廢棄」改成 remote 模式下可用、守門與輸出和本機相同。
- `change-plan`: 「plan 動詞輸出波次與阻擋清單」改成 --strict-overlap 在 remote 模式可用。
- `discuss-skill`: 「中途轉出教學」的救援路徑改成 speclink discuss restore。
- `user-documentation`: 「已知缺口逐項列出」拿掉兩項已解決的缺口，並規定已解決的缺口留在表格下方的清單。

## Impact

- Affected specs: change-lifecycle、change-plan、discuss-skill、user-documentation
- Affected crates: 無。speclink-core、speclink-cli、speclink-host、speclink-server、speclink-node 與 apps/desktop 的程式碼都不改。
- Affected code:
  - New: (none)
  - Modified: (none)
  - Removed: (none)

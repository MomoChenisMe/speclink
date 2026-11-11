## Context

**技能的現況**

- 技能的原稿在 `speclink-core` 的 assets/skills 底下，一支一個檔。渲染是純文字取代：規格資料夾的佔位符、工具名稱、斜線指令的寫法。沒有條件語法，也沒有「本機或遠端」的分支。自訂工具有兩種呼叫形式（CLI 與 tool-call），本文相同，只差開頭一段說明。
- 遠端專案初始化時，技能裡的規格資料夾一律渲染成 `speclink/`，即使 checkout 裡沒有這個資料夾。
- 守門測試 `skill_verbization` 掃全部渲染後的技能，擋的寫法只有一種：`read`、`glob`、`open` 後面跟著規格資料夾底下的三類路徑或詞彙表。它看不到 discuss 的「`path` 加 `spec.md`」、沒說方法的「讀 artifacts」「直接修正」、以及 `config.yaml`。
- `list --specs --json` 的每一筆有 `id` 與 `path`。在本機 `path` 是資料夾的絕對路徑；在遠端它是儲存的鍵（`specs/<capability>`）。`show <capability>` 回傳整份規格，Purpose 在最前面；沒有只回 Purpose 的形式。
- `new artifact … --force` 會整份覆寫既有的 artifact。

**`change rank` 的現況**

- 動詞的解析在 `speclink-verbs`，轉成命令時回傳「只能在有 checkout 的一端執行」。CLI 的 dispatch 把它宣告成只限本機，遠端直接拒絕、不發請求。
- 實作是引擎的 `set_rank`：它從各變更的 metadata 讀出順序鍵，算出要寫的鍵，經呼叫端給的寫入函式逐一寫出。CLI 的寫入函式會把每個變更的鍵寫到它自己的副本（有 worktree 就寫 worktree 的副本）。
- 遠端的順序不存在 metadata：範圍有一份看板順序文件（鍵 `board/order`，內容是變更與討論各一張「名稱對順序鍵」的表，格式定義在 `speclink-protocol`）。Speclink Desktop 拖排時自己組出整份文件，帶版本寫回。`plan` 的命令可以收一張外部的順序表，示範用 server 的 plan 路由就是把這份文件解析後傳進去。
- 引擎沒有「排序」的命令。

限制：

- 技能原稿有機械守門：每個檔不超過 500 行、強調字（`MUST`、`NEVER` 等）有逐檔的上限且多半剛好用滿、不能寫死斜線指令與工具名稱。
- 改技能原稿要同時前進資產版號、重新產生 golden 快照與 lock 檔。
- discuss 在 Claude Code 上禁用編輯工具，所以它本來就不能直接改檔。

## Goals / Non-Goals

**Goals:**

- discuss、propose、ingest 三支技能裡，每一個讀或寫規格文件的步驟都指名一個動詞。
- 這三支技能不再叫 agent 用清單回傳的路徑開檔。
- `speclink change rank` 在本機、遠端專案、內嵌宿主三種形態都能執行，而且只有一份排序的實作。
- 本機的行為與輸出逐位元不變。

**Non-Goals:**

- 其餘技能與守門的全面加嚴（變更 skills-verbs-second-wave）。
- 搬回封存討論記錄的動詞、寫入詞彙表的動詞。
- Desktop 的遠端拖排改用新命令。
- 技能渲染的條件語法。

## Decisions

### D1: 技能文字的改法

逐處列出，行號不寫，用原文的關鍵句定位。

**discuss**

| 原文的意思 | 改成 |
|---|---|
| 使用者提到某個變更時「讀它的 artifacts」 | 用 `speclink show <name> --json` 看它有哪些文件，用 `speclink artifact cat <artifact> --change <name>` 讀內容 |
| 讀候選規格的 Purpose：「每一筆的 `path` 是資料夾，裡面的 `spec.md` 有 Purpose」 | 對每個候選執行 `speclink show <capability>`，讀輸出最前面的 Purpose；加一句：清單的 `path` 只是識別用，不要拿來開檔 |
| 「需要讀整份規格時」 | 同一個 `speclink show <capability>` 的輸出 |
| 把封存的討論記錄搬回來（搬檔案、去掉日期前綴） | 加條件：規格資料夾在這個 checkout 裡才照做；不在時停下來告訴使用者，這一步要由能直接操作規格儲存的人在 Server 上處理 |

**propose**

| 原文的意思 | 改成 |
|---|---|
| 從討論文件（路徑）擷取結論 | 從 `speclink discuss show <slug> --json` 的 `content` 擷取 |
| 讀記錄的結論與 frontmatter 的 `promoted_to`（沒說來源） | 同上 |
| `outputPath`：「artifact 要寫到哪裡」 | 「引擎存放它的位置，僅供顯示；寫入一律用 `speclink new artifact`」 |
| 「專案在設定檔設了 `spec_locale` 時」 | 「`speclink workflow-config show --json` 的 `specLocale` 有值時」 |
| 自我檢查後「直接修正」、analyze 之後「修正受影響的 artifact」、validate 失敗「修正後重驗」 | 用 `speclink artifact cat` 讀出全文，改好後用 `speclink new artifact <id> --change <name> --stdin --force` 整份寫回（delta spec 用 `new artifact spec <capability>`） |
| 「讀其他進行中變更提案的 Impact」 | `speclink artifact cat proposal --change <other>`；清單裡那個變更帶 `worktree` 時從它的 `worktree.path` 執行 |
| 把封存的討論記錄搬回來 | 和 discuss 相同的條件句 |

**ingest**

| 原文的意思 | 改成 |
|---|---|
| 「更新前先讀既有的 artifacts」 | `speclink artifact cat <artifact> --change <name>` |
| 「把新內容併進既有的提案」「新增任務並保留已完成的項目」「不要移除既有內容」 | 讀出全文 → 在全文上修改 → 用 `speclink new artifact <id> --change <name> --stdin --force` 整份寫回。任務清單另加一句：既有的每一行任務原樣保留，包含勾選狀態與行尾的識別註解 |
| 「專案在設定檔設了 `spec_locale` 時」 | 同 propose |
| 自我檢查與 analyze 之後的修正 | 同 propose |
| 「讀其他進行中變更提案的 Impact」 | 同 propose |

寫法上的約束：不增加強調字（每個檔的上限已滿）；整份寫回的說明在每支技能只寫一次，其他步驟引用它；三個檔都維持在 500 行以內。

「整份寫回」在兩種情況會被引擎拒絕，技能要寫明遇到時停下來告訴使用者，不要改用編輯檔案：引擎以錯誤拒絕覆寫時（例如任務已全部完成的清單）。實作時用測試列出引擎實際會拒絕的情況，技能的那一句照實際的訊息寫。

- **否決：新增只回 Purpose 的動詞形式** — propose 已經用 `show` 讀整份再看開頭；多一個形式要多一組線上格式。
- **否決：依形態渲染兩份文字** — 渲染沒有條件語法，而且改經動詞之後三種形態的步驟本來就相同；只有「搬回記錄」一處需要條件。

### D2: 這次加的守門

`skill_verbization` 新增一個測試，只掃 discuss、propose、ingest 三支，掃 Claude 與自訂工具 tool-call 兩種渲染結果：

- 不得出現「`path` 與 `spec.md` 在同一行」的句子。
- propose 與 ingest 不得出現 `config.yaml`。
- 每一支都必須出現 `speclink artifact cat`；propose 與 ingest 必須出現 `--stdin --force`；discuss 必須出現 `speclink show`。
- 含「Move the record file」的那一行必須同時含條件的關鍵字（checkout）。

這是針對這次改動的回歸檢查，不是通用的規則；通用的規則在下一個變更定。

### D3: 引擎命令 ChangeRank

`speclink-core` 新增 `command/change_rank.rs`：

- `Command::ChangeRank { name, anchor, side, force }`，`side` 是前或後。
- 結果 `ChangeRankOutcome { change, anchor, position }`，和今天 `--json` 的三個欄位相同。
- 事件 `DomainEvent::ChangeRanked { change }`，種類名 `change-ranked`。
- 執行：呼叫既有的 `set_rank`。兩個輸入改由儲存提供（D4）：順序鍵的來源，以及寫入函式。

守門的順序、錯誤訊息、整欄補上順序鍵、取中點、依賴檢查都在 `set_rank` 與它呼叫的函式裡，這個變更不改它們的邏輯，只把「讀順序鍵」從固定讀 metadata 改成問儲存。

`speclink-verbs` 的 `change rank` 轉成命令時改成回傳 `ChangeRank`，不再回傳「只能在有 checkout 的一端執行」。輸出函式新增 `ChangeRankOutcome` 的分支，文字與 `--json` 和今天 CLI 印的相同（這段輸出今天在 CLI，跟著搬進共用的輸出）。

### D4: 順序鍵由儲存決定讀寫的位置

引擎內層的 `Store` trait 新增兩個方法，都有預設實作：

- `board_ranks(&self) -> BTreeMap<String, String>`：全部進行中變更的順序鍵。預設實作讀各變更 metadata 的 `board_rank`（今天的 `meta_ranks`）。
- `write_board_ranks(&self, ranks: &[(String, String)]) -> Result<()>`：寫入一批。預設實作對每一筆呼叫既有的 metadata 文字手術（純量經 `yaml_scalar` 跳脫），依給的順序逐一寫。

各儲存：

| 儲存 | 讀 | 寫 |
|---|---|---|
| 檔案實作、測試儲存、Node 的 JS 儲存橋接 | 預設（metadata） | 預設（metadata） |
| Host 的儲存視圖 `PortView` | 讀 `board/order`，用 `speclink-protocol` 既有的寬鬆解析取出變更那張表；文件不存在時是空的 | 讀出整份 → 更新變更那張表的對應項目 → 整份寫回 `board/order`，暫存成一筆寫入 |

寫入順序與失敗後的狀態：

- **儲存接口上**：一次命令的全部順序鍵（補上的與被移動的）是對同一份文件的一筆寫入，和事件在同一次提交。失敗時什麼都沒寫。預期版本是這次讀到的版本；Desktop 同時拖排時，後到的一方收到版本衝突。
- **本機（預設實作）**：和今天相同——補上的鍵逐一寫入，最後寫被移動的那個；中途失敗就停止，已寫的保留。這是既有規格明寫接受的半套（部分補上的欄仍是合法順序，重跑可以繼續）。

`PortView` 整份寫回時，討論那張表原樣帶回。格式裡它不認得的欄位會被丟掉；這份格式由 `speclink-protocol` 定義，目前只有兩張表。

- **否決：遠端也把順序鍵寫進各變更的 metadata** — 遠端的順序正典是看板順序文件，Desktop 與 plan 都讀它；寫進 metadata 會出現兩個來源。
- **否決：CLI 在用戶端組出整份文件再寫回（和 Desktop 相同）** — 內嵌宿主的 Agent 工具沒有 HTTP 用戶端，它需要的是引擎命令；而且會是排序邏輯的第二個落點。
- **否決：把寫入函式繼續當成命令的參數** — 命令要能從線上與 Node 的 dispatch 建出來，參數必須是資料。

### D5: 本機的 CLI 路徑不改

本機的 `change rank` 維持今天的寫法：CLI 直接呼叫 `set_rank`，傳入會依 worktree 選副本的寫入函式。理由是「哪個變更的鍵寫到哪個副本」是只有 checkout 知道的事實，而且這條路徑的輸出與寫入必須逐位元不變。

所以三條路徑用的是同一個 `set_rank`：本機 CLI 直接呼叫；遠端與內嵌宿主經 `Command::ChangeRank`。

### D6: 線上的路由與 CLI 的遠端分支

- `speclink-protocol`：`RankChangeRequest { anchor, position, force }` 與 `RankChangeResponse { change, anchor, position }`，camelCase；`position` 的值是 `before` 或 `after`，`force` 可省略、省略是假。
- 示範用 server：`POST /projects/{key}/changes/{name}/rank`，經既有的執行入口執行 `ChangeRank`，回應帶新的範圍版本。它是寫入，所以受既有的編輯者檢查。事件 `change-ranked` 在推送的分類裡歸到該變更。
- `speclink-remote`：`Client::rank_change(name, anchor, position, force)`。
- CLI：dispatch 的宣告把 `change rank` 從只限本機改成兩種模式都有；遠端分支呼叫上面的函式，再用共用的輸出函式印出結果。拒絕的錯誤（不同階段、已有順序鍵、跨依賴）由 Server 以既有的 `refused` 回來，CLI 印出同一句訊息並以非 0 結束。

和本機的一個差別：本機的錯誤「缺 metadata」會帶出檔案路徑；遠端沒有檔案，訊息由儲存視圖的錯誤決定。這屬於既有規格列的「兩種模式輸出的已知差異」那一類，規格的情境只對本機斷言路徑。

### D7: Node SDK

`dispatch` 經共用的解析與輸出自動得到 `change rank`。JS 儲存橋接用預設實作（讀寫 metadata），需要既有的選填方法 `readChangeMeta` 與 `writeChangeMeta`；沒有提供時，以和蓋章相同的方式在動手前拒絕。

## Implementation Contract

**行為**

- 三支技能渲染後的文字符合 D1 的每一列，並通過 D2 的檢查與既有的守門。
- 本機：`speclink change rank` 的 stdout、stderr、exit code 與寫出的檔案，對 change-plan 既有的每個情境逐位元不變。
- 遠端專案：`speclink change rank c --before a` 成功時 stdout 是 `✓ c ranked before a`，`--json` 是 `{ "change", "anchor", "position" }`；Server 的看板順序文件裡 c 的鍵小於 a 的鍵；`speclink plan --json` 的順序 c 在 a 前；沒有任何變更的 metadata 被改寫。
- 經儲存接口執行 `ChangeRank`：儲存收到正好一次提交，內含對 `board/order` 的一筆寫入與一筆 `change-ranked` 事件。
- 被拒絕的情況（錨點不存在、同一個變更、不同階段、已有順序鍵而沒有強制、跨依賴）：錯誤碼 `refused`（找不到是 `not_found`），訊息和本機相同，儲存沒有提交。

**介面與資料形狀**

- 引擎：`Command::ChangeRank`、`ChangeRankOutcome`、`DomainEvent::ChangeRanked`、`Store::board_ranks`、`Store::write_board_ranks`。
- 線上：`POST /projects/{key}/changes/{name}/rank`，請求與回應見 D6。
- 技能資產的版號前進一版；golden 快照與 lock 檔同批更新。

**失敗的樣子**

| 情況 | 結果 |
|---|---|
| 遠端：Desktop 同時改了看板順序文件 | `revision_conflict`；CLI 顯示既有的衝突訊息，非 0 |
| 遠端：舊版 Server 沒有這條路由 | 404；CLI 顯示既有的找不到訊息，非 0 |
| 遠端：使用者不是編輯者 | 既有的權限錯誤 |
| 看板順序文件的內容無法解析 | 當成空表（既有的寬鬆解析），命令照常執行並寫回合法的內容 |
| Node 的 JS 儲存沒有 metadata 的兩個選填方法 | `refused`，動手前拒絕 |

**驗收**

- `cargo test -p speclink-core --test it skill_verbization::` 與 `skill_authoring::` 通過；`cargo test -p speclink-core` 的 golden 與 lock 已更新並通過。
- `cargo test -p speclink-core`：`ChangeRank` 在測試儲存上涵蓋 change-plan 既有情境裡不涉及 worktree 的每一個。
- `cargo test -p speclink-host --test port_rank`：儲存接口上的一次提交、補上順序鍵、衝突。
- `cargo test -p speclink-cli --test it plan_verbs::` 既有的本機測試不改斷言（除了「遠端拒絕」那一個改寫）；`remote_change_rank::` 涵蓋遠端。
- `cargo test -p speclink-server`、`cargo test -p speclink-remote`、Node 的測試通過。

**範圍**

- 在範圍內：三支技能的原稿、對應的守門、`change rank` 的命令與遠端路徑。
- 不在範圍內：其他技能的原稿、桌面 app、`plan` 的任何行為、使用者文件。

## Risks / Trade-offs

- [回歸對照] 本機 `change rank` 不能變。→ 本機路徑不改（D5）；既有的 `plan_verbs` 測試不改斷言。把輸出搬進共用的輸出函式時，先存下今天的輸出再比對。
- [回歸對照] 技能的 golden 快照會整批改變。→ 這是刻意的變更，同批更新；用 diff 確認只有三支技能與版號行變了。
- [強調字上限] 改寫句子時容易帶入 `NEVER`、`MUST`。→ D1 明寫不增加；`skill_authoring` 會擋。
- [跨平台] 新增的程式沒有路徑與程序呼叫。技能文字裡的 here-document 範例沿用既有的寫法。→ 沒有新的平台差異。
- [兩條遠端排序的路並存] Desktop 的拖排與新的命令都寫同一份文件。→ 兩者都帶版本比對；格式相同（同一個型別）。列在 Non-Goals，之後收成一條。
- [模型不照做] 技能改成「整份寫回」後，agent 在本機仍可能直接改檔。→ 本機直接改檔的結果和整份寫回相同，不會壞；遠端與內嵌沒有檔案可改，只能照技能做。
- [儲存解耦] 順序鍵的位置從寫死 metadata 變成由儲存決定，是朝「引擎不知道儲存媒介」的方向走一步。

## Migration Plan

沒有資料搬移。使用者執行 `speclink update` 取得新的技能檔。要退回時還原這個變更的 commit 並重新執行 `speclink update`。

## Open Questions

- `new artifact … --force` 實際會在哪些情況拒絕覆寫（已知：任務全部完成的清單）。實作時用測試確認後，把技能裡的那一句寫成和實際相符。

  **已確認**（任務 1.2，測試在 `speclink-core` 的 `lifecycle::newcmd::tests::force_write_back_*`）：

  | 情況 | 結果 |
  |---|---|
  | 覆寫既有的提案與設計 | 成功，內容整份換成新的 |
  | 覆寫部分任務已勾選的任務清單 | 成功；既有任務行（勾選狀態與行尾識別註解）逐字不變，只有新加的任務行拿到新的識別註解 |
  | 覆寫任務全部勾選的任務清單 | 拒絕，什麼都沒寫；訊息是 `Tasks must contain at least one checkbox (- [ ])`，錯誤碼 `invalid_argv` |
  | 覆寫已開工變更的 delta spec | 成功；引擎不看變更是否已開工 |

  其餘既有的拒絕不是「整份寫回」特有的：內容缺必要區段（提案沒有 `## Why`、設計沒有 `## Context`、delta 沒有操作區塊）、metadata 壞掉、capability 不在正典又沒帶 `--new`。技能的那一句因此寫成：「`new artifact` 以錯誤拒絕寫回時（例如任務已全部勾選的清單），停下來把訊息告訴使用者，不要改成直接編輯檔案」。
- 既有規格裡「以新順序重跑依賴檢查」在儲存接口上讀的是哪一份順序：預期是看板順序文件。實作時用測試確認 `set_rank` 內部每一處讀順序鍵的地方都改成問儲存，沒有遺漏直接讀 metadata 的地方。

  **已確認**（任務 2.2、2.3）：`set_rank` 的路徑上只有 `move_start` 一處讀順序鍵，已改成 `store.board_ranks()`；之後的欄序、補章與依賴檢查都用這張表推算，不再讀 metadata。`speclink-host` 的 `port_rank` 測試確認：經儲存接口執行時，順序鍵只寫進 `board/order`，任何變更的 metadata 都沒有被寫入。`plan`、`change depends` 與 `board_order` 等其他讀法維持讀 metadata，不在這個變更的範圍內。

  實作上的兩個細節：
  - 引擎命令讓 `set_rank` 照原樣執行，只把它的寫入先收集起來，最後一次交給 `Store::write_board_ranks`。`set_rank` 本來就在第一筆寫入之前做完全部檢查，所以結果相同。
  - `Store` 多了一個父 trait `AsDynStore`（每個有固定大小的實作都自動取得），讓 `write_board_ranks` 的預設實作能把自己交給吃 `&dyn Store` 的 `set_board_rank`。實作者不必改任何程式。

## 稽核結論

任務 4.1 以 sharp-edges 清單（`speclink instructions --skill audit` 的 Discipline Mode）檢查新增的介面。結論：沒有需要修正的問題；下列每一項都有測試釘住。

| 檢查項目 | 結論 | 釘住它的測試 |
|---|---|---|
| 遠端的排序請求不能繞過編輯者檢查 | 新路由掛在專案路由上，由既有的寫入守門統一擋下 reader（403），處理函式不會執行 | `speclink-server` 的 `api::plan_api::a_reader_cannot_rank`；`api::write_guard` 的路由清單已加上 `POST changes/demo/rank` |
| 名稱與錨點不能帶出範圍以外的鍵 | `set_rank` 只接受作用中的變更名稱（帶 `/`、`..` 的名稱讀作不存在）；儲存接口上只寫 `board/order` 一個固定的鍵，變更名稱只是文件裡的 JSON 鍵，由 serde 跳脫 | `speclink-host` 的 `port_rank::a_name_that_leaves_the_changes_folder_writes_nothing`；`speclink-core` 既有的 `rank_refuses_unknown_archived_or_corrupt_names` |
| 看板順序文件無法解析時不 panic、不清空討論的順序 | 變更那張表沿用 `BoardOrderDoc::from_content` 的寬鬆讀法（壞內容讀成沒有順序鍵）；討論那張表另外讀，只要它本身是合法的表就原樣寫回，即使變更那張表的型別是錯的 | `port_rank::unusable_board_content_reads_as_no_ranks_and_is_rewritten`、`port_rank::the_discussion_table_is_kept` |
| 寫進 metadata 的預設實作經過純量跳脫 | 預設實作呼叫既有的 `set_board_rank`：順序鍵必須全是小寫字母、名稱必須是單一路徑段，比跳脫更嚴格；其餘位元組逐字保留 | `speclink-core` 的 `command::change_rank::tests::write_board_ranks_refuses_a_key_that_is_not_a_plain_scalar`、`write_board_ranks_defaults_to_meta_writes_in_order` |
| 守門失敗的每一條路徑都沒有寫入 | 不存在的變更在 `set_rank` 之前就以 `not_found` 結束；其餘守門都在 `set_rank` 的第一筆寫入之前；命令只在 `set_rank` 成功後才把收集到的寫入交給儲存；儲存接口上命令失敗時暫存的寫入整批丟棄 | `change_rank::tests` 的各個拒絕情境（寫入次數為 0）、`port_rank::a_refused_move_commits_nothing`、`plan_api::an_absent_force_reads_as_false` |
| 技能的新文字不會讓 agent 在沒有檔案的環境改用直接編輯 | 新句子只指向動詞；整份寫回被拒絕時明寫「停下來告訴使用者，不要改成直接編輯檔案」；搬回封存記錄的步驟限定規格資料夾在這個 checkout 時才做 | `speclink-core` 的 `skill_verbization::first_wave_skills_read_and_write_through_verbs` |

其他觀察（不改）：

- 線上的 `position` 是字串，server 只接受 `before`／`after`，其他值回 400 `invalid_argument`；CLI 收到不認得的值時報錯，不猜測。用字串而不是列舉，是為了讓錯誤經過 server 既有的 `invalid_argument` 回應，而不是 JSON 解析失敗的通用錯誤。
- `force` 缺席讀作 false，預設不覆寫既有的順序鍵。
- 遠端用戶端把變更名稱直接放進路徑，與既有的 `set_depends`、`set_schema` 相同；server 端的名稱檢查會擋下不合法的名稱。

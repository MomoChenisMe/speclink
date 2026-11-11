## MODIFIED Requirements

### Requirement: change rank 動詞寫入看板順序鍵
<!-- BEFORE: remote 模式一律拒絕且不發請求；順序鍵只寫進 change 的 .meta.yaml -->
<!-- REMOVED-SCENARIO: remote 模式拒絕 -->

`speclink change rank <name> (--before <other> | --after <other>) [--force] [--json]` SHALL 把 `<name>` 的 `board_rank` 改寫為排在 `<other>` 之前或之後的位置，使 plan 的配置順序與看板欄內順序同步改變。`--before` 與 `--after` SHALL 互斥且必擇一。守門 SHALL 依序：`<name>` 或 `<other>` 非作用中 change、metadata 壞掉、或兩者相同即錯；兩者階段不同即錯（rank 只在同欄內有意義）；`<name>` 已有 `board_rank` 且未帶 `--force` 即錯，訊息含 `already has a board rank` 與 `--force` 字樣；以新 rank 重跑 rank 移動的依賴檢查被擋即錯並印出 must_follow／must_precede。要寫入的 change（`<name>` 或補章對象）缺 `.meta.yaml` 時即錯，訊息含該檔路徑與 `is missing`。任一守門失敗 SHALL 以非零 exit code 結束且零寫入——所有檢查皆在第一筆寫入之前。新 rank SHALL 以看板排序鍵的中點演算（小寫英文字母、位元組字典序）落在 `<other>` 與其鄰居之間，鄰居與欄序 SHALL 取看板顯示序（plan 配置順序投影到該欄，即經宣告依賴修正後的基底序）；該欄（`<name>` 除外）任一 change 缺 rank、或 rank 序與顯示序不一致時，SHALL 先依顯示序對整欄補章（`<name>` 除外），再寫 `<name>`——欄序、補章、中點與依賴檢查 SHALL 與看板拖排為同一份引擎實作（board-card-order「欄內存在缺 rank 卡時整欄補章」）；寫入順序 SHALL 為補章逐一寫入、最後寫 `<name>`，中途失敗 SHALL 停止並回報、已寫入的補章保留（部分補章的欄仍是合法順序，重跑可續行）。每次寫入 SHALL 沿 depends 動詞的文字手術規則：其餘位元組逐字保留、純量經跳脫。成功時 stdout SHALL 為一行 `✓ <name> ranked before <other>`（或 `after`）；`--json` 時 stdout SHALL 為 `{ "change": string, "anchor": string, "position": "before" | "after" }`，SHALL NOT 含 rank 值。remote 模式 SHALL 把同一個請求（錨點、前或後、是否強制）送到 server 執行：守門、欄序、補章、中點與依賴檢查 SHALL 與本機為同一份引擎實作；順序鍵 SHALL 讀自並寫入該範圍的 board resource 文件，SHALL NOT 改寫任何 change 的 .meta.yaml；一次請求的全部順序鍵（補章與 `<name>`）SHALL 在同一次提交寫入，任一守門失敗或提交失敗時 SHALL 零寫入；stdout 與 `--json` 的形狀 SHALL 與本機相同。board resource 在這次請求讀取之後被另一方改寫時 SHALL 以非零 exit code 回報版本衝突且零寫入。沒有本機規格檔的內嵌宿主經引擎命令執行時 SHALL 適用與 remote 模式相同的規則。成功時命令層 SHALL 回報一筆以該 change 為主體的 change-ranked 領域事件。在主 checkout 且 worktree 政策開啟時，欄序與守門 SHALL 讀 plan 同一視角（主 checkout 的 change 名冊，映射到 worktree 者取其副本的值；worktree 分支後才在主 checkout 新建的 change 同樣在欄內），每筆寫入（補章與 `<name>`）SHALL 落在該 change 自己的副本——有映射寫其 worktree 副本、無映射寫主 checkout，與桌面卡片拖排同一路由。

#### Scenario: 無 rank 的 change 排到另一個之前

- **WHEN** 同為提案中的 a（rank: n）與 c（無 rank）之間執行 speclink change rank c --before a，欄內只有這兩個 change
- **THEN** c 的 .meta.yaml 多一行 board_rank，其值字典序小於 n，stdout 為 `✓ c ranked before a`，exit code 為 0，隨後 speclink plan --json 的 changes 順序 c 在 a 前

#### Scenario: 已有 rank 未帶 --force 被拒

- **WHEN** c 已有 board_rank，執行 speclink change rank c --after a
- **THEN** exit code 非零，stderr 含 `already has a board rank` 與 `--force`，c 的檔案逐位元不變

#### Scenario: 帶 --force 覆寫既有 rank

- **WHEN** c 已有 board_rank，執行 speclink change rank c --after a --force
- **THEN** c 的 board_rank 改為字典序大於 a 的值，exit code 為 0

#### Scenario: 跨宣告依賴被拒

- **WHEN** c 的 depends_on 含 a，執行 speclink change rank c --before a
- **THEN** exit code 非零，stderr 含 `cannot move 'c' there` 與 `it depends on a`，零寫入

#### Scenario: 缺 rank 的欄先整欄補章

- **WHEN** 提案中欄內 a、b、c 皆無 rank，基底序為 a、b、c，執行 speclink change rank c --before b
- **THEN** a 與 b 各獲得一個 rank（a 小於 b），c 的 rank 落在 a 與 b 之間，plan 的順序為 a、c、b

#### Scenario: 不同階段被拒

- **WHEN** a 為進行中、c 為提案中，執行 speclink change rank c --before a
- **THEN** exit code 非零，stderr 說明兩者不在同一欄，零寫入

#### Scenario: 缺 .meta.yaml 的 change 零寫入拒絕

- **WHEN** 提案中欄內 p1、p3 有 .meta.yaml 且皆無 rank，p2 的目錄只有 proposal.md，分別執行 speclink change rank p2 --before p1 與 speclink change rank p3 --before p1
- **THEN** 兩次皆 exit code 非零、stderr 含 `speclink/changes/p2/.meta.yaml is missing`，p1 與 p3 皆未被補章

#### Scenario: worktree 內的 change 與主 checkout 新建的錨點

- **WHEN** 主 checkout 且 worktree 政策開啟，x 已映射到 worktree、y 在 worktree 分支後才於主 checkout 新建，兩者同為提案中且皆無 rank，執行 speclink change rank y --after x
- **THEN** exit code 為 0，x 的補章寫入 x 的 worktree 副本、主 checkout 內 x 的 .meta.yaml 逐位元不變，y 的 rank 寫入主 checkout，隨後 plan 的順序 x 在 y 前；反向執行 speclink change rank x --before y 同樣成功

#### Scenario: remote 模式寫入 board resource

- **WHEN** 於 remote 模式，board resource 記有 a 的順序鍵 n、c 沒有順序鍵，兩者同為提案中且欄內只有這兩個 change，執行 speclink change rank c --before a
- **THEN** stdout 為 `✓ c ranked before a`，exit code 為 0；board resource 文件裡 c 的順序鍵字典序小於 n；c 與 a 的 .meta.yaml 逐位元不變；隨後 speclink plan --json 的 changes 順序 c 在 a 前

#### Scenario: remote 模式的 --json 形狀與本機相同

- **WHEN** 於 remote 模式執行 speclink change rank c --after a --json
- **THEN** stdout 為 `{ "change": "c", "anchor": "a", "position": "after" }`，不含順序鍵的值

#### Scenario: remote 模式缺順序鍵的欄一次提交補章

- **WHEN** 於 remote 模式，提案中欄內 a、b、c 在 board resource 皆無順序鍵，基底序為 a、b、c，執行 speclink change rank c --before b
- **THEN** board resource 文件的版本只前進一次，文件裡 a、b、c 都有順序鍵且 a 小於 c、c 小於 b；plan 的順序為 a、c、b

#### Scenario: remote 模式守門失敗零寫入

- **WHEN** 於 remote 模式，c 在 board resource 已有順序鍵，執行 speclink change rank c --after a
- **THEN** exit code 非零，stderr 含 `already has a board rank` 與 `--force`；board resource 文件的內容與版本不變

#### Scenario: remote 模式 board resource 被同時改寫

- **WHEN** 於 remote 模式執行 speclink change rank c --before a 的過程中，另一方在 server 讀取之後、提交之前改寫了 board resource
- **THEN** exit code 非零，stderr 說明版本衝突；board resource 是另一方寫入的內容

#### Scenario: 內嵌宿主經引擎命令排序

- **WHEN** 經儲存接口對一個含 a（有順序鍵）與 c（無順序鍵）的範圍執行「把 c 排到 a 之前」的引擎命令
- **THEN** 儲存收到正好一次提交，內含對看板順序文件的一筆寫入與一筆 change-ranked 事件；沒有任何 change 的 metadata 被寫入

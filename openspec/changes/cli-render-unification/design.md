## Context

CLI 的每個動詞在本機（fs store）與 remote（server store）兩模式各有一條執行路：本機走引擎 runtime 取得 core outcome 後渲染，remote 走 protocol client 取得 wire DTO 後渲染。渲染這一段目前分兩派：status／show／instructions／validate／analyze 已採「wire DTO 轉回 core 型別、餵同一支渲染函式」（既有正典樣板，remote-cli-parity design 決策 6）；其餘動詞（list、discuss 全家、task done／undone、in-progress remove、discard、archive、station 動詞群）是兩份手寫輸出，parity 靠複製貼上與凍結測試維持，已出現活漂移（remote list 不渲染 invalid 標記）。另有兩個動詞的 wire 本身載不夠：archive 回應缺封存結果欄位、station show 回應缺工單原文。

來源討論 improve-cli-command-layer 已裁定：全動詞收斂為單一渲染、wire 缺欄位同批補齊、明文分歧保留。

## Goals / Non-Goals

**Goals:**

- 每個動詞的輸出渲染只存在一支函式，兩模式共用；模式差異只活在資料組裝與守門
- wire 補齊 archive 與 station show 的缺欄位（serde default 向後相容），remote 人眼輸出對齊本機
- --json 形狀兩模式維持既有契約零變更
- 舊 server × 新 CLI 的混版組合可辨識並優雅退化

**Non-Goals:**

- 不動 22 處 remote_ctx 分岔的 dispatch 結構（候選 2，deferred）
- 不改 include! 檔案組織（候選 3，deferred）
- wire→core 轉接不搬進 speclink-remote（候選 4，deferred）
- C 類明文分歧不抹平（見 D5 清單）
- 本機模式輸出零變更；desktop 不在範圍

## Decisions

**D1 渲染契約：core outcome 型別是渲染的唯一入口**

每動詞一支 render 函式，參數是 core outcome 型別（或該動詞既有的 core 領域型別），本機現行輸出文本即正典。remote 路徑一律先把 wire DTO 轉回 core 型別再餵同一支函式。理由：樣板已在（render_show 明言「兩模式餵進同一個 ShowOutcome，輸出逐位元一致」），比替代案「渲染函式吃 wire 型別」正確——core 型別是引擎契約、與 storage 解耦方向一致，wire 型別是傳輸細節。歸屬邊界：渲染函式含 ANSI 與文本、留在 speclink-cli；core outcome 型別不動或僅補欄位、歸 speclink-core。

**D2 轉接落點與失效語意**

wire→core 轉接維持在 speclink-cli 的 remote 路徑內（既有 to_* 樣板），每動詞單層、不疊 wrapper。station ticket 轉接需把 wire 的 phase 字串轉回引擎 phase 型別：已知 token（discovery／validation）正常轉、null 轉 legacy、未知 token 明確報錯（fail loud）——未知 token 代表 server 比 CLI 新且語意未知，靜默吞掉會渲染出錯誤事實。

**D3 wire 欄位擴充（全部 serde default 向後相容，欄位名 camelCase）**

- ArchiveResponse 補：datedName（選填字串）、specs 各項補 added／modified／removed／renamed 計數、snapshotCreated（選填布林）、archivedDiscussions（slug 與檔名的清單，預設空）、evidenceRecorded（選填布林）
- ReviewTicketResponse 補：content（選填字串，工單原文全文）——review 與 verify 兩站共用同一 DTO，一次補齊兩站
- ConcludeDiscussionResponse 補：restaleFlagged（字串清單，預設空）——fs 側 conclude 會列出被 re-conclude 打回重收的變更，wire 原本整個回應都丟棄。這是實作期（任務 4.2）盤點 discuss 全家時發現的同類缺口，非新決策：變更名對遠端使用者同樣有意義，且規格「動詞人眼輸出的兩模式同形」要求分歧清單以外一律逐位元一致。不需哨兵——空清單即「無變更被打回」，與舊 server 的沉默同義。
- 混版退化採**單一哨兵欄位**：archive 以 datedName 是否在場判定 server 新舊——在場走完整渲染（與本機同形），缺席整體退回現行簡短輸出，不做半新半舊的混合渲染；station show 人眼路徑以 content 在場印原文、缺席退回現行結構化摘要。單一哨兵避免「計數有、名稱沒有」的碎片狀態
- evidenceRecorded 缺席不印零證據提示（不憑空捏造 stderr 提示）
- server 端 routes 從引擎 outcome 回填新欄位（server 跑的就是引擎，資料現成）；remote client 方法隨 DTO 更新

**D4 station show 的 --json 組裝收斂**

已驗證：本機 ticket_json 與 wire ReviewTicketResponse 直印的 JSON 形狀現在同形（欄位集合、camelCase、null 語意一致）。收斂方向定為「remote 也走 ticket_json」——wire DTO 轉回 core ticket 後餵同一組裝。這不只是去重：D3 給 ReviewTicketResponse 補 content 欄位後，直印 wire DTO 會讓 content 漏進 --json 輸出、破壞既有形狀契約；走 ticket_json 則 --json 形狀天然不變。回歸對照 crates/speclink-cli/tests/it/no_raw_wire_json.rs 的守門意圖與此一致。

**D5 明文分歧清單（保留，不追 byte parity）**

1. new change 的 Path 行——本機印、remote 不印：server 端路徑對本機使用者無意義
2. list 的 worktree 標示——remote 恆缺席：worktree 是本機主 checkout 的觀察面，server 沒有這回事
3. status 的 schema 覆寫旗標在 remote 明確拒絕——server 的 workflow config 決定 schema（remote-verb-parity C 類裁定）
4. workflow-config 的文件標籤——remote 以 config.yaml 為標籤：server 端沒有本機路徑可印
5. discuss promote 的 Path 行與其後的 propose 提示行——本機印、remote 不印：新變更目錄是 store 端的檔案系統位置，與第 1 項（new change 的 Path 行）同一條裁定。實作期（任務 4.2）補列：兩行綁在一起去留，remote 保持既有的單行輸出

清單之外的一切人眼輸出，兩模式 SHALL 逐位元一致。

**D6 remote 刻意輸出變更（凍結對照同步更新）**

1. remote list 開始渲染 invalid 標記（漂移修正）
2. remote archive 改印本機同形的完整結果（新 server）
3. remote station show 人眼路徑改印工單原文（新 server）

對應凍結對照以本機文本為準逐字更新；變更方向一律是「補上本機已有的資訊」，不動本機對照。

## Implementation Contract

**Behavior（出貨後可觀察）**

- 對新 server：list、discuss 全家、task done／undone、in-progress remove、discard、archive、station add-round／stamp／discard／show 的人眼輸出兩模式逐位元一致（D5 清單四項除外）
- --json 輸出：兩模式的欄位集合與形狀維持現行契約零變更；ReviewTicketResponse 的 content 欄位不出現在任何 --json 輸出
- 對舊 server（缺新欄位）：archive 與 station show 整體退回現行 remote 輸出；其餘動詞不受混版影響
- 本機模式：人眼與 --json 全部零變更
- exit code、stderr 守門訊息、stdin 用法全部不變；不新增子指令或旗標

**Interface / data shape**

- wire（camelCase，皆 serde default）：ArchiveResponse.datedName／snapshotCreated／evidenceRecorded／archivedDiscussions；ArchivedSpec.added／modified／removed／renamed；ReviewTicketResponse.content
- CLI 內部：每動詞一支 render 函式吃 core outcome 型別；remote 路徑的 to_* 轉接每動詞單層

**Failure modes**

- station ticket 轉接遇未知 phase token → 明確錯誤退出（fail loud），訊息指出 token 與來源
- 哨兵欄位缺席（舊 server）→ 整體退化為現行輸出，不混合渲染
- evidenceRecorded 缺席 → 不印零證據提示

**Acceptance criteria**

- cargo test -p speclink-cli 全綠：remote_verb_parity.rs 與 no_raw_wire_json.rs 依 D6 更新後，斷言 remote 輸出與本機文本同形；新增舊 server 退化案例（缺哨兵欄位的回應 fixture）
- cargo test -p speclink-protocol 全綠：新欄位的 serde 缺席讀取 round-trip（空物件反序列化成功）
- cargo test -p speclink-server 與 -p speclink-remote 對應整合測試全綠：server 回填新欄位、client 讀取
- 人工確認：remote_commands.rs 內不再存在與本機重複的人眼渲染文本（守門 bail 與 D5 分歧項除外）

**Scope boundaries**

- In scope：上列動詞的渲染收斂、三個 wire DTO 欄位擴充、server 回填、client 更新、對應測試更新
- Out of scope：dispatch 分岔結構、檔案切分、speclink-remote 轉接層、desktop 消費新欄位、C 類分歧、本機輸出的任何變動

## Risks / Trade-offs

- **回歸對照面**：remote 凍結輸出刻意變更，對照更新若非逐字取自本機文本會引入新漂移——驗收以「同一 fixture 兩模式跑、文本相等」的斷言取代手抄對照。golden（skills／assets 衍生鏈）不受影響：本變更不動技能與資產。
- **跨平台**：輸出為純文字與 JSON，無新增 git 互動；archivedDiscussions 的檔名是 store 內名稱、非 OS 路徑，Windows 分隔符無涉。既有 no-color 行為由共用渲染函式自然保留。
- **混版矩陣**：新 CLI × 舊 server 靠哨兵欄位退化（有測試釘住）；舊 CLI × 新 server 靠 serde 忽略未知欄位（protocol 既有慣例）。
- **wire 負載**：ReviewTicketResponse 帶工單原文全文（可達數十 KB），僅 station show 端點載——單發請求、人眼閱讀場景，可接受；替代案（另開原文端點）多一條路由與一次往返，不值。
- **朝 storage 解耦靠攏**：渲染吃 core outcome 型別強化「引擎 outcome 是唯一契約」——store 後端（fs／server）差異被限制在組裝層，符合規格驅動引擎的解耦方向。

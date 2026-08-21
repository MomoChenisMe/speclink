## Context

capability 名稱由 AI 在 propose 階段決定，引擎現況三處皆不設防：`new artifact spec` 收什麼名字建什麼資料夾（newcmd.rs 的 resolve_output 直接拼路徑）；validate 與 archive 雖能判斷「正典無此 capability」，但只拿來守新 capability 的 Purpose 品質與 ADDED-only；propose 技能的既有規格掃描步驟只顯示、不擋、不留痕。上游 OpenSpec 至 v1.10.0 亦無確定性防護。後果：`auth` 已存在時 AI 另建 `authentication`，全 ADDED 的 delta 一路靜默通過，封存後留下兩份語意重複的正典規格。本專案已有 71 個正典 capability，命名家族大量共用字根（archive-merge／archive-skill、commit-skill／config-skill）。源頭討論：capability-naming-dedup。

## Goals / Non-Goals

**Goals:**

- 「開新 capability」從隱性副作用變成顯性宣告：建立點確定性拒絕未宣告的新名稱，AI 在做決定的當下就拿到近似既有名與其 Purpose 首行。
- 第二張網涵蓋所有繞過建立點的入口（ingest 直接編修、手寫目錄）：validate 以 warning 提示近似名。
- 技能資產與引擎行為同步：propose／ingest 指令教會 AI 新流程，錯誤訊息自帶自救指引。

**Non-Goals:**

- 相似度門檻硬擋（命名家族誤殺；相似度只排序建議）。
- archive 階段的新防護（實作完成後才擋代價最高；維持既有 Purpose 守門與 ADDED-only）。
- `--new` 的 metadata 留痕（Purpose 守門已強制新 capability 說明用途）。
- 純語意重複偵測（`login` vs `auth`；Purpose 文字留作未來素材）。
- 設定欄位新增（openspec/config.yaml／.speclink.yaml 不動）。

## Decisions

**D1 主閘落在引擎的 new_artifact，單一實作落點。** 守門邏輯放 speclink-core 的 newcmd（resolve_output 之後、寫入之前），不放 CLI 層：CLI／Node 經命令層 dispatch 同語意進入。remote 是明文分歧：`--remote` 的 new artifact 走 server 的 raw artifact PUT——那是直接寫入通道（If-Match 決定 create／overwrite），不經此閘，`--new` 在 remote 模式無作用；繞過建立點的入口一律由 validate 第二網（D5）涵蓋。拒絕歸類為既有錯誤碼 `refused`（前置條件拒絕、須帶旗標放行的語意完全吻合），不擴充封閉錯誤碼集合。拒絕發生在任何檔案寫入之前，stdin 內容不落盤，無半套狀態問題。

**D2 擋在二元事實，相似度只排序。** 閘門條件唯一：候選名稱不在正典 capability 清單（大小寫逐字比對——比對走 `list_canonical_capabilities` 的清單，不走檔案系統存在性判定：大小寫不敏感的 fs 會把 `Auth` 當 `auth` 靜默放行）。例外一條：本 change 已存在同名 delta spec（先前已以 `--new` 宣告過）時放行，交給下游既有的覆寫保護——宣告一次即可，重寫不再重複要求；此側同樣以 delta capability 清單逐字比對，不走檔案系統存在性。近似名單僅供建議，排序規則：token 完全包含（`auth` ⊂ `authentication`；比對做 ASCII 大小寫折疊，被包含側須達 3 字元以擋退化候選的雜訊）優先，次為 kebab token 交集數，再次為編輯距離，取前三。不設任何「太像就擋」的門檻——71 個正典中的命名家族會被門檻誤殺，而二元事實無參數可調、行為可預測。

**D3 名單來源＝正典＋進行中 change 的 delta。** 建議池為正典 capabilities（標注 canonical、附正典 spec 的 Purpose 首行）加上未封存 change 的 delta capabilities（含當前 change 的其他 delta——同一 change 內兩個近似新名也要互相看見；標注 in-flight: 該 change 名、附 delta 的 Purpose 首行；缺 Purpose 則略去該行），同名以正典優先去重。validate 端唯一濾掉的是受檢 capability 自身；與候選完全同名的 in-flight delta 在主閘訊息中不列入相似名清單，改以獨立句點名開立它的 change 並指路 `--new`（「沿用確切名稱」的指引對同名是死路）。全部組合既有 store 讀取介面（正典列舉、delta 列舉、規格讀取），不改 store trait、不動雙 store conformance。

**D4 命名知識統轄於 speclink-core 內部模組 capname。** 新模組 capname 承載三件事：排序純函式（輸入候選名與帶來源標注的既有名集合，輸出至多三筆的結構化建議清單——名稱、來源、Purpose 首行）、建議池組裝（唯讀組合既有 store 介面，主閘與 validate 共用同一份）、建議行的共用格式。不含 ANSI、不寫入儲存；訊息文字由呼叫端（newcmd 的錯誤訊息、validate 的 warning 文字）組裝。

**D5 validate warning 與既有新開 capability 早檢查同點掛載。** validate 對每個「正典無同名」的 delta capability，跑同一個 capname 建議池；有建議即報 warning（不影響 valid 結果），訊息附「同一 capability 就改用既有名；確為新 capability 可忽略」。warning 與建立點主閘共用 capname，兩處輸出一致。

**D6 技能資產四項增補與三連動。** propose.md：掃描結果留痕於 proposal、New Capabilities 每項附「為何既有規格不涵蓋」、寫明 `--new` 語意與時機；ingest.md：新增 delta capability 前先對照既有名。資產內文變更連動 MARKER_VERSION（crates/speclink-core/src/init.rs）、golden snapshots 與 assets.lock（crates/speclink-core/tests/golden/）；`speclink update` 再生的各專案 SKILL.md 屬衍生物，不進 evidence，收尾以 git status 盤點。

**D7 已知限制：worktree 內新建的 delta 對其他 checkout 不可見。** 跨 change 比對只看得到當前 checkout 的 changes 目錄。接受此限制：propose 慣例在主 checkout 執行，worktree 主要承載 apply；平行撞名最終仍有 validate warning 與封存順序把關。不為此做跨 worktree 掃描。

## Implementation Contract

**Behavior（可觀察行為）：**

- `speclink new artifact spec <cap> --change <name>`，`<cap>` 不在正典（清單逐字比對）、本 change 亦無同名 delta、且未帶 `--new`：指令拒絕，exit code 非零，錯誤碼語意 refused；訊息含最多三個近似既有名（來源標注＋Purpose 首行）與兩條指引（沿用既有名／帶 `--new` 重跑）；與候選完全同名的 in-flight delta 以獨立句點名其 change，不列入相似名清單；不寫入任何檔案。
- 本 change 已有同名 delta spec：不再要求 `--new`，直接進入既有的覆寫保護流程（未帶 `--force` 報 already-exists、帶 `--force` 覆寫）。
- 同指令帶 `--new`：行為與現行完全一致（含既有的 delta 格式驗證與覆寫保護）。
- `--remote` 模式：new artifact 走 server 的 raw artifact PUT（直接寫入通道），不經主閘，`--new` 無作用；由 validate 第二網涵蓋。
- `<cap>` 命中正典：行為與輸出與現行位元級一致，不受本變更影響。
- `speclink validate <name>`：delta 中「正典無同名」的 capability 若建議池非空，報 warning 級發現（不改變 valid 布林）；既有 capability 的 delta 不受影響。

**Interface：**

- CLI：`new artifact spec` 子指令新增旗標 `--new`（無值布林）。
- 引擎：new_artifact 函式簽名增加新 capability 確認參數（bool）；命令層 argv 對應傳遞。
- capname 模組：pub fn 接受候選名、既有名集合（名稱＋來源＋Purpose 首行），回傳排序後最多三筆建議（同一結構原樣回傳）；另承載建議池組裝與建議行格式（crate 內部）。

**Tests（TDD，先測後寫）：**

- capname 單元測試：包含優先於 token 交集、token 交集優先於編輯距離、上限三筆、無近似時回空清單。
- newcmd 閘門測試：正典命中放行、未命中拒絕且不落盤、`--new` 放行、in-flight delta 出現在建議清單。
- validate lint 測試：近似新名報 warning 且 valid 不變、無近似新名不報、既有 capability 不報。
- CLI 整合測試（--test it）：refused 拒絕的 exit code 與訊息形狀、`--new` 成功路徑。
- 資產連動：golden 再生後 snapshot 測試綠、assets.lock 一致、MARKER_VERSION 斷言更新。

**Migration：**

- 既有測試或腳本中「以全新名稱建 delta spec」的路徑補 `--new`；技能資產同步更新後 AI 代理照新指令自然帶旗標。

## Context

三刀重切後本刀（①）交付可複用核心：補完 `@speclink/engine`（SDK）與 `Store` 縫、修訂 verb-contract 正典，讓任何整合者以 SDK ＋自實作 Store 建自家 server。

現況（決定取捨的既有事實）：
- **dispatch 只路由 4 動詞**：`crates/speclink-node/src/lib.rs` 的 `run_dispatch` 僅 `list`/`status`/`new`/`claim`；其餘動詞的業務邏輯在 `speclink-core` 都有（CLI 在用），差沒從 Node router 暴露。
- **analyze/validate/drift 目前 client-side**：`docs/verb-contract.md` §6 明列這四個（含 show）在 remote 模式由 CLI 內嵌引擎本地算，契約無對應端點。
- **Store 縫既成**：`speclink-core` 的 `Store` trait ＋ `crates/speclink-node/src/store_bridge.rs` 的 JS bridge 已支援 host 以 JS 實作 Store；`store-abstraction`／`node-sdk` 為現行正典。
- **推播不在契約**：`docs/verb-contract.md` §preamble/§7 明列 push 屬 host concern、在請求/回應契約外。
- **遠端讀取既有雛形**：verb-contract §5「store 文件讀取動詞」要求技能經 `artifact cat` 讀文件、禁指路徑；`remote-connection` 的「指令區塊的 remote 變體」要求 marker 用動詞措辭。

## Goals / Non-Goals

**Goals:**
- dispatch 補完可遠端託管的完整動詞集，複用 core 邏輯、不改語意。
- analyze/validate/drift 於遠端模式改 server 端運算：引擎能對 host store 算出報告並經 dispatch 暴露、verb-contract 新增端點、CLI remote 路由至之。
- verb-contract 新增可選、傳輸無關的推播通道宣告欄；引擎本體零推播機制。
- 遠端 agent 經動詞讀文件涵蓋補完後的動詞集。
- Store 縫文件化為公開整合面（host 實作 Store ＋建自家 server 的指南）。

**Non-Goals:**
- 不含 pg Store 實作與 demo server（②）、不含 desktop 遠端模式（③）。
- 引擎不實作任何推播傳輸（SSE／WebSocket 屬 server／client 層）。
- 不改動各動詞的業務語意——僅路由暴露與運算位置。

## Decisions

### D1: dispatch 動詞擴充

於 `run_dispatch` 增補路由：`archive`、`task done`、`artifact cat`、`instructions`、`language`、`config`、`spec` 讀取、`discuss` 全套。每動詞複用 `speclink-core` 既有函式與 `Store` 既有方法、回傳與 fs `--json` 對齊的 camelCase payload。`instructions` 得複用既有 `instructions.render()` API（apply 時擇一）。

替代方案：host 各自以 TS 重實作動詞。**駁回**——複製 core 業務邏輯、必然版本漂移，違「canon 在 core 單一真相」。

### D2: analyze/validate/drift server 端運算

引擎將 analyze/validate/drift 對 host store 的文件算出報告並經 dispatch 暴露；`docs/verb-contract.md` 新增對應唯讀端點；CLI remote 模式改路由這三者至端點（修訂 §6/§7 原「client-side」）。`crates/speclink-remote/src/client.rs` 加 client 方法、`crates/speclink-cli/src/remote_commands.rs` 改路由。

替代方案：維持 client-side（CLI 內嵌引擎本地算）。**駁回**——各人 CLI/引擎版本歧異會使全隊對同一 server 資料的 analyze/validate/drift 結果分裂；team server 應釘住分析語意。（技術上現有內嵌 Rust 引擎的 client 仍可本地算，故此為刻意的正確性選擇。）

### D3: 推播宣告欄與引擎零推播

`docs/verb-contract.md` 新增可選、傳輸無關的宣告欄 `events:{url,transport}`（掛 whoami/config metadata；明標 push 在請求/回應契約外、僅供 client 發現 server 推播通道）。引擎本體不含任何推播機制。

替代方案：對所有 server 規定 SSE（**駁回**——鎖死用其他傳輸如 WebSocket 的整合者，其 server 資料同步不進 client）；宣告欄降為 server↔client 私下約定（**駁回**——整合者少契約背書的統一發現方式、互通稍弱）。

### D4: 遠端 agent 經動詞讀文件

遠端技能與 marker 導引 agent 一律經 `artifact cat`／`language show`／`discuss show`／`show` 讀文件（涵蓋補完後的動詞集），禁讀本地路徑（遠端無本地 `openspec/`）。延伸 verb-contract §5 既有「store 文件讀取動詞」至補完的動詞集。

替代方案：agent 於遠端仍讀檔案系統。**駁回**——遠端無本地檔，讀路徑必撲空。

### D5: Store 縫公開整合面文件化

以文件（`docs/integration.md` 新增、`docs/sdk-node.md` 補充）說明 host 如何實作 `Store` 介面 ＋以 `createEngine` 建自家 server——「拆出儲存邏輯」的可複用交付。不新增抽象層，只文件化既有縫。

替代方案：不文件化、留讀原始碼。**駁回**——使用者要 Store 縫成可獨立交付、可整合自家系統的頭牌，需明確整合面文件。

## Implementation Contract

#### node-sdk（MODIFIED capability）

- **可觀察行為**：`engine.dispatch(argv,{stdin})` 除既有 `list`/`status`/`new`/`claim` 外，路由 `archive`、`task done`、`artifact cat`、`instructions`、`language`、`config`、`spec`、`discuss` 全套，各回與 fs `--json` 對齊的 camelCase 結構、經 host `Store` 讀寫。另可對 host store 算 `analyze`/`validate`/`drift` 並回其報告 payload。
- **失敗模式**：路由外動詞仍以帶 `code` 的 `Error` 拒絕（invalid argv）；不改各動詞既有語意。
- **驗收**：`crates/speclink-node/__test__` 對 mock JS Store 斷言各動詞 dispatch payload 與 fs 一致；analyze/validate/drift 對 mock store 回正確報告。
- **In scope**：路由既有 core 邏輯至 dispatch。**Out**：改動業務語意；pg/desktop。

#### verb-contract（MODIFIED capability）

- **可觀察行為**：(1) 遠端模式 `analyze`/`validate`/`drift` 由 server 端運算——契約含對應唯讀端點，CLI remote 呼叫端點取報告而非本地算，人眼與 `--json` 輸出與 fs 模式一致。(2) server metadata（whoami/config）得帶可選 `events:{url,transport}`；client 讀不到即無推播、讀到未支援之 transport 即忽略。(3) 遠端 agent 經動詞讀文件涵蓋補完後的動詞集。
- **失敗模式**：analyze/validate/drift 端點對不存在的 change 回 `404 not_found`；宣告欄缺席為正常狀態（client 退回輪詢，非錯誤）。
- **驗收**：CLI remote `analyze`/`validate`/`drift` 對假 server（複用 `crates/speclink-remote/tests` 的 tiny_http 模式）取報告、輸出與 fs 一致；`docs/verb-contract.md` §6 涵蓋圖與 §7 對照修訂；宣告欄於契約文件明載為可選、傳輸無關。
- **In scope**：契約端點與宣告欄定義、CLI client 側路由。**Out**：server 端端點實作（②）、推播傳輸實作（②/③）、client 推播消費（③）。

#### Store 公開整合面（docs）

- **可觀察行為**：`docs/integration.md` 明載 host 實作 `Store` 的完整方法契約與以 `createEngine` 建自家 server 的路徑；`docs/sdk-node.md` 補整合面連結。
- **驗收**：內容審視確認方法契約齊備、範例可循。

## Risks / Trade-offs

- **[修訂 verb-contract 正典（analyze/validate/drift 運算位置、§6/§7）]** → docs/verb-contract.md 與 verb-contract spec delta 同步修訂；以 CLI remote 對假 server 的輸出一致性測試為回歸紅線。
- **[analyze/validate/drift 運算位置改變，CLI 遠端輸出須與 fs 逐位元一致]** → 複用假 server 測試對打 fs 輸出；改動前確認 parity 對照仍綠或刻意更新。
- **[dispatch 擴充動到 napi crate（mingw cdylib 連結風險）]** → 依既知，crate 測試以 `--lib` 跑；napi build 後以 __test__ Vitest 驗。
- **[跨平台]** → 路徑/換行不假設單一平台；複用既有 Store 方法不新增檔案系統假設。

## Migration Plan

- 純加法為主：dispatch 新動詞、契約新端點/宣告欄為既有契約的加法（clients 忽略未知欄位）。
- analyze/validate/drift 由 client-side 改 server 端為契約語意修訂——CLI 於 remote 模式改行為；fs 模式不變。回滾＝CLI remote 路由改回本地算（契約端點可留為可選）。

## Open Questions

- `instructions`/`analyze`/`validate`/`drift` 走 dispatch 或既有 render/compute API——apply 時擇一（兩者皆產一致 payload）。
- 宣告欄掛 whoami 或 config——apply 時依 client 讀取時機定案（皆為 metadata 讀）。

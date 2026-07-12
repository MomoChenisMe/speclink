## Context

三個入口各自組裝引擎呼叫：`speclink-cli` 的 handler（crates/speclink-cli/src/commands.rs，1700 行）直呼 core 模組函式並自行渲染；`speclink-node` 的 dispatch（crates/speclink-node/src/lib.rs）以手刻 argv router 重組 list/status/new/claim 四動詞的 core 呼叫；桌面 app 經 apps/desktop/core 再直呼一次。core 沒有「命令」這個概念——動詞的輸入、輸出與錯誤散落在各入口的組裝碼裡。

設定解析 fail-open 的實際呼叫點：AppConfig::load（壞檔靜默回預設）被 workspace.rs 的 spec_dir 解析與 resolve_mode、instructions 組裝、init、CLI deprecated-keys 警告使用；WorkflowConfig::from_text（壞檔靜默回預設）被 CLI、Node、instructions、discuss 使用。桌面設定頁（apps/desktop/core/src/settings.rs）已因此自行做嚴格解析並在註解言明是為了繞開 core 的靜默 fallback——本刀把這個修正下沉回 core，消除繞道。

約束：CLI 人眼輸出與 `--json` 是回歸保護對象（parity 31 項／color 16 項／twin 8 情境）；core 不得含 ANSI 與呈現邏輯；TDD 紅綠重構。

## Goals / Non-Goals

**Goals:**

- core 出現唯一的 typed 命令執行層：command 輸入、outcome 輸出、穩定錯誤碼、domain events。
- CLI 與 Node dispatch 改經此層執行，行為與輸出凍結不變。
- 設定檔「存在即必須可解析」：壞檔在所有入口一致報錯停止，缺檔才給預設。

**Non-Goals:**

- Store trait、revision、CAS、UoW、事件持久化／訂閱（teamstore-contract 與 server 刀）。
- Project/Repo binding、actor 身分（binding-and-policy 刀）；事件因此暫不含 actor 與 revision。
- 桌面 app 遷移到 runtime（Phase 3）；本刀對 desktop 僅做 core 簽名變更的機械式跟進，無 UX 變更。
- dispatch 動詞擴增、envelope 形狀變更、生命週期狀態機閘門。

## Decisions

### 決策一：runtime 落在 speclink-core 的 command 模組

新增 crates/speclink-core/src/command/ 模組：`Command`（封閉 enum，依領域分組）、每命令對應的 typed outcome、`CommandError`（穩定錯誤碼＋語意訊息）、`DomainEvent`、單一進入函式 execute（吃 `&dyn Store` 與可選 `Workspace`，回 outcome＋events）。runtime 只做編排（change 解析、schema 解析、事件建構），流程邏輯仍在既有 core 模組函式——runtime 是前門，不是重寫。

- 為何不是新 crate `speclink-command`：core 即引擎家，多一個 crate 邊界沒有任何消費者需要；等 TeamStore 刀若有需要再談拆分。
- 為何不是 trait-per-command 物件：動詞集合封閉且已知，closed enum 讓 match 窮舉可被編譯器檢查；trait 物件是為開放集合準備的抽象，這裡是過度設計。
- 朝 storage 解耦靠攏：execute 只依賴 `&dyn Store`，不碰 PathBuf 與檔案佈局；後續 TeamStore 刀替換 Store 實作時 runtime 不動。

### 決策二：動詞覆蓋判準——讀寫 Store 的領域動詞才進 runtime

| 分組 | 動詞 | 事件 |
|---|---|---|
| 查詢（進 runtime） | list、show、status、instructions、validate、analyze、drift、artifact cat、language show、discuss list／show | 無 |
| 變更（進 runtime） | new change、new artifact、task done、task undone、claim、in-progress add、archive、discard、discuss new／context／add-round／conclude／promote／link／seal／archive／discard | 有 |
| 不進 runtime | init、update（bootstrap＋技能同步）、config（使用者層 AppData 設定檔）、schema 工具、completion、templates、feedback、demo、link／unlink／auth（連線管理）、remote HTTP 攔截（crates/speclink-cli/src/remote_commands.rs） | — |

- 判準：動詞的語意主體是否為 Store 內的規格文件。init／update 是 workspace bootstrap，server 永遠不會暴露它們；硬收進 runtime 只添 API 噪音。
- 替代方案（全動詞入 runtime）被否決：見上；且 completion／templates 等純本機工具連 Store 都不碰。
- 時序前提：task undone 動詞由 task-uncheck-cli 變更引入（勾選的反向動詞，同為讀寫 Store 的變更型動詞）。本變更開工前 task-uncheck-cli 須已落地——baseline exe 於其落地後保存，undone 的輸出凍結基準即為其現行 CLI 實作；若屆時未落地，開工前的 drift 檢查會把此列舉標為前提缺口。

### 決策三：typed error 與穩定錯誤碼註冊表

`CommandError` 攜帶封閉的錯誤碼 enum＋語意訊息。碼值域以現行 Node dispatch 已存在的字串為基底（invalid_argv、not_found、error），新增 invalid_config（設定檔存在但解析失敗）與 refused（需 --force 類的拒絕，如 discard 已開工、discuss discard 已有 rounds）；store_error 與 panic 維持在 Node envelope 層。CLI 把 CommandError 映射為既有錯誤文字＋非零 exit code——語意訊息字串就是現行 CLI 訊息，是回歸對照的一部分，不得改寫。

- 替代方案（anyhow 貫穿、各入口字串比對分類）被否決：Phase 2 server 需要把錯誤映射到 HTTP 語意，SDK 需要可程式判斷的碼；字串比對脆且已在 Node 端證明要靠 downcast 補洞。

### 決策四：domain events 的種類、載荷與發出點

`DomainEvent` 依變更型動詞一一對應：ChangeCreated、ArtifactCreated、TaskCompleted、TaskUncompleted、ChangeClaimed、ChangeMarkedInProgress、ChangeArchived、ChangeDiscarded、DiscussionCreated、DiscussionContextSet、DiscussionRoundAdded、DiscussionConcluded、DiscussionPromoted、DiscussionLinked、DiscussionSealed、DiscussionArchived、DiscussionDiscarded。載荷＝主體識別（change 名／discussion slug）＋該次變更的最小事實（artifact id、task 編號、轉出的 change 名等）＋ occurredAt（UTC）。不含 actor 與 revision（分屬 binding 與 teamstore 刀），事件契約標示 experimental，outbox 落地前不做相容承諾。

- 發出點：execute 在 core 函式成功返回後、由 typed outcome 建構事件——單一發出點、零侵入既有函式簽名。複合動詞發多事件（promote＝DiscussionPromoted＋ChangeCreated）。
- 替代方案（事件在各 core 模組內部發出）被否決：得改動全部核心函式簽名、且在 UoW 存在前沒有下沉的收益；teamstore-contract 刀把事件搬進 commit 時再下沉。

### 決策五：設定解析 fail-closed——存在即必須可解析

AppConfig::load 與 WorkflowConfig::from_text 由「壞檔→預設」改為 typed 結果：缺檔→預設（行為不變）；檔案存在但 serde_yaml 解析失敗（語法錯誤、型別不符）→ 設定錯誤（含 workspace 相對路徑＋解析原因），映射為 invalid_config。fail-closed 落在載入函式本身，所有呼叫者（runtime 內外、含 init／update 等 bootstrap 動詞與 resolve_mode）一致得到錯誤——壞 `.speclink.yaml` 從此不會解析成 fs 模式，壞 `openspec/config.yaml` 不會產生「政策全滅」的 instructions。

- serde 相容性：可成功解析的既有檔案行為完全不變（含未知欄位的容忍度不變）；只有 serde_yaml 回錯的情形從寬容轉嚴格。
- 桌面設定頁既有的自行嚴格解析改為呼叫下沉後的 typed 載入函式（機械式跟進，錯誤顯示路徑沿用既有 UI）。
- 替代方案（只修 `.speclink.yaml`、`openspec/config.yaml` 留寬容）被否決：藍圖 P0 驗收第 7 條要求兩者一致 fail-closed；政策靜默消失正是 CLAUDE.md 記錄的既知風險。

### 決策六：CLI 與 Node 的遷移策略——逐動詞群、輸出凍結

CLI handler 分三群遷移（查詢群→變更群→discuss 群），clap 定義與渲染碼不動；每群遷移完成即跑回歸對照。重構第一步前先建置並保存 baseline exe，供自我基線雙沙盒對照（scratchpad 基建會消失，基線放非暫存位置）。Node 端保留 argv mini-parser 與 envelope 組裝，解析結果改建構 typed Command 交 runtime；verb_list／verb_status／verb_new／verb_claim 內重組 core 呼叫的邏輯隨遷移刪除。

- 替代方案（由 runtime 型別自動生成 clap 定義）被否決：help 與錯誤輸出是回歸對照的一部分，生成器的輸出漂移風險遠大於少寫的樣板。
- git 行為：drift／archive 對 git 的呼叫方式與跨平台行為完全不變——runtime 只是把既有函式包進命令層。

## Implementation Contract

**可觀察行為：**

- 覆蓋表內所有動詞，人眼輸出與 `--json` 和遷移前逐位元一致（唯一例外＝下述壞設定檔情境）。
- 壞 `.speclink.yaml` 或壞 `openspec/config.yaml` 存在時：CLI 任何讀取該檔的動詞以非零 exit code 結束，錯誤訊息含該檔的 workspace 相對路徑與解析原因；Node dispatch 回 ok:false、code 為 invalid_config；缺檔行為不變（預設）。
- Node dispatch 對相同 store 狀態回傳與遷移前相同的 envelope 值；錯誤碼字串維持既有值域＋新增 invalid_config、refused。
- 覆蓋表「變更」列的每個動詞成功時回傳表列對應的事件（含 occurredAt 與主體識別）；查詢動詞回傳空事件。

**介面／資料形狀：** speclink_core::command 公開 `Command`、各 typed outcome、`CommandError`（碼 enum：invalid_argv、not_found、invalid_config、refused、error）、`DomainEvent`、execute 進入點（`&dyn Store` ＋可選 workspace →outcome＋events）。JSON 對外形狀維持既有 camelCase 契約，由 CLI 渲染層產生、不因 runtime 而變。

**失敗模式：** 設定錯誤＝invalid_config（fail-closed，不退預設、不改 binding/mode）；找不到主體＝not_found；參數不合法＝invalid_argv；需 --force 的拒絕＝refused；其餘＝error。錯誤訊息文字沿用現行 CLI 訊息（回歸對照凍結）。

**驗收準則：**

```text
cargo test -p speclink-core          # runtime 單元＋config fail-closed＋事件斷言（本機 Windows 需 --lib）
cargo test -p speclink-cli           # CLI 整合測試
cd crates/speclink-node && npm run build && npm test   # dispatch 相容層（vitest）
parity_suite 31 項 / color_suite 16 項 / twin harness 8 情境 全綠
baseline exe 自我對照：遷移前後對同一 workspace 跑覆蓋表動詞、diff 輸出為空（壞設定情境除外）
```

**範圍邊界：** in——crates/speclink-core（command 模組、config.rs、workspace.rs）、crates/speclink-cli（handler 改route）、crates/speclink-node（dispatch 改route）、apps/desktop/core 的機械式簽名跟進；out——Store trait 與 speclink-fs、remote_commands.rs 攔截路徑、桌面 UX、事件持久化、dispatch 動詞擴增。

## Risks / Trade-offs

- [CLI 輸出回歸破壞] → 分群遷移、每群後跑 parity／color／twin；重構前保存 baseline exe 做自我基線對照（不放 scratchpad）。
- [壞設定 fail-closed 使既有工作流突然報錯（例如殘留的壞 `.speclink.yaml`）] → 錯誤訊息明確指出檔案與修法（修檔或刪檔）；這是 P0 刻意行為，proposal 已標 BREAKING。
- [AppConfig::load 簽名變更波及面廣（workspace.rs、init、instructions、desktop core）] → 編譯器驅動的機械式遷移；各呼叫點只傳播錯誤、不各自解讀。
- [事件契約過早固化] → payload 最小化、標示 experimental；teamstore-contract 刀移入 UoW 時允許破壞性調整。
- [跨平台] → 錯誤訊息中的路徑用 workspace 相對顯示；Windows 本機 cargo test 帶 --lib 的怪癖記在 tasks 的驗證註記。

## Migration Plan

1. config fail-closed（typed 載入函式＋全呼叫點傳播＋新測試）——獨立可回退的第一步。
2. command 模組型別（Command／outcome／CommandError／DomainEvent）＋查詢群 execute。
3. 變更群 execute ＋事件建構與斷言。
4. CLI 分群切換到 runtime，每群跑回歸。
5. Node dispatch 改 route，刪除 verb_* 內的重組邏輯。
6. baseline exe 全量對照收尾。

回退策略：各步獨立 commit，出問題 revert 對應步；無資料格式遷移、無設定檔格式變更。

## Open Questions

無——role／binding／revision 等留白已明確劃給後續刀。

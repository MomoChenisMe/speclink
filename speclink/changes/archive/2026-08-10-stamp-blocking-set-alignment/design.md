## Context

兩站品質站的乾淨蓋章守門由 crates/speclink-core/src/station.rs 的共用 stamp gate 承載：末輪 findings 計數（`last_round().findings.len()`）不分嚴重度，任何一筆（含 SUGGESTION）都要求 `--accept` 才放行。兩站 skill 的「阻斷集」定義卻只算必修（CRITICAL＋有現實觸發路徑的 WARNING），SUGGESTION 歸可裁。兩套定義都寫進正典（verify-station／review-station 的守門條文 vs verify-skill／review-skill 的迴圈規則），落差在每次記錄 SUGGESTION 時觸發：該輪注定到不了乾淨蓋章，只能修掉或問使用者 `--accept`，形成「乾脆不記 SUGGESTION」的反誘因。討論 stamp-blocking-set-alignment 裁定：往嚴重度分界收斂。

## Goals / Non-Goals

**Goals:**

- 引擎乾淨蓋章的阻斷集與 skill 的必修概念收斂到同一條可計算的分界：嚴重度標籤（CRITICAL／WARNING 擋、SUGGESTION 不擋）。
- 消滅「記 SUGGESTION 就逼修或逼問」的反誘因，讓可裁觀察能無摩擦地留在工單紀錄。
- 三份技能資產（verify／review／quality）的迴圈與蓋章敘述與新引擎行為一致，且衍生物（golden、assets.lock、兩個 render target 的技能檔）同批再生。

**Non-Goals:**

- 不在蓋章 meta 記錄 `--accept` 與否或殘留 SUGGESTION 數（誠實性靠工單 git 歷史；討論 Deferred）。
- 不引入行內「可裁」標記、不改工單行格式（`- [SEVERITY] path — 描述` 與 `(accepted)` token 照舊）。
- 不放行已接受的必修 findings——`--accept` 儀式不變。
- 不動 desktop GUI、server API、protocol shape、`speclink analyze` 分級體系。

## Decisions

**D1 — 阻斷分界落在嚴重度，不落在新標記。** 引擎讀得懂的只有工單行的 severity 標籤；skill 的必修／可裁是判斷式分類，引擎算不出。與其發明行內標記讓引擎讀懂判斷（格式、解析器、跨站波及都大），不如收緊 skill 的記錄紀律：可裁事項一律記 SUGGESTION，讓嚴重度成為分界的唯一載體。代價是 severity 從「多嚴重」兼任「擋不擋」，可裁級的 WARNING 從此不存在——討論已裁定接受此代價。

**D2 — 守門計數改為過濾 SUGGESTION，落點只在 station.rs 的共用 gate。** 兩站共用同一段守門（stamp gate 的條件 (2)），改一處兩站同時生效；local 與 remote 路徑都經同一引擎函式，無平行實作（回歸對照 crates/speclink-cli/tests/it/remote_verb_parity.rs 不受影響——動詞介面與 `--json` shape 皆不變）。帶 `(accepted)` token 的行不另設豁免：它的 severity 是必修級就照樣擋、要 `--accept`；是 SUGGESTION 級就本來不擋（新流程下 SUGGESTION 不再進入接受機制，此情況僅出現於舊工單）。

**D3 — 拒絕訊息點名必修數量。** stderr 從「N unresolved finding(s)」改為點名待處理必修數（例：`the last round has N outstanding must-fix finding(s) (CRITICAL/WARNING)`），提示語維持「先修正重驗或 `--accept`」。SUGGESTION 不再出現在拒絕理由裡。用 outstanding 而非 unresolved：計數含帶 `(accepted)` 的必修行（D2——接受不豁免守門），已裁決未修者不宜稱 unresolved。

**D4 — 技能側的詢問門檻同步收窄。** 三選項詢問（修正後重審／接受現狀蓋章／先不蓋章）僅在必修 findings 存在時觸發；僅 SUGGESTION 的輪由技能直接乾淨蓋章（單站直接呼叫）或照舊「先不蓋章」離場（quality 時序中）。接受機制（`(accepted)` token＋`--accept`）收窄為必修級專用。quality 的帶保留章定義同步收窄：僅使用者裁示不修的必修構成保留。

**D5 — 衍生物同批再生。** 三份資產內文變更觸發 MARKER_VERSION 遞增（crates/speclink-core/src/init.rs）、golden 快照與 assets.lock 再生、claude 與 codex 兩個 render target 的技能檔再生。這是既有三連動慣例，不是新機制。

## Implementation Contract

**行為：**

- `speclink review stamp <change>` 與 `speclink verify stamp <change>`（無 `--accept`）：末輪僅含 SUGGESTION 級 findings 時 exit code 0、正常蓋章（五欄寫入＋工單刪除，原子性不變）；末輪含至少一筆 CRITICAL 或 WARNING 級未解 findings 時 exit code 非零、stderr 點名未解必修數並提示 `--accept` 或先修正重驗，metadata 與工單皆不變。
- `--accept` 行為不變：豁免必修擋章，照常蓋章。
- 任務未全完成的守門條件 (1) 不變。
- `--json` 輸出 shape 與欄位名不變；人眼輸出僅 stderr 拒絕訊息措辭改變。

**介面／資料形狀：** CLI 子指令、旗標、工單行格式、`(accepted)` token、`.openspec.yaml` 蓋章五欄——全部不變。變的只有守門判定與拒絕訊息文字。

**失敗模式：** 守門拒絕維持 Refusal（exit code 非零、無任何寫入）；壞 meta fail-closed 行為不變。

**驗收：**

- speclink-core 單元測試：末輪僅 SUGGESTION → stamp 成功；末輪含 WARNING → 拒絕且訊息點名必修；末輪含 CRITICAL＋SUGGESTION → 拒絕；`--accept` 照常放行——review 與 verify 兩站各自覆蓋（測試分別落在 crates/speclink-core/src/review.rs 與 crates/speclink-core/src/verify.rs 的測試模組，共用邏輯的測試落在 station.rs 測試模組）。
- CLI 整合測試（crates/speclink-cli/tests/it/review_verbs.rs、crates/speclink-cli/tests/it/verify_verbs.rs）：SUGGESTION-only 蓋章成功與必修拒絕的 exit code／stderr 斷言。
- golden：cargo test -p speclink-core --test it 的 render_golden 全綠（資產再生後）。
- 技能檔驗收：再生後的 claude 與 codex 技能檔含收窄後的迴圈敘述（必修觸發詢問、SUGGESTION 不擋章）。

**範圍邊界：** in scope——station.rs 守門與訊息、兩站測試、三份技能資產與衍生物、五份 spec delta。out of scope——meta 欄位新增、工單格式、desktop／server／protocol／Node SDK、analyze 分級、既有 `(accepted)` 舊工單的遷移（見 Migration）。

## Risks / Trade-offs

- [golden 與 CLI 測試回歸] 資產內文變更會使 golden 快照與 assets.lock 過期、拒絕訊息變更會使既有 stderr 斷言失敗 → 同一批 commit 內完成：MARKER_VERSION 遞增、golden 與 assets.lock 再生、測試斷言同步更新；跑全量 speclink-core＋speclink-cli 測試確認無漏網。
- [併行 change 的版號行對撞] 進行中的 cli-mode-dispatch-convergence 若也動 MARKER_VERSION，合併時版號行對撞 → 依既有慣例：合併後重生衍生物（golden／assets.lock／技能檔），不手動挑邊。
- [舊工單的語意漂移] 既有工單可能存有 SUGGESTION 級 `(accepted)` 行（舊流程產物）——新守門下它們本來就不擋章，行為只會變寬不會變嚴，無需遷移；必修級 `(accepted)` 行為不變。
- [severity 兼任阻斷分界] 記錄者把該擋的問題誤標 SUGGESTION 會靜默放行 → 技能資產明文收緊：WARNING 保留給必修級 correctness 判定、可裁一律 SUGGESTION;誤標風險本來就存在（舊流程誤標也會誤導使用者裁示），未新增攻擊面。
- [跨平台] 無新的路徑／換行／git 行為；拒絕訊息為純 ASCII stderr 文字，Windows／macOS／Linux 無差異。

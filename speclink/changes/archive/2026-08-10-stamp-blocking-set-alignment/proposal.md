## Why

兩站品質站（review／verify）存在兩套「什麼擋住蓋章」的定義，且都寫進了正典：引擎的乾淨蓋章守門要求「末輪零未解 findings」（不分嚴重度，SUGGESTION 也擋章），而兩站 skill 的「阻斷集」只算必修（must-fix：CRITICAL＋有現實觸發路徑的 WARNING）。落差的後果是：透過 AI 代理跑品質站的開發者每記一筆 SUGGESTION，該輪就注定到不了乾淨蓋章——只能修掉它、或停下問使用者 `--accept`。這產生反誘因：代理為省摩擦而不記 SUGGESTION，工單紀錄的誠實性被扭曲（討論 stamp-blocking-set-alignment 的結論；起點為長期撞到的實務摩擦）。

目標使用者是以 AI 代理執行 SDD 品質站（review／verify／quality 技能）的開發者；情境是 apply 完成後、archive 之前的品質收尾階段。

## What Changes

- **引擎蓋章守門收斂到嚴重度分界**：`review stamp` 與 `verify stamp` 的守門條件 (2) 從「工單末輪零未解 findings」改為「工單末輪零未解**必修** findings」——必修＝CRITICAL／WARNING 級；SUGGESTION 級不擋乾淨章，記了也能直接蓋。`--accept` 語意不變（帶保留蓋章），但豁免範圍收窄為必修級。
- **拒絕訊息改點名必修**：stamp 未帶 `--accept` 被必修 findings 擋下時，stderr 說明未解必修數量；僅 SUGGESTION 的末輪不再觸發拒絕。
- **兩站 skill 的可裁記法收緊**：可裁（discretionary）事項一律記為 SUGGESTION 級；WARNING 保留給必修級的 correctness 判定。三選項詢問僅在必修 findings 存在時觸發；僅 SUGGESTION 的輪由技能直接執行乾淨蓋章，無需使用者批准。`(accepted)` 標記機制照舊，但僅必修級的接受項需要 `--accept` 蓋章。
- **quality 技能的帶保留章定義收窄**：僅「使用者裁示不修的必修級 findings」構成帶保留章；SUGGESTION 殘留不構成保留，收尾補蓋直接落乾淨章。
- **相容性影響**：
  - CLI 行為：僅 SUGGESTION 的末輪執行 stamp（無 `--accept`）從 exit code 非零變為 0 並成功蓋章；stamp 拒絕時的 stderr 措辭改變（點名必修）。`--json` 輸出 shape 無變更。
  - 既有使用者遷移：無需動作——舊行為下擋章的必修 findings 依然擋章；放寬的只有 SUGGESTION 級。
  - 技能資產：verify／review／quality 三份 skill assets 內文變更，須同批完成 MARKER_VERSION 遞增、golden 快照與 assets.lock 再生，並再生 claude 與 codex 兩個 render target 的技能輸出（含 .claude/skills/ 下對應 SKILL.md）。

## Non-Goals

- 不在蓋章 meta 欄位記錄「是否 `--accept`」或殘留 SUGGESTION 數——誠實性維持靠工單的 git 歷史（討論 Deferred 項；日後有需要再開新討論）。
- 不引入行內「可裁」新標記讓引擎讀懂判斷式分類——嚴重度標籤已足以承載阻斷分界（討論已排除）。
- 不放行已接受（`(accepted)`）的必修 findings——`--accept` 的誠實儀式不變（討論已排除）。
- 不動 desktop GUI——其僅在 store 測試碰到 Severity 型別，無「零 findings 才能蓋」的呈現依賴。
- 不動 `speclink analyze` 的 Critical／Warning／Suggestion 分級——那是 artifact 一致性分析的獨立體系，與工單 findings 無關。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `verify-station`: 蓋章守門條件 (2) 由「末輪零未解 findings」改為「末輪零未解必修（CRITICAL／WARNING）findings」，SUGGESTION 不擋章；拒絕情境與乾淨蓋章情境隨之更新。
- `review-station`: 同上——守門條件 (2) 收斂到必修分界，Scenario 更新。
- `verify-skill`: 三選項詢問改為僅必修存在時觸發；僅 SUGGESTION 的 discovery／validation 輪直接乾淨蓋章；可裁事項一律記 SUGGESTION。
- `review-skill`: 同 verify-skill，另含 severity 對映收緊——可裁 smell 一律 SUGGESTION，WARNING 保留給必修級 correctness 判定。
- `quality-skill`: 帶保留章的構成收窄為必修級殘留；SUGGESTION 殘留的收尾補蓋落乾淨章。

## Impact

- Affected specs: `verify-station`、`review-station`、`verify-skill`、`review-skill`、`quality-skill`（皆為修改，無新增）
- Affected code:
  - Modified:
    - crates/speclink-core/src/station.rs（守門計數改為過濾 SUGGESTION；拒絕訊息）
    - crates/speclink-core/src/verify.rs、crates/speclink-core/src/review.rs（單元測試更新＋新增僅 SUGGESTION 蓋章案例）
    - crates/speclink-cli/tests/it/verify_verbs.rs、crates/speclink-cli/tests/it/review_verbs.rs（CLI 層測試）
    - crates/speclink-core/assets/skills/verify.md、crates/speclink-core/assets/skills/review.md、crates/speclink-core/assets/skills/quality.md（技能資產內文）
    - crates/speclink-core/src/init.rs（MARKER_VERSION 遞增）
    - crates/speclink-core/tests/golden/assets.lock 與 golden 快照（隨資產再生）
    - .claude/skills/speclink-verify/SKILL.md、.claude/skills/speclink-review/SKILL.md、.claude/skills/speclink-quality/SKILL.md（由資產再生流程同步）
  - New: (none)
  - Removed: (none)
- 影響的 crate／app：speclink-core（引擎與資產）、speclink-cli（測試）；desktop 與 server 不動。
- 併行注意：進行中變更 cli-mode-dispatch-convergence 若同樣觸發 MARKER_VERSION 遞增，合併時版號行會對撞——依既有慣例以重生衍生物解決，不挑邊。

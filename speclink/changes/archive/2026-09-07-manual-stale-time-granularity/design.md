## Context

手冊過期判定的兩個輸入都只有日期：正典 spec.md 內 `@trace` 區塊的 `updated:` 由 speclink-core 的 archive 以 `util::today()` 寫入；手冊頁 frontmatter 的 `generated` 由 manual 技能寫入。manual-pages 契約規定「規格日期不早於頁日期（同日也算）即過期」，desktop-core（apps/desktop/core/src/manual.rs）與 manual 技能都照此實作。同日封存後生成是常態工作流，因此每次生成後都會留下假警報，且同日再跑不會消失。

讀取 `updated:` 的程式只有 desktop-core 一處（正規表達式只認 `YYYY-MM-DD`）；`speclink trace`、規格頁 footer 與前端 trace 解析只讀 `source`。寫入端只有 archive 的 trace_block 一處，local 與 remote store 共用同一段核心程式。前端不顯示 `generated`。

## Goals / Non-Goals

**Goals:**

- 兩邊都有時間時，過期判定能分出同一天內的先後；生成後同日封存仍會被標記。
- 現有正典的純日期 `updated:` 與現有手冊頁的純日期 `generated` 不需遷移，照舊可讀、照舊以「同日也算」判定。
- 生成端與讀取端仍用同一套判定基準，基準只寫在 manual-pages 契約。

**Non-Goals:**

- 不回改 201 個封存變更留下的既有 `updated:` 行。
- 不改 UI 文案「可能過期」，不改 i18n 字串。
- 不用 git commit 時間判定（兩端都要跑 git、未 commit 無時間，違反同基準）。
- 不改「只比日但晚於才算」（生成後同日封存會永遠漏判，契約明文拒絕）。
- 不動封存目錄名的日期前綴（`YYYY-MM-DD-<change>`）。
- 不動 remote 模式的手冊行為（仍是空狀態）。

## Decisions

### 時戳格式採 RFC 3339 帶時區偏移量，秒級

`2026-09-05T23:17:28+08:00` 這種寫法。理由：跨機器封存與生成是常態，本地時間不帶偏移會在兩台機器間反向誤判；帶偏移量後兩個時戳可換算成同一瞬間比較。精度到秒即可，同一秒不算過期。
替代方案：UTC 加 `Z`——也是合法 RFC 3339，讀取端一併接受；但寫入端統一輸出本地偏移量，讓人眼讀到的日期與封存目錄名的日期前綴一致。
替代方案：Unix epoch 整數——人眼不可讀，與 frontmatter 其他欄位風格不合。

### archive 以同一個「現在」產生目錄日期前綴與 updated 時戳

speclink-core 的 util 新增一個函式，一次取得 `chrono::Local::now()` 並回傳（純日期、RFC 3339 時戳）兩個字串；archive 用純日期組封存目錄名、用時戳組 trace 區塊。避免跨午夜時目錄名與 updated 落在不同日。`util::today()` 保留給其他呼叫端。trace_block 仍是唯一寫入點，local 與 remote store 共用。

### 判定改三段式，以量詞取代最大／最小值

契約規則：

1. 兩邊都帶時間：規格時戳晚於頁時戳（換算同一瞬間後嚴格大於）才算過期。
2. 任一邊只有日期：規格的日曆日不早於頁的日曆日（同日也算）即過期。帶時間的一方取其時戳自身偏移量下的日曆日。
3. `sources` 為空、`generated` 缺席或格式不對、規格不存在或無可解析的 `updated:`：不標記。

desktop-core 實作上不再算每個 capability 的（最小、最大）日期，改成：
- 過期＝頁的任一 source 規格內，存在任一 `updated` 時戳「在頁的 generated 之後」（依上述兩段規則）。
- 未入冊＝該 capability 不在任何頁的 sources，且它的每一個 `updated` 時戳都在每一頁的 generated 之後。
理由：純日期與帶時間混在同一組時沒有自然的全序，量詞寫法不需要定義跨格式排序，也和原本的「最大 updated 對 generated」「最小 updated 對最大 generated」在單一格式下等價。

### 讀取端同時接受純日期與 RFC 3339

desktop-core 以一個小型 enum 表示時戳（純日 或 帶偏移量的瞬間）。解析順序：先試 RFC 3339，再試 `YYYY-MM-DD`，都失敗視為不存在。`updated:` 行的正規表達式改抓整段非空白字元，交給同一個解析器。`generated` 走同一個解析器。JSON 索引裡的 `generated` 欄位改回傳 frontmatter 原字串（先前是重新格式化成 `%Y-%m-%d`），格式不對時仍為 null；前端不顯示此欄，無畫面影響。

### manual 技能寫入帶偏移量的本地時戳

asset（crates/speclink-core/assets/skills/manual.md）的 frontmatter 表把 `generated` 格式改為 RFC 3339，stale page 定義改為三段式，範例 frontmatter 改用時戳。技能檔給 agent 一條取得時戳的建議指令（python3 的 `datetime.now().astimezone().isoformat(timespec="seconds")`；無 python3 時以 date 指令輸出並把偏移量補上冒號）。改 asset 內文即要 bump ASSET_VERSION（v1.29.0 → v1.30.0）、重生 render_golden 四份快照與 assets.lock；渲染到 .claude 與 .agents 的 SKILL.md 由 speclink update 再生，不手改。

### 詞條只更新定義

LANGUAGE.md「可能過期」詞條的 definition 改寫為三段式規則，avoid 與 why 不動；「可能」的理由（比對只證明來源動過）在時間比較下仍成立。

## Implementation Contract

**Behavior**

- `speclink archive <change>` 成功後，被物化的每個正典需求下方的 trace 區塊為三行：`<!-- @trace`、`source: <change>`、`updated: <RFC 3339 時戳>`、`-->`。時戳的日曆日與封存目錄名前綴相同。
- desktop 的 `list_manual_pages` 索引：每頁 `stale` 依契約三段式計算；`uncoveredNew` 依三段式的「全部晚於」計算；`generated` 為 frontmatter 原字串或 null。
- manual 技能生成或重生的頁，frontmatter `generated` 為 RFC 3339 時戳；技能的過期報告與 desktop 對同一組檔案給出相同的過期頁集合。

**Interface / data shape**

- speclink-core `util`：新增回傳（純日期字串、RFC 3339 時戳字串）的函式，兩者取自同一個 `Local::now()`。
- desktop-core `manual.rs`：時戳 enum（純日／瞬間）與「後者是否在前者之後」的比較函式；`updated:` 正規表達式改為 `^\s*updated:\s*(\S+)\s*$`。
- 索引 JSON 欄位名不變：`slug`、`title`、`section`、`order`、`keywords`、`sources`、`generated`、`stale`、`uncoveredNew`、`malformed`。

**Failure modes**

- `generated` 或 `updated:` 不是純日期也不是 RFC 3339：視為缺席，該頁不標記、該時戳不參與未入冊判定；不報錯、不寫 log。
- 規格不存在或無 trace：該 source 不參與判定。

**Acceptance criteria**

- speclink-core archive 單元測試：封存後正典含 `updated: ` 後接符合 `\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[+-]\d{2}:\d{2}` 的時戳，且其前十字元等於封存目錄名前綴。
- desktop-core 單元測試以固定字串覆蓋判定表：
  | 頁 generated | 規格 updated | stale |
  | --- | --- | --- |
  | 2026-09-05T23:31:00+08:00 | 2026-09-05T23:17:28+08:00 | false |
  | 2026-09-05T23:31:00+08:00 | 2026-09-05T23:40:00+08:00 | true |
  | 2026-09-05T23:31:00+08:00 | 2026-09-05T23:31:00+08:00 | false（同秒） |
  | 2026-09-05T23:31:00+08:00 | 2026-09-05T15:40:00Z | true（換算同一瞬間為 23:40+08:00） |
  | 2026-09-05T23:31:00+08:00 | 2026-09-05 | true（退回同日） |
  | 2026-09-05 | 2026-09-05T23:17:28+08:00 | true（退回同日） |
  | 2026-09-06T00:10:00+08:00 | 2026-09-05 | false |
  | 2026-09-05T23:31:00+08:00 | not-a-date | false |
  未入冊以同表對稱覆蓋：全部 `updated` 都晚於全部 `generated` 才計入。
- render_golden 測試通過且 assets.lock 版號為 v1.30.0；`speclink update` 後 .claude 與 .agents 的 speclink-manual SKILL.md 與 golden 一致。
- `speclink validate manual-stale-time-granularity` 通過。

**Scope boundaries**

- In scope：archive 寫入格式、util 時戳函式、desktop-core 判定與解析、manual 技能 asset 與版號連動、四份規格、LANGUAGE.md 詞條。
- Out of scope：i18n 字串、前端元件、`speclink trace` 輸出、server-web、既有正典與手冊頁的回改、封存目錄命名。

## Risks / Trade-offs

- [archive 單元測試以 `util::today()` 組出的期望值全紅] → 改成正規表達式斷言時戳格式，並以時戳前十字元對照目錄名，測試不依賴牆鐘。
- [render_golden 與 assets.lock 紅燈] → 這是刻意變更：先 bump ASSET_VERSION、cargo build、再跑 speclink update 與 golden 再生；再生的 37 份 SKILL.md 不進 evidence，收尾以 git status 盤點。
- [跨平台時區] → chrono 的 `Local` 在 Windows／macOS／Linux 都能給偏移量；測試不用牆鐘，只用固定字串。
- [agent 取時戳的指令跨平台] → 技能檔給 python3 一行指令為主、date 為備；格式錯誤時讀取端視為缺席，只會少標不會炸。
- [純日期與時戳混用期間的判定] → 退回同日規則，最壞情況與今日相同（假警報），下一個日曆日重生即收斂。
- [手冊頁 `generated` 由 agent 手寫] → 契約以 RFC 3339 為準，desktop 同時接受純日期，寫錯不會讓頁消失。

## Migration Plan

1. 合併後不需資料遷移；既有正典與手冊頁照舊可讀。
2. 使用者下次執行 speclink update 取得新版技能檔；下次封存起 `updated:` 帶時間；下次生成起 `generated` 帶時間。
3. 回退：還原這批程式碼即可，已寫入的 RFC 3339 `updated:` 行會被舊 desktop-core 的日期正規表達式視為不存在（不標記），不會炸。

## Open Questions

無。

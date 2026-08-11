## Context

Purpose 相關機制現況：archive 建新 capability 時 delta 無 `## Purpose` 即寫入佔位句（crates/speclink-core/src/archive.rs 的 delta_purpose 與佔位分支）；規格指引 asset（crates/speclink-core/assets/schema/spec-driven/specs.instruction.md，經 include_str! 編入、由 instructions 動詞注入給代理）全文零次提及 Purpose；CLI `validate --specs` 旗標已宣告但從未被讀取（crates/speclink-cli/src/verbs/checks.rs），core 只有 validate_change、無正典規格驗證入口。上游 OpenSpec 的對照：schema 指引寫死規則（50+ 字元）、zod schema 把 Purpose 設必填、validate 錯誤訊息附 GUIDE_* 修復指引與範例骨架；其 36 個 capability 零佔位殘留。桌面端已有佔位偵測前綴（apps/desktop/core/src/query.rs 的 PURPOSE_TBD_PREFIX，以測試與 archive 產生器釘住一致）。

## Goals / Non-Goals

**Goals**

- 新 capability 的 delta 缺合格 Purpose 時：propose 收尾的 validate 當場報 error（帶修復指引）、archive 拒絕放行。
- `validate --specs` 接上正典規格驗證：缺段／空＝error，過短＝warning（strict），佔位句＝warning。
- 規則的單一正典落在 specs.instruction.md；門檻常數（50 字元）單一定義三處共用。

**Non-Goals**

- 不改 propose／archive 技能文字的規則內容（validate-only；技能既有的「fix errors and re-validate」迴圈即承載修復，討論第七輪裁定）。
- 不動既有 capability 的 delta Purpose 忽略語意（delta_purpose 永不覆蓋既有正典 Purpose，含測試釘住的行為原樣保留）。
- 不做上游的另兩項帶入期品質檢查（目標已有不同 Purpose 時警告、帶入後不可讀退回佔位）——討論明列 Deferred。
- 存量 66 份佔位的補寫不在本 change（獨立 change spec-purpose-backfill，以本 change 的 validate --specs 為驗收工具）。

## Decisions

### D1 合格判準單一函式、門檻常數單一定義

core 新增 Purpose 合格判準（存在 `## Purpose` 區段、內容非空、trim 後長度 ≥ 50 字元）與常數 MIN_PURPOSE_LENGTH = 50（對齊上游同名常數值）。change 早期檢查、archive 硬擋、正典規格驗證三處呼叫同一判準——不合格的定義只有一份，三道防線不會漂移。長度以字元計（Rust chars().count()，非 bytes）——中文 Purpose 不因 UTF-8 多位元組被高估。

### D2 change 早期檢查掛在 validate_change、僅對「新開 capability」觸發

validate_change 對每個 delta capability 判斷正典是否已有該 capability（store 的正典規格存在性）：不存在（＝本 change 新開）且 delta Purpose 不合格 → error；已存在 → 不檢查（既有 capability 的 delta 不該寫 Purpose，寫了也被忽略，不報錯維持向後相容）。錯誤訊息照上游 GUIDE_* 模式自帶修復指引：說明規則（新 capability 的 delta 以 `## Purpose` 開頭、一兩句、50 字元以上）並附範例骨架——propose 技能收尾跑 validate 失敗即修，錯誤訊息即教材。

### D3 archive 硬擋取代佔位寫入、加入既有 fail-closed 守門清單

archive 的合併守門對「正典尚無此 capability 且 delta Purpose 不合格」回拒絕（violation 進既有 merge_violations 通道，維持零半套寫入語意）；通過者照現行 delta_purpose 帶入。佔位寫入分支保留為死路防禦（理論上不可達——守門先擋），不刪除：REMOVED 語意的正典生成路徑與歷史測試依賴其存在。skip_specs 封存不觸發（無 delta 可驗）。

### D4 validate --specs 的接線與兩模式同形

CLI `--specs` 接上 core 新增的正典規格驗證：對每份正典規格報——缺 `## Purpose` 區段或內容為空＝error；內容不足 50 字元＝warning（僅 --strict 報）；內容以佔位前綴開頭＝warning（佔位句長度恆超過 50，長度門檻抓不到它，需獨立判準）。輸出沿用既有 render_validate_results 渲染（逐項 ✓/✗＋error/warn，any invalid 非零收尾）。旗標語意：`--specs`＝只驗規格；`--all`＝changes＋specs 皆驗；預設（皆缺席）＝現行 change 驗證不變——向後相容。remote 模式由 client 以既有正典規格讀取端點取內容、本地跑同一驗證器，輸出兩模式同形（比照 show 的組合語意，不新開 server 端點）。

### D5 佔位偵測前綴移至 core 單一定義

佔位前綴「TBD - created by archiving」自 desktop 的 PURPOSE_TBD_PREFIX 上移為 speclink-core 的公開常數，archive 產生器、--specs 佔位判準、desktop query.rs 三處共用；desktop 既有的一致性測試改釘 core 常數。單一真相取代「兩處字串＋測試釘一致」。

### D6 asset 內文變更的三連動

specs.instruction.md 內文變更依既有正典紀律連動：MARKER_VERSION 遞增、golden 快照再生、assets.lock 更新。規則段內容對齊上游 schema.yaml 的 Purpose 段（新 capability 才寫、50+ 字元、既有 capability 不要寫、改既有 Purpose 直接編輯正典檔、附範例），措辭融入現有 asset 的行文風格。

## Risks / Trade-offs

- **硬擋比上游嚴**：上游缺 Purpose 仍放行寫佔位；我們拒絕。風險是舊 change（守門上線前建立、delta 無 Purpose）封存時被擋——屬預期行為（錯誤訊息指路補一段即可），且與 archive 既有 fail-closed 風格一致。
- **50 字元對中文的鬆緊**：50 個中文字元約一兩句完整說明，與上游英文 50 字元的資訊量相當偏嚴格；但單一數字三處共用的簡單性優先，實跑偏緊再調常數（一處改）。
- **`--specs` 與 `--changes` 同傳**：語意採聯集（兩者皆驗），與 `--all` 等效——避免旗標組合矩陣。

## Migration Plan

無資料遷移。存量 66 份佔位在本 change 落地後由 `validate --specs` 以 warning 顯形（不擋 change 驗證、不擋封存既有 capability 的 delta），補寫由後續 change spec-purpose-backfill 收拾。

## Open Questions

（無）

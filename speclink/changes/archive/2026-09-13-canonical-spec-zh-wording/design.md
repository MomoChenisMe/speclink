## Context

使用者裁定：繁中把 OpenSpec 的 specs（`openspec/specs/<capability>/spec.md`）叫「正式規格」，與變更裡的「delta 規格」成對；英文維持 specs／canonical specs。討論 canonical-spec-zh-wording 兩輪定案，第二輪選「連光桿也改」。

**現況盤點（提案時複驗，不含封存區）**。同一個概念有五種寫法，分佈如下：

| 寫法 | 正式規格散文 | 手冊 | docs／README | 程式碼與測試 | 使用者看得到的字串 |
| --- | --- | --- | --- | --- | --- |
| 正典規格 | 45 | 36 | 10 | 37 | 2（packages/ui/src/i18n.tsx 的 specs.heading、specs.empty） |
| 正典 spec(s)／正典 spec.md | 23 | 0 | 10 | 19 | 0 |
| 規格正典 | 0 | 0 | 9 | 0 | 0 |
| 光桿「正典」指 specs 這一側 | 約 115 行 | 約 45 行 | 約 25 行 | 約 100 行（不改） | 0 |
| 光桿「正典」的別義 | 約 40 行 | 4 行 | 約 25 行 | — | 0 |

delta 那一側已經一致：i18n 與手冊寫「delta 規格」，docs 與規格寫「delta specs」，不需要動。

**約束一：詞彙守門只擋新增、不追舊帳，但 avoid 一加就照出存量。** `scripts/docs/vocabulary-guard.test.mjs` 動態解析 `openspec/LANGUAGE.md` 每條 avoid，掃描面是 desktop／server-web／ui 三份 i18n、README 兩份、技能資產與 `docs/` 全部 markdown。帶括號限定的 avoid 詞只有限定語是「使用者可見文案中」才進守門，其他限定語一律跳過。守門紅時 CI 的 UI／Desktop／Server Web／Rust 四組會被整批跳過。

**約束二：光桿「正典」不是一個概念。** 專案內至少五種意思——specs 集合、正典詞彙（LANGUAGE.md）、正典值／正典檔／正典歸屬（workflow-config）、正典化／正典模板（技能生成）、正典 YAML／正典載入（workflow-schemas），以及「本文是⋯的正典」這類泛指唯一真相的用法。機械替換必然改壞別義。

**約束三：改標題的兩條路風險不同。** 走 delta 的 MODIFIED 整塊取代改 Requirement 或 Scenario 名，引擎視為未宣告刪除，validate 與 analyze 都抓不到、要到 archive 才炸。直接改正式規格檔則不經合併，沒有這條雷；溯源（trace）按 `@trace` 區塊與位置歸屬、不按標題名配對，改名不影響溯源鏈。

**約束四：手冊的過期判定看時戳、不看內文。** 手冊頁只在 sources 所指規格的 `@trace updated` 晚於頁的 generated 時才算「可能過期」並重生。直接改正式規格檔不動 `@trace`，手冊不會被判過期。

## Goals / Non-Goals

**Goals:**

- 活的檔案裡「正典規格」「正典 spec(s)」「規格正典」歸零；指 specs 這一側的光桿「正典」在正式規格、手冊、docs、README 全數改成「正式規格」。
- 「正式規格」取得詞條地位，舊寫法進 avoid，守門擋住回流。
- 零行為變化：不改識別符、CLI 旗標、`--json` 形狀、技能資產或任何邏輯。

**Non-Goals:**

- 不回改 `openspec/changes/archive/`、`openspec/discussions/archive/`、CHANGELOG.md、release-notes.json——歷史 artifacts 不回改（LANGUAGE.md 原則；D4）。
- 不改光桿「正典」的別義：正典詞彙、正典值、正典化、正典 YAML、正典順序、「本文是⋯的正典」（D1）。
- 不改程式碼註解裡的光桿「正典」；只改複合寫法、引用改名標題的註解，以及審查站點名的同句混用（D6）。
- 不改英文：桌面英文版 "Canonical specs"、技能裡的 canon、`--specs` 旗標維持。
- 不為「delta」直出於繁中文案立明文例外——討論列為延後，不在本變更決定。
- 不改 `openspec/config.yaml` 的 context 文字（兩處「正典」皆為泛指唯一真相的別義）。

## Decisions

### D1：光桿「正典」的改與不改以一條判準決定

**決定**：只有指「`openspec/specs/` 合併後的規格集合（相對 delta）」的「正典」才改成「正式規格」。其他一律不動。

**判準的具體樣貌**：

- 改：「delta 合併進正典」「正典中不存在該 capability」「正典有 auth」「正典未動」「正典全文」「正典 Purpose」「遠端正典在 server 上」「正典 → 舊討論查核 → 程式碼」。
- 不改：「正典詞彙」「正典詞」「正典值」「正典檔」「正典歸屬」「正典序」「正典化」「正典模板」「正典原文」「正典 YAML」「正典載入」「正典順序交棒」「品質站正典」「本文是目前能不能用的正典」「Rust 型別為正典」「Query 加 ETag 重讀正典」（後者指 server 資料面的唯一真相，不是規格檔）。

**每份正式規格的處置**（提案時逐行判讀）：

- 全改：archive-merge、spec-validation、capability-naming-guard、change-analysis、desktop-manual-page、manual-pages、manual-skill、context-projection、verify-evidence、drift-computation、change-lifecycle、task-identity。
- 部分改：desktop-app（正典內容／正典全文／正典 Purpose 改；「正典詞」五處不改）、user-documentation（「遠端的正典在 Store」「讀者不需翻正典」「與正典一致」「server-identity 正典」改；「使用流程正典」「能力狀態正典」「正典詞彙」「工作流正典」「安全邊界的正典」不改）、discuss-skill（正典證據／不等正典／正典讀取／正典先行／正典接地／與正典衝突／正典零命中／正典 → 舊討論查核 → 程式碼 改；「正典詞彙轉譯」不改）。
- 只改複合寫法、光桿不動：delivery-baseline（「現行正典為 crates/<group>」是路徑慣例）、phase2-acceptance（「與正典一致」指 server 查詢結果）、remote-workspace-data（「重讀正典」指資料面）、propose-skill（「正典化生成」是技能生成）。
- 只有複合寫法、無光桿：archive-skill、server-context-api、trace-skill、trace-verb、verb-contract。
- 完全不動：config-skill、remote-connection、workspace-tools、baseline-skill、review-skill、quality-skill、verify-skill、manual-task-marker、client-protocol、workflow-schemas、workflow-config、ui-copy-vocabulary、worktree-apply-skill、worktree-merge-skill、discussion-docs。

**手冊與 docs 的處置**：手冊 16 頁的光桿全改（policy-config「config.yaml 裡的正典內容」與 desktop-quality「正典詞」不動，about 頁的「正典檔」「工作流正典」「正典詞彙」不動）；docs 改「行為與邊界的正典則是 openspec/specs/」「規則的正典有兩份」「動詞契約正典」「Client Protocol 正典」「正典動詞契約」「合併正典」「Host 邊界的正典是 openspec/specs/」「開箱流程的正典是 openspec/specs/」「遠端正典」（implementation-refactor-roadmap 的 projection 一句）這類直接指規格檔的句子；platform-architecture 只改複合寫法與文件狀態欄那句直接指規格檔的「現行的行為正典是 openspec/specs/ 底下的規格」，光桿「遠端正典」指整個遠端 store 不動。

**替代方案：只改複合寫法**。先例 zh-tw-vocabulary-drawer-and-quality-station 的 D1 就是這樣做。否決：delta 與 specs 對照最密集的句子（archive-merge 拒絕封存的七種情形、spec-validation 的守門）用的正是光桿，改完仍是「正典」，沒達到分清楚的目標。

### D2：「正式規格」立為 LANGUAGE.md 詞條，四個複合寫法進機械守門、光桿帶語境限定

**決定**：在 `openspec/LANGUAGE.md` 詞彙末尾新增：

```
### 正式規格

- **definition**: `openspec/specs/<capability>/spec.md`——封存後的現況唯一真相，對照 OpenSpec 的 specs。與變更裡的「delta 規格」（`openspec/changes/<name>/specs/`，封存時合併進正式規格）成對；英文維持 specs／canonical specs。
- **avoid**: 正典規格、正典 spec、正典 specs、規格正典、正典（指這個規格集合時）
- **why**: 「正典」讀不出「合併後的定案」，與 delta 對著看才分得清哪個是草稿、哪個是定案。「正典」在專案內另有正典詞彙、正典值、正典化、正典 YAML 等別義，那些不改。2026-09-12 討論「canonical-spec-zh-wording」定案。
```

**守門行為**：前四個 avoid 含中文字、無限定語，進機械守門；「正典（指這個規格集合時）」的限定語不是「使用者可見文案中」，守門跳過——光桿「正典」有別義，進守門會誤命中「正典詞彙」等正當用法。「正典 spec」同時涵蓋「正典 spec.md」與「正典 specs」的行首匹配。

**同批清零的守門面存量**：i18n.tsx 兩條、README「規格正典」四處、docs 十份的複合寫法。技能資產與 desktop／server-web 兩份 messages.ts 零命中。

**替代方案：不立詞條、只改字**。否決：沒有 avoid 就沒有守門，下一個寫文案的代理沒依據，改完會漂回來——這正是先例變更關掉的洞。

### D3：正式規格散文直接改字、零 delta，標題改名走直編

**決定**：25 份正式規格用找字替換加逐句判讀直接改，不寫 delta。16 個 Requirement／Scenario 標題直接改名：

| 規格 | 舊標題 | 新標題 |
| --- | --- | --- |
| spec-validation | validate --specs 驗證正典規格 | validate --specs 驗證正式規格 |
| server-read-api | 正典 spec 內文可讀 | 正式規格內文可讀 |
| server-read-api | 讀取存在的正典 spec | 讀取存在的正式規格 |
| archive-merge | snapshot 先於正典寫入 | snapshot 先於正式規格寫入 |
| archive-merge | 既有正典 Purpose 不受 delta 影響 | 既有正式規格 Purpose 不受 delta 影響 |
| change-analysis | MODIFIED 需求不在正典 | MODIFIED 需求不在正式規格 |
| change-analysis | 無正典時每 capability 一筆 | 無正式規格時每 capability 一筆 |
| capability-naming-guard | 命中正典名稱照常放行 | 命中正式規格名稱照常放行 |
| context-projection | 投影含正典與 delta specs | 投影含正式規格與 delta specs |
| desktop-app | 選定 spec 以抽屜顯示其正典內容 | 選定 spec 以抽屜顯示其正式規格內容 |
| desktop-app | 展開卡片顯示正典全文 | 展開卡片顯示正式規格全文 |
| discuss-skill | 正典先行的漏斗偵察 | 正式規格先行的漏斗偵察 |
| discuss-skill | 正典接地與三分對照 | 正式規格接地與三分對照 |
| discuss-skill | 需求與正典衝突時列為假設 | 需求與正式規格衝突時列為假設 |
| discuss-skill | 正典零命中時流程照舊 | 正式規格零命中時流程照舊 |
| user-documentation | 非成員錯誤碼敘述與正典一致 | 非成員錯誤碼敘述與正式規格一致 |

**理由**：零語意變化——每條需求的 SHALL 與 scenario 的 WHEN／THEN 都不變，只換名詞。走 delta 要為約 40 條需求各寫一個 MODIFIED 整塊，而標題改名在 MODIFIED 裡是未宣告刪除（約束三）。先例 spec-purpose-backfill 直編 67 份規格的 Purpose、零 delta。

**不動 `@trace`**：`@trace` 記錄哪個變更最後動過該需求，本變更不是語意變更，不改寫歸屬；代價是 `speclink trace` 對這些需求仍指向原變更，可接受。

**在途 delta 的撞名檢查**：直編前確認沒有在途變更對這 16 個標題所在的 capability 帶 MODIFIED delta（提案時 `speclink list` 為零在途變更）。若日後有，改名後那份 delta 的來源需求名會對不上，archive 會以「來源需求名不存在」拒絕——這是既有守門，不需新機制。

**替代方案：走 delta 的 RENAMED Requirements**。可以處理 Requirement 改名，但處理不了 Scenario 改名（RENAMED 只認需求層），16 個標題有 13 個是 Scenario。否決。

### D4：已封存的 90 檔不回改

**決定**：`openspec/changes/archive/`（82 檔，含 `.evidence.json` 與封存的 delta 規格）、`openspec/discussions/archive/`（8 檔）、CHANGELOG.md、release-notes.json 一字不動。

**理由**：LANGUAGE.md 原則「歷史 artifacts（已封存的討論／變更）不回改」；封存區的 delta 規格與 `.evidence.json` 是溯源鏈的證據，改了等於改歷史。這與使用者最初的「所有」有衝突，討論第一輪明列、使用者未反對。

### D5：手冊直接改字、不重生，generated 欄位不動

**決定**：16 頁手冊用與正式規格相同的判準直接改字；frontmatter 的 `generated` 維持原值。

**理由**：約束四——直編規格不動 `@trace`，重生不會觸發；硬要重生等於整本重寫，diff 不可審。`generated` 不推進是因為內文不是技能這次生成的，推進會偽造生成紀錄。代價：手冊內文與規格散文都由人改，兩邊各自對判準負責，驗收時以 grep 交叉確認。

### D6：程式碼與測試只改兩類文字

**決定**：（1）複合寫法「正典規格」「正典 spec(s)」「正典 spec.md」——含 doc comment、行內註解、測試的 assert 訊息與 fixture 標題；（2）引用了 D3 改名標題的註解——「validate --specs 驗證正典規格」三處、「命中正典名稱照常放行」兩處、「MODIFIED 需求不在正典」「無正典時每 capability 一筆」「snapshot 先於正典寫入」「既有正典 Purpose 不受 delta 影響」「選定 spec 以抽屜顯示其正典內容」各一處。光桿「正典」的其他註解不動；唯一例外是審查站點名的同句混用——同一句或同一段一半已改成「正式規格」、一半還是「正典」——這種一併改齊（第 2 輪實際改了 manual_pages_dir_ignored.rs、validate.rs、phase2_chain.rs 三處註解、phase2_chain.rs 的 step 標籤與 App.test.tsx 的測試名）。

**理由**：註解不是使用者看得到的字，不在詞彙表範圍；改複合寫法是因為使用者要「所有」，成本是純替換；改標題引用是因為註解指向規格標題，標題改了引用不改就變成指向不存在的段落。唯一的真字串（i18n 兩條）與唯一的真斷言（specList 測試）歸使用者可見面，在 D2 同批。

**crate 邊界**：不動任何邏輯——speclink-core、speclink-cli、speclink-server、apps/desktop 的改動全是註解與測試字串，packages/ui 只動 i18n 字面與註解。Host／Protocol 契約零變更。

### D7：驗收面

**決定**：五道檢查全綠才算完成：

1. `node --test "scripts/**/*.test.mjs"`——詞彙守門與 docs 對等測試（H2 序列與截圖集合不受影響：所有含「正典」的 H2 都不在成對文件內）。
2. `speclink validate --specs --strict`——25 份直編規格仍是合法規格（Purpose 門檻不變）。
3. `npm test -w packages/ui`——specList 斷言與 fixture 更新後綠。
4. `cargo test -p speclink-core --test it render_golden::`——golden 不動的回歸對照。
5. grep 歸零：活的檔案（排除 archive、node_modules、target、dist）裡「正典規格」「正典 spec」「規格正典」零命中；「正典」剩餘命中逐一屬於 D1 不改清單。

## Implementation Contract

- **可觀察行為**：桌面 zh-TW 規格頁標題顯示「正式規格」，無規格時顯示「此專案尚無正式規格」；英文版不變。CLI 人眼輸出與 `--json` 完全不變。
- **守門契約**：`openspec/LANGUAGE.md` 詞條「正式規格」存在後，vocabulary-guard 對掃描面內任一「正典規格」「正典 spec」「正典 specs」「規格正典」報違規；光桿「正典」不報。
- **規格契約**：25 份正式規格的需求數、scenario 數、每條的 SHALL 語意與 WHEN／THEN 結構不變；只有 D3 表列的 16 個標題與散文名詞改字。
- **失敗模式**：無執行期行為，唯一的失敗面是測試——守門紅表示掃描面有漏改；validate --specs 紅表示規格格式被改壞；ui 測試紅表示 i18n 與斷言不同步。
- **驗收**：D7 五道全綠。
- **範圍邊界**：in scope＝提案 Impact 列出的檔案；out of scope＝封存區、別義的「正典」、程式碼註解的光桿「正典」（審查站點名的同句混用除外）、英文文案、技能資產、任何識別符。

## Risks / Trade-offs

- [回歸對照：golden 與 CLI 測試] → 技能資產零命中，golden 不會變；CLI 測試只改註解與 assert 訊息，跑 `cargo test -p speclink-cli --test it` 確認可編譯且綠。
- [跨平台：CRLF 與路徑] → 守門測試以 LF 與 CRLF 同一套判定；本變更只改 markdown 與字串，不動路徑邏輯。Windows CI 只跑同一份守門測試，無額外面。
- [逐句判讀漏改或誤改] → D1 列出每份規格的處置與不改清單；驗收第 5 道 grep 剩餘「正典」逐一對照清單；使用者抽審改動最多的兩份規格（archive-merge、discuss-skill）。
- [手冊與規格改字不同步] → 兩邊同一判準；驗收 grep 對手冊與規格分別歸零。
- [守門紅燈整批跳過 CI 四組] → D2 明定同批清零；提交前本地先跑守門測試。
- [平行 change 對撞] → 提案時零在途變更；開工前再以 `speclink list` 複查，若有在途 delta 觸及 D3 的 capability，先協調順序。

## Migration Plan

無。純文字變更，不需遷移；既有工作區不需執行任何指令。回滾＝git revert 單一 commit。

## Open Questions

無。

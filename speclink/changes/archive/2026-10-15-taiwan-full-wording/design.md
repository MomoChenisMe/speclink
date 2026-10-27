## Context

「全量」散在三類給人讀的文件裡：正式規格 13 份、手冊 4 頁、openspec/config.yaml 的一條 tasks 規則。它在專案內有五種意思，討論 test-speed-and-wording 定案按意思換成台灣慣用詞，英文維持 full test suite。

現況盤點（2026-10-15，排除 openspec/changes/archive 與 openspec/discussions）：

- 正式規格：config-skill 以外 12 份共 28 處；config-skill 目前 4 處，但在途變更 config-skill-rule-routing 的 delta 會整塊取代含這 4 處的需求，封存後依目前 delta 為 7 處。
- 手冊：manual、policy-config、remote-context、remote-offline 四頁共 7 處。
- openspec/config.yaml：rules 的 tasks 段有一條規則 1 處。
- vocabulary-guard 的掃描面（三份 i18n、README.md、README.en.md、docs、技能原稿）：0 處。
- 程式碼註解與測試名稱：約 25 檔，不在本變更範圍（見 Non-Goals），只有 D7 的一行例外。

本變更不動任何 crate 或 app 的程式碼，不涉及 Host／Protocol 邊界、YAML 文字手術或 local／remote 雙路徑。

## Goals / Non-Goals

**Goals:**

- 本變更範圍內的檔案（13 份正式規格、4 頁手冊、openspec/config.yaml）「全量」零命中。
- openspec/LANGUAGE.md 有「完整測試」詞條，avoid 收「全量」，vocabulary-guard 自動把它納入守門詞。
- 4 個含「全量」的 Requirement／Scenario 標題改名，引用舊標題的地方同批改齊。
- 改完後 `speclink validate --specs --strict` 與 scripts 守門全綠，需求數與 scenario 數不變。

**Non-Goals:**

- 不改英文：技能原稿已用 full test suite，不動技能原稿，不進 ASSET_VERSION。
- 不改程式碼註解與測試名稱裡的「全量」（約 25 檔）：不是給使用者讀的文件，漸進汰換。唯一例外見 D7。
- 不動封存區（openspec/changes/archive/、openspec/discussions/archive/）、在途討論記錄、CHANGELOG.md、release-notes.json。
- 不走 delta、不經封存合併、不動 `@trace` 時戳。
- 不重生手冊、不改手冊 frontmatter。
- 不改在途變更 config-skill-rule-routing 的任何檔案：它正在過品質關卡，改了會讓章過期。

## Decisions

### D1 按意思分流的替換對照表

原則：修飾動作時用「全部」，指一整套東西時用「完整」。逐處對照（左為現行字面片段，右為改後）：

| 意思 | 所在 | 現行 | 改後 |
|---|---|---|---|
| 測試範圍 | config-skill（封存後字面） | 驗證步驟要包含全量測試 | 驗證步驟要包含完整測試 |
| 測試範圍 | config-skill | 未設定、預設全量／預設全量／表格「未設定，預設全量」 | 預設完整測試／表格「未設定，預設完整測試」 |
| 測試範圍 | config-skill | 答「全量」時；使用者選擇全量時 | 答「完整測試」時；使用者選擇完整測試時 |
| 測試範圍 | delivery-baseline Purpose | root 一道指令跑完的全量驗證 | root 一道指令跑完的完整驗證 |
| 測試範圍 | delivery-baseline | 並執行 root 全量驗證 | 並執行 root 完整驗證 |
| 測試範圍 | review-skill、verify-skill | repo 慣例的全量測試指令 | repo 慣例的完整測試指令 |
| 測試範圍 | review-skill、verify-skill | 不執行 workspace 全量測試／不執行全量測試 | 不執行 workspace 完整測試／不執行完整測試 |
| 測試範圍 | policy-config 手冊頁 | 要跑全量測試還是只跑受影響面；答「全量」時 | 要跑完整測試還是只跑受影響面；答「完整測試」時 |
| 測試範圍 | config.yaml rules.tasks | 全量 `npm run test:all` 只由 CI 守門 | 完整測試 `npm run test:all` 只由 CI 守門 |
| 重新載入 | remote-workspace-data | 觸發全量重載後 | 觸發全部重新載入後 |
| 重新查詢 | remote-resilience（兩處）、remote-offline 手冊頁 | store 全量重查後；全量重查並重啟 | store 全部重新查詢後；全部重新查詢並重啟 |
| 重新收斂 | phase2-acceptance（兩處）、server-event-stream | 以輪詢與查詢全量收斂；以 Query 與 ETag 全量收斂 | 以輪詢與查詢全部重新收斂；以 Query 與 ETag 全部重新收斂 |
| 重新建立 | context-projection、remote-context 手冊頁 | 以新 snapshot 全量重建投影 | 以新 snapshot 全部重新建立投影 |
| 失效通知 | remote-resilience、remote-offline 手冊頁 | 發全量失效通知 | 發出全部失效的通知 |
| 任務計數 | manual-task-marker（三處） | 全量計數；表頭「全量(完成/總數)」 | 全部任務計數；表頭「全部任務(完成/總數)」 |
| 手冊重生 | manual-skill、manual 手冊頁 | 方全量重生；才會全量重生 | 方全部重新產生；才會全部重新產生 |
| 一整套投影 | context-projection、server-context-api、remote-context 手冊頁 | SHALL 為全量；回全量投影內容；沒指定流程時為全量 | SHALL 為完整投影；回完整投影內容；沒指定流程時為完整投影 |
| 一整套呈現 | desktop-app（三處） | 還原全量呈現；表格「還原全量」；全量還原且 | 還原全部呈現；表格「還原全部」；還原全部且 |

句子其他部分不動、不重寫段落；SHALL 與 WHEN／THEN 結構不變。

替代方案：一律換成「完整」——「完整重查」「完整計數」讀不出「重新做一遍」與「全部任務」的意思，否決。

### D2 「完整測試」詞條立在 LANGUAGE.md

openspec/LANGUAGE.md 詞彙末尾新增一條：

- **definition**：repo 慣例的整套測試——本 repo 為 root `npm run test:all`，CI 守門跑同一組；對照詞「只跑受影響面的測試」（config-skill 與 openspec/config.yaml 的既有寫法）。英文維持 full test suite。
- **avoid**：全量
- **why**：「全量」不是台灣慣用詞。專案內它另有四種意思，依「修飾動作用『全部』、指一整套東西用『完整』」改寫：全部重新載入／查詢／收斂、全部任務計數、全部重新產生、完整投影。2026-10-15 討論「test-speed-and-wording」定案。

avoid 只放「全量」一個詞，不帶語境限定：五種意思都要換，所以守門要擋全部。vocabulary-guard 從 LANGUAGE.md 動態解析守門詞，掃描面目前零命中，加詞條不會打紅。

替代方案：avoid 寫成「全量（指測試範圍時）」——帶語境限定的詞守門會跳過，等於沒擋，否決。

### D3 標題改名清單

四個標題同批改名，直接改正式規格：

| 規格 | 層級 | 現行 | 改後 |
|---|---|---|---|
| delivery-baseline | Requirement | root 單一指令全量驗證 | root 單一指令完整驗證 |
| config-skill（封存後字面） | Scenario | 答全量時不寫規則 | 答完整測試時不寫規則 |
| manual-skill | Scenario | 首次全量生成 | 首次生成完整手冊 |
| review-skill | Scenario | 專案未寫測試政策時跑全量 | 專案未寫測試政策時跑完整測試 |

「首次生成完整手冊」沿用該 scenario THEN 句「產出…的完整手冊」的字面。引用舊標題的地方：delivery-baseline 內文一處引用自身標題（同檔一起改）、scripts/release/delivery-gate.test.mjs 一行註解（D7）。改名後全 repo（排除封存區與討論記錄）舊標題零命中。

直接改標題安全的前提：沒有在途 delta 以舊名 MODIFIED 這些需求。apply 開工時 config-skill-rule-routing 已封存（D4），在途變更裡沒有別人引用這四個標題；開工時以 `speclink list --json` 的 `deltaCapabilities` 再確認一次。

### D4 等 config-skill-rule-routing 封存後才開工

config-skill-rule-routing 的 delta MODIFIED 了 config-skill 含「全量」的需求。它封存時以 delta 整塊取代正式規格，若本變更先改字，「全量」會被蓋回來，Scenario 改名還會讓它的 MODIFIED 找不到需求而封存失敗。

做法：propose 時以 `speclink change depends taiwan-full-wording --on config-skill-rule-routing` 宣告前置，`speclink plan` 會把本變更排在它之後。apply 第一件事是重新盤點：確認 config-skill-rule-routing 已在封存區，再對範圍內的檔案重跑一次 grep，以實際命中為準（它的品質關卡可能改到 delta 字面）。

替代方案：等它封存後才 propose——討論原本這樣定，但 propose 只寫 artifact、不動正式規格，以前置宣告擋住 apply 即可達到同一目的，否決等待。

### D5 手冊直接改字、不重生

四頁手冊直接改內文，frontmatter（含 `generated`）一字不動。直接改正式規格不動 `@trace` 時戳，手冊過期判定不會因本變更把這幾頁標成「可能過期」，重生不會被觸發，所以手動改。

例外狀況：config-skill-rule-routing 封存時會更新 config-skill 的 `@trace`，policy-config 頁可能因此已被標過期——那是它的封存帶來的，本變更只改字、不處理過期標示，留給下次 `/speclink-manual`。

### D6 config.yaml 的規則經 workflow-config rules 寫入

openspec/config.yaml 的 rules 中 tasks 段有一條規則含「全量 `npm run test:all`」。不手改 YAML：以 `speclink workflow-config show --json` 取出 tasks 段規則，只把那一條的「全量」換成「完整測試」，經 `speclink workflow-config rules tasks --stdin --dry-run` 看 diff，確認只有那一行變動後再去掉 `--dry-run` 寫入。其餘規則逐字不變，順序不變。

### D7 只改引用改名標題的那一行註解

scripts/release/delivery-gate.test.mjs 有一行段落註解寫「delivery-baseline「root 單一指令全量驗證」」，指向 D3 改名的需求。改名後它會指向不存在的標題，所以同批改成新標題。其他程式碼註解與測試名稱裡的「全量」不動（Non-Goals）。

### D8 驗收以 grep、驗證指令與抽審三道完成

- grep：範圍內 18 個檔（13 份規格、4 頁手冊、openspec/config.yaml）「全量」零命中；四個舊標題在全 repo（排除封存區與 openspec/discussions/）零命中。
- 指令：`speclink validate --specs --strict` 綠；`node --test scripts/*.test.mjs scripts/*/*.test.mjs` 綠（含 vocabulary-guard 與 delivery-gate）；`speclink language show` 列出「完整測試」詞條。
- 結構：每份規格改前改後的 Requirement 數與 Scenario 數相同；git diff 只有字詞替換、沒有結構行增減。
- 抽審：使用者讀改動最多的句子，判斷通順與意思分流是否正確。

## Implementation Contract

- **可觀察行為**：讀者在 13 份正式規格、4 頁手冊、openspec/config.yaml 讀不到「全量」，改讀 D1 對照表的詞；`speclink language show` 多一條「完整測試」詞條；`speclink show delivery-baseline` 的需求標題為「root 單一指令完整驗證」。
- **介面與資料形狀**：CLI 旗標、`--json` 欄位、識別符、技能原稿、i18n 全部不變；需求與 scenario 的數量不變。
- **失敗模式**：漏改一處 → D8 的 grep 抓到；改壞結構 → `speclink validate --specs --strict` 報錯；詞條格式錯 → vocabulary-guard 的詞條解析測試轉紅；config-skill-rule-routing 還沒封存就開工 → D4 的開工檢查擋下，先停下回報，不改 config-skill。
- **驗收**：D8 三道全過。
- **範圍內**：D1 對照表列的字面、D2 詞條、D3 四個標題、D6 一條規則、D7 一行註解。
- **範圍外**：其餘程式碼註解與測試名稱、封存區、討論記錄、手冊重生與過期標示、config-skill-rule-routing 的檔案、英文字面。

## Risks / Trade-offs

- [config-skill-rule-routing 的品質關卡改了 delta 字面，D1 的 config-skill 列對不上] → D4 的開工重盤以實際 grep 為準；新冒出的「全量」依 D1 原則（動作用全部、一整套用完整）改。
- [直接改標題，日後有變更以舊名 MODIFIED 這些需求] → 開工時確認沒有在途 delta 碰這四個標題；之後的變更從正式規格複製新標題，自然對得上。
- [加了 avoid「全量」，日後有人在 docs 或 i18n 寫「全量」會被 vocabulary-guard 擋下] → 這正是目的；守門訊息會指向 LANGUAGE.md 的詞條。
- [回歸對照：golden 與 CLI 測試] → 技能原稿與程式碼零變動，golden 不受影響；本變更不跑 Rust 測試。
- [跨平台] → 只改 UTF-8 文字檔，無路徑或行尾差異；Windows CI 的 scripts 守門讀同一份 LANGUAGE.md。
- [改字讓句子變長或不通順] → D8 的使用者抽審；不通順的句子逐句修，不重寫段落。

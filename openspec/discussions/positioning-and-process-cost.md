---
topic: speclink 定位建議 1～4：README 定位句、官方小修正路徑、章失效重跑成本、流程成本數字（第 5 項 agent 工具層延後）
slug: positioning-and-process-cost
status: promoted
created: 2026-10-21
created_by: MomoChen <momochenisme@gmail.com>
promoted_to: readme-positioning, process-cost-metrics, no-spec-change-schema, change-schema-verb, stamp-order-rules, quality-gate-wording-in-specs, spec-dir-migration, desktop-spec-dir-migration, spec-dir-path-rewrite, self-spec-dir-migration
---

# Discussion: speclink 定位建議 1～4：README 定位句、官方小修正路徑、章失效重跑成本、流程成本數字（第 5 項 agent 工具層延後）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

起因：使用者先做了一份「SDD 是否走回瀑布式」的外部研究，再請 AI 依 speclink 的歷史規格、記憶與程式碼分析定位。分析結論是對外說法（從個人到團隊的 SDD 引擎平台）與實際使用（個人＋AI agent、小批次規格加上封存前的品質關卡）有落差，並提出五項建議；本討論處理其中四項：README 定位句、官方小修正路徑、章失效後的重跑成本、流程成本數字。第 5 項（agent 工具層）由使用者延後，等其他專案完成後再實驗，留在 agent-tool-sdk-layering。
未跑追問階段：四項都已有具體方向，直接列假設；第 3 項沒有可量目標，假設附上猜測門檻。
相關正式規格：user-documentation（README 標語逐字寫在第一條需求）、workflow-schemas、propose-skill、review-station、verify-station、change-lifecycle、quality-skill、desktop-app；目前沒有進行中的變更。
Prior discussions: cross-station-staleness, quality-gate-speed, code-review-stage, evidence-gate-false-blocks, agent-tool-sdk-layering

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-10-21)

**Focus**: 四項建議的決策地圖與初始假設
**Position**: 以 A（定位句）為上游，B（小修正路徑）與 D（流程成本數字）依 A，C（章失效）獨立；使用者未指出 A、D 有誤，B 追問 schema 路線，C 要求先量化。
- 決策地圖：A 定位句 → B 小修正路徑（B1 判準、B2 少寫哪些文件與品質關卡怎麼處理）、D 流程成本數字（D1 量什麼、D2 缺的資料要不要開始記、D3 顯示在哪）；C 章失效（C1 量頻率、C2 改不改機制）
- A 假設：README 標語與第一段改以「開發者帶 AI agent、交付可被驗收」為主軸，Remote 降為團隊延伸；與正式規格衝突——user-documentation 第一條需求逐字寫死標語與 PM／PO／RD 語意（spec.md:11），config.yaml:12、:47 同一說法
- B 假設：三層（直接改不開變更／小修正只寫 proposal＋tasks、預設跳過品質關卡／完整流程），判準是「有沒有改到正式規格承諾的行為」而非行數；技術上 design 已選用（fork.schema.yaml:3）、缺 delta 只警告（validate.rs:374-375），擋住的是 tasks 依賴 specs（fork.schema.yaml:246-247）與 propose 技能「依賴鏈上都要建」（propose.md:405）
- C 假設：先量「蓋章後失效又重蓋」的頻率，猜測門檻為近 100 個封存變更超過 10% 才改引擎；章以範圍內檔案雜湊判失效（station.rs:738-757），封存守門無略過選項（archive.rs:76-105）
- D 假設：第一步由引擎從現有檔案算行數、任務數、建立到封存天數，在已封存頁顯示專案中位數；第二步（另議）才在 .openspec.yaml 記輪數與時分，因為工單蓋章即刪、時間欄位只有日期
**Ruled out**: B 以行數或檔案數當小修正判準——「很小但改了行為」的修改會跳過 delta，正式規格與程式碼脫節
**Open**: B 是否從 schema 下手、OpenSpec 的 schema 切換怎麼做；C 實測頻率；A 的 PM／PO 是否移出目標使用者；D 顯示位置（專案整體或每個變更）

### Round 2 — interview (2026-10-21)

**Focus**: C 章失效後重新蓋章的實際頻率，是否值得改引擎
**Position**: 頻率低於 10% 門檻，不改引擎；把兩條預防規矩從個人記憶搬進正式文件與 quality 技能（待使用者確認）。
- 最近 100 個封存變更（08-27～10-16）確定重蓋 6 個（6%）；對話紀錄完整涵蓋的 09-22～10-21 只有 1／54（約 2%）；git 只看得到下界，但要超過 10% 還得再藏 5 個以上，可能性低
- 12 次重蓋的原因：main 前進或合併 6、蓋章後小修 2、其他 2、CI 修補 1、勾 [M] 碰到 tasks.md 1；每次多跑約 2.7 輪（最多各 3 輪，server-web-list-pages 約 30 分鐘）
- 比例在 09-17 前後下降（之前 5／46、之後 1／54），時間點與「先等 CI 綠再蓋章」「先 rebase 再跑品質關卡」兩條規矩寫進記憶吻合；代價轉成預防動作（一個月約 7 次先等 CI）
- 12 次重跑首輪裡有 5 次抓到新問題（engine-npm-publish、workspace-sync-entrypoints、release-assets-trim、add-change-plan-remote、server-web-list-pages）——只複驗改動部分的輕量重蓋會漏掉這些
- 兩條規矩目前只在使用者記憶（stamp-after-ci-green），docs/workflow 只在 worktree 段提到「想先看 CI 就先合併」（workflow.zh-TW.md:179），外部使用者看不到
- 量測依據：scratchpad/restamp/ 的 git_restamp.py 與 tx_scan.py；封存的失效拒絕訊息 08-11 才上線，對話紀錄期間零觸發
**Ruled out**: 輕量重蓋（只複驗失效的檔案）——頻率低、且完整首輪在 5／12 次重跑裡抓到新問題，與 cross-station-staleness 當時否決的結論一致
**Open**: B 是否從 schema 下手（OpenSpec 做法已查明）；A 的 PM／PO 是否移出目標使用者；D 顯示位置

### Round 3 — interview (2026-10-21)

**Focus**: B 小修正路徑要從哪一層下手
**Position**: 使用者選做法 ①：內建第二個 schema `quick-fix`，每個變更由 AI 判定用 spec-driven 或 quick-fix。
- 選擇規則（使用者裁定）：來自討論時依討論結論判定；不經討論直接 propose 也照樣判定；判斷不了就問使用者；使用者自己指定時照指定
- OpenSpec 查證（v1.14.3）：只內建 spec-driven（tdd 在 v1.0 前移除，PR #586）；自訂 schema 放 openspec/schemas/<name>/；專案預設寫 config.yaml 的 schema:；單一變更以 new change --schema 寫進 .openspec.yaml，建立後沒有指令可改；技能只在使用者點名時帶 --schema；官方對小修正的建議是照 spec-driven 寫 MODIFIED，純重構設 skip_specs: true
- speclink 現況：schema 查找順序與 OpenSpec 相同（schema.rs:283-334）、已有 new change --schema；propose 技能照 applyRequires 鏈決定產哪些文件（propose.md:258-320），但建立變更時從不帶 --schema（propose.md:104-120）；archive 照 specs/ 目錄合併，不看 schema
- 實際資料：284 個封存變更有 14 個沒有 delta，多是重構（cli-verb-family-modules、lifecycle-*）、用詞修正（canonical-spec-zh-wording、taiwan-full-wording）與 Windows 路徑小修；依 Why 關鍵字粗估的 52 個 bug 修正有 51 個寫了 delta——修 bug 常要補情境把正確行為釘住，所以「是 bug 修正」不等於 quick-fix，判準仍是「有沒有改到正式規格承諾的行為」
- 與正式規格衝突：workflow-schemas「單一正典載入」寫死內建只有 spec-driven，要改這條需求
**Ruled out**: ② skip_specs 變更標記——使用者要變更從建立起就以 schema 定型、由 AI 依討論判定；③ 只改技能與文件——和 tasks 依賴 specs 的 schema 矛盾，變更上也不留標記
**Open**: 要不要第三個內建 schema；quick-fix 做到一半發現改到行為時怎麼轉回 spec-driven；quick-fix 的文件組成（design 是否保留選用）與品質關卡預設；OpenSpec CLI 讀不到 quick-fix 的相容影響；A 的 PM／PO；D 顯示位置；C 待使用者確認

### Round 4 — interview (2026-10-21)

**Focus**: 除了 quick-fix，要不要再內建第三個 schema
**Position**: 現在不加第三個內建 schema（使用者未反對）；實驗型做法再出現兩次以上才重議 spike schema。
- 候選 spike（寫下要驗證的問題與成功標準、做完把結果記回討論、程式可丟）：只有 agent-tool-sdk-layering 一個實例；它會收斂到一個結果，與 feasibility-first-discuss 否決 explore 型動詞的理由（反收斂）可分開看，但需求只出現一次
- 候選「找不到原因的 bug」（類似 Kiro Bugfix）：speclink 一貫先在討論裡查原因、排修法順序（如 desktop-hover-dead-after-foreground），再開 spec-driven 變更，52 個 bug 修正都這樣完成
- 候選「設計先行」（類似 Kiro Design-first）：spec-driven 的 specs 與 design 本來就在 proposal 之後並行，desktop 91 個變更沒有卡住的紀錄
- 候選「只改文件或規格」：不改行為，quick-fix 已涵蓋
- 每多一個內建 schema 的成本：多一份內嵌資產與 golden、AI 判定從二選一變三選一、OpenSpec CLI 讀不到；OpenSpec 自己也移除過第二個內建（tdd）；有特殊需要的專案可用 schema init 自建
**Ruled out**: 現在內建 spike、bugfix 或 design-first schema——各自的需求實例不足或已被既有流程涵蓋
**Open**: quick-fix 中途改 schema 的做法；quick-fix 的文件組成與品質關卡預設；OpenSpec CLI 相容影響；A 的 PM／PO 與標語；D 顯示位置

### Round 5 — interview (2026-10-21)

**Focus**: quick-fix 變更做到一半發現改到行為時，怎麼轉回 spec-driven
**Position**: 選 a：新增動詞 `speclink change schema <name> <schema>`，雙向可切；ingest 技能發現改到正式規格承諾的行為時呼叫它，再補寫 delta。
- 和既有的 `change depends`、`change rank` 同一族，只改 .openspec.yaml 的 schema:；新動詞依 verb-contract 要在本機、remote、桌面三處同語意
- 從 spec-driven 切到 quick-fix 時，要求變更裡沒有 delta
- 目的：變更的 schema 標籤永遠和內容一致，AI 選錯 schema 的代價變小
**Ruled out**: b 不換 schema、直接在 quick-fix 裡補 delta——看板標籤與內容不符，verify 也不會照 delta 檢查；c 廢棄後重開 spec-driven——已勾任務與 evidence 要重來
**Open**: quick-fix 的文件組成（design 選用）與品質關卡預設；OpenSpec CLI 相容影響；A 的 PM／PO 與標語；D 顯示位置

### Round 6 — interview (2026-10-21)

**Focus**: A README 定位句與 PM／PO 的位置
**Position**: 使用者同意：標語與第一段改以「AI 寫的變更可被驗收」為主軸，PM／PO 從第一段移到團隊延伸段，不刪除。
- 標語：「讓 AI 寫的每個變更，都對得上規格、查得到證據」；英文 "Every AI-built change, checked against its spec"（文字在 propose 時可再修）
- 第一段：每個變更先寫一小份規格、交給 AI agent 實作，封存前由審查與驗證兩道品質關卡檢查並留下證據；規格是 repo 裡的純文字、相容 OpenSpec
- 第二段：一個人用 Local Repo 就夠；團隊要共用規格再接 Remote Store，PM、PO 在這一層參與討論與提案
- config.yaml 提案規則的目標使用者改為「主要是透過 AI agent 開發的開發者；團隊情境下包含 PO 與 PM」，專案說明同步
- 依據：實際使用者只有開發者；PM 到 RD 交接關卡在路線圖寫「還要重新決定是否需要」（roadmap.zh-TW.md:69）；桌面定位為純檢視器
- 要改正式規格 user-documentation 第一條需求（標語逐字寫在 spec.md:11）
**Ruled out**: 從目標使用者整個刪掉 PM／PO——團隊與 Remote 情境仍需要他們，只是不放第一段
**Open**: quick-fix 的文件組成與品質關卡預設；OpenSpec CLI 相容影響；D 顯示位置

### Round 7 — interview (2026-10-21)

**Focus**: quick-fix 的文件組成與品質關卡預設
**Position**: 使用者同意：proposal → design（選用）→ tasks、沒有 specs；品質關卡不綁 schema，照現行規則依風險決定；verify 在這類變更上改查「沒有改到正式規格承諾的行為」。
- 資料：14 個沒寫 delta 的封存變更有 10 個寫了 design、9 個兩道品質關卡都跑（多為重構）；兩道都沒跑的 5 個是 3 個任務的 Windows 路徑小修、spec-purpose-backfill、taiwan-full-wording 等——「不改行為」不等於「小」
- proposal 的 Why 必須寫「為什麼這個變更不改行為」，作為選此 schema 的依據
- verify 發現改到行為＝用 `change schema` 切回 spec-driven 的觸發點（接上一輪的切換動詞）
**Ruled out**: quick-fix 預設跳過品質關卡（第 1 輪假設）——重構類變更多半需要審查與驗證，資料不支持
**Open**: 第二個 schema 的名稱（quick-fix 字面與實際族群不符）；OpenSpec CLI 相容影響；D 顯示位置

### Round 8 — interview (2026-10-21)

**Focus**: 第二個內建 schema 的名稱
**Position**: 改名為 `no-spec-change`，中文顯示名「不改規格」，寫進 LANGUAGE.md。
- quick-fix 的「fix」會讓 AI 把修 bug 選成它，但 52 個修 bug 的變更有 51 個要寫 delta；「quick」也和實際族群不符（多為寫了 design、跑兩道品質關卡的重構）
- 名稱字面即判準：「這個變更會不會改到正式規格？」不會就選它
**Ruled out**: quick-fix——字面誘導誤判；refactor——涵蓋不了用詞修正與平台小修；skip-specs——「略過」會被讀成可以不寫文件
**Open**: OpenSpec CLI 讀不到 no-spec-change 的相容影響；D 顯示位置

### Round 9 — interview (2026-10-21)

**Focus**: OpenSpec 相容性的方向
**Position**: 改為單向相容：支援 OpenSpec → speclink（OpenSpec 專案讀得進來），不再保證 speclink → OpenSpec（OpenSpec CLI 讀得回去）。
- no-spec-change 的 OpenSpec CLI 相容題因此消失，文件不必寫 `speclink schema fork` 繞道
- user-documentation「使用者文件載明本地產物的 OpenSpec 結構相容性」要改寫：保留純 Markdown／YAML、不裝 Speclink 也讀得懂、每次變動都看得到 Git diff；「沿用 OpenSpec 目錄結構」改成「可從 OpenSpec 專案匯入」
- README 致謝段的 OpenSpec 保留，起源事實不變
**Ruled out**: 文件註明先跑 `speclink schema fork no-spec-change` — 單向後不再承諾 OpenSpec CLI 讀得回去；init／update 自動複製 schema 到每個 repo — 同上理由，且每個 repo 多一份會隨引擎升版過期的複本
**Open**: 規格資料夾改名為 speclink/；D 顯示位置

### Round 10 — interview (2026-10-21)

**Focus**: 規格資料夾從 openspec/ 改名為 speclink/
**Position**: 方向定案：這一版之後改名，並支援既有專案遷移；OpenSpec 專案匯入走同一條遷移路徑。
- 引擎已有 `.speclink.yaml` 的 `spec_dir` 欄位（workspace/config.rs:62），預設值 `openspec`（workspace/workspace.rs:37），核心工作是改預設值與舊目錄偵測，不是新功能
- desktop 與 host 有多處寫死 `"openspec"`、不讀 spec_dir（例：apps/desktop/core/src/project.rs:128 的 init），改名時一併修正
- 波及面：39 份正式規格共 326 處 `openspec/` 字面、20 份 docs、7 份技能 asset、desktop／ui 前端約 100 處（多為測試）
- 遷移有前例：`.speclink.remote.yaml` 廢止時，discover 仍認得舊檔，CLI 每次印一行固定警告（speclink-cli common.rs:16）
- 單向相容（Round 9）讓「OpenSpec 專案 → speclink」與「舊版 speclink → 新版」成為同一件事：把 openspec/ 搬到 speclink/
**Ruled out**: 無
**Open**: 改名細節是否另開討論（遷移觸發方式、各變更 `.openspec.yaml` 是否改名、遷移時有進行中的 worktree、正式規格字面改寫法、local→remote 遷移的備份目錄名）；D 顯示位置

### Round 11 — interview (2026-10-21)

**Focus**: 改名細節的去處，與既有 speclink 專案在 update 時自動改名
**Position**: 改名細節另開討論；使用者要 `speclink update` 自動偵測舊資料夾並改名，本輪只記下這個起點與初步條件，由新討論定案。
- 偵測條件：`.speclink.yaml` 沒有啟用的 `spec_dir` 行、有 `openspec/`、沒有 `speclink/`；init 只在非預設時寫入 spec_dir（workspace/init.rs:145），所以這個組合就是舊版預設佈局
- 不動的情況：`.speclink.yaml` 明寫 spec_dir（含 `openspec`）視為使用者自選；遠端模式沒有本機規格資料夾；沒有 `.speclink.yaml` 的純 OpenSpec 專案走 init 匯入，不走 update
- 改名時同時寫入 `spec_dir: speclink`：spec_dir 從第一個 commit（5d7fa5c8，2026-07-01）就有，隊友的舊版 CLI 也找得到新資料夾
- 初步的暫緩條件：有進行中的 worktree（分支裡仍是 openspec/）、openspec/ 有未提交的修改；暫緩時比照 `.speclink.remote.yaml` 前例，每次印一行警告與要跑的指令
- update 改不到使用者自己的腳本、CI 設定、.gitignore 裡的 openspec/ 路徑，只能在改名後提醒
**Ruled out**: 無
**Open**: D 顯示位置

### Round 12 — interview (2026-10-21)

**Focus**: D 流程成本數字的顯示位置與第二步時程
**Position**: 數字放在已封存頁最上方、顯示專案整體的中位數；第二步（記錄輪數與時分）依使用者要求與第一步一起做。
- 不放每個變更的數字磚：數字磚的定義是文件本身的計數、不是指標儀表（LANGUAGE.md:269-273），且提案頁 6 種候選只取 4 格已滿（changeDocSummary.tsx:95-120）
- 第一步從現有檔案算：文件行數（中位數與 P90）、任務數中位數、建立到封存天數中位數
- 第二步新增欄位：蓋章時把工單輪數寫進 `.openspec.yaml`（`reviewed_rounds`／`verified_rounds`；工單蓋章即刪，station.rs:643-657 是唯一寫入點），封存時寫 `archived_at`
- 時間欄位改寫成帶時區偏移量的 RFC 3339 時戳，讀取端同時接受純日期；前例是 manual-stale-time-granularity（2026-09-07），現有記錄一律不回改
- 讀取端要改：preflight.rs:39、:95 目前只接受 `%Y-%m-%d`
- 舊的 284 筆只有日期、沒有輪數；時數與輪數只從有新欄位的變更算，畫面標出樣本數
**Ruled out**: 每個變更的數字磚 — 與用詞表定義衝突且格子已滿；第二步延後 — 使用者要一起做，且工單蓋章即刪，晚一天記就少一天資料
**Open**: 分刀計畫與結論

### Round 13 — interview (2026-10-21)

**Focus**: 分刀計畫的調整，與資料夾遷移改回本討論處理
**Position**: 使用者同意三項調整，並改變第 11 輪的決定：遷移細節不另開討論，直接在本討論定案，成為最後一刀、下一版才上線。
- 第 1 刀的 README 不寫「可從 OpenSpec 匯入」：現在沒有乾淨的匯入路徑——在 OpenSpec 專案跑 `speclink init` 會停在 Already initialized（workspace/init.rs:90-91），加 `--force` 會把 openspec/config.yaml 蓋回範本（init.rs:156），專案說明與規則遺失；README 先只保留純文字承諾，匯入說明由遷移那一刀寫
- 新增第 6 刀：8 份正式規格約 27 處仍寫「品質站」（含 tray-status-menu、user-documentation 各一條需求標題），用詞守門只掃畫面文案所以沒抓到；以 MODIFIED delta 改成「品質關卡」，標題改名依規則用 RENAMED 或 REMOVED＋ADDED
- 遷移的決策樹：M1 範圍（只改資料夾名，或連各變更的 `.openspec.yaml` 一起改）→ M2 update 自動偵測的偵測與暫緩條件（第 11 輪起點）→ M3 OpenSpec 專案的匯入路徑 → M4 正式規格 326 處 `openspec/` 字面的寫法 → M5 周邊（local→remote 備份目錄名、遠端投影 `.speclink/context/` 內的鏡像、寫死 `"openspec"` 不讀 spec_dir 的程式）
**Ruled out**: 遷移另開討論（第 11 輪）— 使用者要在本討論一次定案
**Open**: M1 範圍；M2 偵測與暫緩條件；M3 匯入路徑；M4 規格字面寫法；M5 周邊

### Round 14 — interview (2026-10-21)

**Focus**: M1 遷移範圍——各變更的 `.openspec.yaml` 是否一起改名
**Position**: 使用者決定一起改名為 `.meta.yaml`；update 遷移時，本機所有變更（進行中與已封存）的檔案一次改名，檔案內容不改寫。
- 不用 `.speclink.yaml`：discover 往上找到第一個有它的資料夾就當專案根（workspace/workspace.rs:33-34），放進變更資料夾會認錯根目錄
- `.meta.yaml` 與程式名稱 ChangeMeta、DocumentId::ChangeMeta 一致；與 `.evidence.json` 一樣以內容命名、不帶品牌
- 讀取順序沿用 speclink-fs layout.rs `legacy_change_evidence` 前例：先讀 `.meta.yaml`，沒有才讀 `.openspec.yaml`；只寫新檔名，寫入時移除舊檔
- 遠端資料庫不搬：已封存變更的鍵維持 `.openspec.yaml`，靠讀取順序讀到；新封存寫 `.meta.yaml`；三種儲存驅動都還沒有資料遷移路徑（store-sqlite schema.rs:12-15，SCHEMA_VERSION 為 1）
- 舊檔內容不改寫：時間欄位維持純日期、不補輪數（第 12 輪：現有記錄一律不回改）
- 讀取順序也讓 OpenSpec 專案的變更、遷移前開的分支不經轉換就讀得到
- 檔名集中在一處定義（目前 25 個 Rust 檔共 90 行、前端 6 處）
**Ruled out**: 保留 `.openspec.yaml` — 使用者要名稱與品牌一致；改名為 `.speclink.yaml` — 與專案根偵測衝突；遠端資料庫一起遷移 — 要在三種儲存驅動各寫第一套遷移程式，只為換一個使用者看不到的鍵名
**Open**: M2 偵測與暫緩條件（含隊友仍用舊版 CLI：M1 之後，第 11 輪「寫入 spec_dir 讓舊版 CLI 找得到」不再成立）；M3 匯入路徑；M4 規格字面寫法；M5 周邊

### Round 15 — interview (2026-10-21)

**Focus**: M2 隊友仍用舊版 CLI 時的處理
**Position**: 遷移時不寫入 `spec_dir: speclink`，讓舊版 CLI 明顯出錯，再加兩道提醒；第 11 輪「寫入 spec_dir 讓舊版 CLI 找得到」作廢。
- 不寫 spec_dir：舊版 CLI 照預設找 `openspec/`，找不到就是空專案（speclink-fs lib.rs:115 讀不到資料夾回空清單），隊友立刻發現；代價是隊友若此時建變更，舊版會重建 `openspec/`
- 提醒一：新版發現 `speclink/` 與 `openspec/` 並存時，每次印一行警告，說明 `openspec/` 可能是舊版 CLI 建的、升級後搬過來；比照 `.speclink.remote.yaml` 前例
- 提醒二：update 遷移完成後提示「團隊成員先升級 speclink，再 pull 這個 commit」
**Ruled out**: 寫入 `spec_dir: speclink` — M1 改名後舊版 CLI 讀不到 `.meta.yaml`，章、schema、相依全看成空值（lib.rs:40），且舊版寫入的 `.openspec.yaml` 會被新版忽略，資料默默分岔
**Open**: M2 偵測、不動與暫緩條件的定案；M3 匯入路徑；M4 規格字面寫法；M5 周邊

### Round 16 — interview (2026-10-21)

**Focus**: M2 遷移的偵測、不動與暫緩條件
**Position**: 使用者同意五點：偵測與不動條件沿用第 11 輪，暫緩條件定案，新增「暫緩期間照常可用」。
- 偵測：`.speclink.yaml` 沒有啟用的 spec_dir、有 `openspec/`、沒有 `speclink/`
- 不動：spec_dir 明寫任何值（含 `openspec`）、遠端模式、沒有 `.speclink.yaml` 的純 OpenSpec 專案（交給 M3）
- 暫緩：`git worktree list` 有主目錄以外的 worktree（host worktree.rs:41 已有讀取程式）；`openspec/` 有未提交修改；在 linked worktree 裡跑 update 一律不遷移，只在主目錄搬一次
- 暫緩期間照常可用：沒寫 spec_dir、`speclink/` 不存在但 `openspec/` 存在時，新版照舊讀 `openspec/`，每次印一行警告與遷移條件；否則預設改成 `speclink/` 後，暫緩中的專案會變成空專案
- 搬法：檔案系統直接改名資料夾與各 `.openspec.yaml`，內容不變讓 git 認成改名；update 不 commit；完成後提示要 commit 的檔案、隊友先升級再 pull、自己的腳本／CI／.gitignore 裡的 `openspec/` 要自己改
**Ruled out**: 無
**Open**: M3 匯入路徑；M4 規格字面寫法；M5 周邊

### Round 17 — interview (2026-10-21)

**Focus**: M3 OpenSpec 專案的匯入路徑
**Position**: 使用者同意：init 遇到「有 `openspec/`、沒有 `.speclink.yaml`」即視為匯入，自動完成，不另外詢問。
- 搬移：與 update 遷移共用同一段程式，`openspec/` 改為 `speclink/`、各 `.openspec.yaml` 改為 `.meta.yaml`
- 保留 OpenSpec 原有內容：config.yaml（專案說明與規則）、schemas/、正式規格、進行中與已封存的變更；不再用範本覆蓋（現況 init.rs:156 的 `--force` 會蓋掉）
- 補上 speclink 檔案：`.speclink.yaml`、`.gitignore` 設定、所選工具的技能檔
- 擋下條件同 M2（其他 worktree、`openspec/` 有未提交修改）；init 是主動指令，所以直接拒絕並說明，不走暫緩
- 完成後提示：要 commit 的檔案；OpenSpec CLI 從此讀不到（第 9 輪單向相容）；OpenSpec 自己安裝的技能與指令檔不動，由使用者自行刪除
- 不先詢問的理由：執行 init 即表示要開始用 speclink，且改動未 commit、可用 git 復原
**Ruled out**: 匯入前先詢問確認 — init 本身就是意圖，改動可用 git 復原
**Open**: 桌面 app 更新時的遷移機制（使用者追加）；M4 規格字面寫法；M5 周邊

### Round 18 — interview (2026-10-21)

**Focus**: 桌面 app 更新時的遷移機制
**Position**: 使用者同意八點：遷移跟著既有的「技能檔過期」提示走，不另做流程。
- 依據：開專案時探測技能檔（desktop store.ts:793），過期就出提示卡與對話框，按更新呼叫與 CLI 同一個引擎 update（desktop core project.rs:154）；略過只記這一版（assetPrompt.ts:57）；app 升版後引擎版號改變，每個專案下次開啟都會出提示
- 探測多回報遷移狀態（無、待遷移、暫緩與原因）；待遷移時對話框寫明資料夾改名、N 個 `.openspec.yaml` 改名、不會 commit
- 暫緩時列出原因（例：其他 worktree 清單），按更新只更新技能檔，與 CLI 的 M2 規則相同
- 完成後顯示與 CLI 相同的提醒；略過照舊，專案照常讀 `openspec/`，下一版再提示
- 遷移後重新解析規格資料夾並重啟監看（src-tauri watch.rs），否則畫面停止自動更新
- app 自己升版時不在背景遷移任何專案，只在使用者開啟該專案並按更新時搬
- 「加入專案」遇到 OpenSpec 專案走 M3 匯入，對話框多一句資料夾會改名
- 桌面寫死 `"openspec"` 的地方改讀專案實際的規格資料夾（例：project.rs:128 的 init）
**Ruled out**: app 升版時在背景一次遷移所有專案 — 其他專案可能有未提交工作或 worktree，背景改名讓使用者措手不及；另做一套遷移流程 — 既有技能檔提示每次升版都會出現，足以承載
**Open**: M4 規格字面寫法；M5 周邊

### Round 19 — interview (2026-10-21)

**Focus**: M4 正式規格裡舊名稱的寫法
**Position**: 使用者同意：直接寫新路徑（`speclink/…`、`.meta.yaml`），描述舊格式的需求與歷史記錄不改，路徑改寫另拆一刀。
- 範圍：40 份規格 146 條需求寫到 `openspec/` 或 `.openspec.yaml`（後者 77 次、13 份）；標題只有兩處——user-documentation 相容性需求（第 1 刀整條改寫）與 change-plan 情境「缺 .openspec.yaml 的 change 零寫入拒絕」（改名要宣告 REMOVED-SCENARIO）
- 保留舊名稱：描述讀取順序、偵測 `openspec/`、新舊資料夾並存警告等舊格式本身的需求
- 不改：已封存的變更與討論，記的是當時的事實
- 分刀：遷移程式一刀，只帶行為真正改變的需求；其餘需求的路徑改寫緊接著另一刀，同一版發布
**Ruled out**: 規格資料夾相對路徑（`changes/<name>/…`＋用詞表定義）— 讀者要先知道它不在 repo 根目錄，且與 `.claude/`、`.speclink/` 等根目錄路徑混雜；路徑改寫與遷移程式同一刀 — 146 條需求讓審查與驗證過重
**Open**: M5 周邊

### Round 20 — interview (2026-10-21)

**Focus**: M5 遷移的周邊項目
**Position**: 使用者同意六點，遷移部分全部定案。
- 本機轉遠端的備份資料夾改用實際資料夾名稱 `<資料夾名>.migrated-<日期>`（現寫死於 desktop src-tauri remote.rs:762）
- 遠端 Context Projection 路徑改為 `speclink/…`（server api/context.rs:28、:147-156），兩種模式看到同一名稱；投影可隨時重建；新舊 client／server 混用的影響於 propose 時查證
- 寫死 `"openspec"` 的程式全部改讀專案實際資料夾，並加一組非預設 spec_dir 的測試抓漏改
- 7 份技能檔隨遷移程式一刀修改；技能不得假設資料夾一定叫 `speclink/`（暫緩中的專案仍是 `openspec/`），寫「規格資料夾（預設 `speclink/`）」或取指令輸出的路徑
- 文件：遷移與匯入說明隨遷移程式；README、20 份 docs、LANGUAGE.md 的路徑隨路徑改寫那一刀
- speclink repo 自身的遷移另開一刀，在路徑改寫封存後於主目錄跑 update，同一變更修 scripts/docs/vocabulary-guard.test.mjs:80、scripts/docs/docs-screenshots.mjs、scripts/release/delivery-gate.test.mjs 與 .github/PULL_REQUEST_TEMPLATE.md；不改承諾的行為，用 no-spec-change，完成後才發版
**Ruled out**: 備份名稱改寫死 `speclink.migrated-<日期>` — 未遷移專案轉遠端時名稱會與實際資料夾不符；遠端投影維持 `openspec/` — AI 在兩種模式看到不同名稱
**Open**: 分刀計畫與結論

## Conclusion

**Decision**: speclink 定位收斂為「讓 AI 寫的每個變更都對得上規格、查得到證據」，分 9 刀落地：第 1～6 刀先發一版；第 7～9 刀把規格資料夾從 `openspec/` 遷到 `speclink/`，下一版發布。
- 順序：第 1、2 刀最先做（第 2 刀越早上線、累積的資料越多）；第 3、4、5、7 刀都改技能檔、會升 ASSET_VERSION，依序做、不平行；第 6 刀沒有相依；第 1～6 刀完成後發版；第 7、8、9 刀依序做，第 9 刀完成後再發版
- **cut 1 `readme-positioning`**: README 標語、前兩段與相容說明改寫，PM／PO 移到團隊延伸段
  - 標語：「讓 AI 寫的每個變更，都對得上規格、查得到證據」；英文 "Every AI-built change, checked against its spec"（文字在 propose 時可再修）
  - 第一段：每個變更先寫一小份規格、交給 AI agent 實作，封存前由審查與驗證兩道品質關卡檢查並留下證據；規格是 repo 裡的純文字
  - 第二段：一個人用 Local Repo 就夠；團隊要共用規格再接 Remote Store，PM、PO 在這一層參與討論與提案
  - 相容說明改為單向：保留純 Markdown／YAML、不裝 Speclink 也讀得懂、每次變動看得到 Git diff；拿掉「相容 OpenSpec」，也不寫「可從 OpenSpec 匯入」（現在 init 遇到 `openspec/` 會停在 Already initialized，`--force` 會把 config.yaml 蓋回範本），匯入說明由 cut 7 寫；致謝段的 OpenSpec 保留
  - README 中英兩版與 getting-started 中英兩版同步
  - config.yaml 的專案說明與提案規則，目標使用者改為「主要是透過 AI agent 開發的開發者；團隊情境下包含 PO 與 PM」
  - 正式規格：user-documentation 第一條需求（標語逐字寫在 spec.md:11）與「使用者文件載明本地產物的 OpenSpec 結構相容性」改寫；後者標題含 OpenSpec，改名依規則宣告
- **cut 2 `process-cost-metrics`**: 已封存頁最上方顯示專案整體的流程成本中位數，並開始記錄品質關卡輪數與時分
  - 從現有檔案算：文件行數（中位數與 P90）、任務數中位數、建立到封存天數中位數
  - 新欄位：蓋章時把工單輪數寫進變更資料檔（`reviewed_rounds`／`verified_rounds`），寫入點是蓋章刪工單的同一步（station.rs:643-657）；封存時寫 `archived_at`
  - 時間欄位改寫成帶時區偏移量的 RFC 3339 時戳，讀取端同時接受純日期（preflight.rs:39、:95 目前只接受 `%Y-%m-%d`）；前例是 manual-stale-time-granularity；現有記錄一律不回改
  - 時數與輪數只從有新欄位的變更算，畫面標出樣本數
  - 不放進每個變更的數字磚
- **cut 3 `no-spec-change-schema`**: 內建第二個 schema `no-spec-change`（中文「不改規格」），給不改正式規格承諾行為的變更
  - 文件組成：proposal → design（選用）→ tasks，沒有 specs
  - proposal 的 Why 必須寫「為什麼這個變更不改行為」，作為選此 schema 的依據
  - 選擇規則：來自討論時依討論結論判定；不經討論直接 propose 也照樣判定；判斷不了就問使用者；使用者指定時照指定；建立變更時帶 `--schema`
  - 判準是「這個變更會不會改到正式規格？」，不會才選它；修 bug 多半要寫 delta 釘住正確行為（52 個有 51 個），不等於 no-spec-change
  - 品質關卡不綁 schema，照現行規則依風險決定；verify 在這類變更上檢查「沒有改到正式規格承諾的行為」
  - LANGUAGE.md 新增「不改規格」詞條
  - 正式規格：workflow-schemas「單一正典載入」寫死內建只有 spec-driven，要改這條需求
- **cut 4 `change-schema-verb`**: 新增 `speclink change schema <name> <schema>`，雙向切換變更的 schema（依 cut 3）
  - 與 `change depends`、`change rank` 同一族，只改變更資料檔的 `schema:`；依 verb-contract 在本機、remote、桌面三處同語意
  - 切到 no-spec-change 時，要求變更裡沒有 delta
  - ingest 技能發現改到正式規格承諾的行為時呼叫它，再補寫 delta；verify 發現改到行為時，也以它切回 spec-driven
- **cut 5 `stamp-order-rules`**: 把兩條預防章失效的規矩寫進 workflow 文件與 quality 技能，不改引擎
  - 「先等 CI 綠再蓋章」：章凍結內容指紋，蓋章後再改一行就要整套重跑兩道品質關卡
  - 「先 rebase 再跑品質關卡」：main 前進改到範圍內的檔，零衝突合併也會讓章過期
  - 依據：最近 100 個封存變更重蓋 6%、近一個月約 2%，比例在這兩條規矩開始使用後下降；兩條規矩目前只在個人記憶，外部使用者看不到
- **cut 6 `quality-gate-wording-in-specs`**: 8 份正式規格約 27 處「品質站」改為「品質關卡」
  - 範圍：config-skill、server-verb-api、review-skill、tray-status-menu、change-diff-scope、user-documentation、skill-routing、client-protocol；ui-copy-vocabulary 把它列為禁用詞的出現處保留
  - 以 MODIFIED delta 改寫；tray-status-menu「面板變更列的品質站章」與 user-documentation「品質站蓋章效果與非成員錯誤碼的文件揭露」兩個標題要改名——只改標題用 RENAMED，連內文一起改用 REMOVED＋ADDED
- **cut 7 `spec-dir-migration`**: 規格資料夾預設改為 `speclink/`、變更資料檔改名為 `.meta.yaml`，提供 update 自動遷移、init 匯入 OpenSpec 專案與桌面提示（下一版）
  - 變更資料檔：`.openspec.yaml` 改名為 `.meta.yaml`（不用 `.speclink.yaml`，discover 往上找到它就當專案根，會認錯根目錄）；讀取先找 `.meta.yaml`、沒有才讀 `.openspec.yaml`，只寫新檔名、寫入時移除舊檔（前例 speclink-fs layout.rs 的 `legacy_change_evidence`）；檔名集中一處定義
  - 遠端資料庫不搬：已封存變更的鍵維持 `.openspec.yaml`，靠讀取順序讀到；新封存寫 `.meta.yaml`
  - update 遷移的偵測：`.speclink.yaml` 沒有啟用的 spec_dir、有 `openspec/`、沒有 `speclink/`
  - 不動：spec_dir 明寫任何值（含 `openspec`）、遠端模式、沒有 `.speclink.yaml` 的純 OpenSpec 專案（走 init 匯入）
  - 暫緩：有主目錄以外的 worktree（`git worktree list`）、`openspec/` 有未提交修改；在 linked worktree 裡跑 update 一律不遷移，只在主目錄搬一次
  - 暫緩期間照常可用：沒寫 spec_dir、只有 `openspec/` 時新版照舊讀它，每次印一行警告與遷移條件
  - 搬法：檔案系統直接改名資料夾，以及所有變更（進行中與已封存）的 `.openspec.yaml`，內容不改寫，讓 git 認成改名；update 不 commit；完成後提示要 commit 的檔案、團隊成員先升級再 pull、自己的腳本／CI／.gitignore 裡的 `openspec/` 要自己改
  - 舊版 CLI：遷移時不寫入 `spec_dir: speclink`，讓舊版 CLI 看到空專案而明顯出錯；新版發現 `speclink/` 與 `openspec/` 並存時，每次印一行警告，說明 `openspec/` 可能是舊版 CLI 建的
  - init 匯入：遇到有 `openspec/`、沒有 `.speclink.yaml` 時自動匯入，與 update 共用搬移程式；保留 config.yaml、schemas/、正式規格與變更，不用範本覆蓋；補上 `.speclink.yaml`、`.gitignore` 設定與所選工具的技能檔；擋下條件同暫緩，但直接拒絕並說明；完成後提示要 commit 的檔案、OpenSpec CLI 從此讀不到、OpenSpec 自己安裝的技能與指令檔由使用者自行刪除
  - 桌面：遷移跟著既有的技能檔過期提示走；探測多回報遷移狀態（無、待遷移、暫緩與原因）；對話框寫明資料夾改名、N 個檔案改名、不會 commit；暫緩時列出原因，按更新只更新技能檔；完成後顯示與 CLI 相同的提醒；略過照舊；遷移後重新解析規格資料夾並重啟監看；app 升版時不在背景遷移任何專案；「加入專案」遇到 OpenSpec 專案走匯入，對話框說明資料夾會改名
  - 周邊：本機轉遠端的備份資料夾改用 `<資料夾名>.migrated-<日期>`；遠端 Context Projection 路徑改為 `speclink/…`，新舊 client／server 混用的影響在 propose 時查證；寫死 `"openspec"` 的程式全部改讀實際資料夾，並加一組非預設 spec_dir 的測試抓漏改
  - 7 份技能檔隨本刀修改，不得假設資料夾一定叫 `speclink/`（暫緩中的專案仍是 `openspec/`），寫「規格資料夾（預設 `speclink/`）」或取指令輸出的路徑；遷移與匯入的使用說明隨本刀寫入文件
  - 本刀的 delta 只帶行為真正改變的需求（偵測、讀取順序、匯入、桌面提示等）
- **cut 8 `spec-dir-path-rewrite`**: 其餘正式規格與文件裡的舊路徑改為新路徑（依 cut 7，同一版發布）
  - 範圍：40 份規格共 146 條需求寫到 `openspec/` 或 `.openspec.yaml`（扣掉 cut 7 已改的）；README、20 份 docs、LANGUAGE.md 的路徑
  - 寫法：直接寫新路徑 `speclink/…` 與 `.meta.yaml`；描述舊格式本身的需求（讀取順序、偵測 `openspec/`、並存警告）保留舊名稱
  - change-plan 情境「缺 .openspec.yaml 的 change 零寫入拒絕」改名，要宣告 REMOVED-SCENARIO
  - 已封存的變更與討論不改，它們記的是當時的事實
- **cut 9 `self-spec-dir-migration`**: speclink repo 自身遷移（依 cut 8；用 no-spec-change）
  - cut 8 封存後，先 commit 本刀的文件，再在主目錄（沒有其他 worktree、沒有未提交修改）跑 update 遷移
  - 同一變更修好遷移後會壞的 scripts/docs/vocabulary-guard.test.mjs（:80 寫死讀 `openspec/LANGUAGE.md`）、scripts/docs/docs-screenshots.mjs、scripts/release/delivery-gate.test.mjs 與 .github/PULL_REQUEST_TEMPLATE.md
  - 完成後才發版
**Rationale**: 外界說 SDD 像瀑布式，核心是批次大小與流程成本；speclink 的資料顯示實際用法是個人加 AI agent 的小批次規格，再加封存前的驗收，所以定位往「可驗收」收斂，成本只在資料顯示有浪費的地方砍——不改行為的變更不必寫 delta（no-spec-change）、章失效靠預防規矩而不是放寬引擎（重蓋比例低，且完整首輪在 5／12 次重跑抓到新問題）、用數字讓流程成本看得見；品牌獨立改為單向相容，加上一套不會讓未遷移專案壞掉的遷移。
**Rejected alternatives**:
- 以行數或檔案數當小修正判準 — 很小但改了行為的修改會跳過 delta，正式規格與程式碼脫節
- 輕量重蓋（只複驗失效的檔案） — 重蓋頻率低，且完整首輪在 5／12 次重跑抓到新問題
- skip_specs 變更標記 — 使用者要變更從建立起就以 schema 定型、由 AI 依討論判定
- 只改技能與文件、不加 schema — 與 tasks 依賴 specs 的 schema 矛盾，變更上也不留標記
- 現在內建 spike、bugfix 或 design-first schema — 需求實例不足，或已被既有流程涵蓋
- 不換 schema、直接在 no-spec-change 裡補 delta — 看板標籤與內容不符，verify 也不會照 delta 檢查
- 發現改到行為就廢棄、重開 spec-driven — 已勾任務與 evidence 要重來
- 從目標使用者整個刪掉 PM／PO — 團隊與 Remote 情境仍需要他們，只是不放第一段
- no-spec-change 預設跳過品質關卡 — 14 個沒寫 delta 的封存變更有 9 個兩道都跑，多為需要審查的重構
- 第二個 schema 叫 quick-fix — 「fix」會讓 AI 把修 bug 選成它，「quick」也與實際族群不符
- 第二個 schema 叫 refactor — 涵蓋不了用詞修正與平台小修
- 第二個 schema 叫 skip-specs — 「略過」會被讀成可以不寫文件
- 文件註明先跑 `speclink schema fork no-spec-change` 給 OpenSpec CLI 用 — 改為單向相容，不再承諾 OpenSpec CLI 讀得回去
- init／update 自動把 schema 複製到每個 repo — 同上，且每個 repo 多一份會隨引擎升版過期的複本
- 流程成本數字放進每個變更的數字磚 — 數字磚定義為文件本身的計數、不是指標儀表，且提案頁格子已滿
- 流程成本第二步延後 — 工單蓋章即刪，晚一天記就少一天資料
- 遷移細節另開討論 — 使用者要在本討論一次定案
- 保留 `.openspec.yaml` 檔名 — 使用者要名稱與品牌一致
- 變更資料檔改名為 `.speclink.yaml` — 與專案根偵測衝突
- 遠端資料庫一起遷移 — 要在三種儲存驅動各寫第一套遷移程式，只為換一個使用者看不到的鍵名
- 遷移時寫入 `spec_dir: speclink` 讓舊版 CLI 找得到 — 舊版讀不到 `.meta.yaml`，章、schema、相依全看成空值，舊版寫入的檔案又被新版忽略，資料默默分岔
- init 匯入前先詢問確認 — 執行 init 就是要開始用 speclink，改動未 commit、可用 git 復原
- 桌面 app 升版時在背景一次遷移所有專案 — 其他專案可能有未提交工作或 worktree
- 另做一套桌面遷移流程 — 既有技能檔過期提示每次升版都會出現，足以承載
- 正式規格改用規格資料夾的相對路徑 — 讀者要先知道它不在 repo 根目錄，且與 `.claude/`、`.speclink/` 等根目錄路徑混雜
- 路徑改寫與遷移程式放同一刀 — 146 條需求讓審查與驗證過重
- 備份資料夾寫死 `speclink.migrated-<日期>` — 未遷移專案轉遠端時，名稱與實際資料夾不符
- 遠端 Context Projection 維持 `openspec/` — AI 在兩種模式看到不同名稱
**Deferred**:
- 第 5 項 agent 工具層（agent-tool-sdk-layering） — 使用者要等其他專案完成後再實驗
- 第三個內建 schema（spike） — 實驗型做法只出現一次，再出現兩次以上才重議
- 章失效的引擎改動（例如只複驗失效檔案） — 重蓋比例低於 10%；回到 10% 以上再議
**Capture to**: proposal | design | spec | tasks | LANGUAGE.md（新增「不改規格」詞條；用詞漂移：「品質站」是 avoid 詞，本次對話、使用者記憶檔（已修正）與 8 份正式規格都用過，正式規格由 cut 6 修正）
**Next**: /speclink-propose --from-discussion positioning-and-process-cost（從 cut 1 開始；cut 1、cut 2 可先做）

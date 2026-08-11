## Context

現況(已探查):「任務全勾」是三道守門的共同條件——review 技能的開跑自檢、verify 引擎的落工單守門、兩站引擎的蓋章守門;封存守門檢查工單未結與任務完成度,不檢查章失效。freshness 失效判定與章錨欄位(reviewed_scope／verified_scope、reviewed_tasks_total／verified_tasks_total)已實作,正式碼消費者為 desktop 變更清單的審查/驗證徽章(apps/desktop/core/src/query.rs);`[P]` 平行標記已解析並隨 payload 上線,零消費者。desktop 任務分頁讀 tasks.md 原文並於 UI 端解析,僅剝行尾 stable-ID 註解。來源討論 review-before-manual-test-tasks 已裁定:引入 `[M]` 標記、三道守門統一改判「寫碼任務全完成」、章語意定為「驗證過」、freshness 接上封存守門。

## Goals / Non-Goals

**Goals:**

- `[M]` 手動測試任務標記正典化:行文法、解析、payload 曝光
- 三道守門統一採「寫碼任務全完成」預測子,單一實作共用
- freshness 任務錨改 manual-aware,定案語意並同步唯一正式碼消費者(desktop 徽章)
- 封存新增章失效守門,補上「蓋章後改碼無人擋」的既有缺口
- 五技能(review／verify／quality／propose／apply)文字同步與衍生物三連動

**Non-Goals:**

- GUI 的 `[M]` 徽章與寫碼／手測進度拆分顯示——任務列表顯示原文的字面 `[M]` 前綴即可(使用者反而能辨識手測任務),徽章化留待後續
- `[P]` 平行慣例的啟用(維持休眠)
- freshness 判定經 desktop 協定曝光(既有條文的紅線維持,本次只接封存守門)
- 章失效守門的豁免旗標(不設 accept-stale 類出口——重驗的出口是站別技能的續輪機制)
- 警告不擋模式(討論已否決:「驗證過」的章不得悄悄過期)

## Decisions

**D1 `[M]` 行文法與解析。** checkbox 後的前綴槽,與 `[P]` 同槽:解析器以重複剝除迴圈同時接受 `[P]` 與 `[M]`(順序不敏感、各至多一次),display 描述剝除標記——與 `[P]` 現行為一致,payload 消費端拿到乾淨文字。GUI 端讀原文故字面顯示 `[M]`,刻意接受(見 Non-Goals)。UI 勾選與拖放的寫回為行級改寫、只動 checkbox 記號,前綴自然保留,零腐蝕風險。
　替代案:獨立行尾註解(如 stable-ID 樣式)——被否:手寫負擔高、propose 起草不順手;`[P]` 前綴慣例已立,同槽最低認知成本。

**D2 「寫碼任務全完成」預測子。** 定義:所有 manual=false 的任務皆已勾。零任務 change 與全 `[M]` change 空真成立——守門僅在「寫碼任務總數大於零且未全完成」時拒絕,與既有 total>0 條件的組合語意同構。正典定義落在 manual-task-marker spec;單一實作落在 tasks 模組的進度統計(同時回傳全量與寫碼兩組計數),三道守門與 freshness 共用同一份計數。
　替代案:各守門自行過濾 manual 任務——被否:重複邏輯必漂移。

**D3 三道守門改判。** verify 落工單守門與兩站蓋章守門(station 模組)換用 D2 預測子,拒絕訊息改點名寫碼任務計數;review 技能的開跑守門(技能文字)改讀 payload 的寫碼進度欄位。守門的位置與順序皆不變——只換判定條件。

**D4 freshness 任務錨 manual-aware。** 章錨欄位形狀不變:reviewed_tasks_total／verified_tasks_total 仍記蓋章當時全任務總數(舊章資料相容)。判定式改為:「當前全任務總數＝蓋章時總數」且「寫碼任務全完成」,任一不成立即 stale;內容錨(scope 檔雜湊)規則不變。結果:蓋章後補勾或取消勾 `[M]` 任務不影響章;新增或刪除任務(總數變)→ stale;取消勾寫碼任務 → stale;改 scope 檔 → stale。freshness 門面簽名增寫碼計數參數(呼叫端:desktop 變更清單徽章、新的封存守門與測試)。已知限制:任務錨信任任務行的標記字面——蓋章後把寫碼任務改標 `[M]` 並退勾,任務錨不破(僅內容錨兜底);封存的任務完成度守門仍要求全勾(含 `[M]`),故改標無法讓未完成的工作進封存,不視為威脅。

**D5 封存的章失效守門。** 落在引擎單筆封存流程本體(與任務完成度守門同層),順序在任務完成度守門之後、任何封存檔案效果之前——任務未完成與章失效並存時,任務守門先拒、訊息不變。對 review 章與 verify 章各判一次:章欄位齊備且判 stale → 拒絕,非零 exit code,stderr 點名站別與破錨原因(內容錨列首個不符檔;任務錨述計數),指路重跑該站技能後再封存;兩章皆 stale 並列點名。Unknown(章欄位不全)與無章 → 放行,行為與現況逐位元一致。兩個邊界比照既有守門收斂:`--mark-tasks-complete` 的前置全勾寫入之前先判章失效(拒絕路徑零寫入,未手測的 `[M]` 不被代勾);工單開立中的站,其舊章不入失效判定——該站的封存處置由未結工單守門(擋下或 `--carry-*` 帶走)承載。批次封存經同一引擎流程,stale 章的拒絕沿既有 fail-fast 樣式中止批次並點名該 change,不靜默跳過。remote 封存通道(server 側)無工作樹可讀,內容錨無從判定:remote 路徑僅判任務錨(store 側資料足夠),內容錨跳過——非對稱屬已知限制,記入 spec 條文。
　替代案:守門放 CLI 動詞層——被否:desktop 與 server 通道漏接,違反任務完成度守門「引擎流程本體生效」的先例;豁免旗標——被否(見 Non-Goals)。

**D6 payload 曝光與 verb-contract。** instructions apply --json 的 tasks 逐項增 manual 欄位(鏡射既有 parallel);progress 增 codeTotal／codeComplete／codeRemaining 三欄。加欄不改名、不移除既有欄位;verb-contract 的 --json 形狀凍結契約以 delta 釘入新欄。其餘動詞輸出形狀不動。

**D7 技能文字與三連動。** review:開跑守門改讀 codeRemaining,大於零即停(原訊息語意);等於零而 remaining 大於零(僅餘 `[M]`)時繼續開審,並於報告向使用者點名尚餘手測任務與「蓋章可先落、手測完成後封存」的時序。verify:成品驗證 vs 中途盤點的分流改判 codeRemaining。quality:前提句轉述改「寫碼任務全完成」。propose:tasks 起草指引新增——人工驗收／手動測試類任務加 `[M]` 前綴。apply:一行原則——`[M]` 任務不由 agent 代勾,寫碼任務全勾即回報 apply 完成並提醒手測留給使用者。五技能 assets(claude 與 codex 兩形)、.claude/skills 同步檔、MARKER_VERSION、golden snapshot、assets.lock 三連動一次完成。

## Implementation Contract

**Behavior(可觀察行為):**

1. 解析:tasks.md 行「- [ ] [M] 描述」→ instructions apply --json 該任務 manual=true 且 description 不含標記;progress 帶 codeTotal／codeComplete／codeRemaining。無 `[M]` 的 change 僅新增欄位,其值與全量計數一致。
2. verify add-round:寫碼任務全完成、僅餘 `[M]` 未勾 → 落工單成功;寫碼任務未完成 → 拒絕,stderr 點名寫碼任務計數。
3. review stamp 與 verify stamp:同一條件放行/拒絕;蓋章寫入的 tasks_total 錨仍為全任務總數。
4. freshness 四情境:蓋章後補勾 `[M]` → fresh;改任一 scope 檔 → stale;新增任務 → stale;取消勾寫碼任務 → stale。
5. archive:帶 stale 章 → 拒絕(非零 exit code,stderr 點名站別、指路重跑該站);全 fresh、無章、Unknown → 與現況逐位元一致;任務未完成與 stale 並存 → 任務守門先拒且訊息不變;remote 通道僅判任務錨。
6. 技能:五技能依 D7 落地,claude 與 codex 兩形 golden 對照涵蓋。

**Interface / data shape:**

- Task 結構增 manual 旗標;tasks 進度統計回傳全量與寫碼兩組計數
- instructions apply --json:tasks 逐項 manual 欄位;progress 的 codeTotal／codeComplete／codeRemaining
- freshness 門面簽名增寫碼計數參數;引擎封存流程增章失效檢查(本機讀工作樹計算內容錨)
- 拒絕訊息:stamp 與 verify add-round 守門訊息改點名寫碼任務;archive 新增 stale 章拒絕訊息,樣式沿封存既有 Refusal 慣例

**Verification targets:**

- cargo 單元測試:tasks 解析(`[M]`/`[P]` 組合、剝離、兩組計數)、station 守門(add-round 與 stamp 各情境)、freshness 四情境、archive 守門(stale 拒絕/Unknown 放行/順序)
- CLI 整合測試:crates/speclink-cli/tests/it/manual_task_gates.rs 覆蓋行為 1–5;既有 review_verbs、verify_verbs、archive_readiness_gate 測試依新訊息調整
- golden:render_golden 全綠(MARKER_VERSION 與 assets.lock 更新後)
- 全套:cargo test 之 it 目標與 workspace 單元測試全綠

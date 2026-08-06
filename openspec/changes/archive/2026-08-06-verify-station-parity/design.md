## Context

change code-review-stage 已落地品質站機制（工單生命週期、章與雙錨、archive 守門、desktop 標示），以「常數參數化的具體函式」寫在 crates/speclink-core/src/review.rs——其 design D1 明訂：第二實例到來時再提升共通碼。本變更即該第二實例：verify 側補上工單、章與標示。既有 verify 技能為 `context: fork`＋Explore（read-only、可跑 Bash），三維度（Completeness／Correctness／Coherence）與 CRITICAL／WARNING／SUGGESTION 分級不動。討論 code-review-stage 第六～八輪定案：對稱補齊、共用機制、保留「verify 隨時可跑」的刻意不對稱。

後續討論 code-review-convergence-boundary 與 apply-provenance-scope 又定案：兩站都只能有一輪 discovery，Round 2+ 降為 remediation validation；程式碼證據須來自 frozen change hunks，而非 touched 整檔。前置 change converge-review-remediation-rounds 先建立 Apply baseline、Host change-diff resolver、old/new hunk ranges、structured round 與 fail-closed 行為；本變更只增加 verify 站別 adapter 與 snapshot 生命週期，不建立第二套 diff 演算法或全面 Apply provenance。

## Goals / Non-Goals

**Goals:**

- 驗證站端到端：`speclink verify` 動詞家族 → verify.md 工單 → verified_* 章（雙錨）→ desktop 驗證標示 → skill 收尾迴圈
- 共通生命週期提升為站別參數化共用碼，review 對外行為與 CLI 面零變化
- 兩站在 desktop 與 CLI 上呈現同一套心智模型（狀態機同構、守門同構、訊息同構）
- Round 1 對 frozen change patch 與全部 change artifacts 作唯一 discovery；Round 2+ 只驗收未解 findings 與 remediation patch，並以嚴格進展規則保證自動迴圈有終點

**Non-Goals:**

- verify 三維度檢查內容與分級邏輯的變更
- 中途盤點輪落工單
- server-web 凍結度、審查站行為變更（proposal Non-Goals 已列）
- touched schema、逐 edit 攔截、無 Git／跨 Host 重播與完整 Apply provenance

## Decisions

### D1 共通碼提升

新檔 crates/speclink-core/src/station.rs 承載站別參數化的共通生命週期（工單骨架產生／驗證／解析、structured phase／patch 欄位、蓋章原子寫入、指紋計算、失效純函式、archive 守門檢查）；站別差異收斂為一組常數（工單檔名、meta 欄位前綴、狀態詞、CLI 家族名）。crates/speclink-core/src/review.rs 改為薄實例（常數組＋委派），新增 crates/speclink-core/src/verify.rs 為第二薄實例。review 的公開函式簽名、converge-review-remediation-rounds 新增的 nullable JSON 欄位與 structured round 行為均不變；兩個前置 change 的測試不改一字即為回歸網。

- **替代案**：複製 review.rs 為 verify 專用碼——否決：兩份漂移，守門與失效規則日後分歧正是討論定案要避免的；此刻有兩個實例，提升時機正確（單實例期不建抽象的約束已解除）。

### D2 蓋章守門同構

`verify stamp` 守門與審查站同一條：任務全完成＋工單末輪零未解 findings、`--accept` 豁免後者；`verified_scope` 指紋＝工單各輪 Scope 聯集（路徑正規化與 CRLF→LF 規則同審查站，由 station.rs 共用碼保證位元級同構）。

- **替代案**：verify 蓋章放寬為「零 CRITICAL 即可蓋」——否決：兩站守門規則分歧，使用者要記兩套；「帶 WARNING 蓋章」的語意已由 `--accept` 承載，不需第二種放寬。

### D3 add-round 引擎守門

刻意不對稱：`verify add-round` 於任務未全數完成時拒絕（非零 exit code）。理由：verify 檢查可中途跑（進度盤點是 Completeness 維度的既有功能），若盤點輪誤落工單，「未結工單」將失去「成品驗證未收尾」的語意、還會誤觸 archive 守門。審查站無此引擎守門（其技能執行起點即自檢任務完成度，不存在中途跑情境）。

- **替代案**：靠 skill 文案自律不落工單——否決：乾淨 subagent 或手動操作者不受文案約束，引擎守門才是契約。

### D4 archive 雙工單守門

archive 的未結工單檢查由 station.rs 共用：偵測 verify.md → 預設拒絕、三處置（`verify stamp`／`verify discard`／`--carry-verify`）；review 與 verify 工單並存 → stderr 並列兩組處置，兩旗標可同時帶。`--carry-verify` 帶走的化石工單即封存側「曾驗證未通過」的證據。

- **替代案**：單一 `--carry-station` 旗標涵蓋兩種工單——否決：帶走哪種工單是兩個獨立決定，合併旗標喪失明示性。

### D5 desktop 同構呈現

speclink-desktop-core 的 query 層以與 reviewStatus 相同的判定碼計算 `verifyStatus`（active：`none`／`inVerify`／`verified`／`verifiedStale`；archived：`none`／`verified`／`verifiedNotPassed`），凍結度重算共用 station.rs 失效純函式。卡片兩章並排、順序固定（審查章在前、驗證章在後），各自 tooltip；抽屜驗證資訊列與審查資訊列同構。文案取 LANGUAGE.md 正典詞（驗證中／已驗證／已驗證·其後有變動／曾驗證未通過）。驗證章配色與圖示（討論 card-drawer-header-colors 裁決）：tone 值 SHALL 與審查章樣式表同值——色承載狀態（進行中藍、通過章色、其後有變動琥珀、未通過紅）、圖示形狀承載站別；驗證站圖示採 lucide 盾牌系（inVerify=Shield、verified=ShieldCheck、verifiedStale=ShieldAlert、verifiedNotPassed=ShieldX），與審查站徽章系（Stamp/BadgeCheck/BadgeAlert/BadgeX）可區辨。實作上 verify tone 表直接引用審查章樣式表的同值常數（單一來源），不複製色階字面。

- **替代案**：合併兩站為單一「品質」章（聚合狀態）——否決：討論已定案兩站互不遮蔽，聚合章重新引入混合裁決。

### D6 verify skill 收尾迴圈

skills.rs 的 verify 模板更新，但三維度與分級本身不變：任務未全完成時維持現行中途盤點，只作對話報告、不呼叫 `verify scope` 或 `verify add-round`。任務全完成時，主線先取得 verify frozen scope，再以 fork 執行對應 phase：

- discovery（唯一 Round 1）：讀取全部 change artifacts，並以 frozen change patch 與必要的呼叫端／測試作程式碼證據，完整執行 Completeness／Correctness／Coherence
- validation（Round 2+）：只取得上輪未解 findings、accepted 清單、remediation patch 與必要脈絡；逐筆判定原 finding 已解／未解，只能新增 remediation patch 直接造成的 regression，不得重新掃描未修改區域或整份 change

主線把相同 phase、patch hash、Scope 與 findings 寫入工單；未解 finding 以原文延續，避免靠改寫假裝集合縮小。令 Bn 為第 n 輪 triage 後「未接受且要求修正」的必修集合：Bn 為空且無 accepted 時 `verify stamp`；Bn 為空且有 accepted 時等使用者明示 `verify stamp --accept`；`0 < |Bn| < |Bn-1|` 才能再次選擇修正；`|Bn| >= |Bn-1|` 在記錄本輪後立即 failed，保留工單、不蓋章且不自動再試。不設固定最大輪數。

與 remediation patch 無關的新事項不進目前 round。只有同時具有現實觸發路徑、重現／失敗測試／明確 invariant 證據之一，且影響安全、資料損失或錯誤行為，才能讓本站以 scope changed／failed 結束並另開 discovery 或衍生 change；其他只列後續提示。修正一律回主線依 TDD 慣例執行，fork 不改檔；互動維持修正後重驗／接受蓋章／先不蓋三選項，codex 變體用純文字詢問。claude 與 codex 雙工具模板同步、golden 乾淨樹再生。

- **替代案**：把 verify 改成主線 orchestrator（與 review 完全同形）——否決：verify 檢查段單一 agent 即可（無兩軸 fan-out 需求），fork 隔離維持既有 token 與上下文優勢；只有互動收尾必須在主線。
- **替代案**：每次修正後完整重跑三維度 discovery——否決：同一檔未修改區域可持續產生新 finding，沒有可證明的終止條件。

兩站蓋章時序（討論 cross-station-staleness 定案）：站章凍結範圍檔內容指紋，蓋章後任何修改——含另一站 findings 的修正——都會把章轉「其後有變動」。兩站都跑時的慣例＝兩站 discovery 都以「先不蓋章」離場、findings 統一修正、各自 validation（validation patch 為上輪凍結點至現值的全部差異，機械式涵蓋他站修正）後兩章接連蓋。此慣例純屬技能／文件層時序，引擎與本 change 規格零變更，落點為 README 兩站分工表一句（任務 5.2）。乾淨 discovery（零 findings）依收尾迴圈仍立即蓋章、不設「先不蓋」出口——他站後續修正造成的暫態降級屬已知且接受（封存側定格為有章即綠、不重算凍結度）。

- **替代案**：為零 findings 增設「先不蓋章」出口——否決：須同步修改 review 站已封存的技能正典（MARKER_VERSION 升版＋golden 全套再生），且單堵 verify 側會破壞兩站同構；降級僅為封存前暫態，成本不成比例。

### D7 系統匣面板站章

討論 tray-station-badges 定案：macOS 面板的變更列比照卡片渲染品質站章。判準為「tray 收行動訊號、看板收閱讀脈絡」——站章直接影響收尾動作（未結工單擋封存、降級章提示留意），故納入；建立者頭像（單人專案零鑑別度）、來源討論標記（面板已有「已轉出」分區承載出身）、restale 與 metaError 標記不納入。兩章並排順序與卡片一致（審查章前、驗證章後），位於名稱與任務數之間；圖示、色調與 tooltip 詞條與卡片共用同一組樣式表（既有 reviewStyle 與本變更新增的 verify 對應樣式），不另建第二份對照。面板僅列 active change，紅色「曾審查／曾驗證未通過」結局章天然不出現。原生選單（非 macOS，及 macOS 面板建立失敗的後備）維持現行純文字 label 不變。資料面零新增：TraySnapshot.changes 即協定的 ChangeItem 清單，reviewStatus／verifyStatus 欄位已流至面板，僅呈現層未用。

- **替代案**：原生選單 label 塞 unicode 章字元——否決：單一字元承載四態、無 tooltip 無色彩，不可辨識。
- **替代案**：比照卡片納入全部行內符號（頭像／討論泡）——否決：閱讀脈絡在一瞥介面是雜訊。
- **替代案**：tray 審查章塞回 change code-review-stage——否決：其任務全完成且審查中，擴 scope 破蓋章守門並拖延封存；且面板站章為單一 Requirement 同時規範兩章，拆兩個 active change 對同一 Requirement 出 delta 直接衝突。

### D8 共用 frozen scope 與站別 snapshot

本變更依賴 converge-review-remediation-rounds，直接復用其 `.speclink/review-scopes/<change>/baseline.json` 與 crates/speclink-host/src/change_diff.rs；不再實作 Git diff、rename、untracked、old/new ranges 或歧義判定。新增 `speclink verify scope` 作站別 adapter，旗標、human／JSON payload、`needsInput` 與 `--no-color` 契約和 `review scope` 同構，但 phase 由 verify.md 判定：無工單為 discovery，有 structured 工單為 validation。

verify 的 remediation snapshots 放在 `.speclink/review-scopes/<change>/verify-snapshots/<digest-hex>.json`，與 review snapshots 分開清理；內容格式復用 Host snapshot 型別，patch hash 同樣錨定 old/new hunk ranges 與 before／after hashes。`verify stamp`／`verify discard` 只清除 verify snapshots並保留 Apply baseline與 review snapshots；清除失敗只警告，不回滾 canonical mutation。legacy verify round 缺 phase／patch 或 referenced snapshot 缺失時 fail closed，保留工單並要求使用者明示 discard 後重新 discovery，不得退回 touched 整檔重驗。

這是站別 review-time snapshot，不是 Apply provenance：資料不進 touched、TeamStore 或 metadata，也不承諾無 Git、跨 Host或 workspace 前進後重播。remote workspace 仍由 agent 的 local checkout 執行相同 Host resolver，只透過 typed client 取得 verify ticket；server 不新增 Git endpoint。

- **替代案**：讓 verify 直接讀 review snapshots——否決：兩站可獨立執行與蓋章，任一站清理 snapshot 都會破壞另一站的續輪。
- **替代案**：verify 以 touched 整檔另算 scope——否決：重複演算法且重新引入超出 change hunks 的 discovery。

## Implementation Contract

**In scope**：speclink-core（station.rs 新增、review.rs 薄化、verify.rs 新增、model.rs、archive.rs、skills.rs、listing.rs parity 延伸）、speclink-host（既有 resolver 的 verify 站別 adapter／snapshot namespace）、speclink-cli（verify 子命令）、speclink-protocol／remote／server（structured verify round parity，不新增 Git endpoint）、speclink-desktop-core（verifyStatus）、packages/ui 與 apps/desktop（標示、對話框、i18n、系統匣面板站章）、golden、README。
**Out of scope**：verify 三維度內容與分級、change-diff 演算法重寫、touched schema、完整 Apply provenance、server-web、審查站對外行為。

可驗證行為：

1. `verify add-round` 於任務 4/5 時非零 exit 並說明；任務 5/5 時建立／追加 verify.md；structured Round 1 只能是 discovery，Round 2+ 只能是 validation，phase／patch 必須成對
2. `verify stamp` 守門與效果與審查站同構：五個 verified 欄位＋工單刪除為同一原子寫入
3. 失效：修改 verified_scope 任一檔內容 → verifiedStale；行尾差異不觸發
4. review.rs 薄化後，change code-review-stage 的全部測試不修改而通過（共通碼提升的回歸網）
5. `speclink list --json` 在 meta 帶 verified_* 時輸出形狀不變（parity pin 延伸）
6. archive：僅 verify 工單 → 三處置含 `--carry-verify`；雙工單並存 → stderr 並列兩組處置；`--carry-verify` 後封存目錄含 verify.md 且封存側顯示「曾驗證未通過」
7. desktop query：verifyStatus 四態＋archived 三態各有 fixture；卡片兩章並排順序固定
8. golden：verify skill 模板（claude／codex）再生後明確包含唯一 discovery、validation 不探索新事項、必修集合不縮小即 failed
9. tray 面板：變更列兩章並排順序固定且圖示／tooltip 與卡片同構；頭像／討論泡等非站章符號不出現於列；原生選單變更列標籤位元級不變
10. `verify scope --json` 與 `review scope --json` 的 resolved／needsInput shape 同構；兩站 snapshot namespace 與 cleanup 互不影響
11. 以 2 筆必修開始的 fixture：2→1 可續跑，1→1 立即 failed 且無 stamp；1→0 乾淨蓋章；只剩 accepted 走 `--accept`
12. snapshot 缺失、patch hash 漂移或 legacy 工單無法對應 snapshot 時保留工單並非零停止，不退回 touched 整檔 discovery

## Risks / Trade-offs

- **回歸對照（最優先）**：共通碼提升動到 review.rs——以「change code-review-stage 測試零修改通過」為硬性驗收；CLI listing parity pin 延伸 verified_*。
- **跨平台**：指紋規則由 station.rs 單一實作保證兩站同構，Windows 路徑與 CRLF 行為隨審查站既有測試矩陣。
- **golden 再生髒樹污染**：乾淨樹再生（既有紀律）。
- **與 change code-review-stage 的順序耦合**：本變更以其完成為前置；若平行開工，station.rs 提升將與 review.rs 實作互相踩踏——實作前確認前置 change 已封存或至少任務全綠。
- **與 converge-review-remediation-rounds 的順序耦合**：station.rs 薄化與 verify scope 必須建立在其 structured parser 與 Host resolver 之後；以該 change 測試零修改通過釘住回歸，不平行修改同一 review.rs／skills.rs。
- **站別 snapshot 清理**：兩站共用 Apply baseline但不能共用可刪除 snapshot；以獨立 verify-snapshots namespace 與交叉 cleanup fixture 防止互相失效。

# review-skill Specification

## Purpose

/speclink-review 技能的內容與行為：技能檔的生成與正典化、審查流程怎麼跑（Standards 與 Correctness 兩軸與 Standards 軸的 smell baseline）、審查結果的裁量分類，以及修復迴圈的驗證門與收尾。本 capability 保證審查產出跟隨專案語系、續輪把已接受事項前饋以免反覆爭同一點，且續輪冒出重大晚發問題時有安全退出而非硬蓋章了事。

## Requirements

### Requirement: 審查技能的生成與正典化

`speclink update` SHALL 生成 `/speclink-review` 技能檔至 claude 與 codex 兩工具的技能目錄，內容以引擎內的正典模板為準（golden 對照涵蓋）。同次更新 SHALL 將生成之 CLAUDE.md／AGENTS.md 的 workflow 行改為含模型發起入口、品質關卡與並行品質站的版本（`discuss?/improve? → propose → apply ⇄ ingest → (quality? | review? ∥ verify?) → archive`），並於技能使用清單加入審查站的觸發時機（實作完成、封存之前、由使用者判斷是否執行）。

#### Scenario: 技能檔生成

- **WHEN** 於已啟用 speclink 的專案執行 `speclink update`
- **THEN** claude 與 codex 的技能目錄各出現 speclink-review 技能檔，且內容與 golden 對照一致

#### Scenario: workflow 行更新

- **WHEN** `speclink update` 完成後讀取生成的 CLAUDE.md
- **THEN** workflow 行含 `(quality? | review? ∥ verify?)` 且技能清單含審查站條目


<!-- @trace
source: add-improve-flow
updated: 2026-08-07
-->

---
### Requirement: 審查流程的技能行為

技能文件 SHALL 指示主線 orchestrator 依序執行:(1) 選定 change;(2) 守門自檢改判寫碼任務——自 instructions apply payload 讀寫碼進度,codeRemaining 大於零即停止;寫碼任務全完成而僅餘 `[M]` 手動測試任務未勾時 SHALL 繼續,並於結果呈現向使用者點名尚餘的手動測試任務與「蓋章可先落、手動測試完成後封存」的時序;(3) 呼叫 review scope 取得 frozen patch——無工單時 phase=discovery,有工單時 phase=validation;needsInput(僅發生於 discovery)時 SHALL 等待使用者提供可信 base、hash-pinned hunk selection 或隔離 worktree,不得以 touched 整檔替代;(4) 讀取 change artifacts 作判準脈絡;(5) 依 phase 執行以下分流;(6) 並列呈現結果與 remediation triage;(7) 以 review add-round 寫入相同 phase、patchHash、Scope 與 findings。

Discovery SHALL 將同一 frozen patch 平行交給 Standards(repo 慣例文件＋smell baseline,repo 文件優先)與 Correctness(bug hunting)兩個 read-only sub-agent,各以 400 字內回報並以 CRITICAL/WARNING/SUGGESTION 分級。兩軸 SHALL 只以 change hunks 與判斷直接影響所需的呼叫端、測試為審查面;兩份報告 SHALL 原樣並列,不合併、不跨軸重排。Spec compliance SHALL NOT 在審查站裁決。

Validation SHALL 只把上輪未解 findings、accepted 清單、remediation patch 與必要脈絡交給對應 axes;sub-agent SHALL 逐筆判定原 finding 已解/未解,並只回報 remediation patch 直接引入的 regression。remediation patch 內 attribution 為 "adjacent" 的段落 SHALL 由 sub-agent 逐段確認確屬本次修復——不屬於本次修復的 adjacent 段 SHALL 以 regression 回報,SHALL NOT 靜默採認。未解 finding SHALL 由主線以原文寫入新輪;已解 finding SHALL 從新輪 findings 移除;未修改區域的新 smell、SUGGESTION 或既存問題 SHALL NOT 加入。scope 回報的 outOfScopeChanged 非空時,主線 SHALL 於呈現結果時原樣轉知使用者,SHALL NOT 將其列入審查面或工單 findings。

artifacts 稀薄時 sub-agent SHALL 僅憑 code 與測試判斷,不臆造需求。locale SHALL 沿用既有「審查產出的語言綁定」契約;phase、patchHash、severity、axis prefix 與 path 保持英文 token。

#### Scenario: 寫碼任務未完成即停

- **WHEN** 對寫碼任務 3/5 的 change 執行 speclink-review
- **THEN** 技能停止並說明審查站要求寫碼任務全數完成,不呼叫 review scope、不派出 sub-agent、不寫工單

#### Scenario: 僅餘手動任務時照常開審

- **WHEN** 對寫碼任務全勾、僅一個 `[M]` 任務未勾的 change 執行 speclink-review
- **THEN** 技能照常進入 review scope 與 discovery 流程,並於結果呈現點名尚餘的手動測試任務與後續時序

#### Scenario: 首輪只審 frozen change hunks

- **WHEN** touchedFiles 含一份 300 行檔案,而 resolved discovery patch 只含其中兩個 hunks
- **THEN** Standards 與 Correctness 都收到相同兩個 hunks及必要上下文,不把其餘未修改內容當 discovery 面

#### Scenario: 續輪只驗收上輪 findings 與 remediation patch

- **WHEN** Round 1 有兩筆未解 findings,修正 patch 只改其中一檔並新增一個呼叫端
- **THEN** validation 只判定兩筆原 finding 與該 patch 的直接 regression,不重新掃描整個 finding 檔案或 change

#### Scenario: adjacent 段須確認歸屬

- **WHEN** 驗證輪的 remediation patch 含一個 attribution "adjacent" 的段落
- **THEN** sub-agent 簡報載明須逐段確認該段確屬本次修復,不屬於者以 regression 回報

#### Scenario: 末輪零 findings 時重試蓋章而非重審

- **WHEN** 工單末輪 findings 為空但先前 stamp 因外部守門失敗而留下工單
- **THEN** 技能在守門恢復後直接重試 review stamp,不派出新的 discovery 或 validation

#### Scenario: legacy 工單缺 snapshot 時 fail closed

- **WHEN** 工單有 findings但 lastRound.patchHash 為 null,且 host 沒有可對應 snapshot
- **THEN** 技能說明無法精確重建 remediation delta,保留工單並等待使用者明示 discard 後重新 discovery,不得重審整檔


<!-- @trace
source: manual-task-marker-gates
updated: 2026-08-11
-->

---
### Requirement: Standards 軸的 smell baseline

技能文件的 Standards sub-agent 指示 SHALL 逐字內嵌 smell baseline 正典原文（來源：Matt Pocock code-review skill，MIT；正典文字見 design「D7a」）：引言（"On top of whatever the repo documents…"）、兩條約束規則（The repo overrides——repo 文件化標準勝出、被其背書者壓下不報；Always a judgement call——smells 一律為 "possible X" 標籤式啟發、永不作 hard violation，且 tooling 已強制的規則跳過不報）、"(Refactoring, ch.3)" 出處，以及 12 條 smells 逐項（what it is → how to fix）。12 個專有名詞 SHALL 原文不動：Mysterious Name、Duplicated Code、Feature Envy、Data Clumps、Primitive Obsession、Repeated Switches、Shotgun Surgery、Divergent Change、Speculative Generality、Message Chains、Middle Man、Refused Bequest。生成之技能檔 SHALL 含一行出處註記。

smell baseline 的套用 SHALL 依 config 的 review.baseline 決定：extend（含未設定）時，Standards 軸的 sub-agent 簡報 SHALL 帶入 baseline；replace 時，簡報 SHALL NOT 帶入 baseline，12 條 smell 不檢查、不入單。技能檔內文 SHALL 照舊逐字內嵌 baseline，不因設定而改變生成內容。12 個專有名詞的清單 SHALL 由引擎單一提供給呈現端（如 desktop 設定頁），且 SHALL 以測試釘住該清單與技能檔內嵌的 12 條逐名一致。

#### Scenario: 生成技能檔含完整 baseline

- **WHEN** `speclink update` 生成 speclink-review 技能檔（claude 與 codex）
- **THEN** 兩份檔內皆含 12 條 smells 專有名詞逐項、兩條約束規則與 "(Refactoring, ch.3)" 出處，內容與 design「D7a」正典文字逐字一致

##### Example: 專有名詞逐一在場

- **GIVEN** 生成的 speclink-review 技能檔
- **WHEN** 逐一檢索 Mysterious Name、Duplicated Code、Feature Envy、Data Clumps、Primitive Obsession、Repeated Switches、Shotgun Surgery、Divergent Change、Speculative Generality、Message Chains、Middle Man、Refused Bequest
- **THEN** 12 個名稱皆命中，且各項皆帶「→」修法句

#### Scenario: replace 時簡報不帶 baseline

- **WHEN** 檢視生成的 speclink-review 技能檔關於 Standards 軸簡報的段落
- **THEN** 段落明定 review.baseline 為 replace 時簡報不帶 smell baseline、12 條不檢查不入單；為 extend 或未設定時照舊帶入

#### Scenario: 引擎提供的 12 條清單與技能檔一致

- **WHEN** 執行 speclink-core 的測試
- **THEN** 測試確認引擎提供的 12 個名稱與生成的 speclink-review 技能檔內嵌的 12 條逐名一致，任一端改名或增刪即失敗


<!-- @trace
source: review-rules-config
updated: 2026-10-14T16:37:17+08:00
-->

---
### Requirement: 審查產出的語言綁定

技能文件 SHALL 將 workflow config 解析出的 locale（守門自檢步驟所取 payload 的 `locale` 欄位）綁定整條審查產出鏈：兩軸 sub-agent 指示 SHALL 攜帶該語言並要求 finding 描述以該語言撰寫；並列呈現與 `review add-round` 寫入工單的 findings 描述 SHALL 與 sub-agent 產出同語言，主線 SHALL NOT 另行翻譯。severity 標籤（CRITICAL／WARNING／SUGGESTION）、軸前綴（`Standards:`／`Correctness:`）、檔案路徑與指令行 SHALL 維持英文；工單固定骨架屬動詞所有文法（見 review-station 規格），不在本條範圍。locale 未設定時，sub-agent 報告、呈現與工單記錄 SHALL 為英文。smell baseline 正典原文（design「D7a」）屬 sub-agent 指示本文，SHALL 維持逐字英文內嵌，不受本條影響。

#### Scenario: locale tw 時工單 findings 為中文

- **WHEN** 專案 openspec/config.yaml 設 locale: tw，審查輪產出 findings 並以 `review add-round` 寫入工單
- **THEN** 該輪 findings 行的描述文字為繁體中文，severity 標籤與 `Standards:`／`Correctness:` 前綴維持英文

#### Scenario: locale 未設定時全英文

- **WHEN** 專案未設定 locale，執行一輪審查
- **THEN** sub-agent 報告、並列呈現與工單 findings 描述皆為英文


<!-- @trace
source: code-review-stage
updated: 2026-08-02
code:
  - AGENTS.md
  - CLAUDE.md
  - README.en.md
  - README.md
  - apps/desktop/core/src/cache.rs
  - apps/desktop/core/src/query.rs
  - apps/desktop/core/src/verbs.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src/App.tsx
  - apps/desktop/src/__tests__/store.test.ts
  - apps/desktop/src/__tests__/tauriDataSource.test.ts
  - apps/desktop/src/adapter/tauriDataSource.ts
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/store.ts
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/review_verbs.rs
  - crates/speclink-core/assets/skills/review.md
  - crates/speclink-core/src/archive.rs
  - crates/speclink-core/src/command/mod.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/inprogress.rs
  - crates/speclink-core/src/lib.rs
  - crates/speclink-core/src/listing.rs
  - crates/speclink-core/src/model.rs
  - crates/speclink-core/src/review.rs
  - crates/speclink-core/src/skills.rs
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/teststore.rs
  - crates/speclink-core/src/util.rs
  - crates/speclink-core/tests/golden/assets.lock
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/render_golden.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/commit.rs
  - crates/speclink-node/src/store_bridge.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/client_errors.rs
  - crates/speclink-remote/tests/typed_client.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/e2e_cli.rs
  - crates/speclink-server/tests/read_api.rs
  - crates/speclink-server/tests/review_api.rs
  - packages/ui/src/__tests__/reviewBadge.test.tsx
  - packages/ui/src/adapter.ts
  - packages/ui/src/components/ArchivedDrawer.tsx
  - packages/ui/src/components/ArchivedList.tsx
  - packages/ui/src/components/ChangeCard.tsx
  - packages/ui/src/components/ReviewArchiveDialog.tsx
  - packages/ui/src/components/RichDetailDrawer.tsx
  - packages/ui/src/components/reviewStyle.tsx
  - packages/ui/src/i18n.tsx
  - packages/ui/src/index.ts
-->

---
### Requirement: 審查後的迴圈與收尾

Discovery 呈現與 triage 後，有必修 findings 時技能 SHALL 沿既有三選項讓使用者選擇：修正後重審／接受現狀蓋章／先不蓋章；零 findings 或僅 SUGGESTION 級 findings 時 SHALL NOT 詢問——單站直接呼叫時記錄該輪後直接執行 review stamp（乾淨蓋章，SUGGESTION 級不擋章、無需批准）。修正 SHALL 一律由主線依專案 TDD 慣例執行，sub-agent 不得修改檔案；修正後 SHALL 先通過「修復迴圈的驗證門」，再開始 validation。於 quality 時序中（由 /speclink-quality 依序呼叫時），零 findings 或僅 SUGGESTION 的 discovery 與必修集合淨空的 validation 輪皆 SHALL NOT 當場蓋章，SHALL 改走既有「先不蓋章」離場，蓋章延至 quality 的收尾補蓋；惟編排方明示本次呼叫為收尾補蓋時，此禁蓋例外 SHALL NOT 適用——該呼叫中必修淨空的輪即蓋。quality 收尾補蓋 SHALL 區分必修淨空末輪的來源：外部守門失敗留下者沿既有路徑直接重試 stamp；quality 時序刻意留下者 SHALL 僅於收尾補蓋呼叫中蓋章——該呼叫先以 review scope 確認凍結點後內容未再移動，無移動即重試 stamp，有移動則於同一呼叫內先跑 validation 輪至必修淨空再蓋，SHALL NOT 對未驗證的移動直接補蓋；同一必修淨空末輪若由非收尾補蓋的呼叫進入，無論凍結點後有無移動皆 SHALL NOT 蓋章。單站直接呼叫時行為不變。

每輪 validation 後，技能 SHALL 以未接受的必修集合 Bn 與上輪 Bn-1 比較：

- Bn 為空且沒有 accepted 必修 findings 時 SHALL 執行 review stamp，結果為 passed clean——末輪殘留的 SUGGESTION 級 findings 不擋章、無需批准
- Bn 為空且仍有 accepted 必修 findings 時 SHALL 推薦使用者明示 review stamp --accept，結果為 passed with reservations
- Bn 非空且數量嚴格小於 Bn-1 時 SHALL 允許使用者再次選擇修正後驗收、接受現狀或先不蓋章
- Bn 數量大於或等於 Bn-1 時 SHALL 在記錄本輪後立即以 failed 結束自動迴圈，保留工單、不蓋章且不得自動再試

blocking set 的縮小只決定能否繼續自動修正，SHALL NOT 被描述為品質分數或通過。技能 SHALL NOT 設固定最大輪數；每次允許續跑都必須嚴格下降。互動工具不可用時 SHALL 以純文字詢問並等待回覆。

#### Scenario: 乾淨首輪自動蓋章

- **WHEN** 單站直接呼叫且 discovery 的兩軸皆零 findings
- **THEN** 技能記錄零 findings 的 discovery round，執行 review stamp 並回報 passed clean

#### Scenario: 僅 SUGGESTION 首輪直接蓋章

- **WHEN** 單站直接呼叫且 discovery 僅記錄 SUGGESTION 級 findings、無任何必修
- **THEN** 技能記錄該 discovery round 後不發三選項詢問，直接執行 review stamp 並回報 passed clean，報告列出 SUGGESTION 清單

#### Scenario: quality 時序中乾淨首輪先不蓋章

- **WHEN** 於 quality 時序中 discovery 的兩軸皆零 findings
- **THEN** 技能記錄零 findings 的 discovery round，以「先不蓋章」離場，工單與 host-local snapshot 保留，不執行 review stamp

#### Scenario: quality 時序中複驗淨空仍先不蓋章

- **WHEN** 於 quality 時序中（本次呼叫非收尾補蓋）validation 輪後必修集合為空且無 accepted 必修 findings
- **THEN** 技能記錄該輪後以「先不蓋章」離場，不執行 review stamp，蓋章延至 quality 的收尾補蓋

#### Scenario: 非收尾呼叫進入乾淨末輪不蓋章

- **WHEN** quality 時序中非收尾補蓋的呼叫進入已存在的必修淨空未蓋章末輪，且 review scope 顯示凍結點後無內容移動
- **THEN** 技能回報無新內容可判並結束，不執行 review stamp、不動工單

#### Scenario: quality 收尾補蓋前內容再移動則先驗後蓋

- **WHEN** quality 收尾補蓋時 review scope 顯示必修淨空末輪凍結點後仍有內容移動
- **THEN** 技能於同一呼叫內先執行 validation 輪，必修淨空後即執行 review stamp（收尾補蓋呼叫不受禁蓋例外攔截），不對未驗證的移動直接補蓋

#### Scenario: 有進展時允許再驗收

- **WHEN** 上輪有兩筆必修，validation 後剩一筆未解且沒有直接 regression
- **THEN** 技能記錄新輪並允許再次選擇修正，SHALL NOT 宣稱已通過

#### Scenario: 第一個無進展輪立即停止

- **WHEN** 上輪有一筆必修，validation 後同一筆仍未解
- **THEN** 技能記錄該輪後回報 failed，保留工單、不蓋章且不再派出 sub-agent

#### Scenario: 只剩保留事項

- **WHEN** validation 已解決所有必修但末輪仍帶 accepted 必修 findings
- **THEN** 技能推薦 review stamp --accept，不再為 accepted items 啟動 validation

#### Scenario: 先不蓋章離場

- **WHEN** 使用者在可選擇的節點選擇先不蓋章
- **THEN** 技能結束，工單與 host-local snapshot 保留，metadata 不變


<!-- @trace
source: stamp-blocking-set-alignment
updated: 2026-08-10
-->

---
### Requirement: 審查結果的裁量分類

技能 SHALL 於兩軸結果並列呈現後、詢問使用者之前，對本輪每筆 finding 給出處置分類並隨報告一併呈現（不改動工單記錄格式）：**必修**——CRITICAL 級、Correctness 軸判定有現實觸發路徑的 bug（含 WARNING 級）、文件化 repo 標準的明確違反、專案審查規則 critical 或 warning 清單的違反；**可裁**——"possible X" 措辭的 smell 判斷、專案審查規則 suggestion 清單的違反與其他 SUGGESTION 級事項，每筆附一行修繕成本與效益的裁量理由。可裁事項 SHALL 一律以 SUGGESTION 級記錄入單、SHALL NOT 以 WARNING 級入單——WARNING 保留給必修級判定；SUGGESTION 級不擋蓋章、不進入接受機制。三選項詢問 SHALL 僅於必修項存在時發出並帶明確推薦：推薦「修正後重審」並列出必修清單；僅剩可裁項時 SHALL NOT 詢問，依「審查後的迴圈與收尾」直接乾淨蓋章（單站直接呼叫）或先不蓋章離場（quality 時序中）。

#### Scenario: 有必修項時推薦修正

- **WHEN** 某輪 findings 含一筆 CRITICAL 與三筆 possible-X 措辭的 SUGGESTION
- **THEN** 呈現的分類為 1 筆必修、3 筆可裁，三選項詢問以「修正後重審」為推薦選項且附必修清單

#### Scenario: 僅剩可裁項時不詢問直接蓋章

- **WHEN** 某輪 findings 僅含 SUGGESTION 級可裁事項，無 CRITICAL、無現實路徑 bug、無文件化標準違反
- **THEN** 技能不發三選項詢問；單站直接呼叫時記錄該輪後直接執行 review stamp（乾淨蓋章），報告列出 SUGGESTION 清單

#### Scenario: 可裁事項一律以 SUGGESTION 入單

- **WHEN** Standards 軸對一段程式碼給出 possible Feature Envy 的 smell 判斷
- **THEN** 該筆以 SUGGESTION 級寫入工單，SHALL NOT 以 WARNING 級入單

#### Scenario: 專案規則 warning 清單的違反列為必修

- **WHEN** config 的 review.warning 含「新增的公開 API 要有對應的測試」，Standards 軸判定本輪新增的公開 API 沒有測試
- **THEN** 該筆以 WARNING 級入單並分類為必修，觸發三選項詢問；同輪 review.suggestion 清單的違反以 SUGGESTION 級入單並分類為可裁


<!-- @trace
source: review-rules-config
updated: 2026-10-14T16:37:17+08:00
-->

---
### Requirement: 修復迴圈的驗證門

使用者選擇「修正後重審」(或於 quality 時序中裁示修正)時，技能 SHALL 於修正完成後、派出下一輪 sub-agent 之前，執行建置並跑受影響範圍的測試，要求全數通過；未通過時 SHALL 先修復至全綠，不得帶著建置或測試失敗進入下一輪。受影響範圍 SHALL 依序判定：

1. 修正改到的檔案落在某個任務說明列出的路徑內時，跑該任務說明寫的驗證指令（任務原文取自 `speclink instructions apply --change <name> --json` 的 `tasks`）；
2. 任務沒涵蓋的檔案，依專案文件化的測試政策（CLAUDE.md、AGENTS.md 或 workflow config 的產出規則）選擇測試；
3. 專案未寫任何測試政策時，跑 repo 慣例的全量測試指令。

建置 SHALL 照跑，SHALL NOT 因範圍判定而略過。結果呈現 SHALL 列出本次跑了哪些測試指令、略過哪些範圍，並註明略過的範圍交由 CI 守門。

修正動到 artifact（規格、design、tasks）時，技能 SHALL 依三種情況處理：

- 只改程式碼使其符合規格：SHALL NOT 修改 artifact。
- 規格字面要修、意圖不變：主線直接修改 artifact，並在 `speclink validate <change>` 通過後才進下一輪；結果呈現 SHALL 列出改了哪幾段；下一輪的 sub-agent 簡報 SHALL 附上改過段落的原文——驗證 patch 不含 spec 目錄下的檔案，不附上則下一輪看不到修正。
- 意圖或範圍改變（修正需要新增任務，或改變某條 Requirement 的意圖）：技能 SHALL 以既有「範圍改變」出口結束本站（保留工單、不蓋章），並建議先執行 `/speclink-ingest` 再重跑；SHALL NOT 於迴圈內自行新增或退回任務。

#### Scenario: 修正引入的編譯錯誤被驗證門攔下

- **WHEN** 修正過程改壞另一呼叫端導致專案建置失敗
- **THEN** 技能於派出下一輪 sub-agent 之前發現並修復，下一輪開始時建置與受影響範圍的測試全綠

#### Scenario: 只跑任務寫的驗證指令

- **WHEN** 修正只改了任務 2.1 說明列出的檔案，該任務說明寫明以 `cargo test -p speclink-core` 驗證
- **THEN** 技能執行建置與 `cargo test -p speclink-core`，不執行 workspace 全量測試；結果呈現列出已跑的指令與略過的其餘 package，並註明交由 CI 守門

#### Scenario: 專案未寫測試政策時跑全量

- **WHEN** 修正改到的檔案不在任何任務說明的路徑內，且專案的 CLAUDE.md、AGENTS.md 與 workflow config 都沒有測試政策
- **THEN** 技能執行 repo 慣例的全量測試指令，與本變更前的行為相同

#### Scenario: 規格字面修正後驗證並附上段落

- **WHEN** 使用者裁示的修正把 delta 規格某個 Scenario 的 THEN 改寫成與程式碼一致、意圖不變
- **THEN** 技能修改該 delta 規格、`speclink validate` 通過後才進下一輪，結果呈現列出改過的段落，下一輪簡報附上該段原文

#### Scenario: 意圖改變時走範圍改變出口

- **WHEN** 某筆 finding 的修正需要在 tasks.md 新增一項任務
- **THEN** 技能不新增任務、不寫入新輪，以「範圍改變」結束本站並保留工單、不蓋章，建議先執行 /speclink-ingest 再重跑


<!-- @trace
source: quality-gate-speedup
updated: 2026-10-14T21:31:03+08:00
-->

---
### Requirement: 已接受事項的續輪前饋

接受機制 SHALL 僅適用於必修級 findings——SUGGESTION 級不擋章、不需接受，未修的 SUGGESTION 依既有 validation 規則以未解 finding 原文續帶、SHALL NOT 附 `(accepted)` 標記。已裁定接受而未修正的必修 findings，技能於續輪 SHALL 雙軌處置：(1) 續輪 sub-agent 的指示 SHALL 附上該清單並明令不得重報同一事項或其近似變體；(2) 續輪記錄 SHALL 由主線將這些事項原樣帶入該輪 findings 清單，並於行末附結構性標記 `(accepted)`（比照 severity 標籤維持英文、不隨 locale 翻譯），使末輪工單忠實反映殘留保留事項，蓋章走 `review stamp --accept`。跨 session 接手時，技能 SHALL 以末輪帶 `(accepted)` 標記的行重建不重報清單。

#### Scenario: 接受過的事項不再重報

- **WHEN** Round N 的一筆 WARNING 級現實路徑 bug 經使用者裁定接受後執行下一輪
- **THEN** 續輪 sub-agent 指示含該事項的不重報清單，且 Round N+1 的工單記錄由主線原樣帶入該筆事項並以 `(accepted)` 標記收尾

#### Scenario: 跨 session 重建不重報清單

- **WHEN** 另一 session 對末輪含 `(accepted)` 標記行的工單執行 `/speclink-review`
- **THEN** 該 session 的 sub-agent 指示以標記行重建不重報清單，標記事項不被重報


<!-- @trace
source: stamp-blocking-set-alignment
updated: 2026-08-10
-->

---
### Requirement: 續輪重大晚發問題的安全退出

Validation 偶然看見與 remediation patch 無關的新問題時，技能 SHALL NOT 將它加入目前 findings 或重新開啟 discovery。只有問題同時具有現實觸發路徑、重現方式／失敗測試／明確 invariant 破壞之一，且影響安全、資料損失或錯誤行為時，技能 SHALL 以 scope changed／failed 結束本站，保留原工單且不蓋章，並建議另開 discovery 或衍生 change。證據不足或不達門檻的事項 SHALL 僅列為後續提示，不阻斷目前 validation。

#### Scenario: 無關 smell 不加入續輪

- **WHEN** validation 期間注意到未修改鄰檔的 possible Duplicated Code
- **THEN** 該事項不寫入目前 round、不改變 blocking set，也不觸發新的 discovery

#### Scenario: 有證據的資料損失問題終止本站

- **WHEN** validation 期間發現與 remediation patch 無關但有失敗測試可重現的資料損失
- **THEN** 技能回報 scope changed／failed、保留工單且不蓋章，建議另開 discovery 或衍生 change

<!-- @trace
source: converge-review-remediation-rounds
updated: 2026-08-03
code:
  - AGENTS.md
  - CLAUDE.md
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/it/remote_verb_parity.rs
  - crates/speclink-cli/tests/it/review_verbs.rs
  - crates/speclink-core/assets/skills/apply.md
  - crates/speclink-core/assets/skills/review.md
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/review.rs
  - crates/speclink-core/src/tasks.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/golden/assets.lock
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/it/render_golden.rs
  - crates/speclink-host/src/change_diff.rs
  - crates/speclink-host/src/lib.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/it/typed_client.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/it/review_api.rs
-->

---
### Requirement: 專案審查規則的讀取與套用

技能 SHALL 於開審前以 speclink workflow-config show --json 讀取 review 物件（baseline 與 critical、warning、suggestion 三個清單），SHALL NOT 直接讀取 config 檔；remote 模式經同一動詞取得 server 端設定。讀取失敗（例如 config 無法解析）時技能 SHALL 原樣回報錯誤並停止，SHALL NOT 在缺少專案規則的情況下開審。

Discovery 時 Standards 軸的 sub-agent 簡報 SHALL 依嚴重度分組帶入三個清單的規則原文；Correctness 軸簡報 SHALL NOT 帶入。違反專案規則的 finding SHALL 以該規則所在清單決定嚴重度（critical→CRITICAL、warning→WARNING、suggestion→SUGGESTION），sub-agent 與主線 SHALL NOT 另行升降級；finding 描述 SHALL 原文引用被違反的規則，描述其餘部分依「審查產出的語言綁定」使用解析後的 locale。專案規則與 repo 文件化標準衝突時，專案規則 SHALL 勝出。Validation 輪 SHALL NOT 因專案規則產生新 finding——續輪只判前輪 finding 是否已解與 remediation patch 的直接 regression，與既有語意相同。三個清單皆空且 baseline 為 extend 時，審查行為 SHALL 與未設定 review 時完全相同。

#### Scenario: 違反 critical 清單的規則

- **WHEN** config 的 review.critical 含「所有 SQL 一律用參數化查詢，不得用字串拼接」，frozen patch 中有一段以字串拼接組出 SQL
- **THEN** Standards 軸回報一筆 CRITICAL finding，描述原文引用該規則；主線照該嚴重度入單並分類為必修

##### Example: 工單中的 finding 行（locale 為 tw）

- **GIVEN** review.critical 含「所有 SQL 一律用參數化查詢，不得用字串拼接」，locale 為 tw
- **WHEN** crates/x/src/db.rs 第 42 行以 format! 拼接 WHERE 子句
- **THEN** 工單記為 `- [CRITICAL] crates/x/src/db.rs — Standards: 違反專案審查規則「所有 SQL 一律用參數化查詢，不得用字串拼接」：第 42 行以 format! 拼接 WHERE 子句`

#### Scenario: 規則只進 Standards 軸

- **WHEN** 檢視生成的 speclink-review 技能檔關於 discovery 簡報的段落
- **THEN** 段落明定 Standards 簡報依嚴重度分組帶入專案規則原文、Correctness 簡報不帶入、嚴重度由規則所在清單決定且不得另行升降級

#### Scenario: 讀不到設定即停止

- **WHEN** openspec/config.yaml 的 review 區塊含未知子鍵，執行 speclink-review
- **THEN** 技能原樣回報 workflow-config show 的錯誤並停止，不呼叫 review scope、不派出 sub-agent、不寫工單

#### Scenario: 續輪不因規則新增 finding

- **WHEN** Round 1 後使用者在 review.critical 新增一條規則，接著跑 validation 輪
- **THEN** validation 只判定前輪 finding 是否已解與 remediation patch 的直接 regression，不依新規則回報新 finding

<!-- @trace
source: review-rules-config
updated: 2026-10-14T16:37:17+08:00
-->

---
### Requirement: 接受 CRITICAL 必修項的逐條確認

使用者於任何接受途徑（修正後重審時選擇不修某些必修項而以 `(accepted)` 帶入、選擇接受現狀蓋章、或依推薦明示 review stamp --accept）所接受的必修項含 CRITICAL 級時，技能 SHALL 先逐條列出這些 CRITICAL finding 的原文，並以互動工具要求使用者明確確認一次（互動工具不可用時以純文字詢問並等待回覆）；確認後才記錄 `(accepted)` 或執行 --accept 蓋章。使用者不確認時 SHALL 回到原本的選擇節點，SHALL NOT 記錄 `(accepted)`、SHALL NOT 蓋章。所接受的必修項只含 WARNING 級時，流程 SHALL 與現行相同、不追加確認。本需求不改變引擎的蓋章守門：CRITICAL 與 WARNING 照舊都擋乾淨蓋章、都可經 --accept 放行。

#### Scenario: 接受項含 CRITICAL 時追加確認

- **WHEN** 某輪必修項為一筆 CRITICAL 與一筆 WARNING，使用者選擇接受現狀蓋章
- **THEN** 技能先列出該筆 CRITICAL 原文並詢問是否確定接受；使用者確認後才執行 review stamp --accept

#### Scenario: 不確認則回到選擇

- **WHEN** 技能列出待接受的 CRITICAL 並詢問確認，使用者回答不接受
- **THEN** 技能回到三選項節點，不記錄 `(accepted)`、不執行 review stamp，工單不變

#### Scenario: 只接受 WARNING 時不追加確認

- **WHEN** 某輪必修項只有兩筆 WARNING，使用者選擇接受現狀蓋章
- **THEN** 技能照現行流程執行 review stamp --accept，不追加確認

<!-- @trace
source: review-rules-config
updated: 2026-10-14T16:37:17+08:00
-->

---
### Requirement: 審查呈現標明套用的規則

技能於並列呈現兩軸結果時，SHALL 在開頭先印一行，寫明本次 Standards 軸套用的預設清單模式與三個專案規則清單的條數；baseline 為 replace 且三個清單皆空時，該行 SHALL 寫明 Standards 軸只依 repo 文件審查。該行 SHALL 依「審查產出的語言綁定」以解析後的 locale 撰寫（tw、ja、en；未設定時為英文），extend、replace 與 CRITICAL、WARNING、SUGGESTION 等 token 維持英文。該行只出現在呈現中，SHALL NOT 寫入工單。

#### Scenario: extend 加專案規則

- **WHEN** review 為 baseline 未設定、critical 兩條、warning 一條、suggestion 一條，locale 為 tw，執行 speclink-review 的 discovery
- **THEN** 呈現開頭一行寫明預設 12 條（extend）加上專案規則 CRITICAL 2、WARNING 1、SUGGESTION 1；工單不含此行

##### Example: 各 locale 的開頭行

| locale | review 設定 | 開頭行 |
| ------ | ----------- | ------ |
| tw | extend、2／1／1 | 審查規則：預設 12 條（extend）＋專案規則 CRITICAL 2、WARNING 1、SUGGESTION 1 |
| ja | extend、清單皆空 | 審査ルール：既定の 12 項目（extend）、プロジェクトルールなし |
| en | replace、清單皆空 | Review rules: replace, no project rules — the Standards axis judges against repo documents only |
| 未設定 | replace、critical 1 | Review rules: replace, project rules CRITICAL 1, WARNING 0, SUGGESTION 0 |

#### Scenario: replace 且無規則時寫明只依 repo 文件

- **WHEN** review 為 baseline replace、三個清單皆空，執行 speclink-review
- **THEN** 呈現開頭一行寫明未套用預設 12 條、沒有專案規則、Standards 軸只依 repo 文件審查

<!-- @trace
source: review-rules-config
updated: 2026-10-14T16:37:17+08:00
-->

---
### Requirement: 審查子代理簡報的查找紀律

技能派出的 Standards 與 Correctness 子代理簡報 SHALL 遵守下列查找紀律：

- 鎖定 patch 在 25 KB 以內時 SHALL 直接貼入簡報；超過時 SHALL 只給檔案路徑與 hunk 的行號範圍。
- 自動產生的檔案（golden snapshot、assets.lock、由 `speclink update` 再生的技能檔）SHALL 只給路徑，並指示子代理對它們最多搜尋一次。
- SHALL 指示子代理不得整份讀取大檔（以行號範圍讀取），不得做沒有指定目標的遞迴搜尋（只搜尋簡報列出的函式名、型別名與檔案）。
- discovery SHALL NOT 設工具呼叫次數上限；validation SHALL 指示工具呼叫上限 5 次。
- 簡報結尾 SHALL 指示「沒查完就直說，不要猜」；子代理回報未查完的部分 SHALL 原樣列入結果呈現，由使用者決定是否補查。
- finding 指向鎖定 patch 以外的呼叫端時，SHALL 錨定在 patch 內被呼叫的檔案，路徑 SHALL NOT 帶行號（配合 review-station「finding 路徑落在工單範圍內」）。
- 於 quality 時序中，簡報照本條組成，由編排方與驗證站的簡報於同一則訊息派出。

#### Scenario: 小差異直接貼入簡報

- **WHEN** discovery 的鎖定 patch 為 18 KB
- **THEN** 兩軸簡報都直接含該 patch 全文，並列出要查的函式名，不要求子代理自行讀取 patch 檔

#### Scenario: 大差異只給範圍

- **WHEN** discovery 的鎖定 patch 為 60 KB，其中 40 KB 是 golden snapshot
- **THEN** 簡報只給各檔路徑與 hunk 行號範圍，golden snapshot 只給路徑並註明最多搜尋一次

#### Scenario: 複驗輪限五次工具呼叫

- **WHEN** validation 輪派出子代理
- **THEN** 簡報載明工具呼叫上限 5 次與「沒查完就直說，不要猜」

#### Scenario: 未查完的部分照實呈現

- **WHEN** Correctness 子代理回報有兩個呼叫端未查完
- **THEN** 結果呈現原樣列出未查完的兩個呼叫端，不把它們當成已確認無問題

<!-- @trace
source: quality-gate-speedup
updated: 2026-10-14T21:31:03+08:00
-->

---
### Requirement: quality 時序轉交的 CRITICAL 確認

於 quality 時序中，編排方已於停下處逐條列出 CRITICAL finding 原文並取得使用者明確確認，且呼叫本站時明示「CRITICAL 已逐條確認」者，技能 SHALL 以該轉交作為「接受 CRITICAL 必修項的逐條確認」所要求的確認，SHALL NOT 再次詢問。轉交所列的 CRITICAL 少於本站本次要接受的 CRITICAL 時，技能 SHALL 只對未列的部分依該條逐條確認。單站直接呼叫時行為不變。

#### Scenario: 轉交後不再詢問

- **WHEN** quality 收尾補蓋呼叫本站並明示「CRITICAL 已逐條確認」，工單待接受的 CRITICAL 只有轉交所列的那一筆
- **THEN** 技能直接執行帶 `--accept` 的 review stamp，不再詢問使用者

#### Scenario: 轉交不完整時補問未列者

- **WHEN** 轉交只列一筆 CRITICAL，而工單待接受的 CRITICAL 有兩筆
- **THEN** 技能只對未列的那一筆列出原文並要求確認，確認後才蓋帶保留章

<!-- @trace
source: quality-gate-speedup
updated: 2026-10-14T21:31:03+08:00
-->
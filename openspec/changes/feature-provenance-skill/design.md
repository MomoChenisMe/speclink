## Context

溯源鏈的每一環已存在於 repo 內的 metadata：正典規格 openspec/specs/<capability>/spec.md 的 @trace 註解（source＝最後一次動該 Requirement 的 change 名）、封存 change 目錄的 .openspec.yaml（from_discussion）、討論記錄 frontmatter（promoted_to 扇出清單）、封存 change 目錄的 .evidence.json（逐 task 的觸及檔案，2026-08 起的封存才有）。現況無任何動詞或技能組裝這條鏈；舊 @trace 曾附 code 檔案清單但受平行 session 污染，已在 evidence-home-and-trace-slim 後停用。

分層現況：Store trait（crates/speclink-core/src/store.rs）已提供 change／討論／規格的讀取把手；路徑正典在 crates/speclink-fs/src/layout.rs（specs_dir、archive_dir、discussions_dir 等）；CLI 動詞經 crates/speclink-core/src/command/mod.rs dispatch。

## Goals / Non-Goals

**Goals:**

- 引擎動詞 speclink trace <capability>：一次組裝該 capability 的完整封存演進鏈，人讀與 --json 雙輸出。
- 產品技能 /speclink-trace：自然語言問題 → 敘事答案，兩種靜默降級（evidence 缺失走 git 反查、查無規格走 codebase 考古）與 live code 收尾。
- openspec/LANGUAGE.md 補「溯源」詞條。

**Non-Goals:**

- desktop 溯源面板（--json 為其鋪路，面板不在此 change）。
- server／remote 端 trace API、node-sdk 綁定——trace 為 local 動詞；remote_verb_parity 測試是逐動詞情境測試、無列舉閘門，local-only 不需豁免登記。
- 回填舊封存 evidence、清理舊 @trace code 清單——presence check 降級取代資料遷移。
- 引擎內建 git 考古——歸技能層。

## Decisions

**D1：鏈的列舉以封存目錄為主、@trace 為歸屬註記。** 「動過 capability X 的 change 集合」由封存目錄的 specs/<capability>/ 子目錄存在性列舉（完整歷史）；@trace source 另行提供「每條 Requirement 目前定稿出自哪個 change」的歸屬。捨棄「只讀 @trace source」——@trace 每塊只記最後一手，中間演進會漏。

**D2：只組封存鏈，進行中 change 不進 trace 輸出。** 進行中 change 的 delta 尚未折入正典，其資訊由技能的 canon pass（speclink list 已列出）自然帶出；trace 輸出混入未定案內容會讓「怎麼來的」與「正要變成怎樣」混在一起。

**D3：evidence 為逐 change 的存在性偵測（presence check）。** .evidence.json 存在→輸出逐 task 的 files；不存在→該 change 的 evidence 欄位為 null。絕不回讀舊 @trace 的 code 清單（污染已證實）、絕不以日期分界描述行為——null 是機器可讀的降級訊號，由技能層接手 git 反查。

**D4：crate 落點——組裝純函式歸 core、路徑與讀取歸 fs、呈現歸 CLI。** 鏈組裝為 crates/speclink-core/src/trace.rs 的純函式：輸入是 Store trait 讀出的資料結構，輸出 TraceReport，不含 ANSI、不假設儲存媒介。Store trait 若缺把手（列舉封存 change 目錄名、讀 .evidence.json 原文、讀討論 frontmatter），在 store.rs 增讀取方法並由 fs adapter 實作。CLI 端 crates/speclink-cli/src/verbs/trace.rs 只做參數解析與渲染，dispatch 走 command/mod.rs 既有慣例（參照 capability-naming-guard 的 newcmd 前例）。唯一實作落點在 core，無平行實作。

**D5：技能與引擎的契約是 --json 的 evidence null。** 技能資產 crates/speclink-core/assets/skills/trace.md 依 skills.rs 既有註冊慣例加入資產集（連動 MARKER_VERSION、tests/golden/assets.lock 與 golden 再生）。技能流程對使用者輸出永遠是同一種附來源路徑的敘事答案；「降級中」「舊時期」等內部管線字眼不得出現在答案文案。

**D6：capability 不存在時的錯誤依循 naming guard 的近似建議慣例。** trace 對不存在的 capability 報錯並列出至多三個近似名（重用 capname 排序邏輯），供技能的 canon pass 失敗時回頭修正對應。

## Implementation Contract

**行為（引擎）**：speclink trace <capability> 在含正典規格的 repo 內輸出該 capability 的演進鏈——依封存日期由舊至新列出動過它的封存 change，每個 change 帶：封存目錄名、from_discussion（無則 null）、evidence（.evidence.json 的逐 task files；檔案不存在則 null）；來源討論帶：slug、live／archived 位置、promoted_to 全清單，以及每個兄弟 change 觸及的 capability 名集合；另列每條 Requirement 的現行 @trace source 歸屬。人讀輸出為縮排樹；--json 輸出穩定形狀（camelCase）：

```
{ "capability": "...",
  "requirements": [ { "name": "...", "source": "<change名>" } ],
  "changes": [ { "name": "...", "archivedDir": "<日期-名>", "fromDiscussion": "<slug>|null",
                  "evidence": [ { "taskId": "...", "files": ["..."] } ] | null } ],
  "discussions": [ { "slug": "...", "archived": true|false,
                     "promotedTo": [ { "change": "...", "capabilities": ["..."] } ] } ] }
```

**失敗模式**：capability 無正典規格→非零 exit，stderr 帶近似名建議（至多三筆），--json 不輸出成功 payload；封存 change 的 .openspec.yaml 缺 from_discussion→該欄 null（靜默）；.evidence.json 缺→evidence null（靜默）；討論檔已封存→照樣讀出（archived: true）。單一 change 的欄位缺漏不使整體失敗。

**行為（技能）**：/speclink-trace <自然語言問題> 先以 canon pass 對應 capability；命中→呼叫 speclink trace --json 取鏈，讀來源討論（speclink discuss show）的結論與 rounds、各 change 的 proposal Why，evidence 有值讀其檔案、null 則以 git log 反查（commit 訊息帶 change 名的慣例）取觸及檔案，最終讀 live code 確認現況後輸出敘事答案（決策、被否方案、關聯規格、來源路徑）；未命中→git log／blame 考古後同格式作答。降級全程靜默。

**驗收**：cargo test -p speclink-core trace 綠燈（組裝純函式單元測試：D1 並集、D2 排除進行中、D3 null 語意、D6 近似建議）；cargo test -p speclink-cli --test it trace 綠燈（整合：人讀輸出、--json 形狀、無規格報錯、evidence null 路徑）；cargo test -p speclink-core --test it render_golden 綠燈（資產三連動）；speclink update 後 .claude/skills/speclink-trace/SKILL.md 存在；手動：對真實 capability 跑一次 /speclink-trace 並認可敘事品質。

**範圍邊界**：in scope＝上述動詞、技能資產、LANGUAGE.md 詞條、對應測試；out of scope＝Non-Goals 全項、既有 @trace 格式變動、desktop 與 server 任何端點。

## Risks / Trade-offs

- **@trace source 指向的 change 找不到封存目錄**（手動改名等髒資料）：該歸屬照列、changes 清單缺其明細——鏈寬容組裝，單環髒資料不整鏈失敗。
- **演進鏈可能很長**（熱門 capability 數十個 change）：v1 不做截斷與篩選旗標，人讀輸出全列；若實際過長由技能自行摘要，引擎篩選旗標留待 desktop 面板需求時再加。
- **舊 commit 反查非引擎保證**：git 慣例（scope 帶 change 名）是本 repo 實況，其它專案未必成立——技能文字把 git 反查寫成「盡力線索」而非保證，找不到就以討論／提案內容作答。

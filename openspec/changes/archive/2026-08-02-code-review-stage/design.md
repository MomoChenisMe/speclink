## Context

speclink 現行品質站僅 verify（spec 合規三維度），工藝品質無檢查點。討論 code-review-stage（8 輪）已定案審查站設計。實作依附的既有機制：

- **生命週期站前例**：`started_*` 欄位組（crates/speclink-core/src/model.rs 的 ChangeMeta，`#[serde(default)]`、缺席＝未開工）
- **round 文件前例**：discuss 動詞家族——core 函式收 `&dyn Store`（crates/speclink-core/src/discuss.rs 的 add_round），remote 模式由 store 抽象承接
- **archive fail-closed gate 前例**：crates/speclink-core/src/archive.rs 於搬移前守門
- **desktop 增列前例**：speclink-desktop-core（apps/desktop/core）的 query 層在 CLI listing 形狀之上以 meta 增列 desktop 專屬欄位（boardRank），CLI `--json` 形狀不動——parity pin 的實作路徑
- **skill 正典化前例**：crates/speclink-core/src/skills.rs 逐 skill 模板＋crates/speclink-core/tests/golden 對照
- **雜湊依賴**：sha2 已是 speclink-core 依賴，指紋不需新外部依賴
- **touched 記錄**：`.speclink/touched/<change>.json`（gitignored、本機工作狀態）——初審範圍來源，不跨機器

## Goals / Non-Goals

**Goals:**

- 審查站端到端可用：CLI 動詞 → 工單 → 章（雙錨）→ desktop 標示 → skill → 生成文件
- 機制以「站別常數參數化」實作，後續 verify 站接入時只補常數組與動詞註冊，不重寫生命週期
- 本地與 remote 模式行為一致（動詞面唯一寫入路徑）

**Non-Goals:**

- verify 站實例（後續變更）
- QualityStation trait／generic 抽象——單一實例期不建抽象層，第二實例到來時再提升共通碼
- server-web 凍結度計算、範圍外新檔指紋、per-finding 追蹤（proposal Non-Goals 已列）

## Decisions

### D1 品質站機制落點

speclink-core 新 module，常數參數化、不建 trait。

`crates/speclink-core/src/review.rs` 承載工單生命週期與蓋章。站別差異（工單檔名 `review.md`、meta 欄位前綴 `reviewed_`、狀態詞）集中為一組常數，函式簽名不含站別泛型。

- **替代案**：立刻定義 QualityStation trait 供 review／verify 兩實例——否決：目前僅一個實例，抽象層無第二個消費者（YAGNI）；verify 變更接入時再從具體碼提升，屆時有兩個實例可校準介面。

### D2 工單文件與儲存

change 目錄下的動詞驅動 markdown 文件。

- 位置 `openspec/changes/<name>/review.md`，透過 `&dyn Store` 讀寫（與 discuss 動詞同型）——本地隨 git、remote 走 store 文件管道，朝「storage 解耦的規格驅動引擎」靠攏：core 不知道媒介。
- 固定骨架：標題行＋`## Round N` 區段；每輪含 `**Scope**:`（本輪審查檔案清單，repo-root 相對路徑）與 findings 行（`- [severity] path — 描述`，severity ∈ CRITICAL／WARNING／SUGGESTION）。格式由動詞產生與驗證，skill 文案明訂禁手寫。
- 工單是 sidecar：不註冊進 workflow schema，status／validate 白名單制天然忽略（status.rs 以 schema.artifact(id) 查詢）。
- **替代案**：結構化 JSON 存 `.speclink/`——否決（討論已 ruled out）：gitignored 不跨機器，違背交接需求。

### D3 章與雙錨

stamp 單一 unit_of_work，失效判定為讀取端純函式。

- ChangeMeta 新欄位（全部 `#[serde(default)]`，缺席＝未審查，pre-migration 檔案可讀——向後相容）：`reviewed_at`／`reviewed_by`／`reviewed_with`（比照 started_*）、`reviewed_tasks_total`（蓋章時任務總數）、`reviewed_scope`（清單項 `{ path, hash }`；path 為 repo-root 相對、`/` 分隔——Windows 路徑正規化後寫入；hash 為檔案內容 sha256，行尾 CRLF→LF 正規化後計算，避免 git autocrlf 環境誤降級）。
- stamp 流程：守門（任務全完成＋工單末輪零未解 findings，`--accept` 跳過後者）→ 計算指紋（scope＝工單各輪 Scope 聯集；聯集中已不存在於工作樹的檔跳過不入錨——修正可刪除／改名早輪審過的檔，死檔不得永久卡死蓋章；全數消失則拒絕；存在但非 UTF-8 仍 fail-closed）→ 寫 meta＋刪工單於同一 unit_of_work——不留「章已寫、工單還在」的半套狀態。
- 失效判定：core 提供純函式（輸入 meta、當前任務統計、檔案讀取閉包；輸出 fresh／stale／unknown），desktop-core 呼叫（有工作樹）；CLI 不輸出。
- **替代案**：git HEAD／commit 錨——否決（討論已 ruled out）：蓋章時工作樹 dirty、無關 commit 誤報；指紋不依賴 git 也符合 drift 從 WorkspaceFacts 快照運作的先例。

### D4 CLI 子命令面

`review add-round <change> --mode? --stdin`、`review show <change> [--json]`、`review stamp <change> [--accept]`、`review discard <change>`。`--json` 欄位 camelCase（`rounds`、`lastRound`、`findings`、`severity`、`scope`）——對外契約。exit code：成功 0；守門不過／格式不符 → 非零＋原因（人眼與 stderr 一致）。

- **替代案**：單一 `review` 指令以旗標分流——否決：與 `discuss add-round`／`task done` 的動詞分隔慣例不一致。

### D4a server review 動詞端點（remote 承載）

實作揭露原「不新增 server API」前提不成立：既有文件管道承載不了 review 動詞——`put_artifact` 白名單只收 proposal／design／tasks／specs、artifacts 路由無 DELETE、meta 無泛寫端點、`import` 為遷移專用。依 discuss 動詞家族先例補 server 動詞端點，引擎面加 `Command::Review*` 變體（薄包裝既有 `core::review` 函式，server 經 `verb::run` 跑 BridgeStore 自動獲得原子 commit、事件通知與 ETag）：

- `GET /changes/{name}/review` → show（回應鏡射 CLI `--json` shape：camelCase `rounds`／`lastRound`／`findings`）
- `POST /changes/{name}/review/rounds` → add-round（body：`content`）
- `POST /changes/{name}/review/stamp` → stamp（body：`accept`、`agent?`、`scope: [{path, hash}]`、`missing?: [path]`）
- `DELETE /changes/{name}/review` → discard

指紋歸屬：內容指紋只有工作樹持有者能算——remote 模式由 CLI 讀本地 checkout 計算 scope 雜湊隨 stamp 請求上 wire；checkout 內已不存在的聯集檔由 CLI 以 `missing` 清單明示宣告（server 無工作樹、無從驗證存在性，宣告與雜湊同屬提交端權威）；server 端驗證分割「提交 path 集合 ∪ missing ＝工單各輪 Scope 聯集且不相交」，不成立即拒（工單在讀取後被追加輪次的 CAS 式保護），不重算不信任內容；`missing` 缺席讀作空清單，舊 client 即原嚴格相等。`reviewed_by` 於 server 端取 binding actor（與既有寫入動詞一致）；核心 `stamp` 拆出 scope 注入變體（`stamp_with_scope`），本地閉包路徑行為不變。

- **替代案**：擴 `put_artifact` 白名單＋補 artifact DELETE＋meta 泛寫端點——否決：把動詞語意攤平成文件操作，繞過守門（蓋章守門、原子性）且 meta 泛寫破壞 server 對 metadata 的唯一寫入權威。
- **替代案**：server 端重算指紋——否決：server 無工作樹，被審的是 checkout 內容不是 store 文件。

### D5 archive 未結工單守門

archive.rs 既有 fail-closed gate 前加檢查：偵測 review.md 存在 → 預設拒絕，訊息列三處置（`review stamp`／`review discard`／`--carry-review` 明示帶走）。`--carry-review` 時工單隨目錄搬移，成為封存側「曾審查未通過」標示的證據。無工單時行為零變化。

- **替代案**：自動 discard 工單再封存——否決：靜默銷毀「未通過」證據，與使用者裁定的三選項語意不符。

### D6 desktop 資料流與 UI

- speclink-desktop-core 的 query 層增列 `reviewStatus` 欄位（camelCase 對外契約）：active＝`none`／`inReview`（工單存在）／`reviewed`／`reviewedStale`；archived＝`reviewed`／`reviewedNotPassed`（化石工單無章）。凍結度以 D3 純函式在 desktop-core 計算（不依賴 Tauri、可獨立 cargo test）；Tauri command 維持單行委派。
- UI（packages/ui）：ChangeCard 行內小章（lucide 既有 icon 家族＋Tooltip，維持極簡卡片——不加文字列）；RichDetailDrawer 審查資訊列（狀態詞＋時間＋審查者）；ArchivedList／ArchivedDrawer 同狀態機；封存入口偵測 `inReview` 彈三選項對話框。文案用 LANGUAGE.md 詞彙：審查中／已審查／已審查·其後有變動／曾審查未通過。
- **替代案**：前端直接讀 .openspec.yaml——否決：remote 模式讀不到、繞過協定契約。

### D7 skill 與生成文件

- skills.rs 新增 `/speclink-review` 正典模板（claude／codex 雙工具）：主線 orchestrator，流程＝選 change（比照 verify 選擇邏輯）→ 守門自檢（未全完成即停并說明）→ 定範圍（續輪＝`review show --json` 末輪 findings 檔集；末輪零 findings＝以各輪 Scope 聯集定界（去除死檔、不重掃全 change）；初審＝touched 檔集；無 touched＝詢問 git 基準）→ 讀 artifacts 當判準脈絡 → 平行兩個 read-only sub-agent（Standards：repo 慣例文件＋smell baseline 正典原文（D7a）、repo 優先；Correctness：bug 獵捕），各報告 400 字內、分級三檔 → 並列呈現不合併不重排＋一行總結 → `review add-round` → 有 findings 以 AskUserQuestion 三選項（codex 變體以純文字詢問）→ 修正回主線 → 空輪 `review stamp`。
- Standards sub-agent 指示內嵌 smell baseline 正典原文（討論 review-skill-smell-baseline 定案）：引言、兩條約束規則（The repo overrides／Always a judgement call，含 skip anything tooling already enforces）、"(Refactoring, ch.3)" 出處、12 條 smells 逐項（what it is → how to fix），專有名詞與原文逐字不動；模板附一行出處註記（Matt Pocock skills repo，MIT）。severity 對應：smells 以 "possible X" 措辭、落 WARNING／SUGGESTION；CRITICAL 留給文件化標準明確違反與 Correctness 軸的 bug。正典文字見 D7a。
- instructions.rs：workflow 行改為 `discuss? → propose → apply ⇄ ingest → (review? ∥ verify?) → archive`、技能清單加審查站；golden 於乾淨樹再生。
- **替代案**：skill 用 `context: fork`＋Explore（比照 verify）——否決：fork 內不能 fan-out sub-agent、不能互動詢問。
- **替代案**：smell baseline 以「Fowler smells 基線」一句帶過、由模板作者自編——否決（討論 review-skill-smell-baseline）：無逐項內容的基線等於憑印象自編，照抄正典原文一次釘死。

### D7a Smell baseline 正典文字

自 Source doc（Matt Pocock skills repo，MIT）逐字轉錄；5.1 實作時整段照抄進 Standards sub-agent 指示，不改寫、不翻譯：

```markdown
On top of whatever the repo documents, the Standards axis always carries the smell baseline below — a fixed set of Fowler code smells (Refactoring, ch.3) that applies even when a repo documents nothing. Two rules bind it:

- **The repo overrides.** A documented repo standard always wins; where it endorses something the baseline would flag, suppress the smell.
- **Always a judgement call.** Each smell is a labelled heuristic ("possible Feature Envy"), never a hard violation — and, like any standard here, skip anything tooling already enforces.

Each smell reads what it is → how to fix; match it against the diff:

- **Mysterious Name** — a function, variable, or type whose name doesn't reveal what it does or holds. → rename it; if no honest name comes, the design's murky.
- **Duplicated Code** — the same logic shape appears in more than one hunk or file in the change. → extract the shared shape, call it from both.
- **Feature Envy** — a method that reaches into another object's data more than its own. → move the method onto the data it envies.
- **Data Clumps** — the same few fields or params keep travelling together (a type wanting to be born). → bundle them into one type, pass that.
- **Primitive Obsession** — a primitive or string standing in for a domain concept that deserves its own type. → give the concept its own small type.
- **Repeated Switches** — the same switch/if-cascade on the same type recurs across the change. → replace with polymorphism, or one map both sites share.
- **Shotgun Surgery** — one logical change forces scattered edits across many files in the diff. → gather what changes together into one module.
- **Divergent Change** — one file or module is edited for several unrelated reasons. → split so each module changes for one reason.
- **Speculative Generality** — abstraction, parameters, or hooks added for needs the spec doesn't have. → delete it; inline back until a real need shows.
- **Message Chains** — long a.b().c().d() navigation the caller shouldn't depend on. → hide the walk behind one method on the first object.
- **Middle Man** — a class or function that mostly just delegates onward. → cut it, call the real target direct.
- **Refused Bequest** — a subclass or implementer that ignores or overrides most of what it inherits. → drop the inheritance, use composition.
```

### D7b 裁量層與收斂機制

本 change 自身工單的三輪實審暴露迴圈不收斂的三個成因：續輪新 sub-agent 對同批檔案挖出新的 possible-X 實例（棘輪效應）、修復本身擴大審查面、修復未經全量驗證引入回歸（跨 crate 編譯失敗直到下一輪才被發現）。技能模板補三個機制，引擎面零改動（stamp 守門與 `--accept` 語意不變）：

- **裁量分類**：兩軸呈現後每筆 finding 標必修（CRITICAL／現實路徑 bug／文件化標準明確違反）或可裁（possible-X 與 SUGGESTION，附一行成本效益裁量）；三選項詢問帶推薦——有必修推薦「修正後重審」並列必修清單、僅剩可裁推薦「接受現狀蓋章」。
- **驗證門**：選「修正後重審」時，修正完成後、下一輪 sub-agent 派出前，跑專案完整建置與測試且全綠——修復引入的回歸不得流入下一輪。
- **接受前饋**：已接受未修正的 findings 由主線原樣帶入續輪記錄並以 `(accepted)` 結構性標記收尾（比照 severity 標籤維持英文；末輪工單忠實反映保留事項、蓋章走 `--accept`），續輪 sub-agent 指示附不重報清單阻斷重複發現；跨 session 接手以標記行重建清單。

- **替代案**：迴圈至零 findings 才蓋章——否決：possible-X 為判斷題，對同批檔案理論上永不歸零；本 change 實測三輪共 38 筆 findings 仍持續產出新實例。
- **替代案**：per-finding resolve 動詞與逐項處置狀態——否決（proposal Non-Goals 已列）：工單格式與動詞面不動，處置狀態由技能層承載即可。

### D7c 產出語言與 locale 綁定

- 缺口（討論 review-ticket-locale 定案）：模板原僅把 locale 綁在「呈現給使用者」的內容，sub-agent 回報契約與 `add-round` 記錄無語言約束——下游專案設 locale: tw 仍得到英文工單；本 repo 工單為中文屬全域 zh-tw 指引的意外副作用，非契約保證。
- 綁定三處，仿 verify 模板既有綁定句式（報告散文走 locale、severity 標籤／指令行／code 參照留英文、未設定則英文）：(1) 守門自檢保留 payload 時聲明 locale 適用整條產出鏈；(2) 兩軸 sub-agent 指示攜帶解析後 locale，finding 描述以該語言撰寫；(3) 並列呈現與 `add-round` 記錄與 sub-agent 產出同語言、主線不翻譯——原「render verbatim」與「write in the resolved locale」的矛盾隨之消解（verbatim 渲染的已是 locale 語言產出）。引擎零改動：`add-round` 文法只約束 `- [SEVERITY] path — description` 行形，描述語言自由。
- **替代案**：本地化工單骨架與 severity 標籤——否決：骨架為 parse_round 逐行驗證的動詞文法，需動 parser＋golden＋既有工單相容性，且違反 LANGUAGE.md「結構標記不在詞彙範圍」慣例。
- **替代案**：只綁 `add-round`、sub-agent 維持英文由主線翻譯——否決：逐輪手工翻譯把漂移寫進永久記錄，且與「render verbatim」直接衝突。

## Implementation Contract

**In scope**：speclink-core（review.rs、model.rs、archive.rs、command 分派、skills.rs、instructions.rs）、speclink-cli（review 子命令，本地與 remote 兩路徑）、speclink-protocol（review DTOs）、speclink-server（D4a 動詞端點）、speclink-remote（client 方法）、speclink-desktop-core（query 增列與凍結度）、packages/ui 四元件＋adapter、apps/desktop i18n、golden、README。
**Out of scope**：verify 站、server-web console、desktop remote 卡片的 review 標示（remote list 項目不帶 reviewStatus，UI 自然降級）。

可驗證行為：

1. `review add-round` 對不存在的 change 非零 exit；對合法輸入建立／追加 `## Round N`，`review show --json` 的 `lastRound` 反映之
2. `review stamp` 於任務未全完成或末輪有未解 findings 時非零 exit 並說明；`--accept` 放行後者；成功時 meta 五欄位齊備且 review.md 不存在（同一 unit_of_work）
3. 指紋：蓋章後修改任一 scope 檔內容 → 失效純函式回 stale；未修改 → fresh；CRLF 差異不觸發 stale
4. CLI `speclink list --json` 在 meta 帶 reviewed_* 時輸出形狀不變（parity pin 測試延伸）
5. desktop query：四態 `reviewStatus` 各有測試 fixture；archived 側 `reviewedNotPassed` 由化石工單觸發
6. archive：有工單預設拒絕且訊息含三處置；`--carry-review` 放行且工單隨目錄搬移；無工單行為與現行完全相同
7. golden：skills 與 instructions 模板再生後對照通過；`/speclink-review` 於 claude 與 codex 兩工具皆生成，且兩份生成檔皆含 D7a 全段（12 條 smells 專有名詞與兩條約束規則逐字在場）
8. UI：卡片小章與抽屜資訊列依 `reviewStatus` 渲染四態；封存入口於 `inReview` 時彈三選項
9. server（D4a）：四路由整合測試走完整迴圈（add-round → GET → stamp 拒 → accept 蓋章 → meta 帶章且工單刪除）；stamp 提交的 scope path 集合與工單聯集不等時拒絕；remote CLI 於 server 不可達時非零 exit 且 stderr 為連線錯誤（spec「remote 模式下的動詞行為」）
10. 裁量層（D7b）：claude 與 codex 兩份生成技能檔皆含裁量分類（必修／可裁判準與三選項推薦）、驗證門（下一輪前專案建置與測試全綠）、接受前饋（不重報清單＋主線帶入續輪記錄）三段內容（關鍵詞檢核），且 D7a smell baseline 全段仍逐字在場
11. locale 綁定（D7c）：claude 與 codex 兩份生成技能檔皆含三處綁定內容——sub-agent 指示攜帶 locale 且 finding 描述以該語言撰寫、呈現與 `add-round` 記錄同語言不翻譯、severity 標籤與 `Standards:`／`Correctness:` 前綴留英文、locale 未設定則全英文（關鍵詞檢核）

## Risks / Trade-offs

- **回歸對照（最優先）**：CLI listing parity pin——延伸 listing.rs 既有測試：meta 帶全套 reviewed_* 時 `list --json` 必須與無欄位時序列化同形。
- **跨平台**：指紋的路徑分隔（Windows `\`→`/` 正規化後入 meta）與行尾（CRLF→LF 正規化後雜湊）；Windows CI 既有測試矩陣涵蓋。
- **golden 再生髒樹污染**：再生一律於乾淨樹執行（既有紀律）。
- **平行 session 工單互踩**：append-only＋動詞化縮小衝突面；殘餘風險由既有 commit 前重盤 git status 紀律承接。
- **skill token 成本**：兩 sub-agent 各 400 字報告上限；續輪範圍收斂到上輪 findings 檔集，重審不重掃全 change。

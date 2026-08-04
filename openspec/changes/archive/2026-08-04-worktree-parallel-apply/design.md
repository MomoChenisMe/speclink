## Context

討論 worktree-parallel-apply 定案：並行 apply 以 git worktree 隔離、desktop 維持單一專案身分、主看板即時反映各 worktree 進度。本刀（第一刀）落地引擎與技能面；desktop 呈現屬第二刀。既有結構的關鍵事實：

- `crates/speclink-core/src/listing.rs` 的 changes_json 是 list `--json` 的唯一組裝點（CLI、server、Node SDK parity 共用），欄位序凍結、加法欄位以可空＋缺席不序列化維持位元級相容。
- core 只透過 Store trait（`crates/speclink-core/src/store.rs`）讀 change 資料，不碰儲存媒介；本地 git 事實的先例歸 Host（`crates/speclink-host/src/drift.rs` 明文「Local git/worktree facts are a client (Host) responsibility」）。
- 工作流政策四層解析在 `crates/speclink-core/src/config.rs`（WorkflowPolicyFields ＋ SPECLINK_* 環境覆寫）；CLI 寫入動詞為 workflow-config set（`crates/speclink-cli/src/commands.rs` 的正典鍵清單）；wire 是整份文件（protocol 的 ConfigResponse／PutConfigRequest 走 content 字串＋revision），逐欄位不上 wire。
- 技能生成一技能一模板（`crates/speclink-core/src/skills.rs` registry ＋ assets/skills/*.md 經 include_str! 內嵌），apply 模板已以「If tdd: true is set」句式於執行期消費政策。

## Goals / Non-Goals

**Goals:**

- 有活躍 worktree 的 change，在主資料夾的 list（人眼與 `--json`）即時反映 worktree 內的任務進度與開工狀態。
- config.yaml 新增 worktree 政策欄位，走既有四層解析與 workflow-config 動詞。
- 兩個技能（apply-with-worktree、worktree-merge）以生成期組裝落地，claude 與 codex 都生成。
- worktree 的 discovery 零新增持久化儲存：git 名冊＋分支命名慣例即登記簿。

**Non-Goals:**

- desktop 端一切（watch 擴充、卡片標示、抽屜資訊、GUI toggle）——第二刀 worktree-live-board。
- remote workspace 的聚合讀（TeamStore 集中、天生即時）。
- 並行時機自動偵測、每次 apply 都開 worktree、worktree 名冊持久化、desktop merge 按鈕。
- archive／snapshots 流程不動（archive 永遠在主 checkout 執行）。
- list 以外動詞（status、show、drift 等）的 overlay——本刀只保證看板觀察面。

## Decisions

**D1 — discovery 與映射：分支命名慣例，git 是唯一登記簿。**
worktree 由技能以分支 speclink/<change名> 建立於 sibling 巢 <repo資料夾名>.worktrees/<change名>/。discovery 於 Host 執行 git worktree list --porcelain，解析每個 linked worktree 的絕對路徑與分支；分支名剝去 speclink/ 前綴後與主 workspace 的活躍 change 名比對，三條件同時成立才建立映射：(a) 分支符合慣例、(b) 同名 change 存在於主 workspace 且未封存、(c) 該 worktree 路徑下 openspec/changes/<change名>/ 可讀。任一不成立則靜默略過、回讀主副本（fail-open 到既有行為）。detached HEAD 與 prunable（資料夾已刪）條目一律略過。替代案「顯式登記簿檔案」否決：多一份要維護一致性的狀態，而 git 名冊本身已權威。

**D2 — overlay 落點：Host 的 Store 裝飾器＋listing 的事實參數。**
新模組 `crates/speclink-host/src/worktree.rs` 提供兩件事：(1) WorktreeFacts——discovery 結果（change名 → 絕對路徑＋分支）；(2) 一個包住既有 Store 的裝飾器，對映射中的 change 將 read_artifact／read_change_meta／find_change／updated_at_secs 等讀取重導向到 worktree 副本（openspec/changes/<change名>/ 為根），其餘 change 與所有寫入原樣透傳（寫入永遠落主副本——本刀 overlay 僅供讀取觀察）。core 的 changes_json 維持原簽名不動；新增一個帶 facts 的組裝變體，facts 缺席（空表）時輸出與既有 changes_json 位元級相同——server 與 remote parity 呼叫端不改、零介入。替代案「core 直接跑 git」否決：違反 core 不碰媒介的邊界與 drift 先例。

**D3 — overlay 只在主 worktree 的 local workspace 生效。**
觸發條件：workspace 為 local 且其根目錄的 .git 是目錄（主 checkout）。在 linked worktree 內執行 list（.git 為檔案）不做 overlay——避免遞迴視角混亂；remote workspace 不做 overlay。CLI 的 local list 路徑組裝時：讀 workflow 政策，worktree 欄位解析為 true 才執行 discovery 與包裝，false／缺席時完全不 spawn git——未啟用者的 list 行為與效能零變化。

**D4 — config 欄位與動詞：worktree 為第五個政策欄位，比照 tdd／audit。**
`crates/speclink-core/src/config.rs`：WorkflowPolicyFields 增 worktree（可空布林，缺席＝false）；環境覆寫增 SPECLINK_WORKTREE（沿用既有布林解析）；`.speclink.yaml` 不受理此鍵（不進 deprecated 相容清單——新欄位沒有歷史檔案要相容）。CLI workflow-config set 的正典鍵清單增 worktree、show（人眼與 `--json`）同步呈現。wire 零改動（整份文件流通）。

**D5 — list 輸出契約：加法、缺席即位元級不變。**
`--json` 的 change 條目新增可空欄位 worktree：物件 { "path": 絕對路徑字串, "branch": 分支名字串 }（camelCase；serde skip 缺席）。人眼輸出對映射中的 change 於既有行尾追加一段固定標示「 [worktree]」（無色彩依賴，`--no-color` 同字面）。任務計數、status、開工戳記等既有欄位不改名不改型——它們的值因 Store 重導向自然反映 worktree 副本。位元級相容以測試釘死而非 golden 檔：list 的人眼輸出由 CLI 直接 println 產出，`crates/speclink-core/tests/golden` 只收技能渲染快照，沒有 list 人眼 fixture 家族可加。三顆釘子分別是 core 的「facts 空表＝與既有組裝位元級相同」、CLI 的「政策關閉時人眼與 `--json` 皆與無 worktree 情境逐位元一致」、CLI 的「標示字面釘死且 `--no-color` 同字面」；既有技能渲染 golden 全數不動。

**D6 — 技能：編譯期組合保持 body 為靜態字串。**
apply-with-worktree 的 body 以 concat!(include_str!(前置段), include_str!(apply.md), include_str!(收尾段)) 於編譯期拼成——維持 registry 的 body: &'static str 型別不動、apply 本體單一來源（同一份 assets/skills/apply.md）。新資產：apply-worktree-pre.md（讀政策拒跑、preflight、建 worktree、成本提示、進入 worktree 的 cwd 約定）與 apply-worktree-post.md（在 worktree 內 commit、停在 merge 前並指向 worktree-merge 技能）；worktree-merge.md 為獨立完整模板。registry 新增兩條：apply-with-worktree（fork: false、disallow_edit: false、for_codex: true）、worktree-merge（fork: false、disallow_edit: true——僅執行 git 指令、衝突即停不代編、for_codex: true）；skill_body 查表同步。CLAUDE.md／AGENTS.md 注入區塊的技能使用時機清單增兩條（隨既有 skills 再生機制落地）。替代案「執行期跨技能委派」已於討論否決。

**D7 — 詞彙：「worktree」直出記為明文例外。**
`openspec/LANGUAGE.md` 原則區新增明文例外條目：worktree 得直出於使用者可見文案（含第二刀的卡片標示與抽屜資訊），理由與先例（config.yaml 頁簽、討論 slug——開發者工具中 git 使用者的原生心智模型）、裁定出處（本討論）與日期一併記載。

## Implementation Contract

**行為（完成後可觀察）：**

- 於主資料夾啟用政策（workflow-config set worktree true）並存在分支 speclink/<change名> 的 linked worktree 時：speclink list 的該 change 行尾出現「 [worktree]」標示；list `--json` 該條目出現 worktree 物件欄位；completedTasks／status 反映 worktree 副本的即時內容（worktree 內勾一個 task，主資料夾重跑 list 即見計數 +1）。
- worktree 移除（git worktree remove）後重跑 list：標示與欄位消失、數值回讀主副本——無任何殘留狀態。
- 政策關閉（欄位缺席或 false）時：list 不執行 discovery，輸出與本刀落地前位元級相同。
- workflow-config show 呈現 worktree 欄位；SPECLINK_WORKTREE=true 覆寫檔案值；workflow-config set worktree 接受 true／false，非法值的錯誤行為與既有政策鍵一致（stderr 訊息＋非零 exit code）。
- speclink skills 再生後（claude 工具）出現 speclink-apply-with-worktree 與 speclink-worktree-merge 兩個 SKILL.md；codex 目標亦生成對應產物；apply-with-worktree 的內文包含完整 apply 本體流程（非引用）。

**介面／資料形狀：**

- list `--json` change 條目新欄位：worktree（可空物件；path 字串＝worktree 絕對路徑、branch 字串＝分支全名；缺席時不序列化）。
- WorktreeFacts：change名 → { path, branch } 的唯讀映射，由 Host discovery 產生。
- Store 裝飾器：實作既有 Store trait，建構子收（內層 Store、WorktreeFacts、各 worktree 的 spec 根路徑）；讀取方法對映射中 change 重導向，寫入方法一律透傳主副本。
- config：WorkflowPolicyFields.worktree（Option<bool>）；環境變數 SPECLINK_WORKTREE；workflow-config set 鍵名 worktree。
- 技能檔名慣例沿既有 registry（生成目錄名 speclink-<name>）：apply-with-worktree、worktree-merge。

**失敗模式：**

- git 不可用或 git worktree list 失敗：discovery 回空表、list 照常輸出（無標示無欄位）——觀察面永不因 git 失敗而報錯。
- 映射三條件任一不成立（分支不合慣例、change 不存在或已封存、worktree 內 spec 目錄不可讀）：該條目靜默略過、回讀主副本。
- worktree 副本內單一 change 的 artifact 損壞：與主副本同款的既有 fail-closed 診斷（metaError 欄位）如實呈現，不因 overlay 而降級。
- worktree-merge 技能遇衝突：停止並回報衝突檔案，不自行代編、不 commit 半套 merge。

**驗收（如何確認契約成立）：**

- cargo test -p speclink-core：facts 空表時 changes_json 變體輸出與既有 changes_json 位元級相同的斷言；有 facts 時 worktree 欄位形狀（camelCase、path／branch 存在）與計數重導向的單元測試。
- speclink-host 單元測試：porcelain 輸出解析（正常、detached、prunable）、三條件映射判定、Store 裝飾器重導向與透傳。
- crates/speclink-cli/tests/ 整合測試：真實 git repo ＋ git worktree add 的端到端情境——啟用政策後 list 人眼含「 [worktree]」、`--json` 含欄位；移除 worktree 後輸出還原；政策關閉時位元級不變。跨平台：路徑比對不得假設分隔符（Windows 絕對路徑）。
- render_golden：既有技能渲染 golden 全數不動（list 人眼輸出無 golden 家族，其位元級相容由上述 CLI 測試釘住）。
- skills 生成測試：兩個新技能的渲染輸出釘 golden；apply-with-worktree 內文包含 apply 本體的既有標記段落（斷言組合完整）。
- workflow-config 的 CLI 測試：set worktree true → show 與 `--json` 呈現；非法值報錯。

**範圍邊界：**

- In：core（listing 變體、config 欄位）、host（worktree 模組）、cli（list 組裝、workflow-config 鍵、測試）、skills（兩模板＋registry＋注入清單）、openspec/LANGUAGE.md。
- Out：desktop 全部、server 端點行為（ConfigResponse 等 wire 不動；server 的 changes_json 呼叫端走原簽名零介入）、Node SDK 新表面（parity fixture 僅隨欄位缺席情境維持不變）、list 以外動詞的 overlay、archive 流程。

## Risks / Trade-offs

- **git spawn 成本**：GUI 進程 spawn git 極慢的環境已有先例（desktop 以預熱緩解）。本刀只影響 CLI 的 list 路徑（政策開啟時每次多一個 git worktree list，毫秒級可接受；關閉時零成本）；第二刀的 desktop 輪詢面必須自帶快取／去抖，契約已由「facts 作為參數注入」預留。
- **回歸保護面**：list 人眼輸出無 golden 家族（它在 CLI 直接 println），保護落在 CLI 整合測試；以「空 facts 位元級不變」與「政策關閉時兩路輸出逐位元一致」把無關情境的回歸風險釘死。
- **分支撞名**：使用者自建 speclink/<非change名> 分支不會誤映射（條件 b 擋下）；同名但指向他 repo 的 worktree 不可能出現在本 repo 的 git 名冊。
- **技能組合的可讀性**：apply 本體被前後段包夾，模板銜接處的語氣與步驟編號需在資產內文處理（前置段結尾明示「以下為 apply 本體流程，於 worktree 內執行」）；golden 釘住渲染結果防走樣。
- **跨平台**：porcelain 路徑為絕對路徑，Windows 大小寫與分隔符差異由路徑正規化處理；整合測試覆蓋 Windows CI。

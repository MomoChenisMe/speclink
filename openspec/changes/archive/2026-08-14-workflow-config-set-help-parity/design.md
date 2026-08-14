## Context

`workflow-config set` 接受的政策鍵有五個，由 `crates/speclink-cli/src/verbs/config.rs` 的 `POLICY_KEYS` 常數宣告（註解自稱「in canonical order」），未知鍵的錯誤訊息直接 join 這個常數印出，因此錯誤訊息一直是對的。同一支子指令的 clap 說明文字則是另一份手寫的 doc comment，只列四個鍵，是這次要修的錯誤資訊。第二處同性質的漏列在 `<VALUE>` 參數說明：它寫「tdd/audit 收 true 或 false」，漏了同為布林的 worktree。

正典 `openspec/specs/workflow-config/spec.md` 自身也自相矛盾，而且不只一處：「工作流政策的正典歸屬與四層解析順序」需求明列五欄且規定 set SHALL 接受 worktree 鍵，但另外三條需求仍停在四欄的舊世界，全都是同一批漏更新：

1. **「workflow-config set 政策欄位寫入」**：寫「key SHALL 限 locale、spec_locale、tdd、audit 四者」，缺鍵插入的正典序也只列四鍵——實際程式碼（`crates/speclink-core/src/config.rs` 的 desired 陣列）已是五鍵序。這條就是過時 help 字面的來源。
2. **「workflow-config show 動詞」**：寫「政策四欄」，`--json` payload 欄位清單只列 locale、specLocale、tdd、audit、context、rules。實測人眼輸出已有 worktree 一列、`--json` 的鍵序已是 locale、specLocale、tdd、audit、worktree、context、rules。更明顯的自相矛盾是：同一份 spec 的 worktree 專屬需求寫著「workflow-config show 的人眼與 --json 輸出形狀不變（worktree 欄位既已存在，此處為維持既有輸出的相容性聲明）」——它預設 show 需求已經寫了 worktree，但 show 需求根本沒寫。
3. **「init 範本的政策寫入位置」**：寫「locale、spec_locale、tdd、audit 的註解示例區」，scenario 的 THEN 也只列四鍵。實測在乾淨 temp repo 跑 speclink init 產出的範本已含 worktree 註解示例，覆寫提示行也已列出 SPECLINK_WORKTREE。

三處都是正典落後於實作，方向一致（少一個 worktree），必須同批收——分批修等於在同一份 spec 上開平行 change，版號行會對撞。

**變更落點與邊界**：程式碼改動全部落在 `speclink-cli`。這是純粹的 CLI 呈現層修正——clap 的說明文字在 argv 解析層產生，早於模式解析，`speclink-core` 的政策語意、`speclink-host` 的裁決、以及 wire contract 一律不動。`workflow-config` 雖是 Dual 動詞（fs／remote 雙模式），但 help 由 clap 在模式解析前一次產生、兩模式共用同一份，不存在雙路徑平行實作的問題，也不牽動 `crates/speclink-cli/tests/it/remote_verb_parity.rs` 的對照面。

## Goals / Non-Goals

**Goals:**

- `speclink workflow-config set --help` 列出的政策鍵與實際接受的鍵集合逐字一致（五個）。
- `<VALUE>` 參數說明涵蓋全部只收 true/false 的鍵（tdd、audit、worktree）。
- 讓「help 少列一個鍵」這類漂移在結構上不可能重演，或至少有紅燈測試把它擋在 CI。
- 正典 workflow-config 的三條落後需求（set、show、init 範本）與同 spec 的其他需求、與實際行為三者對齊。
- 正典新宣告的每一段行為都有測試載體——沒有測試守門的正典等於沒有契約。

**Non-Goals:**

- 不改 `set worktree` 的任何執行行為（技能足跡同步、關閉時的活躍 worktree 擋下）。
- 不把 worktree 的專屬行為敘述寫進 --help（見 D3）。
- 不改任何 `--json` 欄位名或 payload shape。
- 不動 `docs/configuration.md` 與 `docs/configuration.zh-TW.md`：兩份文件的鍵清單已是正確的五個。
- 不把 KEY 參數 enum 化或掛 clap `value_parser` 白名單：那會讓 clap 接管未知鍵的報錯，改變既有錯誤訊息與 exit path，屬行為變更。
- 不動 `new.rs` 與 `query.rs`（掃描結論見 D2）。
- 不改 show 與 init 的任何實際輸出：這兩處是正典追上實作，不是實作追上正典（見 D6）。
- 不動 `openspec/specs/config-skill/spec.md` 的「政策四欄逐項詢問」（判定見 D5）。

## Decisions

### D1: help 說明由 POLICY_KEYS 生成

**採用**：子指令說明改由 `POLICY_KEYS` 在執行期組出字串，掛在 clap 的 `#[command(about = ...)]` 屬性，取代手寫 doc comment。

clap 的 doc comment 是編譯期字面，無法插值；但 clap 4 的 derive 屬性接受任意表達式，本 crate 已有現成先例——`crates/speclink-cli/src/main.rs` 的頂層 `#[command(...)]` 就用 `version = VERSION.as_str()`，而 `VERSION` 是一個 `std::sync::LazyLock<String>`。本決策沿用同一個模式：宣告一個 `LazyLock<String>` 常數，內容由 `POLICY_KEYS` join 而成，屬性寫 `about = <該常數>.as_str()`。代價是一個 static 與一次啟動期字串配置，沒有新相依、沒有新抽象層，且與同 crate 既有寫法一致，不是為此發明的機制。

**替代方案（不採用）**：(a) 維持手寫字面、只補一個字——治標，同一個 bug 會在下次增刪政策鍵時重演，且完全沒回答「兩個真相來源」；(b) 用 `long_about` 另寫一段——只是多一份手寫字面，讓漂移面變大；(c) 把 `POLICY_KEYS` 提升到 `speclink-core` 由多方共用——目前只有這支 CLI 消費它，跨 crate 搬遷屬未被要求的彈性。

`<VALUE>` 參數說明**不做動態生成**：它需要的是「布林鍵子集」，而程式裡沒有這個常數（`policy_bool` 的三個呼叫點是唯一事實）。只為一行 help 立一個 `POLICY_BOOL_KEYS` 常數，等於再造一個可能與呼叫點脫節的真相來源，弊大於利。改為修正字面，並由 D4 的測試把它釘死。

### D2: 同類漂移掃描結論

掃描分兩面：程式碼面（`crates/speclink-cli/src/verbs/config.rs` 與相鄰動詞檔的手寫枚舉）與正典面（`openspec/specs/workflow-config/spec.md` 全檔的政策鍵敘述）。

**程式碼面**

- **config.rs 的 set 子指令說明**（少 worktree）與 **`<VALUE>` 參數說明**（少 worktree）：確認漂移，納入範圍。
- **config.rs 的 show 子指令說明**：寫的是「政策欄位、context、rules」三類，不逐一枚舉欄位名，因此沒有可漂移的清單，**不需改**。show 的實際輸出（人眼與 `--json`）已經印五欄。
- **config.rs 的 rules 參數說明**：以「proposal, design, specs, tasks, ...」帶省略號的示例呈現，實際接受值由 schema 在執行期決定，語意上本就是示例而非窮舉，**不需改**。
- **`crates/speclink-cli/src/verbs/new.rs` 的 artifact TYPE 說明**：列 proposal、design、tasks、spec，與同檔未知型別的錯誤訊息所列完全相同——同樣是兩份手寫字面，但**目前一致，沒有漂移**，依「只做被要求的事」不納入範圍。
- **`crates/speclink-cli/src/verbs/query.rs` 的 --sort 說明**：列 name、modified、created，與 `crates/speclink-core/src/listing.rs` 的排序分支一致，**沒有漂移**。

因此本變更的程式碼面就是 config.rs 的兩行說明加一個 static，不擴散。

**正典面**（`openspec/specs/workflow-config/spec.md` 全檔逐條複驗）

- **set 需求、show 需求、init 範本需求**：三條都停在四欄，實作皆已五欄。**確認漂移，全部納入範圍**（實測證據見 Context）。
- **舊政策鍵的 deprecation 警告需求**：只列 locale、spec_locale、tdd、audit 四鍵——**這是正確的，不需改**。同一份 spec 的解析順序需求已明文規定 worktree 無歷史舊鍵、寫在 `.speclink.yaml` 不生效且不產生警告，因此警告需求列四鍵正是刻意的，補上 worktree 反而會把正確的正典改錯。
- **worktree 專屬需求（技能足跡同步與關閉擋下）**：本來就寫五欄世界的行為，不需改。

判別準則：凡是**枚舉政策鍵集合**的敘述才要補 worktree；凡是敘述**只適用於舊鍵相容層**的，維持四鍵。這條準則同時解釋了為什麼 deprecation 警告不動。

### D3: worktree 專屬行為不進 --help

正典規定 `set worktree` 寫入成功後會同步技能足跡，且由 true 改 false 時若存在活躍 linked worktree 會拒絕寫入。**這些不寫進 --help**，理由三點：

1. **擋下發生時本來就會自己講清楚**：現行拒絕訊息會逐列每個活躍 worktree 的 change 名、分支與路徑，並指示先用 worktree-merge 收尾。使用者在真正撞到的那一刻拿到的資訊，比事前在 help 掃到一句摘要更有用。
2. **每多一句手寫敘述就多一個漂移面**——那正是本變更要消滅的 bug 類型。行為敘述與正典的距離越長，越容易在下次行為調整時被遺忘。
3. **--help 的職責是機械契約**（有哪些鍵、收哪些值、有哪些旗標、exit code 語意），敘事性行為說明屬於 `docs/configuration.md` 與正典 spec。

**取捨代價**：使用者要到實際執行才會知道關閉可能被擋。接受此代價，因為擋下是 fail-closed 且訊息自足，不會造成資料損失或難以理解的失敗。

### D4: 紅燈測試落在既有的 workflow_config.rs

測試加在 `crates/speclink-cli/tests/it/workflow_config.rs`，**不另開檔案**：該檔是這支動詞的既有測試面（已涵蓋 set 的鍵驗證、值驗證、dry-run、remote 寫入路徑），而同 crate 的 `crates/speclink-cli/tests/it/archive_evidence_gate.rs` 已有對子指令 help 輸出做斷言的先例，放這裡沒有新慣例成本。

測試的關鍵設計是**不硬寫第五份清單**（正典、常數、help、錯誤訊息之外）：整合測試在別的 crate、拿不到私有的 `POLICY_KEYS`，若在測試裡再抄一份清單，只是把漂移搬到測試檔。改為從同一支 binary 取兩份輸出互相對照——set --help 印的鍵集合，對上未知鍵錯誤訊息（Use one of: ...）印的鍵集合——斷言兩者逐字相同。任何人日後增刪 `POLICY_KEYS` 而忘了讓 help 跟上，這條測試就紅。

TDD 順序：先寫這條測試看它紅（因為 help 現在少 worktree），再改 help 讓它綠。

### D5: config-skill 的政策逐項詢問不納入本案

`openspec/specs/config-skill/spec.md` 規定 speclink-config 技能「政策四欄（locale、spec_locale、tdd、audit）SHALL 逐項詢問使用者、不由技能推斷」。表面上這也是「四欄」，但**判定為不納入**——它不是漂移，是一個還沒被做過的設計判斷。

理由三點：

1. **worktree 是流程開關，不是 artifact 生成政策**。另外四欄影響的是 AI 產出物的語言與紀律（artifact 寫哪國語言、要不要 TDD、要不要 audit），speclink-config 技能的職責正是從 codebase 推導這類生成政策。worktree 決定的是「你要不要用平行 worktree 跑 apply」，屬於開發者的工作方式偏好，codebase 裡沒有任何訊號可以推導它。
2. **它牽動技能足跡，代問的風險不對稱**。答「要」會實際生成兩顆 worktree 技能檔、答「不要」會刪掉它們；由技能在一連串設定問題中夾帶這一題，使用者容易順手答錯，而錯的那一邊會改動檔案系統。另外四欄答錯只是改一行 config。
3. **本案的性質是「正典追上實作」**，把 worktree 加進技能問答則是**改變技能行為**——那需要改 `crates/speclink-core/assets/skills/config.md` 的內文，會直接觸發 `MARKER_VERSION` 進版、golden 重生、`assets.lock` 重生三連動，成本與風險跟本案完全不是一個量級，混進來會讓一個純文字校正變成產物層 BREAKING。

**替代處置**：這是一個獨立的產品判斷，適合另開討論（speclink-config 該不該問 worktree），不適合塞進一個 help 校正案。本案明確不動 config-skill 正典與 config.md 資產。

### D6: 兩處新增正典校正的測試載體判定

新納入的兩條需求性質相同（正典文字校正，實作不動），但**測試載體的處置不同**，依「這段行為現在有沒有人守門」判定：

- **show 需求：已有載體，不新增測試。** `crates/speclink-cli/tests/it/workflow_config.rs` 的 `show_prints_canonical_policy_context_and_rules` 已斷言人眼輸出有 worktree 一列並顯示未設定與預設關閉；`show_json_payload_is_camel_case_with_null_for_unset` 已斷言 payload 的 worktree 為 false。正典補上 worktree 之後，這兩條既有測試就是它的載體，再加一條只是重複。
- **init 範本需求：沒有載體，新增一條。** 全 repo 搜尋確認**沒有任何測試斷言 init 產出的 config.yaml 範本註解區內容**——`speclink init` 生成的範本雖然已含 worktree 與 SPECLINK_WORKTREE，卻沒有人守門，改壞不會有測試變紅。正典既然要明文宣告範本內容，就得同時給它載體，否則是「測不到的邊界＝沒有契約」。新測試落在 `crates/speclink-cli/tests/it/init_tools.rs`（該檔已有跑 speclink init 的隔離 temp 環境與 HOME 隔離），斷言生成的 openspec/config.yaml 含五個政策鍵註解示例與五個 SPECLINK_* 覆寫名、且 .speclink.yaml 不含政策鍵。

**誠實標註**：這條 init 測試對現行實作**一寫就是綠的**，它不是 TDD 的紅燈，而是回歸釘樁（characterization test）——本案唯一的紅燈在 D4 的 help 對照測試。為確認釘樁真的有效（而非恆綠的假測試），實作時要做一次一次性變異檢查：暫時拿掉範本裡的 worktree 註解行，確認測試轉紅，再還原。

## Implementation Contract

**Behavior（使用者觀察到什麼）**

- 執行 speclink workflow-config set --help：子指令說明行列出全部五個政策鍵，順序與 `POLICY_KEYS` 宣告序相同（locale、spec_locale、tdd、audit、worktree）。
- 同一份輸出的 `<VALUE>` 參數說明指出 tdd、audit、worktree 三者收 true 或 false。
- 執行 speclink workflow-config --help：子指令一覽中 set 那一列的說明同步更新（clap 共用同一個 about）。
- 其餘一切不變：子指令名、參數位置、旗標（--dry-run、--no-color）、成功與失敗的 exit code、成功訊息、diff 內容、未知鍵與非法值的錯誤訊息文字。
- **show 與 init 的輸出一個位元都不變**：`speclink workflow-config show`（人眼與 `--json`）與 `speclink init` 生成的兩個設定檔範本，內容與現行完全相同。這兩處交付的是「正典文字追上既有行為」與「init 範本從此有測試守門」，不是新行為。

**Interface / data shape**

- 新增一個 crate 內私有的 `LazyLock<String>` static（UPPER_SNAKE_CASE 命名），內容為 set 子指令的說明字串，由 `POLICY_KEYS` join 組成。
- `WorkflowConfigCommands::Set` 變體改以 clap 的 `command(about = ...)` 屬性取得說明，原 doc comment 移除（留著會與屬性重複，且是死掉的第二真相來源）。
- `<VALUE>` 參數的 doc comment 字面補上 worktree。
- `crates/speclink-cli/tests/it/init_tools.rs` 新增一條斷言 init 範本內容的測試（依 D6），不改 `crates/speclink-core/src/init.rs` 的範本字面。
- 無 `--json` 面變動、無 wire contract 變動、無設定欄位變動。

**Failure modes**

- 本變更不新增任何失敗路徑。未知鍵、非法布林值、關閉 worktree 遇活躍 worktree 的三種既有失敗行為（訊息與 exit code）逐字不變。

**Acceptance criteria**

1. `crates/speclink-cli/tests/it/workflow_config.rs` 新增的 help 對照測試通過：help 所列鍵集合 ＝ 未知鍵錯誤訊息所列鍵集合，且 `<VALUE>` 說明涵蓋三個布林鍵。
2. 該測試在改 help 之前先跑出紅燈（TDD 紅→綠的證據）。
3. `crates/speclink-cli` 既有測試全數通過，尤其 workflow_config.rs 的既有案例與 remote_verb_parity.rs。
4. `cargo test -p speclink-core --test it render_golden::` 全綠且 golden 快照**零改動**（見 Risks 的三連動段）。
5. `openspec/specs/workflow-config/spec.md` 的 delta 通過 speclink validate。
6. `crates/speclink-cli/tests/it/init_tools.rs` 新增的範本測試通過，且經過一次變異檢查證明它不是恆綠。
7. `speclink workflow-config show`（人眼與 `--json`）與 `speclink init` 的產出與改動前逐位元相同。

**Scope boundaries**

- 範圍內：config.rs 的 set 說明與 `<VALUE>` 說明、新增的 static、workflow_config.rs 的新測試、init_tools.rs 的新範本測試、workflow-config 正典的三條需求 delta。
- 範圍外：任何執行行為（含 show 與 init 的實際輸出）、其他動詞檔、`crates/speclink-core/src/init.rs` 的範本字面、docs/、技能資產與 `crates/speclink-core/assets/skills/config.md`、`openspec/specs/config-skill/`、`MARKER_VERSION`、golden 快照、`assets.lock`、desktop 與 server-web。

## Risks / Trade-offs

- **[誤觸技能三連動（`MARKER_VERSION` 進版 ＋ golden 重生 ＋ `assets.lock` 重生）]** → 已核實**不會觸發**：以 help 的兩段字面（子指令說明、`<VALUE>` 說明）全 repo 搜尋，命中只有 config.rs 自己，`crates/speclink-core/assets/skills/` 與 `crates/speclink-core/tests/golden/` 底下（含 neutral-cli 與 claude-worktree 兩份快照）皆無引用。快照裡出現的是 skill 內文自己寫的 speclink workflow-config set 指令範例，與 clap help 字面無關；`crates/speclink-core/tests/it/render_golden.rs` 的相關斷言檢查的也是 skill 內文含 set worktree true 這串指令，不碰 help。緩解：實作後跑一次 render_golden 測試並確認 golden 目錄 git status 乾淨——**若 golden 出現差異，代表前述核實有誤，應停下重新評估而非直接重生快照**。
- **[撞上既有凍結測試]** → `crates/speclink-cli/tests/it/workflow_config.rs` 現有測試已核實**沒有任何一條斷言 help 文字**，因此改 help 不會弄紅既有案例；風險主要來自新測試自己寫得太脆（見下一條）。
- **[新測試對 clap 換行與終端寬度過度敏感]** → clap 會依終端寬度折行，逐字比對整段 help 會在不同環境（CI、Windows、窄終端）飄。緩解：測試只從輸出中抓「鍵名是否出現」與「鍵集合是否相等」，不比對整段版面、不比對空白與換行位置。
- **[跨平台]** → 唯一的平台差異是換行（Windows 的 CRLF）與 ANSI 色碼。緩解：測試以既有 TempProject 執行器取 stdout（既有測試已在三平台跑過同一條路徑），比對前 trim，且不依賴顏色（help 本身不上色，`--no-color` 也不影響）。
- **[與平行進行中變更撞檔]** → 進行中的 zh-tw-vocabulary-drawer-and-quality-station 會動技能資產、`MARKER_VERSION`、golden 與 `assets.lock`；本變更完全不碰這四者，spec delta 也落在不同 capability（對方是 ui-copy-vocabulary、worktree-apply-skill、worktree-merge-skill，本案是 workflow-config），**無檔案層或 delta 層重疊**。緩解：不需要協調，但收尾 commit 時仍逐檔盤點 git status，避免夾帶對方的再生產物。
- **[clap 表達式屬性行為與預期不符]** → 若 `about = <static>.as_str()` 在本版 clap 上編譯或渲染不如預期，退路是保留手寫字面、只靠 D4 的測試防漂（治標但可接受）。風險低：同 crate 的 `version = VERSION.as_str()` 已是運行中的同型用法。
- **[正典 delta 改動的是既有需求整塊]** → workflow-config 的三條需求（set、show、init 範本）各以 MODIFIED 整塊取代，需完整保留每條需求原有的其他規定與**全部既有 scenario 名稱一字不改**（set 的值驗證、文字層 read-modify-write、dry-run、remote 版本守衛；show 的環境變數不覆寫、remote 形狀一致、fail-closed；init 的既有專案不受影響）。漏抄或改名等於未宣告刪除，validate 與 analyze 都抓不到，會拖到 archive 才炸。
- **[init 範本測試把實作細節釘太死]** → 若測試逐字比對整份範本，日後任何註解措辭調整都會誤紅。緩解：只斷言五個政策鍵名、五個 SPECLINK_* 名各自出現於註解區，以及 `.speclink.yaml` 不含政策鍵，不比對排版、縮排與說明文字。
- **[init 範本測試是恆綠的假測試]** → 新測試對現行實作一寫就綠（見 D6），若寫錯斷言目標會永遠不紅。緩解：實作時做一次性變異檢查（暫時移除範本的 worktree 註解行確認轉紅後還原），並把結果記在對應 task。
- **[任務數超過 15]** → 併入兩條正典校正後任務數為 17，略高於建議上限，但其中 6 條是驗證與盤點型任務（每條數分鐘），實作型任務只有 3 條。不拆成多個 change 是刻意的：三條需求同屬 `openspec/specs/workflow-config/spec.md`，拆開會造成同一份 spec 的平行 change 與版號行對撞（合併時只能重生衍生物，不能挑邊）。

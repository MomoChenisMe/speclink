## Context

使用者裁定「抽屜」「品質站」不合繁中用語習慣，收斂為「詳情面板」「品質關卡」。

**現況盤點（本變更提案時複驗）**。兩個詞在 repo 內的分佈極度不對稱——絕大多數不是使用者看得到的字：

| 位置 | 抽屜 | 品質站 | 性質 |
| --- | --- | --- | --- |
| `openspec/LANGUAGE.md` | 9 | 4 | 正典詞彙（散落於其他詞條的定義文與兩條明文例外） |
| `openspec/specs/**` | 263 | 35 | 規格散文；其中 41 個 requirement／scenario 標題含「抽屜」 |
| `apps/` 原始碼 | 99 | 9 | 除 3 處 i18n 字串外全為註解與測試名 |
| `packages/` 原始碼 | 171 | 5 | 全為註解與測試名，零字串 |
| `crates/` | 0 | 28 | 註解、技能資產內文、golden 快照 |
| `.claude/skills/`、`.agents/skills/` | 0 | 各 6 | 由技能資產再生的衍生物 |
| `docs/`、兩份 README | 0 | 0 | `user-docs-overhaul` 已收斂 |

**使用者可見面只有 9 處**，全部落在四個檔案：desktop 一則 toast、server-web 兩則首次導覽提示、worktree 兩支技能資產對使用者說的交棒文字。其餘約 570 處中，「抽屜」幾乎都是**元件模式的內部代稱**（描述某個 UI 是抽屜式的），不是畫面上出現的字。

**約束一：技能資產的三連動**。`crates/speclink-core/assets/skills/` 的內文一改，`crates/speclink-core/tests/it/render_golden.rs` 的鎖會擋下——render 輸出變了而 `MARKER_VERSION` 沒動即失敗。必須同批處理進版、golden 重生與 `assets.lock` 重生。

**約束二：規格改名的引擎風險**。41 個 requirement／scenario 標題含「抽屜」。MODIFIED delta 是整塊取代，改動 scenario 名在引擎眼中等同**未宣告的刪除**，`validate` 與 `analyze` 都抓不到，要到 archive 才爆。

**約束三：LANGUAGE.md 有既存裁定**。第 12 行「討論 slug 例外」與第 13 行「worktree 例外」兩條明文例外的內文多次使用「抽屜」，且第 12 行帶四筆範圍擴充紀錄（desktop-card-identity 2026-07-09；desktop-ux-polish 2026-07-11；tray-copy-and-panel-mode 2026-07-16；change-drawer-header-redesign 2026-08-04）。動它的字面等於碰一條既有裁定的紀錄。

## Goals / Non-Goals

**Goals:**

- 使用者看到的每一處舊詞都換成新詞，兩個介面（文件 vs GUI／技能）用詞一致。
- 兩個詞取得正典地位，舊詞列入 `avoid`，未來新文案有依據可循。
- 有自動化守門擋住舊詞回流，不靠人盯。
- 動到既有裁定紀錄時留下明確痕跡，不靜默改寫。

**Non-Goals:**

- 不回改 `openspec/specs/**` 散文、程式碼註解與測試名（D1）。
- 不回改 `openspec/changes/archive/`（D3）。
- 不改英文文案；`LANGUAGE.md` 範圍本就排除英文 CLI 輸出。
- 不改任何識別符（`RichDetailDrawer`、`SpecDrawer`、`archivedDrawerBase`…）、CSS 類名或 `--json` 欄位名。
- 不改任何行為、CLI 旗標或輸出形狀。

## Decisions

### D1：改動邊界收斂到使用者可見文案面，規格散文與程式碼註解不回改

**決定**：本次只改 9 處使用者可見文案 ＋ `LANGUAGE.md` 正典，**不**回改 `openspec/specs/**` 散文（298 處）與程式碼註解／測試名（284 處）。

**理由**。三個層次：

1. **性質不同**。規格與註解裡的「抽屜」是元件模式的內部代稱，不是使用者看到的字；使用者可見文案才是本次裁定的標的。全 repo 改名把「統一內部術語」偷渡成「修正使用者用詞」，兩件事該分開決策。
2. **專案既有原則已經給了答案**。`LANGUAGE.md` 開頭寫明「一個概念一個詞：同義詞在 avoid 列出，**舊文案陸續汰換**；歷史 artifacts 不回改」。漸進汰換是這個專案處理詞彙變更的既定作法，不是本次發明的例外。
3. **全改的成本與風險不成比例**。41 個 requirement／scenario 標題含「抽屜」，整批改名觸發約束二的未宣告刪除陷阱——`validate`／`analyze` 綠燈、archive 才爆，而且爆在一個純文案變更上。單一 change 的 diff 破 580 處、跨 60 餘檔，也會與任何平行 change 大面積對撞。

**替代方案：全 repo 一次改到底**。優點是 repo 內用詞絕對一致，不留兩套詞。否決理由如上第 3 點——收益是內部一致性（沒有使用者感知得到），代價是 archive 期爆炸風險與大面積合併衝突。

**替代方案：連規格散文一起改、只避開標題**。仍要動 298 處、跨 14 個 spec 檔，且「標題留舊詞、內文用新詞」的檔內不一致比兩套詞並存更難讀。否決。

**兩套詞並存怎麼辦**。這是本決定明確接受的取捨，控制手段是 D2 的 `avoid` 詞條加 D4 的守門測試：規格散文用舊詞不影響使用者，但只要舊詞想跨進使用者可見面就會被測試擋下。漂移的風險口被守住了，剩下的是漸進汰換的存量。

### D2：兩個詞立為 LANGUAGE.md 正典詞條，舊詞入 avoid

**決定**：在 `openspec/LANGUAGE.md` 的「詞彙」段新增「詳情面板」與「品質關卡」兩個詞條，各含 definition／avoid／why 與裁定日期 2026-08-14；`avoid` 分別列「抽屜」與「品質站」。同時把散落在既有詞條定義文中的舊詞換成新詞（第 49、61、87、163、165 行等）。

**理由**。這是整個變更裡唯一能防止詞彙漂回去的機制。沒有詞條就沒有 `avoid`，D1 選擇的漸進汰換路線就會變成「改完就漂回來」——正是因為 D1 不做全 repo 改名，D2 才是**必要**而非可選。

**替代方案：只改文案不立詞條**。否決：改完的 9 處沒有任何東西釘住，下一個寫 GUI 文案的代理照樣寫「抽屜」。

### D3：已封存變更與歷史 artifacts 一律不回改

**決定**：`openspec/changes/archive/` 與 `openspec/discussions/archive/` 底下的舊詞完全不動。

**理由**。兩條獨立理由指向同一結論：(a) 封存內容是稽核資料，記錄「當時怎麼決定的」，改寫等同竄改歷史紀錄，`@trace` 溯源也會與當時的實作對不上；(b) `LANGUAGE.md` 原則已明文寫死「歷史 artifacts（已封存的討論／變更）不回改」——這不是本次要重新決定的事，照既有正典執行即可。

**替代方案：一併改寫封存內容以求全域一致**。否決，理由同上。

### D4：以守門測試釘死使用者可見文案面

**決定**：新增 `scripts/vocabulary-guard.test.mjs`，掛進 `package.json` 既有的 `node --test "scripts/**/*.test.mjs"`（`test:all` 已涵蓋），與 `docs-links.test.mjs`、`docs-parity.test.mjs` 同一層。

**守門範圍**（使用者可見文案面）：兩個 i18n 訊息檔、`crates/speclink-core/assets/skills/` 全部 `.md`、`docs/` 全部 `.md`、`README.md` 與 `README.en.md`。

**為何不會誤傷識別符**。守門比對的是 CJK 字串「抽屜」「品質站」；`RichDetailDrawer`、`SpecDrawer`、`archivedDrawerBase` 是純 ASCII，永遠不匹配。這是設計上的保證，不是巧合——本變更不改任何識別符（見 Non-Goals），所以守門不需要任何識別符白名單。

**為何 `openspec/LANGUAGE.md` 必須排除在守門範圍外**。詞條的 `avoid` 行**依設計就要寫出舊詞**（`avoid: 抽屜`），把 LANGUAGE.md 納入守門會讓正典自己違規。排除是正確解，不是漏網。

**為何不守 `openspec/specs/**` 與原始碼**。D1 已決定那裡的存量不回改，納入守門會立刻紅燈。守門的職責是**擋住新的越界**，不是追討存量。

**替代方案：`validate` 加 lint 規則**。否決：`validate` 的職責是 change artifacts 的結構正確性，跨到 repo 全域文案掃描是職責錯置，且引擎層改動遠大於一支測試檔。

**替代方案：只靠人工 grep 收尾**。否決：一次性檢查擋不住之後的回流，等於沒有守門。

### D5：LANGUAGE.md 兩條明文例外只換用詞，裁定內容與紀錄一字不動

**決定**：第 12 行「討論 slug 例外」與第 13 行「worktree 例外」**只**替換其中的「抽屜」字面，裁定語句、適用範圍列舉的其他項目、以及四筆範圍擴充紀錄（含 change 名與日期）全部逐字保留。不重述、不重新裁定、不新增或移除任何範圍項目。並在本變更的收尾以人工確認這兩行除用詞外零差異。

**理由**。這兩條是使用者裁定過的紀錄，載明了裁定理由與四次範圍擴充的來源。本次裁定的標的是「用什麼詞稱呼這個元件」，不是「這條例外還成不成立」——後者從未被討論，靜默連帶改寫會讓一條有效裁定的邊界變得可疑。只換用詞是最小且忠實的動作：例外講的是「討論 slug 得直出於某些位置」，那些位置的名字換了，例外本身沒變。

**替代方案：連裁定一起重述**。否決：等同以文案變更之名重新裁定一條未經討論的既有決策。若日後認為例外範圍該調整，那是另一個變更、另一次使用者裁定。

### D6：替換規則避免「詳情詳情面板」疊字

**決定**：「抽屜」不是無腦全域取代，依下表替換：

| 原字面 | 替換後 |
| --- | --- |
| `詳情抽屜` | `詳情面板` |
| `討論抽屜` | `討論詳情面板` |
| `已封存抽屜` | `已封存詳情面板` |
| 其餘裸用 `抽屜` | `詳情面板` |

「品質站」→「品質關卡」為 1 對 1 直換，無疊字問題。

**理由**。既有文字已大量使用「變更詳情抽屜」「討論抽屜」等複合詞；直接全域取代會產生「變更詳情詳情面板」。此表讓替換結果可預期，也讓 D5 的「只換用詞」有明確判準。

### D7：技能資產改動走既有三連動，不繞過鎖

**決定**：改 `crates/speclink-core/assets/skills/` 內文後，依既有機制依序處理——`crates/speclink-core/src/init.rs` 的 `MARKER_VERSION` 自 `v1.19.12` 進版、以 `UPDATE_GOLDEN=1` 重生 golden 快照、再於乾淨樹以 `UPDATE_ASSETS_LOCK=1` 重生 `assets.lock`。`.claude/skills/` 與 `.agents/skills/` 為再生衍生物，由 `speclink update` 產出，不手改。

**理由**。`render_golden.rs` 的鎖刻意設計成「render 輸出變了但版號沒動」即失敗，且拒絕在版號未動時寫穿 lock。這是既有紀律，本變更照走即可——重點是**不要**試圖手改 golden 或 lock 繞過它。

**crate 邊界**：改動只落在 `speclink-core`（資產內文與 `MARKER_VERSION` 常數）與兩個前端 app 的 i18n 資源；不觸及 `speclink-fs`、store crates、`speclink-host` 的任何邏輯，`speclink-cli`／`speclink-server`／`speclink-node` 零改動。本變更無 local／remote 雙路徑實作問題——技能資產是單一來源，兩端共用同一份再生產物。

## Implementation Contract

**Behavior**（可觀察結果）：

- desktop 在不支援的工作區處置品質關卡工單時，toast 顯示「此工作區不支援品質關卡工單處置」（原「品質站」）。英文語系文案不變。
- server-web 首次導覽的兩則提示（使用者頁導覽、列表主要動作導覽）以「詳情面板」描述右側滑出的面板（原「抽屜」）。
- `speclink update` 再生後，`speclink-apply-with-worktree` 與 `speclink-worktree-merge` 兩支技能對使用者說的交棒文字以「品質關卡」稱呼 review ∥ verify（原「品質站」），claude 與 codex 兩種產出皆然。
- `openspec/LANGUAGE.md` 含「詳情面板」與「品質關卡」兩個詞條，各自的 `avoid` 列出對應舊詞。
- 詞彙守門測試在使用者可見文案面出現「抽屜」或「品質站」時失敗，並指出違規檔案與行。

**Interface / data shape**：無新增或變更的 CLI 子指令、旗標、`--json` 欄位或 IPC 契約。i18n 訊息鍵名（`store.reviewActionUnsupported`、`tour.navUsers.hint`、`tour.listPrimary.hint`）全部維持不變，只換值。`MARKER_VERSION` 字串值改變，其型別與用法不變。

**Failure modes**：

- 守門測試失敗時以非零 exit code 結束，訊息載明違規檔案路徑、行號與命中的舊詞——響亮失敗，不靜默通過。
- 資產改了但 `MARKER_VERSION` 未進版時，`render_golden` 既有斷言失敗並在訊息中給出修法；本變更不新增也不弱化這個行為。
- 既有工作區未執行 `speclink update` 時維持舊文案，不報錯（純文案，無相容性斷裂）。

**Acceptance criteria**：

- `node --test "scripts/**/*.test.mjs"` 通過，且新守門測試確實會在植入舊詞時失敗（測試本身需涵蓋正反兩面）。
- `cargo test -p speclink-core --test it render_golden::` 通過（golden 與 lock 已同批重生）。
- `npm test -w apps/desktop`、`npm test -w apps/server-web`、`npm test -w packages/ui` 通過。
- `./target/debug/speclink validate zh-tw-vocabulary-drawer-and-quality-station` 回報 valid。
- 人工確認 `openspec/LANGUAGE.md` 第 12、13 行除「抽屜」→「詳情面板」外與變更前逐字相同（D5）。
- 人工確認 `git status` 未遺漏 `speclink update` 再生的受管檔。

**Scope boundaries**：

- **In scope**：`openspec/LANGUAGE.md`；`apps/desktop/src/i18n/messages.ts` 與 `apps/server-web/src/i18n/messages.ts` 的 zh-TW 區塊；`crates/speclink-core/assets/skills/apply-worktree-post.md` 與 `worktree-merge.md`；`crates/speclink-core/src/init.rs` 的 `MARKER_VERSION`；golden 快照與 `assets.lock`；`speclink update` 再生的 `.claude/skills/`、`.agents/skills/`；新守門測試；`worktree-apply-skill` 與 `worktree-merge-skill` 兩份 delta spec。
- **Out of scope**：`openspec/specs/**` 其餘散文；全部程式碼註解與測試名；`openspec/changes/archive/`；英文文案；任何識別符、CSS 類名與 `--json` 欄位；任何行為變更。

## Risks / Trade-offs

- **[repo 內兩套詞並存]** → D1 明確接受的取捨。控制手段是 D2 的 `avoid` 詞條與 D4 的守門測試：存量不影響使用者，新的越界會被擋下。
- **[回歸對照：golden 與 assets.lock 刻意變動]** → 依 D7 的既有三連動處理，先進版再重生，不手改鎖。變動已在提案的「相容性影響」記載，審查時可辨識為刻意而非意外。
- **[`speclink update` 再生大量受管檔，收尾易漏]** → 驗收條件明列以 `git status` 盤點再生產物；提交前重盤一次。
- **[D5 的逐字保留靠人工把關]** → 除人工確認外，改動限縮在單一詞彙替換（D6 的替換表給出明確判準），`git diff` 上這兩行應只有詞彙差異，超出即為越界。
- **[跨平台]** → 本變更只動文字內容與一支 Node 測試，無路徑分隔、換行或 git 行為相依。守門測試以 UTF-8 讀檔並比對 CJK 字串，Windows／macOS／Linux 行為一致；比對不依賴行尾字元。
- **[守門範圍選擇可能被誤讀為漏網]** → D4 已寫明 `LANGUAGE.md` 與規格散文的排除理由，避免後續審查誤判為疏漏而擴大範圍。

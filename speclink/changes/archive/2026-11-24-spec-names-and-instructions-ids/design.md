## Context

`speclink instructions <產出物> --change <名稱> --json` 的回應由 speclink-core 的 `build_artifact` 產生（`crates/engine/speclink-core/src/workspace/instructions.rs`），找不到代號的錯誤由同一個 crate 的 `run_instructions` 產生（`crates/engine/speclink-core/src/command/mod.rs`）。本機 CLI、Server（遠端連線的 CLI 向它要回應）與 Node SDK 的 `dispatch` 都走這兩個函式，所以指示與錯誤訊息各只有一個實作落點。

討論 embedded-host-needs-9 查到的現況：

- specs 的指示最後有一段語言說明：「Write spec prose in <語言>. Structural markers … stay in English …」。它沒有說 `### Requirement:` 與 `#### Scenario:` 後面的名稱。propose、ingest 技能講 `locale` 的段落也只講標記。模型把一份 ADDED 的 delta 規格改成規格語言時，把 12 個名稱都翻掉，回答也沒提。
- 指示前面的「MODIFIED requirements workflow」已經有一句「Ensure header text matches exactly」，但它只講 MODIFIED 的需求名稱，而且離語言說明很遠；REMOVED、RENAMED 與情境名稱沒有對應的句子。
- 名稱對不上正式規格時，`speclink validate` 的合併守門已經在封存前擋下（MODIFIED、REMOVED、RENAMED 的名稱，以及 MODIFIED 區塊漏掉的正式規格情境），和 `--strict` 無關。所以這次不需要新的檢查，缺的只是事前的指示。
- 這個 repo 在 `spec_locale: tw` 下，870 條需求名稱與 3735 個情境名稱全部含中文：名稱跟著規格語言是既有的慣例。
- delta 規格在三個動詞有三種寫法：`speclink instructions specs`、`speclink artifact cat specs/<capability>`、`speclink new artifact spec <capability>`。ingest 技能用同一個 `<artifact-id>` 寫前兩個動詞；propose 技能叫模型用 `speclink artifact cat <artifact-id>` 讀前置產出物，而任務清單的前置產出物代號是 `specs`，`artifact cat specs` 會被拒絕。
- `artifact cat` 的錯誤會列出可用的寫法；`instructions` 的錯誤只有 `Artifact '<代號>' not found in schema`。

限制：技能原稿一改，內嵌資產版號、golden 快照與 `assets.lock` 必須同批更新；技能原稿的全大寫強調字有登記上限（`crates/engine/speclink-core/tests/it/skill_authoring.rs`），不能增加；propose 與 ingest 的固定用語由 `crates/engine/speclink-core/tests/it/skill_verbization.rs` 守門。

## Goals / Non-Goals

**Goals:**

- specs 的指示自己說清楚名稱的規則：新寫的名稱跟著規格語言，指向正式規格的名稱照原文。
- 完整的名稱規則只在引擎說一次；技能只指向它。
- ingest 改到既有名稱時，使用者在摘要看得到。
- 技能把 delta 規格的三種寫法分清楚；寫錯代號時，`instructions` 的錯誤訊息讓呼叫者一次就能改對。
- 本機、遠端連線、內嵌宿主得到同一份指示與同一句錯誤，各只有一個實作落點。

**Non-Goals:**

- 不讓 `speclink instructions` 接受 `specs/<capability>` 或 `spec` 當成 `specs`（討論已排除：兩種寫法並存，會讓人以為每個能力各有一份指示）。
- 不新增檢查，也不改 `speclink validate` 的錯誤訊息；「漏掉情境時建議宣告 REMOVED-SCENARIO」的句子不動（討論的延後項目）。
- 不改 specs 的 schema 原稿（`crates/engine/speclink-core/assets/schema/spec-driven/fork.schema.yaml`），所以 `speclink schema fork` 的輸出逐位元不變。
- 不在回應新增欄位；`locale` 的值不變。
- 不改 baseline 技能：它直接寫正式規格，沒有指向正式規格的名稱要保留。
- 不改 apply、verify、archive、commit、config 技能：討論已查過，它們沒有混用代號。
- ADDED 的名稱改掉後，引用舊名稱的任務會讓 `speclink analyze` 出 Coverage 警告，這次不處理（討論的延後項目）。
- 不處理 validate 錯誤結尾的「see: speclink drift」指向（第八輪留下的延後項目）。

## Decisions

### D1 名稱規則寫進引擎的規格語言說明

`build_artifact` 對 `specs` 產出物加的那一段語言說明，在「… so the analyzer and archive tooling can parse them.」之後、中文模糊用語的提醒之前，多三句（`<顯示名稱>` 就是同一份回應的 `locale`）：

```
Requirement and scenario names are prose too: write them in <顯示名稱>. The exception is a name that points at the canonical spec: the requirement name under `## MODIFIED Requirements` or `## REMOVED Requirements`, the `FROM:` name of a RENAMED entry, a canonical scenario name that a MODIFIED block carries over, and the name inside a `<!-- REMOVED-SCENARIO: … -->` declaration. Copy each of those exactly as the canonical spec writes it, in whatever language it is in — a changed name fails validation.
```

三種規格語言都寫（English 也寫），因為「照正式規格原文」和規格語言無關。整段仍以 `Write spec prose in <顯示名稱>.` 開頭，仍是指示的最後一段；規格語言是 `tw` 時，中文模糊用語的提醒仍在這一段的最後。

用語上寫「write them in」而不是「write new names in」：既有 delta 規格裡的 ADDED 名稱算不算「new」會有歧義。規則是「名稱跟著內文走，例外是指向正式規格的名稱」，所以使用者要求把內文改成規格語言時，ADDED 的名稱跟著改，MODIFIED 的名稱不動。

四種例外逐一點名，包含討論結論沒有列出的 REMOVED-SCENARIO 宣告：它的名稱也要和正式規格的情境名稱相同，合併守門才認。

替代方案：
- 改既有 delta 規格時名稱一律不動（來源文件的提議）。不採用：封存後，中文的正式規格會多出英文的需求與情境名稱。
- 名稱一律英文，比照結構標記。不採用：推翻這個 repo 全部名稱的慣例。
- 把規則寫進 schema 原稿的「MODIFIED requirements workflow」。不採用：已複製出去的自訂 schema 拿不到，而且模型是在讀語言說明時決定要不要翻名稱，規則要放在同一段。

### D2 技能只加一句指向，完整規則不重述

propose 技能（`crates/engine/speclink-core/assets/skills/propose.md` 的 instructions JSON 欄位清單裡 `locale` 那一項）與 ingest 技能（`crates/engine/speclink-core/assets/skills/ingest.md` 更新步驟裡說明 `locale` 的那一段），在「… always stay in English.」之後各加同一句：

```
For the specs artifact, the `instruction` also says which requirement and scenario names keep the canonical spec's wording — follow it.
```

技能不列四種例外。原因：兩份文字會漂移，第八輪定下的原則是同一件事由引擎說一次。既有的固定用語「for the specs artifact it is the spec language」不動。新句子不含全大寫強調字。

替代方案：
- 只改引擎、技能不動。不採用：技能段落只講標記，模型會以為那就是全部的規則。
- 引擎與技能各寫一份完整規則。不採用：兩份會漂移。

### D3 ingest 的摘要列出改到的名稱

ingest 技能最後的摘要步驟（「Summary and next steps」的 Show 清單）在「Artifacts created/updated」之後多一項：

```
- Names changed: every requirement or scenario name this update changed, one per line as `<old name> → <new name>` — leave this item out when no name changed
```

這一項放在技能、不放在引擎的指示：它是 ingest 對使用者的報告，引擎不知道是哪支技能在呼叫。propose 不加：它建立新的 delta 規格，沒有既有的名稱。

### D4 技能分清 delta 規格的三種寫法

ingest 技能的「For each artifact, get instructions first」那個命令區塊之後加一段：

```
`<artifact-id>` is an id that `speclink status --change "<name>" --json` lists, such as `proposal`, `design`, `specs` or `tasks`. A change has one `specs` instruction for all of its delta specs: run `speclink instructions specs --change "<name>" --json`, with no capability name. Only the read and the write name the capability: `speclink artifact cat specs/<capability>` reads one delta spec, and `speclink new artifact spec <capability>` writes it back.
```

propose 技能讀前置產出物的那一項，在原句之後加：

```
The `specs` dependency is read one delta spec at a time: for each entry of `deltaSpecs` in `speclink show <name> --json` (listed as `<capability>/spec.md`), run `speclink artifact cat specs/<capability> --change "<name>"`.
```

兩段都不含全大寫強調字，也不提任何規格檔的路徑（`skill_verbization` 的守門）。

替代方案：把三種寫法寫進共用段落 `artifact-write-back.md`。不採用：那一段有 6 支技能在用，其中 4 支不呼叫 `instructions <產出物>`，加了只是雜訊。

### D5 instructions 的找不到訊息列出可用的代號

`run_instructions` 在 `build_artifact` 回 `None` 時，錯誤訊息由

```
Artifact '<代號>' not found in schema
```

改成

```
Artifact '<代號>' not found in schema. Use <清單>, or apply
```

`<清單>` 是這個變更的產出流程定義的全部產出物代號，以 `, ` 分隔，順序用 `crate::status::display_order`（和 `speclink status` 的 `artifacts` 相同）。內建的 spec-driven 得到「Use proposal, design, specs, tasks, or apply」，no-spec-change 得到「Use proposal, design, tasks, or apply」，自訂的產出流程列出它自己的代號。`apply` 只列一次：自訂的產出流程有代號叫 `apply` 的產出物時不重複列它，沒有定義任何產出物時訊息是「Artifact '<代號>' not found in schema. Use apply」。錯誤類別維持 `ErrorCode::NotFound`，開頭的句子不變，所以只比對開頭的呼叫者不受影響。

句型比照 `artifact cat` 既有的「Unknown artifact '<x>'. Use proposal, design, tasks, or specs/<capability>」。不針對 `specs/` 開頭的代號另加提示：清單裡已經有 `specs`，多一個分支沒有必要。

替代方案：`instructions` 接受 `specs/<capability>` 當成 `specs`。不採用，理由見 Non-Goals。

### D6 內嵌資產版號與 golden 同批更新

技能原稿改了，`ASSET_VERSION`（`crates/engine/speclink-core/src/workspace/init.rs`）由 v1.60.0 升到 v1.61.0，`crates/engine/speclink-core/tests/golden/` 的十份技能快照與 `assets.lock` 用既有的更新方式重新產生。順序：先改原稿與版號、建置 CLI，再重新產生 golden 與這個 repo 自己安裝的技能檔（.claude/skills、.agents/skills）。

## Implementation Contract

**行為**

- `instructions specs --json` 的 `instruction` 最後一段，三種規格語言都含 D1 的三句，其中 `<顯示名稱>` 等於同一份回應的 `locale`：

| `spec_locale` 的有效值 | 最後一段的開頭 | 名稱規則的第一句 | 最後一段的結尾 |
| --- | --- | --- | --- |
| `tw` | `Write spec prose in Traditional Chinese (繁體中文).` | `… write them in Traditional Chinese (繁體中文).` | 中文模糊用語的提醒 |
| `ja` | `Write spec prose in Japanese (日本語).` | `… write them in Japanese (日本語).` | `… a changed name fails validation.` |
| 未設定或 `en` | `Write spec prose in English.` | `… write them in English.` | `… a changed name fails validation.` |

- `instructions proposal|design|tasks --json` 與 `instructions apply --json` 的 `instruction` 不含 `Requirement and scenario names are prose`。
- 人眼輸出（不帶 `--json`）的 Instruction 區段是同一段文字；`--no-color` 下文字相同。
- `instructions <不在清單的代號> --change <名稱>`：exit code 非 0，stdout 沒有輸出，stderr 是 `Artifact '<代號>' not found in schema. Use <清單>, or apply`；帶不帶 `--json` 訊息相同。`specs/<capability>` 與 `spec` 都屬於不在清單的代號。清單內的代號與 `apply` 照常回應。找不到變更時是既有的「Change '<名稱>' not found.」，不列代號。
- 產生的 speclink-propose 與 speclink-ingest 技能（每一種工具、規格在不在 checkout 兩種情況）含 D2 的那一句；speclink-ingest 另含 D3 的摘要項目與 D4 的三種寫法；speclink-propose 另含 D4 的 `deltaSpecs` 讀法。

**介面與資料形狀**

回應的欄位名稱與型別不變（`locale` 仍是字串，`instruction` 仍是字串）。沒有新的命令、旗標、設定鍵或 serde 結構；不讀寫任何設定檔，既有的 `speclink/config.yaml` 照常讀得進來。錯誤仍是 `ErrorCode::NotFound`，經 `dispatch` 的錯誤碼仍是 `not_found`。

**失敗模式**

不新增失敗路徑。`speclink/config.yaml` 無法解析時 `instructions` 照舊以非零 exit code 失敗（fail-closed）。連到還沒升級的 Server 時，CLI 原樣輸出 Server 的指示與錯誤（舊文字），不補、不改。這次的改動不寫入任何文件，沒有多筆寫入的順序問題。

**驗收**

- `cargo test -p speclink-core`：`build_artifact` 的單元測試斷言三種規格語言的最後一段含 D1 的句子與四種例外、其他產出物不含；`command` 的測試斷言 spec-driven 與 no-spec-change 兩種產出流程的錯誤訊息全文，以及清單內的代號與 `apply` 照常回應。
- `cargo test -p speclink-core --test it`：`skill_verbization` 斷言 propose、ingest 的每一種產生結果含 D2、D3、D4 的固定用語；`skill_authoring` 的強調字上限照常通過；`render_golden` 在 golden 更新後通過。
- `cargo test -p speclink-cli --test it instructions_policy`：經 CLI 斷言 `instruction` 的名稱規則、人眼輸出與 `--no-color`、錯誤訊息的 exit code 與 stdout／stderr。
- `npm --prefix crates/adapters/speclink-node test`：經 `host.dispatch` 得到同一段名稱規則，以及 `not_found` 加同一句訊息。

**範圍**

- 範圍內：`build_artifact` 對 specs 的語言說明、`run_instructions` 的找不到訊息、propose 與 ingest 的技能原稿、資產版號與 golden、相關測試、`docs/configuration.md` 與 `docs/configuration.zh-TW.md` 的 `spec_locale` 說明。
- 範圍外：Non-Goals 列的每一項；speclink-fs、speclink-host、speclink-server、speclink-remote、apps/desktop 的程式（它們只轉送回應與錯誤）；`speclink/manual/` 的手冊頁（封存後照手冊的重生程序處理）。

**crate 邊界與共用方式**

兩處改動都在 speclink-core，都是字串的組合，不含 ANSI、不讀檔案系統、不呼叫 git。CLI、Server、Node SDK 經既有的命令入口取得結果，不新增平行實作；`crates/adapters/speclink-cli/tests/it/remote_verb_parity.rs` 的對照不需要改。

## Risks / Trade-offs

- [回歸對照] 每個專案的 `instructions specs` 的 `instruction` 都會變長；`instructions` 找不到代號的訊息也變長。→ 兩者都是刻意的變更，寫進規格；固定這段文字的測試與十份 golden 同批更新，更新後用 git diff 確認 golden 只有 propose、ingest 兩支技能的新句子與版號不同。全 repo 搜尋過，沒有程式或測試比對舊的錯誤訊息全文。
- [指示變長] specs 的指示多約 500 個字元。→ 這一段只在寫 specs 產出物時出現；和已經觀察到的錯誤（12 個名稱被翻掉）相比可以接受。
- [模型仍可能翻掉名稱] 指示只是文字。→ MODIFIED、REMOVED、RENAMED 與沿用的情境名稱被改掉時，`speclink validate` 照舊在封存前擋下；ADDED 的改名由 ingest 的摘要讓使用者看到。
- [版本落差] 新技能配舊引擎時，技能說「指示會寫明名稱規則」，但舊指示沒有。→ 那一句只是指向，模型找不到規則時行為和現在相同；CLI 在技能檔比自己新時會顯示技能版本提醒。
- [自訂 schema] 已複製出去的 schema 帶著舊的 specs 指示原文。→ 名稱規則由引擎加在最後一段，不在 schema 原稿裡，自訂 schema 也拿得到。
- [跨平台] 改動只有字串的組合，不碰路徑、檔案系統與子程序，Windows、macOS、Linux 行為相同。
- [儲存解耦] 名稱規則與可用代號都由引擎放進回應，技能不假設本機有規格檔；propose 讀 delta 規格改走 `show --json` 的 `deltaSpecs` 與 `artifact cat`，規格不在 checkout 時同樣可用。

## Context

GitHub issue #2 回報：長流程 skill 中 Agent 會中途改用英文。討論 skill-conversation-language 已定案：`locale` 維持「AI 產出文件與報告的語言」，不擴大成對話語言；對話語言漂移改由 skill 文字處理，並補齊 discuss、improve、config、audit 四個 skill 的 `locale` 綁定。

現況：

- 技能原稿放在 speclink-core 的 `assets/skills/`，由 `skills.rs` 的 registry 對應到 19 個產品 skill；review、verify、apply-with-worktree 用 `concat!` 在編譯期組合多份原稿。
- 所有 SKILL.md 都經同一個渲染函式 `render_skill_file_for` 產出：frontmatter →（claude fork skill 的）fork 前言或（自訂描述子的）invocation 說明 → 套用 `Substitutions` 後的內文。`speclink init`、`speclink update`、speclink-node 的 `render`、assets.lock 指紋都走這個函式。
- `speclink instructions --skill <name>` 走 `skill_body`，直接印出原稿，不經渲染函式；apply 用它讀 tdd 與 audit 的紀律段。
- 技能原稿受 `skill_authoring.rs` 守門：每份原稿要在 `EMPHASIS_CAPS` 登記大寫強調字上限，且不得超過 500 行。discuss（上限 5）、improve（2）、config（4）、audit（1）四份原稿的強調字都剛好用滿。
- 改任何原稿的內容都要升 ASSET_VERSION（`init.rs`），並以 `UPDATE_GOLDEN`、`UPDATE_ASSETS_LOCK` 再生六份 render golden 與 assets.lock。

邊界：本變更全部落在 speclink-core（技能原稿、渲染函式與其測試）。speclink-cli、speclink-node、desktop 都只呼叫同一個渲染函式，不改程式碼；技能只渲染一次，沒有 local／remote 兩條路徑的問題。

## Goals / Non-Goals

**Goals:**

- 每個產品 skill 的 SKILL.md 開頭都有同一段對話語言規則，且這段規則只有一份來源。
- discuss、improve、config、audit 的產出依 `locale` 撰寫，與 propose、ingest、review、verify、drift、analyze、manual、baseline 一致。
- 不改 `locale` 的語意、預設值與任何 CLI 輸出。

**Non-Goals:**

- 不讓 `locale` 決定對話語言（討論中已否決：團隊設 `locale: en` 時會強迫成員用英文對話）。
- 不改子代理的 brief：子代理不直接對使用者說話，review、verify 的報告本來就綁 `locale`。
- 不改 commit、worktree-merge、quality、trace 的內文：commit 的摘要句取自 proposal（已是 `locale` 語言），其餘三個不寫文件。
- 不在 `instructions apply --json` 加 `specLocale`，也不讓 apply、archive 查 `spec_locale`：它們只改既有檔案，由共用段落的「沿用原檔語言」處理。
- 不讓 `workflow-config show` 套用 SPECLINK_LOCALE 環境變數（manual、baseline 已用同樣方式讀 `locale`）。
- 不回改已存在的討論記錄。

## Decisions

### 共用對話語言段落在渲染函式插入

新增原稿 `conversation-language.md`，在 `skills.rs` 以 `include_str!` 讀成常數；`render_skill_file_for` 在 fork 前言或 invocation 說明之後、技能內文之前，把這段（經同一個 `Substitutions`）寫入，並與內文之間空一行。

替代方案：照討論結論的字面，在 19 個內文常數各自 `concat!` 這份原稿。否決理由：要改 19 處，且 review、verify、apply-with-worktree 本來就是多份原稿的 `concat!`，容易漏接或接兩次；在渲染函式插入只改一處，仍是單一來源，符合結論「單一來源」的本意。另一個好處是 `instructions --skill` 印出的原稿不含這段，apply 讀 audit 紀律段時不會在對話中重複出現。

### 段落措辭

段落內容（英文，與其他原稿一致；不用任何大寫強調字，所以新原稿在 `EMPHASIS_CAPS` 登記上限 0）：

```
**Conversation language**: write to the user in the language they use with you — progress updates, questions, and summaries. This skill is written in English, and its example messages show structure, not language: translate their prose, and keep commands, file paths, code, and fixed tokens (severity labels, status words) as they are. When a step sets the language of a document or a report (for example through `locale`), that step decides for that output. When you edit a document that already exists and no step sets its language, keep the language it is written in.
```

四句的先後就是優先順序：步驟規定的語言勝過「沿用原檔語言」，兩者都只管文件與報告，不管對話。

### 四個 skill 的 locale 綁定寫在執行該動作的步驟

依 skill-authoring「每條規則只在執行該動作的步驟完整陳述一次」，綁定句寫在各自產出內容的那一步，讀法統一為 `speclink workflow-config show --json` 的 `locale` 欄位（`tw` 繁體中文、`ja` 日文、`en` 或 null 英文）：

- discuss：Step 5（建立記錄）加 **Record language**。新建記錄的 Context、各輪與結論用 `locale` 語言；topic 維持使用者原文；欄位標記（`**Focus**:`、`**Position**:`、`**Ruled out**:`、`**Open**:` 與結論各欄位）、slug、指令與程式碼維持英文。恢復既有記錄時沿用它原本的語言。
- improve：Step 4（記錄 candidates）加同樣的 **Record language**；improve 的 topic 由模型撰寫，所以也用 `locale` 語言。
- config：Step 1 第 4 項已讀 `workflow-config show --json`；Step 2 起草處加一句，`context` 與 `rules` 的新行用 `locale` 語言，review 規則照使用者原話。
- audit：Phase 3（彙整與報告）加 **Report language**，比照 analyze、drift 的寫法；嚴重度（Critical／High／Medium／Low）、檔案路徑與指令維持英文。只綁獨立模式，Discipline Mode 不產出報告。

### 既有討論記錄沿用原語言

替代方案：恢復舊記錄時也改用 `locale`。否決理由：升級時尚未結論的舊記錄（多半跟著對話語言寫）會變成一份檔案兩種語言；新建記錄用 `locale`、既有記錄沿用原語言，與共用段落「沿用原檔語言」同一條原則。

### 版號與再生

ASSET_VERSION 由 v1.48.0 升為 v1.49.0（新增技能行為，次版號）。改完原稿後，以 `UPDATE_GOLDEN=1` 再生六份 render golden、以 `UPDATE_ASSETS_LOCK=1` 再生 assets.lock，最後以 `speclink update` 再生 repo 自用的技能檔。

### 使用者文件補一句

`docs/configuration.md` 與 `docs/configuration.zh-TW.md` 設定鍵表格的 `locale` 列，用途欄改寫為「AI 產出文件與報告的語言；不決定 Agent 跟你對話的語言」（英文版同義）。兩份同步改，維持中英對照。

## Implementation Contract

**Behavior**

- 任何工具（claude、codex、copilot、自訂描述子）產出的每一份產品 skill SKILL.md，在 frontmatter（與 fork 前言或 invocation 說明）之後、技能內文之前，都恰好出現一次 `**Conversation language**:` 段落，內容與上述措辭逐字相同。
- `speclink instructions --skill <name>` 的輸出不含該段落。
- discuss、improve 的 SKILL.md 在建立記錄的步驟含 **Record language** 規則；config 的 SKILL.md 在起草步驟含 `locale` 綁定；audit 的 SKILL.md 在 Phase 3 含 **Report language** 規則；四者都點名 `speclink workflow-config show --json` 的 `locale` 欄位，並寫明哪些固定 token 維持英文、`locale` 為 null 時寫英文。

**Interface / data shape**

- 新常數（例如 `CONVERSATION_LANGUAGE`）以 `include_str!` 讀入新原稿；`render_skill_file_for` 的簽名不變。
- 不新增 CLI 指令、旗標或 JSON 欄位。

**Failure modes**

- 新原稿未在 `EMPHASIS_CAPS` 登記、或段落用了大寫強調字 → `skill_authoring.rs` 守門紅燈，訊息點名檔案。
- 改了原稿未升版號 → assets.lock 測試紅燈，訊息帶修法。
- 執行時 `workflow-config show --json` 失敗 → discuss、improve、audit 以英文撰寫（比照 analyze、drift 的「讀不到就寫英文」）；config 照它對固定輸入來源既有的處理。

**Acceptance criteria**

- `render_golden.rs` 新增測試：claude 與 codex 兩個工具的每一份產品 skill 都恰好含一次段落開頭 `**Conversation language**:` 與「keep the language it is written in」，且段落在技能內文第一行之前；`skill_body` 取得的原稿不含段落。
- `render_golden.rs` 新增測試：discuss、improve、config、audit 四份 SKILL.md 含各自的 `locale` 綁定字句（`speclink workflow-config show --json`、`locale`、維持英文的 token 清單）。
- `skill_authoring.rs` 的守門測試全綠（含新原稿上限 0）。
- 六份 render golden 與 assets.lock 以新版號再生後，`cargo test -p speclink-core` 全綠。
- `speclink update` 後，repo 內 claude 與 codex 技能目錄的 speclink 技能檔都帶有段落與 v1.49.0。

**Scope boundaries**

- In scope：新原稿、渲染函式插入、四份原稿的綁定句、守門登記、版號、golden、assets.lock、兩份 configuration 文件、repo 自用技能檔再生。
- Out of scope：其他 15 份原稿的內文、子代理 brief、`instructions` payload、`workflow-config` 動詞、desktop 與 server 程式碼、已存在的討論記錄與手冊頁。

## Risks / Trade-offs

- [回歸對照：六份 render golden 與 assets.lock 一次大量變動，真正的改動被淹沒] → 先寫失敗的內容測試再改原稿，再生 golden 後用 git diff 確認每份快照的差異只有段落、四個綁定句與版號三種。
- [回歸對照：其他 crate 的測試若釘住技能檔的開頭] → 已查無釘住內文第一行的測試。speclink-node 的 render 測試只比對「渲染結果＝init 產出」並檢查中性渲染不含 `/speclink-` 與 `{{`，段落兩者皆無；speclink-cli 的版號測試讀 `ASSET_VERSION` 常數，升版號後跑 `cargo test -p speclink-cli --test it engine_version` 確認。
- [跨平台：Windows 的 CRLF 讓段落與內文之間的空行數不同] → golden 與 assets.lock 已以 `normalize_eol` 正規化；段落常數與內文之間以固定的 `\n\n` 串接，不依賴原稿檔尾換行。
- [四份原稿的強調字已用滿上限] → 新增的綁定句只用小寫語氣，不調高任何上限。
- [規則不保證 Agent 永不漂移] → 接受；回覆 issue 時已說明，之後再收集實例。
- [沒設 `locale` 的專案，新建討論記錄、config 起草與 audit 報告改為英文] → 刻意的行為改變，與 proposal 現況一致，已寫進 proposal 的相容性影響與 issue 回覆。

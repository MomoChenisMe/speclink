## ADDED Requirements

### Requirement: 技能共用的對話語言段落

speclink 渲染的每一份產品技能檔（registry 中的全部技能；claude、codex、copilot 三個內建工具與自訂描述子皆同）SHALL 在 frontmatter 與 fork 前言或 invocation 說明（若有）之後、技能內文之前，恰好含一次共用的對話語言段落，段落以 `**Conversation language**:` 開頭。段落的來源 SHALL 為單一原稿 crates/engine/speclink-core/assets/skills/conversation-language.md，SHALL NOT 在各技能原稿各存一份複本。

段落 SHALL 依序規定四件事，前者優先於後者：

1. 對使用者說的話（進度、提問、摘要）以使用者跟 Agent 講話的語言撰寫。
2. 技能以英文撰寫、回報範例只示範格式：範例的散文要翻譯，指令、檔案路徑、程式碼與固定 token（嚴重度標籤、狀態詞）維持原樣。
3. 步驟規定文件或報告的語言（例如經 `locale`）時，該產出以該步驟為準。
4. 修改既有文件且沒有步驟規定語言時，沿用該文件原本的語言。

段落 SHALL NOT 含全大寫強調字，其原稿在強調字守門的上限表登記為 0。對話語言 SHALL NOT 由 `locale` 決定：`locale` 為 tw、ja、en 或未設定時，段落內容逐字相同。`speclink instructions --skill <name>` 輸出的原稿 SHALL NOT 含此段落；內部技能（tdd、clarify、sync）不渲染為技能檔，不在本條範圍。

此為刻意變更：所有產品技能檔在內文前多出此段落，段落之外的技能內容維持既有位元級輸出（另行變更的技能除外）。

#### Scenario: 每份產品技能檔恰含一次段落

- **WHEN** 在空目錄執行 speclink init 並選 claude 與 codex 兩個工具
- **THEN** .claude/skills 與 .agents/skills 下每一份 speclink-* 技能的 SKILL.md 都恰好含一次 `**Conversation language**:`，位置在 frontmatter 結尾 `---` 之後、該技能內文第一行之前；exit code 為 0

#### Scenario: fork 技能的前言在段落之前

- **WHEN** 渲染 claude 工具的 speclink-drift 技能檔（fork 技能，帶 fork 前言）
- **THEN** 技能檔依序為 frontmatter、fork 前言、對話語言段落、drift 技能內文

#### Scenario: 自訂描述子的技能檔同樣帶段落

- **WHEN** .speclink.yaml 宣告一個自訂描述子後執行 speclink update
- **THEN** 該描述子技能目錄下每一份 speclink-* 技能檔都在 invocation 說明之後、技能內文之前恰好含一次段落

#### Scenario: instructions --skill 的原稿不含段落

- **WHEN** 執行 speclink instructions --skill audit
- **THEN** stdout 為 audit 技能原稿，不含 `**Conversation language**:`；exit code 為 0

#### Scenario: 段落用了大寫強調字時守門失敗

- **WHEN** 對話語言段落的原稿含 `NOT` 這類全大寫強調字，執行 cargo test -p speclink-core
- **THEN** 技能原稿守門測試失敗，訊息點名 conversation-language.md 的強調字上限 0 與實際數量

##### Example: 段落與 locale 無關

| openspec/config.yaml 的 locale | 技能檔中的段落 |
| ------------------------------ | -------------- |
| tw | 與原稿逐字相同的英文段落 |
| ja | 與原稿逐字相同的英文段落 |
| en | 與原稿逐字相同的英文段落 |
| 未設定 | 與原稿逐字相同的英文段落 |

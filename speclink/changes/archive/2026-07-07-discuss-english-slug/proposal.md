## Why

討論記錄的檔名由主題經 slugify 直接衍生,中文主題產生中文檔名;變更(change)的名稱則由 agent 在提案時衍生英文 kebab-case,兩者不一致。目標使用者是透過 AI 代理跑 SDD 的開發者,情境為 discuss 技能記錄討論之時:希望討論檔名與變更一致採英文命名,便於跨平台檔案管理、連結引用與終端輸入。2026-07-07 討論「discuss-檔名英文命名」定案:英文名由雙語的 agent 衍生,引擎只提供覆寫入口與驗證,不承擔翻譯。

## What Changes

- speclink discuss new 新增選配 --slug 旗標(speclink-cli):值必須為純 ASCII kebab-case(小寫英數字與連字號、不得為空),非法值以非零 exit code 報錯並不落任何檔案。
- 提供 --slug 時,以該值作為檔名與 frontmatter 的 slug;topic 維持使用者原文(如中文)供顯示(speclink-core 的建立函式新增 slug 覆寫參數)。
- 未提供 --slug 時維持現行 slugify 後備行為(保留 CJK 字元)——既有呼叫方式行為不變。
- discuss 技能指示更新:建立討論記錄時一律從主題衍生英文 kebab-case slug 並以 --slug 傳入。內嵌技能與 repo 技能實例、render golden 基準三處同步。
- 順帶效益:discuss promote 未帶名稱時預設變更名等於討論 slug,英文化後此預設值變得可用。

相容性影響:--slug 為純新增旗標;discuss new 既有的人眼輸出與 --json 欄位不變,未帶 --slug 的行為逐位元不變,回歸對照不受影響。既有中文檔名討論不回改。

## Non-Goals

- 不做引擎端音譯或翻譯(需拼音庫依賴,且拼音對臺灣使用者不可讀)。
- 不改 slugify 的 CJK 後備行為(丟棄 CJK 會讓純中文主題塌成無意義 slug,比中文檔名更糟)。
- 不回改既有中文檔名討論(改名會斷 from_discussion 連結,遷移成本換零功能收益)。
- 不動 discuss 其他子指令(show、list、promote 等)的介面與輸出。

## Capabilities

### New Capabilities

- `discussion-docs`: 討論文件的建立與命名契約——slug 自主題衍生的後備規則、--slug 覆寫入口與其驗證。

### Modified Capabilities

(none)

## Impact

- Affected specs: 新增 discussion-docs
- Affected code:
  - Modified: crates/speclink-cli/src/main.rs、crates/speclink-cli/src/commands.rs、crates/speclink-core/src/discuss.rs、crates/speclink-core/src/util.rs、crates/speclink-core/assets/skills/discuss.md、.claude/skills/speclink-discuss/SKILL.md、.agents/skills/speclink-discuss/SKILL.md、crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - New: (none)
  - Removed: (none)

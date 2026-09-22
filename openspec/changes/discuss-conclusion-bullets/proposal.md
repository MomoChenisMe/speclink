## Why

討論記錄的 Conclusion 沒有條列規則：技能的文件規則第 5 條只規定輪的 Position「超過一句就條列」，Decision 卻允許整段長句，本專案最長 3947 字、wadpilot 的 team-server-and-admin 3355 字一行。桌面 app 的結論分頁（ConclusionView）已把六欄位拆成標籤區塊、欄位內文交 Markdown 渲染，來源有條列就能渲染成清單，所以牆狀結論的根因在寫法不在渲染器。第 5 條落地後（2026-09-10，drawer-document-readability）12 份記錄有 11 份零超長 Position 行，證明同型規則有效。目標使用者是透過 AI 代理跑 /speclink-discuss 的開發者：agent 依技能寫結論，使用者在桌面結論分頁與 propose 的提案裡讀它。

## What Changes

- speclink-core 內嵌的 discuss 技能 asset（crates/engine/speclink-core/assets/skills/discuss.md）的 Document rules 新增第 8 條「Conclusion bullets over prose」：Decision、Rejected alternatives、Deferred 超過一句 SHALL 條列——一句定論起頭，之後 `- ` 一點一行；Rationale、Capture to、Next 維持單段。與第 5 條對稱。
- 第 8 條同時規定多刀結論的固定寫法：每刀一個 bullet，開頭 `**cut N \`change-name\`**：一句範圍`，細節縮排子項、一子項一件事；Decision 保留全部定案細節、不縮水、子項不回指輪次。
- 第 8 條規定 Rejected alternatives 每項一行「方案——落敗理由」；Deferred 每項一行「問題——為何現在不解」或單寫 none。
- 技能檔的 conclude 模板（`speclink discuss conclude … --stdin` 範例）與「Capture decisions」的 Conclusion 摘要格式同步改為條列形，讓 agent 照抄模板就符合第 8 條。
- ASSET_VERSION（crates/engine/speclink-core/src/workspace/init.rs）自 v1.39.0 升至 v1.40.0；五份 render golden 與 assets.lock 於乾淨樹重生；`speclink update` 重生 .claude 與 .agents 下的 speclink-discuss SKILL.md（其餘技能實例只有 version 行變動）。
- 正式規格 discuss-skill 的「討論記錄的樹慣例與格式不變」MODIFIED：補上結論欄位的條列規則與多刀寫法，並維持「既有記錄不需遷移」的保證。
- 影響的技能與工具：speclink-discuss 一支，claude／codex／neutral 三種渲染目標同源；引擎動詞、CLI 旗標、`--json` 輸出、config 欄位皆不變，無相容性影響；既有討論記錄不回改，舊格式結論在桌面仍走「討論結論以欄位標籤呈現」的既有渲染。

## Non-Goals

- 不改桌面 app 的 ConclusionView、Markdown 渲染或 96ch 行寬（specs-archive-pagination 已定）；不做「；」或「cut N」的啟發式自動拆行——會切壞正常句子。
- 不縮減 Decision 內容、不把細節搬到輪：輪是 append-only、含被推翻立場，propose 靠 Decision 拿最終狀態與刀清單。
- 不改 `discuss promote` 把整段 Conclusion 預填進提案 Why 的引擎行為：預填只是鷹架，propose 技能整份重寫 proposal.md。
- 不改引擎寫進記錄檔的 scaffold 註解（discuss.rs 的 Document rules 註解）；不回改任何既有討論記錄。wadpilot 的 team-server-and-admin 由使用者於規則落地後自行重跑 conclude，不在本變更範圍。

## Capabilities

### New Capabilities

(none)——規格掃描命中 discuss-skill（技能訪談行為與記錄格式）、discussion-docs（記錄檔名與鏈結語意）、desktop-app（結論分頁渲染）；本變更只改技能文字規則，落在 discuss-skill 既有的「討論記錄的樹慣例與格式不變」需求內，不需新 capability。

### Modified Capabilities

- `discuss-skill`: 「討論記錄的樹慣例與格式不變」補上 Conclusion 欄位的條列規則（Decision／Rejected alternatives／Deferred 超過一句條列、多刀每刀一 bullet、Rationale／Capture to／Next 單段）與模板同步；骨架、輪欄位、append-only 與不遷移的保證不變。

## Impact

- Affected specs: discuss-skill（MODIFIED）
- Affected code:
  - New: (none)
  - Modified:
    - crates/engine/speclink-core/assets/skills/discuss.md（Document rules 第 8 條、conclude 模板範例、Capture decisions 摘要格式）
    - crates/engine/speclink-core/src/workspace/init.rs（ASSET_VERSION v1.39.0 → v1.40.0）
    - crates/engine/speclink-core/tests/golden/claude.snapshot.md、crates/engine/speclink-core/tests/golden/claude-worktree.snapshot.md、crates/engine/speclink-core/tests/golden/codex.snapshot.md、crates/engine/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/engine/speclink-core/tests/golden/neutral-tool-call.snapshot.md（UPDATE_GOLDEN=1 重生）
    - crates/engine/speclink-core/tests/golden/assets.lock（UPDATE_ASSETS_LOCK=1 重生）
    - .claude/skills/speclink-discuss/SKILL.md、.agents/skills/speclink-discuss/SKILL.md（speclink update 重生；其餘 35 份 SKILL.md 僅 version 行變動）
  - Removed: (none)

## Why

「兩站都跑」（review＋verify）已成為本專案的預設收尾流程，但兩站正典的「修完即蓋章」預設會讓後蓋站的修正把先蓋的章打黃（「已完成·其後有變動」），目前只能靠本機非正典技能 .claude/skills/speclink-quality 以時序編排避開——它不進 golden、不三處同步，規則也不在任何生成文件中。討論 cross-station-staleness（2026-08-04）Deferred 的「重議產品化」條款經使用者裁定正式成立：把 quality 編排收進引擎正典，讓「兩章皆過才正式蓋」成為有官方入口、有同步保障的正典流程。

目標使用者：透過 AI 代理跑 SDD 的開發者。使用情境：change 任務全數完成後、封存之前的品質關卡階段——事前已知兩站都要跑時走 `/speclink-quality`；只跑單站仍直接呼叫 `/speclink-review` 或 `/speclink-verify`。

## What Changes

- **新增正典技能 quality**：crates/speclink-core 新增技能 asset（兩站時序編排：review 檢查先不蓋章 → verify 檢查先不蓋章 → 兩站 findings 統一修正 → 各自複驗 → 兩章接連蓋 → 封存），註冊進 skills.rs 技能表，`speclink update` 生成 speclink-quality 技能檔至已啟用工具（claude／codex），golden 對照涵蓋。
- **兩站正典補 quality 時序例外**：review 與 verify 技能 asset 各補「quality 時序中零 findings 的 discovery 不自動蓋章、改走先不蓋章出口」——堵掉原「乾淨首輪自動蓋章」在 quality 時序下破壞「兩章皆過才蓋」保證的縫隙（cross-station-staleness 原記為 Deferred，本次收斂）。
- **生成文件範本更新**：init.rs 範本的 workflow 行加入 quality 入口、技能使用清單加入 quality 條目（觸發時機：兩站都跑時；單站直接呼叫該站技能）；CLAUDE.md／AGENTS.md 經 `speclink update` 刷新，不手改生成物。
- **README 說明文件**：README.md／README.en.md 兩站分工表補「兩站都跑 → /speclink-quality」入口與時序說明。
- **單站語意零變更**：使用者自行單獨觸發 review 或 verify 時維持現行「修完即蓋」預設；蓋章後被後續修正打黃屬正常警示，封存定格回綠。
- 無新增或變更 CLI 子指令、旗標、stdin 與 exit code；無設定欄位（openspec/config.yaml／.speclink.yaml）變更。

**相容性影響**：人眼輸出與 `--json` shape 皆不變。生成物內容刻意變更——golden snapshot（claude／codex／neutral 各 target 與 assets.lock）、技能檔（speclink-quality 新增；speclink-review／speclink-verify 補例外行）、CLAUDE.md／AGENTS.md workflow 行與技能清單——同批更新 golden；MARKER_VERSION 提升使既有專案的 `speclink update` 重新生成上述檔案。本機既有 .claude/skills/speclink-quality/SKILL.md 由引擎生成物取代。

## Non-Goals (optional)

詳見 design.md 的 Goals / Non-Goals；要點：不新增引擎狀態層支援（無 quality 工單／章／動詞）、不改兩站的檢查內容與裁決邏輯、不做看板／GUI 的「品質關卡進行中」顯示、不改單站流程語意。

## Capabilities

### New Capabilities

- `quality-skill`: 品質關卡編排技能——生成與正典化（技能檔、workflow 行與技能清單條目、golden 涵蓋）、兩站時序行為（先不蓋章 → 統一修正 → 各自複驗 → 接連蓋章）、事後變卦等邊界情況。

### Modified Capabilities

- `review-skill`: 「審查後的迴圈與收尾」補 quality 時序例外（零 findings 不自動蓋章、改走先不蓋章離場）；「審查技能的生成與正典化」的 workflow 行文字更新為含 quality 入口的版本。
- `verify-skill`: 「驗證收尾迴圈」補同構的 quality 時序例外。（此能力由進行中 change verify-station-parity 新增；本 change 相依其先落地封存，delta 以其後的正典為基準。）

## Impact

- Affected specs: 新增 quality-skill；修改 review-skill、verify-skill
- Affected code:
  - New: crates/speclink-core/assets/skills/quality.md
  - Modified: crates/speclink-core/src/skills.rs、crates/speclink-core/src/init.rs、crates/speclink-core/assets/skills/review.md、crates/speclink-core/assets/skills/verify.md、crates/speclink-core/tests/it/render_golden.rs、crates/speclink-core/tests/it/skill_verbization.rs、crates/speclink-core/tests/golden/assets.lock、crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md、CLAUDE.md、AGENTS.md、README.md、README.en.md、.claude/skills/speclink-quality/SKILL.md、.claude/skills/speclink-review/SKILL.md、.claude/skills/speclink-verify/SKILL.md
  - Removed: （無）
- 相依：verify-station-parity 先落地封存後本 change 才可開工（verify 站的工單與章、verify-skill 正典為本 change 的修改基準）

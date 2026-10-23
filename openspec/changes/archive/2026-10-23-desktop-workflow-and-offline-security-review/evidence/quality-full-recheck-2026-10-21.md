# 技能格式修正與完整品質重跑（2026-10-21）

## 第 1 步完成

- 原始失敗已重現：archive 5 > 4、verify 22 > 20、security-review 未登記。
- archive 與 verify 只將非必要的大寫 NOT 改為 not；流程與停止／安全條件不變。security-review 登記實際上限 2，未提高任何既有上限。
- 初次調整 archive 的措辭碰到 archive/commit 共用文字回歸；改在選擇變更的句子精簡大寫，保留共用文字。資產鎖依版本規則重新遞增，最終 v1.50.5；golden、assets.lock、生成技能及 Node binding 已同步。
- 已通過：cargo build --offline -q -p speclink-cli；cargo test --offline -q -p speclink-core（1081 單元、94 整合、1 ignored）；render_golden 63；skill_authoring 11；CLI init 27；remote_verb_parity 49；Node build 後 57；scripts 386；git diff --check。
- 同類排查：逐份掃描 assets/skills/*.md 與 EMPHASIS_CAPS，由整合測試確認全部登記且未超限；其餘原稿保留，沒有同類失敗。無新增參數或設定降級點，audit 三觀點確認未改產品安全判定。
- 全 repo 的 test:all 仍由 CI 守門；本步未重跑無改動的 UI、Server 與跨平台實機。公司 Proxy 任務 12.3 保持延期。

## 同類排查計數

```text
analyze.md: 1
apply-worktree-post.md: 4
apply-worktree-pre.md: 21
apply.md: 8
archive.md: 4
audit.md: 1
baseline.md: 8
clarify.md: 4
commit.md: 6
config.md: 4
conversation-language.md: 0
discuss.md: 5
drift.md: 2
improve.md: 2
ingest.md: 10
manual.md: 5
propose.md: 6
quality.md: 3
review-intro.md: 1
review.md: 19
security-review.md: 2
station-shared.md: 0
sync.md: 5
tdd.md: 4
trace.md: 3
verify-intro.md: 1
verify.md: 20
worktree-merge.md: 18
```

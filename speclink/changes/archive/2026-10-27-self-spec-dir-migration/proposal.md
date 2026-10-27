## Why

spec-dir-migration、desktop-spec-dir-migration 與 spec-dir-path-rewrite 封存後，產品已改用 `speclink/` 與 `.meta.yaml`，但 speclink 這個 repo 自己的規格仍在 `openspec/`（舊佈局，每次執行都會印遷移警告），repo 的工具也寫死 `openspec/`：文件守門與發版守門的腳本、文件截圖腳本、技能評測的夾具與評分規則、`.gitignore`、`.dockerignore`、PR 範本、貢獻指南、repo 專用的 release 技能，以及文件中描述 repo 自身與指向 repo 內檔案的相對連結。本變更是討論 positioning-and-process-cost 分刀計畫的最後一刀（第 9 刀），完成後才發版。

這個變更不改任何正式規格承諾的行為：它只用已發布的 `speclink update` 搬移本 repo 自己的資料，並把 repo 工具與文件裡的路徑對齊正式規格（spec-dir-path-rewrite 已把正式規格改為 `speclink/`）；產品程式碼不變。查過的正式規格（位於規格資料夾的 specs/<capability>/spec.md，搬移前在 openspec/、搬移後在 speclink/）：spec-dir-migration（遷移行為照原樣使用）、user-documentation（README、貢獻指南與 PR 範本）、ui-copy-vocabulary（詞彙守門的掃描面）、delivery-baseline（發版守門與 .dockerignore）、skill-authoring（技能評測）、release-skill（release 技能）、config-skill（貢獻指南的引用）。

## What Changes

- 在主目錄（沒有其他 worktree、規格資料夾沒有未提交修改）以新建置的 CLI 執行 `speclink update`，把 `openspec/` 改名為 `speclink/`，所有進行中與已封存變更的 `.openspec.yaml` 改名為 `.meta.yaml`，內容不變。
- 更新 repo 工具的路徑：scripts/docs/vocabulary-guard.test.mjs（LANGUAGE.md 的位置與排除的掃描面）、scripts/docs/docs-screenshots.mjs（示範工作區的路徑與資料檔名）、scripts/release/delivery-gate.test.mjs（略過的歷史路徑）、`.gitignore`（向量搜尋資料庫的位置）、`.dockerignore`（排除規格資料夾）、.github/PULL_REQUEST_TEMPLATE.md、CONTRIBUTING.md、.claude/skills/release/SKILL.md、integrations/claude-code/skill-evals 下的夾具、評分規則與提示、integrations/claude-code/speclink-skills 的 README。
- 更新文件中描述 repo 自身的段落與指向 repo 內檔案的相對連結（`../openspec/...` 改為 `../speclink/...`）：README（中英）、docs/server-deployment、docs/remote-getting-started、docs/sdk-node、docs/product-status、docs/roadmap、docs/verb-contract、docs/development（中英）。說明舊佈局相容的句子（例如「舊版建立的 `openspec/` 專案」）照原樣保留。
- 規格資料夾裡描述本 repo 的設定與詞彙：speclink/config.yaml 的專案說明與產出規則、speclink/LANGUAGE.md 的詞條定義，路徑改為 `speclink/` 與 `.meta.yaml`。
- 封存後發版。

## Non-Goals

- 不改產品程式碼與正式規格的需求。
- 不改歷史記錄：已封存變更與討論的內文、CHANGELOG 保留當時的路徑。
- 不在 repo 外做任何事（例如使用者的記憶檔）。
- 不在有其他 worktree 或未提交修改時強行搬移；條件不符就先處理那些狀態。

## Impact

- Affected code:
  - Moved: openspec/ → speclink/（含所有變更資料檔改名）
  - Modified: scripts/docs/vocabulary-guard.test.mjs, scripts/docs/docs-screenshots.mjs, scripts/release/delivery-gate.test.mjs, .gitignore, .dockerignore, .github/PULL_REQUEST_TEMPLATE.md, CONTRIBUTING.md, .claude/skills/release/SKILL.md, integrations/claude-code/skill-evals/evals/fixtures.sh, integrations/claude-code/skill-evals/evals/analyze-reports-only/scaffold.sh, integrations/claude-code/skill-evals/evals/drift-reports-only/scaffold.sh, integrations/claude-code/skill-evals/evals/manual-generates-pages/scaffold.sh, integrations/claude-code/skill-evals/evals/manual-generates-pages/prompt.md, integrations/claude-code/skill-evals/evals/manual-generates-pages/graders/manual-page-written.md, integrations/claude-code/skill-evals/evals/archive-folds-delta/prompt.md, integrations/claude-code/skill-evals/evals/archive-folds-delta/graders/no-hand-edit.md, integrations/claude-code/skill-evals/evals/archive-folds-delta/graders/no-hand-write.md, integrations/claude-code/skill-evals/evals/archive-folds-delta/graders/spec-merged.md, integrations/claude-code/skill-evals/evals/apply-leaves-manual-task/graders/code-task-checked.md, integrations/claude-code/skill-evals/evals/apply-leaves-manual-task/graders/manual-task-left-open.md, integrations/claude-code/skill-evals/evals/baseline-map-before-specs/graders/no-specs-yet.md, integrations/claude-code/skill-evals/evals/ingest-folds-new-requirement/graders/delta-has-new-requirement.md, integrations/claude-code/skill-evals/evals/propose-from-discussion/graders/manual-marker-after-checkbox.md, integrations/claude-code/skill-evals/evals/propose-from-discussion/graders/no-marker-after-number.md, integrations/claude-code/speclink-skills/README.md, integrations/claude-code/speclink-skills/README.en.md, README.md, README.zh-TW.md, docs/server-deployment.md, docs/server-deployment.zh-TW.md, docs/remote-getting-started.md, docs/remote-getting-started.zh-TW.md, docs/sdk-node.md, docs/sdk-node.zh-TW.md, docs/product-status.md, docs/product-status.zh-TW.md, docs/roadmap.md, docs/roadmap.zh-TW.md, docs/verb-contract.md, docs/verb-contract.zh-TW.md, docs/development.md, docs/development.zh-TW.md, speclink/config.yaml, speclink/LANGUAGE.md
- 影響的 crate／app：沒有。
- 生成的技能：repo 自用的技能檔由 `speclink update` 以 `speclink/` 重新渲染；技能原稿不變，不升 ASSET_VERSION。
- 相容性影響：這個 repo 的協作者要先升級 speclink 再 pull 這個 commit。

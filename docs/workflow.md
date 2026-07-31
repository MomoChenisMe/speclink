# Complete Speclink SDD Workflow

[繁體中文](workflow.zh-TW.md) · **English**

This is the user-facing workflow canon for what each stage does, when to use or skip it, its completion criteria, and the next route. For a first Local Repo loop, start with [Getting Started](getting-started.md). To decide whether a product capability is usable today, see [Product Capability Status](product-status.md).

## Mental model / 心智模型

```text
onboard? → discuss? → propose → apply ⇄ ingest → archive
                         ↑
                 resume after pause: drift first

utilities: validate / analyze / audit / commit / verify and evidence
```

- `onboard` creates current-behavior canonical specs once for an existing codebase.
- `discuss` is optional and only for decisions that need convergence; a question alone creates no discussion.
- `propose → apply ⇄ ingest → archive` is the main change lifecycle.
- `drift` conditionally precedes resumed work. `validate`, `analyze`, `audit`, `commit`, and verify/evidence are utilities or gates, not lifecycle states every change visits in sequence.

## Choose the entry / 選擇入口

Ask these five questions in order; the first match is the recommended entry:

| Question / 問題 | Answer / 判斷 | Recommended entry / 推薦入口 |
| --- | --- | --- |
| Do you only want understanding, with no pending decision? | Yes | Answer directly; do not create a discussion. |
| Is there already a related change? | Yes | Continue implementation with `apply`; if new context changes artifacts, use `ingest`. |
| Was the change idle, or might its basis have changed? | Yes | Run `drift`, then return to `apply` or route to `ingest`. |
| Did requirements or external context change during implementation? | Yes | `ingest`, then resume `apply` after updating artifacts. |
| Is the new requirement clear? | Yes / No | Clear: `propose`. Trade-offs remain: `discuss`. |

If an existing codebase has no canonical specs, run `onboard` once before the change flow. It creates current truth, not future aspirations.

## Lifecycle and utilities / 生命週期與工具

| Kind / 類型 | Stages / 階段 | Meaning / 意義 |
| --- | --- | --- |
| Main lifecycle / 主生命週期 | `propose`, `apply`, `ingest`, `archive` | Plan, implement, update requirements, and merge into canon. |
| Conditional / 條件式 | `onboard`, `discuss`, `drift` | Only for adopting an existing codebase, settling a decision, or resuming idle work. |
| Quality and safety / 品質與安全 | `validate`, `analyze`, `audit`, verify/evidence | Check structure, artifact coherence, security sharp edges, and implementation evidence. |
| Git utility / Git 工具 | `commit` | Commit only files related to one change; it does not move lifecycle state. |

## Stage reference / 階段參考

### onboard

- **Purpose / 目的:** Derive current-behavior canonical specs from existing code and tests.
- **Use / 使用:** An adopted codebase has no specs, or uncovered capabilities need gap-filling.
- **Skip / 跳過:** Canonical coverage is adequate, or the request describes new behavior.
- **Input / 輸入:** README, entry points, source, tests, configuration, and a user-confirmed capability map.
- **Outputs / 產物:** `openspec/specs/<capability>/spec.md` directly; no change is created.
- **Claude:** `/speclink-onboard [scope]`.
- **Codex:** `$speclink-onboard [scope]`.
- **CLI/Host:** There is no `speclink onboard` subcommand. The Agent writes canonical specs after investigation, then runs `speclink validate --specs --all --strict`.
- **Done / 完成:** The user confirms capability boundaries, specs cite observable evidence, and strict validation passes.
- **Next / 下一步:** Use `propose` for new behavior or `discuss` first when it is fuzzy.
- **Recover / 恢復:** If an existing spec must change, open a change instead of rewriting it in onboard.

### discuss

- **Purpose / 目的:** Converge on a trade-off and preserve the rationale round by round.
- **Use / 使用:** Requirements are fuzzy, several designs are plausible, or a verdict is needed.
- **Skip / 跳過:** The user only wants understanding, or requirements are already clear.
- **Input / 輸入:** One focused topic, current code/spec context, and the decision to settle. The topic may also be a document path (a hand-written plan, a plan-mode output, or any readable doc) — its claims are then triaged one by one against the codebase.
- **Outputs / 產物:** Context, Rounds, and Conclusion in `openspec/discussions/<slug>.md`.
- **Claude:** `/speclink-discuss <topic>`.
- **Codex:** `$speclink-discuss <topic>`.
- **CLI/Host:** `speclink discuss new/context/add-round/conclude`; after conclusion choose `promote`, `link`, `seal`, or `archive` as described below.
- **Done / 完成:** Conclusion records Decision, Rationale, Rejected alternatives, Deferred, Capture to, and Next.
- **Next / 下一步:** Create a complete new change, create a fast scaffold, reflect into an existing change, or archive a “do not implement” decision.
- **Recover / 恢復:** A substantive discussion should conclude and archive. Use `discuss discard` only when no useful reasoning was produced.

### propose

- **Purpose / 目的:** Create a change and every artifact required by its schema for implementation.
- **Use / 使用:** A clear new requirement, or a concluded discussion becoming a complete proposal.
- **Skip / 跳過:** Direct questions, current-behavior onboarding, or new context for an existing change.
- **Input / 輸入:** A clear requirement, a concluded discussion slug, or a document path via `--from-doc`.
- **Outputs / 產物:** Change metadata, proposal, delta specs, tasks, and design when its condition applies. The schema DAG and `applyRequires` determine the actual set.
- **Claude:** `/speclink-propose <change>`, `/speclink-propose --from-discussion <slug>`, or `/speclink-propose --from-doc <path>`.
- **Codex:** `$speclink-propose <change>`, `$speclink-propose --from-discussion <slug>`, or `$speclink-propose --from-doc <path>`.
- **CLI/Host:** `speclink new change`, `speclink instructions <artifact> --json`, `speclink new artifact ... --stdin`, `speclink analyze`, and `speclink validate`.
- **Done / 完成:** `speclink status --change <name> --json` shows all `applyRequires` artifacts complete; analyze has no Critical/Warning and validation passes.
- **Next / 下一步:** The user decides when to invoke `apply`.
- **Recover / 恢復:** After `discuss promote`, run propose for the same change to finish artifacts. If requirements remain unclear, return to discuss.

### apply

- **Purpose / 目的:** Implement tasks against artifacts and the implementation contract, verifying each completion.
- **Use / 使用:** All `applyRequires` artifacts are complete.
- **Skip / 跳過:** Artifacts are missing, requirements are changing, or dormant work has not been checked with drift.
- **Input / 輸入:** Proposal, design when present, delta specs, tasks, and the current workspace.
- **Outputs / 產物:** Code/document changes, tests or checks, checked tasks, and touched-file evidence.
- **Claude:** `/speclink-apply <change>`.
- **Codex:** `$speclink-apply <change>`.
- **CLI/Host:** The Agent reads `speclink instructions apply --change <name> --json`; each verified task ends with `speclink task done --change <name> <id>`.
- **Done / 完成:** Every task behavior, contract item, and verification target passes; apply instructions return `state: all_done`.
- **Next / 下一步:** After quality and implementation checks, `archive`. When requirements change, `ingest` first.
- **Recover / 恢復:** Use `speclink task undone` after rolling back a task. Refresh apply instructions if a Remote Context Projection is stale or modified.

### ingest

- **Purpose / 目的:** Merge new conversation, plans, external documents, or discussion decisions into an existing change.
- **Use / 使用:** Requirements/context changes during implementation, or a discussion conclusion belongs in an existing change.
- **Skip / 跳過:** Pure implementation with no artifact change, or no active change exists.
- **Input / 輸入:** Existing change plus new context; link a discussion first when applicable.
- **Outputs / 產物:** Merged proposal/design/spec/task updates while preserving completed tasks.
- **Claude:** `/speclink-ingest <change>`.
- **Codex:** `$speclink-ingest <change>`.
- **CLI/Host:** Read `speclink instructions ... --json` per artifact, then run analyze/validate. After linked discussion content lands, run `speclink discuss seal <slug> <change>`.
- **Done / 完成:** New context is reflected in every affected artifact, completed tasks are unchanged, analysis/validation pass, and a linked discussion is sealed.
- **Next / 下一步:** Return to `apply`.
- **Recover / 恢復:** Refresh obsolete assumptions before resuming. Never seal a link whose content has not landed.

### drift

- **Purpose / 目的:** Compare an idle change with current code, design anchors, touched files, and basis.
- **Use / 使用:** Resuming after a pause or when external commits may overlap the change.
- **Skip / 跳過:** Short continuous apply work whose basis has not changed.
- **Input / 輸入:** Change artifacts, Git history, current code, and evidence.
- **Outputs / 產物:** Light/Moderate/Heavy drift report and one recommended next command.
- **Claude:** `/speclink-drift <change>`.
- **Codex:** `$speclink-drift <change>`.
- **CLI/Host:** `speclink drift <change> --json`.
- **Done / 完成:** The report covers dormancy, broken anchors, task collisions, and the recommended route.
- **Next / 下一步:** Light usually returns to apply. Stale requirements or deltas route to ingest; Heavy drift refreshes artifacts first.
- **Recover / 恢復:** Preserve unknown external changes; never solve drift by resetting or overwriting the user's worktree.

### validate

- **Purpose / 目的:** Check change/spec structure, required fields, and schema rules.
- **Use / 使用:** After proposal or artifact updates, before archive, and during documentation acceptance.
- **Skip / 跳過:** Do not skip it before delivery; exploratory reading alone needs no run.
- **Input / 輸入:** Change name, spec, or `--all` scope.
- **Outputs / 產物:** Valid/invalid result, optionally JSON.
- **Claude:** No standalone generated skill; proposal/ingest/archive workflows call it.
- **Codex:** No standalone generated skill; use the CLI directly.
- **CLI/Host:** `speclink validate <change>`; all canonical specs use `speclink validate --specs --all --strict`.
- **Done / 完成:** Exit code 0 and a valid result for the target.
- **Next / 下一步:** Analyze/verify implementation, or proceed toward archive.
- **Recover / 恢復:** Fix artifacts and rerun; do not hide failures with `--no-validate`.

### analyze

- **Purpose / 目的:** Check Coverage, Consistency, Ambiguity, and Gaps across proposal, design, specs, and tasks.
- **Use / 使用:** After proposal/ingest and during final artifact regression.
- **Skip / 跳過:** It may be skipped for read-only queries; it is not a code test.
- **Input / 輸入:** One active change.
- **Outputs / 產物:** Four-dimension findings with severity, location, and recommendation.
- **Claude:** No standalone generated skill; artifact workflows call it.
- **Codex:** No standalone generated skill; use the CLI directly.
- **CLI/Host:** `speclink analyze <change> --json`.
- **Done / 完成:** No Critical/Warning; explicitly assess whether Suggestions affect delivery.
- **Next / 下一步:** Fix artifacts, apply, or final verification.
- **Recover / 恢復:** Fix Critical artifact contracts before implementation.

### audit

- **Purpose / 目的:** Review changed code for dangerous defaults, type confusion, and silent failures.
- **Use / 使用:** Security-sensitive APIs, configuration, identity, Store/Server boundaries, or `audit: true`.
- **Skip / 跳過:** Pure documentation with no interface or security-semantic change.
- **Input / 輸入:** Change-specific diff, design, and specs.
- **Outputs / 產物:** Severity-ordered sharp-edge findings; no lifecycle state change.
- **Claude:** `/speclink-audit <change>`.
- **Codex:** `$speclink-audit <change>`.
- **CLI/Host:** There is no `speclink audit` subcommand; the skill reads artifacts and diff.
- **Done / 完成:** Each finding has a location, misuse path, and remedy, or the report explicitly has no findings.
- **Next / 下一步:** Fix and retest/apply, or proceed toward archive.
- **Recover / 恢復:** “Caller responsibility” is not a reason to keep a dangerous interface.

### verify and evidence

- **Purpose / 目的:** Compare implementation with tasks, delta specs, and design, preserving completion evidence.
- **Use / 使用:** After apply and before archive.
- **Skip / 跳過:** A no-checkout environment can only run observable artifact/server checks and must state that limitation.
- **Input / 輸入:** Change artifacts, diff, tests, and task evidence.
- **Outputs / 產物:** Conformance conclusion, test results, and `task done` evidence; remote evidence has a known gap.
- **Claude:** This repository has no generated `/speclink-verify`.
- **Codex:** This repository has no generated `$speclink-verify`.
- **CLI/Host:** Currently combine `speclink task done`, validate/analyze, and project tests. An engine verify asset exists but is not an installed entry point.
- **Done / 完成:** Every Requirement/Scenario and task contract has observable evidence, with limitations recorded.
- **Next / 下一步:** Archive when green; route requirement divergence to ingest and implementation gaps to apply.
- **Recover / 恢復:** Never present artifact-only validate/analyze success as code correctness.

### commit

- **Purpose / 目的:** Selectively commit artifacts and implementation files related to one Speclink change.
- **Use / 使用:** A traceable, change-scoped Git commit is desired.
- **Skip / 跳過:** The user has another commit strategy or has not confirmed the set.
- **Input / 輸入:** Change name, Git status, touched files, and task progress.
- **Outputs / 產物:** User-confirmed selective staging and a Git commit.
- **Claude:** `/speclink-commit <change>`.
- **Codex:** `$speclink-commit <change>`.
- **CLI/Host:** The skill combines Speclink status/artifact reads with Git and never uses `git add .` or `git add -A`.
- **Done / 完成:** The commit contains only confirmed change files and reports its hash/message.
- **Next / 下一步:** Continue apply or archive when complete; commit is not a replacement for archive.
- **Recover / 恢復:** Exclude unrelated changes and reconfirm; never overwrite or clean them.

### archive

- **Purpose / 目的:** Merge delta specs into canonical specs and archive the completed change and related discussion.
- **Use / 使用:** All tasks are done, artifacts are valid, assumptions are current, and required verification passes.
- **Skip / 跳過:** Tasks remain, deltas are stale, verification fails, or requirements are still changing.
- **Input / 輸入:** Ready change, complete final-state deltas, and completion evidence.
- **Outputs / 產物:** Updated canonical specs and an `openspec/changes/archive/` record. The related discussion co-archives when its last surviving change archives.
- **Claude:** `/speclink-archive <change>`.
- **Codex:** `$speclink-archive <change>`.
- **CLI/Host:** `speclink archive <change>`; do not use `--no-validate` or `--mark-tasks-complete` to bypass unfinished work.
- **Done / 完成:** The CLI succeeds, canonical delta counts are correct, and the active change moves to archive.
- **Next / 下一步:** Use a change-scoped commit when the archive result should be committed.
- **Recover / 恢復:** Normalize incomplete deltas first. Route stale assumptions through drift/ingest instead of forcing archive.

## Discussion outcomes / 討論結論分流

| Outcome / 結論去向 | Use when / 使用時機 | Command or skill / 呼叫 | Result / 結果 | Required next step / 必要下一步 |
| --- | --- | --- | --- | --- |
| New change, complete proposal / 新 change、完整提案 | The conclusion should immediately become all required artifacts. | `$speclink-propose --from-discussion <slug>` (Claude: `/speclink-propose`) | Creates and links a change, then runs the complete artifact workflow. | The user decides when to apply after artifacts are green. |
| New change, fast scaffold / 新 change、快速轉為變更骨架 | A change identity is needed now and full proposal work can follow. | `speclink discuss promote <slug> [--name <change>]` | Creates the change, prefills proposal Why, links both sides, and marks the discussion promoted. It is not apply-ready. | Run propose for that change to complete schema-required artifacts. |
| Existing change / 既有 change | The conclusion updates a change already in progress. | `speclink discuss link <slug> <change>` → `$speclink-ingest <change>` → `speclink discuss seal <slug> <change>` | Link only forges the change-side source chain; ingest lands content; seal then marks the discussion promoted. | Return to apply. |
| Do not implement / 決定不實作 | Substantive reasoning concluded that no change should be made. | `speclink discuss archive <slug>` | Preserves rationale without creating an empty change. | None; a later topic can open a new discussion. |

One discussion can fan out into several changes and accumulates names in `promoted_to`. It co-archives when its last surviving related change archives. Never seal immediately after link: seal asserts that the decision has already landed in artifacts.

## Recovery paths / 恢復路徑

| Symptom / 症狀 | Route / 恢復路徑 |
| --- | --- |
| Only a proposal scaffold exists after promote | Run propose for the same change; do not apply directly. |
| A conclusion belongs in an existing change | `link → ingest → seal`; all three are required. |
| A change was idle | Drift first; Light returns to apply, stale assumptions route to ingest. |
| Requirements change during implementation | Ingest, rerun analyze/validate, then resume apply. |
| Apply reports a missing artifact | Return to propose and finish the `applyRequires` chain. |
| A task was checked by mistake or implementation rolled back | `speclink task undone --change <name> <id>`. |
| Context Projection is STALE or modified | Do not edit the projection; reacquire instructions to refresh it. |
| Analyze reports Critical | Fix artifact coverage/consistency/gaps before implementation. |
| Archive reports stale or incomplete deltas | Drift/ingest and normalize deltas, then validate again. |

## Call layers / 呼叫層級

| Layer / 層級 | Responsibility / 責任 | Example / 範例 |
| --- | --- | --- |
| Speclink Skill | Workflow knowledge: when the Agent reads context, creates or validates artifacts, and stops. | Claude `/speclink-propose`; Codex `$speclink-propose` |
| `speclink` CLI | Local/Remote command adapter for status, instructions, artifacts, tasks, and lifecycle verbs. | `speclink status --change demo --json` |
| Speclink Host/Runtime | Application boundary combining Engine, Store, identity, binding, revisions, transactions, and events. | Embedded Host or `speclink-server` |

A skill is not the runtime. Do not assume every Host uses the same literal call: Claude uses slash commands, Codex uses `$skill`, and CLI is a lower-level entry.

## Current limitations / 目前限制

- This repository has no generated `$speclink-verify` or `/speclink-verify`. Current verification combines project tests, `task done` evidence, validate, and analyze; see [Product Capability Status](product-status.md).
- Validate/analyze checks artifacts, not code tests or complete implementation conformance.
- Desktop Server Connections are available, while complete Desktop Remote Workspace remains Partial.
- Legacy remote REST v1 is deprecated. New work uses the current Client Protocol/Host path.

## Related documents / 相關文件

- [Local Repo Getting Started](getting-started.md)
- [Product Capability Status](product-status.md)
- [Platform architecture blueprint](platform-architecture.zh-TW.md) (Traditional Chinese)
- [Implementation refactoring roadmap](implementation-refactor-roadmap.zh-TW.md) (Traditional Chinese)

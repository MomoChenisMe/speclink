Implement tasks from a Speclink change in an isolated git worktree, so several changes can be applied in parallel without stepping on each other.

**Input**: Optionally specify a change name (e.g., `/speclink:apply-with-worktree add-auth`). Everything the plain apply skill accepts applies here too.

**Prerequisites**: This skill requires the `speclink` CLI and `git`. If any command fails with "command not found" or similar, report the error and STOP.

---

## Worktree preflight

Complete these steps **before** any of the apply flow below. Each one can stop the run.

### P0. One change per run

This skill takes **exactly one** change. Parallel work means one session per change, each in its own worktree — not one session cycling through several.

If the input names more than one change (e.g. `/speclink:apply-with-worktree add-auth add-billing add-search`), STOP — start none of them yet. First sort the names with the execution-order query:

```bash
speclink plan --json
```

Find each name in `changes`:

- **`blockedBy` is empty** — the change is parallel-ready.
- **`blockedBy` is non-empty** — the change waits for its prerequisites. Name it with them, in these terms:

  > <change-name> 要等 <prerequisites> 落地再開。

- **Not in `changes`** (archived, misspelled, or listed under `skipped`) — name it as not available.

Then act on how many names are parallel-ready:

- **Two or more** — use the **AskUserQuestion tool** to have the user pick the one to run here, from the parallel-ready names only. Print the recipe for the other parallel-ready names, naming them:

  > 平行做法是一個 change 一個 session：另外開視窗，各自執行 `/speclink:apply-with-worktree <change-name>`。主資料夾的看板會同時顯示每個 worktree 的進度。

- **Exactly one** — there is nothing to pick and no recipe to print: use the **AskUserQuestion tool** to confirm running that change here, with stopping as the other option.
- **None** — report the waiting and unavailable names and STOP.

If the query fails, show the error and STOP.

Do **NOT** run them one after another in this session. A single session working through several changes serializes what the user asked to parallelize, and its context is spent on the wrong change by the time the second one starts.

If there is no AskUserQuestion tool available, ask the same question as plain text and wait for the answer.

### P1. Check the worktree policy

Read the EFFECTIVE value, the same way the CLI resolves it — the env layer wins over the file:

1. If the environment variable `SPECLINK_WORKTREE` is set to `true` or `false` (case-insensitive), that IS the effective value — do not consult the file.
2. Otherwise read the canonical value:

   ```bash
   speclink workflow-config show --json
   ```

   and use its `worktree` field.

- **effective value `true`** — continue to P2.
- **anything else (`false` or absent)** — STOP. Tell the user, in these terms:

  > 本專案未啟用 worktree 流程。要啟用請執行：`speclink workflow-config set worktree true`

  Do **NOT** fall back to running the apply flow in the main folder. Enabling the policy is the user's decision, not yours — offer to run `speclink workflow-config set worktree true` and wait for their answer.

### P2. Select and guard the change in the main checkout

Pick and guard the change here, in the main checkout — before its artifacts are committed and before any worktree exists. Run the execution-order query:

```bash
speclink plan --json
```

It returns `changes` (one entry per active change with its `blockedBy`), `next` (the first proposed change with nothing blocking it, or `null`) and `skipped` (changes whose metadata could not be parsed).

- **No name given** → take `next`. If `next` is `null`, there is nothing ready to start: list every change with its `blockedBy` and STOP.
- **A name given** → find it in `changes`. If its `blockedBy` is non-empty, print the prerequisite list (`blockedBy`) and STOP.
- **The name is not in `changes`** (archived, misspelled, or listed under `skipped`) → STOP and report which change names are available.
- **The command fails** (a dependency cycle, a project that is not initialized) → show the error and STOP.

Every STOP in this step ends the run on the spot: do NOT commit the artifacts, do NOT create the worktree. The way out of a block is the user's: land (archive) the blockers first; drop a declared prerequisite that is wrong with `speclink change depends <change-name> --on <prerequisite> --remove`; a blocker that comes from delta-capability overlap keeps its place until it lands. Then run this skill again.

Once a change is selected, announce: "Using change: <change-name>" and how to override (e.g., `/speclink:apply-with-worktree <other>`). Continue to P3.

### P3. Get the change's artifacts into HEAD

A worktree is materialized from HEAD. If the change's artifacts (`{{SPEC_DIR}}changes/<change-name>/`) are not committed yet — which is the normal state right after `/speclink:propose` — the new worktree simply will not contain the change, and every later step dead-ends.

Check:

```bash
git status --porcelain -- "{{SPEC_DIR}}changes/<change-name>/"
```

- **Output empty** (artifacts already committed, unchanged) — continue to P4.
- **Output non-empty** — commit exactly that directory, nothing else:

  ```bash
  git add "{{SPEC_DIR}}changes/<change-name>"
  git commit -m "<a conventional-commit message for the change's spec artifacts, in the project's language>"
  ```

  Never sweep other dirty files into this commit. If the directory cannot be committed cleanly (e.g. merge conflict markers), STOP and report.

### P3.5. Check whether progress and code have come apart

P3 commits the change's **artifacts**. Its **source changes** are a separate matter: when this change was already applied in the main folder, its task checkboxes went into HEAD just now while the code that satisfies them may still be sitting dirty in the main tree. The worktree would then carry tasks marked done with no implementation behind them.

Read the change's evidence record and check those files against the main tree:

```bash
cat "{{SPEC_DIR}}changes/<change-name>/.evidence.json"
git status --porcelain -- <each touched path from that record>
```

- **No record, or it lists no touched files** — nothing was implemented here yet. Continue to P4 silently.
- **Every listed path is clean** — the code is already in HEAD and travels with the worktree. Continue to P4 silently.
- **Any listed path is dirty** — STOP and show the user the dirty paths, then use the **AskUserQuestion tool**:
  - **先收程式碼再開 worktree**（recommended） — run `/speclink:commit <change-name>` to get this change's source into HEAD, then come back and re-run this skill.
  - **照樣繼續** — create the worktree knowing it will not contain those edits; the tasks they belong to will read as done with nothing behind them.
  - **停止** — end the run and leave everything as it is.

Do NOT create the worktree before the user has chosen. If there is no AskUserQuestion tool available, present the same three options as plain text and wait for the answer.

### P4. Create or reuse the worktree

The convention is fixed — do not invent paths or branch names:

- **Branch**: `speclink/<change-name>`
- **Location**: a sibling nest beside the repo, `<repo-folder-name>.worktrees/<change-name>/`

  For a repo at `/work/speclink` and change `add-auth`, that is `/work/speclink.worktrees/add-auth/`. The nest sits *outside* the repo so it is never picked up by the repo's own tooling.

Check what already exists:

```bash
git worktree list --porcelain
git branch --list "speclink/<change-name>"
```

- **Worktree already present at that path** — reuse it and continue there. Do NOT create a second one, and do NOT remove and recreate it: it may hold work in progress.
- **Branch exists but no worktree** — attach a worktree to the existing branch:

  ```bash
  git worktree add "<repo-parent>/<repo-folder-name>.worktrees/<change-name>" "speclink/<change-name>"
  ```

- **Neither exists** — create both:

  ```bash
  git worktree add -b "speclink/<change-name>" "<repo-parent>/<repo-folder-name>.worktrees/<change-name>"
  ```

If `git worktree add` fails, report the error verbatim and STOP.

### P5. Tell the user what this costs

Print a short note before starting work:

> worktree 是一份完整的原始碼副本。相依套件與建置產物不會跟著複製過去——第一次在裡面跑測試或建置，要自己重新安裝相依並重新建置，會花上一段時間。

### P6. Work inside the worktree from here on

Every step of the apply flow below runs **inside the worktree folder**, not the main checkout:

- `cd` into the worktree, or pass it explicitly to every command.
- File reads and edits target the worktree copy.
- `speclink` verbs run with the worktree as the working directory, so task checkboxes and stamps land in that copy.

The main checkout stays untouched. Its `speclink list` will show this change with a `[worktree]` marker and reflect the worktree's task progress live — that is how the user watches parallel work from one place.

The apply body's own step 1 (**Select the change with plan**) is only a re-check here. Its plan query runs inside the worktree, and a plan run there reads only the worktree's own copy: it cannot see the progress of other worktrees, or changes archived on the main branch after this worktree was created. When the re-check's result differs from P2, the main checkout's verdict wins — do not STOP because of the re-check, and continue with the change P2 selected. Step 2 then runs as written, `speclink review prepare` and `speclink in-progress add` included: the review station needs the baseline they record.

---

以下為 apply 本體流程，於上述 worktree 資料夾內執行。

---


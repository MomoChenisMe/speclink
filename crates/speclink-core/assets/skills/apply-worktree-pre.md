Implement tasks from a Speclink change in an isolated git worktree, so several changes can be applied in parallel without stepping on each other.

**Input**: Optionally specify a change name (e.g., `/speclink:apply-with-worktree add-auth`). Everything the plain apply skill accepts applies here too.

**Prerequisites**: This skill requires the `speclink` CLI and `git`. If any command fails with "command not found" or similar, report the error and STOP.

---

## Worktree preflight

Complete these steps **before** any of the apply flow below. Each one can stop the run.

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

### P2. Confirm the change exists and is not archived

```bash
speclink list --json
```

The change must appear among the active changes. If it does not (unknown name, or already archived), STOP and report which change names are available.

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

---

以下為 apply 本體流程，於上述 worktree 資料夾內執行。

---


---
name: speclink-manual
description: "Use when a human-readable operating manual is needed, or when someone wants to be walked through how to operate the system — generates a wiki-style Markdown manual under openspec/manual/ from the canonical specs, or gives an in-conversation tour with every answer sourced."
license: MIT
compatibility: Requires speclink CLI.
metadata:
  author: speclink
  version: "v1.31.0"
  generatedBy: "Speclink"
---

Generate a human-readable operating manual from the canonical specs, or walk the user through the system in conversation. The manual is a wiki-style set of Markdown pages under `openspec/manual/`, written for someone who joined today and only wants to know how to operate the system.

**Input**: Optional arguments after `/speclink-manual`. They pick the mode:

| Arguments | Mode |
| --- | --- |
| none | **Generation** (the default) — read the specs, write `openspec/manual/*.md` |
| contain `導覽` or `tour` | **Tour** — no file is written; guide the user through the system in conversation |
| anything else | **Generation with a scope hint** — e.g. one section, one page, or "全部重生"; restate the scope you understood at the top of the summary |

**Prerequisites**: This skill requires the `speclink` CLI. If any `speclink` command fails with "command not found" or similar, report the error and STOP.

**The one-source rule**: the manual's content comes from the canonical specs under `openspec/specs/` and nothing else. NEVER read README files, a `docs/` directory, or source code as a content source — not to fill a gap, not to "confirm" wording, not for screenshots. Where the specs are silent, the manual is silent or says so. Every page cites the capabilities it was written from.

---

## Generation mode

### Step 0: Refuse on a remote-bound project

Read `.speclink.yaml` at the workspace root. No file, or a file without a `remote` section, means a local project — continue with Step 1. A `remote` section means the project is bound to a remote store and generation is not supported yet: the pages would land in this local checkout only and never reach the store. Print

```
remote 模式尚不支援手冊生成（導覽模式不受此限：/speclink-manual 導覽）
```

and STOP. Zero files are written. Tour mode is unaffected by this check.

### Step 1: Read the existing manual

If `openspec/manual/` exists, read the frontmatter of every `.md` page in it before anything else. Keep, per page: filename, `title`, `section`, `order`, `sources`, `generated`. This index drives the staleness report (Step 3) and the order-preservation rule (Step 5).

A page whose frontmatter cannot be parsed is listed in the report and skipped — never overwritten.

No manual yet → every page is new; the report says so.

### Step 2: Collect the capabilities and sort them

1. `speclink list --specs --json` — every canonical capability (`specs[].id`).
2. For each capability read its Purpose (`speclink show <capability> --item-type spec` — the `## Purpose` section comes first) and sort it into one of two bins:
   - **user-facing** — something a person operates or observes: screens, panels, commands they type, skills they call, outputs they read, files they open;
   - **engine-internal** — storage, protocol and wire contracts, host runtime, test harness, build and release plumbing, per-skill content contracts.
   Only user-facing capabilities enter the manual.
3. Purpose empty or a `TBD` placeholder → judge from the `### Requirement:` headings in the same `speclink show` output instead (there is no heading-only view; read its `## Requirements` section).

When a manual already exists, the cheap parts are still read for every capability — the Purpose (to sort it) and the `@trace updated` stamps (to judge staleness) — but the Requirement bodies are read only for the specs behind stale pages and unlisted capabilities (Step 3). Do not re-read every spec body on a second run.

### Step 3: Staleness report

Compute, from the Step 1 index and the specs:

| Term | Definition |
| --- | --- |
| **stale page** | any capability in the page's `sources` has an `@trace updated` stamp that is *after* the page's `generated`. "After" is decided by format: when both are RFC 3339 timestamps, the spec stamp must be strictly later than the page stamp (same second is not after); when either side is a plain `YYYY-MM-DD` date, compare calendar days and a same-day tie counts (an archive on the day of generation must not slip through — a timestamp's day is the day in its own offset). A value that is neither format is ignored |
| **unlisted capability** | a user-facing capability that appears in no page's `sources` |
| **orphan page** | a page with a non-empty `sources` whose capabilities have all disappeared from `openspec/specs/` — reported, NEVER deleted |

`index.md` and `about.md` (empty `sources`) are derived pages: never stale, never orphan, never "unlisted" — they are rewritten whenever any other page is added or regenerated (Step 5), keeping their `section` and `order`. The `@trace updated` stamps are read from every `<!-- @trace … updated: … -->` block inside the canonical spec file — newer archives write an RFC 3339 timestamp with offset (`2026-09-05T23:17:28+08:00`), older ones a plain date; the **stale page** rule above decides, stamp by stamp, whether any of them is after the page's `generated`.

Then decide what to write:

- **No manual yet** → write everything (Steps 4–5).
- **Manual exists** → by default regenerate only the stale pages, add a page for each unlisted capability, and rewrite `index.md` and `about.md` so the entry links, the contradiction list and the compilation stamp reflect the new set. A page whose regenerated content would be byte-identical to the file on disk counts as untouched, not regenerated — do not rewrite it. Untouched pages stay byte-identical. Regenerate everything only when the user explicitly asks for it (e.g. "全部重生"); even then, existing `section` and `order` are preserved.
- **Manual exists, nothing stale, nothing unlisted** → write nothing, `index.md` and `about.md` included. Report that the manual is up to date and stop. A scope hint is the user's explicit request for that scope and overrides this: regenerate what the hint names.

### Step 4: Choose the journey backbone

The manual is organized as journeys — what a new user does, in order — not as a list of capabilities. Take the backbone from the first source that exists:

1. **Scenario-script specs** — acceptance scripts, walkthroughs, end-to-end journeys: their chapter order and stations become the manual's.
2. **Routing / hand-off specs** — skill hand-off tables and entry-routing contracts: the edges give the order.
3. **User-documentation specs** — the structure of the user docs canon.
4. **None of the above** — rebuild the journeys by functional domain from the user-facing capability specs. The about page then states that the journeys were rebuilt (重建), not transcribed.

When 1–3 applies, the about page names the specs the journeys were transcribed from (轉寫自).

### Step 5: Write the pages — the page contract

This is the `manual-pages` contract. Every reader — the desktop manual page, a static-site generator, a person — relies on it, so follow it exactly.

**Location and filename**: every page is a file directly under `openspec/manual/` (create the directory when missing). Filenames are kebab-case ASCII ending in `.md` — `first-login.md`; never spaces, capitals, or non-ASCII. Nothing is written anywhere else.

**Frontmatter** — YAML, exactly these fields and no others:

| Field | Type | Required | Meaning |
| --- | --- | --- | --- |
| `title` | string | yes | the page's human title |
| `section` | string | yes | sidebar section name |
| `order` | integer | yes | global sort key, unique across the manual; step by 10 by convention |
| `keywords` | string array | no | search terms |
| `sources` | capability-name array | yes | the canonical specs this page was written from; `[]` only on the index and about pages |
| `generated` | RFC 3339 timestamp with offset | yes | when this page was last generated, to the second, with the local offset (`2026-09-05T23:31:00+08:00`); the desktop reader also accepts the older plain `YYYY-MM-DD` form — never write that form anew |

Pages are ordered by `order` alone; sections are ordered by the smallest `order` inside each. Example:

```yaml
---
title: 第一次登入
section: 開始使用
order: 20
keywords: [登入, github, 審核]
sources: [github-oauth, user-pending-blocked-pages]
generated: 2026-09-05T23:31:00+08:00
---
```

Take the `generated` value from the clock at the moment you write the page, never by hand. Pick the first command that works on the machine:

```bash
python3 -c 'import datetime; print(datetime.datetime.now().astimezone().isoformat(timespec="seconds"))'   # local offset, e.g. 2026-09-05T23:31:00+08:00
date -u +%Y-%m-%dT%H:%M:%SZ                                                                                 # macOS / Linux without python3: UTC with Z
```

```powershell
(Get-Date).ToString("yyyy-MM-ddTHH:mm:ssK")                                                                 # Windows PowerShell: local offset
```

Before writing, check the value matches `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(Z|[+-]\d{2}:\d{2})$` — an offset without a colon (`+0800`) is NOT accepted by the reader, which would then treat the page as never stale without any error. Every page written in one run may share the same stamp.

**Body conventions**:

- GitHub Flavored Markdown. Callouts use GitHub Alert syntax only: `> [!NOTE]`, `> [!TIP]`, `> [!WARNING]`, `> [!CAUTION]`.
- Links between pages use the relative filename: `[認識畫面](layout.md)`.
- The last non-empty paragraph of every page is the source line: `**出處**：` followed by the same capability names as `sources`, each in backticks — e.g. **出處**：`github-oauth`、`user-pending-blocked-pages`.
- No HTML tags. No `--json` field names, flags, or code details — unless the text is a command or skill name the user actually types.
- Write the prose in the workspace's locale (`locale` from `speclink workflow-config show --json`). The contract literals — `**出處**：` and the about page's title — stay verbatim.
- Write for a reader who joined today: what they see, what they press, what happens next, and where it can go wrong — quoting button labels, confirmation texts and error exits exactly as the specs state them.
- When two specs disagree, follow the one whose latest `@trace updated` stamp is after the other's (the same after-rule as the **stale page** definition; a tie keeps the one archived later by directory name) in the page body and list the conflict on the about page. Never resolve a contradiction silently.

**Two required pages**:

- `index.md` — `title` is the manual's name, `section` is the first section, `order` is the smallest in the manual. Body: one sentence positioning the system, at most three core concepts, and entry links grouped by role.
- `about.md` — `title` is `本手冊的來源`, `order` is the largest in the manual. Body states: the source scope is `openspec/specs/`; the journey backbone (transcribed from which specs, or rebuilt); the list of contradictions found between newer and older descriptions inside the specs (write `未發現` when there are none); known limits (no screenshots — the running product wins); and the compilation date.

When no capability is user-facing, still write both pages: `about.md` says `尚無可入冊的使用者面向能力` and no other page exists.

**Order preservation on regeneration**:

- A page whose filename already exists keeps its `section` and `order` verbatim (unless the user explicitly asked to reorder).
- A new page takes an integer between its neighbours (between 20 and 30 → 25); existing pages are never renumbered. When no integer fits — neighbours 20 and 21, or a page that must land after the last page while `about.md` has to stay the maximum — do NOT renumber on your own: leave that page out of this run, list it in the report, and ask the user for a reorder; their explicit request is what allows renumbering.
- Pages that are not being regenerated are not touched at all — byte-identical.
- Orphan pages (Step 3) stay on disk and appear in the report.

### Step 6: Report

End with a summary in the conversation:

```
## 手冊生成摘要

- 新增：N 頁（<filenames>）
- 重生：N 頁（<filenames>）
- 未動：N 頁
- 可能過期：<list, or 無>
- 未入冊能力：<list, or 無>
- about 頁記錄的矛盾：N 條
- 跳過（frontmatter 無法解析）／來源已消失：<list, or 無>

手冊異動建議以一般 git commit 收尾。
```

When a scope hint was given, restate the scope you understood first. When nothing was written, say the manual is already up to date. The commit line is a suggestion only — never run the commit yourself.

---

## Tour mode

Tour mode writes NOTHING — no manual pages, no notes, no scratch files. It is a conversation.

1. **Manual exists** (`openspec/manual/` present): read every page's frontmatter as the index.
   - Ask exactly one question first: which role the user has (e.g. developer running SDD through an agent, product owner, someone who joined today), so you can pick the entry from the index page's role links.
   - Then walk the journey in `section` / `order` order, one station at a time, reading the page body as you go; answer questions as they come.
   - Every answer names its source: the capability (from the page's `sources`) or the page's filename.
2. **No manual**: say so — `尚無手冊，改以規格直接導覽` — then tour from the specs: `speclink list --specs` for the map, `speclink show <capability> --item-type spec` for each station, sources cited by capability name.
3. **Remote-bound project**: tour mode proceeds as usual, from an existing manual or from the specs.

When the tour ends you may suggest running generation mode (`/speclink-manual`) to produce the manual — a suggestion only. NEVER invoke another skill from here.

---

## Guardrails

- Specs are the only content source. README, docs and code are off-limits for manual content — in both modes.
- Generation writes only under `openspec/manual/`; tour writes nothing; a remote-bound project gets no generation at all.
- Never delete a page. Never renumber an existing page unless the user explicitly asks for a reorder. Never overwrite a page whose frontmatter you could not parse.
- Frontmatter has exactly the six fields above. The about page's title and the `**出處**：` line are contract literals.
- Contradictions inside the specs are recorded on the about page, never silently resolved.
- Tool skill: no fixed next step. The commit line in the summary and the tour's closing suggestion are suggestions — this skill never runs a commit or another skill.

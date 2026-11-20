# backup-format-1.tar

A backup in backup format version 1, made once by speclink-server 0.12.0
(commit 637da561, before the backup code moved to the store port). The
tests use it to check that this version can verify and restore a backup
of an older version. It holds only made-up data. Do not make it again:
the code that wrote this format is gone.

## How it was made

A throwaway test filled the in-memory store of 0.12.0 and a
SQLite identity database, checked the data with the CLI (`list`,
`list --specs`, `show`, `plan`) against the 0.12.0 server, and called
`speclink_server::backup::create`. `speclink-server verify-backup` of
0.12.0 answers 0 for the file.

## Identity

- One project `demo` (name `Demo`) with two repos: `backend`, `web`.
- One user: email `fixture-member@example.com`, display `Fixture Member`,
  a member (editor) of `demo`, not an admin.
- One access key of that user. Its plaintext is below; the backup holds
  only its hash. It opens nothing but a server restored from this file.

      spk_pat_e76cb79b426bb66cff601ce25394561dbe2b3b16656b7612ce2a42400b7e9400

- No audit record.

## Scope demo/backend: 19 documents

| Kind | Documents |
| --- | --- |
| change metadata | `add-auth`, `add-billing` (depends on `add-auth`) |
| change artifacts | `add-auth`: proposal.md, design.md, tasks.md, specs/auth/spec.md; `add-billing`: proposal.md, tasks.md |
| evidence | `add-auth` |
| canonical spec | `accounts` |
| discussion | `rate-limiting` (open), `2026-10-20-naming` (archived) |
| archived change | `2026-10-25-initial-setup`: .meta.yaml, proposal.md, tasks.md, specs/accounts/spec.md |
| workflow config | one |
| glossary (LANGUAGE) | one |
| board order | changes `add-auth` = a0, `add-billing` = a1; discussions `rate-limiting` = a0 |

What the 0.12.0 CLI showed for this scope:

- `list`: `add-auth` (1 of 2 tasks, created 2026-11-01, delta capability
  `auth`), then `add-billing` (0 of 1 tasks, created 2026-11-03).
- `list --specs`: `accounts`.
- `show add-auth`: schema `spec-driven`, delta spec `auth/spec.md`.
- `plan`: wave 1 is `add-auth`, wave 2 is `add-billing` (blocked by
  `add-auth`).

## Scope demo/web: 3 documents

- change metadata and proposal.md of `new-landing` (created 2026-11-05)
- the workflow config

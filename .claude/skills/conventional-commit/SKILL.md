---
name: conventional-commit
description: Write git commit messages following the Conventional Commits v1.0.0 specification. ALWAYS use this skill when committing code changes, writing commit messages, reviewing or validating commit message format — even when the user simply asks to "commit this" or "write a commit message" without explicitly mentioning conventional commits. Also triggers when the user mentions conventional commits, commit message standards, semantic commit messages, structured commit format, commit types (feat, fix, refactor, chore, etc.), commit scopes, BREAKING CHANGE notation, git trailers, or asks whether a commit message is valid. Supports type selection, optional scope, ! suffix and BREAKING CHANGE footer, multi-paragraph body, git trailer footers, and Semantic Versioning correlation.
---

# Conventional Commit

Write git commit messages following the [Conventional Commits v1.0.0](https://www.conventionalcommits.org/en/v1.0.0/) specification. For the complete formal specification (16 rules with RFC 2119 keywords), see [references/specification.md](references/specification.md).

## Commit Message Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Structure Rules

- The **first line** (header) contains: type, optional scope, optional `!`, colon, space, and description.
- A **blank line** MUST separate the header from the body (if body is present).
- A **blank line** MUST separate the body from the footer section (if footers are present).
- The header (type + scope + description combined) SHOULD be kept under **72 characters**.

## Types

| Type | Purpose | SemVer Impact | When to Use |
|---|---|---|---|
| `feat` | New feature | MINOR (x.**Y**.z) | Adding new functionality, capabilities, or user-facing behavior |
| `fix` | Bug fix | PATCH (x.y.**Z**) | Correcting incorrect behavior, fixing errors or crashes |
| `build` | Build system / external deps | — | Changes to build tools, npm scripts, webpack, bundler configs, dependency updates |
| `chore` | Maintenance tasks | — | Routine tasks that don't modify src or test files (e.g., updating .gitignore, tooling configs) |
| `ci` | CI configuration | — | Changes to CI/CD pipeline files (GitHub Actions, Jenkins, CircleCI, etc.) |
| `docs` | Documentation only | — | README, JSDoc, API docs, comments, wikis — no code logic change |
| `style` | Code formatting | — | Whitespace, semicolons, indentation, linting fixes — no logic change. NOT CSS styling |
| `refactor` | Code restructuring | — | Restructuring code without changing external behavior (neither a fix nor a feature) |
| `perf` | Performance improvement | — | Changes that improve performance (faster algorithms, caching, lazy loading) |
| `test` | Tests | — | Adding, correcting, or refactoring tests — no production code change |

- Types other than `feat` and `fix` do NOT directly trigger a SemVer version bump, but MAY appear in changelogs.
- Types MUST NOT be treated as case sensitive by tooling, but **lowercase is the most common convention**.
- Custom types beyond this list are allowed by the spec (e.g., `revert`, `merge`), but the above are the standard set.

## Scope

An optional noun in parentheses describing the codebase section affected.

**Format**: `<type>(<scope>): <description>`

**Rules**:
- Scope MUST be a **noun** describing a section of the codebase.
- Scope MUST be surrounded by **parentheses**.
- Keep scope names **consistent** within a project.

**Common scope conventions**:
- Module/package name: `feat(auth):`, `fix(parser):`, `refactor(api):`
- Component name: `fix(button):`, `style(navbar):`
- Layer: `feat(backend):`, `fix(frontend):`
- File/area: `docs(readme):`, `ci(github-actions):`

**Examples**:
```
feat(auth): add OAuth2 login support
fix(parser): handle nested brackets correctly
docs(readme): update installation steps
ci(github-actions): add Node 20 to test matrix
refactor(db): extract connection pooling into separate module
```

## BREAKING CHANGE

Breaking changes indicate incompatible API changes, correlating with **MAJOR** in SemVer (**X**.y.z).

### Two Mechanisms (can coexist)

**1. Append `!` after type/scope** (in the header):
```
feat!: remove deprecated API endpoints
feat(api)!: change response format from array to object
fix!: drop support for Node 14
```

**2. Footer with `BREAKING CHANGE:`** (in the footer section):
```
feat(api): change user response format

BREAKING CHANGE: the /users endpoint now returns an object with a
"data" key instead of a flat array
```

### Rules

- `BREAKING CHANGE` in footers MUST be **uppercase**.
- `BREAKING-CHANGE` (with hyphen) is a valid synonym for `BREAKING CHANGE` in footers.
- `!` can be used with **any** type: `feat!:`, `fix!:`, `refactor!:`, `chore!:`, `build!:`, etc.
- If `!` is used, the `BREAKING CHANGE:` footer MAY be omitted — the description after `!:` SHALL describe the breaking change.
- Both mechanisms can be used together for emphasis:
  ```
  refactor!: drop support for Node 14

  BREAKING CHANGE: minimum Node.js version is now 18.x
  ```

## Footer Format

Footers follow the [git trailer convention](https://git-scm.com/docs/git-interpret-trailers).

### Format

```
<token><separator><value>
```

- **Token**: A word or hyphenated words (e.g., `Reviewed-by`, `Refs`, `Closes`).
  - Token MUST use **`-` (hyphen) in place of whitespace** characters.
  - Exception: `BREAKING CHANGE` (with space) is allowed as a token.
- **Separator**: Either `:<space>` (colon + space) or `<space>#` (space + hash).
- **Value**: A string that MAY contain spaces and newlines.
  - Parsing terminates when the next valid token/separator pair is observed.

### Common Footer Tokens

| Token | Separator | Purpose | Example |
|---|---|---|---|
| `BREAKING CHANGE` | `: ` | Describe breaking API change | `BREAKING CHANGE: env vars now override config` |
| `BREAKING-CHANGE` | `: ` | Synonym for BREAKING CHANGE | `BREAKING-CHANGE: removed legacy auth` |
| `Refs` | `: ` | Reference issues/PRs | `Refs: #123, #456` |
| `Closes` | `: ` | Auto-close issues | `Closes: #789` |
| `Fixes` | ` #` | Auto-close issues (alt) | `Fixes #101` |
| `Reviewed-by` | `: ` | Code reviewer attribution | `Reviewed-by: Alice <alice@example.com>` |
| `Acked-by` | `: ` | Acknowledgment | `Acked-by: Bob` |
| `Co-Authored-By` | `: ` | Co-author attribution | `Co-Authored-By: Name <email>` |
| `Signed-off-by` | `: ` | Sign-off (DCO) | `Signed-off-by: Dev <dev@example.com>` |

### Multi-Footer Example

```
feat(api): add user endpoint

Add GET /users and POST /users endpoints with pagination support.
Rate limiting is set to 100 requests per minute.

Reviewed-by: Alice <alice@example.com>
Refs: #123, #456
Co-Authored-By: Bob <bob@example.com>
```

## Writing Guidelines

1. **Imperative mood** in the description: "add feature" not "added feature" or "adds feature". Think of it as completing the sentence: "If applied, this commit will _[your description]_."
2. **No period** at the end of the description line.
3. **Lowercase** the first letter of the description (after the colon and space).
4. **Under 72 characters** for the full header line (type + scope + description).
5. **Body explains WHY**, not WHAT: The diff shows what changed; the body should explain the motivation, context, and reasoning behind the change.
6. **One logical change per commit**: Do not mix unrelated changes. If a commit touches authentication and also fixes a typo in an unrelated file, split them into two commits.
7. **Blank line** between header and body, and between body and footers — this is required by the spec and by git tooling.
8. **Wrap body text** at 72 characters per line for readability in terminal-based git tools.
9. **Follow the project's language conventions** for description and body. Check the project's CLAUDE.md, existing commit history, or team guidelines to determine the expected language (e.g., English, Chinese, Japanese). When the project specifies a language, use it consistently in both the description and body — types, scopes, and footer tokens always remain in English.

## Examples

### Minimal (description only)

```
docs: correct typo in contributing guide
```

```
chore: update dependencies to latest versions
```

```
style: fix indentation in auth module
```

```
test: add unit tests for password validation
```

### With Scope

```
feat(auth): add OAuth2 login support
```

```
fix(parser): handle empty input without crashing
```

```
perf(images): add lazy loading for gallery thumbnails
```

```
refactor(db): extract connection pooling into separate module
```

### With Body

```
feat(auth): add OAuth2 login support

Implement OAuth2 authorization code flow to allow users to
authenticate with GitHub and Google accounts. This replaces the
legacy username/password authentication for third-party providers.
```

```
fix(parser): handle empty input without crashing

Return empty result instead of throwing NullPointerException
when parser receives null or empty string input. The root cause
was a missing null check in the tokenizer's init method.
```

### With Body and Footers

```
feat(api): add user search endpoint

Add GET /users/search endpoint supporting full-text search
across name and email fields with pagination. Results are
sorted by relevance score by default.

Refs: #234
Reviewed-by: Alice <alice@example.com>
```

```
fix(auth): prevent session fixation attack

Regenerate session ID after successful authentication to prevent
session fixation attacks. The old session data is migrated to the
new session automatically.

Closes: #891
Signed-off-by: Security Team <security@example.com>
```

### BREAKING CHANGE

```
feat!: drop support for Node 14

BREAKING CHANGE: minimum Node.js version is now 18.x.
Update your CI/CD pipelines and deployment environments accordingly.
```

```
feat(api)!: change response envelope format

Wrap all API responses in a standard envelope with "data", "meta",
and "errors" keys. Previously, endpoints returned raw data arrays.

BREAKING CHANGE: all API responses now use { data, meta, errors }
envelope format instead of returning raw arrays.

Refs: #567
```

### Revert

```
revert: let us never again speak of the noodle incident

Refs: 676104e, a]215868
```

## Complete Specification

For the complete formal specification including all 16 rules (with RFC 2119 MUST/MAY/MUST NOT keywords), BNF-style grammar definition, SemVer correlation details, and FAQ, read [references/specification.md](references/specification.md).

# Contributing to Speclink

Thank you for your help. This guide tells you how to set up the repo, which tests to run, and how to send a pull request.

You can write issues and pull requests in English or Traditional Chinese.

## Report a bug or ask for a feature

- Use the issue forms: **Bug report** or **Feature request**.
- Search the open issues first. Add your case to an issue that already exists.
- Do not put access tokens, setup tokens, or internal URLs in an issue.
- Do not report a security problem in a public issue. Follow [SECURITY.md](SECURITY.md).

## Set up the repo

You need:

- A stable Rust toolchain.
- Node.js 21 or later, and npm.

Then install the frontend dependencies and build the server web console once:

```bash
npm install
npm run build -w apps/server-web
```

The five one-command entry points (`npm run dev`, `npm run dev:server`, `npm run dev:desktop`, `npm run dev:reset`, and `npm run cli`) are in [Development](docs/development.md). That document also lists the system packages that the desktop app needs.

## Run the tests

Run the tests for each area that you change. CI runs the same commands.

| You changed | Run |
| --- | --- |
| Anything (always run this; it also checks the docs) | `node --test scripts/*.test.mjs scripts/*/*.test.mjs` |
| One Rust crate | `cargo test -p <crate>` |
| `packages/ui` | `npm test -w packages/ui` |
| `apps/desktop` | `npm test -w apps/desktop` |
| `apps/server-web` | `npm test -w apps/server-web` |
| The Node SDK | `npm --prefix crates/adapters/speclink-node run build`, then `npm --prefix crates/adapters/speclink-node test` |

`npm run test:all` runs everything. It takes a long time, and a fresh clone needs three build steps first. See [Tests](docs/development.md#tests).

If you change the human output or the `--json` output of a command, update the golden files in the same pull request. The golden tests are in `crates/engine/speclink-core/tests/golden`.

## Describe a behavior change with Speclink

Speclink builds Speclink. The specs of the project are in `speclink/specs/`, and each behavior change has a folder in `speclink/changes/`.

- **A small fix** (a typo, a clear bug, a test): send a pull request with a test. You do not need a change folder.
- **A new behavior, or a change to an existing behavior**: open an issue first, so that we agree on the direction. Then create a change with your agent (`/speclink-propose <change-name>`, or `$speclink-propose <change-name>` in Codex). The change holds the proposal, the delta specs, and the task list. Implement it with `/speclink-apply`.

If you do not use an AI agent, the CLI does the same work: `speclink new change <change-name>`. See [Getting started](docs/getting-started.md) and the [SDD workflow](docs/workflow.md).

## Write commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```text
<type>(<scope>): <description>
```

- Common types: `feat`, `fix`, `docs`, `refactor`, `test`, `build`, `chore`.
- The scope is the area that you change, for example `cli`, `desktop`, `server-web`, or `docs`.
- A commit that belongs to a Speclink change uses the change name as the scope: `speclink(<change-name>): <description>`.
- The maintainers write descriptions in Traditional Chinese. English descriptions are welcome too.

## Send a pull request

1. Fork the repo and create a branch from `main`.
2. Make the change and add tests.
3. Run the tests from [Run the tests](#run-the-tests).
4. If the change affects users, update the docs. Each document has an English version (`<name>.md`) and a Traditional Chinese version (`<name>.zh-TW.md`). Keep both versions in step. If you cannot write one of the languages, say so in the pull request, and a maintainer will help.
5. Open the pull request and fill in the template.

## License

When you send a contribution, you agree that it is licensed under the [MIT License](LICENSE) of this project.

## Why

SpecLink 是 greenfield 專案，目前尚無任何 capability spec。任何後續 capability（change CRUD、artifact write、apply、review、archive、discuss、skill 部署）都先需要一個**可被 init 出來、可被 status 觀察、且 disk layout 已固化**的 project boundary。沒有 bootstrap，沒有任何 workflow 能在實機上跑起來。

本 change 是 MVP walking skeleton 的第一片：把 `speclink init / status / link / unlink` 與 two-root storage layout（`.speclink/` artifact root + `.git/speclink/` state root）作為**第一個可端到端執行的能力**落地，為後續所有 capability 提供基底。設計依據 `doc/speclink-design.md` §13.1、§13.9、§14、§16.1、§18.1 #1–#15。

## What Changes

- 新增 CLI 指令（皆屬「人類設定階段」，不由 AI skill 呼叫）：
  - `speclink init` — 在當前 working dir 建立 LocalProvider project：git 檢查 → 寫 `.speclink/link.yaml` → 建立 `.git/speclink/` state root → schema seed → state.db migration → `.gitignore` 追加 `.speclink/link.yaml`
  - `speclink status` — 印出 project_id、provider、artifact_root、state_root、git_head、requires_git；支援 `--json`
  - `speclink link <project_id>` — 把現有 working dir 綁定到既存 project 的 state（用於 clone 後 re-bind 或 worktree 設定）
  - `speclink unlink` — 解除綁定（移除 `.speclink/link.yaml` 中的 binding 段；不刪 artifact，不刪 state.db）
- 強制 git：non-git working dir 在 `init` 直接拒絕，error code `project.requires_git`，並提示 `git init` 後重試
- Two-root storage layout（disk contract）：
  - `.speclink/` 為 artifact root（git-tracked，跨 worktree 共享 via 工作樹本身）
  - `.git/speclink/` 為 state root（不進 git，跨 worktree 自動共享 via `git rev-parse --git-common-dir`）
  - Path resolution 統一走 `git rev-parse --git-common-dir` 取得 state root parent
- 預設 `.gitignore` 寫入單行 `.speclink/link.yaml`（避免本機 binding 被誤 commit）
- JSON output schema：`status --json` 與所有非互動 init/link/unlink 命令輸出統一 envelope `{ ok, data, warnings?, requestId }` 或 `{ ok: false, error: { code, message, hint?, retryable, retry_after_ms? }, requestId }`
- Exit code：成功 0；使用者輸入錯誤 2；validation failed 3；conflict 7（已 init 又再 init 而 force 旗標未給）；`project.requires_git` 視為使用者輸入錯誤 2
- Error code 新增：`project.requires_git`、`project.already_initialized`、`project.link_target_not_found`
- Doctor finding 預留註冊（finding code only，diagnostic 實作由後續 change 處理）：`doctor.project.requires_git`、`doctor.state.db_missing`（auto-fixable，引導到後續 `restore` change）、`doctor.state.db_corrupted`、`doctor.state.db_schema_invalid`

## Non-Goals

- ❌ `change.create` / `change.list` / `change.show` / `change.delete` — 由後續 `add-change-crud` 負責
- ❌ `artifact.read` / `artifact.write` — 由後續 `add-artifact-io` 負責
- ❌ `apply.*` / `review.*` / `archive.*` / `discuss.*` — 各自獨立 change
- ❌ Schema management（`schema.list/show/fork/delete`）— 雖然 `init` 會 seed 預設 schema files 到 `.speclink/schemas/`，但 CLI 表面只在後續 change 暴露
- ❌ Skill 部署（`speclink init --skill <target>`、AGENTS.md/CLAUDE.md marker）— 由後續 `add-skill-deploy` 負責
- ❌ HttpProvider 任何實作 — Provider trait skeleton 留待 `add-provider-trait` 抽出，本 change 只走具體 LocalProvider 路徑
- ❌ `speclink restore --from-artifacts` — recovery CLI 由後續 `add-state-recovery` 負責；本 change 只先佔 finding code
- ❌ `speclink doctor` 完整實作 — 本 change 只註冊 4 個 finding code 字串常量，不寫檢查邏輯
- ❌ Multi-project listing、cross-project commands、prune、export — 全屬 deferred
- ❌ Worktree 自動 marker 寫入 — 走 `git rev-parse --git-common-dir` 自然共享即可

## Capabilities

### New Capabilities

- `project-bootstrap`: `speclink init / status / link / unlink` 的 CLI 表面、JSON envelope、exit code、error code 與 acceptance 行為
- `local-storage-layout`: `.speclink/`（artifact root）與 `.git/speclink/`（state root）的 disk contract、path resolution 規則、`.gitignore` 規範、跨 worktree 共享行為

### Modified Capabilities

(none — there are no existing capability specs to modify)

## Impact

- Affected specs:
  - New capability spec for project-bootstrap
  - New capability spec for local-storage-layout
- Affected code:
  - New: crates/cli/Cargo.toml
  - New: crates/cli/src/main.rs
  - New: crates/cli/src/commands/init.rs
  - New: crates/cli/src/commands/status.rs
  - New: crates/cli/src/commands/link.rs
  - New: crates/cli/src/commands/unlink.rs
  - New: crates/cli/src/output.rs
  - New: crates/runtime/Cargo.toml
  - New: crates/runtime/src/lib.rs
  - New: crates/runtime/src/bootstrap.rs
  - New: crates/runtime/src/paths.rs
  - New: crates/runtime/src/git.rs
  - New: crates/runtime/src/error.rs
  - New: crates/provider/Cargo.toml
  - New: crates/provider/src/lib.rs
  - New: crates/provider/src/types.rs
  - New: crates/provider-local/Cargo.toml
  - New: crates/provider-local/src/lib.rs
  - New: crates/provider-local/src/store.rs
  - New: crates/provider-local/src/link_yaml.rs
  - New: crates/provider-local/src/state_db.rs
  - New: Cargo.toml
  - New: tests/cli/init_status.rs
  - New: tests/cli/link_unlink.rs
  - New: tests/cli/non_git.rs
- Affected crates: `cli`、`runtime`、`provider`、`provider-local`
- Affected design refs: doc/speclink-design.md §13.1, §13.6, §13.9, §14, §14.1, §16.1, §17.3, §17.4, §17.5, §18.1 #1–#15, §19.3

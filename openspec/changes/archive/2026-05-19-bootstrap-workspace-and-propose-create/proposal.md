## Why

SpecLink 是 greenfield 專案，目前沒有任何可執行程式碼。要驗證設計文件提出的「Skill + CLI + 可替換 Provider」核心架構是否站得住腳，必須先打穿一條完整的 vertical slice：AI skill 呼叫 CLI、CLI 解析 provider、寫入 artifact 到 local provider、最後回傳穩定 JSON。本 change 建立 Cargo workspace 骨架與第一條 end-to-end 流程，作為後續所有 CLI 指令、HTTP provider、analyzer/validator 等功能的基礎。

## What Changes

- 建立 Cargo workspace 根目錄（workspace `Cargo.toml`、`rust-toolchain.toml`），初始 4 個 crate：`crates/cli/`、`crates/runtime/`、`crates/provider/`、`crates/provider-local/`
- 在 `crates/provider/` 定義 `Provider` async trait（`Send + Sync`）與五個共用資料型別：`Project`、`Change`、`Artifact`、`Pack`、`State`
- 在 `crates/provider/` 定義 provider resolution 五層優先序的型別與解析函式（command flag → project config → global active profile → env var → local fallback）
- 在 `crates/provider-local/` 實作 local filesystem provider，儲存於 `.speclink/`，內含 `config.toml`、`state.db`（SQLite via `rusqlite`，僅一張 `in_progress_change` 表）、`changes/<change-id>/proposal.md`
- 在 `crates/runtime/` 提供 instructions resolver 與 artifact write orchestration 的最小骨架（僅服務 `propose create` 一條指令）
- 在 `crates/cli/` 加入第一條 AI workflow 指令 `speclink propose create --change <id> --summary <text> --json`
- 在 `crates/cli/` 統一處理 `--json` / `--no-color` / `--quiet` 旗標與 MVP 範圍 exit code（0、1、2、5、6）
- 新增 4 個 capability spec：`provider-resolution`、`local-provider-storage`、`cli-propose-create`、`cli-machine-interface`

**目標使用者情境**：
- **AI skill 呼叫階段**：skill 可透過 `speclink propose create --change <id> --summary <text> --json` 建立 change 並寫入 proposal，未登入時自動 fallback 至 local provider 且 stdout 不出現中斷錯誤
- **工程師本機階段**：可在沒有任何遠端設定的情況下直接使用 CLI（驗證 fallback 為 always-on 路徑）
- **CI 階段**：暫不涉及（HTTP provider 與 auth 延後）

**`speclink propose create` 是 AI skill 可呼叫的指令**，對應 spec capability `cli-propose-create`，相對於人類設定階段的 `provider add` / `auth login` / `project bind`（皆延後）。

## Non-Goals

- 不實作 `crates/provider-http/`（HTTP provider）— 在 trait 定下 `async + Send + Sync` 後當作下一個 change
- 不實作 `crates/auth/`、keychain 整合、token 儲存、device code flow、OAuth、PAT
- 不實作其他 CLI 指令：`discuss start/capture`、`instructions`、`status`、`artifact write`（除 proposal 外其他 artifact）、`analyze`、`validate`、`pack create`、`unpack`、`apply start`、`task done`、`drift`、`finish`、`archive`
- 不實作人類設定指令：`provider add/list/use`、`auth login/status/logout`、`project bind/status`
- 不實作 `crates/analyzer/`、`crates/validator/`、`crates/pack/`、`crates/skill-templates/`
- 不實作完整 lifecycle state machine — 僅支援 `draft → proposed` 一步（藉由 `propose create` 完成）
- 不處理 optimistic concurrency control（artifact `expectedVersion` / `If-Match`、`artifact.version_conflict` 錯誤）
- 不處理 archive 流程、pack/unpack 跨機器同步
- 不引入 `crates/types/` — 所有共用型別放在 `crates/provider/`，待出現非 provider 共用型別時再評估拆分
- 不為未來假設性需求預留介面：native provider plugin、WASM provider、多人即時鎖、Web UI、shell completion、TUI、progress bar
- 不支援 `--stdin` 讀入 proposal 內文 — `propose create` 第一版僅接受 `--summary` flag 寫入單行摘要（避免在 MVP 引入 stdin parsing 複雜度，後續 `artifact write` 指令再處理）
- 不處理 Windows / macOS / Linux keychain 後端差異（無 keychain）

## Capabilities

### New Capabilities

- `provider-resolution`: provider 解析的五層優先序、local filesystem fallback 行為、未登入或 provider 不可用時的 `provider.not_authenticated` warning 與 fallback 啟用/停用兩種設定下的退出行為
- `local-provider-storage`: local provider 在 `.speclink/` 的目錄結構、SQLite `state.db` 的 schema（僅 `in_progress_change` 表）、artifact 寫入時的目錄建立與檔案格式
- `cli-propose-create`: `speclink propose create` 指令的 clap 介面、必要與選用旗標、stdin 行為（本版禁用）、JSON output schema、成功與失敗的 exit code 與 error code
- `cli-machine-interface`: CLI 共用的 machine interface 規範 — `--json` / `--no-color` / `--quiet` 旗標語意、統一 exit code 表（0 成功、1 一般錯誤、2 使用者輸入錯誤、5 provider unavailable、6 auth required no fallback）、error code 命名規則（點分隔）、JSON 外層 schema、warning 與 error 結構

### Modified Capabilities

(none)

## Impact

- Affected specs:
  - New: `provider-resolution`、`local-provider-storage`、`cli-propose-create`、`cli-machine-interface`
- Affected crates:
  - New: `crates/cli/`、`crates/runtime/`、`crates/provider/`、`crates/provider-local/`
  - 不動: 暫不存在的 `crates/provider-http/`、`crates/auth/`、`crates/analyzer/`、`crates/validator/`、`crates/pack/`、`crates/skill-templates/`
- Affected code:
  - New:
    - workspace 根目錄 manifest（Cargo workspace 設定檔）
    - rust-toolchain.toml
    - crates/cli/Cargo.toml
    - crates/cli/src/main.rs
    - crates/cli/src/cli.rs
    - crates/cli/src/commands/mod.rs
    - crates/cli/src/commands/propose.rs
    - crates/cli/src/output.rs
    - crates/cli/src/exit_code.rs
    - crates/cli/tests/propose_create.rs
    - crates/runtime/Cargo.toml
    - crates/runtime/src/lib.rs
    - crates/runtime/src/propose.rs
    - crates/provider/Cargo.toml
    - crates/provider/src/lib.rs
    - crates/provider/src/model.rs
    - crates/provider/src/resolution.rs
    - crates/provider/src/error.rs
    - crates/provider-local/Cargo.toml
    - crates/provider-local/src/lib.rs
    - crates/provider-local/src/storage.rs
    - crates/provider-local/src/state_db.rs
    - crates/provider-local/src/error.rs
    - openspec/specs/provider-resolution/spec.md
    - openspec/specs/local-provider-storage/spec.md
    - openspec/specs/cli-propose-create/spec.md
    - openspec/specs/cli-machine-interface/spec.md
  - Modified: 無（greenfield）
  - Removed: 無
- Affected crate dependencies（保證無循環）:
  - cli → runtime、provider
  - runtime → provider
  - provider-local → provider
  - provider → 無專案內 crate（只依賴第三方）
- 跨 crate 變更必要性論證：greenfield bootstrap 需要同時建立 4 個 crate 才能形成可執行的 vertical slice — provider trait 沒有實作就無法驗證、local provider 沒有 runtime 呼叫就無法測 fallback、runtime 沒有 CLI 入口就無法被 skill 觸發。本 change 後，後續每個 change 預期收斂至 1-2 個 crate。

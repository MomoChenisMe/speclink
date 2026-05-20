## Why

`add-artifact-write-and-status` 完成後，SpecLink 可在 local provider 上寫入完整 SDD artifact 並查詢 status，但 change 開完後**沒有任何方法可以收尾**：既無法把該 change 從 `in_progress_change` 表移除、也無法把 `<change>/specs/` 的 delta 套用到主 spec 目錄。本 change 補上 archive 流程，讓 SpecLink 可以完整跑完 propose → artifact write → status → archive 的一個 SDD 週期，並讓主 spec 隨 change 演化累積。這是 dogfooding SpecLink 開發 SpecLink 自身（取代 Spectra workflow）的最後一塊缺口。

## What Changes

- 在 `crates/cli/` 新增 AI workflow 指令 `speclink archive <change> --json`：將 `.speclink/changes/<id>/` 移動至 `.speclink/changes/archive/YYYY-MM-DD-<id>/`、清空 `in_progress_change` 表中該 change 的 row、更新 `metadata.json` 的 `state` 為 `archived` 並補上 `archivedAt` 時間戳
- 在 `crates/provider/` 新增 trait method `Provider::archive_change(project_id, change_id) -> Result<ArchivedChange, ProviderError>`，回傳含 archive 路徑與 spec sync 結果的結構
- 在 `crates/provider-local/` 實作 `archive_change`：執行目錄 rename、SQLite 清除、metadata.json 更新（全部以 best-effort 順序執行，失敗時 rollback）
- 在 `crates/provider-local/` 與 `crates/runtime/` 之間新增 **spec delta merge** 邏輯：解析 `<change>/specs/<capability>/spec.md` 中的 `## ADDED Requirements` / `## MODIFIED Requirements` / `## REMOVED Requirements` / `## RENAMED Requirements` 四種 heading，套用到 `.speclink/specs/<capability>/spec.md`（主 spec 目錄，本 change 首次建立）
- 在 `crates/provider/` 新增 `State` enum variant `Archived`，並更新對應 serde 序列化字串
- 在 `crates/cli/` 提供 `--dry-run` 旗標讓 AI skill 預演 archive（檢查 delta 解析、印出將套用的 requirement 變更但不真正寫檔）
- 沿用 envelope schema、exit code 與 error code naming
- 新增 capability spec：`cli-archive`、`spec-delta-merge`；修改 `local-provider-storage`（補 archive 目錄結構與主 spec 目錄、`State::Archived` 與 metadata.json 的 `archivedAt` 欄位）

**目標使用者情境**：
- **AI skill 呼叫階段**：skill 在收到使用者「完成這個 change」訊號後執行 `speclink archive <id>`，CLI 自動處理目錄搬移、SQLite 清理、spec sync；skill 無需知道 `.speclink/` 內部結構
- **工程師本機階段**：可直接 `speclink archive <id>` 收尾一個 change，下次 `propose create` 不會被前一次的 `in_progress_change` 干擾
- **CI 階段**：暫不涉及

## Non-Goals

- 不引入 `in_progress` 狀態 — Change 2 已確立 artifact write 不更新 metadata.json，目前沒有自然的觸發點推進 `proposed → in_progress`；待 `apply start` 指令落地（未來 change）時再評估
- 不引入 `reviewing` / `accepted` / `rejected` / `cancelled` 等狀態 — 屬於 review/feedback loop，與 archive 流程獨立
- 不實作 `archive --restore` 或從 archive 取回 change 的反向操作 — archive 為單向
- 不實作 archive 跨機器同步（archive bundle、export、import）— 由未來 `pack` / `unpack` 處理
- 不實作 archive trace（在 main spec 加 `<!-- @trace ... -->` 註記）— 屬於 archive bookkeeping enhancement，第一版只做純 delta merge
- 不在 archive 時驗證 main spec 之間的 cross-reference 一致性 — 由未來 `analyze` 指令處理
- 不為 spec delta merge 引入 schema/grammar validator — 解析失敗時直接回錯誤，不嘗試 partial merge
- 不引入主 spec 的 spec-driven schema 標記檔（如 `spec_id`、`version`）— 主 spec 就是合併後的純 markdown
- 不變更 `propose create` / `artifact write` / `status` 三條既有指令的行為（除了 `State` enum 多一個 variant 對序列化的影響）
- 不引入新 crate；既有 4 crate 已足以容納

## Capabilities

### New Capabilities

- `cli-archive`: `speclink archive` 指令的 clap 介面、必要與選用旗標（含 `--dry-run`）、JSON output schema（含 archive path 與 delta 套用結果摘要）、成功與失敗的 exit code 與 error code
- `spec-delta-merge`: 從 `<change>/specs/<capability>/spec.md` 中解析 delta heading（ADDED / MODIFIED / REMOVED / RENAMED）並套用至 `.speclink/specs/<capability>/spec.md` 的演算法 — 包含主 spec 不存在時的 ADD 全建、heading 匹配規則、衝突處理（如 MODIFIED 找不到對應 requirement）

### Modified Capabilities

- `local-provider-storage`: 新增 `.speclink/specs/<capability>/spec.md` 主 spec 目錄、`.speclink/changes/archive/YYYY-MM-DD-<id>/` archive 目錄結構；metadata.json 新增 `archivedAt` 欄位；`State` enum 新增 `Archived` 與其字串表示

## Impact

- Affected specs:
  - New: `openspec/specs/cli-archive/spec.md`、`openspec/specs/spec-delta-merge/spec.md`
  - Modified: `openspec/specs/local-provider-storage/spec.md`（新增 archive 目錄、主 spec 目錄、State::Archived、archivedAt）
- Affected crates:
  - Modified: `crates/cli/`、`crates/runtime/`、`crates/provider/`、`crates/provider-local/`
- Affected code:
  - New:
    - crates/cli/src/commands/archive.rs
    - crates/cli/tests/archive.rs
    - crates/cli/tests/archive_snapshots.rs
    - crates/runtime/src/archive.rs
    - crates/runtime/src/spec_delta.rs
    - crates/provider-local/tests/archive_integration.rs
    - openspec/changes/add-local-archive/specs/cli-archive/spec.md
    - openspec/changes/add-local-archive/specs/spec-delta-merge/spec.md
    - openspec/changes/add-local-archive/specs/local-provider-storage/spec.md
  - Modified:
    - crates/cli/src/cli.rs（新增 Archive subcommand）
    - crates/cli/src/commands/mod.rs
    - crates/cli/src/main.rs（dispatch）
    - crates/cli/src/output.rs（新增 ArchiveData 與 SpecSyncSummary）
    - crates/cli/src/exit_code.rs（新增 archive 相關 error code mapping）
    - crates/runtime/src/lib.rs（exports）
    - crates/provider/src/lib.rs（新增 archive_change trait method 與 ArchivedChange type）
    - crates/provider/src/model.rs（State::Archived、ArchivedChange、SpecDeltaSummary、archivedAt metadata 欄位）
    - crates/provider/src/error.rs（新增 archive 相關 ProviderError variant）
    - crates/provider-local/src/lib.rs（實作 archive_change，整合 spec delta merge）
    - crates/provider-local/src/storage.rs（新增 archive directory rename、main spec write helper）
    - crates/provider-local/src/error.rs（新增 archive 相關 LocalProviderError variant）
  - Removed: 無
- Affected crate dependencies（無變更，無循環）:
  - cli → runtime、provider、provider-local（既有）
  - runtime → provider（既有；spec_delta module 純算法，不引入新外部 crate）
  - provider-local → provider、runtime（**新增** — provider-local 需呼叫 runtime 的 spec_delta merge；或反向由 runtime 編排，見 design.md decision）
- 跨 crate 變更必要性論證：trait 加 method（provider）與實作（provider-local）必須同時改；archive 編排（包含 delta merge）若放 runtime 須 provider-local 提供原子目錄操作 helper；CLI 新增 subcommand 經 runtime 編排。三層改動為單一 vertical slice 切面。

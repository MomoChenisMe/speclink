# server-policy-write Specification

## Purpose

工作流政策在 remote 側的讀寫與授權模型：config 內容連同 revision 一起下發、政策寫入一律以 CAS 進行並先經引擎驗證，以及 membership role 的最小模型與 role 經 capabilities 的傳播。本 capability 保證併發寫入不互相覆蓋，且 client 端的 capabilities 只是提示——server 才是授權的最終防線。

## Requirements

### Requirement: config 內容與 revision 隨讀取下發

config 讀取端點 SHALL 回應 workflow config 文件原文（缺席為 null）、schema 名與 scope revision，revision SHALL 與回應 ETag 同值。讀取 SHALL 不受 role 限制——reader 與 editor 皆可讀。

#### Scenario: 讀取回內容與一致的 revision

- **WHEN** 對含 workflow config 的 scope 呼叫 config 讀取端點
- **THEN** 回應含文件原文與 revision，且 revision 與 ETag 頭同值

<!-- @trace
source: remote-workflow-policy
updated: 2026-07-20
code:
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/read_api.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/policy_write.rs
-->

---
### Requirement: policy 寫入一律 CAS 且經引擎驗證

policy 寫入端點 SHALL 要求完整文件原文與 expected revision——缺 expected revision SHALL 拒收，SHALL NOT 存在無條件覆寫路徑。寫入 SHALL 依序：role 檢查（非 editor 即 403 且不觸文件）→ 引擎 WorkflowConfig fail-closed 驗證（解析失敗回 invalid_config 且不落盤）→ expected revision 的 CAS 提交（不符回 revision_conflict 且內容不變；成功回新 revision）。引擎驗證 SHALL 併涵蓋政策值域：內容可解析但 locale 非 tw／ja／en、或 spec_locale 非 tw／ja／en／auto（大小寫敏感；鍵不存在即未設定，恆合法）時，SHALL 拒絕且不落盤，錯誤 message SHALL 指出欄位名與合法代碼集合；SHALL 沿用既有錯誤 envelope 結構與 reason 種類，SHALL NOT 為值域驗證新增 wire 形狀。政策由 server 守住——client 端任何驗證僅為 UX，SHALL NOT 被信任為防線。

#### Scenario: revision 過期寫入被拒且無副作用

- **WHEN** 以過期的 expected revision 提交合法的 config 內容
- **THEN** 回應 revision_conflict，store 中文件內容與 revision 皆未改變

#### Scenario: 壞 YAML 不落盤

- **WHEN** 以正確的 expected revision 提交無法解析的 YAML
- **THEN** 回應 invalid_config 並指出解析錯誤，store 中文件未改變

#### Scenario: 非法 locale 值不落盤

- **WHEN** 以正確的 expected revision 提交可解析、但 locale 值為「繁體中文」的 config 內容
- **THEN** 寫入被拒，錯誤 message 指出 locale 欄位與合法代碼 tw、ja、en；store 中文件內容與 revision 皆未改變


<!-- @trace
source: workflow-config-locale-validation
updated: 2026-07-30
code:
  - apps/desktop/src/__tests__/projectSettingsView.test.tsx
  - apps/desktop/src/i18n/messages.ts
  - apps/desktop/src/views/ProjectSettingsView.tsx
  - crates/speclink-cli/tests/workflow_config.rs
  - crates/speclink-core/assets/skills/config.md
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/policy_write.rs
  - docs/configuration.md
  - docs/configuration.zh-TW.md
-->

---
### Requirement: membership role 最小模型

project membership SHALL 攜帶 role（editor｜reader），預設 editor；identity 儲存升版 SHALL 使既有 membership 一律為 editor（升版後行為不變）。admin 設定 membership SHALL 可指定 role 且變更入 audit（含新 role 值）；invitation 建立的 membership SHALL 固定 editor。

#### Scenario: 升版後既有成員行為不變

- **WHEN** 既有 identity 資料庫升版至含 role 的 schema
- **THEN** 全部既有 membership 的 role 為 editor，policy 寫入行為與升版前一致

<!-- @trace
source: remote-workflow-policy
updated: 2026-07-20
code:
  - crates/speclink-server/src/admin.rs
  - crates/speclink-server/src/identity.rs
  - crates/speclink-server/src/identity_sqlite.rs
  - crates/speclink-server/tests/admin_pages.rs
  - crates/speclink-server/tests/admin_system.rs
  - crates/speclink-server/tests/audit.rs
  - crates/speclink-server/tests/identity.rs
  - crates/speclink-server/tests/policy_write.rs
-->

---
### Requirement: role 經 capabilities 傳播且 server 為最終防線

binding handshake 的 capabilities SHALL 含 policy 寫入權布林（membership role 為 editor 時真）；desktop 據此停用僅為 UX，寫入端點對 reader 的 403 SHALL 為最終防線——即使 client 繞過 UI 直接呼叫亦被拒。

#### Scenario: reader 直呼寫入端點被拒

- **WHEN** role 為 reader 的使用者繞過 UI 以合法 payload 直接呼叫 policy 寫入端點
- **THEN** 回應 403，文件未改變；其 binding 回應中 policy 寫入權為假

<!-- @trace
source: remote-workflow-policy
updated: 2026-07-20
code:
  - apps/desktop/src-tauri/src/remote.rs
  - apps/desktop/src/adapter/workspace.ts
  - apps/desktop/src/session.ts
  - apps/desktop/src/views/ProjectSettingsView.tsx
  - crates/speclink-protocol/src/binding.rs
  - crates/speclink-server/src/auth.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/policy_write.rs
-->
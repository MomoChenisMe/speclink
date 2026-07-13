# remote-connection Specification

## Purpose

TBD - created by archiving change 'verb-contract-and-remote-client'. Update Purpose after archive.

## Requirements

### Requirement: remote 初始化與連接指令

speclink init --store remote --url <url> [--repo <name>] SHALL 執行 workspace init（指令檔 marker、技能、settings、gitignore）並將 url 與 repo 寫入 .speclink.yaml 的 remote 區段（檔案不存在時建立、既有欄位保留），且 SHALL NOT 建立 openspec/ 目錄樹、SHALL NOT 建立獨立連接檔。speclink link <url> [--repo <name>] SHALL 寫入或更新 remote 區段；speclink unlink SHALL 移除 remote 區段並保留檔內其他欄位。init 或 link 當下若已有可用憑證，CLI SHALL 立即向 server 查驗 repo 是否屬於專案並回報結果；無憑證時 SHALL 提示執行 speclink auth login。

#### Scenario: remote 初始化不建規格樹

- **WHEN** 於空目錄執行 speclink init --store remote --url https://team.example.com/speclink/projects/foo --repo backend
- **THEN** 生成 CLAUDE.md marker、技能目錄與含 remote 區段（url 與 repo 欄位如參數）的 .speclink.yaml，且不存在 openspec/ 目錄與 .speclink.remote.yaml；exit code 為 0

#### Scenario: link 保留既有欄位

- **WHEN** .speclink.yaml 已含 tools 清單，以有效憑證執行 speclink link https://team.example.com/speclink/projects/foo --repo backend 且 repo 在註冊表
- **THEN** remote 區段被寫入且 tools 清單原值保留；exit code 為 0

#### Scenario: link 時 repo 不在專案註冊表

- **WHEN** 已有可用憑證，執行 speclink link https://team.example.com/speclink/projects/foo --repo typo-name，而 server 註冊表無 typo-name
- **THEN** exit code 非 0，stderr 訊息指出 repo 不在專案內並列出可用的註冊名清單，remote 區段不被寫入

#### Scenario: unlink 移除連接

- **WHEN** 於 remote 模式專案執行 speclink unlink
- **THEN** .speclink.yaml 的 remote 區段被移除、檔內其他欄位保留，後續指令回到 fs 模式的行為


<!-- @trace
source: remote-section-in-speclink-yaml
updated: 2026-07-05
code:
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/doc_verbs.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-cli/tests/remote_read_path.rs
  - crates/speclink-cli/tests/remote_section.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/mode_resolution.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/client_errors.rs
  - docs/team-mode.md
  - docs/team-mode.zh-TW.md
-->

---
### Requirement: repo 身分攜帶與歸屬防呆

remote 模式下每個動詞 SHALL 自動攜帶 remote 區段的 repo 名；server 回應 change 歸屬不符時，CLI SHALL 以非 0 exit code 結束並輸出同時指出 change 歸屬 repo 與當前 repo 名的單行訊息。

#### Scenario: 跑錯 repo 被擋下

- **WHEN** 於 remote 區段 repo 欄位為 frontend 的專案執行 speclink claim add-rate-limit，而該 change 歸屬 backend
- **THEN** exit code 非 0，stderr 訊息同時含 backend 與 frontend 兩個名稱與改正指引


<!-- @trace
source: remote-section-in-speclink-yaml
updated: 2026-07-05
code:
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/doc_verbs.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-cli/tests/remote_read_path.rs
  - crates/speclink-cli/tests/remote_section.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/mode_resolution.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/client_errors.rs
  - docs/team-mode.md
  - docs/team-mode.zh-TW.md
-->

---
### Requirement: git remote 參考值的輔助警告

speclink link 與 speclink auth status 執行時，若 server 註冊表提供本 repo 的 git url 參考值且與本地 git remote 不一致，CLI SHALL 於 stderr 輸出一行輔助警告（提示可能在 fork 或鏡像上工作）；此警告 SHALL NOT 影響指令結果與 exit code（僅警告、不強制）。本地非 git 目錄或 server 未提供參考值時 SHALL 靜默略過此檢查。

#### Scenario: fork 上工作僅警告不阻擋

- **WHEN** 本地 git remote 指向 fork，而 server 註冊表的 git url 參考值為原始 repo，以有效憑證執行 speclink link 某專案 url --repo backend
- **THEN** remote 區段照常寫入、exit code 為 0，stderr 出現一行 fork／鏡像提示警告

#### Scenario: 無參考值時靜默

- **WHEN** server 註冊表未提供本 repo 的 git url 參考值，執行 speclink auth status
- **THEN** 不輸出任何 git remote 相關警告


<!-- @trace
source: remote-section-in-speclink-yaml
updated: 2026-07-05
code:
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/doc_verbs.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-cli/tests/remote_read_path.rs
  - crates/speclink-cli/tests/remote_section.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/mode_resolution.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/client_errors.rs
  - docs/team-mode.md
  - docs/team-mode.zh-TW.md
-->

---
### Requirement: 指令區塊的 remote 變體
remote 模式下 init 生成的 CLAUDE.md／AGENTS.md marker 區塊 SHALL 使用 remote 措辭：指明規格與 change 存於團隊系統、一律使用 speclink 動詞、SHALL NOT 指示本地讀寫規格檔；fs 模式的 marker 內容維持路徑措辭。

#### Scenario: remote marker 不含本地路徑句
- **WHEN** 執行 speclink init --store remote --url 某專案 url 後檢視 CLAUDE.md 的 SPECLINK marker 區塊
- **THEN** 區塊內容不含 openspec/specs 或 openspec/changes 路徑字樣，並含「使用 speclink 動詞存取」的指引

<!-- @trace
source: verb-contract-and-remote-client
updated: 2026-07-05
code:
  - .speclink.yaml
  - Cargo.lock
  - Cargo.toml
  - README.md
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/main.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/doc_verbs.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-cli/tests/remote_read_path.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-core/Cargo.toml
  - crates/speclink-core/assets/skills/archive.md
  - crates/speclink-core/assets/skills/commit.md
  - crates/speclink-core/assets/skills/discuss.md
  - crates/speclink-core/assets/skills/propose.md
  - crates/speclink-core/assets/skills/sync.md
  - crates/speclink-core/src/discuss.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/instructions.rs
  - crates/speclink-core/src/preflight.rs
  - crates/speclink-core/src/status.rs
  - crates/speclink-core/src/store.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/golden/claude.snapshot.md
  - crates/speclink-core/tests/golden/codex.snapshot.md
  - crates/speclink-core/tests/golden/neutral-cli.snapshot.md
  - crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md
  - crates/speclink-core/tests/golden/remote-claude.marker.md
  - crates/speclink-core/tests/mode_resolution.rs
  - crates/speclink-core/tests/render_golden.rs
  - crates/speclink-core/tests/skill_verbization.rs
  - crates/speclink-fs/src/layout.rs
  - crates/speclink-fs/src/lib.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/auth.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/auth_store.rs
  - crates/speclink-remote/tests/client_errors.rs
  - docs/team-mode.md
  - docs/team-mode.zh-TW.md
  - docs/verb-contract.md
  - docs/verb-contract.zh-TW.md
-->

---
### Requirement: remote 區段與模式解析
`.speclink.yaml` 的 remote 區段（欄位：url 選填——缺席時由環境變數 SPECLINK_STORE_URL 供給、含專案範疇；repo 選填為本 repo 在專案內的註冊名）存在時，CLI SHALL 以 remote 模式運作；不存在時 SHALL 以 fs 模式運作。remote 區段與 openspec/ 目錄並存時，remote 模式 SHALL 生效且 CLI SHALL 於 stderr 輸出一行並存警告。環境變數 SPECLINK_STORE_URL 存在時 SHALL 覆寫區段的 url。remote 區段存在但區段 url 與環境變數皆缺席時，CLI SHALL 以非 0 exit code 明確失敗並同時提示 remote.url 欄位與 SPECLINK_STORE_URL 兩種設定方式，SHALL NOT 靜默改以 fs 模式執行。

`.speclink.yaml` 檔案存在但無法解析（YAML 語法錯誤或型別不符）時，模式判定 SHALL fail-closed：任何依賴模式判定或應用層設定的指令 SHALL 以非零 exit code 失敗，stderr SHALL 指出 .speclink.yaml 與解析原因；SHALL NOT 視為無 remote 區段而以 fs 模式執行，SHALL NOT 發出任何遠端請求。檔案不存在時 SHALL 以 fs 模式運作。此 fail-closed 行為屬對 Spectra 2.3.1 的刻意分歧（Spectra 於壞檔時靜默退回預設）。

#### Scenario: 有 remote 區段即 remote 模式

- **WHEN** .speclink.yaml 含 remote 區段（url 指向團隊 server），執行 speclink list --json
- **THEN** 指令向區段 url 的契約端點發出請求（而非讀取本地 openspec/），輸出 JSON 形狀與 fs 模式一致

#### Scenario: 並存時 remote 勝出並警告

- **WHEN** .speclink.yaml 含 remote 區段且專案根同時有 openspec/ 目錄，執行 speclink list
- **THEN** 指令以 remote 模式執行，stderr 恰有一行警告指出兩者並存且 remote 生效

#### Scenario: 環境變數覆寫區段 url

- **WHEN** 設定 SPECLINK_STORE_URL 指向另一 server，執行 speclink list
- **THEN** 請求發往環境變數指定的 url

#### Scenario: url 兩處皆缺時明確失敗

- **WHEN** remote 區段僅含 repo 欄位、未設 SPECLINK_STORE_URL，執行 speclink list
- **THEN** exit code 非 0，stderr 訊息指出 url 缺失並同時提示 remote.url 欄位與 SPECLINK_STORE_URL 兩種設定方式，不以 fs 模式執行

#### Scenario: 壞 .speclink.yaml 不落入 fs 模式

- **WHEN** .speclink.yaml 含 YAML 語法錯誤且專案根有 openspec/ 目錄，執行 speclink list
- **THEN** exit code 非 0，stderr 指出 .speclink.yaml 與解析原因，指令不讀取本地 openspec/、不發出任何遠端請求

---
### Requirement: 殘留連接檔的遷移警告

專案根存在 .speclink.remote.yaml 時，CLI SHALL 於 stderr 輸出一行遷移警告——指引將 url 與 repo 搬入 .speclink.yaml 的 remote 區段並刪除舊檔，並說明舊檔不參與模式判定——且 SHALL NOT 解析該檔內容；模式判定 SHALL 僅以 .speclink.yaml 為準。此警告 SHALL NOT 影響指令結果與 exit code。

#### Scenario: 殘留舊檔僅警告不生效

- **WHEN** 專案根含 .speclink.remote.yaml（url 指向某 server），.speclink.yaml 無 remote 區段且 openspec/ 目錄存在，執行 speclink list
- **THEN** 指令以 fs 模式讀取本地 openspec/ 執行，stderr 恰有一行遷移警告，exit code 為 0

<!-- @trace
source: remote-section-in-speclink-yaml
updated: 2026-07-05
code:
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/doc_verbs.rs
  - crates/speclink-cli/tests/remote_connect.rs
  - crates/speclink-cli/tests/remote_read_path.rs
  - crates/speclink-cli/tests/remote_section.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-core/src/config.rs
  - crates/speclink-core/src/init.rs
  - crates/speclink-core/src/workspace.rs
  - crates/speclink-core/tests/mode_resolution.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/client_errors.rs
  - docs/team-mode.md
  - docs/team-mode.zh-TW.md
-->

---
### Requirement: remote 動詞經 handshake 建立的連線語境執行

remote 模式的動詞執行 SHALL 以 binding handshake 建立的連線語境為前置：handshake 成功後動詞請求 SHALL 自動攜帶確認過的 project 與 repo 身分；handshake 因 API version 不相容、binding 缺失、無權限或多義而失敗時，動詞 SHALL 以非零 exit code 停止並輸出指向原因的錯誤，SHALL NOT 回退為未驗證的逐 verb 呼叫。連線設定（.speclink.yaml 的 remote 區段）的欄位與語意 SHALL 不變。

#### Scenario: handshake 失敗動詞即停

- **WHEN** stub server 的 handshake 回應為 binding 多義（兩個候選 repo），隨後執行任一 remote 動詞
- **THEN** 動詞以非零 exit code 結束、stderr 指出 binding 多義與候選清單；無動詞請求被送出

#### Scenario: 設定欄位不變

- **WHEN** 以現行 .speclink.yaml 的 remote 區段（url 與 repo key）啟動 remote 動詞且 handshake 成功
- **THEN** 動詞行為與輸出與現行一致；設定檔無需任何修改

<!-- @trace
source: protocol-typed-client
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-cli/Cargo.toml
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/doc_verbs.rs
  - crates/speclink-cli/tests/no_raw_wire_json.rs
  - crates/speclink-cli/tests/remote_handshake_gate.rs
  - crates/speclink-cli/tests/remote_read_path.rs
  - crates/speclink-cli/tests/remote_write_path.rs
  - crates/speclink-protocol/Cargo.toml
  - crates/speclink-protocol/src/binding.rs
  - crates/speclink-protocol/src/command.rs
  - crates/speclink-protocol/src/context.rs
  - crates/speclink-protocol/src/error.rs
  - crates/speclink-protocol/src/events.rs
  - crates/speclink-protocol/src/lib.rs
  - crates/speclink-protocol/src/query.rs
  - crates/speclink-remote/Cargo.toml
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/src/lib.rs
  - crates/speclink-remote/tests/client_errors.rs
  - crates/speclink-remote/tests/handshake.rs
  - crates/speclink-remote/tests/typed_client.rs
-->
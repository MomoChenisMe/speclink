## ADDED Requirements

### Requirement: 倉庫路徑守門

scripts 測試面 SHALL 含一條靜態斷言，遞迴走訪 repo 內的文字檔（排除 .git、node_modules、target、dist、`.speclink`、`.dev`、`openspec/changes`、`openspec/discussions` 與 CHANGELOG.md，並跳過 `<!-- @trace` 到 `-->` 之間的行、以及帶 `path-guard:allow` 標記的行），確認不殘留搬移前的舊形路徑：(a) `crates/speclink-<name>` 形式的 crate 路徑（現行正典為 `crates/<group>/speclink-<name>`，group ∈ engine／host／store／protocol／adapters）；(b) 已搬入 `scripts/<group>/` 的腳本以 `scripts/<檔名>` 直接引用的形式（group ∈ release／npm／desktop／docs／dev）。`scripts/install.sh`、`scripts/install.ps1` 與 `scripts/install.test.mjs` SHALL 維持於 `scripts/` 根且 SHALL NOT 被此斷言視為舊形——README 與 getting-started 的 raw GitHub 安裝網址逐字不變。`.speclink` 與 `.dev`（本機執行與開發時產生的狀態，非版本控管內容）與 `openspec/changes`（進行中提案逐字引用搬移前路徑）SHALL 一併排除。逐字引用舊形路徑當反例的文件行 SHALL 以 `path-guard:allow` 標記豁免，其餘行 SHALL NOT 豁免。命中時 SHALL 以非零 exit 失敗並列出檔案與行號；零命中時通過。本斷言由 `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 執行，主 CI 的 scripts 測試步驟 SHALL 以同一組樣式呼叫（bash 展開，兩個明確樣式，不依賴 globstar）。 <!-- path-guard:allow -->

#### Scenario: 殘留舊形 crate 路徑被抓出

- **WHEN** repo 內任一文字檔（非排除範圍、非 @trace 註解區塊、未帶豁免標記）含字串 crates/speclink-core/src/init.rs，執行 node --test scripts/*.test.mjs scripts/*/*.test.mjs <!-- path-guard:allow -->
- **THEN** 該測試失敗、非零 exit code，輸出列出該檔案與行號

#### Scenario: 安裝腳本的根路徑不算舊形

- **WHEN** README.md 含 raw GitHub 網址 https://raw.githubusercontent.com/MomoChenisMe/speclink/main/scripts/install.sh，且 repo 內無其他舊形路徑，執行同一指令
- **THEN** 該測試通過，exit code 0

#### Scenario: 歷史記錄與 @trace 區塊不受檢

- **WHEN** openspec/changes 底下的提案、openspec/discussions 的記錄、`.speclink` 與 `.dev` 的本機產生狀態、或正典規格 `<!-- @trace` 區塊內的檔案清單含 crates/speclink-cli/src/commands.rs，且非排除範圍內零命中 <!-- path-guard:allow -->
- **THEN** 該測試通過，exit code 0

#### Scenario: 反例文件以豁免標記排除自身命中

- **WHEN** openspec/specs/delivery-baseline/spec.md 的某行逐字引用 crates/speclink-core/src/init.rs 當反例，且該行帶 `path-guard:allow` 標記，且 repo 內無其他舊形路徑，執行同一指令
- **THEN** 該測試通過，exit code 0；同一檔案內未帶標記的行仍受檢

#### Scenario: 主 CI 以兩個明確樣式跑 scripts 測試面

- **WHEN** 檢視 .github/workflows/ci.yml 的 scripts 測試步驟
- **THEN** 步驟指令為 node --test scripts/*.test.mjs scripts/*/*.test.mjs 且該步驟宣告 shell: bash；delivery-gate 測試以此字面斷言

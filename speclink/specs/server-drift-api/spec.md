# server-drift-api Specification

## Purpose

remote 模式下 drift 的取得方式：server 只提供規格面 drift 端點、工作區面事實不進 wire，由 client 在本機合併兩側後輸出，並以單點函式負責 wire 與引擎型別的往返映射。本 capability 保證 remote drift 的輸出與本地路徑逐字凍結一致，且伺服器不需要、也拿不到使用者的本機工作區狀態。

## Requirements

### Requirement: 規格面 drift 端點且工作區面不進 wire

server SHALL 提供 change-scoped 的 drift 端點：對單一 store snapshot 回報 (a) 以引擎規格面計算的 spec drift（規格面維度與規格假設）、(b) 該 snapshot 的 basis digests、(c) 該 change 的 store 面輸入（created metadata、design/tasks 內容，及該 change 的 evidence 記錄文字——有記錄才出現，缺席即缺席）——client 的工作區面計算讀這些，缺席與空內容 SHALL 可區別。以上三者 SHALL 出自同一個 snapshot。回應 SHALL 為 protocol 的 drift DTO（camelCase、可匯出 JSON Schema）。

wire DTO SHALL NOT 含任何工作區面（code/git）欄位——broken anchors、工作區維度與 git 事實皆屬 client 本機，server SHALL NOT 執行任何 git 操作。（(c) 是 store 事實而非工作區事實：server 由 snapshot 即知，下行供 client 自算其半，不構成工作區面上行；evidence 記錄亦然——它是 store 保存的歷史事實，client 的 Environment 維度據此取得該 change 的 touched 檔案清單。）

端點沿用 bearer 前置與 binding 裁決；未知 change SHALL 回 404 not_found；計算 SHALL 為唯讀，SHALL NOT 產生事件或取寫入鎖。

#### Scenario: 規格面報告、basis 與 store 面輸入

- **WHEN** 對含 delta specs 與 design 的 change 請求 drift 端點
- **THEN** 回應含規格面維度、規格假設、該 snapshot 的 basis digests，以及 created 與 design/tasks 內容；回應結構無任何工作區/git 欄位（含 broken anchors）

#### Scenario: 缺席的 artifact 不偽裝成空內容

- **WHEN** 對無 design.md 的 change 請求 drift 端點
- **THEN** 回應如實標示 design 缺席，而非回空字串——client 的 Structure 維度據此區別「無 design」與「design 為空」

#### Scenario: 未知 change 拒絕

- **WHEN** 以不存在的 change 名請求 drift 端點
- **THEN** 回 404 且 reason 為 not_found

#### Scenario: store 保存的 evidence 隨回應下行

- **WHEN** 對曾以 task done 落過 evidence 的 change 請求 drift 端點
- **THEN** 回應的 store 面輸入含該 change 的 evidence 記錄文字，與 store 保存的內容一致；對照組：從未落過 evidence 的 change，該欄位缺席而非空字串


<!-- @trace
source: remote-task-evidence
updated: 2026-08-25
-->

---
### Requirement: remote drift 合併於 client 且輸出凍結

CLI 的 drift 動詞在 remote 模式 SHALL：自 server 取規格面報告、basis 與 store 面輸入，於本機收集工作區事實並以該 store 面輸入計算工作區面（有 workspace 且 git 可用時）、經引擎唯一合併器合併後以與 fs 模式相同的渲染路徑輸出——同一 change 內容下 remote（有 checkout）與 fs 模式的人眼與 --json 輸出 SHALL 逐位元同形。本機無 checkout 或 git 不可用時，工作區面 SHALL 依三值語意如實標示不可得（SHALL NOT 呈現為乾淨），規格面照常回報，動詞成功結束。SHALL NOT 存在第二個合併實作。

#### Scenario: 有 checkout 輸出同形

- **WHEN** 同一 change 內容分別於 fs 模式與 remote 模式（本機 checkout 存在）執行 drift --json
- **THEN** 兩者輸出的結構與欄位形狀一致；規格面內容相同、工作區面反映本機事實

#### Scenario: 無 checkout 誠實標示

- **WHEN** 於無本機 workspace 的目錄以 remote 模式執行 drift
- **THEN** 動詞成功；輸出含規格面報告與工作區面不可得的既有標示；不出現「工作區乾淨」的斷言

#### Scenario: server 失敗不出偽報告

- **WHEN** server 回 503 期間執行 remote drift
- **THEN** 動詞以既有 remote 錯誤訊息失敗（非零 exit code）；不輸出缺規格面的部分報告

---

<!-- @trace
source: server-drift-api
updated: 2026-07-16
code:
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/remote_drift.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/drift.rs
  - crates/speclink-protocol/src/drift.rs
  - crates/speclink-protocol/src/lib.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/typed_client.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/drift_api.rs
-->

---
### Requirement: wire 與引擎型別映射單點往返

drift 的 wire DTO 與引擎規格面型別之間的轉換 SHALL 單點實作於 Host 層（唯一同時依賴引擎與 protocol、且 server 與 client 皆已依賴的組合點）且可往返：引擎型別經 wire 序列化再反序列化 SHALL 結構相等；引擎核心型別 SHALL NOT 為 wire 需求增加序列化標註；protocol crate SHALL NOT 依賴引擎。規格面計算的組合點 SHALL 由 Host 以專用查詢入口封裝，回傳同一 snapshot 的規格面報告與 basis digests；橋接的引擎 Store 視圖 SHALL NOT 對 Host 之外公開。

#### Scenario: 往返結構相等

- **WHEN** 對含多筆規格假設（ADDED／MODIFIED／RENAMED 各式 reason）與規格面維度的報告執行「引擎型別 → wire DTO → 引擎型別」往返
- **THEN** 往返後結構相等；wire JSON 欄位皆為 camelCase

<!-- @trace
source: server-drift-api
updated: 2026-07-16
code:
  - crates/speclink-cli/src/commands.rs
  - crates/speclink-cli/src/remote_commands.rs
  - crates/speclink-cli/tests/remote_drift.rs
  - crates/speclink-host/src/bridge.rs
  - crates/speclink-host/src/drift.rs
  - crates/speclink-protocol/src/drift.rs
  - crates/speclink-protocol/src/lib.rs
  - crates/speclink-remote/src/client.rs
  - crates/speclink-remote/tests/typed_client.rs
  - crates/speclink-server/src/app.rs
  - crates/speclink-server/src/routes.rs
  - crates/speclink-server/tests/drift_api.rs
-->
# teamstore-contract Specification

## Purpose

TeamStore 的儲存契約：manifest 宣告契約版本、能力與等級，讀取以 typed Result 區分失敗類別，文件以 Project 與 Repo scope 的邏輯 locator 定址，consistent snapshot 提供固定時點視圖，Unit of Work 為唯一寫入路徑且 commit 原子，並含不可變歷史、transactional outbox 與 versioned bundle 的匯出匯入。本 capability 保證任意實作都以同一套可重用的 conformance suite 驗證——契約由測試釘死，不靠文件口頭約定。

## Requirements

### Requirement: manifest 宣告契約版本、能力與等級

TeamStore 實作 SHALL 經 manifest 回報 contract version、driver 識別、capability 集合（snapshot、cas、transaction、history、outbox、migration、backup、cluster）與能力等級（local-single-writer、single-node、cluster 三級之一）。宣告 SHALL 可被程式讀取以供啟動驗證與 conformance 分級；宣告 single-node（含）以上等級者，snapshot、cas、transaction、history 與 outbox capabilities SHALL 全數存在。

#### Scenario: 宣告 single-node 但缺必要能力即不通過

- **WHEN** 某實作的 manifest 宣告 single-node 等級但 capability 集合缺 outbox，對其執行 conformance suite
- **THEN** suite 於能力檢查階段判整體不通過，並指出缺失的 capability


<!-- @trace
source: teamstore-contract-v2
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-store/Cargo.toml
  - crates/speclink-store/src/conformance/mod.rs
  - crates/speclink-store/src/error.rs
  - crates/speclink-store/src/lib.rs
  - crates/speclink-store/src/memory.rs
  - crates/speclink-store/src/types.rs
  - crates/speclink-store/src/uow.rs
-->

---
### Requirement: 讀取以 typed Result 區分失敗類別

TeamStore 的所有讀取 SHALL 回傳 typed Result：故障以封閉錯誤集合表達——not_found、permission_denied、revision_conflict（帶 expected 與 actual revision）、unavailable、corrupt（帶原因）、backend（帶來源描述），各附穩定錯誤碼字串；「文件不存在」的正常情形 SHALL 以成功值內的空值表達，SHALL NOT 與故障混同。實作 SHALL NOT 以空集合或預設值掩蓋錯誤。

#### Scenario: 損壞文件回 corrupt 而非不存在

- **WHEN** snapshot 內某文件的持久化內容損壞，讀取該文件
- **THEN** 回傳 corrupt 錯誤並帶原因；同一呼叫對真正不存在的文件回傳成功的空值，兩者可被呼叫端區分


<!-- @trace
source: teamstore-contract-v2
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-store/Cargo.toml
  - crates/speclink-store/src/conformance/mod.rs
  - crates/speclink-store/src/error.rs
  - crates/speclink-store/src/lib.rs
  - crates/speclink-store/src/memory.rs
  - crates/speclink-store/src/types.rs
  - crates/speclink-store/src/uow.rs
-->

---
### Requirement: 文件定址採 Project 與 Repo scope 的邏輯 locator

TeamStore 的文件身分 SHALL 為 ProjectId、RepoId 與 DocumentId 三元組；DocumentId SHALL 為封閉的邏輯種類集合（change metadata、change artifact、change evidence、canonical spec、discussion、workflow config、archived 文件、language 與 board order），SHALL NOT 以實體路徑作跨媒介身分。board order 為 scope 層級單文件種類（同 workflow config 形狀），change evidence 為 change 層級單文件種類（一個 change 至多一份，內容為 evidence 記錄的序列化文字，store 不解讀），三個官方 driver 的編碼／解碼與 conformance suite SHALL 涵蓋兩者，export bundle 的泛用文件列舉 SHALL 自動包含兩者。跨 project 或跨 repo 的讀寫 SHALL 被隔離：對不屬於該 scope 的文件操作回 not_found 或 permission_denied，SHALL NOT 回傳其他 tenant 的資料。

#### Scenario: tenant scope 隔離

- **WHEN** 以 repo A 的 scope 讀取僅存在於 repo B 的同名 canonical spec
- **THEN** 回傳成功的空值或 permission_denied（依實作的權限模型），絕不回傳 repo B 的內容

#### Scenario: board order 種類 round-trip

- **WHEN** 對任一官方 driver 以 UoW 寫入 board order 文件後重開 store 讀取
- **THEN** 內容逐位元組一致，且該文件出現於同 scope 的 export bundle

#### Scenario: change evidence 種類 round-trip

- **WHEN** 對任一官方 driver 以 UoW 寫入某 change 的 evidence 文件後重開 store 讀取
- **THEN** 內容逐位元組一致，且該文件出現於同 scope 的 export bundle

#### Scenario: 封閉集合外的種類不存在

- **WHEN** 檢視 DocumentId 的種類定義
- **THEN** 集合恰為 change metadata、change artifact、change evidence、canonical spec、discussion、workflow config、archived 文件、language 與 board order 九種，無其他變體


<!-- @trace
source: remote-task-evidence
updated: 2026-08-25
-->

---
### Requirement: consistent snapshot 提供固定時點視圖

snapshot SHALL 回傳綁定單一 project revision 的一致視圖：視圖內全部文件屬同一時點，讀取 SHALL NOT 受後續並行 commit 影響。

#### Scenario: mixed snapshot 不出現

- **WHEN** 讀方取得 snapshot 後，寫方 commit 修改了視圖內兩份文件，讀方繼續讀取該 snapshot
- **THEN** 讀方看到的兩份文件皆為 snapshot 時點的舊內容（不出現一新一舊），且 snapshot 的 revision 不變


<!-- @trace
source: teamstore-contract-v2
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-store/Cargo.toml
  - crates/speclink-store/src/conformance/mod.rs
  - crates/speclink-store/src/error.rs
  - crates/speclink-store/src/lib.rs
  - crates/speclink-store/src/memory.rs
  - crates/speclink-store/src/types.rs
  - crates/speclink-store/src/uow.rs
-->

---
### Requirement: Unit of Work 是唯一寫入路徑且 commit 原子

寫入 SHALL 經 begin unit of work（攜 command 識別與 actor）暫存、以 commit 一次生效：全部文件寫入、project revision 遞增、immutable history 追加與 outbox 追加 SHALL 全部生效或全部不生效。每筆暫存寫入 SHALL 攜 expected revision（新建為「不得已存在」）；任一 expected revision 與現況不符時 commit SHALL 整體以 revision_conflict 拒絕並回報衝突文件、expected 與 actual。rollback SHALL 丟棄暫存且不留任何狀態。

#### Scenario: CAS race 恰一方成功

- **WHEN** 兩個 unit of work 以相同的 expected revision 併發修改同一文件並先後 commit
- **THEN** 恰一方成功；另一方收到 revision_conflict 且能讀出對方造成的 actual revision；store 內容等於成功方的寫入

#### Scenario: partial commit 不外洩

- **WHEN** 以故障注入使 commit 在文件寫入完成後、outbox 追加前崩潰，隨後重建 store 檢視
- **THEN** 該 commit 的全部效果不可見（文件、project revision、history、outbox 皆為 commit 前狀態）


<!-- @trace
source: teamstore-contract-v2
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-store/Cargo.toml
  - crates/speclink-store/src/conformance/mod.rs
  - crates/speclink-store/src/error.rs
  - crates/speclink-store/src/lib.rs
  - crates/speclink-store/src/memory.rs
  - crates/speclink-store/src/types.rs
  - crates/speclink-store/src/uow.rs
-->

---
### Requirement: immutable history 記錄每次文件變更

每次 commit SHALL 為受影響文件追加不可變的 revision 記錄：actor、UTC 時間戳、內容 digest 與來源 command 識別；刪除 SHALL 以 tombstone revision 表達。歷史 SHALL 可依文件查詢；回退 SHALL 以追加新 revision 表達，SHALL NOT 改寫或刪除既有歷史。

#### Scenario: 刪除留 tombstone 且歷史完整

- **WHEN** 對同一文件先後 commit 建立、修改、刪除三個 unit of work，查詢其歷史
- **THEN** 歷史含三筆 revision（建立、修改、tombstone），各帶 actor 與時間戳；tombstone 後讀取該文件回成功的空值


<!-- @trace
source: teamstore-contract-v2
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-store/Cargo.toml
  - crates/speclink-store/src/conformance/mod.rs
  - crates/speclink-store/src/error.rs
  - crates/speclink-store/src/lib.rs
  - crates/speclink-store/src/memory.rs
  - crates/speclink-store/src/types.rs
  - crates/speclink-store/src/uow.rs
-->

---
### Requirement: transactional outbox 與 cursor 重讀

commit 攜帶的 event records SHALL 與文件寫入同原子落入 outbox；outbox SHALL 支援自任意 cursor 起重讀與消費確認，事件 SHALL 帶可重播的順序。outbox 追加失敗時整個 commit SHALL 不生效。

#### Scenario: crash recovery 後三者一致

- **WHEN** 以故障注入於 commit 各階段邊界崩潰並重建 store，比對文件內容、project revision、history 與 outbox
- **THEN** 四者一致地反映「該 commit 完整生效」或「完全未發生」其中之一；自 cursor 0 重讀 outbox 得到與生效 commit 一一對應的事件序列


<!-- @trace
source: teamstore-contract-v2
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-store/Cargo.toml
  - crates/speclink-store/src/conformance/mod.rs
  - crates/speclink-store/src/error.rs
  - crates/speclink-store/src/lib.rs
  - crates/speclink-store/src/memory.rs
  - crates/speclink-store/src/types.rs
  - crates/speclink-store/src/uow.rs
-->

---
### Requirement: export 與 import 以 versioned bundle 往返

export SHALL 輸出帶格式版本、scope、project revision 與逐文件 digest 的 bundle；import SHALL 驗證格式版本與 digest、依指定模式（全新建立或覆蓋）套用並回報逐文件結果，驗證失敗 SHALL 拒絕且不部分套用。全新建立模式的前置 SHALL 為「目標 scope 不持有任何文件」——scope 持有任何文件（無論是否與 bundle 內文件同名）即 SHALL 整筆拒絕（backend 類別）、不部分套用、scope 狀態不變；SHALL NOT 以「bundle 內文件是否已存在」代替此檢查。conformance suite SHALL 含此邊界的 gate，全部實作 SHALL 一致通過。

#### Scenario: round-trip 內容一致

- **WHEN** 對含多份文件的 repo 執行 export，將 bundle import 到全新 store 後逐文件比對
- **THEN** 全部文件內容一致；新 store 的每份文件歷史以 import 為起點；digest 驗證通過

#### Scenario: 全新建立模式拒絕非空 scope

- **WHEN** 目標 scope 已持有一份 bundle 外的文件 X，以全新建立模式 import 只含文件 Y 的 bundle
- **THEN** import 整筆拒絕且錯誤為 backend 類別；scope 仍只持有 X、project revision 未動、無任何 Y 的痕跡

#### Scenario: 覆蓋模式不受空 scope 前置影響

- **WHEN** 對持有既有文件的 scope 以覆蓋模式 import 同名文件的 bundle
- **THEN** import 成功且逐文件結果回報覆蓋；全新建立前置不適用於此模式


<!-- @trace
source: import-createnew-gate
updated: 2026-07-16
code:
  - crates/speclink-store-sqlite/src/lib.rs
  - crates/speclink-store/src/conformance/mod.rs
  - crates/speclink-store/src/memory.rs
-->

---
### Requirement: conformance suite 可對任意實作重用執行

conformance suite SHALL 以可重用的程式入口輸出：任何 TeamStore 實作（官方 driver、自訂 Store、參考實作）SHALL 能以同一入口受測。suite SHALL 至少涵蓋 CAS race、mixed snapshot、partial commit、outbox failure、crash recovery 與 tenant scope 六類情境；受測實作宣告的每項 capability SHALL 有對應測試，任一失敗即整體不通過。隨契約提供的 in-memory reference store SHALL 通過完整 suite，作為契約可實作性的證明。

#### Scenario: in-memory reference 通過完整 suite

- **WHEN** 以 conformance 入口對 in-memory reference store 執行完整 suite
- **THEN** 六類情境與能力對應測試全數通過，suite 回報通過的 capability 清單與契約版本

<!-- @trace
source: teamstore-contract-v2
updated: 2026-07-13
code:
  - Cargo.lock
  - Cargo.toml
  - crates/speclink-store/Cargo.toml
  - crates/speclink-store/src/conformance/mod.rs
  - crates/speclink-store/src/error.rs
  - crates/speclink-store/src/lib.rs
  - crates/speclink-store/src/memory.rs
  - crates/speclink-store/src/types.rs
  - crates/speclink-store/src/uow.rs
-->
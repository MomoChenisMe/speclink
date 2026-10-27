## ADDED Requirements

### Requirement: manifest 宣告契約版本、能力與等級

TeamStore 實作 SHALL 經 manifest 回報 contract version、driver 識別、capability 集合（snapshot、cas、transaction、history、outbox、migration、backup、cluster）與能力等級（local-single-writer、single-node、cluster 三級之一）。宣告 SHALL 可被程式讀取以供啟動驗證與 conformance 分級；宣告 single-node（含）以上等級者，snapshot、cas、transaction、history 與 outbox capabilities SHALL 全數存在。

#### Scenario: 宣告 single-node 但缺必要能力即不通過

- **WHEN** 某實作的 manifest 宣告 single-node 等級但 capability 集合缺 outbox，對其執行 conformance suite
- **THEN** suite 於能力檢查階段判整體不通過，並指出缺失的 capability

### Requirement: 讀取以 typed Result 區分失敗類別

TeamStore 的所有讀取 SHALL 回傳 typed Result：故障以封閉錯誤集合表達——not_found、permission_denied、revision_conflict（帶 expected 與 actual revision）、unavailable、corrupt（帶原因）、backend（帶來源描述），各附穩定錯誤碼字串；「文件不存在」的正常情形 SHALL 以成功值內的空值表達，SHALL NOT 與故障混同。實作 SHALL NOT 以空集合或預設值掩蓋錯誤。

#### Scenario: 損壞文件回 corrupt 而非不存在

- **WHEN** snapshot 內某文件的持久化內容損壞，讀取該文件
- **THEN** 回傳 corrupt 錯誤並帶原因；同一呼叫對真正不存在的文件回傳成功的空值，兩者可被呼叫端區分

### Requirement: 文件定址採 Project 與 Repo scope 的邏輯 locator

TeamStore 的文件身分 SHALL 為 ProjectId、RepoId 與 DocumentId 三元組；DocumentId SHALL 為封閉的邏輯種類集合（change metadata、change artifact、canonical spec、discussion、workflow config 與 archived 文件），SHALL NOT 以實體路徑作跨媒介身分。跨 project 或跨 repo 的讀寫 SHALL 被隔離：對不屬於該 scope 的文件操作回 not_found 或 permission_denied，SHALL NOT 回傳其他 tenant 的資料。

#### Scenario: tenant scope 隔離

- **WHEN** 以 repo A 的 scope 讀取僅存在於 repo B 的同名 canonical spec
- **THEN** 回傳成功的空值或 permission_denied（依實作的權限模型），絕不回傳 repo B 的內容

### Requirement: consistent snapshot 提供固定時點視圖

snapshot SHALL 回傳綁定單一 project revision 的一致視圖：視圖內全部文件屬同一時點，讀取 SHALL NOT 受後續並行 commit 影響。

#### Scenario: mixed snapshot 不出現

- **WHEN** 讀方取得 snapshot 後，寫方 commit 修改了視圖內兩份文件，讀方繼續讀取該 snapshot
- **THEN** 讀方看到的兩份文件皆為 snapshot 時點的舊內容（不出現一新一舊），且 snapshot 的 revision 不變

### Requirement: Unit of Work 是唯一寫入路徑且 commit 原子

寫入 SHALL 經 begin unit of work（攜 command 識別與 actor）暫存、以 commit 一次生效：全部文件寫入、project revision 遞增、immutable history 追加與 outbox 追加 SHALL 全部生效或全部不生效。每筆暫存寫入 SHALL 攜 expected revision（新建為「不得已存在」）；任一 expected revision 與現況不符時 commit SHALL 整體以 revision_conflict 拒絕並回報衝突文件、expected 與 actual。rollback SHALL 丟棄暫存且不留任何狀態。

#### Scenario: CAS race 恰一方成功

- **WHEN** 兩個 unit of work 以相同的 expected revision 併發修改同一文件並先後 commit
- **THEN** 恰一方成功；另一方收到 revision_conflict 且能讀出對方造成的 actual revision；store 內容等於成功方的寫入

#### Scenario: partial commit 不外洩

- **WHEN** 以故障注入使 commit 在文件寫入完成後、outbox 追加前崩潰，隨後重建 store 檢視
- **THEN** 該 commit 的全部效果不可見（文件、project revision、history、outbox 皆為 commit 前狀態）

### Requirement: immutable history 記錄每次文件變更

每次 commit SHALL 為受影響文件追加不可變的 revision 記錄：actor、UTC 時間戳、內容 digest 與來源 command 識別；刪除 SHALL 以 tombstone revision 表達。歷史 SHALL 可依文件查詢；回退 SHALL 以追加新 revision 表達，SHALL NOT 改寫或刪除既有歷史。

#### Scenario: 刪除留 tombstone 且歷史完整

- **WHEN** 對同一文件先後 commit 建立、修改、刪除三個 unit of work，查詢其歷史
- **THEN** 歷史含三筆 revision（建立、修改、tombstone），各帶 actor 與時間戳；tombstone 後讀取該文件回成功的空值

### Requirement: transactional outbox 與 cursor 重讀

commit 攜帶的 event records SHALL 與文件寫入同原子落入 outbox；outbox SHALL 支援自任意 cursor 起重讀與消費確認，事件 SHALL 帶可重播的順序。outbox 追加失敗時整個 commit SHALL 不生效。

#### Scenario: crash recovery 後三者一致

- **WHEN** 以故障注入於 commit 各階段邊界崩潰並重建 store，比對文件內容、project revision、history 與 outbox
- **THEN** 四者一致地反映「該 commit 完整生效」或「完全未發生」其中之一；自 cursor 0 重讀 outbox 得到與生效 commit 一一對應的事件序列

### Requirement: export 與 import 以 versioned bundle 往返

export SHALL 輸出帶格式版本、scope、project revision 與逐文件 digest 的 bundle；import SHALL 驗證格式版本與 digest、依指定模式（全新建立或覆蓋）套用並回報逐文件結果，驗證失敗 SHALL 拒絕且不部分套用。

#### Scenario: round-trip 內容一致

- **WHEN** 對含多份文件的 repo 執行 export，將 bundle import 到全新 store 後逐文件比對
- **THEN** 全部文件內容一致；新 store 的每份文件歷史以 import 為起點；digest 驗證通過

### Requirement: conformance suite 可對任意實作重用執行

conformance suite SHALL 以可重用的程式入口輸出：任何 TeamStore 實作（官方 driver、自訂 Store、參考實作）SHALL 能以同一入口受測。suite SHALL 至少涵蓋 CAS race、mixed snapshot、partial commit、outbox failure、crash recovery 與 tenant scope 六類情境；受測實作宣告的每項 capability SHALL 有對應測試，任一失敗即整體不通過。隨契約提供的 in-memory reference store SHALL 通過完整 suite，作為契約可實作性的證明。

#### Scenario: in-memory reference 通過完整 suite

- **WHEN** 以 conformance 入口對 in-memory reference store 執行完整 suite
- **THEN** 六類情境與能力對應測試全數通過，suite 回報通過的 capability 清單與契約版本

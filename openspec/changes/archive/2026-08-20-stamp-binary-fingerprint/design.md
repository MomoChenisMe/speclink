## Context

品質站的內容指紋鏈：core 的 station 模組持有共用實作——content_fingerprint（文字：CRLF→LF 正規化後 SHA-256）、fingerprint_scope（逐檔算指紋，讀檔閉包簽名回傳文字）、freshness（失效判定）。五個消費端：core 的 review 與 verify 模組（本機蓋章與轉出）、封存守門（archive 的 guard_stale_stamps 讀內容錨）、CLI 的 remote 章路徑（工作樹持有者算指紋後提交）、desktop core 的 freshness 重算（reviewStatus）。凍結端（change diff 層）對 binary／非 UTF-8 檔既定設計為「僅雜湊、無 hunk」地收進 frozen scope，落章端卻只吃 UTF-8——組合成 2026-08-20 實際撞上的死鎖（詳 proposal）。

## Goals / Non-Goals

**Goals:**

- 非 UTF-8 scope 檔可落章、可判失效：位元組 SHA-256 分流
- 文字檔指紋逐位元組不變：既有已落的章零影響
- 四個消費端走同一分流（共用實作單點改）

**Non-Goals:**

- 凍結端行為、remote stamp 協定形狀、審查子代理對 binary 的呈現（見 proposal Non-Goals）
- server 端任何改動（指紋計算全在 client／本機側）

## Decisions

### D1 指紋分流入口

station 模組新增 content_fingerprint_bytes：輸入位元組，可解為 UTF-8 → 轉呼既有 content_fingerprint（文字規則不動，回歸相容由既有測試釘住）；不可解 → 原始位元組直接 SHA-256。既有 content_fingerprint 函式保留原名原簽名——review／verify 的既有測試與 desktop core 的測試都在用它，砍掉會放大波及面而無收益。

### D2 讀檔閉包換位元組

fingerprint_scope 與 freshness 的讀檔閉包簽名由回傳文字改回傳位元組（Option 的 Vec<u8>）；閉包回 None 維持「缺檔或 I/O 讀取失敗」語意——蓋章側在 file_exists 分流後，present 檔回 None 即 I/O 錯誤、照舊 bail。UTF-8 判定收進 D1 的分流入口，不在各呼叫端重複。五個消費端的閉包（core review／verify、封存守門 guard_stale_stamps、CLI remote 章路徑、desktop core 重算）由編譯器帶著改，全部從讀文字改讀位元組。

**替代方案**：閉包簽名不動、另加一個 bytes 閉包參數（雙閉包同傳，呼叫端更囉唆且兩者可能不一致，捨）；在呼叫端各自 fallback（分流邏輯散四處，捨）。

### D3 規格的最終態

review-station 的內容指紋錨與失效判定、verify-station 的驗證指紋錨與失效判定兩條 requirement 走 MODIFIED（delta 已隨本 change 落）：hash 定義改為分流敘述、「非 UTF-8 使蓋章失敗」改為「I/O 讀取失敗使蓋章失敗（非 UTF-8 不是讀取失敗）」；死檔跳過、聯集全消失拒章、missing 清單分割驗證、任務錨、曝光紅線全部原文保留。

## Implementation Contract

**可觀察行為**：

- 工單 Scope 含存在的非 UTF-8 檔時 review stamp 與 verify stamp 落章成功，reviewed_scope／verified_scope 記該檔原始位元組 SHA-256
- 落章後該檔位元組變動 → freshness 判 stale；未變動 → fresh；desktop 的 reviewStatus 重算同語意
- 純文字 scope 檔的指紋值與本 change 之前逐位元組相同
- present 檔 I/O 讀取失敗（如 EACCES）仍拒章，訊息維持既有事實陳述形

**介面**：station 模組新增一個 bytes 指紋入口；fingerprint_scope 與 freshness 的讀檔閉包簽名改位元組；content_fingerprint 原名原簽名保留。

**失敗形**：拒章路徑（聯集全消失、I/O 錯誤）的 exit code 與訊息語意不變。

**驗收**：cargo test -p speclink-core 全綠（新增：非 UTF-8 指紋分流、含 binary 的 stamp 與 freshness 案例；既有文字指紋測試不改值）；cargo test -p speclink-cli --test it 全綠；cargo test -p speclink-desktop-core 全綠；speclink validate stamp-binary-fingerprint 通過。

**範圍邊界**：in scope——crates/speclink-core/src/station.rs、crates/speclink-core/src/review.rs、crates/speclink-core/src/verify.rs、crates/speclink-core/src/archive.rs（封存守門閉包）、crates/speclink-core/src/util.rs（bytes 讀檔 helper）、crates/speclink-cli/src/verbs/station.rs、apps/desktop/core/src/query.rs、兩站規格 delta。out of scope——凍結端（change diff 層）、speclink-server、speclink-protocol、store 三驅動、desktop 前端。

## Risks / Trade-offs

- [位元組指紋對文字檔誤用會破壞行尾不失效語意] → 分流以「可否 UTF-8 解讀」為唯一判準且收在單一入口；行尾 scenario 測試維持
- [遺漏某個消費端的閉包] → 簽名變更由編譯器強制全數跟上，漏改即編譯錯
- [大型 binary 讀進記憶體算雜湊] → scope 檔本來就整檔讀入算文字雜湊，行為等量；不做串流優化（無實際需求）
- [合法 UTF-8 的二進位內容（純 ASCII fixture、含 0x00 的資料檔）走文字分支，內容中 0D 0A→0A 的位元組變動指紋不變] → 已知取捨：分流判準是「可否 UTF-8 解讀」而非「是不是 binary」，與既有文字檔行尾不失效語意一致；要改判準屬規格變更

## Migration Plan

既有章零遷移（文字指紋值不變）。回滾＝revert 單一 commit。

## Open Questions

無。

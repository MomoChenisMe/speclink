## 1. 引擎：驗證輪歸因重寫

- [x] 1.1 驗證輪以內容移動歸因並回走快照鏈（design D1、D2；涵蓋 spec 需求「frozen snapshot 綁定 discovery 與 validation patch」的移動歸因與 adjacent 段）：`TicketBinding` 增列各輪 patchHash 鏈欄位（新→舊），CLI 的 Ticket→TicketBinding 組裝處補傳；`resolve_validation_scope` 改為對上一輪快照 dirtyFilesAtCapture 逐條比對現況雜湊——preserved 內走既有 remediation_segment，preserved 外沿鏈回走取最近收錄輪重建凍結後狀態（存的 afterText 或 base commit＋delta 重放）產生 adjacent 段。先寫紅測（host 直測 fixture 造快照鏈）：(a) Round 1 收錄 A、B，findings 只點 A，修復改 A＋B → 驗證 patch 含兩段、B 自其凍結後狀態起算；(b) Round 2 後再改 B → Round 3 沿鏈取 Round 2 快照重建，正常 resolved；(c) 內容未動的候選檔不進 patch。再實作至綠。檔案：`crates/speclink-host/src/change_diff.rs`、`crates/speclink-cli/src/commands.rs`。驗證：cargo test -p speclink-host 新增案例通過。 <!-- speclink-task:tsk_01KZ864P5XB1VWN8WV9Q9ZWJWW -->
- [x] 1.2 never-captured 檔註記＋放行、驗證輪 needsInput 消滅（design D3；涵蓋 spec 需求「歧義 scope 必須 fail closed 並以 hash-pinned selection 解鎖」的 needsInput 限 discovery 與「frozen snapshot 綁定…」的 outOfScopeChanged）：刪除 validation 的 PreviouslyDirtyChanged needsInput 回傳路徑與對應 AmbiguityReason 變體，resolver 回傳增列範圍外變動清單；回走鏈中任一輪快照缺失、工單 patchHash 與快照不符維持非零硬錯誤且訊息點名缺失的 patchHash。先寫紅測：(a) discovery 時未收錄的髒檔於驗證期變動 → resolved、清單含該路徑、patch 不含；(b) 鏈中快照被移除 → 硬錯誤。再實作至綠。檔案：`crates/speclink-host/src/change_diff.rs`。驗證：cargo test -p speclink-host 通過。 <!-- speclink-task:tsk_01KZ864P5X7W1HVYK7QW3MQXM9 -->

## 2. CLI：payload 與 human 契約

- [x] 2.1 出身標記與範圍外註記進輸出契約（design D4；涵蓋 spec 需求「review scope 的 human 與 JSON 契約」）：resolved --json 的 files 於 validation phase 帶 attribution（"finding"｜"adjacent"｜"new"）、discovery 缺席；payload 增列 outOfScopeChanged:string[]（恆存在）；human 路徑於清單非空時輸出一行範圍外變動路徑。先寫紅測（CLI 整合測試跑完整 verb 流程斷言 JSON 欄位值與 human 行、discovery payload 無 attribution 欄位），再實作。檔案：`crates/speclink-cli/src/commands.rs`、`crates/speclink-cli/tests/it/review_verbs.rs`。驗證：cargo test -p speclink-cli --test it review_verbs 通過。 <!-- speclink-task:tsk_01KZ864P5X2P9G2WAV8BAFCW8K -->
- [x] 2.2 discovery 回歸釘住：既有 discovery needsInput 全情境（dirtyAtStart、active overlap、baseline 缺失、candidate 漂移）與 hash-pinned selection 測試維持全綠、行為逐位元不變；原依賴 validation needsInput 的測試改寫為新語意（resolved＋adjacent 或 outOfScopeChanged）。檔案：`crates/speclink-cli/tests/it/review_verbs.rs`。驗證：cargo test -p speclink-cli --test it 全綠。 <!-- speclink-task:tsk_01KZ864P5XTRX3HV16V2RFETRM -->

## 3. 技能文本與產物

- [x] 3.1 review 技能文本兩段改寫（design D5；涵蓋 spec 需求「審查流程的技能行為」的 adjacent 確認與 needsInput 限 discovery）：step 3 的 needsInput 處置段改為 discovery 專屬（快照缺失硬錯誤的處置文字保留）；validation 簡報段增列「attribution=adjacent 的段落逐段確認確屬本次修復、不屬於者以 regression 回報」；主線對 outOfScopeChanged 非空時原樣轉知使用者、不入審查面。`MARKER_VERSION` 遞增一版。檔案：`crates/speclink-core/assets/skills/review.md`、`crates/speclink-core/src/init.rs`。驗證：cargo test -p speclink-core（golden 尚未再生前預期紅，由 3.2 收綠）。 <!-- speclink-task:tsk_01KZ864P5XY8X47B2GKPBJV71J -->
- [x] 3.2 golden 與 assets.lock 再生（單次、lock 最後）：UPDATE_GOLDEN=1 再生受影響 snapshot，人工過目 diff 限於 review 技能兩段文字與版本戳；最後 UPDATE_ASSETS_LOCK=1 再生 assets.lock（一次 bump 只再生一次，不得提前）。檔案：`crates/speclink-core/tests/golden`。驗證：cargo test -p speclink-core 全綠。 <!-- speclink-task:tsk_01KZ864P5X7854529T13HH1QZZ -->

## 4. 收尾驗證

- [x] 4.1 死結情境端對端重演與全套綠：以 scratch 專案（安裝流程外的 ./target/debug/speclink）重演本次實測情境——discovery 全候選凍結→修復動到未點名檔→Round 2 resolved 含 adjacent 段→再修→Round 3 resolved 沿鏈回走，全程無 needsInput；最後 cargo test --workspace 全綠。驗證：手動情境五步輸出符合 Implementation Contract 行為 1–3，workspace 測試 0 failed。 <!-- speclink-task:tsk_01KZ864P5X95Q8B95PRP2CB213 -->

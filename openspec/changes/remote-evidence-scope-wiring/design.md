## Context

fs 模式的品質站 scope 解析自專案 Store 讀 TouchedRecord（apply 的 task done 落盤的 evidence 記錄），touched 認領自動成立。remote 模式下 task done 把同一份 evidence 送進 server store（remote-task-evidence 落地），但站台的 remote 分支在組 scope 請求時另建了一個本地 FsStore 去讀 evidence——本地無檔，touched 恆空。既有前例：remote drift 已把 store 端 evidence 隨 drift 輸入送到 checkout 端計算（protocol 的 DriftChangeInputs 帶 evidence，CLI 以 RemoteDriftStore 消費）。typed remote client 已有 change_evidence 讀取函式，現無生產呼叫者。

## Goals / Non-Goals

**Goals:**

- remote 模式的 review scope 與 verify scope 自 server evidence 自動取得 touched 認領，行為與 fs 模式同語意。
- 併行認領守門（other claims）於 remote 生效。
- 正典 change-diff-scope 的 remote 條款改為可成立的敘述。
- remote scope 測試改走真實資料來源（mock server 供應 evidence），不再手塞本地檔。

**Non-Goals:**

- 不動 server 與 protocol（端點與 wire 已在）。
- 不動 assets/skills 技能敘述（避免版號三連動）。
- 不做 desktop 遠端勾任務 touched 收集、不做 evidence 回填（既有紅線）。

## Decisions

- **D1 touched 來源走 client.change_evidence，不走 context projection**：remote 分支組 scope 請求前，以 typed client 讀該 change 的 evidence，將其 touched files 餵進 scope 請求的 touched 認領。捨棄「把 evidence 加進 .speclink/context 投影」的替代方案——投影是唯讀快取面、非動詞資料通道，且 drift 前例已示範 client 直讀。
- **D2 多 actor 聯集**：evidence 記錄含多筆 entries（不同 actor、不同 head commit）時，touched 取全部 entries 的聯集——與 fs 模式 TouchedRecord 的 all_files 語意一致，不引入新的合併規則。
- **D3 head commit 僅存證**：scope 解析只消費 touched 路徑；evidence 內的 head commit 不參與 baseline 或 candidate 計算（fs 模式亦然）。審查端 checkout 未 fetch 到該 commit 不構成障礙，不新增任何 fetch 要求。
- **D4 other claims 守門逐 change 讀取**：remote 分支以 client 列出其他 active changes，逐一讀 evidence 作為 other claims 輸入；某 change 無 evidence 視為零認領（與 fs 模式檔案缺席同語意）。捨棄「remote 放棄守門」——守門存在是為多人防撞，remote 正是多人場景，放棄與動機相反。
- **D5 缺席 fail-closed 不變**：evidence 缺席或 touched 為空時，維持 EmptyTouched 的 needsInput 與 --base、--candidate-hash、--include-hunk 手動跳脫閥；本刀不改任何 fail-closed 語意，只補資料來源。
- **D6 讀取失敗即中止**：evidence 讀取遇離線、認證失效或 server 錯誤時，command 非零結束、不寫 baseline 與 snapshot——沿正典 remote 條款既有的錯誤語意，不靜默降級成空認領（靜默降級會把連線問題偽裝成「沒有 touched」的 needsInput，誤導使用者）。

## Implementation Contract

- 行為：remote workspace 下，change 完成 task done（帶 touched files）後，review scope 與 verify scope 不需任何旗標即回 resolved payload，欄位與 fs 模式逐位一致；同 workspace 另一 active change 的 evidence 與本 change 重疊時，回報 other-claims 守門結果，與 fs 模式同形。
- 驗證：crates/speclink-cli/tests/it/remote_verb_parity.rs 的 remote scope 測試群改由 mock server 的 evidence 端點供應 touched（移除手塞本地檔的 helper 用法）；新增「evidence 缺席時 needsInput 與 EmptyTouched 理由」與「多 actor entries 聯集」兩測試；驗證指令 cargo test -p speclink-cli --test it。
- 邊界：本刀只動 crates/speclink-cli 的站台 remote 分支與其測試；server、protocol、engine 的 scope resolver 零改動。

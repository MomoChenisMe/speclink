## Summary

引擎寫檔單點 util::write_file 改為原子寫（同目錄暫存檔＋rename），並把兩處旁路（CLI 設定編輯、desktop 設定寫入）收編到同一入口——並行讀者從此只會讀到舊全文或新全文，消滅共享真相檔案的撕裂讀。

## Motivation

本地模式的真相是 openspec/ 檔案樹與 .speclink.yaml、openspec/config.yaml。現行 util::write_file（crates/speclink-core/src/util.rs）是普通 std::fs::write——truncate 後寫入，寫入過程中另一個 process（desktop、CLI、agent、編輯器）讀同一檔案可能讀到空檔或半份內容。症狀鏈具體且事後極難歸因：.speclink.yaml 讀到半份 → 引擎 fail-closed 視為非專案 → 專案在 desktop 上短暫「消失」；tasks.md 或 artifact 讀到半份 → 清單與進度呈現錯誤狀態。

討論 desktop-cli-multi-workspace-concurrency 裁定：跨 process 檔案鎖（防遺失更新）因未曾實際咬人而刻意延後，原子寫因成本近零、能消滅一整類不可重現 bug 而先行。codebase 已有同款先例：crates/speclink-host/src/change_diff.rs 的 write_baseline 即為暫存檔＋rename（註解自承防 half-written baseline）。

## Proposed Solution

1. util::write_file 改為原子落盤：於目的檔同目錄寫入暫存檔（唯一後綴，避免並行寫者互撞暫存名）→ std::fs::rename 到目的路徑。rename 在同一檔案系統內原子，讀者任一時點看到的都是舊全文或新全文。
2. Windows 語意：std::fs::rename 可覆蓋既存檔，但目的檔被其他 process 開啟時可能失敗（sharing violation）。rename 失敗時退回直接 std::fs::write 並清理暫存檔——行為不劣於原子化前，不把平台限制放大成動詞失敗；原子保證在 unix 全額成立、Windows 為 best-effort。
3. 旁路收編：crates/speclink-cli/src/verbs/config.rs 的設定檔編輯寫入、apps/desktop/core/src/settings.rs 的兩處 config.yaml 寫入，改走 util::write_file（speclink-desktop-core 已依賴 speclink-core，無新增依賴）。
4. crates/speclink-host/src/change_diff.rs 的私有暫存檔（git no-index diff 用、同進程自寫自讀）非共享真相，不在收編範圍。

## Non-Goals

- 不做跨 process 檔案鎖：遺失更新（寫寫競態）是另一個問題，討論裁定刻意延後，真的發生再啟動。
- 不做 fsync 持久化保證：斷電情境的資料完整性不在範圍，現行也無此保證。
- 不動 speclink-remote 的 credentials 寫入：已有 rotation lock 序列化寫入者，且檔案在 user-level 目錄，屬另一個 seam。
- 不動測試碼中的直接 fs::write：fixture 建置非共享真相。

## 相容性影響

- 人眼輸出與 `--json` 皆不變，無使用者遷移動作。
- unix 上「目的檔被設為唯讀」不再擋下寫入——rename 看的是所在目錄的權限而非檔案本身
  的權限，唯讀的 openspec/config.yaml 會被覆蓋（既有 mode 保留，見下）。原先的失敗是
  普通 fs::write 的附帶效果，並非任何規格所要求的保護。Windows 不受影響：rename 覆蓋
  唯讀檔會被拒，退回的直接寫入同樣被拒，仍為錯誤。
- symlink 目的檔：寫入前追蹤連結至最終目標，temp＋rename 落在目標上——連結拓撲與
  原本 fs::write 穿透寫入一致。hard link 是已知界限：rename 產生新 inode，其他連結名
  留在舊內容上（原地覆寫與原子替換本質互斥），接受此界限。
- 檔案權限：unix 上覆寫前以 `fs::metadata` 取目的檔既有 mode 套回暫存檔（best-effort），
  0600 的設定檔不會被放寬成 umask 預設；owner／ACL 不在保留範圍。
- 暫存檔建不出來（如父目錄不可寫但目的檔可寫）時退回直接寫入——「行為不劣於原子化前」
  對暫存檔建立與 rename 兩步全額成立。
- 回歸對照連帶：apps/desktop/core/src/settings.rs 兩個「寫檔階段失敗」測試原以唯讀目的檔
  製造失敗，改為 unix 鎖父目錄＋檔案（暫存與退回直寫兩路都擋）／Windows 鎖檔案，錯誤
  訊息與「原檔不變」四項斷言全數保留。

## Alternatives Considered

- 跨 process 檔案鎖：能一併解遺失更新，但要付鎖粒度選擇與平台差異的複雜度，且該風險未曾實際發生——延後而非排除。
- 樂觀 CAS（寫回前比對讀時 hash）：「讀時版本」要穿過所有寫入動詞簽名，改動面大，且本地檔案可被編輯器外改使比對基準易失效——排除。

## Impact

- Affected specs: command-runtime（新增本地寫檔原子落盤 requirement）
- Affected code:
  - Modified: crates/speclink-core/src/util.rs, crates/speclink-cli/src/verbs/config.rs, apps/desktop/core/src/settings.rs

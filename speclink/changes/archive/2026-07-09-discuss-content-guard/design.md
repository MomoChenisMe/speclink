## Context

discuss 的內容寫入動詞 context／add-round／conclude 皆以 stdin 取內容。CLI 現以「帶 --stdin 才讀、否則空字串」處理，core::discuss 的 add_round／conclude／set_context 又對內容零檢查，疊加成 silent failure（漏帶 --stdin → 空字串 → 靜默寫入空區段、仍回報成功）。三個前門——本地 CLI（commands）、遠端 CLI（remote_commands）、桌面 Tauri——皆呼叫同一組 core::discuss 函式。承 discuss discuss-record-integrity 結論方案 a。

## Goals / Non-Goals

**Goals:**

- 讓空內容寫入在寫入點 fail-loud，杜絕靜默損毀記錄。
- 一處防護覆蓋三前門。
- 漏帶 --stdin 但有管線內容時仍正確寫入。

**Non-Goals:**

- 不新增改寫既有 Round 的 fill／edit 動詞——Round 維持純 append-only（context／conclude 既有重跑覆寫行為不變）。
- 不做事後掃描既有空 round 的 validate lint（列為未來）。
- 不觸及 discuss 以外的內容寫入路徑（如 new artifact，另有內容驗證）。
- 不移除 --stdin 旗標（維持被接受以相容既有腳本與技能指示）。

## Decisions

### D1：空內容 guard 置於 core::discuss 而非各 CLI handler

於 core::discuss 的 add_round／conclude／set_context 開頭加 content.trim().is_empty() → bail，訊息含「內容為空」與「是否漏帶 --stdin」提示。置於 core 使本地 CLI、遠端 CLI、桌面 Tauri 一次受保護。

- 替代：於各 CLI handler 分別擋——否決：漏掉桌面 Tauri、且邏輯三重複。

### D2：CLI 以 IsTerminal 判管線讀取 stdin，--stdin 降為相容旗標

CLI 於標準輸入非互動終端（std::io::IsTerminal 判為管線／重導）時讀取其內容作為動詞內容，不論是否帶 --stdin；互動終端無管線時給空字串、交由 D1 的 core guard 報錯。--stdin 旗標維持被接受（不再是讀取的必要條件）。

- 替代：一律無條件讀 stdin——否決：互動終端無管線會阻塞等 EOF；IsTerminal 判別避免 hang 又消除漏旗標的靜默。
- 替代：移除 --stdin 旗標——否決：破壞既有腳本與技能指示的相容。

### D3：Round 維持純 append-only

不新增 fill／edit 動詞；既有 2 份損毀記錄已以 discard+recreate 修復。D1 治本後空 round 無法再生，補全需求消失，新動詞屬臆測性基建且侵蝕 append-only 帳本原則。

- 替代：加「僅補空 round」窄動詞——否決：D1 使空 round 不再產生，該動詞無使用情境（YAGNI）。

## Implementation Contract

- 行為：add-round／conclude／context 收到去除前後空白後為空的內容時以錯誤中止、不寫入任何區段；conclude 遇空內容 SHALL NOT 將 status 翻為 concluded。錯誤訊息指出內容為空並提醒可能漏帶 --stdin。以管線提供非空內容而未帶 --stdin 時，該內容被當作動詞內容寫入。
- 介面／資料形狀：core::discuss::add_round／conclude／set_context 於空內容回 Err（bail）。CLI 於 stdin 非互動終端時讀取內容；--stdin 為被接受但非必要的旗標。無新 --json 欄位、無新 IPC。
- 失敗模式：空內容 → Err 並附提示訊息；互動終端無管線 → 視為空內容並經 core guard 報錯（不阻塞）。
- 驗收：core 單元測試驗 add_round／conclude／set_context 對空及純空白內容回 Err、且不改動檔案；CLI 整合測試驗「管線空內容 → 錯誤退出」與「管線非空內容未帶 --stdin → 正確寫入」。驗證：`cargo test -p speclink-core`（Windows 如遇 cdylib 連結問題以 `--lib` 限縮）與 `cargo test -p speclink-cli` 相關整合測試。
- 範圍邊界：in scope＝core::discuss 三動詞的 guard、CLI（本地與遠端）的 stdin 讀取條件、對應測試。out of scope＝new artifact 等其他寫入路徑、fill／edit 動詞、事後 validate 掃描、--stdin 旗標移除。

## Risks / Trade-offs

- [IsTerminal 跨平台行為] → 緩解：std::io::IsTerminal 為標準庫穩定 API，Windows／macOS／Linux 一致；管線與重導皆判為非終端。
- [既有腳本行為改變] → 可接受：新增的是「非空才寫、空即報錯」與「管線亦讀」，對既有帶 --stdin 且提供非空內容的呼叫無影響。

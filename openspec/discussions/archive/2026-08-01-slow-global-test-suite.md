---
topic: 目前在跑一些全局測試時，一跑就要1小時起跳，這是為什麼？
slug: slow-global-test-suite
status: promoted
promoted_to: merge-integration-test-binaries
created: 2026-08-01
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 目前在跑一些全局測試時，一跑就要1小時起跳，這是為什麼？

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者反映 `npm run test:all` 全局測試一跑就一小時起跳，想知道原因。模式選擇：assumptions（codebase scout 找到大量相關檔案：package.json test:all 鏈、apps/desktop/src-tauri/tests/common/mod.rs 的 HARNESS_GATE、crates/speclink-server 50 支整合測試等）。使用者要求先實測一輪找出時間分佈，故第一、二回合以逐步計時實測（cargo test 拆 --no-run 與執行兩段）與程序級觀測（ps 取樣、syspolicyd 監看、A/B 重執行驗證）為證據。進行中的 changes（code-review-stage、add-improve-flow、verify-station-parity）與本題無關。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-01)

**Focus**: 一小時到底花在 test:all 鏈的哪一段（逐步計時實測）
**Position**: 時間幾乎全部消失在 `cargo test --workspace` 的「執行」段，且不在測試邏輯本身：
- 前 5 步（node scripts 1s、vitest ×3 共 40s、server-web build 4s）合計 45 秒
- `cargo test --no-run`（編譯＋連結 112 支整合測試 binary）熱 cache 下僅 1 秒
- 執行段跑了 110 分鐘以上仍未完（130+ 支 binary），但已完成 suite 的 libtest「finished in」總和只有約 4 分鐘
- 每支 binary 牆鐘約 60 秒，其中約 59 秒發生在進入 main 之前（ps 取樣：spawn 後 80 秒 CPU 0:00.00、RSS 32KB）
- 真正慢的測試只有 phase3_chain（136s，含 75 秒級實時恢復等待）與含 watch 模組的 unittests（49s）
**Ruled out**: 編譯連結成本（--no-run 1 秒，熱 cache 全命中）；前端測試與 build（45 秒）；「測試太多太慢」（內部總和約 4-5 分鐘）；build 目錄鎖等待（log 中 Blocking waiting 零筆）
**Open**: 每支 binary exec 前的 60 秒凍結是什麼機制造成

### Round 2 — assumptions (2026-08-01)

**Focus**: exec 前 60 秒凍結的機制為何（含 A/B 重執行驗證）
**Position**: 元凶是 macOS Gatekeeper（syspolicyd）對每支新連結 binary 首次執行做的惡意軟體評估，掃描稅約 60 秒／支 × 112 支 ≈ 1 小時：
- 凍結期間 syspolicyd 以 43.8% CPU 運轉，累計 CPU 時間 25.6 小時（歷次 test:all 都在付這筆稅）
- 測試 binary 每支 67-81MB（debug 檔），整檔雜湊掃描成本與大小成正比
- A/B 驗證：已掃過的 binary 重跑 0.010-0.015 秒（評估結果有快取）；但 syspolicyd 忙碌中重跑其他已掃過 binary 仍凍 23-41 秒（快取查詢的 XPC 呼叫排隊）；待其安靜後三支皆 0.01 秒
- 含義：每次改動核心 crate → 112 支 binary 換 hash 檔名重連結 → 全部視為「首次執行」→ 重付整筆掃描稅；deps/ 已累積 756 支歷代執行檔、target/ 滾至 67G
**Ruled out**: rust-analyzer 的 cargo check --all-targets 同場搶 target/（確有其事、會加劇排隊，但非主因）；測試內嵌 cargo build（admin_e2e／phase2_chain／backup_e2e／e2e_cli／desktop harness 共 5 處，本輪僅觸發 1 次小編譯 9 個 crate）；快取失效的系統性問題（二次重跑證明快取有效）
**Open**: 解法收斂——Developer Tools 豁免（系統設定將跑測試的終端機加入白名單）與合併 112 支整合測試 binary 兩者的取捨或並行；主跑完成後的完整分佈表尚待封頂

### Round 3 — assumptions (2026-08-01)

**Focus**: 完整分佈封頂與合併整合測試 binary 的設計取捨
**Position**: 最終數據坐實診斷，解法定為「Developer Tools 豁免（已開）＋合併整合測試 binary」雙管齊下：
- 6b 帳面 23300 秒中約 4.5 小時是筆電闔蓋睡眠（pmset log：10:02 Clamshell Sleep），真實 cargo 執行段約 117 分鐘；133 支 suite 測試內部總和僅 427 秒（7.1 分），掃描稅約 103 分鐘，與 112 支 × 60 秒模型吻合
- 四大真實慢測：phase3_chain 136s、admin_e2e 73s、backup_e2e 68s、watch unittests 49s，合計 5.4 分鐘
- 合併設計：各 crate 的 tests/*.rs 搬入 tests/it/ 子目錄成模組，tests/it/main.rs 統一宣告（Cargo 只認頂層 .rs 與 */main.rs 為 target）→ binary 數 113 → 10；檔數分佈 server 48、cli 27、remote 11、desktop 8、store-postgres 6、store-fs 5、core 5、單檔 crate ×3
- 風險掃描：測試檔中 env::set_var / set_current_dir 零筆，合併無程序狀態汙染；desktop 的 HARNESS_GATE 為 static Mutex，合併後語意不變；e2e 系列的 Once cargo build 合併後反而只建一次
- 已知代價：跨檔測試首次同 process 並行，loopback server 同時存活數上升（EINVAL 前科在 desktop，server 側需觀察，備援＝加同款 gate 或 --test-threads）；abort 級 crash 會帶走同 binary 其餘測試
- 使用者已於本機開啟 Developer Tools 豁免；合併的剩餘收益＝link 次數 113→10、target/（現 67G）體積、未豁免環境（CI／新機器／Windows Defender 同類稅）的保險
**Ruled out**: cargo-nextest 作為主解（per-test process 隔離更穩但不減 binary 數，掃描與 link 稅不變，可作補充）；只靠豁免不合併（其他機器與 CI 仍重付整筆稅）
**Open**: 合併範圍一次全做（10 crate 同型機械改動）或先做四大戶（server/cli/remote/desktop 拿走 94/113）；6b rc=1 為 speclink-remote doctest 編譯錯（引用尚不存在的 review DTO，屬 in-flight change 的平行工作污染，另行處理）

### Round 4 — assumptions (2026-08-01)

**Focus**: 豁免開啟後的實測驗證——結果不生效，並更正第三回合探針的誤判
**Position**: 豁免尚未生效，且找到「每次跑都一小時」的完整解釋（不只改 code 才重繳稅）：
- 豁免後重跑：6b 又回到每支約 1 分鐘節奏（23 支 suite 內部總和 2 秒、牆鐘 25 分），binary 凍在 exec 前、syspolicyd 67.3% CPU——掃描稅重現，已中止本輪省時
- 前景直接執行新連結的 phase3_chain 也要 44.8 秒 → 排除背景/前景脈絡差異，結論＝豁免沒吃到
- 責任程序鏈頂端是 Warp（/Applications/Warp.app）——豁免要勾的是 Warp；若勾的是其他終端 App 則無效，勾完建議完全退出重開 Warp
- 更正第三回合：探針 0.67 秒不是豁免生效的證據——重簽 cdhash 的 admin_api 副本內容早已被掃過，syspolicyd 的掃描快取以「內容」為鍵；用未掃過內容（新 phase3_chain 副本重簽）重驗＝42.7 秒，誤判成立
- 新成本駕駛：run 1 的 napi build（step 8）以不同旗標寫入共用 target/，弄髒 fingerprints → run 2 的 6a 重編 18 個 crate＋重連結（281 秒）→ 全部 binary 換新 cdhash → 即使零程式碼改動，每輪 test:all 都重付整筆掃描稅。這解釋了「每次都一小時起跳」而非只在改動後
**Ruled out**: 背景任務脈絡導致豁免失效（前景同慢）；codesign 重簽繞過掃描（同為 42.7 秒）
**Open**: 使用者確認 Developer Tools 清單為 Warp 並重啟後重測；napi build 與 cargo test 的 target/ 互踩（隔離 CARGO_TARGET_DIR 或調整步驟順序）納入合併 change 或另案

### Round 5 — assumptions (2026-08-01)

**Focus**: Warp 豁免修正後的端到端驗證
**Position**: 豁免生效，全局測試從一小時級壓到十分鐘級，驗證通過：
- 探針：兩支從未掃過的 44MB binary（sse_events、web_assets）首次執行 0.010-0.014 秒，syspolicyd 0% CPU（修正前同條件 44.8 秒）
- 全套 run 3 總時間 6 分 20 秒：前端步驟 16s、cargo 編譯連結 48s、cargo 執行 313s（90 支 suite）、node 側 3s
- cargo 執行段因既有測試失敗 fail-fast 截斷 43 支（含 desktop 的 phase3_chain 136s），補回估算完整乾淨跑約 9-10 分鐘
- napi build 本輪 1 秒（cache 命中）——101 秒與 target/ 污染只在其真正重建時發生
- e2e_cli 失敗歸因：斷言期望 stderr 含 "server unavailable"，CLI 實印 "server unreachable — check the connection url"；工作樹存在平行 session 的未提交 .rs 變更（speclink-cli、desktop core），run 1（今早、工作樹僅 .md 變更）同測試通過——屬平行 change 的半成品污染，與本題無關
**Ruled out**: 失敗為 flaky（斷言與實際輸出的字樣差是決定性的）；豁免仍未生效（探針與 syspolicyd 皆反證）
**Open**: 無——進入結論

## Conclusion

**Decision**: 全局測試一小時起跳的元凶是 macOS Gatekeeper（syspolicyd）對每支新連結測試 binary 首次執行的惡意軟體掃描（約 60 秒／支 × 112 支），且 test:all 內 napi build 與 cargo test 互踩共用 target/ 使每輪都重連結、重繳稅。解法雙管齊下：(1) 已完成——將 Warp 加入系統設定的開發者工具豁免（勾對 App 並重啟後實測生效，全套從一小時級降至十分鐘級）；(2) 待實作——合併整合測試 binary（各 crate tests/*.rs 收進 tests/it/ 模組、tests/it/main.rs 統一宣告，113 支 → 10 支），並隔離 napi build 的 CARGO_TARGET_DIR（或調整步驟順序）杜絕每輪快取互踩。
**Rationale**: 實測三輪定位——測試本身僅 7.1 分鐘，其餘全是掃描稅與其排隊效應；稅基＝binary 數 × 檔案大小 × 每輪重連結頻率。豁免消滅本機稅率，合併消滅稅基（並同時省 113→10 次 link 與 target/ 體積），對未豁免環境（CI、新機器、Windows Defender 同類機制）仍有效。
**Rejected alternatives**: cargo-nextest 作為主解（不減 binary 數，掃描與 link 稅不變；可另作補充）；只靠豁免不合併（其他機器與 CI 仍全額繳稅）；只合併四大戶（同型機械改動，分批徒增 churn）。
**Deferred**: server 測試合併後首次同 process 並行的 loopback EINVAL 風險（實跑觀察，備援＝common harness 加 desktop 同款 gate 或 --test-threads）；speclink-remote doctest 編譯錯誤與 e2e_cli "server unavailable/unreachable" 斷言字樣不一致（平行 in-flight change 的半成品，該 change 收尾時處理）；phase3_chain 136 秒的 75 秒級實時等待瘦身（獨立小題，收益有限）。
**Capture to**: proposal（合併整合測試 binary ＋ napi target 隔離，一個 change）
**Next**: /speclink-propose --from-discussion slow-global-test-suite

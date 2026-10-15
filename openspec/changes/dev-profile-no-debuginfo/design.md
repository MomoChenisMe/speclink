## Context

本機完整測試慢，不是測試本身慢，而是編譯器每次啟動都在讀一個塞爆的目錄。

- macOS 上 dev profile 預設 `split-debuginfo=unpacked`：除錯資訊不連進執行檔，而是留在每個 codegen unit 的 .o 中間檔，由除錯器事後去讀。cargo 與 rustc 都不會刪舊的 .o，所以每次重新編譯 workspace crate 或重新連結測試執行檔，都會多留一批。
- 主 checkout 的 target/debug/deps 已有 2,076,545 個 .o（target 共 173 GB），光列出這個目錄就要 2 分 45 秒。
- rustc 啟動時解析 `-L dependency=<deps 目錄>`，會把整個目錄讀過一遍。對一支正在編譯的 rustc 取樣，2346 次中有 2339 次停在讀目錄的系統呼叫；CPU 使用率只有 6–18%。每編一個 crate 就付一次這筆成本。
- 所以 speclink-desktop 的 lib 測試在主 checkout 要 1000–1800 秒，在全新 worktree 只要 5 秒；CI 每次從快取起建、不累積，Rust 測試步驟 4–5 分鐘。
- 前一次的變慢原因（Gatekeeper 對新執行檔的掃描）已由討論 slow-global-test-suite 處理，本變更處理的是第二個、本機獨有的原因。

本變更不動任何 crate 的程式碼，邊界只在 workspace 的建置設定（根目錄 Cargo.toml）與 scripts 測試面的 delivery gate。

## Goals / Non-Goals

**Goals:**

- dev 與 test 建置不再產生 .o 中間檔，target/debug/deps 不再無限累積。
- 主 checkout 清空一次後，本機 npm run test:all 回到約 10 分鐘。
- 設定被移除時，scripts 測試面會轉紅，不讓累積靜默復發。

**Non-Goals:**

- 不改 release profile，也不改任何 crate 的程式碼或測試內容。
- 不引入 cargo-nextest，不拆分或加速個別慢測試（例如 phase3_chain）——清空後若本機仍明顯超過 10 分鐘，再另開題目。
- 不處理「全量」用詞，那是同一討論的第二刀 taiwan-full-wording。
- 不清理 worktree 的 target 目錄：worktree 的 target 是新建的，量小，且設定生效後同樣不再累積。
- 不改 CI workflow。

## Decisions

### D1：用 `[profile.dev] debug = false`，不用 macOS 限定的 packed

在 workspace 根 Cargo.toml 新增 `[profile.dev]` 區段，只設 `debug = false`。test profile 繼承 dev，cargo test 一併生效。

- 實測（scratchpad 小專案，rustc 1.97）：預設設定編一次留 25 個 .o，改一行再編變 50 個；`debug = false` 為 0 個。
- 為什麼不用 macOS 限定的 `-C split-debuginfo=packed`：它也不留 .o，並保留完整除錯資訊（收進 .dSYM），但每支執行檔連結後要多跑一次 dsymutil，而且要用 cfg 限定在 macOS 的 rustflags 寫法。本專案目前沒有人用逐行除錯，付這筆成本沒有收益。日後真的需要逐行除錯，改走這個做法即可。
- 為什麼不用 `debug = "line-tables-only"`：實測仍留 25 個 .o，不解決累積。
- 為什麼不用 macOS 的 `split-debuginfo=off`：一樣失去除錯資訊，卻仍付產生除錯資訊的編譯成本。
- 附帶效果：所有平台的 dev 建置都少了除錯資訊，編譯與連結變輕，CI 也受益。

### D2：delivery-baseline 立需求，delivery gate 加一條靜態守門

這個設定的價值在「不累積」，被移除時不會馬上出事，要過幾個月才會再變慢。所以用 scripts 測試面守住它。

- 守門放進既有的 delivery gate 靜態契約測試（scripts/release/delivery-gate.test.mjs）：它本來就以解析設定檔內容的方式守 test:all、CI 與 release 的結構保證，不執行 build。新增一條測試即可，不另開檔案。
- 斷言內容：根 Cargo.toml 的 `[profile.dev]` 區段存在，且區段內 `debug` 的值為 `false`（同時接受 `0`，兩者在 Cargo 的語意相同）。
- 解析方式：以逐行讀取找出 `[profile.dev]` 標頭到下一個 `[` 標頭之間的區段，再比對 `debug = false` 或 `debug = 0`。不引入 TOML 解析套件——scripts 測試面目前零外部依賴，且守門只看一個鍵。

### D3：主 checkout 用 cargo clean 一次清空

設定只防止新的累積，不會刪掉已經存在的 208 萬個 .o，所以要清一次。

- 用 cargo clean，不手動刪 deps 底下的 .o：設定改變會讓所有 crate 的指紋失效、全部重建，留著舊產物沒有意義，整個清掉最單純，也一併回收 173 GB 硬碟。
- 清空前先確認沒有其他 session 的 cargo 或 rustc 行程在跑——清掉別人正在用的 target 會讓對方的建置中途失敗。
- 清空本身要刪兩百多萬個檔案，可能要好幾分鐘，在背景執行。
- 這是 repo 外的一次性動作，不進 commit。

## Implementation Contract

**Behavior**

- 在任一平台以 dev 或 test profile 建置任何 workspace 成員後，target/debug/deps 內沒有 .o 檔。
- 程式行為不變：dev 建置的 debug_assertions 與整數溢位檢查維持開啟（`debug` 鍵不控制它們）；release 建置的輸出不變。
- CLI 的人眼輸出與 `--json` 輸出不變；golden 快照與 CLI 測試的期望值不變。
- 測試失敗時，panic 訊息仍帶檔名與行號（例：`panicked at src/lib.rs:2:13`）。失去的只有 lldb 逐行除錯，以及 RUST_BACKTRACE 呼叫堆疊上的行號（只剩函式名稱）。

**Interface / data shape**

- 根 Cargo.toml 新增的區段只有兩行：`[profile.dev]` 標頭與 `debug = false`。不新增其他 profile 鍵，不新增 .cargo/config.toml。
- delivery gate 新增一條測試，名稱寫明它守的是「dev profile 關閉除錯資訊」。

**Failure modes**

- 有人移除 `[profile.dev]` 區段，或把 `debug` 改回 `true`、`1`、`2`、`"full"`、`"line-tables-only"` 等值 → delivery gate 那條測試失敗，訊息指出 Cargo.toml 的 dev profile 必須關閉除錯資訊與原因（避免 .o 累積）。

**Acceptance criteria**

- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 全綠，且新守門在把 `debug = false` 暫時拿掉時會失敗（RED 先行）。
- 主 checkout 清空後跑一次 npm run test:all 全綠，記錄總耗時與各段大致時間；目標約 10 分鐘。已知併行負載下的 packages/ui drawer 逾時 flaky 若出現，只單獨補跑該站，不重跑整鏈。
- 跑完後以不排序的方式列出 target/debug/deps 的檔案並計數，.o 為 0。

**Scope boundaries**

- In scope：根 Cargo.toml 的 dev profile、delivery-baseline 的一條新需求、delivery gate 的一條新測試、主 checkout target 的一次清空與量測。
- Out of scope：release profile、各 crate 程式碼、CI workflow、worktree 的 target、cargo-nextest、個別慢測試、用詞變更。

## Risks / Trade-offs

- [失去逐行除錯] → 本專案目前沒有使用；需要時改走 macOS 限定的 packed（D1），一處設定即可切換。
- [清空時有其他 session 正在建置] → 清空前檢查 cargo 與 rustc 行程；有就等它結束再清。
- [CI 快取失效] → 改 Cargo.toml 的 profile 會讓 rust-cache 這次之後的第一輪全部重建，三平台的該次 CI 會比平常慢；之後回到正常，而且因為少了除錯資訊，編譯與連結比以前輕。
- [跨平台] Windows（MSVC）原本把除錯資訊放進 .pdb、Linux 原本放進執行檔本身，兩者都沒有 .o 累積問題；`debug = false` 在三個平台都是合法值，只是一併省掉 .pdb 與除錯區段。
- [回歸對照] golden 快照與 CLI 測試比對的是輸出文字，不受除錯資訊影響；仍以三平台 CI 全綠確認。
- [清空後第一次建置要從頭編] → 量測時把這次冷建置的耗時一起記下，與之後的熱建置分開看。

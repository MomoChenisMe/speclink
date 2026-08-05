## Context

review 站驗證輪的凍結審查面由 `crates/speclink-host/src/change_diff.rs` 的 `resolve_validation_scope`（行 629 起）解析：驗證 patch＝上輪 findings 點名檔的 remediation_segment＋凍結後新髒檔；上輪已髒、不在 preserved scope 且又變動的檔觸發 needsInput（AmbiguityReason::PreviouslyDirtyChanged，無 candidateHash／hunk ids）。快照（`.speclink/review-scopes/<change>/snapshots/`，依 patchHash 命名）存留至 stamp／discard 才清；`dirty_files_at_capture` 僅存 PathHash。core 的 `Ticket` 解析（`crates/speclink-core/src/review.rs`）逐輪帶 `patch_hash`，但 host 的 `TicketBinding` 只收上輪。兩次實測（記憶 review-validation-scope-gap）證實：鄰居檔修復第一輪漏審、下一輪死結。本 change 源自討論 review-validation-scope-movement。

## Goals / Non-Goals

**Goals:**

- 驗證輪歸因基準改為「內容移動」：候選檔動了就進驗證面，不看 findings 點名。
- 缺的 before 內容以快照鏈回走重建；驗證輪 needsInput 分支消滅。
- patch 段帶出身標記（finding／adjacent／new）；skill 簡報對 adjacent 段要求確認歸屬。
- discovery 時被排除的檔變動＝註記＋放行。

**Non-Goals:** discovery 歸因與 needsInput 語意；快照儲存加寬；validation 新逃生口；工單格式與 add-round／stamp 契約；remote 協定與 server（host-local resolver，remote 自動同語意）。

## Decisions

**D1 — 歸因基準：內容移動，不是 findings 點名。** 驗證輪對上一輪快照 `dirty_files_at_capture` 逐條比對現況雜湊：沒動＝不進驗證面（今日行為）；動了依 D2 歸類。findings 點名清單降為出身標記的輸入之一，不再決定「誰能進驗證面」。
（替代案：維持點名基準＋人工補附——對引擎隱形，下一輪必卡，已兩度實測，否決。）

**D2 — before 重建：沿工單 Patch 雜湊鏈回走既有快照。** 動了的檔分三類：(a) 在上輪快照 preserved scope（patch deltas＋carried texts）內——走既有 `remediation_segment`，行為不變；(b) 不在上輪、但更早某輪快照收錄過——自工單各輪 `patch_hash`（新→舊）逐輪 `load_snapshot`，取最近收錄該檔的快照重建凍結後狀態（UTF-8 走存的 afterText；無 text 時 base commit 內容＋delta hunks 重放），diff(重建→現況) 為 adjacent 段；(c) 任何輪都沒收錄過——走 D3。介面連動：`TicketBinding` 增列各輪雜湊鏈欄位（新→舊，含上輪），CLI 組裝處（commands.rs 既有 Ticket→TicketBinding 映射）補傳；核心 Ticket 解析已逐輪帶 patch_hash，無解析面改動。
（替代案：每輪快照收錄全部髒檔內容——回走用既有資料即足，加寬儲存無必要，討論已否決。）

**D3 — never-captured 檔：註記＋放行，驗證輪 needsInput 消滅。** discovery 時被使用者排除的檔（從未進任何快照的 preserved scope）於驗證期間變動：凍結照常 resolved，該檔不進 patch，payload 增列範圍外變動清單、human 輸出一行 FYI。`AmbiguityReason::PreviouslyDirtyChanged` 與 validation 的 NeedsInput 回傳路徑刪除；snapshot 缺失、工單 patchHash 與 snapshot 不符維持既有硬錯誤（bail，非 needsInput）。discovery 的全部 needsInput 情境不動。
（替代案：對 never-captured 檔保留 needsInput——平行 session 改無關檔又會卡死複驗，正是要消滅的行為，討論已否決。）

**D4 — 出身標記進 files payload。** resolved payload 的 files 每項增列 attribution 欄位，值域 "finding"｜"adjacent"｜"new"：上輪 findings 點名者為 finding；D2(b) 回走重建者為 adjacent；凍結後新髒檔為 new。欄位為加法，既有欄位形狀不變；human 輸出的計數行併報三類數量。discovery phase 的 files 不帶此欄位（無上輪可歸因）——序列化以 Option 缺席呈現。

**D5 — skill 文本兩段改寫，走既有再生紀律。** `crates/speclink-core/assets/skills/review.md`：(1) step 3 的 needsInput 處置段改為 discovery 專屬（validation 不再產生 needsInput；快照缺失的硬錯誤處置文字保留）；(2) validation 簡報段增列「patch 內 attribution=adjacent 的段落須逐段確認確屬本次修復、非平行 session 漏入；不屬於者以回歸回報」。MARKER_VERSION 遞增、golden snapshot 與 assets.lock 依「內嵌資產版本鎖定紀律」再生（lock 最後、一次 bump 只再生一次）。

**D6 — 測試落點。** host 單元測試直測 `resolve_validation_scope`（fixture 造多輪快照鏈）；CLI 整合測試（review_verbs.rs）跑完整 verb 流程斷言 payload 欄位、範圍外註記行、needsInput 消滅與 discovery 回歸；golden 再生檔為 skill 文本改動的機械結果，人工過目 diff 限兩段文字與版本戳。

## Implementation Contract

**行為（使用者可觀察）：**

1. Round N（N≥2）凍結：上一輪快照的候選檔中，內容動過者全數進驗證 patch——findings 檔照舊、非 findings 檔以最近收錄輪重建 before 產生 adjacent 段；沒動的檔不進。連續多輪修復同一非 findings 檔可正常回走（N-2 以前的收錄輪）。
2. discovery 時被排除的檔於驗證期間變動：review scope 照常 exit 0 resolved，stdout（human）多一行範圍外變動 FYI 列出路徑，--json payload 增列該清單欄位；該檔不進 patch、不觸發 needsInput。
3. validation phase 不再有任何 needsInput 出口；快照缺失或工單 patchHash 不符仍非零硬錯誤、訊息不變。discovery phase 的行為（含全部 needsInput 情境與 hash-pinned selection）逐位元不變。
4. resolved --json 的 files 每項在 validation phase 帶 attribution（"finding"｜"adjacent"｜"new"）；discovery phase 缺席。既有欄位不變。
5. 再生後的 review SKILL.md：needsInput 段只描述 discovery 情境；validation 分流段含 adjacent 段確認歸屬指示。其他技能檔內容不變（僅版本戳）。

**介面／資料形狀：** `TicketBinding` 增列 `patch_hash_chain: Vec<String>`（新→舊；既有 `patch_hash` 欄位保留為鏈首的相容別名或直接改讀鏈首，實作擇一並全呼叫端同步）；resolved payload files 增列 `attribution: Option<String>`；payload 增列範圍外變動清單欄位（string[]，恆存在、無變動時為空陣列）。快照 version 1 格式不變。

**失敗模式：** 回走鏈中某輪快照檔案缺失→硬錯誤（與既有「snapshot 缺失」同語意，訊息點名缺失的 patchHash）；重建後雜湊與 dirty_files_at_capture 記錄不符（理論不可達，防禦性）→硬錯誤；binary 檔的 adjacent 段→僅 hashes（沿用既有 binary 處理）。

**驗收：** Success Criteria 1–5 各對應至少一個自動化測試；discovery 回歸以既有 review_verbs.rs 測試全綠釘住；`cargo test --workspace` 全綠。

**範圍邊界：** in scope＝上述行為與測試、skill 文本兩段、產物再生；out of scope＝Non-Goals 全部。

## Risks / Trade-offs

- **污染防護位移**：adjacent 段可能夾入平行 session 對候選檔的改動，從「引擎擋下」變「評審確認」——接受：原行為是死結而非防護，且 skill 簡報明文要求逐段確認歸屬（D5），可見性優於阻斷。
- **回走成本**：逐輪 load_snapshot＋重放，輪數實務 2–4，成本可忽略；防禦上限沿用工單輪數。
- **golden 再生**：review.md 兩段文字改動觸發全 marker 版本戳遞增與 snapshot 再生——一次性，diff 人工過目限縮於兩段與版本戳。
- **序列化相容**：payload 欄位皆加法；桌面與 remote 消費端讀不到新欄位也不受影響（serde 缺席容忍）。

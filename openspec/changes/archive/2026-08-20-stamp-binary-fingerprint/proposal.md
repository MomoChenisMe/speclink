## Problem

品質站的章對「存在但非 UTF-8」的 scope 檔一律拒絕落章（review-station 規格明文：存在但無法以 UTF-8 讀取的檔 SHALL 仍使蓋章失敗）。但凍結端（change diff 層）本來就把 binary 檔以「僅雜湊、無 hunk」的形式收進 frozen scope——兩端組合成死鎖：binary 檔一旦進了工單任一輪的 Scope，聯集就永遠含著它，章永遠落不了，唯一出路是把檔案從工作樹移走。2026-08-20 在 schema-engine-openspec-parity 的結章實際撞上（一張誤入凍結面的截圖），靠使用者刪檔才解。下次若 binary 是 change 的正當一部分（圖片資產、測試 fixture），刪檔這條路也沒有。

## Root Cause

凍結端與落章端對 binary 的態度不一致：凍結端已定案「binary／non-UTF-8 keep hashes only」（並在 FileDelta 記好 sha256），落章端的指紋函式卻只接受 UTF-8 文字（行尾正規化後 SHA-256），規格也把這個限制寫成了 SHALL。指紋錨的語意是「章綁在被審查的內容上」——binary 內容一樣可以綁，只是不能走文字正規化那條路。

## Proposed Solution

指紋規則分流：檔案內容可以 UTF-8 讀取 → 維持現行「CRLF→LF 正規化後 SHA-256」（既有已落的章逐位元組相容）；不可 UTF-8 讀取 → 改記「原始位元組的 SHA-256」（不做行尾正規化），蓋章照常進行。失效判定共用同一分流——binary 內容變動一樣觸發 stale。「檔案消失＝跳過」「聯集全消失＝拒章」「I/O 讀取失敗＝拒章」等既有語意全部不變。實作面：共用實作（core 的 station 模組）新增 bytes 入口，讀檔閉包從回傳文字改回傳位元組，四個呼叫端（core 的 review／verify、封存守門、CLI 的 remote 章路徑、desktop core 的 freshness 重算）隨簽名連動；兩站規格的指紋錨 requirement 走 MODIFIED 更新至最終態。

## Non-Goals

- 凍結端行為不變：binary 進 frozen scope 的「僅雜湊、無 hunk」既定設計維持
- 不做「凍結時排除 binary」（binary 變更會完全逃過品質站）也不做「stamp 跳過 binary」（章蓋完後 binary 可無聲變動）——兩案皆棄
- 審查面不變：binary 檔對兩站的審查子代理仍是「有這個檔、內容不可讀」的存在，不新增 binary diff 呈現
- remote stamp 協定形狀不變：client 端本來就負責算指紋再提交，分流發生在 client 的計算內

## Success Criteria

- 工單 Scope 含非 UTF-8 檔時，review stamp 與 verify stamp 都能落章，reviewed_scope／verified_scope 記該檔的位元組 SHA-256
- 落章後該 binary 檔內容變動 → 失效判定 stale；未變動 → fresh
- 純文字 scope 的既有章：指紋值與改動前逐位元組相同（回歸零影響）
- 檔案消失跳過、聯集全消失拒章、I/O 錯誤拒章三條既有語意的測試全數維持綠

## Impact

- Affected specs: review-station（MODIFIED：內容指紋錨與失效判定）、verify-station（MODIFIED：驗證指紋錨與失效判定）
- Affected code:
  - Modified: crates/speclink-core/src/station.rs（指紋分流與閉包簽名）、crates/speclink-core/src/review.rs、crates/speclink-core/src/verify.rs（轉出面與讀檔閉包）、crates/speclink-core/src/archive.rs（封存守門的讀檔閉包）、crates/speclink-core/src/util.rs（新增 read_bytes_opt helper）、crates/speclink-cli/src/verbs/station.rs（remote 章路徑的讀檔閉包）、apps/desktop/core/src/query.rs（freshness 重算的讀檔閉包）
  - New: (none)
  - Removed: (none)

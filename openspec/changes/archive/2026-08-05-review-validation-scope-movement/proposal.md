## Problem

review 站的驗證輪（validation phase）有一個已兩度實測撞上的盲區（2026-08-04 golden 再生檔、2026-08-05 worktree-toggle-and-guards 手動附檔）：修復動到 findings 沒點名的鄰居檔（呼叫端、測試、再生的 golden）時——

1. **第一輪漏審**：驗證 patch 只由「上輪 findings 點名的檔」＋「凍結後才變髒的新檔」組成，鄰居檔的修復內容不進驗證面，評審審不到（本次 remediation 有三分之一在驗證面之外，靠人工重建 diff 附進簡報補救）。
2. **下一輪死結**：這些檔在快照裡只記了雜湊（`dirty_files_at_capture` 為 PathHash，無內容），下一輪凍結時被判成 "previously dirty files changed outside the preserved scope" 而 needsInput，且此形態 payload 無 candidateHash、無 hunk ids，hash-pinned 逃生口不可用——verb 只剩 stop 或 discard，正式驗證輪開不出來，只能以非正式複驗＋stamp --accept 收尾。

## Root Cause

歸因基準用錯了軸：resolver 以「findings 有沒有點名」決定驗證面，但真實的修復幾乎必然動到點名清單以外的鄰居。而快照對未點名髒檔只存雜湊不存內容（`crates/speclink-host/src/change_diff.rs:657` 註解自陳 "their before was never saved — another change to them cannot be attributed"），偵測到移動也產不出精確 diff。關鍵事實：重建「改之前內容」所需的資料其實一直在磁碟上——各輪快照存留至蓋章才清，discovery 快照帶全候選檔對 base commit 的完整 delta——只是 resolver 只載入上一輪快照、不回頭拿。

## Proposed Solution

驗證輪的範圍歸因從「findings 點名」改為「內容真的動了沒」（承討論 review-validation-scope-movement 結論）：

- **偵測**：對上一輪快照 `dirty_files_at_capture` 的每個條目比對現況雜湊，動了才需要歸因。
- **preserved 內（findings 檔）**：走既有 remediation_segment，行為不變。
- **preserved 外、但更早某輪快照收錄過**：沿工單各輪 Patch 雜湊鏈回走，取最近一份收錄該檔的快照重建凍結後狀態（存的 text，或 base commit＋delta 重放），diff(重建→現況) 以「adjacent 段」進驗證 patch——即本次人工補救的機械化。
- **任何輪都沒收錄過（discovery 時被使用者排除的檔）**：註記＋放行——排除本身是 discovery 的使用者處置，複驗不替已裁定的事再擋路。驗證輪的 needsInput 分支就此消滅（snapshot 缺失仍為硬錯誤）。
- **出身標記**：驗證 patch 的 files 帶 finding／adjacent／new 歸屬欄位，review skill 簡報對 adjacent 段要求評審確認「確屬本次修復、非平行 session 漏入」——污染防護從「引擎拒絕凍結」移到「評審可見」。

介面連動：TicketBinding 自「僅上輪 patchHash」擴為攜帶各輪雜湊鏈（core 的 Ticket 解析本就逐輪帶 patch_hash，CLI 組裝處補傳）；review scope 的 --json payload 增列歸屬欄位與「範圍外變動」註記清單；review skill 文本同步改寫 needsInput 處置段與 validation 簡報段。

## Non-Goals

- 不動 discovery phase 的歸因與 needsInput 語意——fail-closed 與 hash-pinned selection 逃生口原樣保留（那裡有使用者處置工具）。
- 不加寬快照儲存（每輪收錄全部髒檔內容）——回走用既有快照即足，經討論否決。
- 不做 needsInput 的新逃生口（如 validation 的 hunk selection）——分支整個消滅，不需要逃生口。
- 不動 review 工單格式與 add-round／stamp 契約——只動 scope 解析。
- 不動 remote 協定與 server——scope 解析是 host-local，remote 走同一 resolver 自動獲得同語意。

## Success Criteria

1. 修復動到 findings 未點名的候選檔時，該檔自上次凍結以來的差異以 adjacent 段出現在驗證 patch 中（可由整合測試斷言 patch 內容與歸屬欄位）。
2. 連續多輪修復同一未點名檔不再觸發 needsInput：第 N 輪凍結沿雜湊鏈回走到最近收錄輪，正常 resolved。
3. discovery 時被排除的檔在驗證期間變動：凍結照常 resolved，payload 與 human 輸出帶一行範圍外變動註記，該檔不進 patch。
4. 快照缺失、工單 patchHash 與快照不符的既有硬錯誤行為不變；discovery 的全部 needsInput 情境不變（回歸測試釘住）。
5. 再生後的 review SKILL.md：needsInput 處置段不再含 validation 情境；validation 簡報段含 adjacent 段確認歸屬的指示。

## Impact

- Affected specs: `change-diff-scope`（修改：frozen snapshot 綁定、歧義 fail-closed、scope human/JSON 契約三條需求）、`review-skill`（修改：審查流程的技能行為）
- Affected code:
  - Modified（引擎）: `crates/speclink-host/src/change_diff.rs`（resolve_validation_scope 歸因重寫、TicketBinding 雜湊鏈、adjacent 段重建、範圍外註記）、`crates/speclink-cli/src/commands.rs`（TicketBinding 組裝、payload 與 human 輸出）、`crates/speclink-cli/src/remote_commands.rs`（remote 側 TicketBinding 組裝——欄位替換是破壞式的，兩個呼叫端必須同步）
  - Modified（技能資產）: `crates/speclink-core/assets/skills/review.md`（needsInput 處置段、validation 簡報段）、`crates/speclink-core/src/init.rs`（MARKER_VERSION 遞增）
  - Modified（測試與對照）: `crates/speclink-cli/tests/it/review_verbs.rs`、`crates/speclink-core/tests/golden`（snapshot 與 assets.lock 再生）、`crates/speclink-core/tests/it/render_golden.rs`（如需）
  - New: (none)
  - Removed: (none)
- 相容性影響：review scope --json 的 files 增列歸屬欄位、payload 增列範圍外變動清單（皆為加法，既有欄位形狀不變）；validation 的 needsInput 不再發生（原本撞到的使用者只會感到死結消失）；技能文本再生需要既有 update 流程（版本戳遞增）。

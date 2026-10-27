---
topic: discuss 記錄的完整性防護與空 round 修補機制
slug: discuss-record-integrity
status: promoted
promoted_to: discuss-content-guard
created: 2026-07-09
---

# Discussion: discuss 記錄的完整性防護與空 round 修補機制

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

本次 discuss 工作階段中，agent 在 `add-round`／`conclude` 漏帶 `--stdin`，CLI 把「沒帶旗標」翻成空字串靜默寫入，導致 spec-trace-display、promoted-discussion-board-ux 兩份記錄的 Round／Conclusion 全空（Context 因有帶 --stdin 而正常）。使用者提出兩問：(1) 是否需要檢查機制防這問題；(2) 目前 round 只能附加，是否需要讓 LLM 能「補全」。

模式：assumptions（掃到 discuss.rs、commands.rs、remote_commands.rs、main.rs）。

根因（程式碼坐實）：
- `core::discuss::add_round`(discuss.rs:254)、`conclude`(:687) 對 content 零檢查，直接 `content.trim_end()` 寫入；`set_context`(:240) 只查 section 存在、不查空。
- 本地 CLI `commands.rs:1702`、remote CLI `remote_commands.rs:690` 皆 `let content = if stdin { read_stdin() } else { String::new() };`——缺 --stdin 即空字串。
- 這是 CLAUDE.md 明列要避免的 silent failure。

關鍵不對稱：`conclude`(:692)、`set_context`(:242) 用 `replace_section` 覆寫（重跑即改寫，conclude 註解明說「replaces the previous conclusion」）；唯獨 `add_round`(:265, round_no=count+1) 純 append，無 replace 路徑。→「補全」對 Context/Conclusion 已具備，缺口只有 Round。

介面深度檢查：seam＝core::discuss（三前門—本地CLI/remoteCLI/桌面Tauri—皆薄包 core，guard 擋在 core 一次覆蓋）；深度＝guard 是真行為（拒空）非轉發；刪除測試＝拿掉即回退為靜默 bug。無新 IPC/儲存。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-09)

**Focus**: 針對「空 round 靜默寫入」該用什麼防護（Q1）與修補（Q2）機制
**Position**: 治本於寫入點、保留 append-only：
- 根因＝silent failure：add_round/conclude 對 content 零檢查（discuss.rs:254/687），CLI 把缺 --stdin 翻成空字串（commands.rs:1702、remote_commands.rs:690）。
- Q1 正解＝寫入時 fail-loud 的 emptiness guard，放 **core** 一處，一次覆蓋本地CLI／remoteCLI／桌面Tauri 三前門；順帶拿掉 CLI 的 `else { String::new() }` 讓忘記旗標也不靜默。
- Q2：「補全」對 Context／Conclusion 已存在（replace_section 覆寫），缺口精確只有 Round（純 append，無 replace）。
- Round 的 append-only 是刻意原則（技能 Document rules），不為補全一般性破例。
- Q1 治本後未來不再產生空 round，補全只剩既存 2 份一次性清理（discard+recreate），為此建 fill-round 動詞屬臆測性基建（YAGNI）。
**Ruled out**: 事後 lint 當主防護（弱於寫入時擋，不阻壞寫入）；在每個 CLI handler 各擋（漏桌面 Tauri、三份重複）；一般性 round 編輯動詞（侵蝕 append-only 帳本）
**Open**: Round 立場 a（純 append-only＋discard 修既存）vs b（窄範圍 fill-round 受控例外）

## Conclusion

**Decision**: (Q1 檢查) 於 `core::discuss` 的 `add_round`／`conclude`／`set_context` 加 emptiness guard——`content.trim().is_empty()` 即 `bail!`，訊息提示「內容為空，是否忘了 --stdin」；擋在 core 一處覆蓋本地CLI／remoteCLI／桌面Tauri 三前門。並移除 CLI 的 `else { String::new() }`、改為一律讀 stdin，使忘帶旗標不再靜默（確切 stdin 機制留 propose）。(Q2 補全) 採方案 a：Round 維持**純 append-only**，不新增編輯／fill 動詞；Context／Conclusion 既有 replace_section 覆寫能力保留。既存 2 份空檔（spec-trace-display、promoted-discussion-board-ux）以 discard+recreate 一次性修復。
**Rationale**: silent failure 是根本反模式，修復應在寫入點 fail-loud、且置於 core 讓三前門一次受保護。Q1 治本後空 round 無法再生，補全需求退化為一次性歷史清理，故 fill-round 動詞既臆測（YAGNI）又侵蝕 append-only 帳本原則——不值得。
**Rejected alternatives**: 事後 lint 當主機制（不阻壞寫入，弱於寫入時擋）；在各 CLI handler 分別擋（漏桌面 Tauri、邏輯三重複）；新增一般性 round 編輯或 fill-round 動詞（違反刻意的 append-only 原則）
**Deferred**: 可選的事後 validate 掃描空 round（僅在手改／git merge 等非工具路徑被證實會再生空檔時才建）；stdin 讀取確切機制（一律讀 vs TTY 偵測）留 propose/design
**Capture to**: proposal（新變更：discuss 空內容 guard ＋ 移除 --stdin 靜默陷阱）
**Next**: /speclink-propose --from-discussion discuss-record-integrity

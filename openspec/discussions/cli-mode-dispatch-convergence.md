---
topic: CLI 本機/remote 分岔決策收斂到 dispatch——逼每個動詞表態
slug: cli-mode-dispatch-convergence
status: promoted
promoted_to: cli-mode-dispatch-convergence
created: 2026-08-08
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: CLI 本機/remote 分岔決策收斂到 dispatch——逼每個動詞表態

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

前因：improve-cli-command-layer（2026-08-07，已轉出歸檔）盤出四個候選；候選 1（cli-render-unification）已落地封存，使用者指示接續討論候選 2「本機/remote 分岔決策收到 dispatch 一處，逼每個動詞表態」。

模式：assumptions——commands.rs／remote_commands.rs／main.rs 三檔俱在，前討論已載明摩擦證據（remote-verb-parity 盤點的 A 類事故：cmd_show 讀本機空 store 回錯資料、cmd_in_progress 靜默寫本機）。

偵察（候選 1 之後的現況）：remote_ctx() 散佈分岔仍 22 處（commands.rs 全檔）；remote_ctx() 含 workspace 探索→模式解析→連線握手且 fail-closed（remote_commands.rs:25–50）；野外已有三種表態樣板——claim 的 fs 明寫拒絕（commands.rs:53）、status --schema 的 remote 拒絕（remote_commands.rs:161）、bulk archive 的 remote 拒絕（remote_commands.rs:963）。dispatch 為窮盡 match（commands.rs:6–42），共 31 個頂層動詞。

相關 change：cli-render-unification（已封存，候選 1）。進行中 change 零、無撞車。候選 3（重切檔案）、候選 4（wire→core 轉接收斂）仍留在原討論待後續。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-08-08)

**Focus**: 候選 2 的落地形狀——分岔決策如何收斂、判定時機、粒度與動詞分類
**Position**: 使用者確認全部四項假設，形狀定為表驅動的模式宣告：
- 形狀：dispatch 窮盡 match 的右手邊從裸函式呼叫改為模式形狀 enum（ModeFree／Dual{fs臂,remote臂}／RemoteOnly{fs 拒絕}）——Dual 少一臂即編譯不過，「靜默走本機」的預設從結構上消滅
- 判定時機：決策點集中、執行點惰性——ModeFree 動詞永不觸發 remote_ctx()；因其含握手且 fail-closed（remote_commands.rs:25–50），無條件先判會讓 completion/schemas 在壞 .speclink.yaml 目錄下從能跑變炸
- 粒度：表列頂層動詞；discuss／station 家族以家族臂滿足——remote_discuss 對 11 個子指令窮盡 match 無 catch-all（remote_commands.rs:1280–1350），remote_station 同，葉子級保證已由 clap enum 的窮盡性承擔
- 分類（31 動詞全盤）：ModeFree 12（init、update、link、unlink、auth、schemas、templates、feedback、schema、config、completion、demo）、Dual 18（list、show、validate、analyze、drift、archive、discard、artifact、language、status、instructions、new、workflow-config、task、in-progress、discuss、review、verify）、RemoteOnly 1（claim）；link/unlink/auth 不消費模式而是改模式，故屬 ModeFree
- 臂內明文差異不動：status --schema、bulk archive 的 remote 拒絕與 C 類明文分歧（Path 行、worktree 欄）留在臂內，本刀只動決策落點
- 介面深度四項過站：接縫在 dispatch 旁的宣告層（不新開 crate，檔案重切歸候選 3）；單層轉接，remote 臂直呼既有 remote_* 函式；藏住惰性判定與雙臂窮盡性，非轉發殼；刪除測試——刪掉即回到 22 處散佈分岔與靜默本機預設
**Ruled out**: trait 雙臂（30+ 動詞、clap 參數型別各異，樣板成本最高）；lint 守門（無編譯期保證，事故仍靠人抓）；dispatch 無條件先判再派（靜態動詞行為回歸＋多付握手）；葉子粒度上表（行數 ×3，與 clap subcommand 結構重複表達）；FsOnly variant（現無純 fs-only 動詞，YAGNI——出現時再加，編譯器逼窮盡）
**Open**: none——形狀、時機、粒度、分類俱裁定

## Conclusion

**Decision**: CLI 的本機/remote 分岔決策收斂到 dispatch 的表驅動模式宣告——31 個頂層動詞逐一表態其模式形狀（ModeFree／Dual{fs臂,remote臂}／RemoteOnly{fs 拒絕}），Dual 少一臂即編譯不過；模式判定惰性執行，ModeFree 動詞永不觸發 remote_ctx()；discuss／station 家族以家族臂滿足，葉子窮盡性由 clap enum match 承擔。
**Rationale**: 22 處散佈的 `if let Some(ctx) = remote_ctx()` 讓新動詞預設「靜默走本機」，remote-verb-parity 盤點的 A 類事故（cmd_show 讀空 store 回錯資料、cmd_in_progress 靜默寫本機丟開工歸屬）正是此結構的產物；收成宣告後事故從「靠人工盤點抓」變成「編譯器擋」。關鍵取捨是決策集中 vs 執行提前——remote_ctx() 含握手且 fail-closed，執行提前會讓靜態動詞在壞 yaml 下行為回歸，故只集中決策、不提前執行。
**Rejected alternatives**: trait 雙臂（30+ 動詞、clap 參數型別各異，樣板成本最高）；lint 守門（無編譯期保證）；dispatch 無條件先判再派（ModeFree 動詞行為回歸＋多付握手）；葉子粒度上表（行數 ×3，與 clap 重複表達）；FsOnly variant（現無此類動詞，YAGNI）。
**Deferred**: none
**Capture to**: proposal
**Next**: /speclink-propose --from-discussion cli-mode-dispatch-convergence

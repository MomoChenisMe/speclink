## Why

CLI 的本機（fs）/remote 模式分岔決策目前散在 22 個函式開頭的 remote_ctx() 檢查，新動詞的預設行為是「靜默走本機 store」——忘寫這行檢查不會報錯，只會默默讀寫錯的資料。remote-verb-parity（2026-07-30）盤點的 A 類事故（show 於 remote 模式讀本機空 store 回錯資料、in-progress 靜默寫本機丟失開工歸屬）正是這個結構的產物。本變更把分岔決策收斂到 dispatch 的表驅動模式宣告：每個頂層動詞必須表態其模式形狀，雙模式動詞少一臂即編譯不過，事故從「靠人工盤點抓」變成「編譯器擋」。

目標使用者是透過 AI 代理跑 SDD 的開發者：CLI 是 agent 的操作面，動詞在兩模式下作用於正確的 store 是所有技能判讀的前提。對應 workflow 全階段的動詞執行面。

來源討論：cli-mode-dispatch-convergence（improve-cli-command-layer 候選 2；候選 1 已由 cli-render-unification 落地）。

## What Changes

- **模式形狀宣告（結構重構核心）**：dispatch 窮盡 match 的右手邊從裸函式呼叫改為模式形狀宣告，四種形狀——ModeFree（與 store 模式無關，dispatch 不做模式判定）、Dual（本機臂與 remote 臂皆必填，缺一臂編譯不過）、FsOnly（remote 模式明寫拒絕——只解析模式、不握手，離線同樣拒絕且 server 零請求）、RemoteOnly（fs 模式明寫拒絕）。影響 crate：speclink-cli。
- **散佈分岔移除**：22 處各動詞函式開頭的 remote_ctx() 檢查，加上 demo 函式內的 remote 拒絕檢查，全數移除；模式判定改由 dispatch 依宣告執行——Dual／FsOnly／RemoteOnly 動詞判定一次後派給對應臂或拒絕，ModeFree 動詞永不觸發判定。
- **動詞分類全盤表態（31 個頂層動詞）**：ModeFree 11（init、update、link、unlink、auth、schemas、templates、feedback、schema、config、completion——其中 link／unlink／auth 是連線管理，不消費模式而是改模式，連線解析自理）；Dual 18（list、show、validate、analyze、drift、archive、discard、artifact、language、status、instructions、new、workflow-config、task、in-progress、discuss、review、verify）；FsOnly 1（demo——正典 verb-contract 已以 SHALL 要求其 remote 拒絕，現行實作寫在函式內，收進宣告層）；RemoteOnly 1（claim）。
- **家族臂維持**：discuss 與 review／verify 兩站的 Dual 兩臂即既有家族函式（本機家族函式 vs remote 家族函式），子指令層的窮盡性由 clap enum 的窮盡 match 承擔（remote 家族函式無 catch-all，新增子指令兩臂皆編譯不過）。
- **新增凍結對照**：宣告層三類邊界行為首度入測試與正典——ModeFree 動詞於壞 .speclink.yaml 目錄下仍正常執行、FsOnly（demo）的 remote 拒絕（離線同拒、server 零請求、文案不變）、RemoteOnly（claim）的 fs 拒絕（文案不變）；既有 Dual 動詞的 fail-closed 對照不動。
- 不新增任何子指令或旗標；兩模式的人眼輸出、--json 形狀、exit code、拒絕文案全數不變（純結構重構，行為凍結）。

**相容性影響**：

- 本機與 remote 模式：輸出零變更，既有凍結對照（remote_verb_parity、remote_read_path、remote_write_path 等）不動且必須維持全綠。
- 模式判定時機：22 處 remote_ctx() 檢查與 demo 的拒絕檢查原本就位於各函式／子指令臂的第一步，上移到 dispatch 後可觀察行為不變（clap 參數解析本就先於 dispatch 完成）。
- wire 契約、server、desktop 皆不動；既有使用者無需遷移動作。

## Non-Goals

- 不動檔案切分與 include! 文字包含（improve-cli-command-layer 候選 3，deferred）。
- wire→core 轉接不搬進 speclink-remote（候選 4，deferred）。
- 臂內的明文行為決策不動：status --schema 的 remote 拒絕、bulk archive 的 remote 拒絕、C 類明文分歧（new change 的 Path 行、list 的 worktree 欄）維持在臂內現狀。
- 不採 trait 雙臂（30+ 動詞、clap 參數型別各異，樣板成本最高）、不採 lint 守門（無編譯期保證）、不採 dispatch 無條件先判模式（ModeFree 動詞於壞 yaml 下會行為回歸並多付握手）、不逼葉子粒度上表（與 clap subcommand 結構重複表達）。

註：來源討論曾以「現無此類動詞」為由排除 FsOnly 形狀，該前提經實作盤點推翻——demo 即現行 FsOnly 動詞（正典 verb-contract 的 SHALL 要求＋函式內既有拒絕實作），依討論結論自載的規則「出現時再加，編譯器逼窮盡」納入。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `verb-contract`: 模式分岔的單點宣告入契約——動詞三分類清單、ModeFree 不觸發模式解析（壞 .speclink.yaml 下仍可執行）、RemoteOnly 的 fs 明寫拒絕

## Impact

- Affected specs: verb-contract
- Affected code:
  - New:
    - crates/speclink-cli/tests/it/mode_dispatch.rs（宣告層三類邊界行為的凍結對照：ModeFree 壞 yaml 可執行、demo remote 拒絕、claim fs 拒絕）
  - Modified:
    - crates/speclink-cli/src/commands.rs（dispatch 改表驅動模式宣告、移除各函式的 remote_ctx 與 demo 的模式檢查分岔）
    - crates/speclink-cli/src/remote_commands.rs（remote_ctx 的呼叫端收斂到 dispatch，remote 臂函式簽名隨宣告微調）
    - crates/speclink-cli/tests/it/main.rs（登錄新測試模組）
  - Removed: (none)

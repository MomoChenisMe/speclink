## Context

CLI 的 31 個頂層動詞由 dispatch 的窮盡 match 派發，但「本機（fs）/remote 模式該走哪邊」的決策散在 22 個函式或子指令臂開頭的 remote_ctx() 檢查，外加 demo 函式內一段自寫的 remote 拒絕。remote_ctx() 的內涵是三段：workspace 探索 → resolve_store_mode（便宜、fail-closed——壞 .speclink.yaml 直接報錯）→ 連線握手（僅於 remote 模式，含認證與版本檢查）。忘寫檢查的動詞預設「靜默走本機 store」，remote-verb-parity（2026-07-30）盤點的 A 類事故即其產物。

前置變更 cli-render-unification（已封存）已把渲染面收斂為每動詞單一函式；本變更收斂分岔決策面，兩者合計完成 improve-cli-command-layer 的候選 1 與候選 2。相關正典：verb-contract（remote 動詞行為契約，含 demo 的 remote 拒絕 SHALL）、command-runtime（引擎命令層的動詞覆蓋表，其排除清單與本設計的 ModeFree 分類同向）。

## Goals / Non-Goals

**Goals**

- 每個頂層動詞在 dispatch 一處表態其模式形狀；Dual 動詞缺任一臂即編譯不過，「靜默走本機」的預設從結構上消滅。
- 模式判定惰性執行：ModeFree 動詞零判定；FsOnly 只解析模式、不握手；Dual／RemoteOnly 於需要時才判定與連線。
- 兩模式全部可觀察行為凍結：輸出、--json 形狀、exit code、拒絕文案逐字不變。

**Non-Goals**

- 不動檔案切分與 include! 結構（候選 3）；wire→core 轉接不搬家（候選 4）。
- 臂內的行為決策不動：status --schema 與 bulk archive 的 remote 拒絕、C 類明文分歧（Path 行、worktree 欄）留在臂內。
- 不改 clap 參數結構、不增減任何子指令與旗標。

## Decisions

**D1：形狀宣告以「每形狀一支泛型組合子」落地，不用 trait、不用 boxed closure enum**

dispatch 的每個 match 臂改為呼叫形狀組合子，臂函式以函式值傳入：

```
Commands::List(a)  => dual(a, 本機臂, remote臂)     // 兩臂皆為必填參數
Commands::Claim(a) => remote_only(a, remote臂)      // fs 拒絕文案住在組合子內或以參數傳入
Commands::Demo     => fs_only(本機臂)               // remote 拒絕，判定不握手
Commands::Init(a)  => cmd_init(a)                   // ModeFree：直呼，dispatch 不介入
```

組合子是泛型函式（參數型別隨動詞的 clap Args 泛化），「少一臂」即「少一個必填參數」——編譯錯誤。這是討論裁定「表驅動宣告」的 Rust 慣用實現：與 enum 宣告等價，但零 boxed closure、零生命週期噪音。

- 否決 trait 雙臂：30+ 動詞每個開 struct + impl，樣板成本最高（討論已裁定）。
- 否決 lint／測試守門：無編譯期保證（討論已裁定）。
- 否決 boxed closure 的 enum 表：與組合子表達力相同，但多付 Box 與借用整理，無額外收益。

**D2：判定分兩層——「模式解析」與「連線」分離，各形狀觸發矩陣如下**

remote_ctx() 現況把解析與握手綁在一起；本設計把觸發權交給形狀：

| 形狀 | 模式解析（便宜、fail-closed） | 連線（握手） |
|------|------------------------------|--------------|
| ModeFree | 不觸發 | 不觸發 |
| FsOnly | 觸發——Remote 即拒絕 | 不觸發（離線同拒、server 零請求，demo 現行行為） |
| Dual | 觸發 | 僅 Remote 模式 |
| RemoteOnly | 觸發——Fs 即拒絕 | 僅 Remote 模式 |

否決「dispatch 無條件先判再派」：ModeFree 動詞（completion、config 等）在壞 .speclink.yaml 目錄下會從能跑變炸——行為回歸，且多付握手（討論已裁定）。

**D3：宣告粒度為頂層動詞；多子指令動詞以家族臂滿足**

表列 31 個頂層動詞。含子指令的 Dual 動詞（task、new、artifact、language、in-progress、discuss、review、verify、workflow-config）的兩臂為家族函式——本機家族函式與 remote 家族函式各自對子指令 enum 窮盡 match、無 catch-all，新增子指令時兩臂皆編譯不過，葉子級窮盡性由 clap enum 承擔。review／verify 的 clap → StationVerb 正規化函式共用，兩臂各自呼叫，不複製正規化邏輯。

否決葉子粒度上表：表行數約 ×3，與 clap subcommand 結構重複表達同一件事（討論已裁定）。

**D4：FsOnly 形狀納入——修正來源討論的一項前提**

討論曾以「現無純 fs-only 動詞」為由排除 FsOnly（YAGNI，出現時再加）。實作盤點推翻該前提：demo 的函式內既有 remote 拒絕（比照 claim 的 fail-loud，只判模式不握手），且 verb-contract 正典以 SHALL 要求此行為。依討論結論自載的規則納入 FsOnly，demo 的函式內檢查收進宣告層，拒絕文案逐字保留。

**D5：動詞分類全表（31 個，實作時照表宣告）**

- **ModeFree 11**：init、update、link、unlink、auth、schemas、templates、feedback、schema、config、completion。link／unlink／auth 是連線管理——不消費模式而是改模式，其內部的連線解析自理，dispatch 不介入。注意：ModeFree 指「dispatch 不做 store 模式判定」，不等於「不讀 .speclink.yaml」——schemas／templates／update 等的 workspace 探索本就解析該檔（取 spec_dir），壞檔下的既有失敗維持現狀（實測 2026-08-09：schemas 於壞 yaml exit 1；completion、config 免疫）。
- **Dual 18**：list、show、validate、analyze、drift、archive、discard、artifact、language、status、instructions、new、workflow-config、task、in-progress、discuss、review、verify。部分 Dual 動詞於模式解析前保留不消費 store 的前置步驟（凍結既有順序）：instructions 的 `--skill` 分流走 ModeFree 路徑、workflow-config 的 argv／stdin 正規化先於模式解析、review／verify 的 clap → StationVerb 正規化先行（雙臂宣告於 station_dual）。
- **FsOnly 1**：demo。
- **RemoteOnly 1**：claim。

## Implementation Contract

**觀察行為（全部凍結，變更即 bug）**

- 兩模式所有動詞的 stdout／stderr／exit code 逐位元不變；既有整合測試（remote_verb_parity、remote_read_path、remote_write_path、config_fail_closed、no_raw_wire_json 等）零修改全綠。
- claim 於 fs 模式、demo 於 remote 模式的拒絕文案逐字不變；demo 拒絕不發任何 server 請求。
- 不讀取專案設定的 ModeFree 動詞於壞 .speclink.yaml 目錄下正常執行（completion、config 印出內容、exit 0）；schemas／templates／update 因 workspace 探索讀檔的既有失敗行為不變；Dual 動詞於同環境維持 fail-closed（config_fail_closed 既有對照）。

**結構保證（編譯期）**

- dispatch 的每個 Commands variant 必經一個形狀組合子或明寫直呼（ModeFree）；Dual 組合子的兩臂為必填參數。
- commands.rs 與 remote_commands.rs 中不再存在函式內的 remote_ctx() 分岔（remote_ctx 僅由組合子層呼叫）；demo 函式內不再有模式檢查。

**新增測試（crates/speclink-cli/tests/it/mode_dispatch.rs）**

- ModeFree 於壞 yaml 可執行：BAD_YAML 專案下跑 completion 與 config，斷言 exit 0 且 stderr 不含 .speclink.yaml。
- FsOnly 拒絕：remote 設定的專案（不啟 server）跑 demo，斷言非零 exit、stderr 含現行文案、無網路請求發出。
- RemoteOnly 拒絕：fs 專案跑 claim，斷言非零 exit、stderr 含現行文案。

**驗證目標**

- cargo test -p speclink-cli --test it 全綠（含新模組 mode_dispatch）。
- 以 grep 斷言結構：commands.rs 內 remote_ctx( 的呼叫點僅存在於組合子定義處。

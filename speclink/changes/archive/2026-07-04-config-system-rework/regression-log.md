# 回歸對照記錄（task 6.3）

2026-07-04 執行自我基線雙沙盒對照（沿用 store-trait-and-fs-adapter 核准的作法）：

- **基線**：git HEAD `19230df` ＋ 前一 session 遺留的未提交修改（`assets/skills/{archive,commit,discuss}.md` 措辭、`init.rs` 的 MARKER_VERSION v1.2.0 與討論歸檔措辭——皆非本 change 範圍）建置的 release exe。
- **對照**：本 change 完成後的 release exe。
- **方法**：39 個指令區塊（24 parity ＋ 6 color[CLICOLOR_FORCE=1] ＋ 9 twin/樹狀 SHA），雙沙盒逐 byte 比對，路徑三種拼法與 ISO 時間戳正規化。腳本 `regress.ps1` 位於 session scratchpad（會隨 session 清空；沿用先前建議：如需長期保留應收進 repo `scripts/`）。

## 結果：39/39 通過（0 unexpected divergence）

## 刻意分歧清單（新版設定佈局造成）

| 情境 | 分歧 | 原因 |
| --- | --- | --- |
| 含舊政策鍵的專案執行任何指令（P15–P22、C05–C06、T4） | stderr 多一行 `speclink: warning: deprecated policy keys …`；stdout 位元級不變 | deprecation 警告需求 |
| `instructions tasks --json`（P10、P21） | `instruction` 欄位尾端多 TDD／audit 紀律段落 | 政策注入改取四層解析結果 |
| `SPECLINK_*` 環境變數設定時（P23、P24） | 新 exe 尊重環境變數覆寫；基線無此層 | 新功能（四層解析第一層） |
| `speclink init` 產生的 `.speclink.yaml`、`openspec/config.yaml`（T1、T2） | 範本內容改變（政策示例移至 config.yaml、workspace 檔瘦身） | init 範本需求 |

## 明確驗證不變的部分

- 新佈局 fixture 下的 list／status／show／validate／analyze／schemas／templates／instructions(proposal/specs/apply) 輸出位元級一致。
- init／update 生成的 CLAUDE.md、AGENTS.md、`.claude/skills/`、`.agents/skills/` 全部位元級一致（T1、T2、T4 樹狀 SHA）。
- task done、new change 的 stdout 與樹狀效果一致（T3、T5）。
- color 輸出（ANSI）一致（C01–C04）。

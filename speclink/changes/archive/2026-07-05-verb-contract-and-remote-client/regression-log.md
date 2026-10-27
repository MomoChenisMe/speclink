# 回歸對照記錄（task 6.3）

2026-07-05 執行自我基線雙沙盒對照（沿用 store-trait-and-fs-adapter 與 config-system-rework 核准的作法）：

- **基線**：git HEAD `cfb760b` worktree 建置的 release exe（不含前 session 遺留的未提交資產修改——`assets/skills/{archive,commit,sync}.md` 的 completeness 措辭改寫屬先前已歸檔 change 的產物，會出現在 twin 分歧清單並標注）。
- **對照**：本 change 完成後的 release exe。
- **方法**：32 個指令區塊（22 parity ＋ 6 color[CLICOLOR_FORCE=1] ＋ 4 twin/樹狀 SHA），雙沙盒逐 byte 比對，沙盒路徑三種拼法（`\`、`\\`、`/`）與 ISO 時間戳正規化。腳本 `regress.ps1` 位於 session scratchpad（會隨 session 清空；沿用先前建議：如需長期保留應收進 repo `scripts/`）。

## 結果：stdout parity／color 28/28 通過（0 unexpected divergence）；twin 樹狀分歧全數為刻意更新

## 刻意分歧清單（技能措辭動詞化與遺留措辭造成）

| 情境 | 分歧 | 原因 |
| --- | --- | --- |
| T1–T3 `speclink-discuss/SKILL.md` | Step 0 詞彙載入從「read `openspec/LANGUAGE.md`」改為「Run `speclink language show`」 | 本 change 技能動詞化（store 文件讀取動詞） |
| T1–T3 `speclink-propose/SKILL.md` | overlap scan 從 Glob 檔案路徑改為 `speclink list --specs --json`＋`speclink show <spec-id>`；依賴閱讀改為 `speclink artifact cat <artifact-id> --change` | 本 change 技能動詞化 |
| T1–T3 `speclink-commit/SKILL.md` | tasks／proposal 讀取改為 `speclink artifact cat`；另含前 session 遺留的 completeness 措辭改寫 | 本 change 技能動詞化＋前 session 遺留（非本 change） |
| T1–T3 `speclink-archive/SKILL.md` | delta sync → completeness 措辭改寫 | **前 session 遺留（非本 change）** |
| golden snapshots（claude/codex/neutral-cli/neutral-tool-call） | 因上述資產措辭更新刻意重錄（UPDATE_GOLDEN=1）；比對加入行尾正規化（CRLF→LF），使 golden 在 core.autocrlf=true 的 checkout 上可重現 | 本 change task 6.3 |
| 新增 golden `remote-claude.marker.md` | remote 模式 marker 變體（無 openspec/ 路徑句、含動詞指引） | 本 change task 6.1/6.2 |

## 明確驗證不變的部分

- list／list --json／list --specs（含 --json）／status（含 --json）／show（change 與 spec）／validate／instructions（proposal、design、tasks、apply；人眼與 --json）／schemas／templates／discuss list（含 --json）／discuss show --json 輸出位元級一致（P1–P19）。
- task done --json、new change、new artifact 的 stdout 一致（P20–P22）。
- color 輸出（ANSI）一致（C1–C6）。
- init 生成的 CLAUDE.md／AGENTS.md marker（fs 措辭不變）、`.claude/settings.json`、`.gitignore`、`.speclink.yaml` 與其餘全部 skills 位元級一致（T1–T3 樹狀 SHA）；update stdout 一致（T4）。

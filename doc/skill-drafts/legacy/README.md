# Legacy Single-File Skill Drafts

> **狀態**：已停用、僅供歷史對照（superseded）。
>
> **取代者**：`doc/skill-drafts/<skill>/{workflow.md, bindings/{bash,tool}.md, frontmatter.yaml}` 結構（design.md §4.3）。

---

## 為什麼搬走？

原始 5 個 skill drafts 為 single-file 結構，**workflow 邏輯與 bash CLI command 混在一起**，無法支援 design.md §4.3 規範的 workflow + bindings 拆分（host-agnostic workflow + 多 host bindings）。

2026-05-21 將 5 個 skill 全部拆成新結構：

| 原 single-file | 新結構 |
|---|---|
| `speclink-propose.md` | `speclink-propose/{workflow,bindings/{bash,tool},frontmatter}.md/yaml` |
| `speclink-apply.md` | `speclink-apply/...` |
| `speclink-archive.md` | `speclink-archive/...` |
| `speclink-discuss.md` | `speclink-discuss/...` |
| `speclink-ingest.md` | `speclink-ingest/...` |

新結構的 workflow.md 用 canonical operation id（如 `change.create`）引用，無內嵌 CLI command；bindings/{bash,tool}.md 提供 op → 具體 verb 翻譯表，deploy 時 engine 依目標 host 拼合（design.md §4.3 / §20）。

## 為什麼保留？

- **Proof of work**：拆分前後可對照、驗證資訊未漏
- **歷史記錄**：拆分過程的設計演進
- **Fallback**：若新結構出現未預期問題、可暫時參考原始版本

## 警告

- **不要**從此目錄 deploy skill 到任何 agent host
- **不要**繼續維護這幾個檔案；任何後續更新只進新結構
- **不要**將此目錄列入 codegen / install 路徑

實作階段 `speclink init --tools <host>` 與 `speclink.installSkills()` 都應該**只**從新結構 deploy。

## 何時刪除

實作階段 MVP ship 後、確認新結構穩定且 deploy pipeline 跑得起來，可考慮 git rm 這整個 `legacy/` 目錄。在那之前先保留。

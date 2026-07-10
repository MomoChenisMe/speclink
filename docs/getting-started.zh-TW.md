# 入門教學

> English version: [getting-started.md](getting-started.md)

> **文件狀態：**本文描述目前已實作的本地 repo 流程。Local/Remote 平台的目標架構與交付階段，以[平台架構藍圖](platform-architecture.zh-TW.md)為準。

本教學在純本地專案走完一輪完整的規格驅動開發(SDD)循環:**init → discuss → propose → apply → verify → archive**。

Speclink 的設計是由 AI 代理(Claude Code 或 Codex)透過生成的 `/speclink-*` 技能驅動,底下由 `speclink` CLI 擔任引擎。以下每一步同時展示要呼叫的技能與背後運作的 CLI。

## 0. 安裝

從原始碼建置(需 Rust 工具鏈):

```
cargo install --path crates/speclink-cli
speclink --version
```

預期輸出:`speclink 0.1.0 (x64)`(架構後綴依平台而異)。

## 1. init — 初始化專案

```
speclink init
```

預期輸出:

```
✓ Initialized at <你的專案>\openspec
Generated files for: claude
```

這會建立 `openspec/` 規格目錄(`specs/`、`changes/archive/`、`config.yaml`)、`.speclink.yaml` 應用設定、`.gitignore` 的 `.speclink/` 條目,以及 AI 工具檔案(`CLAUDE.md` 與 `/speclink-*` 技能)。已安裝的 AI 工具會自動偵測;可用 `--tools claude,codex` 明確指定。

## 2. discuss — 需求模糊時的選用步驟

在代理中執行 `/speclink-discuss add csv export`。代理透過 CLI 把討論記錄成持久文件:

```
speclink discuss new "add csv export"     → ✓ Created discussion: add-csv-export
speclink discuss add-round <slug> --stdin → ✓ Recorded round 1 (interview) …
speclink discuss conclude <slug> --stdin  → ✓ Concluded discussion 'add-csv-export'
```

文件位於 `openspec/discussions/add-csv-export.md`,逐輪累積。需求已經清楚時可完全跳過此步。

## 3. propose — 規劃 change

執行 `/speclink-propose add-csv-export`(或加 `--from-discussion add-csv-export` 以收斂後的討論播種)。代理建立 change 與四個產物:

```
speclink new change add-csv-export --agent claude
speclink new artifact proposal --change add-csv-export --stdin
speclink new artifact spec csv-export --change add-csv-export --stdin
speclink new artifact design --change add-csv-export --stdin
speclink new artifact tasks --change add-csv-export --stdin
```

隨時可查看進度:

```
speclink status --change add-csv-export
```

預期輸出:artifact DAG,以 `✓ done`／`○ ready`／`✗ blocked` 標示;四個產物齊備後顯示 `✓ All artifacts complete`。

## 4. apply — 實作任務

執行 `/speclink-apply add-csv-export`。代理讀取產物、逐一完成 `tasks.md` 的核取方塊,並記錄每項完成:

```
speclink task done 1 --change add-csv-export
→ ✓ Task 1 marked as done: <任務描述>
```

代理透過 `speclink instructions apply --change add-csv-export --json` 取得 context 檔案、進度與剩餘任務(全部勾選後 state 變為 `all_done`)。

## 5. verify — 對照產物驗證實作

執行 `/speclink-verify add-csv-export`。代理將實作與 spec delta、design 契約逐一對照。結構健檢也可直接執行:

```
speclink validate add-csv-export   → ✓ add-csv-export — valid
speclink analyze add-csv-export    → 四維度發現報告
```

## 6. archive — 讓 change 落地

執行 `/speclink-archive add-csv-export`,或直接:

```
speclink archive add-csv-export -y
```

預期輸出:

```
✓ Archived: add-csv-export → 2026-07-04-add-csv-export
Specs applied: csv-export (added: 1, modified: 0, removed: 0, renamed: 0)
```

delta 規格合併進正典 `openspec/specs/csv-export/spec.md`,change 目錄移入 `openspec/changes/archive/`;若這是從某討論晉升的最後一個 change,該討論會一併歸檔。

## 檔案放哪裡

| 路徑 | 用途 |
| --- | --- |
| `openspec/specs/<cap>/spec.md` | 正典規格(當前事實) |
| `openspec/changes/<name>/` | 進行中的 change 提案 |
| `openspec/changes/archive/` | 已歸檔的 change |
| `openspec/discussions/` | 討論文件 |
| `openspec/config.yaml` | 工作流設定 |
| `.speclink.yaml` | 應用設定(宿主側) |
| `.speclink/` | 工作資料(gitignored) |

Engine、TeamStore、Server 與 UI 的目標架構見[平台架構藍圖](platform-architecture.zh-TW.md)。

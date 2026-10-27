---
title: 產出流程 schema 管理
section: SDD 工作流
order: 220
keywords: [schema, 產出流程, spec-driven, no-spec-change, 不改規格, change schema, schema init, schema validate]
sources: [workflow-schemas]
generated: 2026-10-27T11:03:57+08:00
---
# 產出流程 schema 管理

產出流程（schema）定義一個變更會產出哪些文件、什麼順序。內建的產出流程有兩個：spec-driven，以及 no-spec-change（不改規格）。你可以列出、檢查、複製或新建產出流程，也可以替一個進行中的變更換產出流程。

## 內建的兩個產出流程

執行 `speclink schemas` 會列出可用的產出流程。內建的只有這兩個：

| 名稱 | 說明文字 | 產出的文件 |
| --- | --- | --- |
| spec-driven | 「Default OpenSpec workflow - proposal → specs → tasks (design optional)」 | 提案、規格差異（delta）、任務，設計選用 |
| no-spec-change | 「No-spec-change workflow - proposal → tasks (design optional), for changes that alter no promised behavior」 | 提案、任務，設計選用。沒有規格差異 |

每個 artifact 各自帶有 template。名稱的解析順序是專案層 → user 層 → 內建：專案裡有同名的產出流程時，它會遮蔽內建的那一個。`speclink schema which --all` 會同時列出被遮蔽的內建位置。

每個內建流程的定義都只有一份。`speclink schema fork spec-driven` 與 `speclink schema fork no-spec-change` 複製出來的 schema.yaml，各自與對應的內建定義逐位元組相同。spec-driven 的 specs 指引含 Purpose 段規則、MODIFIED 工作流的 BEFORE 註記步驟，以及 REMOVED-SCENARIO 合併門檻段落。

兩個內建流程在本機與 remote 專案都可以用。

## 不改規格：no-spec-change

no-spec-change 給「不改任何正式規格承諾的行為」的變更用，例如純重構、搬檔、換內部名稱。它和 spec-driven 的差別：

- **提案**：範本有 Why、What Changes、Non-Goals（選用）與 Impact，沒有 Capabilities 段。Why 要寫明「為什麼這個變更不改正式規格承諾的行為」，並列出你查過的正式規格。
- **任務**：最後一個任務要用既有測試確認相關正式規格的行為沒變。任務不必引用需求名稱。
- **設計**：指引與範本和 spec-driven 相同。
- **檢查**：`speclink validate` 不會出「No delta specs found」警告。spec-driven 的變更沒有 delta 時照舊會出這條警告。
- **封存**：不必帶 `--skip-specs`，正式規格不會有任何變動。

建立時指定：`speclink new change tidy --schema no-spec-change`。`/speclink-propose` 會自己判斷要用哪一個，見[提案](propose.md)。驗證站對這類變更改做「行為檢查」，見[驗證站](verify.md)。

### 不改規格的變更不能寫 delta

對 no-spec-change 的變更執行 `speclink new artifact spec <capability> --change <名稱>`（帶 `--stdin` 或 `--force` 也一樣）會被拒絕，什麼都不寫。錯誤訊息說明這個變更的產出流程沒有 specs，並提示 `speclink change schema <名稱> spec-driven`。只要產出流程裡沒有 specs 這個 artifact，規則都相同；有 specs 的照常寫入。

## 替變更換產出流程：speclink change schema

```
speclink change schema <變更名> <產出流程名>
speclink change schema <變更名> <產出流程名> --json
```

這個指令只改變更狀態檔裡 `schema:` 那一行，其他內容逐位元組保留。狀態檔沒有這一行時，指令會新增一行。成功時 stdout 印一行 `✓ <變更名> schema: <舊值> → <新值>`。新舊相同時印 `✓ <變更名> schema: <值> (unchanged)`，不寫檔。

例：變更 tidy 目前是 spec-driven、沒有 delta，執行 `speclink change schema tidy no-spec-change`，stdout 是 `✓ tidy schema: spec-driven → no-spec-change`。

下列情況會被拒絕。拒絕時 stderr 印一行原因，任何檔案都不改：

| 情況 | 結果 |
| --- | --- |
| 變更名不是進行中的變更（名稱逐字比對） | 找不到該變更 |
| 變更的狀態檔解析不了 | 拒絕 |
| 產出流程名不是小寫 kebab-case（含路徑分隔符、`..`、空白、大寫字母，或空字串），例如 `../evil`、`Spec-Driven` | 名稱不合法（`must be lowercase kebab-case`），在找產出流程之前就擋下 |
| 專案層、user 層與內建都找不到這個產出流程 | 與其他指令相同的「找不到 schema」訊息 |
| 新產出流程沒有 specs，而變更已有 delta | 列出這些 delta 的 capability，例如有 `specs/auth/spec.md` 時列出 auth |
| 變更有未結的驗證工單 | 提示先執行 `speclink verify discard <變更名>` |
| 狀態檔的 `schema:` 不是寫在行首的單行（例如 `schema : x`，或值寫在下一行） | 提示先改成單行的 `schema: <名稱>` |

**驗證章會被清掉**：變更已有驗證章時，切換會在同一次寫入裡移除驗證章，stdout 第二行印 `  verify stamp cleared — re-run verify under <新值>`。驗證站記下的輪數與審查章保留不動。換了產出流程，驗證要依新流程重跑。

**worktree**：在主 checkout、worktree 政策開啟，而且這個變更已綁定 worktree 時，指令寫的是 worktree 裡的那一份，主副本不變。

這個指令在本機與 remote 模式的效果與輸出形狀相同。

## 指令

| 指令 | 作用 |
| --- | --- |
| `speclink schemas` | 列出可用的產出流程與說明 |
| `speclink schema which --all` | 列出全部可解析的產出流程，各自標示解析到的位置與來源層級（project／user／built-in）。同名被遮蔽的位置一併列出 |
| `speclink schema validate <名稱>` | 檢查該產出流程，含它引用的每個 template 檔是否存在。加 `--verbose` 逐項印出驗證步驟與結果 |
| `speclink schema fork <名稱>` | 複製一份產出流程供專案修改 |
| `speclink schema init <名稱>` | 建立新的產出流程骨架。加 `--default` 把它設為專案預設 |
| `speclink change schema <變更名> <名稱>` | 替一個進行中的變更換產出流程，見上方說明 |

### schema init 產出什麼

`speclink schema init my-flow` 產出三個檔：

| 路徑 | 內容 |
| --- | --- |
| speclink/schemas/my-flow/schema.yaml | 兩個 artifact（plan、tasks）與 apply 區塊 |
| speclink/schemas/my-flow/templates/plan.md | `# plan` |
| speclink/schemas/my-flow/templates/tasks.md | `# tasks` |

骨架自身就通過載入與 validate。沒帶 `--description` 也不會產出無法解析的檔案。

加 `--default` 時，指令把 `schema: my-flow` 寫進 `speclink/config.yaml`，檔內其餘內容逐位元組保留。config.yaml 不存在時，建立只含 schema 鍵的檔案。

### 名稱規則

fork 與 init 的目的名稱必須是小寫 kebab-case：小寫字母開頭，只含小寫字母、數字與連字號。不符合時指令失敗，錯誤訊息說明名稱須為小寫 kebab-case。例如 `speclink schema init My_Schema` 會被拒絕。

## 載入時的檢查

任何動詞載入產出流程時都做下列檢查。任一失敗即回錯誤，該產出流程不能被任何動詞使用：

- artifact id 不得重複。
- requires 只能指向存在的 artifact id。錯誤訊息指名該 artifact 與不存在的 id。
- requires 不得循環。錯誤訊息印出完整環路徑，例如「a → b → a」。
- version 鍵必填，且須為正整數。0 與 1.5 都不合法。
- 每個 artifact 的 description 鍵必填，值可為空字串。
- 每個 artifact 的 template 鍵必填且非空。

validate 檢查 template 檔是否存在。自訂產出流程宣告了 templates 目錄裡沒有的檔，validate 失敗並指名缺席的檔。

**出處**：`workflow-schemas`

---
title: 產出流程 schema 管理
section: SDD 工作流
order: 220
keywords: [schema, 產出流程, spec-driven, schema init, schema validate]
sources: [workflow-schemas]
generated: 2026-09-02
---
# 產出流程 schema 管理

產出流程（schema）定義一個變更會產出哪些文件、什麼順序。內建的產出流程叫 spec-driven。你可以列出、檢查、複製或新建產出流程。

## 內建的 spec-driven

執行 `speclink schemas` 會列出可用的產出流程。spec-driven 的說明文字是「Default OpenSpec workflow - proposal → specs → tasks (design optional)」。它的四個 artifact 各自帶有 template。

內建定義只有一份。`speclink schema fork spec-driven` 複製出來的 schema.yaml 與內建定義逐位元組相同。裡面的 specs 指引含 Purpose 段規則、MODIFIED 工作流的 BEFORE 註記步驟，以及 REMOVED-SCENARIO 合併門檻段落。

## 指令

| 指令 | 作用 |
| --- | --- |
| `speclink schemas` | 列出可用的產出流程與說明 |
| `speclink schema which --all` | 列出全部可解析的產出流程，各自標示解析到的位置與來源層級（project／user／built-in）。同名被遮蔽的位置一併列出 |
| `speclink schema validate <名稱>` | 檢查該產出流程，含它引用的每個 template 檔是否存在。加 `--verbose` 逐項印出驗證步驟與結果 |
| `speclink schema fork <名稱>` | 複製一份產出流程供專案修改 |
| `speclink schema init <名稱>` | 建立新的產出流程骨架。加 `--default` 把它設為專案預設 |

### schema init 產出什麼

`speclink schema init my-flow` 產出三個檔：

| 路徑 | 內容 |
| --- | --- |
| openspec/schemas/my-flow/schema.yaml | 兩個 artifact（plan、tasks）與 apply 區塊 |
| openspec/schemas/my-flow/templates/plan.md | `# plan` |
| openspec/schemas/my-flow/templates/tasks.md | `# tasks` |

骨架自身就通過載入與 validate。沒帶 `--description` 也不會產出無法解析的檔案。

加 `--default` 時，指令把 `schema: my-flow` 寫進 `openspec/config.yaml`，檔內其餘內容逐位元組保留。config.yaml 不存在時，建立只含 schema 鍵的檔案。

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

---
title: 安全稽核：/speclink-audit
section: SDD 工作流
order: 185
keywords: [audit, 安全稽核, 安全檢查, 報告, locale, 嚴重度]
sources: [audit-skill, "user-documentation#完整工作流指南說明用途與使用時機"]
generated: 2026-10-16T23:25:26+08:00
---

# 安全稽核：/speclink-audit

`/speclink-audit` 是安全檢查。它是工具技能，需要時才呼叫，不是每個變更都要經過的步驟。

## 兩種用法

- **獨立模式**：你直接呼叫 `/speclink-audit`，它檢查變更後產出一份報告。
- **精簡檢查清單**：實作（apply）期間會引用 audit 的精簡檢查清單（Discipline Mode）。這種用法不產出報告。apply 什麼時候引用它，屬於實作的流程，不在本技能的規格裡。

## 報告用哪種語言

獨立模式的報告照專案的 locale 撰寫，結尾的摘要也一樣。以 `speclink workflow-config languages --json` 取得實際語言；查詢失敗時回報錯誤並停止。

| locale | 報告的說明、標題與表格標籤 | 嚴重度、檔案路徑與指令 |
| --- | --- | --- |
| tw | 繁體中文 | 英文 |
| ja | 日文 | 英文 |
| en | 英文 | 英文 |
| 未設定 | 主機／Server 系統語言 | 英文 |

嚴重度標籤 Critical、High、Medium、Low 一律維持英文。例如 locale 是 tw、找到一個問題時，報告的說明與修法是繁體中文，嚴重度（例如 High）與檔案路徑是英文。

精簡檢查清單的內容不受這條報告語言規定影響。locale 與對話語言的分工，見[工作流政策與設定](policy-config.md)。

下一步：回到[工作流總覽：站別與交棒](workflow-overview.md)。

**出處**：`audit-skill`、`user-documentation`

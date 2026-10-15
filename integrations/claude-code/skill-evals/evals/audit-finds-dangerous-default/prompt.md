---
name: audit-finds-dangerous-default
description: 'audit 回報預設關閉 TLS 驗證的危險預設值，不改檔（技能 frontmatter 禁用 Edit／Write）。技能錯（保留）：舊版本文 Phase 3 寫「If fixable: apply the fix directly」，與 frontmatter 矛盾，舊版會嘗試 Edit 而被拒。'
tags: [skill:audit]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Bash, Edit, Write, Skill, Agent]
---

/speclink-audit src/http.js

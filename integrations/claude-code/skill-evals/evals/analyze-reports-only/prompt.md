---
name: analyze-reports-only
description: analyze 只回報跨 artifact 的檢查發現，不改任何檔案。
tags: [skill:analyze]
max_turns: 30
timeout_seconds: 600
allowed_tools: [Read, Glob, Grep, Bash, Edit, Write, Skill, Agent]
---

/speclink-analyze demo-change

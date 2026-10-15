---
name: propose-from-discussion
description: propose 從討論建立 change；人工驗收任務的 [M] 標記緊貼 checkbox、跑 validate、不自動接著 apply。
tags: [skill:propose]
max_turns: 80
timeout_seconds: 1800
allowed_tools: [Read, Glob, Grep, Bash, Edit, Write, Skill, Agent]
---

/speclink-propose --from-discussion search-bar（直接使用這份討論，change 名稱用 add-search-bar）

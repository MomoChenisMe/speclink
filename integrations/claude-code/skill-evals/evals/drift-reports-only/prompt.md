---
name: drift-reports-only
description: drift 只回報 change 與程式碼的偏移（指出 src/old.js 已不存在），不改任何檔案。
tags: [skill:drift]
max_turns: 30
timeout_seconds: 600
allowed_tools: [Read, Glob, Grep, Bash, Edit, Write, Skill, Agent]
---

/speclink-drift add-greeting

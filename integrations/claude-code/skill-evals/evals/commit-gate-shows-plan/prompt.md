---
name: commit-gate-shows-plan
description: commit 先呈現只含本 change 檔案的計畫與訊息、請使用者確認，確認前不 commit。
tags: [skill:commit]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Bash, Edit, Write, Skill, Agent]
---

/speclink-commit add-greeting

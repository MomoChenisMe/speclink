---
name: verify-freezes-scope
description: verify 凍結範圍後回報規格不符與選項並停下，有必修發現時不蓋章。
tags: [skill:verify]
max_turns: 60
timeout_seconds: 1500
allowed_tools: [Read, Glob, Grep, Bash, Edit, Write, Skill, Agent]
---

/speclink-verify add-greeting

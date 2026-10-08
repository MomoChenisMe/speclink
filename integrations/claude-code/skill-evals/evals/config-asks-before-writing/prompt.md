---
name: config-asks-before-writing
description: config 一次只問一個政策欄位，或先呈現待核准的 diff；核准前不寫入 workflow config。
tags: [skill:config]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Bash, Edit, Write, Skill, Agent]
---

/speclink-config

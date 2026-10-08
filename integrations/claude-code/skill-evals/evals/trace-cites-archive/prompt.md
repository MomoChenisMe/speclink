---
name: trace-cites-archive
description: trace 說出 greeting 規格的來源 change，不改任何檔案。
tags: [skill:trace]
max_turns: 30
timeout_seconds: 600
allowed_tools: [Read, Glob, Grep, Bash, Edit, Write, Skill, Agent]
---

/speclink-trace greeting 這個能力是怎麼來的？

---
name: quality-pauses-after-round
description: 'quality 跑完兩站後把發現一起回報並停下，不蓋章。技能錯（保留）：舊版在 Haiku 上跑完 review 站就結束這一輪（說「Verify runs next」卻沒跑 verify），沒有兩站一起回報。'
tags: [skill:quality]
max_turns: 100
timeout_seconds: 2400
allowed_tools: [Read, Glob, Grep, Bash, Edit, Write, Skill, Agent]
---

/speclink-quality add-greeting

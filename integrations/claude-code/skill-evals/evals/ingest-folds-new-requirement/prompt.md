---
name: ingest-folds-new-requirement
description: ingest 把新需求併進進行中 change 的規格 delta，並跑 validate。
tags: [skill:ingest]
max_turns: 50
timeout_seconds: 1200
allowed_tools: [Read, Glob, Grep, Bash, Edit, Write, Skill, Agent]
---

/speclink-ingest add-greeting 新需求：greeting.txt 第二行要加上中文問候「你好」

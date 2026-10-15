---
name: worktree-merge-lands-branch
description: worktree-merge 把 speclink/add-greeting 合回主分支，再移除 worktree。
tags: [skill:worktree-merge]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Bash, Edit, Write, Skill, Agent]
---

/speclink-worktree-merge add-greeting

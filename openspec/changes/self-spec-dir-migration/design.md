## Context

`speclink update` 的遷移（spec-dir-migration）只在三個條件都成立時搬移：主目錄（不是 linked worktree）、沒有其他 worktree、規格資料夾沒有未提交修改（含未追蹤檔）。本變更自己的文件就在 `openspec/changes/self-spec-dir-migration/` 下，apply 期間勾選任務會改到 tasks.md，所以「何時 commit、何時執行 update」的順序決定遷移會不會被暫緩。另外，本機安裝的 `speclink`（~/.local/bin 與 /opt/homebrew/bin）可能是舊版，舊版沒有遷移功能。

repo 工具在搬移後會立刻壞掉：scripts/docs/vocabulary-guard.test.mjs 讀 `openspec/LANGUAGE.md`、scripts/release/delivery-gate.test.mjs 以 `openspec/changes` 與 `openspec/discussions` 為略過路徑、文件的連結檢查會抓到 `../openspec/...` 的斷鏈。這些修正必須和搬移在同一個 commit，否則 CI 會紅。

## Goals / Non-Goals

**Goals:**

- 以產品已發布的遷移搬移本 repo，不手動搬檔。
- 搬移與工具修正在同一個 commit 落地，CI 全程可綠。

**Non-Goals:**

- 不改產品程式碼與正式規格的需求；不改歷史記錄與 CHANGELOG。

## Decisions

### D1：先 commit、再遷移、最後一起提交

順序固定：(1) 確認遷移相關的三個變更已封存、`git worktree list` 只有主目錄（不必等第一批發版：本變更封存後，九刀一次發版）；(2) 把本變更目前的文件與勾選狀態 commit，讓規格資料夾乾淨；(3) `cargo build -p speclink-cli` 後以 `./target/debug/speclink update` 遷移（不用可能過期的安裝版）；(4) 修正 repo 工具、文件連結與評測夾具；(5) 跑完文件守門與發版守門後，把搬移與修正放在同一個 commit。遷移後才勾選的任務寫在 `speclink/changes/self-spec-dir-migration/tasks.md`，隨同一個 commit 提交。替代方案「手動 git mv」被否決：要驗證的正是產品的遷移流程。

### D2：工具與文件的改寫範圍

只改指向本 repo 規格資料夾的路徑與連結：腳本的讀檔路徑與略過路徑、`.gitignore` 的向量搜尋資料庫、`.dockerignore` 的排除項、PR 範本、貢獻指南、release 技能、技能評測的夾具與評分規則（評測以 `speclink init` 建立示範專案，新版建立 `speclink/`）、文件中描述 repo 自身的句子與 `../openspec/...` 相對連結。歷史記錄（已封存的變更與討論、CHANGELOG）保留原路徑；發版守門的略過路徑改為 `speclink/changes` 與 `speclink/discussions`，因為歷史記錄搬移後仍在略過範圍內。

## Implementation Contract

- 搬移後 `./target/debug/speclink list` 的 stderr 沒有佈局警告，進行中與已封存變更的數量與搬移前相同，每個變更資料夾只有 `.meta.yaml`。
- `node --test scripts/*.test.mjs scripts/*/*.test.mjs` 通過；`git grep -n "openspec/"` 在歷史記錄與 CHANGELOG 以外沒有命中。
- **範圍內**：proposal Impact 所列檔案與搬移本身。**範圍外**：產品程式碼、正式規格需求、歷史記錄。

## Risks / Trade-offs

- [遷移被暫緩] → D1 的 (1)(2) 先排除 worktree 與未提交修改；暫緩時依 stdout 的原因處理後再執行。
- [舊版 CLI] → 一律用 `./target/debug/speclink`，執行前確認 `--version` 的 engine 版號不低於 spec-dir-migration 封存時的版號。
- [桌面 app 開著監看本 repo] → Windows 才會占用資料夾；macOS 上搬移後依 desktop-spec-dir-migration 的行為重新解析監看。
- [協作者用舊版 CLI] → 發版說明提醒先升級再 pull。

## Migration Plan

本變更就是遷移本身。回滾：把 `speclink/` 改回 `openspec/`、`.meta.yaml` 改回 `.openspec.yaml`，再還原工具修正的 commit。

## Open Questions

(none)

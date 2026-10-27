## Why

品質關卡的章凍結範圍內檔案的內容指紋，蓋章後任何修改都會讓章失效，而且沒有略過旗標；蓋章時工單已刪除，重新蓋章要從首輪重跑兩道關卡。實測最近 100 個封存變更有 6% 重新蓋章，主因是主分支前進或合併（6 次）與蓋章後為了 CI 再修（3 次），平均每次多跑約 2.7 輪；比例在「先 rebase 再跑關卡」「先等 CI 綠再蓋章」兩條規矩開始使用後明顯下降（之前 5／46、之後 1／54）。但這兩條規矩只存在作者的個人記憶，外部使用者看不到；docs/workflow 甚至寫著蓋章後檔案被改時「回該關卡再跑一輪」，低估了重跑成本。討論決定不改引擎（完整首輪在 5／12 次重跑中抓到新問題，只複驗失效檔案會漏掉它們），改把規矩寫進文件與 quality 技能。本變更是討論 positioning-and-process-cost 分刀計畫的第 5 刀。

目標使用者與情境：透過 AI agent 開發的開發者（團隊情境下含 PO 與 PM），在 `/speclink-quality` 跑兩道品質關卡、準備蓋章的階段；以及閱讀 workflow 文件「品質關卡的共同規則」的人。

## What Changes

- docs/workflow.zh-TW.md 與 docs/workflow.md 的「品質關卡的共同規則」新增兩條：先把最新的主分支合進來（rebase 或合併）再跑關卡；專案有 CI 時先等 CI 在所有平台通過再蓋章。兩條都寫出原因（章凍結內容指紋、重蓋要從首輪重跑）。
- 同一節「蓋章後檔案又被改」的說明改正：重新蓋章要從首輪重跑（工單在蓋章時已刪除），不是「再跑一輪」。
- quality 技能準備兩道關卡時（第一輪凍結範圍之前），若變更在主分支以外的分支上（worktree 或功能分支），以 git 比對「主分支在分岔點之後改過的檔」與「本分支改過的檔」；有交集時停下來建議先 rebase，等使用者決定。
- 使用者照建議 rebase 或合併後，quality 技能先確認本變更的檔（含新檔）都已 commit，再以 `speclink review prepare --base <新的分岔點>` 移動兩站的起點，避免第一輪把主分支的修改算成本變更的。
- quality 技能每輪停下來詢問時，「進入收尾蓋章」選項在 repo 有 CI 設定時附註「建議先確認目前 commit 的 CI 已全部通過」；不新增問題。

## Non-Goals

- 不改引擎：不加略過旗標、不做只複驗失效檔案的輕量重蓋、不放寬封存的章失效守門。
- 不在技能裡自動執行 rebase、push 或查詢 CI 狀態；技能只比對與提醒（使用者 rebase 或合併後的 commit 確認與起點重設除外），動作由使用者決定。
- 不改 review 與 verify 單站技能；兩條規矩在 workflow 文件對單站使用者同樣適用。
- 不處理 worktree 分支 push 不觸發 CI 的專案設定問題，那是各專案 CI 設定的範圍。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `quality-skill`: 新增蓋章前的兩條預防提醒（跑關卡前比對主分支、收尾蓋章選項附註 CI）。
- `user-documentation`: 新增 workflow 文件載明蓋章前兩條預防規矩與重蓋成本的需求。

## Impact

- Affected specs: `quality-skill`, `user-documentation`
- Affected code:
  - Modified: docs/workflow.zh-TW.md, docs/workflow.md, crates/engine/speclink-core/assets/skills/quality.md, crates/engine/speclink-core/src/workspace/init.rs, crates/engine/speclink-core/tests/golden/assets.lock
- 影響的 crate／app：speclink-core 的 quality 技能原稿與版號；沒有程式邏輯變動。
- 設定欄位：沒有。
- 生成的技能：speclink-quality 的技能檔（claude、codex、copilot）內容改變；ASSET_VERSION 由 v1.51.0 升為 v1.52.0，golden snapshot 與 assets.lock 再生。
- 相容性影響：CLI 人眼與 `--json` 輸出不變；quality 技能多一個可能的停頓點（主分支改過本變更的檔時）。

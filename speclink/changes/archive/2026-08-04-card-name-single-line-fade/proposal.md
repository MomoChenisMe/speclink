## Why

看板卡片的名稱過長時（例如 worktree-parallel-apply 這類長 slug），標題折行會把行內尾隨的複製鈕一起帶到次行——複製鈕孤零零掉在名稱下方，識別列被撐成兩行、同欄卡片高度不齊。正典規格目前明文要求「標題 SHALL NOT 截斷——完整顯示並自然折行」，這條要求本身就是缺陷來源。使用者要求改為單行截斷、複製鈕留在同一列，截斷處以漸層淡出收尾。

實作已先行落地於 packages/ui（使用者要求直接調整、不走 change），本提案補齊規格對齊，避免正典與實作背離在 verify 站爆開。

目標使用者：以桌面 app（與 server-web）看板檢視變更與討論的開發者、PO；使用情境涵蓋 propose 之後的任何工作流階段——看板是常駐入口，長名稱在此專案是常態而非例外。

## What Changes

- 看板全尺寸卡（變更卡與討論卡）的識別列標題改為恆單行：過長時就地截斷，截斷處以尾端漸層淡出呈現，不折行、不以省略號或硬切收尾
- 複製鈕改為與標題同列的獨立元素且不被壓縮，名稱被壓縮時複製鈕仍留在文字尾端、不落次行
- 討論卡標題原本以 break-all 強制斷字折行，一併收斂到同一行為（正典本就要求兩種卡骨架統一）
- 兩張卡的名稱列抽為單一共用元件，行為由一份實作供給，兩處不再各自演化
- 規格層：desktop-app 的 requirement「看板卡片統一解剖學」MODIFIED——改寫標題截斷與複製鈕位置兩句，並改寫 Scenario「長標題折行時複製鈕仍緊跟末字元」

相容性影響：純呈現層改動。無新增或變更 CLI 子指令、旗標、stdin 與 exit code；人眼輸出與 --json shape 均不動，不觸及任何 golden 對照；無設定欄位（openspec/config.yaml、.speclink.yaml）變動；無技能或注入區塊變動。既有測試中兩條釘住舊行為（折行不截斷、複製鈕在標題元素內）的斷言同批更新，屬刻意變更。

## Non-Goals

- 不動封存頁清單列與抽屜標頭的複製鈕——它們不是看板全尺寸卡，版面約束與字級皆不同
- 不動討論欄底部 promoted 細列的複製鈕（細列骨架與字級不同，不在統一解剖學的約束內）
- 不為截斷的名稱加 tooltip 顯示全名：hover 已有複製鈕可取得完整名稱，再加浮層是重複解法
- 不改看板欄寬，也不動卡片另外兩列（描述列、meta 列）
- 已否決：只在 hover 時淡出、平常以省略號收尾——複製鈕在非 hover 時仍佔位，兩種收尾切換會讓名稱尾巴在滑鼠進出時抖動

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- desktop-app: 看板卡片統一解剖學——標題由「不截斷、自然折行」改為「恆單行、過長就地截斷並於截斷處漸層淡出」；複製鈕由「行內尾隨、標題折行時位於末行文字尾」改為「與標題同列尾隨、不因標題過長落到次行」

## Impact

- 影響的 app：packages/ui（看板卡片元件，desktop 與 server-web 共用）。apps/desktop 與 apps/server-web 透過共用元件受益，無各自改動
- 不影響任何 crate：無 Rust 端改動，引擎、CLI、host、protocol 均不動
- Affected specs: desktop-app
- Affected code:
  - New: packages/ui/src/components/CardNameRow.tsx, packages/ui/src/__tests__/cardNameRow.test.tsx
  - Modified: packages/ui/src/components/ChangeCard.tsx, packages/ui/src/components/DiscussionColumn.tsx, packages/ui/src/__tests__/kanban.test.tsx, packages/ui/src/__tests__/discussionColumn.test.tsx
  - Removed: (none)

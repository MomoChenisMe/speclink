> **Roadmap**: 四情境預設 GUI 工具矩陣的第 ② 刀（共 5，序 4→3→2→1）。來源討論 `四情境預設-gui-工具矩陣`。
> **依賴**: ① desktop-shell-and-browser（本刀在其桌面殼上加 agent 面板）。**下游**: agent 選型與 ACP 接線經驗餵給 ④ web-agent-channel。
> **狀態**: 待完整 propose（本檔為 promote 骨架）。
> **現況更新（2026-07-05，① 完成後）**: ① 實際交付超出原述——生命週期看板（teal 深淺主題）＋Spectra 級詳情抽屜＋互動任務（勾選/排序回寫 tasks.md）＋封存獨立頁；前端體系定案 shadcn/Tailwind/Zustand（packages/ui 設計系統）。本刀的 agent 對話面板 SHALL 以同一設計系統整合（缺的 shadcn 原語如 ScrollArea/Textarea 屆時補進 packages/ui）；另插隊一刀 desktop-config-multiproject（B 開啟專案+自動 init／C 設定頁／D i18n）將先於本刀。

## Why

第 ① 刀交付的桌面版是唯讀瀏覽＋動詞按鈕，還不是「像 spectra.exe」——spectra 桌面版內建對話式 agent（反組譯確認為 GitHub Copilot CLI 的 ACP 模式）。本刀在桌面殼加對話式 agent 面板，讓使用者在桌面 app 內直接與 agent 對話執行完整 SDD，情境 4 達到 spectra-complete。agent 做成可切換（Copilot / Claude Code），與現行 skills 生態並容。

## What Changes

- 桌面 app 新增對話式 agent 面板（訊息串、工具呼叫顯示、權限核准 UI）。
- 雙 ACP 接線：spawn `copilot --acp` 與 Claude Code 的 ACP，經 settings 切換；agent 透過 speclink 動詞執行 SDD。
- ACP session 生命週期管理（initialize / new_session / cancel / 連線關閉處理）。

<!-- 細節（ACP 接線協定版本、agent 選型設定、權限模型）待 /speclink-propose 於 design 階段定案 -->

## Capabilities

### New Capabilities

- `desktop-agent`: 桌面 app 內建對話式 agent 面板，雙 ACP 接線（Copilot / Claude Code），情境 4 spectra-complete。

## Impact

- Affected: 桌面 app（① 交付）新增 agent 面板與 ACP 接線層。
- 外部依賴: 使用者機器需具備 `copilot` 或 Claude Code CLI（依所選 agent）。
- 不影響 CLI、fs 模式與引擎行為。

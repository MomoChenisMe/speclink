# Verify — desktop-settings-editors

## Round 1

**Phase**: discovery
**Patch**: sha256:47cdb59c48426b696dfe322dc8509aad6ed930cb03f3c3c8d1051ad53d6f485f
**Scope**: apps/desktop/package.json, apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/codeEditor.test.tsx, apps/desktop/src/__tests__/helpers/codemirror.ts, apps/desktop/src/__tests__/projectSettingsView.test.tsx, apps/desktop/src/components/CodeEditor.tsx, apps/desktop/src/i18n/messages.ts, apps/desktop/src/views/project-settings/ContextPage.tsx, apps/desktop/src/views/project-settings/RulesPage.tsx, apps/desktop/src/views/project-settings/parts.tsx, apps/desktop/src/views/project-settings/useProjectSettings.ts, package-lock.json
- [SUGGESTION] apps/desktop/src/__tests__/projectSettingsView.test.tsx — Correctness: 情境「編輯專案說明並儲存」要求重讀檔案後縮排逐字元保留。前端測試（約第 331 行）只驗證送出去的參數；後端的原樣往返測試（`config/tests.rs:882`）只涵蓋檔尾換行，沒有多層縮排。建議在 `crates/engine/speclink-core/src/workspace/config/tests.rs` 補一條往返斷言，直接用情境字串 `"# 規範\n\n- 第一層\n  - 第二層\n    - 第三層\n"`。
- [SUGGESTION] apps/desktop/src/views/project-settings/parts.tsx — Correctness: `useEditorSave`（約第 1136–1162 行）只在「儲存進行中」擋重送，沒有跟著 `wfDisabled` 擋。會碰到的情況是：編輯中發生 remote 版本衝突（`conflict !== null`），此時儲存鈕已停用，但頁層 ⌘S（以及編輯器的 Mod-s，唯讀時快捷鍵照樣會跑）仍會再送一次寫入，繞過衝突對照卡。後端有版本檢查，所以不會寫壞檔案。建議讓 `useEditorSave` 多收一個 `disabled` 參數，`ps.wfDisabled` 為真時直接 return。
- [SUGGESTION] apps/desktop/src/views/project-settings/RulesPage.tsx — Coherence: tasks.md 2.2 的字面還寫「`Card size="nested"`（`SettingsGroupHeader` 等寬鍵名…）」，但 design D3 已改成本地組合卡頭，實作（約第 64–74 行）也照 design 做。建議把 tasks.md 2.2 的描述同步成 design D3 的寫法，避免日後追溯時誤判。
- [SUGGESTION] apps/desktop/src/components/CodeEditor.tsx — Coherence: 實作和 design D1 的列舉有幾處小出入：多加了 `highlightActiveLine`／`highlightActiveLineGutter`，D1 的擴充清單沒列（這兩個是 spec 要求「游標行」上色所需）；theme 另外用了 `var(--foreground)`、`var(--destructive)` 與 `color-mix`，不在 D1 列舉的那組 CSS 變數裡；Mod-s 放在 keymap 最前面，且沒有 `onSave` 時回傳 false，與 D1 寫的順序不同。這些都還在「只用設計 token」的範圍內。建議在 D1 補上這兩個擴充與完整的 token 清單，讓文件和程式碼一致。

## Round 2

**Phase**: validation
**Patch**: sha256:f2dceaa87329fc39ed7a6b0e295a6b8061751002ce2e4a4fac651d469ec671d6
**Scope**: apps/desktop/src/__tests__/App.test.tsx, apps/desktop/src/__tests__/codeEditor.test.tsx, apps/desktop/src/__tests__/helpers/codemirror.ts, apps/desktop/src/__tests__/platform.test.ts, apps/desktop/src/__tests__/projectSettingsView.test.tsx, apps/desktop/src/components/CodeEditor.tsx, apps/desktop/src/components/ProjectActionsMenu.tsx, apps/desktop/src/platform.ts, apps/desktop/src/views/project-settings/ContextPage.tsx, apps/desktop/src/views/project-settings/RulesPage.tsx, apps/desktop/src/views/project-settings/parts.tsx, apps/desktop/src/views/project-settings/useProjectSettings.ts, apps/desktop/vitest.setup.ts, crates/engine/speclink-core/src/workspace/config/tests.rs, packages/ui/src/__tests__/menus.test.tsx, packages/ui/src/__tests__/ui.test.tsx, packages/ui/src/components/ui/kbd.tsx, packages/ui/src/components/ui/tabs.tsx
- [SUGGESTION] apps/desktop/vitest.setup.ts — Coherence: 這次修補把 jsdom 的 Range 量測補丁（polyfill，用來補上 jsdom 沒有的瀏覽器功能）從 `helpers/codemirror.ts` 搬到 `vitest.setup.ts`，design D1 也已同步。但 tasks.md 1.1 的字面還寫「`apps/desktop/src/__tests__/helpers/codemirror.ts`（jsdom polyfill）」，現在那支檔只剩讀寫編輯器和按快捷鍵的輔助。建議把 1.1 這一句改成和 design D1 一致。

## Round 3

**Phase**: validation
**Patch**: sha256:1c0e078ac89f192f4f299614172ec883c7bd27c16428110c13dc9260a1f30d5a
**Scope**: apps/desktop/src/__tests__/codeEditor.test.tsx, apps/desktop/src/components/CodeEditor.tsx, apps/desktop/src/views/project-settings/parts.tsx, apps/desktop/src/views/project-settings/useProjectSettings.ts, packages/ui/src/components/ui/kbd.tsx

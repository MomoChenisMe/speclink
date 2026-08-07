## Why

實跑 scoped 測試政策落地（來源討論 change-scoped-test-policy）時，speclink-config 技能暴露三處脫節：判準四字面要求執行引用指令（一條規則的核實要付數分鐘測試）、討論裁決落地的 rule 有被未來重跑以「非固定輸入集可導出」誤刪的風險（誤刪即 scoped 政策失效、任務驗證退回全量）、scope hint 的收窄範圍只有隱含解讀。同時使用者裁定：大型專案全量測試昂貴（如本 repo 一輪 20 分鐘級），技能應在整理 config.yaml 時主動詢問任務驗證的測試範圍，把「只跑受影響面」從手工規則制度化為政策提問。目標使用者是透過 AI 代理跑 SDD 的開發者與 PO/PM，對應 workflow 的 config 整理階段（speclink-config 技能）。

## What Changes

調整 config 技能正典資產（速記 A–D，後續 artifacts 沿用此對照）：

- **A 判準四改靜態驗證**：驗引用「存在且可解析」一律用靜態便宜手段——路徑查檔案系統、測試名 grep 原始碼、npm script 查 package.json 宣告、CLI 子指令對 --help 面；不得執行被引用的測試或建置指令（判準一的 speclink instructions payload 探測不在此限）
- **B 刪除理由限定原則**：rule 只因不過四判準而被刪，不得因「無法自固定輸入集導出」而被刪——保障使用者裁決型 rule（討論結論落地者）免標記即受保護
- **C scope hint 收窄語意明文化**：scope hint 收窄判準一至三的全面重審至範圍內 artifacts；判準四的引用核實恆為全文件掃描；無 hint 維持全文件
- **D 政策提問增列第五問**：任務驗證步驟要全量測試或只跑受影響面——答受影響面則技能自已讀的 dependency manifests 組出專案客製的對應規則落 rules 的 tasks 段（沿既有 dry-run 核准流程落地）；答全量則不寫規則；現行文件已有測試範圍規則時提問帶現值
- 隨動：產物層版號 MARKER_VERSION 升版（v1.18.1 → v1.18.2）、golden 快照與 assets.lock 同批再生、兩工具（claude／codex）生成技能檔隨 update 再生

相容性影響:渲染輸出屬刻意變更——speclink-config 技能檔（兩 tool flavor）內容更新、其餘生成技能檔僅 frontmatter 版號隨升版變動,golden 同批刻意更新並於本提案記載;CLI 人眼輸出與 --json 契約無任何變動;既有 workspace 於 speclink update 時取得新版技能檔,未更新者僅版本探測報 stale,行為不變。

## Non-Goals

- 不新增引擎政策欄位（如 test_scope）:rules 管道已存在且經 scoped 政策落地驗證有效;欄位方案需動設定三層解析、tasks 指示注入、desktop 設定頁與 remote 可編輯欄位,且引擎不識專案結構、客製對應表仍得靠 rules 補——重複管道,來源討論已否決
- 不新增 rule 來源標記機制:config.yaml 重寫會移除註解,標記無法存續——來源討論已否決
- 不動 workflow-config 動詞的 CLI 介面、旗標與 --json 契約
- 不回改任何既有專案的 config.yaml 內容（本 repo 的 scoped 測試規則已於先前落地,不在本變更範圍）

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `config-skill`: 四條內容判準的驗證成本與刪除理由語意更新（A、B）、scope hint 收窄語意明文化（C）、政策提問增列任務驗證測試範圍第五問（D）

## Impact

- Affected specs: config-skill（一條 MODIFIED 需求涵蓋 A/B/C、一條 ADDED 需求涵蓋 D）
- Affected code:
  - Modified: crates/speclink-core/assets/skills/config.md、crates/speclink-core/src/init.rs、crates/speclink-core/tests/it/render_golden.rs、crates/speclink-core/tests/golden/assets.lock、crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/claude-worktree.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md、crates/speclink-core/tests/golden/remote-claude.marker.md、.claude/skills/speclink-config/SKILL.md、.agents/skills/speclink-config/SKILL.md、CLAUDE.md、AGENTS.md（後兩者僅注入區塊版號；其餘生成技能檔僅 frontmatter 版號隨 update 再生）
  - New: 無
  - Removed: 無

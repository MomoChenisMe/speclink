---
topic: 新增 Workspace 的來源卡用詞應去掉 Speclink 品牌前綴，直接稱 Server
slug: server-wording-debrand
status: promoted
promoted_to: server-wording-debrand
created: 2026-09-03
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 新增 Workspace 的來源卡用詞應去掉 Speclink 品牌前綴，直接稱 Server

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

使用者在「新增 Workspace」chooser 第一步截圖指出：右側來源卡寫「Speclink Server」，但正確用詞應是直接稱「Server」，不該綁定產品品牌。

採 assumptions 模式，未跑 grill 階段：需求本身已可驗證（哪個字要改、改成什麼都明確），只需確認掃描面與連帶影響。純文案變更，不引入新架構縫，故跳過 interface depth check。

Scout 結果：
- `speclink list --json` 與 `speclink discuss list --json` 皆為空——無進行中的變更或討論。
- `speclink language show`：正典對「server」這個概念**完全沒有詞條**（canon 沉默）。既有的三條「明文例外」（config.yaml 頁籤、討論 slug、worktree 直出）構成一條裁定線：開發者工具中原生詞即最直觀。
- 相關規格：`openspec/specs/workspace-chooser/spec.md`，兩處 Requirement 內文把「Speclink Server」寫進 SHALL（:11 來源分流、:220 最近開啟清單）。
- 相關歷史：`workspace-chooser-onboarding`（2026-07-20，chooser 導入）、`chooser-recent-workspaces`（2026-09-03，最近開啟清單）兩個已封存變更是這批字面的來源。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-09-03)

**Focus**: 拿掉「Speclink Server」的品牌前綴，掃描面到哪裡為止、目標詞是什麼？

**Position**: 五條假設使用者全數確認——目標詞為英文 `Server`，掃描面涵蓋四個文案鍵、規格兩處與詞彙表新詞條，中英不一致另案處理。

- **目標詞是 `Server`（英文大寫），不是「伺服器」**：使用者要的是「去品牌」而非「中譯」。同一個 chooser 內既有「遷移到 Server…」（`messages.ts:60`）與「以 Server 為準」（`messages.ts:82`）已用英文大寫 Server，改後當下一致。
- **掃描面是四個文案鍵、雙語共 6 行**：`chooser.server`（`messages.ts:61,390`，截圖那張卡）、`chooser.serverTitle`（`:51,380`，「選擇 Speclink Server」，是點卡後的下一步標題）、`servers.help`（`:256,588`，「已儲存的 Speclink server 連線」）。
- **品牌字保留處**：`tray.open`「開啟 Speclink」、`assets.*` 技能檔說明、server-web `setup.title`——這些的 Speclink 指產品本身而非 server，不動。
- **規格衝突要正面改**（canon triage：conflicts with canon）：`openspec/specs/workspace-chooser/spec.md:11` 與 `:220` 把「Speclink Server」寫進 Requirement 內文的 SHALL。只改 UI 不改規格，封存時對不上。
- **詞彙表要立「Server」詞條**（canon triage：canon is silent）：definition 為 speclink server 服務端，avoid 列 `Speclink Server`。why 須沿既有裁定線（config.yaml 頁籤／討論 slug／worktree）說明為何直出英文而非中譯。
- **連帶檔案**：`apps/desktop/src/__tests__/workspaceChooser.test.tsx` 五處以 `/Speclink Server/` 定位按鈕，不改即紅；`openspec/manual/desktop-projects.md`（2 處）與 `desktop-remote.md`（1 處）；`docs/platform-architecture.zh-TW.md`（2 處，開發文件，順手掃）。
- **不回改**：`openspec/changes/archive/` 與 `openspec/discussions/archive/` 下的歷史 artifacts，依正典原則「歷史 artifacts 不回改」。

**Ruled out**: 
- 「只改截圖那一張卡」——點下卡片的下一步標題仍寫「選擇 Speclink Server」，同一流程前後打架。
- 「把 Server 中譯為『伺服器』」——掃描面從 6 行擴大到整族 chooser／migration 文案，且與使用者「直接稱作 Server」的字面要求相反。
- 「本次順便統一 Server／伺服器的中英不一致」——設定頁頁籤與系統匣用「伺服器」、chooser 族用「Server」是既存矛盾，與品牌前綴無關；一次做完會把 6 行擴大到約 25 個文案鍵加對應測試。刻意留給另一場討論。

**Open**: 無——五條假設全數確認，可收斂。

## Conclusion

**Decision**: 把 desktop 使用者可見文案中的「Speclink Server」去品牌化為「Server」，掃描面固定為三個文案鍵（雙語 6 行）＋規格 2 處＋測試 5 處＋手冊 3 處＋開發文件 2 處，並在 `openspec/LANGUAGE.md` 立「Server」詞條把結果釘死。

- 文案（`apps/desktop/src/i18n/messages.ts`）：`chooser.server` → `Server`；`chooser.serverTitle` → `選擇 Server` / `Choose a Server`；`servers.help` 去掉 Speclink 前綴。
- 規格（`openspec/specs/workspace-chooser/spec.md`）：`:11` 與 `:220` 兩處 Requirement 內文字面改為「Server」。
- 測試（`apps/desktop/src/__tests__/workspaceChooser.test.tsx`）：五處 `/Speclink Server/` 定位改為 `/Server/`（需確認不與其他按鈕的可及性名稱撞名）。
- 手冊（`openspec/manual/desktop-projects.md` 2 處、`desktop-remote.md` 1 處）與開發文件（`docs/platform-architecture.zh-TW.md` 2 處）同批更新。
- 不動：`tray.open`、`assets.*`、server-web `setup.title` 等指產品本身的 Speclink 字；`openspec/changes/archive/` 與 `openspec/discussions/archive/` 下的歷史 artifacts。

**Rationale**: 關鍵取捨是「掃描面切在哪」。切在單張卡片會讓同一流程的下一步標題自相矛盾；切到「連中英用詞一起統一」則把 6 行變成約 25 個文案鍵，與使用者的實際要求（去品牌）無關。切在「所有帶 Speclink 前綴的 server 文案」是唯一自洽又最小的一刀。目標詞選英文 `Server` 而非中譯，因為同一個 chooser 內已有「遷移到 Server…」「以 Server 為準」，且沿用正典既有的「開發者工具中原生詞即最直觀」裁定線。

**Rejected alternatives**: 
- 只改截圖那張來源卡——下一步標題仍是「選擇 Speclink Server」，前後打架。
- 中譯為「伺服器」——與使用者字面要求相反，且掃描面暴增。
- 本次一併統一「Server／伺服器」中英不一致——既存問題，與品牌前綴正交；混進來會讓這一刀失焦。

**Deferred**: 「Server」與「伺服器」的中英用詞統一（設定頁頁籤、`servers.*` 全族、系統匣 `tray.recovery.server` 用中文，chooser／migration 族用英文）——刻意留待另一場討論。本次只保證新立的詞條不阻擋日後統一。

**Capture to**: proposal ＋ specs/workspace-chooser/spec.md ＋ openspec/LANGUAGE.md（vocabulary drift：正典對 server 概念沉默，本次補詞條）

**Next**: /speclink-propose --from-discussion server-wording-debrand

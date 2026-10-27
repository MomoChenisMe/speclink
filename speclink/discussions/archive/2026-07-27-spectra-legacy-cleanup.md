---
topic: 對齊 Spectra 的基準、測試與提及是否可全面移除（README 保留參考）
slug: spectra-legacy-cleanup
status: promoted
promoted_to: spectra-legacy-cleanup
created: 2026-07-27
created_by: MomoChen <momochenisme@gmail.com>
---

# Discussion: 對齊 Spectra 的基準、測試與提及是否可全面移除（README 保留參考）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

speclink 已完成，使用者裁定不再以 Spectra 為對齊目標：開發期用於對齊 Spectra 的基準、測試與各處提及是否可全面移除？唯一明確保留要求：README 仍要明確提到參考 Spectra。

模式：assumptions——全樹掃描（排除 node_modules／target／討論記錄）得 Spectra 提及的完整分布：README 兩份、docs/platform-architecture、正典規格 13 份（含 SHALL 句內的「parity 基線」定義）、封存變更約 40 份、LANGUAGE.md 的 why 記錄兩處、內嵌技能資產 archive.md（連動 golden snapshot 四份與 repo 技能實例兩處）、crates 源碼註解 128 處、packages/ui 與 apps/desktop 註解數處、CLI 測試註解兩檔、prompt.md。

前情：config.yaml 的 Spectra 提及與指向不存在 parity／color／twin suite 的規則已在 config-yaml-setup-skill 討論中清除；該討論同時確認 repo 內所有現存回歸保護（render_golden、speclink-cli 23 個整合測試、baseline exe 手法）均為自我基線、與 Spectra 無關。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — assumptions (2026-07-27)

**Focus**: 「移除對齊 Spectra 的基準與測試」實際上有什麼可移？各類提及分別該怎麼處理？
**Position**: 可執行的「基準與測試移除」是空集合——該做的是措辭改寫，分六類各有不同處置：
- **基準與測試：無物可移**。parity／color／twin suite 從不在 repo（scratchpad 已滅失）；repo 內現存的 render_golden、23 個 CLI 整合測試、baseline exe 對照全是自我基線，只在註解提及 Spectra，測試本體必須保留。
- **README（兩份）：改寫非僅保留**。現文「以 Spectra 2.3.1 所附 CLI 為行為參考與相容基準，並以 parity/golden tests 保護」有兩處失真：「相容基準」是進行式承諾（已不成立）、「parity tests」不存在。改寫為歷史參考語氣（「設計之初以 Spectra App 2.3.1 所附 CLI 為行為參考」＋自有 golden 保護），滿足使用者「明確提到參考 Spectra」的要求。docs/platform-architecture 同語氣改寫。
- **正典規格 13 份 18 處：改寫為自有契約、須走變更流程**。SHALL 句以「對 Spectra 2.3.1 的 parity 基線不變」定義凍結對象（如 command-runtime、verb-contract），不可單純刪除——句子的主體「既有人眼輸出與 --json 形狀 SHALL 維持位元級一致」已是自有契約，Spectra 括號可移、「對 Spectra 的刻意延伸」類敘述改為「本動詞為 speclink 擴充」。規格是正典，改動以 delta 表述走 propose。
- **內嵌技能資產 archive.md:175（「Unlike Spectra…」）：改寫，且成本最高**。這是產品出貨文本（教 agent RENAMED 行為時以 Spectra 對比），改寫為不提 Spectra 的直述；動它觸發三處同步（core assets、.claude/skills、.agents/skills）＋乾淨樹 golden 再生四份（CLAUDE.md 紅線：dirty 樹再生會烙進未提交狀態）。
- **源碼註解 128＋ 處：批次改寫為中性措辭（建議、可裁掉）**。「(matches Spectra)」對新維護者是誤導——會誤以為需安裝 Spectra 驗證；改寫為「(frozen output shape)」類措辭保留凍結理由、去掉外部依賴暗示。純註解 mechanical commit、零行為變更；歷史淵源 git blame 可考。使用者可裁定改為 boy-scout（碰到才改）以省 30＋ 檔的 diff。
- **不可動**：封存變更約 40 份與 LANGUAGE.md 的 why 記錄（LANGUAGE 原則明載歷史 artifacts 不回改；why 是決策履歷）；prompt.md 是專案初始文件，屬歷史。
**Ruled out**: 刪除仍在保護行為的測試（它們是自我基線，與 Spectra 無關）；回改封存 artifacts；README 完全移除 Spectra（使用者明確要求保留參考）。
**Open**: 源碼註解走批次改寫還是 boy-scout？本清理與 A／B 是否各自獨立成刀（建議獨立：規格 delta ＋ 文案 mechanical，與功能變更零耦合）？

### Round 2 — assumptions (2026-07-27)

**Focus**: 兩個開放問題的裁定。
**Position**: 使用者裁定：源碼註解走**批次改寫**（一個 mechanical commit；當下無 in-flight 變更，零衝突時機）；本清理**獨立成一刀**（主題 C），與 A（remote-login-ux-gaps）、B（config-yaml-setup-skill）的功能變更零耦合。

## Conclusion

**Decision**: 獨立一刀清除 Spectra 的進行式對齊語意，保留歷史參考與全部自我基線測試。六類處置：(1) 基準與測試無物可移——parity／color／twin suite 從不在 repo，現存 render_golden、23 個 CLI 整合測試、baseline exe 對照全是自我基線，測試本體保留；(2) README 兩份與 docs/platform-architecture 改寫為歷史參考語氣（「設計之初以 Spectra App 2.3.1 所附 CLI 為行為參考」），移除「相容基準」進行式承諾與不存在的「parity tests」字樣——仍明確提及 Spectra，滿足保留要求；(3) 正典規格 13 份 18 處以 delta 改寫為自有契約：SHALL 句主體「輸出維持位元級一致」不變，移「對 Spectra 2.3.1 的 parity 基線」括號、「對 Spectra 的刻意延伸」改「speclink 擴充」；(4) 內嵌技能資產 archive.md:175 的「Unlike Spectra…」改寫為不提 Spectra 的直述，三處同步（core assets、.claude/skills、.agents/skills）＋乾淨樹 golden 再生四份；(5) 源碼註解 128＋ 處（crates ＋ packages/ui ＋ apps/desktop ＋ 測試註解）批次改寫為中性措辭（如 frozen output shape），單一 mechanical commit、零行為變更；(6) 封存變更約 40 份、LANGUAGE.md why 記錄、prompt.md 不動（歷史 artifacts 不回改）。

**Rationale**: 「對齊 Spectra」已從目標變成歷史；殘留的進行式措辭（相容基準、parity 基線不變、matches Spectra）會誤導新維護者以為需安裝 Spectra 驗證，且 README 聲稱的 parity tests 根本不存在——這不是懷舊問題，是文件失真問題。輸出凍結的約束本身保留，只是凍結的權威從「與 Spectra 一致」改為「speclink 自己已發佈的契約」。

**Rejected alternatives**: 刪除提及 Spectra 的測試（它們是自我基線，保護的是現行為）；回改封存 artifacts（LANGUAGE 原則禁止）；README 完全移除 Spectra（使用者明確要求保留參考）；源碼註解 boy-scout 漸改（使用者裁定批次，當下零 in-flight 變更是最佳時機）。

**Deferred**: 無。

**Capture to**: proposal（新變更，獨立於 remote-login-ux-gaps 與 config-yaml-setup-skill）

**Next**: /speclink-propose --from-discussion spectra-legacy-cleanup

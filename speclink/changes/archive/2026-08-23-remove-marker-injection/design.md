## Context

marker 區塊是 init／update 寫進指令檔（CLAUDE.md、AGENTS.md、custom tool instructions_file）的靜態路由層，內容與技能 description、引擎閘門、{{SPEC_DIR}} 代換、SKILL.md 內嵌 Invocation 前言全面重複。本設計把路由職責遷移到技能與 CLI（OpenSpec 1.0 同路線），並定義遺留 marker 的剝除、版號探測的新基準與 desktop 的替代呈現。範圍界定：**in scope**＝注入拆除、剝除遷移、description 與交棒句改寫、版號探測改基準、golden 重整、desktop 調整、docs 同步；**out of scope**＝動態指示機制（status／instructions／preflight payload）、continue 式技能、marker 剝除互動確認、歷史封存文件回改。

## Goals / Non-Goals

- Goals: 指令檔零受管區塊；路由由 description（入口）＋交棒句（出口）＋CLI 狀態（狀態內）三層承載；既有專案 update 一次收斂到無 marker 狀態；版號探測與降版拒絕在無 marker 下依然可靠。
- Non-Goals: 見 proposal Non-Goals；另不改技能檔的生成位置與 frontmatter 結構（version 欄位既存，直接沿用）。

## Decisions

### D1: 注入拆除面

`crates/speclink-core/src/init.rs` 移除 `instructions_body`、`custom_instructions_body`、`upsert_marker` 與所有把 marker 寫入指令檔的路徑（fs init、remote init、update、tools 收斂、adopt 補齊）。受管生成物的新定義：技能檔（各工具 skills 目錄）、`.speclink.yaml`、`.gitignore` 的 `.speclink/` 條目——指令檔完全退出受管集合。desktop 初始化對話框與工具同步（`apps/desktop/core/src/settings.rs`）同步不再產生 marker。兩支 body 函式的唯一外部消費者是 Node SDK 的 `instructions.render()`（`crates/speclink-node/src/render.rs`）——該公開 API 一併移除（breaking change）：自建 harness 的 system prompt 不再取用集中路由文字，改與其他 host 同路，由 `skills.render()` 產出的技能檔 description 承載入口路由。remote 模式不需要 remote 措辭的替代載體：remote 語意已由技能本文（remote 模式下 contextFiles 指向 Context Projection 的指引）與 instructions payload 承載。

### D2: 遺留剝除（update 一律剝）

`speclink update` 對每個工具目標（內建 claude／codex 與所有 custom 描述子）檢查其指令檔：存在 `SPECLINK:START..END` 區塊即以既有 `remove_marker` 語意剝除——只動區塊與其分隔空行、保留使用者內容、剝除後全空的檔案刪除。無區塊時不觸碰檔案（位元級不變）。剝除不需確認、不需旗標，屬 update 的常規收斂行為；stdout 摘要列出剝除的檔案。init 於全新專案本來就不產生指令檔，無剝除需求；對既有 marker 的專案 re-init（--force）同樣走剝除。

### D3: instructions_file 欄位棄用

`crates/speclink-core/src/config.rs` 的 custom 描述子：`instructions_file` 由必填（require_field）轉為選填且不生效——解析容忍該欄位存在（舊 .speclink.yaml 不炸），但引擎不再讀它生成任何東西；`speclink update`（工具同步面）對仍帶該欄位的描述子輸出一行棄用提示（非錯誤、走 stderr）。描述子的剝除目標：欄位仍在時，update 剝除該檔案的 marker 區塊（D2）；欄位已移除時無從得知舊檔案位置，不剝（文件載明此邊界）。

### D4: 入口路由——description 觸發情境句

`crates/speclink-core/src/skills.rs` registry() 的 18 個對外技能 description 改寫為「情境 → 用我」句式，每句涵蓋原 marker 路由表對應 bullet 的觸發情境，英文撰寫（description 屬 agent 面）。句式規範：先觸發情境（Use when...），後一句話說明產出；改寫後長度上限比照現有最長者（quality）。內部技能（tdd、clarify、sync）無 SKILL.md 生成，不改。逐句措辭於 skill-routing spec 的 delta 中以需求形式規範句式與涵蓋面，不在 spec 逐字釘 18 句（措辭是資產內容，由 golden 鎖定）。

### D5: 出口路由——交棒句邊集

各技能資產結尾補（或改寫既有的）「Next steps」段，狀態相依、只建議、絕不代跑（明文 guidance only）。邊集（源自討論第 5 輪）：

| 技能 | 出邊 |
|---|---|
| onboard | specs 生成完：需求清楚→propose；還模糊→discuss |
| discuss | 依結論：promote／propose --from-discussion；併入既有 change→link 後 ingest；不做→discuss archive；無實質→discard（既有內文已載，僅補齊格式） |
| improve | 記錄成討論後同 discuss |
| propose | artifacts 齊→apply；平行多 change→apply-with-worktree（保留既有 NEVER invoke apply 條款，交棒句為建議句） |
| apply | 全勾→品質站（review ∥ verify 或 quality）或 archive；剩 [M]→品質站可先跑、archive 待手動；中途需求變→ingest（既有內文已有雛形，統一格式） |
| apply-with-worktree | commit 完→品質站（worktree 內）→worktree-merge |
| worktree-merge | 合併清理完→回主 checkout archive |
| drift | delta 假設過期→ingest；無漂移→apply（保留既有「不自動呼叫」條款） |
| ingest | 更新完→回 apply；linked discussion 來源→seal 後同前 |
| review | 落章→另一站（若要）或 archive |
| verify | 落章→archive |
| quality | 每輪停；兩站落章→archive（worktree 內→worktree-merge） |
| archive | 終點，無出邊 |
| commit、analyze、audit、config、trace | 工具技能：無固定出邊（analyze 發現缺口建議 ingest 屬既有內文） |

### D6: 版號探測與降版拒絕改基準

過期探測（`instruction_status`／`differing_files`）與 `refuse_downgrade` 的版本來源由「指令檔 marker 標記版號」改為「該工具 skills 目錄下技能檔 frontmatter 的 `version` 欄位」（任一技能檔即可代表產物層版號；讀取失敗視同無版號、不阻擋）。`MARKER_VERSION` 常數更名為 `ASSET_VERSION`（技能產物層版號），語意不變：bump 時機、golden 與 assets.lock 三連動、engine_version 測試面全部沿用。desktop 的過期提示（`apps/desktop/core/src/project.rs`）同步改讀技能版本，UI 措辭把「指令檔過期」改為「技能檔過期」。

### D7: golden 面重整

`remote-claude.marker.md` 刪除（唯一獨立 marker golden）；多檔渲染快照（claude.snapshot.md、claude-worktree.snapshot.md、codex.snapshot.md、neutral-cli.snapshot.md、neutral-tool-call.snapshot.md）於再生後不再含 CLAUDE.md／AGENTS.md／instructions_file 段；`render_golden.rs` 移除 marker 專屬測項。assets.lock 隨資產改寫再生。本 change 的資產改動走一次 ASSET_VERSION bump（minor 位：行為面改變——注入拆除）。

### D8: delta 宣告紀律

remote-connection 的「remote marker 措辭」需求整項移除，delta SHALL 以 REMOVED 明示宣告；workspace-tools 內多個需求的 marker 子句以 MODIFIED 整塊取代時，凡 scenario 改名或刪除 SHALL 附 REMOVED-SCENARIO 註解（引擎不抓未宣告刪除，漏宣告到 archive 才炸）。

## Risks / Trade-offs

- **入口路由依賴 host 載入 description**：不主動載入技能清單的 host 失去路由表——接受此風險：主流工具（Claude Code、Codex 系）皆載入，OpenSpec 同一假設已實證；custom harness 由其 skills_dir 自理。
- **描述子移除 instructions_file 後的舊檔殘留**（D3 邊界）：機率低、後果小（一塊死文字），文件載明手動刪除即可。
- **交棒句與引擎狀態不同步**：交棒句是建議不是閘門，錯誤建議會被引擎拒絕（如 worktree 內 archive），不會造成損壞。

## Migration Plan

單向：使用者升級引擎後跑 `speclink update`，技能再生＋marker 剝除一次完成；不提供反向重注入。回滾＝退回舊版引擎再跑 update（舊引擎會重寫 marker）。

## Open Questions

（無——討論 init-marker-openspec-alignment 已裁定所有方向性問題。）

---
topic: wire→core 型別轉接的落點——收進 speclink-remote 供 CLI 與 desktop 共用是否成立（improve-cli-command-layer 候選 4 回訪）
slug: improve-wire-convert-seam
status: concluded
created: 2026-08-10
created_by: MomoChen <momochenisme@gmail.com>
kind: improve
---

# Discussion: wire→core 型別轉接的落點——收進 speclink-remote 供 CLI 與 desktop 共用是否成立（improve-cli-command-layer 候選 4 回訪）

<!--
Document rules:
- Rounds are appended by `speclink discuss add-round`; never rewrite an earlier round.
  A changed position gets a new round that names what changed and why.
- Each round distills one focus question: **Focus** / **Position** / **Ruled out** / **Open**.
- The conclusion must resolve or explicitly defer every open question left by the rounds.
-->

## Context

improve-cli-command-layer（2026-08-07 掃描，已隨候選 1 封存）Deferred 清單的最後一項回訪。候選 1～3 已依序落地封存：cli-render-unification（08-08）、cli-mode-dispatch-convergence（08-10）、cli-verb-family-modules（08-10）。候選 4 當時標 speculative，理由明寫「desktop 端能省多少取決於它 fs／remote 兩路的內部形狀，尚未驗證——若 desktop 的目標型別本來就不是 core 型別，此候選只剩 CLI 端小勝」。本討論的第一件事就是補這個驗證，且因候選 1～3 已重排 CLI 結構（remote_commands.rs 已不存在、轉接散進動詞族模組），原候選引用的行號與檔案全數重錨。

範圍：wire→core 型別轉接的落點——crates/speclink-cli/src/verbs/ 內的 to_* 轉接、crates/speclink-remote/src/convert.rs 灘頭堡、apps/desktop/src-tauri/src/remote.rs 與 apps/desktop/core 的型別消費面。

排除項（決策記錄比對）：remote-verb-parity（07-30）已裁定的 C 類明文分歧不重議；cli-render-unification 討論輪已裁定「轉接搬進 speclink-remote 是候選 4 的事、本刀不動」（Round 2 介面深度檢查第 2 項），本討論即是那筆帳。進行中 change 與討論皆為零，無撞車。

## Rounds

<!-- `### Round N — <mode> (<date>)` entries are appended here by the CLI. -->

### Round 1 — scan (2026-08-10)

**Focus**: 候選 4 的前提在候選 1～3 落地後的程式碼裡是否還成立
**Position**: 不成立——原候選的核心前提「desktop 另養一套 wire→core 轉接」查證為偽，且共用轉接在重疊面上結構性不可行。候選解消，值得留下的是把既有落點規則明文化。查證事實如下：

**現況盤點**：
- CLI 側：8 支 wire→core 轉接散在 5 個動詞族模組——verbs/instructions.rs（to_apply_instructions:141、to_artifact_instructions:173）、verbs/query.rs（to_list_change_json:322、to_status_report:358）、verbs/lifecycle.rs（to_archive_outcome:278）、verbs/station.rs（to_station_ticket:721、to_station_round:745）、verbs/discuss.rs（to_discussion_info:456）。皆為檔內私有 fn，單一消費端由建構保證。
- 灘頭堡已存在：crates/speclink-remote/src/convert.rs（51 行，remote-verb-parity 時建立）收 validate/analyze 兩支轉接，CLI（verbs/checks.rs:265、273）與 desktop（remote.rs:1459、1469）都在用——「兩個消費端要同一 core 型別」的轉接已經共用了。
- desktop 側：零自養 wire→core 轉接（grep 全 src-tauri 與 core crate 確認）。remote 路徑把 protocol 型別直接當工作型別（BoardSnapshot 持 Vec<ChangeSummary>，remote.rs:848；change_stage 直讀 wire 欄位，remote.rs:834），最後 serde 直接序列化給前端。

**前提崩落的三個事實**：

(a) **無重複可殺**。原候選寫「desktop 的 remote.rs 直接吃 protocol 型別另養一套」，暗示與 CLI 的 to_* 平行重複。實況：desktop 的目標型別不是 core 型別——fs 路徑（apps/desktop/core/src/query.rs:14 list_changes_at）從 store 組前端 JSON payload 再疊桌面欄位（whyExcerpt、created、review 狀態），remote 路徑直接序列化 wire。兩路都不經過「wire→core」這一步。真正重疊處（validate/analyze，兩端都要 core 型別餵各自下游）已在 convert.rs 共用。

(b) **重疊面上共用轉接結構性不可行**。CLI 的轉接是消費端政策、不是中性欄位對拷：to_list_change_json 刻意丟掉 server extras（repo/lifecycle/claimedBy/startedAt），worktree 欄恆 None 附 spec 引註（query.rs:331–333「remote 恆缺席」）；to_status_report 註解明言「server extras simply don't cross this boundary」。而 desktop 的看板邏輯恰恰吃那些被 CLI 丟掉的欄位（change_stage 讀 started_at、排序 overlay 讀 lifecycle）。同一支轉接無法同時服務兩端——兩邊要保留的欄位集方向相反。

(c) **單一消費端搬家＝搬移非集中**。8 支 to_* 各只有一個消費端，且落點正是候選 3 剛建立的 locality——一族一檔，檔內含參數、雙臂、轉接、渲染，動詞的完整故事在一處。搬進 speclink-remote 是把故事再拆散、換來「共用之名單用之實」的假共層。原候選的 Wins「wire 欄位演進只付一次轉接成本」在單消費端下本來就只付一次。

**介面深度四項檢查**（無條件過站）：
1. 接縫位置：正確的接縫已經存在——convert.rs 收「兩個消費端都要同一 core 型別」的轉接，消費端專屬政策留在消費端。現況即正解。
2. 轉接數量：每動詞單層轉接、無疊 wrapper。搬家不減少轉接數，只改變地址。
3. 深度：更深的版本（client 直接回 core 型別、把整個 protocol 層藏進 speclink-remote）被事實 (b) 堵死——desktop 需要 wire 欄位，client 無法統一回 core 型別；硬做會逼出兩套 client 介面，更淺。
4. 刪除測試：假想把 8 支搬進 convert.rs 再刪掉——複雜度原封不動回到各動詞檔，行為不變、動詞故事反而更完整。搬家不集中任何複雜度，測試不過。

**掃描中順帶觀察（本討論範圍外）**：desktop 的 fs/remote 兩路前端 payload 形狀不同——fs 版疊 whyExcerpt/created/createdBy/review 狀態等 store 衍生欄位，remote 版是裸 wire ChangeSummary（缺上述欄位、多 server extras）。前端如何吸收未追。這是 desktop 範圍的另一次掃描方向，與原討論排除項「desktop 熱點另計」同一筆。

**Ruled out**: （暫無——本輪為查證輪，裁定待使用者）
**Open**: 結論方向——(i) no change：候選 4 以「前提查證為偽」收案，落點規則（轉接在出現第二個要同一 core 型別的消費端時才進 convert.rs）寫進結論即決策記錄；(ii) 同 (i) 另開超小 change 把落點規則補進 convert.rs doc comment（現有註解已隱含此意，只差一句明文）；(iii) 使用者不同意解消、想烤問某個事實

## Conclusion

**Decision**: no change——候選 4（wire→core 型別轉接收進 speclink-remote 供 CLI 與 desktop 共用）解消，不落任何 change。落點規則以本結論為決策記錄：一支轉接在出現第二個要同一 core 型別的消費端時才搬進 speclink-remote::convert（validate/analyze 即先例）；消費端專屬的欄位取捨政策永遠留在消費端。
**Rationale**: 候選成立的前提查證為偽。(a) desktop 零自養 wire→core 轉接——它的目標型別是前端 payload 不是 core 型別，fs 路徑從 store 組 JSON、remote 路徑直接序列化 wire，兩路都不經過 wire→core，無重複可殺；(b) 兩端唯一真正重疊處（validate/analyze）已於 remote-verb-parity 收進 convert.rs 共用，候選想做的事在它成立的範圍內已完成；(c) CLI 剩餘 8 支 to_* 各為單一消費端且承載 CLI 輸出政策（刻意丟 server extras、worktree 恆 None 附 spec 引註），desktop 看板恰恰消費那些被丟棄的欄位（change_stage 讀 started_at、排序讀 lifecycle），同一支轉接無法服務兩端；單消費端搬家是搬移非集中，刪除測試不過，且拆散 cli-verb-family-modules 剛建立的「一族一檔含參數、雙臂、轉接、渲染」locality。介面深度四項檢查（Round 1）四項同向：現況即正確接縫。
**Rejected alternatives**: 原候選全量版（8 支 to_* 搬進 speclink-remote）——假共層，理由同 Rationale (c)；更深版本（client 直接回 core 型別、把 protocol 層整個藏進 speclink-remote）——desktop 需要 CLI 刻意丟棄的 wire 欄位，統一回 core 型別結構性不可行，硬做逼出兩套 client 介面反而更淺；另開超小 change 補 convert.rs doc comment——現有檔頭註解已隱含共用條件，一句明文不值一個 change 的流程成本，規則以本結論為記錄（使用者裁定）。
**Deferred**: desktop fs/remote 兩路前端 payload 形狀分歧（fs 版疊 whyExcerpt／created／review 狀態等 store 衍生欄位，remote 版為裸 wire ChangeSummary——缺上述欄位、多 server extras；前端如何吸收未追）——desktop 範圍，留給未來以 desktop 為向的 improve 掃描。

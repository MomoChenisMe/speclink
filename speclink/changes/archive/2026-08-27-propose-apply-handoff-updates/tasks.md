## 1. 技能資產內文

- [x] 1.1 propose 收尾加盤點環節：在 crates/speclink-core/assets/skills/propose.md 的 Next steps 段前加指示——提案中（未開工）變更 ≥2 時以 speclink list --json 盤點並判定執行順序（硬信號：delta capability 重疊→須依序；軟信號：讀 proposal 與 tasks 推測程式碼重疊或依賴）；有效 worktree 政策（含 SPECLINK_WORKTREE 覆寫）開啟時分「可平行——各開一個 session 走 apply-with-worktree（多 session 配方）」與「須依序」兩組，關閉時給單一建議順序；僅 1 個提案中變更時維持現行出邊。落實 propose-skill delta 的 requirement『收尾盤點提案中變更的執行順序』。完成判準：內文含觸發條件（≥2）、硬軟信號、兩組分流與「僅建議、不自動呼叫任何技能」字句。 <!-- speclink-task:tsk_01M10VRBKXDNK0E8654WEDZFM0 -->
- [x] 1.2 apply 出邊補跳過品質站路徑：crates/speclink-core/assets/skills/apply.md 的 Next steps「全部勾完」邊補明示——直接走 /speclink:archive，或走 /speclink:commit 的 Archive first, then commit together 一步到位；Output On Completion 模板同步措辭。落實 skill-routing delta 的 requirement『出口交棒由技能結尾承載』的 apply 交棒句與品質站交棒句（後者由任務 1.3 完成字面）。完成判準：Next steps 與完工模板皆含直接封存路徑；僅剩 [M] 的邊字面不變。 <!-- speclink-task:tsk_01M10VRBKXG8DY7FWTZV91TH58 -->
- [x] 1.3 review 與 verify 出邊改兩條：crates/speclink-core/assets/skills/review.md 與 crates/speclink-core/assets/skills/verify.md 的落章邊改為「主 checkout 落章→/speclink:archive；worktree 內落章→先提交蓋章寫入的 meta 異動→/speclink:worktree-merge」，移除「the other station if the user wants it」交叉提醒。完成判準：兩檔皆無交叉提醒字句、皆含 worktree 分支；findings 未修完的既有邊不動。 <!-- speclink-task:tsk_01M10VRBKX6S68DKW8AY24G514 -->
- [x] 1.4 archive 尾端加收尾提交提醒：crates/speclink-core/assets/skills/archive.md 結尾新增一段——封存完成後提醒使用者提交本次封存產生的異動（/speclink:commit 或一般提交皆可），明文僅提醒、不代跑。落實 archive-skill delta 的 requirement『封存完成後的收尾提交提醒』。完成判準：結尾段含提交提醒與「僅提醒」字句。 <!-- speclink-task:tsk_01M10VRBKX2FGH0ZBN4APGDA6D -->
- [x] 1.5 discuss 結論邊改單推 propose 入口：crates/speclink-core/assets/skills/discuss.md 的 Next steps「結論值得自己開變更」邊改為僅建議 /speclink:propose --from-discussion，移除該邊並列的 promote；中途轉出段的 promote 教學與其餘出邊（link＋ingest、archive、discard）不動。落實 discuss-skill delta 的 requirement『結論後交棒單推 propose 入口』。完成判準：結論邊僅含 propose 入口，promote 僅出現於中途轉出段。 <!-- speclink-task:tsk_01M10VRBKX8E4ZN0V4YJJGGJTB -->

## 2. 版號與 golden 同步

- [x] 2.1 遞增產物版號：crates/speclink-core/src/init.rs 的 ASSET_VERSION 由 v1.21.0 改為 v1.22.0。完成判準：常數值更新，註解紀律（僅 render 內容變動時遞增）成立於本批變更。 <!-- speclink-task:tsk_01M10VRBKX9N96FRX8HR4SX7RD -->
- [x] 2.2 刻意再生 golden 與鎖檔：以 UPDATE_GOLDEN=1 與 UPDATE_ASSETS_LOCK=1 執行 cargo test -p speclink-core --test it render_golden::，審閱 crates/speclink-core/tests/golden/ 下五份 snapshot 與 assets.lock 的 diff，確認只反映任務 1.1–1.5 的字面變更與版號。完成判準：不帶環境變數重跑 cargo test -p speclink-core --test it 全綠。 <!-- speclink-task:tsk_01M10VRBKX32RW7B6RBVG16KK7 -->
- [x] 2.3 再生本 repo 的工具技能檔：以本批建置的 CLI（cargo run -p speclink-cli -- update）執行 update，再生 claude 與 codex 目標的技能檔。完成判準：.claude/skills/ 下受影響技能（propose、apply、apply-with-worktree、review、verify、archive、discuss）內容與資產一致、frontmatter 版號為 v1.22.0；quality 技能檔僅 frontmatter 版號行變動（資產內文未變，版號 stamp 隨全體遞增）。 <!-- speclink-task:tsk_01M10VRBKX6JFJJGAMKNBFVA59 -->

## 3. 詞彙與收尾

- [x] 3.1 詞彙微調：openspec/LANGUAGE.md「轉為變更」詞條定義改為不限定已結論的討論——promote 主場為中途轉出（單項談定即可轉出），並於 why 補記本次路由裁定（結論後單推 propose --from-discussion；propose-apply-handoff-updates，2026-08-27）。完成判準：詞條定義與新路由一致。 <!-- speclink-task:tsk_01M10VRBKXJRHRN496W1MD31VK -->
- [x] 3.2 收尾盤點：以 git status 對照 proposal 的 Impact 清單，確認資產、init.rs、golden 五份與 assets.lock、LANGUAGE.md、再生技能檔皆入列且無漏檔；重跑 cargo test -p speclink-core --test it 確認全綠。完成判準：git status 與 Impact 清單一致，測試綠。 <!-- speclink-task:tsk_01M10VRBKXB7QYVEYNSE2TQQFG -->

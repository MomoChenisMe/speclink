## 1. core：link 流程函式（TDD）

- [x] 1.1 於 crates/speclink-core/src/discuss.rs 的 #[cfg(test)] 模組撰寫 link 測試群（對應規格「討論以 link 動詞併入既有變更」）：成功鑄鏈後變更 meta 含 from_discussion 且討論 frontmatter 為 status: promoted＋promoted_to 含變更名；四守衛（討論不存在／已封存／變更不存在／變更已連結其他討論）各自回錯且兩側檔案逐位元不變；同組合重跑冪等成功不改檔；promoted_to 既有值逗號累加不覆蓋；變更 meta 無尾換行時容錯。驗證：cargo test -p speclink-core --lib 新測試全數失敗（紅）
- [x] 1.2 於 crates/speclink-core/src/discuss.rs 實作 link 流程函式：守衛全過才寫、變更側先寫（read_change_meta 追加 from_discussion 行後 write_change_meta）、討論側呼叫既有 mark_promoted（design D1–D4）。驗證：cargo test -p speclink-core --lib 1.1 測試群全綠且既有測試無退化
- [x] 1.3 重構 link 與 promote 的共用段（僅在出現重複時抽 helper，無重複則記錄不動）。驗證：cargo test -p speclink-core --lib 維持全綠

## 2. cli：discuss link 子指令

- [x] 2.1 於 crates/speclink-cli/src/main.rs 新增 discuss 族 Link 子指令（位置參數 slug、change，旗標僅 --json）並於 crates/speclink-cli/src/commands.rs 接上 core：成功時 exit 0、stdout 單行成功訊息含兩名稱（--no-color 無 ANSI）；--json 輸出 slug 與 change 欄位（camelCase）；守衛失敗非零 exit、stderr 單句原因。驗證：cargo build 後對臨時測試 repo 執行成功案例與任一守衛案例，斷言 exit code、stderr 與 --json payload 欄位
- [x] 2.2 確認既有指令輸出逐位元不變：跑 scratchpad 的 parity／color 回歸對照；若對照涵蓋 discuss --help 清單（會多 link 一行）則屬刻意更新並記錄於對照基線。驗證：對照套件通過或差異僅限 discuss --help 新增行

## 3. 技能指示三處同步與 golden（規格「技能指示引導 ingest 型結論先鑄鏈」）

- [x] 3.1 落實規格需求「技能指示引導 ingest 型結論先鑄鏈」——修改內嵌技能資產：crates/speclink-core/assets/skills/discuss.md 的 conclude 步驟加入「Capture to 指向既有變更時，先執行 speclink discuss link <slug> <change> 再導向 /speclink-ingest <change>」指引；crates/speclink-core/assets/skills/ingest.md 加入「更新源自討論結論時，確認來源討論已以 speclink discuss link 連結目標變更」提示。驗證：speclink init 於臨時目錄生成的 claude 與 codex 技能檔內容含上述指示文句
- [x] 3.2 同步 repo 技能實例：.claude/skills/speclink-discuss/SKILL.md、.agents/skills/speclink-discuss/SKILL.md、.claude/skills/speclink-ingest/SKILL.md、.agents/skills/speclink-ingest/SKILL.md 反映與 assets 相同的指示。驗證：逐檔內容審視，指示段與 assets 對應段一致
- [ ] 3.3 全部改動提交後，於乾淨工作樹執行 UPDATE_GOLDEN=1 cargo test -p speclink-core --test render_golden 再生基準並審視 diff 僅含新指示文字。驗證：不帶 UPDATE_GOLDEN 重跑 cargo test -p speclink-core --test render_golden 全綠

## 4. 詞彙修訂與端對端驗證

- [ ] 4.1 修訂 openspec/LANGUAGE.md「已轉出變更」詞條：definition 自「至少轉出過一個變更」放寬為「至少連結一個變更（轉出或併入）」，why 註記 link 動詞為併入路徑。驗證：speclink language show 輸出呈現新定義
- [ ] 4.2 端對端手動驗證斷鏈已補：於臨時測試 repo 走 discuss new → conclude → speclink discuss link → 對目標變更執行 archive，斷言討論記錄自動移入 openspec/discussions/archive/ 且 speclink discuss list --json 不再列出該討論為 live。驗證：檔案位置與 --json 清單斷言

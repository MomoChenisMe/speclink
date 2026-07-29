## 1. 引擎值域驗證（共用 seam）

- [ ] 1.1 撰寫 core 值域驗證測試：crates/speclink-core/src/config.rs 的 `#[cfg(test)]` 新增——locale 接受 tw／ja／en、拒絕「繁體中文」與 TW（大小寫敏感）；spec_locale 另接受 auto、拒絕 zh-Hant；None（未設定）恆合法；update_workflow_config_text 收非法 fields 回 Err（訊息含欄位名、收到值、合法集合）且合法 fields 輸出與現行為逐字元一致。預期：新測試紅、既有測試不受影響。 <!-- speclink-task:tsk_01KYQ01CQD783NWXGXPC7MVXBD -->
- [ ] 1.2 實作 speclink-core 驗證函式（合法集合常數與 pub 驗證入口，依 design「D1：值域驗證落在 speclink-core 的 update_workflow_config_text」與「D3：合法集合凍結為 locale_display 的 frozen mapping」置於 crates/speclink-core/src/config.rs），update_workflow_config_text 組字前呼叫。驗證：cargo test -p speclink-core 全綠。 <!-- speclink-task:tsk_01KYQ01CQDCRM47TTKW1ZJ5ZD8 -->

## 2. CLI set 動詞拒絕非法代碼

- [ ] 2.1 撰寫 CLI 整合測試（對應規格需求「workflow-config set 政策欄位寫入」）：crates/speclink-cli/tests/workflow_config.rs 新增——workflow-config set locale 繁體中文 → exit 非 0、stderr 含 tw／ja／en、config.yaml 逐位元不變；set spec_locale 繁體中文 --dry-run → exit 非 0、stdout 無 diff；set locale TW → 拒絕；set spec_locale auto 與 set locale 空字串（移除鍵）維持成功。預期：新測試紅。 <!-- speclink-task:tsk_01KYQ01CQDCHWJW5HK24P4ZPY2 -->
- [ ] 2.2 接通 CLI 錯誤路徑：crates/speclink-cli/src/commands.rs 讓 seam 的驗證 Err 以既有非零 exit code＋stderr 慣例冒出（不在 CLI 層重複驗證、不產生第二份措辭）。驗證：cargo test -p speclink-cli 全綠，且 speclink workflow-config set locale ja --dry-run 仍正常印 diff。 <!-- speclink-task:tsk_01KYQ01CQDWMSJ2SVQRGWWPCAJ -->

## 3. server 最終防線

- [ ] 3.1 撰寫 server 整合測試（對應規格需求「policy 寫入一律 CAS 且經引擎驗證」）：crates/speclink-server/tests/policy_write.rs 新增——正確 expected revision、可解析但 locale 為「繁體中文」的內容 → 寫入被拒、store 文件內容與 revision 皆不變、錯誤 message 含欄位名與合法集合、不新增 reason 種類；locale 為 tw 的內容 → 成功回新 revision。預期：新測試紅。 <!-- speclink-task:tsk_01KYQ01CQDFNPATSP85X78BJC3 -->
- [ ] 3.2 實作 server 端點值域驗證：crates/speclink-server/src/routes.rs 的 policy 寫入在既有 WorkflowConfig::from_text 檢查後呼叫 speclink-core 同一驗證函式，非法即沿用 invalid_config 錯誤家族拒絕（design「D2：server 端點擴充引擎 fail-closed 驗證至 locale 值域」）。驗證：cargo test -p speclink-server 全綠。 <!-- speclink-task:tsk_01KYQ01CQDSPWRMWJ28Q6EC0NC -->

## 4. 桌面未知值顯性呈現

- [ ] 4.1 撰寫桌面測試（對應規格需求「設定頁政策下拉的未知值顯性呈現」）：apps/desktop/src/__tests__/projectSettingsView.test.tsx 新增——儲存值 locale=「繁體中文」時渲染出該原始值、無效標註與引導提示文字；儲存值 tw 或未設定時無任何無效標註（jsdom 渲染斷言，不模擬下拉開合；以 Node 20 執行）。預期：新測試紅。 <!-- speclink-task:tsk_01KYQ01CQD5PEMP9FX42APVKM0 -->
- [ ] 4.2 實作未知值呈現：apps/desktop/src/views/ProjectSettingsView.tsx 兩個 Select 對選項集外的非空值動態插入帶警示樣式的項目與欄位提示（design「D4：桌面未知值顯性呈現」，local config.yaml 簽與遠端 Workflow 簽同一元件路徑共用）；apps/desktop/src/i18n/messages.ts 新增 zh-TW 與 en 字串。驗證：npm test -w apps/desktop 全綠。 <!-- speclink-task:tsk_01KYQ01CQDYAWVGWN53BQR1K4H -->

## 5. 技能指引與文件

- [ ] 5.1 更新技能三處實例（對應規格需求「技能規定政策語系欄位寫入代碼」；design「D5：技能文件三處同步與 golden 再生」）：先確認 git status 僅含本變更 diff（golden 再生的乾淨樹前置），再於 crates/speclink-core/assets/skills/config.md 政策欄位段落加入——locale 僅接受 tw／ja／en、spec_locale 僅接受 tw／ja／en／auto、自然語言回答映射為代碼（示例「繁體中文」→ tw）、禁止寫入顯示名稱；.claude/skills/speclink-config/SKILL.md 與 .agents/skills/speclink-config/SKILL.md 同步同一內容。 <!-- speclink-task:tsk_01KYQ01CQDPSCVTVQBZ86YWDF5 -->
- [ ] 5.2 再生 render golden：UPDATE_GOLDEN=1 cargo test -p speclink-core --test render_golden 更新 crates/speclink-core/tests/golden 四份 snapshot，審視 diff 僅含 config.md 新增指引段落。驗證：無 UPDATE_GOLDEN 重跑 render_golden 通過。 <!-- speclink-task:tsk_01KYQ01CQD8HMG2700ZYG3XC6H -->
- [ ] 5.3 更新使用文件：docs/configuration.md 與 docs/configuration.zh-TW.md 的 locale／spec_locale 說明補合法代碼集合與「顯示名稱會被拒絕」。驗證：內容與 spec 的合法集合一致。 <!-- speclink-task:tsk_01KYQ01CQDRHATJ5DQ6WSH7H5Q -->

## 6. 端到端收斂

- [ ] 6.1 全量回歸：cargo test --workspace 與 npm test -w apps/desktop（Node 20）全綠，含 golden 與 CLI baseline 既有測試。 <!-- speclink-task:tsk_01KYQ01CQDX61H80M8VQ7T5S5K -->
- [ ] 6.2 實資料驗收並修復既有髒值：對含髒值的 wad-old-web 專案開桌面設定頁確認 locale 下拉顯示「繁體中文」與無效標註 → 改選 tw（spec_locale 同）儲存 → 於 wad-old-web checkout 執行 speclink workflow-config show --json，斷言 payload 的 locale 與 specLocale（camelCase 欄位）值為 tw；再以 speclink workflow-config set locale 繁體中文 驗證 remote 模式被 server 拒絕且 exit 非 0。 <!-- speclink-task:tsk_01KYQ01CQDBPMW8WKXFSDDA6FQ -->

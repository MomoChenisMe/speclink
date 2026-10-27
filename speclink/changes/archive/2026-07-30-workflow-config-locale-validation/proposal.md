## Problem

一次 /speclink-config 技能執行把政策欄位的值寫成了顯示名稱字串——remote store（wad-old-web 專案）的 locale 與 spec_locale 存入「繁體中文」而非語系代碼 tw。後果分兩層：

- 桌面設定頁的 locale／spec_locale 下拉選單只有固定代碼選項（tw／ja／en，spec_locale 另有 auto），儲存值對不上任何選項時 Radix Select 靜默渲染空白——值存在但使用者看不見，也無任何錯誤提示。
- 引擎注入 instructions 時未知 locale 值原樣回傳（locale_display 的 echo-back），「繁體中文」四字碰巧可被 AI 理解，產出語言未立即出錯，反而掩蓋了資料已髒的事實。

## Root Cause

寫入鏈上沒有任何一層驗證 locale 值域，而顯示層又靜默吞掉未知值：

- CLI 的 set 動詞值解析（crates/speclink-cli/src/commands.rs 的 set_policy_field）對 tdd／audit 以 policy_bool 嚴格驗證（連 yes／1 都拒收），locale／spec_locale 卻是任意字串照單全收。
- 所有官方寫入路徑共用的改寫 seam（crates/speclink-core/src/config.rs 的 update_workflow_config_text）只在 YAML 不可解析時 fail-closed，不檢查 locale 值域。
- server 的 policy 寫入端點（crates/speclink-server/src/routes.rs）的引擎 fail-closed 驗證同樣只涵蓋可解析性。
- 桌面設定頁（apps/desktop/src/views/ProjectSettingsView.tsx）的 Select 對不在選項集的值沒有 fallback 顯示。
- config 技能文件（crates/speclink-core/assets/skills/config.md）只列出政策欄位名稱，未言明值必須是語系代碼，執行技能的 AI 便照使用者口語答案字面寫入。

## Proposed Solution

寫入嚴格、讀取寬容，三個面向各補一層：

1. **引擎值域驗證（共用 seam）**：speclink-core 新增 locale 代碼驗證——locale 限 tw／ja／en，spec_locale 限 tw／ja／en／auto，空值（移除鍵）仍合法——於 update_workflow_config_text 落實。CLI 本地與遠端、桌面本地與遠端的寫入全部經過此函式，一處落實全數覆蓋；拒絕時錯誤訊息列出合法代碼。
2. **server 最終防線**：policy 寫入端點的引擎 fail-closed 驗證擴充至 locale 值域，非法值不落盤——沿用 server-policy-write 既有原則（client 端驗證僅為 UX，server 為防線），使未升級或非官方 client 也寫不進髒值。
3. **讀取路徑維持寬容**：WorkflowConfig::from_text 與 locale_display 的 echo-back 行為不變——既有髒值仍可讀可顯示，不因升級而讓專案讀取炸掉；防呆只擋新的寫入。
4. **桌面顯示防呆**：儲存值不在選項集時，下拉不再靜默空白——顯示原始值並標註無效、附提示文字引導重選合法代碼；選定合法代碼儲存即覆蓋髒值（也是既有髒資料的修復路徑）。
5. **技能提醒**：config 技能的政策欄位段落明文規定 locale／spec_locale 只接受語系代碼、列出合法集合、要求把使用者的自然語言回答（如「繁體中文」）映射為代碼後寫入、禁止寫顯示名稱。內嵌資產與 repo 技能實例同步更新，render golden 再生。

## Non-Goals

- 不改動讀取端的寬容語意：手動編輯 config.yaml 寫入其他 BCP-47 代碼仍會被原樣注入 instructions（逃生口保留），驗證只作用於官方寫入動詞與端點。
- 不做既有髒資料的自動遷移或掃描——wad-old-web 的髒值由使用者經桌面下拉（或 CLI set）重寫一次即修復，屬操作行為非程式碼。
- 不擴充合法語系集合（如新增其他語言選項）——維持 locale_display 既有的 frozen mapping 三語系。
- 不改動環境變數層（SPECLINK_LOCALE 等）的解析行為。

## Success Criteria

- 執行 speclink workflow-config set locale 繁體中文 → 非零 exit code，stderr 列出合法代碼（tw／ja／en），config 檔逐位元不變；spec_locale 同理（合法集合另含 auto）；set locale tw 與 set locale 空字串（移除鍵）維持成功。
- 對 server policy 寫入端點提交 locale 為非法值的 config 內容 → 拒絕且 store 內容不變；合法代碼寫入成功。
- 桌面開啟儲存值為「繁體中文」的專案設定頁 → locale 下拉顯示該原始值並標註無效、附引導文字；改選 tw 儲存後 store 值為 tw、下拉正常顯示。
- 渲染後的 speclink-config 技能文件含語系代碼指引段落；render golden 測試通過。
- 既有回歸：workflow-config 動詞測試、server policy_write 測試、桌面 projectSettingsView 測試全數通過。

## Impact

- Affected specs: workflow-config（modified）、server-policy-write（modified）、desktop-config（modified）、config-skill（modified）
- Affected code:
  - Modified: crates/speclink-core/src/config.rs、crates/speclink-cli/src/commands.rs、crates/speclink-server/src/routes.rs、apps/desktop/src/views/ProjectSettingsView.tsx、apps/desktop/src/i18n/messages.ts、crates/speclink-core/assets/skills/config.md、.claude/skills/speclink-config/SKILL.md、.agents/skills/speclink-config/SKILL.md、crates/speclink-core/tests/golden/claude.snapshot.md、crates/speclink-core/tests/golden/codex.snapshot.md、crates/speclink-core/tests/golden/neutral-cli.snapshot.md、crates/speclink-core/tests/golden/neutral-tool-call.snapshot.md、docs/configuration.md、docs/configuration.zh-TW.md
  - 測試（Modified）: crates/speclink-cli/tests/workflow_config.rs、crates/speclink-server/tests/policy_write.rs、apps/desktop/src/__tests__/projectSettingsView.test.tsx、crates/speclink-core/src/config.rs 內的單元測試
  - New: 無
  - Removed: 無

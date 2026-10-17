## MODIFIED Requirements

### Requirement: policy 寫入一律 CAS 且經引擎驗證
<!-- BEFORE: policy 寫入已驗證 YAML、locale 與 spec_locale，尚未驗證安全政策值域。 -->

policy 寫入端點 SHALL 要求完整文件原文與 expected revision——缺 expected revision SHALL 拒收，SHALL NOT 存在無條件覆寫路徑。寫入 SHALL 依序：role 檢查（非 editor 即 403 且不觸文件）→ 引擎 WorkflowConfig fail-closed 驗證（解析失敗回 invalid_config 且不落盤）→ expected revision 的 CAS 提交（不符回 revision_conflict 且內容不變；成功回新 revision）。引擎驗證 SHALL 併涵蓋政策值域：內容可解析但 locale 非 tw／ja／en、或 spec_locale 非 tw／ja／en／auto（大小寫敏感；鍵不存在即未設定，恆合法）時，SHALL 拒絕且不落盤，錯誤 message SHALL 指出欄位名與合法代碼集合；SHALL 沿用既有錯誤 envelope 結構與 reason 種類，SHALL NOT 為值域驗證新增 wire 形狀。政策由 server 守住——client 端任何驗證僅為 UX，SHALL NOT 被信任為防線。

#### Scenario: revision 過期寫入被拒且無副作用

- **WHEN** 以過期的 expected revision 提交合法的 config 內容
- **THEN** 回應 revision_conflict，store 中文件內容與 revision 皆未改變

#### Scenario: 壞 YAML 不落盤

- **WHEN** 以正確的 expected revision 提交無法解析的 YAML
- **THEN** 回應 invalid_config 並指出解析錯誤，store 中文件未改變

#### Scenario: 非法 locale 值不落盤

- **WHEN** 以正確的 expected revision 提交可解析、但 locale 值為「繁體中文」的 config 內容
- **THEN** 寫入被拒，錯誤 message 指出 locale 欄位與合法代碼 tw、ja、en；store 中文件內容與 revision 皆未改變

policy 寫入 SHALL 另驗證 security_verification 的已知欄位型別、scope／版本／ASVS等級，缺欄位視為disabled，原有未知欄位保留；資料是否已在裝置安裝不作server政策寫入前提。錯誤 SHALL 沿用 invalid_config envelope 並指出欄位，不新增無條件覆寫。舊client未編輯安全欄位的文字改寫 SHALL 保留該區塊。

#### Scenario: 非法安全政策不落盤

- **WHEN** editor以正確revision提交ASVS level=L4或啟用Top10 version=2030
- **THEN** invalid_config指出安全欄位，文件及revision不變；reader仍先受403拒絕

#### Scenario: 沒有安裝資料仍可儲存政策

- **WHEN** editor提交合法Top10 2025安全政策，而server未安裝標準資料
- **THEN** CAS成功保存政策並前進revision，不在server下載或掃描，執行時由裝置另判資料缺失

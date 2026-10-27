## MODIFIED Requirements

### Requirement: 核准頁 session 保護且明確確認
<!-- BEFORE: 未登入一律導向登入頁，啟用頁只接受手動輸入 user code，未保留 Desktop 啟用 URL 的上下文。 -->

核准頁 SHALL 要求已登入的 session。未登入瀏覽器以格式合格的 `user_code` 查詢參數開啟核准頁時，登入流程 SHALL 僅保留該裝置碼，成功登入後 SHALL 返回同一核准頁並預填該碼；缺少或格式不合的查詢值 SHALL NOT 被傳遞或反映。已登入瀏覽器以格式合格的 `user_code` 開啟核准頁時，頁面 SHALL 預填該碼，但 GET SHALL NOT 查詢授權狀態、變更授權狀態或略過下一步。使用者 SHALL 提交 user code 並得到明確的核准／拒絕確認步驟；核准或拒絕 SHALL 記錄操作者身分。核准頁的變更型 POST SHALL 沿用同源驗證。未知、已用或逾期的 user code SHALL 得到同一無效回應，SHALL NOT 區分原因。

#### Scenario: 未登入不能核准

- **WHEN** 未登入的瀏覽器直接開啟核准頁並嘗試提交 user code
- **THEN** 被導向登入頁；該授權請求維持未核准

#### Scenario: Desktop 啟用上下文經登入保留

- **WHEN** 未登入的瀏覽器開啟 `/activate?user_code=ABCD-EFGH`，使用者成功登入
- **THEN** 瀏覽器返回 `/activate?user_code=ABCD-EFGH`，裝置碼欄位預填 `ABCD-EFGH`，且授權請求仍為 pending

#### Scenario: 預填後仍須明確確認

- **WHEN** 已登入使用者開啟帶有 pending user code 的核准頁
- **THEN** 頁面只預填裝置碼並提供下一步；使用者提交下一步後才看到核准與拒絕選項

#### Scenario: 缺少或格式不合的短碼不反映

- **WHEN** 使用者直接開啟無 `user_code` 的核准頁，或以不合 `XXXX-XXXX` 格式的值開啟核准頁
- **THEN** 已登入瀏覽器看到空白裝置碼欄位；未登入瀏覽器的登入頁與登入後 Location 均不含該值

#### Scenario: GET 不洩漏 user code 狀態

- **WHEN** 已登入瀏覽器分別以格式合格但不存在、已使用及已逾期的 user code 開啟核准頁
- **THEN** 三者得到相同的預填輸入頁，且沒有任何授權記錄被變更

#### Scenario: 無效 user code 不洩漏狀態

- **WHEN** 於核准頁分別輸入不存在的、已核准過的、已逾期的 user code
- **THEN** 三者得到相同的無效回應文字

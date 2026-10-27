## MODIFIED Requirements

### Requirement: binding 與認證前置 fail closed

所有路由 SHALL 前置認證與 binding 裁決：bearer 憑證缺失或無效 SHALL 回 401 permission_denied——憑證查驗對 identity 儲存逐請求進行（hash 命中、未撤銷、未過期、所屬 user 為 active），SHALL NOT 存在組態檔靜態 token 表的認證路徑；actor 非 URL project 的 member SHALL 回 403 permission_denied。project key 未註冊 SHALL 回 404 not_found；X-Speclink-Repo 標頭指向未註冊 repo SHALL 回 not_found；缺標頭且該 project 註冊多個 repo SHALL 拒絕並於 message 指出候選需明示，SHALL NOT 自動選擇；恰一個 repo 時 SHALL 綁定之。X-Speclink-Api-Version 與 server 不相容 SHALL 拒絕並帶版本原因。前置任一步失敗 SHALL NOT 執行動詞。/binding SHALL 回 actor、project、repo、apiVersion、engineVersion 與 capabilities——宣告 polling 端點與 etag 支援，並宣告 sse push transport（事件端點 url 與 resume 支援），宣告 SHALL 與實際服務的端點一致。

#### Scenario: 未知 token 拒於門外

- **WHEN** 以 identity 儲存中不存在的 token 呼叫任一查詢路由
- **THEN** 回 401 且 reason 為 permission_denied；server 未執行任何 engine 動詞

#### Scenario: repo 多義拒絕不代選

- **WHEN** 對註冊兩個 repo 的 project 不帶 X-Speclink-Repo 呼叫 /binding
- **THEN** 回拒絕且 message 指出需明示 repo；SHALL NOT 回任一候選的成功 binding

#### Scenario: 有效 PAT 完成 binding

- **WHEN** 以 /account 建立的有效 PAT 對 actor 具 membership 的 project 呼叫 /binding
- **THEN** 回成功 binding，actor 為該 PAT 所屬 user 的身分

#### Scenario: capabilities 宣告含 sse 與 polling

- **WHEN** 完成相容的 /binding handshake
- **THEN** capabilities 的 events 宣告同時含 sse transport（resume 為 true）與既有 polling 宣告

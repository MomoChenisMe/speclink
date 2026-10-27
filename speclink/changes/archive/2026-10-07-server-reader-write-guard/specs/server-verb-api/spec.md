## ADDED Requirements

### Requirement: 寫入端點一律要求 editor 角色

server SHALL 對專案範圍內每一個寫入請求（HTTP 方法不是 GET 或 HEAD）先檢查呼叫者的 membership role：reader SHALL 收 HTTP 403、錯誤封套的 reason 為 `permission_denied`，處理函式 SHALL NOT 執行、scope revision SHALL NOT 改變、SHALL NOT 產生領域事件；editor 放行，行為與本需求生效前相同。此檢查 SHALL 落在單一位置，並 SHALL 對之後新增的專案範圍寫入端點自動生效（預設擋下）。以 POST 傳輸的唯讀查詢 `POST /context` SHALL 排除在外，reader 可呼叫。未通過認證或非成員的請求 SHALL 維持既有的錯誤回應，不因本需求改變。

#### Scenario: reader 建立變更被拒

- **WHEN** 以 reader role 憑證呼叫 POST /changes，payload 為合法的變更名稱 `add-auth`
- **THEN** 回應為 HTTP 403、reason 為 `permission_denied`；之後以 editor 呼叫 GET /changes 的清單不含 `add-auth`，scope revision 與請求前相同

#### Scenario: CLI 顯示角色原因

- **WHEN** reader 在已 link 到 project `demo` 的資料夾執行 `speclink new change add-auth`
- **THEN** exit code 非 0，stderr 為 `Error: access denied — your role in project 'demo' is reader; this action needs the editor role; ask a project admin`；非成員執行同一指令時，stderr 為 `Error: access denied — actor is not a member of project 'demo'; ask a project admin`

#### Scenario: reader 勾任務被拒

- **WHEN** 以 reader role 憑證呼叫 POST /changes/{name}/tasks/1/done
- **THEN** 回應為 HTTP 403、reason 為 `permission_denied`；該變更的 tasks.md 內容與請求前逐字相同

#### Scenario: reader 寫入討論被拒

- **WHEN** 以 reader role 憑證呼叫 POST /discussions/{slug}/rounds，內容為合法的一輪
- **THEN** 回應為 HTTP 403、reason 為 `permission_denied`；該討論記錄與請求前逐字相同

#### Scenario: reader 取文件快照照常

- **WHEN** 以 reader role 憑證呼叫 POST /context
- **THEN** 回應成功，內容與以 editor 呼叫同一請求相同

#### Scenario: editor 寫入不受影響

- **WHEN** 以 editor role 憑證呼叫 POST /changes 建立 `add-auth`，再呼叫 POST /changes/add-auth/tasks/1/done
- **THEN** 兩個請求都成功，結果與本需求生效前相同

#### Scenario: 未認證的寫入維持既有錯誤

- **WHEN** 不帶 bearer token 呼叫 POST /changes
- **THEN** 回應與本需求生效前相同（HTTP 401、reason 為 `permission_denied`），不回 403

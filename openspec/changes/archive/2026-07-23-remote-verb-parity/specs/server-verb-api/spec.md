## ADDED Requirements

### Requirement: validate 與 analyze 為唯讀衍生查詢端點

server SHALL 提供 GET /changes/{name}/validate 與 GET /changes/{name}/analyze：經 Command gateway 執行與本地相同的引擎運算（validate 固定 spec_driven schema、單 change；analyze 回完整 AnalyzeReport），回應為 typed DTO 附 scope ETag。兩端點 SHALL 對 reader 與 editor 皆可用、SHALL NOT 產生任何寫入或事件發布；change 不存在時 SHALL 回 404 與語義化訊息。同一 scope 內容下，端點回傳的驗證錯誤集合與 analyze findings SHALL 與本地 fs 模式對同一內容的結果一致。

#### Scenario: validate 結果與本地一致

- **WHEN** 對 server 上一個 proposal 缺 Why 段的 change 呼叫 GET /changes/{name}/validate
- **THEN** 回應列出與 fs 模式 speclink validate 同一 change 內容時相同的錯誤項，HTTP 200 且 scope revision 不前進

#### Scenario: reader 可執行唯讀動詞

- **WHEN** 以 reader role 的成員憑證呼叫 GET /changes/{name}/analyze
- **THEN** 回應為完整 AnalyzeReport（HTTP 200），不因 role 被拒

#### Scenario: 缺席 change 回 404

- **WHEN** 呼叫 GET /changes/no-such/validate
- **THEN** HTTP 404，body 含語義化訊息指出該 change 不存在

---
### Requirement: DELETE change 為 discard 全語意

server SHALL 提供 DELETE /changes/{name}（force 布林 query 參數、預設 false）：執行 fail-closed meta 檢查、started-work guard、來源討論 unlink、change 全部文件的原子刪除與 touched 記錄清理。force=false 且該 change 帶開工痕跡（started_at 已蓋或任一任務已勾）時 SHALL 拒絕且無任何寫入，錯誤 reason SHALL 機器可判為需要 force 的拒絕；meta 損壞時 SHALL 拒絕（含 force=true）。刪除成功 SHALL 於同一提交單元發布事件使訂閱端收到 invalidate。

#### Scenario: 未開工 change 直接刪除

- **WHEN** 對無開工痕跡的 change 呼叫 DELETE /changes/{name}（force=false）
- **THEN** 該 change 的 meta 與全部 artifacts 自 scope 消失、後續 list 不含它，SSE 訂閱端收到 invalidate

#### Scenario: 已開工 change 需要 force

- **WHEN** 對已勾選任務的 change 呼叫 DELETE /changes/{name}（force=false）
- **THEN** 回拒絕錯誤且 reason 機器可判為需要 force，scope 內容零改動；改以 force=true 重呼叫則刪除成功

#### Scenario: 刪除連帶 unlink 來源討論

- **WHEN** 刪除一個由討論 promote 而來的 change
- **THEN** 該討論的 promoted_to 清單移除此 change 名（清單空時討論狀態回復），與刪除同次操作完成

---
### Requirement: 任務搬移端點與重編號效果

server SHALL 提供 POST /changes/{name}/tasks/move（from、to 為 1-based checkbox ordinal，before 為可省略側別）：僅搬移 checkbox 行本身（群組標題與其他行不動），省略 before 時依方向推斷（向上插錨前、向下插錨後），成功後重算「數字.數字」編號前綴並一次寫回，效果與本地任務拖排逐位元一致。from/to 越界或該 change 無 tasks.md 時 SHALL 拒絕且無任何寫入。搬移成功 SHALL 發布事件使訂閱端收到 invalidate。

#### Scenario: 跨群組搬移重編號

- **WHEN** 對 tasks.md 含兩個編號群組的 change 呼叫 move 把第 1 個任務移到第 3 個任務之後
- **THEN** tasks.md 的該 checkbox 行落於錨行之後、兩群組的「數字.數字」前綴依新序重算，其餘行逐字元不變

#### Scenario: 越界拒絕零副作用

- **WHEN** 對只有 3 個任務的 change 呼叫 move（from=5）
- **THEN** 回拒絕錯誤指出索引超界，tasks.md 內容與 scope revision 皆不變

---
### Requirement: 寫入動詞 editor 限定

DELETE /changes/{name} 與 POST /changes/{name}/tasks/move SHALL 檢查呼叫者 membership role：reader SHALL 收 403 且 reason 機器可判為權限不足、scope 零改動；editor 放行。capability 描述 SHALL 對 reader 將刪除與任務拖排標示為停用，validate/analyze 對全 role 標示可用。

#### Scenario: reader 的刪除被拒

- **WHEN** 以 reader role 憑證呼叫 DELETE /changes/{name}
- **THEN** HTTP 403、reason 機器可判為權限不足，該 change 完整保留

#### Scenario: capability 依 role 呈現

- **WHEN** reader 與 editor 各自完成 handshake 取得 capability 描述
- **THEN** reader 的描述中刪除與任務拖排為停用、validate/analyze 為可用；editor 的四項皆可用

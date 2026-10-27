## MODIFIED Requirements

### Requirement: 審查流程的技能行為

技能文件 SHALL 指示主線 orchestrator 依序執行：(1) 選定 change；(2) 守門自檢，任務未全數完成即停止；(3) 呼叫 review scope 取得 frozen patch——無工單時 phase=discovery，有工單時 phase=validation；needsInput（僅發生於 discovery）時 SHALL 等待使用者提供可信 base、hash-pinned hunk selection 或隔離 worktree，不得以 touched 整檔替代；(4) 讀取 change artifacts 作判準脈絡；(5) 依 phase 執行以下分流；(6) 並列呈現結果與 remediation triage；(7) 以 review add-round 寫入相同 phase、patchHash、Scope 與 findings。

Discovery SHALL 將同一 frozen patch 平行交給 Standards（repo 慣例文件＋smell baseline，repo 文件優先）與 Correctness（bug hunting）兩個 read-only sub-agent，各以 400 字內回報並以 CRITICAL／WARNING／SUGGESTION 分級。兩軸 SHALL 只以 change hunks 與判斷直接影響所需的呼叫端、測試為審查面；兩份報告 SHALL 原樣並列，不合併、不跨軸重排。Spec compliance SHALL NOT 在審查站裁決。

Validation SHALL 只把上輪未解 findings、accepted 清單、remediation patch 與必要脈絡交給對應 axes；sub-agent SHALL 逐筆判定原 finding 已解／未解，並只回報 remediation patch 直接引入的 regression。remediation patch 內 attribution 為 "adjacent" 的段落 SHALL 由 sub-agent 逐段確認確屬本次修復——不屬於本次修復的 adjacent 段 SHALL 以 regression 回報，SHALL NOT 靜默採認。未解 finding SHALL 由主線以原文寫入新輪；已解 finding SHALL 從新輪 findings 移除；未修改區域的新 smell、SUGGESTION 或既存問題 SHALL NOT 加入。scope 回報的 outOfScopeChanged 非空時，主線 SHALL 於呈現結果時原樣轉知使用者，SHALL NOT 將其列入審查面或工單 findings。

artifacts 稀薄時 sub-agent SHALL 僅憑 code 與測試判斷，不臆造需求。locale SHALL 沿用既有「審查產出的語言綁定」契約；phase、patchHash、severity、axis prefix 與 path 保持英文 token。

#### Scenario: 任務未完成即停

- **WHEN** 對任務 3/5 的 change 執行 speclink-review
- **THEN** 技能停止並說明審查站要求任務全數完成，不呼叫 review scope、不派出 sub-agent、不寫工單

#### Scenario: 首輪只審 frozen change hunks

- **WHEN** touchedFiles 含一份 300 行檔案，而 resolved discovery patch 只含其中兩個 hunks
- **THEN** Standards 與 Correctness 都收到相同兩個 hunks及必要上下文，不把其餘未修改內容當 discovery 面

#### Scenario: 續輪只驗收上輪 findings 與 remediation patch

- **WHEN** Round 1 有兩筆未解 findings，修正 patch 只改其中一檔並新增一個呼叫端
- **THEN** validation 只判定兩筆原 finding 與該 patch 的直接 regression，不重新掃描整個 finding 檔案或 change

#### Scenario: adjacent 段須確認歸屬

- **WHEN** 驗證輪的 remediation patch 含一個 attribution "adjacent" 的段落
- **THEN** sub-agent 簡報載明須逐段確認該段確屬本次修復，不屬於者以 regression 回報

#### Scenario: 末輪零 findings 時重試蓋章而非重審

- **WHEN** 工單末輪 findings 為空但先前 stamp 因外部守門失敗而留下工單
- **THEN** 技能在守門恢復後直接重試 review stamp，不派出新的 discovery 或 validation

#### Scenario: legacy 工單缺 snapshot 時 fail closed

- **WHEN** 工單有 findings但 lastRound.patchHash 為 null，且 host 沒有可對應 snapshot
- **THEN** 技能說明無法精確重建 remediation delta，保留工單並等待使用者明示 discard 後重新 discovery，不得重審整檔

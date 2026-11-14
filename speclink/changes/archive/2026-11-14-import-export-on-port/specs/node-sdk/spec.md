## ADDED Requirements

### Requirement: 宿主的匯出與匯入方法

Node SDK 的宿主套件 SHALL 提供 `host.exportScope({ identity, scope })`，回傳 Promise，解析為第二版的包（`formatVersion`、`scopeVersion`、`documents`，每項 `key`、`content`、`digest`）；以及 `host.importScope({ identity, scope, bundle })`，回傳 Promise，解析為 `{ scopeVersion, documents }`（documents 為寫入的份數）。`bundle` SHALL 可以是第一版或第二版。兩者 SHALL 經過與 host.dispatch 相同的掛鉤，並 SHALL 於背景工作執行，SHALL NOT 阻塞 JS 事件迴圈。失敗時 SHALL 以帶 code 的 Error 拒絕：包不合格為 `invalid_argv`；目標範圍不是空的、掛鉤拒絕為 `refused`；儲存層的失敗沿用既有的三個 code。createHost SHALL 接受選填的 `limits.importBytes`（正整數）；不是正整數時 SHALL 同步拋出錯誤。host.handleRequest SHALL 回答匯入與匯出的路由，並套用同一個上限。

#### Scenario: 匯出再匯入

- **WHEN** 以 JS 的記憶體儲存建立 host，在範圍 s1 建立一個變更後 exportScope，再把結果 importScope 到範圍 s2
- **THEN** 對 s2 執行 `['list', '--json']` 得到同一個變更；s2 的 commit 被呼叫正好一次

#### Scenario: 匯入非空的範圍

- **WHEN** 對已有變更的範圍呼叫 importScope
- **THEN** Promise 以 code 為 refused 的 Error 拒絕；commit 沒有被呼叫

#### Scenario: 包不合格

- **WHEN** importScope 的 bundle 有一份文件的 digest 與內容不符
- **THEN** Promise 以 code 為 invalid_argv 的 Error 拒絕，message 含該文件的鍵

#### Scenario: 掛鉤拒絕

- **WHEN** beforeCommand 對 scope-import 回傳拒絕，呼叫 importScope
- **THEN** Promise 以 code 為 refused 的 Error 拒絕，message 是宿主的原因

#### Scenario: 上限經 handleRequest 生效

- **WHEN** 以 `limits: { importBytes: 1024 }` 建立 host，經 handleRequest 送出本文為 4096 位元組的匯入請求
- **THEN** 回應的 status 是 413

#### Scenario: 上限不是正整數

- **WHEN** 以 `limits: { importBytes: 0 }` 呼叫 createHost
- **THEN** 同步拋出錯誤

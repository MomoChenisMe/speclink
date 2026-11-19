## ADDED Requirements

### Requirement: 單筆變更的詳細輸出

`speclink show <change> --json` SHALL 接受旗標 `--detail`，不讀 stdin。帶上它時，輸出的 `review` 與 `verify` 物件在 `state` 為 `stamped` 時 SHALL 在既有欄位之外增列 `scope`：陣列，每一項是 `{ "path": 字串, "hash": 字串 }`，`path` 是蓋章時記下的 repo 相對路徑（以 `/` 分隔），`hash` 是蓋章時記下的內容指紋（小寫十六進位），順序 SHALL 與蓋章時記錄的順序相同。已蓋章但沒有記下任何指紋的關卡，`scope` SHALL 是空陣列。`scope` 的鍵名與每一項的鍵名 SHALL 與線上格式單筆變更回應的 `scope` 相同。`state` 為 `inProgress` 的關卡、以及沒有工單也沒有章而不輸出的關卡，SHALL 與不帶 `--detail` 時相同。`show` 的其他欄位 SHALL 不受 `--detail` 影響；`show` 的對象是正式規格時（`--item-type spec`，或名稱解析成正式規格），輸出 SHALL 與不帶 `--detail` 時相同。

不帶 `--detail` 的 `speclink show <change> --json` SHALL 維持既有的位元級輸出，SHALL NOT 含 `scope`。`speclink list --json --detail` SHALL 維持既有輸出，SHALL NOT 含 `scope`。不帶 `--json` 的 `--detail` SHALL 以用法錯誤拒絕：CLI 以非零 exit code 結束、訊息走 stderr 並指出 `--json`、stdout 沒有輸出；經宿主套件執行（`host.dispatch`、掛鉤的 `read`、Node 的 `engine.dispatch`）時以 `invalid_argv` 拒絕。成功時 exit code SHALL 是 0。`--detail` SHALL NOT 改變人眼輸出，因為它不能脫離 `--json` 使用；`--no-color` 對 JSON 輸出沒有作用。輸出 SHALL 不隨 `locale`（`tw`、`ja`、`en` 或未設定）改變。這個旗標 SHALL NOT 寫入任何檔案，`.meta.yaml` 與 `.speclink/` 底下的檔案 SHALL 不變。

三種執行方式對相同的資料 SHALL 輸出相同的 `review` 與 `verify`：本機專案、連到遠端的 CLI、宿主套件。連到遠端的 CLI SHALL 以既有的單筆變更回應組出 `scope`，SHALL NOT 另外發請求。連到不回報關卡欄位的舊版 Server 時，`review` 與 `verify` SHALL 不出現，指令 SHALL NOT 失敗。離線或認證失效時 SHALL 與不帶 `--detail` 的 `show` 相同地失敗。這個動詞只讀取，沒有 revision 衝突的情況。

蓋章之後檔案是否又被改過，SHALL NOT 由這個輸出判斷：`scope` 只是蓋章時記下的指紋，比對由呼叫端自己做。

#### Scenario: 已蓋章的關卡帶出指紋

- **WHEN** 變更 demo 的審查已蓋章，章記下 `src/a.rs` 與 `src/b.rs` 兩個檔的指紋；驗證有未結的工單；執行 `speclink show demo --json --detail`
- **THEN** exit code 是 0；`review` 含 `state` `stamped`、`stampedAt`、`rounds`、`tasksTotal` 與 `scope`，`scope` 依序是 `src/a.rs` 與 `src/b.rs` 兩項，各有 `path` 與 `hash`；`verify` 是 `{ "state": "inProgress" }`，沒有 `scope`

##### Example: 各種關卡記錄對應的輸出

| 關卡的記錄 | 不帶 `--detail` | 帶 `--detail` |
| --- | --- | --- |
| 沒有工單也沒有章 | 不輸出這個關卡 | 不輸出這個關卡 |
| 工單未結 | `{ "state": "inProgress" }` | `{ "state": "inProgress" }` |
| 已蓋章，記下兩個檔 | 既有欄位，沒有 `scope` | 既有欄位加 `scope`（兩項） |
| 已蓋章，沒有記下任何指紋 | 既有欄位，沒有 `scope` | 既有欄位加 `"scope": []` |

#### Scenario: 不帶 --detail 時輸出不變

- **WHEN** 對同一個已蓋章的 demo 執行 `speclink show demo --json`
- **THEN** 輸出與引入 `--detail` 之前逐位元相同，`review` 與 `verify` 都沒有 `scope`

#### Scenario: 清單的詳細輸出不帶指紋

- **WHEN** 對含已蓋章的 demo 的專案執行 `speclink list --json --detail`
- **THEN** demo 這一項的 `review` 沒有 `scope`，輸出與這個變更之前相同

#### Scenario: --detail 不帶 --json 被拒絕

- **WHEN** 執行 `speclink show demo --detail`
- **THEN** 指令以非零 exit code 結束，stderr 指出 `--detail` 要和 `--json` 一起用，stdout 沒有輸出

#### Scenario: 宿主套件以 invalid_argv 拒絕

- **WHEN** 經 `host.dispatch` 執行 `['show', 'demo', '--detail']`，以及在 `beforeCommand` 裡執行 `read(['show', 'demo', '--detail'])`
- **THEN** 兩者都以 Error 拒絕，`code` 是 `invalid_argv`，儲存沒有收到寫入

#### Scenario: 正式規格不受影響

- **WHEN** 對正式規格 auth 執行 `speclink show auth --item-type spec --json --detail`
- **THEN** 輸出與 `speclink show auth --item-type spec --json` 相同

#### Scenario: 三種執行方式輸出相同

- **WHEN** 本機專案、一台 Server 與一個宿主套件的儲存含相同的變更 demo 與相同的審查章，分別執行 `speclink show demo --json --detail`、連到該 Server 執行同一個指令、以及經 `host.dispatch` 執行同一個 argv
- **THEN** 三個結果的 `review` 深度相等，都含相同的 `scope`

#### Scenario: 連到舊版 Server 時不失敗

- **WHEN** CLI 連到一台單筆變更回應沒有關卡欄位的 Server，執行 `speclink show demo --json --detail`
- **THEN** 指令成功，輸出沒有 `review` 與 `verify`

#### Scenario: 離線時與不帶 --detail 相同地失敗

- **WHEN** CLI 連到的 Server 無法連線，執行 `speclink show demo --json --detail`
- **THEN** exit code 與 stderr 訊息與執行 `speclink show demo --json` 時相同，stdout 沒有輸出

---
title: 認識資料：變更、討論與規格
section: 開始使用
order: 40
keywords: [openspec, 變更, 討論, 規格, 生命週期, 封存, 廢棄]
sources: [change-lifecycle, discussion-docs, user-documentation]
generated: 2026-09-07T13:20:04+08:00
---

# 認識資料：變更、討論與規格

Speclink 的資料都是純文字檔，放在專案的 `openspec/` 資料夾。你不用打開 Speclink 也能讀寫它們，每一次變動都會出現在 git diff 裡。這一頁介紹資料夾長什麼樣、一個變更會經過哪些狀態、討論記錄與變更怎麼連在一起。

## openspec 資料夾

本地模式的 `openspec/` 沿用 OpenSpec 的目錄結構，內容全是 Markdown 與 YAML：

| 路徑 | 放什麼 |
| --- | --- |
| `specs/<capability>/spec.md` | 正典規格。每個 capability 一份，是「現況唯一真相」 |
| `changes/<名稱>/` | 進行中的變更。裡面有提案、設計、任務清單與 delta 規格 |
| `changes/archive/` | 已封存的變更 |
| `config.yaml` | 工作流政策與專案設定，見[工作流政策與設定](policy-config.md) |

Speclink 在這個結構上多加了兩樣東西：

- `discussions/`：討論記錄，每份一個 `.md` 檔。已封存的討論在 `discussions/archive/`。
- 每個變更資料夾裡的 `.openspec.yaml`：這個變更的狀態檔。

> [!NOTE]
> 這個相容性只適用本地模式。Remote 模式的正典存在 server 的 Store，本機只有唯讀投影。見 [remote 模式總覽](remote-overview.md)。

## 變更的生命週期

變更的狀態真相只有一個地方：它自己資料夾裡的 `.openspec.yaml`。這個檔案記錄三站的章：

| 站 | 記錄什麼 | 何時寫入 |
| --- | --- | --- |
| 建立 | 建立時間、建立者、建立工具 | 變更建立時 |
| 開工 | 開工日期、開工者 | 第一次標記開工、或第一次完成任務時 |
| 封存 | 封存時間、封存者 | 封存時 |

封存不會剝掉開工站的欄位。三站的章會一起留在封存目錄裡。

### 開工標記

有兩種方式讓變更進入「進行中」：

- 執行 `speclink in-progress add <變更名>`。
- 執行 `speclink task done` 完成任何一項任務。這會在同一個動作裡順手蓋開工章。

開工章只蓋一次。已經開工的變更再標記或再完成任務，章不會變。

> [!NOTE]
> `speclink in-progress add` 對不存在的變更名會靜默成功：沒有輸出、不寫任何檔案。這是刻意保留的既有行為。打錯名字時不會有錯誤提示。

### 退回提案中

誤開工的變更可以退回「提案中」：

```
speclink in-progress remove <變更名>
```

只有「零工作痕跡」的變更才能退回：任務清單裡沒有任何已勾的任務，而且沒有任何觸及檔案的記錄。不符合時指令拒絕，stderr 列出已勾任務數與觸及檔案清單，並說明出路：已勾任務可以取消勾選後重試；觸及記錄要由人或 agent 判斷處理。這個指令沒有強制旗標。

與 in-progress add 不同，remove 對不存在的變更名會明確報錯。未開工的變更執行 remove 會直接成功、不寫檔。

### 廢棄變更

不做了的變更用 discard 刪掉：

```
speclink discard <變更名>
speclink discard <變更名> --force
```

discard 會刪掉整個變更資料夾與它的觸及記錄檔，並解開它與討論記錄之間的鏈結（見下文）。有工作痕跡的變更不帶 `--force` 時拒絕：stderr 提示有工作痕跡與 `--force`，任何檔案都不會動。

| 已開工 | 有已勾任務 | 不帶 --force 的結果 |
| --- | --- | --- |
| 否 | 否 | 放行 |
| 是 | 否 | 拒絕 |
| 否 | 是 | 拒絕 |
| 是 | 是 | 拒絕 |

成功時 stdout 報告已刪除的變更名，以及每份解鏈討論的 slug 與回退後的狀態。Remote 模式不支援 discard，執行會以錯誤結束。

### 封存

變更完成後用封存把 delta 規格併入正典，並把變更移到 `changes/archive/`。封存前有三道守門：任務完成度、不能在 linked worktree 內執行、品質關卡的章沒有失效。細節見[封存](archive.md)。

### 待重新反映

變更曾經反映某份討論的結論，而那份討論後來又重新下了結論時，變更會被標為「待重新反映」。這時：

- `speclink show <變更名>` 會列出待重新反映的討論。
- `speclink analyze <變更名>` 會出一條資訊性的發現，提醒你重新 ingest 以同步新結論。

重新反映完成、執行 seal 後，標記會清掉。

### 認領

團隊模式的 Store 支援「認領」變更：認領後狀態檔記錄認領者與時間。同一人重複認領直接成功；別人已認領時拒絕，並指出目前持有人。本地模式不支援認領。

### 狀態檔壞掉時

`.openspec.yaml` 存在但無法解析時，所有會改狀態的動作都會拒絕，並指出檔案位置與解析原因：標記開工、認領、完成任務、取消完成、新增產物、封存、廢棄。壞掉的狀態檔不會被當成「未開工」處理，discard 帶 `--force` 也一樣拒絕。修好檔案再重試。

## 討論記錄

討論記錄放在 `openspec/discussions/<slug>.md`。slug 是檔名，也是所有討論指令的把手。每份記錄有三個區段：Context（背景）、Rounds（輪）、Conclusion（結論）。輪只能往後追加，不能改寫既有的輪。

記錄的狀態有三種：

| 狀態 | 意思 |
| --- | --- |
| open | 討論中 |
| concluded | 已寫入結論 |
| promoted | 已轉出變更：至少連結了一個變更 |

建立討論時會蓋建立者章，取自 git 身分。git 身分取不到時省略。

## 討論與變更怎麼連起來

鏈結有兩個方向：

- 變更側：變更的狀態檔記錄它來自哪些討論。一個變更可以來自多份討論。
- 討論側：討論記錄記錄它轉出了哪些變更。一份討論可以轉出多個變更。

三個指令會動這條鏈：

| 指令 | 效果 |
| --- | --- |
| `speclink discuss promote <slug>` | 中途轉出：建立新變更骨架並連結 |
| `speclink discuss link <slug> <變更名>` | 把討論連到既有變更。只寫變更側，討論記錄不動 |
| `speclink discuss seal <slug> <變更名>` | 標記討論的內容已反映到該變更：討論狀態變成 promoted |

變更封存時，它來源的討論會一起封存，條件是：該討論的結論已經寫入，而且沒有其他進行中的變更引用它。沒寫結論的討論留在原地，之後照常可以加輪與下結論。變更被 discard 時，討論側的連結會移除；連結清空後，討論狀態退回 concluded（有結論）或 open（無結論）。

操作細節見[討論：需求還模糊時](discuss.md)。

**出處**：`change-lifecycle`、`discussion-docs`、`user-documentation`

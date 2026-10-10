---
title: 基準盤點：既有專案採用 Speclink
section: SDD 工作流
order: 90
keywords: [baseline, 基準, 盤點, 既有專案, 採用, capability map, rules.specs, onboard]
sources: ["baseline-skill#盤點前取得 workflow config 並套用 specs 產出規則", "baseline-skill#基準盤點的行為邊界", "skill-routing#入口路由由技能描述承載", "skill-routing#出口交棒由技能結尾承載", "user-documentation#工作流正典逐站列出技能與完成判準", "workflow-config#workflow-config languages 只讀查詢"]
generated: 2026-10-10T08:24:25+08:00
---

# 基準盤點：既有專案採用 Speclink

專案已經有程式碼、但還沒有任何規格時，用 `/speclink-baseline` 技能替它建立第一批正式規格。這一站只做「記錄現況」：把系統今天已經有的行為寫成規格，不建變更、不改程式碼。這一站舊稱 onboard，循舊名找到這裡的讀者對上新站名即可。

## 這一站做什麼、不做什麼

技能守六條邊界：

1. **只記錄已存在的行為**。每條需求都要能追溯到實際讀過的程式碼或測試。無法驗證的推論會標記出來，或直接省略。
2. **不修改任何程式碼**。
3. **不建立變更**。正式規格直接寫進 `openspec/specs/<capability>/spec.md`，變更目錄底下不會多出任何東西。
4. **已有規格時只補缺**。技能進入補缺模式，只盤點還沒被覆蓋的行為區域，既有的規格檔一個字都不動。要改既有規格，走變更流程，見[提案：建立變更與產物](propose.md)。
5. **capability map 經你確認前，不寫任何規格**。
6. **寫完後做嚴格結構驗證**，並修正結構性發現。

一次盤點做完之後，變更目錄沒有新檔案，repo 裡除了新增的規格檔以外沒有其他異動。

## 流程

1. **讀工作流設定**：技能一開始執行 `speclink workflow-config show --json` 取得專案說明與規格產出規則，再執行 `speclink workflow-config languages --json` 確認實際使用的語言。細節見下一節。
2. **提出 capability map**：技能提出一份能力對照表（哪些行為歸哪個 capability），並等你確認。你可以合併、拆分或刪除 capability。確認前，規格目錄下沒有任何新檔。
3. **寫入規格**：技能依確認後的清單，把規格寫進 `openspec/specs/`。
4. **驗證**：技能執行 `speclink validate --specs --all --strict`。有結構性發現就修正後重跑。
5. **報告**：最後的報告列出建立的 capability（含需求數與情境數）、標記為未驗證的行為、刻意略過的區域，以及規則揭露段。

## 盤點前先讀工作流設定

技能透過兩個只讀查詢取得資料，不會自己讀取或解析設定檔：

```bash
speclink workflow-config show --json
speclink workflow-config languages --json
```

第一個查詢取得已儲存的設定：專案說明作為盤點與每份規格的背景；規格產出規則決定每份規格要遵守的內容。第二個查詢取得具體語言，技能依其中的規格語言撰寫散文。

| 規格語言設定 | 規格散文的語言 |
| --- | --- |
| 未設定 | 執行 Speclink 的作業系統語言 |
| 舊 auto | 跟隨產物語言；產物語言也未設定時採作業系統語言 |
| tw、ja、en | 分別為繁體中文、日文、英文 |

中文系統語言對應繁體中文，日文對應日文；其他語言或無法取得時使用英文。本地查詢採本機語言；remote 查詢採 server 的語言。技能不從你的電腦、聊天語言或 App 介面猜測 server 的語言。兩個查詢都不套用個人的環境變數覆寫，也不把偵測到的語言寫回設定。

結構標記與 SHALL／MUST 關鍵字仍維持英文。

**規格產出規則**：設定了規格產出規則且清單非空時，本輪產生的每一份規格都要遵守每一條規則。規則原文照套，不翻譯、不挑選適用性。沒有設定或清單為空時，規格內容規則與現行相同。

技能在兩處揭露這輪套用了哪些規則：capability map 的確認訊息，以及最後的報告。兩處文字相同。

| 規格產出規則的狀態 | 揭露段 |
| --- | --- |
| 有 N 條規則 | 首行為 `Specs rules applied this run (from rules.specs, N entries):`，其後逐條編號列出規則原文 |
| 空清單，或沒有這個鍵 | 單行 `Specs rules applied this run: none (no rules.specs configured)` |

> [!NOTE]
> 這些規則是 agent 產生內容時必須遵守的指令。`speclink validate` 只檢查結構，不會機械式驗證自由文字的規則有沒有被遵守。

**任一查詢失敗就停止**：設定檔無法解析、語言代碼無效、remote 模式離線或認證失效時，技能回報錯誤，不寫任何規格，也不退回手讀設定檔。舊 server 無法提供具體語言時，查詢會提示升級 server；技能停止，不自行猜測。

工作流設定的內容與寫法見[工作流政策與設定](policy-config.md)。

## 做完之後

技能結尾只有兩條建議，不代跑、也不列舉全部技能的總表：

| 結束狀態 | 建議下一步 |
| --- | --- |
| 需求已經清楚 | `/speclink-propose` |
| 需求還模糊 | `/speclink-discuss` |

**出處**：`baseline-skill`、`skill-routing`、`user-documentation`、`workflow-config`

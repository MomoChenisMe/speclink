---
title: 基準盤點：既有專案採用 Speclink
section: SDD 工作流
order: 90
keywords: [baseline, 基準, 盤點, 既有專案, 採用, capability map, rules.specs, onboard]
sources: [baseline-skill, skill-routing, user-documentation]
generated: 2026-09-02
---

# 基準盤點：既有專案採用 Speclink

專案已經有程式碼、但還沒有任何規格時，用 `/speclink-baseline` 技能替它建立第一批正典規格。這一站只做「記錄現況」：把系統今天已經有的行為寫成規格，不建變更、不改程式碼。這一站舊稱 onboard，循舊名找到這裡的讀者對上新站名即可。

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

1. **讀工作流設定**：技能一開始就執行 `speclink workflow-config show --json`，取得專案說明、規格語言與規格產出規則。細節見下一節。
2. **提出 capability map**：技能提出一份能力對照表（哪些行為歸哪個 capability），並等你確認。你可以合併、拆分或刪除 capability。確認前，規格目錄下沒有任何新檔。
3. **寫入規格**：技能依確認後的清單，把規格寫進 `openspec/specs/`。
4. **驗證**：技能執行 `speclink validate --specs --all --strict`。有結構性發現就修正後重跑。
5. **報告**：最後的報告列出建立的 capability（含需求數與情境數）、標記為未驗證的行為、刻意略過的區域，以及規則揭露段。

## 盤點前先讀工作流設定

技能只透過 `speclink workflow-config show --json` 讀設定。它不會自己去讀 `openspec/config.yaml`，也不會自己解析 YAML。從結果讀三個欄位：

| 欄位 | 用途 |
| --- | --- |
| 專案說明（context） | 盤點與每份規格的背景 |
| 規格語言（specLocale） | 決定規格散文用什麼語言寫 |
| 規格產出規則（rules.specs） | 本輪產生的每份規格都要遵守的規則清單 |

**規格語言**：

| specLocale 的值 | 規格散文的語言 |
| --- | --- |
| 未設定 | 英文 |
| auto | 跟隨同一份設定的介面語言（locale） |
| tw、ja、en | 該語言 |

三種情況下，結構標記與 SHALL／MUST 關鍵字都維持英文。

**規格產出規則**：設定了 rules.specs 且清單非空時，本輪產生的每一份規格都要遵守每一條規則。規則原文照套，不翻譯、不挑選適用性。沒有設定或清單為空時，規格內容規則與現行相同。

技能在兩處揭露這輪套用了哪些規則：capability map 的確認訊息，以及最後的報告。兩處文字相同。

| rules.specs 的狀態 | 揭露段 |
| --- | --- |
| 有 N 條規則 | 首行為 `Specs rules applied this run (from rules.specs, N entries):`，其後逐條編號列出規則原文 |
| 空清單，或沒有這個鍵 | 單行 `Specs rules applied this run: none (no rules.specs configured)` |

> [!NOTE]
> 這些規則是 agent 產生內容時必須遵守的指令。`speclink validate` 只檢查結構，不會機械式驗證自由文字的規則有沒有被遵守。

**設定讀不到就停止**：`speclink workflow-config show --json` 失敗時（`openspec/config.yaml` 無法解析、remote 模式離線或認證失效），技能回報錯誤並停止。它不寫任何規格，也不會退回去手讀設定檔。remote 模式與本地模式的設定內容形狀相同，技能對兩種模式沒有差別規定。

工作流設定的內容與寫法見[工作流政策與設定](policy-config.md)。

## 做完之後

技能結尾只有兩條建議，不代跑、也不列舉全部技能的總表：

| 結束狀態 | 建議下一步 |
| --- | --- |
| 需求已經清楚 | `/speclink-propose` |
| 需求還模糊 | `/speclink-discuss` |

**出處**：`baseline-skill`、`skill-routing`、`user-documentation`

# 架構說明

> English version: [architecture.md](architecture.md)

Speclink 是一套規格驅動開發(SDD)引擎。它最核心的架構承諾是:**引擎不知道規格文件怎麼存放**。文件今天可以是 git 儲存庫裡的 Markdown,明天可以放在團隊系統的 REST API 後面——引擎的流程邏輯完全不變。

## 三層架構

```
┌────────────────────────────────────────────────────────┐
│  呈現／宿主層                                           │
│  speclink-cli — 參數解析、輸出渲染、色彩,               │
│  以及挑選儲存實作的組裝點                                │
└──────────────┬─────────────────────────────────────────┘
               │  以 &dyn Store 呼叫引擎流程
┌──────────────▼─────────────────────────────────────────┐
│  引擎層                                                 │
│  speclink-core — SDD 流程邏輯:change、artifact、       │
│  validate／analyze／drift／archive、任務、討論、        │
│  schema、instructions                                   │
└──────────────┬─────────────────────────────────────────┘
               │  Store 介面(儲存縫線)
┌──────────────▼─────────────────────────────────────────┐
│  儲存層                                                 │
│  speclink-fs — 預設實作:本地檔案系統上的                │
│  經典 openspec/ 目錄佈局                                │
└────────────────────────────────────────────────────────┘
```

- **speclink-core**(引擎)持有全部流程規則:change 何時算完成、delta 如何合併進正典規格、drift 的意義、討論如何收斂。它不以 `std::fs` 直接觸碰規格目錄;架構檢查測試(`crates/speclink-core/tests/no_direct_fs.rs`)強制此事。
- **speclink-fs**(儲存)持有全部佈局知識:`specs/<cap>/spec.md`、`changes/<name>/`、`changes/archive/<date>-<name>/`、`discussions/<slug>.md`、`config.yaml`、mtime 推導的排序、歸檔命名。要接上不同的儲存後端,替換的就是這一層。
- **speclink-cli**(宿主)是兩者唯一的交會點:每個指令建立 `FsStore`,以 `&dyn Store` 傳入 core 流程。

## 儲存縫線:`Store`

`speclink_core::store::Store` 是同步、object-safe 的介面,語彙是 SDD 領域而非檔案系統:

- change — 列舉/查找/建立/存在檢查/`updated_at_secs`
- artifact — 讀/寫/存在(以 schema 輸出路徑識別,如 `proposal.md`、`specs/<cap>/spec.md`)
- delta 與正典規格 — 能力列舉、讀/寫
- archive — 依日期名搬移 change、戳記其 metadata
- discussion — 建立/讀取/附加/歸檔,含防撞名的歸檔命名
- workflow config — 原始文件讀取(fs 語境下即 `openspec/config.yaml`)

兩類資料刻意**留在縫線之外**:

- **宿主工作資料**(`speclink_core::workspace::Workspace`):`.speclink/` 工作目錄(touched 記錄、歸檔快照)、`.speclink.yaml` 應用設定、專案根 walk-up 探索。這些屬於執行引擎的機器——就算換成遠端儲存後端,它們依然留在本地。
- **git 互動**:drift 的提交視窗分析與 archive 的 `@trace` 蒐集,關注的是*程式碼*儲存庫,與規格文件存放在哪無關,屬引擎流程。

## 行為保證

這道縫線以「純重構」方式切出:所有 CLI 指令的人眼輸出、`--json` payload、exit code 與檔案系統效果,均與重構前位元級一致(以雙沙盒回歸對照驗證,涵蓋 parity、色彩、drift/archive 情境)。

## 下一步

這道縫線是三個後續 change 的地基:

```
store-trait-and-fs-adapter(本篇)
        │
        ├─► config-system-rework
        │     工作流層級設定搬家至儲存側 config.yaml;
        │     宿主側 .speclink.yaml 保留 bootstrap 鍵;
        │     tools 改為自述式描述子
        │
        ├─► verb-contract-and-remote-client
        │     實作講 REST 動詞契約的遠端 Store(PAT 認證、
        │     樂觀鎖)——PO/PM 工具不需本地 repo,
        │     RD/QA 留在 git
        │
        └─► node-sdk
              以 napi 綁定發佈 @speclink/engine:createEngine
              可注入 JavaScript 的 Store 物件——object-safe
              的縫線正是動態注入得以成立的前提
```

上述各 change 落地時,會各自在本文件增補對應章節。

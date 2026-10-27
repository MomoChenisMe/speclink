## Context

packages/ui 已有 shadcn 原語（Button、Input、Select、Tabs、Sheet、Badge、AlertDialog 等）且大部分表單場景已採用；殘留的原生控制項為任務清單與初始化對話框的 checkbox（原生 input＋accent-color，Windows 呈 OS chrome）、設定頁兩處裸 textarea、以及約 26 處手寫樣式的 raw button（分佈於 packages/ui 五個元件與 apps/desktop 三個檔案）。現有 ui 勾選原語是「原生 input 風格化」版且無人使用；@radix-ui/react-checkbox 未安裝。本刀純前端，Rust 層零接觸。相關者：桌面 app 全體使用者。

## Goals / Non-Goals

**Goals**
- 勾選控制項主題化：非 OS 原生外觀、主色勾選態、深淺主題一致、無障礙語意保留。
- 多行輸入主題化：設定頁 textarea 取用 token 化樣式。
- 按鈕 focus／disabled 態統一：活路徑 raw button 收斂到 ui 按鈕變體。

**Non-Goals**
- GFM 唯讀 checkbox（markdown 渲染產物）；死元件（ChangeBoard／ChangeList／ChangeListItem／DocumentTree／DocumentViewer）；視覺重設計；任務互動行為變更。

## Decisions

### D1：Checkbox 升級為 Radix 原語

ui 勾選原語重寫為 @radix-ui/react-checkbox 的 shadcn 實作：Root 為 button 元素（checkbox 角色、aria-checked、空白鍵切換原生支援）、Indicator 內放勾選圖示；樣式主題化——未勾為 border-input 空框、勾選為主色底＋前景色勾、disabled 半透明。API 由 onChange 改 onCheckedChange，消費端（任務清單列、初始化對話框工具多選）轉接時 aria-label、checked、disabled 條件逐一保留。jsdom 可測性：role=checkbox 查詢＋aria-checked 斷言取代 HTMLInputElement.checked（原原語註解的「原生可測」理由由此取代）。
替代案：維持原生 input＋accent-color——Windows OS chrome 不吃主題，正是本刀動機，否決；appearance-none 自繪 CSS——重造 Radix 已解決的鍵盤與 aria 語意，否決。

### D2：Textarea 為樣式化原生、無新依賴

多行輸入沒有 OS chrome 繪製問題（僅需 border／bg／focus token 化），shadcn 的 Textarea 本就是樣式化原生 textarea——新建原語直接採此形，不引入 Radix。設定頁兩處（專案說明、產出規則）換用，onChange 與受控值不變。
替代案：contentEditable 或編輯器元件——遠超需求（純多行純文字），否決。

### D3：按鈕收斂到既有變體、視覺近似不重設計

活路徑 raw button 改用 ui 按鈕原語：icon 鈕（複製名稱、全螢幕、關分頁、拖曳把手等）用 ghost＋icon 尺寸、文字動作鈕用 ghost／outline＋sm，原 className 中與變體重複的樣式移除、佈局類（間距、對齊）保留。行為契約：onClick／disabled／aria-label／role 逐一不變，測試的 getByRole("button", { name }) 查詢全數照舊。dnd 把手的 {...attributes} {...listeners} 展開順序保留在元件 props 之後。範圍以 App 實際渲染路徑為準：TaskList、RichDetailDrawer、ChangeCard、DiscussionColumn、ArchivedList、App、ProjectTabs、SettingsView。
替代案：全 repo 含死元件一併換——對無人掛載的舊清單 UI 花工，違反外科手術原則，否決；只換表單控制項不動按鈕——使用者已明示按鈕一併統一，否決。

### D4：GFM 唯讀 checkbox 維持 CSS

markdown 內容中的任務清單 checkbox 由 react-markdown 渲染（disabled、純呈現、非表單控制項），維持 desktop-reading-experience 刀的 CSS 樣式（懸掛縮排＋accent 色）；以元件 override 替換屬渲染層改造、收益低，排除。

## Implementation Contract

**可觀察行為**
1. 任務分頁與初始化對話框的勾選框：未勾選呈主題邊框空框、勾選呈主色底＋勾選圖示，Windows 深淺主題下外觀一致、非 OS 原生繪製；每個勾選框曝露 checkbox 角色與既有標籤（「任務 N」／工具名），滑鼠點擊與空白鍵皆可切換；readOnly／封存檢視 disabled 照舊。
2. 勾選行為不退化：樂觀更新即時翻轉、批次操作 disabled 例外、拖曳把手拖放照舊——desktop-task-interactions 的全部測試不改語意（僅查詢方式由 input 改 role）仍綠。
3. 設定頁兩處多行輸入帶主題化樣式（border-input、bg 與 focus ring 取自 token），輸入與儲存行為不變。
4. 活路徑上不再有 raw button 元素以手寫樣式呈現動作鈕；鍵盤 Tab 聚焦任一動作鈕顯示一致的 focus 可視環；全部既有 aria 名稱與回呼不變。
5. npm run build -w apps/desktop 產物中上述控制項照常運作（真實視窗驗證）。

**驗收目標**
- packages/ui 測試：任務勾選框 role=checkbox＋aria-checked 斷言、空白鍵切換、readOnly disabled；工具列與拖放既有案例全綠。npm test -w packages/ui 全綠。
- apps/desktop 測試：初始化對話框工具多選 role=checkbox 斷言、設定頁 textarea 主題 class 斷言；npm test -w apps/desktop 全綠。
- 真實視窗：勾選框主題外觀（勾／未勾、深淺主題）、設定頁輸入、Tab focus 環巡檢截圖。

**範圍邊界**
- In scope：packages/ui 的 ui/checkbox、新 ui/textarea、index 匯出、TaskList、RichDetailDrawer、ChangeCard、DiscussionColumn、ArchivedList；apps/desktop 的 App、ProjectTabs、SettingsView；相應測試。
- Out of scope：Rust 全部、markdown 渲染層、死元件、視覺重設計、行為變更。

## Risks / Trade-offs

- [Radix Checkbox 改變 DOM 形狀（input→button）連動測試與樣式選擇器] → 測試斷言集中改為 role／aria-checked；grep 全 repo 確認無 input[type=checkbox] 選擇器指向任務清單（GFM 的 .markdown 選擇器不受影響）。
- [按鈕變體替換造成視覺回歸] → 逐檔替換後跑 jsdom 全套＋真實視窗截圖對照；變體選擇以「近似現狀」為準，發現落差以 className 補齊。
- [dnd 把手換 Button 後拖曳失效] → listeners 展開保留、activationConstraint 不動；拖放既有測試＋真實視窗實拖驗證。
- [回歸對照] → CLI 零接觸，parity／color 不受影響。

## Migration Plan

無資料遷移。純前端元件替換，隨一般建置發佈；回滾即還原 commit 重建。

## Open Questions

（無——範圍已由使用者裁決：表單控制項＋按鈕全換、死元件不動）

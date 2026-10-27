## Summary

讓桌面 app 的 Sonner toast 使用 Speclink 既有語意色彩、表面與字體尺度，使失敗回饋在淺色與深色模式下都與主介面一致。

## Motivation

目前 toast 只以 `theme="system"` 跟隨作業系統明暗模式，實際表面、邊框、文字、失敗色與圓角仍採 Sonner 內建灰階；因此它雖然功能正確，視覺上卻像外掛元件，且 `toast.error` 未沿用主介面的 destructive 語意。目標使用者是透過 AI 代理跑 SDD 的開發者、PO 與 PM；問題會出現在桌面 app 的日常 propose、apply 與 archive 操作失敗時。

## Proposed Solution

- 將 toast 的一般表面、文字、邊框、圓角、陰影與字體對齊 Speclink 現有設計 tokens，不再由 Sonner 內建白／黑灰階決定。
- 將失敗 toast 的視覺語意接到既有 destructive token；保留中性的 card 表面，以 destructive 圖示與邊框／文字強調失敗，避免整張高飽和紅底破壞主介面的克制風格。
- 淺色與深色模式共用同一組語意 token 映射，由既有 `prefers-color-scheme` token 值自然切換，不建立第二套 toast 色票。
- 以元件測試釘住 token class／style 映射，並以 release app 人眼確認 toast 在抽屜遮罩上方時的淺色與深色觀感。

## Non-Goals

- 不變更 toast 的訊息文字、主詞格式、6 秒逾時、關閉鈕、單槽取代、成功靜默或浮層層級。
- 不重設整個桌面 app 色票，不新增 toast 專用品牌色，也不引入新的主題或 CSS-in-JS 套件。
- 不改動其他回饋面、確認對話框、抽屜或看板卡片的視覺。
- 不改動 speclink-core、speclink-cli 或任何 Rust crate；CLI 子指令、旗標、stdin、exit code、人眼輸出與 `--json` 均不變，既有回歸對照不受影響。
- 不新增或修改 `.speclink.yaml`、`openspec/config.yaml` 設定欄位；不改動 claude／codex 技能或注入區塊。

## Alternatives Considered

- 啟用 Sonner 內建 rich colors：仍使用 Sonner 自有 HSL 紅色與表面，無法與 Speclink 的 OKLCH destructive token 及 card surface 對齊。
- 僅保留 `theme="system"`：只能切換明暗，無法解決色票、邊框、字體與圓角不一致。
- 為 toast 另建一套 raw hex／HSL 色票：會形成第二個色彩系統，且淺色、深色需各自維護。

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `desktop-app`: 補充「看板全域操作成功靜默、失敗以 toast 浮層呈現」需求，要求 toast 使用桌面 app 的語意設計 tokens，並在淺色與深色模式維持一致的表面與失敗語意。

## Impact

- Affected specs: `desktop-app`
- Affected code:
  - Modified:
    - `packages/ui/src/components/ui/sonner.tsx`
    - `packages/ui/src/__tests__/sonner.test.tsx`
  - New: (none)
  - Removed: (none)
- Dependencies: 不新增、升級或移除依賴。
- Compatibility: 僅 toast 視覺改變；行為、訊息內容、前端公開匯出與 CLI／core 契約不變，既有使用者無需遷移。

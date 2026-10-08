import { Check, Copy } from "lucide-react";

import { useI18n } from "../i18n";
import { cn } from "../lib/utils";
import { Button } from "./ui/button";
import { useCopied } from "./useCopied";

/** hover 才顯現、複製成功期間保持可見——卡片標題旁的複製鈕共用這組 class。 */
export const REVEAL_ON_HOVER =
  "opacity-0 transition-opacity group-hover:opacity-100 data-[copied]:opacity-100";

export interface CopyButtonProps {
  value: string;
  /** 無障礙名稱與 hover 提示。 */
  label: string;
  /** icon：只有圖示（列尾、標題旁）；sm：圖示＋「複製／已複製」文字的外框小鈕。 */
  size?: "sm" | "icon";
  /** 覆寫寫入剪貼簿的路徑（例如系統匣走 Tauri clipboard）；拋錯或 reject 視為失敗。 */
  onCopy?: (value: string) => void | Promise<void>;
  /** 退出 tab 順序用（系統匣面板）。 */
  tabIndex?: number;
  /** 放在 hover 時整列主色反白的列裡（系統匣面板）：hover 時圖示改用主色前景色。 */
  inverted?: boolean;
  className?: string;
}

/**
 * 複製鈕（spec desktop-app「共用元件唯一來源」；design D8）：點擊不冒泡（不觸發列本體
 * 的開啟）；寫入成功後呈主色勾號 1.5 秒並以 status 宣告，失敗靜默、不呈勾號。成功期間
 * 帶 `data-copied`，hover 才顯現的呼叫端以 `data-[copied]:opacity-100` 保持可見。
 */
export function CopyButton({ value, label, size = "icon", onCopy, tabIndex, inverted, className }: CopyButtonProps) {
  const { t } = useI18n();
  const [copied, markCopied] = useCopied();
  const copy = (e: React.MouseEvent) => {
    e.stopPropagation();
    let result: void | Promise<void>;
    try {
      result = onCopy ? onCopy(value) : navigator.clipboard.writeText(value);
    } catch {
      return;
    }
    if (result instanceof Promise) result.then(markCopied, () => {});
    else markCopied();
  };
  const icon = copied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />;
  return (
    <>
      <Button
        type="button"
        variant={size === "sm" ? "outline" : "subtle"}
        size={size === "sm" ? "sm" : "icon-xs"}
        aria-label={label}
        title={label}
        tabIndex={tabIndex}
        data-copied={copied ? "" : undefined}
        className={cn(
          "shrink-0",
          size === "sm" ? "gap-1.5" : "h-5 w-5",
          copied && "text-primary",
          inverted && "group-hover:text-primary-foreground",
          className,
        )}
        onClick={copy}
      >
        {icon}
        {size === "sm" && (copied ? t("common.copied") : t("common.copy"))}
      </Button>
      <span role="status" aria-live="polite" className="sr-only">
        {copied ? t("common.copied") : ""}
      </span>
    </>
  );
}

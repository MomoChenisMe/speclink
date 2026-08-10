import { Check, Copy } from "lucide-react";

import { Button } from "./ui/button";
import { HighlightText } from "./HighlightText";
import { useCopied } from "./useCopied";

/**
 * 看板全尺寸卡的識別列名稱＋複製鈕（spec「看板卡片統一解剖學」：標題恆單行、
 * 複製鈕同列尾隨）：複製鈕緊跟名稱最後一個字元後，不以 flex 推至卡片右緣。
 * 名稱過長時就地截斷、以省略號收尾（與全系統其餘截斷同一收尾），複製鈕收在
 * 同一列不落次行。變更卡與討論卡共用（骨架統一）。
 */
export function CardNameRow({
  text,
  copyLabel,
  highlight,
}: {
  text: string;
  copyLabel: string;
  highlight?: string;
}) {
  const [copied, markCopied] = useCopied();
  const copy = (e: React.MouseEvent) => {
    e.stopPropagation();
    void navigator.clipboard?.writeText(text);
    markCopied();
  };
  return (
    <span className="flex min-w-0 flex-1 items-center gap-1">
      <span
        data-name
        className="font-mono font-semibold text-sm leading-tight min-w-0 truncate"
      >
        <HighlightText text={text} query={highlight} />
      </span>
      <Button
        type="button"
        variant="ghost"
        size="icon"
        aria-label={copyLabel}
        className={`inline-flex h-4 w-4 shrink-0 text-muted-foreground hover:text-foreground transition-opacity ${copied ? "opacity-100" : "opacity-0 group-hover:opacity-100"}`}
        onClick={copy}
      >
        {copied ? <Check className="h-3 w-3 text-primary" /> : <Copy className="h-3 w-3" />}
      </Button>
    </span>
  );
}

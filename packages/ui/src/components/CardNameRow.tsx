import { useEffect, useRef, useState } from "react";
import { Check, Copy } from "lucide-react";

import { Button } from "./ui/button";
import { HighlightText } from "./HighlightText";

/** 名稱截斷處的漸層淡出（取代硬切）：末段 2rem 由不透明漸到全透。 */
const FADE_MASK = "linear-gradient(to right, #000 calc(100% - 2rem), transparent)";

/**
 * 看板全尺寸卡的識別列名稱＋複製鈕（spec「看板卡片統一解剖學」：標題恆單行、
 * 複製鈕同列尾隨——取代 design D4 的折行版本）：複製鈕緊跟名稱最後一個字元後，
 * 不以 flex 推至卡片右緣。名稱過長時就地截斷並在尾端漸層淡出，複製鈕收在同一
 * 列不落次行。變更卡與討論卡共用（骨架統一）。
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
  const [copied, setCopied] = useState(false);
  const copy = (e: React.MouseEvent) => {
    e.stopPropagation();
    void navigator.clipboard?.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };
  // 「名稱是否被截斷」CSS 量不到，只能比對 scrollWidth／clientWidth；欄寬變動
  // 由 ResizeObserver 補量。沒溢出就不套遮罩，否則短名稱末尾會被誤淡。
  const nameRef = useRef<HTMLSpanElement>(null);
  const [truncated, setTruncated] = useState(false);
  useEffect(() => {
    const el = nameRef.current;
    if (!el) return;
    const measure = () => setTruncated(el.scrollWidth > el.clientWidth + 1);
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => observer.disconnect();
  }, [text, highlight]);
  return (
    <span className="flex min-w-0 flex-1 items-center gap-1">
      <span
        ref={nameRef}
        data-name
        data-fade={truncated ? "true" : undefined}
        className="font-mono font-semibold text-sm leading-tight min-w-0 overflow-hidden whitespace-nowrap"
        style={truncated ? { maskImage: FADE_MASK, WebkitMaskImage: FADE_MASK } : undefined}
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

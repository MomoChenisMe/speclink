import { CopyButton, REVEAL_ON_HOVER } from "./CopyButton";
import { HighlightText } from "./HighlightText";

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
  return (
    <span className="flex min-w-0 flex-1 items-center gap-1">
      <span
        data-name
        className="font-mono font-semibold text-sm leading-tight min-w-0 truncate"
      >
        <HighlightText text={text} query={highlight} />
      </span>
      <CopyButton value={text} label={copyLabel} className={REVEAL_ON_HOVER} />
    </span>
  );
}

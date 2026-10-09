import type { KeyboardEvent, ReactNode } from "react";
import { ChevronRight } from "lucide-react";

import { cn } from "../lib/utils";
import { CopyButton, REVEAL_ON_HOVER } from "./CopyButton";

export interface ListRowProps {
  /** 等寬標題（spec 名、change 名、討論 slug）。 */
  title: string;
  titleClassName?: string;
  /** 給了就在標題後緊跟 hover 才顯現的複製鈕。 */
  copyValue?: string;
  /** 複製鈕的無障礙名稱（copyValue 存在時必填）。 */
  copyLabel?: string;
  /** 標題下方一行截斷的描述列；缺席時列退回單行。 */
  description?: ReactNode;
  /** 最左槽（已封存列的日期）。 */
  leading?: ReactNode;
  /** 右端 meta 槽（計數、徽章、相對時間），› 緊接其後。 */
  meta?: ReactNode;
  onClick: () => void;
  /** 標題旁的章（如改進小章）。 */
  children?: ReactNode;
  /** 呼叫端的 DOM 錨點（data-spec、data-archived…）。 */
  [dataAttr: `data-${string}`]: string | undefined;
}

/**
 * 共用列（spec desktop-app「規格與封存卡片收合資訊」；design D1）：整列可點、細線
 * 分隔、hover 微灰、右端 › 收尾。鍵盤 Enter／Space 觸發開啟——只認列本體上的按鍵，
 * 複製鈕自己的 Enter 不冒成開列。複製鈕點擊由 CopyButton 停止冒泡。
 */
export function ListRow({
  title,
  titleClassName,
  copyValue,
  copyLabel,
  description,
  leading,
  meta,
  onClick,
  children,
  ...dataAttrs
}: ListRowProps) {
  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.target !== e.currentTarget) return;
    if (e.key !== "Enter" && e.key !== " ") return;
    e.preventDefault();
    onClick();
  };
  return (
    <div
      {...dataAttrs}
      role="button"
      tabIndex={0}
      className="group flex cursor-pointer items-center gap-4 border-b border-border/70 px-4 py-3 outline-none last:border-b-0 hover:bg-foreground/[0.03] focus-visible:bg-foreground/[0.03]"
      onClick={onClick}
      onKeyDown={onKeyDown}
    >
      {leading !== undefined && (
        <span className="shrink-0 text-xs tabular-nums text-muted-foreground">{leading}</span>
      )}
      <div className="min-w-0 flex-1">
        <div data-title-group className="flex items-center gap-1">
          <span className={cn("min-w-0 truncate font-mono text-sm font-semibold", titleClassName)}>{title}</span>
          {children}
          {copyValue !== undefined && (
            <CopyButton value={copyValue} label={copyLabel ?? ""} className={REVEAL_ON_HOVER} />
          )}
        </div>
        {description !== undefined && description !== null && (
          <div data-desc className="truncate text-[13px] text-muted-foreground">
            {description}
          </div>
        )}
      </div>
      {meta !== undefined && (
        <span className="flex shrink-0 items-center gap-3 text-xs text-muted-foreground">{meta}</span>
      )}
      <ChevronRight aria-hidden="true" className="h-4 w-4 shrink-0 text-muted-foreground" />
    </div>
  );
}

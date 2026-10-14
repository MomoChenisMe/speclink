import { useId, type KeyboardEvent, type ReactNode } from "react";
import { ChevronRight } from "lucide-react";

import { cn } from "../lib/utils";
import { CopyButton, REVEAL_ON_HOVER } from "./CopyButton";

export interface ListRowProps {
  /** 標題（spec 名、change 名、討論 slug；預設等寬）。 */
  title: string;
  /** false 時標題改一般字（人名、專案名）；預設 true 等寬。 */
  mono?: boolean;
  /** 給了就在標題後緊跟 hover 才顯現的複製鈕：value 寫入剪貼簿、label 為無障礙名稱。 */
  copy?: { value: string; label: string };
  /** 標題下方一行截斷的描述列；缺席時列退回單行。 */
  description?: ReactNode;
  /** 最左槽（已封存列的日期）。 */
  leading?: ReactNode;
  /** 右端 meta 槽（計數、徽章、相對時間），› 緊接其後。 */
  meta?: ReactNode;
  /** meta 之後、› 之前的列尾動作槽（撤銷鈕）；槽內點擊不冒泡成開列。 */
  actions?: ReactNode;
  /** 不給就是靜態列：沒有按鈕語意、不可聚焦、無 hover 與 ›。 */
  onClick?: () => void;
  /** 不可點的列（規格溯源的無封存記錄來源）：灰字、無 hover、無 ›，點擊與按鍵不觸發。 */
  disabled?: boolean;
  /** 複製鈕之後的章（如改進小章）。 */
  children?: ReactNode;
  /** 呼叫端的 DOM 錨點（data-spec、data-archived…）。 */
  [dataAttr: `data-${string}`]: string | undefined;
}

/**
 * 共用列（spec desktop-app「規格與封存卡片收合資訊」；design D1）：整列可點、細線
 * 分隔、hover 微灰、右端 › 收尾。鍵盤 Enter／Space 觸發開啟——只認列本體上的按鍵，
 * 複製鈕與動作槽自己的 Enter 不冒成開列。複製鈕點擊由 CopyButton 停止冒泡、動作槽的點擊
 * 由槽的包裝停止冒泡。列的朗讀名稱只取標題（aria-labelledby），不把描述與 meta 串成一長串。
 * 不給 onClick 是靜態列（server-web-list-pages design D1：憑證列、邀請列）。
 */
export function ListRow({
  title,
  mono = true,
  copy,
  description,
  leading,
  meta,
  actions,
  onClick,
  disabled = false,
  children,
  ...dataAttrs
}: ListRowProps) {
  const titleId = useId();
  const clickable = onClick !== undefined;
  const active = clickable && !disabled;
  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.target !== e.currentTarget) return;
    if (e.key !== "Enter" && e.key !== " ") return;
    e.preventDefault();
    onClick?.();
  };
  return (
    <div
      {...dataAttrs}
      role={clickable ? "button" : undefined}
      tabIndex={clickable ? (disabled ? -1 : 0) : undefined}
      aria-labelledby={clickable ? titleId : undefined}
      aria-disabled={(clickable && disabled) || undefined}
      className={cn(
        "group flex items-center gap-4 border-b border-border/70 px-4 py-3 outline-none last:border-b-0",
        disabled && "cursor-default text-muted-foreground",
        active &&
          "cursor-(--cursor-interaction) hover:bg-foreground/5 focus-visible:bg-foreground/5 focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring",
      )}
      onClick={active ? onClick : undefined}
      onKeyDown={active ? onKeyDown : undefined}
    >
      {leading !== undefined && (
        <span className="shrink-0 text-xs tabular-nums text-muted-foreground">{leading}</span>
      )}
      <div className="min-w-0 flex-1">
        <div data-title-group className="flex items-center gap-1">
          <span id={titleId} className={cn("min-w-0 truncate text-sm font-semibold", mono && "font-mono")}>
            {title}
          </span>
          {copy && <CopyButton value={copy.value} label={copy.label} className={REVEAL_ON_HOVER} />}
          {children}
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
      {actions !== undefined && actions !== null && actions !== false && (
        // 只攔點擊冒泡；鍵盤由 onKeyDown 只認列本體擋住。
        <div className="flex shrink-0 items-center gap-2" onClick={(e) => e.stopPropagation()}>
          {actions}
        </div>
      )}
      {active && <ChevronRight aria-hidden="true" className="h-4 w-4 shrink-0 text-muted-foreground" />}
    </div>
  );
}

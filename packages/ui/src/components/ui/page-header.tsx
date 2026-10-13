import type { ReactNode } from "react";

import { cn } from "../../lib/utils";

/** 頁標題區（design D6）：24px 一般字重標題＋灰字說明，右側動作槽與標題底緣對齊。
 *  `level` 只換標籤不換樣式：desktop 頁標題在 app 標題之下用 h2；server 後台沒有外層標題，用 h1。 */
function PageHeader({
  title,
  description,
  actions,
  level = 2,
  className,
}: {
  title: string;
  description?: string;
  actions?: ReactNode;
  level?: 1 | 2;
  className?: string;
}) {
  const Heading = level === 1 ? "h1" : "h2";
  return (
    <div data-page-header className={cn("flex shrink-0 items-end justify-between gap-3", className)}>
      <div className="min-w-0">
        <Heading className="text-2xl font-normal">{title}</Heading>
        {description && <p className="mt-0.5 text-[13px] text-muted-foreground">{description}</p>}
      </div>
      {actions && <div className="flex shrink-0 items-center gap-1.5">{actions}</div>}
    </div>
  );
}

export { PageHeader };

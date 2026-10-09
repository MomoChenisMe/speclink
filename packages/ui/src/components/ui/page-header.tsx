import type { ReactNode } from "react";

import { cn } from "../../lib/utils";

/** 頁標題區（design D6）：24px 一般字重標題＋灰字說明，右側動作槽與標題底緣對齊。 */
function PageHeader({
  title,
  description,
  actions,
  className,
}: {
  title: string;
  description?: string;
  actions?: ReactNode;
  className?: string;
}) {
  return (
    <div data-page-header className={cn("flex shrink-0 items-end justify-between gap-3", className)}>
      <div className="min-w-0">
        <h2 className="text-2xl font-normal">{title}</h2>
        {description && <p className="mt-0.5 text-[13px] text-muted-foreground">{description}</p>}
      </div>
      {actions && <div className="flex shrink-0 items-center gap-1.5">{actions}</div>}
    </div>
  );
}

export { PageHeader };

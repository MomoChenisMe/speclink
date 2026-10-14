import type { ReactNode } from "react";

import { cn } from "../../lib/utils";

/** 頁標題區（design D6）：24px 一般字重標題＋灰字說明，右側動作槽與標題底緣對齊。
 *  `level` 只換標籤不換樣式：desktop 頁標題在 app 標題之下用 h2；server 後台沒有外層標題，用 h1。
 *  並排放不下時動作槽換到標題下方一列、槽內控件再各自換行（server-web-list-pages design D1）；
 *  標題欄以 16rem 為換行基準，長說明照舊在欄內折行，不會因說明長就把動作擠下去。 */
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
    <div data-page-header className={cn("flex shrink-0 flex-wrap items-end justify-between gap-3", className)}>
      <div className="min-w-0 grow basis-64">
        <Heading className="text-2xl font-normal">{title}</Heading>
        {description && <p className="mt-0.5 text-[13px] text-muted-foreground">{description}</p>}
      </div>
      {actions && <div className="flex max-w-full shrink-0 flex-wrap items-center gap-1.5">{actions}</div>}
    </div>
  );
}

export { PageHeader };

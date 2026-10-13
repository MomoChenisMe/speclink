// 麵包屑（server-web-shell-alignment design D4）：desktop 主區頂列與 server 後台頂列共用一份。
import { Fragment } from "react";

import { useI18n } from "../i18n";
import { cn } from "../lib/utils";

export interface Crumb {
  label: string;
  onClick?: () => void;
  /** 等寬字型（詳情頁的項目名：變更名或討論 slug）。 */
  mono?: boolean;
}

/** 一列 12px 灰字，項間半透明「/」；最後一項為前景色，有 onClick 的項是按鈕。 */
export function Breadcrumbs({ crumbs }: { crumbs: Crumb[] }) {
  const { t } = useI18n();
  return (
    <nav
      aria-label={t("breadcrumbs.label")}
      className="flex min-w-0 flex-1 items-center gap-1.5 text-xs text-muted-foreground"
    >
      {crumbs.map((crumb, i) => {
        const last = i === crumbs.length - 1;
        const cls = cn("truncate", last && "text-foreground", crumb.mono && "font-mono");
        return (
          <Fragment key={i}>
            {i > 0 && <span className="opacity-50">/</span>}
            {crumb.onClick ? (
              <button type="button" className={cn(cls, "hover:text-foreground")} onClick={crumb.onClick}>
                {crumb.label}
              </button>
            ) : (
              <span className={cls}>{crumb.label}</span>
            )}
          </Fragment>
        );
      })}
    </nav>
  );
}

import type { HTMLAttributes } from "react";
import { CircleHelp } from "lucide-react";
import { MESSAGES, useI18n } from "../i18n";
import { Button } from "./ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "./ui/popover";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";

/** 審查站預設 smell 名稱清單（名稱由引擎提供）：desktop 設定頁的審查規則頁與工單分頁的
 * 套用規則列共用同一個元件。 */
export function BaselineSmellList({ names, baseline = "extend", ...rest }: { names: readonly string[]; baseline?: "extend" | "replace" } & HTMLAttributes<HTMLUListElement>) {
  const { t } = useI18n();
  return (
    <TooltipProvider>
    <ul data-baseline-smells {...rest} className="m-0 flex list-none flex-wrap gap-1.5 p-0">
      {names.map((name) => {
        const label = t("reviewSmell.help").replace("{name}", name);
        const known = Object.prototype.hasOwnProperty.call(MESSAGES["zh-TW"], `reviewSmell.${name}.purpose`);
        return <li key={name} className="flex items-center gap-1 rounded-md border border-border bg-muted/40 px-2 py-0.5 font-mono text-xs">
          {name}
          <Popover>
            <Tooltip>
              <TooltipTrigger asChild>
                <PopoverTrigger asChild>
                  <Button type="button" variant="subtle" size="icon-xs" aria-label={label}><CircleHelp className="h-3.5 w-3.5" aria-hidden /></Button>
                </PopoverTrigger>
              </TooltipTrigger>
              <TooltipContent>{label}</TooltipContent>
            </Tooltip>
            <PopoverContent aria-label={label} className="max-w-80 space-y-2 font-sans text-sm">
              <p className="font-medium">{name}</p>
              {known ? <>
                <p>{t(`reviewSmell.${name}.purpose`)}</p>
                <p className="text-muted-foreground">{t(`reviewSmell.${name}.example`)}</p>
              </> : <p>{t("reviewSmell.unknown")}</p>}
              <p className="text-xs text-muted-foreground">{t("reviewSmell.baselineHelp")}</p>
              {baseline === "replace" && <p className="text-xs">{t("reviewSmell.replaced")}</p>}
            </PopoverContent>
          </Popover>
        </li>;
      })}
    </ul>
    <p className="text-xs text-muted-foreground">{t("reviewSmell.baselineHelp")}</p>
    {baseline === "replace" && <p className="text-xs text-muted-foreground">{t("reviewSmell.replaced")}</p>}
    </TooltipProvider>
  );
}

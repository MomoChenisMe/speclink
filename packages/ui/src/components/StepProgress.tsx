// 多步驟進度列（server-web-shell-alignment design D7）：desktop 工作區精靈與 server 初始設定共用一份。
import { useI18n } from "../i18n";
import { cn } from "../lib/utils";

/** 「[前綴 · ]步驟 n / total」小字（11px 大寫字距灰字）加步驟條，已到的段為主色。 */
export function StepProgress({ current, total, prefix }: { current: number; total: number; prefix?: string }) {
  const { t } = useI18n();
  const text = t("steps.progress").replace("{current}", String(current)).replace("{total}", String(total));
  return (
    <div className="mb-1 flex items-center justify-between gap-4">
      <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-muted-foreground">
        {prefix ? `${prefix} · ${text}` : text}
      </span>
      <div data-testid="step-bar" className="flex gap-1" aria-hidden="true">
        {Array.from({ length: total }, (_, i) => (
          <span key={i} className={cn("h-1 w-8 rounded-full bg-muted", i < current && "bg-primary")} />
        ))}
      </div>
    </div>
  );
}

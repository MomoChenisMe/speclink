// 設定頁的五個零件（desktop-settings-mode design D3）：應用程式設定與專案設定的每一頁都以
// 頁標題、卡、列、列右側下拉排版；群組標題由分類欄群組與卡的小標共用。另有外開連結列
// （desktop-about-page design D2），版面同列但整列可點。
import { useId, type ReactNode } from "react";
import { ExternalLink } from "lucide-react";

import { cn } from "../../lib/utils";
import { SEMANTIC_TONE } from "../../tone";
import { Card } from "./card";
import { PageHeader } from "./page-header";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./select";

/** 頁標題：包 PageHeader；`note` 為其下一行 12px 灰字（本機頁的「僅存於此裝置」、
 *  專案設定的等寬檔案鍵或 policy revision）。 */
export function SettingsPageHeader({
  title,
  description,
  actions,
  note,
}: {
  title: string;
  description?: string;
  actions?: ReactNode;
  note?: ReactNode;
}) {
  return (
    <div className="flex flex-col gap-1.5">
      <PageHeader title={title} description={description} actions={actions} />
      {note && (
        <p data-settings-note className="m-0 text-xs text-muted-foreground">
          {note}
        </p>
      )}
    </div>
  );
}

/** 群組標題：分類欄的群組（12px 粗體灰字）與卡的小標（`size="card"`，14px）共用；
 *  圖示與計數徽章選配。 */
export function SettingsGroupHeader({
  icon,
  label,
  count,
  size = "group",
}: {
  icon?: ReactNode;
  label: string;
  count?: number;
  size?: "group" | "card";
}) {
  return (
    <div
      className={cn(
        "flex items-center gap-1.5 [&_svg]:h-3.5 [&_svg]:w-3.5 [&_svg]:shrink-0",
        size === "group"
          ? "px-2.5 text-xs font-semibold text-muted-foreground"
          : "px-1 text-sm font-medium text-foreground",
      )}
    >
      {icon}
      <span className="truncate">{label}</span>
      {count !== undefined && (
        <span className="inline-flex h-4 min-w-4 items-center justify-center rounded-full bg-muted px-1 text-[11px] font-normal tabular-nums text-muted-foreground">
          {count}
        </span>
      )}
    </div>
  );
}

/** 卡：14px 小標＋16px 圓角卡，卡內一列一筆、列間細線；以小標為區域名。`aside` 放在小標同一列
 *  右端（連結或 12px 灰字說明；server-web-list-pages design D1）。 */
export function SettingsCard({
  title,
  count,
  icon,
  aside,
  children,
}: {
  title: string;
  count?: number;
  icon?: ReactNode;
  aside?: ReactNode;
  children: ReactNode;
}) {
  return (
    <section aria-label={title} className="flex flex-col gap-2">
      <div data-settings-card-head className="flex items-center gap-3">
        <div className="min-w-0 flex-1">
          <SettingsGroupHeader icon={icon} label={title} count={count} size="card" />
        </div>
        {aside && (
          <div data-settings-card-aside className="shrink-0 px-1 text-xs text-muted-foreground">
            {aside}
          </div>
        )}
      </div>
      <Card data-settings-card-body className="divide-y divide-border">
        {children}
      </Card>
    </section>
  );
}

/** 列：左 14px 標籤＋12px 灰字說明、右控制項；`warning` 時說明改警示色；`children` 渲染在
 *  列下方（展開的細節、行內錯誤）。以標籤為群組名，呼叫端可就列找控制項；給了 `htmlFor`
 *  時標籤只標註該控制項，不再兼作群組名（同一個名字不指向兩個元素）。根節點帶 `group`，
 *  控制項可用 `group-hover:` 依列滑過顯示。 */
export function SettingsRow({
  label,
  description,
  control,
  htmlFor,
  warning,
  children,
}: {
  /** 群組名取自標籤的文字內容；可放等寬版號之類的行內標記。 */
  label: ReactNode;
  description?: ReactNode;
  control: ReactNode;
  htmlFor?: string;
  warning?: boolean;
  children?: ReactNode;
}) {
  const labelId = useId();
  const Label = htmlFor ? "label" : "span";
  return (
    <div role="group" aria-labelledby={htmlFor ? undefined : labelId} className="group grid grid-cols-[1fr_auto] items-center gap-x-4 gap-y-2 px-4 py-3">
      <div className="flex min-w-0 flex-col gap-0.5">
        <Label id={labelId} htmlFor={htmlFor} className="text-sm">
          {label}
        </Label>
        {description && (
          <p className={cn("m-0 text-xs", warning ? SEMANTIC_TONE.warning : "text-muted-foreground")}>
            {description}
          </p>
        )}
      </div>
      <div className="flex shrink-0 items-center justify-end gap-2">{control}</div>
      {children && <div className="col-span-2 min-w-0">{children}</div>}
    </div>
  );
}

/** 外開連結列（desktop-about-page design D2）：版面同 `SettingsRow`，但整列是按鈕——hover
 *  灰底、右端固定外開圖示，不放控制項。 */
export function SettingsLinkRow({
  label,
  description,
  onClick,
}: {
  label: ReactNode;
  description?: ReactNode;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="grid w-full grid-cols-[1fr_auto] items-center gap-x-4 px-4 py-3 text-left transition-colors hover:bg-muted focus-visible:bg-muted focus-visible:outline-none"
    >
      <span className="flex min-w-0 flex-col gap-0.5">
        <span className="text-sm">{label}</span>
        {description && <span className="text-xs text-muted-foreground">{description}</span>}
      </span>
      <ExternalLink aria-hidden className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
    </button>
  );
}

export interface SettingsChoiceOption {
  value: string;
  label: string;
  disabled?: boolean;
}

/** 列右側下拉：包 Select，觸發鈕 h-8、13px 字。值不得為空字串（Radix 限制），
 *  「未設定」之類的狀態由呼叫端映射成哨兵值。 */
export function SettingsChoice({
  value,
  options,
  onChange,
  ariaLabel,
  disabled,
  id,
}: {
  value: string;
  options: SettingsChoiceOption[];
  onChange: (value: string) => void;
  ariaLabel?: string;
  disabled?: boolean;
  id?: string;
}) {
  return (
    <Select value={value} onValueChange={onChange} disabled={disabled}>
      <SelectTrigger id={id} aria-label={ariaLabel} className="w-auto min-w-40 text-[13px]">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {options.map((option) => (
          <SelectItem key={option.value} value={option.value} disabled={option.disabled}>
            {option.label}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}

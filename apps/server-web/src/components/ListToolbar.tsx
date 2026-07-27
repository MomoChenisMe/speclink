import type { ReactNode } from "react";
import { Search } from "lucide-react";
import {
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  useI18n,
} from "@speclink/ui";

// 管理列表頁的共用工具列（server-web-console「管理列表提供搜尋、篩選、分頁與具引導的
// 空狀態」）：關鍵字搜尋固定在最前，其後接該頁自己的篩選器。控制項留在列表上方而非列內
// ——列內不得含任何輸入控制項。
//
// 所有控件統一 h-9，與 Input／SelectTrigger 原語的預設高度相同。

/** 工具列控件的共同高度，讓 input、select 與日期欄並排時對齊。 */
const CONTROL_HEIGHT = "h-9";

/**
 * 「全部」在頁面 state 裡是空字串（直接餵給 API 的 query 參數），但 Radix Select 禁止
 * 空字串當 item value——以這個哨符在元件內部來回換，呼叫端仍只認得空字串。
 */
const ALL = "__all__";

export function ListToolbar({
  search,
  onSearchChange,
  searchLabel,
  children,
}: {
  search: string;
  onSearchChange: (value: string) => void;
  /** 覆寫預設的「搜尋」標籤。 */
  searchLabel?: string;
  /** 該頁自己的篩選器（例如狀態、動作、來源、時間區間）。 */
  children?: ReactNode;
}) {
  const { t } = useI18n();
  return (
    <div className="flex flex-wrap items-end gap-3">
      <div className="min-w-0 space-y-1.5">
        <Label htmlFor="list-search">{searchLabel ?? t("common.search")}</Label>
        <div className="relative">
          <Search
            aria-hidden="true"
            className="pointer-events-none absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground"
          />
          <Input
            id="list-search"
            type="search"
            value={search}
            onChange={(e) => onSearchChange(e.target.value)}
            className={`w-56 max-w-full pl-8 ${CONTROL_HEIGHT}`}
          />
        </div>
      </div>
      {children}
    </div>
  );
}

/** 工具列內一個帶 label 的下拉篩選器；空字串代表不篩選，選項為「全部」。 */
export function ToolbarSelect({
  id,
  label,
  allLabel,
  value,
  onChange,
  children,
}: {
  id: string;
  label: string;
  /** 「不篩選」那一項的文字，例如「全部狀態」。 */
  allLabel: string;
  value: string;
  onChange: (value: string) => void;
  /** 該篩選器自己的選項（`SelectItem`），不含「全部」。 */
  children: ReactNode;
}) {
  return (
    <div className="space-y-1.5">
      <Label htmlFor={id}>{label}</Label>
      <Select value={value === "" ? ALL : value} onValueChange={(v) => onChange(v === ALL ? "" : v)}>
        <SelectTrigger id={id} className={`w-36 ${CONTROL_HEIGHT}`}>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value={ALL}>{allLabel}</SelectItem>
          {children}
        </SelectContent>
      </Select>
    </div>
  );
}

/** 工具列內一個帶 label 的日期輸入（時間區間的一端）。 */
export function ToolbarDate({
  id,
  label,
  value,
  onChange,
}: {
  id: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <div className="space-y-1.5">
      <Label htmlFor={id}>{label}</Label>
      <Input
        id={id}
        type="date"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className={`w-40 ${CONTROL_HEIGHT}`}
      />
    </div>
  );
}

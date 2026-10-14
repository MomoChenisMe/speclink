import type { ReactNode } from "react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@speclink/ui";

/**
 * 「全部」在頁面 state 裡是空字串（直接餵給 API 的 query 參數），但 Radix Select 禁止
 * 空字串當 item value——以這個哨符在元件內部來回換，呼叫端仍只認得空字串。
 */
const ALL = "__all__";

/** 頁標題區的篩選下拉（server-web-list-pages design D2）：沒有可見標籤，以 `label` 作無障礙名稱；
 *  高度與搜尋框同為 h-8（SelectTrigger 預設）。空字串代表不篩選，選項為「全部」。 */
export function FilterSelect({
  label,
  allLabel,
  value,
  onChange,
  children,
}: {
  label: string;
  /** 「不篩選」那一項的文字，例如「全部狀態」。 */
  allLabel: string;
  value: string;
  onChange: (value: string) => void;
  /** 該篩選器自己的選項（`SelectItem`），不含「全部」。 */
  children: ReactNode;
}) {
  return (
    <Select value={value === "" ? ALL : value} onValueChange={(v) => onChange(v === ALL ? "" : v)}>
      <SelectTrigger aria-label={label} className="w-auto min-w-32 text-[13px]">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value={ALL}>{allLabel}</SelectItem>
        {children}
      </SelectContent>
    </Select>
  );
}

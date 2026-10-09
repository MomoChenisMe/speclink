import { Search } from "lucide-react";

import { cn } from "../lib/utils";
import { Input } from "./ui/input";

export interface SearchFieldProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  /** 只放版面（如縮窄寬度）。 */
  className?: string;
}

/** 全圓搜尋框（design D3；與看板同款）：左側搜尋圖示＋280px 圓角輸入框。 */
export function SearchField({ value, onChange, placeholder, className }: SearchFieldProps) {
  return (
    <div className={cn("relative", className)}>
      <Search
        aria-hidden="true"
        className="pointer-events-none absolute left-3 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground"
      />
      <Input
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="h-8 w-[280px] max-w-full rounded-full pl-8"
      />
    </div>
  );
}

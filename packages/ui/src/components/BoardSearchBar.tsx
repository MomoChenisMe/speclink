import { useEffect, useRef } from "react";
import { Search, SlidersHorizontal, X } from "lucide-react";

import { useI18n } from "../i18n";
import { Button } from "./ui/button";
import { Input } from "./ui/input";

/**
 * 看板搜尋工具列（design D5，單列）：搜尋輸入填滿剩餘寬度（帶圖示、輸入非空時
 * 的清除鈕與即時命中數）＋同高的篩選開關鈕（漏斗；啟用中篩選帶計數徽章）。
 * 點開關於其下方彈出篩選面板（children，由宿主組裝）；再點開關、點面板外或
 * 按 Esc 關閉——關閉不清除已啟用篩選。Cmd+F／Ctrl+F 聚焦快捷鍵。
 * 純受控元件——query 與面板開闔狀態皆在宿主，本元件不持久化。
 */
export function BoardSearchBar({
  query,
  onQuery,
  hitCount,
  filtersOpen,
  onToggleFilters,
  onCloseFilters,
  activeFilterCount = 0,
  children,
  disabledReason,
}: {
  query: string;
  onQuery: (q: string) => void;
  /** 過濾後各欄卡片總數（僅 query 非空時呈現）。 */
  hitCount: number;
  /** 篩選面板是否開啟（宿主狀態）。 */
  filtersOpen?: boolean;
  /** 篩選開關鈕（未提供時不渲染開關與面板）。 */
  onToggleFilters?: () => void;
  /** 關閉面板（Esc、點面板外）；未提供時退回 onToggleFilters。 */
  onCloseFilters?: () => void;
  /** 啟用中的篩選維度數——開關鈕上的計數徽章（0＝不顯）。 */
  activeFilterCount?: number;
  /** 篩選面板內容（design D5）——三維度選單與全部清除。 */
  children?: React.ReactNode;
  /** 搜尋不可用的說明（remote capability 缺口）：提供時輸入 disabled 並以其
   * 文字為 tooltip（title）、聚焦快捷鍵停用；缺席＝照常。 */
  disabledReason?: string;
}) {
  const { t } = useI18n();
  const inputRef = useRef<HTMLInputElement>(null);
  const popoverRef = useRef<HTMLDivElement>(null);
  // 全域快捷鍵聚焦：macOS Cmd+F、其他平台 Ctrl+F（spec「快捷鍵聚焦搜尋輸入」）。
  useEffect(() => {
    if (disabledReason !== undefined) return; // 停用時快捷鍵一併不掛。
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f") {
        e.preventDefault();
        inputRef.current?.focus();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [disabledReason]);
  // 面板關閉路徑：Esc 與點擊面板外（開關鈕在 popoverRef 內，點它走 toggle 不誤關）。
  const close = onCloseFilters ?? onToggleFilters;
  useEffect(() => {
    if (!filtersOpen || !close) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") close();
    };
    const onDown = (e: MouseEvent) => {
      const target = e.target as Element | null;
      // 面板內的 Select 展開後，選單被 Radix portal 到 body，DOM 上不在 popoverRef 內。
      // 不排除它的話，選任何一個篩選值都會順手把整個面板關掉——原生 select 由作業系統
      // 繪製、不產生 DOM 事件，換成 Radix 才浮現這條路徑。
      if (target?.closest?.("[data-radix-popper-content-wrapper]")) return;
      if (popoverRef.current && !popoverRef.current.contains(target as Node)) close();
    };
    window.addEventListener("keydown", onKey);
    document.addEventListener("mousedown", onDown);
    return () => {
      window.removeEventListener("keydown", onKey);
      document.removeEventListener("mousedown", onDown);
    };
  }, [filtersOpen, close]);
  const active = query.trim().length > 0;
  return (
    <div className="flex w-full shrink-0 items-center gap-1.5">
      <div className="relative min-w-0 flex-1">
        <Search className="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
        <Input
          ref={inputRef}
          placeholder={t("kanban.searchPlaceholder")}
          value={query}
          disabled={disabledReason !== undefined}
          title={disabledReason}
          onChange={(e) => onQuery(e.target.value)}
          className={`pl-8 ${active ? "pr-24" : ""}`}
        />
        {active && (
          <div className="absolute right-2 top-1/2 flex -translate-y-1/2 items-center gap-1">
            <span data-testid="search-hits" className="text-[11px] tabular-nums text-muted-foreground">
              {t("kanban.hitCount").replace("{n}", String(hitCount))}
            </span>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              aria-label={t("kanban.clearSearch")}
              className="h-5 w-5 text-muted-foreground hover:text-foreground"
              onClick={() => {
                onQuery("");
                inputRef.current?.focus();
              }}
            >
              <X className="h-3.5 w-3.5" />
            </Button>
          </div>
        )}
      </div>
      {onToggleFilters && (
        <div ref={popoverRef} className="relative shrink-0">
          <Button
            type="button"
            variant="ghost"
            size="icon"
            aria-label={t("filter.toggle")}
            aria-expanded={!!filtersOpen}
            onClick={onToggleFilters}
            className={`relative h-9 w-9 ${filtersOpen ? "bg-accent text-foreground" : "text-muted-foreground"}`}
          >
            <SlidersHorizontal className="h-4 w-4" />
            {activeFilterCount > 0 && (
              <span className="absolute -right-0.5 -top-0.5 inline-flex h-3.5 min-w-3.5 items-center justify-center rounded-full bg-primary px-0.5 text-[9px] font-bold text-primary-foreground">
                {activeFilterCount}
              </span>
            )}
          </Button>
          {filtersOpen && (
            <div
              data-filter-popover
              className="absolute right-0 top-full z-20 mt-1.5 flex w-64 flex-col gap-2.5 rounded-md border border-border bg-card p-3 shadow-md"
            >
              {children}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

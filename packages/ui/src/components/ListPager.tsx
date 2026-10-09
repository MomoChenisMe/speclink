import { useState, type KeyboardEvent } from "react";
import { ChevronLeft, ChevronRight } from "lucide-react";

import { useI18n } from "../i18n";
import { cn } from "../lib/utils";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./ui/select";

/** 每頁筆數預設值（spec「清單最新在前與換頁瀏覽」）——清單元件與測試共用。 */
export const PAGE_SIZE = 20;

/** 每頁筆數下拉的預設選項。 */
const PAGE_SIZE_OPTIONS = [20, 50, 100];

/** 頁碼視窗要列的頁數上限；超過時恆列首尾頁、目前頁前後各一頁，其餘以 … 收攏。 */
const FULL_WINDOW = 7;

/**
 * 頁碼視窗（design D2）：總頁數 ≤ 7 全列；否則恆列 1 與 M，中段為 page−1..page+1，
 * 與兩端不相鄰處插入 "…"。純函式。
 */
export function pageWindow(page: number, pageCount: number): Array<number | "…"> {
  if (pageCount <= FULL_WINDOW) return Array.from({ length: pageCount }, (_, i) => i + 1);
  const lo = Math.max(2, page - 1);
  const hi = Math.min(pageCount - 1, page + 1);
  const out: Array<number | "…"> = [1];
  if (lo > 2) out.push("…");
  for (let n = lo; n <= hi; n++) out.push(n);
  if (hi < pageCount - 1) out.push("…");
  out.push(pageCount);
  return out;
}

const clamp = (n: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, n));

export interface ListPagerProps {
  /** 目前頁碼（1 起算）。 */
  page: number;
  /** 總頁數。 */
  pageCount: number;
  /** 過濾後總筆數；0 時整條工具列不渲染。 */
  total: number;
  /** 每頁筆數（受控，呼叫端持有）。 */
  pageSize: number;
  pageSizeOptions?: number[];
  /** 換頁回呼——受控形態，頁碼狀態由呼叫端持有。 */
  onPage: (next: number) => void;
  /** 改每頁筆數回呼；呼叫端重算總頁數並鉗制頁碼。 */
  onPageSize: (size: number) => void;
}

/**
 * 換頁工具列（spec「清單最新在前與換頁瀏覽」；design D2：自建受控元件，不採 shadcn
 * Pagination）：左側「第 a–b 筆，共 N 筆」＋每頁筆數下拉；總頁數 > 1 時右側為
 * ‹ 頁碼 › 與跳頁輸入（Enter 或失焦提交、越界鉗制、非數字忽略）。
 */
export function ListPager({
  page,
  pageCount,
  total,
  pageSize,
  pageSizeOptions = PAGE_SIZE_OPTIONS,
  onPage,
  onPageSize,
}: ListPagerProps) {
  const { t } = useI18n();
  const [jump, setJump] = useState("");
  if (total === 0) return null;

  const first = (page - 1) * pageSize + 1;
  const last = Math.min(page * pageSize, total);
  const range = t("pager.range")
    .replace("{a}", String(first))
    .replace("{b}", String(last))
    .replace("{n}", String(total));
  const perPage = (n: number) => t("pager.perPage").replace("{n}", String(n));
  // 「跳到 {n} 頁」拆成輸入框前後的兩段文字；缺 {n} 時整句在前。
  const [jumpPrefix, jumpSuffix = ""] = t("pager.jumpTo").split("{n}");

  // 跳頁提交：空字串與非數字忽略，其餘鉗制到 1..pageCount。
  const commitJump = () => {
    if (jump.trim() === "") return;
    const n = Number(jump);
    if (!Number.isFinite(n)) return;
    onPage(clamp(Math.trunc(n), 1, pageCount));
    setJump("");
  };
  const onJumpKey = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") commitJump();
  };

  return (
    <div className="flex items-center justify-between gap-3 px-4 py-2 text-xs text-muted-foreground">
      <div className="flex items-center gap-3">
        <span className="tabular-nums">{range}</span>
        <Select value={String(pageSize)} onValueChange={(v) => onPageSize(Number(v))}>
          <SelectTrigger aria-label={perPage(pageSize)} className="h-7 w-auto gap-1 text-xs">
            <SelectValue>{perPage(pageSize)}</SelectValue>
          </SelectTrigger>
          <SelectContent>
            {pageSizeOptions.map((n) => (
              <SelectItem key={n} value={String(n)}>
                {perPage(n)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      {pageCount > 1 && (
        <nav className="flex items-center gap-1">
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            aria-label={t("pager.prev")}
            disabled={page <= 1}
            onClick={() => onPage(page - 1)}
          >
            <ChevronLeft className="h-3.5 w-3.5" />
          </Button>
          {pageWindow(page, pageCount).map((item, i) =>
            item === "…" ? (
              <span key={`gap-${i}`} className="px-1 select-none">
                …
              </span>
            ) : (
              <Button
                key={item}
                type="button"
                variant="ghost"
                size="icon-sm"
                aria-label={t("pager.pageN").replace("{n}", String(item))}
                aria-current={item === page ? "page" : undefined}
                className={cn("tabular-nums", item === page && "bg-primary/12 font-medium text-primary")}
                onClick={() => onPage(item)}
              >
                {item}
              </Button>
            ),
          )}
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            aria-label={t("pager.next")}
            disabled={page >= pageCount}
            onClick={() => onPage(page + 1)}
          >
            <ChevronRight className="h-3.5 w-3.5" />
          </Button>
          <label className="ml-2 flex items-center gap-1">
            <span>{jumpPrefix}</span>
            <Input
              type="number"
              min={1}
              max={pageCount}
              aria-label={t("pager.jump")}
              className="h-7 w-[3em] px-1 text-center text-xs"
              value={jump}
              onChange={(e) => setJump(e.target.value)}
              onKeyDown={onJumpKey}
              onBlur={commitJump}
            />
            <span>{jumpSuffix}</span>
          </label>
        </nav>
      )}
    </div>
  );
}

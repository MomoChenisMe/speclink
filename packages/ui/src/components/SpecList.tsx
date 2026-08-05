import { useMemo, useRef, useState } from "react";
import { Check, Copy, FileText, History } from "lucide-react";

import type { SpecItem } from "../adapter";
import { useI18n } from "../i18n";
import { matchesQuery } from "../search";
import { relativeDays } from "../time";
import { SEMANTIC_TONE } from "../tone";
import { Input } from "./ui/input";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";
import { ListPager, PAGE_SIZE } from "./ListPager";

/** 規格卡（spec-archive-drawer design D7）：標題＋複製鈕成群組、meta（需求數、
 * 溯源變更數、相對修改時間）靠右；第二列 Purpose 摘要一行截斷，佔位時改顯
 * 琥珀「Purpose 待補」警示。點整列開唯讀規格抽屜，無行內展開。 */
function SpecCard({ item, onOpen }: { item: SpecItem; onOpen: (capability: string) => void }) {
  const { t } = useI18n();
  const [copied, setCopied] = useState(false);

  const copy = (e: React.MouseEvent) => {
    e.stopPropagation();
    void navigator.clipboard?.writeText(item.id);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };

  const rel = relativeDays(item.modifiedAt, t);
  const reqCount = item.requirementCount ?? 0;
  const traceCount = item.traceCount ?? 0;
  return (
    <TooltipProvider>
      <div
        data-spec={item.id}
        className="group cursor-pointer rounded-lg border border-border bg-card p-3 transition-[border-color,box-shadow] hover:border-primary/60 hover:shadow-md"
        onClick={() => onOpen(item.id)}
      >
        <div className="flex items-center gap-2.5">
          {/* 標題＋複製鈕成一個群組吃 flex-1（標題 truncate、複製鈕緊跟 hover 顯現）。 */}
          <span data-title-group className="flex min-w-0 flex-1 items-center gap-1">
            <span className="min-w-0 truncate text-sm font-medium">{item.id}</span>
            <span
              role="button"
              aria-label={copied ? t("specs.copied") : t("common.copyName")}
              className={`shrink-0 text-muted-foreground transition-opacity hover:text-foreground ${copied ? "opacity-100" : "opacity-0 group-hover:opacity-100"}`}
              onClick={copy}
            >
              {copied ? <Check className="h-3.5 w-3.5 text-primary" /> : <Copy className="h-3.5 w-3.5" />}
            </span>
          </span>
          {/* meta 靠右：需求數徽章、溯源變更數、相對修改時間。 */}
          <span className="flex shrink-0 items-center gap-2 text-xs text-muted-foreground">
            <Tooltip>
              <TooltipTrigger asChild>
                <span
                  aria-label={t("specs.requirementCount").replace("{n}", String(reqCount))}
                  className="inline-flex items-center gap-1 tabular-nums"
                >
                  <FileText className="h-3 w-3" />
                  {reqCount}
                </span>
              </TooltipTrigger>
              <TooltipContent>
                {t("specs.requirementCount").replace("{n}", String(reqCount))}
              </TooltipContent>
            </Tooltip>
            {traceCount > 0 && (
              <Tooltip>
                <TooltipTrigger asChild>
                  <span
                    aria-label={t("specs.traceCount").replace("{n}", String(traceCount))}
                    className="inline-flex items-center gap-1 tabular-nums"
                  >
                    <History className="h-3 w-3" />
                    {traceCount}
                  </span>
                </TooltipTrigger>
                <TooltipContent>{t("specs.traceCount").replace("{n}", String(traceCount))}</TooltipContent>
              </Tooltip>
            )}
            {rel && <span className="tabular-nums">{rel}</span>}
          </span>
        </div>
        {/* 描述列：purposeTbd 以琥珀警示取代摘要，否則 Purpose 首行一行截斷；皆缺席時整列缺席。 */}
        {item.purposeTbd ? (
          <div className={`mt-1 text-[11px] font-medium ${SEMANTIC_TONE.warning}`}>
            {t("specs.purposeTbd")}
          </div>
        ) : (
          item.purposeExcerpt && (
            <div className="mt-1 truncate text-[11px] text-muted-foreground">{item.purposeExcerpt}</div>
          )
        )}
      </div>
    </TooltipProvider>
  );
}

export interface SpecListProps {
  specs: SpecItem[];
  /** 點卡片開唯讀規格抽屜（capability 定址）。 */
  onOpen: (capability: string) => void;
}

/** 規格頁（design D1）：正典 spec 卡片清單＋名稱搜尋（design D3：大小寫不敏感
 * 子字串、純前端即打即濾）；點卡片開抽屜檢視全文，無行內展開、無任何規格寫入動詞。
 * 清單最新在前（modifiedAt 降冪、缺席殿後、名稱升冪決勝）並依 PAGE_SIZE 換頁
 *（spec「清單最新在前與換頁瀏覽」）——排序與換頁純屬呈現層。
 * 版面填滿視窗高度：搜尋框與標題列固定頂部、卡片清單於內部容器捲動、
 * 換頁控制列沉底常駐（不捲動即可換頁）。 */
export function SpecList({ specs, onOpen }: SpecListProps) {
  const { t } = useI18n();
  // 搜尋字串留元件內——規格頁無跨視圖保留需求（比對規則共用 matchesQuery）。
  const [query, setQuery] = useState("");
  // 頁碼 state 以 min(page, pageCount) 鉗制派生——清單縮短不停在越界頁。
  const [rawPage, setRawPage] = useState(1);
  // 內部捲動容器 ref——換頁後歸位（清單自己捲、頁面不捲）。
  const scrollRef = useRef<HTMLDivElement>(null);

  const sorted = useMemo(
    () =>
      [...specs].sort((a, b) => {
        // modifiedAt 降冪；缺席者一律殿後；同值（含皆缺席）以名稱字母升冪決勝。
        if (a.modifiedAt && b.modifiedAt && a.modifiedAt !== b.modifiedAt)
          return a.modifiedAt < b.modifiedAt ? 1 : -1;
        if (!!a.modifiedAt !== !!b.modifiedAt) return a.modifiedAt ? -1 : 1;
        return a.id.localeCompare(b.id);
      }),
    [specs],
  );
  const filtered = sorted.filter((s) => matchesQuery(query, s.id));
  const pageCount = Math.max(1, Math.ceil(filtered.length / PAGE_SIZE));
  const page = Math.min(rawPage, pageCount);
  const pageItems = filtered.slice((page - 1) * PAGE_SIZE, page * PAGE_SIZE);

  const goPage = (next: number) => {
    setRawPage(next);
    if (scrollRef.current) scrollRef.current.scrollTop = 0;
  };

  return (
    <div className="flex h-full min-h-0 flex-col gap-3 max-w-3xl mx-auto w-full">
      <Input
        placeholder={t("specs.searchPlaceholder")}
        value={query}
        onChange={(e) => {
          setQuery(e.target.value);
          setRawPage(1);
        }}
      />
      <div className="flex items-center gap-2">
        <h2 className="text-base font-semibold">{t("specs.heading")}</h2>
        <span className="inline-flex items-center justify-center min-w-5 h-5 px-1.5 rounded-full bg-muted text-muted-foreground text-xs font-medium tabular-nums">
          {filtered.length}
        </span>
      </div>
      <div ref={scrollRef} data-list-scroll className="flex flex-1 min-h-0 flex-col gap-2.5 overflow-y-auto">
        {specs.length === 0 ? (
          <div className="text-muted-foreground text-sm py-8 text-center">{t("specs.empty")}</div>
        ) : filtered.length === 0 ? (
          <div className="text-muted-foreground text-sm py-8 text-center">{t("specs.noResults")}</div>
        ) : (
          pageItems.map((s) => <SpecCard key={s.id} item={s} onOpen={onOpen} />)
        )}
      </div>
      <ListPager page={page} pageCount={pageCount} onPage={goPage} />
    </div>
  );
}

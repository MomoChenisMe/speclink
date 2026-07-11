import { useState } from "react";
import { Check, Copy, FileText, History } from "lucide-react";

import type { SpecItem } from "../adapter";
import { useI18n } from "../i18n";
import { matchesQuery } from "../search";
import { relativeDays } from "../time";
import { Input } from "./ui/input";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";

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
          <div className="mt-1 text-[11px] font-medium text-amber-600 dark:text-amber-500">
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
 * 子字串、純前端即打即濾）；點卡片開抽屜檢視全文，無行內展開、無任何規格寫入動詞。 */
export function SpecList({ specs, onOpen }: SpecListProps) {
  const { t } = useI18n();
  // 搜尋字串留元件內——規格頁無跨視圖保留需求（比對規則共用 matchesQuery）。
  const [query, setQuery] = useState("");
  const filtered = specs.filter((s) => matchesQuery(query, s.id));
  return (
    <div className="flex flex-col gap-3 max-w-3xl mx-auto w-full">
      <Input
        placeholder={t("specs.searchPlaceholder")}
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      <div className="flex items-center gap-2">
        <h2 className="text-base font-semibold">{t("specs.heading")}</h2>
        <span className="inline-flex items-center justify-center min-w-5 h-5 px-1.5 rounded-full bg-muted text-muted-foreground text-xs font-medium tabular-nums">
          {filtered.length}
        </span>
      </div>
      <div className="flex flex-col gap-2.5">
        {specs.length === 0 ? (
          <div className="text-muted-foreground text-sm py-8 text-center">{t("specs.empty")}</div>
        ) : filtered.length === 0 ? (
          <div className="text-muted-foreground text-sm py-8 text-center">{t("specs.noResults")}</div>
        ) : (
          filtered.map((s) => <SpecCard key={s.id} item={s} onOpen={onOpen} />)
        )}
      </div>
    </div>
  );
}

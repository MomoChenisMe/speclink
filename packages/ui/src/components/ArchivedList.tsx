import { useState } from "react";
import { Check, Code2, Copy, GitFork, MessageSquareText } from "lucide-react";

import type { ArchivedItem, DiscussionItem } from "../adapter";
import { useI18n } from "../i18n";
import { matchesQuery } from "../search";
import { Badge } from "./ui/badge";
import { Input } from "./ui/input";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";
import type { ArchivedTarget } from "./ArchivedDrawer";

/** 標題後緊跟的複製鈕（design D7 卡片版面）：hover 顯現、copied 打勾回饋、
 * 點擊不冒泡（不開抽屜）。 */
function CopyButton({ value, label }: { value: string; label: string }) {
  const { t } = useI18n();
  const [copied, setCopied] = useState(false);
  const copy = (e: React.MouseEvent) => {
    e.stopPropagation();
    void navigator.clipboard?.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };
  return (
    <span
      role="button"
      aria-label={copied ? t("specs.copied") : label}
      className={`shrink-0 text-muted-foreground transition-opacity hover:text-foreground ${copied ? "opacity-100" : "opacity-0 group-hover:opacity-100"}`}
      onClick={copy}
    >
      {copied ? <Check className="h-3.5 w-3.5 text-primary" /> : <Copy className="h-3.5 w-3.5" />}
    </span>
  );
}

/** 封存變更卡（design D7）：日期＋標題＋複製鈕成群組、meta 靠右——任務徽章
 *（全完成靜默、未全完成琥珀警示：「沒做完就封存」才是需要被看見的異常）、
 * 觸及規格數、createdBy 頭像圓點（與 ChangeCard 同款）、來源討論標記。
 * 點整列開唯讀抽屜，無行內展開。 */
function ArchivedCard({ item, onOpen }: { item: ArchivedItem; onOpen: (target: ArchivedTarget) => void }) {
  const { t } = useI18n();
  const badge =
    item.tasksTotal != null && item.tasksDone != null ? `${item.tasksDone}/${item.tasksTotal}` : null;
  const incomplete = badge != null && item.tasksDone! < item.tasksTotal!;
  const specCount = item.specCount ?? 0;
  const discussions = item.fromDiscussions ?? [];
  return (
    <div
      data-archived={item.datedName}
      className="group cursor-pointer rounded-lg border border-border bg-card p-3 transition-[border-color,box-shadow] hover:border-primary/60 hover:shadow-md"
      onClick={() => onOpen({ kind: "change", datedName: item.datedName })}
    >
      <div className="flex items-center gap-2.5">
        <span className="shrink-0 text-xs text-muted-foreground tabular-nums">{item.date}</span>
        <span data-title-group className="flex min-w-0 flex-1 items-center gap-1">
          <span className="min-w-0 truncate text-sm font-medium">{item.name}</span>
          <CopyButton value={item.datedName} label={t("archived.copyName")} />
        </span>
        <span className="flex shrink-0 items-center gap-2 text-xs text-muted-foreground">
          {badge && (
            <Badge
              variant="secondary"
              className={`shrink-0 tabular-nums ${incomplete ? "bg-amber-500/15 text-amber-600 dark:text-amber-500" : ""}`}
            >
              {badge}
            </Badge>
          )}
          {specCount > 0 && (
            <Tooltip>
              <TooltipTrigger asChild>
                <span
                  aria-label={t("archived.specCount").replace("{n}", String(specCount))}
                  className="inline-flex items-center gap-1 tabular-nums"
                >
                  <Code2 className="h-3 w-3" />
                  {specCount}
                </span>
              </TooltipTrigger>
              <TooltipContent>{t("archived.specCount").replace("{n}", String(specCount))}</TooltipContent>
            </Tooltip>
          )}
          {item.createdBy && (
            <Tooltip>
              <TooltipTrigger asChild>
                <span
                  aria-label={item.createdBy}
                  className="inline-flex h-4 w-4 shrink-0 items-center justify-center rounded-full bg-primary text-[9px] font-bold text-primary-foreground"
                >
                  {item.createdBy.charAt(0).toUpperCase()}
                </span>
              </TooltipTrigger>
              <TooltipContent>{item.createdBy}</TooltipContent>
            </Tooltip>
          )}
          {discussions.length > 0 && (
            <Tooltip>
              <TooltipTrigger asChild>
                <span aria-label={t("card.fromDiscussion")} className="shrink-0 text-primary/60">
                  <MessageSquareText className="h-3.5 w-3.5" />
                </span>
              </TooltipTrigger>
              <TooltipContent>
                {t("card.fromDiscussionTitle").replace("{name}", discussions.join(", "))}
              </TooltipContent>
            </Tooltip>
          )}
        </span>
      </div>
    </div>
  );
}

/** 封存討論卡（design D7）：日期＋topic＋複製 slug 鈕成群組、meta＝「N 輪」＋
 * 衍生變更數徽章（自既有 promotedTo 長度派生）。點整列開唯讀抽屜。 */
function ArchivedDiscussionCard({
  item,
  onOpen,
}: {
  item: DiscussionItem;
  onOpen: (target: ArchivedTarget) => void;
}) {
  const { t } = useI18n();
  const promoted = item.promotedTo.length;
  return (
    <div
      data-archived-discussion={item.slug}
      className="group cursor-pointer rounded-lg border border-border bg-card p-3 transition-[border-color,box-shadow] hover:border-primary/60 hover:shadow-md"
      onClick={() => onOpen({ kind: "discussion", slug: item.slug })}
    >
      <div className="flex items-center gap-2.5">
        <span className="shrink-0 text-xs text-muted-foreground tabular-nums">{item.created}</span>
        <span data-title-group className="flex min-w-0 flex-1 items-center gap-1">
          <span className="min-w-0 truncate text-sm font-medium">{item.topic}</span>
          <CopyButton value={item.slug} label={t("discussion.copySlug")} />
        </span>
        <span className="flex shrink-0 items-center gap-2 text-xs text-muted-foreground">
          <span className="tabular-nums">{t("common.rounds").replace("{n}", String(item.rounds))}</span>
          {promoted > 0 && (
            <Tooltip>
              <TooltipTrigger asChild>
                <span
                  aria-label={t("archived.promotedCount").replace("{n}", String(promoted))}
                  className="inline-flex items-center gap-1 tabular-nums"
                >
                  <GitFork className="h-3 w-3" />
                  {promoted}
                </span>
              </TooltipTrigger>
              <TooltipContent>
                {t("archived.promotedCount").replace("{n}", String(promoted))}
              </TooltipContent>
            </Tooltip>
          )}
        </span>
      </div>
    </div>
  );
}

export interface ArchivedListProps {
  archived: ArchivedItem[];
  query: string;
  onQuery: (q: string) => void;
  /** 封存討論（討論節；缺席時不顯示該節，向後相容）。 */
  archivedDiscussions?: DiscussionItem[];
  /** 點卡片開唯讀封存抽屜（discriminated target：封存變更或封存討論）。 */
  onOpen: (target: ArchivedTarget) => void;
}

/** 已封存獨立頁（design D7 雙節）：兩節皆為卡片清單、點卡開抽屜、無行內展開；
 * 搜尋同時過濾「變更」與「討論」兩節。 */
export function ArchivedList({ archived, query, onQuery, archivedDiscussions, onOpen }: ArchivedListProps) {
  const { t } = useI18n();
  // 比對規則共用 matchesQuery（與看板一致的單一真相）。
  const filtered = archived.filter((a) => matchesQuery(query, a.name));
  const discussions = (archivedDiscussions ?? []).filter((d) => matchesQuery(query, d.topic, d.slug));
  const showDiscussions = archivedDiscussions !== undefined;
  return (
    <TooltipProvider>
      <div className="flex flex-col gap-3 max-w-3xl mx-auto w-full">
        <Input placeholder={t("archived.searchPlaceholder")} value={query} onChange={(e) => onQuery(e.target.value)} />
        <div className="flex items-center gap-2">
          <h2 className="text-base font-semibold">{t("archived.changesHeading")}</h2>
          <span className="inline-flex items-center justify-center min-w-5 h-5 px-1.5 rounded-full bg-muted text-muted-foreground text-xs font-medium tabular-nums">
            {filtered.length}
          </span>
        </div>
        <div className="flex flex-col gap-2.5">
          {filtered.length === 0 ? (
            <div className="text-muted-foreground text-sm py-8 text-center">{t("archived.noChanges")}</div>
          ) : (
            filtered.map((a) => <ArchivedCard key={a.datedName} item={a} onOpen={onOpen} />)
          )}
        </div>
        {showDiscussions && (
          <>
            <div className="flex items-center gap-2 pt-2">
              <h2 className="text-base font-semibold">{t("archived.discussionsHeading")}</h2>
              <span className="inline-flex items-center justify-center min-w-5 h-5 px-1.5 rounded-full bg-muted text-muted-foreground text-xs font-medium tabular-nums">
                {discussions.length}
              </span>
            </div>
            <div className="flex flex-col gap-2.5">
              {discussions.length === 0 ? (
                <div className="text-muted-foreground text-sm py-8 text-center">{t("archived.noDiscussions")}</div>
              ) : (
                discussions.map((d) => <ArchivedDiscussionCard key={d.slug} item={d} onOpen={onOpen} />)
              )}
            </div>
          </>
        )}
      </div>
    </TooltipProvider>
  );
}

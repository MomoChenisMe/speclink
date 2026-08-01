import { useEffect, useMemo, useRef, useState } from "react";
import { Check, Code2, Copy, GitFork, MessageSquareText } from "lucide-react";

import type { ArchivedItem, DiscussionItem } from "../adapter";
import { useI18n } from "../i18n";
import { matchesQuery } from "../search";
import { Badge } from "./ui/badge";
import { Input } from "./ui/input";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";
import type { ArchivedTarget } from "./ArchivedDrawer";
import { ListPager, PAGE_SIZE } from "./ListPager";
import { REVIEW_ICON, REVIEW_LABEL_KEY, REVIEW_TONE } from "./reviewStyle";

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
          {/* 審查結局標示（spec「已封存側的審查標示」）：帶章＝已審查、化石工單
              ＝曾審查未通過（永久標示）、皆無＝無元素。 */}
          {(item.reviewStatus === "reviewed" || item.reviewStatus === "reviewedNotPassed") && (
            <Tooltip>
              <TooltipTrigger asChild>
                <span
                  aria-label={t(REVIEW_LABEL_KEY[item.reviewStatus])}
                  className={`shrink-0 ${REVIEW_TONE[item.reviewStatus]}`}
                >
                  {REVIEW_ICON[item.reviewStatus]}
                </span>
              </TooltipTrigger>
              <TooltipContent>{t(REVIEW_LABEL_KEY[item.reviewStatus])}</TooltipContent>
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

/** 子頁籤標籤上的筆數徽章——沿用頁面計數 pill 樣式。 */
const COUNT_PILL_CLS =
  "inline-flex items-center justify-center min-w-5 h-5 px-1.5 rounded-full bg-muted text-muted-foreground text-xs font-medium tabular-nums";

/** 已封存獨立頁（design D3 子頁籤）：搜尋框頂置、其下「變更」「討論」兩子頁籤
 * 各帶過濾後筆數徽章；兩節皆為卡片清單、點卡開抽屜、無行內展開；搜尋同時過濾
 * 兩節、兩子頁籤頁碼互相獨立（spec「已封存頁含討論節」「清單最新在前與換頁瀏覽」）。
 * 清單最新在前：封存變更依 datedName 字典序降冪、封存討論依 created 降冪同日
 * slug 升冪；archivedDiscussions 缺席（向後相容路徑）時子頁籤列缺席。
 * 版面填滿視窗高度：搜尋框與子頁籤列固定頂部、卡片清單於內部容器捲動、
 * 換頁控制列沉底常駐（不捲動即可換頁）。 */
export function ArchivedList({ archived, query, onQuery, archivedDiscussions, onOpen }: ArchivedListProps) {
  const { t } = useI18n();
  // 兩子頁籤頁碼互相獨立；以 min(page, pageCount) 鉗制派生，清單縮短不停在越界頁。
  const [changeRawPage, setChangeRawPage] = useState(1);
  const [discRawPage, setDiscRawPage] = useState(1);
  // 填滿高度版面：卡片清單於內部容器捲動、換頁控制列沉底常駐，兩子頁籤各自
  // 持有捲動容器 ref 供換頁歸位（spec「清單最新在前與換頁瀏覽」）。
  const changeScrollRef = useRef<HTMLDivElement>(null);
  const discScrollRef = useRef<HTMLDivElement>(null);

  // 搜尋字串變更（query 為外部受控 prop）：兩側頁碼皆回第 1 頁。
  useEffect(() => {
    setChangeRawPage(1);
    setDiscRawPage(1);
  }, [query]);

  // datedName 前綴 YYYY-MM-DD 使字典序＝時間序，降冪即封存日期新→舊（同日由字串降冪涵蓋）。
  const sortedChanges = useMemo(
    () => [...archived].sort((a, b) => (a.datedName < b.datedName ? 1 : a.datedName > b.datedName ? -1 : 0)),
    [archived],
  );
  // created 降冪；同日以 slug 字母升冪決勝。
  const sortedDiscussions = useMemo(
    () =>
      [...(archivedDiscussions ?? [])].sort((a, b) => {
        if (a.created !== b.created) return a.created < b.created ? 1 : -1;
        return a.slug.localeCompare(b.slug);
      }),
    [archivedDiscussions],
  );

  // 比對規則共用 matchesQuery（與看板一致的單一真相）。
  const filtered = sortedChanges.filter((a) => matchesQuery(query, a.name));
  const discussions = sortedDiscussions.filter((d) => matchesQuery(query, d.topic, d.slug));
  const showDiscussions = archivedDiscussions !== undefined;

  const changePageCount = Math.max(1, Math.ceil(filtered.length / PAGE_SIZE));
  const changePage = Math.min(changeRawPage, changePageCount);
  const changeItems = filtered.slice((changePage - 1) * PAGE_SIZE, changePage * PAGE_SIZE);
  const discPageCount = Math.max(1, Math.ceil(discussions.length / PAGE_SIZE));
  const discPage = Math.min(discRawPage, discPageCount);
  const discItems = discussions.slice((discPage - 1) * PAGE_SIZE, discPage * PAGE_SIZE);

  // 換頁後內部捲動容器捲回頂部（清單自己捲、頁面不捲）。
  const resetScroll = (ref: React.RefObject<HTMLDivElement | null>) => {
    if (ref.current) ref.current.scrollTop = 0;
  };

  const changesPane = (
    <>
      <div ref={changeScrollRef} data-list-scroll className="flex flex-1 min-h-0 flex-col gap-2.5 overflow-y-auto">
        {filtered.length === 0 ? (
          <div className="text-muted-foreground text-sm py-8 text-center">{t("archived.noChanges")}</div>
        ) : (
          changeItems.map((a) => <ArchivedCard key={a.datedName} item={a} onOpen={onOpen} />)
        )}
      </div>
      <ListPager
        page={changePage}
        pageCount={changePageCount}
        onPage={(n) => {
          setChangeRawPage(n);
          resetScroll(changeScrollRef);
        }}
      />
    </>
  );

  return (
    <TooltipProvider>
      <div className="flex h-full min-h-0 flex-col gap-3 max-w-3xl mx-auto w-full">
        <Input placeholder={t("archived.searchPlaceholder")} value={query} onChange={(e) => onQuery(e.target.value)} />
        {showDiscussions ? (
          <Tabs defaultValue="changes" className="flex flex-1 min-h-0 flex-col gap-3">
            <TabsList>
              <TabsTrigger value="changes">
                {t("archived.changesHeading")}
                <span className={COUNT_PILL_CLS}>{filtered.length}</span>
              </TabsTrigger>
              <TabsTrigger value="discussions">
                {t("archived.discussionsHeading")}
                <span className={COUNT_PILL_CLS}>{discussions.length}</span>
              </TabsTrigger>
            </TabsList>
            <TabsContent value="changes" className="flex flex-1 min-h-0 flex-col gap-3">
              {changesPane}
            </TabsContent>
            <TabsContent value="discussions" className="flex flex-1 min-h-0 flex-col gap-3">
              <div ref={discScrollRef} data-list-scroll className="flex flex-1 min-h-0 flex-col gap-2.5 overflow-y-auto">
                {discussions.length === 0 ? (
                  <div className="text-muted-foreground text-sm py-8 text-center">{t("archived.noDiscussions")}</div>
                ) : (
                  discItems.map((d) => <ArchivedDiscussionCard key={d.slug} item={d} onOpen={onOpen} />)
                )}
              </div>
              <ListPager
                page={discPage}
                pageCount={discPageCount}
                onPage={(n) => {
                  setDiscRawPage(n);
                  resetScroll(discScrollRef);
                }}
              />
            </TabsContent>
          </Tabs>
        ) : (
          <>
            {/* 向後相容路徑：無討論清單資料，維持原「已封存的變更」標題＋計數。 */}
            <div className="flex items-center gap-2">
              <h2 className="text-base font-semibold">{t("archived.changesHeading")}</h2>
              <span className={COUNT_PILL_CLS}>{filtered.length}</span>
            </div>
            {changesPane}
          </>
        )}
      </div>
    </TooltipProvider>
  );
}

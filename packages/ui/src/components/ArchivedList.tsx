import { useEffect, useMemo, useRef, useState } from "react";
import { Code2, GitFork, MessageSquareText } from "lucide-react";

import type { ArchivedItem, DiscussionItem } from "../adapter";
import { useI18n } from "../i18n";
import { matchesQuery } from "../search";
import { SEMANTIC_SURFACE, SEMANTIC_TONE } from "../tone";
import { Badge } from "./ui/badge";
import { PageHeader } from "./ui/page-header";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";
import type { ArchivedTarget } from "./ArchivedDrawer";
import { ImproveStamp } from "./ImproveStamp";
import { isImproveKind } from "./improveStyle";
import { ListCard } from "./ListCard";
import { ListPager, PAGE_SIZE } from "./ListPager";
import { ListRow } from "./ListRow";
import { REVIEW_ICON, REVIEW_LABEL_KEY, REVIEW_TONE } from "./reviewStyle";
import { SearchField } from "./SearchField";
import { VERIFY_ICON, VERIFY_LABEL_KEY, VERIFY_TONE } from "./verifyStyle";

/** 封存變更列（spec「規格與封存卡片收合資訊」；design D4）：日期在最左、等寬標題＋
 * 複製鈕、Why 首句描述列（缺席時列退回單行）、meta 為任務徽章（全完成靜默、未全完成
 * 琥珀警示：「沒做完就封存」才是需要被看見的異常）、觸及規格數、審查／驗證結局、
 * createdBy 頭像圓點、來源討論標記。點整列開唯讀抽屜。 */
function ArchivedRow({ item, onOpen }: { item: ArchivedItem; onOpen: (target: ArchivedTarget) => void }) {
  const { t } = useI18n();
  const badge =
    item.tasksTotal != null && item.tasksDone != null ? `${item.tasksDone}/${item.tasksTotal}` : null;
  const incomplete = badge != null && item.tasksDone! < item.tasksTotal!;
  const specCount = item.specCount ?? 0;
  const discussions = item.fromDiscussions ?? [];
  const specLabel = t("archived.specCount").replace("{n}", String(specCount));
  return (
    <ListRow
      data-archived={item.datedName}
      leading={item.date}
      title={item.name}
      copyValue={item.datedName}
      copyLabel={t("archived.copyName")}
      description={item.whyExcerpt}
      meta={
        <>
          {badge && (
            <Badge
              variant="secondary"
              className={`shrink-0 tabular-nums ${
                incomplete ? `${SEMANTIC_SURFACE.warning} ${SEMANTIC_TONE.warning}` : ""
              }`}
            >
              {badge}
            </Badge>
          )}
          {specCount > 0 && (
            <Tooltip>
              <TooltipTrigger asChild>
                <span aria-label={specLabel} className="inline-flex items-center gap-1 tabular-nums">
                  <Code2 className="h-3 w-3" />
                  {specCount}
                </span>
              </TooltipTrigger>
              <TooltipContent>{specLabel}</TooltipContent>
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
          {/* 驗證結局標示（spec「已封存側的驗證標示」）：與審查結局並存，順序固定在其後。 */}
          {(item.verifyStatus === "verified" || item.verifyStatus === "verifiedNotPassed") && (
            <Tooltip>
              <TooltipTrigger asChild>
                <span
                  aria-label={t(VERIFY_LABEL_KEY[item.verifyStatus])}
                  className={`shrink-0 ${VERIFY_TONE[item.verifyStatus]}`}
                >
                  {VERIFY_ICON[item.verifyStatus]}
                </span>
              </TooltipTrigger>
              <TooltipContent>{t(VERIFY_LABEL_KEY[item.verifyStatus])}</TooltipContent>
            </Tooltip>
          )}
          {item.createdBy && (
            <Tooltip>
              <TooltipTrigger asChild>
                <span
                  aria-label={item.createdBy}
                  className="inline-flex h-4 w-4 shrink-0 items-center justify-center rounded-full bg-muted text-[9px] font-bold text-muted-foreground"
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
        </>
      }
      onClick={() => onOpen({ kind: "change", datedName: item.datedName })}
    />
  );
}

/** 封存討論列（design D4）：slug 為等寬標題＋複製 slug 鈕、改進小章在標題旁、topic 降為
 * 描述列；meta 為日期、「N 輪」、衍生變更數（自既有 promotedTo 長度派生）——與看板討論卡
 * 同構（slug 為 CLI 動詞把手；LANGUAGE.md 受控例外）。點整列開唯讀抽屜。 */
function ArchivedDiscussionRow({
  item,
  onOpen,
}: {
  item: DiscussionItem;
  onOpen: (target: ArchivedTarget) => void;
}) {
  const { t } = useI18n();
  const promoted = item.promotedTo.length;
  const promotedLabel = t("archived.promotedCount").replace("{n}", String(promoted));
  return (
    <ListRow
      data-archived-discussion={item.slug}
      title={item.slug}
      copyValue={item.slug}
      copyLabel={t("discussion.copySlug")}
      description={item.topic}
      meta={
        <>
          <span className="tabular-nums">{item.created}</span>
          <span className="tabular-nums">{t("common.rounds").replace("{n}", String(item.rounds))}</span>
          {promoted > 0 && (
            <Tooltip>
              <TooltipTrigger asChild>
                <span aria-label={promotedLabel} className="inline-flex items-center gap-1 tabular-nums">
                  <GitFork className="h-3 w-3" />
                  {promoted}
                </span>
              </TooltipTrigger>
              <TooltipContent>{promotedLabel}</TooltipContent>
            </Tooltip>
          )}
        </>
      }
      onClick={() => onOpen({ kind: "discussion", slug: item.slug })}
    >
      {/* 改進小章：封存後標示不變（spec「已封存的改進討論維持標示」）。 */}
      {isImproveKind(item.kind) && <ImproveStamp />}
    </ListRow>
  );
}

export interface ArchivedListProps {
  archived: ArchivedItem[];
  query: string;
  onQuery: (q: string) => void;
  /** 封存討論（討論節；缺席時不顯示該節，向後相容）。 */
  archivedDiscussions?: DiscussionItem[];
  /** 點列開唯讀封存抽屜（discriminated target：封存變更或封存討論）。 */
  onOpen: (target: ArchivedTarget) => void;
  /** 頁標題區的標題；缺席時不渲染頁標題區，只有搜尋框。 */
  title?: string;
  /** 頁標題區的灰字說明。 */
  description?: string;
}

/** 分頁標籤上的筆數徽章——沿用頁面計數 pill 樣式。 */
const COUNT_PILL_CLS =
  "inline-flex items-center justify-center min-w-5 h-5 px-1.5 rounded-full bg-muted text-muted-foreground text-xs font-medium tabular-nums";

type Section = "changes" | "discussions";

const EMPTY_CLS = "py-8 text-center text-sm text-muted-foreground";

/** 已封存頁（spec「已封存頁含討論節」；design D4）：頁標題區（標題、說明、右端全圓
 * 搜尋框）之下為一張列表卡，卡頂為卡片標頭式分頁「變更」「討論」各帶過濾後筆數徽章；
 * 兩節皆為列式清單、點列開抽屜、無行內展開；搜尋同時過濾兩節、兩節頁碼與每頁筆數
 * 互相獨立（spec「清單最新在前與換頁瀏覽」）。清單最新在前：封存變更依 datedName
 * 字典序降冪、封存討論依 created 降冪同日 slug 升冪；archivedDiscussions 缺席（向後
 * 相容路徑）時分頁列缺席、列表卡維持完整圓角。版面填滿主區高度：頁標題區與分頁列
 * 固定、列於卡內捲動、工具列固定卡底。 */
export function ArchivedList({
  archived,
  query,
  onQuery,
  archivedDiscussions,
  onOpen,
  title,
  description,
}: ArchivedListProps) {
  const { t } = useI18n();
  const [section, setSection] = useState<Section>("changes");
  // 兩節頁碼與每頁筆數互相獨立；頁碼以 min(page, pageCount) 鉗制派生，清單縮短不停在越界頁。
  const [changeRawPage, setChangeRawPage] = useState(1);
  const [changePageSize, setChangePageSize] = useState(PAGE_SIZE);
  const [discRawPage, setDiscRawPage] = useState(1);
  const [discPageSize, setDiscPageSize] = useState(PAGE_SIZE);
  // 兩節共用列表卡的捲動容器——換頁後歸位（清單自己捲、頁面不捲）。
  const scrollRef = useRef<HTMLDivElement>(null);

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

  const changePageCount = Math.max(1, Math.ceil(filtered.length / changePageSize));
  const changePage = Math.min(changeRawPage, changePageCount);
  const changeItems = filtered.slice((changePage - 1) * changePageSize, changePage * changePageSize);
  const discPageCount = Math.max(1, Math.ceil(discussions.length / discPageSize));
  const discPage = Math.min(discRawPage, discPageCount);
  const discItems = discussions.slice((discPage - 1) * discPageSize, discPage * discPageSize);

  // 換頁後內部捲動容器捲回頂部（清單自己捲、頁面不捲）。
  const resetScroll = () => {
    if (scrollRef.current) scrollRef.current.scrollTop = 0;
  };
  // 改每頁筆數：重算總頁數並把頁碼鉗制到末頁（寫回 state，改回原筆數時不彈回舊頁）。
  const clampTo = (count: number, size: number) => (p: number) => Math.min(p, Math.max(1, Math.ceil(count / size)));

  const changesPager = (
    <ListPager
      page={changePage}
      pageCount={changePageCount}
      total={filtered.length}
      pageSize={changePageSize}
      onPage={(n) => {
        setChangeRawPage(n);
        resetScroll();
      }}
      onPageSize={(size) => {
        setChangePageSize(size);
        setChangeRawPage(clampTo(filtered.length, size));
        resetScroll();
      }}
    />
  );
  const discussionsPager = (
    <ListPager
      page={discPage}
      pageCount={discPageCount}
      total={discussions.length}
      pageSize={discPageSize}
      onPage={(n) => {
        setDiscRawPage(n);
        resetScroll();
      }}
      onPageSize={(size) => {
        setDiscPageSize(size);
        setDiscRawPage(clampTo(discussions.length, size));
        resetScroll();
      }}
    />
  );

  const changesRows =
    filtered.length === 0 ? (
      <div className={EMPTY_CLS}>{t("archived.noChanges")}</div>
    ) : (
      changeItems.map((a) => <ArchivedRow key={a.datedName} item={a} onOpen={onOpen} />)
    );

  const search = (
    <SearchField value={query} placeholder={t("archived.searchPlaceholder")} onChange={onQuery} />
  );

  return (
    <TooltipProvider>
      <div className="flex h-full min-h-0 w-full flex-col gap-4">
        {title ? <PageHeader title={title} description={description} actions={search} /> : search}
        {showDiscussions ? (
          <Tabs
            value={section}
            onValueChange={(v) => {
              setSection(v as Section);
              resetScroll();
            }}
            className="flex min-h-0 flex-1 flex-col"
          >
            <ListCard
              scrollRef={scrollRef}
              header={
                <TabsList variant="card">
                  <TabsTrigger value="changes">
                    {t("archived.changesHeading")}
                    <span className={COUNT_PILL_CLS}>{filtered.length}</span>
                  </TabsTrigger>
                  <TabsTrigger value="discussions">
                    {t("archived.discussionsHeading")}
                    <span className={COUNT_PILL_CLS}>{discussions.length}</span>
                  </TabsTrigger>
                </TabsList>
              }
              footer={section === "changes" ? changesPager : discussionsPager}
            >
              <TabsContent value="changes">{changesRows}</TabsContent>
              <TabsContent value="discussions">
                {discussions.length === 0 ? (
                  <div className={EMPTY_CLS}>{t("archived.noDiscussions")}</div>
                ) : (
                  discItems.map((d) => <ArchivedDiscussionRow key={d.slug} item={d} onOpen={onOpen} />)
                )}
              </TabsContent>
            </ListCard>
          </Tabs>
        ) : (
          // 向後相容路徑：無討論清單資料，列表卡不帶分頁列、只有變更清單。
          <ListCard scrollRef={scrollRef} footer={changesPager}>
            {changesRows}
          </ListCard>
        )}
      </div>
    </TooltipProvider>
  );
}

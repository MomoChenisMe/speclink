import { useEffect, useMemo, useRef, useState } from "react";
import { FileText, History } from "lucide-react";

import type { SpecItem } from "../adapter";
import { useI18n } from "../i18n";
import { matchesQuery } from "../search";
import { relativeDays } from "../time";
import { SEMANTIC_TONE } from "../tone";
import { PageHeader } from "./ui/page-header";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";
import { ListCard } from "./ListCard";
import { ListPager, PAGE_SIZE } from "./ListPager";
import { ListRow } from "./ListRow";
import { SearchField } from "./SearchField";

/** 規格列（spec「規格與封存卡片收合資訊」；design D3）：等寬名稱＋複製鈕、描述列為
 * Purpose 摘要一行截斷（佔位時改顯琥珀「Purpose 待補」警示）、meta 為需求數、溯源
 * 變更數（>0 才顯示）與相對修改時間。點整列開唯讀規格檢視，無行內展開。 */
function SpecRow({ item, onOpen }: { item: SpecItem; onOpen: (capability: string) => void }) {
  const { t } = useI18n();
  const rel = relativeDays(item.modifiedAt, t);
  const reqCount = item.requirementCount ?? 0;
  const traceCount = item.traceCount ?? 0;
  const reqLabel = t("specs.requirementCount").replace("{n}", String(reqCount));
  const traceLabel = t("specs.traceCount").replace("{n}", String(traceCount));
  return (
    <ListRow
      data-spec={item.id}
      title={item.id}
      copyValue={item.id}
      copyLabel={t("common.copyName")}
      description={
        item.purposeTbd ? (
          <span className={`font-medium ${SEMANTIC_TONE.warning}`}>{t("specs.purposeTbd")}</span>
        ) : (
          (item.purposeExcerpt ?? undefined)
        )
      }
      meta={
        <>
          <Tooltip>
            <TooltipTrigger asChild>
              <span aria-label={reqLabel} className="inline-flex items-center gap-1 tabular-nums">
                <FileText className="h-3 w-3" />
                {reqCount}
              </span>
            </TooltipTrigger>
            <TooltipContent>{reqLabel}</TooltipContent>
          </Tooltip>
          {traceCount > 0 && (
            <Tooltip>
              <TooltipTrigger asChild>
                <span aria-label={traceLabel} className="inline-flex items-center gap-1 tabular-nums">
                  <History className="h-3 w-3" />
                  {traceCount}
                </span>
              </TooltipTrigger>
              <TooltipContent>{traceLabel}</TooltipContent>
            </Tooltip>
          )}
          {rel && <span className="tabular-nums">{rel}</span>}
        </>
      }
      onClick={() => onOpen(item.id)}
    />
  );
}

export interface SpecListProps {
  specs: SpecItem[];
  /** 點列開唯讀規格檢視（capability 定址）。 */
  onOpen: (capability: string) => void;
  /** 外部指定聚焦的 capability（手冊出處跳規格）：翻到該列所在頁並捲至該列；
   * 搜尋字串遮住它時清空搜尋。 */
  focus?: string | null;
  /** 頁標題區的標題；缺席時不渲染頁標題區，只有搜尋框。 */
  title?: string;
  /** 頁標題區的灰字說明。 */
  description?: string;
}

/** 規格頁（spec「規格頁提供清單、搜尋與展開檢視」；design D3）：頁標題區（標題、說明、
 * 右端全圓搜尋框——大小寫不敏感子字串、純前端即打即濾）之下為一張列表卡，一列一份
 * 正式規格；點列開檢視，無行內展開、無任何規格寫入動詞。清單最新在前（modifiedAt
 * 降冪、缺席殿後、名稱升冪決勝）並依每頁筆數換頁（預設 20、不持久化）——排序與換頁
 * 純屬呈現層。版面填滿主區高度：頁標題區固定頂部、列於卡內捲動、工具列固定卡底。 */
export function SpecList({ specs, onOpen, focus, title, description }: SpecListProps) {
  const { t } = useI18n();
  // 搜尋字串留元件內——規格頁無跨視圖保留需求（比對規則共用 matchesQuery）。
  const [query, setQuery] = useState("");
  // 頁碼 state 以 min(page, pageCount) 鉗制派生——清單縮短不停在越界頁。
  const [rawPage, setRawPage] = useState(1);
  const [pageSize, setPageSize] = useState(PAGE_SIZE);
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
  const pageCount = Math.max(1, Math.ceil(filtered.length / pageSize));
  const page = Math.min(rawPage, pageCount);
  const pageItems = filtered.slice((page - 1) * pageSize, page * pageSize);

  useEffect(() => {
    if (!focus) return;
    const visible = filtered.some((s) => s.id === focus);
    const at = (visible ? filtered : sorted).findIndex((s) => s.id === focus);
    if (at < 0) return;
    if (!visible) setQuery("");
    setRawPage(Math.floor(at / pageSize) + 1);
    // 換頁後列才在 DOM：下一幀再捲至該列。
    const frame = requestAnimationFrame(() => {
      scrollRef.current?.querySelector(`[data-spec="${focus}"]`)?.scrollIntoView({ block: "nearest" });
    });
    return () => cancelAnimationFrame(frame);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [focus]);

  const resetScroll = () => {
    if (scrollRef.current) scrollRef.current.scrollTop = 0;
  };
  const goPage = (next: number) => {
    setRawPage(next);
    resetScroll();
  };
  // 改每頁筆數：重算總頁數並把頁碼鉗制到末頁（寫回 state，改回原筆數時不彈回舊頁）。
  const changePageSize = (size: number) => {
    setPageSize(size);
    setRawPage((p) => Math.min(p, Math.max(1, Math.ceil(filtered.length / size))));
    resetScroll();
  };

  const search = (
    <SearchField
      value={query}
      placeholder={t("specs.searchPlaceholder")}
      onChange={(value) => {
        setQuery(value);
        setRawPage(1);
      }}
    />
  );

  return (
    <TooltipProvider>
      <div className="flex h-full min-h-0 w-full flex-col gap-4">
        {title ? <PageHeader title={title} description={description} actions={search} /> : search}
        <ListCard
          scrollRef={scrollRef}
          footer={
            <ListPager
              page={page}
              pageCount={pageCount}
              total={filtered.length}
              pageSize={pageSize}
              onPage={goPage}
              onPageSize={changePageSize}
            />
          }
        >
          {specs.length === 0 ? (
            <div className="py-8 text-center text-sm text-muted-foreground">{t("specs.empty")}</div>
          ) : filtered.length === 0 ? (
            <div className="py-8 text-center text-sm text-muted-foreground">{t("specs.noResults")}</div>
          ) : (
            pageItems.map((s) => <SpecRow key={s.id} item={s} onOpen={onOpen} />)
          )}
        </ListCard>
      </div>
    </TooltipProvider>
  );
}

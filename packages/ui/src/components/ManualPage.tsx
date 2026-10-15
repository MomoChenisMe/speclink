import { useEffect, useRef, useState, type MouseEvent, type ReactNode } from "react";
import { BookOpen, ChevronLeft, ChevronRight, CloudOff } from "lucide-react";

import type { ManualIndex, ManualPageItem } from "../adapter";
import { useI18n } from "../i18n";
import { cn } from "../lib/utils";
import { countHeadings, manualLinkSlug, splitLeadingHeading, stripSourcesLine } from "../manualDoc";
import { parseSpecSections } from "../specSections";
import { calendarDay, traceSpan, type TracePoint } from "../specSummary";
import type { DocumentRef, DocumentTarget } from "./DetailHeader";
import { DocBody, DocHeader, DocTiles, type DocTile } from "./DetailShell";
import { InlineMarks } from "./DocParts";
import { ManualToc } from "./ManualToc";
import { ManualTree } from "./ManualTree";
import { Markdown } from "./Markdown";
import { DocSkeleton, RowSkeleton } from "./skeletons";
import { Badge, badgeVariants } from "./ui/badge";
import { Button } from "./ui/button";
import { Callout } from "./ui/callout";
import { Card } from "./ui/card";
import { StatusPill } from "./ui/status-pill";

/** 手冊閱讀欄（design D5）：768px 置中；閱讀卡與底列內容同寬對齊。 */
const MANUAL_COLUMN_CLS = "mx-auto w-full max-w-[768px]";

/** 手冊內文的章節字級（desktop-spec-manual-pages design D6）：h2 16px、h3 14px，皆粗體。Streamdown 的
 * 標題尺寸在 utilities 層，後代選擇器的特異性就夠。 */
const MANUAL_HEADINGS_CLS =
  "[&_.markdown_h2]:text-base [&_.markdown_h2]:font-semibold [&_.markdown_h3]:text-sm [&_.markdown_h3]:font-semibold";

export interface ManualPageProps {
  /** 手冊索引（已依閱讀序排好）；null＝載入中（骨架）。 */
  index: ManualIndex | null;
  /** 讀取一頁去 frontmatter 的內文；不存在回 null、讀取失敗 reject。 */
  loadPage: (slug: string) => Promise<string | null>;
  /** 點頁尾出處的 capability：在手冊頁上開該規格的抽屜、不切頁（App 接線：store.openSpec）。 */
  onOpenSpec: (capability: string) => void;
  /** 正典 capability 清單——出處只對存在者可點，不存在者為純文字。 */
  capabilities: string[];
  /** 刷新世代——變動時重載目前頁內文（外部寫入後的 workspace-changed）。 */
  refreshGen?: number;
  /** 閱讀卡標題列右端的文件動作（「開啟 ▾」，對應 `openspec/manual/<slug>.md`）；未傳時缺席。 */
  renderDocumentActions?: (target: DocumentTarget | null) => ReactNode;
  /** 頁檔的實際相對路徑（宿主向 core 查詢，跟著專案的 spec 目錄名走）：出處行首段；缺席或查無時該段缺席。 */
  locateDocument?: (doc: DocumentRef) => Promise<{ relative: string } | null>;
  /** 讀一份正式規格全文：過期提示框取各出處規格的最近 @trace；缺席或失敗時該行退回不帶日期的句子。 */
  loadSpec?: (capability: string) => Promise<string | null>;
}

/** 已載入的頁內文；body 為 null＝載入失敗或頁不存在。 */
interface LoadedDoc {
  slug: string;
  body: string | null;
}

/** 上一頁／下一頁兩向的差異：DOM 錨點、aria-label、小字標籤鍵與對齊。 */
const PAGE_NAV = {
  prev: { anchor: "data-manual-prev", ariaKey: "pager.prev", labelKey: "manual.prevLabel", align: "items-start" },
  next: { anchor: "data-manual-next", ariaKey: "pager.next", labelKey: "manual.nextLabel", align: "items-end" },
} as const;

/** 底列的上一頁／下一頁框線鈕（design D5）：小字標籤＋目標頁標題；無目標時呼叫端不渲染。 */
function PageNavButton({
  direction,
  target,
  onSelect,
}: {
  direction: "prev" | "next";
  target: ManualPageItem;
  onSelect: (slug: string) => void;
}) {
  const { t } = useI18n();
  const nav = PAGE_NAV[direction];
  return (
    <Button
      variant="outline"
      size="sm"
      aria-label={t(nav.ariaKey)}
      className="h-auto gap-1.5 py-1"
      onClick={() => onSelect(target.slug)}
      {...{ [nav.anchor]: target.slug }}
    >
      {direction === "prev" && <ChevronLeft className="h-4 w-4" />}
      <span className={cn("flex flex-col leading-tight", nav.align)}>
        <span className="text-[11px] font-normal text-muted-foreground">{t(nav.labelKey)}</span>
        <span className="max-w-48 truncate">{target.title}</span>
      </span>
      {direction === "next" && <ChevronRight className="h-4 w-4" />}
    </Button>
  );
}

/** 出處籤：正典存在者為可點的框線籤（開該規格詳情頁），不存在者為純文字。出處行與底列共用。 */
function SourceChips({
  names,
  canonical,
  onOpenSpec,
}: {
  names: string[];
  canonical: Set<string>;
  onOpenSpec: (capability: string) => void;
}) {
  return names.map((name) =>
    canonical.has(name) ? (
      <button
        key={name}
        type="button"
        className={cn(badgeVariants({ variant: "outline" }), "cursor-(--cursor-interaction) font-mono hover:bg-muted")}
        onClick={() => onOpenSpec(name)}
      >
        {name}
      </button>
    ) : (
      <span key={name} className="font-mono">
        {name}
      </span>
    ),
  );
}

/** 過期提示框的讀取結果：該頁各出處規格的最近 @trace（讀不到或無 @trace 為 null）。 */
interface StaleSources {
  slug: string;
  latest: Record<string, TracePoint | null>;
}

/** 手冊頁（desktop-manual-page design「側欄樹、搜尋與上下頁在前端由索引推導」；
 * desktop-list-pages-reskin design D5 三欄：240px 目錄樹、768px 置中閱讀欄、200px 本頁
 * 目錄）：中欄以共用 Markdown 於白底閱讀卡內渲染選定頁——卡由上而下為標題區（頁標題、分區籤、
 * 「可能過期」籤、「開啟 ▾」、出處行）、數字磚、過期提示框與內文（desktop-spec-manual-pages
 * design D6）；白底上緣細線的底列（出處籤＋上一頁／下一頁）固定不隨內文捲動。本元件只管選頁、
 * 內文與過期提示框的載入（latest-wins）與切分；唯讀、無任何寫入操作。 */
export function ManualPage({
  index,
  loadPage,
  onOpenSpec,
  capabilities,
  refreshGen,
  renderDocumentActions,
  locateDocument,
  loadSpec,
}: ManualPageProps) {
  const { t } = useI18n();
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState<string | null>(null);
  const [doc, setDoc] = useState<LoadedDoc | null>(null);
  const [staleSources, setStaleSources] = useState<StaleSources | null>(null);
  const [location, setLocation] = useState<{ slug: string; relative: string } | null>(null);
  // 內文與過期提示框載入的 latest-wins 序號；載入函式以 ref 持有，宿主每次渲染換 lambda 不重載。
  const seq = useRef(0);
  const staleSeq = useRef(0);
  const loadPageRef = useRef(loadPage);
  loadPageRef.current = loadPage;
  const loadSpecRef = useRef(loadSpec);
  loadSpecRef.current = loadSpec;
  const locateRef = useRef(locateDocument);
  locateRef.current = locateDocument;

  const pages = index?.present ? index.pages : [];
  // 目前頁由索引現況推導：選定頁消失（外部刪除、索引重載）即回首頁。
  const current = pages.find((p) => p.slug === selected) ?? pages[0] ?? null;
  const currentSlug = current?.slug ?? null;
  // 出處名以索引 sources 為單一真相；manual-pages 契約：一項可帶 `#<Requirement 名>` 錨定，
  // 出處只看井號前的 capability 名——切掉錨定、去頭尾空白（讀取端也這麼做）後再去重
  //（防撞 React key，也防同 capability 多錨定重複）。
  const sourceNames = Array.from(new Set((current?.sources ?? []).map((s) => s.split("#", 1)[0].trim())));

  useEffect(() => {
    if (!currentSlug) {
      // 索引清空（切換 workspace）：丟棄舊內文與選頁，並作廢在途的載入——新 workspace
      // 的首頁 slug 多半同名（契約規定叫 index），舊內文不得當成新頁顯示或事後寫回。
      seq.current++;
      setDoc(null);
      setSelected(null);
      return;
    }
    const mine = ++seq.current;
    loadPageRef.current(currentSlug).then(
      (body) => {
        if (seq.current === mine) setDoc({ slug: currentSlug, body });
      },
      () => {
        if (seq.current === mine) setDoc({ slug: currentSlug, body: null });
      },
    );
  }, [currentSlug, refreshGen]);

  // 出處行首段的頁檔路徑：換頁向 core 查一次（頁檔位置不隨內文重載而變）；換頁或卸載作廢在途回應。
  // 索引清空（切換 workspace）時丟掉舊路徑：新專案的同名頁在自己的位置查到前不得沿用。
  useEffect(() => {
    const locate = locateRef.current;
    if (!currentSlug || !locate) {
      setLocation(null);
      return;
    }
    let live = true;
    void locate({ manual: currentSlug }).then(
      (found) => {
        if (live) setLocation(found && { slug: currentSlug, relative: found.relative });
      },
      () => undefined,
    );
    return () => {
      live = false;
    };
  }, [currentSlug]);

  // 過期提示框（design D6）：只在索引判為過期時讀各出處規格全文，取其最近 @trace；換頁作廢、
  // latest-wins。過期與否只看索引的 stale，不在這裡重判。sourceNames 每次渲染都是新陣列，
  // effect 依賴改用換行相接的字串（capability 名為 kebab-case、不含換行），內容不變就不重讀。
  const staleSlug = current?.stale ? current.slug : null;
  const staleCaps = current?.stale ? sourceNames.join("\n") : "";
  useEffect(() => {
    const mine = ++staleSeq.current;
    const load = loadSpecRef.current;
    if (!staleSlug || !load) {
      // 離開過期頁或索引清空（切換 workspace）：丟掉上一份出處日期——新專案的同名過期頁
      // 在自己的規格讀完前不得沿用。
      setStaleSources(null);
      return;
    }
    const read = (cap: string) =>
      load(cap).then(
        (md) => [cap, md ? traceSpan(parseSpecSections(md)).latest : null] as const,
        () => [cap, null] as const,
      );
    void Promise.all(staleCaps.split("\n").filter(Boolean).map(read)).then((entries) => {
      if (staleSeq.current === mine) setStaleSources({ slug: staleSlug, latest: Object.fromEntries(entries) });
    });
  }, [staleSlug, staleCaps, refreshGen]);

  const loaded = doc && current && doc.slug === current.slug ? doc : null;
  // 內文拆三段：尾端出處行剝掉（出處列改讀索引 sources）、開頭 H1 → 閱讀卡標題、其餘 → 內文。
  const parsed = loaded?.body != null ? splitLeadingHeading(stripSourcesLine(loaded.body)) : null;
  const markdownBody = parsed?.body ?? null;

  const bodyRef = useRef<HTMLDivElement | null>(null);
  // 換頁回頂；外部改內文的重載（refreshGen）不動捲動位置。
  useEffect(() => {
    if (bodyRef.current) bodyRef.current.scrollTop = 0;
  }, [currentSlug]);

  if (index === null) {
    return (
      <div className="flex h-full min-h-0">
        <aside className="flex w-60 shrink-0 flex-col gap-1 border-r border-border bg-sidebar p-3">
          <RowSkeleton />
          <RowSkeleton />
          <RowSkeleton />
        </aside>
        <div className="flex-1 p-6">
          <DocSkeleton />
        </div>
      </div>
    );
  }

  if (!current) {
    const remote = index.reason === "remote";
    return (
      <div
        data-manual-empty={remote ? "remote" : "none"}
        className="flex h-full flex-col items-center justify-center gap-3 p-5 text-center"
      >
        <BookOpen className="h-10 w-10 text-muted-foreground/40" />
        <h2 className="text-lg font-semibold">{remote ? t("manual.remoteTitle") : t("manual.emptyTitle")}</h2>
        <p className="max-w-md text-sm text-muted-foreground">
          {remote ? t("manual.remoteDesc") : t("manual.emptyDesc")}
        </p>
      </div>
    );
  }

  const position = pages.indexOf(current);
  const prev = position > 0 ? pages[position - 1] : null;
  const next = position < pages.length - 1 ? pages[position + 1] : null;
  const canonical = new Set(capabilities);
  const heading = parsed?.heading ?? current.title;
  // 數字磚（design D6）：內文未載入或載入失敗時整列缺席。
  const headingCounts = markdownBody !== null ? countHeadings(markdownBody) : null;
  const tiles: DocTile[] = headingCounts
    ? [
        { value: headingCounts.h2, label: t("manual.tileSections") },
        { value: headingCounts.h3, label: t("manual.tileSubsections") },
        { value: current.keywords.length, label: t("manual.tileKeywords") },
        { value: sourceNames.length, label: t("manual.tileSources") },
      ]
    : [];
  // 過期提示框：內文在、索引判為過期，且各出處規格讀完（沒有 loadSpec 時直接退回不帶日期的句子）。
  const staleLatest = staleSources?.slug === current.slug ? staleSources.latest : null;
  const staleCallout = current.stale && markdownBody !== null && (!loadSpec || staleLatest) && (
    <div data-doc-callout>
      <Callout data-callout="stale" tone="warning" title={t("manual.stale")} className="my-0">
        <ul className="my-1 flex flex-col gap-0.5 text-sm">
          {sourceNames.map((cap) => {
            const latest = staleLatest?.[cap];
            return (
              <li key={cap}>
                {latest
                  ? t("manual.staleCalloutSource")
                      .replace("{cap}", cap)
                      .replace("{date}", calendarDay(latest.updated))
                      .replace("{change}", latest.source)
                  : t("manual.staleCalloutSourceUnknown").replace("{cap}", cap)}
              </li>
            );
          })}
        </ul>
        {current.generated && (
          <p className="text-sm">{t("manual.staleCalloutGenerated").replace("{generated}", current.generated)}</p>
        )}
        <p className="text-sm text-muted-foreground">
          <InlineMarks text={t("manual.staleCalloutHint")} />
        </p>
      </Callout>
    </div>
  );

  // 內文的跨頁連結（契約：相對檔名 `layout.md`）在手冊內切頁——放給 WebView 直接導航
  // 會整頁離開 app；連到不存在的頁只擋下導航、停在原頁。其他 href 不攔。
  const onBodyClick = (e: MouseEvent<HTMLDivElement>) => {
    const anchor = (e.target as HTMLElement).closest("a");
    const slug = anchor ? manualLinkSlug(anchor.getAttribute("href") ?? "") : null;
    if (!slug) return;
    e.preventDefault();
    if (pages.some((p) => p.slug === slug)) setSelected(slug);
  };

  return (
    <div className="flex h-full min-h-0">
      <ManualTree
        pages={pages}
        currentSlug={current.slug}
        query={query}
        onQuery={setQuery}
        onSelect={setSelected}
        uncoveredCount={index.uncoveredNew.length}
      />

      <div data-manual-content className="flex min-h-0 flex-1">
        <div className="flex min-h-0 flex-1 flex-col">
          <div
            ref={bodyRef}
            data-manual-body
            className="min-h-0 flex-1 overflow-y-auto px-6 py-5"
            onClick={onBodyClick}
          >
            <Card data-manual-card className={`${MANUAL_COLUMN_CLS} flex flex-col gap-5 p-6`}>
              <DocHeader
                title={heading}
                badges={
                  <>
                    <Badge variant="secondary">{current.section ?? t("manual.sectionOther")}</Badge>
                    {current.stale && (
                      <StatusPill data-manual-stale-pill tone="warning">
                        {t("manual.stale")}
                      </StatusPill>
                    )}
                  </>
                }
                actions={renderDocumentActions?.({ doc: { manual: current.slug }, content: loaded?.body ?? null })}
                provenance={[
                  location?.slug === current.slug && <span className="font-mono">{location.relative}</span>,
                  current.generated && (
                    <span data-manual-generated>{t("manual.generatedAt").replace("{generated}", current.generated)}</span>
                  ),
                  sourceNames.length > 0 && (
                    <span className="inline-flex flex-wrap items-center gap-1.5">
                      {t("manual.sources")}
                      <SourceChips names={sourceNames} canonical={canonical} onOpenSpec={onOpenSpec} />
                    </span>
                  ),
                ]}
              />
              <DocTiles tiles={tiles} />
              {staleCallout}
              <div className={MANUAL_HEADINGS_CLS}>
                <DocBody>
                  {!loaded ? (
                    <DocSkeleton />
                  ) : loaded.body === null ? (
                    <div
                      data-manual-load-failed
                      className="flex items-center gap-2 py-6 text-sm text-muted-foreground"
                    >
                      <CloudOff className="h-4 w-4 shrink-0" />
                      <span>{t("manual.loadFailed")}</span>
                    </div>
                  ) : (
                    <Markdown content={markdownBody} />
                  )}
                </DocBody>
              </div>
            </Card>
          </div>
          <footer data-manual-footer className="shrink-0 border-t border-border bg-card px-6 py-2.5">
            <div className={`${MANUAL_COLUMN_CLS} flex items-center justify-between gap-3`}>
              {sourceNames.length > 0 ? (
                <div
                  data-manual-sources
                  className="flex min-w-0 flex-wrap items-center gap-1.5 text-xs text-muted-foreground"
                >
                  <span>{t("manual.sources")}</span>
                  <SourceChips names={sourceNames} canonical={canonical} onOpenSpec={onOpenSpec} />
                </div>
              ) : (
                <span />
              )}
              <div className="flex shrink-0 items-center gap-2">
                {prev && <PageNavButton direction="prev" target={prev} onSelect={setSelected} />}
                {next && <PageNavButton direction="next" target={next} onSelect={setSelected} />}
              </div>
            </div>
          </footer>
        </div>
        <ManualToc bodyRef={bodyRef} markdownBody={markdownBody} />
      </div>
    </div>
  );
}

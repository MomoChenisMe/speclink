import { useEffect, useRef, useState, type MouseEvent } from "react";
import { BookOpen, ChevronLeft, ChevronRight, CloudOff } from "lucide-react";

import type { ManualIndex, ManualPageItem } from "../adapter";
import { useI18n } from "../i18n";
import { cn } from "../lib/utils";
import { manualLinkSlug, splitLeadingHeading, stripSourcesLine } from "../manualDoc";
import { ManualToc } from "./ManualToc";
import { ManualTree } from "./ManualTree";
import { Markdown } from "./Markdown";
import { DocSkeleton, RowSkeleton } from "./skeletons";
import { badgeVariants } from "./ui/badge";
import { Button } from "./ui/button";
import { Card } from "./ui/card";

/** 手冊閱讀欄（design D5）：768px 置中；頁首、閱讀卡與底列內容同寬對齊。 */
const MANUAL_COLUMN_CLS = "mx-auto w-full max-w-[768px]";

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
}

/** 已載入的頁內文；body 為 null＝載入失敗或頁不存在。 */
interface LoadedDoc {
  slug: string;
  body: string | null;
}

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
  const prev = direction === "prev";
  const hook = prev ? { "data-manual-prev": target.slug } : { "data-manual-next": target.slug };
  return (
    <Button
      variant="outline"
      size="sm"
      aria-label={t(prev ? "pager.prev" : "pager.next")}
      className="h-auto gap-1.5 py-1"
      onClick={() => onSelect(target.slug)}
      {...hook}
    >
      {prev && <ChevronLeft className="h-4 w-4" />}
      <span className={cn("flex flex-col leading-tight", prev ? "items-start" : "items-end")}>
        <span className="text-[11px] font-normal text-muted-foreground">
          {t(prev ? "manual.prevLabel" : "manual.nextLabel")}
        </span>
        <span className="max-w-48 truncate">{target.title}</span>
      </span>
      {!prev && <ChevronRight className="h-4 w-4" />}
    </Button>
  );
}

/** 手冊頁（desktop-manual-page design「側欄樹、搜尋與上下頁在前端由索引推導」；
 * desktop-list-pages-reskin design D5 三欄：240px 目錄樹、768px 置中閱讀欄、200px 本頁
 * 目錄）：中欄以共用 Markdown 於白底閱讀卡內渲染選定頁——頁首（標題＋產生時間）與
 * 白底上緣細線的底列（出處籤＋上一頁／下一頁）固定不隨內文捲動。本元件只管選頁、
 * 內文載入（latest-wins）與三段切分；唯讀、無任何寫入操作。 */
export function ManualPage({ index, loadPage, onOpenSpec, capabilities, refreshGen }: ManualPageProps) {
  const { t } = useI18n();
  const [query, setQuery] = useState("");
  const [selected, setSelected] = useState<string | null>(null);
  const [doc, setDoc] = useState<LoadedDoc | null>(null);
  // 內文載入的 latest-wins 序號；loadPage 以 ref 持有，宿主每次渲染換 lambda 不重載。
  const seq = useRef(0);
  const loadPageRef = useRef(loadPage);
  loadPageRef.current = loadPage;

  const pages = index?.present ? index.pages : [];
  // 目前頁由索引現況推導：選定頁消失（外部刪除、索引重載）即回首頁。
  const current = pages.find((p) => p.slug === selected) ?? pages[0] ?? null;
  const currentSlug = current?.slug ?? null;

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

  const loaded = doc && current && doc.slug === current.slug ? doc : null;
  // 內文拆三段：尾端出處行剝掉（出處列改讀索引 sources）、開頭 H1 → 固定頁首、其餘 → 捲動區。
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
  // 出處名以索引 sources 為單一真相；manual-pages 契約：一項可帶 `#<Requirement 名>` 錨定，
  // 出處只看井號前的 capability 名——切掉錨定、去頭尾空白（讀取端也這麼做）後再去重
  //（防撞 React key，也防同 capability 多錨定重複）。
  const sourceNames = Array.from(new Set(current.sources.map((s) => s.split("#", 1)[0].trim())));
  const canonical = new Set(capabilities);
  const heading = parsed?.heading ?? current.title;

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
          <header data-manual-header className="shrink-0 px-6 pt-6 pb-3">
            <div className={MANUAL_COLUMN_CLS}>
              <h1 className="text-2xl font-normal">{heading}</h1>
              {current.generated && (
                <p data-manual-generated className="mt-1 text-xs text-muted-foreground">
                  {t("manual.generatedAt").replace("{generated}", current.generated)}
                </p>
              )}
            </div>
          </header>
          <div
            ref={bodyRef}
            data-manual-body
            className="min-h-0 flex-1 overflow-y-auto px-6 py-5"
            onClick={onBodyClick}
          >
            <Card data-manual-card className={`${MANUAL_COLUMN_CLS} p-6`}>
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
                  {sourceNames.map((name) =>
                    canonical.has(name) ? (
                      <button
                        key={name}
                        type="button"
                        className={cn(badgeVariants({ variant: "outline" }), "cursor-pointer font-mono hover:bg-muted")}
                        onClick={() => onOpenSpec(name)}
                      >
                        {name}
                      </button>
                    ) : (
                      <span key={name} className="font-mono">
                        {name}
                      </span>
                    ),
                  )}
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

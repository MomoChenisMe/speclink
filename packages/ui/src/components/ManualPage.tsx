import { useEffect, useRef, useState, type MouseEvent } from "react";
import { BookOpen, ChevronLeft, ChevronRight, CloudOff } from "lucide-react";

import type { ManualIndex } from "../adapter";
import { useI18n } from "../i18n";
import { manualLinkSlug, splitLeadingHeading, stripSourcesLine } from "../manualDoc";
import { ManualToc } from "./ManualToc";
import { ManualTree } from "./ManualTree";
import { Markdown, READING_COLUMN_CLS } from "./Markdown";
import { DocSkeleton, RowSkeleton } from "./skeletons";
import { Button } from "./ui/button";

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

/** 手冊頁（desktop-manual-page design「側欄樹、搜尋與上下頁在前端由索引推導」）：
 * 左欄 ManualTree、右側 ManualToc，中間以共用 Markdown 與閱讀欄渲染選定頁——頁首
 * 標題與頁尾上一頁／下一頁＋出處列固定不隨內文捲動。本元件只管選頁、內文載入
 * （latest-wins）與三段切分；唯讀、無任何寫入操作。 */
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
        <aside className="flex w-64 shrink-0 flex-col gap-1 border-r border-border py-5 pl-5 pr-3">
          <RowSkeleton />
          <RowSkeleton />
          <RowSkeleton />
        </aside>
        <div className="flex-1 p-5">
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
  // 出處名以索引 sources 為單一真相；去重防撞 React key。
  const sourceNames = Array.from(new Set(current.sources));
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
          <header data-manual-header className="shrink-0 border-b border-border px-5 pt-5 pb-3">
            <h1 className={`${READING_COLUMN_CLS} text-xl font-bold tracking-tight`}>{heading}</h1>
          </header>
          <div
            ref={bodyRef}
            data-manual-body
            className="min-h-0 flex-1 overflow-y-auto px-5 py-4"
            onClick={onBodyClick}
          >
            <div className={READING_COLUMN_CLS}>
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
            </div>
          </div>
          <footer data-manual-footer className="shrink-0 border-t border-border px-5 pt-3 pb-5">
            <div className={`${READING_COLUMN_CLS} flex flex-col gap-3`}>
              {sourceNames.length > 0 && (
                <div
                  data-manual-sources
                  className="flex flex-wrap items-center gap-1.5 text-xs text-muted-foreground"
                >
                  <span>{t("manual.sources")}</span>
                  {sourceNames.map((name) =>
                    canonical.has(name) ? (
                      <button
                        key={name}
                        type="button"
                        className="rounded border border-border bg-muted px-1.5 py-0.5 font-mono text-[11px] text-primary hover:bg-accent"
                        onClick={() => onOpenSpec(name)}
                      >
                        {name}
                      </button>
                    ) : (
                      <span key={name} className="rounded border border-border px-1.5 py-0.5 font-mono text-[11px]">
                        {name}
                      </span>
                    ),
                  )}
                </div>
              )}
              <div className="flex items-center justify-between gap-2">
                {prev ? (
                  <Button
                    variant="outline"
                    size="sm"
                    aria-label={t("pager.prev")}
                    data-manual-prev={prev.slug}
                    className="gap-1"
                    onClick={() => setSelected(prev.slug)}
                  >
                    <ChevronLeft className="h-4 w-4" />
                    <span className="max-w-48 truncate">{prev.title}</span>
                  </Button>
                ) : (
                  <span />
                )}
                {next ? (
                  <Button
                    variant="outline"
                    size="sm"
                    aria-label={t("pager.next")}
                    data-manual-next={next.slug}
                    className="gap-1"
                    onClick={() => setSelected(next.slug)}
                  >
                    <span className="max-w-48 truncate">{next.title}</span>
                    <ChevronRight className="h-4 w-4" />
                  </Button>
                ) : (
                  <span />
                )}
              </div>
            </div>
          </footer>
        </div>
        <ManualToc bodyRef={bodyRef} markdownBody={markdownBody} />
      </div>
    </div>
  );
}

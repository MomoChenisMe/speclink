import { useEffect, useRef, useState } from "react";
import { BookOpen, ChevronLeft, ChevronRight, CloudOff } from "lucide-react";

import type { ManualIndex, ManualPageItem } from "../adapter";
import { useI18n } from "../i18n";
import { matchesQuery } from "../search";
import { SEMANTIC_SURFACE, SEMANTIC_TONE } from "../tone";
import { Markdown, READING_COLUMN_CLS } from "./Markdown";
import { DocSkeleton, RowSkeleton } from "./skeletons";
import { Button } from "./ui/button";
import { Input } from "./ui/input";

export interface ManualPageProps {
  /** 手冊索引（已依閱讀序排好）；null＝載入中（骨架）。 */
  index: ManualIndex | null;
  /** 讀取一頁去 frontmatter 的內文；不存在回 null、讀取失敗 reject。 */
  loadPage: (slug: string) => Promise<string | null>;
  /** 點頁尾出處的 capability：切至規格頁並展開該規格卡（App 接線）。 */
  onOpenSpec: (capability: string) => void;
  /** 正典 capability 清單——出處只對存在者可點，不存在者為純文字。 */
  capabilities: string[];
  /** 刷新世代——變動時重載目前頁內文（外部寫入後的 workspace-changed）。 */
  refreshGen?: number;
}

interface Section {
  label: string;
  pages: ManualPageItem[];
}

/** 以 section 對閱讀序中連續的頁分組（core 已把同分區的頁排在一起）；缺 section
 * 的頁歸「其他」。 */
function groupSections(pages: ManualPageItem[], otherLabel: string): Section[] {
  const sections: Section[] = [];
  for (const page of pages) {
    const label = page.section ?? otherLabel;
    const last = sections[sections.length - 1];
    if (last && last.label === label) last.pages.push(page);
    else sections.push({ label, pages: [page] });
  }
  return sections;
}

// 頁尾出處行（manual-pages 契約）：最後一個非空段落以 `**出處**：` 開頭、反引號列
// capability 名。抽成可點的出處列，內文不重複呈現。
const SOURCES_LINE_RE = /^\*\*出處\*\*[：:]\s*(.*)$/;

/** 從內文尾端抽出出處行：回傳（去掉該行的內文、名稱清單）；無出處行時名稱為 null。 */
function splitSourcesLine(body: string): { body: string; names: string[] | null } {
  const lines = body.split("\n");
  let last = lines.length - 1;
  while (last >= 0 && lines[last].trim() === "") last--;
  const match = last >= 0 ? SOURCES_LINE_RE.exec(lines[last].trim()) : null;
  if (!match) return { body, names: null };
  return {
    body: lines.slice(0, last).join("\n"),
    names: Array.from(match[1].matchAll(/`([^`]+)`/g), (m) => m[1]),
  };
}

/** 已載入的頁內文；body 為 null＝載入失敗或頁不存在。 */
interface LoadedDoc {
  slug: string;
  body: string | null;
}

/** 手冊頁（desktop-manual-page design「側欄樹、搜尋與上下頁在前端由索引推導」）：
 * 左側依分區的頁面樹＋搜尋列（大小寫不敏感比對 title 與 keywords）、右側以共用
 * Markdown 與閱讀欄渲染選定頁、頁尾上一頁／下一頁與出處列；stale 頁列帶「可能
 * 過期」、索引底部提示未入冊規格數。唯讀、無任何寫入操作。 */
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

  if (index === null) {
    return (
      <div className="flex h-full min-h-0">
        <aside className="flex w-60 shrink-0 flex-col gap-1 border-r border-border pr-3">
          <RowSkeleton />
          <RowSkeleton />
          <RowSkeleton />
        </aside>
        <div className="flex-1 pl-5">
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
        className="flex h-full flex-col items-center justify-center gap-3 text-center"
      >
        <BookOpen className="h-10 w-10 text-muted-foreground/40" />
        <h2 className="text-lg font-semibold">{remote ? t("manual.remoteTitle") : t("manual.emptyTitle")}</h2>
        <p className="max-w-md text-sm text-muted-foreground">
          {remote ? t("manual.remoteDesc") : t("manual.emptyDesc")}
        </p>
      </div>
    );
  }

  const visibleSections = groupSections(
    pages.filter((p) => matchesQuery(query, p.title, ...p.keywords)),
    t("manual.sectionOther"),
  );
  const position = pages.indexOf(current);
  const prev = position > 0 ? pages[position - 1] : null;
  const next = position < pages.length - 1 ? pages[position + 1] : null;

  const loaded = doc && doc.slug === current.slug ? doc : null;
  const split = loaded?.body != null ? splitSourcesLine(loaded.body) : null;
  // 出處名去重（出處行重複列名時不撞 React key）。
  const sourceNames = Array.from(new Set(split?.names ?? current.sources));
  const canonical = new Set(capabilities);

  return (
    <div className="flex h-full min-h-0">
      <aside className="flex w-60 shrink-0 flex-col gap-2 border-r border-border pr-3">
        <Input
          placeholder={t("manual.searchPlaceholder")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <nav data-manual-tree className="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto">
          {visibleSections.length === 0 ? (
            <div data-manual-no-results className="py-4 text-center text-sm text-muted-foreground">
              {t("manual.noResults")}
            </div>
          ) : (
            visibleSections.map((section, i) => (
              // 以位置為 key：明寫 section「其他」的組與缺 section 歸「其他」的組可能不相鄰。
              <section key={i} data-manual-section={section.label}>
                <h3 className="px-2 text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">
                  {section.label}
                </h3>
                <ul className="mt-1 flex flex-col">
                  {section.pages.map((p) => {
                    const active = p.slug === current.slug;
                    return (
                      <li key={p.slug}>
                        <button
                          type="button"
                          data-manual-page={p.slug}
                          aria-current={active ? "page" : undefined}
                          className={`flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm ${
                            active
                              ? "bg-primary font-medium text-primary-foreground"
                              : "text-foreground hover:bg-muted"
                          }`}
                          onClick={() => setSelected(p.slug)}
                        >
                          <span className="min-w-0 flex-1 truncate">{p.title}</span>
                          {p.stale && (
                            <span
                              data-manual-stale
                              className={`shrink-0 text-[10px] font-medium ${active ? "" : SEMANTIC_TONE.warning}`}
                            >
                              {t("manual.stale")}
                            </span>
                          )}
                        </button>
                      </li>
                    );
                  })}
                </ul>
              </section>
            ))
          )}
        </nav>
        {index.uncoveredNew.length > 0 && (
          <div
            data-manual-uncovered
            className={`mt-auto rounded-md border px-2 py-1.5 text-xs ${SEMANTIC_SURFACE.warning}`}
          >
            {t("manual.uncoveredNew").replace("{n}", String(index.uncoveredNew.length))}
          </div>
        )}
      </aside>

      <div data-manual-content className="flex min-h-0 flex-1 flex-col overflow-y-auto pl-5">
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
            <Markdown content={split ? split.body : loaded.body} />
          )}
          <footer className="mt-8 flex flex-col gap-3 border-t border-border pt-4">
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
          </footer>
        </div>
      </div>
    </div>
  );
}

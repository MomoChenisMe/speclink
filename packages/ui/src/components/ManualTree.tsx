import type { ManualPageItem } from "../adapter";
import { useI18n } from "../i18n";
import { cn } from "../lib/utils";
import { groupSections } from "../manualDoc";
import { matchesQuery } from "../search";
import { SEMANTIC_SURFACE, SEMANTIC_TONE } from "../tone";
import { SearchField } from "./SearchField";

export interface ManualTreeProps {
  /** 全部頁（已依閱讀序排好）。 */
  pages: ManualPageItem[];
  currentSlug: string;
  query: string;
  onQuery: (query: string) => void;
  onSelect: (slug: string) => void;
  /** 手冊生成後新增且未入冊的規格數；0 不顯示提示。 */
  uncoveredCount: number;
}

/** 手冊頁左欄（spec「手冊頁的側欄樹與閱讀序」「手冊頁的搜尋列」「可能過期與未入冊的
 * 標示」；design D5：240px 淡灰底、列為 NavItem 風格）：搜尋列以大小寫不敏感比對 title
 * 與 keywords、命中頁依分區分組列出（stale 者附「可能過期」）、底部在有未入冊規格時
 * 顯示計數提示。 */
export function ManualTree({ pages, currentSlug, query, onQuery, onSelect, uncoveredCount }: ManualTreeProps) {
  const { t } = useI18n();
  const visibleSections = groupSections(
    pages.filter((p) => matchesQuery(query, p.title, ...p.keywords)),
    t("manual.sectionOther"),
  );
  return (
    <aside className="flex w-60 shrink-0 flex-col gap-2 border-r border-border bg-sidebar p-3">
      <SearchField value={query} onChange={onQuery} placeholder={t("manual.searchPlaceholder")} />
      <nav data-manual-tree className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto">
        {visibleSections.length === 0 ? (
          <div data-manual-no-results className="py-4 text-center text-sm text-muted-foreground">
            {t("manual.noResults")}
          </div>
        ) : (
          visibleSections.map((section, i) => (
            // 以位置為 key：明寫 section「其他」的組與缺 section 歸「其他」的組可能不相鄰。
            <section
              key={i}
              data-manual-section={section.label}
              className={i > 0 ? "border-t border-border pt-3" : undefined}
            >
              <h3 className="px-2.5 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
                {section.label}
              </h3>
              <ul className="mt-1 flex flex-col gap-0.5">
                {section.pages.map((p) => {
                  const active = p.slug === currentSlug;
                  return (
                    <li key={p.slug}>
                      <button
                        type="button"
                        data-manual-page={p.slug}
                        aria-current={active ? "page" : undefined}
                        className={cn(
                          "flex h-8 w-full items-center gap-2 rounded-lg px-2.5 text-left text-[13px] transition-colors",
                          active ? "bg-primary/12 font-medium text-primary" : "text-foreground hover:bg-foreground/5",
                        )}
                        onClick={() => onSelect(p.slug)}
                      >
                        <span className="min-w-0 flex-1 truncate">{p.title}</span>
                        {p.stale && (
                          <span data-manual-stale className={`shrink-0 text-[10px] font-medium ${SEMANTIC_TONE.warning}`}>
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
      {uncoveredCount > 0 && (
        <div
          data-manual-uncovered
          className={`mt-auto rounded-md border px-2 py-1.5 text-xs ${SEMANTIC_SURFACE.warning}`}
        >
          {t("manual.uncoveredNew").replace("{n}", String(uncoveredCount))}
        </div>
      )}
    </aside>
  );
}

import { useEffect, useState, type RefObject } from "react";

import { useI18n } from "../i18n";

/** 錨點列的一筆：內文 h2／h3 的 DOM id、純文字與層級。 */
interface TocEntry {
  id: string;
  text: string;
  level: 2 | 3;
}

export interface ManualTocProps {
  /** 內文捲動區——錨點由它渲染後的 DOM 推導，點擊也在它裡面捲動。 */
  bodyRef: RefObject<HTMLDivElement | null>;
  /** 目前渲染的內文；換了就重掃標題。 */
  markdownBody: string | null;
}

/** 手冊頁右側錨點列（spec「內頁渲染與出處跳規格」）：由渲染後的 DOM 掃 h2／h3（與畫面
 * 一致，不另解析 Markdown）、給 id；點擊捲至該標題；捲動時以容器頂緣下方 12px 內最後
 * 一個標題為目前段。內文沒有 h2／h3 時不渲染。 */
export function ManualToc({ bodyRef, markdownBody }: ManualTocProps) {
  const { t } = useI18n();
  const [toc, setToc] = useState<TocEntry[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);

  useEffect(() => {
    const root = bodyRef.current;
    const heads = root ? Array.from(root.querySelectorAll<HTMLElement>(".markdown h2, .markdown h3")) : [];
    heads.forEach((el, i) => {
      el.id = `manual-h-${i}`;
    });
    setToc(heads.map((el) => ({ id: el.id, text: el.textContent ?? "", level: el.tagName === "H3" ? 3 : 2 })));
    setActiveId(heads[0]?.id ?? null);
    if (!root || heads.length === 0) return;
    const onScroll = () => {
      const top = root.getBoundingClientRect().top + 12;
      let hit = heads[0];
      for (const el of heads) {
        if (el.getBoundingClientRect().top <= top) hit = el;
        else break;
      }
      setActiveId(hit.id);
    };
    root.addEventListener("scroll", onScroll, { passive: true });
    return () => root.removeEventListener("scroll", onScroll);
  }, [bodyRef, markdownBody]);

  if (toc.length === 0) return null;
  return (
    <nav
      data-manual-toc
      aria-label={t("manual.toc")}
      className="hidden w-52 shrink-0 flex-col gap-1 overflow-y-auto border-l border-border py-5 pl-3 pr-5 lg:flex"
    >
      <div className="px-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
        {t("manual.toc")}
      </div>
      <ul className="flex flex-col">
        {toc.map((h) => {
          const active = h.id === activeId;
          return (
            <li key={h.id}>
              <button
                type="button"
                data-manual-anchor={h.id}
                aria-current={active ? "location" : undefined}
                className={`w-full truncate border-l-2 py-1 text-left text-xs ${h.level === 3 ? "pl-5" : "pl-2"} ${
                  active
                    ? "border-primary font-medium text-primary"
                    : "border-transparent text-muted-foreground hover:text-foreground"
                }`}
                onClick={() =>
                  bodyRef.current
                    ?.querySelector<HTMLElement>(`#${h.id}`)
                    ?.scrollIntoView({ block: "start", behavior: "smooth" })
                }
              >
                {h.text}
              </button>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}

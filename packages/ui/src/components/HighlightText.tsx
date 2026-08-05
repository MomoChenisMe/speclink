import { SEMANTIC_SURFACE } from "../tone";

/**
 * 命中高亮（design D7）：以不分大小寫子字串定位 query 於 text 的首個命中，
 * 命中原文以 mark 標示（琥珀底、不改字色——「注意這裡」屬警示層級，主色留給
 * 連結與互動）。無 query 或無連續子字串命中（僅模糊命中）時原樣輸出、不高亮。
 */
export function HighlightText({ text, query }: { text: string; query?: string }) {
  const q = (query ?? "").trim().toLowerCase();
  if (!q) return <>{text}</>;
  const idx = text.toLowerCase().indexOf(q);
  if (idx < 0) return <>{text}</>;
  return (
    <>
      {text.slice(0, idx)}
      <mark className={`rounded-sm text-inherit ${SEMANTIC_SURFACE.warning}`}>
        {text.slice(idx, idx + q.length)}
      </mark>
      {text.slice(idx + q.length)}
    </>
  );
}

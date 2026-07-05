import type { DeltaCounts } from "../delta";

/** Spectra 式彩色 delta 計數：+新增=綠、~修改=琥珀、-移除=紅、→更名=藍。 */
export function DeltaBadges({ counts }: { counts: DeltaCounts }) {
  const parts: { text: string; cls: string }[] = [];
  if (counts.added) parts.push({ text: `+${counts.added}`, cls: "text-emerald-600 dark:text-emerald-400" });
  if (counts.modified) parts.push({ text: `~${counts.modified}`, cls: "text-amber-600 dark:text-amber-400" });
  if (counts.removed) parts.push({ text: `-${counts.removed}`, cls: "text-red-600 dark:text-red-400" });
  if (counts.renamed) parts.push({ text: `→${counts.renamed}`, cls: "text-sky-600 dark:text-sky-400" });
  if (parts.length === 0) return null;
  return (
    <span className="inline-flex gap-1 text-xs font-semibold tabular-nums">
      {parts.map((p) => (
        <span key={p.text} className={p.cls}>
          {p.text}
        </span>
      ))}
    </span>
  );
}

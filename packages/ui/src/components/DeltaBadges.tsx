import { splitDeltaSections, type DeltaCounts } from "../delta";
import { useI18n } from "../i18n";
import { Markdown } from "./Markdown";
import { LABEL_CLS } from "./SectionedDoc";

/** delta 四種操作的色彩 class——計數徽章與規格分頁色標區段共用的單一來源。 */
export const DELTA_COLORS: Record<keyof DeltaCounts, string> = {
  added: "text-emerald-600 dark:text-emerald-400",
  modified: "text-amber-600 dark:text-amber-400",
  removed: "text-red-600 dark:text-red-400",
  renamed: "text-sky-600 dark:text-sky-400",
};

const DELTA_MARKS: Record<keyof DeltaCounts, string> = {
  added: "+",
  modified: "~",
  removed: "-",
  renamed: "→",
};

const DELTA_LABEL_KEYS: Record<keyof DeltaCounts, string> = {
  added: "delta.added",
  modified: "delta.modified",
  removed: "delta.removed",
  renamed: "delta.renamed",
};

/** Spectra 式彩色 delta 計數：+新增=綠、~修改=琥珀、-移除=紅、→更名=藍。 */
export function DeltaBadges({ counts }: { counts: DeltaCounts }) {
  const parts: { text: string; cls: string }[] = [];
  if (counts.added) parts.push({ text: `+${counts.added}`, cls: DELTA_COLORS.added });
  if (counts.modified) parts.push({ text: `~${counts.modified}`, cls: DELTA_COLORS.modified });
  if (counts.removed) parts.push({ text: `-${counts.removed}`, cls: DELTA_COLORS.removed });
  if (counts.renamed) parts.push({ text: `→${counts.renamed}`, cls: DELTA_COLORS.renamed });
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

/**
 * delta spec 的色標區段檢視（spec「規格分頁 delta 區段以色標呈現」）：
 * `## <OP> Requirements` 機器標題不直出，改為色標區段標頭（配色同 DELTA_COLORS）；
 * 區段內文交給共用 Markdown 照排。無 delta 標題的文件整篇照常渲染。
 * RichDetailDrawer 規格分頁與 ArchivedList 規格分頁共用。
 */
export function DeltaSpecView({ markdown, empty }: { markdown: string | null; empty?: string }) {
  const { t } = useI18n();
  if (!markdown || !markdown.trim()) return <Markdown content={markdown} empty={empty} />;
  return (
    <div>
      {splitDeltaSections(markdown).map((s, i) =>
        s.op === null ? (
          <Markdown key={i} content={s.content} />
        ) : (
          <div key={i}>
            <div
              data-delta-section={s.op}
              className={`mt-4 mb-1 first:mt-0 flex items-center gap-1.5 ${LABEL_CLS} ${DELTA_COLORS[s.op]}`}
            >
              <span aria-hidden="true">{DELTA_MARKS[s.op]}</span>
              <span>{t(DELTA_LABEL_KEYS[s.op])}</span>
            </div>
            <Markdown content={s.content} />
          </div>
        ),
      )}
    </div>
  );
}

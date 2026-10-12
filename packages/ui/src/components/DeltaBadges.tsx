import { useEffect, useRef } from "react";

import { specDeltaCounts, splitDeltaSections, type DeltaCounts } from "../delta";
import { removedScenarios } from "../docModel";
import { useI18n } from "../i18n";
import { parseSpecSections, type SpecRequirement } from "../specSections";
import { SEMANTIC_TONE } from "../tone";
import { Button } from "./ui/button";
import { SECTION_HEADING_CLS } from "./DocParts";
import { Markdown } from "./Markdown";
import { RequirementBlock } from "./RequirementBlock";

/** delta 四種操作的色彩 class——計數徽章與規格分頁色標區段共用的單一來源。
 *  新增＝成功、修改＝警示、移除＝destructive、更名＝進行中，色值在 theme.css。 */
export const DELTA_COLORS: Record<keyof DeltaCounts, string> = {
  added: SEMANTIC_TONE.success,
  modified: SEMANTIC_TONE.warning,
  removed: SEMANTIC_TONE.danger,
  renamed: SEMANTIC_TONE.inProgress,
};

/** 同色的左色條（delta 需求段）：色值同 DELTA_COLORS 的語意 token。 */
export const DELTA_BORDER: Record<keyof DeltaCounts, string> = {
  added: "border-status-success",
  modified: "border-status-warning",
  removed: "border-destructive",
  renamed: "border-status-progress",
};

const DELTA_MARKS: Record<keyof DeltaCounts, string> = {
  added: "+",
  modified: "~",
  removed: "-",
  renamed: "→",
};

/** delta 四種操作的 i18n 詞條鍵——規格分頁色標區段與排程分頁的操作標籤共用。 */
export const DELTA_LABEL_KEYS: Record<keyof DeltaCounts, string> = {
  added: "delta.added",
  modified: "delta.modified",
  removed: "delta.removed",
  renamed: "delta.renamed",
};

/** 彩色 delta 計數：+新增=綠、~修改=琥珀、-移除=紅、→更名=藍。 */
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

/** 需求段的 REMOVED-SCENARIO 宣告（需求文或 Scenario 內文都可能放）：取出名稱、去掉宣告與 BEFORE 註解行。 */
function withoutRemovals(req: SpecRequirement): { req: SpecRequirement; names: string[] } {
  const body = removedScenarios(req.body);
  const scenarios = req.scenarios.map((sc) => ({ title: sc.title, ...removedScenarios(sc.body) }));
  return {
    req: { ...req, body: body.content, scenarios: scenarios.map((sc) => ({ title: sc.title, body: sc.content })) },
    names: [...body.names, ...scenarios.flatMap((sc) => sc.names)],
  };
}

/** 一個 op 區段的內文：ADDED／MODIFIED 每條需求成需求段（第一條展開），其餘照 Markdown。 */
function DeltaSectionBody({ op, content }: { op: keyof DeltaCounts; content: string }) {
  const parsed = op === "added" || op === "modified" ? parseSpecSections(content) : null;
  if (!parsed) return <Markdown content={content} />;
  return (
    <>
      {parsed.intro && <Markdown content={parsed.intro} />}
      {parsed.requirements.map((r, i) => {
        const { req, names } = withoutRemovals(r);
        return <RequirementBlock key={i} req={req} delta={{ op, removedScenarios: names, defaultOpen: i === 0 }} />;
      })}
      {parsed.appendix && <Markdown content={parsed.appendix} />}
    </>
  );
}

/**
 * delta spec 的色標區段檢視（spec「規格分頁 delta 區段以色標呈現」；design D6）：
 * `## <OP> Requirements` 機器標題不直出，改為色標區段標頭（配色同 DELTA_COLORS）；ADDED／MODIFIED
 * 的每條需求以規格詳情頁同一個需求段呈現（操作色左色條、REMOVED-SCENARIO 提示框、可收合），
 * REMOVED／RENAMED 內文照 Markdown。無 delta 標題的文件整篇照常渲染。
 * ChangeDetailPage 與 ArchivedDetailPage 的規格文件共用。
 */
export function DeltaSpecView({ markdown }: { markdown: string | null }) {
  const { t } = useI18n();
  if (!markdown || !markdown.trim()) return <Markdown content={markdown} />;
  return (
    <div className="flex flex-col gap-4">
      {splitDeltaSections(markdown).map((s, i) =>
        s.op === null ? (
          <Markdown key={i} content={s.content} />
        ) : (
          <div key={i} className="flex flex-col gap-3">
            <div
              data-delta-section={s.op}
              data-doc-heading
              className={`flex items-center gap-1.5 ${SECTION_HEADING_CLS} ${DELTA_COLORS[s.op]}`}
            >
              <span aria-hidden="true">{DELTA_MARKS[s.op]}</span>
              <span>{t(DELTA_LABEL_KEYS[s.op])}</span>
            </div>
            <DeltaSectionBody op={s.op} content={s.content} />
          </div>
        ),
      )}
    </div>
  );
}

/** 規格文件的能力頭（design D6）：等寬能力名（本頁目錄錨點）、delta 計數，有回呼時「看正式規格」。 */
export function CapabilityHeader({
  cap,
  counts,
  onOpenSpec,
}: {
  cap: string;
  counts: DeltaCounts;
  onOpenSpec?: (capability: string) => void;
}) {
  const { t } = useI18n();
  return (
    <div data-capability-header className="flex min-w-0 items-center gap-2 border-b border-border/70 pb-2">
      <h2 data-doc-heading className="min-w-0 truncate font-mono text-sm font-semibold">
        {cap}
      </h2>
      <DeltaBadges counts={counts} />
      {onOpenSpec && (
        <Button type="button" variant="ghost" size="sm" className="ml-auto" onClick={() => onOpenSpec(cap)}>
          {t("doc.viewSpec")}
        </Button>
      )}
    </div>
  );
}

/**
 * 規格文件的一個能力段（能力頭＋delta 檢視），變更與已封存兩頁共用。`jump` 指到它時捲到段首並回報清除：
 * 捲動放在段自己的 effect，因為 Radix 的 TabsContent 在切換後的下一輪 render 才掛上內文，頁層 effect
 * 那時還找不到段。
 */
export function DeltaCapSection({
  cap,
  doc,
  jump,
  onJumped,
  onOpenSpec,
}: {
  cap: string;
  doc: string | null;
  jump: { cap: string } | null;
  onJumped: () => void;
  onOpenSpec?: (capability: string) => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (jump?.cap !== cap) return;
    ref.current?.scrollIntoView?.({ block: "start" });
    onJumped();
  }, [jump, cap, onJumped]);
  return (
    <div ref={ref} data-spec-cap={cap} className="mb-6 flex flex-col gap-3">
      <CapabilityHeader cap={cap} counts={specDeltaCounts(doc)} onOpenSpec={onOpenSpec} />
      {/* 逐項空態（capability 有列出、其 spec.md 為空）不套整體性文案。 */}
      <DeltaSpecView markdown={doc} />
    </div>
  );
}

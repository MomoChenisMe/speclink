import { useEffect, useRef, useState } from "react";
import { Clock } from "lucide-react";

import { itemMarkdown } from "../docModel";
import {
  CONCLUSION_LABELS,
  captureTargets,
  commandChips,
  cutPromotion,
  nameSpans,
  parseConclusion,
  parseCuts,
  splitOption,
  splitRows,
  splitVerdict,
  type ConclusionLabel,
  type CutCard,
} from "../discussionModel";
import { useI18n } from "../i18n";
import { SEMANTIC_SURFACE } from "../tone";
import { StatusPill, type StatusPillTone } from "./ui/status-pill";
import { CopyButton } from "./CopyButton";
import { DocSection, InlineMarks, MoreToggle, NAME_CHIP_CLS, NameChip, OptionReasonRows, RowList } from "./DocParts";
import { Markdown } from "./Markdown";

/** 分刀卡變更籤的階段：名稱在 `promoted_to` 中時有階段籤，`onOpen` 缺席為不可點。 */
export interface CutStage {
  label: string;
  tone: StatusPillTone;
  onOpen?: () => void;
}

export interface ConclusionDocProps {
  text: string;
  empty?: string;
  /** 分刀卡變更籤的階段與開啟；回 null＝不在 `promoted_to`（不可點、無階段籤）。 */
  stageOf?: (name: string) => CutStage | null;
  /** 記錄去向的規格籤：名稱在 `specNames` 中才可點。 */
  specNames?: string[];
  onOpenSpec?: (capability: string) => void;
  /** 「前往」請求：把該刀卡捲到閱讀區中央（seq 遞增，同一刀可重複跳）；處理完呼叫 `onJumped` 清除。 */
  jump?: { id: string; seq: number } | null;
  onJumped?: () => void;
}

/** 欄位 → 區段標題詞條（也是 `data-section` 的值）。 */
const FIELD_KEYS: Record<ConclusionLabel, string> = {
  Decision: "conclusion.decision",
  Rationale: "conclusion.rationale",
  "Rejected alternatives": "conclusion.rejected",
  Deferred: "conclusion.deferred",
  "Capture to": "conclusion.captureTo",
  Next: "conclusion.next",
};

/** 定案細節預設只畫前幾條。 */
const DETAILS_SHOWN = 4;

/**
 * 結論文件（desktop-discussion-doc-views design D2；spec「討論結論以欄位標籤呈現」）：六欄位各一段，決定之後
 * 有分刀時多一段「分刀」；欄位一個都沒有時整篇 Markdown 退回。討論詳情頁與已封存討論頁共用。
 */
export function ConclusionDoc({ text, empty, stageOf, specNames, onOpenSpec, jump, onJumped }: ConclusionDocProps) {
  const { t } = useI18n();
  const ref = useRef<HTMLDivElement>(null);
  // 刀卡在這一輪 commit 後才在 DOM 裡，effect 裡捲動才找得到；找不到時只清請求。
  useEffect(() => {
    if (!jump) return;
    ref.current
      ?.querySelector(`[data-cut="${CSS.escape(jump.id)}"]`)
      ?.scrollIntoView({ block: "center", behavior: "smooth" });
    onJumped?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [jump]);
  const parsed = parseConclusion(text);
  if (!parsed) return <Markdown content={text} empty={empty} />;
  const { lead, fields } = parsed;
  return (
    <div ref={ref}>
      {lead && (
        <div className="mb-5">
          <Markdown content={lead} />
        </div>
      )}
      {CONCLUSION_LABELS.filter((l) => fields[l] !== undefined).map((l) => {
        const md = fields[l]!;
        const title = t(FIELD_KEYS[l]);
        switch (l) {
          case "Decision":
            return <DecisionSections key={l} md={md} stageOf={stageOf} />;
          case "Rejected alternatives":
          case "Deferred": {
            const rows = splitRows(md).map(splitOption);
            return (
              <DocSection key={l} sectionKey={FIELD_KEYS[l]} title={title} count={rows.length > 0 ? rows.length : undefined}>
                <OptionReasonRows items={rows} icon={l === "Deferred" ? <Clock className="h-3.5 w-3.5" /> : undefined} />
              </DocSection>
            );
          }
          case "Capture to":
            return (
              <DocSection key={l} sectionKey={FIELD_KEYS[l]} title={title}>
                <CaptureChips md={md} specNames={specNames ?? []} onOpenSpec={onOpenSpec} />
              </DocSection>
            );
          case "Next":
            return (
              <DocSection key={l} sectionKey={FIELD_KEYS[l]} title={title}>
                <NextCommands md={md} />
              </DocSection>
            );
          default:
            return (
              <DocSection key={l} sectionKey={FIELD_KEYS[l]} title={title}>
                <Markdown content={md} />
              </DocSection>
            );
        }
      })}
    </div>
  );
}

/** 決定：定論句放大、定案細節收合；分刀列抽成其後的「分刀」段。 */
function DecisionSections({ md, stageOf }: { md: string; stageOf?: ConclusionDocProps["stageOf"] }) {
  const { t } = useI18n();
  const [expanded, setExpanded] = useState(false);
  const { verdict, details } = splitVerdict(md);
  const { cuts, rest } = parseCuts(details);
  const rows = rest.map(itemMarkdown);
  const hidden = expanded ? 0 : Math.max(0, rows.length - DETAILS_SHOWN);
  return (
    <>
      <DocSection sectionKey="conclusion.decision" title={t("conclusion.decision")} count={rows.length > 0 ? rows.length : undefined}>
        <div className="flex flex-col gap-3">
          {verdict && (
            <p data-verdict className="text-[17px] leading-relaxed">
              <InlineMarks text={verdict} />
            </p>
          )}
          {rows.length > 0 && <RowList items={hidden > 0 ? rows.slice(0, DETAILS_SHOWN) : rows} />}
          {hidden > 0 && (
            <div>
              <MoreToggle
                label={t("conclusion.moreDetails").replace("{n}", String(hidden))}
                onExpand={() => setExpanded(true)}
              />
            </div>
          )}
        </div>
      </DocSection>
      {cuts.length > 0 && (
        <DocSection sectionKey="conclusion.cuts" title={t("conclusion.cuts")} count={cuts.length}>
          <div className="flex flex-col gap-2.5">
            {cuts.map((cut) => (
              <CutCardView key={cut.id} cut={cut} stageOf={stageOf} />
            ))}
          </div>
        </DocSection>
      )}
    </>
  );
}

/** 一張分刀卡：識別籤、範圍句、變更數；變更名籤附階段籤；卡內細線列。整刀未轉出時橄欖色虛線框、卡頭
 * 「尚未轉出」籤；部分轉出時未轉出的變更名籤之後接同款籤。 */
function CutCardView({ cut, stageOf }: { cut: CutCard; stageOf?: ConclusionDocProps["stageOf"] }) {
  const { t } = useI18n();
  const promotion = cutPromotion(cut, (name) => stageOf?.(name) != null);
  const todo = promotion === "none";
  const notPromoted = <StatusPill tone="manual">{t("conclusion.notPromoted")}</StatusPill>;
  return (
    <section
      data-cut={cut.id}
      data-cut-todo={todo ? "" : undefined}
      className={`rounded-xl border px-4 py-3 ${todo ? `border-dashed ${SEMANTIC_SURFACE.manual}` : "border-border"}`}
    >
      <header className="flex min-w-0 items-center gap-2">
        <span className={NAME_CHIP_CLS}>{cut.id}</span>
        {todo && notPromoted}
        {cut.title && (
          <span className="min-w-0 font-medium">
            <InlineMarks text={cut.title} />
          </span>
        )}
        <span className="ml-auto shrink-0 text-xs text-muted-foreground tabular-nums">
          {t("conclusion.cutChanges").replace("{n}", String(cut.names.length))}
        </span>
      </header>
      {cut.names.length > 0 && (
        <div className="mt-2 flex flex-wrap items-center gap-1.5">
          {cut.names.map((name) => {
            const stage = stageOf?.(name) ?? null;
            return (
              <span key={name} data-cut-change={name} className="inline-flex items-center gap-1">
                <NameChip name={name} onClick={stage?.onOpen} />
                {stage ? (
                  <StatusPill tone={stage.tone} className={stage.tone === "destructive" ? "line-through" : undefined}>
                    {stage.label}
                  </StatusPill>
                ) : (
                  promotion === "partial" && notPromoted
                )}
              </span>
            );
          })}
        </div>
      )}
      {cut.items.length > 0 && (
        <div className="mt-2.5">
          <RowList items={cut.items} />
        </div>
      )}
    </section>
  );
}

/** 記錄去向：每項一顆籤；等於規格名時整顆可點，內含規格名時籤內的名稱可點；切不出籤時照 Markdown。 */
function CaptureChips({
  md,
  specNames,
  onOpenSpec,
}: {
  md: string;
  specNames: string[];
  onOpenSpec?: (capability: string) => void;
}) {
  const targets = captureTargets(md);
  if (!targets) return <Markdown content={md} />;
  return (
    <div className="flex flex-wrap items-center gap-1.5">
      {targets.map((target, i) => {
        if (onOpenSpec && specNames.includes(target)) {
          return (
            <span key={i} data-capture-chip className="inline-flex">
              <NameChip name={target} onClick={() => onOpenSpec(target)} />
            </span>
          );
        }
        const spans = onOpenSpec ? nameSpans(target, specNames) : [];
        let last = 0;
        // 籤文字可能是一整句：可折行、寬度不超過容器（不沿用名稱籤的不換行）。
        return (
          <span key={i} data-capture-chip className="inline-block max-w-full rounded bg-muted px-1.5 py-0.5 font-mono text-xs break-words">
            {spans.flatMap(({ name, start, end }) => {
              const parts = [
                target.slice(last, start),
                <button
                  key={start}
                  type="button"
                  className="text-primary underline-offset-2 hover:underline"
                  onClick={() => onOpenSpec!(name)}
                >
                  {name}
                </button>,
              ];
              last = end;
              return parts;
            })}
            {target.slice(last)}
          </span>
        );
      })}
    </div>
  );
}

/** 指令籤：等寬指令原文＋複製鈕。「下一步」與未轉出提示框共用。 */
export function CommandChip({ command }: { command: string }) {
  const { t } = useI18n();
  return (
    <span data-command-chip className={`${NAME_CHIP_CLS} gap-1`}>
      {command}
      <CopyButton value={command} label={t("conclusion.copyCommand")} className="h-5 w-5" />
    </span>
  );
}

/** 下一步：Markdown 之後列出 `/speclink-*` 指令籤，每顆附複製鈕。 */
function NextCommands({ md }: { md: string }) {
  const commands = commandChips(md);
  return (
    <div className="flex flex-col gap-2.5">
      <Markdown content={md} />
      {commands.length > 0 && (
        <div className="flex flex-wrap items-center gap-1.5">
          {commands.map((command) => (
            <CommandChip key={command} command={command} />
          ))}
        </div>
      )}
    </div>
  );
}

import type { ChangeItem, StationTicket, TicketStation } from "../adapter";
import { specDeltaCounts, sumDeltaCounts, type DeltaCounts } from "../delta";
import {
  listItemsOf,
  parseCapabilities,
  parseDecisions,
  parseGoalsNonGoals,
  parseImpact,
  parseListItems,
  proposalShape,
  removedScenarios,
  splitH2Sections,
  splitTaskText,
  type H2Section,
} from "../docModel";
import { parseTaskDoc, type TaskDocItem } from "../tasks";
import { ArrowRight } from "lucide-react";

import { Badge } from "./ui/badge";
import { Button } from "./ui/button";
import { Callout } from "./ui/callout";
import { InlineMarks } from "./DocParts";
import { DELTA_COLORS, DELTA_LABEL_KEYS } from "./DeltaBadges";
import type { DocSummary, DocTile } from "./DetailShell";
import { planSummary } from "./PlanDoc";
import { impactFileCount } from "./ProposalDoc";
import { ticketSummary } from "./TicketView";

type Doc = string | null | undefined;

/** 頁已持有的文件狀態：undefined＝載入在途、null＝不存在。 */
export interface ChangeDocData {
  proposal: Doc;
  design: Doc;
  tasksMd: Doc;
  specDocs: Record<string, string | null> | undefined;
  /** 作用中變更（排程用）；已封存頁為 null。 */
  change: ChangeItem | null;
  tickets: Record<TicketStation, StationTicket | null | undefined>;
  /** 唯讀（已封存）：工單不給「可以蓋章」提示框。 */
  readOnly?: boolean;
  /** 手動提示框的「前往」：頁讓任務文件捲到第 ordinal 個任務並高亮；缺席時不出「前往」鈕。 */
  onLocateTask?: (ordinal: number) => void;
}

const EMPTY: DocSummary = { badges: null, tiles: [], callout: null };

const SHAPE_KEYS = { bugFix: "doc.shapeBugFix", refactor: "doc.shapeRefactor", feature: "doc.shapeFeature" } as const;

/** 籤槽的 delta 操作順序（spec「規格的籤槽與數字磚」：修改在前）。 */
const OP_ORDER: (keyof DeltaCounts)[] = ["modified", "added", "removed", "renamed"];

const sectionOf = (sections: H2Section[], ...keys: string[]) => sections.find((s) => s.key !== null && keys.includes(s.key));
const itemCount = (md: string) => listItemsOf(parseListItems(md)).length;

/**
 * 閱讀卡標題區的推導單一入口（design D9）：依目前文件回籤槽、數字磚與提示框，全部由文件結構推出；
 * 文件未載入或無內容時三者皆空。提案的提示框由頁放在能力段尾，其他文件放在數字磚與內文之間。
 */
export function changeDocSummary(tab: string, data: ChangeDocData, t: (key: string) => string): DocSummary {
  switch (tab) {
    case "proposal":
      return data.proposal ? proposalSummary(data.proposal, data.specDocs, t) : EMPTY;
    case "design":
      return data.design ? designSummary(data.design, t) : EMPTY;
    case "tasks":
      return data.tasksMd ? tasksSummary(data.tasksMd, t, data.onLocateTask) : EMPTY;
    case "specs":
      return specsSummary(data.specDocs, t);
    case "plan":
      return data.change ? planSummary(data.change, t) : EMPTY;
    case "review":
    case "verify": {
      const summary = ticketSummary(tab, data.tickets[tab], t);
      return data.readOnly ? { ...summary, callout: null } : summary;
    }
    default:
      return EMPTY;
  }
}

function proposalSummary(proposal: string, specDocs: ChangeDocData["specDocs"], t: (key: string) => string): DocSummary {
  const doc = splitH2Sections(proposal);
  if (!doc) return EMPTY;
  const { sections } = doc;
  const shape = proposalShape(doc);
  const caps = sectionOf(sections, "sections.capabilities");
  const capRows = (key: string) =>
    parseCapabilities(caps?.subsections.find((sub) => sub.key === key)?.content ?? "").filter((r) => r.name);
  const impact = sectionOf(sections, "sections.impact");
  const parsedImpact = impact ? parseImpact(impact.content) : null;
  // spec 順序取章節存在的前四格；影響段不符形狀時「影響的檔案」缺席。
  const candidates: (DocTile | null)[] = [
    tileFrom(sectionOf(sections, "sections.whatChanges", "sections.proposedSolution"), "doc.tileSteps", t),
    tileFrom(sectionOf(sections, "sections.nonGoals"), "doc.tileNonGoals", t),
    tileFrom(sectionOf(sections, "sections.successCriteria"), "doc.tileCriteria", t),
    caps
      ? {
          value: capRows("sections.newCapabilities").length + capRows("sections.modifiedCapabilities").length,
          label: t("doc.tileCapabilities"),
        }
      : null,
    parsedImpact ? { value: impactFileCount(parsedImpact), label: t("doc.tileFiles") } : null,
    tileFrom(sectionOf(sections, "sections.alternatives"), "doc.tileAlternatives", t),
  ];
  // 改寫提示框：修改的能力中、本變更已載入 delta 規格者各一行。
  const notes = capRows("sections.modifiedCapabilities")
    .map((r) => r.name!)
    .filter((cap) => specDocs?.[cap])
    .map((cap) => {
      const doc = specDocs![cap]!;
      const removed = removedScenarios(doc).names.length;
      const line = t("doc.capabilityRewritten")
        .replace("{cap}", cap)
        .replace("{n}", String(specDeltaCounts(doc).modified));
      return removed > 0 ? line + t("doc.capabilityRemovedScenarios").replace("{m}", String(removed)) : line;
    });
  return {
    badges: shape && <Badge variant="outline">{t(SHAPE_KEYS[shape])}</Badge>,
    tiles: candidates.filter((c): c is DocTile => c !== null).slice(0, 4),
    callout:
      notes.length > 0 ? (
        <Callout tone="note" title={t("markdown.alertNote")} className="my-0">
          {notes.map((line) => (
            <p key={line} className="text-sm">
              {line}
            </p>
          ))}
        </Callout>
      ) : null,
  };
}

/** 列式章節的列點數磚；章節缺席為 null。 */
function tileFrom(section: H2Section | undefined, label: string, t: (key: string) => string): DocTile | null {
  return section ? { value: itemCount(section.content), label: t(label) } : null;
}

function designSummary(design: string, t: (key: string) => string): DocSummary {
  const doc = splitH2Sections(design);
  if (!doc) return EMPTY;
  const { sections } = doc;
  const decisions = sectionOf(sections, "sections.decisions");
  const goals = sectionOf(sections, "sections.goalsNonGoals");
  const split = goals ? parseGoalsNonGoals(goals.content) : null;
  const tiles: (DocTile | null)[] = [
    decisions ? { value: parseDecisions(decisions.content).cards.length, label: t("doc.tileDecisions") } : null,
    split ? { value: `${itemCount(split.goals)} ／ ${itemCount(split.nonGoals)}`, label: t("doc.tileGoals") } : null,
    tileFrom(sectionOf(sections, "sections.risks"), "doc.tileRisks", t),
    tileFrom(sectionOf(sections, "sections.openQuestions"), "doc.tileOpenQuestions", t),
  ];
  return { badges: null, tiles: tiles.filter((c): c is DocTile => c !== null), callout: null };
}

type TaskItem = Extract<TaskDocItem, { kind: "task" }>;

function tasksSummary(
  tasksMd: string,
  t: (key: string) => string,
  onLocateTask?: (ordinal: number) => void,
): DocSummary {
  const items = parseTaskDoc(tasksMd);
  const tasks = items.filter((i): i is TaskItem => i.kind === "task");
  if (tasks.length === 0) return EMPTY;
  const done = tasks.filter((x) => x.done).length;
  const manual = tasks.filter((x) => x.manual);
  // 提示框只提醒還要親手做的：已勾的手動任務不列，數字磚仍計總數。
  const pendingManual = manual.filter((x) => !x.done);
  return {
    badges:
      done === tasks.length ? (
        <Badge>{t("tasks.allDone")}</Badge>
      ) : (
        <Badge variant="secondary" className="tabular-nums">{`${done}/${tasks.length}`}</Badge>
      ),
    tiles: [
      { value: tasks.length, label: t("tasks.tileTotal") },
      { value: done, label: t("tasks.tileDone") },
      { value: manual.length, label: t("tasks.tileManual") },
      { value: items.length - tasks.length, label: t("tasks.tileGroups") },
    ],
    callout:
      pendingManual.length > 0 ? (
        <Callout tone="manual" title={t("tasks.manualCallout").replace("{n}", String(pendingManual.length))} className="my-0">
          <ul className="my-1 flex flex-col gap-0.5 text-sm">
            {pendingManual.map((m) => {
              const parts = splitTaskText(m.text);
              return (
                <li key={m.ordinal} data-manual-task className="flex items-center gap-2">
                  {parts.number && <span className="font-mono text-xs text-muted-foreground">{parts.number}</span>}
                  <span className="min-w-0">
                    <InlineMarks text={parts.title} />
                  </span>
                  {onLocateTask && (
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="ml-auto h-6 shrink-0 gap-1 px-2"
                      onClick={() => onLocateTask(m.ordinal)}
                    >
                      {t("tasks.locate")}
                      <ArrowRight className="h-3.5 w-3.5" />
                    </Button>
                  )}
                </li>
              );
            })}
          </ul>
          <p className="text-sm text-muted-foreground">{t("tasks.manualCalloutHint")}</p>
        </Callout>
      ) : null,
  };
}

function specsSummary(specDocs: ChangeDocData["specDocs"], t: (key: string) => string): DocSummary {
  const docs = Object.values(specDocs ?? {}).filter((d): d is string => !!d);
  if (docs.length === 0) return EMPTY;
  const counts = sumDeltaCounts(docs.map(specDeltaCounts));
  const removed = docs.reduce((n, d) => n + removedScenarios(d).names.length, 0);
  return {
    badges: OP_ORDER.filter((op) => counts[op] > 0).map((op) => (
      <Badge key={op} variant="outline" data-delta-count={op} className={DELTA_COLORS[op]}>
        {`${t(DELTA_LABEL_KEYS[op])} ${counts[op]}`}
      </Badge>
    )),
    tiles: [
      { value: Object.keys(specDocs ?? {}).length, label: t("doc.tileCapsTouched") },
      { value: counts.modified, label: t("doc.tileModifiedReqs") },
      { value: removed, label: t("doc.tileRemovedScenarios") },
      { value: `${counts.added} ／ ${counts.renamed}`, label: t("doc.tileAddedRenamed") },
    ],
    callout: null,
  };
}

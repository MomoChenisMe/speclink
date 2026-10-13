import type { ReactNode } from "react";
import { ArrowRight } from "lucide-react";

import type { ArchivedItem, ChangeItem } from "../adapter";
import {
  contextModel,
  openItems,
  parseConclusion,
  parseCuts,
  roundStats,
  splitDiscussionSections,
  splitRounds,
  splitRows,
  splitVerdict,
  type ContextModel,
  type ContextNames,
  type CutCard,
  type DiscussionRound,
  type DiscussionSections,
} from "../discussionModel";
import { Button } from "./ui/button";
import { Callout } from "./ui/callout";
import { cutPromotionOf } from "./ConclusionDoc";
import type { DocSummary, DocTile } from "./DetailShell";
import { CommandChip, InlineMarks, NAME_CHIP_CLS } from "./DocParts";
import { rowStage, type PromotedRow } from "./PromotedDoc";

/** 討論頁的文件節點：記錄切分成功時結論、討論過程、背景，失敗時「記錄」；衍生變更恆在。 */
export type DiscussionTab = "conclusion" | "rounds" | "context" | "promote" | "record";

/** 兩頁自記錄與清單推出的同一份資料：渲染器的分刀、背景與衍生列、閱讀卡標題區的推導都吃它。 */
export interface DiscussionDocData {
  sections: DiscussionSections | null;
  rounds: DiscussionRound[] | null;
  conclusion: ReturnType<typeof parseConclusion>;
  cuts: CutCard[];
  /** 背景的推導（記錄切分失敗時為 null）：ContextDoc 與背景數字磚共用。 */
  context: ContextModel | null;
  status: string;
  hold: boolean;
  created: string | null;
  rows: PromotedRow[];
  recordPath: string | null;
  /** 未轉出提示框的「前往」：只有討論詳情頁傳入（已封存討論頁沒有提示框）。 */
  onLocateCut?: (id: string) => void;
}

/** 兩頁共用的名稱清單（背景名稱籤、先前討論比對）：本討論的 slug 由 `self` 排除。 */
export function discussionNames(
  self: string,
  lists: { specNames?: string[]; changes?: ChangeItem[]; archivedChanges?: ArchivedItem[]; discussionSlugs?: string[] },
): ContextNames {
  return {
    specs: lists.specNames ?? [],
    active: (lists.changes ?? []).map((c) => c.name),
    archived: (lists.archivedChanges ?? []).map((a) => a.name),
    discussions: lists.discussionSlugs ?? [],
    self,
  };
}

/** 記錄全文（undefined＝載入中、null＝不存在）與頁已持有的清單 → `DiscussionDocData`。 */
export function discussionDocData(input: {
  text: string | null | undefined;
  status: string;
  hold: boolean;
  created: string | null;
  recordPath: string | null;
  rows: PromotedRow[];
  names: ContextNames;
  onLocateCut?: (id: string) => void;
}): DiscussionDocData {
  const sections = input.text ? splitDiscussionSections(input.text) : null;
  const conclusion = sections ? parseConclusion(sections.conclusion) : null;
  const decision = conclusion?.fields.Decision;
  return {
    sections,
    rounds: sections ? splitRounds(sections.rounds) : null,
    conclusion,
    cuts: decision !== undefined ? parseCuts(splitVerdict(decision).details).cuts : [],
    context: sections ? contextModel(sections.context, input.names) : null,
    status: input.status,
    hold: input.hold,
    created: input.created,
    rows: input.rows,
    recordPath: input.recordPath,
    onLocateCut: input.onLocateCut,
  };
}

type Translate = (key: string) => string;

const EMPTY: DocSummary = { badges: null, tiles: [], callout: null };

const fill = (template: string, values: Record<string, string | number>) =>
  Object.entries(values).reduce((s, [k, v]) => s.replace(`{${k}}`, String(v)), template);

/** 一份文件的閱讀卡標題區：數字磚與提示框、出處行。 */
interface DocHeaderParts {
  summary: (data: DiscussionDocData, t: Translate) => DocSummary;
  provenance: (data: DiscussionDocData, t: Translate) => ReactNode[];
}

/**
 * 討論文件閱讀卡標題區的推導單一入口（design D6，對稱 changeDocSummary）：數字磚與提示框全由記錄結構與清單
 * 推出，缺資料少一格、條件不成立沒有提示框；記錄未載入或切分失敗時皆空。討論文件沒有籤。
 */
export function discussionDocSummary(tab: DiscussionTab, data: DiscussionDocData, t: Translate): DocSummary {
  const parts = data.sections ? DOC_HEADERS[tab] : undefined;
  return parts ? parts.summary(data, t) : EMPTY;
}

/** 閱讀卡出處行（design D6）：結論為記錄路徑與「N 輪之後寫入」、討論過程為日期範圍與各 mode 輪數、背景為
 * 「記錄建立時寫入一次」與建立日、衍生變更為分組依據；已封存討論頁沒有記錄路徑段。 */
export function discussionDocProvenance(tab: DiscussionTab, data: DiscussionDocData, t: Translate): ReactNode[] {
  const parts = data.sections ? DOC_HEADERS[tab] : undefined;
  return parts ? parts.provenance(data, t) : [];
}

function conclusionTiles(data: DiscussionDocData, t: Translate): DocTile[] {
  const fields = data.conclusion?.fields;
  if (!fields) return [];
  const rowsOf = (md: string | undefined) => (md === undefined ? null : splitRows(md).length);
  const details = fields.Decision === undefined ? null : parseCuts(splitVerdict(fields.Decision).details).rest.length;
  const changes = new Set(data.cuts.flatMap((c) => c.names)).size;
  const tiles: (DocTile | null)[] = [
    details === null ? null : { value: details, label: t("conclusion.tileDetails") },
    data.cuts.length > 0 ? { value: data.cuts.length, label: fill(t("conclusion.tileCuts"), { n: changes }) } : null,
    rowsOf(fields["Rejected alternatives"]) === null
      ? null
      : { value: rowsOf(fields["Rejected alternatives"])!, label: t("conclusion.tileRejected") },
    rowsOf(fields.Deferred) === null ? null : { value: rowsOf(fields.Deferred)!, label: t("conclusion.tileDeferred") },
  ];
  return tiles.filter((tile): tile is DocTile => tile !== null);
}

/** 結論的未轉出提示框（live 討論、至少一刀整刀未轉出）：每刀一列附「前往」，末行接轉出指令籤。判定與刀卡同一支
 * `cutPromotionOf`（同一份衍生列），列出的刀＝畫成虛線框的刀。只排除已封存：引擎只拒絕從已封存的記錄轉出，
 * 已轉出、未保留的 live 討論照樣能轉出下一刀。 */
function unpromotedCallout(data: DiscussionDocData, t: Translate): ReactNode {
  if (data.status === "archived") return null;
  const stageOf = rowStage(data.rows);
  const todo = data.cuts.filter((cut) => cutPromotionOf(cut, stageOf) === "none");
  if (todo.length === 0) return null;
  const { onLocateCut } = data;
  const slug = data.context?.names.self;
  return (
    <Callout tone="manual" title={fill(t("conclusion.unpromotedCallout"), { n: todo.length })} className="my-0">
      <ul className="my-1 flex flex-col gap-0.5 text-sm">
        {todo.map((cut) => (
          <li key={cut.id} data-unpromoted-cut={cut.id} className="flex items-center gap-2">
            <span className={NAME_CHIP_CLS}>{cut.id}</span>
            {cut.title && (
              <span className="min-w-0">
                <InlineMarks text={cut.title} />
              </span>
            )}
            {onLocateCut && (
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="ml-auto h-6 shrink-0 gap-1 px-2"
                onClick={() => onLocateCut(cut.id)}
              >
                {t("tasks.locate")}
                <ArrowRight className="h-3.5 w-3.5" />
              </Button>
            )}
          </li>
        ))}
      </ul>
      {slug && (
        <p className="flex flex-wrap items-center gap-1.5 text-sm text-muted-foreground">
          {t("conclusion.unpromotedHint")}
          <CommandChip command={`/speclink-propose --from-discussion ${slug}`} />
        </p>
      )}
    </Callout>
  );
}

function roundsSummary(data: DiscussionDocData, t: Translate): DocSummary {
  const rounds = data.rounds;
  if (!rounds || rounds.length === 0) return EMPTY;
  const stats = roundStats(rounds);
  const tiles: DocTile[] = [
    { value: rounds.length, label: t("rounds.tileRounds") },
    { value: stats.days, label: t("rounds.tileDays") },
    ...stats.modes.map((m) => ({ value: m.count, label: m.mode })),
  ].slice(0, 4);
  if (!data.conclusion) return { ...EMPTY, tiles };
  const lastOpen = rounds[rounds.length - 1].fields.Open;
  const open = lastOpen === undefined ? null : openItems(lastOpen);
  return {
    ...EMPTY,
    tiles,
    callout: (
      <Callout tone="tip" title={t("rounds.converged")} className="my-0">
        <p className="text-sm">{fill(t("rounds.convergedBody"), { n: rounds.length })}</p>
        {open && (
          <p className="text-sm">
            {open.none ? t("rounds.convergedOpenNone") : fill(t("rounds.convergedOpenLeft"), { n: open.items.length })}
          </p>
        )}
      </Callout>
    ),
  };
}

function contextSummary(data: DiscussionDocData, t: Translate): DocSummary {
  const tile = (count: number, label: string): DocTile | null => (count > 0 ? { value: count, label: t(label) } : null);
  const ctx = data.context;
  // 先前討論：`Prior discussions:` 行在時恆有一格（含 0），行缺席時沒有這格。
  const prior = ctx && ctx.meta.prior !== null ? ctx.rel.prior.length : null;
  const tiles: (DocTile | null)[] = [
    tile(ctx?.rel.specs.length ?? 0, "context.tileSpecs"),
    tile(ctx?.rel.changes.length ?? 0, "context.tileChanges"),
    tile(ctx?.rel.archived.length ?? 0, "context.tileArchived"),
    prior === null ? null : { value: prior, label: t("context.tilePrior") },
  ];
  return {
    ...EMPTY,
    tiles: tiles.filter((tile): tile is DocTile => tile !== null),
    callout:
      data.rounds?.[0]?.mode === "assumptions" ? (
        <Callout tone="note" title={t("context.noGrill")} className="my-0">
          <p className="text-sm">{t("context.noGrillBody")}</p>
        </Callout>
      ) : null,
  };
}

function promoteSummary(data: DiscussionDocData, t: Translate): DocSummary {
  const { rows } = data;
  if (rows.length === 0) return EMPTY;
  // 已封存討論頁的退回列所在未知：兩格寧缺勿錯。
  const known = rows.every((r) => r.state !== null);
  const count = (state: PromotedRow["state"]) => rows.filter((r) => r.state === state).length;
  const withTasks = rows.filter((r) => r.tasks);
  const done = withTasks.reduce((n, r) => n + r.tasks!.done, 0);
  const total = withTasks.reduce((n, r) => n + r.tasks!.total, 0);
  const tiles: (DocTile | null)[] = [
    { value: rows.length, label: t("promoted.tileChanges") },
    known ? { value: count("archived"), label: t("promoted.tileArchived") } : null,
    known ? { value: count("active"), label: t("promoted.tileActive") } : null,
    withTasks.length > 0 ? { value: `${done} / ${total}`, label: t("promoted.tileTasks") } : null,
  ];
  const active = rows.filter((r) => r.state === "active");
  const last = data.status === "promoted" && !data.hold && active.length === 1 ? active[0] : null;
  return {
    ...EMPTY,
    tiles: tiles.filter((tile): tile is DocTile => tile !== null),
    callout: last && (
      <Callout tone="tip" title={t("promoted.lastCut")} className="my-0">
        <p className="text-sm">{fill(t("promoted.lastCutBody"), { name: last.name, stage: last.stage.label })}</p>
      </Callout>
    ),
  };
}

function roundsProvenance(data: DiscussionDocData, t: Translate): ReactNode[] {
  if (!data.rounds || data.rounds.length === 0) return [];
  const stats = roundStats(data.rounds);
  return [
    stats.first === stats.last ? stats.first : fill(t("rounds.dateRange"), { first: stats.first!, last: stats.last! }),
    ...stats.modes.map((m) => fill(t("rounds.modeRounds"), { mode: m.mode, n: m.count })),
  ];
}

/** 文件 → 標題區；「記錄」（切分失敗的整篇退回）沒有標題區。 */
const DOC_HEADERS: Partial<Record<DiscussionTab, DocHeaderParts>> = {
  conclusion: {
    summary: (data, t) => ({ ...EMPTY, tiles: conclusionTiles(data, t), callout: unpromotedCallout(data, t) }),
    provenance: (data, t) => [
      data.recordPath && <span className="font-mono">{data.recordPath}</span>,
      data.conclusion && data.rounds && fill(t("conclusion.afterRounds"), { n: data.rounds.length }),
    ],
  },
  rounds: { summary: roundsSummary, provenance: roundsProvenance },
  context: { summary: contextSummary, provenance: (data, t) => [t("context.writtenOnce"), data.created] },
  promote: {
    summary: promoteSummary,
    provenance: (data, t) =>
      data.rows.length === 0 ? [] : [t(data.cuts.length > 0 ? "promoted.byCuts" : "promoted.byOrder")],
  },
};

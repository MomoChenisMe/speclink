import { useEffect, useRef, useState } from "react";

import {
  REVIEW_LISTS,
  type ReviewList,
  type ReviewRules,
  type StationTicket,
  type TicketFinding,
  type TicketRound,
  type TicketStation,
} from "../adapter";
import { findingAxis } from "../docModel";
import { useI18n } from "../i18n";
import { cn } from "../lib/utils";
import { SEMANTIC_SURFACE } from "../tone";
import { BaselineSmellList } from "./BaselineSmellList";
import { Badge } from "./ui/badge";
import { Callout } from "./ui/callout";
import { ExpandToggle, SECTION_HEADING_CLS } from "./DocParts";
import { SeverityTag } from "./SeverityTag";
import type { DocSummary } from "./DetailShell";
import { DocSkeleton } from "./skeletons";

/** 階段詞：discovery →「首輪」、validation →「複驗」（LANGUAGE.md 詞條）；legacy 輪無。 */
const PHASE_KEY = {
  discovery: "ticket.phase.discovery",
  validation: "ticket.phase.validation",
} as const;

/** 描述行尾的結構 token `(accepted)`，允許尾端空白；只在行尾辨識，行中出現原樣顯示。 */
const ACCEPTED_TAIL = /\s*\(accepted\)\s*$/;

/** 描述原文 → 顯示文字＋是否標「已接受」（design D4）。 */
function splitAcceptedToken(text: string): { shown: string; accepted: boolean } {
  return ACCEPTED_TAIL.test(text)
    ? { shown: text.replace(ACCEPTED_TAIL, ""), accepted: true }
    : { shown: text, accepted: false };
}

/** 工單分頁的載入三態（design D3）：undefined＝載入在途、null＝載入完成無工單
 * （不存在、遠端 404、讀取或解析失敗）、物件＝有內容。 */
type TicketDoc = StationTicket | null | undefined;

/** 兩站工單的載入狀態（design D3／D5，兩個抽屜共用）：`key` 為 change 名或封存
 * 目錄名，null＝抽屜關閉或無主體（留住內容陪滑出動畫）；`present` 為分頁是否出現
 * ——出現時載入、隨 `gen` 重載、消失時清回未載入；換 `key` 先清回骨架再載。
 * latest-wins：每站一個序號，清空與換主體也推進序號，晚到的舊回應一律丟棄。
 * 失敗與無工單同一終態 null（spec：讀取或解析失敗顯示空態而非錯誤）。 */
export function useStationTickets(
  key: string | null,
  present: Record<TicketStation, boolean>,
  gen: number,
  load: ((key: string, station: TicketStation) => Promise<StationTicket | null>) | undefined,
): Record<TicketStation, TicketDoc> {
  return {
    review: useStationTicket("review", key, present.review, gen, load),
    verify: useStationTicket("verify", key, present.verify, gen, load),
  };
}

function useStationTicket(
  station: TicketStation,
  key: string | null,
  present: boolean,
  gen: number,
  load: ((key: string, station: TicketStation) => Promise<StationTicket | null>) | undefined,
): TicketDoc {
  const [doc, setDoc] = useState<TicketDoc>(undefined);
  const seq = useRef(0);
  const shownKey = useRef<string | null>(null);
  useEffect(() => {
    if (!key) return;
    const mine = ++seq.current;
    if (!present || key !== shownKey.current) setDoc(undefined);
    shownKey.current = key;
    if (!present) return;
    const settle = (v: TicketDoc) => {
      if (seq.current === mine) setDoc(v);
    };
    (load ? load(key, station) : Promise.resolve(null)).then(settle).catch(() => settle(null));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, present, gen]);
  return doc;
}

/** 兩個抽屜共用的分頁內容：三態依「抽屜文件載入以 skeleton 呈現」——載入中骨架、
 * 無工單空態文案、有內容交 [`TicketView`]。 */
export function TicketTabBody({ station, ticket }: { station: TicketStation; ticket: TicketDoc }) {
  const { t } = useI18n();
  if (ticket === undefined) return <DocSkeleton />;
  if (ticket === null) {
    return <div className="text-muted-foreground text-sm py-6">{t("ticket.empty")}</div>;
  }
  return <TicketView station={station} ticket={ticket} />;
}

export interface TicketViewProps {
  station: TicketStation;
  ticket: StationTicket;
}

const SEVERITIES: TicketFinding["severity"][] = ["CRITICAL", "WARNING", "SUGGESTION"];

/** 一輪三級嚴重度各自的條數。 */
function severityCounts(round: TicketRound): Record<TicketFinding["severity"], number> {
  const counts = { CRITICAL: 0, WARNING: 0, SUGGESTION: 0 };
  for (const f of round.findings) counts[f.severity] += 1;
  return counts;
}

/** 依序號升序的輪次（引擎不保證檔內順序）。 */
const sortedRounds = (ticket: StationTicket) => [...ticket.rounds].sort((a, b) => a.index - b.index);

/**
 * 工單文件的籤、數字磚與可蓋章提示框（spec「詳情頁的工單分頁」；design D8）：籤槽為「第 N 輪」與
 * 末輪階段詞，數字磚為輪數與末輪三級計數；末輪 CRITICAL 與 WARNING 皆為 0 時 tip「可以蓋章」並
 * 點名該站的 stamp 指令。工單未載入或不存在時全空。
 */
export function ticketSummary(
  station: TicketStation,
  ticket: StationTicket | null | undefined,
  t: (key: string) => string,
): DocSummary {
  if (!ticket || ticket.rounds.length === 0) return { badges: null, tiles: [], callout: null };
  const rounds = sortedRounds(ticket);
  const last = rounds[rounds.length - 1];
  const counts = severityCounts(last);
  const [before, after] = t("ticket.stampHow").split("{cmd}");
  return {
    badges: (
      <>
        <Badge variant="secondary">{roundLabel(t, last.index)}</Badge>
        {last.phase && <Badge variant="outline">{t(PHASE_KEY[last.phase])}</Badge>}
      </>
    ),
    tiles: [
      { value: rounds.length, label: t("ticket.tileRounds") },
      ...SEVERITIES.map((sev) => ({ value: counts[sev], label: t("ticket.tileLast").replace("{sev}", sev) })),
    ],
    callout:
      counts.CRITICAL === 0 && counts.WARNING === 0 ? (
        <Callout tone="tip" title={t("ticket.stampable")} className="my-0">
          <p className="text-sm">
            {counts.SUGGESTION > 0
              ? t("ticket.stampableHint").replace("{n}", String(counts.SUGGESTION))
              : t("ticket.stampableNone")}
          </p>
          <p className="text-sm text-muted-foreground">
            {before}
            <code className="font-mono">{`speclink ${station} stamp`}</code>
            {after}
          </p>
        </Callout>
      ) : null,
  };
}

/**
 * 結構化工單（spec desktop-app「詳情頁的工單分頁」、design D8）：輪次依序號升序逐輪一張卡，
 * 末輪展開、其餘收合；展開後先列套用規則列（該輪有記錄時），再列範圍路徑與 finding（色章、
 * 軸向籤、等寬路徑、原文描述、行尾 accepted 籤）。站名、輪數與計數在閱讀卡標題區（ticketSummary），內文不另畫段標題列。
 * 唯讀、無虛擬化。
 */
export function TicketView({ station, ticket }: TicketViewProps) {
  const rounds = sortedRounds(ticket);
  const smells = ticket.reviewBaselineSmells ?? [];
  return (
    <div data-ticket-view data-station={station} className="flex flex-col gap-2">
      {/* key 綁輪數：追加輪後整組重掛，預設「末輪展開、其餘收合」重新成立
          （spec Scenario「工單追加輪後分頁更新」），不必逐輪同步狀態。輪的 key 用
          陣列位置——引擎不檢查 `## Round N` 唯一，手改出的同序號不得撞 key。 */}
      <ol key={rounds.length} className="flex flex-col gap-1.5">
        {rounds.map((round, i) => (
          <RoundBlock key={i} round={round} smells={smells} defaultOpen={i === rounds.length - 1} />
        ))}
      </ol>
    </div>
  );
}

function roundLabel(t: (key: string) => string, index: number): string {
  return t("ticket.round").replace("{n}", String(index));
}

/** 輪卡（與排程文件同一種邊框卡片）：卡頭 16px「第 N 輪」（本頁目錄錨點）、階段詞、範圍檔數、條數與
 * 各嚴重度計數籤；展開後先列套用規則列（該輪有記錄時）、再列範圍路徑與 finding。 */
function RoundBlock({ round, smells, defaultOpen }: { round: TicketRound; smells: string[]; defaultOpen: boolean }) {
  const { t } = useI18n();
  const [open, setOpen] = useState(defaultOpen);
  const phase = round.phase ? t(PHASE_KEY[round.phase]) : null;
  const counts = severityCounts(round);
  const meta = [
    phase,
    t("ticket.scopeFiles").replace("{n}", String(round.scope.length)),
    t("ticket.findings").replace("{n}", String(round.findings.length)),
  ].filter(Boolean);
  return (
    <li data-ticket-round={round.index} className="overflow-hidden rounded-md border border-border/60 bg-muted/20">
      <ExpandToggle
        data-ticket-round-header
        data-ticket-round-toggle
        open={open}
        onToggle={() => setOpen((v) => !v)}
        className="flex-wrap gap-x-2 gap-y-1 px-3 py-2"
      >
        <h3 data-doc-heading className={SECTION_HEADING_CLS}>
          {roundLabel(t, round.index)}
        </h3>
        <span className="text-xs tabular-nums text-muted-foreground">{meta.map((m) => ` · ${m}`).join("")}</span>
        {SEVERITIES.filter((sev) => counts[sev] > 0).map((sev) => (
          <SeverityTag key={sev} severity={sev} data-ticket-count={sev} className="tabular-nums">
            {`${sev} ${counts[sev]}`}
          </SeverityTag>
        ))}
      </ExpandToggle>
      {open && (
        <div className="flex flex-col gap-2 border-t border-border/60 px-3 py-2">
          {round.rules && <RulesRow rules={round.rules} smells={smells} />}
          <ul className="flex flex-col gap-0.5">
            {round.scope.map((path) => (
              <li key={path} data-ticket-scope-path className="break-all font-mono text-[11px] text-muted-foreground">
                {path}
              </li>
            ))}
          </ul>
          {round.findings.length === 0 ? (
            <p className="text-xs text-muted-foreground">{t("ticket.noFindings")}</p>
          ) : (
            <ul className="flex flex-col gap-1.5">
              {round.findings.map((f, i) => (
                <FindingRow key={`${f.path}-${i}`} finding={f} />
              ))}
            </ul>
          )}
        </div>
      )}
    </li>
  );
}

/** 專案規則清單的嚴重度籤：清單名的大寫（critical → CRITICAL）。 */
const severityOf = (list: ReviewList) => list.toUpperCase();

/** 套用規則列（spec「詳情頁的工單分頁」）：預設收合；列頭依 baseline 與專案規則條數寫模式並附非零的
 * 條數籤，replace 且沒有專案規則時琥珀底；展開後列預設清單（或寫明沒有套用）、各嚴重度的規則原文與
 * 末尾註記。N 取 `smells` 的長度。 */
function RulesRow({ rules, smells }: { rules: ReviewRules; smells: string[] }) {
  const { t } = useI18n();
  const [open, setOpen] = useState(false);
  const count = (key: string, n: number) => t(key).replace("{count}", String(n));
  const groups = REVIEW_LISTS.filter((list) => rules[list].length > 0);
  const extend = rules.baseline === "extend";
  const mode = extend
    ? groups.length > 0
      ? "ticket.rules.extendWithRules"
      : "ticket.rules.baselineOnly"
    : groups.length > 0
      ? "ticket.rules.replaceWithRules"
      : "ticket.rules.replaceEmpty";
  return (
    <div
      data-ticket-rules
      className={cn("rounded-md border", !extend && groups.length === 0 ? SEMANTIC_SURFACE.warning : "border-border/60")}
    >
      <ExpandToggle
        data-ticket-rules-toggle
        open={open}
        onToggle={() => setOpen((v) => !v)}
        className="flex-wrap gap-x-2 gap-y-1 px-2 py-1.5 text-xs"
      >
        <span data-ticket-rules-label className="font-medium">
          {t("ticket.rules.label")}
        </span>
        <span data-ticket-rules-mode className="text-muted-foreground">
          {count(mode, smells.length)}
        </span>
        {groups.map((list) => (
          <SeverityTag key={list} severity={severityOf(list)} data-ticket-rules-count={severityOf(list)} className="tabular-nums">
            {`${severityOf(list)} ${rules[list].length}`}
          </SeverityTag>
        ))}
      </ExpandToggle>
      {open && (
        <div data-ticket-rules-body className="flex flex-col gap-2 border-t border-border/60 px-2 py-2 text-xs">
          {extend ? (
            <div data-ticket-rules-baseline className="flex flex-col gap-1">
              <div className="flex items-center gap-1.5">
                <SeverityTag data-ticket-severity severity="SUGGESTION" />
                <span>{count("ticket.rules.baselineOnly", smells.length)}</span>
              </div>
              <BaselineSmellList names={smells} />
            </div>
          ) : (
            <p data-ticket-rules-baseline className="m-0 text-muted-foreground">
              {count("ticket.rules.baselineNotApplied", smells.length)}
            </p>
          )}
          {groups.map((list) => (
            <div key={list} data-ticket-rules-group={severityOf(list)} className="flex flex-col gap-1">
              <div className="flex items-center gap-1.5">
                <SeverityTag data-ticket-severity severity={severityOf(list)} />
                <span>{count("ticket.rules.projectCount", rules[list].length)}</span>
              </div>
              {/* 規則原文：不翻譯、不截斷、可換行。 */}
              <ul className="m-0 flex list-disc flex-col gap-0.5 pl-5">
                {rules[list].map((rule, i) => (
                  <li key={i} className="whitespace-pre-wrap break-words">
                    {rule}
                  </li>
                ))}
              </ul>
            </div>
          ))}
          <p data-ticket-rules-note className="m-0 text-muted-foreground">
            {t("ticket.rules.note")}
          </p>
        </div>
      )}
    </div>
  );
}

function FindingRow({ finding }: { finding: TicketFinding }) {
  const { t } = useI18n();
  // 先切行尾 accepted token、再取開頭的軸向前綴（「Standards: (accepted) 語意不清」的 token 在行中，原樣）。
  const { shown: tail, accepted } = splitAcceptedToken(finding.text);
  const { axis, text: shown } = findingAxis(tail);
  return (
    <li data-ticket-finding className="flex flex-wrap items-center gap-x-1.5 gap-y-0.5 text-xs">
      <SeverityTag data-ticket-severity severity={finding.severity} />
      {axis && (
        <Badge data-ticket-axis variant="outline" className="shrink-0">
          {axis}
        </Badge>
      )}
      <span data-ticket-path className="min-w-0 break-all font-mono text-[11px] text-muted-foreground">
        {finding.path}
      </span>
      {accepted && (
        <span
          data-ticket-accepted
          className="shrink-0 rounded border border-border/60 px-1 py-0.5 text-[10px] font-medium text-muted-foreground"
        >
          {t("ticket.accepted")}
        </span>
      )}
      {/* 描述原文：不翻譯、不截斷、可換行。 */}
      <span data-ticket-text className="min-w-0 basis-full whitespace-pre-wrap break-words">
        {shown}
      </span>
    </li>
  );
}

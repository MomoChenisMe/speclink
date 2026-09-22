import { useState } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";

import type { StationTicket, TicketFinding, TicketRound, TicketStation } from "../adapter";
import { useI18n } from "../i18n";
import { SEMANTIC_SURFACE, SEMANTIC_TONE } from "../tone";
import { DocSkeleton } from "./skeletons";

/** 嚴重度色章：CRITICAL 破壞色、WARNING 琥珀、SUGGESTION 中性——與 AnalyzePanel 的
 * 發現卡同語意同色；三紅分工（tone.ts 表頭）之外不另立一紅。標籤維持英文，與
 * 工單原文一致。 */
const SEVERITY_CLS: Record<TicketFinding["severity"], string> = {
  CRITICAL: "bg-destructive/15 text-destructive",
  WARNING: `${SEMANTIC_SURFACE.warning} ${SEMANTIC_TONE.warning}`,
  SUGGESTION: "bg-muted text-muted-foreground",
};

/** 站名沿用分頁標籤——分頁名與狀態列章籤的站名一致（proposal）。 */
const STATION_KEY: Record<TicketStation, string> = {
  review: "drawer.tab.review",
  verify: "drawer.tab.verify",
};

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
export type TicketDoc = StationTicket | null | undefined;

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

/**
 * 結構化工單（spec desktop-app「詳情抽屜的工單分頁」、design D4）：段標題列站名、
 * 末輪輪數、末輪階段詞與三級計數；輪次依序號升序逐輪可收合，末輪展開、其餘收合；
 * 每條 finding 一列（色章、等寬路徑、原文描述、行尾 accepted 籤）。唯讀、無虛擬化。
 */
export function TicketView({ station, ticket }: TicketViewProps) {
  const { t } = useI18n();
  // IPC 邊界防禦：rounds 缺席或空陣列（畸形 payload）以空態呈現，不讓抽屜崩潰。
  const rounds = [...(ticket.rounds ?? [])].sort((a, b) => a.index - b.index);
  const last = rounds[rounds.length - 1];
  if (!last) {
    return <p className="text-xs text-muted-foreground">{t("ticket.empty")}</p>;
  }
  const counts = { CRITICAL: 0, WARNING: 0, SUGGESTION: 0 };
  for (const f of last.findings) {
    if (f.severity in counts) counts[f.severity] += 1;
  }
  const phase = last.phase ? t(PHASE_KEY[last.phase]) : null;
  const title = `${t(STATION_KEY[station])} · ${roundLabel(t, last.index)}${phase ? `（${phase}）` : " · "}`;
  return (
    <div data-ticket-view data-station={station} className="flex flex-col gap-2">
      <div data-ticket-header className="flex flex-wrap items-baseline gap-x-1 text-sm">
        <span className="font-semibold">{title}</span>
        <span className="text-xs tabular-nums text-muted-foreground">
          <span className={counts.CRITICAL > 0 ? SEMANTIC_TONE.danger : undefined}>
            CRITICAL {counts.CRITICAL}
          </span>
          {" · "}
          <span className={counts.WARNING > 0 ? SEMANTIC_TONE.warning : undefined}>
            WARNING {counts.WARNING}
          </span>
          {" · "}
          <span>SUGGESTION {counts.SUGGESTION}</span>
        </span>
      </div>
      {/* key 綁末輪序號：追加輪後整組重掛，預設「末輪展開、其餘收合」重新成立
          （spec Scenario「工單追加輪後分頁更新」），不必逐輪同步狀態。 */}
      <ol key={last.index} className="flex flex-col gap-1.5">
        {rounds.map((round) => (
          <RoundBlock key={round.index} round={round} defaultOpen={round.index === last.index} />
        ))}
      </ol>
    </div>
  );
}

function roundLabel(t: (key: string) => string, index: number): string {
  return t("ticket.round").replace("{n}", String(index));
}

function RoundBlock({ round, defaultOpen }: { round: TicketRound; defaultOpen: boolean }) {
  const { t } = useI18n();
  const [open, setOpen] = useState(defaultOpen);
  const [scopeOpen, setScopeOpen] = useState(false);
  const phase = round.phase ? t(PHASE_KEY[round.phase]) : null;
  return (
    <li data-ticket-round={round.index} className="rounded-md border border-border/60 bg-muted/20">
      <div
        data-ticket-round-header
        className="flex flex-wrap items-center gap-x-1 px-2 py-1 text-xs text-muted-foreground"
      >
        <button
          type="button"
          data-ticket-round-toggle
          aria-expanded={open}
          onClick={() => setOpen((v) => !v)}
          className="inline-flex items-center gap-1 font-semibold text-foreground hover:underline"
        >
          {open ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />}
          {`${roundLabel(t, round.index)}${phase ? ` · ${phase}` : ""}`}
        </button>
        {" · "}
        <button
          type="button"
          aria-expanded={scopeOpen}
          onClick={() => setScopeOpen((v) => !v)}
          className="hover:text-foreground hover:underline"
        >
          {t("ticket.scopeFiles").replace("{n}", String(round.scope.length))}
        </button>
        {" · "}
        <span className="tabular-nums">
          {t("ticket.findings").replace("{n}", String(round.findings.length))}
        </span>
      </div>
      {scopeOpen && (
        <ul className="flex flex-col gap-0.5 border-t border-border/60 px-2 py-1.5">
          {round.scope.map((path) => (
            <li key={path} data-ticket-scope-path className="break-all font-mono text-[11px] text-muted-foreground">
              {path}
            </li>
          ))}
        </ul>
      )}
      {open && (
        <div className="border-t border-border/60 px-2 py-1.5">
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

function FindingRow({ finding }: { finding: TicketFinding }) {
  const { t } = useI18n();
  const { shown, accepted } = splitAcceptedToken(finding.text);
  return (
    <li data-ticket-finding className="flex flex-wrap items-center gap-x-1.5 gap-y-0.5 text-xs">
      <span
        data-ticket-severity
        className={`shrink-0 rounded px-1 py-0.5 text-[10px] font-medium ${
          SEVERITY_CLS[finding.severity] ?? "bg-muted text-muted-foreground"
        }`}
      >
        {finding.severity}
      </span>
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

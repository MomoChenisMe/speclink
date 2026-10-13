import { useState } from "react";
import { Check, ChevronDown, X } from "lucide-react";

import { itemMarkdown } from "../docModel";
import { openItems, splitRounds, splitRows, splitVerdict, type DiscussionRound } from "../discussionModel";
import { useI18n } from "../i18n";
import { cn } from "../lib/utils";
import { SEMANTIC_SURFACE, SEMANTIC_TONE } from "../tone";
import { InlineMarks, ROW_MD_CLS, RowList } from "./DocParts";
import { Markdown } from "./Markdown";

/** 節點色點：assumptions 主色、interview 取看板討論欄色相，其他 mode 中性（mode 字串透傳、不設白名單）。 */
const MODE_DOT: Record<string, string> = {
  assumptions: "bg-primary",
  interview: "bg-stage-discussion",
};

/** 欄位小標。 */
function FieldLabel({ label }: { label: string }) {
  return <div className="mb-1 text-xs font-medium text-muted-foreground">{label}</div>;
}

/**
 * 討論過程文件（desktop-discussion-doc-views design D3；spec「討論輪以卡片呈現」）：時間軸上每輪一張卡、由新到舊
 * 排列（只改呈現，記錄檔的輪順序不變），卡頭為輪次（本頁目錄錨點）、焦點、mode 與日期；首末輪（畫面最下與最上）
 * 預設展開、其餘收合，點卡頭切換。任一輪標題不符 scaffold 時整篇 Markdown 退回。討論詳情頁與已封存討論頁共用。
 */
export function RoundsDoc({ text, empty }: { text: string; empty?: string }) {
  // 使用者點過的輪：記錄檔中的位置 → 展開與否；沒點過的依預設（首末輪展開）。以記錄位置而非畫面位置記錄：引擎只往
  // 檔尾追加，新輪插在最上時不錯位；手改記錄出現重號時兩張卡也各自切換。
  const [toggled, setToggled] = useState<Record<number, boolean>>({});
  const rounds = splitRounds(text);
  if (rounds === null) return <Markdown content={text} empty={empty} />;
  if (rounds.length === 0) return <Markdown content="" empty={empty} />;
  // 輪次大者在上；同號（手改記錄）時記錄中較後者在上。
  const newestFirst = rounds.map((round, at) => ({ round, at })).sort((a, b) => b.round.round - a.round.round || b.at - a.at);
  return (
    <ol
      data-round-timeline
      className="relative flex flex-col gap-3 pl-6 before:absolute before:top-3 before:bottom-3 before:left-[6px] before:w-px before:bg-border"
    >
      {newestFirst.map(({ round, at }, i) => {
        const open = toggled[at] ?? (i === 0 || i === newestFirst.length - 1);
        return (
          <RoundCard
            key={at}
            round={round}
            open={open}
            onToggle={() => setToggled((prev) => ({ ...prev, [at]: !open }))}
          />
        );
      })}
    </ol>
  );
}

function RoundCard({ round, open, onToggle }: { round: DiscussionRound; open: boolean; onToggle: () => void }) {
  const { t } = useI18n();
  const [focus = "", ...focusRest] = (round.fields.Focus ?? "").split("\n");
  return (
    <li data-round={round.round} className="relative">
      <span
        aria-hidden="true"
        data-round-dot
        className={cn(
          "absolute top-3.5 -left-6 h-3.5 w-3.5 rounded-full border-2 border-background",
          MODE_DOT[round.mode] ?? "bg-muted-foreground/50",
        )}
      />
      <div className="overflow-hidden rounded-xl border border-border">
        <h3>
          <button
            type="button"
            aria-expanded={open}
            onClick={onToggle}
            className="flex w-full min-w-0 cursor-(--cursor-interaction) items-center gap-2 px-4 py-2.5 text-left transition-colors hover:bg-foreground/5"
          >
            {/* 輪次籤是靜態編號，與同列 mode／日期同層級——走中性，不佔用主色。 */}
            <span
              data-doc-heading
              className="shrink-0 rounded-full bg-muted px-2 py-0.5 text-xs font-semibold text-muted-foreground"
            >
              {t("rounds.roundN").replace("{n}", String(round.round))}
            </span>
            {/* 收合時一行截斷、展開時折行完整顯示（長焦點不被裁掉）。 */}
            <span data-round-focus className={cn("min-w-0 flex-1 text-sm font-medium", open ? "break-words" : "truncate")}>
              {focus.trim() && <InlineMarks text={focus.trim()} />}
            </span>
            <span className="shrink-0 rounded-full bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">
              {round.mode}
            </span>
            <span className="shrink-0 text-xs text-muted-foreground tabular-nums">{round.date}</span>
            <ChevronDown
              aria-hidden="true"
              className={cn("h-3.5 w-3.5 shrink-0 text-muted-foreground transition-transform", open && "rotate-180")}
            />
          </button>
        </h3>
        {open && (
          <div className="flex flex-col gap-3 border-t border-border/70 px-4 py-3">
            {round.lead && <Markdown content={round.lead} />}
            {focusRest.join("\n").trim() && <Markdown content={focusRest.join("\n").trim()} />}
            {round.fields.Position !== undefined && <PositionField md={round.fields.Position} />}
            {round.fields["Ruled out"] !== undefined && (
              <div data-round-field="ruledOut">
                <FieldLabel label={t("rounds.ruledOut")} />
                <ul className="flex flex-col gap-1.5">
                  {splitRows(round.fields["Ruled out"]).map((row, i) => (
                    <li key={i} data-ruled-out className="flex items-start gap-2">
                      <X aria-hidden="true" className="mt-1.5 h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                      <Markdown content={row} className={ROW_MD_CLS} />
                    </li>
                  ))}
                </ul>
              </div>
            )}
            {round.fields.Open !== undefined && <OpenField md={round.fields.Open} />}
          </div>
        )}
      </div>
    </li>
  );
}

/** 立場：定論句中等字重、其後列點為細線列。 */
function PositionField({ md }: { md: string }) {
  const { t } = useI18n();
  const { verdict, details } = splitVerdict(md);
  return (
    <div data-round-field="position">
      <FieldLabel label={t("rounds.position")} />
      {verdict && (
        <p data-position-verdict className="font-medium">
          <InlineMarks text={verdict} />
        </p>
      )}
      {details.length > 0 && (
        <div className="mt-2">
          <RowList items={details.map(itemMarkdown)} />
        </div>
      )}
    </div>
  );
}

/** 待決：收斂標記為一行帶勾選圖示的灰字原文，否則每條一顆琥珀籤。 */
function OpenField({ md }: { md: string }) {
  const { t } = useI18n();
  const open = openItems(md);
  return (
    <div data-round-field="open">
      <FieldLabel label={t("rounds.open")} />
      {open.none ? (
        <div data-open-none className="flex items-center gap-1.5 text-sm text-muted-foreground">
          <Check aria-hidden="true" className="h-3.5 w-3.5 shrink-0" />
          {open.items[0]}
        </div>
      ) : (
        <div className="flex flex-wrap gap-1.5">
          {open.items.map((item, i) => (
            <span
              key={i}
              data-open-item
              className={cn(
                "rounded-full border px-2 py-0.5 text-xs",
                SEMANTIC_SURFACE.warning,
                SEMANTIC_TONE.warning,
              )}
            >
              <InlineMarks text={item} />
            </span>
          ))}
        </div>
      )}
    </div>
  );
}

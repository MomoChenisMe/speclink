import { useState } from "react";
import { SortableContext, useSortable, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import {
  Archive,
  ArrowUpRight,
  Check,
  ChevronDown,
  ChevronRight,
  Copy,
  MessageSquareText,
} from "lucide-react";

import type { ArchivedItem, ChangeItem, DiscussionItem, SearchHit } from "../adapter";
import { cardDndId } from "../boardDnd";
import { useI18n } from "../i18n";
import { changeStage, STAGE_BADGE } from "../stage";
import { Button } from "./ui/button";
import { Card, CardContent, CardHeader } from "./ui/card";
import { CardNameRow } from "./CardNameRow";
import { HighlightText } from "./HighlightText";
import { ImproveStamp, isImproveKind } from "./ImproveStamp";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";

/**
 * promoted_to 子變更的階段標示（純前端由清單存在性派生）——active 清單命中
 * 依看板欄位規則；封存清單命中為已封存；兩者皆無為已刪除（討論維持已轉出
 * 不回退，歷史事實不回滾）。回傳 i18n key，呼叫端以 t() 取顯示字串。
 */
export function discussionChipStage(
  name: string,
  changes: ChangeItem[],
  archived: ArchivedItem[],
): string {
  const active = changes.find((c) => c.name === name);
  if (active) return `stage.${changeStage(active)}`;
  if (archived.some((a) => a.name === name)) return "discussion.chipArchived";
  return "discussion.chipDeleted";
}

/**
 * promoted 子變更 chip 的配色（design D2）——與 discussionChipStage 同分類規則：
 * active 命中取看板階段的 STAGE_BADGE teal 濃度、封存命中為中性色、皆無為
 * destructive 加刪除線（已刪除）。與看板欄配色同一來源，不引入新色輪。
 */
export function discussionChipClass(
  name: string,
  changes: ChangeItem[],
  archived: ArchivedItem[],
): string {
  const active = changes.find((c) => c.name === name);
  if (active) return STAGE_BADGE[changeStage(active)];
  if (archived.some((a) => a.name === name)) return "bg-muted text-muted-foreground";
  return "bg-destructive/15 text-destructive line-through";
}

export interface DiscussionColumnProps {
  /** 看板上的討論（active 清單；封存討論不進此欄）。 */
  discussions: DiscussionItem[];
  /** active change 清單（chips 階段派生）。 */
  changes: ChangeItem[];
  /** 已封存 change 清單（chips 已封存態派生）。 */
  archived: ArchivedItem[];
  onOpenDiscussion?: (slug: string) => void;
  /** concluded 卡的封存動詞（app 端接確認流程）。轉為變更（promote）已自 GUI 撤除。 */
  onArchiveDiscussion?: (slug: string) => void;
  /**
   * 欄內拖排（design D6）：true 時全卡（open／concluded）掛 sortable——
   * 需在宿主的 DndContext 內；promoted 收合列是衍生樹檢視，不參與拖排。
   */
  sortable?: boolean;
  /** 搜尋字串（design D7）：slug 子字串命中時高亮。 */
  highlight?: string;
  /** 全文命中（design D6）：命中的討論卡呈 snippet 行。 */
  fulltextHits?: SearchHit[];
}

const STATUS_BADGE: Record<string, { labelKey: string; cls: string }> = {
  open: { labelKey: "discussion.statusOpen", cls: "bg-primary/8 text-primary/70" },
  concluded: { labelKey: "discussion.statusConcluded", cls: "bg-primary/12 text-primary" },
};

export function DiscussionCard({
  d,
  onOpenDiscussion,
  onArchiveDiscussion,
  highlight,
  hit,
}: { d: DiscussionItem; hit?: SearchHit } & Pick<
  DiscussionColumnProps,
  "onOpenDiscussion" | "onArchiveDiscussion" | "highlight"
>) {
  const { t } = useI18n();
  const badge = STATUS_BADGE[d.status] ?? STATUS_BADGE.open;
  return (
    <Card
      data-discussion={d.slug}
      className="group cursor-pointer transition-[border-color,box-shadow] hover:border-primary/60 hover:shadow-md"
      onClick={() => onOpenDiscussion?.(d.slug)}
    >
      <CardHeader className="p-3 flex-row items-start gap-1.5">
        {/* slug（檔名）為標題——CLI 動詞把手，等寬強調；topic 降為卡身描述（LANGUAGE.md
            受控例外）。名稱列與變更卡共用（骨架統一）。 */}
        <CardNameRow text={d.slug} copyLabel={t("discussion.copySlug")} highlight={highlight} />
        <span className={`shrink-0 rounded-full px-1.5 py-0.5 text-[10px] font-semibold ${badge.cls}`}>
          {t(badge.labelKey)}
        </span>
        {/* 改進小章（spec「看板討論卡片的改進標示」）：隨 kind 恆定——已轉出與
            已封存側同樣顯示，不隨生命週期狀態變化或消失。 */}
        {isImproveKind(d.kind) && <ImproveStamp />}
        {/* 建立者圓點（anatomy 識別列右端）：全名進 tooltip、卡面不直出——與變更卡同款。 */}
        {d.createdBy && (
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <span
                  aria-label={d.createdBy}
                  className="inline-flex h-4 w-4 shrink-0 items-center justify-center rounded-full bg-muted text-[9px] font-bold text-muted-foreground"
                >
                  {d.createdBy.charAt(0).toUpperCase()}
                </span>
              </TooltipTrigger>
              <TooltipContent>{d.createdBy}</TooltipContent>
            </Tooltip>
          </TooltipProvider>
        )}
      </CardHeader>
      <CardContent className="p-3 pt-0 gap-2">
        <span className="text-xs leading-snug text-foreground/80">
          <HighlightText text={d.topic} query={highlight} />
        </span>
        {/* meta 列（anatomy 三列骨架）：輪數與建立時間並排；建立者圓點已上移識別列。 */}
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <span className="tabular-nums">{t("common.rounds").replace("{n}", String(d.rounds))}</span>
          <span className="tabular-nums">{d.created}</span>
        </div>
        {/* stopPropagation 掛在按鈕自身而非整列：鈕旁空白仍冒泡開討論抽屜。 */}
        {d.status === "concluded" && (
          <div className="flex gap-1.5">
            <Button
              variant="outline"
              size="sm"
              className="h-6 gap-1 px-2 text-xs"
              onClick={(e) => {
                e.stopPropagation();
                onArchiveDiscussion?.(d.slug);
              }}
            >
              <Archive className="h-3 w-3" /> {t("common.archive")}
            </Button>
          </div>
        )}
        {/* 全文命中 snippet 行（design D6/D7）：記錄全文命中時呈前後文＋高亮。 */}
        {hit && (
          <div
            data-snippet
            className="flex items-start gap-1 border-t border-border/40 pt-1.5 text-[11px] leading-snug text-muted-foreground"
          >
            <MessageSquareText className="mt-0.5 h-3 w-3 shrink-0" />
            <span className="min-w-0">
              <span className="font-mono">{hit.artifact}</span>{" "}
              <HighlightText text={hit.snippet} query={highlight} />
            </span>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

/** 可拖排的全卡包裝：拖曳時原卡留在原位變淡，移動視覺由宿主 DragOverlay 呈現。 */
function SortableDiscussionCard({
  d,
  ...rest
}: { d: DiscussionItem; hit?: SearchHit } & Pick<
  DiscussionColumnProps,
  "onOpenDiscussion" | "onArchiveDiscussion" | "highlight"
>) {
  const { t } = useI18n();
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: cardDndId("discussion", d.slug),
  });
  return (
    <div
      ref={setNodeRef}
      style={{
        transform: CSS.Transform.toString(transform),
        transition,
        ...(isDragging ? { opacity: 0.35 } : undefined),
      }}
      {...attributes}
      {...listeners}
      aria-label={t("kanban.dragCard").replace("{name}", d.topic)}
    >
      <DiscussionCard d={d} {...rest} />
    </div>
  );
}

function PromotedRow({
  d,
  changes,
  archived,
  onOpenDiscussion,
}: { d: DiscussionItem } & Pick<DiscussionColumnProps, "changes" | "archived" | "onOpenDiscussion">) {
  const { t } = useI18n();
  const [copied, setCopied] = useState(false);
  const copySlug = (e: React.MouseEvent) => {
    e.stopPropagation();
    void navigator.clipboard?.writeText(d.slug);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };
  // 衍生樹細列：slug（檔名）為首行錨點——CLI 動詞把手，帶複製鈕（LANGUAGE.md
  // 受控例外，desktop-ux-polish 擴充）；topic 降為次行描述；子變更以樹狀前綴
  // 逐列列出——父子（討論→衍生變更）關係一眼可讀。
  return (
    <div
      role="button"
      tabIndex={0}
      data-discussion={d.slug}
      className="group cursor-pointer rounded-md border border-border/60 bg-background/60 px-2 py-1.5 text-left transition-colors hover:border-primary/60"
      onClick={() => onOpenDiscussion?.(d.slug)}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") onOpenDiscussion?.(d.slug);
      }}
    >
      {/* 複製鈕行內尾隨（design D4 複製鈕位置規則）：break-all 多行 slug 的按鈕
          直接跟在最後一個字元後流動，不以 flex 推至列右緣。 */}
      <span className="min-w-0 break-all font-mono text-xs font-semibold leading-tight">
        {d.slug}
        <Button
          type="button"
          variant="ghost"
          size="icon"
          aria-label={t("discussion.copySlug")}
          className={`ml-1 inline-flex h-4 w-4 align-text-bottom text-muted-foreground hover:text-foreground transition-opacity ${copied ? "opacity-100" : "opacity-0 group-hover:opacity-100"}`}
          onClick={copySlug}
        >
          {copied ? <Check className="h-3 w-3 text-primary" /> : <Copy className="h-3 w-3" />}
        </Button>
      </span>
      <span className="mt-0.5 block truncate text-[11px] text-muted-foreground">{d.topic}</span>
      <span className="mt-1 flex flex-col gap-0.5">
        {d.promotedTo.map((name, i) => (
          <span key={name} className="flex items-center gap-1 text-[11px] leading-tight">
            <span className="shrink-0 font-mono text-muted-foreground">
              {i === d.promotedTo.length - 1 ? "└" : "├"}
            </span>
            <span className="min-w-0 truncate font-medium">{name}</span>
            <span
              className={`shrink-0 rounded-full px-1.5 py-0.5 text-[10px] ${discussionChipClass(name, changes, archived)}`}
            >
              {t(discussionChipStage(name, changes, archived))}
            </span>
          </span>
        ))}
      </span>
    </div>
  );
}

/**
 * 看板第 0 欄「討論」（兩級呈現，上下兩區同屏、不互斥切換；design D3）：
 * - 上區：open／concluded 全尺寸卡（open 唯讀、concluded 帶「封存」動詞——
 *   轉為變更已自 GUI 撤除）。
 * - 欄底：「已轉出 N」常駐收合列（有 promoted 時呈現、預設收合、不持久化），
 *   點按就地展開 promoted 衍生樹細列（slug 首行＋topic＋衍生變更樹＋階段 chip）。
 */
export function DiscussionColumn({
  discussions,
  changes,
  archived,
  onOpenDiscussion,
  onArchiveDiscussion,
  sortable,
  highlight,
  fulltextHits,
}: DiscussionColumnProps) {
  const { t } = useI18n();
  const full = discussions.filter((d) => d.status !== "promoted");
  const promoted = discussions.filter((d) => d.status === "promoted");
  // D3：欄底收合列的展開狀態——元件 local、預設收合、不跨啟動持久化。
  const [promotedOpen, setPromotedOpen] = useState(false);
  const hitBySlug = new Map(
    (fulltextHits ?? []).filter((h) => h.kind === "discussion").map((h) => [h.id, h]),
  );
  const fullCards = full.map((d) =>
    sortable ? (
      <SortableDiscussionCard
        key={d.slug}
        d={d}
        highlight={highlight}
        hit={hitBySlug.get(d.slug)}
        onOpenDiscussion={onOpenDiscussion}
        onArchiveDiscussion={onArchiveDiscussion}
      />
    ) : (
      <DiscussionCard
        key={d.slug}
        d={d}
        highlight={highlight}
        hit={hitBySlug.get(d.slug)}
        onOpenDiscussion={onOpenDiscussion}
        onArchiveDiscussion={onArchiveDiscussion}
      />
    ),
  );
  // 欄頭（色條／圖示／計數）一律中性：討論欄不是生命週期階段，主色深淺階梯是
  // 看板三欄的語彙，照抄會讓「顏色＝階段」的讀法失準（系統匣的討論分區同樣中性）。
  return (
    <div
      data-column="discussions"
      className="flex h-full min-h-0 flex-1 min-w-[250px] max-w-[360px] flex-col gap-2 rounded-xl border-t-4 border-t-border bg-muted/40 p-2"
    >
      <div className="flex items-center gap-1.5 px-1.5 pt-0.5 shrink-0">
        <MessageSquareText className="h-3.5 w-3.5 text-muted-foreground/60" />
        <h2 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
          {t("discussion.heading")}
        </h2>
        <div className="flex-1" />
        <span
          data-testid="column-count"
          className="inline-flex items-center justify-center min-w-5 h-5 px-1.5 rounded-full text-[11px] font-semibold tabular-nums bg-muted text-muted-foreground"
        >
          {full.length}
        </span>
      </div>
      <div className="flex-1 min-h-0 overflow-y-auto flex flex-col gap-2">
        {full.length === 0 && promoted.length === 0 && (
          <p className="px-1.5 pt-2 text-xs text-muted-foreground">{t("discussion.none")}</p>
        )}
        {sortable ? (
          <SortableContext
            items={full.map((d) => cardDndId("discussion", d.slug))}
            strategy={verticalListSortingStrategy}
          >
            {fullCards}
          </SortableContext>
        ) : (
          fullCards
        )}
      </div>
      {promoted.length > 0 && (
        <div className="shrink-0 flex flex-col gap-1.5 border-t border-border/60 pt-1.5">
          <Button
            type="button"
            variant="ghost"
            size="sm"
            aria-expanded={promotedOpen}
            onClick={() => setPromotedOpen((v) => !v)}
            className="h-6 w-full justify-start gap-1 px-1.5 text-[11px] font-semibold text-muted-foreground hover:text-foreground"
          >
            <ArrowUpRight className="h-3 w-3" />
            <span className="tabular-nums">
              {t("discussion.promotedBar").replace("{n}", String(promoted.length))}
            </span>
            {promotedOpen ? (
              <ChevronDown className="h-3 w-3" />
            ) : (
              <ChevronRight className="h-3 w-3" />
            )}
          </Button>
          {promotedOpen && (
            <div className="flex max-h-64 flex-col gap-1.5 overflow-y-auto">
              {promoted.map((d) => (
                <PromotedRow
                  key={d.slug}
                  d={d}
                  changes={changes}
                  archived={archived}
                  onOpenDiscussion={onOpenDiscussion}
                />
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

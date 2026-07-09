import { useState } from "react";
import { SortableContext, useSortable, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { Archive, ChevronDown, ChevronRight, MessageSquareText } from "lucide-react";

import type { ArchivedItem, ChangeItem, DiscussionItem } from "../adapter";
import { cardDndId } from "../boardDnd";
import { useI18n } from "../i18n";
import { changeStage } from "../stage";
import { Button } from "./ui/button";
import { Card, CardContent, CardHeader } from "./ui/card";

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
}

const STATUS_BADGE: Record<string, { labelKey: string; cls: string }> = {
  open: { labelKey: "discussion.statusOpen", cls: "bg-primary/8 text-primary/70" },
  concluded: { labelKey: "discussion.statusConcluded", cls: "bg-primary/12 text-primary" },
};

export function DiscussionCard({
  d,
  onOpenDiscussion,
  onArchiveDiscussion,
}: { d: DiscussionItem } & Pick<
  DiscussionColumnProps,
  "onOpenDiscussion" | "onArchiveDiscussion"
>) {
  const { t } = useI18n();
  const badge = STATUS_BADGE[d.status] ?? STATUS_BADGE.open;
  return (
    <Card
      data-discussion={d.slug}
      className="cursor-pointer transition-[border-color,box-shadow] hover:border-primary/60 hover:shadow-md"
      onClick={() => onOpenDiscussion?.(d.slug)}
    >
      <CardHeader className="p-3 flex-row items-start gap-1.5">
        <span className="font-semibold text-sm leading-tight min-w-0 flex-1">{d.topic}</span>
        <span className={`shrink-0 rounded-full px-1.5 py-0.5 text-[10px] font-semibold ${badge.cls}`}>
          {t(badge.labelKey)}
        </span>
      </CardHeader>
      <CardContent className="p-3 pt-0 gap-2">
        <span className="text-xs text-muted-foreground tabular-nums">
          {t("common.rounds").replace("{n}", String(d.rounds))}
        </span>
        {d.status === "concluded" && (
          <div className="flex gap-1.5" onClick={(e) => e.stopPropagation()}>
            <Button
              variant="outline"
              size="sm"
              className="h-6 gap-1 px-2 text-xs"
              onClick={() => onArchiveDiscussion?.(d.slug)}
            >
              <Archive className="h-3 w-3" /> {t("common.archive")}
            </Button>
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
}: { d: DiscussionItem } & Pick<
  DiscussionColumnProps,
  "onOpenDiscussion" | "onArchiveDiscussion"
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
  // 衍生樹（design D2）：topic 首行為識別錨點（slug 不出現於看板）、
  // 子變更以樹狀前綴逐列列出——父子（討論→衍生變更）關係一眼可讀。
  return (
    <Button
      type="button"
      variant="ghost"
      data-discussion={d.slug}
      className="block h-auto w-full whitespace-normal rounded-md border border-border/60 bg-background/60 px-2 py-1.5 text-left font-normal hover:border-primary/60 hover:bg-background/60"
      onClick={() => onOpenDiscussion?.(d.slug)}
    >
      <span className="block truncate text-xs font-semibold leading-tight">{d.topic}</span>
      <span className="mt-1 flex flex-col gap-0.5">
        {d.promotedTo.map((name, i) => (
          <span key={name} className="flex items-center gap-1 text-[11px] leading-tight">
            <span className="shrink-0 font-mono text-muted-foreground">
              {i === d.promotedTo.length - 1 ? "└" : "├"}
            </span>
            <span className="min-w-0 truncate font-medium">{name}</span>
            <span className="shrink-0 rounded-full bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">
              {t(discussionChipStage(name, changes, archived))}
            </span>
          </span>
        ))}
      </span>
    </Button>
  );
}

/**
 * 看板第 0 欄「討論」（兩級呈現）：open／concluded 為全尺寸卡（open 唯讀、
 * concluded 帶「封存」動詞——轉為變更已自 GUI 撤除）；promoted 收合於欄底
 * 「已轉出變更的討論」群組——每列為 topic＋衍生變更樹（design D2）。
 */
export function DiscussionColumn({
  discussions,
  changes,
  archived,
  onOpenDiscussion,
  onArchiveDiscussion,
  sortable,
}: DiscussionColumnProps) {
  const { t } = useI18n();
  const full = discussions.filter((d) => d.status !== "promoted");
  const promoted = discussions.filter((d) => d.status === "promoted");
  const [showPromoted, setShowPromoted] = useState(true);
  const fullCards = full.map((d) =>
    sortable ? (
      <SortableDiscussionCard
        key={d.slug}
        d={d}
        onOpenDiscussion={onOpenDiscussion}
        onArchiveDiscussion={onArchiveDiscussion}
      />
    ) : (
      <DiscussionCard
        key={d.slug}
        d={d}
        onOpenDiscussion={onOpenDiscussion}
        onArchiveDiscussion={onArchiveDiscussion}
      />
    ),
  );
  return (
    <div
      data-column="discussions"
      className="flex h-full min-h-0 flex-1 min-w-[250px] max-w-[360px] flex-col gap-2 rounded-xl border-t-4 border-t-primary/15 bg-muted/40 p-2"
    >
      <div className="flex items-center gap-1.5 px-1.5 pt-0.5 shrink-0">
        <MessageSquareText className="h-3.5 w-3.5 text-primary/40" />
        <h2 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
          {t("discussion.heading")}
        </h2>
        <div className="flex-1" />
        <span
          data-testid="column-count"
          className="inline-flex items-center justify-center min-w-5 h-5 px-1.5 rounded-full text-[11px] font-semibold tabular-nums bg-primary/8 text-primary/70"
        >
          {discussions.length}
        </span>
      </div>
      <div className="flex-1 min-h-0 overflow-y-auto flex flex-col gap-2">
        {discussions.length === 0 && (
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
        {promoted.length > 0 && (
          <div className="mt-auto flex flex-col gap-1 pt-2">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="h-auto justify-start gap-1 px-1.5 py-0.5 text-[11px] font-semibold text-muted-foreground hover:bg-transparent hover:text-foreground"
              onClick={() => setShowPromoted((v) => !v)}
            >
              {showPromoted ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />}
              {t("discussion.promotedGroup")}
              <span className="tabular-nums">({promoted.length})</span>
            </Button>
            {showPromoted &&
              promoted.map((d) => (
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
    </div>
  );
}

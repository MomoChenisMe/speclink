import { useState } from "react";
import {
  DndContext,
  DragOverlay,
  PointerSensor,
  useSensor,
  useSensors,
  useDroppable,
  type DragEndEvent,
  type DragStartEvent,
} from "@dnd-kit/core";
import { SortableContext, useSortable, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { Archive, CircleCheckBig, Hammer, Lightbulb, type LucideIcon } from "lucide-react";

import type { ArchivedItem, CardKind, ChangeItem, DiscussionLists, SearchHit } from "../adapter";
import {
  archiveZoneVisible,
  cardDndId,
  parseCardDndId,
  resolveCardDrop,
  type ColumnCards,
} from "../boardDnd";
import { useI18n } from "../i18n";
import {
  EMPTY_FILTERS,
  matchesFilters,
  matchesFuzzy,
  matchesQuery,
  type BoardFilters,
} from "../search";
import { changeStage, STAGE_BADGE, STAGE_BAR, STAGE_ICON, STAGES, type Stage } from "../stage";
import { BoardSearchBar } from "./BoardSearchBar";
import { ChangeCard } from "./ChangeCard";
import { DiscussionCard, DiscussionColumn } from "./DiscussionColumn";
import { Button } from "./ui/button";
import { NativeSelect } from "./ui/select";

/** 移動 8px 才視為拖曳——否則 dnd-kit 會吃掉單純點擊，導致卡片無法開啟詳情。 */
export const DRAG_ACTIVATION_DISTANCE = 8;

/** 各階段的視覺主題——單一 teal 色相、以深淺表達生命週期推進，守住主色系。 */
const STAGE_STYLE: Record<Stage, { icon: LucideIcon; top: string; badge: string; bar: string; iconCls: string }> = {
  proposed: {
    icon: Lightbulb,
    top: "border-t-primary/25",
    badge: STAGE_BADGE.proposed,
    bar: STAGE_BAR.proposed,
    iconCls: STAGE_ICON.proposed,
  },
  "in-progress": {
    icon: Hammer,
    top: "border-t-primary/55",
    badge: STAGE_BADGE["in-progress"],
    bar: STAGE_BAR["in-progress"],
    iconCls: STAGE_ICON["in-progress"],
  },
  ready: {
    icon: CircleCheckBig,
    top: "border-t-primary",
    badge: STAGE_BADGE.ready,
    bar: STAGE_BAR.ready,
    iconCls: STAGE_ICON.ready,
  },
};

export interface KanbanBoardProps {
  changes: ChangeItem[];
  onOpenChange?: (name: string) => void;
  /** 封存請求：ready 卡的封存鈕與拖曳落點皆經此觸發（app 端接確認流程）。 */
  onArchive?: (name: string) => void;
  /** 討論清單；提供時看板擴為四欄，active 進第 0 欄（封存討論不上板）。 */
  discussions?: DiscussionLists;
  /** 已封存 change 清單（衍生樹細列的已封存態派生）。 */
  archivedChanges?: ArchivedItem[];
  onOpenDiscussion?: (slug: string) => void;
  /** concluded 討論卡的歸檔動詞（app 端接確認流程）。轉為變更已自 GUI 撤除。 */
  onArchiveDiscussion?: (slug: string) => void;
  /** 看板搜尋字串（選配，與 onQuery 成對提供時渲染搜尋輸入並過濾卡片）。 */
  query?: string;
  onQuery?: (q: string) => void;
  /** 全文搜尋不可用的說明（remote capability 缺口）：提供時搜尋輸入 disabled
   * 並以其文字為 tooltip（title）；缺席＝照常（本地不受影響）。 */
  searchUnavailableReason?: string;
  /** workspace 全文查詢命中（design D6）：命中卡片併入可見集合並呈 snippet 行。 */
  fulltextHits?: SearchHit[];
  /**
   * 欄內拖排寫回（design D5/D6）：同欄放開時以 arrayMove 後的前後鄰居回報
   * （null＝欄頂／欄底）；未提供時卡片不掛 sortable（封存拖放照舊）。
   */
  onReorder?: (kind: CardKind, id: string, prevId: string | null, nextId: string | null) => void;
  /** 拖排不可用時的使用者可見說明；文字由宿主依 capability 與語系提供。 */
  reorderUnavailableReason?: string;
  /** 拖曳手勢期間（按住～放開）回報 true——宿主據此讓外部刷新讓路（任務列同款）。 */
  onDragActiveChange?: (active: boolean) => void;
}

function Column({
  stage,
  count,
  children,
}: {
  stage: Stage;
  count: number;
  children: React.ReactNode;
}) {
  const { t } = useI18n();
  const { setNodeRef, isOver } = useDroppable({ id: stage });
  const style = STAGE_STYLE[stage];
  const Icon = style.icon;
  return (
    <div
      ref={setNodeRef}
      data-column={stage}
      className={`flex h-full min-h-0 flex-1 min-w-[250px] max-w-[360px] flex-col gap-2 rounded-xl border-t-4 ${style.top} p-2 transition-colors ${
        isOver ? "bg-accent/60 ring-2 ring-primary/50" : "bg-muted/40"
      }`}
    >
      <div className="flex items-center gap-1.5 px-1.5 pt-0.5 shrink-0">
        <Icon className={`h-3.5 w-3.5 ${style.iconCls}`} />
        <h2 className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
          {t(`stage.${stage}`)}
        </h2>
        <div className="flex-1" />
        <span
          data-testid="column-count"
          className={`inline-flex items-center justify-center min-w-5 h-5 px-1.5 rounded-full text-[11px] font-semibold tabular-nums ${style.badge}`}
        >
          {count}
        </span>
      </div>
      <div className="flex-1 min-h-0 overflow-y-auto flex flex-col gap-2">{children}</div>
    </div>
  );
}

/**
 * 拖曳變更卡時才浮現的封存落點（design D8）：絕對定位浮層疊於看板右緣上方、
 * 不參與欄列 flex 佈局——浮現與消失時欄寬零變動。半透明底＋backdrop 讓其
 * 下方的欄內容仍可辨識。
 */
function ArchiveDropZone() {
  const { t } = useI18n();
  const { setNodeRef, isOver } = useDroppable({ id: "archived" });
  return (
    <div
      ref={setNodeRef}
      data-column="archived"
      className={`absolute inset-y-2 right-2 z-10 flex w-[120px] flex-col items-center justify-center gap-2 rounded-xl border-2 border-dashed backdrop-blur-sm transition-colors ${
        isOver
          ? "border-primary bg-accent/80 text-primary"
          : "border-border bg-background/80 text-muted-foreground"
      }`}
    >
      <Archive className="h-5 w-5" />
      <span className="text-xs font-medium">{t("kanban.dropToArchive")}</span>
    </div>
  );
}

function SortableCard({
  change,
  barClass,
  highlight,
  hit,
  ...rest
}: { change: ChangeItem; barClass: string; highlight?: string; hit?: SearchHit } & Pick<
  KanbanBoardProps,
  "onOpenChange" | "onArchive"
>) {
  const { t } = useI18n();
  // 拖曳時原卡片留在原位變淡；移動的視覺由 DragOverlay 呈現（不受欄位 overflow 裁切）。
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: cardDndId("change", change.name),
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
      aria-label={t("kanban.dragCard").replace("{name}", change.name)}
    >
      <ChangeCard
        change={change}
        barClass={barClass}
        highlight={highlight}
        hit={hit}
        onOpen={rest.onOpenChange}
        onArchive={rest.onArchive}
      />
    </div>
  );
}

/**
 * 生命週期看板：討論（第 0 欄，傳入 discussions 時）＋提案中／進行中／已就緒
 * 三欄（彩色主題）；欄內拖排（onReorder 提供時）、拖曳時浮現封存落點。
 */
export function KanbanBoard({
  changes,
  onOpenChange,
  onArchive,
  discussions,
  archivedChanges,
  onOpenDiscussion,
  onArchiveDiscussion,
  query,
  onQuery,
  fulltextHits,
  searchUnavailableReason,
  onReorder,
  reorderUnavailableReason,
  onDragActiveChange,
}: KanbanBoardProps) {
  const [activeId, setActiveId] = useState<string | null>(null);
  const { t } = useI18n();
  // 搜尋過濾（spec「看板搜尋過濾卡片」）：變更卡以名稱與摘要、討論卡以主題與
  // slug 比對；比對規則共用 matchesQuery（與已封存頁一致）。空（或僅空白）即全量。
  // 篩選 chips（design D5）：元件 local、不持久化；與搜尋字串 AND 交集。
  const showSearch = query !== undefined && onQuery !== undefined;
  const [filters, setFilters] = useState<BoardFilters>(EMPTY_FILTERS);
  // 篩選 chips 的展開狀態（design D5）：預設收合、不持久化；啟用中維度數供開關鈕徽章。
  const [filtersOpen, setFiltersOpen] = useState(false);
  const activeFilterCount = [filters.createdBy, filters.createdWithin, filters.fromDiscussion].filter(
    (v) => v !== null,
  ).length;
  const today = new Date().toISOString().slice(0, 10);
  // 三層比對任一命中即顯示（spec）：欄位子字串、名稱層 fuzzy（僅名稱／slug，
  // design D7）、全文命中集合（design D6）——再與篩選 chips 取 AND。
  const q = query ?? "";
  const hitByCard = new Map((fulltextHits ?? []).map((h) => [`${h.kind}:${h.id}`, h]));
  const visibleChanges = changes.filter(
    (c) =>
      (matchesQuery(q, c.name, c.summary) ||
        matchesFuzzy(q, c.name) ||
        hitByCard.has(`change:${c.name}`)) &&
      matchesFilters(filters, c, today),
  );
  const visibleDiscussions = discussions?.active.filter(
    (d) =>
      (matchesQuery(q, d.topic, d.slug) ||
        matchesFuzzy(q, d.slug) ||
        hitByCard.has(`discussion:${d.slug}`)) &&
      matchesFilters(filters, d, today),
  );
  // chip 選項自現有清單派生：建立者去重、來源討論取 promoted_to 非空者。
  const creators = Array.from(
    new Set(
      [...changes.map((c) => c.createdBy), ...(discussions?.active ?? []).map((d) => d.createdBy)].filter(
        (x): x is string => !!x,
      ),
    ),
  );
  const sourceDiscussions = (discussions?.active ?? []).filter((d) => d.promotedTo.length > 0);
  const byStage: Record<Stage, ChangeItem[]> = { proposed: [], "in-progress": [], ready: [] };
  for (const c of visibleChanges) byStage[changeStage(c)].push(c);
  const dragging = activeId !== null;
  const activeCard = activeId ? parseCardDndId(activeId) : null;
  const activeChange =
    activeCard?.kind === "change" ? changes.find((c) => c.name === activeCard.id) ?? null : null;
  const activeDiscussion =
    activeCard?.kind === "discussion"
      ? discussions?.active.find((d) => d.slug === activeCard.id) ?? null
      : null;

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: DRAG_ACTIVATION_DISTANCE } }),
  );

  // 落點解析輸入：每欄「可見」卡的識別碼（視覺序）——搜尋過濾中拖排沿同一語意
  //（spec：新鍵介於可見前後鄰居之間）。討論欄只有全卡參與（promoted 收合列不拖）。
  const columns: ColumnCards[] = [
    ...(visibleDiscussions
      ? [{ kind: "discussion" as const, ids: visibleDiscussions.filter((d) => d.status !== "promoted").map((d) => d.slug) }]
      : []),
    ...STAGES.map((stage) => ({ kind: "change" as const, ids: byStage[stage].map((c) => c.name) })),
  ];

  const handleDragStart = (e: DragStartEvent) => {
    setActiveId(String(e.active.id));
    onDragActiveChange?.(true);
  };

  const handleDragEnd = (e: DragEndEvent) => {
    setActiveId(null);
    // 放開即結束手勢；同一事件內 onReorder 使宿主寫回＋refresh，讓路無縫接手。
    onDragActiveChange?.(false);
    if (!e.over) return;
    const over = String(e.over.id);
    const active = String(e.active.id);
    if (over === "archived") {
      const card = parseCardDndId(active);
      if (card?.kind === "change") onArchive?.(card.id);
      return;
    }
    // 同欄放開才寫回；跨欄、欄容器、原位一律 null → 彈回、零寫入。
    const drop = resolveCardDrop(columns, active, over);
    if (drop) onReorder?.(drop.kind, drop.id, drop.prevId, drop.nextId);
  };

  return (
    <DndContext
      sensors={sensors}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      onDragCancel={() => {
        setActiveId(null);
        onDragActiveChange?.(false);
      }}
    >
      <div className="flex h-full min-h-0 flex-col gap-3">
      {reorderUnavailableReason && (
        <div
          role="note"
          className="rounded-md border border-border bg-muted/40 px-3 py-2 text-xs text-muted-foreground"
        >
          {reorderUnavailableReason}
        </div>
      )}
      {showSearch && (
        <BoardSearchBar
          query={query}
          onQuery={onQuery}
          disabledReason={searchUnavailableReason}
          hitCount={visibleChanges.length + (visibleDiscussions?.length ?? 0)}
          filtersOpen={filtersOpen}
          onToggleFilters={() => setFiltersOpen((v) => !v)}
          onCloseFilters={() => setFiltersOpen(false)}
          activeFilterCount={activeFilterCount}
        >
          {/* 篩選面板內容（design D5）：三維度選單直欄堆疊，選回「全部」即單獨清除。 */}
          <div className="flex flex-col gap-1">
            <span className="text-xs font-medium text-muted-foreground">{t("filter.createdBy")}</span>
            <NativeSelect
              aria-label={t("filter.createdBy")}
              value={filters.createdBy ?? ""}
              onChange={(e) => setFilters({ ...filters, createdBy: e.target.value || null })}
            >
              <option value="">{t("filter.all")}</option>
              {creators.map((c) => (
                <option key={c} value={c}>
                  {c}
                </option>
              ))}
            </NativeSelect>
          </div>
          <div className="flex flex-col gap-1">
            <span className="text-xs font-medium text-muted-foreground">{t("filter.createdWithin")}</span>
            <NativeSelect
              aria-label={t("filter.createdWithin")}
              value={filters.createdWithin ?? ""}
              onChange={(e) =>
                setFilters({
                  ...filters,
                  createdWithin: (e.target.value || null) as BoardFilters["createdWithin"],
                })
              }
            >
              <option value="">{t("filter.all")}</option>
              <option value="7d">{t("filter.range7d")}</option>
              <option value="30d">{t("filter.range30d")}</option>
              <option value="earlier">{t("filter.rangeEarlier")}</option>
            </NativeSelect>
          </div>
          <div className="flex flex-col gap-1">
            <span className="text-xs font-medium text-muted-foreground">{t("filter.fromDiscussion")}</span>
            <NativeSelect
              aria-label={t("filter.fromDiscussion")}
              value={filters.fromDiscussion ?? ""}
              onChange={(e) => setFilters({ ...filters, fromDiscussion: e.target.value || null })}
            >
              <option value="">{t("filter.all")}</option>
              {sourceDiscussions.map((d) => (
                <option key={d.slug} value={d.slug}>
                  {d.slug}
                </option>
              ))}
            </NativeSelect>
          </div>
          {activeFilterCount > 0 && (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="h-7 justify-center text-xs text-muted-foreground hover:text-foreground"
              onClick={() => setFilters(EMPTY_FILTERS)}
            >
              {t("filter.clearAll")}
            </Button>
          )}
        </BoardSearchBar>
      )}
      {/* relative wrapper 供封存落點浮層錨定於「可視」右緣（design D8）——浮層
          在捲動容器之外、不進 flex 流，欄寬零變動且不隨水平捲動漂移。 */}
      <div className="relative flex-1 min-h-0">
      {/* safe center：寬螢幕置中、內容溢出時回到可捲動的靠左 */}
      <div className="flex h-full min-h-0 gap-3 overflow-x-auto [justify-content:safe_center]">
        {discussions && (
          <DiscussionColumn
            discussions={visibleDiscussions ?? []}
            changes={changes}
            archived={archivedChanges ?? []}
            onOpenDiscussion={onOpenDiscussion}
            onArchiveDiscussion={onArchiveDiscussion}
            sortable={!!onReorder}
            highlight={q}
            fulltextHits={fulltextHits}
          />
        )}
        {STAGES.map((stage) => (
          <Column key={stage} stage={stage} count={byStage[stage].length}>
            {onReorder ? (
              <SortableContext
                items={byStage[stage].map((c) => cardDndId("change", c.name))}
                strategy={verticalListSortingStrategy}
              >
                {byStage[stage].map((c) => (
                  <SortableCard
                    key={c.name}
                    change={c}
                    barClass={STAGE_STYLE[stage].bar}
                    highlight={q}
                    hit={hitByCard.get(`change:${c.name}`)}
                    onOpenChange={onOpenChange}
                    onArchive={onArchive}
                  />
                ))}
              </SortableContext>
            ) : (
              byStage[stage].map((c) => (
                <ChangeCard
                  key={c.name}
                  change={c}
                  barClass={STAGE_STYLE[stage].bar}
                  highlight={q}
                  hit={hitByCard.get(`change:${c.name}`)}
                  onOpen={onOpenChange}
                  onArchive={onArchive}
                />
              ))
            )}
          </Column>
        ))}
      </div>
      {/* 僅拖曳變更卡時浮現（archiveZoneVisible）：討論卡不可封存、不得造成佈局變動。 */}
      {dragging && archiveZoneVisible(activeId) && <ArchiveDropZone />}
      </div>
      </div>
      {/* 拖曳浮動複本：渲染在最上層，不受欄位 overflow 裁切 */}
      <DragOverlay dropAnimation={null}>
        {activeChange ? (
          <div className="shadow-lg rounded-lg rotate-2 cursor-grabbing">
            <ChangeCard change={activeChange} barClass={STAGE_STYLE[changeStage(activeChange)].bar} />
          </div>
        ) : activeDiscussion ? (
          <div className="shadow-lg rounded-lg rotate-2 cursor-grabbing">
            <DiscussionCard d={activeDiscussion} />
          </div>
        ) : null}
      </DragOverlay>
    </DndContext>
  );
}

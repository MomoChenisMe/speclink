import { useState } from "react";
import {
  DndContext,
  DragOverlay,
  PointerSensor,
  useSensor,
  useSensors,
  useDraggable,
  useDroppable,
  type DragEndEvent,
  type DragStartEvent,
} from "@dnd-kit/core";
import { Archive, CircleCheckBig, Hammer, Lightbulb, type LucideIcon } from "lucide-react";

import type { ArchivedItem, ChangeItem, DiscussionLists } from "../adapter";
import { changeStage, STAGES, STAGE_LABEL, type Stage } from "../stage";
import { ChangeCard } from "./ChangeCard";
import { DiscussionColumn } from "./DiscussionColumn";

/** 各階段的視覺主題——單一 teal 色相、以深淺表達生命週期推進，守住主色系。 */
const STAGE_STYLE: Record<Stage, { icon: LucideIcon; top: string; badge: string; bar: string; iconCls: string }> = {
  proposed: {
    icon: Lightbulb,
    top: "border-t-primary/25",
    badge: "bg-primary/8 text-primary/70",
    bar: "bg-primary/50",
    iconCls: "text-primary/50",
  },
  "in-progress": {
    icon: Hammer,
    top: "border-t-primary/55",
    badge: "bg-primary/12 text-primary",
    bar: "bg-primary/75",
    iconCls: "text-primary/75",
  },
  ready: {
    icon: CircleCheckBig,
    top: "border-t-primary",
    badge: "bg-primary text-primary-foreground",
    bar: "bg-primary",
    iconCls: "text-primary",
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
  /** concluded 討論卡的轉為變更動詞（app 端接確認流程）。 */
  onPromoteDiscussion?: (slug: string) => void;
  /** concluded 討論卡的歸檔動詞（app 端接確認流程）。 */
  onArchiveDiscussion?: (slug: string) => void;
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
          {STAGE_LABEL[stage]}
        </h2>
        <div className="flex-1" />
        <span className={`inline-flex items-center justify-center min-w-5 h-5 px-1.5 rounded-full text-[11px] font-semibold tabular-nums ${style.badge}`}>
          {count}
        </span>
      </div>
      <div className="flex-1 min-h-0 overflow-y-auto flex flex-col gap-2">{children}</div>
    </div>
  );
}

/** 拖曳中才浮現的封存落點。 */
function ArchiveDropZone() {
  const { setNodeRef, isOver } = useDroppable({ id: "archived" });
  return (
    <div
      ref={setNodeRef}
      data-column="archived"
      className={`flex h-full w-[140px] shrink-0 flex-col items-center justify-center gap-2 rounded-xl border-2 border-dashed transition-colors ${
        isOver ? "border-primary bg-accent/60 text-primary" : "border-border text-muted-foreground"
      }`}
    >
      <Archive className="h-5 w-5" />
      <span className="text-xs font-medium">拖到此封存</span>
    </div>
  );
}

function DraggableCard({ change, barClass, ...rest }: { change: ChangeItem; barClass: string } & Pick<KanbanBoardProps, "onOpenChange" | "onArchive">) {
  // 拖曳時原卡片留在原位變淡；移動的視覺由 DragOverlay 呈現（不受欄位 overflow 裁切）。
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({ id: change.name });
  return (
    <div ref={setNodeRef} style={isDragging ? { opacity: 0.35 } : undefined} {...attributes} {...listeners}>
      <ChangeCard change={change} barClass={barClass} onOpen={rest.onOpenChange} onArchive={rest.onArchive} />
    </div>
  );
}

/**
 * 生命週期看板：討論（第 0 欄，傳入 discussions 時）＋提案中／進行中／已就緒
 * 三欄（彩色主題）；拖曳時浮現封存落點。
 */
export function KanbanBoard({
  changes,
  onOpenChange,
  onArchive,
  discussions,
  archivedChanges,
  onOpenDiscussion,
  onPromoteDiscussion,
  onArchiveDiscussion,
}: KanbanBoardProps) {
  const [activeName, setActiveName] = useState<string | null>(null);
  const byStage: Record<Stage, ChangeItem[]> = { proposed: [], "in-progress": [], ready: [] };
  for (const c of changes) byStage[changeStage(c)].push(c);
  const dragging = activeName !== null;
  const activeChange = activeName ? changes.find((c) => c.name === activeName) ?? null : null;

  // 移動 8px 才視為拖曳——否則 dnd-kit 會吃掉單純點擊，導致卡片無法開啟詳情。
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
  );

  const handleDragStart = (e: DragStartEvent) => setActiveName(String(e.active.id));

  const handleDragEnd = (e: DragEndEvent) => {
    setActiveName(null);
    if (e.over?.id === "archived" && e.active) onArchive?.(String(e.active.id));
  };

  return (
    <DndContext sensors={sensors} onDragStart={handleDragStart} onDragEnd={handleDragEnd} onDragCancel={() => setActiveName(null)}>
      {/* safe center：寬螢幕置中、內容溢出時回到可捲動的靠左 */}
      <div className="flex gap-3 h-full min-h-0 overflow-x-auto [justify-content:safe_center]">
        {discussions && (
          <DiscussionColumn
            discussions={discussions.active}
            changes={changes}
            archived={archivedChanges ?? []}
            onOpenDiscussion={onOpenDiscussion}
            onPromote={onPromoteDiscussion}
            onArchiveDiscussion={onArchiveDiscussion}
          />
        )}
        {STAGES.map((stage) => (
          <Column key={stage} stage={stage} count={byStage[stage].length}>
            {byStage[stage].map((c) => (
              <DraggableCard
                key={c.name}
                change={c}
                barClass={STAGE_STYLE[stage].bar}
                onOpenChange={onOpenChange}
                onArchive={onArchive}
              />
            ))}
          </Column>
        ))}
        {dragging && <ArchiveDropZone />}
      </div>
      {/* 拖曳浮動複本：渲染在最上層，不受欄位 overflow 裁切 */}
      <DragOverlay dropAnimation={null}>
        {activeChange ? (
          <div className="shadow-lg rounded-lg rotate-2 cursor-grabbing">
            <ChangeCard change={activeChange} barClass={STAGE_STYLE[changeStage(activeChange)].bar} />
          </div>
        ) : null}
      </DragOverlay>
    </DndContext>
  );
}

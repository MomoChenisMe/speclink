import { useEffect, useRef, useState } from "react";
import {
  DndContext,
  DragOverlay,
  KeyboardSensor,
  PointerSensor,
  closestCenter,
  useSensor,
  useSensors,
  type DragEndEvent,
  type DragStartEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { CheckCheck, GripVertical, LocateFixed, RotateCcw } from "lucide-react";

import { useI18n } from "../i18n";
import { parseTaskDoc, resolveDropTarget, type TaskDocItem } from "../tasks";

export interface TaskListProps {
  markdown: string | null;
  /** 勾選/取消第 ordinal 個任務（1-based）。 */
  onToggle?: (ordinal: number, done: boolean) => void;
  /**
   * 拖放落點：把第 from 個任務移到以第 to 個任務為錨的位置（皆 1-based、一次
   * 到位）。before=true＝插錨前（群組標題落點解析為組首）；省略＝方向推斷。
   */
  onReorder?: (from: number, to: number, before?: boolean) => void;
  /** 寫入進行中時鎖定互動。 */
  busy?: boolean;
  /** 拖曳手勢期間（按住～放開）回報 true——宿主據此讓外部內容重載讓路。 */
  onDragActiveChange?: (active: boolean) => void;
  /** 唯讀呈現（封存檢視）：核取方塊 disabled、不渲染拖曳把手與工具列。 */
  readOnly?: boolean;
  /** 批次設定全部任務完成狀態（true＝全部已完成、false＝重置任務）。 */
  onSetAll?: (done: boolean) => void;
}

type TaskItem = Extract<TaskDocItem, { kind: "task" }>;

/** 任務列內容（checkbox＋文字）——一般列與 DragOverlay 共用。 */
function TaskRowBody({
  item,
  readOnly,
  onToggle,
}: {
  item: TaskItem;
  readOnly?: boolean;
  onToggle?: (ordinal: number, done: boolean) => void;
}) {
  const { t } = useI18n();
  return (
    <>
      <input
        type="checkbox"
        aria-label={t("tasks.checkbox").replace("{n}", String(item.ordinal))}
        className="mt-1 shrink-0 accent-[var(--primary)]"
        checked={item.done}
        disabled={readOnly}
        onChange={(e) => onToggle?.(item.ordinal, e.target.checked)}
      />
      <span
        className={`flex-1 text-base leading-relaxed ${
          item.done ? "text-muted-foreground line-through decoration-muted-foreground/50" : ""
        }`}
      >
        {item.text}
      </span>
    </>
  );
}

/** 群組標題：不可拖，但以 disabled sortable 項參與讓位序列（design D6）——
 * 拖曳中任務與標題保持相對順序、讓位視覺不穿越群組邊界；標題本身可為落點（組首）。 */
function SortableGroupHeading({ id, text }: { id: string; text: string }) {
  const { setNodeRef, transform, transition } = useSortable({ id, disabled: true });
  return (
    <h4
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition }}
      className="text-base font-bold mt-4 mb-1.5 first:mt-0"
    >
      {text}
    </h4>
  );
}

/** 可拖曳任務列：拖曳監聽只綁 ⠿ 把手——點擊核取方塊與文字不經過拖曳。 */
function SortableTaskRow({
  item,
  onToggle,
  highlight,
  rowRef,
}: {
  item: TaskItem;
  onToggle?: (ordinal: number, done: boolean) => void;
  /** 「下一個未完成」的短暫高亮標記。 */
  highlight?: boolean;
  /** 列元素回報（定位捲動用）。 */
  rowRef?: (el: HTMLDivElement | null) => void;
}) {
  const { t } = useI18n();
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: item.ordinal,
  });
  return (
    <div
      ref={(el) => {
        setNodeRef(el);
        rowRef?.(el);
      }}
      style={{ transform: CSS.Transform.toString(transform), transition }}
      data-highlight={highlight ? "true" : "false"}
      className={`group/task flex items-start gap-2 py-1 pl-1 rounded-md transition-colors hover:bg-muted/50 ${
        isDragging ? "opacity-40" : ""
      } ${highlight ? "bg-accent ring-1 ring-primary/40" : ""}`}
    >
      <button
        type="button"
        aria-label={t("tasks.drag").replace("{n}", String(item.ordinal))}
        className="mt-1 shrink-0 cursor-grab touch-none text-muted-foreground/50 hover:text-foreground"
        {...attributes}
        {...listeners}
      >
        <GripVertical className="h-3.5 w-3.5" />
      </button>
      <TaskRowBody item={item} onToggle={onToggle} />
    </div>
  );
}

/** 互動任務清單：頂部批次工具列＋群組標題＋可勾選 checkbox＋⠿ 把手拖放排序，
 * 與 tasks.md 聯動。 */
export function TaskList({ markdown, onToggle, onReorder, busy, onDragActiveChange, readOnly, onSetAll }: TaskListProps) {
  const { t } = useI18n();
  const items = parseTaskDoc(markdown);
  const taskItems = items.filter((i): i is TaskItem => i.kind === "task");
  const [activeOrdinal, setActiveOrdinal] = useState<number | null>(null);
  // 「下一個未完成」的短暫高亮（ordinal）；列元素表供定位捲動。
  const [highlightOrdinal, setHighlightOrdinal] = useState<number | null>(null);
  const highlightTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const rowRefs = useRef(new Map<number, HTMLDivElement>());
  // PointerSensor distance 8：位移門檻內的按放是點擊，不啟動拖曳（看板同款教訓）。
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  const allDone = taskItems.length > 0 && taskItems.every((i) => i.done);

  const locateNext = () => {
    const next = taskItems.find((i) => !i.done);
    if (!next) return;
    rowRefs.current.get(next.ordinal)?.scrollIntoView({ block: "center", behavior: "smooth" });
    setHighlightOrdinal(next.ordinal);
    if (highlightTimer.current) clearTimeout(highlightTimer.current);
    highlightTimer.current = setTimeout(() => setHighlightOrdinal(null), 1600);
  };

  // n 快捷鍵（任務分頁掛載中即作用；分頁未啟用時本元件未掛載，自然不搶鍵）。
  // 輸入元件內打字不觸發；無依賴陣列＝每次渲染重掛，回呼永遠讀到最新解析結果。
  useEffect(() => {
    if (readOnly) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "n" || e.ctrlKey || e.metaKey || e.altKey) return;
      const el = e.target as HTMLElement | null;
      if (el && (el.tagName === "INPUT" || el.tagName === "TEXTAREA" || el.isContentEditable)) return;
      locateNext();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  // 高亮計時器卸載清理。
  useEffect(
    () => () => {
      if (highlightTimer.current) clearTimeout(highlightTimer.current);
    },
    [],
  );

  if (items.length === 0) {
    return <div className="text-muted-foreground text-sm py-6">{t("tasks.empty")}</div>;
  }

  const rows = items.map((item, i) =>
    item.kind === "group" ? (
      readOnly ? (
        <h4 key={`g-${i}`} className="text-base font-bold mt-4 mb-1.5 first:mt-0">
          {item.text}
        </h4>
      ) : (
        <SortableGroupHeading key={`g-${i}`} id={`g-${i}`} text={item.text} />
      )
    ) : readOnly ? (
      <div
        key={`t-${item.ordinal}`}
        className="group/task flex items-start gap-2 py-1 pl-1 rounded-md hover:bg-muted/50"
      >
        <TaskRowBody item={item} readOnly onToggle={onToggle} />
      </div>
    ) : (
      <SortableTaskRow
        key={`t-${item.ordinal}`}
        item={item}
        onToggle={onToggle}
        highlight={highlightOrdinal === item.ordinal}
        rowRef={(el) => {
          if (el) rowRefs.current.set(item.ordinal, el);
          else rowRefs.current.delete(item.ordinal);
        }}
      />
    ),
  );

  // 唯讀（封存檢視）：無工具列、無把手、無 DndContext。
  if (readOnly) {
    return <div className="flex flex-col">{rows}</div>;
  }

  const toolbarBtn =
    "flex items-center gap-1.5 rounded px-2 py-1 text-sm text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:pointer-events-none disabled:opacity-40";
  const active = activeOrdinal != null ? taskItems.find((t) => t.ordinal === activeOrdinal) : null;
  // 讓位序列＝視覺順序（標題與任務交錯）——標題入列使讓位位移對齊群組邊界。
  const sortableIds = items.map((item, i) => (item.kind === "group" ? `g-${i}` : item.ordinal));

  const handleDragStart = (e: DragStartEvent) => {
    setActiveOrdinal(Number(e.active.id));
    onDragActiveChange?.(true);
  };
  const handleDragEnd = (e: DragEndEvent) => {
    setActiveOrdinal(null);
    // 放開即結束手勢；同一事件內 onReorder 會使宿主進入寫回 busy，讓路無縫接手。
    onDragActiveChange?.(false);
    const { active: a, over } = e;
    if (!over) return;
    const target = resolveDropTarget(items, Number(a.id), over.id as number | string);
    if (!target) return;
    onReorder?.(Number(a.id), target.to, target.before);
  };

  return (
    <>
      {/* 批次操作工具列（spec「任務分頁提供批次操作工具列」）：全部已完成／下一個
          未完成（n）／重置任務；全完成時前兩鍵不可用，批次寫回期間整列 disabled。 */}
      <div className="mb-2 flex items-center gap-1 rounded-md border border-border px-2 py-1.5">
        <button
          type="button"
          className={toolbarBtn}
          disabled={allDone || busy}
          onClick={() => onSetAll?.(true)}
        >
          <CheckCheck className="h-3.5 w-3.5" /> {t("tasks.completeAll")}
        </button>
        <button
          type="button"
          className={toolbarBtn}
          disabled={allDone || busy}
          onClick={locateNext}
        >
          <LocateFixed className="h-3.5 w-3.5" /> {t("tasks.nextUndone")}
          <kbd className="rounded border border-border bg-muted px-1 text-[10px] text-muted-foreground">
            n
          </kbd>
        </button>
        <div className="flex-1" />
        <button
          type="button"
          className={toolbarBtn}
          disabled={busy}
          onClick={() => onSetAll?.(false)}
        >
          <RotateCcw className="h-3.5 w-3.5" /> {t("tasks.resetAll")}
        </button>
      </div>
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      onDragCancel={() => {
        setActiveOrdinal(null);
        onDragActiveChange?.(false);
      }}
    >
      <SortableContext items={sortableIds} strategy={verticalListSortingStrategy}>
        <div className={`flex flex-col ${busy ? "opacity-60 pointer-events-none" : ""}`}>{rows}</div>
      </SortableContext>
      {/* DragOverlay：浮起列逃出抽屜捲動容器的 overflow 裁切（看板同款）。 */}
      <DragOverlay>
        {active ? (
          <div className="flex items-start gap-2 py-1 pl-1 rounded-md border border-border bg-card shadow-lg">
            <span className="mt-1 shrink-0 text-muted-foreground/50">
              <GripVertical className="h-3.5 w-3.5" />
            </span>
            <TaskRowBody item={active} />
          </div>
        ) : null}
      </DragOverlay>
    </DndContext>
    </>
  );
}

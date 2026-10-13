import { useEffect, useId, useRef, useState, type CSSProperties, type MouseEvent, type Ref } from "react";
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
import { CheckCheck, ChevronDown, ChevronRight, GripVertical, Hand, LocateFixed, RotateCcw } from "lucide-react";

import { splitTaskText } from "../docModel";
import { useI18n } from "../i18n";
import { SEMANTIC_SURFACE, SEMANTIC_TONE } from "../tone";
import { InlineMarks, NAME_CHIP_CLS, SECTION_HEADING_CLS } from "./DocParts";
import { Markdown } from "./Markdown";
import { parseTaskDoc, resolveDropTarget, taskKey, type TaskDocItem } from "../tasks";
import { Button } from "./ui/button";
import { Card } from "./ui/card";
import { Checkbox } from "./ui/checkbox";

export interface TaskListProps {
  markdown: string | null;
  /** 勾選/取消第 ordinal 個任務（1-based）；帶 ID 任務同時回報 stableId
   * （tsk_ 前綴），勾選請求以它定址、無 ID 舊檔走 ordinal 相容路徑。 */
  onToggle?: (ordinal: number, done: boolean, stableId?: string) => void;
  /**
   * 拖放落點：把第 from 個任務移到以第 to 個任務為錨的位置（皆 1-based、一次
   * 到位）。before=true＝插錨前（群組標題落點解析為組首）；省略＝方向推斷。
   * 未提供＝拖排停用（remote capability 缺口）：不渲染把手、不掛 DndContext，
   * 勾選與批次工具列照常。
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
  /** 點任務標題後的 D 籤（design D5）：宿主切到設計文件並展開該決策卡；缺席時 D 籤不可點。 */
  onJumpDecision?: (id: string) => void;
  /** 定位請求（頁的手動提示框「前往」；design D14）：捲到第 ordinal 個任務並短暫高亮，唯讀時也可；
   * seq 遞增讓同一列可重複定位。 */
  locate?: { ordinal: number; seq: number } | null;
  /** 定位處理完——頁據此清掉請求。 */
  onLocated?: () => void;
}

type TaskItem = Extract<TaskDocItem, { kind: "task" }>;

/** 群組標題開頭的「N.」（任務描述的「X.Y」序號由 splitTaskText 切）。 */
const GROUP_ORDINAL_RE = /^(\d+)\.\s*([\s\S]*)$/;

/** 序號籤：任務列與群組卡頭共用。 */
const ORDINAL_CHIP_CLS = "mt-0.5 shrink-0 rounded bg-muted px-1.5 py-0.5 font-mono text-xs tabular-nums text-muted-foreground";

/** 列內的按鈕在 label 裡：點擊不得冒到列上觸發勾選。 */
function inRow(action: () => void) {
  return (e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    action();
  };
}

/**
 * 任務列內容（spec「任務群組標題與章節標籤同款式」；design D5）——一般列與 DragOverlay 共用。
 * 描述拆「標題／內文／驗證」三層：預設只有勾選框、序號籤、標題、D 籤、手動籤與列尾展開鈕；
 * 展開後在標題下方畫內文與驗證。`titleOnly`（拖曳浮起列）只畫標題層。
 */
function TaskRowBody({
  item,
  checkboxId,
  disabled,
  onToggle,
  onJumpDecision,
  titleOnly,
}: {
  item: TaskItem;
  checkboxId?: string;
  disabled?: boolean;
  onToggle?: (ordinal: number, done: boolean, stableId?: string) => void;
  onJumpDecision?: (id: string) => void;
  titleOnly?: boolean;
}) {
  const { t } = useI18n();
  const [open, setOpen] = useState(false);
  const parts = splitTaskText(item.text);
  const detail = [parts.note, parts.body].filter(Boolean).join("\n\n");
  const expandable = !titleOnly && (detail !== "" || parts.verify !== null);
  const n = String(item.ordinal);
  return (
    <>
      <Checkbox
        id={checkboxId}
        aria-label={t("tasks.checkbox").replace("{n}", n)}
        className="mt-1"
        checked={item.done}
        disabled={disabled}
        onCheckedChange={(v) => onToggle?.(item.ordinal, v === true, item.stableId)}
      />
      {/* 描述欄：標題那一行＋展開層。手動任務的徽章不進這一欄的文字流（spec「任務列的
          手動任務徽章」；design D14）——它在標題列尾，序號與標題的起始欄與無標記列同位。 */}
      <span className="flex min-w-0 flex-1 flex-col gap-1">
        <span data-task-title-row className="flex min-w-0 items-start gap-2">
          {parts.number && <span className={ORDINAL_CHIP_CLS}>{parts.number}</span>}
          <span
            data-task-title
            className={`min-w-0 flex-1 text-[15px] leading-relaxed ${
              item.done ? "text-muted-foreground line-through decoration-muted-foreground/50" : ""
            }`}
          >
            <InlineMarks text={parts.title} />
          </span>
          {parts.decisions.map((id) =>
            onJumpDecision ? (
              <button
                key={id}
                type="button"
                data-task-decision={id}
                className={`${NAME_CHIP_CLS} mt-0.5 text-primary hover:bg-accent hover:underline`}
                onClick={inRow(() => onJumpDecision(id))}
              >
                {id}
              </button>
            ) : (
              <span key={id} data-task-decision={id} className={`${NAME_CHIP_CLS} mt-0.5`}>
                {id}
              </span>
            ),
          )}
          {item.manual && (
            <span
              aria-label={t("tasks.manual")}
              // 手動語意色（暖黃，面色＋文字色），與看板「待手動」章、手動任務提示框同一個 token。
              className={`mt-0.5 inline-flex shrink-0 items-center gap-1 rounded-full border px-1.5 py-0.5 text-[11px] leading-none ${SEMANTIC_SURFACE.manual} ${SEMANTIC_TONE.manual}`}
            >
              <Hand className="h-3 w-3" />
              {t("tasks.manual")}
            </span>
          )}
          {expandable && (
            <Button
              type="button"
              variant="subtle"
              size="icon-xs"
              data-task-expand
              aria-expanded={open}
              aria-label={t(open ? "tasks.collapse" : "tasks.expand").replace("{n}", n)}
              className="mt-0.5 h-5 w-5 shrink-0"
              onClick={inRow(() => setOpen((v) => !v))}
            >
              {open ? <ChevronDown className="h-3.5 w-3.5" /> : <ChevronRight className="h-3.5 w-3.5" />}
            </Button>
          )}
        </span>
        {expandable && open && (
          // 展開層仍在列的 label 裡：點內文不勾選（連結照常可點），選字複製不受影響。
          <span
            data-task-detail
            className="flex cursor-auto flex-col gap-2 pb-1 text-[15px]"
            onClick={(e) => {
              if (!(e.target instanceof Element && e.target.closest("a"))) e.preventDefault();
            }}
          >
            {detail && <Markdown content={detail} className="[&_p]:my-0" />}
            {parts.verify && (
              <span className="flex flex-col gap-0.5">
                <span className="text-xs font-semibold text-muted-foreground">{t("tasks.verifyLabel")}</span>
                <Markdown content={parts.verify} className="[&_p]:my-0" />
              </span>
            )}
          </span>
        )}
      </span>
    </>
  );
}

/** 任務列外框共用樣式。 */
const ROW_CLS = "group/task flex items-start gap-2 py-1 pl-1 rounded-md";

/** 列的底色：「下一個未完成」高亮優先；未勾的手動任務整列暖黃淡底（design D14）；其餘 hover 才淡灰。
 * 唯讀列點了沒反應，不給 hover 底色。 */
function rowSurface(item: TaskItem, interactive: boolean, highlight?: boolean): string {
  if (highlight) return "bg-accent ring-1 ring-primary/40";
  if (item.manual && !item.done) return interactive ? "bg-status-manual/12 hover:bg-status-manual/20" : "bg-status-manual/12";
  return interactive ? "hover:bg-foreground/5" : "";
}

/** 手動任務列的標記（未勾與已勾都帶）。 */
const manualMark = (item: TaskItem) => (item.manual ? { "data-task-manual": "" } : {});

/** 整列為勾選框的 label（design D8）：可寫時 htmlFor 指向勾選框、游標為手指；唯讀與
 * 批次寫回中（busy）不帶 htmlFor、游標預設，勾選框同時停用——隱式關聯也不會觸發。 */
function rowLabelProps(checkboxId: string, interactive: boolean) {
  return {
    htmlFor: interactive ? checkboxId : undefined,
    className: interactive ? "cursor-pointer" : "cursor-default",
    onClick: ignoreClickAfterSelecting,
  };
}

/** 拖選文字後放開，click 仍會經 label 轉送到勾選框——頁面有選取文字時不轉送，選字不等於
 * 勾選。雙擊選字的第二下照常放行：第一下按下時尚無選取、已切換一次，兩下互相抵銷。
 * 直接點在勾選框上照常切換。 */
function ignoreClickAfterSelecting(e: MouseEvent<HTMLLabelElement>) {
  if (e.detail >= 2) return;
  if (e.target instanceof Element && e.target.closest('[role="checkbox"]')) return;
  if (window.getSelection()?.toString()) e.preventDefault();
}

/** 不可拖排的任務列（唯讀或拖排停用）：保留高亮與定位捲動。 */
function PlainTaskRow({
  item,
  interactive,
  onToggle,
  onJumpDecision,
  highlight,
  rowRef,
}: {
  item: TaskItem;
  interactive: boolean;
  onToggle?: (ordinal: number, done: boolean, stableId?: string) => void;
  onJumpDecision?: (id: string) => void;
  highlight?: boolean;
  rowRef?: (el: HTMLElement | null) => void;
}) {
  const checkboxId = useId();
  const label = rowLabelProps(checkboxId, interactive);
  return (
    <label
      ref={rowRef}
      htmlFor={label.htmlFor}
      onClick={label.onClick}
      data-highlight={highlight ? "true" : "false"}
      {...manualMark(item)}
      className={`${ROW_CLS} ${label.className} transition-colors ${rowSurface(item, interactive, highlight)}`}
    >
      <TaskRowBody
        item={item}
        checkboxId={checkboxId}
        disabled={!interactive}
        onToggle={onToggle}
        onJumpDecision={onJumpDecision}
      />
    </label>
  );
}

/** 群組卡頭：左端序號籤、16px 粗體標題（本頁目錄錨點）、右端小進度「已完成/總數」。 */
function GroupHeader({
  text,
  done,
  total,
  nodeRef,
  style,
}: {
  text: string;
  done: number;
  total: number;
  nodeRef?: Ref<HTMLDivElement>;
  style?: CSSProperties;
}) {
  const numbered = GROUP_ORDINAL_RE.exec(text);
  return (
    <div ref={nodeRef} style={style} className="flex items-start gap-2 border-b border-border/70 px-4 py-3">
      {numbered && <span className={ORDINAL_CHIP_CLS}>{numbered[1]}</span>}
      <h3 data-doc-heading className={`min-w-0 flex-1 ${SECTION_HEADING_CLS}`}>
        {numbered ? numbered[2] : text}
      </h3>
      <span data-task-group-progress className="mt-0.5 shrink-0 font-mono text-xs tabular-nums text-muted-foreground">
        {`${done}/${total}`}
      </span>
    </div>
  );
}

/** 可拖排時的群組卡頭：不可拖，但以 disabled sortable 項參與讓位序列（design D6）——
 * 拖曳中任務與標題保持相對順序、讓位視覺不穿越群組邊界；卡頭本身可為落點（組首）。 */
function SortableGroupHeader({ id, ...props }: { id: string; text: string; done: number; total: number }) {
  const { setNodeRef, transform, transition } = useSortable({ id, disabled: true });
  return <GroupHeader {...props} nodeRef={setNodeRef} style={{ transform: CSS.Transform.toString(transform), transition }} />;
}

/** 可拖曳任務列：拖曳監聽只綁 ⠿ 把手——點擊核取方塊與文字不經過拖曳；點把手不切換勾選。 */
function SortableTaskRow({
  item,
  interactive,
  onToggle,
  onJumpDecision,
  highlight,
  rowRef,
}: {
  item: TaskItem;
  interactive: boolean;
  onToggle?: (ordinal: number, done: boolean, stableId?: string) => void;
  onJumpDecision?: (id: string) => void;
  /** 「下一個未完成」的短暫高亮標記。 */
  highlight?: boolean;
  /** 列元素回報（定位捲動用）。 */
  rowRef?: (el: HTMLElement | null) => void;
}) {
  const { t } = useI18n();
  const checkboxId = useId();
  const label = rowLabelProps(checkboxId, interactive);
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: item.ordinal,
  });
  return (
    <label
      ref={(el) => {
        setNodeRef(el);
        rowRef?.(el);
      }}
      htmlFor={label.htmlFor}
      onClick={label.onClick}
      style={{ transform: CSS.Transform.toString(transform), transition }}
      data-highlight={highlight ? "true" : "false"}
      {...manualMark(item)}
      className={`${ROW_CLS} ${label.className} transition-colors ${rowSurface(item, interactive, highlight)} ${
        isDragging ? "opacity-40" : ""
      }`}
    >
      <Button
        type="button"
        variant="ghost"
        size="icon"
        aria-label={t("tasks.drag").replace("{n}", String(item.ordinal))}
        className="mt-1 h-4 w-4 shrink-0 cursor-grab touch-none text-muted-foreground/50 hover:text-foreground"
        {...attributes}
        {...listeners}
        // 把手在 label 內：點擊不得冒到列上觸發勾選。
        onClick={(e) => {
          e.preventDefault();
          e.stopPropagation();
        }}
      >
        <GripVertical className="h-3.5 w-3.5" />
      </Button>
      <TaskRowBody
        item={item}
        checkboxId={checkboxId}
        disabled={!interactive}
        onToggle={onToggle}
        onJumpDecision={onJumpDecision}
      />
    </label>
  );
}

/** 互動任務清單：頂部批次工具列＋群組卡＋可勾選 checkbox＋⠿ 把手拖放排序，
 * 與 tasks.md 聯動。 */
export function TaskList({
  markdown,
  onToggle,
  onReorder,
  busy,
  onDragActiveChange,
  readOnly,
  onSetAll,
  onJumpDecision,
  locate,
  onLocated,
}: TaskListProps) {
  const { t } = useI18n();
  const items = parseTaskDoc(markdown);
  const taskItems = items.filter((i): i is TaskItem => i.kind === "task");
  const [activeOrdinal, setActiveOrdinal] = useState<number | null>(null);
  // 「下一個未完成」的短暫高亮（ordinal）；列元素表供定位捲動。
  const [highlightOrdinal, setHighlightOrdinal] = useState<number | null>(null);
  const highlightTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const rowRefs = useRef(new Map<number, HTMLElement>());
  // PointerSensor distance 8：位移門檻內的按放是點擊，不啟動拖曳（看板同款教訓）。
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  const allDone = taskItems.length > 0 && taskItems.every((i) => i.done);

  // 捲到某列並短暫高亮——「下一個未完成」與頁的「前往」共用。
  const highlightRow = (ordinal: number) => {
    rowRefs.current.get(ordinal)?.scrollIntoView({ block: "center", behavior: "smooth" });
    setHighlightOrdinal(ordinal);
    if (highlightTimer.current) clearTimeout(highlightTimer.current);
    highlightTimer.current = setTimeout(() => setHighlightOrdinal(null), 1600);
  };

  const locateNext = () => {
    const next = taskItems.find((i) => !i.done);
    if (next) highlightRow(next.ordinal);
  };

  // 頁的定位請求：列在這一輪 commit 後已掛上 ref，effect 裡捲動才找得到。
  useEffect(() => {
    if (!locate) return;
    highlightRow(locate.ordinal);
    onLocated?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [locate]);

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

  // 拖曳進行中按 Esc 只取消拖曳：PointerSensor 不標記已處理，Esc 會冒到詳情頁的關頁快捷鍵。
  // 捕獲階段先標記，宿主的 window 監聽見 defaultPrevented 即讓路。
  useEffect(() => {
    if (activeOrdinal === null) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") e.preventDefault();
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [activeOrdinal]);

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

  // 可拖排＝互動且宿主供拖排寫回；capability 缺口（remote）時把手與 DndContext
  // 一併不上——絕不渲染點了沒事的假 affordance。
  const sortable = !readOnly && onReorder !== undefined;

  // 列可回應點擊＝非唯讀且不在批次寫回中（busy）。
  const interactive = !readOnly && !busy;
  const rowRef = (ordinal: number) => (el: HTMLElement | null) => {
    if (el) rowRefs.current.set(ordinal, el);
    else rowRefs.current.delete(ordinal);
  };

  const renderRow = (item: TaskItem) =>
    !sortable ? (
      // 唯讀或無拖排：無把手；互動時保留高亮與定位捲動（「下一個未完成」照常）。
      <PlainTaskRow
        key={`t-${taskKey(item)}`}
        item={item}
        interactive={interactive}
        onToggle={onToggle}
        onJumpDecision={onJumpDecision}
        highlight={highlightOrdinal === item.ordinal}
        rowRef={rowRef(item.ordinal)}
      />
    ) : (
      <SortableTaskRow
        key={`t-${taskKey(item)}`}
        item={item}
        interactive={interactive}
        onToggle={onToggle}
        onJumpDecision={onJumpDecision}
        highlight={highlightOrdinal === item.ordinal}
        rowRef={rowRef(item.ordinal)}
      />
    );

  // 依群組標題切成卡（design D5）：第一個標題之前的任務為無卡頭清單；卡頭 id 沿用
  // `g-<items 索引>`，拖放落點解析（resolveDropTarget）不動。
  const segments: { index: number | null; text: string; tasks: TaskItem[] }[] = [];
  items.forEach((item, i) => {
    if (item.kind === "group") segments.push({ index: i, text: item.text, tasks: [] });
    else {
      if (segments.length === 0) segments.push({ index: null, text: "", tasks: [] });
      segments[segments.length - 1].tasks.push(item);
    }
  });
  const rows = (
    <div className="flex flex-col gap-3">
      {segments.map((seg) => {
        if (seg.index === null) {
          return (
            <div key="lead" className="flex flex-col">
              {seg.tasks.map(renderRow)}
            </div>
          );
        }
        const header = { text: seg.text, done: seg.tasks.filter((x) => x.done).length, total: seg.tasks.length };
        return (
          <Card key={`g-${seg.index}`} size="nested" data-task-group>
            {sortable ? <SortableGroupHeader id={`g-${seg.index}`} {...header} /> : <GroupHeader {...header} />}
            {seg.tasks.length > 0 && <div className="flex flex-col px-2 py-1.5">{seg.tasks.map(renderRow)}</div>}
          </Card>
        );
      })}
    </div>
  );

  // 唯讀（封存檢視）：無工具列、無把手、無 DndContext。
  if (readOnly) return rows;

  // 工具列鍵共通樣式（design D3：ghost＋sm 變體，補齊近似現狀的字級與字色）。
  const toolbarBtn = "gap-1.5 px-2 text-sm font-normal text-muted-foreground hover:text-foreground";
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

  /* 批次操作工具列（spec「任務分頁提供批次操作工具列」）：全部已完成／下一個
     未完成（n）／重置任務；全完成時前兩鍵不可用，批次寫回期間整列 disabled。 */
  const toolbar = (
    <>
    <p className="mb-1.5 text-xs text-muted-foreground">{t(sortable ? "detail.tasksHint" : "detail.tasksHintToggle")}</p>
    <div className="mb-2 flex items-center gap-1 rounded-md border border-border px-2 py-1.5">
      <Button
        type="button"
        variant="ghost"
        size="sm"
        className={toolbarBtn}
        disabled={allDone || busy}
        onClick={() => onSetAll?.(true)}
      >
        <CheckCheck className="h-3.5 w-3.5" /> {t("tasks.completeAll")}
      </Button>
      <Button
        type="button"
        variant="ghost"
        size="sm"
        className={toolbarBtn}
        disabled={allDone || busy}
        onClick={locateNext}
      >
        <LocateFixed className="h-3.5 w-3.5" /> {t("tasks.nextUndone")}
        <kbd className="rounded border border-border bg-muted px-1 text-[10px] text-muted-foreground">
          n
        </kbd>
      </Button>
      <div className="flex-1" />
      <Button
        type="button"
        variant="ghost"
        size="sm"
        className={toolbarBtn}
        disabled={busy}
        onClick={() => onSetAll?.(false)}
      >
        <RotateCcw className="h-3.5 w-3.5" /> {t("tasks.resetAll")}
      </Button>
    </div>
    </>
  );

  // 拖排停用（capability 缺口）：工具列與勾選照常、不掛 DndContext。
  if (!sortable) {
    return (
      <>
        {toolbar}
        <div className={`flex flex-col ${busy ? "opacity-60 pointer-events-none" : ""}`}>{rows}</div>
      </>
    );
  }

  return (
    <>
      {toolbar}
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
            <span className="mt-1 flex h-4 w-4 shrink-0 items-center justify-center text-muted-foreground/50">
              <GripVertical className="h-3.5 w-3.5" />
            </span>
            <TaskRowBody item={active} titleOnly />
          </div>
        ) : null}
      </DragOverlay>
    </DndContext>
    </>
  );
}

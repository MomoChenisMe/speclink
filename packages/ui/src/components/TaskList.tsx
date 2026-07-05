import { ChevronDown, ChevronUp } from "lucide-react";

import { parseTaskDoc } from "../tasks";

export interface TaskListProps {
  markdown: string | null;
  /** 勾選/取消第 ordinal 個任務（1-based）。 */
  onToggle?: (ordinal: number, done: boolean) => void;
  /** 把第 ordinal 個任務上移/下移一位。 */
  onMove?: (ordinal: number, dir: "up" | "down") => void;
  /** 寫入進行中時鎖定互動。 */
  busy?: boolean;
}

/** 互動任務清單：群組標題＋可勾選 checkbox＋上下移動排序，與 tasks.md 聯動。 */
export function TaskList({ markdown, onToggle, onMove, busy }: TaskListProps) {
  const items = parseTaskDoc(markdown);
  const taskCount = items.filter((i) => i.kind === "task").length;
  if (items.length === 0) {
    return <div className="text-muted-foreground text-sm py-6">（無任務）</div>;
  }
  return (
    <div className={`flex flex-col ${busy ? "opacity-60 pointer-events-none" : ""}`}>
      {items.map((item, i) =>
        item.kind === "group" ? (
          <h4 key={`g-${i}`} className="text-sm font-bold mt-4 mb-1.5 first:mt-0">
            {item.text}
          </h4>
        ) : (
          <div
            key={`t-${item.ordinal}`}
            className="group/task flex items-start gap-2 py-1 pl-1 rounded-md hover:bg-muted/50"
          >
            <input
              type="checkbox"
              aria-label={`任務 ${item.ordinal}`}
              className="mt-1 shrink-0 accent-[var(--primary)]"
              checked={item.done}
              onChange={(e) => onToggle?.(item.ordinal, e.target.checked)}
            />
            <span
              className={`flex-1 text-[13px] leading-relaxed ${
                item.done ? "text-muted-foreground line-through decoration-muted-foreground/50" : ""
              }`}
            >
              {item.text}
            </span>
            <span className="flex flex-col shrink-0 opacity-0 group-hover/task:opacity-100 transition-opacity">
              <button
                type="button"
                aria-label={`上移任務 ${item.ordinal}`}
                className="text-muted-foreground hover:text-foreground disabled:opacity-30"
                disabled={item.ordinal === 1}
                onClick={() => onMove?.(item.ordinal, "up")}
              >
                <ChevronUp className="h-3.5 w-3.5" />
              </button>
              <button
                type="button"
                aria-label={`下移任務 ${item.ordinal}`}
                className="text-muted-foreground hover:text-foreground disabled:opacity-30"
                disabled={item.ordinal === taskCount}
                onClick={() => onMove?.(item.ordinal, "down")}
              >
                <ChevronDown className="h-3.5 w-3.5" />
              </button>
            </span>
          </div>
        ),
      )}
    </div>
  );
}

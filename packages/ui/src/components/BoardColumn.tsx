import type { ReactNode } from "react";
import type { LucideIcon } from "lucide-react";

/** 欄的色相組（圖示、色相條、圖示色、計數徽章色），全部取 stage.ts 的 stage-* token。 */
export interface ColumnTone {
  icon: LucideIcon;
  bar: string;
  iconCls: string;
  badge: string;
}

/**
 * 看板欄的容器卡（desktop-board-reskin design D1）：白底細框 16px 圓角，頂端 3px 色相條、
 * 白底標頭（色相圖示、欄名、計數）下細線、卡片區鋪側欄淡灰並於欄內縱向捲動——與設定頁、
 * 系統匣面板同一套容器規則。階段欄與討論欄共用，欄外觀只在這裡定義。
 *
 * 欄容器不作 droppable：跨欄放開為彈回＋零寫入（spec「跨欄拖曳不改變變更階段」），
 * isOver 高亮會假示可跨欄放置；唯一的欄外落點是封存浮層。
 */
export function BoardColumn({
  id,
  title,
  tone,
  count,
  empty,
  footer,
  children,
}: {
  /** data-column 值：階段名或 "discussions"。 */
  id: string;
  title: string;
  tone: ColumnTone;
  /** 計數徽章；null＝計數未知（首訪載入中或失敗），顯示 0 會謊報空，徽章整個不出。 */
  count: number | null;
  /** 卡片區頂端的置中灰字空文案；null＝不顯示。 */
  empty: string | null;
  /** 欄底固定區（討論欄的已轉出展開區與收合列），不隨卡片區捲動。 */
  footer?: ReactNode;
  children: ReactNode;
}) {
  const Icon = tone.icon;
  return (
    <div
      data-column={id}
      className="flex h-full min-h-0 flex-col overflow-hidden rounded-2xl border border-border bg-sidebar"
    >
      <div className={`h-[3px] shrink-0 ${tone.bar}`} />
      <div className="flex items-center gap-2 bg-card px-3 py-2.5 border-b border-border/70 shrink-0">
        <Icon className={`h-3.5 w-3.5 ${tone.iconCls}`} />
        <h2 className="text-xs font-semibold text-foreground">{title}</h2>
        <div className="flex-1" />
        {count !== null && (
          <span
            data-testid="column-count"
            className={`inline-flex items-center justify-center min-w-5 h-5 px-1.5 rounded-full text-[11px] font-semibold tabular-nums ${tone.badge}`}
          >
            {count}
          </span>
        )}
      </div>
      <div className="flex-1 min-h-0 overflow-y-auto flex flex-col gap-2 p-2">
        {empty !== null && <p className="px-2 py-6 text-center text-xs text-muted-foreground">{empty}</p>}
        {children}
      </div>
      {footer}
    </div>
  );
}

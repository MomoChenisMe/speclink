// 面板樣式呈現層（spec「面板樣式（macOS）」；design D5）：主視窗推送的
// TraySnapshot 薄渲染——分區內容與原生選單同源（同一 store 快照），不自建
// 資料查詢路徑。原生質感（vibrancy、不搶焦點、失焦收合、高度自適應）由
// Rust 側面板視窗與入口層承擔，此元件只負責結構與回呼；複製鈕常駐列尾、
// 分區圖示沿用看板同款（KanbanBoard 的 stage 圖示）。
import { useState, type ReactNode } from "react";
import {
  changeStage,
  STAGES,
  cn,
  useI18n,
  type ChangeItem,
  type Stage,
} from "@speclink/ui";
import {
  ArrowUpRight,
  Check,
  CircleCheckBig,
  Copy,
  Hammer,
  Lightbulb,
  MessageSquareText,
  type LucideIcon,
} from "lucide-react";

import { OVERFLOW_LIMIT, type TraySnapshot } from "../tray";

/** 分區圖示與看板欄位同款（KanbanBoard）：提案中／進行中／已就緒。 */
const STAGE_ICONS: Record<Stage, LucideIcon> = {
  proposed: Lightbulb,
  "in-progress": Hammer,
  ready: CircleCheckBig,
};

export interface TrayPanelProps {
  /** 主視窗推送的快照；null＝尚未收到（lazy 建窗後的短暫空窗）。 */
  snapshot: TraySnapshot | null;
  onOpenProject: (root: string) => void;
  onOpenChange: (name: string) => void;
  onOpenDiscussion: (slug: string) => void;
  onOpenApp: () => void;
  /** 複製回呼（面板入口接 clipboard 外掛——Rust 端，不受焦點限制）。 */
  onCopy: (text: string) => void;
}

/** 列尾常駐複製鈕：stopPropagation 使複製不觸發列本體的開啟；
    點擊後短暫轉勾號回饋（1.2 秒復原——看板 ChangeList 的 copied 同模式）。 */
function CopyButton({ label, text, onCopy }: { label: string; text: string; onCopy: (t: string) => void }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={(e) => {
        e.stopPropagation();
        onCopy(text);
        setCopied(true);
        setTimeout(() => setCopied(false), 1200);
      }}
      className="shrink-0 rounded p-1 text-muted-foreground hover:bg-foreground/15 group-hover:text-primary-foreground"
    >
      {copied ? (
        <Check className="h-3.5 w-3.5 text-primary group-hover:text-primary-foreground" />
      ) : (
        <Copy className="h-3.5 w-3.5" />
      )}
    </button>
  );
}

function SectionHeader({ icon: Icon, label }: { icon: LucideIcon; label: string }) {
  return (
    <div className="flex items-center gap-1.5 px-2 pt-1.5 pb-0.5 text-[11px] font-medium text-muted-foreground">
      <Icon className="h-3 w-3" />
      {label}
    </div>
  );
}

/** 原生選單式列：hover 整列 accent 反白（含子元素經 group-hover 反白）。 */
const rowClass =
  "group flex w-full items-center gap-2 rounded-[5px] px-2 py-1 text-left hover:bg-primary hover:text-primary-foreground";

/** 分區溢出（spec「分區溢出摺疊」）：直列前 5、其餘收進「還有 N 個…」可展開列。 */
function OverflowGroup({
  rows,
  moreLabel,
  collapseLabel,
}: {
  rows: ReactNode[];
  moreLabel: (n: number) => string;
  collapseLabel: string;
}) {
  const [expanded, setExpanded] = useState(false);
  if (rows.length <= OVERFLOW_LIMIT) return <>{rows}</>;
  return (
    <>
      {rows.slice(0, OVERFLOW_LIMIT)}
      {expanded && rows.slice(OVERFLOW_LIMIT)}
      <div className={rowClass} onClick={() => setExpanded((v) => !v)}>
        <span className="text-[12px] text-muted-foreground group-hover:text-primary-foreground">
          {expanded ? collapseLabel : moreLabel(rows.length - OVERFLOW_LIMIT)}
        </span>
      </div>
    </>
  );
}

export function TrayPanel({
  snapshot,
  onOpenProject,
  onOpenChange,
  onOpenDiscussion,
  onOpenApp,
  onCopy,
}: TrayPanelProps) {
  const { t } = useI18n();
  const moreLabel = (n: number) => t("tray.more").replace("{n}", String(n));
  const collapseLabel = t("tray.collapse");
  const tabs = snapshot?.tabs ?? [];
  const changes = snapshot?.changes ?? [];
  const discussions = snapshot?.discussions ?? [];
  const openDiscussions = discussions.filter((d) => !d.promoted);
  const promotedDiscussions = discussions.filter((d) => d.promoted);
  const staged = STAGES.map((stage) => ({
    stage,
    items: changes.filter((c) => changeStage(c) === stage),
  })).filter((s) => s.items.length > 0);

  return (
    <div className="flex flex-col p-1.5 text-[13px] text-foreground">
      {/* 專案區：作用中打勾（與原生選單同序） */}
      {tabs.length > 0 && (
        <>
          {tabs.map((tab) => {
            const active = tab.root === snapshot?.activeRoot;
            return (
              <div
                key={tab.root}
                data-testid={`panel-project-${tab.root}`}
                data-active={active ? "true" : "false"}
                onClick={() => onOpenProject(tab.root)}
                className={cn(rowClass, active && "font-medium")}
              >
                <span className="w-3.5 shrink-0 text-xs">{active ? "✓" : ""}</span>
                <span className="truncate">{tab.name}</span>
              </div>
            );
          })}
          <hr className="my-1 border-foreground/10" />
        </>
      )}

      {/* 生命週期分區：各階段 header＋變更列（真進度條取代 unicode 方塊） */}
      {staged.length === 0 ? (
        <div className="px-2 py-1 text-muted-foreground">{t("tray.noChanges")}</div>
      ) : (
        staged.map(({ stage, items }) => (
          <section key={stage}>
            <SectionHeader icon={STAGE_ICONS[stage]} label={t(`stage.${stage}`)} />
            <OverflowGroup
              moreLabel={moreLabel}
              collapseLabel={collapseLabel}
              rows={items.map((c) => (
                <ChangeRow key={c.name} c={c} onOpen={onOpenChange} onCopy={onCopy} copyLabel={t("tray.copyName")} />
              ))}
            />
          </section>
        ))
      )}

      <hr className="my-1 border-foreground/10" />

      {/* 討論區分流（spec「討論列表」）：「討論」列討論中、「已轉出」列已轉出；
          slug 為題、topic 為描述（識別錨點慣例，與看板討論卡一致） */}
      {openDiscussions.length > 0 ? (
        <section>
          <SectionHeader icon={MessageSquareText} label={t("tray.discussionsHeader")} />
          <OverflowGroup
            moreLabel={moreLabel}
            collapseLabel={collapseLabel}
            rows={openDiscussions.map((d) => (
              <DiscussionRow key={d.slug} d={d} onOpen={onOpenDiscussion} onCopy={onCopy} copyLabel={t("tray.copySlug")} />
            ))}
          />
        </section>
      ) : (
        <div className="px-2 py-1 text-muted-foreground">
          {t("tray.discussions").replace("{n}", "0")}
        </div>
      )}
      {promotedDiscussions.length > 0 && (
        <section>
          <SectionHeader icon={ArrowUpRight} label={t("tray.promotedHeader")} />
          <OverflowGroup
            moreLabel={moreLabel}
            collapseLabel={collapseLabel}
            rows={promotedDiscussions.map((d) => (
              <DiscussionRow key={d.slug} d={d} onOpen={onOpenDiscussion} onCopy={onCopy} copyLabel={t("tray.copySlug")} />
            ))}
          />
        </section>
      )}

      <hr className="my-1 border-foreground/10" />

      {/* 動作區：開啟主視窗（面板樣式下進 app 的把手） */}
      <div onClick={onOpenApp} className={rowClass}>
        <ArrowUpRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground group-hover:text-primary-foreground" />
        {t("tray.open")}
      </div>
    </div>
  );
}

function DiscussionRow({
  d,
  onOpen,
  onCopy,
  copyLabel,
}: {
  d: { slug: string; topic: string };
  onOpen: (slug: string) => void;
  onCopy: (text: string) => void;
  copyLabel: string;
}) {
  return (
    <div
      data-testid={`panel-discussion-${d.slug}`}
      onClick={() => onOpen(d.slug)}
      className={rowClass}
    >
      <div className="min-w-0 flex-1">
        <div className="truncate font-mono text-[12px]">{d.slug}</div>
        <div className="truncate text-[11px] text-muted-foreground group-hover:text-primary-foreground/80">
          {d.topic}
        </div>
      </div>
      <CopyButton label={copyLabel} text={d.slug} onCopy={onCopy} />
    </div>
  );
}

function ChangeRow({
  c,
  onOpen,
  onCopy,
  copyLabel,
}: {
  c: ChangeItem;
  onOpen: (name: string) => void;
  onCopy: (text: string) => void;
  copyLabel: string;
}) {
  const pct = c.totalTasks > 0 ? Math.round((c.completedTasks / c.totalTasks) * 100) : 0;
  return (
    <div data-testid={`panel-change-${c.name}`} onClick={() => onOpen(c.name)} className={rowClass}>
      <div className="min-w-0 flex-1">
        <div className="flex items-baseline gap-2">
          <span className="truncate">{c.name}</span>
          {c.totalTasks > 0 && (
            <span className="ml-auto shrink-0 text-[11px] tabular-nums text-muted-foreground group-hover:text-primary-foreground/80">
              {`${c.completedTasks}/${c.totalTasks}`}
            </span>
          )}
        </div>
        {c.totalTasks > 0 && (
          <div className="mt-1 h-1 overflow-hidden rounded-full bg-foreground/10 group-hover:bg-primary-foreground/25">
            <div
              className="h-full rounded-full bg-primary group-hover:bg-primary-foreground"
              style={{ width: `${pct}%` }}
            />
          </div>
        )}
      </div>
      <CopyButton label={copyLabel} text={c.name} onCopy={onCopy} />
    </div>
  );
}

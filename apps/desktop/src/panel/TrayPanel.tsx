// 面板樣式呈現層（spec「面板樣式（macOS）」；design D5）：主視窗推送的
// TraySnapshot 薄渲染——分區內容與原生選單同源（同一 store 快照），不自建
// 資料查詢路徑。原生質感（vibrancy、不搶焦點、失焦收合、高度自適應）由
// Rust 側面板視窗與入口層承擔，此元件只負責結構與回呼；複製鈕常駐列尾、
// 分區圖示沿用看板同款（KanbanBoard 的 stage 圖示）。
import {
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type KeyboardEvent,
  type ReactNode,
} from "react";
import {
  Button,
  changeStage,
  CopyButton,
  DISCUSSION_TONE,
  planBlockedBy,
  planBlockedLabel,
  planWave,
  planWaveLabel,
  REVIEW_ICON,
  REVIEW_LABEL_KEY,
  REVIEW_TONE,
  RowSkeleton,
  SEMANTIC_SURFACE,
  SEMANTIC_TONE,
  STAGE_BADGE,
  STAGE_BAR,
  STAGE_ICON,
  STAGES,
  VERIFY_ICON,
  VERIFY_LABEL_KEY,
  VERIFY_TONE,
  cn,
  SectionHeader,
  useI18n,
  type ChangeItem,
  type Stage,
} from "@speclink/ui";
import {
  AlertTriangle,
  ArrowUpRight,
  ChevronRight,
  CircleCheckBig,
  Cloud,
  CloudOff,
  Folder,
  GitBranch,
  Hammer,
  Lightbulb,
  LoaderCircle,
  LogIn,
  MessageSquareText,
  Plus,
  Power,
  RefreshCw,
  Server,
  Settings,
  SlidersHorizontal,
  type LucideIcon,
} from "lucide-react";

import {
  ADD_TILE,
  PROJECT_TILE_ACTIVE,
  PROJECT_TILE_IDLE,
  RAIL_SQUARE,
  projectInitial,
} from "../components/projectTile";
import { OVERFLOW_LIMIT, type TraySnapshot, type TrayTabSnapshot } from "../tray";
import type { Platform } from "../platform";
import { readTrayPromotedExpanded, writeTrayPromotedExpanded } from "./promotedExpanded";
import {
  readTrayCollapsedSections,
  writeTrayCollapsedSections,
  type TraySectionId,
} from "./sectionCollapse";
import { tabRevealScrollLeft } from "./tabStrip";
import { edgeFade, edgeFadeMask, type EdgeFadeSide, type ScrollMetrics } from "./edgeFade";

/** 分區圖示與看板欄位同款（KanbanBoard）：提案中／進行中／已就緒。 */
const STAGE_ICONS: Record<Stage, LucideIcon> = {
  proposed: Lightbulb,
  "in-progress": Hammer,
  ready: CircleCheckBig,
};

export interface TrayPanelProps {
  /** 主視窗推送的快照；null＝尚未收到（lazy 建窗後的短暫空窗）。 */
  snapshot: TraySnapshot | null;
  /** 面板所在平台（tray-panel-windows design D2）：只決定根層外觀——macOS 為毛玻璃補光
      與 13px 圓角，Windows 為不透明側欄底色、不畫圓角；其餘呈現不分平台。 */
  platform: Platform;
  onOpenProject: (key: string) => void;
  onOpenChange: (name: string) => void;
  onOpenDiscussion: (slug: string) => void;
  onOpenApp: () => void;
  /** 開主視窗並切至作用中專案的專案設定頁（動作區「專案設定」；與原生選單同語意）。 */
  onOpenProjectSettings: () => void;
  /** 開主視窗並切至設定頁（動作區「設定」；與原生選單同語意）。 */
  onOpenSettings: () => void;
  /** 結束 app（動作區「結束」；面板端經 Rust 命令結束行程）。 */
  onQuit: () => void;
  /** 複製回呼（面板入口接 clipboard 外掛——Rust 端，不受焦點限制）。 */
  onCopy: (text: string) => void;
  /** 快速加入專案（design D7）：開資料夾選擇器，選定即加入並切換、取消無事。 */
  onAddProject: () => void;
  /** 原地 retry：只回流主視窗 store，不顯示或聚焦主視窗。 */
  onRetryWorkspace: (key: string) => void;
  /** 顯式在主視窗顯示對應 recovery destination。 */
  onOpenRecovery: (key: string) => void;
  /** 顯式顯示並聚焦對應 connection 的伺服器設定。 */
  onOpenServerSettings: (connectionId: string) => void;
  /** 顯式顯示並聚焦對應 connection 的登入流程。 */
  onReauthenticate: (connectionId: string) => void;
}

/** 分區卡標頭（tray-panel-restyle design D2）：12px 粗體前景色；其下有內容（列、骨架、
    失敗提示、收合列）才畫細線——空狀態卡只有標頭，不留一條孤線。 */
const sectionHeaderClass = (hasBody: boolean) =>
  cn("px-3 py-2 text-foreground", hasBody && "border-b border-border/70");

/** 面板卡片（spec「面板樣式（macOS）」；tray-panel-restyle design D1）：白底細框 12px
    圓角卡疊在補光層上——tab 條、各分區、動作區各一張。overflow-hidden 讓最後一列的
    hover 底色被卡角裁切。 */
function PanelCard({
  testid,
  className,
  children,
}: {
  testid: string;
  /** 附加樣式（空狀態卡的最小高度與垂直置中，design D8）。 */
  className?: string;
  children: ReactNode;
}) {
  return (
    <section
      data-testid={testid}
      className={cn("flex flex-col overflow-hidden rounded-xl border border-border bg-card", className)}
    >
      {children}
    </section>
  );
}

/** 空狀態分區卡附加樣式（design D8）：最小高度＋內容垂直置中——生命週期
    零筆階段卡與討論零筆卡共用，維持空狀態同構。 */
const emptyCardClass = "min-h-12 justify-center";

/** role="button" 的 div 補上 Enter／Space 切換（與原生 button 一致：按住不放的自動重複不再切換）。 */
const toggleOnKey = (toggle: () => void) => (event: KeyboardEvent<HTMLElement>) => {
  if (event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    if (!event.repeat) toggle();
  }
};

/** 分區的載入狀態（design D3）：四張內容卡同一個答案，由 store 導出的旗標換算。 */
type SectionLoad = "loading" | "failed" | "ready";

/**
 * 可收合的內容卡（spec「面板內容卡收合」；design「收合觸發沿用已轉出收合列的模式」）：
 * 整條分區標題為觸發區（role="button"、tabIndex=-1、Enter／Space，同已轉出收合列），
 * 箭頭放 SectionHeader 的 action 插槽、在計數之後（展開朝下、收合朝右）。收合時只留
 * 標題一行——其下內容卸載（溢出列的展開暫態隨之歸零），不套空狀態最小高度、標題下
 * 不畫細線。
 *
 * 載入狀態由卡片統一換算：載入中為佔位列、失敗為失敗提示，兩者都不出計數（不謊報 0）；
 * 有料時才顯示 count 與 children。empty＝有料時其下也沒有內容的空狀態卡。
 */
function SectionCard({
  id,
  collapsed,
  onToggle,
  load,
  icon,
  label,
  count,
  countClassName,
  empty,
  children,
}: {
  id: TraySectionId;
  collapsed: boolean;
  onToggle: () => void;
  load: SectionLoad;
  icon: ReactNode;
  label: string;
  count: number;
  countClassName: string;
  empty: boolean;
  children: ReactNode;
}) {
  const { t } = useI18n();
  const ready = load === "ready";
  const isEmpty = ready && empty;
  return (
    <PanelCard
      testid={`panel-section-${id}`}
      className={!collapsed && isEmpty ? emptyCardClass : undefined}
    >
      <div
        data-testid="panel-section-toggle"
        role="button"
        aria-expanded={!collapsed}
        tabIndex={-1}
        onClick={onToggle}
        onKeyDown={toggleOnKey(onToggle)}
      >
        <SectionHeader
          icon={icon}
          label={label}
          count={ready ? count : undefined}
          countClassName={countClassName}
          action={
            <ChevronRight
              className={cn("h-3.5 w-3.5 shrink-0 text-muted-foreground transition-transform", !collapsed && "rotate-90")}
            />
          }
          className={sectionHeaderClass(!collapsed && !isEmpty)}
        />
      </div>
      {!collapsed &&
        (load === "loading" ? (
          <SectionSkeleton />
        ) : load === "failed" ? (
          <SectionLoadFailed label={t("tray.loadFailed")} />
        ) : (
          children
        ))}
    </PanelCard>
  );
}

/** 分區載入失敗的終態列（spec「面板分區載入失敗終態」）：讀不到 ≠ 確認是空的，
    故不走空狀態文案；亦不留骨架——載入已經結束了。 */
function SectionLoadFailed({ label }: { label: string }) {
  return (
    <div
      data-testid="panel-section-load-failed"
      role="status"
      className="flex min-h-8 items-center gap-2 px-2 py-1 text-[11px] text-muted-foreground"
    >
      <CloudOff className="h-3.5 w-3.5 shrink-0" />
      <span>{label}</span>
    </div>
  );
}

/** 分區載入中的佔位列組（design D5）：兩列即讀得出「有內容正在載」。 */
function SectionSkeleton() {
  return (
    <>
      <RowSkeleton />
      <RowSkeleton />
    </>
  );
}

/** 捲動停止後指示條淡出的延遲（ms）——比照 macOS overlay 捲軸的滯留時間。 */
const SCROLL_IDLE_MS = 800;

/**
 * 監看可捲區的捲動量測（tab 條與中段共用）：立即回報一次，之後於捲動時（scrolled=true）與
 * 可捲區或其直接子節點尺寸改變時（scrolled=false）再回報——中段的內容 wrapper 隨收合與快照
 * 增減改變高度，可捲區本身被 flex 約束、外框不變。回傳解除監看的函式。
 */
function watchScroll(
  el: HTMLElement,
  axis: "x" | "y",
  onMeasure: (metrics: ScrollMetrics, scrolled: boolean) => void,
): () => void {
  const measure = (scrolled: boolean) =>
    onMeasure(
      axis === "x"
        ? { position: el.scrollLeft, scrollSize: el.scrollWidth, viewportSize: el.clientWidth }
        : { position: el.scrollTop, scrollSize: el.scrollHeight, viewportSize: el.clientHeight },
      scrolled,
    );
  const onScroll = () => measure(true);
  const observer = new ResizeObserver(() => measure(false));
  measure(false);
  el.addEventListener("scroll", onScroll, { passive: true });
  observer.observe(el);
  for (const child of Array.from(el.children)) observer.observe(child);
  return () => {
    el.removeEventListener("scroll", onScroll);
    observer.disconnect();
  };
}

/** 遮罩掛在可捲區本身（不隨內容捲動）；WebKit 仍認前綴屬性，兩個都給。 */
const edgeFadeStyle = (axis: "x" | "y", side: EdgeFadeSide) => {
  const mask = edgeFadeMask(axis, side);
  return { maskImage: mask, WebkitMaskImage: mask };
};

/**
 * 可捲中段（spec 三段式版面）：生命週期／討論分區或復原卡；捲動面自 body 收斂至此容器，
 * overscroll-none 隨遷防 rubber-band。一次監看同時供上下緣淡出與捲動指示條：
 * - 上下緣依是否還有被捲走的內容淡出（design「中段上下緣淡出：與 tab 條共用邊緣淡出」），
 *   卡片不會在頁首／頁尾邊緣被硬切；遮罩掛在捲動容器上，指示條與頁首頁尾都在它之外、不被淡出。
 * - 捲動指示條（spec：捲動時浮現、停止後自動隱去、不佔版面寬度）自繪而非用原生捲軸——系統
 *   「總是顯示捲軸」偏好會讓原生捲軸常駐且佔寬、無法淡出，而 spec 要求此行為不依賴系統偏好。
 *   內容未溢出時不渲染；指示條為捲動容器的兄弟節點，才不會隨內容一起捲走。
 * 內層 wrapper 保持自然高度，入口層的 ResizeObserver 觀察的是這層（高度量測依據）。捲動時只有
 * 本元件重繪：卡片是上層傳入的 children，不隨捲動重繪。
 */
function PanelScrollArea({ children }: { children: ReactNode }) {
  const ref = useRef<HTMLDivElement>(null);
  const [fade, setFade] = useState<EdgeFadeSide>("none");
  const [thumb, setThumb] = useState<{ top: number; height: number } | null>(null);
  const [active, setActive] = useState(false);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    let timer: ReturnType<typeof setTimeout> | null = null;
    const unwatch = watchScroll(el, "y", (metrics, scrolled) => {
      setFade(edgeFade(metrics));
      const { position, scrollSize, viewportSize } = metrics;
      if (scrollSize <= viewportSize) {
        setThumb(null);
      } else {
        const height = Math.max(24, (viewportSize / scrollSize) * viewportSize);
        const top = (viewportSize - height) * (position / (scrollSize - viewportSize));
        setThumb({ top, height });
      }
      if (scrolled) {
        setActive(true);
        if (timer !== null) clearTimeout(timer);
        timer = setTimeout(() => setActive(false), SCROLL_IDLE_MS);
      }
    });
    return () => {
      unwatch();
      if (timer !== null) clearTimeout(timer);
    };
  }, []);
  return (
    <div className="relative flex min-h-0 flex-1 flex-col">
      <div
        ref={ref}
        data-testid="panel-scroll"
        data-fade={fade}
        style={edgeFadeStyle("y", fade)}
        className="min-h-0 flex-1 overflow-y-auto overscroll-none [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
      >
        <div data-testid="panel-scroll-content" className="flex flex-col gap-2">
          {children}
        </div>
      </div>
      {thumb && (
        <div
          aria-hidden
          data-testid="panel-scroll-indicator"
          data-active={active ? "true" : "false"}
          className={cn(
            "pointer-events-none absolute right-0.5 w-1 rounded-full bg-foreground/35 transition-opacity duration-300",
            active ? "opacity-100" : "opacity-0",
          )}
          style={{ top: thumb.top, height: thumb.height }}
        />
      )}
    </div>
  );
}

/** 卡內的列（design D2）：細線分隔、hover 整列半透明前景色灰底——列內符號維持本色。
    變更列、討論列、溢出列與收合列共用。 */
const rowClass =
  "flex w-full items-center gap-2 border-b border-border/70 px-3 py-1.5 text-left last:border-b-0 hover:bg-foreground/5";

/** 動作區塊卡的按鈕（文字鈕與圖示鈕共用）：hover 半透明前景色灰底，與面板列同語彙。 */
const actionButtonClass = "flex items-center rounded-lg hover:bg-foreground/5";

/** 動作區塊卡的圖示鈕：名稱走 aria-label＋原生 title——面板是無焦點的一瞥介面，
    掛 Radix Tooltip 會多一層 portal 與焦點管理（同站章 tooltip 的做法）。 */
function ActionIconButton({
  label,
  Icon,
  onClick,
}: {
  label: string;
  Icon: LucideIcon;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      tabIndex={-1}
      aria-label={label}
      title={label}
      onClick={onClick}
      className={cn(actionButtonClass, "h-7 w-7 shrink-0 justify-center text-muted-foreground")}
    >
      <Icon className="h-3.5 w-3.5" />
    </button>
  );
}

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
        <span className="text-[12px] text-muted-foreground">
          {expanded ? collapseLabel : moreLabel(rows.length - OVERFLOW_LIMIT)}
        </span>
      </div>
    </>
  );
}

/** 專案 tab 與「加入專案」共用的直欄容器（方塊＋其下文字，hover 灰底）。 */
const TAB_ITEM = "flex shrink-0 flex-col items-center gap-1 rounded-lg px-1 py-1.5 hover:bg-foreground/5";

function tabStatusLabel(tab: TrayTabSnapshot, t: (key: string) => string): string {
  if (tab.source === "local") return "";
  if (tab.status === "restoring") return t("tray.recovery.restoring");
  if (tab.status === "offline") return t("tray.recovery.offline");
  if (tab.status === "needs-reauth" || tab.failureKind === "needs-reauth") {
    return t("tray.recovery.needsReauth");
  }
  if (tab.failureKind === "access-denied") return t("tray.recovery.accessDenied");
  if (tab.failureKind === "not-found") return t("tray.recovery.notFound");
  if (tab.failureKind === "unknown") return t("tray.recovery.unknown");
  if (tab.status === "error") return t("tray.recovery.unreachable");
  return "";
}

/**
 * 分頁狀態文字的語意色（spec「介面狀態語意色分層」）：選取由首字母方塊承載，
 * 狀態則由方塊下這行文字承載——還原中＝藍、離線／需重新登入＝琥珀警示、
 * 連不上＝紅。無狀態文字的本機分頁走中性。
 */
function tabStatusTone(tab: TrayTabSnapshot): string {
  if (tab.source === "local") return "text-muted-foreground";
  if (tab.status === "restoring") return SEMANTIC_TONE.inProgress;
  if (tab.status === "offline" || tab.status === "needs-reauth" || tab.failureKind === "needs-reauth") {
    return SEMANTIC_TONE.warning;
  }
  if (tab.status === "error") return SEMANTIC_TONE.danger;
  return "text-muted-foreground";
}

function RemoteTabIcon({ tab }: { tab: Extract<TrayTabSnapshot, { source: "remote" }> }) {
  if (tab.status === "restoring") {
    return <LoaderCircle className="h-4 w-4 animate-spin motion-reduce:animate-none" />;
  }
  if (tab.status === "offline") return <CloudOff className="h-4 w-4" />;
  if (tab.status === "needs-reauth" || tab.failureKind === "needs-reauth") {
    return <LogIn className="h-4 w-4" />;
  }
  if (tab.status === "error") return <AlertTriangle className="h-4 w-4" />;
  return <Cloud className="h-4 w-4" />;
}

function PanelRecoveryCard({
  tab,
  onRetry,
  onOpenRecovery,
  onOpenServerSettings,
  onReauthenticate,
}: {
  tab: Extract<TrayTabSnapshot, { source: "remote" }>;
  onRetry: () => void;
  onOpenRecovery: () => void;
  onOpenServerSettings: () => void;
  onReauthenticate: () => void;
}) {
  const { t } = useI18n();
  const restoring = tab.status === "restoring";
  const needsReauth = tab.failureKind === "needs-reauth";
  const summary = tabStatusLabel(tab, t);
  // 卡面依狀態分色：還原中＝進行中的藍、需重新登入＝警示琥珀、其餘失敗＝錯誤紅。
  // （舊版一律塗琥珀，連不上與「等一下就好」看起來一樣嚴重。）
  const tone = restoring ? "inProgress" : needsReauth ? "warning" : "danger";
  return (
    <section
      data-testid="panel-recovery-card"
      data-status={tab.status}
      role="status"
      aria-live="polite"
      // 白卡（tray-panel-restyle design D5），只借語意色的邊框——故 surface 之後再蓋回白底。
      className={cn("rounded-xl border p-3", SEMANTIC_SURFACE[tone], "bg-card")}
    >
      <div className="flex items-start gap-2.5">
        <span
          className={cn(
            "flex h-8 w-8 shrink-0 items-center justify-center rounded-lg",
            SEMANTIC_SURFACE[tone],
            SEMANTIC_TONE[tone],
          )}
        >
          {restoring ? (
            <LoaderCircle className="h-4 w-4 animate-spin motion-reduce:animate-none" />
          ) : needsReauth ? (
            <LogIn className="h-4 w-4" />
          ) : (
            <AlertTriangle className="h-4 w-4" />
          )}
        </span>
        <div className="min-w-0">
          <div className="font-semibold">{summary}</div>
          <div className="mt-0.5 truncate text-[11px] text-muted-foreground">{tab.name}</div>
        </div>
      </div>

      <dl className="mt-3 grid gap-2 rounded-lg bg-muted/50 p-2 text-[11px]">
        <div className="flex min-w-0 items-center gap-1.5">
          <Cloud className="h-3 w-3 shrink-0 text-muted-foreground" />
          <dt className="text-muted-foreground">{t("tray.recovery.workspace")}</dt>
          <dd className="ml-auto truncate font-medium">{tab.name}</dd>
        </div>
        <div className="flex min-w-0 items-center gap-1.5">
          <Server className="h-3 w-3 shrink-0 text-muted-foreground" />
          <dt className="text-muted-foreground">{t("tray.recovery.server")}</dt>
          <dd className="ml-auto truncate font-medium">
            {tab.source === "remote" ? tab.serverLabel : ""}
            {tab.serverOrigin ? ` · ${tab.serverOrigin}` : ""}
          </dd>
        </div>
      </dl>

      <div className="mt-3 grid grid-cols-2 gap-1.5">
        {!restoring &&
          (needsReauth ? (
            <Button size="sm" tabIndex={-1} onClick={onReauthenticate} className="col-span-2">
              <LogIn className="h-3.5 w-3.5" />
              {t("tray.recovery.reauthenticate")}
            </Button>
          ) : (
            <Button size="sm" tabIndex={-1} onClick={onRetry} className="col-span-2">
              <RefreshCw className="h-3.5 w-3.5" />
              {t("tray.recovery.retry")}
            </Button>
          ))}
        <Button variant="outline" size="sm" tabIndex={-1} onClick={onOpenRecovery}>
          <ArrowUpRight className="h-3 w-3" />
          {t("tray.recovery.open")}
        </Button>
        <Button variant="outline" size="sm" tabIndex={-1} onClick={onOpenServerSettings}>
          <Settings className="h-3 w-3" />
          {t("tray.recovery.settings")}
        </Button>
      </div>
    </section>
  );
}

export function TrayPanel({
  snapshot,
  platform,
  onOpenProject,
  onOpenChange,
  onOpenDiscussion,
  onOpenApp,
  onOpenProjectSettings,
  onOpenSettings,
  onQuit,
  onCopy,
  onAddProject,
  onRetryWorkspace,
  onOpenRecovery,
  onOpenServerSettings,
  onReauthenticate,
}: TrayPanelProps) {
  const { t } = useI18n();
  const tabStripRef = useRef<HTMLDivElement>(null);
  // 已轉出收合列的展開狀態（design D4）：面板 webview 只建一次，重啟 app 後由本機記憶讀回。
  const [promotedExpanded, setPromotedExpanded] = useState(readTrayPromotedExpanded);
  const togglePromoted = () => {
    const next = !promotedExpanded;
    setPromotedExpanded(next);
    writeTrayPromotedExpanded(next);
  };
  const moreLabel = (n: number) => t("tray.more").replace("{n}", String(n));
  const discussionRow = (d: { slug: string; topic: string }) => (
    <DiscussionRow key={d.slug} d={d} onOpen={onOpenDiscussion} onCopy={onCopy} copyLabel={t("tray.copySlug")} />
  );
  const collapseLabel = t("tray.collapse");
  // 內容卡收合（spec「面板內容卡收合」）：本機記憶、所有專案共用一份——切換專案與快照更新
  // 都不改它，只有點擊會改（收合中的卡有新項目也不自動展開）。
  const [collapsedSections, setCollapsedSections] = useState(readTrayCollapsedSections);
  const toggleSection = (id: TraySectionId) => {
    const next = new Set(collapsedSections);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    setCollapsedSections(next);
    writeTrayCollapsedSections(next);
  };
  // 收合狀態與切換由分區 id 從同一份收合集合推出：呼叫端只寫一次 id。
  const sectionCollapse = (id: TraySectionId) => ({
    id,
    collapsed: collapsedSections.has(id),
    onToggle: () => toggleSection(id),
  });
  const tabs = snapshot?.tabs ?? [];
  const changes = snapshot?.changes ?? [];
  const discussions = snapshot?.discussions ?? [];
  const openDiscussions = discussions.filter((d) => !d.promoted);
  const promotedDiscussions = discussions.filter((d) => d.promoted);
  const staged = STAGES.map((stage) => ({
    stage,
    Icon: STAGE_ICONS[stage],
    items: changes.filter((c) => changeStage(c) === stage),
  }));
  const pendingTabKey = snapshot?.pendingTabKey ?? null;
  // 首訪載入中：分區以佔位列呈現。條件整條由 store 導出（design D3）——面板
  // 不自行組合旗標，與主視窗看板恆為同一個答案。快照未到（null）同樣算載入
  // 中——面板剛開、主視窗尚未推第一份快照，此時畫空分區與畫佔位列是同一個問題。
  // 首訪載入失敗的終態：載入已結束（故不與骨架並存），但沒有真值可畫——
  // 顯示失敗提示而非空態文案。
  const sectionLoad: SectionLoad =
    !snapshot || snapshot.workspaceLoading ? "loading" : snapshot.workspaceLoadFailed ? "failed" : "ready";
  const activeKey = snapshot?.activeKey ?? null;

  // tab 條左右淡出（design「專案 tab 條：「＋」固定右端、邊緣淡出、捲到作用中」）：tab 尺寸
  // 固定，改在 tab 清單改變時重新監看——新 tab 才被觀察，且立即重算一次。
  const [tabFade, setTabFade] = useState<EdgeFadeSide>("none");
  const tabKeys = tabs.map((tab) => tab.key).join("\n");
  useEffect(() => {
    const strip = tabStripRef.current;
    if (!strip) return;
    return watchScroll(strip, "x", (metrics) => setTabFade(edgeFade(metrics)));
  }, [tabKeys]);

  // 捲到作用中 tab：作用中專案改變時；面板視窗只建一次、之後隱藏／顯示，再次開啟時
  // 面板取得焦點，也捲一次。直接設 scrollLeft 而非 scrollIntoView——後者會連帶捲動外層。
  useLayoutEffect(() => {
    const strip = tabStripRef.current;
    if (!strip || activeKey === null) return;
    const reveal = () => {
      const tab = strip.querySelector<HTMLElement>('[role="tab"][data-active="true"]');
      if (!tab) return;
      strip.scrollLeft = tabRevealScrollLeft({
        tabLeft: tab.offsetLeft,
        tabWidth: tab.offsetWidth,
        scrollLeft: strip.scrollLeft,
        viewportWidth: strip.clientWidth,
      });
    };
    reveal();
    window.addEventListener("focus", reveal);
    return () => window.removeEventListener("focus", reveal);
  }, [activeKey]);

  const activeTab = tabs.find((tab) => tab.key === activeKey);
  const activeRecovery =
    activeTab?.status === "restoring" || activeTab?.status === "error" ? activeTab : null;
  const activeStale =
    activeTab?.status === "offline" || activeTab?.status === "needs-reauth"
      ? activeTab
      : null;

  return (
    // macOS 補光層（tray-panel-content-priority「毛玻璃找回」）：HudWindow（NSVisualEffectView
    // 舊世代）無亮度自適應——卡內亮度改由不透明白卡錨定，根層只鋪側欄面色約六成（隨深淺
    // 模式），毛玻璃自外框與卡片縫隙透出約四成（九成時幾乎看不見）。六成是當初針對深色
    // 背景實測的濃度。圓角 13px 與 Rust 側 apply_vibrancy 半徑一致——補光畫出圓角外會在
    // 頂角留下方形殘料。Windows（tray-panel-windows design D1／D2）視窗不透明、無毛玻璃，
    // 根層鋪不透明側欄面色；圓角由 Windows 11 系統畫，面板不自繪。
    // h-screen 撐滿視窗：三段式版面的中段（flex-1）才有約束高度。
    // 卡與卡之間只以 gap 分隔，不畫分割線。
    <div
      data-testid="panel-root"
      className={cn(
        "flex h-screen flex-col gap-2 p-2 text-[13px] text-foreground",
        platform === "windows" ? "bg-sidebar" : "rounded-[13px] bg-sidebar/60",
      )}
    >
      {/* 固定頁首（spec 三段式版面）：tab 條卡常駐、不隨內容捲動。 */}
      <div data-testid="panel-header" className="shrink-0">
      <PanelCard testid="panel-project-card">
      <div className="flex">
      {/* 專案 tab 條（spec「面板樣式（macOS）」；tray-panel-restyle design D3）：首字母
          方塊與主視窗圖示列同形（取 projectTile 同一份常數，作用中實心主色）＋其下專案名，
          超寬橫向捲動且隱藏捲軸，有 tab 被遮住的一側以遮罩淡出；點 tab 沿用 open-project
          原地切換語意（不喚主視窗）。刻意用 div 非 button——面板中唯一可 tab 元素會吃到
          WebKit 的預設焦點（design D4）。relative 讓 tab 的 offsetLeft 以可捲區為基準
          （捲到作用中 tab 的計算依此）。 */}
      <div
        ref={tabStripRef}
        data-testid="panel-project-tabs"
        data-fade={tabFade}
        role="tablist"
        aria-label={t("app.projectRail")}
        style={edgeFadeStyle("x", tabFade)}
        className="relative flex min-w-0 flex-1 gap-1 overflow-x-auto p-1.5 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
      >
          {tabs.map((tab) => {
            // 識別與切換把手皆為 locator key（workspace-session 決策 6）。
            const active = tab.key === snapshot?.activeKey;
            return (
              <div
                key={tab.key}
                data-testid={`panel-project-${tab.key}`}
                data-active={active ? "true" : "false"}
                data-status={tab.status}
                role="tab"
                aria-selected={active}
                aria-label={`${tab.name}${
                  tab.key === pendingTabKey
                    ? `，${t("app.tabSwitching")}`
                    : tabStatusLabel(tab, t)
                      ? `，${tabStatusLabel(tab, t)}`
                      : ""
                }`}
                tabIndex={-1}
                onClick={() => onOpenProject(tab.key)}
                // 名稱在方塊下會截斷（max-w-14），title 帶出全名。
                title={tab.name}
                className={cn(TAB_ITEM, "w-16")}
              >
                {/* 選取只由方塊承載：作用中不論 ready／restoring／error 皆實心主色（與圖示列一致），
                    狀態交給其下狀態文字的語意色。 */}
                <span
                  aria-hidden
                  data-project-tile
                  className={cn(RAIL_SQUARE, active ? PROJECT_TILE_ACTIVE : PROJECT_TILE_IDLE)}
                >
                  {/* 切換中蓋過首字母：探測擋在翻頁前，此處是使用者唯一的「有在動」訊號。 */}
                  {tab.key === pendingTabKey ? (
                    <LoaderCircle
                      data-tab-pending="true"
                      className="h-4 w-4 animate-spin motion-reduce:animate-none"
                    />
                  ) : tab.source === "remote" ? (
                    <RemoteTabIcon tab={tab} />
                  ) : (
                    projectInitial(tab.name)
                  )}
                </span>
                <span
                  className={cn(
                    "max-w-14 truncate text-[11px] leading-none",
                    active ? "font-medium text-foreground" : "text-muted-foreground",
                  )}
                >
                  {tab.name}
                </span>
                {tab.source === "remote" && tab.status !== "ready" && (
                  <span className={cn("max-w-14 truncate text-[9px] leading-none", tabStatusTone(tab))}>
                    {tabStatusLabel(tab, t)}
                  </span>
                )}
              </div>
            );
          })}
      </div>
        {/* 快速加入專案（design D7）：固定在 tab 條卡右端、不隨 tab 捲走——專案再多也看得到；
            位置與主視窗圖示列「專案在前、＋在後」同序。與專案 tab 同構：虛線框「＋」（與
            圖示列同形）＋其下名稱——只留圖示在一排帶名稱的 tab 旁顯得突兀。文字不截斷：
            英文「Add project」比專案 tab 寬，這一項跟著文字長（min-w 與專案 tab 對齊）。
            外層 py-1.5 對齊可捲區內距，「＋」與首字母方塊同高。 */}
        <div className="shrink-0 py-1.5 pr-1.5">
          <div
            data-testid="panel-add-project"
            title={t("tray.addProject")}
            aria-label={t("tray.addProject")}
            onClick={onAddProject}
            className={cn(TAB_ITEM, "group min-w-16")}
          >
            <span
              aria-hidden
              data-project-tile
              className={cn(RAIL_SQUARE, ADD_TILE, "group-hover:bg-muted group-hover:text-foreground")}
            >
              <Plus className="h-4 w-4" />
            </span>
            <span className="whitespace-nowrap text-[11px] leading-none text-muted-foreground">
              {t("tray.addProject")}
            </span>
          </div>
        </div>
      </div>
      </PanelCard>
      </div>

      {/* 可捲中段（spec 三段式版面）：生命週期／討論分區或復原卡。 */}
      <PanelScrollArea>
      {activeRecovery ? (
        <PanelRecoveryCard
          tab={activeRecovery}
          onRetry={() => onRetryWorkspace(activeRecovery.key)}
          onOpenRecovery={() => onOpenRecovery(activeRecovery.key)}
          onOpenServerSettings={() => onOpenServerSettings(activeRecovery.connectionId)}
          onReauthenticate={() => onReauthenticate(activeRecovery.connectionId)}
        />
      ) : (
        <>
          {activeStale && (
            <div
              data-testid="panel-stale-status"
              role="status"
              // 琥珀淡底卡（design D5）：與專案欄的提示卡同語意，保留淡底而非白卡。
              className={cn(
                "flex min-h-8 items-center gap-2 rounded-xl border px-3 py-2 text-[11px]",
                SEMANTIC_SURFACE.warning,
              )}
            >
              <CloudOff className={cn("h-3.5 w-3.5 shrink-0", SEMANTIC_TONE.warning)} />
              <span className="font-medium">{tabStatusLabel(activeStale, t)}</span>
              <span className="ml-auto text-muted-foreground">{t("tray.recovery.stale")}</span>
              {activeStale.status === "needs-reauth" && (
                // 中性 outline：警示由整列的琥珀底承載，按鈕再塗琥珀只會兩片顏色互搶。
                <Button
                  variant="outline"
                  size="sm"
                  tabIndex={-1}
                  onClick={() => onReauthenticate(activeStale.connectionId)}
                >
                  {t("tray.recovery.reauthenticate")}
                </Button>
              )}
            </div>
          )}

      {/* 生命週期分區：三階段分區卡常駐（工作站常駐、衍生群組有料才現）——
          零筆階段呈標題＋計數 0 的空狀態卡（與討論分區同構，design D8），
          分區位置固定不隨資料增減跳動；原生選單的全空佔位文案不在此重現。
          生命週期在前、討論在後（與原生選單同序）：討論一多也不會把三個階段擠出第一屏。 */}
      {staged.map(({ stage, Icon, items }) => (
        <SectionCard
          key={stage}
          {...sectionCollapse(stage)}
          load={sectionLoad}
          icon={<Icon className={cn("h-3.5 w-3.5", STAGE_ICON[stage])} />}
          label={t(`stage.${stage}`)}
          count={items.length}
          countClassName={STAGE_BADGE[stage]}
          empty={items.length === 0}
        >
          <OverflowGroup
            moreLabel={moreLabel}
            collapseLabel={collapseLabel}
            rows={items.map((c) => (
              <ChangeRow
                key={c.name}
                c={c}
                stage={stage}
                onOpen={onOpenChange}
                onCopy={onCopy}
                copyLabel={t("tray.copyName")}
              />
            ))}
          />
        </SectionCard>
      ))}

      {/* 討論卡（spec「討論列表」）：標頭計數只算討論中；已轉出收於卡底「已轉出 N ›」
          收合列（有料才現），展開區落在收合列正上方。slug 為題、topic 為描述（識別錨點
          慣例，與看板討論卡一致）。四種狀態（載入中、失敗、有料、空）共用同一標題，只差
          計數與其下內容：載入中為佔位列、失敗為失敗提示（計數未知，不謊報 0）、空為
          計數 0 的空狀態卡（與非空同構，design D8）。 */}
      <SectionCard
        {...sectionCollapse("discussions")}
        load={sectionLoad}
        icon={<MessageSquareText className={cn("h-3.5 w-3.5", DISCUSSION_TONE.icon)} />}
        label={t("tray.discussionsHeader")}
        count={openDiscussions.length}
        countClassName={DISCUSSION_TONE.badge}
        empty={openDiscussions.length === 0 && promotedDiscussions.length === 0}
      >
        <OverflowGroup
          moreLabel={moreLabel}
          collapseLabel={collapseLabel}
          rows={openDiscussions.map(discussionRow)}
        />
        {promotedDiscussions.length > 0 && (
          <>
            {promotedExpanded && (
              <div data-testid="panel-promoted-rows" className="flex flex-col border-b border-border/70 bg-muted/40">
                <OverflowGroup
                  moreLabel={moreLabel}
                  collapseLabel={collapseLabel}
                  rows={promotedDiscussions.map(discussionRow)}
                />
              </div>
            )}
            {/* 計數在文字裡、中性配色，不給色相徽章（desktop-app「介面狀態語意色分層」）。 */}
            <div
              data-testid="panel-promoted-toggle"
              role="button"
              aria-expanded={promotedExpanded}
              tabIndex={-1}
              onClick={togglePromoted}
              onKeyDown={toggleOnKey(togglePromoted)}
              className={cn(rowClass, "text-xs font-medium text-muted-foreground")}
            >
              <span>{t("discussion.promotedBar").replace("{n}", String(promotedDiscussions.length))}</span>
              {/* 收合箭頭統一規則：收合 ›、展開時指向展開出來的內容——細列展開在本列上方，故朝上。 */}
              <ChevronRight
                className={cn("ml-auto h-3.5 w-3.5 shrink-0 transition-transform", promotedExpanded && "-rotate-90")}
              />
            </div>
          </>
        )}
      </SectionCard>

        </>
      )}
      </PanelScrollArea>

      {/* 固定頁尾（spec 三段式版面）：動作區卡常駐、不隨內容捲動。
          動作區單列：「開啟 Speclink」文字鈕佔滿左側（最常用、保留文字），右側專案設定、
          設定、結束三顆圖示鈕（與原生選單同序——專案層動作在前、app 層在後；圖示沿用
          側欄同款）；「設定」與「結束」之間以細直線分隔符隔開防誤按——只留加寬空白看起來像
          排版出錯。四顆皆 tabIndex=-1 退出 Tab 順序（同複製鈕）。 */}
      <div data-testid="panel-footer" className="shrink-0">
      <PanelCard testid="panel-actions-card">
      <div className="flex items-center gap-0.5 p-1">
        <button
          type="button"
          tabIndex={-1}
          onClick={onOpenApp}
          className={cn(actionButtonClass, "min-w-0 flex-1 gap-2 px-2 py-1.5 text-left")}
        >
          <ArrowUpRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
          <span className="truncate">{t("tray.open")}</span>
        </button>
        <ActionIconButton label={t("app.navProjectSettings")} Icon={SlidersHorizontal} onClick={onOpenProjectSettings} />
        <ActionIconButton label={t("tray.settings")} Icon={Settings} onClick={onOpenSettings} />
        <span aria-hidden data-testid="panel-action-separator" className="mx-1 h-4 w-px shrink-0 bg-border" />
        <ActionIconButton label={t("tray.quit")} Icon={Power} onClick={onQuit} />
      </div>
      </PanelCard>
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
        <div className="truncate text-[11px] text-muted-foreground">{d.topic}</div>
      </div>
      {/* 列尾常駐複製鈕：tabIndex=-1 退出 tab 順序（design D4）——面板成 key window 後 WebKit
          會把焦點給第一個可 tab 元素，複製鈕不退出就吃到焦點框。變更列同理。 */}
      <CopyButton value={d.slug} label={copyLabel} onCopy={onCopy} tabIndex={-1} />
    </div>
  );
}

/**
 * 變更列的品質站章（spec tray-status-menu「面板變更列的品質站章」；design D7）：
 * 站章直接影響收尾動作（未結工單擋封存、降級章提示留意），所以進 tray；建立者
 * 頭像、來源討論標記、restale 與 metaError 是閱讀脈絡，留在看板。
 *
 * 圖示、色調與 tooltip 詞條全部取自 packages/ui 匯出的兩張站別樣式表——面板不
 * 另建第二份對照，否則卡片改了色、tray 不會跟著改。tooltip 以 `title` 承載：
 * 面板是無焦點的一瞥介面，掛 Radix Tooltip 會多一層 portal 與焦點管理。
 */
function StationBadges({ c }: { c: ChangeItem }) {
  const { t } = useI18n();
  const review = c.reviewStatus && c.reviewStatus !== "none" ? c.reviewStatus : null;
  const verify = c.verifyStatus && c.verifyStatus !== "none" ? c.verifyStatus : null;
  if (!review && !verify) return null;
  return (
    <>
      {review && (
        <span
          aria-label={t(REVIEW_LABEL_KEY[review])}
          title={t(REVIEW_LABEL_KEY[review])}
          className={cn("shrink-0", REVIEW_TONE[review])}
        >
          {REVIEW_ICON[review]}
        </span>
      )}
      {verify && (
        <span
          aria-label={t(VERIFY_LABEL_KEY[verify])}
          title={t(VERIFY_LABEL_KEY[verify])}
          className={cn("shrink-0", VERIFY_TONE[verify])}
        >
          {VERIFY_ICON[verify]}
        </span>
      )}
    </>
  );
}

function ChangeRow({
  c,
  stage,
  onOpen,
  onCopy,
  copyLabel,
}: {
  c: ChangeItem;
  /** 所屬生命週期階段——進度條填色取 STAGE_BAR 深淺階梯（design D3）。 */
  stage: Stage;
  onOpen: (name: string) => void;
  onCopy: (text: string) => void;
  copyLabel: string;
}) {
  const { t } = useI18n();
  const pct = c.totalTasks > 0 ? Math.round((c.completedTasks / c.totalTasks) * 100) : 0;
  // 波次與阻擋（spec tray-status-menu「變更列的波次標示與順序同源」；design D7）：
  // 列首波次數字與看板卡片共用 card.wave 詞條；被擋列整列變淡、title 列前置。
  // 判定歸 planWave／planBlockedBy 單一入口，缺 wave（remote、成環）列與從前一致。
  const wave = planWave(c);
  const blockedBy = planBlockedBy(c);
  const blockedTitle = blockedBy.length > 0 ? planBlockedLabel(blockedBy, t) : undefined;
  return (
    <div
      data-testid={`panel-change-${c.name}`}
      onClick={() => onOpen(c.name)}
      // 淡化落在列的內容（直接子節點）上：列本身降透明度會連底部分隔線一起變淡。
      className={cn(rowClass, blockedBy.length > 0 && "[&>*]:opacity-60")}
      title={blockedTitle}
    >
      <div className="min-w-0 flex-1">
        <div className="flex items-baseline gap-2">
          {wave !== null && (
            <span
              aria-label={planWaveLabel(wave, t)}
              title={planWaveLabel(wave, t)}
              className="shrink-0 text-[11px] font-semibold tabular-nums text-primary"
            >
              {wave}
            </span>
          )}
          <span className="truncate">{c.name}</span>
          <StationBadges c={c} />
          {/* worktree 標記：掛著＝工作正於副本進行中，與站章同屬影響收尾動作
              的狀態（merge 前不能封存），故進 tray；色調與 tooltip 詞條沿看板
              卡片同款（SEMANTIC_TONE.inProgress＋card.worktree），不另建對照。 */}
          {c.worktree && (
            <span
              aria-label={t("card.worktree")}
              title={t("card.worktreeTitle").replace("{branch}", c.worktree.branch)}
              className={cn("shrink-0", SEMANTIC_TONE.inProgress)}
            >
              <GitBranch className="h-3.5 w-3.5" />
            </span>
          )}
          {c.totalTasks > 0 && (
            <span className="ml-auto shrink-0 text-[11px] tabular-nums text-muted-foreground">
              {`${c.completedTasks}/${c.totalTasks}`}
            </span>
          )}
        </div>
        {c.totalTasks > 0 && (
          <div className="mt-1 h-1 overflow-hidden rounded-full bg-foreground/10">
            <div
              data-testid="panel-progress-fill"
              className={cn("h-full rounded-full", STAGE_BAR[stage])}
              style={{ width: `${pct}%` }}
            />
          </div>
        )}
      </div>
      <CopyButton value={c.name} label={copyLabel} onCopy={onCopy} tabIndex={-1} />
    </div>
  );
}

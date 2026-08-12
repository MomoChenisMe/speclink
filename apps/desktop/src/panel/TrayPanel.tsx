// 面板樣式呈現層（spec「面板樣式（macOS）」；design D5）：主視窗推送的
// TraySnapshot 薄渲染——分區內容與原生選單同源（同一 store 快照），不自建
// 資料查詢路徑。原生質感（vibrancy、不搶焦點、失焦收合、高度自適應）由
// Rust 側面板視窗與入口層承擔，此元件只負責結構與回呼；複製鈕常駐列尾、
// 分區圖示沿用看板同款（KanbanBoard 的 stage 圖示）。
import { useEffect, useRef, useState, type ReactNode, type RefObject } from "react";
import {
  changeStage,
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
  useI18n,
  type ChangeItem,
  type Stage,
} from "@speclink/ui";
import {
  AlertTriangle,
  ArrowUpRight,
  Check,
  CircleCheckBig,
  Cloud,
  CloudOff,
  Copy,
  Folder,
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

import { OVERFLOW_LIMIT, type TraySnapshot, type TrayTabSnapshot } from "../tray";

/** 分區圖示與看板欄位同款（KanbanBoard）：提案中／進行中／已就緒。 */
const STAGE_ICONS: Record<Stage, LucideIcon> = {
  proposed: Lightbulb,
  "in-progress": Hammer,
  ready: CircleCheckBig,
};

export interface TrayPanelProps {
  /** 主視窗推送的快照；null＝尚未收到（lazy 建窗後的短暫空窗）。 */
  snapshot: TraySnapshot | null;
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

/** 列尾常駐複製鈕：stopPropagation 使複製不觸發列本體的開啟；
    點擊後短暫轉勾號回饋（1.2 秒復原——看板 ChangeList 的 copied 同模式）。 */
function CopyButton({ label, text, onCopy }: { label: string; text: string; onCopy: (t: string) => void }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      type="button"
      // 退出 tab 順序（design D4）：面板成 key window 後 WebKit 會把焦點給
      // 第一個可 tab 元素——複製鈕是面板唯一 button，不退出就吃到焦點框。
      tabIndex={-1}
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

function SectionHeader({
  icon: Icon,
  label,
  iconCls,
  count,
  badgeCls,
  countUnknown,
}: {
  icon: LucideIcon;
  label: string;
  /** 分區圖示色（生命週期依 STAGE_ICON 階梯、討論／已轉出分區中性）。 */
  iconCls: string;
  /** 分區項目計數（design D8）：徽章與看板欄計數同語彙。 */
  count: number;
  /** 計數徽章配色：生命週期取 STAGE_BADGE[stage]、討論／已轉出分區中性。 */
  badgeCls: string;
  /** 計數未知（首訪載入中或載入失敗）：顯示 0 會謊報空——徽章整個不出。 */
  countUnknown?: boolean;
}) {
  return (
    <div className="flex items-center gap-1.5 px-2 pt-1.5 pb-1 text-xs font-semibold text-muted-foreground">
      <Icon className={cn("h-3.5 w-3.5", iconCls)} />
      {label}
      {!countUnknown && (
        <span
          data-testid="panel-section-count"
          className={cn(
            "ml-auto inline-flex h-5 min-w-5 items-center justify-center rounded-full px-1.5 text-[11px] font-semibold tabular-nums",
            badgeCls,
          )}
        >
          {count}
        </span>
      )}
    </div>
  );
}

/** 分區卡片容器（spec「面板樣式（macOS）」；design D2）：半透明圓角卡疊在
    vibrancy 上——底色用主題 token 的低透明度（非寫死色值），毛玻璃可透出。 */
function SectionCard({
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
    <section data-testid={testid} className={cn("flex flex-col rounded-lg bg-foreground/5 p-1", className)}>
      {children}
    </section>
  );
}

/** 空狀態分區卡附加樣式（design D8）：最小高度＋內容垂直置中——生命週期
    零筆階段卡與討論零筆卡共用，維持空狀態同構。 */
const emptyCardClass = "min-h-12 justify-center";

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

/** 區塊分割線（spec「面板樣式（macOS）」區塊順序與分割線）：低透明度細線
    疊在毛玻璃上，僅出現於 tab 條後與內容區塊之間——區塊內部（分區卡之間）
    不加線；以 div 而非 hr 實作，面板卡片化後不使用原生分隔線元素。 */
function Divider() {
  return <div data-testid="panel-divider" aria-hidden className="mx-1 h-px shrink-0 bg-foreground/10" />;
}

/** 捲動停止後指示條淡出的延遲（ms）——比照 macOS overlay 捲軸的滯留時間。 */
const SCROLL_IDLE_MS = 800;

/**
 * 中段捲動指示條（spec「面板樣式（macOS）」：捲動時浮現、停止後自動隱去、
 * 不佔版面寬度）。自繪而非用原生捲軸——index.css 的全域 `::-webkit-scrollbar`
 * 會把 WebKit 切成自訂 legacy 捲軸（常駐且佔寬），系統「總是顯示捲軸」偏好
 * 亦然；兩者皆使原生捲軸不可能淡出。內容未溢出時不渲染。
 */
function ScrollIndicator({ targetRef }: { targetRef: RefObject<HTMLDivElement | null> }) {
  const [thumb, setThumb] = useState<{ top: number; height: number } | null>(null);
  const [active, setActive] = useState(false);
  useEffect(() => {
    const el = targetRef.current;
    if (!el) return;
    let timer: ReturnType<typeof setTimeout> | null = null;
    const measure = () => {
      const { scrollTop, scrollHeight, clientHeight } = el;
      if (scrollHeight <= clientHeight) {
        setThumb(null);
        return;
      }
      const height = Math.max(24, (clientHeight / scrollHeight) * clientHeight);
      const travel = clientHeight - height;
      const top = travel * (scrollTop / (scrollHeight - clientHeight));
      setThumb({ top, height });
    };
    const onScroll = () => {
      measure();
      setActive(true);
      if (timer !== null) clearTimeout(timer);
      timer = setTimeout(() => setActive(false), SCROLL_IDLE_MS);
    };
    measure();
    el.addEventListener("scroll", onScroll, { passive: true });
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => {
      el.removeEventListener("scroll", onScroll);
      observer.disconnect();
      if (timer !== null) clearTimeout(timer);
    };
  }, [targetRef]);
  if (!thumb) return null;
  return (
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
 * 分頁狀態文字的語意色（spec「介面狀態語意色分層」）：選取由分頁外框表達，
 * 狀態則由列內這行文字承載——還原中＝藍、離線／需重新登入＝琥珀警示、
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
    return <LoaderCircle className="h-3.5 w-3.5 animate-spin motion-reduce:animate-none" />;
  }
  if (tab.status === "offline") return <CloudOff className="h-3.5 w-3.5" />;
  if (tab.status === "needs-reauth" || tab.failureKind === "needs-reauth") {
    return <LogIn className="h-3.5 w-3.5" />;
  }
  if (tab.status === "error") return <AlertTriangle className="h-3.5 w-3.5" />;
  return <Cloud className="h-3.5 w-3.5" />;
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
      // 卡底維持 bg-foreground/5（毛玻璃透出），只借語意色的邊框——故 surface 之後
      // 再蓋回底色。
      className={cn("rounded-xl border p-3", SEMANTIC_SURFACE[tone], "bg-foreground/5")}
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

      <dl className="mt-3 grid gap-2 rounded-lg bg-background/45 p-2 text-[11px]">
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
            <button
              type="button"
              tabIndex={-1}
              onClick={onReauthenticate}
              className="col-span-2 flex min-h-8 items-center justify-center gap-1.5 rounded-md bg-primary px-2 font-medium text-primary-foreground hover:bg-primary/90"
            >
              <LogIn className="h-3.5 w-3.5" /> {t("tray.recovery.reauthenticate")}
            </button>
          ) : (
            <button
              type="button"
              tabIndex={-1}
              onClick={onRetry}
              className="col-span-2 flex min-h-8 items-center justify-center gap-1.5 rounded-md bg-primary px-2 font-medium text-primary-foreground hover:bg-primary/90"
            >
              <RefreshCw className="h-3.5 w-3.5" /> {t("tray.recovery.retry")}
            </button>
          ))}
        <button
          type="button"
          tabIndex={-1}
          onClick={onOpenRecovery}
          className="flex min-h-8 items-center justify-center gap-1 rounded-md bg-foreground/8 px-2 text-[11px] hover:bg-foreground/15"
        >
          <ArrowUpRight className="h-3 w-3" /> {t("tray.recovery.open")}
        </button>
        <button
          type="button"
          tabIndex={-1}
          onClick={onOpenServerSettings}
          className="flex min-h-8 items-center justify-center gap-1 rounded-md bg-foreground/8 px-2 text-[11px] hover:bg-foreground/15"
        >
          <Settings className="h-3 w-3" /> {t("tray.recovery.settings")}
        </button>
      </div>
    </section>
  );
}

export function TrayPanel({
  snapshot,
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
  const scrollRef = useRef<HTMLDivElement>(null);
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
  }));
  const pendingTabKey = snapshot?.pendingTabKey ?? null;
  // 首訪載入中：分區以佔位列呈現。條件整條由 store 導出（design D3）——面板
  // 不自行組合旗標，與主視窗看板恆為同一個答案。快照未到（null）同樣算載入
  // 中——面板剛開、主視窗尚未推第一份快照，此時畫空分區與畫佔位列是同一個問題。
  const loading = !snapshot || snapshot.workspaceLoading;
  // 首訪載入失敗的終態：載入已結束（故不與骨架並存），但沒有真值可畫——
  // 顯示失敗提示而非空態文案。
  const loadFailed = !loading && snapshot.workspaceLoadFailed;
  const activeTab = tabs.find((tab) => tab.key === snapshot?.activeKey);
  const activeRecovery =
    activeTab?.status === "restoring" || activeTab?.status === "error" ? activeTab : null;
  const activeStale =
    activeTab?.status === "offline" || activeTab?.status === "needs-reauth"
      ? activeTab
      : null;

  return (
    // 極淡主色漸層 wash（design D3）：低透明度不遮蔽 vibrancy。圓角 13px 與
    // Rust 側 apply_vibrancy 半徑一致——wash 畫出圓角外會在頂角留下方形殘料。
    // h-screen 撐滿視窗：三段式版面的中段（flex-1）才有約束高度，wash 隨之固定於視窗。
    // bg-background/60 補光基底：HudWindow（NSVisualEffectView 舊世代）無亮度自適應、
    // 深色背景會把面板整片壓暗——以主題背景色半透明層錨定亮度（隨深淺模式自動），
    // 毛玻璃自剩餘透明度透出；濃度為真實視窗調參值。
    <div
      data-testid="panel-root"
      className="flex h-screen flex-col gap-1.5 rounded-[13px] bg-background/60 bg-linear-to-b from-foreground/5 to-transparent p-2 text-[13px] text-foreground"
    >
      {/* 固定頁首（spec 三段式版面）：tab 條＋其下分割線常駐、不隨內容捲動。 */}
      <div data-testid="panel-header" className="flex shrink-0 flex-col gap-1.5">
      {/* 專案 tab 條（spec「面板樣式（macOS）」；design D1）：首字母 avatar＋
          專案名、作用中實心主色卡、超寬橫向捲動且隱藏捲軸；點 tab 沿用
          open-project 原地切換語意（不喚主視窗）。刻意用 div 非 button——
          面板中唯一可 tab 元素會吃到 WebKit 的預設焦點（design D4）。
          條常駐（零專案時仍有尾端「加入專案」項，design D7）。 */}
      <div
        data-testid="panel-project-tabs"
        role="tablist"
        aria-label={t("app.workspaceTabs")}
        className="flex gap-1 overflow-x-auto p-0.5 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
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
                className={cn(
                  "flex min-h-14 shrink-0 flex-col items-center justify-center gap-1 rounded-lg border px-2.5 py-1.5",
                  // 選取一律由主色外框表達；狀態交給列內狀態文字的語意色。
                  // （舊版非 ready 的作用中分頁塗琥珀，選取與警示混成同一片顏色。）
                  active && tab.status === "ready"
                    ? "border-primary bg-primary text-primary-foreground"
                    : active
                      ? "border-primary/60 text-foreground"
                      : "border-transparent hover:bg-primary/10",
                )}
              >
                <span
                  aria-hidden
                  className={cn(
                    "flex h-6 w-6 items-center justify-center rounded-md text-[13px] font-semibold",
                    active ? "bg-primary-foreground/20" : "bg-primary/10 text-primary",
                  )}
                >
                  {/* 切換中蓋過 avatar：探測擋在翻頁前，此處是使用者唯一的「有在動」訊號。 */}
                  {tab.key === pendingTabKey ? (
                    <LoaderCircle
                      data-tab-pending="true"
                      className="h-3.5 w-3.5 animate-spin motion-reduce:animate-none"
                    />
                  ) : tab.source === "remote" ? (
                    <RemoteTabIcon tab={tab} />
                  ) : (
                    tab.name.charAt(0).toUpperCase()
                  )}
                </span>
                <span className="max-w-24 truncate text-[11px] leading-none">{tab.name}</span>
                {tab.source === "remote" && tab.status !== "ready" && (
                  <span className={cn("max-w-24 truncate text-[9px] leading-none", tabStatusTone(tab))}>
                    {tabStatusLabel(tab, t)}
                  </span>
                )}
              </div>
            );
          })}
        {/* 尾端快速加入專案（design D7）：開資料夾選擇器，選定即加入並切換。 */}
        <div
          data-testid="panel-add-project"
          title={t("tray.addProject")}
          aria-label={t("tray.addProject")}
          onClick={onAddProject}
          className="flex shrink-0 flex-col items-center justify-center gap-1 rounded-lg px-2.5 py-1.5 text-muted-foreground hover:bg-primary/10 hover:text-primary"
        >
          <span className="flex h-6 w-6 items-center justify-center rounded-md border border-dashed border-muted-foreground/40">
            <Plus className="h-3.5 w-3.5" />
          </span>
          <span className="text-[11px] leading-none">{t("tray.addProject")}</span>
        </div>
      </div>

      {/* 區塊順序（spec「面板樣式（macOS）」區塊順序與分割線）：tab 條之下
          依序為討論區塊（討論常駐＋已轉出有料才現）、生命週期區塊、動作區塊，
          塊間各一條分割線——首尾兩條隨頁首頁尾常駐，中段內一條隨內容捲動。 */}
      <Divider />
      </div>

      {/* 可捲中段（spec 三段式版面）：討論／生命週期分區或復原卡；捲動面自 body
          收斂至此容器，overscroll-none 隨遷防 rubber-band。原生捲軸隱藏、改由
          ScrollIndicator 自繪（見其註解：原生捲軸在此環境無法淡出且會佔寬）；
          指示條為捲動容器的兄弟節點，才不會隨內容一起捲走。
          內層 wrapper 保持自然高度——中段容器本身被 flex 約束，內容增減不改其
          外框，入口層的 ResizeObserver 觀察的是這層 wrapper（高度量測依據）。 */}
      <div className="relative flex min-h-0 flex-1 flex-col">
      <div
        ref={scrollRef}
        data-testid="panel-scroll"
        className="min-h-0 flex-1 overflow-y-auto overscroll-none [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
      >
      <div data-testid="panel-scroll-content" className="flex flex-col gap-1.5">
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
              className={cn(
                "flex min-h-8 items-center gap-2 rounded-lg border px-2.5 py-1.5 text-[11px]",
                SEMANTIC_SURFACE.warning,
              )}
            >
              <CloudOff className={cn("h-3.5 w-3.5 shrink-0", SEMANTIC_TONE.warning)} />
              <span className="font-medium">{tabStatusLabel(activeStale, t)}</span>
              <span className="ml-auto text-muted-foreground">{t("tray.recovery.stale")}</span>
              {activeStale.status === "needs-reauth" && (
                <button
                  type="button"
                  tabIndex={-1}
                  onClick={() => onReauthenticate(activeStale.connectionId)}
                  // 中性 outline：警示由整列的琥珀底承載，按鈕再塗琥珀只會兩片顏色互搶。
                  className="rounded border border-border bg-background/60 px-2 py-1 font-medium hover:bg-foreground/10"
                >
                  {t("tray.recovery.reauthenticate")}
                </button>
              )}
            </div>
          )}

      {/* 討論區分流（spec「討論列表」）：「討論」列討論中、「已轉出」列已轉出；
          slug 為題、topic 為描述（識別錨點慣例，與看板討論卡一致） */}
      {loading ? (
        /* 首訪載入中（design D5）：標題照常、內容為佔位列——與計數 0 的空狀態卡可區分。 */
        <SectionCard testid="panel-section-discussions">
          <SectionHeader
            icon={MessageSquareText}
            iconCls="text-muted-foreground/70"
            label={t("tray.discussionsHeader")}
            count={0}
            badgeCls="bg-muted text-muted-foreground"
            countUnknown
          />
          <SectionSkeleton />
        </SectionCard>
      ) : loadFailed ? (
        /* 首訪載入失敗：計數未知（不謊報 0），內容為失敗提示而非空態文案。 */
        <SectionCard testid="panel-section-discussions">
          <SectionHeader
            icon={MessageSquareText}
            iconCls="text-muted-foreground/70"
            label={t("tray.discussionsHeader")}
            count={0}
            badgeCls="bg-muted text-muted-foreground"
            countUnknown
          />
          <SectionLoadFailed label={t("tray.loadFailed")} />
        </SectionCard>
      ) : openDiscussions.length > 0 ? (
        <SectionCard testid="panel-section-discussions">
          <SectionHeader
            icon={MessageSquareText}
            iconCls="text-muted-foreground/70"
            label={t("tray.discussionsHeader")}
            count={openDiscussions.length}
            badgeCls="bg-muted text-muted-foreground"
          />
          <OverflowGroup
            moreLabel={moreLabel}
            collapseLabel={collapseLabel}
            rows={openDiscussions.map((d) => (
              <DiscussionRow key={d.slug} d={d} onOpen={onOpenDiscussion} onCopy={onCopy} copyLabel={t("tray.copySlug")} />
            ))}
          />
        </SectionCard>
      ) : (
        /* 空狀態與非空同構（design D8）：標題＋計數 0、最小高度垂直置中。 */
        <SectionCard testid="panel-section-discussions" className={emptyCardClass}>
          <SectionHeader
            icon={MessageSquareText}
            iconCls="text-muted-foreground/70"
            label={t("tray.discussionsHeader")}
            count={0}
            badgeCls="bg-muted text-muted-foreground"
          />
        </SectionCard>
      )}
      {!loading && promotedDiscussions.length > 0 && (
        <SectionCard testid="panel-section-promoted">
          <SectionHeader
            icon={ArrowUpRight}
            iconCls="text-muted-foreground/70"
            label={t("tray.promotedHeader")}
            count={promotedDiscussions.length}
            badgeCls="bg-muted text-muted-foreground"
          />
          <OverflowGroup
            moreLabel={moreLabel}
            collapseLabel={collapseLabel}
            rows={promotedDiscussions.map((d) => (
              <DiscussionRow key={d.slug} d={d} onOpen={onOpenDiscussion} onCopy={onCopy} copyLabel={t("tray.copySlug")} />
            ))}
          />
        </SectionCard>
      )}

      <Divider />

      {/* 生命週期分區：三階段分區卡常駐（工作站常駐、衍生群組有料才現）——
          零筆階段呈標題＋計數 0 的空狀態卡（與討論分區同構，design D8），
          分區位置固定不隨資料增減跳動；原生選單的全空佔位文案不在此重現。 */}
      {staged.map(({ stage, items }) => (
        <SectionCard
          key={stage}
          testid={`panel-section-${stage}`}
          className={!loading && !loadFailed && items.length === 0 ? emptyCardClass : undefined}
        >
          <SectionHeader
            icon={STAGE_ICONS[stage]}
            iconCls={STAGE_ICON[stage]}
            label={t(`stage.${stage}`)}
            count={items.length}
            badgeCls={STAGE_BADGE[stage]}
            countUnknown={loading || loadFailed}
          />
          {loading ? (
            <SectionSkeleton />
          ) : loadFailed ? (
            <SectionLoadFailed label={t("tray.loadFailed")} />
          ) : (
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
          )}
        </SectionCard>
      ))}

        </>
      )}
      </div>
      </div>
      <ScrollIndicator targetRef={scrollRef} />
      </div>

      {/* 固定頁尾（spec 三段式版面）：分割線＋動作區常駐、不隨內容捲動。
          動作區：開啟主視窗、專案設定、設定、結束（spec 動作區塊四項；與原生選單同序——
          專案層動作在前、app 層在後，呼應主視窗側欄層次；圖示沿用側欄同款） */}
      <div data-testid="panel-footer" className="flex shrink-0 flex-col gap-1.5">
      <Divider />
      <div onClick={onOpenApp} className={rowClass}>
        <ArrowUpRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground group-hover:text-primary-foreground" />
        {t("tray.open")}
      </div>
      <div onClick={onOpenProjectSettings} className={rowClass}>
        <SlidersHorizontal className="h-3.5 w-3.5 shrink-0 text-muted-foreground group-hover:text-primary-foreground" />
        {t("app.navProjectSettings")}
      </div>
      <div onClick={onOpenSettings} className={rowClass}>
        <Settings className="h-3.5 w-3.5 shrink-0 text-muted-foreground group-hover:text-primary-foreground" />
        {t("tray.settings")}
      </div>
      <div onClick={onQuit} className={rowClass}>
        <Power className="h-3.5 w-3.5 shrink-0 text-muted-foreground group-hover:text-primary-foreground" />
        {t("tray.quit")}
      </div>
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

/**
 * 變更列的品質站章（spec tray-status-menu「面板變更列的品質站章」；design D7）：
 * 站章直接影響收尾動作（未結工單擋封存、降級章提示留意），所以進 tray；建立者
 * 頭像、來源討論標記、restale 與 metaError 是閱讀脈絡，留在看板。
 *
 * 圖示、色調與 tooltip 詞條全部取自 packages/ui 匯出的兩張站別樣式表——面板不
 * 另建第二份對照，否則卡片改了色、tray 不會跟著改。tooltip 以 `title` 承載：
 * 面板是無焦點的一瞥介面，掛 Radix Tooltip 會多一層 portal 與焦點管理。
 */
/** 列 hover 反白（主色底）時站章隨列改前景色（spec tray-status-menu「面板變更列的
    品質站章」）：紫章壓在主色底上對比不足，反白期間與同列名稱、任務數、進度條一致，
    站別改由圖示形狀承辨；指標離開即回復共用色調表。 */
const STAMP_HOVER = "group-hover:text-primary-foreground";

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
          className={cn("shrink-0", REVIEW_TONE[review], STAMP_HOVER)}
        >
          {REVIEW_ICON[review]}
        </span>
      )}
      {verify && (
        <span
          aria-label={t(VERIFY_LABEL_KEY[verify])}
          title={t(VERIFY_LABEL_KEY[verify])}
          className={cn("shrink-0", VERIFY_TONE[verify], STAMP_HOVER)}
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
  const pct = c.totalTasks > 0 ? Math.round((c.completedTasks / c.totalTasks) * 100) : 0;
  return (
    <div data-testid={`panel-change-${c.name}`} onClick={() => onOpen(c.name)} className={rowClass}>
      <div className="min-w-0 flex-1">
        <div className="flex items-baseline gap-2">
          <span className="truncate">{c.name}</span>
          <StationBadges c={c} />
          {c.totalTasks > 0 && (
            <span className="ml-auto shrink-0 text-[11px] tabular-nums text-muted-foreground group-hover:text-primary-foreground/80">
              {`${c.completedTasks}/${c.totalTasks}`}
            </span>
          )}
        </div>
        {c.totalTasks > 0 && (
          <div className="mt-1 h-1 overflow-hidden rounded-full bg-foreground/10 group-hover:bg-primary-foreground/25">
            <div
              data-testid="panel-progress-fill"
              className={cn(
                "h-full rounded-full group-hover:bg-primary-foreground",
                STAGE_BAR[stage],
              )}
              style={{ width: `${pct}%` }}
            />
          </div>
        )}
      </div>
      <CopyButton label={copyLabel} text={c.name} onCopy={onCopy} />
    </div>
  );
}

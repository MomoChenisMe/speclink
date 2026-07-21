// 面板樣式呈現層（spec「面板樣式（macOS）」；design D5）：主視窗推送的
// TraySnapshot 薄渲染——分區內容與原生選單同源（同一 store 快照），不自建
// 資料查詢路徑。原生質感（vibrancy、不搶焦點、失焦收合、高度自適應）由
// Rust 側面板視窗與入口層承擔，此元件只負責結構與回呼；複製鈕常駐列尾、
// 分區圖示沿用看板同款（KanbanBoard 的 stage 圖示）。
import { useState, type ReactNode } from "react";
import {
  changeStage,
  STAGE_BADGE,
  STAGE_BAR,
  STAGE_ICON,
  STAGES,
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
}: {
  icon: LucideIcon;
  label: string;
  /** 分區圖示主色（生命週期依 STAGE_ICON 階梯、討論分區 text-primary）。 */
  iconCls: string;
  /** 分區項目計數（design D8）：徽章與看板欄計數同語彙。 */
  count: number;
  /** 計數徽章配色：生命週期取 STAGE_BADGE[stage]、討論分區取看板討論欄同款。 */
  badgeCls: string;
}) {
  return (
    <div className="flex items-center gap-1.5 px-2 pt-1.5 pb-1 text-xs font-semibold text-muted-foreground">
      <Icon className={cn("h-3.5 w-3.5", iconCls)} />
      {label}
      <span
        data-testid="panel-section-count"
        className={cn(
          "ml-auto inline-flex h-5 min-w-5 items-center justify-center rounded-full px-1.5 text-[11px] font-semibold tabular-nums",
          badgeCls,
        )}
      >
        {count}
      </span>
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

/** 區塊分割線（spec「面板樣式（macOS）」區塊順序與分割線）：低透明度細線
    疊在毛玻璃上，僅出現於 tab 條後與內容區塊之間——區塊內部（分區卡之間）
    不加線；以 div 而非 hr 實作，面板卡片化後不使用原生分隔線元素。 */
function Divider() {
  return <div data-testid="panel-divider" aria-hidden className="mx-1 h-px shrink-0 bg-foreground/10" />;
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
  return (
    <section
      data-testid="panel-recovery-card"
      data-status={tab.status}
      role="status"
      aria-live="polite"
      className="rounded-xl border border-amber-500/30 bg-foreground/5 p-3"
    >
      <div className="flex items-start gap-2.5">
        <span className="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-amber-500/15 text-amber-700 dark:text-amber-300">
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
    <div
      data-testid="panel-root"
      className="flex flex-col gap-1.5 rounded-[13px] bg-linear-to-b from-primary/5 to-transparent p-2 text-[13px] text-foreground"
    >
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
                aria-label={`${tab.name}${tabStatusLabel(tab, t) ? `，${tabStatusLabel(tab, t)}` : ""}`}
                tabIndex={-1}
                onClick={() => onOpenProject(tab.key)}
                className={cn(
                  "flex min-h-14 shrink-0 flex-col items-center justify-center gap-1 rounded-lg border px-2.5 py-1.5",
                  active && tab.status === "ready"
                    ? "border-primary bg-primary text-primary-foreground"
                    : active
                      ? "border-amber-500/60 bg-amber-500/15 text-foreground"
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
                  {tab.source === "remote" ? (
                    <RemoteTabIcon tab={tab} />
                  ) : (
                    tab.name.charAt(0).toUpperCase()
                  )}
                </span>
                <span className="max-w-24 truncate text-[11px] leading-none">{tab.name}</span>
                {tab.source === "remote" && tab.status !== "ready" && (
                  <span className="max-w-24 truncate text-[9px] leading-none text-muted-foreground">
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
          塊間各一條分割線。 */}
      <Divider />

      {activeRecovery ? (
        <>
          <PanelRecoveryCard
            tab={activeRecovery}
            onRetry={() => onRetryWorkspace(activeRecovery.key)}
            onOpenRecovery={() => onOpenRecovery(activeRecovery.key)}
            onOpenServerSettings={() => onOpenServerSettings(activeRecovery.connectionId)}
            onReauthenticate={() => onReauthenticate(activeRecovery.connectionId)}
          />
          <Divider />
        </>
      ) : (
        <>
          {activeStale && (
            <div
              data-testid="panel-stale-status"
              role="status"
              className="flex min-h-8 items-center gap-2 rounded-lg border border-amber-500/30 bg-amber-500/10 px-2.5 py-1.5 text-[11px]"
            >
              <CloudOff className="h-3.5 w-3.5 shrink-0 text-amber-700 dark:text-amber-300" />
              <span className="font-medium">{tabStatusLabel(activeStale, t)}</span>
              <span className="ml-auto text-muted-foreground">{t("tray.recovery.stale")}</span>
              {activeStale.status === "needs-reauth" && (
                <button
                  type="button"
                  tabIndex={-1}
                  onClick={() => onReauthenticate(activeStale.connectionId)}
                  className="rounded bg-amber-500/15 px-2 py-1 font-medium hover:bg-amber-500/25"
                >
                  {t("tray.recovery.reauthenticate")}
                </button>
              )}
            </div>
          )}

      {/* 討論區分流（spec「討論列表」）：「討論」列討論中、「已轉出」列已轉出；
          slug 為題、topic 為描述（識別錨點慣例，與看板討論卡一致） */}
      {openDiscussions.length > 0 ? (
        <SectionCard testid="panel-section-discussions">
          <SectionHeader
            icon={MessageSquareText}
            iconCls="text-primary"
            label={t("tray.discussionsHeader")}
            count={openDiscussions.length}
            badgeCls={STAGE_BADGE.proposed}
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
            iconCls="text-primary"
            label={t("tray.discussionsHeader")}
            count={0}
            badgeCls={STAGE_BADGE.proposed}
          />
        </SectionCard>
      )}
      {promotedDiscussions.length > 0 && (
        <SectionCard testid="panel-section-promoted">
          <SectionHeader
            icon={ArrowUpRight}
            iconCls="text-primary"
            label={t("tray.promotedHeader")}
            count={promotedDiscussions.length}
            badgeCls={STAGE_BADGE.proposed}
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
          className={items.length === 0 ? emptyCardClass : undefined}
        >
          <SectionHeader
            icon={STAGE_ICONS[stage]}
            iconCls={STAGE_ICON[stage]}
            label={t(`stage.${stage}`)}
            count={items.length}
            badgeCls={STAGE_BADGE[stage]}
          />
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

      <Divider />
        </>
      )}

      {/* 動作區：開啟主視窗、設定、結束（spec 動作區塊三項；與原生選單同序） */}
      <div onClick={onOpenApp} className={rowClass}>
        <ArrowUpRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground group-hover:text-primary-foreground" />
        {t("tray.open")}
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

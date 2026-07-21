// 專案分頁列（design D10，UI 形態對齊 Spectra）：active 分頁 teal 粗框標示
// 目前專案、✕ 僅 active 與 hover 顯示、「＋」掛尾端接資料夾選擇器、徽章顯示
// 進行中變更數（hover tooltip）、失效分頁錯誤態（警示標記＋自分頁移除）。
// 分頁識別＝locator key（workspace-session 決策 1）；顯示文字與行為不變。
import { AlertTriangle, Cloud, CloudOff, Folder, LoaderCircle, LogIn, Plus, X } from "lucide-react";
import {
  Button,
  cn,
  useI18n,
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@speclink/ui";

import type { ProjectTab } from "../tabs";
import {
  locatorKey,
  type RemoteConnectionStateEvent,
  type RemoteWorkspaceRecoveryState,
  type RemoteWorkspaceStatus,
} from "../session";

export interface ProjectTabsProps {
  tabs: ProjectTab[];
  activeKey: string | null;
  /** 失效分頁錯誤（locator key → 單行訊息）。 */
  tabErrors: Record<string, string>;
  recoveryStates?: Record<string, RemoteWorkspaceRecoveryState>;
  connectionStates?: Record<string, RemoteConnectionStateEvent | undefined>;
  onActivate?: (key: string) => void;
  onClose?: (key: string) => void;
  /** 「＋」入口：開新增 Workspace chooser。 */
  onOpen?: () => void;
}

function TabBadge({ badge, tabKey }: { badge: number | null; tabKey: string }) {
  const { t } = useI18n();
  if (badge === null) return null;
  return (
    <TooltipProvider delayDuration={300}>
      <Tooltip>
        <TooltipTrigger asChild>
          <span
            data-badge={tabKey}
            className="inline-flex items-center justify-center min-w-4 h-4 px-1 rounded-full bg-primary/12 text-primary text-[10px] font-semibold tabular-nums"
          >
            {badge}
          </span>
        </TooltipTrigger>
        <TooltipContent>{t("app.tabBadgeTooltip").replace("{n}", String(badge))}</TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}

export function ProjectTabs({
  tabs,
  activeKey,
  tabErrors,
  recoveryStates = {},
  connectionStates = {},
  onActivate,
  onClose,
  onOpen,
}: ProjectTabsProps) {
  const { t } = useI18n();
  return (
    <div
      role="tablist"
      aria-label={t("app.workspaceTabs")}
      className="flex items-center gap-1 min-w-0 overflow-x-auto"
      data-project-tabs
    >
      {tabs.map((tab) => {
        const key = locatorKey(tab.locator);
        // tooltip：local 顯示路徑；remote 有 checkout 時明示連接路徑，否則顯示 locator key。
        const path =
          tab.locator.kind === "local"
            ? tab.locator.root
            : tab.locator.checkoutRoot
              ? t("app.checkoutTooltip").replace("{path}", tab.locator.checkoutRoot)
              : key;
        const remote = tab.locator.kind === "remote";
        const recovery = recoveryStates[key];
        const connectionState = connectionStates[key]?.state;
        const status: RemoteWorkspaceStatus = recovery
          ? recovery.status
          : connectionState === "offline" || connectionState === "needs-reauth"
            ? connectionState
            : tabErrors[key] !== undefined
              ? "error"
              : "ready";
        const active = key === activeKey;
        const error = status === "error";
        const statusLabel =
          status === "restoring"
            ? t("remote.recovery.restoringShort")
            : status === "offline"
              ? t("remote.recovery.offlineShort")
              : status === "needs-reauth"
                ? t("remote.recovery.reauthShort")
                : status === "error"
                  ? recovery?.status === "error"
                    ? t(`remote.recovery.${recovery.failure.kind}Short`)
                    : t("remote.recovery.unknownShort")
                  : "";
        const title = status === "ready" ? path : statusLabel;
        const statusIcon =
          status === "restoring" ? (
            <LoaderCircle
              data-tab-status="restoring"
              className="h-3 w-3 shrink-0 animate-spin text-primary motion-reduce:animate-none"
            />
          ) : status === "offline" ? (
            <CloudOff
              data-tab-status="offline"
              data-cloud-off={key}
              className="h-3 w-3 shrink-0 text-amber-600 dark:text-amber-400"
              strokeWidth={2.5}
            />
          ) : status === "needs-reauth" ? (
            <LogIn
              data-tab-status="needs-reauth"
              className="h-3 w-3 shrink-0 text-amber-600 dark:text-amber-400"
            />
          ) : status === "error" ? (
            <AlertTriangle
              data-tab-status="error"
              className="h-3 w-3 shrink-0 text-amber-600 dark:text-amber-400"
            />
          ) : remote ? (
            <Cloud
              data-tab-status="ready"
              data-cloud={key}
              className="h-3 w-3 shrink-0 text-primary"
              strokeWidth={2.5}
            />
          ) : (
            <Folder data-tab-status="ready" data-folder={key} className="h-3 w-3 shrink-0" />
          );
        return (
          <div
            key={key}
            data-tab={key}
            data-active={String(active)}
            data-error={String(error)}
            title={status === "ready" ? path : undefined}
            onClick={() => {
              if (!active) onActivate?.(key);
            }}
            className={cn(
              "group flex items-center rounded-md border text-xs shrink-0 transition-colors",
              active
                ? "border-2 border-primary bg-primary/8 font-semibold"
                : "border-border text-muted-foreground hover:text-foreground hover:bg-muted",
              error && "border-amber-500/50 bg-amber-500/5",
            )}
          >
            <button
              type="button"
              role="tab"
              aria-selected={active}
              data-status={status}
              title={title}
              className="flex min-w-0 cursor-pointer items-center gap-1.5 rounded-l-md px-2 py-1 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1"
              onKeyDown={(event) => {
                if ((event.key === "Enter" || event.key === " ") && !active) {
                  event.preventDefault();
                  onActivate?.(key);
                }
              }}
            >
              {statusIcon}
              <span className="truncate max-w-[140px]">{tab.name}</span>
              {status !== "ready" && <span className="sr-only">，{statusLabel}</span>}
              {status === "ready" && <TabBadge badge={tab.badge} tabKey={key} />}
            </button>
            {error ? (
              <Button
                type="button"
                variant="ghost"
                size="icon"
                aria-label={t("app.removeTab")}
                className="mr-1 h-6 w-6 shrink-0 text-muted-foreground hover:text-foreground"
                onClick={(e) => {
                  e.stopPropagation();
                  onClose?.(key);
                }}
              >
                <X className="h-3 w-3" />
              </Button>
            ) : (
              // ✕ 僅 active 與 hover 顯示（design D10）。
              <Button
                type="button"
                variant="ghost"
                size="icon"
                aria-label={t("app.closeTab")}
                className={cn(
                  "mr-1 h-6 w-6 shrink-0 text-muted-foreground hover:text-foreground transition-opacity",
                  active ? "opacity-100" : "opacity-0 group-hover:opacity-100",
                )}
                onClick={(e) => {
                  e.stopPropagation();
                  onClose?.(key);
                }}
              >
                <X className="h-3 w-3" />
              </Button>
            )}
          </div>
        );
      })}
      <Button
        type="button"
        variant="ghost"
        size="icon"
        aria-label={t("app.addWorkspace")}
        className="h-6 w-6 shrink-0 text-muted-foreground hover:text-foreground hover:bg-muted"
        onClick={onOpen}
      >
        <Plus className="h-4 w-4" />
      </Button>
    </div>
  );
}

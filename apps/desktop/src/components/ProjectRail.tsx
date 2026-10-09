// 圖示列（design D3）：品牌標記、每個開著的專案一個首字母方塊、「＋」、底部齒輪。
// 方塊不顯示計數徽章；角落狀態點：路徑失效或復原錯誤＝紅、remote 離線或需重新登入
// 或技能檔需要處理（上次探測為過期／缺失／較新）＝琥珀；探測中與復原中以 spinner 取代首字母。右鍵＝專案動作選單；本機路徑失效
// 時點擊開錯誤選單而不切換（remote 復原錯誤仍可選，主區是復原頁）。
import type { ReactNode } from "react";
import { LoaderCircle, Plus, Settings } from "lucide-react";
import {
  BrandMark,
  ContextMenu,
  ContextMenuContent,
  ContextMenuTrigger,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
  SEMANTIC_TONE,
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
  cn,
  useI18n,
} from "@speclink/ui";

import type { AssetPromptState } from "../assetPrompt";
import type { Platform } from "../platform";
import type { ProjectTab } from "../tabs";
import {
  locatorKey,
  type RemoteConnectionStateEvent,
  type RemoteWorkspaceRecoveryState,
  type RemoteWorkspaceStatus,
} from "../session";
import { ProjectActionItems, type ProjectActions } from "./ProjectActionsMenu";

export interface ProjectRailProps {
  tabs: ProjectTab[];
  activeKey: string | null;
  /** 失效分頁錯誤（locator key → 單行訊息）。 */
  tabErrors: Record<string, string>;
  recoveryStates?: Record<string, RemoteWorkspaceRecoveryState>;
  connectionStates?: Record<string, RemoteConnectionStateEvent | undefined>;
  /** 連線 id → 連線名稱（remote 無 checkout 時 tooltip 用）。 */
  connectionNames?: Record<string, string>;
  /** 探測進行中的目標方塊（store 的 pendingTabKey）。 */
  pendingKey?: string | null;
  /** 各分頁上次的技能檔提示（locator key → 值；非 null 即亮琥珀點）。 */
  assetPrompts?: Record<string, AssetPromptState | null>;
  platform: Platform;
  /** 右鍵與錯誤選單的動作（與標題列「⋯」同一份）。 */
  actions: ProjectActions;
  onActivate?: (key: string) => void;
  /** 「＋」入口：開新增專案對話框。 */
  onOpen?: () => void;
  onOpenSettings?: () => void;
  settingsActive?: boolean;
  /** 齒輪上方的「有新版本」狀態鈕（desktop-notice-relocation design D2；無事時為 null）。 */
  updateButton?: ReactNode;
}

/** 方塊與左側標題列的 hover 路徑：本機根；remote 有 checkout 明示連接路徑，否則連線名稱加 Project/Repo。 */
export function tabPathHint(
  tab: ProjectTab,
  connectionNames: Record<string, string>,
  t: (key: string) => string,
): string {
  const { locator } = tab;
  if (locator.kind === "local") return locator.root;
  if (locator.checkoutRoot) return t("app.checkoutTooltip").replace("{path}", locator.checkoutRoot);
  const connection = connectionNames[locator.connectionId] ?? locator.connectionId;
  return `${connection} · ${locator.projectId}/${locator.repoId}`;
}

const SQUARE = "relative flex h-8 w-8 shrink-0 items-center justify-center rounded-lg text-[13px] font-medium";

export function ProjectRail({
  tabs,
  activeKey,
  tabErrors,
  recoveryStates = {},
  connectionStates = {},
  connectionNames = {},
  pendingKey = null,
  assetPrompts = {},
  platform,
  actions,
  onActivate,
  onOpen,
  onOpenSettings,
  settingsActive = false,
  updateButton,
}: ProjectRailProps) {
  const { t } = useI18n();
  return (
    <TooltipProvider>
      <div
        data-project-rail
        className="flex w-14 shrink-0 flex-col items-center border-r border-border pb-3 pt-2"
      >
        <BrandMark size={26} className="mb-3.5" />
        <div
          role="tablist"
          aria-label={t("app.projectRail")}
          aria-orientation="vertical"
          className="flex flex-col items-center gap-2"
        >
          {tabs.map((tab) => {
            const key = locatorKey(tab.locator);
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
            const pending = key === pendingKey;
            // 技能檔狀況是上次看過時的狀態（背景分頁不主動探測），只說「需要處理」不帶版號。
            const assetFlag = assetPrompts[key] != null;
            const assetHint = assetFlag ? t("assets.railHint") : "";
            // 本機路徑失效：點擊開錯誤選單、不切換。remote 錯誤的主區是復原頁，照常可選。
            const localError = tab.locator.kind === "local" && status === "error";
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
                        : localError
                          ? tabErrors[key]
                          : t("remote.recovery.unknownShort")
                      : "";
            const path = tabPathHint(tab, connectionNames, t);
            const dot =
              status === "error"
                ? "error"
                : status === "offline" || status === "needs-reauth" || assetFlag
                  ? "warning"
                  : null;
            const ariaLabel = [tab.name, statusLabel, assetHint, pending ? t("app.tabSwitching") : ""]
              .filter(Boolean)
              .join("，");
            const button = (
              <button
                type="button"
                role="tab"
                aria-selected={active}
                aria-label={ariaLabel}
                data-tab={key}
                data-active={String(active)}
                data-error={String(status === "error")}
                data-status={status}
                className={cn(
                  SQUARE,
                  "transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                  active
                    ? "bg-primary text-primary-foreground"
                    : "bg-muted text-muted-foreground hover:bg-muted/70",
                )}
                onClick={() => {
                  if (!active && !localError) onActivate?.(key);
                }}
                onKeyDown={(event) => {
                  if ((event.key === "Enter" || event.key === " ") && !active && !localError) {
                    event.preventDefault();
                    onActivate?.(key);
                  }
                }}
              >
                {pending ? (
                  <LoaderCircle
                    data-tab-status="pending"
                    role="img"
                    aria-label={t("app.tabSwitching")}
                    className="h-4 w-4 animate-spin motion-reduce:animate-none"
                  />
                ) : status === "restoring" ? (
                  <LoaderCircle
                    data-tab-status="restoring"
                    className={cn("h-4 w-4 animate-spin motion-reduce:animate-none", SEMANTIC_TONE.inProgress)}
                  />
                ) : (
                  Array.from(tab.name.trim())[0]?.toUpperCase()
                )}
                {dot && (
                  <span
                    data-tab-dot={dot}
                    className={cn(
                      "absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full border-2 border-sidebar",
                      dot === "error" ? "bg-destructive" : "bg-status-warning",
                    )}
                  />
                )}
              </button>
            );
            // 右鍵觸發器要直接包住按鈕：包在 DropdownMenu（非 DOM 的 Root）外面時，
            // Slot 傳下的 onContextMenu 會被丟掉。
            const trigger = <ContextMenuTrigger asChild>{button}</ContextMenuTrigger>;
            return (
              <Tooltip key={key}>
                <ContextMenu>
                  {localError ? (
                    <DropdownMenu>
                      <TooltipTrigger asChild>
                        <DropdownMenuTrigger asChild>{trigger}</DropdownMenuTrigger>
                      </TooltipTrigger>
                      <DropdownMenuContent side="right" align="start">
                        <ProjectActionItems
                          menu="dropdown"
                          tab={tab}
                          platform={platform}
                          actions={actions}
                          error={tabErrors[key]}
                        />
                      </DropdownMenuContent>
                    </DropdownMenu>
                  ) : (
                    <TooltipTrigger asChild>{trigger}</TooltipTrigger>
                  )}
                  <ContextMenuContent>
                    <ProjectActionItems
                      menu="context"
                      tab={tab}
                      platform={platform}
                      actions={actions}
                      error={localError ? tabErrors[key] : undefined}
                    />
                  </ContextMenuContent>
                </ContextMenu>
                <TooltipContent side="right" className="max-w-80">
                  <div className="font-medium">{tab.name}</div>
                  <div className="break-all text-muted-foreground">{path}</div>
                  {statusLabel && <div className="text-muted-foreground">{statusLabel}</div>}
                  {assetHint && <div className="text-muted-foreground">{assetHint}</div>}
                </TooltipContent>
              </Tooltip>
            );
          })}
        </div>
        <button
          type="button"
          aria-label={t("app.newProject")}
          className={cn(
            SQUARE,
            "mt-2 border border-dashed border-border bg-transparent text-muted-foreground hover:bg-muted hover:text-foreground",
          )}
          onClick={onOpen}
        >
          <Plus className="h-4 w-4" />
        </button>
        {/* 沉底的一組：更新鈕（有事才渲染）＋齒輪；gap 只在兩者都在時才出現。 */}
        <div className="mt-auto flex flex-col items-center gap-1">
          {updateButton}
          <button
            type="button"
            aria-label={t("app.navSettings")}
            className={cn(
              SQUARE,
              settingsActive
                ? "bg-primary text-primary-foreground"
                : "bg-transparent text-muted-foreground hover:bg-muted hover:text-foreground",
            )}
            onClick={onOpenSettings}
          >
            <Settings className="h-4 w-4" />
          </button>
        </div>
      </div>
    </TooltipProvider>
  );
}

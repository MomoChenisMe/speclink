// 專案分頁列（design D10，UI 形態對齊 Spectra）：active 分頁 teal 粗框標示
// 目前專案、✕ 僅 active 與 hover 顯示、「＋」掛尾端接資料夾選擇器、徽章顯示
// 進行中變更數（hover tooltip）、失效分頁錯誤態（警示標記＋自分頁移除）。
// 分頁識別＝locator key（workspace-session 決策 1）；顯示文字與行為不變。
import { AlertTriangle, Cloud, Folder, Plus, X } from "lucide-react";
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
import { locatorKey } from "../session";

export interface ProjectTabsProps {
  tabs: ProjectTab[];
  activeKey: string | null;
  /** 失效分頁錯誤（locator key → 單行訊息）。 */
  tabErrors: Record<string, string>;
  onActivate?: (key: string) => void;
  onClose?: (key: string) => void;
  /** 「＋」入口：開資料夾選擇器。 */
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

export function ProjectTabs({ tabs, activeKey, tabErrors, onActivate, onClose, onOpen }: ProjectTabsProps) {
  const { t } = useI18n();
  return (
    <div className="flex items-center gap-1 min-w-0 overflow-x-auto" data-project-tabs>
      {tabs.map((tab) => {
        const key = locatorKey(tab.locator);
        // tooltip：local 顯示路徑、remote 顯示 locator key（分頁名已是 Project/Repo）。
        const path = tab.locator.kind === "local" ? tab.locator.root : key;
        const remote = tab.locator.kind === "remote";
        const active = key === activeKey;
        const error = tabErrors[key] !== undefined;
        return (
          <div
            key={key}
            data-tab={key}
            data-active={String(active)}
            data-error={String(error)}
            title={error ? tabErrors[key] : path}
            className={cn(
              "group flex items-center gap-1.5 rounded-md border px-2 py-1 text-xs shrink-0 cursor-pointer transition-colors",
              active
                ? "border-2 border-primary bg-primary/8 font-semibold"
                : "border-border text-muted-foreground hover:text-foreground hover:bg-muted",
              error && "text-muted-foreground/60",
            )}
            onClick={() => {
              if (!active) onActivate?.(key);
            }}
          >
            {error && <AlertTriangle className="h-3 w-3 text-amber-500 shrink-0" />}
            {/* 來源圖示（remote-data-source §10.5）：remote＝主色 cloud、local＝
                folder——兩者並立讓分頁列一眼可辨資料來源。 */}
            {remote ? (
              <Cloud data-cloud={key} className="h-3 w-3 shrink-0 text-primary" strokeWidth={2.5} />
            ) : (
              <Folder data-folder={key} className="h-3 w-3 shrink-0" />
            )}
            <span className="truncate max-w-[140px]">{tab.name}</span>
            {!error && <TabBadge badge={tab.badge} tabKey={key} />}
            {error ? (
              <Button
                type="button"
                variant="ghost"
                size="icon"
                aria-label={t("app.removeTab")}
                className="h-4 w-4 shrink-0 text-muted-foreground hover:text-foreground"
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
                  "h-4 w-4 shrink-0 text-muted-foreground hover:text-foreground transition-opacity",
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
        aria-label={t("app.openProject")}
        className="h-6 w-6 shrink-0 text-muted-foreground hover:text-foreground hover:bg-muted"
        onClick={onOpen}
      >
        <Plus className="h-4 w-4" />
      </Button>
    </div>
  );
}

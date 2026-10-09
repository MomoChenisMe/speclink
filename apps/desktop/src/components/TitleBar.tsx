// 三平台標題列（design D1／D6）：左側標題列（圖示列＋專案欄共用）與主區頂列，
// 兩者整列可拖曳（deep：子樹皆可拖、按鈕等可點元素由 Tauri 自動排除）。
import { Fragment, useEffect, useState, type ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Copy, Minus, Square, X } from "lucide-react";
import { cn, useI18n } from "@speclink/ui";

import type { Platform } from "../platform";

export interface Crumb {
  label: string;
  onClick?: () => void;
}

export function LeftTitleBar({
  title,
  path,
  menu,
}: {
  /** 純文字標題（作用中專案名或「設定」）；零專案時不傳。 */
  title?: string;
  /** hover 顯示的完整路徑。 */
  path?: string;
  /** 右端動作槽（專案動作選單的「⋯」）。 */
  menu?: ReactNode;
}) {
  // 標題左緣對齊下方專案欄的導覽文字：圖示列 56＋專案欄內距 8＋導覽項內距 10＋
  // 圖示 16＋間距 8＝98px（macOS 紅綠燈落在其左側留白）。下緣細線與主區頂列
  // 同高，頂列成為橫貫視窗的一條帶，圖示列分隔線從這條線往下。
  return (
    <div
      data-testid="left-titlebar"
      data-tauri-drag-region="deep"
      className="flex h-12 shrink-0 select-none items-center gap-1 border-b border-border pl-[98px] pr-2"
    >
      {title && (
        <span title={path} className="min-w-0 flex-1 truncate text-sm font-medium">
          {title}
        </span>
      )}
      {menu && <div className="ml-auto shrink-0">{menu}</div>}
    </div>
  );
}

export function MainTitleBar({ platform, crumbs }: { platform: Platform; crumbs: Crumb[] }) {
  return (
    <div
      data-testid="main-titlebar"
      data-tauri-drag-region="deep"
      className="flex h-12 shrink-0 select-none items-center border-b border-border bg-background pl-5"
    >
      <nav className="flex min-w-0 flex-1 items-center gap-1.5 text-xs text-muted-foreground">
        {crumbs.map((crumb, i) => {
          const last = i === crumbs.length - 1;
          const cls = cn("truncate", last && "text-foreground");
          return (
            <Fragment key={i}>
              {i > 0 && <span className="opacity-50">/</span>}
              {crumb.onClick ? (
                <button
                  type="button"
                  className={cn(cls, "hover:text-foreground")}
                  onClick={crumb.onClick}
                >
                  {crumb.label}
                </button>
              ) : (
                <span className={cls}>{crumb.label}</span>
              )}
            </Fragment>
          );
        })}
      </nav>
      {platform === "windows" && <WindowControls />}
    </div>
  );
}

/** Windows 自繪視窗鈕：API 被拒（權限缺）時按鈕無效、只記 console.error。 */
export function WindowControls() {
  const { t } = useI18n();
  const [maximized, setMaximized] = useState(false);
  useEffect(() => {
    const win = getCurrentWindow();
    let disposed = false;
    const sync = () =>
      win
        .isMaximized()
        .then((m) => {
          if (!disposed) setMaximized(m);
        })
        .catch(console.error);
    void sync();
    const unlisten = win.onResized(() => void sync()).catch(() => null);
    return () => {
      disposed = true;
      void unlisten.then((u) => u?.());
    };
  }, []);
  const run = (action: "minimize" | "toggleMaximize" | "close") => () =>
    void getCurrentWindow()[action]().catch(console.error);
  const btn = "flex h-12 w-[46px] items-center justify-center text-foreground";
  return (
    <div className="flex shrink-0 self-stretch">
      <button
        type="button"
        aria-label={t("titlebar.minimize")}
        className={cn(btn, "hover:bg-muted")}
        onClick={run("minimize")}
      >
        <Minus className="h-4 w-4" />
      </button>
      <button
        type="button"
        aria-label={t(maximized ? "titlebar.restore" : "titlebar.maximize")}
        className={cn(btn, "hover:bg-muted")}
        onClick={run("toggleMaximize")}
      >
        {maximized ? <Copy className="h-3.5 w-3.5" /> : <Square className="h-3.5 w-3.5" />}
      </button>
      <button
        type="button"
        aria-label={t("titlebar.close")}
        className={cn(btn, "hover:bg-destructive hover:text-white")}
        onClick={run("close")}
      >
        <X className="h-4 w-4" />
      </button>
    </div>
  );
}

// 最近開啟清單（design D7）：新增專案對話框第一步與零專案空狀態共用同一份清單與
// 開啟流程；兩個 variant 只差外框。
import { useState } from "react";
import { AlertTriangle, Cloud, Folder, X } from "lucide-react";
import { cn, useI18n } from "@speclink/ui";

import type { ConnectionView, ConnectionsAdapter } from "../adapter/connections";
import type { WorkspaceAdapter } from "../adapter/workspace";
import type { RecentEntry } from "../recents";
import { locatorKey } from "../session";

export interface RecentListProps {
  variant: "dialog" | "card";
  /** 已濾掉專案列上開著的項目（visibleRecents）。 */
  entries: RecentEntry[];
  connections: ConnectionView[];
  /** 連線清單是否讀取成功——未就緒時 remote 列不得判成「連線已移除／已登出」。 */
  connectionsReady: boolean;
  /** 點擊開啟失敗的錯誤（locator key → 原因）。 */
  errors: Record<string, string>;
  busy?: boolean;
  onOpen: (entry: RecentEntry) => void;
  onRemove: (entry: RecentEntry) => void;
}

export function RecentList({
  variant,
  entries,
  connections,
  connectionsReady,
  errors,
  busy = false,
  onOpen,
  onRemove,
}: RecentListProps) {
  const { t } = useI18n();
  if (entries.length === 0) return null;
  return (
    <section
      data-recent-list={variant}
      className={cn("flex flex-col gap-2", variant === "card" && "rounded-xl border border-border bg-card p-4")}
    >
      <p className="m-0 flex items-baseline gap-1.5 text-sm font-medium">
        <span>{t("recent.title")}</span>
        <span className="text-xs font-normal tabular-nums text-muted-foreground">{entries.length}</span>
      </p>
      <div
        data-recent-rows
        className={cn(
          "flex max-h-[240px] flex-col divide-y divide-border overflow-y-auto",
          variant === "dialog" && "rounded-xl border border-border",
        )}
      >
        {entries.map((entry) => {
          const key = locatorKey(entry.locator);
          const connectionId = entry.locator.kind === "remote" ? entry.locator.connectionId : null;
          const connection = connectionId ? connections.find((c) => c.id === connectionId) : null;
          // 連線清單載入完成前不下判斷；載入後才分「已移除」與「已登出」。
          const connectionReason =
            connectionId && connectionsReady
              ? !connection
                ? t("chooser.recentConnectionMissing")
                : !connection.loggedIn
                  ? t("chooser.recentConnectionLoggedOut")
                  : null
              : null;
          const reason = errors[key] ?? connectionReason;
          const subtitle =
            entry.locator.kind === "local" ? entry.locator.root : (connection?.name ?? null);
          return (
            <div
              key={key}
              className={cn(
                "group flex items-start gap-1 px-3 py-2 transition-colors",
                reason ? "bg-destructive/5" : "hover:bg-muted/60",
              )}
            >
              {/* 開啟失敗的錯誤態仍可再點重試；連線已移除／已登出才停用。 */}
              <button
                type="button"
                disabled={busy || Boolean(connectionReason)}
                onClick={() => onOpen(entry)}
                className="flex min-w-0 flex-1 items-start gap-2 text-left disabled:cursor-not-allowed"
              >
                <span className="mt-0.5 shrink-0 text-muted-foreground">
                  {reason ? (
                    <AlertTriangle className="h-4 w-4 text-destructive" />
                  ) : entry.locator.kind === "local" ? (
                    <Folder className="h-4 w-4" />
                  ) : (
                    <Cloud className="h-4 w-4" />
                  )}
                </span>
                <span className="min-w-0 flex-1">
                  <span className="flex items-center gap-1.5">
                    <span className="truncate text-sm font-medium text-foreground">{entry.name}</span>
                    {entry.locator.kind === "remote" && (
                      <span className="shrink-0 rounded-full bg-muted px-1.5 text-[11px] text-muted-foreground">
                        {t("chooser.remoteTag")}
                      </span>
                    )}
                  </span>
                  {subtitle && (
                    <span
                      className={cn(
                        "block truncate text-xs text-muted-foreground",
                        entry.locator.kind === "local" && "font-mono",
                      )}
                    >
                      {subtitle}
                    </span>
                  )}
                  {reason && <span className="block text-xs text-destructive">{reason}</span>}
                </span>
              </button>
              <button
                type="button"
                aria-label={`${t("chooser.recentRemove")} ${entry.name}`}
                onClick={() => onRemove(entry)}
                className="mt-0.5 shrink-0 rounded p-0.5 text-muted-foreground opacity-0 transition-opacity hover:text-foreground focus-visible:opacity-100 group-hover:opacity-100"
              >
                <X className="h-3.5 w-3.5" />
              </button>
            </div>
          );
        })}
      </div>
    </section>
  );
}

/** 最近開啟條目的開啟流程（design D4）：本機先探測再沿既有開啟流程（分流由
 * openProjectAt 承擔）；remote 以原 connection／scope／checkout 走既有 remote 開啟。
 * 失敗只標記該列——錯誤態只活在呼叫端畫面，重試或 reset 即清。 */
export function useRecentOpener({
  workspace,
  connections,
  connectionAdapter,
  onRefreshConnections,
  onOpenLocal,
  onOpenRemote,
  onLocalProbed,
}: {
  workspace: Pick<WorkspaceAdapter, "openProject">;
  connections: ConnectionView[];
  connectionAdapter: Pick<ConnectionsAdapter, "inspectCheckout">;
  /** 重整連線清單；回傳讀取是否成功（成功才判定 remote 列的連線錯誤態）。 */
  onRefreshConnections: () => Promise<boolean>;
  onOpenLocal: (path: string) => Promise<void>;
  onOpenRemote: (connectionId: string, target: string, checkoutRoot?: string) => Promise<void>;
  /** 本機探測成功、交給開啟流程之前（對話框於此關閉自己）。 */
  onLocalProbed?: () => void;
}) {
  const { t } = useI18n();
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [busy, setBusy] = useState(false);
  /** 連線清單是否讀取成功——載入中或讀取失敗時，remote 列都不得判成
   * 「連線已移除」：清單初值是空陣列，讀取失敗又保留現值，先判斷會把有效
   * 條目誤標成已移除且停用開啟。 */
  const [connectionsReady, setConnectionsReady] = useState(false);

  async function refreshConnections() {
    setConnectionsReady(await onRefreshConnections().catch(() => false));
  }

  /** 重新進入（對話框重開、空狀態出現）：清錯誤態、重讀連線清單。 */
  function reset() {
    setErrors({});
    setConnectionsReady(false);
    void refreshConnections();
  }

  async function open(entry: RecentEntry) {
    const key = locatorKey(entry.locator);
    setErrors(({ [key]: _retried, ...rest }) => rest);
    setBusy(true);
    try {
      if (entry.locator.kind === "local") {
        await workspace.openProject(entry.locator.root);
        onLocalProbed?.();
        await onOpenLocal(entry.locator.root);
      } else {
        const { connectionId, projectId, repoId, checkoutRoot } = entry.locator;
        // 綁著 checkout 的條目先驗資料夾仍與該 scope 一致（與其他 checkout 開啟
        // 路徑同一步）——handshake 只問伺服器，不會發現資料夾已消失。這一步要
        // connection 的 origin；規格模式（無 checkout 綁定）只需 connectionId，
        // 不得因清單未就緒而被擋下。
        if (checkoutRoot) {
          const connection = connections.find((c) => c.id === connectionId);
          if (!connection) {
            // 清單未就緒時列仍是啟用的——此處補判，理由與就緒後的列同一句純文字。
            setErrors((prev) => ({ ...prev, [key]: t("chooser.recentConnectionMissing") }));
            return;
          }
          await connectionAdapter.inspectCheckout(checkoutRoot, connection.origin, projectId, repoId);
        }
        await onOpenRemote(connectionId, `${projectId}/${repoId}`, checkoutRoot);
      }
    } catch (reason) {
      setErrors((prev) => ({ ...prev, [key]: String(reason) }));
    } finally {
      setBusy(false);
    }
  }

  return { errors, busy, connectionsReady, open, reset, refreshConnections };
}

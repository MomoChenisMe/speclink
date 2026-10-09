// 零專案空狀態（design D7）：字標、標題、說明、「新增專案」與「連線 Server」、
// 最近開啟卡（與新增專案對話框同一份清單與開啟流程）、底部 CLI 提示。
import { useEffect } from "react";
import { Cloud, FolderOpen } from "lucide-react";
import { Button, Wordmark, useI18n } from "@speclink/ui";

import type { ConnectionView, ConnectionsAdapter } from "../adapter/connections";
import type { WorkspaceAdapter } from "../adapter/workspace";
import type { RecentEntry } from "../recents";
import { locatorKey } from "../session";
import type { WorkspaceChooserIntent } from "../store";
import { RecentList, useRecentOpener } from "./RecentList";

export interface EmptyWorkspaceProps {
  /** 已濾掉專案列上開著的項目（零專案時即全部記錄）。 */
  recents: RecentEntry[];
  connections: ConnectionView[];
  /** 重整連線清單；回傳讀取是否成功（成功才判定 remote 列的連線錯誤態）。 */
  onRefreshConnections: () => Promise<boolean>;
  workspace: Pick<WorkspaceAdapter, "openProject">;
  connectionAdapter: Pick<ConnectionsAdapter, "inspectCheckout">;
  onOpenLocal: (path: string) => Promise<void>;
  onOpenRemote: (connectionId: string, target: string, checkoutRoot?: string) => Promise<void>;
  onRemoveRecent: (key: string) => void;
  onOpenChooser: (intent?: WorkspaceChooserIntent) => void;
}

export function EmptyWorkspace({
  recents,
  connections,
  onRefreshConnections,
  workspace,
  connectionAdapter,
  onOpenLocal,
  onOpenRemote,
  onRemoveRecent,
  onOpenChooser,
}: EmptyWorkspaceProps) {
  const { t } = useI18n();
  const recent = useRecentOpener({
    workspace,
    connections,
    connectionAdapter,
    onRefreshConnections,
    onOpenLocal,
    onOpenRemote,
  });
  useEffect(() => {
    // 掛載時讀一次連線清單；之後的連線變動由 store 推進 connections。
    recent.reset();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  return (
    <div className="flex h-full items-center justify-center overflow-y-auto">
      <div className="flex w-[420px] max-w-full flex-col gap-4">
        <Wordmark className="h-10 self-start" />
        <div>
          <h2 className="text-2xl font-normal">{t("app.emptyTitle")}</h2>
          <p className="mt-1 text-[13px] text-muted-foreground">{t("app.emptyDesc")}</p>
        </div>
        <div className="flex gap-2">
          <Button className="gap-1.5" onClick={() => onOpenChooser()}>
            <FolderOpen className="h-4 w-4" /> {t("app.newProject")}
          </Button>
          <Button
            variant="outline"
            className="gap-1.5"
            onClick={() => onOpenChooser({ initialStep: "server" })}
          >
            <Cloud className="h-4 w-4" /> {t("app.connectServer")}
          </Button>
        </div>
        <RecentList
          variant="card"
          entries={recents}
          connections={connections}
          connectionsReady={recent.connectionsReady}
          errors={recent.errors}
          busy={recent.busy}
          onOpen={(entry) => void recent.open(entry)}
          onRemove={(entry) => onRemoveRecent(locatorKey(entry.locator))}
        />
        <p className="text-xs text-muted-foreground">{t("app.emptyCli")}</p>
      </div>
    </div>
  );
}

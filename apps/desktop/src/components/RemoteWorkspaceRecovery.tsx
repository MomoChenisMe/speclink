import {
  AlertTriangle,
  ChevronRight,
  CloudCog,
  LoaderCircle,
  LogIn,
  RefreshCw,
  Server,
  Settings,
  X,
} from "lucide-react";
import { useId, useState } from "react";
import { Button, useI18n } from "@speclink/ui";

import type { ConnectionView } from "../adapter/connections";
import type { RemoteWorkspaceRecoveryState } from "../session";
import type { ProjectTab } from "../tabs";

export interface RemoteWorkspaceRecoveryProps {
  tab: ProjectTab;
  recovery: RemoteWorkspaceRecoveryState;
  connection?: ConnectionView;
  onRetry: () => void;
  onOpenSettings: () => void;
  onReauthenticate: () => void;
  onRemove: () => void;
}

export function RemoteWorkspaceRecovery({
  tab,
  recovery,
  connection,
  onRetry,
  onOpenSettings,
  onReauthenticate,
  onRemove,
}: RemoteWorkspaceRecoveryProps) {
  const { t } = useI18n();
  const [technicalDetailOpen, setTechnicalDetailOpen] = useState(false);
  const technicalDetailId = useId();
  const serverName = connection?.name ?? t("remote.recovery.serverUnknown");
  const serverOrigin = connection?.origin ??
    (tab.locator.kind === "remote" ? tab.locator.connectionId : "");

  if (recovery.status === "restoring") {
    return (
      <section
        role="status"
        aria-live="polite"
        data-testid="remote-workspace-recovery"
        className="mx-auto flex h-full max-w-xl flex-col items-center justify-center gap-5 px-8 text-center"
      >
        <div className="flex h-12 w-12 items-center justify-center rounded-2xl border border-primary/20 bg-primary/8 text-primary">
          <LoaderCircle className="h-6 w-6 animate-spin motion-reduce:animate-none" />
        </div>
        <div className="space-y-1.5">
          <h1 className="text-xl font-semibold tracking-tight">
            {t("remote.recovery.restoringTitle")}
          </h1>
          <p className="text-sm text-muted-foreground">
            {t("remote.recovery.restoringDesc")}
          </p>
        </div>
        <WorkspaceIdentity name={tab.name} serverName={serverName} serverOrigin={serverOrigin} />
      </section>
    );
  }

  const { failure } = recovery;
  const title = t(`remote.recovery.${failure.kind}Title`);
  const description = t(`remote.recovery.${failure.kind}Desc`);
  const needsReauth = failure.kind === "needs-reauth";

  return (
    <section
      role="alert"
      aria-live="polite"
      data-testid="remote-workspace-recovery"
      data-recovery-kind={failure.kind}
      className="mx-auto flex h-full max-w-2xl flex-col justify-center px-8 py-10"
    >
      <div className="rounded-2xl border border-amber-500/35 bg-card shadow-sm">
        <div className="flex gap-4 border-b border-border px-6 py-5">
          <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl bg-amber-500/12 text-amber-700 dark:text-amber-300">
            <AlertTriangle className="h-5 w-5" />
          </div>
          <div className="min-w-0 space-y-1">
            <p className="text-xs font-semibold uppercase tracking-[0.14em] text-amber-700 dark:text-amber-300">
              {t("remote.recovery.eyebrow")}
            </p>
            <h1 className="text-xl font-semibold tracking-tight">{title}</h1>
            <p className="max-w-xl text-sm leading-6 text-muted-foreground">{description}</p>
          </div>
        </div>

        <div className="space-y-5 px-6 py-5">
          <WorkspaceIdentity name={tab.name} serverName={serverName} serverOrigin={serverOrigin} />

          <div className="flex flex-wrap items-center gap-2">
            {needsReauth ? (
              <Button type="button" className="gap-2" onClick={onReauthenticate}>
                <LogIn className="h-4 w-4" /> {t("remote.reauthAction")}
              </Button>
            ) : (
              <Button type="button" className="gap-2" onClick={onRetry}>
                <RefreshCw className="h-4 w-4" /> {t("remote.recovery.retry")}
              </Button>
            )}
            <Button type="button" variant="outline" className="gap-2" onClick={onOpenSettings}>
              <Settings className="h-4 w-4" /> {t("remote.recovery.serverSettings")}
            </Button>
            <Button
              type="button"
              variant="ghost"
              className="ml-auto gap-2 text-muted-foreground hover:text-destructive"
              onClick={onRemove}
            >
              <X className="h-4 w-4" /> {t("app.removeTab")}
            </Button>
          </div>

          <div className="rounded-lg border border-border bg-muted/30 px-4 py-3 text-sm">
            <button
              type="button"
              aria-expanded={technicalDetailOpen}
              aria-controls={technicalDetailId}
              className="flex w-full items-center gap-1.5 rounded-sm text-left font-medium text-muted-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
              onClick={() => setTechnicalDetailOpen((open) => !open)}
            >
              <ChevronRight
                aria-hidden="true"
                className={`h-4 w-4 shrink-0 transition-transform motion-reduce:transition-none ${
                  technicalDetailOpen ? "rotate-90" : ""
                }`}
              />
              {t("remote.recovery.technicalDetail")}
            </button>
            {technicalDetailOpen ? (
              <pre
                id={technicalDetailId}
                className="mt-3 max-h-36 overflow-auto whitespace-pre-wrap break-words rounded-md bg-background p-3 font-mono text-xs leading-5 text-foreground"
              >
                {failure.message}
              </pre>
            ) : null}
          </div>
        </div>
      </div>
    </section>
  );
}

function WorkspaceIdentity({
  name,
  serverName,
  serverOrigin,
}: {
  name: string;
  serverName: string;
  serverOrigin: string;
}) {
  const { t } = useI18n();
  return (
    <dl className="grid gap-3 rounded-xl border border-border bg-muted/25 p-4 text-sm sm:grid-cols-2">
      <div className="min-w-0">
        <dt className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
          <CloudCog className="h-3.5 w-3.5" /> {t("remote.recovery.workspace")}
        </dt>
        <dd className="mt-1 truncate font-medium text-foreground">{name}</dd>
      </div>
      <div className="min-w-0">
        <dt className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
          <Server className="h-3.5 w-3.5" /> {t("remote.recovery.server")}
        </dt>
        <dd className="mt-1 truncate font-medium text-foreground">
          {serverName} · <span className="font-normal text-muted-foreground">{serverOrigin}</span>
        </dd>
      </div>
    </dl>
  );
}

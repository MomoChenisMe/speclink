import { useEffect, useState } from "react";
import {
  ArrowLeft,
  Check,
  CloudUpload,
  FolderArchive,
  LoaderCircle,
  Server,
} from "lucide-react";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
  SEMANTIC_SURFACE,
  SEMANTIC_TONE,
  cn,
  useI18n,
} from "@speclink/ui";

import type { MigrationAdapter, MigrationResult } from "../adapter/migration";
import type {
  ConnectionView,
  ConnectionsAdapter,
  ScopesView,
} from "../adapter/connections";
import { ScopeSelection, type ScopeChoice } from "./WorkspaceChooser";

type MigrationStep = "server" | "scopes" | "confirm" | "running" | "success";

export interface MigrationDialogProps {
  open: boolean;
  root: string;
  connections: ConnectionView[];
  connectionAdapter: Pick<ConnectionsAdapter, "scopes">;
  migration: MigrationAdapter;
  onOpenChange: (open: boolean) => void;
  onMigrated: (
    connectionId: string,
    target: string,
    checkoutRoot: string,
  ) => Promise<void>;
}

function message(template: string, values: Record<string, string>): string {
  return Object.entries(values).reduce(
    (text, [key, value]) => text.replace(`{${key}}`, value),
    template,
  );
}

export function MigrationDialog({
  open,
  root,
  connections,
  connectionAdapter,
  migration,
  onOpenChange,
  onMigrated,
}: MigrationDialogProps) {
  const { t } = useI18n();
  const [step, setStep] = useState<MigrationStep>("server");
  const [connection, setConnection] = useState<ConnectionView | null>(null);
  const [scopes, setScopes] = useState<ScopesView | null>(null);
  const [scope, setScope] = useState<ScopeChoice | null>(null);
  const [result, setResult] = useState<MigrationResult | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setStep("server");
    setConnection(null);
    setScopes(null);
    setScope(null);
    setResult(null);
    setBusy(false);
    setError(null);
  }, [open, root]);

  async function selectConnection(selected: ConnectionView) {
    setConnection(selected);
    setScopes(null);
    setScope(null);
    setError(null);
    setStep("scopes");
    setBusy(true);
    try {
      setScopes(await connectionAdapter.scopes(selected.id));
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function runMigration() {
    if (!connection || !scope) return;
    setStep("running");
    setError(null);
    try {
      const migrated = await migration.migrate(
        root,
        connection.id,
        scope.project.key,
        scope.repo.key,
      );
      await onMigrated(
        connection.id,
        `${scope.project.key}/${scope.repo.key}`,
        migrated.checkoutRoot,
      );
      setResult(migrated);
      setStep("success");
    } catch (reason) {
      setError(String(reason));
      setStep("confirm");
    }
  }

  const loggedInConnections = connections.filter((entry) => entry.loggedIn);
  const target = scope ? `${scope.project.key} / ${scope.repo.key}` : "";
  const locked = step === "running";

  return (
    <AlertDialog
      open={open}
      onOpenChange={(isOpen) => {
        if (!locked) onOpenChange(isOpen);
      }}
    >
      <AlertDialogContent className="max-w-xl gap-4" data-testid="migration-dialog">
        <AlertDialogHeader>
          <div className="mb-1 flex items-center gap-2 text-[11px] font-semibold uppercase tracking-[0.16em] text-muted-foreground">
            <CloudUpload className="h-3.5 w-3.5" />
            {t("migration.eyebrow")}
          </div>
          <AlertDialogTitle>
            {step === "success" ? t("migration.successTitle") : t("migration.title")}
          </AlertDialogTitle>
          <AlertDialogDescription>
            {step === "server"
              ? t("migration.serverDesc")
              : step === "scopes"
                ? t("migration.scopeDesc")
                : step === "success"
                  ? t("migration.successDesc")
                  : t("migration.confirmDesc")}
          </AlertDialogDescription>
        </AlertDialogHeader>

        {step === "server" && (
          <div className="flex max-h-[320px] flex-col gap-2 overflow-y-auto pr-1">
            {loggedInConnections.length === 0 ? (
              <p className="m-0 rounded-md border border-dashed border-border p-3 text-sm text-muted-foreground">
                {t("chooser.noLoggedInServers")}
              </p>
            ) : (
              loggedInConnections.map((entry) => (
                <button
                  key={entry.id}
                  type="button"
                  className="flex items-center gap-3 rounded-lg border border-border p-3 text-left transition-colors hover:border-primary/50 hover:bg-muted/60"
                  onClick={() => void selectConnection(entry)}
                >
                  <span className="flex h-8 w-8 items-center justify-center rounded-md bg-muted text-muted-foreground">
                    <Server className="h-4 w-4" />
                  </span>
                  <span className="min-w-0">
                    <span className="block text-sm font-medium">{entry.name}</span>
                    <span className="block truncate text-xs text-muted-foreground">
                      {entry.origin}
                    </span>
                  </span>
                </button>
              ))
            )}
          </div>
        )}

        {step === "scopes" && (
          <ScopeSelection
            scopes={scopes}
            selected={scope}
            busy={busy}
            onSelect={setScope}
          />
        )}

        {(step === "confirm" || step === "running") && scope && connection && (
          <div className="flex flex-col gap-3">
            <div className="grid grid-cols-[1fr_auto_1fr_auto_1fr] items-center gap-2 rounded-lg border border-border bg-muted/25 p-3">
              <div className="min-w-0">
                <span className="block text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
                  {t("migration.routeLocal")}
                </span>
                <span className="block truncate font-mono text-xs" title={root}>
                  {root}
                </span>
              </div>
              <span className="text-muted-foreground" aria-hidden="true">→</span>
              <div className="text-center">
                <FolderArchive className="mx-auto mb-1 h-4 w-4 text-muted-foreground" />
                <span className="text-[11px] font-medium">{t("migration.routeBackup")}</span>
              </div>
              <span className="text-muted-foreground" aria-hidden="true">→</span>
              <div className="min-w-0 text-right">
                <span className="block text-[10px] font-semibold uppercase tracking-[0.14em] text-muted-foreground">
                  Server
                </span>
                <span className="block font-mono text-xs">{target}</span>
              </div>
            </div>
            <div className={`rounded-lg border px-3 py-2.5 text-xs leading-5 ${SEMANTIC_SURFACE.warning}`}>
              <strong className="block text-sm">{target}</strong>
              {t("migration.backupWarning")}
            </div>
            {step === "running" && (
              <p
                role="status"
                className={`m-0 flex items-center gap-2 rounded-md px-3 py-2 text-xs text-muted-foreground ${SEMANTIC_SURFACE.inProgress}`}
              >
                <LoaderCircle
                  className={`h-4 w-4 animate-spin motion-reduce:animate-none ${SEMANTIC_TONE.inProgress}`}
                />
                {t("migration.running")}
              </p>
            )}
          </div>
        )}

        {step === "success" && result && (
          <div className="flex flex-col gap-3">
            <div className={`flex items-start gap-3 rounded-lg border p-3 ${SEMANTIC_SURFACE.success}`}>
              <span
                className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-full ${SEMANTIC_SURFACE.success} ${SEMANTIC_TONE.success}`}
              >
                <Check className="h-4 w-4" />
              </span>
              <div className="min-w-0">
                <p className="m-0 text-sm font-semibold">
                  {message(t("migration.successCount"), {
                    count: String(result.report.documents.length),
                  })}
                </p>
                <p className="mt-1 mb-0 text-xs text-muted-foreground">
                  {t("migration.backupPath")}
                </p>
                <p className="mt-1 mb-0 break-all font-mono text-xs">{result.backupPath}</p>
              </div>
            </div>
          </div>
        )}

        {error && (
          <p
            role="alert"
            className="m-0 rounded-md bg-destructive/10 px-3 py-2 text-xs text-destructive"
          >
            {error}
          </p>
        )}

        <div className="flex items-center justify-between gap-3 border-t border-border pt-3">
          <div>
            {(step === "scopes" || step === "confirm") && (
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="gap-1"
                onClick={() => {
                  setError(null);
                  setStep(step === "confirm" ? "scopes" : "server");
                }}
              >
                <ArrowLeft className="h-3.5 w-3.5" /> {t("chooser.back")}
              </Button>
            )}
          </div>
          <div className="flex items-center gap-2">
            {step !== "success" && (
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={locked}
                onClick={() => onOpenChange(false)}
              >
                {t("app.cancel")}
              </Button>
            )}
            {step === "scopes" && (
              <Button
                type="button"
                size="sm"
                disabled={!scope || busy}
                onClick={() => setStep("confirm")}
              >
                {t("migration.review")}
              </Button>
            )}
            {step === "confirm" && (
              <Button type="button" size="sm" onClick={() => void runMigration()}>
                {t("migration.start")}
              </Button>
            )}
            {step === "success" && (
              <Button type="button" size="sm" onClick={() => onOpenChange(false)}>
                {t("migration.done")}
              </Button>
            )}
          </div>
        </div>
      </AlertDialogContent>
    </AlertDialog>
  );
}

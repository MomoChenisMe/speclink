import { useEffect, useState } from "react";
import { Cloud, Folder, FolderUp } from "lucide-react";
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
  useI18n,
} from "@speclink/ui";

import type { RemoteMarkerConflict } from "../store";

export interface RemoteConflictDialogProps {
  conflict: RemoteMarkerConflict | null;
  onContinueLocal: () => Promise<void>;
  onUseServer: () => Promise<void>;
  onMigrateLocal: () => Promise<void>;
}

export function RemoteConflictDialog({
  conflict,
  onContinueLocal,
  onUseServer,
  onMigrateLocal,
}: RemoteConflictDialogProps) {
  const { t } = useI18n();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setBusy(false);
    setError(null);
  }, [conflict?.path]);

  async function useServer() {
    setBusy(true);
    setError(null);
    try {
      await onUseServer();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  }

  async function migrateLocal() {
    setBusy(true);
    setError(null);
    try {
      await onMigrateLocal();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  }

  return (
    <AlertDialog open={conflict !== null} onOpenChange={() => {}}>
      <AlertDialogContent className="max-w-xl gap-4" data-testid="remote-conflict-dialog">
        <AlertDialogHeader>
          <AlertDialogTitle>{t("chooser.conflictTitle")}</AlertDialogTitle>
          <AlertDialogDescription>{t("chooser.conflictDesc")}</AlertDialogDescription>
        </AlertDialogHeader>
        <div className="rounded-md border border-border bg-muted/40 px-3 py-2 font-mono text-xs text-muted-foreground">
          {conflict?.path}
        </div>
        <div className="grid gap-2">
          <button
            type="button"
            disabled={busy}
            className="flex items-start gap-3 rounded-lg border border-border p-3 text-left transition-colors hover:border-primary/50 hover:bg-muted/60 disabled:opacity-50"
            onClick={() => void onContinueLocal()}
          >
            <Folder className="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground" />
            <span>
              <span className="block text-sm font-semibold">{t("chooser.continueLocal")}</span>
              <span className="mt-0.5 block text-xs leading-5 text-muted-foreground">
                {t("chooser.continueLocalDesc")}
              </span>
            </span>
          </button>
          <button
            type="button"
            disabled={busy}
            className="flex items-start gap-3 rounded-lg border border-amber-500/40 bg-amber-500/8 p-3 text-left transition-colors hover:bg-amber-500/15 disabled:opacity-50"
            onClick={() => void useServer()}
          >
            <Cloud className="mt-0.5 h-4 w-4 shrink-0 text-amber-700 dark:text-amber-300" />
            <span>
              <span className="block text-sm font-semibold">{t("chooser.useServer")}</span>
              <span className="mt-0.5 block text-xs leading-5 text-muted-foreground">
                {t("chooser.useServerDesc")}
              </span>
            </span>
          </button>
          <button
            type="button"
            disabled={busy}
            className="flex items-start gap-3 rounded-lg border border-primary/40 bg-primary/8 p-3 text-left transition-colors hover:bg-primary/12 disabled:opacity-50"
            onClick={() => void migrateLocal()}
          >
            <FolderUp className="mt-0.5 h-4 w-4 shrink-0 text-primary" />
            <span>
              <span className="block text-sm font-semibold">{t("chooser.migrateLocal")}</span>
              <span className="mt-0.5 block text-xs leading-5 text-muted-foreground">
                {t("chooser.migrateLocalDesc")}
              </span>
            </span>
          </button>
        </div>
        {error && (
          <p
            role="alert"
            className="m-0 rounded-md bg-destructive/10 px-3 py-2 text-xs text-destructive"
          >
            {error}
          </p>
        )}
      </AlertDialogContent>
    </AlertDialog>
  );
}

// 專案設定四頁共用的片段（desktop-settings-mode design D8）：欄位說明、未知值提示、解析失敗
// 說明、頁標題右側的編輯／取消／儲存、remote revision 衝突對照卡與頁標題下的檔案鍵註記。
import { AlertTriangle } from "lucide-react";
import {
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  SEMANTIC_SURFACE,
  SEMANTIC_TONE,
  cn,
  useI18n,
} from "@speclink/ui";

import type { PolicyConflict, ProjectSettingsState } from "./useProjectSettings";

export function FieldHelp({ children }: { children: React.ReactNode }) {
  return <p className="text-xs text-muted-foreground m-0">{children}</p>;
}

/** 選項集外儲存值的引導提示（spec 需求「設定頁政策下拉的未知值顯性呈現」）。 */
export function InvalidLocaleHint({ testId, children }: { testId: string; children: React.ReactNode }) {
  return (
    <p data-testid={testId} className={`flex items-start gap-1.5 text-xs m-0 ${SEMANTIC_TONE.warning}`}>
      <AlertTriangle className="h-3.5 w-3.5 shrink-0 mt-0.5" />
      {children}
    </p>
  );
}

export function ParseErrorBanner({ message }: { message: string }) {
  const { t } = useI18n();
  return (
    <p role="alert" className={`flex items-start gap-1.5 text-xs m-0 ${SEMANTIC_TONE.danger}`}>
      <AlertTriangle className="h-3.5 w-3.5 shrink-0 mt-0.5" />
      <span>
        {t("settings.parseErrorHint")} {message}
      </span>
    </p>
  );
}

/** 頁標題右側的編輯／取消／儲存（專案說明頁與產出規則頁共用）。 */
export function CardEditControls({
  editing,
  disabled,
  msg,
  testPrefix,
  onEdit,
  onCancel,
  onSave,
}: {
  editing: boolean;
  disabled: boolean;
  msg: string | null;
  /** data-testid 前綴（context／rules）。 */
  testPrefix: string;
  onEdit: () => void;
  onCancel: () => void;
  onSave: () => void;
}) {
  const { t } = useI18n();
  return (
    <div className="flex items-center gap-2">
      {msg && <span className="text-xs text-muted-foreground">{msg}</span>}
      {editing ? (
        <>
          <Button
            type="button"
            variant="outline"
            size="sm"
            data-testid={`${testPrefix}-cancel`}
            disabled={disabled}
            className="text-sm font-normal text-muted-foreground hover:text-foreground"
            onClick={onCancel}
          >
            {t("app.cancel")}
          </Button>
          <Button
            type="button"
            size="sm"
            data-testid={`${testPrefix}-save`}
            disabled={disabled}
            className="text-sm"
            onClick={onSave}
          >
            {t("settings.save")}
          </Button>
        </>
      ) : (
        <Button
          type="button"
          variant="outline"
          size="sm"
          data-testid={`${testPrefix}-edit`}
          disabled={disabled}
          className="text-sm font-normal text-muted-foreground hover:text-foreground"
          onClick={onEdit}
        >
          {t("settings.edit")}
        </Button>
      )}
    </div>
  );
}

/** remote 409 對照卡：任一頁的寫入都可能撞衝突，各頁頂部都呈現（review R5）。 */
export function ConflictPanel({
  conflict,
  onReload,
  onRetry,
}: {
  conflict: PolicyConflict | null;
  onReload: () => void;
  onRetry: () => Promise<void>;
}) {
  const { t } = useI18n();
  if (conflict === null) return null;
  return (
    <Card
      data-testid="policy-conflict-panel"
      className={cn(SEMANTIC_SURFACE.warning, "bg-muted/20")}
    >
      <CardHeader className="gap-1">
        <CardTitle className="text-base">{t("remote.conflictTitle")}</CardTitle>
        <p className="m-0 text-xs text-muted-foreground">{t("remote.conflictHint")}</p>
        <span
          data-testid="conflict-revision"
          className="font-mono text-xs tracking-wide text-muted-foreground"
        >
          {t("remote.conflictRevision")} {conflict.latest.workflow.revision ?? "—"}
        </span>
      </CardHeader>
      <CardContent className="gap-3">
        <div className="overflow-hidden rounded-md border border-border">
          <div className="grid grid-cols-[minmax(96px,0.7fr)_1fr_1fr] gap-px bg-border text-xs">
            <span className="bg-muted px-2 py-1.5 font-medium" />
            <span className="bg-muted px-2 py-1.5 font-medium">
              {t("remote.serverValue")}
            </span>
            <span className="bg-muted px-2 py-1.5 font-medium">
              {t("remote.myInput")}
            </span>
            {conflict.rows.map((row) => (
              <div
                key={row.key}
                data-testid={`conflict-row-${row.key}`}
                className="col-span-3 grid grid-cols-subgrid gap-px"
              >
                <span className="bg-card px-2 py-1.5 font-mono text-muted-foreground">
                  {row.label}
                </span>
                <span
                  data-testid="conflict-server"
                  className="whitespace-pre-wrap break-words bg-card px-2 py-1.5 font-mono"
                >
                  {row.server}
                </span>
                <span
                  data-testid="conflict-mine"
                  className="whitespace-pre-wrap break-words bg-card px-2 py-1.5 font-mono"
                >
                  {row.mine}
                </span>
              </div>
            ))}
          </div>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button type="button" size="sm" variant="outline" onClick={onReload}>
            {t("remote.reloadServer")}
          </Button>
          <Button type="button" size="sm" onClick={() => void onRetry()}>
            {t("remote.resubmitLatest")}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

/** 頁標題下的註記：local 為等寬檔案路徑與鍵，remote 為等寬 policy revision（檢視者另附角色說明）。 */
export function PageNote({
  remote,
  fileKey,
  testId,
}: {
  remote: ProjectSettingsState["remoteNote"];
  fileKey: string;
  testId: string;
}) {
  const { t } = useI18n();
  if (remote === null) {
    return (
      <span data-testid={testId} className="font-mono">
        {fileKey}
      </span>
    );
  }
  return (
    <span className="flex flex-col gap-2">
      <span data-testid="policy-revision" className="font-mono tracking-wide">
        {t("remote.policyRevision")} {remote.revision ?? "—"}
      </span>
      {remote.reader && (
        <span
          data-testid="policy-reader-note"
          className="rounded-md border border-border bg-muted/40 px-3 py-2 text-sm text-muted-foreground"
        >
          {t("remote.policyReader")}
        </span>
      )}
    </span>
  );
}

/** 頁頂：衝突對照卡＋該頁所掛設定檔的解析失敗說明。 */
export function PageAlerts({ ps, parseError }: { ps: ProjectSettingsState; parseError: string | null }) {
  return (
    <>
      <ConflictPanel conflict={ps.conflict} onReload={ps.reloadServerVersion} onRetry={ps.retryConflict} />
      {parseError !== null && <ParseErrorBanner message={parseError} />}
    </>
  );
}

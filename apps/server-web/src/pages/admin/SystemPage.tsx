import { useState, type ReactNode } from "react";
import { Compass, Database, Download, Server, TriangleAlert, type LucideIcon } from "lucide-react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Badge,
  Button,
  Card,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  SEMANTIC_SURFACE,
  SEMANTIC_TONE,
  useI18n,
} from "@speclink/ui";
import { useClient } from "../../app/context";
import { useAsync } from "../../lib/useAsync";
import { useTour } from "../../components/Tour";
import { AdminError, AdminLoading } from "./states";

// 系統頁（server-web-console「資料操作目的地已併入系統」）：原「資料操作」與「系統狀態」
// 合併為一個目的地與一份 view model。四個區段——執行環境、儲存狀態、匯出、危險區。
// 取得失敗時整頁呈現錯誤與重試，不部分渲染：儲存後端的健康狀態在整個介面只該有一處
// 權威來源，半份資料比沒有資料更容易誤導。

export function SystemPage() {
  const { t } = useI18n();
  const client = useClient();
  const { loading, data, error, reload } = useAsync(() => client.getAdminSystem(), []);
  const [confirmMigrate, setConfirmMigrate] = useState(false);
  const [busy, setBusy] = useState(false);
  const { restart: restartTour } = useTour();

  async function runMigrate() {
    if (busy) return;
    setBusy(true);
    try {
      await client.adminMigrate();
    } finally {
      setBusy(false);
      setConfirmMigrate(false);
      reload();
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h1 className="text-2xl font-semibold">{t("system.title")}</h1>
        {/* 導覽的重新啟動入口。放在系統頁而非各列表頁：它是「設定與維運」性質的動作，
            不是任何列表的 primary action。 */}
        <Button type="button" variant="outline" size="sm" className="gap-1.5" onClick={restartTour}>
          <Compass aria-hidden="true" className="h-4 w-4" />
          {t("tour.restart")}
        </Button>
      </div>
      {loading && <AdminLoading />}
      {error != null && <AdminError onRetry={reload} />}
      {data && (
        <>
          <Section id="runtime" title={t("system.runtime")} icon={Server}>
            <dl className="grid grid-cols-1 gap-3 sm:grid-cols-2">
              <Row label={t("system.engineVersion")} value={data.engineVersion} />
              <Row label={t("system.apiVersion")} value={data.apiVersion} />
              <Row label={t("system.schemaVersion")} value={data.identitySchemaVersion ?? t("common.dash")} />
              <Row label={t("system.storeDriver")} value={data.storeDriver} />
              <Row label={t("system.contractVersion")} value={data.storeContractVersion} />
              <Row label={t("system.storeLevel")} value={data.storeLevel} />
              <Row label={t("system.storeCapabilities")} value={data.storeCapabilities.join("、") || t("common.dash")} />
            </dl>
          </Section>

          <Section id="storage" title={t("system.storage")} icon={Database}>
            <div className="flex flex-wrap items-center gap-3">
              <Badge
                variant="outline"
                className={
                  data.storeHealthy
                    ? `${SEMANTIC_SURFACE.success} ${SEMANTIC_TONE.success}`
                    : `${SEMANTIC_SURFACE.danger} ${SEMANTIC_TONE.danger}`
                }
              >
                {data.storeHealthy ? t("common.normal") : t("common.abnormal")}
              </Badge>
              {!data.storeHealthy && data.storeHealthError && (
                <p className="text-sm text-destructive">{data.storeHealthError}</p>
              )}
            </div>
            {data.outboxBacklogs.length === 0 ? (
              <p className="mt-3 text-sm text-muted-foreground">{t("system.noBacklog")}</p>
            ) : (
              <Card className="mt-3 overflow-hidden">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>{t("system.colProject")}</TableHead>
                      <TableHead>{t("system.colRepo")}</TableHead>
                      <TableHead>{t("system.colBacklog")}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {data.outboxBacklogs.map((b) => (
                      <TableRow key={`${b.project}/${b.repo}`}>
                        <TableCell className="font-mono">{b.project}</TableCell>
                        <TableCell className="font-mono">{b.repo}</TableCell>
                        <TableCell className="tabular-nums">{b.backlog ?? t("common.dash")}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </Card>
            )}
          </Section>

          <Section id="export" title={t("system.export")} icon={Download}>
            {data.scopes.length === 0 ? (
              <p className="text-sm text-muted-foreground">{t("system.noScopes")}</p>
            ) : (
              <ul className="space-y-2">
                {data.scopes.map((s) => (
                  <li key={`${s.project}/${s.repo}`} className="flex items-center gap-3">
                    <span className="min-w-0 flex-1 truncate font-mono text-sm">
                      {s.project}/{s.repo}
                    </span>
                    {/* 檔案下載用原生 <a href>，不是 router Link。 */}
                    <Button asChild variant="outline" size="sm" className="gap-1.5">
                      <a
                        href={s.exportPath}
                        aria-label={t("system.exportScope").replace("{scope}", `${s.project}/${s.repo}`)}
                      >
                        <Download aria-hidden="true" className="h-3.5 w-3.5" />
                        {t("system.download")}
                      </a>
                    </Button>
                  </li>
                ))}
              </ul>
            )}
          </Section>

          <Section id="danger" title={t("system.danger")} icon={TriangleAlert}>
            <div className="flex flex-wrap items-center justify-between gap-3">
              <p className="text-sm text-muted-foreground">
                {t("system.migrateWarning")}
                {!data.migrateAvailable && t("system.migrateUnavailable")}
              </p>
              <Button
                type="button"
                variant="outline"
                disabled={busy || !data.migrateAvailable}
                onClick={() => setConfirmMigrate(true)}
              >
                {t("system.migrate")}
              </Button>
            </div>
          </Section>
        </>
      )}

      <AlertDialog open={confirmMigrate} onOpenChange={(open) => !open && setConfirmMigrate(false)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("system.migrateConfirm")}</AlertDialogTitle>
            <AlertDialogDescription>{t("system.migrateWarning")}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("common.cancel")}</AlertDialogCancel>
            <AlertDialogAction onClick={runMigrate} disabled={busy}>
              {t("system.migrateAction")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

function Section({
  id,
  title,
  icon: Icon,
  children,
}: {
  /** 區段的穩定識別（aria-labelledby 用），與顯示文字脫鉤——文字會隨語言改變。 */
  id: "runtime" | "storage" | "export" | "danger";
  title: string;
  icon: LucideIcon;
  children: ReactNode;
}) {
  const headingId = `system-${id}`;
  return (
    <section aria-labelledby={headingId} className="rounded-md border border-border p-4">
      <h2 id={headingId} className="flex items-center gap-2 text-lg font-medium">
        <Icon
          aria-hidden="true"
          className={`h-4 w-4 ${id === "danger" ? "text-destructive" : "text-muted-foreground"}`}
        />
        {title}
      </h2>
      <div className="mt-3">{children}</div>
    </section>
  );
}

function Row({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="rounded-md bg-muted/40 p-3">
      <dt className="text-sm text-muted-foreground">{label}</dt>
      <dd className="mt-1 break-all font-mono text-sm">{value}</dd>
    </div>
  );
}

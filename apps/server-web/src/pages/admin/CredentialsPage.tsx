import { useState } from "react";
import { Link } from "react-router-dom";
import { KeyRound } from "lucide-react";
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
  SelectItem,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  SEMANTIC_SURFACE,
  SEMANTIC_TONE,
  useI18n,
} from "@speclink/ui";
import { useClient } from "../../app/context";
import { useAsync } from "../../lib/useAsync";
import { ListToolbar, ToolbarSelect } from "../../components/ListToolbar";
import { DataList, type Column } from "../../components/DataList";
import { EmptyState, NoMatchState } from "../../components/EmptyState";
import { AdminError, AdminLoading } from "./states";
import type { AdminCredFamily, AdminPat } from "../../api/client";

// 管理憑證頁（server-admin, server-web-console「管理列表提供搜尋、篩選、分頁與具引導的
// 空狀態」）：全站存取金鑰與裝置憑證的 metadata（絕不呈現祕密——payload 亦無祕密），
// 以兩個分頁區分。撤銷是列尾的明確動作，先以 AlertDialog 確認後立即生效。

type Confirm = { title: string; run: () => Promise<void> };

type T = (key: string) => string;

function patColumns(t: T): Column<AdminPat>[] {
  return [
    { header: t("field.prefix"), primary: true, cell: (p) => <span className="font-mono">{p.prefix}</span> },
    { header: t("field.name"), cell: (p) => p.name },
    { header: t("field.user"), cell: (p) => <span className="font-mono">{p.userId}</span> },
    { header: t("field.created"), cell: (p) => fmtDate(p.createdAt, t("common.dash")) },
    { header: t("field.expires"), cell: (p) => fmtDate(p.expiresAt, t("common.forever")) },
    {
      header: t("field.status"),
      cell: (p) => (
        <Badge
          variant="outline"
          className={
            p.revokedAt
              ? "text-muted-foreground"
              : `${SEMANTIC_SURFACE.success} ${SEMANTIC_TONE.success}`
          }
        >
          {p.revokedAt ? t("common.revoked") : t("common.active")}
        </Badge>
      ),
    },
  ];
}

function deviceColumns(t: T): Column<AdminCredFamily>[] {
  return [
    { header: t("field.source"), primary: true, cell: (f) => f.source },
    { header: t("field.user"), cell: (f) => <span className="font-mono">{f.userId}</span> },
    { header: t("field.created"), cell: (f) => fmtDate(f.createdAt, t("common.dash")) },
    { header: t("field.lastRefresh"), cell: (f) => fmtDate(f.lastRefreshAt, t("common.dash")) },
    {
      header: t("field.status"),
      cell: (f) => (
        <Badge
          variant="outline"
          className={
            f.revokedAt
              ? "text-muted-foreground"
              : `${SEMANTIC_SURFACE.success} ${SEMANTIC_TONE.success}`
          }
        >
          {f.revokedAt ? t("common.revoked") : t("common.active")}
        </Badge>
      ),
    },
  ];
}

/** The date portion of an ISO timestamp. */
function fmtDate(iso: string | null, fallback: string): string {
  return iso ? iso.slice(0, 10) : fallback;
}

/** 關鍵字比對：以使用者看得到的欄位為準，不含不透明 id。 */
function matches(haystack: string[], keyword: string): boolean {
  const needle = keyword.trim().toLowerCase();
  if (!needle) return true;
  return haystack.some((field) => field.toLowerCase().includes(needle));
}

export function CredentialsPage() {
  const { t } = useI18n();
  const client = useClient();
  const { loading, data, error, reload } = useAsync(() => client.getAdminCredentials(), []);
  const [confirm, setConfirm] = useState<Confirm | null>(null);
  const [busy, setBusy] = useState(false);
  const [q, setQ] = useState("");
  const [status, setStatus] = useState("");

  async function runConfirmed() {
    if (!confirm || busy) return;
    setBusy(true);
    try {
      await confirm.run();
    } finally {
      setBusy(false);
      setConfirm(null);
      reload();
    }
  }

  const keep = (revokedAt: string | null) =>
    status === "" || (status === "active" ? revokedAt === null : revokedAt !== null);
  const pats = (data?.pats ?? []).filter(
    (p) => keep(p.revokedAt) && matches([p.prefix, p.name], q),
  );
  const devices = (data?.deviceFamilies ?? []).filter(
    (f) => keep(f.revokedAt) && matches([f.source], q),
  );
  const nothingAtAll =
    data != null && data.pats.length === 0 && data.deviceFamilies.length === 0;

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h1 className="text-2xl font-semibold">{t("credentials.title")}</h1>
        {/* 建立入口常駐而不只出現在空狀態：建完第一把之後就再也找不到新增的地方了。
            存取金鑰只能建給自己，所以指向帳號頁——這裡沒有代建的 API。 */}
        <Button asChild className="gap-1.5">
          <Link to="/account">
            <KeyRound aria-hidden="true" className="h-4 w-4" />
            {t("credentials.createKey")}
          </Link>
        </Button>
      </div>
      {loading && <AdminLoading />}
      {error != null && <AdminError onRetry={reload} />}

      {data &&
        (nothingAtAll ? (
          <EmptyState
            title={t("credentials.emptyTitle")}
            description={t("credentials.emptyBody")}
          />
        ) : (
          <>
            <ListToolbar search={q} onSearchChange={setQ}>
              <ToolbarSelect
                id="credentials-status"
                label={t("field.status")}
                allLabel={t("field.allStatuses")}
                value={status}
                onChange={setStatus}
              >
                <SelectItem value="active">{t("common.active")}</SelectItem>
                <SelectItem value="revoked">{t("common.revoked")}</SelectItem>
              </ToolbarSelect>
            </ListToolbar>

            <Tabs defaultValue="keys" className="space-y-4">
              <TabsList>
                <TabsTrigger value="keys">{t("credentials.tabKeys")}</TabsTrigger>
                <TabsTrigger value="devices">{t("credentials.tabDevices")}</TabsTrigger>
              </TabsList>

              <TabsContent value="keys">
                {pats.length === 0 ? (
                  <NoMatchState />
                ) : (
                  <DataList
                    items={pats}
                    columns={patColumns(t)}
                    keyOf={(p) => p.id}
                    action={(p) =>
                      p.revokedAt ? null : (
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          aria-label={t("credentials.revokeKey").replace("{name}", p.name)}
                          disabled={busy}
                          onClick={() =>
                            setConfirm({
                              title: t("credentials.revokeKeyTitle").replace("{name}", p.name),
                              run: () => client.adminRevokeToken(p.id),
                            })
                          }
                        >
                          {t("common.revoke")}
                        </Button>
                      )
                    }
                  />
                )}
              </TabsContent>

              <TabsContent value="devices">
                {devices.length === 0 ? (
                  <NoMatchState />
                ) : (
                  <DataList
                    items={devices}
                    columns={deviceColumns(t)}
                    keyOf={(f) => f.id}
                    action={(f) =>
                      f.revokedAt ? null : (
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          aria-label={t("credentials.revokeDevice").replace("{name}", f.source)}
                          disabled={busy}
                          onClick={() =>
                            setConfirm({
                              title: t("credentials.revokeDeviceTitle").replace("{name}", f.source),
                              run: () => client.adminRevokeFamily(f.id),
                            })
                          }
                        >
                          {t("common.revoke")}
                        </Button>
                      )
                    }
                  />
                )}
              </TabsContent>
            </Tabs>
          </>
        ))}

      <AlertDialog open={confirm !== null} onOpenChange={(open) => !open && setConfirm(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{confirm?.title}？</AlertDialogTitle>
            <AlertDialogDescription>{t("credentials.revokeBody")}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("common.cancel")}</AlertDialogCancel>
            <AlertDialogAction onClick={runConfirmed} disabled={busy}>
              {t("common.revoke")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

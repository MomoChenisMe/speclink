import { useState } from "react";
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
} from "@speclink/ui";
import { useClient } from "../../app/context";
import { useAsync } from "../../lib/useAsync";
import { AdminError, AdminLoading } from "./states";

// 管理憑證頁（server-admin, D4／D6）：全站 PAT 與裝置憑證家族的 metadata（絕不呈現
// 祕密——payload 亦無祕密）。強制撤銷為破壞性操作，先以 AlertDialog 確認後立即生效。

type Confirm = { title: string; run: () => Promise<void> };

/** The date portion of an ISO timestamp. */
function fmtDate(iso: string | null, fallback: string): string {
  return iso ? iso.slice(0, 10) : fallback;
}

export function CredentialsPage() {
  const client = useClient();
  const { loading, data, error, reload } = useAsync(() => client.getAdminCredentials(), []);
  const [confirm, setConfirm] = useState<Confirm | null>(null);
  const [busy, setBusy] = useState(false);

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

  return (
    <div className="space-y-8">
      <h1 className="text-2xl font-semibold">憑證</h1>
      {loading && <AdminLoading />}
      {error != null && <AdminError onRetry={reload} />}
      {data && (
        <>
          <section className="space-y-4">
            <h2 className="text-lg font-medium">Personal Access Tokens</h2>
            {data.pats.length === 0 ? (
              <p className="text-sm text-muted-foreground">尚無 PAT。</p>
            ) : (
              <Card className="overflow-hidden">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>前綴</TableHead>
                      <TableHead>名稱</TableHead>
                      <TableHead>使用者</TableHead>
                      <TableHead>建立</TableHead>
                      <TableHead>到期</TableHead>
                      <TableHead>狀態</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {data.pats.map((p) => (
                      <TableRow key={p.id}>
                        <TableCell className="font-mono">{p.prefix}</TableCell>
                        <TableCell>{p.name}</TableCell>
                        <TableCell className="font-mono">{p.userId}</TableCell>
                        <TableCell>{fmtDate(p.createdAt, "—")}</TableCell>
                        <TableCell>{fmtDate(p.expiresAt, "永久")}</TableCell>
                        <TableCell>
                          {p.revokedAt ? (
                            <Badge variant="outline">已撤銷</Badge>
                          ) : (
                            <Button
                              type="button"
                              variant="outline"
                              size="sm"
                              aria-label={`撤銷 PAT ${p.name}`}
                              disabled={busy}
                              onClick={() =>
                                setConfirm({
                                  title: `撤銷 PAT「${p.name}」`,
                                  run: () => client.adminRevokeToken(p.id),
                                })
                              }
                            >
                              撤銷
                            </Button>
                          )}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </Card>
            )}
          </section>

          <section className="space-y-4">
            <h2 className="text-lg font-medium">裝置憑證家族</h2>
            {data.deviceFamilies.length === 0 ? (
              <p className="text-sm text-muted-foreground">尚無裝置憑證。</p>
            ) : (
              <Card className="overflow-hidden">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>來源</TableHead>
                      <TableHead>使用者</TableHead>
                      <TableHead>建立</TableHead>
                      <TableHead>最近 refresh</TableHead>
                      <TableHead>狀態</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {data.deviceFamilies.map((f) => (
                      <TableRow key={f.id}>
                        <TableCell>
                          <Badge variant="secondary">{f.source}</Badge>
                        </TableCell>
                        <TableCell className="font-mono">{f.userId}</TableCell>
                        <TableCell>{fmtDate(f.createdAt, "—")}</TableCell>
                        <TableCell>{fmtDate(f.lastRefreshAt, "—")}</TableCell>
                        <TableCell>
                          {f.revokedAt ? (
                            <Badge variant="outline">已撤銷</Badge>
                          ) : (
                            <Button
                              type="button"
                              variant="outline"
                              size="sm"
                              aria-label={`撤銷裝置憑證 ${f.source}`}
                              disabled={busy}
                              onClick={() =>
                                setConfirm({
                                  title: `撤銷裝置憑證「${f.source}」`,
                                  run: () => client.adminRevokeFamily(f.id),
                                })
                              }
                            >
                              撤銷
                            </Button>
                          )}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </Card>
            )}
          </section>
        </>
      )}

      <AlertDialog open={confirm !== null} onOpenChange={(open) => !open && setConfirm(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{confirm?.title}？</AlertDialogTitle>
            <AlertDialogDescription>撤銷後立即生效且無法復原。</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction onClick={runConfirmed} disabled={busy}>
              撤銷
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

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
  Button,
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
              <div className="overflow-x-auto">
                <table className="w-full text-left text-sm">
                  <thead className="text-muted-foreground">
                    <tr>
                      <th className="py-1 pr-4 font-medium">前綴</th>
                      <th className="py-1 pr-4 font-medium">名稱</th>
                      <th className="py-1 pr-4 font-medium">使用者</th>
                      <th className="py-1 pr-4 font-medium">建立</th>
                      <th className="py-1 pr-4 font-medium">到期</th>
                      <th className="py-1 font-medium">狀態</th>
                    </tr>
                  </thead>
                  <tbody>
                    {data.pats.map((p) => (
                      <tr key={p.id} className="border-t">
                        <td className="py-1.5 pr-4 font-mono">{p.prefix}</td>
                        <td className="py-1.5 pr-4">{p.name}</td>
                        <td className="py-1.5 pr-4 font-mono">{p.userId}</td>
                        <td className="py-1.5 pr-4">{fmtDate(p.createdAt, "—")}</td>
                        <td className="py-1.5 pr-4">{fmtDate(p.expiresAt, "永久")}</td>
                        <td className="py-1.5">
                          {p.revokedAt ? (
                            <span className="text-muted-foreground">已撤銷</span>
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
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </section>

          <section className="space-y-4">
            <h2 className="text-lg font-medium">裝置憑證家族</h2>
            {data.deviceFamilies.length === 0 ? (
              <p className="text-sm text-muted-foreground">尚無裝置憑證。</p>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-left text-sm">
                  <thead className="text-muted-foreground">
                    <tr>
                      <th className="py-1 pr-4 font-medium">來源</th>
                      <th className="py-1 pr-4 font-medium">使用者</th>
                      <th className="py-1 pr-4 font-medium">建立</th>
                      <th className="py-1 pr-4 font-medium">最近 refresh</th>
                      <th className="py-1 font-medium">狀態</th>
                    </tr>
                  </thead>
                  <tbody>
                    {data.deviceFamilies.map((f) => (
                      <tr key={f.id} className="border-t">
                        <td className="py-1.5 pr-4">{f.source}</td>
                        <td className="py-1.5 pr-4 font-mono">{f.userId}</td>
                        <td className="py-1.5 pr-4">{fmtDate(f.createdAt, "—")}</td>
                        <td className="py-1.5 pr-4">{fmtDate(f.lastRefreshAt, "—")}</td>
                        <td className="py-1.5">
                          {f.revokedAt ? (
                            <span className="text-muted-foreground">已撤銷</span>
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
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
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

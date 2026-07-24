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

// 管理資料操作頁（server-admin, D4／D6）：列出各 scope 的匯出下載連結，並提供 store
// 遷移。匯出是檔案下載，用原生 <a href>（非 router Link）。遷移先以 AlertDialog 確認。

export function DataPage() {
  const client = useClient();
  const { loading, data, error, reload } = useAsync(() => client.getAdminData(), []);
  const [confirmMigrate, setConfirmMigrate] = useState(false);
  const [busy, setBusy] = useState(false);

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
    <div className="space-y-8">
      <h1 className="text-2xl font-semibold">資料操作</h1>
      {loading && <AdminLoading />}
      {error != null && <AdminError onRetry={reload} />}
      {data && (
        <>
          <section className="space-y-2">
            <h2 className="text-lg font-medium">Store 狀態</h2>
            <p className="text-sm">
              {data.storeHealthy ? "健康" : "異常"}
              {!data.storeHealthy && data.storeHealthError ? `：${data.storeHealthError}` : ""}
            </p>
          </section>

          <section className="space-y-4">
            <h2 className="text-lg font-medium">Scope 匯出</h2>
            {data.scopes.length === 0 ? (
              <p className="text-sm text-muted-foreground">尚無 scope。</p>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-left text-sm">
                  <thead className="text-muted-foreground">
                    <tr>
                      <th className="py-1 pr-4 font-medium">專案</th>
                      <th className="py-1 pr-4 font-medium">儲存庫</th>
                      <th className="py-1 font-medium">匯出</th>
                    </tr>
                  </thead>
                  <tbody>
                    {data.scopes.map((s) => (
                      <tr key={`${s.project}/${s.repo}`} className="border-t">
                        <td className="py-1.5 pr-4 font-mono">{s.project}</td>
                        <td className="py-1.5 pr-4 font-mono">{s.repo}</td>
                        <td className="py-1.5">
                          <a
                            href={s.exportPath}
                            aria-label={`匯出 ${s.project}/${s.repo}`}
                            className="text-primary underline underline-offset-4"
                          >
                            匯出
                          </a>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </section>

          <section className="space-y-3">
            <h2 className="text-lg font-medium">遷移</h2>
            <p className="text-sm text-muted-foreground">將 store 遷移至最新 schema。</p>
            <Button
              type="button"
              variant="outline"
              disabled={busy}
              onClick={() => setConfirmMigrate(true)}
            >
              遷移
            </Button>
          </section>
        </>
      )}

      <AlertDialog open={confirmMigrate} onOpenChange={(open) => !open && setConfirmMigrate(false)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>執行 store 遷移？</AlertDialogTitle>
            <AlertDialogDescription>遷移會變更底層資料結構，請先確認已備份。</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction onClick={runMigrate} disabled={busy}>
              遷移
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

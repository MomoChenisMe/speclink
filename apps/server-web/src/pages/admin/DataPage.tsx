import { useState } from "react";
import { Download } from "lucide-react";
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
          <section className="space-y-3">
            <h2 className="text-lg font-medium">Store 狀態</h2>
            <Card className="flex items-center gap-3 p-4">
              <Badge variant={data.storeHealthy ? "secondary" : "outline"}>
                {data.storeHealthy ? "健康" : "異常"}
              </Badge>
              {!data.storeHealthy && data.storeHealthError && (
                <p className="text-sm text-destructive">{data.storeHealthError}</p>
              )}
            </Card>
          </section>

          <section className="space-y-4">
            <h2 className="text-lg font-medium">Scope 匯出</h2>
            {data.scopes.length === 0 ? (
              <p className="text-sm text-muted-foreground">尚無 scope。</p>
            ) : (
              <Card className="overflow-hidden">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>專案</TableHead>
                      <TableHead>儲存庫</TableHead>
                      <TableHead>匯出</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {data.scopes.map((s) => (
                      <TableRow key={`${s.project}/${s.repo}`}>
                        <TableCell className="font-mono">{s.project}</TableCell>
                        <TableCell className="font-mono">{s.repo}</TableCell>
                        <TableCell>
                          <Button asChild variant="outline" size="sm" className="gap-1.5">
                            <a href={s.exportPath} aria-label={`匯出 ${s.project}/${s.repo}`}>
                              <Download aria-hidden="true" className="h-3.5 w-3.5" />
                              匯出
                            </a>
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </Card>
            )}
          </section>

          <section className="space-y-3">
            <h2 className="text-lg font-medium">遷移</h2>
            <Card className="flex flex-wrap items-center justify-between gap-3 p-4">
              <p className="text-sm text-muted-foreground">將 store 遷移至最新 schema。</p>
              <Button
                type="button"
                variant="outline"
                disabled={busy}
                onClick={() => setConfirmMigrate(true)}
              >
                遷移
              </Button>
            </Card>
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

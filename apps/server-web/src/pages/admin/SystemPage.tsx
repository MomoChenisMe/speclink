import {
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

// 管理系統狀態頁（server-admin, D4／D6）：唯讀呈現引擎／API 版本、store 驅動與能力、
// store 健康，以及各 scope 的 outbox backlog。

export function SystemPage() {
  const client = useClient();
  const { loading, data, error, reload } = useAsync(() => client.getAdminSystem(), []);

  return (
    <div className="space-y-8">
      <h1 className="text-2xl font-semibold">系統狀態</h1>
      {loading && <AdminLoading />}
      {error != null && <AdminError onRetry={reload} />}
      {data && (
        <>
          <dl className="grid grid-cols-1 gap-3 sm:grid-cols-2">
            <Row label="引擎版本" value={data.engineVersion} />
            <Row label="API 版本" value={data.apiVersion} />
            <Row label="Identity schema 版本" value={data.identitySchemaVersion ?? "—"} />
            <Row label="Store 驅動" value={data.storeDriver} />
            <Row label="Store 契約版本" value={data.storeContractVersion} />
            <Row label="Store 等級" value={data.storeLevel} />
            <Row label="Store 能力" value={data.storeCapabilities.join("、") || "—"} />
            <Row
              label="Store 健康"
              value={
                data.storeHealthy ? "健康" : `異常：${data.storeHealthError ?? "未知"}`
              }
            />
          </dl>

          <section className="space-y-4">
            <h2 className="text-lg font-medium">Outbox backlog</h2>
            {data.outboxBacklogs.length === 0 ? (
              <p className="text-sm text-muted-foreground">無 backlog 資料。</p>
            ) : (
              <Card className="overflow-hidden">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>專案</TableHead>
                      <TableHead>儲存庫</TableHead>
                      <TableHead>Backlog</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {data.outboxBacklogs.map((b) => (
                      <TableRow key={`${b.project}/${b.repo}`}>
                        <TableCell className="font-mono">{b.project}</TableCell>
                        <TableCell className="font-mono">{b.repo}</TableCell>
                        <TableCell className="tabular-nums">{b.backlog ?? "—"}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </Card>
            )}
          </section>
        </>
      )}
    </div>
  );
}

function Row({ label, value }: { label: string; value: string | number }) {
  return (
    <Card className="p-3">
      <dt className="text-sm text-muted-foreground">{label}</dt>
      <dd className="mt-1 font-mono text-sm break-all">{value}</dd>
    </Card>
  );
}

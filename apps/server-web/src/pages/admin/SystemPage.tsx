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
              <div className="overflow-x-auto">
                <table className="w-full text-left text-sm">
                  <thead className="text-muted-foreground">
                    <tr>
                      <th className="py-1 pr-4 font-medium">專案</th>
                      <th className="py-1 pr-4 font-medium">儲存庫</th>
                      <th className="py-1 font-medium">Backlog</th>
                    </tr>
                  </thead>
                  <tbody>
                    {data.outboxBacklogs.map((b) => (
                      <tr key={`${b.project}/${b.repo}`} className="border-t">
                        <td className="py-1.5 pr-4 font-mono">{b.project}</td>
                        <td className="py-1.5 pr-4 font-mono">{b.repo}</td>
                        <td className="py-1.5 tabular-nums">{b.backlog ?? "—"}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </section>
        </>
      )}
    </div>
  );
}

function Row({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="rounded-lg border p-3">
      <dt className="text-sm text-muted-foreground">{label}</dt>
      <dd className="mt-1 font-mono text-sm break-all">{value}</dd>
    </div>
  );
}

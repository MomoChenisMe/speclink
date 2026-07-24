import { useClient } from "../../app/context";
import { useAsync } from "../../lib/useAsync";

// 管理總覽：低成本摘要（D4）。載入／錯誤／成功三態一致且可恢復（D6）。
export function OverviewPage() {
  const client = useClient();
  const { loading, data, error, reload } = useAsync(() => client.getAdminOverview(), []);

  return (
    <div>
      <h1 className="text-2xl font-semibold">總覽</h1>
      {loading && (
        <p role="status" aria-live="polite" className="mt-4 text-muted-foreground">
          載入中…
        </p>
      )}
      {error != null && (
        <div role="alert" className="mt-4 space-y-2">
          <p className="text-destructive">載入失敗，發生錯誤。</p>
          <button
            type="button"
            onClick={reload}
            className="rounded-md border px-3 py-1.5 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            重試
          </button>
        </div>
      )}
      {data && (
        <dl className="mt-4 grid grid-cols-2 gap-4 sm:grid-cols-3">
          <Metric label="使用者" value={data.activeUsers} />
          <Metric label="停權" value={data.suspendedUsers} />
          <Metric label="專案" value={data.projects} />
          <Metric label="儲存庫" value={data.repos} />
          <Metric label="有效憑證" value={data.activeCredentials} />
          <Metric label="Schema 版本" value={data.identitySchemaVersion} />
        </dl>
      )}
    </div>
  );
}

function Metric({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-lg border p-3">
      <dt className="text-sm text-muted-foreground">{label}</dt>
      <dd className="mt-1 font-mono text-xl tabular-nums">{value}</dd>
    </div>
  );
}

import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { Button, Card } from "@speclink/ui";
import { useClient } from "../../app/context";
import { useAsync } from "../../lib/useAsync";
import { AdminError, AdminLoading } from "./states";
import type { AdminConnection } from "../../api/client";

// 管理總覽：低成本摘要（D4）。載入／錯誤／成功三態一致且可恢復（D6）。
// setup 完成後 Server 導向 `/admin?welcome=1`，此頁承接連線資訊的呈現
// （server-setup「完成 setup 即可邀請與連線」）；Store 降級狀態亦於此明示
// （server-admin「Store 不健康時 identity 管理仍可用」）。
export function OverviewPage() {
  const client = useClient();
  const [params] = useSearchParams();
  const welcome = params.get("welcome") === "1";
  const { loading, data, error, reload } = useAsync(() => client.getAdminOverview(), []);

  return (
    <div>
      <h1 className="text-2xl font-semibold">總覽</h1>
      {loading && (
        <div className="mt-4">
          <AdminLoading />
        </div>
      )}
      {error != null && (
        <div className="mt-4">
          <AdminError onRetry={reload} />
        </div>
      )}
      {data && (
        <>
          {welcome && data.connection && <WelcomeConnection connection={data.connection} />}
          {!data.storeHealthy && (
            <div
              role="alert"
              className="mt-6 rounded-md border border-destructive/40 bg-destructive/10 p-4 text-sm"
            >
              <p className="font-medium">儲存後端目前無法使用</p>
              <p className="mt-1 text-muted-foreground">
                使用者與憑證管理仍可操作；專案、儲存庫與系統資訊暫時無法讀取。
              </p>
              {data.storeHealthError && (
                <p className="mt-1 break-all font-mono text-xs">{data.storeHealthError}</p>
              )}
            </div>
          )}
          <dl className="mt-6 grid grid-cols-2 gap-4 sm:grid-cols-3">
            <Metric label="使用者" value={data.activeUsers} />
            <Metric label="停權" value={data.suspendedUsers} />
            <Metric label="專案" value={data.projects} />
            <Metric label="儲存庫" value={data.repos} />
            <Metric label="有效憑證" value={data.activeCredentials} />
            <Metric label="Schema 版本" value={data.identitySchemaVersion} />
          </dl>
        </>
      )}
    </div>
  );
}

// setup 完成後的開箱資訊：三個要素各自可複製，並指出下一步是邀請成員。
function WelcomeConnection({ connection }: { connection: AdminConnection }) {
  return (
    <section
      aria-labelledby="welcome-heading"
      className="mt-6 rounded-md border border-border bg-card p-4"
    >
      <h2 id="welcome-heading" className="text-lg font-medium">
        開始使用
      </h2>
      <p className="mt-1 text-sm text-muted-foreground">
        初始設定已完成。以下是連線所需的資訊；下一步是邀請成員加入這個 project。
      </p>
      <dl className="mt-4 space-y-3">
        <CopyRow label="Public URL" value={connection.publicUrl} />
        <CopyRow label="Project" value={connection.projectKey} />
        <CopyRow label="Repo" value={connection.repoKey} />
      </dl>
    </section>
  );
}

function CopyRow({ label, value }: { label: string; value: string }) {
  const [copied, setCopied] = useState(false);
  const copy = () => {
    void navigator.clipboard?.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };
  return (
    <div className="flex items-center gap-3">
      <dt className="w-24 shrink-0 text-sm text-muted-foreground">{label}</dt>
      <dd className="min-w-0 flex-1 truncate font-mono text-sm">{value}</dd>
      <Button type="button" variant="outline" size="sm" aria-label={`複製 ${label}`} onClick={copy}>
        {copied ? "已複製" : "複製"}
      </Button>
      <span role="status" aria-live="polite" className="sr-only">
        {copied ? `${label} 已複製` : ""}
      </span>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: number }) {
  return (
    <Card className="p-4">
      <dt className="text-sm text-muted-foreground">{label}</dt>
      <dd className="mt-1 font-mono text-2xl font-semibold tabular-nums">{value}</dd>
    </Card>
  );
}

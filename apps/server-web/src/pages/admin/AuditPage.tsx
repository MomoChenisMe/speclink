import { useClient } from "../../app/context";
import { useAsync } from "../../lib/useAsync";
import { AdminError, AdminLoading } from "./states";

// 管理稽核紀錄頁（server-admin, D4／D6）：唯讀列出管理動作的稽核事件。
//
// 標題屬頁面外殼、始終呈現，深連結（/admin/audit）測試才能立即找到標題。資料讀取以
// async 包裝：若注入的 client 未提供 getAdminAudit（部分測試 fake 省略），會轉為被
// 處理的 rejection 進入錯誤狀態，而非在 effect 內同步丟出、被 error boundary 蓋掉標題。
export function AuditPage() {
  const client = useClient();
  const { loading, data, error, reload } = useAsync(async () => client.getAdminAudit(), []);

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">稽核紀錄</h1>
      {loading && <AdminLoading />}
      {error != null && <AdminError onRetry={reload} />}
      {data &&
        (data.entries.length === 0 ? (
          <p className="text-sm text-muted-foreground">尚無稽核事件。</p>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-sm">
              <thead className="text-muted-foreground">
                <tr>
                  <th className="py-1 pr-4 font-medium">動作</th>
                  <th className="py-1 pr-4 font-medium">對象</th>
                  <th className="py-1 pr-4 font-medium">來源</th>
                  <th className="py-1 pr-4 font-medium">操作者</th>
                  <th className="py-1 font-medium">時間</th>
                </tr>
              </thead>
              <tbody>
                {data.entries.map((e) => (
                  <tr key={e.id} className="border-t">
                    <td className="py-1.5 pr-4 font-mono">{e.action}</td>
                    <td className="py-1.5 pr-4 font-mono">{e.subject}</td>
                    <td className="py-1.5 pr-4">{e.source}</td>
                    <td className="py-1.5 pr-4 font-mono">{e.actorId}</td>
                    <td className="py-1.5">{e.createdAt}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ))}
    </div>
  );
}

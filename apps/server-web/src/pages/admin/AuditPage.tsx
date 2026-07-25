import {
  Badge,
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
          <Card className="overflow-hidden">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>動作</TableHead>
                  <TableHead>對象</TableHead>
                  <TableHead>來源</TableHead>
                  <TableHead>操作者</TableHead>
                  <TableHead>時間</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {data.entries.map((e) => (
                  <TableRow key={e.id}>
                    <TableCell className="font-mono">{e.action}</TableCell>
                    <TableCell className="font-mono">{e.subject}</TableCell>
                    <TableCell>
                      <Badge variant="secondary">{e.source}</Badge>
                    </TableCell>
                    <TableCell className="font-mono">{e.actorId}</TableCell>
                    <TableCell>{e.createdAt}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </Card>
        ))}
    </div>
  );
}

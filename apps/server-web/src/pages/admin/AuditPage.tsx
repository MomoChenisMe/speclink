import { useState } from "react";
import { Badge, Button, SelectItem, useI18n } from "@speclink/ui";
import { useClient } from "../../app/context";
import { useAsync } from "../../lib/useAsync";
import { ListToolbar, ToolbarDate, ToolbarSelect } from "../../components/ListToolbar";
import { EmptyState } from "../../components/EmptyState";
import { DataList, type Column } from "../../components/DataList";
import { AdminError, AdminLoading } from "./states";
import type { AdminAuditEntry } from "../../api/client";

// 管理稽核紀錄頁（server-admin「稽核篩選與分頁由伺服器套用」）：唯讀清單。關鍵字、
// 動作、來源與時間區間篩選加分頁一律由伺服器計算——稽核事件隨營運單調增長，前端全量
// 載入再篩選會隨資料量線性劣化。此頁只把參數送出並呈現回傳的當頁事件與總頁數。

// 動作篩選的選項＝伺服器的封閉集合（crates/speclink-server/src/audit.rs 的 AuditAction）。
const ACTIONS = [
  "user-invited",
  "invitation-revoked",
  "user-suspended",
  "user-reactivated",
  "membership-changed",
  "admin-flag-changed",
  "project-created",
  "project-renamed",
  "repo-created",
  "repo-renamed",
  "token-revoked",
  "setup-completed",
  "scope-exported",
  "store-migrated",
  "backup-recorded",
];

const SOURCES = ["web", "api", "cli"];

// 動作與來源是伺服器的封閉集合；未知值原樣呈現而不猜——那代表伺服器新增了這裡還沒
// 跟上的種類，顯示原始字串比顯示空白誠實。
function auditColumns(t: (key: string) => string): Column<AdminAuditEntry>[] {
  const label = (namespace: string, value: string) => {
    const key = `audit.${namespace}.${value}`;
    const text = t(key);
    return text === key ? value : text;
  };
  return [
    {
      header: t("audit.colTime"),
      primary: true,
      cell: (e) => <span className="tabular-nums">{e.createdAt.slice(0, 19)}</span>,
    },
    { header: t("audit.colAction"), cell: (e) => label("action", e.action) },
    { header: t("audit.colSubject"), cell: (e) => <span className="font-mono">{e.subject}</span> },
    { header: t("audit.colActor"), cell: (e) => <span className="font-mono">{e.actorId}</span> },
    {
      header: t("audit.colSource"),
      cell: (e) => <Badge variant="secondary">{label("source", e.source)}</Badge>,
    },
  ];
}

export function AuditPage() {
  const { t } = useI18n();
  const client = useClient();
  const [q, setQ] = useState("");
  const [action, setAction] = useState("");
  const [source, setSource] = useState("");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [page, setPage] = useState(1);
  const { loading, data, error, reload } = useAsync(
    async () => client.getAdminAudit({ q, action, source, from, to, page }),
    [q, action, source, from, to, page],
  );

  // 任何篩選變動都回到第 1 頁：留在第 3 頁看新篩選的結果只會得到空白畫面。
  function narrow(apply: () => void) {
    apply();
    setPage(1);
  }

  const totalPages = data?.totalPages ?? 0;

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">{t("audit.title")}</h1>

      <ListToolbar search={q} onSearchChange={(v) => narrow(() => setQ(v))}>
        <ToolbarSelect
          id="audit-action"
          label={t("audit.filterAction")}
          allLabel={t("audit.filterAllActions")}
          value={action}
          onChange={(v) => narrow(() => setAction(v))}
        >
          {ACTIONS.map((value) => (
            <SelectItem key={value} value={value}>
              {t(`audit.action.${value}`)}
            </SelectItem>
          ))}
        </ToolbarSelect>
        <ToolbarSelect
          id="audit-source"
          label={t("audit.filterSource")}
          allLabel={t("audit.filterAllSources")}
          value={source}
          onChange={(v) => narrow(() => setSource(v))}
        >
          {SOURCES.map((value) => (
            <SelectItem key={value} value={value}>
              {t(`audit.source.${value}`)}
            </SelectItem>
          ))}
        </ToolbarSelect>
        <ToolbarDate
          id="audit-from"
          label={t("audit.filterFrom")}
          value={from}
          onChange={(v) => narrow(() => setFrom(v))}
        />
        <ToolbarDate id="audit-to" label={t("audit.filterTo")} value={to} onChange={(v) => narrow(() => setTo(v))} />
      </ListToolbar>

      {loading && <AdminLoading />}
      {error != null && <AdminError onRetry={reload} />}
      {data &&
        (data.entries.length === 0 ? (
          <EmptyState
            title={t("audit.emptyTitle")}
            description={t("audit.emptyBody")}
          />
        ) : (
          <DataList items={data.entries} columns={auditColumns(t)} keyOf={(e) => e.id} />
        ))}

      <div className="flex items-center justify-end gap-3">
        <span className="text-sm text-muted-foreground" aria-live="polite">
          {t("audit.page")
            .replace("{page}", String(page))
            .replace("{total}", String(Math.max(totalPages, 1)))}
        </span>
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={page <= 1}
          onClick={() => setPage((p) => Math.max(1, p - 1))}
        >
          {t("audit.prevPage")}
        </Button>
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={page >= totalPages}
          onClick={() => setPage((p) => p + 1)}
        >
          {t("audit.nextPage")}
        </Button>
      </div>
    </div>
  );
}

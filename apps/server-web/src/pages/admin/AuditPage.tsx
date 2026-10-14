import { useRef, useState } from "react";
import { Inbox } from "lucide-react";
import {
  EmptyState,
  Input,
  ListCard,
  ListPager,
  ListRow,
  PAGE_SIZE,
  SearchField,
  SelectItem,
  StatusPill,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  useI18n,
  PageHeader,
} from "@speclink/ui";
import { useClient } from "../../app/context";
import { useAsync } from "../../lib/useAsync";
import { useMediaQuery } from "../../lib/useMediaQuery";
import { FilterSelect } from "../../components/FilterSelect";
import { AdminError, AdminLoading } from "./states";

// 管理稽核紀錄頁（server-admin「管理 browser API 提供最小且完整的頁面 view model」；
// server-web-list-pages design D7）：唯讀清單。關鍵字、動作、來源與時間區間篩選加分頁一律由
// 伺服器計算——稽核事件隨營運單調增長，前端全量載入再篩選會隨資料量線性劣化。此頁只把參數
// （含每頁筆數）送出，呈現回傳的當頁事件，換頁列的「共 N 筆」取自回應的 total。

// 動作篩選的選項＝伺服器的封閉集合（crates/host/speclink-server/src/admin/audit.rs 的 AuditAction）。
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

const DATE_CLS = "h-8 w-36 text-[13px]";

/** 動作與來源的顯示名稱（總覽的最近活動共用）。兩者是伺服器的封閉集合；未知值原樣呈現而不猜——
 *  那代表伺服器新增了這裡還沒跟上的種類，顯示原始字串比顯示空白誠實。 */
export function auditLabel(t: (key: string) => string, namespace: "action" | "source", value: string) {
  const key = `audit.${namespace}.${value}`;
  const text = t(key);
  return text === key ? value : text;
}

export function AuditPage() {
  const { t } = useI18n();
  const client = useClient();
  const compact = useMediaQuery("(max-width: 1023px)");
  const [q, setQ] = useState("");
  const [action, setAction] = useState("");
  const [source, setSource] = useState("");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(PAGE_SIZE);
  const scrollRef = useRef<HTMLDivElement>(null);
  const { loading, data, error, reload } = useAsync(
    async () => client.getAdminAudit({ q, action, source, from, to, page, limit: pageSize }),
    [q, action, source, from, to, page, pageSize],
  );

  // 任何篩選或每頁筆數變動都回到第 1 頁：留在第 3 頁看新篩選的結果只會得到空白畫面。
  function narrow(apply: () => void) {
    apply();
    setPage(1);
  }

  function resetScroll() {
    if (scrollRef.current) scrollRef.current.scrollTop = 0;
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-4">
      <PageHeader
        level={1}
        title={t("audit.title")}
        description={t("audit.description")}
        actions={
          <>
            <SearchField
              label={t("common.search")}
              placeholder={t("audit.searchPlaceholder")}
              value={q}
              onChange={(v) => narrow(() => setQ(v))}
            />
            <FilterSelect
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
            </FilterSelect>
            <FilterSelect
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
            </FilterSelect>
            <span className="flex items-center gap-1.5">
              <Input
                type="date"
                aria-label={t("audit.filterFrom")}
                value={from}
                onChange={(e) => narrow(() => setFrom(e.target.value))}
                className={DATE_CLS}
              />
              <span aria-hidden="true" className="text-muted-foreground">
                –
              </span>
              <Input
                type="date"
                aria-label={t("audit.filterTo")}
                value={to}
                onChange={(e) => narrow(() => setTo(e.target.value))}
                className={DATE_CLS}
              />
            </span>
          </>
        }
      />

      {loading && <AdminLoading />}
      {error != null && <AdminError onRetry={reload} />}
      {data && (
        <ListCard
          scrollRef={scrollRef}
          footer={
            <ListPager
              compact={compact}
              page={page}
              pageCount={Math.max(data.totalPages, 1)}
              total={data.total}
              pageSize={pageSize}
              onPage={(next) => {
                setPage(next);
                resetScroll();
              }}
              onPageSize={(size) => {
                narrow(() => setPageSize(size));
                resetScroll();
              }}
            />
          }
        >
          {data.entries.length === 0 ? (
            <EmptyState icon={Inbox} title={t("audit.emptyTitle")} description={t("audit.emptyBody")} />
          ) : compact ? (
            data.entries.map((e) => (
              <ListRow
                key={e.id}
                data-audit={e.id}
                mono={false}
                leading={e.createdAt.slice(0, 10)}
                title={auditLabel(t, "action", e.action)}
                description={`${e.subject} · ${e.actorId}`}
                meta={<StatusPill tone="neutral">{auditLabel(t, "source", e.source)}</StatusPill>}
              />
            ))
          ) : (
            // 不用共用 Table：它外層的 overflow-x-auto 會自成捲動容器，表頭的 sticky 就黏在那層、
            // 隨列表卡捲走。直接放 <table>，由列表卡的捲動區負責兩個方向的捲動，表頭才固定得住。
            <table className="w-full caption-bottom text-sm">
              <TableHeader className="sticky top-0 z-10 bg-card">
                <TableRow>
                  <TableHead>{t("audit.colTime")}</TableHead>
                  <TableHead>{t("audit.colAction")}</TableHead>
                  <TableHead>{t("audit.colSubject")}</TableHead>
                  <TableHead>{t("audit.colActor")}</TableHead>
                  <TableHead>{t("audit.colSource")}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {data.entries.map((e) => (
                  <TableRow key={e.id}>
                    <TableCell className="tabular-nums">{e.createdAt.slice(0, 19)}</TableCell>
                    <TableCell>{auditLabel(t, "action", e.action)}</TableCell>
                    <TableCell className="font-mono">{e.subject}</TableCell>
                    <TableCell className="font-mono">{e.actorId}</TableCell>
                    <TableCell>
                      <StatusPill tone="neutral">{auditLabel(t, "source", e.source)}</StatusPill>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </table>
          )}
        </ListCard>
      )}
    </div>
  );
}

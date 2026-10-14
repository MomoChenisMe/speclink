import { useState } from "react";
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
  Table,
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
import { auditLabel } from "../../lib/auditLabel";
import { useListScroll } from "../../lib/useListScroll";
import { useNarrow } from "../../lib/useMediaQuery";
import { FilterSelect } from "../../components/FilterSelect";
import { AdminError, AdminLoading } from "./states";
import type { AdminAudit } from "../../api/client";

// 管理稽核紀錄頁（server-admin「管理 browser API 提供最小且完整的頁面 view model」；
// server-web-list-pages design D7）：唯讀清單。關鍵字、動作、來源與時間區間篩選加分頁一律由
// 伺服器計算——稽核事件隨營運單調增長，前端全量載入再篩選會隨資料量線性劣化。此頁只把參數
// （含每頁筆數）送出，呈現回傳的當頁事件，換頁列的「共 N 筆」取自回應的 total。換頁或改篩選
// 期間照舊顯示上一份回應，列表卡與換頁列不卸載——剛按的頁碼鈕留著，焦點不會掉回頁首。

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

export function AuditPage() {
  const { t } = useI18n();
  const client = useClient();
  const narrow = useNarrow();
  const [q, setQ] = useState("");
  const [action, setAction] = useState("");
  const [source, setSource] = useState("");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(PAGE_SIZE);
  const { scrollRef, scrollToTop } = useListScroll();
  // 回應帶著自己是用哪一頁抓的：頁碼拉回之後、舊回應清掉之前，仍認得出它是越界頁的回應。
  const { loading, data, error, reload } = useAsync(
    async () => ({ page, audit: await client.getAdminAudit({ q, action, source, from, to, page, limit: pageSize }) }),
    [q, action, source, from, to, page, pageSize],
  );
  // 重抓期間換頁列仍可按，可能點到新結果已不存在的頁（伺服器對越界頁碼回空頁）：這份空回應不顯示、
  // 也不留作「上一份」，頁碼拉回最後一頁重抓。
  const outOfRange = data != null && data.page > Math.max(data.audit.totalPages, 1);
  if (data && outOfRange && page === data.page) setPage(Math.max(data.audit.totalPages, 1));
  const fresh = data && !outOfRange ? data.audit : null;
  // 最近一次成功且頁碼有效的回應：重新抓取時 useAsync 會先清空 data，這裡留著上一份繼續顯示。
  const [lastData, setLastData] = useState<AdminAudit | null>(null);
  if (fresh && fresh !== lastData) setLastData(fresh);
  const shown = error == null ? (fresh ?? lastData) : null;

  // 任何篩選或每頁筆數變動都回到第 1 頁：留在第 3 頁看新篩選的結果只會得到空白畫面。
  function refilter(apply: () => void) {
    apply();
    setPage(1);
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
              onChange={(v) => refilter(() => setQ(v))}
            />
            <FilterSelect
              label={t("audit.filterAction")}
              allLabel={t("audit.filterAllActions")}
              value={action}
              onChange={(v) => refilter(() => setAction(v))}
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
              onChange={(v) => refilter(() => setSource(v))}
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
                onChange={(e) => refilter(() => setFrom(e.target.value))}
                className={DATE_CLS}
              />
              <span aria-hidden="true" className="text-muted-foreground">
                –
              </span>
              <Input
                type="date"
                aria-label={t("audit.filterTo")}
                value={to}
                onChange={(e) => refilter(() => setTo(e.target.value))}
                className={DATE_CLS}
              />
            </span>
          </>
        }
      />

      {loading && !shown && <AdminLoading />}
      {error != null && <AdminError onRetry={reload} />}
      {shown && (
        <ListCard
          scrollRef={scrollRef}
          footer={
            <ListPager
              compact={narrow}
              page={page}
              pageCount={Math.max(shown.totalPages, 1)}
              total={shown.total}
              pageSize={pageSize}
              onPage={(next) => {
                setPage(next);
                scrollToTop();
              }}
              onPageSize={(size) => {
                refilter(() => setPageSize(size));
                scrollToTop();
              }}
            />
          }
        >
          {shown.entries.length === 0 ? (
            <EmptyState icon={Inbox} title={t("audit.emptyTitle")} description={t("audit.emptyBody")} />
          ) : narrow ? (
            shown.entries.map((e) => (
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
            // 不包外層捲動容器：由列表卡的捲動區負責兩個方向的捲動，sticky 表頭才固定得住。
            <Table wrapped={false}>
              <TableHeader sticky>
                <TableRow>
                  <TableHead>{t("audit.colTime")}</TableHead>
                  <TableHead>{t("audit.colAction")}</TableHead>
                  <TableHead>{t("audit.colSubject")}</TableHead>
                  <TableHead>{t("audit.colActor")}</TableHead>
                  <TableHead>{t("audit.colSource")}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {shown.entries.map((e) => (
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
            </Table>
          )}
        </ListCard>
      )}
    </div>
  );
}

import { useState, type ReactNode } from "react";
import { Link, useSearchParams } from "react-router-dom";
import {
  Activity,
  ChevronRight,
  CircleAlert,
  FolderGit2,
  History,
  KeyRound,
  MailPlus,
  Users,
  type LucideIcon,
} from "lucide-react";
import { Badge, Button, Card, SEMANTIC_SURFACE, SEMANTIC_TONE, useI18n } from "@speclink/ui";
import { useClient } from "../../app/context";
import { useAsync } from "../../lib/useAsync";
import { AdminError, AdminLoading } from "./states";
import type { AdminConnection, AdminTodo } from "../../api/client";

// 管理總覽（server-web-console「總覽提供可行動入口與待辦」）：四張可點入對應目的地的
// 指標卡，加上需要處理、系統健康與最近活動三個區塊。指標是入口而非裝飾——看到數字的
// 下一步一定是去某一頁做事。沒有待處理事項時整塊不渲染，不留空標題與空清單。
//
// setup 完成後 Server 導向 `/admin?welcome=1`，此頁承接連線資訊的呈現
// （server-setup「完成 setup 即可邀請與連線」）；Store 降級狀態亦於此明示
// （server-admin「Store 不健康時 identity 管理仍可用」）。
export function OverviewPage() {
  const { t } = useI18n();
  const client = useClient();
  const [params] = useSearchParams();
  const welcome = params.get("welcome") === "1";
  const { loading, data, error, reload } = useAsync(() => client.getAdminOverview(), []);

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-semibold">{t("overview.title")}</h1>
      {loading && <AdminLoading />}
      {error != null && <AdminError onRetry={reload} />}
      {data && (
        <>
          {welcome && data.connection && <WelcomeConnection connection={data.connection} />}
          {!data.storeHealthy && (
            <div
              role="alert"
              className="rounded-md border border-destructive/40 bg-destructive/10 p-4 text-sm"
            >
              <p className="font-medium">{t("overview.storeDownTitle")}</p>
              <p className="mt-1 text-muted-foreground">{t("overview.storeDownBody")}</p>
              {data.storeHealthError && (
                <p className="mt-1 break-all font-mono text-xs">{data.storeHealthError}</p>
              )}
            </div>
          )}

          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
            <Metric
              to="/admin/users"
              icon={Users}
              label={t("overview.metricUsers")}
              value={data.activeUsers}
              hint={t("overview.metricUsersHint").replace("{n}", String(data.suspendedUsers))}
            />
            <Metric
              to="/admin/registry"
              icon={FolderGit2}
              label={t("overview.metricProjects")}
              value={data.projects}
              hint={t("overview.metricProjectsHint").replace("{n}", String(data.repos))}
            />
            <Metric
              to="/admin/credentials"
              icon={KeyRound}
              label={t("overview.metricCredentials")}
              value={data.activeCredentials}
              hint={t("overview.metricCredentialsHint")}
            />
            <Metric
              to="/admin/users"
              icon={MailPlus}
              label={t("overview.metricPending")}
              value={data.pendingInvitations}
              hint={t("overview.metricPendingHint")}
            />
          </div>

          {data.todos.length > 0 && (
            <Section id="todos" title={t("overview.todos")} icon={CircleAlert}>
              <ul className="space-y-2">
                {data.todos.map((todo) => (
                  <Todo key={todo.kind} todo={todo} />
                ))}
              </ul>
            </Section>
          )}

          <Section
            id="health"
            title={t("overview.health")}
            icon={Activity}
            link={{ to: "/admin/system", label: t("nav.system") }}
          >
            <div className="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm">
              <span className="flex items-center gap-2">
                {t("overview.healthLabel")}
                <Badge
                variant="outline"
                className={
                  data.storeHealthy
                    ? `${SEMANTIC_SURFACE.success} ${SEMANTIC_TONE.success}`
                    : `${SEMANTIC_SURFACE.danger} ${SEMANTIC_TONE.danger}`
                }
              >
                  {data.storeHealthy ? t("common.normal") : t("common.abnormal")}
                </Badge>
              </span>
              <span className="text-muted-foreground">
                {t("overview.schemaVersion").replace("{n}", String(data.identitySchemaVersion))}
              </span>
            </div>
          </Section>

          <Section
            id="recent"
            title={t("overview.recent")}
            icon={History}
            link={{ to: "/admin/audit", label: t("overview.recentAll") }}
          >
            {data.recentAudit.length === 0 ? (
              <p className="text-sm text-muted-foreground">{t("overview.recentEmpty")}</p>
            ) : (
              <ul className="space-y-1.5 text-sm">
                {data.recentAudit.map((e) => (
                  <li key={e.id} className="flex gap-3">
                    <span className="shrink-0 tabular-nums text-muted-foreground">
                      {e.createdAt.slice(0, 10)}
                    </span>
                    <span className="min-w-0 flex-1 truncate">{e.subject}</span>
                  </li>
                ))}
              </ul>
            )}
          </Section>
        </>
      )}
    </div>
  );
}

// 待辦的文案留在前端：wire 上只有封閉集合的 kind 與目的地，訊息與行動名稱由此對應。
const TODO_KEYS: Record<string, { message: string; action: string }> = {
  "no-active-credentials": {
    message: "overview.todoNoCredentials",
    action: "overview.todoNoCredentialsAction",
  },
  "pending-invitations": {
    message: "overview.todoPendingInvitations",
    action: "overview.todoPendingInvitationsAction",
  },
};

function Todo({ todo }: { todo: AdminTodo }) {
  const { t } = useI18n();
  const keys = TODO_KEYS[todo.kind];
  return (
    <li className="flex flex-wrap items-center justify-between gap-3">
      <span className="min-w-0 flex-1 text-sm">
        {keys ? t(keys.message).replace("{n}", String(todo.count)) : todo.kind}
      </span>
      <Button asChild variant="outline" size="sm">
        <Link to={todo.destination}>{keys ? t(keys.action) : t("overview.todoGo")}</Link>
      </Button>
    </li>
  );
}

function Section({
  id,
  title,
  icon: Icon,
  link,
  children,
}: {
  /** 區段的穩定識別（aria-labelledby 用），與顯示文字脫鉤——文字會隨語言改變。 */
  id: "todos" | "health" | "recent";
  title: string;
  icon: LucideIcon;
  link?: { to: string; label: string };
  children: ReactNode;
}) {
  const headingId = `overview-${id}`;
  return (
    <section aria-labelledby={headingId} className="rounded-md border border-border p-4">
      <div className="flex items-center justify-between gap-3">
        <h2 id={headingId} className="flex items-center gap-2 text-lg font-medium">
          <Icon aria-hidden="true" className="h-4 w-4 text-muted-foreground" />
          {title}
        </h2>
        {link && (
          <Link
            to={link.to}
            className="rounded-md text-sm text-muted-foreground hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            {link.label} →
          </Link>
        )}
      </div>
      <div className="mt-3">{children}</div>
    </section>
  );
}

// 指標卡本身就是連結：看到數字的下一步一定是去對應頁做事。
function Metric({
  to,
  icon: Icon,
  label,
  value,
  hint,
}: {
  to: string;
  icon: LucideIcon;
  label: string;
  value: number;
  hint: string;
}) {
  return (
    <Link
      to={to}
      className="rounded-md focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    >
      <Card className="p-4 transition-colors hover:bg-muted/50">
        <div className="flex items-center justify-between gap-2">
          <span className="flex items-center gap-1.5 text-sm text-muted-foreground">
            <Icon aria-hidden="true" className="h-4 w-4" />
            {label}
          </span>
          <ChevronRight aria-hidden="true" className="h-4 w-4 text-muted-foreground" />
        </div>
        <p className="mt-1 font-mono text-2xl font-semibold tabular-nums">{value}</p>
        <p className="text-xs text-muted-foreground">{hint}</p>
      </Card>
    </Link>
  );
}

// setup 完成後的開箱資訊：三個要素各自可複製，並指出下一步是邀請成員。
function WelcomeConnection({ connection }: { connection: AdminConnection }) {
  const { t } = useI18n();
  return (
    <section aria-labelledby="welcome-heading" className="rounded-md border border-border bg-card p-4">
      <h2 id="welcome-heading" className="text-lg font-medium">
        {t("overview.welcomeTitle")}
      </h2>
      <p className="mt-1 text-sm text-muted-foreground">{t("overview.welcomeBody")}</p>
      <dl className="mt-4 space-y-3">
        <CopyRow label={t("overview.welcomeUrl")} value={connection.publicUrl} />
        <CopyRow label={t("overview.welcomeProjectKey")} value={connection.projectKey} />
        <CopyRow label={t("overview.welcomeRepoKey")} value={connection.repoKey} />
      </dl>
    </section>
  );
}

function CopyRow({ label, value }: { label: string; value: string }) {
  const { t } = useI18n();
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
      <Button
        type="button"
        variant="outline"
        size="sm"
        aria-label={t("common.copyField").replace("{name}", label)}
        onClick={copy}
      >
        {copied ? t("common.copied") : t("common.copy")}
      </Button>
      <span role="status" aria-live="polite" className="sr-only">
        {copied ? t("common.copiedField").replace("{name}", label) : ""}
      </span>
    </div>
  );
}

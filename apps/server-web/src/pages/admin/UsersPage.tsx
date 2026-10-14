import { useState, type FormEvent } from "react";
import { useSearchParams } from "react-router-dom";
import { Mail, UserPlus } from "lucide-react";
import {
  Badge,
  Button,
  Checkbox,
  ConfirmDialog,
  CopyButton,
  EmptyState,
  Label,
  ListCard,
  ListPager,
  ListRow,
  SearchField,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  StatusPill,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  SEMANTIC_SURFACE,
  SEMANTIC_TONE,
  cn,
  useI18n,
  usePaging,
  PageHeader,
} from "@speclink/ui";
import { useClient } from "../../app/context";
import { useAsync } from "../../lib/useAsync";
import { useListScroll } from "../../lib/useListScroll";
import { useNarrow } from "../../lib/useMediaQuery";
import { userLabel } from "../../lib/userLabel";
import { readFormError } from "../../lib/formError";
import { Field } from "../../components/Field";
import { DetailSheet } from "../../components/DetailSheet";
import { FilterSelect } from "../../components/FilterSelect";
import { NoMatchState } from "../../components/NoMatchState";
import { AdminError, AdminLoading } from "./states";
import type { AdminPendingInvitation, AdminProject, AdminUser } from "../../api/client";

// 管理使用者頁（server-web-console「管理列表以抽屜承載建立與編輯」「使用者頁呈現尚未接受的
// 邀請」；server-web-list-pages design D2–D4）：頁標題區右側是搜尋、狀態篩選與邀請；其下一張
// 列表卡，卡頂「使用者｜邀請中」兩個分頁（分頁狀態在網址 ?tab=pending），卡底共用換頁列
// （前端換頁）。使用者列整列可點開細節抽屜、列內不含輸入控制項；邀請由頁面唯一 primary action
// 開啟抽屜。停權等破壞性動作維持確認框確認（確認鈕為紅系）；最後一位 active admin 受保護
// （canSuspend／canRemoveAdmin 為 false 時停用對應控制）。

type Confirm = { title: string; body: string; action: string; run: () => Promise<void> };

/** 邀請抽屜的專案挑選器固定停在這個值；它對不到任何選項，所以 trigger 顯示 placeholder。 */
const PICKER = "__picker__";

/** The date portion of an ISO timestamp. */
function fmtDate(iso: string | null, fallback = "—"): string {
  return iso ? iso.slice(0, 10) : fallback;
}

export function UsersPage() {
  const { t } = useI18n();
  const client = useClient();
  const narrow = useNarrow();
  const { loading, data, error, reload } = useAsync(() => client.getAdminUsers(), []);
  // 成員資格的專案選項來自 registry（專案代號為封閉集合，不開放自由輸入）。
  const registry = useAsync(() => client.getAdminRegistry(), []);
  const projects = registry.data?.projects ?? [];
  const [searchParams, setSearchParams] = useSearchParams();
  // 其他值或缺席都是使用者分頁。
  const tab = searchParams.get("tab") === "pending" ? "pending" : "users";
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [inviteOpen, setInviteOpen] = useState(false);
  const [token, setToken] = useState<string | null>(null);
  const [confirm, setConfirm] = useState<Confirm | null>(null);
  const [busy, setBusy] = useState(false);
  const [q, setQ] = useState("");
  const [status, setStatus] = useState("");
  const { scrollRef, scrollToTop } = useListScroll();
  // 以 id 選取而非快照：列表重載後抽屜內容跟著更新，不會停留在舊資料。
  const selected = data?.users.find((u) => u.id === selectedId) ?? null;
  const needle = q.trim().toLowerCase();
  const matches = (...fields: string[]) =>
    needle === "" || fields.some((field) => field.toLowerCase().includes(needle));
  const visible = (data?.users ?? []).filter(
    (u) => (status === "" || (status === "active" ? u.active : !u.active)) && matches(u.display, u.email),
  );
  const visiblePending = (data?.pending ?? []).filter((i) => matches(i.email, i.display));
  const users = usePaging(visible, { onMove: scrollToTop });
  const pending = usePaging(visiblePending, { onMove: scrollToTop });

  // 搜尋、篩選與切分頁都回到第 1 頁。
  function firstPage() {
    users.setPage(1);
    pending.setPage(1);
  }

  // 分頁寫進網址但不新增瀏覽紀錄：上一頁不會在兩個分頁間來回。
  function switchTab(next: string) {
    setSearchParams(next === "pending" ? { tab: "pending" } : {}, { replace: true });
    firstPage();
    scrollToTop();
  }

  async function runConfirmed() {
    if (!confirm || busy) return;
    setBusy(true);
    try {
      await confirm.run();
    } finally {
      setBusy(false);
      setConfirm(null);
      reload();
    }
  }

  async function runNow(fn: () => Promise<void>) {
    if (busy) return;
    setBusy(true);
    try {
      await fn();
    } finally {
      setBusy(false);
      reload();
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-4">
      <PageHeader
        level={1}
        title={t("users.title")}
        description={t("users.description")}
        actions={
          <>
            <SearchField
              label={t("common.search")}
              placeholder={t("users.searchPlaceholder")}
              value={q}
              onChange={(value) => {
                setQ(value);
                firstPage();
              }}
            />
            {tab === "users" && (
              <FilterSelect
                label={t("field.status")}
                allLabel={t("field.allStatuses")}
                value={status}
                onChange={(value) => {
                  setStatus(value);
                  firstPage();
                }}
              >
                <SelectItem value="active">{t("common.active")}</SelectItem>
                <SelectItem value="suspended">{t("common.suspended")}</SelectItem>
              </FilterSelect>
            )}
            <Button
              type="button"
              data-tour="list-primary"
              className="gap-1.5"
              onClick={() => setInviteOpen(true)}
            >
              <UserPlus aria-hidden="true" className="h-4 w-4" />
              {t("users.invite")}
            </Button>
          </>
        }
      />

      {token && (
        <div
          role="status"
          aria-live="polite"
          className={`rounded-md border p-4 ${SEMANTIC_SURFACE.success}`}
        >
          <p className="text-sm font-medium">{t("users.inviteLinkNotice")}</p>
          {/* 給受邀者的是可直接開啟的連結，不是要他自己拼網址的 token。 */}
          <div className="mt-1 flex items-start gap-3">
            <code className="min-w-0 flex-1 break-all font-mono text-sm">{inviteUrl(token)}</code>
            <CopyButton size="sm" value={inviteUrl(token)} label={t("common.copy")} />
          </div>
        </div>
      )}

      {loading && <AdminLoading />}
      {error != null && <AdminError onRetry={reload} />}
      {data && (
        <Tabs value={tab} onValueChange={switchTab} className="flex min-h-0 flex-1 flex-col">
          <ListCard
            scrollRef={scrollRef}
            header={
              // 計數是未經搜尋與篩選的總數。
              <TabsList variant="card">
                <TabsTrigger value="users" count={data.users.length}>
                  {t("users.tabUsers")}
                </TabsTrigger>
                <TabsTrigger value="pending" count={data.pending.length}>
                  {t("users.pendingTitle")}
                </TabsTrigger>
              </TabsList>
            }
            footer={
              tab === "users" ? (
                <ListPager key="users" compact={narrow} {...users.pager} />
              ) : (
                <ListPager key="pending" compact={narrow} {...pending.pager} />
              )
            }
          >
            <TabsContent value="users">
              {visible.length === 0 ? (
                <NoMatchState />
              ) : (
                users.pageItems.map((u) => (
                  <UserRow key={u.id} user={u} narrow={narrow} onOpen={() => setSelectedId(u.id)} />
                ))
              )}
            </TabsContent>
            <TabsContent value="pending">
              {data.pending.length === 0 ? (
                <EmptyState icon={Mail} title={t("users.pendingEmpty")} description={t("users.pendingHint")} />
              ) : visiblePending.length === 0 ? (
                <NoMatchState />
              ) : (
                <>
                  <p className="border-b border-border/70 px-4 py-3 text-[13px] text-muted-foreground">
                    {t("users.pendingHint")}
                  </p>
                  {pending.pageItems.map((i) => (
                    <PendingRow
                      key={i.id}
                      invitation={i}
                      busy={busy}
                      onRevoke={() =>
                        setConfirm({
                          title: t("users.revokeInviteTitle").replace("{email}", i.email),
                          body: t("users.revokeInviteBody"),
                          action: t("users.revokeInvite"),
                          run: () => client.adminRevokeInvitation(i.id),
                        })
                      }
                    />
                  ))}
                </>
              )}
            </TabsContent>
          </ListCard>
        </Tabs>
      )}

      {selected && (
        <UserDetailSheet
          user={selected}
          projects={projects}
          busy={busy}
          onClose={() => setSelectedId(null)}
          onSetMembership={(body) => runNow(() => client.adminSetMembership(selected.id, body))}
          onSetAdmin={(admin) => runNow(() => client.adminSetAdminFlag(selected.id, admin))}
          onSuspend={() =>
            setConfirm({
              title: t("users.suspendTitle").replace("{name}", selected.display),
              body: t("users.suspendBody"),
              action: t("users.suspend"),
              run: () => client.adminSuspend(selected.id),
            })
          }
          onReactivate={() => runNow(() => client.adminReactivate(selected.id))}
        />
      )}

      <InviteSheet
        open={inviteOpen}
        projects={projects}
        onOpenChange={setInviteOpen}
        onCreated={(created) => {
          setToken(created);
          setInviteOpen(false);
          reload();
        }}
      />

      <ConfirmDialog
        open={confirm !== null}
        onOpenChange={(open) => !open && setConfirm(null)}
        title={confirm?.title ?? ""}
        description={confirm?.body}
        confirmLabel={confirm?.action ?? ""}
        destructive
        busy={busy}
        onConfirm={runConfirmed}
      />
    </div>
  );
}

/** 受邀者要開的連結；token 只在建立當下拿得到，網址在前端拼。 */
function inviteUrl(token: string): string {
  return `${window.location.origin}/invite/${token}`;
}

/** 使用者列（design D4）：頭像字、名稱與 email、管理員籤、狀態籤、所屬專案數；窄螢幕把管理員
 *  併入 email 那一行、meta 只留狀態籤。整列可點開細節抽屜。 */
function UserRow({ user, narrow, onOpen }: { user: AdminUser; narrow: boolean; onOpen: () => void }) {
  const { t } = useI18n();
  const name = userLabel(user);
  const adminLabel = t("users.roleAdmin");
  const statusPill = (
    <StatusPill tone={user.active ? "success" : "warning"}>
      {user.active ? t("common.active") : t("common.suspended")}
    </StatusPill>
  );
  const projectCount = user.memberships.length;
  return (
    <ListRow
      data-user={user.id}
      mono={false}
      leading={
        <span
          data-avatar
          aria-hidden="true"
          className={cn(
            "flex h-7 w-7 items-center justify-center rounded-full text-xs font-medium",
            user.active ? "bg-primary/12 text-primary" : "bg-muted text-muted-foreground",
          )}
        >
          {Array.from(name)[0]}
        </span>
      }
      title={name}
      description={narrow && user.admin ? `${user.email} · ${adminLabel}` : user.email}
      meta={
        narrow ? (
          statusPill
        ) : (
          <>
            {user.admin && <StatusPill tone="neutral">{adminLabel}</StatusPill>}
            {statusPill}
            <span className="tabular-nums">
              {projectCount === 0
                ? t("users.noProjectsShort")
                : t("users.projectCount").replace("{n}", String(projectCount))}
            </span>
          </>
        )
      }
      onClick={onOpen}
    />
  );
}

// 待啟用邀請列：受邀者接受前沒有 user row，不能被停權、沒有憑證也沒有細節可看，所以是靜態列；
// 標示到期日，管理員才知道哪一筆快失效。撤回恆常顯示（破壞性動作，確認框指名受邀者）。
function PendingRow({
  invitation: i,
  busy,
  onRevoke,
}: {
  invitation: AdminPendingInvitation;
  busy: boolean;
  onRevoke: () => void;
}) {
  const { t } = useI18n();
  const role = i.admin ? t("users.roleAdmin") : t("users.roleMember");
  const meta = (i.memberships.length > 0 ? t("users.pendingMetaProjects") : t("users.pendingMeta"))
    .replace("{display}", () => i.display)
    .replace("{role}", role)
    .replace("{projects}", () => i.memberships.join("、"));
  return (
    <ListRow
      data-invite={i.id}
      leading={
        <span className="flex h-7 w-7 items-center justify-center rounded-full bg-muted text-muted-foreground">
          <Mail aria-hidden="true" className="h-3.5 w-3.5" />
        </span>
      }
      title={i.email}
      description={meta}
      meta={<span className="tabular-nums">{t("users.pendingExpires").replace("{date}", fmtDate(i.expiresAt))}</span>}
      actions={
        <Button
          type="button"
          variant="outline"
          size="sm"
          aria-label={t("users.revokeInviteFor").replace("{email}", i.email)}
          disabled={busy}
          onClick={onRevoke}
        >
          {t("users.revokeInvite")}
        </Button>
      }
    />
  );
}

// 細節抽屜：概要、成員資格、憑證與稽核四個分頁。憑證與稽核只在抽屜開啟時才讀取
// （列表頁本身不需要這兩份資料），並在前端裁切為該使用者的項目。
function UserDetailSheet({
  user,
  projects,
  busy,
  onClose,
  onSetMembership,
  onSetAdmin,
  onSuspend,
  onReactivate,
}: {
  user: AdminUser;
  projects: AdminProject[];
  busy: boolean;
  onClose: () => void;
  onSetMembership: (body: { projectKey: string; role: string; member: boolean }) => void;
  onSetAdmin: (admin: boolean) => void;
  onSuspend: () => void;
  onReactivate: () => void;
}) {
  const { t } = useI18n();
  return (
    <DetailSheet
      open
      onOpenChange={(open) => !open && onClose()}
      title={user.display}
      description={t("users.detailMeta")
        .replace("{email}", user.email)
        .replace("{role}", user.admin ? t("users.roleAdmin") : t("users.roleMember"))
        .replace("{date}", fmtDate(user.createdAt))}
      actions={
        user.active ? (
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={!user.canSuspend || busy}
            onClick={onSuspend}
          >
            {t("users.suspend")}
          </Button>
        ) : (
          <Button type="button" variant="outline" size="sm" disabled={busy} onClick={onReactivate}>
            {t("users.reactivate")}
          </Button>
        )
      }
    >
      <Tabs defaultValue="summary" className="flex min-h-0 flex-col gap-4">
        <TabsList>
          <TabsTrigger value="summary">{t("users.tabSummary")}</TabsTrigger>
          <TabsTrigger value="memberships">{t("users.tabMemberships")}</TabsTrigger>
          <TabsTrigger value="credentials">{t("users.tabCredentials")}</TabsTrigger>
          <TabsTrigger value="audit">{t("users.tabAudit")}</TabsTrigger>
        </TabsList>

        <TabsContent value="summary">
          <dl className="space-y-2 text-sm">
            <SummaryRow label={t("field.email")} value={user.email} />
            <SummaryRow label={t("field.status")} value={user.active ? t("common.active") : t("common.suspended")} />
            <SummaryRow label={t("field.created")} value={fmtDate(user.createdAt)} />
          </dl>
          <div className="mt-4 flex items-center gap-2">
            <Checkbox
              id={`admin-flag-${user.id}`}
              checked={user.admin}
              disabled={busy || (user.admin && !user.canRemoveAdmin)}
              onCheckedChange={(v) => onSetAdmin(v === true)}
            />
            <Label htmlFor={`admin-flag-${user.id}`}>{t("users.setAdmin")}</Label>
          </div>
        </TabsContent>

        <TabsContent value="memberships">
          <MembershipEditor user={user} projects={projects} busy={busy} onSet={onSetMembership} />
        </TabsContent>

        <TabsContent value="credentials">
          <UserCredentials userId={user.id} />
        </TabsContent>

        <TabsContent value="audit">
          <UserAudit userId={user.id} />
        </TabsContent>
      </Tabs>
    </DetailSheet>
  );
}

function SummaryRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex gap-3">
      <dt className="w-20 shrink-0 text-muted-foreground">{label}</dt>
      <dd className="min-w-0 break-all">{value}</dd>
    </div>
  );
}

// 成員資格：列出並可逐一移除；新增以「＋ 加入專案」展開的選單——專案來自 registry、
// 角色為 server 端固定集合（editor／reader），皆不開放自由輸入。
function MembershipEditor({
  user,
  projects,
  busy,
  onSet,
}: {
  user: AdminUser;
  projects: AdminProject[];
  busy: boolean;
  onSet: (body: { projectKey: string; role: string; member: boolean }) => void;
}) {
  const { t } = useI18n();
  const [adding, setAdding] = useState(false);
  const [projectKey, setProjectKey] = useState("");
  const [role, setRole] = useState("editor");

  function add(event: FormEvent) {
    event.preventDefault();
    if (busy || !projectKey) return;
    onSet({ projectKey, role, member: true });
    setProjectKey("");
    setRole("editor");
    setAdding(false);
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between gap-2">
        <h3 className="text-sm font-medium">{t("users.tabMemberships")}</h3>
        <Button type="button" variant="outline" size="sm" onClick={() => setAdding(true)}>
          {t("users.addProject")}
        </Button>
      </div>
      {user.memberships.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("users.noMemberships")}</p>
      ) : (
        <ul className="space-y-2">
          {user.memberships.map((m) => (
            <li
              key={`${m.projectKey}:${m.role}`}
              className="flex items-center gap-2 rounded-md border border-border px-3 py-2"
            >
              <span className="min-w-0 flex-1 truncate font-mono text-sm">{m.projectKey}</span>
              <Badge variant="outline">{m.role}</Badge>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                aria-label={t("users.removeMembership")
                  .replace("{name}", user.display)
                  .replace("{project}", m.projectKey)}
                disabled={busy}
                onClick={() => onSet({ projectKey: m.projectKey, role: m.role, member: false })}
              >
                {t("common.remove")}
              </Button>
            </li>
          ))}
        </ul>
      )}
      {adding && (
        <form onSubmit={add} className="flex flex-wrap items-end gap-2 border-t border-border pt-3">
          <div className="space-y-1.5">
            <Label htmlFor={`membership-project-${user.id}`}>{t("field.project")}</Label>
            {/* 未選時以 placeholder 呈現，而非一個空值選項——Radix Select 的 item
                不接受空字串，且「選擇專案…」本來就不是可送出的選項。 */}
            <Select value={projectKey} onValueChange={setProjectKey}>
              <SelectTrigger id={`membership-project-${user.id}`} className="w-36">
                <SelectValue placeholder={t("users.pickProject")} />
              </SelectTrigger>
              <SelectContent>
                {projects.map((p) => (
                  <SelectItem key={p.key} value={p.key}>
                    {p.key}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-1.5">
            <Label htmlFor={`membership-role-${user.id}`}>{t("field.role")}</Label>
            <Select value={role} onValueChange={setRole}>
              <SelectTrigger id={`membership-role-${user.id}`} className="w-28">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="editor">editor</SelectItem>
                <SelectItem value="reader">reader</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <Button type="submit" variant="outline" size="sm" disabled={busy || !projectKey}>
            {t("common.add")}
          </Button>
        </form>
      )}
    </div>
  );
}

function UserCredentials({ userId }: { userId: string }) {
  const { t } = useI18n();
  const client = useClient();
  const { loading, data, error, reload } = useAsync(() => client.getAdminCredentials(), [userId]);
  if (loading) return <AdminLoading />;
  if (error != null) return <AdminError onRetry={reload} />;
  const pats = data?.pats.filter((p) => p.userId === userId) ?? [];
  const devices = data?.deviceFamilies.filter((f) => f.userId === userId) ?? [];
  if (pats.length === 0 && devices.length === 0) {
    return <p className="text-sm text-muted-foreground">{t("users.noCredentials")}</p>;
  }
  return (
    <ul className="space-y-2 text-sm">
      {pats.map((p) => (
        <li key={p.id} className="flex items-center gap-2">
          <span className="font-mono">{p.prefix}</span>
          <span className="min-w-0 flex-1 truncate">{p.name}</span>
          <Badge
          variant="outline"
          className={
            p.revokedAt
              ? "text-muted-foreground"
              : `${SEMANTIC_SURFACE.success} ${SEMANTIC_TONE.success}`
          }
        >
            {p.revokedAt ? t("common.revoked") : t("common.active")}
          </Badge>
        </li>
      ))}
      {devices.map((f) => (
        <li key={f.id} className="flex items-center gap-2">
          <span className="min-w-0 flex-1 truncate">{f.source}</span>
          <Badge
          variant="outline"
          className={
            f.revokedAt
              ? "text-muted-foreground"
              : `${SEMANTIC_SURFACE.success} ${SEMANTIC_TONE.success}`
          }
        >
            {f.revokedAt ? t("common.revoked") : t("common.active")}
          </Badge>
        </li>
      ))}
    </ul>
  );
}

function UserAudit({ userId }: { userId: string }) {
  const { t } = useI18n();
  const client = useClient();
  const { loading, data, error, reload } = useAsync(() => client.getAdminAudit(), [userId]);
  if (loading) return <AdminLoading />;
  if (error != null) return <AdminError onRetry={reload} />;
  const entries = data?.entries.filter((e) => e.actorId === userId || e.subject === userId) ?? [];
  if (entries.length === 0) {
    return <p className="text-sm text-muted-foreground">{t("users.noAudit")}</p>;
  }
  return (
    <ul className="space-y-2 text-sm">
      {entries.map((e) => (
        <li key={e.id} className="flex gap-2">
          <span className="shrink-0 tabular-nums text-muted-foreground">{fmtDate(e.createdAt)}</span>
          <span className="min-w-0 flex-1 truncate">{e.action}</span>
        </li>
      ))}
    </ul>
  );
}

// 邀請抽屜：成功後關閉抽屜、由頁面以 aria-live 呈現一次性 token（token 進入 URL
// 成為邀請連結）。提交失敗保持開啟並保留非祕密輸入，錯誤置於對應欄位附近。
function InviteSheet({
  open,
  projects,
  onOpenChange,
  onCreated,
}: {
  open: boolean;
  projects: AdminProject[];
  onOpenChange: (open: boolean) => void;
  onCreated: (token: string) => void;
}) {
  const { t } = useI18n();
  const client = useClient();
  const [email, setEmail] = useState("");
  const [display, setDisplay] = useState("");
  const [memberships, setMemberships] = useState<string[]>([]);
  const [admin, setAdmin] = useState(false);
  const [pending, setPending] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    if (pending) return;
    setPending(true);
    setMessage(null);
    setFieldErrors({});
    try {
      const { token } = await client.adminInvite({ email, display, memberships, admin });
      setEmail("");
      setDisplay("");
      setMemberships([]);
      setAdmin(false);
      onCreated(token);
    } catch (error) {
      const read = readFormError(error, t("users.inviteError"));
      setMessage(read.message);
      setFieldErrors(read.fieldErrors);
    } finally {
      setPending(false);
    }
  }

  if (!open) return null;

  return (
    <DetailSheet open onOpenChange={onOpenChange} title={t("users.invite")}>
      <form onSubmit={onSubmit} className="space-y-4" noValidate>
        <Field
          id="invite-email"
          label={t("field.email")}
          value={email}
          onChange={setEmail}
          error={fieldErrors.email}
        />
        <Field
          id="invite-display"
          label={t("field.display")}
          value={display}
          onChange={setDisplay}
          error={fieldErrors.display}
        />
        <fieldset className="space-y-2">
          <legend className="text-sm font-medium">{t("users.inviteJoinProjects")}</legend>
          {/* 以下拉挑選再逐一移除，而不是每個專案一列勾選框：專案數量會成長，一整欄
              勾選框在十來個專案之後就不可用了。已選的專案不再出現在可選項裡。 */}
          {projects.length === 0 ? (
            <p className="text-sm text-muted-foreground">{t("users.inviteNoProjects")}</p>
          ) : (
            <>
              {/* 純挑選器，不持有選中值：value 固定綁在一個對不到任何選項的哨符上，
                  所以 trigger 永遠顯示 placeholder，而每次挑選都算一次值變動（同一個
                  專案移除後可以再加回來）。不用 key 重掛——在 Sheet 的 focus trap 裡
                  重掛 Select 會讓兩者互搶焦點直到爆堆疊。 */}
              <Select value={PICKER} onValueChange={(key) => setMemberships((prev) => [...prev, key])}>
                <SelectTrigger id="invite-memberships" aria-label={t("users.inviteJoinProjects")}>
                  <SelectValue placeholder={t("users.pickProject")} />
                </SelectTrigger>
                <SelectContent>
                  {projects
                    .filter((p) => !memberships.includes(p.key))
                    .map((p) => (
                      <SelectItem key={p.key} value={p.key}>
                        {p.name === p.key ? p.key : `${p.name}（${p.key}）`}
                      </SelectItem>
                    ))}
                </SelectContent>
              </Select>
              {memberships.length > 0 && (
                <ul className="space-y-2">
                  {memberships.map((key) => (
                    <li
                      key={key}
                      className="flex items-center gap-2 rounded-md border border-border px-3 py-2"
                    >
                      <span className="min-w-0 flex-1 truncate font-mono text-sm">{key}</span>
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        aria-label={t("users.removeFromInvite").replace("{project}", key)}
                        onClick={() =>
                          setMemberships((prev) => prev.filter((k) => k !== key))
                        }
                      >
                        {t("common.remove")}
                      </Button>
                    </li>
                  ))}
                </ul>
              )}
            </>
          )}
          {fieldErrors.memberships && (
            <p role="alert" className="text-sm text-destructive">
              {fieldErrors.memberships}
            </p>
          )}
        </fieldset>
        <div className="flex items-center gap-2">
          <Checkbox
            id="invite-admin"
            checked={admin}
            onCheckedChange={(v) => setAdmin(v === true)}
          />
          <Label htmlFor="invite-admin">{t("users.setAdmin")}</Label>
        </div>
        {message && Object.keys(fieldErrors).length === 0 && (
          <p role="alert" className="text-sm text-destructive">
            {message}
          </p>
        )}
        <div className="flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
            {t("common.cancel")}
          </Button>
          <Button type="submit" disabled={pending}>
            {pending ? t("common.submitting") : t("users.inviteSubmit")}
          </Button>
        </div>
      </form>
    </DetailSheet>
  );
}

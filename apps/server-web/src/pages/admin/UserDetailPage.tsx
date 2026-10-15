import { useState, type ReactNode } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { Trash2, UserX } from "lucide-react";
import {
  Button,
  ConfirmDialog,
  CopyButton,
  PageHeader,
  REVEAL_ON_GROUP_HOVER,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  SettingsChoice,
  SettingsRow,
  StatusPill,
  Switch,
  TabsContent,
  TabsTrigger,
  useI18n,
} from "@speclink/ui";
import { useClient } from "../../app/context";
import { useAsync, useLastData } from "../../lib/useAsync";
import { useNarrow } from "../../lib/useMediaQuery";
import { userLabel } from "../../lib/userLabel";
import { auditLabel } from "../../lib/auditLabel";
import { fmtDate } from "../../lib/fmtDate";
import { DetailTitle } from "../../layouts/DetailTitle";
import { CardNote } from "../../components/CardNote";
import { RevocationPill } from "../../components/RevocationPill";
import { RevokeButton } from "../../components/RevokeButton";
import { DetailNotFound, DetailTabs } from "./detail";
import { AdminCardState } from "./states";
import type { AdminProject, AdminUser } from "../../api/client";

// 使用者詳情頁（server-web-console「管理列表以詳情頁與對話框承載檢視與建立」；server-web-detail-pages
// design D5）：768px 置中欄，頁標題接狀態籤、頁級動作為停權或重新啟用；其下卡片標頭分頁——概要、
// 成員資格、憑證、稽核，分頁內容為設定型列。資料來自使用者區段（列表與詳情頁共用一份），變更後
// 重載區段；憑證與稽核只在這一頁讀，並在前端裁切為該使用者的項目。

type Confirm = { title: string; body: string; action: string; run: () => Promise<void> };

const ROLES = ["editor", "reader"];

function projectOption(p: AdminProject): string {
  return p.name === p.key ? p.key : `${p.name}（${p.key}）`;
}

export function UserDetailPage({
  users,
  sequence,
  projects,
  reload,
}: {
  users: AdminUser[];
  /** 列表目前呈現的項目（含搜尋與篩選）——‹ n / m › 的序列。 */
  sequence: AdminUser[];
  /** registry 的專案；還沒載入（或載入失敗）時為 null。 */
  projects: AdminProject[] | null;
  reload: () => void;
}) {
  const { t } = useI18n();
  const { id } = useParams();
  const user = users.find((u) => u.id === id);
  if (!user) {
    return (
      <DetailNotFound
        icon={UserX}
        title={t("users.notFoundTitle")}
        body={t("users.notFoundBody")}
        backLabel={t("users.backToList")}
        backTo="/admin/users"
      />
    );
  }
  return <UserDetail user={user} users={users} sequence={sequence} projects={projects} reload={reload} />;
}

function UserDetail({
  user,
  users,
  sequence,
  projects,
  reload,
}: {
  user: AdminUser;
  users: AdminUser[];
  sequence: AdminUser[];
  projects: AdminProject[] | null;
  reload: () => void;
}) {
  const { t } = useI18n();
  const client = useClient();
  const navigate = useNavigate();
  // 分頁是元件 state：‹ › 換使用者時停在同一個分頁。
  const [tab, setTab] = useState("summary");
  const [confirm, setConfirm] = useState<Confirm | null>(null);
  const [busy, setBusy] = useState(false);
  const credentials = useAsync(() => client.getAdminCredentials(), [user.id]);
  // 撤銷後重讀期間留著上一份成功回應，分頁計數不閃掉（design D5）。
  const shownCredentials = useLastData(credentials.data, credentials.error);
  // 存取金鑰與裝置登入同一種列：名稱、說明、撤銷鈕（只在有效時）與撤銷籤。
  const credentialRows = [
    ...(shownCredentials?.pats ?? [])
      .filter((p) => p.userId === user.id)
      .map((p) => ({
        key: `pat:${p.id}`,
        label: p.name,
        description: p.revokedAt
          ? t("users.credKeyRevokedMeta")
              .replace("{prefix}", p.prefix)
              .replace("{date}", fmtDate(p.revokedAt, t("common.dash")))
          : t("users.credKeyMeta")
              .replace("{prefix}", p.prefix)
              .replace("{expires}", fmtDate(p.expiresAt, t("common.forever")))
              .replace("{lastUsed}", fmtDate(p.lastUsedAt, t("common.never"))),
        revokedAt: p.revokedAt,
        revokeLabel: t("credentials.revokeKey").replace("{name}", p.name),
        revokeTitle: t("credentials.revokeKeyTitle").replace("{name}", p.name),
        revoke: () => client.adminRevokeToken(p.id),
      })),
    ...(shownCredentials?.deviceFamilies ?? [])
      .filter((f) => f.userId === user.id)
      .map((f) => ({
        key: `device:${f.id}`,
        label: f.source,
        description: t("users.credDeviceMeta").replace("{date}", fmtDate(f.lastRefreshAt, t("common.dash"))),
        revokedAt: f.revokedAt,
        revokeLabel: t("credentials.revokeDevice").replace("{name}", f.source),
        revokeTitle: t("credentials.revokeDeviceTitle").replace("{name}", f.source),
        revoke: () => client.adminRevokeFamily(f.id),
      })),
  ];

  const name = userLabel(user);
  const index = sequence.findIndex((u) => u.id === user.id);

  function done() {
    reload();
    credentials.reload();
  }

  async function runConfirmed() {
    if (!confirm || busy) return;
    setBusy(true);
    try {
      await confirm.run();
    } finally {
      setBusy(false);
      setConfirm(null);
      done();
    }
  }

  async function runNow(fn: () => Promise<void>) {
    if (busy) return;
    setBusy(true);
    try {
      await fn();
    } finally {
      setBusy(false);
      done();
    }
  }

  return (
    <div className="mx-auto flex w-full max-w-[768px] flex-col gap-6">
      <DetailTitle
        label={name}
        index={index}
        total={sequence.length}
        onStep={(delta) => navigate(`/admin/users/${sequence[index + delta].id}`, { replace: true })}
      />
      <PageHeader
        level={1}
        title={name}
        badge={
          <StatusPill tone={user.active ? "success" : "warning"}>
            {user.active ? t("common.active") : t("common.suspended")}
          </StatusPill>
        }
        description={t("users.detailMeta")
          .replace("{email}", user.email)
          .replace("{role}", user.admin ? t("users.roleAdmin") : t("users.roleMember"))
          .replace("{date}", fmtDate(user.createdAt, t("common.dash")))}
        actions={
          user.active ? (
            <Button
              type="button"
              variant="outline"
              disabled={!user.canSuspend || busy}
              onClick={() =>
                setConfirm({
                  title: t("users.suspendTitle").replace("{name}", name),
                  body: t("users.suspendBody"),
                  action: t("users.suspend"),
                  run: () => client.adminSuspend(user.id),
                })
              }
            >
              {t("users.suspend")}
            </Button>
          ) : (
            <Button
              type="button"
              variant="outline"
              disabled={busy}
              onClick={() => runNow(() => client.adminReactivate(user.id))}
            >
              {t("users.reactivate")}
            </Button>
          )
        }
      />

      <DetailTabs
        value={tab}
        onValueChange={setTab}
        triggers={
          <>
            <TabsTrigger value="summary">{t("users.tabSummary")}</TabsTrigger>
            <TabsTrigger value="memberships" count={user.memberships.length}>
              {t("users.tabMemberships")}
            </TabsTrigger>
            <TabsTrigger value="credentials" count={shownCredentials ? credentialRows.length : undefined}>
              {t("users.tabCredentials")}
            </TabsTrigger>
            <TabsTrigger value="audit">{t("users.tabAudit")}</TabsTrigger>
          </>
        }
      >
        <TabsContent value="summary" className="divide-y divide-border">
          <SettingsRow
            label={t("field.email")}
            control={
              <>
                <span className="font-mono text-[13px]">{user.email}</span>
                <CopyButton value={user.email} label={t("common.copyField").replace("{name}", t("field.email"))} />
              </>
            }
          />
          <SettingsRow
            label={t("field.status")}
            control={
              <StatusPill tone={user.active ? "success" : "warning"}>
                {user.active ? t("common.active") : t("common.suspended")}
              </StatusPill>
            }
          />
          <SettingsRow
            label={t("field.created")}
            control={
              <span className="font-mono text-xs tabular-nums text-muted-foreground">
                {fmtDate(user.createdAt, t("common.dash"))}
              </span>
            }
          />
          <SettingsRow
            label={t("users.roleAdmin")}
            htmlFor={`admin-flag-${user.id}`}
            description={t("users.adminHint")}
            control={
              <Switch
                id={`admin-flag-${user.id}`}
                // 按鈕的名稱 axe 只認 aria-*，不認 <label for>。
                aria-label={t("users.roleAdmin")}
                checked={user.admin}
                disabled={busy || (user.admin && !user.canRemoveAdmin)}
                onCheckedChange={(admin) => runNow(() => client.adminSetAdminFlag(user.id, admin))}
              />
            }
          />
        </TabsContent>

        <TabsContent value="memberships" className="divide-y divide-border">
          <Memberships
            user={user}
            projects={projects}
            busy={busy}
            onSet={(body) => runNow(() => client.adminSetMembership(user.id, body))}
            onRemove={(projectKey, role) =>
              setConfirm({
                title: t("users.removeMembershipTitle").replace("{name}", name).replace("{project}", projectKey),
                body: t("users.removeMembershipBody"),
                action: t("common.remove"),
                run: () => client.adminSetMembership(user.id, { projectKey, role, member: false }),
              })
            }
          />
        </TabsContent>

        <TabsContent value="credentials" className="divide-y divide-border">
          {!shownCredentials ? (
            <AdminCardState error={credentials.error} onRetry={credentials.reload} />
          ) : credentialRows.length === 0 ? (
            <CardNote>{t("users.noCredentials")}</CardNote>
          ) : (
            credentialRows.map((row) => (
              <SettingsRow
                key={row.key}
                label={row.label}
                description={row.description}
                control={
                  <>
                    {!row.revokedAt && (
                      <RevokeButton
                        label={row.revokeLabel}
                        disabled={busy}
                        onClick={() =>
                          setConfirm({
                            title: row.revokeTitle,
                            body: t("credentials.revokeBody"),
                            action: t("common.revoke"),
                            run: row.revoke,
                          })
                        }
                      />
                    )}
                    <RevocationPill revokedAt={row.revokedAt} />
                  </>
                }
              />
            ))
          )}
        </TabsContent>

        <TabsContent value="audit" className="divide-y divide-border">
          <UserAudit userId={user.id} users={users} />
        </TabsContent>
      </DetailTabs>

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

/** 設定型列；窄螢幕時控制項換到標題下方整列、可換行（design D11）。 */
function ControlRow({
  label,
  description,
  children,
}: {
  label: ReactNode;
  description?: ReactNode;
  children: ReactNode;
}) {
  const narrow = useNarrow();
  const controls = <div className="flex flex-wrap items-center gap-2">{children}</div>;
  return (
    <SettingsRow label={label} description={description} control={narrow ? null : controls}>
      {narrow ? controls : null}
    </SettingsRow>
  );
}

const ROLE_OPTIONS = ROLES.map((value) => ({ value, label: value }));

type SetMembership = (body: { projectKey: string; role: string; member: boolean }) => void;

// 成員資格：每列就地改角色（伺服器的成員資格寫入是 upsert）、移除先確認；末列加入專案——專案來自
// registry 且只列尚未加入的，角色為伺服器的固定集合，皆不開放自由輸入。
function Memberships({
  user,
  projects,
  busy,
  onSet,
  onRemove,
}: {
  user: AdminUser;
  projects: AdminProject[] | null;
  busy: boolean;
  onSet: SetMembership;
  onRemove: (projectKey: string, role: string) => void;
}) {
  const { t } = useI18n();

  return (
    <>
      {user.memberships.length === 0 && <CardNote>{t("users.noMemberships")}</CardNote>}
      {user.memberships.map((m) => {
        const projectName = projects?.find((p) => p.key === m.projectKey)?.name;
        return (
          <ControlRow
            key={m.projectKey}
            label={projectName ?? m.projectKey}
            description={<span className="font-mono">{m.projectKey}</span>}
          >
            <SettingsChoice
              value={m.role}
              options={ROLE_OPTIONS}
              ariaLabel={t("users.roleFor").replace("{project}", m.projectKey)}
              disabled={busy}
              onChange={(next) => onSet({ projectKey: m.projectKey, role: next, member: true })}
            />
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              aria-label={t("users.removeMembership").replace("{name}", userLabel(user)).replace("{project}", m.projectKey)}
              disabled={busy}
              className={REVEAL_ON_GROUP_HOVER}
              onClick={() => onRemove(m.projectKey, m.role)}
            >
              <Trash2 aria-hidden="true" className="h-4 w-4" />
            </Button>
          </ControlRow>
        );
      })}
      {/* 以使用者為 key：‹ › 換人時選到一半的專案與角色清空，不會套到下一位身上。 */}
      <AddMembership key={user.id} user={user} projects={projects} busy={busy} onSet={onSet} />
    </>
  );
}

/** 末列「加入專案」：只列尚未加入的專案；專案清單還沒載入時不宣稱「已加入全部專案」。 */
function AddMembership({
  user,
  projects,
  busy,
  onSet,
}: {
  user: AdminUser;
  projects: AdminProject[] | null;
  busy: boolean;
  onSet: SetMembership;
}) {
  const { t } = useI18n();
  const [projectKey, setProjectKey] = useState("");
  const [role, setRole] = useState("editor");
  const joined = new Set(user.memberships.map((m) => m.projectKey));
  const available = (projects ?? []).filter((p) => !joined.has(p.key));
  const placeholder =
    projects === null ? "users.projectsNotLoaded" : available.length === 0 ? "users.allProjectsJoined" : "users.pickProject";

  return (
    <ControlRow label={t("users.addProjectTitle")} description={t("users.addProjectHint")}>
      {/* 未選時以 placeholder 呈現，而非一個空值選項——Radix Select 的 item 不接受空字串。 */}
      <Select value={projectKey} onValueChange={setProjectKey} disabled={available.length === 0}>
        <SelectTrigger aria-label={t("users.addProjectPick")} className="w-auto min-w-40 text-[13px]">
          <SelectValue placeholder={t(placeholder)} />
        </SelectTrigger>
        <SelectContent>
          {available.map((p) => (
            <SelectItem key={p.key} value={p.key}>
              {projectOption(p)}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <SettingsChoice value={role} options={ROLE_OPTIONS} ariaLabel={t("users.addProjectRole")} onChange={setRole} />
      <Button
        type="button"
        variant="outline"
        size="sm"
        disabled={busy || !projectKey}
        onClick={() => {
          onSet({ projectKey, role, member: true });
          setProjectKey("");
          setRole("editor");
        }}
      >
        {t("common.add")}
      </Button>
    </ControlRow>
  );
}

// 稽核：讀稽核紀錄第一頁再裁切為該使用者的事件（操作者或對象是他）；完整查詢在稽核紀錄頁。
function UserAudit({ userId, users }: { userId: string; users: AdminUser[] }) {
  const { t } = useI18n();
  const client = useClient();
  const { loading, data, error, reload } = useAsync(() => client.getAdminAudit(), [userId]);
  if (loading || error != null) return <AdminCardState error={error} onRetry={reload} />;
  const entries = data?.entries.filter((e) => e.actorId === userId || e.subject === userId) ?? [];
  if (entries.length === 0) return <CardNote>{t("users.noAudit")}</CardNote>;
  return (
    <>
      {entries.map((e) => {
        const actor = users.find((u) => u.id === e.actorId);
        return (
          <SettingsRow
            key={e.id}
            label={auditLabel(t, "action", e.action)}
            description={actor ? userLabel(actor) : e.actorId}
            control={
              <span className="font-mono text-xs tabular-nums text-muted-foreground">
                {fmtDate(e.createdAt, t("common.dash"))}
              </span>
            }
          />
        );
      })}
    </>
  );
}

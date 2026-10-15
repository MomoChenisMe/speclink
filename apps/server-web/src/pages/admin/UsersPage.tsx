import { useState, type FormEvent } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { Mail, UserPlus, X } from "lucide-react";
import {
  Button,
  ConfirmDialog,
  EmptyState,
  ListCard,
  ListPager,
  ListRow,
  SearchField,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  SettingsRow,
  StatusPill,
  Switch,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  cn,
  useI18n,
  usePaging,
  PageHeader,
} from "@speclink/ui";
import { useClient } from "../../app/context";
import { useAsync, useLastData } from "../../lib/useAsync";
import { useListScroll } from "../../lib/useListScroll";
import { useNarrow } from "../../lib/useMediaQuery";
import { userLabel } from "../../lib/userLabel";
import { fmtDate } from "../../lib/fmtDate";
import { Field } from "../../components/Field";
import {
  FormDialog,
  FormDialogActions,
  FormMessage,
  SecretDone,
  useDialogSubmit,
} from "../../components/FormDialog";
import { FilterSelect } from "../../components/FilterSelect";
import { NoMatchState } from "../../components/NoMatchState";
import { SectionRoutes } from "./detail";
import { AdminError, AdminLoading } from "./states";
import { UserDetailPage } from "./UserDetailPage";
import type { AdminPendingInvitation, AdminProject, AdminUser } from "../../api/client";

// 使用者區段（server-web-console「管理列表以詳情頁與對話框承載檢視與建立」「使用者頁呈現尚未接受的
// 邀請」；server-web-list-pages design D2–D4；server-web-detail-pages design D1）：`/admin/users` 是
// 列表、`/admin/users/<id>` 是詳情頁。資料與列表狀態（搜尋、狀態篩選、頁碼）放在區段——兩者切換時
// 區段不卸載，回列表時狀態原樣，詳情頁的 ‹ n / m › 也依列表目前呈現的順序。列表頁標題區右側是搜尋、
// 狀態篩選與邀請；其下一張列表卡，卡頂「使用者｜邀請中」兩個分頁（分頁狀態在網址 ?tab=pending），
// 卡底共用換頁列（前端換頁）。使用者列整列可點進詳情頁、列內不含輸入控制項；邀請由頁面唯一 primary
// action 開啟。撤回邀請等破壞性動作維持確認框確認（確認鈕為紅系）。

type Confirm = { title: string; body: string; action: string; run: () => Promise<void> };

/** 邀請對話框的專案挑選器固定停在這個值；它對不到任何選項，所以 trigger 顯示 placeholder。 */
const PICKER = "__picker__";

export function UsersSection() {
  const { t } = useI18n();
  const client = useClient();
  const navigate = useNavigate();
  const narrow = useNarrow();
  const { data: fresh, error, reload } = useAsync(() => client.getAdminUsers(), []);
  // useAsync 重抓時會先清空 data：留著上一份成功回應繼續渲染，詳情頁的變更（改角色、撤銷）
  // 重載時詳情頁才不會卸載重掛、跳回第一個分頁（design D1）。只有第一次載入顯示載入中。
  const data = useLastData(fresh, error);
  // 成員資格的專案選項來自 registry（專案代號為封閉集合，不開放自由輸入）；還沒載入時為 null。
  const registry = useAsync(() => client.getAdminRegistry(), []);
  const projects = registry.data?.projects ?? null;
  const [searchParams, setSearchParams] = useSearchParams();
  // 其他值或缺席都是使用者分頁。
  const tab = searchParams.get("tab") === "pending" ? "pending" : "users";
  const [inviteOpen, setInviteOpen] = useState(false);
  const [confirm, setConfirm] = useState<Confirm | null>(null);
  const [busy, setBusy] = useState(false);
  const [q, setQ] = useState("");
  const [status, setStatus] = useState("");
  const { scrollRef, scrollToTop } = useListScroll();
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

  const list = (
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

      {!data && error == null && <AdminLoading />}
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
                  <UserRow
                    key={u.id}
                    user={u}
                    narrow={narrow}
                    onOpen={() => navigate(`/admin/users/${u.id}`)}
                  />
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

      {/* 只在開著時掛載：每次開啟都是空白表單。 */}
      {inviteOpen && <InviteDialog projects={projects} onClose={() => setInviteOpen(false)} onCreated={reload} />}

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

  return (
    <SectionRoutes
      base="/admin/users"
      param="id"
      list={list}
      detail={data && <UserDetailPage users={data.users} sequence={visible} projects={projects} reload={reload} />}
      error={error}
      onRetry={reload}
    />
  );
}

/** 受邀者要開的連結；token 只在建立當下拿得到，網址在前端拼。 */
function inviteUrl(token: string): string {
  return `${window.location.origin}/invite/${token}`;
}

/** 使用者列（design D4）：頭像字、名稱與 email、管理員籤、狀態籤、所屬專案數；窄螢幕把管理員
 *  併入 email 那一行、meta 只留狀態籤。整列可點進詳情頁。 */
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
      meta={
        <span className="tabular-nums">
          {t("users.pendingExpires").replace("{date}", fmtDate(i.expiresAt, t("common.dash")))}
        </span>
      }
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

// 邀請對話框（design D7）：送出成功後同一個對話框換成已建立狀態，一次性的邀請連結（token 進入 URL
// 成為連結）只在這裡出現、關掉就看不到；成功當下即重載，邀請中分頁的計數隨之加一。提交失敗保持開啟
// 並保留非祕密輸入，錯誤置於對應欄位附近。
function InviteDialog({
  projects,
  onClose,
  onCreated,
}: {
  /** registry 的專案；還沒載入（或載入失敗）時為 null。 */
  projects: AdminProject[] | null;
  onClose: () => void;
  onCreated: () => void;
}) {
  const { t } = useI18n();
  const client = useClient();
  const [email, setEmail] = useState("");
  const [display, setDisplay] = useState("");
  const [memberships, setMemberships] = useState<string[]>([]);
  const [admin, setAdmin] = useState(false);
  const [token, setToken] = useState<string | null>(null);
  const { pending, message, fieldErrors, submit } = useDialogSubmit(t("users.inviteError"));

  function onSubmit(event: FormEvent) {
    event.preventDefault();
    void submit(async () => {
      const created = await client.adminInvite({ email, display, memberships, admin });
      setToken(created.token);
      onCreated();
    });
  }

  if (token) {
    return (
      <FormDialog
        open
        onOpenChange={(open) => !open && onClose()}
        title={t("users.inviteDoneTitle")}
        description={t("users.inviteDoneBody").replace("{email}", () => email)}
        done
      >
        {/* 給受邀者的是可直接開啟的連結，不是要他自己拼網址的 token。 */}
        <SecretDone
          value={inviteUrl(token)}
          copyLabel={t("users.copyLink")}
          note={t("users.inviteDoneNote")}
          onDone={onClose}
        />
      </FormDialog>
    );
  }

  return (
    <FormDialog
      open
      onOpenChange={(open) => !open && onClose()}
      title={t("users.invite")}
      description={t("users.inviteHint")}
    >
      <form onSubmit={onSubmit} className="flex flex-col gap-4" noValidate>
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
          {projects === null || projects.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              {t(projects === null ? "users.projectsNotLoaded" : "users.inviteNoProjects")}
            </p>
          ) : (
            <>
              {/* 純挑選器，不持有選中值：value 固定綁在一個對不到任何選項的哨符上，
                  所以 trigger 永遠顯示 placeholder，而每次挑選都算一次值變動（同一個
                  專案移除後可以再加回來）。 */}
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
                <ul className="divide-y divide-border rounded-xl border border-border">
                  {memberships.map((key) => (
                    <li key={key} className="flex items-center gap-2 py-1.5 pl-3 pr-1.5">
                      <span className="min-w-0 flex-1 truncate text-sm">
                        {projects.find((p) => p.key === key)?.name ?? key}
                      </span>
                      <span className="font-mono text-xs text-muted-foreground">{key}</span>
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon-sm"
                        aria-label={t("users.removeFromInvite").replace("{project}", key)}
                        onClick={() => setMemberships((prev) => prev.filter((k) => k !== key))}
                      >
                        <X aria-hidden="true" className="h-4 w-4" />
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
        {/* 管理員開關與使用者詳情頁同一個設定型列，框起來與上方的已選專案清單同一種外框。 */}
        <div className="rounded-xl border border-border">
          <SettingsRow
            label={t("users.roleAdmin")}
            htmlFor="invite-admin"
            description={t("users.adminHint")}
            control={
              // 按鈕的名稱 axe 只認 aria-*，不認 <label for>。
              <Switch id="invite-admin" aria-label={t("users.roleAdmin")} checked={admin} onCheckedChange={setAdmin} />
            }
          />
        </div>
        <FormMessage message={message} fieldErrors={fieldErrors} />
        <FormDialogActions
          submitLabel={pending ? t("common.submitting") : t("users.inviteSubmit")}
          pending={pending}
          onCancel={onClose}
        />
      </form>
    </FormDialog>
  );
}

import { useState, type FormEvent } from "react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Badge,
  Button,
  Card,
  Checkbox,
  Label,
  NativeSelect,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@speclink/ui";
import { useClient } from "../../app/context";
import { useAsync } from "../../lib/useAsync";
import { readFormError } from "../../lib/formError";
import { Field } from "../../components/Field";
import { AdminError, AdminLoading } from "./states";
import type { AdminProject, AdminUser } from "../../api/client";

// 管理使用者頁（server-admin, D4／D6）：列出使用者與 membership、邀請、停權／復權、
// admin 旗標與 membership 調整。停權為破壞性操作，先以 AlertDialog 確認；最後一位
// active admin 受保護——canSuspend／canRemoveAdmin 為 false 時停用對應控制。

type Confirm = { title: string; run: () => Promise<void> };

export function UsersPage() {
  const client = useClient();
  const { loading, data, error, reload } = useAsync(() => client.getAdminUsers(), []);
  // 成員資格的專案選項來自 registry（project key 為封閉集合，不開放自由輸入）。
  const registry = useAsync(() => client.getAdminRegistry(), []);
  const projects = registry.data?.projects ?? [];
  const [confirm, setConfirm] = useState<Confirm | null>(null);
  const [busy, setBusy] = useState(false);

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
    <div className="space-y-8">
      <h1 className="text-2xl font-semibold">使用者</h1>
      {loading && <AdminLoading />}
      {error != null && <AdminError onRetry={reload} />}
      {data && (
        <>
          {/* Data-Dense Dashboard：資料表置於卡片容器內（與 Desktop 卡片語彙一致）。 */}
          <Card className="overflow-hidden">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>電子郵件</TableHead>
                  <TableHead>顯示名稱</TableHead>
                  <TableHead>狀態</TableHead>
                  <TableHead>管理員</TableHead>
                  <TableHead>成員資格</TableHead>
                  <TableHead>操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {data.users.map((u) => (
                  <TableRow key={u.id} className="align-top">
                    <TableCell>{u.email}</TableCell>
                    <TableCell>{u.display}</TableCell>
                    <TableCell>
                      <Badge variant={u.active ? "secondary" : "outline"}>
                        {u.active ? "有效" : "已停權"}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <div className="flex items-center gap-2">
                        <Checkbox
                          id={`admin-flag-${u.id}`}
                          checked={u.admin}
                          disabled={busy || (u.admin && !u.canRemoveAdmin)}
                          onCheckedChange={(v) =>
                            runNow(() => client.adminSetAdminFlag(u.id, v === true))
                          }
                        />
                        <Label
                          htmlFor={`admin-flag-${u.id}`}
                          className="text-xs text-muted-foreground"
                        >
                          管理員
                        </Label>
                      </div>
                    </TableCell>
                    <TableCell>
                      <MembershipEditor
                        user={u}
                        projects={projects}
                        busy={busy}
                        onSet={(body) => runNow(() => client.adminSetMembership(u.id, body))}
                      />
                    </TableCell>
                    <TableCell>
                      {u.active ? (
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          aria-label={`停權 ${u.display}`}
                          disabled={!u.canSuspend || busy}
                          onClick={() =>
                            setConfirm({
                              title: `停權 ${u.display}`,
                              run: () => client.adminSuspend(u.id),
                            })
                          }
                        >
                          停權
                        </Button>
                      ) : (
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          aria-label={`復權 ${u.display}`}
                          disabled={busy}
                          onClick={() => runNow(() => client.adminReactivate(u.id))}
                        >
                          復權
                        </Button>
                      )}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </Card>

          <InviteForm projects={projects} />
        </>
      )}

      <AlertDialog open={confirm !== null} onOpenChange={(open) => !open && setConfirm(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{confirm?.title}？</AlertDialogTitle>
            <AlertDialogDescription>停權後該使用者無法登入，之後可再復權。</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction onClick={runConfirmed} disabled={busy}>
              停權
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

// 每位使用者的 membership：列出並可逐一移除；新增以下拉選單——專案來自 registry、
// role 為 server 端固定集合（editor／reader），不開放自由輸入。
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
  const [projectKey, setProjectKey] = useState("");
  const [role, setRole] = useState("editor");

  function add(event: FormEvent) {
    event.preventDefault();
    if (busy || !projectKey) return;
    onSet({ projectKey, role, member: true });
    setProjectKey("");
    setRole("editor");
  }

  return (
    <div className="space-y-2">
      {user.memberships.length === 0 ? (
        <span className="text-muted-foreground">無</span>
      ) : (
        <ul className="space-y-1">
          {user.memberships.map((m) => (
            <li key={`${m.projectKey}:${m.role}`} className="flex items-center gap-2">
              <span className="font-mono">{m.projectKey}</span>
              <Badge variant="outline">{m.role}</Badge>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                aria-label={`移除 ${user.display} 在 ${m.projectKey} 的成員資格`}
                disabled={busy}
                onClick={() => onSet({ projectKey: m.projectKey, role: m.role, member: false })}
              >
                移除
              </Button>
            </li>
          ))}
        </ul>
      )}
      <form onSubmit={add} className="flex flex-wrap items-center gap-2">
        <NativeSelect
          aria-label={`${user.display} 專案代碼`}
          value={projectKey}
          onChange={(e) => setProjectKey(e.target.value)}
          className="w-36"
        >
          <option value="">選擇專案…</option>
          {projects.map((p) => (
            <option key={p.key} value={p.key}>
              {p.key}
            </option>
          ))}
        </NativeSelect>
        <NativeSelect
          aria-label={`${user.display} 角色`}
          value={role}
          onChange={(e) => setRole(e.target.value)}
          className="w-28"
        >
          <option value="editor">editor</option>
          <option value="reader">reader</option>
        </NativeSelect>
        <Button
          type="submit"
          variant="outline"
          size="sm"
          aria-label={`為 ${user.display} 新增成員資格`}
          disabled={busy || !projectKey}
        >
          新增
        </Button>
      </form>
    </div>
  );
}

// 邀請表單：成功後把一次性 token 顯示一次（token 進入 URL 成為邀請連結）。
// 成員資格為既有專案的勾選清單——server 端只收 project key（接受邀請時 role 固定
// editor），不開放自由文字輸入。
function InviteForm({ projects }: { projects: AdminProject[] }) {
  const client = useClient();
  const [email, setEmail] = useState("");
  const [display, setDisplay] = useState("");
  const [memberships, setMemberships] = useState<string[]>([]);
  const [admin, setAdmin] = useState(false);
  const [pending, setPending] = useState(false);
  const [token, setToken] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    if (pending) return;
    setPending(true);
    setMessage(null);
    setFieldErrors({});
    try {
      const { token: created } = await client.adminInvite({
        email,
        display,
        memberships,
        admin,
      });
      setToken(created);
      setEmail("");
      setDisplay("");
      setMemberships([]);
      setAdmin(false);
    } catch (error) {
      const read = readFormError(error, "建立邀請時發生錯誤");
      setMessage(read.message);
      setFieldErrors(read.fieldErrors);
    } finally {
      setPending(false);
    }
  }

  return (
    <section className="space-y-4">
      <h2 className="text-lg font-medium">邀請使用者</h2>
      {token && (
        <div
          role="status"
          aria-live="polite"
          className="rounded-md border border-primary bg-primary/5 p-4"
        >
          <p className="text-sm font-medium">邀請已建立，這組一次性邀請 token 只顯示這一次：</p>
          <code className="mt-1 block break-all font-mono text-sm">{token}</code>
        </div>
      )}
      <Card className="max-w-md p-4">
        <form onSubmit={onSubmit} className="space-y-4" noValidate>
          <Field
            id="invite-email"
            label="電子郵件"
            value={email}
            onChange={setEmail}
            error={fieldErrors.email}
          />
          <Field
            id="invite-display"
            label="顯示名稱"
            value={display}
            onChange={setDisplay}
            error={fieldErrors.display}
          />
          <fieldset className="space-y-2">
            <legend className="text-sm font-medium">成員資格</legend>
            {projects.length === 0 ? (
              <p className="text-sm text-muted-foreground">尚無專案。</p>
            ) : (
              projects.map((p) => (
                <div key={p.key} className="flex items-center gap-2">
                  <Checkbox
                    id={`invite-membership-${p.key}`}
                    checked={memberships.includes(p.key)}
                    onCheckedChange={(v) =>
                      setMemberships((prev) =>
                        v === true ? [...prev, p.key] : prev.filter((k) => k !== p.key),
                      )
                    }
                  />
                  <Label
                    htmlFor={`invite-membership-${p.key}`}
                    className="flex items-center gap-2 font-normal"
                  >
                    <code className="font-mono">{p.key}</code>
                    <span className="text-muted-foreground">{p.name}</span>
                  </Label>
                </div>
              ))
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
            <Label htmlFor="invite-admin">設為管理員</Label>
          </div>
          {message && Object.keys(fieldErrors).length === 0 && (
            <p role="alert" className="text-sm text-destructive">
              {message}
            </p>
          )}
          <Button type="submit" disabled={pending}>
            {pending ? "送出中…" : "送出邀請"}
          </Button>
        </form>
      </Card>
    </section>
  );
}

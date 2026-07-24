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
  Button,
  Input,
  Label,
} from "@speclink/ui";
import { useClient } from "../../app/context";
import { useAsync } from "../../lib/useAsync";
import { readFormError } from "../../lib/formError";
import { Field } from "../../components/Field";
import { AdminError, AdminLoading } from "./states";
import type { AdminUser } from "../../api/client";

// 管理使用者頁（server-admin, D4／D6）：列出使用者與 membership、邀請、停權／復權、
// admin 旗標與 membership 調整。停權為破壞性操作，先以 AlertDialog 確認；最後一位
// active admin 受保護——canSuspend／canRemoveAdmin 為 false 時停用對應控制。

type Confirm = { title: string; run: () => Promise<void> };

export function UsersPage() {
  const client = useClient();
  const { loading, data, error, reload } = useAsync(() => client.getAdminUsers(), []);
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
          <div className="overflow-x-auto">
            <table className="w-full text-left text-sm">
              <thead className="text-muted-foreground">
                <tr>
                  <th className="py-1 pr-4 font-medium">電子郵件</th>
                  <th className="py-1 pr-4 font-medium">顯示名稱</th>
                  <th className="py-1 pr-4 font-medium">狀態</th>
                  <th className="py-1 pr-4 font-medium">管理員</th>
                  <th className="py-1 pr-4 font-medium">成員資格</th>
                  <th className="py-1 font-medium">操作</th>
                </tr>
              </thead>
              <tbody>
                {data.users.map((u) => (
                  <tr key={u.id} className="border-t align-top">
                    <td className="py-1.5 pr-4">{u.email}</td>
                    <td className="py-1.5 pr-4">{u.display}</td>
                    <td className="py-1.5 pr-4">{u.active ? "有效" : "已停權"}</td>
                    <td className="py-1.5 pr-4">
                      <div className="flex items-center gap-2">
                        <input
                          type="checkbox"
                          id={`admin-flag-${u.id}`}
                          className="h-4 w-4"
                          checked={u.admin}
                          disabled={busy || (u.admin && !u.canRemoveAdmin)}
                          onChange={(e) =>
                            runNow(() => client.adminSetAdminFlag(u.id, e.target.checked))
                          }
                        />
                        <Label
                          htmlFor={`admin-flag-${u.id}`}
                          className="text-xs text-muted-foreground"
                        >
                          管理員
                        </Label>
                      </div>
                    </td>
                    <td className="py-1.5 pr-4">
                      <MembershipEditor
                        user={u}
                        busy={busy}
                        onSet={(body) => runNow(() => client.adminSetMembership(u.id, body))}
                      />
                    </td>
                    <td className="py-1.5">
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
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <InviteForm />
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

// 每位使用者的 membership：列出並可逐一移除，另有精簡的新增表單。
function MembershipEditor({
  user,
  busy,
  onSet,
}: {
  user: AdminUser;
  busy: boolean;
  onSet: (body: { projectKey: string; role: string; member: boolean }) => void;
}) {
  const [projectKey, setProjectKey] = useState("");
  const [role, setRole] = useState("");

  function add(event: FormEvent) {
    event.preventDefault();
    if (busy || !projectKey.trim() || !role.trim()) return;
    onSet({ projectKey: projectKey.trim(), role: role.trim(), member: true });
    setProjectKey("");
    setRole("");
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
              <span className="text-muted-foreground">{m.role}</span>
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
        <Input
          aria-label={`${user.display} 專案代碼`}
          value={projectKey}
          onChange={(e) => setProjectKey(e.target.value)}
          placeholder="project"
          className="h-8 w-28"
        />
        <Input
          aria-label={`${user.display} 角色`}
          value={role}
          onChange={(e) => setRole(e.target.value)}
          placeholder="role"
          className="h-8 w-24"
        />
        <Button type="submit" variant="outline" size="sm" disabled={busy}>
          新增
        </Button>
      </form>
    </div>
  );
}

// 邀請表單：成功後把一次性 token 顯示一次（token 進入 URL 成為邀請連結）。
function InviteForm() {
  const client = useClient();
  const [email, setEmail] = useState("");
  const [display, setDisplay] = useState("");
  const [memberships, setMemberships] = useState("");
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
      const list = memberships
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
      const { token: created } = await client.adminInvite({
        email,
        display,
        memberships: list,
        admin,
      });
      setToken(created);
      setEmail("");
      setDisplay("");
      setMemberships("");
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
      <form onSubmit={onSubmit} className="max-w-md space-y-4" noValidate>
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
        <Field
          id="invite-memberships"
          label="成員資格（projectKey:role，以逗號分隔）"
          value={memberships}
          onChange={setMemberships}
          error={fieldErrors.memberships}
        />
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="invite-admin"
            className="h-4 w-4"
            checked={admin}
            onChange={(e) => setAdmin(e.target.checked)}
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
    </section>
  );
}

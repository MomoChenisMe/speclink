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
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@speclink/ui";
import { useClient } from "../app/context";
import { useAsync } from "../lib/useAsync";
import { readFormError } from "../lib/formError";
import { Field } from "../components/Field";
import type { DeviceFamilyMeta, PatMeta, SessionMeta } from "../api/client";

// 帳號自助頁（server-identity「帳號 browser API 保持憑證祕密邊界」, D4／D6）：使用者、
// PAT、Web session 與裝置。PAT 明文只在建立時顯示一次；撤銷等破壞性操作先以 AlertDialog
// 確認且立即生效。所有 server 資料留在本 route 的 component state（D1）。

type Confirm = { title: string; run: () => Promise<void> };

/** The date portion of an ISO timestamp, or a fallback when absent. */
function fmtDate(iso: string | null, fallback: string): string {
  return iso ? iso.slice(0, 10) : fallback;
}

export function AccountPage() {
  const client = useClient();
  const { loading, data, reload } = useAsync(() => client.getAccount(), []);
  const [plaintext, setPlaintext] = useState<string | null>(null);
  const [confirm, setConfirm] = useState<Confirm | null>(null);
  const [busy, setBusy] = useState(false);

  if (!data) {
    if (loading) {
      return (
        <p role="status" aria-live="polite" className="text-muted-foreground">
          載入中…
        </p>
      );
    }
    return (
      <div role="alert">
        <p className="text-destructive">無法載入帳號資料。</p>
        <Button type="button" variant="outline" className="mt-3" onClick={reload}>
          重試
        </Button>
      </div>
    );
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

  return (
    <div className="space-y-8">
      <header>
        <h1 className="text-2xl font-semibold">帳號</h1>
        <p className="mt-1 text-muted-foreground">
          {data.user.display}（{data.user.email}）
        </p>
      </header>

      {plaintext && (
        <div
          role="status"
          aria-live="polite"
          className="rounded-md border border-primary bg-primary/5 p-4"
        >
          <p className="text-sm font-medium">新的 PAT，只會顯示這一次，請立即複製保存：</p>
          <code className="mt-1 block break-all font-mono text-sm">{plaintext}</code>
        </div>
      )}

      <PatSection
        pats={data.pats}
        onCreated={setPlaintext}
        afterMutate={reload}
        onRevoke={(pat) =>
          setConfirm({
            title: `撤銷 PAT「${pat.name}」？`,
            run: () => client.revokePat(pat.id),
          })
        }
      />

      <SessionSection sessions={data.sessions} />

      <DeviceSection
        families={data.deviceFamilies}
        onRevoke={(family) =>
          setConfirm({
            title: `撤銷裝置登入「${family.source}」？`,
            run: () => client.revokeDevice(family.id),
          })
        }
      />

      <AlertDialog open={confirm !== null} onOpenChange={(open) => !open && setConfirm(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{confirm?.title}</AlertDialogTitle>
            <AlertDialogDescription>撤銷後無法復原，且立即生效。</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction onClick={runConfirmed} disabled={busy}>
              撤銷
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

// PAT 區塊：建立表單（明文只出現一次）＋列表與逐一撤銷。
function PatSection({
  pats,
  onCreated,
  afterMutate,
  onRevoke,
}: {
  pats: PatMeta[];
  onCreated: (plaintext: string) => void;
  afterMutate: () => void;
  onRevoke: (pat: PatMeta) => void;
}) {
  const client = useClient();
  const [name, setName] = useState("");
  const [expires, setExpires] = useState("");
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
      const { plaintext } = await client.createPat({ name, expires: expires || undefined });
      onCreated(plaintext);
      setName("");
      setExpires("");
      afterMutate();
    } catch (error) {
      const read = readFormError(error, "建立 PAT 時發生錯誤");
      setMessage(read.message);
      setFieldErrors(read.fieldErrors);
    } finally {
      setPending(false);
    }
  }

  return (
    <section className="space-y-4">
      <h2 className="text-lg font-medium">Personal Access Tokens</h2>
      {pats.length === 0 ? (
        <p className="text-sm text-muted-foreground">尚無 PAT。</p>
      ) : (
        <Card className="overflow-hidden">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>前綴</TableHead>
                <TableHead>名稱</TableHead>
                <TableHead>到期</TableHead>
                <TableHead>最近使用</TableHead>
                <TableHead>狀態</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {pats.map((pat) => (
                <TableRow key={pat.id}>
                  <TableCell className="font-mono">{pat.prefix}</TableCell>
                  <TableCell>{pat.name}</TableCell>
                  <TableCell>{fmtDate(pat.expiresAt, "永久")}</TableCell>
                  <TableCell>{fmtDate(pat.lastUsedAt, "從未")}</TableCell>
                  <TableCell>
                    {pat.revokedAt ? (
                      <Badge variant="outline">已撤銷</Badge>
                    ) : (
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        aria-label={`撤銷 PAT ${pat.name}`}
                        onClick={() => onRevoke(pat)}
                      >
                        撤銷
                      </Button>
                    )}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </Card>
      )}

      <Card className="max-w-md p-4">
        <form onSubmit={onSubmit} className="space-y-4" noValidate>
          <Field id="pat-name" label="名稱" value={name} onChange={setName} error={fieldErrors.name} />
          <Field
            id="pat-expires"
            label="到期日（YYYY-MM-DD，留空為永久）"
            value={expires}
            onChange={setExpires}
            error={fieldErrors.expires}
          />
          {message && Object.keys(fieldErrors).length === 0 && (
            <p role="alert" className="text-sm text-destructive">
              {message}
            </p>
          )}
          <Button type="submit" disabled={pending}>
            {pending ? "建立中…" : "建立 PAT"}
          </Button>
        </form>
      </Card>
    </section>
  );
}

// Web session 區塊：唯讀清單（登出由帳號殼的登出動作處理）。
function SessionSection({ sessions }: { sessions: SessionMeta[] }) {
  return (
    <section className="space-y-4">
      <h2 className="text-lg font-medium">Web Sessions</h2>
      {sessions.length === 0 ? (
        <p className="text-sm text-muted-foreground">尚無 session。</p>
      ) : (
        <Card className="overflow-hidden">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>建立</TableHead>
                <TableHead>到期</TableHead>
                <TableHead>狀態</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {sessions.map((session) => (
                <TableRow key={session.id}>
                  <TableCell>{fmtDate(session.createdAt, "—")}</TableCell>
                  <TableCell>{fmtDate(session.expiresAt, "—")}</TableCell>
                  <TableCell>
                    <Badge variant={session.revokedAt ? "outline" : "secondary"}>
                      {session.revokedAt ? "已撤銷" : "有效"}
                    </Badge>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </Card>
      )}
    </section>
  );
}

// 裝置登入區塊：清單與逐一撤銷（撤銷立即使 access token 與 refresh credential 失效）。
function DeviceSection({
  families,
  onRevoke,
}: {
  families: DeviceFamilyMeta[];
  onRevoke: (family: DeviceFamilyMeta) => void;
}) {
  return (
    <section className="space-y-4">
      <h2 className="text-lg font-medium">裝置登入</h2>
      {families.length === 0 ? (
        <p className="text-sm text-muted-foreground">尚無裝置登入。</p>
      ) : (
        <Card className="overflow-hidden">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>來源</TableHead>
                <TableHead>建立</TableHead>
                <TableHead>最近 refresh</TableHead>
                <TableHead>狀態</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {families.map((family) => (
                <TableRow key={family.id}>
                  <TableCell>
                    <Badge variant="secondary">{family.source}</Badge>
                  </TableCell>
                  <TableCell>{fmtDate(family.createdAt, "—")}</TableCell>
                  <TableCell>{fmtDate(family.lastRefreshAt, "—")}</TableCell>
                  <TableCell>
                    {family.revokedAt ? (
                      <Badge variant="outline">已撤銷</Badge>
                    ) : (
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        aria-label={`撤銷裝置登入 ${family.source}`}
                        onClick={() => onRevoke(family)}
                      >
                        撤銷
                      </Button>
                    )}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </Card>
      )}
    </section>
  );
}

import { useState, type FormEvent } from "react";
import { KeyRound } from "lucide-react";
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
  SEMANTIC_SURFACE,
  SEMANTIC_TONE,
  useI18n,
} from "@speclink/ui";
import { useClient } from "../app/context";
import { useAsync } from "../lib/useAsync";
import { readFormError } from "../lib/formError";
import { Field } from "../components/Field";
import { DetailSheet } from "../components/DetailSheet";
import { CopyButton } from "../components/CopyButton";
import type { DeviceFamilyMeta, MembershipMeta, PatMeta, SessionMeta } from "../api/client";

// 帳號自助頁（server-identity「帳號 browser API 保持憑證祕密邊界」, D4／D6）：使用者、
// 存取金鑰、登入工作階段與裝置。金鑰明文只在建立時顯示一次；撤銷等破壞性操作先以
// AlertDialog 確認且立即生效。所有 server 資料留在本 route 的 component state（D1）。

type Confirm = { title: string; run: () => Promise<void> };

/** The date portion of an ISO timestamp, or a fallback when absent. */
function fmtDate(iso: string | null, fallback: string): string {
  return iso ? iso.slice(0, 10) : fallback;
}

export function AccountPage() {
  const { t } = useI18n();
  const client = useClient();
  const { loading, data, reload } = useAsync(() => client.getAccount(), []);
  const [plaintext, setPlaintext] = useState<string | null>(null);
  const [confirm, setConfirm] = useState<Confirm | null>(null);
  const [busy, setBusy] = useState(false);
  const [createOpen, setCreateOpen] = useState(false);

  if (!data) {
    if (loading) {
      return (
        <p role="status" aria-live="polite" className="text-muted-foreground">
          {t("common.loading")}
        </p>
      );
    }
    return (
      <div role="alert">
        <p className="text-destructive">{t("account.loadFailed")}</p>
        <Button type="button" variant="outline" className="mt-3" onClick={reload}>
          {t("common.retry")}
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
      {/* 頁面標題區塊用 div：<header> 會被讀成第二個 banner，與殼層 header 相衝。 */}
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold">{t("account.title")}</h1>
          <p className="mt-1 text-muted-foreground">
            {t("account.identity")
              .replace("{display}", data.user.display)
              .replace("{email}", data.user.email)}
          </p>
        </div>
        {/* 與管理列表頁一致：頁面唯一 primary action 開抽屜，建立欄位不常駐頁面。 */}
        <Button type="button" className="gap-1.5" onClick={() => setCreateOpen(true)}>
          <KeyRound aria-hidden="true" className="h-4 w-4" />
          {t("account.createKey")}
        </Button>
      </div>

      {plaintext && (
        <div
          role="status"
          aria-live="polite"
          className={`rounded-md border p-4 ${SEMANTIC_SURFACE.success}`}
        >
          <p className="text-sm font-medium">{t("account.newKeyNotice")}</p>
          <div className="mt-1 flex items-start gap-3">
            <code className="min-w-0 flex-1 break-all font-mono text-sm">{plaintext}</code>
            <CopyButton value={plaintext} />
          </div>
        </div>
      )}

      <ProjectSection memberships={data.memberships} />

      <PatSection
        pats={data.pats}
        onRevoke={(pat) =>
          setConfirm({
            title: t("account.revokeKeyTitle").replace("{name}", pat.name),
            run: () => client.revokePat(pat.id),
          })
        }
      />

      <SessionSection sessions={data.sessions} />

      <DeviceSection
        families={data.deviceFamilies}
        onRevoke={(family) =>
          setConfirm({
            title: t("account.revokeDeviceTitle").replace("{name}", family.source),
            run: () => client.revokeDevice(family.id),
          })
        }
      />

      <CreateKeySheet
        open={createOpen}
        onOpenChange={setCreateOpen}
        onCreated={(secret) => {
          setPlaintext(secret);
          setCreateOpen(false);
          reload();
        }}
      />

      <AlertDialog open={confirm !== null} onOpenChange={(open) => !open && setConfirm(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{confirm?.title}</AlertDialogTitle>
            <AlertDialogDescription>{t("account.revokeBody")}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("common.cancel")}</AlertDialogCancel>
            <AlertDialogAction onClick={runConfirmed} disabled={busy}>
              {t("common.revoke")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

// 我的專案區塊（server-web-console「帳號頁呈現我的專案」）：唯讀清單，資料只來自
// account summary。這是個人視角——admin 與一般成員看到同一區塊、同一形狀，全部專案
// 的治理視角在 /admin/registry。顯示名缺席時退回專案代號。
function ProjectSection({ memberships }: { memberships: MembershipMeta[] }) {
  const { t } = useI18n();
  return (
    <section className="space-y-4" aria-labelledby="account-projects">
      <h2 id="account-projects" className="text-lg font-medium">
        {t("account.projects")}
      </h2>
      {memberships.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("account.noProjects")}</p>
      ) : (
        <Card className="overflow-hidden">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t("field.project")}</TableHead>
                <TableHead>{t("field.role")}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {memberships.map((membership) => (
                <TableRow key={membership.projectKey}>
                  <TableCell>{membership.projectName || membership.projectKey}</TableCell>
                  <TableCell>{membership.role}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </Card>
      )}
    </section>
  );
}

// 存取金鑰區塊：唯讀列表與逐一撤銷。建立走頁面的 primary action 開抽屜。
function PatSection({
  pats,
  onRevoke,
}: {
  pats: PatMeta[];
  onRevoke: (pat: PatMeta) => void;
}) {
  const { t } = useI18n();
  return (
    <section className="space-y-4">
      <h2 className="text-lg font-medium">{t("account.keys")}</h2>
      {pats.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("account.noKeys")}</p>
      ) : (
        <Card className="overflow-hidden">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t("field.prefix")}</TableHead>
                <TableHead>{t("field.name")}</TableHead>
                <TableHead>{t("field.expires")}</TableHead>
                <TableHead>{t("field.lastUsed")}</TableHead>
                <TableHead>{t("field.status")}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {pats.map((pat) => (
                <TableRow key={pat.id}>
                  <TableCell className="font-mono">{pat.prefix}</TableCell>
                  <TableCell>{pat.name}</TableCell>
                  <TableCell>{fmtDate(pat.expiresAt, t("common.forever"))}</TableCell>
                  <TableCell>{fmtDate(pat.lastUsedAt, t("common.never"))}</TableCell>
                  <TableCell>
                    {pat.revokedAt ? (
                      <Badge variant="outline" className="text-muted-foreground">{t("common.revoked")}</Badge>
                    ) : (
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        aria-label={t("account.revokeKey").replace("{name}", pat.name)}
                        onClick={() => onRevoke(pat)}
                      >
                        {t("common.revoke")}
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

// 建立存取金鑰的抽屜。成功後由頁面呈現一次性明文——明文屬於整頁的一次性回饋，
// 不該跟著抽屜一起關掉。
function CreateKeySheet({
  open,
  onOpenChange,
  onCreated,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated: (plaintext: string) => void;
}) {
  const { t } = useI18n();
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
      setName("");
      setExpires("");
      onCreated(plaintext);
    } catch (error) {
      const read = readFormError(error, t("account.createKeyError"));
      setMessage(read.message);
      setFieldErrors(read.fieldErrors);
    } finally {
      setPending(false);
    }
  }

  if (!open) return null;

  return (
    <DetailSheet open onOpenChange={onOpenChange} title={t("account.createKey")}>
      <form onSubmit={onSubmit} className="space-y-4" noValidate>
        <Field
          id="pat-name"
          label={t("account.keyNameLabel")}
          value={name}
          onChange={setName}
          error={fieldErrors.name}
        />
        <Field
          id="pat-expires"
          label={t("account.keyExpiresLabel")}
          value={expires}
          onChange={setExpires}
          error={fieldErrors.expires}
        />
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
            {pending ? t("common.creating") : t("common.create")}
          </Button>
        </div>
      </form>
    </DetailSheet>
  );
}

// 登入工作階段區塊：唯讀清單（登出由殼層的登出動作處理）。
function SessionSection({ sessions }: { sessions: SessionMeta[] }) {
  const { t } = useI18n();
  return (
    <section className="space-y-4">
      <h2 className="text-lg font-medium">{t("account.sessions")}</h2>
      {sessions.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("account.noSessions")}</p>
      ) : (
        <Card className="overflow-hidden">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t("field.created")}</TableHead>
                <TableHead>{t("field.expires")}</TableHead>
                <TableHead>{t("field.status")}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {sessions.map((session) => (
                <TableRow key={session.id}>
                  <TableCell>{fmtDate(session.createdAt, t("common.dash"))}</TableCell>
                  <TableCell>{fmtDate(session.expiresAt, t("common.dash"))}</TableCell>
                  <TableCell>
                    <Badge
                      variant="outline"
                      className={
                        session.revokedAt
                          ? "text-muted-foreground"
                          : `${SEMANTIC_SURFACE.success} ${SEMANTIC_TONE.success}`
                      }
                    >
                      {session.revokedAt ? t("common.revoked") : t("common.active")}
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
  const { t } = useI18n();
  return (
    <section className="space-y-4">
      <h2 className="text-lg font-medium">{t("account.devices")}</h2>
      {families.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("account.noDevices")}</p>
      ) : (
        <Card className="overflow-hidden">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t("field.source")}</TableHead>
                <TableHead>{t("field.created")}</TableHead>
                <TableHead>{t("field.lastRefresh")}</TableHead>
                <TableHead>{t("field.status")}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {families.map((family) => (
                <TableRow key={family.id}>
                  <TableCell>
                    <Badge variant="secondary">{family.source}</Badge>
                  </TableCell>
                  <TableCell>{fmtDate(family.createdAt, t("common.dash"))}</TableCell>
                  <TableCell>{fmtDate(family.lastRefreshAt, t("common.dash"))}</TableCell>
                  <TableCell>
                    {family.revokedAt ? (
                      <Badge variant="outline" className="text-muted-foreground">{t("common.revoked")}</Badge>
                    ) : (
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        aria-label={t("account.revokeDevice").replace("{name}", family.source)}
                        onClick={() => onRevoke(family)}
                      >
                        {t("common.revoke")}
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

import { useState, type FormEvent } from "react";
import { KeyRound } from "lucide-react";
import {
  Button,
  ConfirmDialog,
  SettingsCard,
  SettingsRow,
  StatusPill,
  useI18n,
  PageHeader,
} from "@speclink/ui";
import { useClient } from "../app/context";
import { fillUser } from "../lib/userLabel";
import { useAsync } from "../lib/useAsync";
import { readFormError } from "../lib/formError";
import { Field } from "../components/Field";
import { FormDialog, FormDialogFooter, SecretDone } from "../components/FormDialog";
import { CardNote } from "../components/CardNote";
import { RevocationPill } from "../components/RevocationPill";
import { RevokeButton } from "../components/RevokeButton";
import type { AccountSummary, DeviceFamilyMeta, PatMeta } from "../api/client";

// 帳號自助頁（server-identity「帳號 browser API 保持憑證祕密邊界」, D4／D6；server-web-list-pages
// design D8、D10）：768px 置中欄，由上而下為我的專案、存取金鑰、裝置登入、登入工作階段四張設定型卡。
// 金鑰明文只在建立對話框內顯示一次（server-web-detail-pages design D7）；存取金鑰與裝置登入的撤銷鈕
// 滑過才出現，撤銷等破壞性操作先以確認框確認且立即生效。所有 server 資料留在本 route 的 component
// state（D1）。

type Confirm = { title: string; run: () => Promise<void> };

/** The date portion of an ISO timestamp, or a fallback when absent. */
function fmtDate(iso: string | null, fallback: string): string {
  return iso ? iso.slice(0, 10) : fallback;
}

export function AccountPage() {
  const { t } = useI18n();
  const client = useClient();
  const { loading, data, reload } = useAsync(() => client.getAccount(), []);
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
    <div className="mx-auto flex w-full max-w-[768px] flex-col gap-6">
      {/* 與管理列表頁一致：頁面唯一 primary action 開對話框，建立欄位不常駐頁面。 */}
      <PageHeader
        level={1}
        title={t("account.title")}
        description={fillUser(t("account.description"), data.user)}
        actions={
          <Button type="button" className="gap-1.5" onClick={() => setCreateOpen(true)}>
            <KeyRound aria-hidden="true" className="h-4 w-4" />
            {t("account.createKey")}
          </Button>
        }
      />

      <AccountCards
        account={data}
        busy={busy}
        onRevokeKey={(pat) =>
          setConfirm({
            title: t("account.revokeKeyTitle").replace("{name}", pat.name),
            run: () => client.revokePat(pat.id),
          })
        }
        onRevokeDevice={(family) =>
          setConfirm({
            title: t("account.revokeDeviceTitle").replace("{name}", family.source),
            run: () => client.revokeDevice(family.id),
          })
        }
      />

      {/* 只在開著時掛載：每次開啟都是空白表單。 */}
      {createOpen && <CreateKeyDialog onClose={() => setCreateOpen(false)} onCreated={reload} />}

      <ConfirmDialog
        open={confirm !== null}
        onOpenChange={(open) => !open && setConfirm(null)}
        title={confirm?.title ?? ""}
        description={t("account.revokeBody")}
        confirmLabel={t("common.revoke")}
        destructive
        busy={busy}
        onConfirm={runConfirmed}
      />
    </div>
  );
}

// 四張卡（design D10）。我的專案（server-web-console「帳號頁呈現我的專案」）是唯讀的個人視角，
// 資料只來自 account summary——admin 與一般成員看到同一張卡、同一形狀，全部專案的治理視角在
// /admin/registry；顯示名缺席時以等寬代號為名、不再重複一次代號。存取金鑰的建立走頁面的 primary
// action 開對話框；登入工作階段唯讀（登出由殼層的登出動作處理）；裝置登入撤銷後立即使 access
// token 與 refresh credential 失效。
function AccountCards({
  account,
  busy,
  onRevokeKey,
  onRevokeDevice,
}: {
  account: AccountSummary;
  busy: boolean;
  onRevokeKey: (pat: PatMeta) => void;
  onRevokeDevice: (family: DeviceFamilyMeta) => void;
}) {
  const { t } = useI18n();
  return (
    <>
      <SettingsCard title={t("account.projects")} aside={t("account.projectsAside")}>
        {account.memberships.length === 0 ? (
          <CardNote>{t("account.noProjects")}</CardNote>
        ) : (
          account.memberships.map((m) => (
            <SettingsRow
              key={m.projectKey}
              label={m.projectName || <span className="font-mono">{m.projectKey}</span>}
              description={m.projectName ? <span className="font-mono">{m.projectKey}</span> : undefined}
              control={
                <StatusPill tone="neutral" className="font-mono">
                  {m.role}
                </StatusPill>
              }
            />
          ))
        )}
      </SettingsCard>

      <SettingsCard title={t("account.keys")} aside={t("account.keysAside")}>
        {account.pats.length === 0 ? (
          <CardNote>{t("account.noKeys")}</CardNote>
        ) : (
          account.pats.map((pat) => (
            <SettingsRow
              key={pat.id}
              label={pat.name}
              description={
                pat.revokedAt
                  ? t("account.keyRevokedMeta")
                      .replace("{prefix}", pat.prefix)
                      .replace("{date}", fmtDate(pat.revokedAt, t("common.dash")))
                  : t("account.keyMeta")
                      .replace("{prefix}", pat.prefix)
                      .replace("{expires}", fmtDate(pat.expiresAt, t("common.forever")))
                      .replace("{lastUsed}", fmtDate(pat.lastUsedAt, t("common.never")))
              }
              control={
                <>
                  {!pat.revokedAt && (
                    <RevokeButton
                      label={t("account.revokeKey").replace("{name}", pat.name)}
                      disabled={busy}
                      onClick={() => onRevokeKey(pat)}
                    />
                  )}
                  <RevocationPill revokedAt={pat.revokedAt} />
                </>
              }
            />
          ))
        )}
      </SettingsCard>

      <SettingsCard title={t("account.devices")} aside={t("account.devicesAside")}>
        {account.deviceFamilies.length === 0 ? (
          <CardNote>{t("account.noDevices")}</CardNote>
        ) : (
          account.deviceFamilies.map((family) => (
            <SettingsRow
              key={family.id}
              label={family.source}
              description={t("account.deviceMeta").replace(
                "{date}",
                fmtDate(family.lastRefreshAt, t("common.dash")),
              )}
              control={
                <>
                  {!family.revokedAt && (
                    <RevokeButton
                      label={t("account.revokeDevice").replace("{name}", family.source)}
                      disabled={busy}
                      onClick={() => onRevokeDevice(family)}
                    />
                  )}
                  <RevocationPill revokedAt={family.revokedAt} />
                </>
              }
            />
          ))
        )}
      </SettingsCard>

      <SettingsCard title={t("account.sessions")}>
        {account.sessions.length === 0 ? (
          <CardNote>{t("account.noSessions")}</CardNote>
        ) : (
          account.sessions.map((session) => (
            <SettingsRow
              key={session.id}
              label={t("account.sessionCreated").replace("{date}", fmtDate(session.createdAt, t("common.dash")))}
              description={t("account.sessionExpires").replace(
                "{date}",
                fmtDate(session.expiresAt, t("common.dash")),
              )}
              control={<RevocationPill revokedAt={session.revokedAt} />}
            />
          ))
        )}
      </SettingsCard>
    </>
  );
}

// 建立存取金鑰的對話框（design D7）：送出成功後同一個對話框換成已建立狀態，一次性明文只在這裡出現、
// 關掉就看不到；成功當下即重載，存取金鑰卡隨之多出一列。
function CreateKeyDialog({ onClose, onCreated }: { onClose: () => void; onCreated: () => void }) {
  const { t } = useI18n();
  const client = useClient();
  const [name, setName] = useState("");
  const [expires, setExpires] = useState("");
  const [pending, setPending] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});
  const [plaintext, setPlaintext] = useState<string | null>(null);

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    if (pending) return;
    setPending(true);
    setMessage(null);
    setFieldErrors({});
    try {
      const created = await client.createPat({ name, expires: expires || undefined });
      setPlaintext(created.plaintext);
      onCreated();
    } catch (error) {
      const read = readFormError(error, t("account.createKeyError"));
      setMessage(read.message);
      setFieldErrors(read.fieldErrors);
    } finally {
      setPending(false);
    }
  }

  if (plaintext) {
    return (
      <FormDialog
        open
        onOpenChange={(open) => !open && onClose()}
        title={t("account.keyDoneTitle")}
        description={t("account.keyDoneBody")}
        done
      >
        <SecretDone value={plaintext} copyLabel={t("common.copy")} onDone={onClose} />
      </FormDialog>
    );
  }

  return (
    <FormDialog
      open
      onOpenChange={(open) => !open && onClose()}
      title={t("account.createKey")}
      description={t("account.createKeyHint")}
    >
      <form onSubmit={onSubmit} className="flex flex-col gap-4" noValidate>
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
        <FormDialogFooter>
          <Button type="button" variant="ghost" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button type="submit" disabled={pending}>
            {pending ? t("common.creating") : t("common.create")}
          </Button>
        </FormDialogFooter>
      </form>
    </FormDialog>
  );
}

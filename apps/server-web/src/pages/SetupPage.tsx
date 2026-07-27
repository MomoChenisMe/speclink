import { useState, type FormEvent } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { Button, useI18n } from "@speclink/ui";
import { useClient, useSession } from "../app/context";
import { useAsync } from "../lib/useAsync";
import { readFormError } from "../lib/formError";
import { Field } from "../components/Field";
import type { SetupState } from "../api/client";

// 開箱流程（server-setup「setup 流程完成開箱四要素」, D2／D3／D8 第三階段）。token 由
// URL query 帶入；兩個提交節點分別建立第一位 Admin 與第一組 Project／Repo，最後節點
// 完成後 Server 已建立 session 並回 `/admin?welcome=1`，SPA 站內導向。
export function SetupPage() {
  const { t } = useI18n();
  const client = useClient();
  const [params] = useSearchParams();
  const token = params.get("token") ?? "";
  const { loading, data } = useAsync(() => client.getSetupState(token), [token]);
  // 建立管理員後前進至 registry 步驟：後續狀態覆蓋初載狀態。
  const [advanced, setAdvanced] = useState<SetupState | null>(null);

  const state = advanced ?? data;

  if (!state) {
    if (loading) {
      return (
        <p role="status" aria-live="polite" className="text-muted-foreground">
          {t("common.loading")}
        </p>
      );
    }
    // 無效／過期／已耗用 token：不可區分的固定訊息，不回顯內部原因。
    return (
      <div role="alert">
        <h1 className="mb-2 text-2xl font-semibold">{t("setup.invalidTitle")}</h1>
        <p className="text-muted-foreground">{t("setup.invalidBody")}</p>
      </div>
    );
  }

  return (
    <div>
      <h1 className="mb-6 text-2xl font-semibold">{t("setup.title")}</h1>
      {state.step === "admin" ? (
        <AdminStep token={token} onAdvance={setAdvanced} />
      ) : (
        <RegistryStep token={token} />
      )}
    </div>
  );
}

// 節點一：建立第一位管理員（active 且帶 admin 旗標）。成功前進至 registry 步驟。
function AdminStep({
  token,
  onAdvance,
}: {
  token: string;
  onAdvance: (state: SetupState) => void;
}) {
  const { t } = useI18n();
  const client = useClient();
  const [email, setEmail] = useState("");
  const [display, setDisplay] = useState("");
  const [password, setPassword] = useState("");
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
      const next = await client.submitSetupAdmin(token, { email, display, password });
      onAdvance(next);
    } catch (error) {
      const read = readFormError(error, t("setup.createAdminError"));
      setMessage(read.message);
      setFieldErrors(read.fieldErrors);
      setPending(false);
    }
  }

  return (
    <form onSubmit={onSubmit} className="space-y-4" noValidate>
      <h2 className="text-lg font-medium">{t("setup.step1")}</h2>
      <Field
        id="email"
        label={t("field.email")}
        type="email"
        autoComplete="email"
        value={email}
        onChange={setEmail}
        error={fieldErrors.email}
      />
      <Field id="display" label={t("field.display")} value={display} onChange={setDisplay} error={fieldErrors.display} />
      <Field
        id="password"
        label={t("field.password")}
        type="password"
        autoComplete="new-password"
        value={password}
        onChange={setPassword}
        error={fieldErrors.password}
      />
      {message && Object.keys(fieldErrors).length === 0 && (
        <p role="alert" className="text-sm text-destructive">
          {message}
        </p>
      )}
      <Button type="submit" disabled={pending}>
        {pending ? t("common.creating") : t("setup.createAdmin")}
      </Button>
    </form>
  );
}

// 節點二：建立第一組 Project／Repo，完成 setup。成功後 Server 已建立 admin session，
// SPA 導向回傳的 `/admin?welcome=1`。
function RegistryStep({ token }: { token: string }) {
  const { t } = useI18n();
  const client = useClient();
  const { refresh } = useSession();
  const navigate = useNavigate();
  const [projectKey, setProjectKey] = useState("");
  const [projectName, setProjectName] = useState("");
  const [repoKey, setRepoKey] = useState("");
  const [repoName, setRepoName] = useState("");
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
      const { destination } = await client.submitSetupRegistry(token, {
        projectKey,
        projectName,
        repoKey,
        repoName,
      });
      await refresh();
      navigate(destination);
    } catch (error) {
      const read = readFormError(error, t("setup.createRegistryError"));
      setMessage(read.message);
      setFieldErrors(read.fieldErrors);
      setPending(false);
    }
  }

  return (
    <form onSubmit={onSubmit} className="space-y-4" noValidate>
      <h2 className="text-lg font-medium">{t("setup.step2")}</h2>
      <Field id="project-key" label={t("registry.projectKeyLabel")} value={projectKey} onChange={setProjectKey} error={fieldErrors.projectKey} />
      <Field id="project-name" label={t("registry.projectNameLabel")} value={projectName} onChange={setProjectName} />
      <Field id="repo-key" label={t("registry.repoKeyLabel")} value={repoKey} onChange={setRepoKey} error={fieldErrors.repoKey} />
      <Field id="repo-name" label={t("registry.repoNameLabel")} value={repoName} onChange={setRepoName} />
      {message && Object.keys(fieldErrors).length === 0 && (
        <p role="alert" className="text-sm text-destructive">
          {message}
        </p>
      )}
      <Button type="submit" disabled={pending}>
        {pending ? t("common.creating") : t("setup.createRegistry")}
      </Button>
    </form>
  );
}

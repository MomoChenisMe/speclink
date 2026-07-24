import { useState, type FormEvent } from "react";
import { Button, Input } from "@speclink/ui";
import { useClient } from "../../app/context";
import { useAsync } from "../../lib/useAsync";
import { readFormError } from "../../lib/formError";
import { Field } from "../../components/Field";
import { AdminError, AdminLoading } from "./states";
import type { AdminProject } from "../../api/client";

// 管理專案與儲存庫頁（server-admin, D4／D6）：列出 project／repo、建立新的 project
// 與 repo，並可對既有 project／repo 更名。key 為身分、不可改，更名只動顯示名稱。

export function RegistryPage() {
  const client = useClient();
  const { loading, data, error, reload } = useAsync(() => client.getAdminRegistry(), []);

  return (
    <div className="space-y-8">
      <h1 className="text-2xl font-semibold">專案與儲存庫</h1>
      {loading && <AdminLoading />}
      {error != null && <AdminError onRetry={reload} />}
      {data && (
        <>
          <CreateProjectForm afterMutate={reload} />
          <section className="space-y-6">
            <h2 className="text-lg font-medium">現有專案</h2>
            {data.projects.length === 0 ? (
              <p className="text-sm text-muted-foreground">尚無專案。</p>
            ) : (
              <ul className="space-y-6">
                {data.projects.map((p) => (
                  <ProjectCard key={p.key} project={p} afterMutate={reload} />
                ))}
              </ul>
            )}
          </section>
        </>
      )}
    </div>
  );
}

function CreateProjectForm({ afterMutate }: { afterMutate: () => void }) {
  const client = useClient();
  const [key, setKey] = useState("");
  const [name, setName] = useState("");
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
      await client.adminCreateProject({ key, name: name.trim() || undefined });
      setKey("");
      setName("");
      afterMutate();
    } catch (error) {
      const read = readFormError(error, "建立 project 時發生錯誤");
      setMessage(read.message);
      setFieldErrors(read.fieldErrors);
    } finally {
      setPending(false);
    }
  }

  return (
    <section className="space-y-4">
      <h2 className="text-lg font-medium">建立 project</h2>
      <form onSubmit={onSubmit} className="max-w-md space-y-4" noValidate>
        <Field id="new-project-key" label="Project key" value={key} onChange={setKey} error={fieldErrors.key} />
        <Field
          id="new-project-name"
          label="Project 名稱"
          value={name}
          onChange={setName}
          error={fieldErrors.name}
        />
        {message && Object.keys(fieldErrors).length === 0 && (
          <p role="alert" className="text-sm text-destructive">
            {message}
          </p>
        )}
        <Button type="submit" disabled={pending}>
          {pending ? "建立中…" : "建立 project"}
        </Button>
      </form>
    </section>
  );
}

function ProjectCard({ project, afterMutate }: { project: AdminProject; afterMutate: () => void }) {
  const client = useClient();
  return (
    <li className="space-y-4 rounded-lg border p-4">
      <div className="flex flex-wrap items-center gap-2">
        <code className="font-mono text-sm">{project.key}</code>
        <span className="text-muted-foreground">{project.name}</span>
      </div>
      <RenameForm
        label={`重新命名專案 ${project.name}`}
        current={project.name}
        onRename={(name) => client.adminRenameProject(project.key, name)}
        afterMutate={afterMutate}
      />

      <div className="space-y-2">
        <h3 className="text-sm font-medium">儲存庫</h3>
        {project.repos.length === 0 ? (
          <p className="text-sm text-muted-foreground">尚無儲存庫。</p>
        ) : (
          <ul className="space-y-2">
            {project.repos.map((r) => (
              <li key={r.key} className="flex flex-wrap items-center gap-2">
                <code className="font-mono text-sm">{r.key}</code>
                <span className="text-muted-foreground">{r.name}</span>
                <RenameForm
                  label={`重新命名儲存庫 ${r.name}`}
                  current={r.name}
                  onRename={(name) =>
                    client.adminRenameRepo({ projectKey: project.key, key: r.key, name })
                  }
                  afterMutate={afterMutate}
                />
              </li>
            ))}
          </ul>
        )}
      </div>

      <CreateRepoForm projectKey={project.key} afterMutate={afterMutate} />
    </li>
  );
}

// 更名表單：只改顯示名稱（key 不可改），故此處無 key 欄位。
function RenameForm({
  label,
  current,
  onRename,
  afterMutate,
}: {
  label: string;
  current: string;
  onRename: (name: string) => Promise<void>;
  afterMutate: () => void;
}) {
  const [name, setName] = useState(current);
  const [pending, setPending] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    if (pending || !name.trim()) return;
    setPending(true);
    setMessage(null);
    try {
      await onRename(name.trim());
      afterMutate();
    } catch (error) {
      setMessage(readFormError(error, "更名時發生錯誤").message);
    } finally {
      setPending(false);
    }
  }

  return (
    <form onSubmit={onSubmit} className="flex flex-wrap items-center gap-2">
      <Input
        aria-label={label}
        value={name}
        onChange={(e) => setName(e.target.value)}
        className="h-8 w-48"
      />
      <Button type="submit" variant="outline" size="sm" disabled={pending}>
        更名
      </Button>
      {message && (
        <span role="alert" className="text-sm text-destructive">
          {message}
        </span>
      )}
    </form>
  );
}

function CreateRepoForm({ projectKey, afterMutate }: { projectKey: string; afterMutate: () => void }) {
  const client = useClient();
  const [key, setKey] = useState("");
  const [name, setName] = useState("");
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
      await client.adminCreateRepo({ projectKey, key, name: name.trim() || undefined });
      setKey("");
      setName("");
      afterMutate();
    } catch (error) {
      const read = readFormError(error, "建立 repo 時發生錯誤");
      setMessage(read.message);
      setFieldErrors(read.fieldErrors);
    } finally {
      setPending(false);
    }
  }

  return (
    <form onSubmit={onSubmit} className="space-y-3 border-t pt-3" noValidate>
      <p className="text-sm font-medium">新增儲存庫</p>
      <div className="flex flex-wrap items-end gap-3">
        <Field
          id={`new-repo-key-${projectKey}`}
          label="Repo key"
          value={key}
          onChange={setKey}
          error={fieldErrors.key}
        />
        <Field
          id={`new-repo-name-${projectKey}`}
          label="Repo 名稱"
          value={name}
          onChange={setName}
          error={fieldErrors.name}
        />
        <Button type="submit" variant="outline" size="sm" disabled={pending}>
          建立 repo
        </Button>
      </div>
      {message && Object.keys(fieldErrors).length === 0 && (
        <p role="alert" className="text-sm text-destructive">
          {message}
        </p>
      )}
    </form>
  );
}

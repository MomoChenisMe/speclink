import { useRef, useState, type FormEvent, type ReactNode } from "react";
import { FolderPlus, Inbox } from "lucide-react";
import {
  Badge,
  Button,
  EmptyState,
  Input,
  Label,
  ListCard,
  ListPager,
  ListRow,
  SearchField,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  useI18n,
  usePaging,
  PageHeader,
} from "@speclink/ui";
import { useClient } from "../../app/context";
import { useAsync } from "../../lib/useAsync";
import { useNarrow } from "../../lib/useMediaQuery";
import { readFormError } from "../../lib/formError";
import { Field } from "../../components/Field";
import { DetailSheet } from "../../components/DetailSheet";
import { NoMatchState } from "../../components/NoMatchState";
import { AdminError, AdminLoading } from "./states";
import type { AdminProject } from "../../api/client";

// 管理專案與儲存庫頁（server-web-console「不可變識別欄位唯讀且更名為顯式動作」「管理列表提供
// 搜尋、篩選、分頁與具引導的空狀態」；server-web-list-pages design D2、D3、D5）：頁標題區右側是
// 搜尋與建立專案，其下一張列表卡（前端換頁），整列可點開專案抽屜。代號是身分、建立後不可變更，
// 一律以唯讀文字呈現並標示；名稱預設唯讀，按「更名」才出現輸入框與確認／取消。建立專案與新增
// 儲存庫由抽屜承載，列表頁不含輸入控制項。

export function RegistryPage() {
  const { t } = useI18n();
  const client = useClient();
  const narrow = useNarrow();
  const { loading, data, error, reload } = useAsync(() => client.getAdminRegistry(), []);
  const [selectedKey, setSelectedKey] = useState<string | null>(null);
  const [createOpen, setCreateOpen] = useState(false);
  const [q, setQ] = useState("");
  const scrollRef = useRef<HTMLDivElement>(null);
  const selected = data?.projects.find((p) => p.key === selectedKey) ?? null;
  const needle = q.trim().toLowerCase();
  const visible = (data?.projects ?? []).filter(
    (p) =>
      needle === "" ||
      [p.name, p.key, ...p.repos.flatMap((r) => [r.name, r.key])].some((field) =>
        field.toLowerCase().includes(needle),
      ),
  );
  const paging = usePaging(visible, {
    onMove: () => {
      if (scrollRef.current) scrollRef.current.scrollTop = 0;
    },
  });

  return (
    <div className="flex h-full min-h-0 flex-col gap-4">
      <PageHeader
        level={1}
        title={t("registry.title")}
        description={t("registry.description")}
        actions={
          <>
            <SearchField
              label={t("common.search")}
              placeholder={t("registry.searchPlaceholder")}
              value={q}
              onChange={(value) => {
                setQ(value);
                paging.setPage(1);
              }}
            />
            <Button
              type="button"
              data-tour="list-primary"
              className="gap-1.5"
              onClick={() => setCreateOpen(true)}
            >
              <FolderPlus aria-hidden="true" className="h-4 w-4" />
              {t("registry.create")}
            </Button>
          </>
        }
      />

      {loading && <AdminLoading />}
      {error != null && <AdminError onRetry={reload} />}
      {data && (
        <ListCard scrollRef={scrollRef} footer={<ListPager compact={narrow} {...paging.pager} />}>
          {/* 空狀態不放建立鈕：建立鈕常駐頁首，一頁只有一個主要動作。 */}
          {data.projects.length === 0 ? (
            <EmptyState icon={Inbox} title={t("registry.emptyTitle")} description={t("registry.emptyBody")} />
          ) : visible.length === 0 ? (
            <NoMatchState />
          ) : (
            paging.pageItems.map((p) => (
              <ListRow
                key={p.key}
                data-project={p.key}
                mono={false}
                title={p.name}
                description={t("registry.projectMeta")
                  .replace("{key}", p.key)
                  .replace("{n}", String(p.repos.length))}
                onClick={() => setSelectedKey(p.key)}
              />
            ))
          )}
        </ListCard>
      )}

      {selected && (
        <ProjectSheet project={selected} onClose={() => setSelectedKey(null)} afterMutate={reload} />
      )}

      <CreateProjectSheet open={createOpen} onOpenChange={setCreateOpen} afterMutate={reload} />
    </div>
  );
}

function ProjectSheet({
  project,
  onClose,
  afterMutate,
}: {
  project: AdminProject;
  onClose: () => void;
  afterMutate: () => void;
}) {
  const { t } = useI18n();
  const client = useClient();
  return (
    <DetailSheet
      open
      onOpenChange={(open) => !open && onClose()}
      title={project.name}
      description={t("registry.projectKeyImmutable").replace("{key}", project.key)}
      actions={
        <RenameAction
          label={t("registry.projectName")}
          current={project.name}
          onRename={(name) => client.adminRenameProject(project.key, name)}
          afterMutate={afterMutate}
        />
      }
    >
      <Tabs defaultValue="repos" className="flex min-h-0 flex-col gap-4">
        <TabsList>
          <TabsTrigger value="repos">{t("registry.tabRepos")}</TabsTrigger>
          <TabsTrigger value="members">{t("registry.tabMembers")}</TabsTrigger>
        </TabsList>
        <TabsContent value="repos">
          <RepoList project={project} afterMutate={afterMutate} />
        </TabsContent>
        <TabsContent value="members">
          <ProjectMembers projectKey={project.key} />
        </TabsContent>
      </Tabs>
    </DetailSheet>
  );
}

function RepoList({ project, afterMutate }: { project: AdminProject; afterMutate: () => void }) {
  const { t } = useI18n();
  const client = useClient();
  const [adding, setAdding] = useState(false);

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between gap-2">
        <h3 className="text-sm font-medium">{t("registry.repos")}</h3>
        <Button type="button" variant="outline" size="sm" onClick={() => setAdding(true)}>
          {t("registry.addRepo")}
        </Button>
      </div>
      {project.repos.length === 0 ? (
        <p className="text-sm text-muted-foreground">{t("registry.noRepos")}</p>
      ) : (
        <ul className="space-y-2">
          {project.repos.map((r) => (
            <li key={r.key} className="space-y-2 rounded-md border border-border px-3 py-2">
              <p className="truncate text-sm font-medium">{r.name}</p>
              <div className="flex items-center gap-2">
                <p className="min-w-0 flex-1 truncate text-sm text-muted-foreground">
                  {t("registry.projectKeyImmutable").replace("{key}", r.key)}
                </p>
                <RenameAction
                  label={t("registry.repoName")}
                  buttonLabel={t("registry.renameNamed").replace("{name}", r.name)}
                  current={r.name}
                  onRename={(name) =>
                    client.adminRenameRepo({ projectKey: project.key, key: r.key, name })
                  }
                  afterMutate={afterMutate}
                />
              </div>
            </li>
          ))}
        </ul>
      )}
      {adding && (
        <CreateRepoForm
          projectKey={project.key}
          onDone={() => {
            setAdding(false);
            afterMutate();
          }}
        />
      )}
    </div>
  );
}

// 顯式編輯模式：預設只有一顆「更名」；按下後才出現輸入框與確認／取消。代號沒有這個
// 入口——它建立後就不可變更，因此連 disabled 輸入框都不呈現（避免暗示「某些條件下可改」）。
function RenameAction({
  label,
  buttonLabel,
  current,
  onRename,
  afterMutate,
}: {
  label: string;
  buttonLabel?: string;
  current: string;
  onRename: (name: string) => Promise<void>;
  afterMutate: () => void;
}): ReactNode {
  const { t } = useI18n();
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(current);
  const [pending, setPending] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const id = `rename-${label}-${current}`;

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    if (pending || !name.trim()) return;
    setPending(true);
    setMessage(null);
    try {
      await onRename(name.trim());
      setEditing(false);
      afterMutate();
    } catch (error) {
      setMessage(readFormError(error, t("registry.renameError")).message);
    } finally {
      setPending(false);
    }
  }

  if (!editing) {
    return (
      <Button
        type="button"
        variant="outline"
        size="sm"
        aria-label={buttonLabel}
        onClick={() => {
          setName(current);
          setMessage(null);
          setEditing(true);
        }}
      >
        {t("common.rename")}
      </Button>
    );
  }

  return (
    <form onSubmit={onSubmit} className="flex flex-wrap items-end gap-2">
      <div className="space-y-1.5">
        <Label htmlFor={id}>{label}</Label>
        <Input
          id={id}
          value={name}
          onChange={(e) => setName(e.target.value)}
          className="h-8 w-40"
        />
      </div>
      <Button type="submit" variant="outline" size="sm" disabled={pending}>
        {t("common.confirm")}
      </Button>
      <Button type="button" variant="ghost" size="sm" onClick={() => setEditing(false)}>
        {t("common.cancel")}
      </Button>
      {message && (
        <span role="alert" className="text-sm text-destructive">
          {message}
        </span>
      )}
    </form>
  );
}

function CreateRepoForm({ projectKey, onDone }: { projectKey: string; onDone: () => void }) {
  const { t } = useI18n();
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
      onDone();
    } catch (error) {
      const read = readFormError(error, t("registry.createRepoError"));
      setMessage(read.message);
      setFieldErrors(read.fieldErrors);
    } finally {
      setPending(false);
    }
  }

  return (
    <form onSubmit={onSubmit} className="space-y-3 border-t border-border pt-3" noValidate>
      <p className="text-sm font-medium">{t("registry.newRepo")}</p>
      <Field
        id={`new-repo-key-${projectKey}`}
        label={t("field.repoKey")}
        hint={t("field.keyImmutable")}
        value={key}
        onChange={setKey}
        error={fieldErrors.key}
      />
      <Field
        id={`new-repo-name-${projectKey}`}
        label={t("registry.repoNameLabel")}
        value={name}
        onChange={setName}
        error={fieldErrors.name}
      />
      {message && Object.keys(fieldErrors).length === 0 && (
        <p role="alert" className="text-sm text-destructive">
          {message}
        </p>
      )}
      <Button type="submit" variant="outline" size="sm" disabled={pending}>
        {pending ? t("common.creating") : t("common.create")}
      </Button>
    </form>
  );
}

// 專案成員：從使用者 view model 反查該專案的成員資格。只在抽屜的成員分頁開啟時讀取，
// 列表頁不為此多打一支 API。
function ProjectMembers({ projectKey }: { projectKey: string }) {
  const { t } = useI18n();
  const client = useClient();
  const { loading, data, error, reload } = useAsync(() => client.getAdminUsers(), [projectKey]);
  if (loading) return <AdminLoading />;
  if (error != null) return <AdminError onRetry={reload} />;
  const members =
    data?.users
      .map((u) => ({ user: u, membership: u.memberships.find((m) => m.projectKey === projectKey) }))
      .filter((row) => row.membership !== undefined) ?? [];
  if (members.length === 0) {
    return <p className="text-sm text-muted-foreground">{t("registry.noMembers")}</p>;
  }
  return (
    <ul className="space-y-2 text-sm">
      {members.map(({ user, membership }) => (
        <li key={user.id} className="flex items-center gap-2">
          <span className="min-w-0 flex-1 truncate">{user.email}</span>
          <Badge variant="outline">{membership?.role}</Badge>
        </li>
      ))}
    </ul>
  );
}

function CreateProjectSheet({
  open,
  onOpenChange,
  afterMutate,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  afterMutate: () => void;
}) {
  const { t } = useI18n();
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
      onOpenChange(false);
      afterMutate();
    } catch (error) {
      const read = readFormError(error, t("registry.createProjectError"));
      setMessage(read.message);
      setFieldErrors(read.fieldErrors);
    } finally {
      setPending(false);
    }
  }

  if (!open) return null;

  return (
    <DetailSheet open onOpenChange={onOpenChange} title={t("registry.create")}>
      <form onSubmit={onSubmit} className="space-y-4" noValidate>
        <Field
          id="new-project-key"
          label={t("field.projectKey")}
          hint={t("field.keyImmutable")}
          value={key}
          onChange={setKey}
          error={fieldErrors.key}
        />
        <Field
          id="new-project-name"
          label={t("registry.projectNameLabel")}
          value={name}
          onChange={setName}
          error={fieldErrors.name}
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
            {pending ? t("common.creating") : t("registry.create")}
          </Button>
        </div>
      </form>
    </DetailSheet>
  );
}

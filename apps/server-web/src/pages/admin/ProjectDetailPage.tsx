import { useState, type FormEvent } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { FolderX } from "lucide-react";
import {
  Button,
  Card,
  EmptyState,
  PageHeader,
  SettingsRow,
  StatusPill,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  useI18n,
} from "@speclink/ui";
import { useClient } from "../../app/context";
import { useAsync } from "../../lib/useAsync";
import { userLabel } from "../../lib/userLabel";
import { readFormError } from "../../lib/formError";
import { DetailTitle } from "../../layouts/DetailTitle";
import { CardNote } from "../../components/CardNote";
import { Field } from "../../components/Field";
import { FormDialog, FormDialogFooter } from "../../components/FormDialog";
import { AdminError, AdminLoading } from "./states";
import type { AdminProject } from "../../api/client";

// 專案詳情頁（server-web-console「管理列表以詳情頁與對話框承載檢視與建立」「不可變識別欄位唯讀且
// 更名為顯式動作」；server-web-detail-pages design D6）：以使用者詳情頁為範本——768px 置中欄，頁標題
// 為專案名稱、說明標示代號建立後不可變更、頁級動作為更名；其下卡片標頭分頁為儲存庫與成員。代號是
// 身分，只以唯讀文字呈現（連 disabled 輸入框都不出現，避免暗示「某些條件下可改」）；名稱按「更名」
// 才在對話框裡出現輸入框。成員由使用者清單反查，唯讀。

type RenameTarget = { kind: "project" } | { kind: "repo"; key: string; name: string };

export function ProjectDetailPage({
  projects,
  sequence,
  reload,
}: {
  projects: AdminProject[];
  /** 列表目前呈現的項目（含搜尋）——‹ n / m › 的序列。 */
  sequence: AdminProject[];
  reload: () => void;
}) {
  const { t } = useI18n();
  const navigate = useNavigate();
  const { key } = useParams();
  const project = projects.find((p) => p.key === key);
  if (!project) {
    return (
      <EmptyState
        icon={FolderX}
        title={t("registry.notFoundTitle")}
        description={t("registry.notFoundBody")}
        action={
          <Button type="button" variant="outline" onClick={() => navigate("/admin/registry")}>
            {t("registry.backToList")}
          </Button>
        }
      />
    );
  }
  return <ProjectDetail project={project} sequence={sequence} reload={reload} />;
}

function ProjectDetail({
  project,
  sequence,
  reload,
}: {
  project: AdminProject;
  sequence: AdminProject[];
  reload: () => void;
}) {
  const { t } = useI18n();
  const client = useClient();
  const navigate = useNavigate();
  // 分頁是元件 state：‹ › 換專案時停在同一個分頁。
  const [tab, setTab] = useState("repos");
  const [rename, setRename] = useState<RenameTarget | null>(null);
  const [addingRepo, setAddingRepo] = useState(false);
  const users = useAsync(() => client.getAdminUsers(), [project.key]);
  const members = (users.data?.users ?? []).flatMap((user) => {
    const membership = user.memberships.find((m) => m.projectKey === project.key);
    return membership ? [{ user, role: membership.role }] : [];
  });
  const index = sequence.findIndex((p) => p.key === project.key);
  const keyMeta = (key: string) => t("registry.detailMeta").replace("{key}", key);

  return (
    <div className="mx-auto flex w-full max-w-[768px] flex-col gap-6">
      <DetailTitle
        label={project.name}
        index={index}
        total={sequence.length}
        onStep={(delta) => navigate(`/admin/registry/${sequence[index + delta].key}`, { replace: true })}
      />
      <PageHeader
        level={1}
        title={project.name}
        description={keyMeta(project.key)}
        actions={
          <Button type="button" variant="outline" onClick={() => setRename({ kind: "project" })}>
            {t("common.rename")}
          </Button>
        }
      />

      <Tabs value={tab} onValueChange={setTab} className="flex flex-col">
        <TabsList variant="card" className="overflow-x-auto">
          <TabsTrigger value="repos" count={project.repos.length}>
            {t("registry.tabRepos")}
          </TabsTrigger>
          <TabsTrigger value="members" count={users.data ? members.length : undefined}>
            {t("registry.tabMembers")}
          </TabsTrigger>
        </TabsList>
        <Card className="rounded-t-none border-t-0">
          <TabsContent value="repos" className="divide-y divide-border">
            {project.repos.length === 0 && <CardNote>{t("registry.noRepos")}</CardNote>}
            {project.repos.map((r) => (
              <SettingsRow
                key={r.key}
                label={r.name}
                description={keyMeta(r.key)}
                control={
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    aria-label={t("registry.renameNamed").replace("{name}", r.name)}
                    onClick={() => setRename({ kind: "repo", key: r.key, name: r.name })}
                  >
                    {t("common.rename")}
                  </Button>
                }
              />
            ))}
            <SettingsRow
              label={t("registry.newRepo")}
              description={t("registry.addRepoHint")}
              control={
                <Button type="button" variant="outline" size="sm" onClick={() => setAddingRepo(true)}>
                  {t("registry.newRepo")}
                </Button>
              }
            />
          </TabsContent>

          <TabsContent value="members" className="divide-y divide-border">
            {users.loading || users.error != null ? (
              <div className="px-4 py-3">
                {users.loading ? <AdminLoading /> : <AdminError onRetry={users.reload} />}
              </div>
            ) : members.length === 0 ? (
              <CardNote>{t("registry.noMembers")}</CardNote>
            ) : (
              members.map(({ user, role }) => (
                <SettingsRow
                  key={user.id}
                  label={userLabel(user)}
                  description={user.email}
                  control={
                    <StatusPill tone="neutral" className="font-mono">
                      {role}
                    </StatusPill>
                  }
                />
              ))
            )}
          </TabsContent>
        </Card>
      </Tabs>

      {rename && (
        <RenameDialog
          title={t(rename.kind === "project" ? "registry.renameProjectTitle" : "registry.renameRepoTitle")}
          label={t(rename.kind === "project" ? "registry.projectName" : "registry.repoName")}
          identity={rename.kind === "project" ? project.key : rename.key}
          current={rename.kind === "project" ? project.name : rename.name}
          onRename={(name) =>
            rename.kind === "project"
              ? client.adminRenameProject(project.key, name)
              : client.adminRenameRepo({ projectKey: project.key, key: rename.key, name })
          }
          onClose={() => setRename(null)}
          onRenamed={reload}
        />
      )}
      {addingRepo && (
        <CreateRepoDialog project={project} onClose={() => setAddingRepo(false)} onCreated={reload} />
      )}
    </div>
  );
}

// 更名對話框（專案與儲存庫共用）：預填目前名稱、標示代號不隨更名改變；空白名稱時確認鈕停用。
// 失敗保持開啟並保留輸入；成功關閉，頁標題、麵包屑或列標題隨重載換成新名稱。
function RenameDialog({
  title,
  label,
  identity,
  current,
  onRename,
  onClose,
  onRenamed,
}: {
  title: string;
  label: string;
  /** 不隨更名改變的代號。 */
  identity: string;
  current: string;
  onRename: (name: string) => Promise<void>;
  onClose: () => void;
  onRenamed: () => void;
}) {
  const { t } = useI18n();
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
      onClose();
      onRenamed();
    } catch (error) {
      setMessage(readFormError(error, t("registry.renameError")).message);
    } finally {
      setPending(false);
    }
  }

  return (
    <FormDialog
      open
      onOpenChange={(open) => !open && onClose()}
      title={title}
      description={t("registry.renameBody").replace("{key}", identity)}
    >
      <form onSubmit={onSubmit} className="flex flex-col gap-4" noValidate>
        <Field id="rename-name" label={label} value={name} onChange={setName} error={message ?? undefined} />
        <FormDialogFooter>
          <Button type="button" variant="ghost" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button type="submit" disabled={pending || !name.trim()}>
            {t("common.confirm")}
          </Button>
        </FormDialogFooter>
      </form>
    </FormDialog>
  );
}

function CreateRepoDialog({
  project,
  onClose,
  onCreated,
}: {
  project: AdminProject;
  onClose: () => void;
  onCreated: () => void;
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
      await client.adminCreateRepo({ projectKey: project.key, key, name: name.trim() || undefined });
      onClose();
      onCreated();
    } catch (error) {
      const read = readFormError(error, t("registry.createRepoError"));
      setMessage(read.message);
      setFieldErrors(read.fieldErrors);
    } finally {
      setPending(false);
    }
  }

  return (
    <FormDialog
      open
      onOpenChange={(open) => !open && onClose()}
      title={t("registry.newRepo")}
      description={t("registry.newRepoBody").replace("{name}", () => project.name)}
    >
      <form onSubmit={onSubmit} className="flex flex-col gap-4" noValidate>
        <Field
          id="new-repo-key"
          label={t("field.repoKey")}
          hint={t("field.keyImmutable")}
          value={key}
          onChange={setKey}
          error={fieldErrors.key}
        />
        <Field
          id="new-repo-name"
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

import { useState, type FormEvent } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { FolderX } from "lucide-react";
import { Button, PageHeader, SettingsRow, StatusPill, TabsContent, TabsTrigger, useI18n } from "@speclink/ui";
import { useClient } from "../../app/context";
import { useAsync } from "../../lib/useAsync";
import { userLabel } from "../../lib/userLabel";
import { DetailTitle } from "../../layouts/DetailTitle";
import { CardNote } from "../../components/CardNote";
import { Field } from "../../components/Field";
import { FormDialog, FormDialogActions, FormMessage, useDialogSubmit } from "../../components/FormDialog";
import { DetailNotFound, DetailTabs } from "./detail";
import { AdminCardState } from "./states";
import type { AdminProject } from "../../api/client";

// 專案詳情頁（server-web-console「管理列表以詳情頁與對話框承載檢視與建立」「不可變識別欄位唯讀且
// 更名為顯式動作」；server-web-detail-pages design D6）：以使用者詳情頁為範本——768px 置中欄，頁標題
// 為專案名稱、說明標示代號建立後不可變更、頁級動作為更名；其下卡片標頭分頁為儲存庫與成員。代號是
// 身分，只以唯讀文字呈現（連 disabled 輸入框都不出現，避免暗示「某些條件下可改」）；名稱按「更名」
// 才在對話框裡出現輸入框。成員由使用者清單反查，唯讀。

/** 更名對話框的對象：專案或儲存庫，在按下「更名」時就組好。 */
type RenameTarget = {
  title: string;
  label: string;
  /** 不隨更名改變的代號。 */
  identity: string;
  current: string;
  rename: (name: string) => Promise<void>;
};

/** 專案詳情頁的網址：代號只擋空白，可能含 / ? # 這類字元，要編碼成一段路徑。 */
export function projectPath(key: string): string {
  return `/admin/registry/${encodeURIComponent(key)}`;
}

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
  const { key } = useParams();
  const project = projects.find((p) => p.key === key);
  if (!project) {
    return (
      <DetailNotFound
        icon={FolderX}
        title={t("registry.notFoundTitle")}
        body={t("registry.notFoundBody")}
        backLabel={t("registry.backToList")}
        backTo="/admin/registry"
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
        onStep={(delta) => navigate(projectPath(sequence[index + delta].key), { replace: true })}
      />
      <PageHeader
        level={1}
        title={project.name}
        description={keyMeta(project.key)}
        actions={
          <Button
            type="button"
            variant="outline"
            onClick={() =>
              setRename({
                title: t("registry.renameProjectTitle"),
                label: t("registry.projectName"),
                identity: project.key,
                current: project.name,
                rename: (name) => client.adminRenameProject(project.key, name),
              })
            }
          >
            {t("common.rename")}
          </Button>
        }
      />

      <DetailTabs
        value={tab}
        onValueChange={setTab}
        triggers={
          <>
            <TabsTrigger value="repos" count={project.repos.length}>
              {t("registry.tabRepos")}
            </TabsTrigger>
            <TabsTrigger value="members" count={users.data ? members.length : undefined}>
              {t("registry.tabMembers")}
            </TabsTrigger>
          </>
        }
      >
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
                  onClick={() =>
                    setRename({
                      title: t("registry.renameRepoTitle"),
                      label: t("registry.repoName"),
                      identity: r.key,
                      current: r.name,
                      rename: (name) => client.adminRenameRepo({ projectKey: project.key, key: r.key, name }),
                    })
                  }
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
            <AdminCardState error={users.error} onRetry={users.reload} />
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
      </DetailTabs>

      {rename && <RenameDialog target={rename} onClose={() => setRename(null)} onRenamed={reload} />}
      {addingRepo && (
        <CreateRepoDialog project={project} onClose={() => setAddingRepo(false)} onCreated={reload} />
      )}
    </div>
  );
}

// 更名對話框（專案與儲存庫共用）：預填目前名稱、標示代號不隨更名改變；空白名稱時確認鈕停用。
// 失敗保持開啟並保留輸入；成功關閉，頁標題、麵包屑或列標題隨重載換成新名稱。
function RenameDialog({
  target,
  onClose,
  onRenamed,
}: {
  target: RenameTarget;
  onClose: () => void;
  onRenamed: () => void;
}) {
  const { t } = useI18n();
  const [name, setName] = useState(target.current);
  const { pending, message, submit } = useDialogSubmit(t("registry.renameError"));

  function onSubmit(event: FormEvent) {
    event.preventDefault();
    if (!name.trim()) return;
    void submit(async () => {
      await target.rename(name.trim());
      onClose();
      onRenamed();
    });
  }

  return (
    <FormDialog
      open
      onOpenChange={(open) => !open && onClose()}
      title={target.title}
      description={t("registry.renameBody").replace("{key}", target.identity)}
    >
      <form onSubmit={onSubmit} className="flex flex-col gap-4" noValidate>
        <Field id="rename-name" label={target.label} value={name} onChange={setName} error={message ?? undefined} />
        <FormDialogActions
          submitLabel={t("common.confirm")}
          pending={pending}
          disabled={!name.trim()}
          onCancel={onClose}
        />
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
  const { pending, message, fieldErrors, submit } = useDialogSubmit(t("registry.createRepoError"));

  function onSubmit(event: FormEvent) {
    event.preventDefault();
    void submit(async () => {
      await client.adminCreateRepo({ projectKey: project.key, key, name: name.trim() || undefined });
      onClose();
      onCreated();
    });
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
        <FormMessage message={message} fieldErrors={fieldErrors} />
        <FormDialogActions
          submitLabel={pending ? t("common.creating") : t("common.create")}
          pending={pending}
          onCancel={onClose}
        />
      </form>
    </FormDialog>
  );
}
